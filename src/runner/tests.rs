use crate::error::Error;
use crate::probe::AuthState;
use crate::results::{set_outcome_by_name, Outcome};
use crate::runner::events::{build_event, build_event_ago};
use crate::runner::Runner;
use nostr_types::{
    EventKind, Filter, Id, IdHex, PreEvent, PrivateKey, PublicKeyHex, Signature, Signer, Unixtime,
};
use serde_json::Value;
use std::ops::Add;
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

    pub async fn test_created_at_events(&mut self) -> Result<(), Error> {
        let event = build_event_ago(&self.registered_user, 0, EventKind::TextNote, &[&[]]);
        if let Err(_) = self.probe.post_event_and_verify(&event).await {
            return Err(Error::CannotPost);
        }

        // 1 week ago
        let event = build_event_ago(
            &self.registered_user,
            60 * 24 * 7,
            EventKind::TextNote,
            &[&[]],
        );
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_one_week_old", outcome);

        // 1 month ago
        let event = build_event_ago(
            &self.registered_user,
            60 * 24 * 7 * 4,
            EventKind::TextNote,
            &[&[]],
        );
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_one_month_old", outcome);

        // 1 year ago
        let event = build_event_ago(
            &self.registered_user,
            60 * 24 * 365,
            EventKind::TextNote,
            &[&[]],
        );
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_one_year_old", outcome);

        // 2015, Thursday, January 1, 2015 12:01:01 AM GMT
        let event = build_event(
            &self.registered_user,
            Unixtime(1420070461),
            EventKind::TextNote,
            &[&[]],
        );
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_from_before_nostr", outcome);

        // 1999, Friday, January 1, 1999 12:01:01 AM
        let event = build_event(
            &self.registered_user,
            Unixtime(915148861),
            EventKind::TextNote,
            &[&[]],
        );
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_from_before_2000", outcome);

        // 1970, Thursday, January 1, 1970 12:00:00 AM
        let event = build_event(
            &self.registered_user,
            Unixtime(0),
            EventKind::TextNote,
            &[&[]],
        );
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_from_1970", outcome);

        // 1969 (negative date)
        let (id, raw_event) = Self::create_raw_event(
            "-200",
            "1",
            "[]",
            &textnonce::TextNonce::new(),
            &self.registered_user,
        )
        .await;
        let outcome = match self.probe.post_raw_event(&raw_event, id).await {
            Ok((true, _)) => Outcome::new(true, None),
            Ok((false, reason)) => Outcome::new(false, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_from_before_1970", outcome);

        // 1 year hence
        let created_at = Unixtime::now().add(Duration::new(86400 * 365, 0));
        let event = build_event(
            &self.registered_user,
            created_at,
            EventKind::TextNote,
            &[&[]],
        );
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_one_year_into_the_future", outcome);

        // distant future
        let created_at = Unixtime(i64::MAX);
        let event = build_event(
            &self.registered_user,
            created_at,
            EventKind::TextNote,
            &[&[]],
        );
        let outcome = match self.probe.post_event_and_verify(&event).await {
            Ok(()) => Outcome::new(true, None),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_in_the_distant_future", outcome);

        // created_at greater than signed 32 bit
        let (id, raw_event) = Self::create_raw_event(
            "2147483649", // 2^31 + 1
            "1",
            "[]",
            &textnonce::TextNonce::new(),
            &self.registered_user,
        )
        .await;
        let outcome = match self.probe.post_raw_event(&raw_event, id).await {
            Ok((true, _)) => Outcome::new(true, None),
            Ok((false, reason)) => Outcome::new(false, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name(
            "accepts_events_with_created_at_greater_than_signed32bit",
            outcome,
        );

        // created_at greater than unsigned 32 bit
        let (id, raw_event) = Self::create_raw_event(
            "4294967297", // 2^32 + 1
            "1",
            "[]",
            &textnonce::TextNonce::new(),
            &self.registered_user,
        )
        .await;
        let outcome = match self.probe.post_raw_event(&raw_event, id).await {
            Ok((true, _)) => Outcome::new(true, None),
            Ok((false, reason)) => Outcome::new(false, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name(
            "accepts_events_with_created_at_greater_than_unsigned32bit",
            outcome,
        );

        // created_at greater than unsigned 32 bit
        let (id, raw_event) = Self::create_raw_event(
            "1e+10",
            "1",
            "[]",
            &textnonce::TextNonce::new(),
            &self.registered_user,
        )
        .await;
        let outcome = match self.probe.post_raw_event(&raw_event, id).await {
            Ok((true, _)) => Outcome::new(true, None),
            Ok((false, reason)) => Outcome::new(false, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name(
            "accepts_events_with_created_at_in_scientific_notation",
            outcome,
        );

        Ok(())
    }

    pub async fn test_fetches(&mut self) {
        let ids: Vec<IdHex> = self
            .event_group_a
            .iter()
            .filter(|(_, (_e, r))| *r)
            .map(|(_, (e, _r))| e.id.into())
            .collect();
        if ids.is_empty() {
            set_outcome_by_name(
                "find_by_id",
                Outcome::err(
                    "Could not test because we could not write any events to read back."
                        .to_string(),
                ),
            );
        } else {
            let num = ids.len();
            let filter = {
                let mut filter = Filter::new();
                filter.ids = ids;
                filter
            };
            self.test_fetch_by_filter_group_a(filter, "find_by_id", Some(num))
                .await;
        }

        let filter = {
            let mut filter = Filter::new();
            let pkh1: PublicKeyHex = self.registered_user.public_key().into();
            let pkh2: PublicKeyHex = self.stranger1.public_key().into();
            filter.authors = vec![pkh1, pkh2];
            filter.kinds = vec![EventKind::TextNote, EventKind::ContactList];
            filter
        };
        self.test_fetch_by_filter_group_a(filter, "find_by_pubkey_and_kind", None)
            .await;

        let filter = {
            let mut filter = Filter::new();
            let pkh: PublicKeyHex = self.registered_user.public_key().into();
            filter.add_author(&pkh);
            filter.add_tag_value('p', pkh.to_string());
            filter
        };
        self.test_fetch_by_filter_group_a(filter, "find_by_pubkey_and_tags", None)
            .await;

        let filter = {
            let mut filter = Filter::new();
            filter.kinds = vec![EventKind::TextNote, EventKind::Other(30383), EventKind::ContactList];
            filter.add_tag_value('n', "approved".to_string());
            filter
        };
        self.test_fetch_by_filter_group_a(filter, "find_by_kind_and_tags", None)
            .await;

        let filter = {
            let mut filter = Filter::new();
            filter.add_tag_value('k', "3036".to_string());
            filter
        };
        self.test_fetch_by_filter_group_a(filter, "find_by_tags", None)
            .await;

        let filter = {
            let mut filter = Filter::new();
            let pkh: PublicKeyHex = self.registered_user.public_key().into();
            filter.add_event_kind(EventKind::Other(30383));
            filter.add_author(&pkh);
            filter.add_tag_value('k', "3036".to_string());
            filter.add_tag_value('n', "approved".to_string());
            filter.limit = Some(20);
            filter
        };
        self.test_fetch_by_filter_group_a(filter, "find_by_multiple_tags", None)
            .await;

        let filter = {
            let mut filter = Filter::new();
            let pkh: PublicKeyHex = self.registered_user.public_key().into();
            filter.add_author(&pkh);
            filter
        };
        self.test_fetch_by_filter_group_a(filter, "find_by_pubkey", None)
            .await;

        let filter = Filter::new();
        self.test_fetch_by_filter_group_a(filter, "find_by_scrape", None)
            .await;
    }

    pub async fn test_event_validation(&mut self) {
        // Create event with bad signature
        let pre_event = PreEvent {
            pubkey: self.registered_user.public_key(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![],
            content: "This is a test.".to_owned(),
        };

        let mut event = self.registered_user.sign_event(pre_event.clone()).unwrap();
        event.sig = Signature::zeroes();
        let outcome = match self.probe.post_event(&event).await {
            Ok((true, _)) => Outcome::new(
                false,
                Some("Accepted event with invalid signature".to_owned()),
            ),
            Ok((false, reason)) => Outcome::new(true, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("verifies_signatures", outcome);

        // Create event with bad ID (but good signature of that bad ID)
        let mut event = self.registered_user.sign_event(pre_event.clone()).unwrap();
        event.id = Id::try_from_hex_string(
            "cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe",
        )
        .unwrap();

        event.sig = self.registered_user.sign_id(event.id).unwrap();
        let outcome = match self.probe.post_event(&event).await {
            Ok((true, _)) => Outcome::new(false, Some("Accepted event with invalid id".to_owned())),
            Ok((false, reason)) => Outcome::new(true, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("verifies_id_hashes", outcome);
    }

    pub async fn test_event_json_edgecases(&mut self) {
        // Try including all nip01 escape sequences
        let (id, raw_event) = Self::create_raw_event(
            &format!("{}", Unixtime::now().0),
            "1",
            "[]",
            r##"linebreak\ndoublequote\"backslash\\carraigereturn\rtab\tbackspace\bformfeed\fend"##,
            &self.registered_user,
        )
        .await;
        let outcome = match self.probe.post_raw_event(&raw_event, id).await {
            Ok((true, _)) => Outcome::new(true, None),
            Ok((false, reason)) => Outcome::new(false, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_nip1_json_escape_sequences", outcome);

        // Try including escape sequences not listed in nip01
        let (id, raw_event) = Self::create_raw_event(
            &format!("{}", Unixtime::now().0),
            "1",
            "[]",
            r#"\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007 \u000b \u000e \u000f \u0010\u0011\u0012\u0013\u0014\u0015\u0016 \/"#,
            &self.registered_user,
        )
            .await;
        let outcome = match self.probe.post_raw_event(&raw_event, id).await {
            Ok((true, _)) => Outcome::new(true, None),
            Ok((false, reason)) => Outcome::new(false, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_unlisted_json_escape_sequences", outcome);

        // Try including all nip01 escape sequences as literals instead of escapes
        let (id, raw_event) = Self::create_raw_event(
            &format!("{}", Unixtime::now().0),
            "1",
            "[]",
            "linebreak\ndoublequote\"backslash\\carraigereturn\rtab\tbackspace\x08formfeed\x0cend",
            &self.registered_user,
        )
        .await;
        let outcome = match self.probe.post_raw_event(&raw_event, id).await {
            Ok((true, _)) => Outcome::new(true, None),
            Ok((false, reason)) => Outcome::new(false, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_literals_for_json_escape_sequences", outcome);

        // Try including non-characters such as FDD1 and 1FFFF
        // &[0xef, 0xb7, 0x91, 0xf4, 0x8f, 0xbf, 0xb2];
        // https://www.unicode.org/faq/private_use.html#noncharacters
        let (id, raw_event) = Self::create_raw_event(
            &format!("{}", Unixtime::now().0),
            "1",
            "[]",
            std::str::from_utf8(&[0xef, 0xb7, 0x91, 0xf4, 0x8f, 0xbf, 0xb2]).unwrap(),
            &self.registered_user,
        )
        .await;
        let outcome = match self.probe.post_raw_event(&raw_event, id).await {
            Ok((true, _)) => Outcome::new(true, None),
            Ok((false, reason)) => Outcome::new(false, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_utf8_non_characters", outcome);

        // NOTE we cant to surrogate pairs in rust strings
        // NOTE we cannot send to the relay anything that is not valid UTF-8 because
        // websockets TEXT only takes valid UTF-8.
        //
        // invalid escape:
        //   br#"ab\zc"#
        // invalid escape:
        //   is_err(r#"\𝄞"#.as_bytes());
        // invalid unicode escape
        //   is_err(r#"\u8g00"#.as_bytes());

        // test duplicated json keys
    }

    pub async fn test_misc_events(&mut self) {
        let (id, raw_event) = Self::create_raw_event(
            &format!("{}", Unixtime::now().0),
            "1",
            "[[],[]]",
            "this event has two empty tags",
            &self.registered_user,
        )
        .await;
        let outcome = match self.probe.post_raw_event(&raw_event, id).await {
            Ok((true, _)) => Outcome::new(true, None),
            Ok((false, reason)) => Outcome::new(false, Some(reason)),
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("accepts_events_with_empty_tags", outcome);
    }

    pub async fn test_ephemeral_events(&mut self) {
        // Setup listener to receive
        let mut listener_probe = crate::probe::Probe::new(self.relay_url.clone());
        let mut filter = Filter::new();
        filter.kinds = vec![EventKind::Ephemeral(25000)];
        let pkh: PublicKeyHex = self.registered_user.public_key().into();
        filter.add_author(&pkh);
        let _ = listener_probe.fetch_events(vec![filter.clone()]).await; // this waits for EOSE

        let fail = |r: String| {
            set_outcome_by_name("ephemeral_subscriptions_work", Outcome::new(false, Some(r)));
            set_outcome_by_name(
                "persists_ephemeral_events",
                Outcome::new(
                    false,
                    Some("Untested because ephemeral events dont work".to_string()),
                ),
            );
        };

        let event = build_event_ago(
            &self.registered_user,
            0,
            EventKind::Ephemeral(25000),
            &[&["test"]],
        );
        match self.probe.post_event(&event).await {
            Ok((true, _)) => {}
            Ok((false, reason)) => fail(reason),
            Err(e) => fail(format!("{e}")),
        };

        // Verify it came in on the listener_probe
        let outcome = match listener_probe.fetch_next_event().await {
            Ok(event2) => {
                if event == event2 {
                    Outcome::new(true, None)
                } else {
                    Outcome::new(
                        false,
                        Some("fetched event did not match submitted event".to_string()),
                    )
                }
            }
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("ephemeral_subscriptions_work", outcome);

        let _ = listener_probe.exit();

        // See if it still returns the ephemeral event
        let outcome = match self.probe.fetch_events(vec![filter.clone()]).await {
            Ok(events) => {
                if events.is_empty() {
                    Outcome::new(false, None)
                } else if events[0] == event {
                    Outcome::new(true, None)
                } else {
                    Outcome::new(
                        false,
                        Some("fetched event did not match submitted event".to_string()),
                    )
                }
            }
            Err(e) => Outcome::new(false, Some(format!("{e}"))),
        };
        set_outcome_by_name("persists_ephemeral_events", outcome);
    }

    pub async fn test_event_order(&mut self) {
        // Load all injected events by id
        let ids: Vec<IdHex> = self
            .event_group_a
            .iter()
            .filter(|(_, (_e, r))| *r)
            .map(|(_, (e, _r))| e.id.into())
            .collect();
        let filter = {
            let mut filter = Filter::new();
            filter.ids = ids;
            filter
        };
        let outcome = match self.probe.fetch_events(vec![filter]).await {
            Ok(events) => {
                if events.len() < 3 {
                    Outcome::new(
                        false,
                        Some("Could not fetch enough events to test.".to_owned()),
                    )
                } else {
                    let mut outcome = Outcome::new(true, None);
                    let mut last = Unixtime(i64::MAX);
                    for event in events.iter() {
                        if event.created_at <= last {
                            last = event.created_at;
                        } else {
                            outcome = Outcome::new(false, Some("Order is wrong".to_owned()));
                            break;
                        }
                    }
                    outcome
                }
            }
            Err(Error::Timeout(e)) => Outcome::new(false, Some(format!("{}", e))),
            Err(e) => Outcome::new(false, Some(format!("{}", e))),
        };
        set_outcome_by_name("events_ordered_from_newest_to_oldest", outcome);
    }

    pub async fn test_limit(&mut self) {
        let filter = {
            let mut filter = Filter::new();
            filter.authors = vec![self.registered_user.public_key().into()];
            filter.add_tag_value('t', "a".to_string());
            filter.add_tag_value('t', "b".to_string());
            filter.kinds = vec![EventKind::TextNote, EventKind::Reaction];
            filter.limit = Some(2);
            filter
        };

        let limit_test_first = &self.event_group_a.get("limit_test_first").unwrap().0;
        let limit_test_second = &self.event_group_a.get("limit_test_second").unwrap().0;

        let outcome = match self.probe.fetch_events(vec![filter]).await {
            Ok(events) => {
                if events.len() != 2 {
                    Outcome::new(
                        false,
                        Some(format!("Got {} events, expected 2", events.len())),
                    )
                } else if events[0].id != limit_test_first.id
                    || events[1].id != limit_test_second.id
                {
                    Outcome::new(false, None)
                } else {
                    Outcome::new(true, None)
                }
            }
            Err(Error::Timeout(_)) => Outcome::new(false, Some("Timed out".to_owned())),
            Err(e) => Outcome::new(false, Some(format!("{}", e))),
        };
        set_outcome_by_name("newest_events_when_limited", outcome);
    }

    pub async fn test_replaceables_basic(&mut self) -> Result<(), Error> {
        let metadata_older = &self.event_group_a.get("metadata_older").unwrap().0;
        let metadata_newer = &self.event_group_a.get("metadata_newer").unwrap().0;

        let metadata_events = {
            let filter = {
                let mut filter = Filter::new();
                let pkh: PublicKeyHex = metadata_older.pubkey.into();
                filter.add_author(&pkh);
                filter.add_event_kind(metadata_older.kind);
                filter
            };
            self.probe.fetch_events(vec![filter]).await?
        };
        match metadata_events.len() {
            0 => {
                set_outcome_by_name("accepts_metadata", Outcome::new(false, None));
                set_outcome_by_name(
                    "replaces_metadata",
                    Outcome::new(false, Some("does not accept it".to_owned())),
                );
            }
            1 => {
                set_outcome_by_name("accepts_metadata", Outcome::new(true, None));
                if metadata_events[0].id == metadata_newer.id {
                    set_outcome_by_name("replaces_metadata", Outcome::new(true, None));
                } else {
                    set_outcome_by_name(
                        "replaces_metadata",
                        Outcome::new(
                            false,
                            Some("The newest metadata was not returned".to_owned()),
                        ),
                    );
                }
            }
            _ => {
                set_outcome_by_name("accepts_metadata", Outcome::new(true, None));
                set_outcome_by_name(
                    "replaces_metadata",
                    Outcome::new(
                        false,
                        Some("returns multiple events in replacement group".to_owned()),
                    ),
                );
            }
        };

        // Check of older metadata event still exists under it's ID (this is ok)
        set_outcome_by_name(
            "replaced_events_still_available_by_id",
            Outcome::new(self.probe.check_exists(metadata_older.id).await?, None),
        );

        let contactlist_older = &self.event_group_a.get("contactlist_older").unwrap().0;
        let contactlist_newer = &self.event_group_a.get("contactlist_newer").unwrap().0;

        let contactlist_events = {
            let filter = {
                let mut filter = Filter::new();
                let pkh: PublicKeyHex = contactlist_older.pubkey.into();
                filter.add_author(&pkh);
                filter.add_event_kind(contactlist_older.kind);
                filter
            };
            self.probe.fetch_events(vec![filter]).await?
        };
        match contactlist_events.len() {
            0 => {
                set_outcome_by_name("accepts_contactlist", Outcome::new(false, None));
                set_outcome_by_name(
                    "replaces_contactlist",
                    Outcome::new(false, Some("does not accept it".to_owned())),
                );
            }
            1 => {
                set_outcome_by_name("accepts_contactlist", Outcome::new(true, None));
                if contactlist_events[0].id == contactlist_newer.id {
                    set_outcome_by_name("replaces_contactlist", Outcome::new(true, None));
                } else {
                    set_outcome_by_name(
                        "replaces_contactlist",
                        Outcome::new(
                            false,
                            Some("The newest contactlist was not returned".to_owned()),
                        ),
                    );
                }
            }
            _ => {
                set_outcome_by_name("accepts_contactlist", Outcome::new(true, None));
                set_outcome_by_name(
                    "replaces_contactlist",
                    Outcome::new(
                        false,
                        Some("returns multiple events in replacement group".to_owned()),
                    ),
                );
            }
        };

        Ok(())
    }

    pub async fn test_replaceable_behavior(&mut self) -> Result<(), Error> {

        let (newer_replaceable, ok) = self.event_group_a.get("newer_replaceable").unwrap();
        let filter = {
            let mut filter = Filter::new();
            let pkh: PublicKeyHex = newer_replaceable.pubkey.into();
            filter.add_author(&pkh);
            filter.add_event_kind(newer_replaceable.kind);
            filter
        };
        if !ok {
            set_outcome_by_name(
                "find_replaceable_event",
                Outcome::new(false, Some("Replaceable event was not accepted".to_owned()))
            );
        } else {
            self.test_fetch_by_filter_group_a(
                filter,
                "find_replaceable_event",
                Some(1)
            ).await;
        }

        /*
        // This should have injected ok, but then been replaced
        let (older_replaceable, ok) = self.event_group_a.get("older_replaceable").unwrap();
        let filter = {
            let mut filter = Filter::new();
            let pkh: PublicKeyHex = older_replaceable.pubkey.into();
            filter.add_author(&pkh);
            filter.add_event_kind(older_replaceable.kind);
            filter
        };
        if !ok {
            set_outcome_by_name(
                "find_replaceable_event",
                Outcome::new(false, Some("Replaceable event was not accepted".to_owned()))
            );
        } else {
            self.test_fetch_by_filter_group_a(
                filter,
                "find_replaceable_event"
            ).await;
        }

    (true, "replaceable_event_removes_previous"),
    (true, "replaceable_event_doesnt_remove_future"),
    (true, "parameterized_replaceable_event_removes_previous"),
    (true, "parameterized_replaceable_event_doesnt_remove_future"),

        self.test_fetch_by_filter_group_a(
            filter,
            "find_replaceable_event"
        ).await;


        // GINA
        */

        let (newer_param_replaceable, ok) = self.event_group_a.get("newer_param_replaceable").unwrap();
        let filter = {
            let mut filter = Filter::new();
            let pkh: PublicKeyHex = newer_param_replaceable.pubkey.into();
            filter.add_author(&pkh);
            filter.add_event_kind(newer_param_replaceable.kind);
            filter.add_tag_value('d', "1".to_owned());
            filter
        };
        if !ok {
            set_outcome_by_name(
                "find_parameterized_replaceable_event",
                Outcome::new(false, Some("Parameterized replaceable event was not accepted".to_owned()))
            );
        } else {
            self.test_fetch_by_filter_group_a(
                filter,
                "find_parameterized_replaceable_event",
                Some(1)
            ).await;
        }

        /*
        // This should have injected ok, but then been replaced
        let (_older_param_replaceable, _) = self.event_group_a.get("older_param_replaceable").unwrap();
        // GINA
        */


        /*
        "replaceable_event_removes_previous"
        "replaceable_event_doesnt_remove_future"
        "parameterized_replaceable_event_removes_previous"
        "parameterized_replaceable_event_doesnt_remove_future"
         */

        Ok(())
    }

    //        let ephemeral = self.event_group_a.get("ephemeral").unwrap();
    // TBD: Test ephemeral again with a 2nd probe subscribed to see if it shows up when posted
}
