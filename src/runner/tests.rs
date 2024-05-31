use crate::error::Error;
use crate::probe::{AuthState, Command};
use crate::results::{set_outcome_by_name, Outcome};
use crate::runner::Runner;
use nostr_types::{
    EventKind, Filter, IdHex, PreEvent, PrivateKey, RelayMessage, Signer, SubscriptionId, Tag,
    Unixtime,
};
use serde_json::Value;
use std::ops::{Add, Sub};
use std::time::Duration;

impl Runner {
    pub async fn test_nip11(&mut self) {
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
    pub async fn test_prompts_for_auth_initially(&mut self) {
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

    pub async fn test_supports_eose(&mut self) {
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
            .await;

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

    pub async fn test_public_access(&mut self) {
        let event_id = self
            .probe
            .post(
                Unixtime::now().unwrap(),
                EventKind::TextNote,
                vec![Tag::new(&["test"])],
                "This is a test from a random keypair. Feel free to delete.".to_string(),
                &self.stranger1,
            )
            .await;

        // Wait for an Ok response
        let outcome = match self.probe.wait_for_ok(event_id).await {
            Ok((ok, _reason)) => {
                if ok {
                    Outcome::Pass
                } else {
                    Outcome::Fail
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
                .await;

            // Wait for events
            let outcome = match self.probe.wait_for_events("public_readback").await {
                Ok(events) => {
                    if !events.is_empty() {
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

    pub async fn test_created_at_events(&mut self) -> Result<(), Error> {
        let mut pre_event = PreEvent {
            pubkey: self.registered_user.public_key(),
            created_at: Unixtime::now().unwrap(),
            kind: EventKind::TextNote,
            tags: vec![],
            content: "Testing created_at variations".to_owned(),
        };

        // now (to verify we can post, not recorded as a test outcome)
        let event_id = self
            .probe
            .post_preevent(&pre_event, &self.registered_user)
            .await;
        let (ok, _reason) = match self.probe.wait_for_ok(event_id).await {
            Ok(data) => data,
            Err(_) => return Err(Error::CannotPost),
        };
        if !ok {
            return Err(Error::CannotPost);
        }

        // 1 week ago
        pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(86400 * 7, 0));
        self.post_event_as_registered_user(&pre_event, "accepts_events_one_week_old")
            .await;

        // 1 month ago
        pre_event.created_at = Unixtime::now()
            .unwrap()
            .sub(Duration::new(86400 * 7 * 4, 0));
        self.post_event_as_registered_user(&pre_event, "accepts_events_one_month_old")
            .await;

        // 1 year ago
        pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(86400 * 365, 0));
        self.post_event_as_registered_user(&pre_event, "accepts_events_one_year_old")
            .await;

        // 2015
        pre_event.created_at = Unixtime(1420070461); // Thursday, January 1, 2015 12:01:01 AM GMT
        self.post_event_as_registered_user(&pre_event, "accepts_events_from_before_nostr")
            .await;

        // 1999
        pre_event.created_at = Unixtime(915148861); // Friday, January 1, 1999 12:01:01 AM
        self.post_event_as_registered_user(&pre_event, "accepts_events_from_before_2000")
            .await;

        // 1970
        pre_event.created_at = Unixtime(0); // Thursday, January 1, 1970 12:00:00 AM
        self.post_event_as_registered_user(&pre_event, "accepts_events_from_1970")
            .await;

        // 1969 (negative date)
        // set_outcome_by_name("accepts_events_from_before_1970", outcome);
        // We would have to construct the JSON manually, nostr-types doesn't handle this

        // 1 year hence
        pre_event.created_at = Unixtime::now().unwrap().add(Duration::new(86400 * 365, 0));
        self.post_event_as_registered_user(&pre_event, "accepts_events_one_year_into_the_future")
            .await;

        // distant future
        pre_event.created_at = Unixtime(i64::MAX);
        self.post_event_as_registered_user(&pre_event, "accepts_events_in_the_distant_future")
            .await;

        // gigantic date
        // set_outcome_by_name("accepts_events_with_created_at_larger_than_64bit", outcome);
        // We would have to construct the JSON manually, nostr-types doesn't handle this

        // date with exponential format
        // set_outcome_by_name("accepts_events_with_exponential_created_at_format", outcome);
        // We would have to construct the JSON manually, nostr-types doesn't handle this

        Ok(())
    }

    /*
    pub async fn test_can_auth_as_unknown(&mut self) -> Result<Outcome, Error> {
            Ok(Outcome::Untested)
        }

        pub async fn test_can_auth_as_known(&mut self) -> Result<Outcome, Error> {
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
