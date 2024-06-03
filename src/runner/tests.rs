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
                let outcome = Outcome::new(false, Some(format!("{}", e)));
                set_outcome_by_name("nip11_provided", outcome.clone());
                setall(outcome);
                return;
            }
        };
        set_outcome_by_name("nip11_provided", Outcome::new(true, None));

        setall(Outcome::new(false, Some("Error parsing nip11".to_owned())));
        if let Value::Object(map) = nip11 {
            if let Some(varray) = map.get("supported_nips") {
                setall(Outcome::new(false, Some("not supported".to_owned())));
                if let Value::Array(vec) = varray {
                    for valelem in vec.iter() {
                        if let Value::Number(vnum) = valelem {
                            if let Some(u) = vnum.as_u64() {
                                match u {
                                    4 => set_outcome_by_name(
                                        "claimed_support_for_nip4",
                                        Outcome::new(true, None),
                                    ),
                                    9 => set_outcome_by_name(
                                        "claimed_support_for_nip9",
                                        Outcome::new(true, None),
                                    ),
                                    11 => set_outcome_by_name(
                                        "claimed_support_for_nip11",
                                        Outcome::new(true, None),
                                    ),
                                    26 => set_outcome_by_name(
                                        "claimed_support_for_nip26",
                                        Outcome::new(true, None),
                                    ),
                                    29 => set_outcome_by_name(
                                        "claimed_support_for_nip29",
                                        Outcome::new(true, None),
                                    ),
                                    40 => set_outcome_by_name(
                                        "claimed_support_for_nip40",
                                        Outcome::new(true, None),
                                    ),
                                    42 => set_outcome_by_name(
                                        "claimed_support_for_nip42",
                                        Outcome::new(true, None),
                                    ),
                                    45 => set_outcome_by_name(
                                        "claimed_support_for_nip45",
                                        Outcome::new(true, None),
                                    ),
                                    50 => set_outcome_by_name(
                                        "claimed_support_for_nip50",
                                        Outcome::new(true, None),
                                    ),
                                    65 => set_outcome_by_name(
                                        "claimed_support_for_nip65",
                                        Outcome::new(true, None),
                                    ),
                                    94 => set_outcome_by_name(
                                        "claimed_support_for_nip94",
                                        Outcome::new(true, None),
                                    ),
                                    96 => set_outcome_by_name(
                                        "claimed_support_for_nip96",
                                        Outcome::new(true, None),
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
        let outcome = match self.probe.wait_for_maybe_auth().await {
            Ok(_) => match self.probe.auth_state() {
                AuthState::NotYetRequested => Outcome::new(false, None),
                _ => Outcome::new(true, None),
            },
            Err(e) => Outcome::new(false, Some(format!("{}", e))),
        };

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
                    outcome = Outcome::new(false, None);
                    break;
                }
                Err(e) => {
                    outcome = Outcome::new(false, Some(format!("{}", e)));
                    break;
                }
            };

            match rm {
                RelayMessage::Eose(subid) => {
                    if subid == our_sub_id {
                        outcome = Outcome::new(true, None);
                    } else {
                        outcome = Outcome::new(
                            false,
                            Some("Got EOSE with unrecognized subid".to_string()),
                        );
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
                    Outcome::new(true, None)
                } else {
                    Outcome::new(false, None)
                }
            }
            Err(Error::Timeout(_)) => Outcome::err("No response to an EVENT submission".to_owned()),
            Err(e) => Outcome::new(false, Some(format!("{}", e))),
        };
        set_outcome_by_name("public_can_write", outcome.clone());

        // If it passed, try to read it back
        if matches!(outcome.pass, Some(true)) {
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
                            Outcome::new(true, None)
                        } else {
                            Outcome::new(false, Some("Returned event is wrong".to_owned()))
                        }
                    } else {
                        Outcome::new(
                            false,
                            Some(
                                "Failed to retrieve event we just successfully submitted."
                                    .to_owned(),
                            ),
                        )
                    }
                }
                Err(Error::Timeout(_)) => {
                    Outcome::new(false, Some("No response to an REQ submission".to_owned()))
                }
                Err(e) => Outcome::new(false, Some(format!("{}", e))),
            };
            set_outcome_by_name("public_can_read_back", outcome);
        } else {
            set_outcome_by_name(
                "public_can_read_back",
                Outcome::new(false, Some("n/a".to_owned())),
            );
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
        let (id, raw_event) = Self::create_raw_event(
            "-200",
            "1",
            "[]",
            "Testing created_at variations",
            &self.registered_user,
        )
        .await;
        self.post_raw_event(&raw_event, id, "accepts_events_from_before_1970")
            .await;

        // 1 year hence
        pre_event.created_at = Unixtime::now().unwrap().add(Duration::new(86400 * 365, 0));
        self.post_event_as_registered_user(&pre_event, "accepts_events_one_year_into_the_future")
            .await;

        // distant future
        pre_event.created_at = Unixtime(i64::MAX);
        self.post_event_as_registered_user(&pre_event, "accepts_events_in_the_distant_future")
            .await;

        // created_at greater than signed 32 bit
        let (id, raw_event) = Self::create_raw_event(
            "2147483649", // 2^31 + 1
            "1",
            "[]",
            "Testing created_at variations",
            &self.registered_user,
        )
        .await;
        self.post_raw_event(
            &raw_event,
            id,
            "accepts_events_with_created_at_greater_than_signed32bit",
        )
        .await;

        // created_at greater than unsigned 32 bit
        let (id, raw_event) = Self::create_raw_event(
            "4294967297", // 2^32 + 1
            "1",
            "[]",
            "Testing created_at variations",
            &self.registered_user,
        )
        .await;
        self.post_raw_event(
            &raw_event,
            id,
            "accepts_events_with_created_at_greater_than_unsigned32bit",
        )
        .await;

        // created_at greater than unsigned 32 bit
        let (id, raw_event) = Self::create_raw_event(
            "1e+10",
            "1",
            "[]",
            "Testing created_at variations",
            &self.registered_user,
        )
        .await;
        self.post_raw_event(
            &raw_event,
            id,
            "accepts_events_with_created_at_in_scientific_notation",
        )
        .await;

        Ok(())
    }
}
