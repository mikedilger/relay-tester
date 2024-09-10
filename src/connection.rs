use crate::error::Error;
use crate::globals::{EventParts, Globals, GLOBALS};
use base64::Engine;
use colorful::{Color, Colorful};
use futures_util::{SinkExt, StreamExt};
use http::Uri;
use nostr_types::{ClientMessage, Event, EventKind, Filter, Id, RelayMessage, SubscriptionId, Tag};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tungstenite::Message;

const WAIT_SECONDS: u64 = 3;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AuthState {
    #[default]
    NotYetRequested,
    Challenged(String),
    InProgress(Id),
    Success,
    Failure(String),
}

pub type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug)]
pub struct FetchResult {
    // If fetch_events_keep_open was used, this will be the open SubscriptionId
    pub sub_id: Option<SubscriptionId>,

    // These are events that came before EOSE.
    pub pre_eose_events: Vec<Event>,

    // These are events that came after EOSE. If None, there wasn't an EOSE.
    pub post_eose_events: Option<Vec<Event>>,

    // It relay closed our subscription, this is the message.
    // If this is None, we timed out
    pub close_msg: Option<String>,
}

impl FetchResult {
    pub fn into_events(self) -> Vec<Event> {
        let mut v: Vec<Event> = self.pre_eose_events;
        if let Some(post) = self.post_eose_events {
            v.extend(post);
        }
        v
    }
}

#[derive(Debug)]
pub struct Connection {
    pub relay_url: String,
    pub websocket: Ws,
    pub auth_state: AuthState,
    pub dup_auth: bool,
    pub next_sub_id: AtomicUsize,
}

impl Connection {
    pub async fn new(relay_url: String) -> Result<Connection, Error> {
        let (host, uri) = url_to_host_and_uri(&relay_url);
        let key: [u8; 16] = rand::random();
        let request = http::request::Request::builder()
            .method("GET")
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                base64::engine::general_purpose::STANDARD.encode(key),
            )
            .uri(uri)
            .body(())?;

        let (websocket, _response) = tokio::time::timeout(
            Duration::new(5, 0),
            tokio_tungstenite::connect_async(request),
        )
        .await??;

        Ok(Connection {
            relay_url,
            websocket,
            auth_state: AuthState::NotYetRequested,
            dup_auth: false,
            next_sub_id: AtomicUsize::new(0),
        })
    }

    // Dont call if we are still connected
    pub async fn reconnect(&mut self) -> Result<(), Error> {
        log!("{}", "*** RECONNECTING ***".color(Color::Red));

        // Wait for a few seconds before reconnecting
        tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;

        let (host, uri) = url_to_host_and_uri(&self.relay_url);
        let key: [u8; 16] = rand::random();
        let request = http::request::Request::builder()
            .method("GET")
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                base64::engine::general_purpose::STANDARD.encode(key),
            )
            .uri(uri)
            .body(())?;

        let (websocket, _response) = tokio::time::timeout(
            Duration::new(5, 0),
            tokio_tungstenite::connect_async(request),
        )
        .await??;

        // Sleep a bit for the handshake to finish, else we can end up with
        // "Websocket: WebSocket protocol error: Handshake not finished"
        tokio::time::sleep(Duration::from_millis(250)).await;

        GLOBALS.disconnected.store(false, Ordering::Relaxed);

        self.websocket = websocket;
        self.auth_state = AuthState::NotYetRequested;
        self.dup_auth = false;
        self.next_sub_id = AtomicUsize::new(0);

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Error> {
        let msg = Message::Close(None);
        log!("{} CLOSING", "-->".color(Color::Khaki1));
        let _ = self.inner_send_message(msg).await;
        GLOBALS.disconnected.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn inner_send_message(&mut self, msg: tungstenite::Message) -> Result<(), Error> {
        if GLOBALS.disconnected.load(Ordering::Relaxed) {
            self.reconnect().await?;
        }

        if let Err(e) = self.websocket.send(msg).await {
            GLOBALS.disconnected.store(true, Ordering::Relaxed);
            Err(e)?
        } else {
            Ok(())
        }
    }

    pub async fn send_message(&mut self, message: ClientMessage) -> Result<(), Error> {
        let wire = serde_json::to_string(&message)?;
        let msg = Message::Text(wire);
        let m = format!("{} {msg}", "-->".color(Color::Khaki1));
        self.inner_send_message(msg).await?;
        log!("{}", m);
        Ok(())
    }

    pub async fn wait_for_message(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<RelayMessage>, Error> {
        let mut timeout = tokio::time::interval(timeout);
        timeout.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        timeout.tick().await; // use up the first immediate tick.

        loop {
            tokio::select! {
                _ = timeout.tick() => {
                    return Ok(None);
                },
                message = self.websocket.next() => {
                    let message = match message {
                        Some(m) => m,
                        None => {
                            GLOBALS.disconnected.store(true, Ordering::Relaxed);
                            tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;
                            return Err(Error::Disconnected);
                        }
                    }?;

                    // Self::display(message.clone())?;

                    // Take action
                    match message {
                        Message::Text(s) => {
                            log!("{} {s}", "<--".color(Color::MediumPurple2a));
                            let output: RelayMessage = serde_json::from_str(&s)?;

                            match output {
                                RelayMessage::Auth(challenge) => {
                                    match self.auth_state {
                                        AuthState::NotYetRequested => self.auth_state = AuthState::Challenged(challenge),
                                        _ => self.dup_auth = true,
                                    }

                                    // This wasn't the message being waited for, so keep waiting
                                    continue;
                                },
                                RelayMessage::Ok(id, is_ok, ref reason) => {
                                    GLOBALS.saw_ok_after_event.store(true, Ordering::Relaxed);
                                    if let AuthState::InProgress(sent_id) = self.auth_state {
                                        if id == sent_id {
                                            self.auth_state = if is_ok {
                                                AuthState::Success
                                            } else {
                                                AuthState::Failure(reason.clone())
                                            };

                                            // This wasn't the message being waited for, so keep waiting
                                            continue;
                                        }
                                    }
                                },
                                _ => { }
                            }

                            return Ok(Some(output));
                        },
                        Message::Binary(_) => { },
                        Message::Ping(_) => { },
                        Message::Pong(_) => { },
                        Message::Close(_) => {
                            GLOBALS.disconnected.store(true, Ordering::Relaxed);
                            tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;
                            return Err(Error::Disconnected);
                        },
                        Message::Frame(_) => unreachable!(),
                    }
                },
            }
        }
    }

    pub async fn authenticate_registered_if_challenged(&mut self) -> Result<(), Error> {
        if let AuthState::Challenged(challenge) = &self.auth_state {
            let event = Globals::make_event(
                EventParts::Basic(
                    EventKind::Auth,
                    vec![
                        Tag::new(&["relay", &self.relay_url]),
                        Tag::new(&["challenge", challenge]),
                    ],
                    "".to_string(),
                ),
                true,
            )?;
            self.auth_state = AuthState::InProgress(event.id);
            self.send_message(ClientMessage::Auth(Box::new(event)))
                .await?;
            let _ = self.wait_for_message(Duration::from_secs(1)).await?; // to await response
        }
        Ok(())
    }

    pub async fn authenticate_stranger_if_challenged(&mut self) -> Result<(), Error> {
        if let AuthState::Challenged(challenge) = &self.auth_state {
            let event = Globals::make_event(
                EventParts::Basic(
                    EventKind::Auth,
                    vec![
                        Tag::new(&["relay", &self.relay_url]),
                        Tag::new(&["challenge", challenge]),
                    ],
                    "".to_string(),
                ),
                false,
            )?;
            self.auth_state = AuthState::InProgress(event.id);
            self.send_message(ClientMessage::Auth(Box::new(event)))
                .await?;
            let _ = self.wait_for_message(Duration::from_secs(1)).await?; // to await response
        }
        Ok(())
    }

    pub async fn fetch_events(
        &mut self,
        filters: Vec<Filter>,
        timeout: Duration,
    ) -> Result<FetchResult, Error> {
        self.fetch_events_inner(filters, timeout, true).await
    }

    pub async fn fetch_events_keep_open(
        &mut self,
        filters: Vec<Filter>,
        timeout: Duration,
    ) -> Result<FetchResult, Error> {
        self.fetch_events_inner(filters, timeout, false).await
    }

    async fn fetch_events_inner(
        &mut self,
        filters: Vec<Filter>,
        timeout: Duration,
        close: bool,
    ) -> Result<FetchResult, Error> {
        let sub_id_usize = self.next_sub_id.fetch_add(1, Ordering::Relaxed);
        let sub_id = SubscriptionId(format!("sub{}", sub_id_usize));
        let client_message = ClientMessage::Req(sub_id.clone(), filters);
        self.send_message(client_message).await?;

        let mut pre_eose_events: Vec<Event> = Vec::new();
        let mut post_eose_events: Vec<Event> = Vec::new();
        let mut eose_happened: bool = false;

        loop {
            let opt_message = self.wait_for_message(timeout).await?;
            if opt_message.is_none() {
                // Close the subscription
                if close {
                    self.close_subscription(sub_id.clone()).await?;
                }

                if eose_happened {
                    return Ok(FetchResult {
                        sub_id: if close { None } else { Some(sub_id) },
                        pre_eose_events,
                        post_eose_events: Some(post_eose_events),
                        close_msg: None,
                    });
                } else {
                    return Ok(FetchResult {
                        sub_id: if close { None } else { Some(sub_id) },
                        pre_eose_events,
                        post_eose_events: None,
                        close_msg: None,
                    });
                }
            }
            match opt_message.unwrap() {
                RelayMessage::Event(sub, box_event) => {
                    if sub == sub_id {
                        if eose_happened {
                            post_eose_events.push((*box_event).clone());
                        } else {
                            pre_eose_events.push((*box_event).clone());
                        }
                    }
                }
                RelayMessage::Closed(sub, msg) => {
                    if sub == sub_id {
                        if eose_happened {
                            return Ok(FetchResult {
                                sub_id: if close { None } else { Some(sub_id) },
                                pre_eose_events,
                                post_eose_events: Some(post_eose_events),
                                close_msg: Some(msg),
                            });
                        } else {
                            return Ok(FetchResult {
                                sub_id: if close { None } else { Some(sub_id) },
                                pre_eose_events,
                                post_eose_events: None,
                                close_msg: Some(msg),
                            });
                        }
                    }
                }
                RelayMessage::Eose(sub) => {
                    if sub == sub_id {
                        eose_happened = true;
                    }
                }
                _ => {}
            }
        }
    }

    // This only works if you already submitted (and did not close) a prior subscription.
    pub async fn collect_events(
        &mut self,
        sub_id: SubscriptionId,
        timeout: Duration,
    ) -> Result<Vec<Event>, Error> {
        let mut events: Vec<Event> = Vec::new();
        loop {
            let opt_message = self.wait_for_message(timeout).await?;
            if opt_message.is_none() {
                return Ok(events);
            }
            if let RelayMessage::Event(sub, box_event) = opt_message.unwrap() {
                if sub == sub_id {
                    events.push(*box_event);
                }
            }
        }
    }

    pub async fn close_subscription(&mut self, sub_id: SubscriptionId) -> Result<(), Error> {
        let client_message = ClientMessage::Close(sub_id);
        self.send_message(client_message).await?;
        Ok(())
    }

    pub async fn post_event(
        &mut self,
        event: Event,
        timeout: Duration,
    ) -> Result<(bool, String), Error> {
        let event_id = event.id;
        let message = ClientMessage::Event(Box::new(event));
        self.send_message(message).await?;
        loop {
            match self.wait_for_message(timeout).await? {
                None => return Err(Error::TimedOut),
                Some(RelayMessage::Ok(id, ok, msg)) => {
                    if id != event_id {
                        continue;
                    }
                    GLOBALS.saw_ok_after_event.store(true, Ordering::Relaxed);
                    return Ok((ok, msg));
                }
                Some(_) => continue,
            }
        }
    }

    pub async fn post_raw_event(
        &mut self,
        event_id: Id,
        json: String,
        timeout: Duration,
    ) -> Result<(bool, String), Error> {
        let wire = format!("[\"EVENT\",{}]", json);
        let msg = Message::Text(wire);
        log!("{} {msg}", "-->".color(Color::Khaki1));
        self.inner_send_message(msg).await?;
        loop {
            match self.wait_for_message(timeout).await? {
                None => return Err(Error::TimedOut),
                Some(RelayMessage::Ok(id, ok, msg)) => {
                    if id != event_id {
                        continue;
                    }
                    GLOBALS.saw_ok_after_event.store(true, Ordering::Relaxed);
                    return Ok((ok, msg));
                }
                Some(_) => continue,
            }
        }
    }
}

pub fn url_to_host_and_uri(url: &str) -> (String, Uri) {
    let uri: http::Uri = url.parse::<http::Uri>().expect("Could not parse url");
    let authority = uri.authority().expect("Has no hostname").as_str();
    let host = authority
        .find('@')
        .map(|idx| authority.split_at(idx + 1).1)
        .unwrap_or_else(|| authority);
    if host.is_empty() {
        panic!("URL has empty hostname");
    }
    (host.to_owned(), uri)
}

pub async fn fetch_nip11() -> Result<serde_json::Value, Error> {
    use reqwest::redirect::Policy;
    use reqwest::Client;
    use std::time::Duration;

    let relay_url = GLOBALS.relay_url.read().clone();
    let (host, uri) = url_to_host_and_uri(&relay_url);
    let scheme = match uri.scheme() {
        Some(refscheme) => match refscheme.as_str() {
            "wss" => "https",
            "ws" => "http",
            u => panic!("Unknown scheme {}", u),
        },
        None => panic!("Relay URL has no scheme."),
    };
    let url = format!("{}://{}{}", scheme, host, uri.path());
    let client = Client::builder()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(60))
        .timeout(Duration::from_secs(60))
        .connection_verbose(true)
        .build()?;
    let response = client
        .get(url)
        .header("Host", host)
        .header("Accept", "application/nostr+json")
        .send()
        .await?;
    let json = response.text().await?;
    let value: serde_json::Value = serde_json::from_str(&json)?;
    Ok(value)
}
