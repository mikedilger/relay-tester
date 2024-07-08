mod inner;
use inner::ProbeInner;

use crate::error::Error;
use http::Uri;
use nostr_types::{
    Event, EventKind, Filter, Id, IdHex, PreEvent, RelayMessage, Signer, SubscriptionId, Tag,
    Unixtime,
};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

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
    receiver: Receiver<String>,
    join_handle: Option<JoinHandle<()>>,
    auth_state: AuthState,
    dup_auth: bool,
    next_sub_id: usize,
}

impl Probe {
    pub fn new(relay_url: String) -> Probe {
        let (to_probe, from_main) = tokio::sync::mpsc::channel::<Command>(100);
        let (to_main, from_probe) = tokio::sync::mpsc::channel::<String>(100);
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
            next_sub_id: 0,
        }
    }

    /// Disconnect nicely from the relay
    pub async fn exit(self) -> Result<(), Error> {
        self.sender.send(Command::Exit).await?;
        if let Some(join_handle) = self.join_handle {
            join_handle.await?;
        }

        Ok(())
    }

    /// Disconnect from the relay, wait for `delay`, and then reconnect
    /// This resets our AuthState
    pub async fn reconnect(&mut self, delay: Duration) -> Result<(), Error> {
        self.sender.send(Command::Exit).await?;

        let mut join_handle: Option<JoinHandle<()>> = None;
        std::mem::swap(&mut self.join_handle, &mut join_handle);
        if join_handle.is_some() {
            let join_handle = join_handle.unwrap();
            join_handle.await?;
        }

        tokio::time::sleep(delay).await;

        let (to_probe, from_main) = tokio::sync::mpsc::channel::<Command>(100);
        let (to_main, from_probe) = tokio::sync::mpsc::channel::<String>(100);

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

    /// The AuthState
    pub fn auth_state(&self) -> AuthState {
        self.auth_state.clone()
    }

    /// Wait for a response from the relay
    ///
    /// AUTH or OK (to our AUTH event) are internally handled. Otherwise it waits for
    /// up to 1 second for some other response then times out.
    pub async fn wait_for_a_response(&mut self) -> Result<RelayMessage, Error> {
        loop {
            let timeout = tokio::time::timeout(Duration::new(1, 0), self.receiver.recv());
            match timeout.await {
                Ok(Some(s)) => {
                    let output: RelayMessage = serde_json::from_str(&s)?;
                    match output {
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
                    }
                }
                Ok(None) => return Err(Error::ChannelIsClosed),
                Err(elapsed) => return Err(elapsed.into()),
            }
        }
    }

    /// Wait for 1 second, ignoring everything not AUTH related.
    pub async fn wait_for_maybe_auth(&mut self) -> Result<(), Error> {
        loop {
            match self.wait_for_a_response().await {
                Ok(_) => continue,                      // some message, but not AUTH, keep waiting
                Err(Error::Timeout(_)) => break Ok(()), // nothing is forthcoming
                Err(e) => break Err(e),
            }
        }
    }

    /// Send a command to the inner probe and on to the relay
    pub async fn send(&self, command: Command) {
        self.sender.send(command).await.unwrap()
    }

    /// Post an event
    pub async fn post_event(&mut self, event: &Event) -> Result<(bool, String), Error> {
        self.send(Command::PostEvent(event.to_owned())).await;
        loop {
            let rm = self.wait_for_a_response().await?;
            if let RelayMessage::Ok(ok_id, ok, reason) = rm {
                if ok_id == event.id {
                    return Ok((ok, reason));
                }
            }
        }
    }

    /// Post an event and verify it can be fetched back by id
    pub async fn post_event_and_verify(&mut self, event: &Event) -> Result<(), Error> {
        let (ok, reason) = self.post_event(&event).await?;
        if !ok {
            return Err(Error::EventNotAccepted(reason));
        }

        let filter = {
            let mut filter = Filter::new();
            let idhex: IdHex = event.id.into();
            filter.add_id(&idhex);
            filter.kinds = vec![event.kind];
            filter
        };
        let events = self.fetch_events(vec![filter]).await?;
        if events.len() != 1 {
            return Err(Error::ExpectedOneEvent(events.len()));
        }
        if events[0] != *event {
            return Err(Error::EventMismatch);
        }

        Ok(())
    }

    pub async fn check_exists(&mut self, id: Id) -> Result<bool, Error> {
        let filter = {
            let mut filter = Filter::new();
            let idhex: IdHex = id.into();
            filter.add_id(&idhex);
            filter
        };
        let events = self.fetch_events(vec![filter]).await?;
        if events.len() == 1 && events[0].id == id {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Post a raw event
    pub async fn post_raw_event(
        &mut self,
        event: &str,
        event_id: Id,
    ) -> Result<(bool, String), Error> {
        self.send(Command::PostRawEvent(event.to_owned())).await;
        loop {
            let rm = self.wait_for_a_response().await?;
            if let RelayMessage::Ok(ok_id, ok, reason) = rm {
                if ok_id == event_id {
                    return Ok((ok, reason));
                }
            }
        }
    }

    /// Fetch events matching a set of filters using REQ, waiting
    /// for them to flow in until EOSE or CLOSED or a timeout.
    pub async fn fetch_events(&mut self, filters: Vec<Filter>) -> Result<Vec<Event>, Error> {
        let sub_id = SubscriptionId(format!("sub{}", self.next_sub_id));
        self.next_sub_id += 1;
        self.send(Command::FetchEvents(sub_id.clone(), filters))
            .await;
        let mut events: Vec<Event> = Vec::new();
        loop {
            let rm = self.wait_for_a_response().await?;
            match rm {
                RelayMessage::Event(sub, box_event) => {
                    if sub == sub_id {
                        events.push((*box_event).clone());
                    }
                }
                RelayMessage::Closed(sub, msg) => {
                    if sub == sub_id {
                        return Err(Error::SubClosed(msg));
                    }
                }
                RelayMessage::Eose(sub) => {
                    if sub == sub_id {
                        return Ok(events);
                    }
                }
                _ => {}
            }
        }
    }

    /// Fetch events matching a set of filters using REQ, waiting
    /// for them to flow in until EOSE or CLOSED or a timeout.
    ///
    /// Check that each event matches the filter.
    ///
    /// Check that all given events (which match the filters) are part of the
    /// returned set.
    ///
    /// Returns the events, as well as how many given events were matched.
    pub async fn fetch_events_and_check<'a, I>(
        &mut self,
        filters: Vec<Filter>,
        given: I,
    ) -> Result<(Vec<Event>, usize), Error>
    where
        I: Iterator<Item = &'a Event>,
    {
        let events = self.fetch_events(filters.clone()).await?;

        let matches_filters = |e: &Event| -> bool {
            for filter in filters.iter() {
                if filter.event_matches(e) {
                    return true;
                }
            }
            false
        };

        // Check that these all match the filters
        for event in events.iter() {
            if !matches_filters(event) {
                return Err(Error::EventDoesNotMatchFilters);
            }
        }

        // Check that all given events which match the filters are
        // present in the output
        let mut matches: usize = 0;
        for given_event in given {
            if matches_filters(given_event) {
                if !events.iter().any(|e| e.id == given_event.id) {
                    return Err(Error::ExpectedEventIsMissing);
                }
                matches += 1;
            }
        }

        Ok((events, matches))
    }

    /// Fetch just one more event on a the current subscription.
    /// Also exit on a CLOSED or a timeout.
    pub async fn fetch_next_event(&mut self) -> Result<Event, Error> {
        let sub_id = SubscriptionId(format!("sub{}", self.next_sub_id - 1));
        loop {
            let rm = self.wait_for_a_response().await?;
            match rm {
                RelayMessage::Event(sub, box_event) => {
                    if sub == sub_id {
                        return Ok((*box_event).clone());
                    }
                }
                RelayMessage::Closed(sub, msg) => {
                    if sub == sub_id {
                        return Err(Error::SubClosed(msg));
                    }
                }
                _ => {}
            }
        }
    }

    /// Fetch events, but just check if it CLOSED after EOSE.
    pub async fn fetch_check_close(&mut self, filters: Vec<Filter>) -> Result<bool, Error> {
        let sub_id = SubscriptionId(format!("sub{}", self.next_sub_id));
        self.next_sub_id += 1;
        self.send(Command::FetchEvents(sub_id.clone(), filters))
            .await;
        let mut eose: bool = false;
        loop {
            let rm = self.wait_for_a_response().await?;
            match rm {
                RelayMessage::Closed(sub, _msg) => {
                    if sub == sub_id {
                        if eose {
                            return Ok(true);
                        }
                    }
                }
                RelayMessage::Eose(sub) => {
                    if sub == sub_id {
                        eose = true;
                    }
                }
                _ => {}
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

    // PRIVATE
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

    pub async fn fetch_nip11(&self) -> Result<serde_json::Value, Error> {
        use reqwest::redirect::Policy;
        use reqwest::Client;
        use std::time::Duration;

        let (host, uri) = crate::probe::url_to_host_and_uri(&self.relay_url);
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
}
