use crate::error::Error;
use crate::probe::AuthState;
use crate::results::{set_outcome_by_name, Outcome};
use crate::runner::events::build_event_ago;
use crate::runner::Runner;
use nostr_types::{EventKind, Filter, Id, PrivateKey, Unixtime};
use serde_json::Value;

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

        let nip11 = match self.probe.fetch_nip11().await {
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

    pub async fn test_eose(&mut self) {
        // A very benign filter.
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

        let outcome = match self.probe.fetch_events(vec![filter]).await {
            Ok(_) => Outcome::new(true, None),
            Err(Error::Timeout(_)) => Outcome::new(false, None),
            Err(e) => Outcome::new(false, Some(format!("{}", e))),
        };
        set_outcome_by_name("supports_eose", outcome);

        // A filter to fetch a single event by id (a complete subscription)
        let filter = {
            let mut filter = Filter::new();
            let id = Id::try_from_hex_string(
                "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            )
            .unwrap();
            filter.ids = vec![id.into()];
            filter.kinds = vec![EventKind::TextNote];
            filter
        };

        let outcome = match self.probe.fetch_check_close(vec![filter]).await {
            Ok(_) => Outcome::new(true, None),
            Err(Error::Timeout(_)) => Outcome::new(false, None),
            Err(e) => Outcome::new(false, Some(format!("{}", e))),
        };
        set_outcome_by_name("closes_complete_subscriptions_after_eose", outcome);

        // Fetch some events of a single author (an incomplete subscription)
        let filter = {
            let mut filter = Filter::new();
            // Use a random author that should have 0 events
            let private_key = PrivateKey::generate();
            let public_key = private_key.public_key();
            filter.add_author(&public_key.into());
            filter.add_event_kind(EventKind::TextNote);
            filter.limit = Some(10);
            filter.until = Some(Unixtime(1_700_000_000)); // some time in the past
            filter
        };
        let outcome = match self.probe.fetch_check_close(vec![filter]).await {
            Ok(_) => Outcome::new(false, None),
            Err(Error::Timeout(_)) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{}", e))),
        };
        set_outcome_by_name("keeps_open_incomplete_subscriptions_after_eose", outcome);
    }

    pub async fn test_ok(&mut self) {
        let event = build_event_ago(&self.stranger1, 0, EventKind::TextNote, &[&["test"]]);
        let outcome = match self.probe.post_event(&event).await {
            Ok((_ok, _reason)) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{}", e))),
        };
        set_outcome_by_name("sends_ok_after_event", outcome);
    }

    pub async fn test_public_access(&mut self) {
        let event = build_event_ago(&self.stranger1, 0, EventKind::TextNote, &[&["test"]]);
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("public_can_write", outcome);
    }

    pub async fn test_public_relay_lists(&mut self) {
        let event = build_event_ago(&self.stranger1, 0, EventKind::RelayList, &[&["test"]]);
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_relay_lists_from_public", outcome);
    }

    pub async fn test_public_dm_relay_lists(&mut self) {
        let event = build_event_ago(&self.stranger1, 0, EventKind::DmRelayList, &[&["test"]]);
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_dm_relay_lists_from_public", outcome);
    }

    pub async fn test_public_ephemeral_events(&mut self) {
        let event = build_event_ago(&self.stranger1, 0, EventKind::WalletResponse, &[&["test"]]);
        let outcome = match self.probe.post_event(&event).await {
            Ok((ok, reason)) => {
                if ok {
                    Outcome::new(true, None)
                } else {
                    Outcome::new(false, Some(reason))
                }
            }
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_ephemeral_events_from_public", outcome);
    }
}
