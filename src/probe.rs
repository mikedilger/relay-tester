use crate::error::Error;
use crate::PREFIXES;
use base64::Engine;
use colorful::{Color, Colorful};
use futures_util::stream::FusedStream;
use futures_util::{SinkExt, StreamExt};
use http::Uri;
use nostr_types::{
    ClientMessage, Event, EventKind, Filter, Id, PreEvent, RelayMessage, Signer, SubscriptionId,
    Tag, Unixtime,
};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tungstenite::Message;

/// These are things we can ask the relay probe to do.
/// Mostly they become messages to the relay.
#[derive(Debug)]
pub enum Command {
    Auth(Event),
    PostEvent(Event),
    PostRawEvent(String),
    FetchEvents(SubscriptionId, Vec<Filter>),
    Exit,
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

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug, Clone, Default)]
pub enum AuthState {
    #[default]
    NotYetRequested,
    Challenged(String),
    InProgress(Id),
    Success,
    Failure(String),
}

#[derive(Debug)]
pub struct Probe {
    pub relay_url: String,
    sender: Sender<Command>,
    receiver: Receiver<RelayMessage>,
    join_handle: Option<JoinHandle<()>>,
    auth_state: AuthState,
    dup_auth: bool,
}

impl Probe {
    pub fn new(relay_url: String) -> Probe {
        let (to_probe, from_main) = tokio::sync::mpsc::channel::<Command>(100);
        let (to_main, from_probe) = tokio::sync::mpsc::channel::<RelayMessage>(100);
        let relay_url_thread = relay_url.clone();
        let join_handle = tokio::spawn(async move {
            let mut probe = ProbeInner {
                input: from_main,
                output: to_main,
            };
            if let Err(e) = probe.connect_and_listen(&relay_url_thread).await {
                eprintln!("{}", e);
            }
        });

        Probe {
            relay_url,
            sender: to_probe,
            receiver: from_probe,
            join_handle: Some(join_handle),
            auth_state: AuthState::NotYetRequested,
            dup_auth: false,
        }
    }

    pub fn auth_state(&self) -> AuthState {
        self.auth_state.clone()
    }

    pub async fn send(&self, command: Command) {
        self.sender.send(command).await.unwrap()
    }

    pub async fn post(
        &self,
        created_at: Unixtime,
        kind: EventKind,
        tags: Vec<Tag>,
        content: String,
        signer: &dyn Signer,
    ) -> Id {
        let pre_event = PreEvent {
            pubkey: signer.public_key(),
            created_at,
            kind,
            tags,
            content,
        };
        self.post_preevent(&pre_event, signer).await
    }

    pub async fn post_preevent(&self, pre_event: &PreEvent, signer: &dyn Signer) -> Id {
        assert_eq!(pre_event.pubkey, signer.public_key());
        let event = signer.sign_event(pre_event.clone()).unwrap();
        let event_id = event.id;
        self.send(Command::PostEvent(event)).await;
        event_id
    }

    pub async fn post_event(&self, event: &Event) {
        self.send(Command::PostEvent(event.to_owned())).await;
    }

    pub async fn post_raw_event(&self, event: &str) {
        self.send(Command::PostRawEvent(event.to_owned())).await;
    }

    pub async fn wait_for_maybe_auth(&mut self) -> Result<(), Error> {
        loop {
            match self.wait_for_a_response().await {
                Ok(_) => continue,                      // some message, but not AUTH, keep waiting
                Err(Error::Timeout(_)) => break Ok(()), // nothing is forthcoming
                Err(e) => break Err(e),
            }
        }
    }

    pub async fn wait_for_ok(&mut self, id: Id) -> Result<(bool, String), Error> {
        loop {
            let rm = self.wait_for_a_response().await?;
            if let RelayMessage::Ok(ok_id, ok, reason) = rm {
                if ok_id == id {
                    return Ok((ok, reason));
                }
            }
        }
    }

    pub async fn wait_for_events(&mut self, subscription: &str) -> Result<Vec<Event>, Error> {
        let mut events: Vec<Event> = Vec::new();
        loop {
            let rm = self.wait_for_a_response().await?;
            match rm {
                RelayMessage::Event(sub, box_event) => {
                    if *sub == subscription {
                        events.push((*box_event).clone());
                    }
                }
                RelayMessage::Closed(sub, msg) => {
                    if *sub == subscription {
                        return Err(Error::SubClosed(msg));
                    }
                }
                RelayMessage::Eose(sub) => {
                    if *sub == subscription {
                        return Ok(events);
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn wait_for_a_response(&mut self) -> Result<RelayMessage, Error> {
        // If one was pushed back, give them that
        loop {
            let timeout = tokio::time::timeout(Duration::new(1, 0), self.receiver.recv());
            match timeout.await {
                Ok(Some(output)) => match output {
                    RelayMessage::Ok(_, _, _) => {
                        if let Some(rm) = self.process_ok(output).await? {
                            // It wasn't our auth response, hand it to the caller
                            return Ok(rm);
                        } else {
                            // it was an AUTH response. Listen for the next response.
                            continue;
                        }
                    }
                    RelayMessage::Auth(challenge) => {
                        match self.auth_state {
                            AuthState::NotYetRequested => {
                                self.auth_state = AuthState::Challenged(challenge);
                            }
                            _ => {
                                self.dup_auth = true;
                            }
                        }

                        // It was an AUTH request. Listen for the next response.
                        continue;
                    }
                    other => return Ok(other),
                },
                Ok(None) => return Err(Error::ChannelIsClosed),
                Err(elapsed) => return Err(elapsed.into()),
            }
        }
    }

    /// This authenticates with a challenge that the relay previously presented,
    /// if in that state.
    pub async fn authenticate(&mut self, signer: &dyn Signer) -> Result<(), Error> {
        self.wait_for_maybe_auth().await?;

        if let AuthState::Challenged(ref challenge) = self.auth_state {
            let pre_event = PreEvent {
                pubkey: signer.public_key(),
                created_at: Unixtime::now().unwrap(),
                kind: EventKind::Auth,
                tags: vec![
                    Tag::new(&["relay", &self.relay_url]),
                    Tag::new(&["challenge", challenge]),
                ],
                content: "".to_string(),
            };

            let event = signer.sign_event(pre_event)?;

            self.auth_state = AuthState::InProgress(event.id);
            self.sender.send(Command::Auth(event)).await.unwrap();
            Ok(())
        } else {
            Err(Error::NotChallenged)
        }
    }

    // internally processes Ok messages prior to returning them, just in case
    // they are related to the authentication process
    async fn process_ok(&mut self, rm: RelayMessage) -> Result<Option<RelayMessage>, Error> {
        match rm {
            RelayMessage::Ok(id, is_ok, ref reason) => {
                if let AuthState::InProgress(sent_id) = self.auth_state {
                    if id == sent_id {
                        self.auth_state = if is_ok {
                            AuthState::Success
                        } else {
                            AuthState::Failure(reason.clone())
                        };
                        Ok(None)
                    } else {
                        // Was an OK about some other event (not the auth event)
                        Ok(Some(rm))
                    }
                } else {
                    // Was an OK about some other event (we haven't sent auth)
                    Ok(Some(rm))
                }
            }
            _ => Err(Error::General(
                "process_ok() called with the wrong kind of RelayMessage".to_owned(),
            )),
        }
    }

    pub async fn exit(self) -> Result<(), Error> {
        self.sender.send(Command::Exit).await.unwrap();
        if let Some(join_handle) = self.join_handle {
            join_handle.await?;
        }

        Ok(())
    }

    pub async fn reconnect(&mut self, delay: Duration) -> Result<(), Error> {
        self.sender.send(Command::Exit).await.unwrap();

        let mut join_handle: Option<JoinHandle<()>> = None;
        std::mem::swap(&mut self.join_handle, &mut join_handle);
        if join_handle.is_some() {
            let join_handle = join_handle.unwrap();
            join_handle.await?;
        }

        tokio::time::sleep(delay).await;

        let (to_probe, from_main) = tokio::sync::mpsc::channel::<Command>(100);
        let (to_main, from_probe) = tokio::sync::mpsc::channel::<RelayMessage>(100);

        let relay_url_thread = self.relay_url.clone();
        let new_join_handle = tokio::spawn(async move {
            let mut probe = ProbeInner {
                input: from_main,
                output: to_main,
            };
            if let Err(e) = probe.connect_and_listen(&relay_url_thread).await {
                eprintln!("{}", e);
            }
        });

        self.sender = to_probe;
        self.receiver = from_probe;
        self.join_handle = Some(new_join_handle);
        self.auth_state = AuthState::NotYetRequested;
        self.dup_auth = false;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ProbeInner {
    input: Receiver<Command>,
    output: Sender<RelayMessage>,
}

impl ProbeInner {
    async fn connect_and_listen(&mut self, relay_url: &str) -> Result<(), Error> {
        let (host, uri) = url_to_host_and_uri(relay_url);

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

        let (mut websocket, _response) = tokio::time::timeout(
            Duration::new(5, 0),
            tokio_tungstenite::connect_async(request),
        )
        .await??;

        let mut ping_timer = tokio::time::interval(Duration::new(15, 0));
        ping_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        ping_timer.tick().await; // use up the first immediate tick.

        loop {
            tokio::select! {
                _ = ping_timer.tick() => {
                    let msg = Message::Ping(vec![0x1]);
                    self.send(&mut websocket, msg).await;
                },
                local_message = self.input.recv() => {
                    match local_message {
                        Some(Command::PostEvent(event)) => {
                            let client_message = ClientMessage::Event(Box::new(event));
                            let wire = serde_json::to_string(&client_message)?;
                            let msg = Message::Text(wire);
                            self.send(&mut websocket, msg).await;
                        },
                        Some(Command::PostRawEvent(event)) => {
                            let wire = format!("[\"EVENT\",{}]", event);
                            let msg = Message::Text(wire);
                            self.send(&mut websocket, msg).await;
                        },
                        Some(Command::Auth(event)) => {
                            let client_message = ClientMessage::Auth(Box::new(event));
                            let wire = serde_json::to_string(&client_message)?;
                            let msg = Message::Text(wire);
                            self.send(&mut websocket, msg).await;
                        },
                        Some(Command::FetchEvents(subid, filters)) => {
                            let client_message = ClientMessage::Req(subid, filters);
                            let wire = serde_json::to_string(&client_message)?;
                            let msg = Message::Text(wire);
                            self.send(&mut websocket, msg).await;
                        },
                        Some(Command::Exit) => {
                            self.send(&mut websocket, Message::Close(None)).await;
                        },
                        None => { }
                    }
                },
                message = websocket.next() => {
                    let message = match message {
                        Some(m) => m,
                        None => {
                            if websocket.is_terminated() {
                                eprintln!("{}", "Connection terminated".color(Color::Orange1));
                            }
                            break;
                        }
                    }?;

                    // Display it
                    Self::display(message.clone())?;

                    // Take action
                    match message {
                        Message::Text(s) => {
                            // Send back to main
                            let relay_message: RelayMessage = serde_json::from_str(&s)?;
                            self.output.send(relay_message).await.unwrap();
                        },
                        Message::Binary(_) => { },
                        Message::Ping(_) => { },
                        Message::Pong(_) => { },
                        Message::Close(_) => break,
                        Message::Frame(_) => unreachable!(),
                    }
                },
            }
        }

        Ok(())
    }

    fn display(message: Message) -> Result<(), Error> {
        match message {
            Message::Text(s) => {
                let relay_message: RelayMessage = serde_json::from_str(&s)?;
                match relay_message {
                    RelayMessage::Auth(challenge) => {
                        eprintln!("{}: AUTH({})", PREFIXES.from_relay, challenge);
                    }
                    RelayMessage::Event(sub, e) => {
                        let event_json = serde_json::to_string(&e)?;
                        eprintln!(
                            "{}: EVENT({}, {})",
                            PREFIXES.from_relay,
                            sub.as_str(),
                            event_json
                        );
                    }
                    RelayMessage::Closed(sub, msg) => {
                        eprintln!("{}: CLOSED({}, {})", PREFIXES.from_relay, sub.as_str(), msg);
                    }
                    RelayMessage::Notice(s) => {
                        eprintln!("{}: NOTICE({})", PREFIXES.from_relay, s);
                    }
                    RelayMessage::Eose(sub) => {
                        eprintln!("{}: EOSE({})", PREFIXES.from_relay, sub.as_str());
                    }
                    RelayMessage::Ok(id, ok, reason) => {
                        eprintln!(
                            "{}: OK({}, {}, {})",
                            PREFIXES.from_relay,
                            id.as_hex_string(),
                            ok,
                            reason
                        );
                    }
                }
            }
            Message::Binary(_) => {
                eprintln!("{}: Binary message received!!!", PREFIXES.from_relay);
            }
            Message::Ping(_) => {
                eprintln!("{}: Ping", PREFIXES.from_relay);
            }
            Message::Pong(_) => {
                eprintln!("{}: Pong", PREFIXES.from_relay);
            }
            Message::Close(_) => {
                eprintln!("{}", "Remote closed nicely.".color(Color::Green));
            }
            Message::Frame(_) => {
                unreachable!()
            }
        }

        Ok(())
    }

    async fn send(&mut self, websocket: &mut Ws, message: Message) {
        match message {
            Message::Text(ref s) => eprintln!("{}: Text({})", PREFIXES.sending, s),
            Message::Binary(_) => eprintln!("{}: Binary(_)", PREFIXES.sending),
            Message::Ping(_) => eprintln!("{}: Ping(_)", PREFIXES.sending),
            Message::Pong(_) => eprintln!("{}: Pong(_)", PREFIXES.sending),
            Message::Close(_) => eprintln!("{}: Close(_)", PREFIXES.sending),
            Message::Frame(_) => eprintln!("{}: Frame(_)", PREFIXES.sending),
        }
        websocket.send(message).await.unwrap()
    }
}
