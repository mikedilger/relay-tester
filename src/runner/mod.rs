use crate::error::Error;
use crate::probe::Probe;
use crate::results::{set_outcome_by_name, Outcome};
use colorful::{Color, Colorful};
use nostr_types::{Event, Filter, Id, KeySigner, PrivateKey, Signer};
use secp256k1::hashes::Hash;
use std::collections::HashMap;
use std::time::Duration;

mod events;

mod pre_auth_tests;
mod registered_tests;
mod stranger_tests;

pub struct Runner {
    relay_url: String,
    probe: Probe,
    stranger1: KeySigner,
    registered_user: KeySigner,

    /// the boolean indicates whether we will expect it to be on the relay
    /// (it is false for events we expect to replace, and it gets set false for events
    /// that the relay gives us ok=false for). We then use it when we test REQ filters
    /// to make sure all events that we expect to be returned actually are returned.
    event_group_a: HashMap<&'static str, (Event, bool)>,
}

impl Runner {
    pub fn new(relay_url: String, private_key: PrivateKey) -> Runner {
        let probe = Probe::new(relay_url.clone());

        let stranger1 = {
            let private_key = PrivateKey::generate();
            KeySigner::from_private_key(private_key, "", 8).unwrap()
        };

        let registered_user = KeySigner::from_private_key(private_key, "", 8).unwrap();

        let mut event_group_a: HashMap<&'static str, (Event, bool)> = HashMap::new();
        for data in events::GROUP_A.iter() {
            let event =
                events::build_event_ago(&registered_user, data.minutes_ago, data.kind, data.tags);
            event_group_a.insert(data.name, (event, data.can_read_back));
        }

        Runner {
            relay_url,
            probe,
            stranger1,
            registered_user,
            event_group_a,
        }
    }

    pub async fn run(&mut self) {
        let mut errors: Vec<Error> = Vec::new();

        // Phase 1:  Pre-Auth
        if let Err(e) = self.run_preauth_tests().await {
            errors.push(e);
        }

        // Authenticate as the registered user
        if let Err(e) = self.probe.authenticate(&self.registered_user).await {
            errors.push(e);
        }

        // Phase 2: Authenticated
        if let Err(e) = self.run_registered_tests().await {
            errors.push(e);
        }

        // Disconnect and authenticate as stranger
        if let Err(e) = self.probe.reconnect(Duration::new(1, 0)).await {
            errors.push(e);
        }

        if let Err(e) = self.probe.authenticate(&self.stranger1).await {
            errors.push(e);
        }

        // Phase 3: Authenticated as an unrecognized user
        if let Err(e) = self.run_stranger_tests().await {
            errors.push(e);
        }

        if !errors.is_empty() {
            eprintln!("\n\nERRORS ---------------------------------");
            for err in &errors {
                eprintln!("{}", err);
            }
            eprintln!("\n----------------------------------------");
        }
    }

    // Tests that run before authenticating
    async fn run_preauth_tests(&mut self) -> Result<(), Error> {
        // nip11_provided
        // claimed_support_for_nip4
        // claimed_support_for_nip9
        // claimed_support_for_nip11
        // claimed_support_for_nip26
        // claimed_support_for_nip29
        // claimed_support_for_nip40
        // claimed_support_for_nip42
        // claimed_support_for_nip45
        // claimed_support_for_nip50
        // claimed_support_for_nip59
        // claimed_support_for_nip65
        // claimed_support_for_nip94
        // claimed_support_for_nip96
        {
            eprintln!("\n{} -----", "TESTING NIP-11".color(Color::LightBlue));
            self.test_nip11().await;
        }

        // prompts_for_auth_initially
        {
            eprintln!(
                "\n{} -----",
                "TESTING INITIAL AUTH PROMPT".color(Color::LightBlue)
            );
            self.test_prompts_for_auth_initially().await;
        }

        // supports_eose
        // closes_complete_subscriptions_after_eose
        // keeps_open_incomplete_subscriptions_after_eose
        {
            eprintln!("\n{} -----", "TESTING EOSE".color(Color::LightBlue));
            self.test_eose().await;
        }

        // sends_ok_after_event
        {
            eprintln!("\n{} -----", "TESTING OK".color(Color::LightBlue));
            self.test_ok().await;
        }

        eprintln!(
            "\n{} ----- ",
            "TESTING PUBLIC ACCESS".color(Color::LightBlue)
        );

        // public_can_write
        self.test_public_access().await;

        // accepts_relay_lists_from_public
        self.test_public_relay_lists().await;

        // accepts_dm_relay_lists_from_public
        self.test_public_dm_relay_lists().await;

        // accepts_ephemeral_events_from_public
        self.test_public_ephemeral_events().await;

        Ok(())
    }

    // Tests that run as the registered user
    async fn run_registered_tests(&mut self) -> Result<(), Error> {
        eprintln!(
            "\n{} ----- ",
            "INJECTING EVENT GROUP A".color(Color::LightBlue)
        );

        for (_name, refval) in self.event_group_a.iter_mut() {
            let (ok, _why) = self.probe.post_event(&refval.0).await?;

            // Remember which events stored properly, so we can check against that set
            if !ok {
                // Remember that we cannot read this one back, because the relay did not
                // accept it
                refval.1 = false;
            }
        }

        // verifies_signatures
        // verifies_id_hashes
        {
            eprintln!(
                "\n{} ----- ",
                "TESTING EVENT VALIDATION".color(Color::LightBlue)
            );
            self.test_event_validation().await;
        }

        // accepts_nip1_json_escape_sequences
        // accepts_unlisted_json_escape_sequences
        // accepts_literals_for_json_escape_sequences
        // accepts_utf8_non_characters
        {
            eprintln!(
                "\n{} ----- ",
                "TESTING JSON EDGE CASES".color(Color::LightBlue)
            );
            self.test_event_json_edgecases().await;
        }

        // accepts_events_one_week_okd
        // accepts_events_one_month_old
        // accepts_events_one_year_old
        // accepts_events_from_before_nostr
        // accepts_events_from_before_2000
        // accepts_events_from_1970
        // accepts_events_from_before_1970
        // accepts_events_one_year_into_the_future
        // accepts_events_in_the_distant_future
        // accepts_events_with_created_at_greater_than_signed32bit
        // accepts_events_with_created_at_greater_than_unsigned32bit
        // accepts_events_with_created_at_in_scientific_notation
        {
            eprintln!(
                "\n{} ----- ",
                "TESTING CREATED_AT VARIATIONS".color(Color::LightBlue)
            );
            self.test_created_at_events().await?;
        }

        // accepts_events_with_empty_tags
        {
            eprintln!("\n{} ----- ", "TESTING MISC EVENTS".color(Color::LightBlue));
            self.test_misc_events().await;
        }

        // ephemeral_subscriptions_work
        // persists_ephemeral_events
        {
            eprintln!(
                "\n{} ----- ",
                "TESTING EPHEMERAL EVENTS".color(Color::LightBlue)
            );
            self.test_ephemeral_events().await;
        }

        // events_ordered_from_newest_to_oldest
        {
            eprintln!("\n{} ----- ", "TESTING EVENT ORDER".color(Color::LightBlue));
            self.test_event_order().await;
        }

        // newest_events_when_limited
        {
            eprintln!("\n{} -----", "TESTING LIMIT".color(Color::LightBlue));
            self.test_limit().await;
        }

        // find_by_id
        // find_by_pubkey_and_kind
        // find_by_pubkey_and_tags
        // find_by_kind_and_tags
        // find_by_tags
        // find_by_multiple_tags
        // find_by_pubkey
        // find_by_scrape
        {
            // FIXME to use injected events
            eprintln!("\n{} ----- ", "TESTING FETCHES".color(Color::LightBlue));
            self.test_fetches().await;
        }

        // accepts_metadata
        // replaces_metadata
        // replaced_events_still_available_by_id
        // accepts_contactlist
        // replaces_contactlist
        // TBD:  replaceable_event_removes_previous
        // TBD:  replaceable_event_doesnt_remove_future
        // TBD:  parameterized_replaceable_event_removes_previous
        // TBD:  parameterized_replaceable_event_doesnt_remove_future
        {
            eprintln!(
                "\n{} ----- ",
                "TESTING REPLACEABLES".color(Color::LightBlue)
            );
            self.test_replaceables_basic().await?;
        }

        // test_replacable_deletes
        // TBD: ?

        // test_deletes
        // TBD: ?

        Ok(())
    }

    // Tests that run as a stranger
    async fn run_stranger_tests(&mut self) -> Result<(), Error> {
        // find_replaceable_event
        // find_parameterized_replaceable_event

        {
            eprintln!(
                "\n{} ----- ",
                "TESTING REPLACEABLE FINDS".color(Color::LightBlue)
            );
            self.test_replaceable_behavior().await?;
        }

        // test_ephemeral
        // TBD: ?

        Ok(())
    }

    pub async fn exit(self) -> Result<(), Error> {
        self.probe.exit().await?;
        Ok(())
    }

    async fn create_raw_event(
        created_at: &str,
        kind: &str,
        tags: &str,
        content: &str,
        signer: &dyn Signer,
    ) -> (Id, String) {
        let serial_for_sig = format!(
            "[0,\"{}\",{},{},{},\"{}\"]",
            signer.public_key().as_hex_string(),
            created_at,
            kind,
            tags,
            content
        );
        let hash = secp256k1::hashes::sha256::Hash::hash(serial_for_sig.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();
        let id = Id(id);
        let signature = signer.sign_id(id).unwrap();

        let raw_event = format!(
            r##"{{"id":"{}","pubkey":"{}","created_at":{},"kind":{},"tags":{},"content":"{}","sig":"{}"}}"##,
            id.as_hex_string(),
            signer.public_key().as_hex_string(),
            created_at,
            kind,
            tags,
            content,
            signature.as_hex_string()
        );

        (id, raw_event)
    }

    async fn test_fetch_by_filter_group_a(
        &mut self,
        filter: Filter,
        outcome_name: &'static str,
        num_matches_expected: Option<usize>,
    ) {
        let given = self
            .event_group_a
            .iter()
            .filter(|(_, (_e, r))| *r)
            .map(|(_, (e, _r))| e);

        let (_events, matches) = match self
            .probe
            .fetch_events_and_check(vec![filter.clone()], given)
            .await
        {
            Ok(pair) => pair,
            Err(e) => {
                set_outcome_by_name(outcome_name, Outcome::new(false, Some(format!("{e}"))));
                return;
            }
        };

        if let Some(expected) = num_matches_expected {
            if matches == expected {
                set_outcome_by_name(outcome_name, Outcome::new(true, None));
            } else {
                set_outcome_by_name(
                    outcome_name,
                    Outcome::new(
                        false,
                        Some(format!("matched {} events, expected {}", matches, expected)),
                    ),
                );
            }
        } else {
            set_outcome_by_name(
                outcome_name,
                Outcome::new(true, Some(format!("matched {} events", matches))),
            );
        }
    }
}
