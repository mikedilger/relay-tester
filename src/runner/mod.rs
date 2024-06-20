use crate::error::Error;
use crate::probe::Probe;
use crate::results::{set_outcome_by_name, Outcome};
use colorful::{Color, Colorful};
use nostr_types::{Event, Filter, Id, KeySigner, PrivateKey, Signer};
use secp256k1::hashes::Hash;
use std::collections::HashMap;
use std::time::Duration;

mod events;
mod tests;

pub struct Runner {
    probe: Probe,
    stranger1: KeySigner,
    registered_user: KeySigner,
    event_group_a: HashMap<&'static str, Event>,
}

impl Runner {
    pub fn new(relay_url: String, private_key: PrivateKey) -> Runner {
        let probe = Probe::new(relay_url);

        let stranger1 = {
            let private_key = PrivateKey::generate();
            KeySigner::from_private_key(private_key, "", 8).unwrap()
        };

        let registered_user = KeySigner::from_private_key(private_key, "", 8).unwrap();

        let event_group_a = events::build_event_group_a(&registered_user);

        Runner {
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
        eprintln!("\n{} -----", "TESTING NIP-11".color(Color::LightBlue));
        self.test_nip11().await;

        eprintln!(
            "\n{} -----",
            "TESTING INITIAL AUTH PROMPT".color(Color::LightBlue)
        );
        self.test_prompts_for_auth_initially().await;

        eprintln!("\n{} -----", "TESTING EOSE".color(Color::LightBlue));
        self.test_eose().await;

        eprintln!("\n{} -----", "TESTING OK".color(Color::LightBlue));
        self.test_ok().await;

        eprintln!(
            "\n{} ----- ",
            "TESTING PUBLIC ACCESS".color(Color::LightBlue)
        );
        self.test_public_access().await;

        Ok(())
    }

    // Tests that run as the registered user
    async fn run_registered_tests(&mut self) -> Result<(), Error> {
        eprintln!(
            "\n{} ----- ",
            "INJECTING EVENT GROUP A".color(Color::LightBlue)
        );

        for (_name, refevent) in &self.event_group_a {
            let _ = self.probe.post_event(refevent).await?;
            // Ignore relay response; some relays may reject older replaced events
            // with ok=false. We will notice that the event is not there.
        }

        // Test event validation
        eprintln!(
            "\n{} ----- ",
            "TESTING EVENT VALIDATION".color(Color::LightBlue)
        );
        self.test_event_validation().await;

        // Test JSON edge cases
        eprintln!(
            "\n{} ----- ",
            "TESTING JSON EDGE CASES".color(Color::LightBlue)
        );
        self.test_event_json_edgecases().await;

        // Test created_at
        eprintln!(
            "\n{} ----- ",
            "TESTING CREATED_AT VARIATIONS".color(Color::LightBlue)
        );
        self.test_created_at_events().await?;

        // Test misc events
        eprintln!("\n{} ----- ", "TESTING MISC EVENTS".color(Color::LightBlue));
        self.test_misc_events().await;

        // Test REQ event order
        self.test_event_order().await;

        // Test LIMIT
        eprintln!("\n{} -----", "TESTING LIMIT".color(Color::LightBlue));
        self.test_limit().await;

        // Test fetches (FIXME to use injected events)
        eprintln!("\n{} ----- ", "TESTING FETCHES".color(Color::LightBlue));
        self.test_fetches().await;

        // Test replaceables
        eprintln!(
            "\n{} ----- ",
            "TESTING REPLACEABLES".color(Color::LightBlue)
        );
        self.test_replaceables_basic().await?;

        // test_ephemeral

        // test_replacable_deletes

        // test_deletes

        Ok(())
    }

    // Tests that run as a stranger
    async fn run_stranger_tests(&mut self) -> Result<(), Error> {
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

    async fn test_fetch_by_filter(&mut self, filter: Filter, outcome_name: &'static str) {
        let events = match self.probe.fetch_events(vec![filter.clone()]).await {
            Ok(events) => events,
            Err(e) => {
                set_outcome_by_name(outcome_name, Outcome::new(false, Some(format!("{e}"))));
                return;
            }
        };

        // Make sure all the events that are returned match the filter
        for event in events.iter() {
            if !filter.event_matches(event) {
                set_outcome_by_name(
                    outcome_name,
                    Outcome::new(
                        false,
                        Some("Event returned doesn't match filter".to_owned()),
                    ),
                );
                return;
            }
        }

        // Make sure all the injected events that match were returned
        let mut matches = 0;
        for (name, event) in &self.event_group_a {
            if filter.event_matches(event) {
                matches += 1;
                if !events.iter().any(|e| e.id == event.id) {
                    set_outcome_by_name(
                        outcome_name,
                        Outcome::new(
                            false,
                            Some(format!("Failed to return matching event label={}", name)),
                        ),
                    );
                }
            }
        }

        set_outcome_by_name(
            outcome_name,
            Outcome::new(true, Some(format!("matched {} events", matches))),
        );
    }
}
