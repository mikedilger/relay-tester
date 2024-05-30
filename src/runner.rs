use crate::error::Error;
use crate::probe::{AuthState, Command, Probe};
use crate::results::{set_outcome_by_name, Outcome};
use nostr_types::{
    EventKind, Filter, IdHex, KeySigner, PreEvent, PrivateKey, RelayMessage, Signer,
    SubscriptionId, Tag, Unixtime,
};
use serde_json::Value;
use std::time::Duration;

pub struct Runner {
    probe: Probe,
    stranger1: KeySigner,
    //stranger2: KeySigner,
    registered_user: KeySigner,
}

impl Runner {
    pub fn new(relay_url: String, private_key: PrivateKey) -> Runner {
        let registered_user = KeySigner::from_private_key(private_key, "", 8).unwrap();

        let stranger1 = {
            let private_key = PrivateKey::generate();
            KeySigner::from_private_key(private_key, "", 8).unwrap()
        };

        /*let stranger2 = {
            let private_key = PrivateKey::generate();
            KeySigner::from_private_key(private_key, "", 8).unwrap()
        };*/

        let probe = Probe::new(relay_url);

        Runner {
            probe,
            registered_user,
            stranger1,
            //stranger2,
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        // Tests that run before authenticating
        self.test_nip11().await;
        self.test_prompts_for_auth_initially().await;
        self.test_supports_eose().await;
        self.test_public_access().await;

        // Authenticate as a stranger
        if self.probe.authenticate(&self.stranger1).await.is_err() {
            eprintln!("Cannot authenticate. Cannot continue testing.");
            return Ok(());
        }

        // Tests that run as a stranger
        // TBD

        // Authenticate as the configured subscribed user
        self.probe.reconnect(Duration::new(1, 0)).await?;
        let _ = self.probe.wait_for_a_response().await;
        if self
            .probe
            .authenticate(&self.registered_user)
            .await
            .is_err()
        {
            eprintln!("Cannot authenticate. Cannot continue testing.");
            return Ok(());
        }

        // Tests that run as the registered user
        // TBD

        Ok(())
    }

    pub async fn exit(self) -> Result<(), Error> {
        self.probe.exit().await?;
        Ok(())
    }

    async fn test_nip11(&mut self) {
        let setall = |outcome: Outcome| {
            set_outcome_by_name("claimed_support_for_nip4", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip9", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip11", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip26", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip29", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip40", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip42", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip45", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip50", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip59", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip65", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip94", outcome.clone());
            set_outcome_by_name("claimed_support_for_nip96", outcome.clone());
        };

        let nip11 = match self.fetch_nip11().await {
            Ok(nip11) => nip11,
            Err(e) => {
                let outcome = Outcome::Fail2(format!("{}", e));
                set_outcome_by_name("nip11_provided", outcome.clone());
                setall(outcome);
                return;
            }
        };
        set_outcome_by_name("nip11_provided", Outcome::Pass);

        setall(Outcome::Fail2("Error parsing nip11".to_owned()));
        if let Value::Object(map) = nip11 {
            if let Some(varray) = map.get("supported_nips") {
                setall(Outcome::Info("not supported".to_owned()));
                if let Value::Array(vec) = varray {
                    for valelem in vec.iter() {
                        if let Value::Number(vnum) = valelem {
                            if let Some(u) = vnum.as_u64() {
                                match u {
                                    4 => set_outcome_by_name(
                                        "claimed_support_for_nip4",
                                        Outcome::Pass,
                                    ),
                                    9 => set_outcome_by_name(
                                        "claimed_support_for_nip9",
                                        Outcome::Pass,
                                    ),
                                    11 => set_outcome_by_name(
                                        "claimed_support_for_nip11",
                                        Outcome::Pass,
                                    ),
                                    26 => set_outcome_by_name(
                                        "claimed_support_for_nip26",
                                        Outcome::Pass,
                                    ),
                                    29 => set_outcome_by_name(
                                        "claimed_support_for_nip29",
                                        Outcome::Pass,
                                    ),
                                    40 => set_outcome_by_name(
                                        "claimed_support_for_nip40",
                                        Outcome::Pass,
                                    ),
                                    42 => set_outcome_by_name(
                                        "claimed_support_for_nip42",
                                        Outcome::Pass,
                                    ),
                                    45 => set_outcome_by_name(
                                        "claimed_support_for_nip45",
                                        Outcome::Pass,
                                    ),
                                    50 => set_outcome_by_name(
                                        "claimed_support_for_nip50",
                                        Outcome::Pass,
                                    ),
                                    65 => set_outcome_by_name(
                                        "claimed_support_for_nip65",
                                        Outcome::Pass,
                                    ),
                                    94 => set_outcome_by_name(
                                        "claimed_support_for_nip94",
                                        Outcome::Pass,
                                    ),
                                    96 => set_outcome_by_name(
                                        "claimed_support_for_nip96",
                                        Outcome::Pass,
                                    ),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn fetch_nip11(&mut self) -> Result<serde_json::Value, Error> {
        use reqwest::redirect::Policy;
        use reqwest::Client;
        use std::time::Duration;

        let (host, uri) = crate::probe::url_to_host_and_uri(&self.probe.relay_url);
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

    async fn test_prompts_for_auth_initially(&mut self) {
        let outcome;
        loop {
            match self.probe.wait_for_a_response().await {
                Ok(_) => {
                    // AUTH would have been captured by probe, so this is some
                    // message other than AUTH that we didn't expect.
                    //
                    // Ignore it.
                    continue;
                }
                Err(Error::Timeout(_)) => {
                    // Expected timeout.
                    outcome = match self.probe.auth_state() {
                        AuthState::NotYetRequested => Outcome::Fail,
                        _ => Outcome::Pass,
                    };
                    break;
                }
                Err(e) => {
                    outcome = Outcome::Fail2(format!("{}", e));
                    break;
                }
            }
        }

        set_outcome_by_name("prompts_for_auth_initially", outcome);
    }

    async fn test_supports_eose(&mut self) {
        // A very benign filter.
        let our_sub_id = SubscriptionId("fetch_by_filter".to_string());
        let filter = {
            let mut filter = Filter::new();
            // Use a random author that should have 0 events
            let private_key = PrivateKey::generate();
            let public_key = private_key.public_key();
            filter.add_author(&public_key.into());
            filter.add_event_kind(EventKind::TextNote);
            filter.limit = Some(10);
            filter
        };

        self.probe
            .send(Command::FetchEvents(our_sub_id.clone(), vec![filter]))
            .await
            .unwrap();

        let outcome;
        loop {
            let rm = match self.probe.wait_for_a_response().await {
                Ok(rm) => rm,
                Err(Error::Timeout(_)) => {
                    outcome = Outcome::Fail;
                    break;
                }
                Err(e) => {
                    outcome = Outcome::Fail2(format!("{}", e));
                    break;
                }
            };

            match rm {
                RelayMessage::Eose(subid) => {
                    if subid == our_sub_id {
                        outcome = Outcome::Pass;
                    } else {
                        outcome = Outcome::Fail2("Got EOSE with unrecognized subid".to_string());
                    }
                    break;
                }
                _ => {
                    // We didn't expect that
                    continue;
                }
            }
        }

        set_outcome_by_name("supports_eose", outcome);
    }

    async fn test_public_access(&mut self) {
        let pre_event = PreEvent {
            pubkey: self.stranger1.public_key(),
            created_at: Unixtime::now().unwrap(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["test"])],
            content: "This is a test from a random keypair. Feel free to delete.".to_string(),
        };

        let event_id = self.probe.post(pre_event, &self.stranger1).await.unwrap();

        // Wait for an Ok response
        let outcome = match self.probe.wait_for_ok().await {
            Ok((id, ok, _reason)) => {
                if id == event_id {
                    if ok {
                        Outcome::Pass
                    } else {
                        Outcome::Fail
                    }
                } else {
                    Outcome::Fail2("Responded to EVENT with OK with a different id".to_owned())
                }
            }
            Err(Error::Timeout(_)) => {
                Outcome::Fail2("No response to an EVENT submission".to_owned())
            }
            Err(e) => Outcome::Fail2(format!("{}", e)),
        };
        set_outcome_by_name("public_can_write", outcome.clone());

        // If it passed, try to read it back
        if matches!(outcome, Outcome::Pass) {
            let idhex: IdHex = event_id.into();
            let our_sub_id = SubscriptionId("public_readback".to_string());
            let mut filter = Filter::new();
            filter.add_id(&idhex);
            filter.add_event_kind(EventKind::TextNote);
            self.probe
                .send(Command::FetchEvents(our_sub_id.clone(), vec![filter]))
                .await
                .unwrap();

            // Wait for events
            let outcome = match self.probe.wait_for_events("public_readback").await {
                Ok(events) => {
                    if events.len() > 0 {
                        if events[0].id == event_id {
                            Outcome::Pass
                        } else {
                            Outcome::Fail2("Returned event is wrong".to_owned())
                        }
                    } else {
                        Outcome::Fail2(
                            "Failed to retrieve event we just successfully submitted.".to_owned(),
                        )
                    }
                }
                Err(Error::Timeout(_)) => {
                    Outcome::Fail2("No response to an REQ submission".to_owned())
                }
                Err(e) => Outcome::Fail2(format!("{}", e)),
            };
            set_outcome_by_name("public_can_read_back", outcome);
        } else {
            set_outcome_by_name("public_can_read_back", Outcome::Info("n/a".to_owned()));
        }
    }

    /*
    async fn test_can_auth_as_unknown(&mut self) -> Result<Outcome, Error> {
            Ok(Outcome::Untested)
        }

        async fn test_can_auth_as_known(&mut self) -> Result<Outcome, Error> {
            // Listen for any final messages first
            loop {
                match self.probe.wait_for_a_response().await {
                    Ok(_) => {
                        // We didn't expect that.
                        continue;
                    }
                    Err(Error::Timeout(_)) => {
                        // expected,
                        break;
                    }
                    Err(e) => {
                        // FIXME: This should be recorded in results instead.
                        return Err(e);
                    }
                }
            }

            Ok(match self.probe.auth_state() {
                AuthState::NotYetRequested => Outcome::Fail,
                AuthState::Challenged(_) => {
                    Outcome::Fail2("Challenged but we failed to AUTH back".to_string())
                }
                AuthState::InProgress(_) => Outcome::Fail2("Did not OK the AUTH".to_string()),
                AuthState::Success => Outcome::Pass,
                AuthState::Failure(s) => Outcome::Fail2(s),
                AuthState::Duplicate => Outcome::Fail2("AUTHed multiple times".to_string()),
            })
    }
        */
}
