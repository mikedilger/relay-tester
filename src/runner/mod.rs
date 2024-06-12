use crate::error::Error;
use crate::probe::Probe;
use crate::results::{set_outcome_by_name, Outcome};
use colorful::{Color, Colorful};
use nostr_types::{Event, Filter, Id, IdHex, KeySigner, PrivateKey, Signer};
use secp256k1::hashes::Hash;
use std::time::Duration;

mod inject;
mod tests;

pub struct Runner {
    probe: Probe,
    stranger1: KeySigner,
    registered_user: KeySigner,
    injected: Vec<Event>,
}

impl Runner {
    pub fn new(relay_url: String, private_key: PrivateKey) -> Runner {
        let probe = Probe::new(relay_url);

        let stranger1 = {
            let private_key = PrivateKey::generate();
            KeySigner::from_private_key(private_key, "", 8).unwrap()
        };

        let registered_user = KeySigner::from_private_key(private_key, "", 8).unwrap();

        let injected = inject::injected_events(&registered_user);

        Runner {
            probe,
            stranger1,
            registered_user,
            injected,
        }
    }

    pub async fn run(&mut self) {
        self.run_preauth_tests().await;

        // Authenticate as the registered user
        if self
            .probe
            .authenticate(&self.registered_user)
            .await
            .is_err()
        {
            eprintln!("Cannot authenticate. Cannot continue testing.");
            return;
        }

        self.run_registered_tests().await;

        // Disconnect and authenticate as stranger
        if self.probe.reconnect(Duration::new(1, 0)).await.is_err() {
            eprintln!("Cannot disconnect/reconnect. Cannot continue testing.");
            return;
        }

        if self.probe.authenticate(&self.stranger1).await.is_err() {
            eprintln!("Cannot authenticate. Cannot continue testing.");
            return;
        }

        self.run_stranger_tests().await;
    }

    // Tests that run before authenticating
    async fn run_preauth_tests(&mut self) {
        eprintln!("\n{} -----", "TESTING NIP-11".color(Color::LightBlue));
        self.test_nip11().await;

        eprintln!(
            "\n{} -----",
            "TESTING INITIAL AUTH PROMPT".color(Color::LightBlue)
        );
        self.test_prompts_for_auth_initially().await;

        eprintln!("\n{} -----", "TESTING EOSE".color(Color::LightBlue));
        self.test_eose().await;

        eprintln!(
            "\n{} ----- ",
            "TESTING PUBLIC ACCESS".color(Color::LightBlue)
        );
        self.test_public_access().await;
    }

    // Tests that run as the registered user
    async fn run_registered_tests(&mut self) {
        eprintln!(
            "\n{} ----- ",
            "INJECTING EVENTS TO FETCH LATER".color(Color::LightBlue)
        );
        // Inject events
        let injected = self.injected.clone();
        for event in injected {
            if let Err(e) = self.post_event_and_verify(&event).await {
                eprintln!("Cannot inject benign events: {e}\nCannot continue testing.");
                return;
            }
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
        if let Err(e) = self.test_created_at_events().await {
            eprintln!("{}", e);
        }

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
        self.test_fetches().await
    }

    // Tests that run as a stranger
    async fn run_stranger_tests(&mut self) {
    }

    pub async fn exit(self) -> Result<(), Error> {
        self.probe.exit().await?;
        Ok(())
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

    async fn post_event_and_verify(&mut self, event: &Event) -> Result<(), Error> {
        let (ok, reason) = self.probe.post_event(&event).await?;
        if !ok {
            return Err(Error::EventNotAccepted(reason));
        }

        let filter = {
            let mut filter = Filter::new();
            let idhex: IdHex = event.id.into();
            filter.add_id(&idhex);
            filter
        };
        let events = self.probe.fetch_events(vec![filter]).await?;
        if events.len() != 1 {
            return Err(Error::ExpectedOneEvent(events.len()));
        }
        if events[0] != *event {
            return Err(Error::EventMismatch);
        }

        Ok(())
    }

    async fn test_fetch_by_filter(
        &mut self,
        filter: Filter,
        expected_count: Option<usize>,
        outcome_name: &'static str,
    ) {
        let events = match self.probe.fetch_events(vec![filter.clone()]).await {
            Ok(events) => events,
            Err(e) => {
                set_outcome_by_name(outcome_name, Outcome::new(false, Some(format!("{e}"))));
                return;
            }
        };

        if let Some(expected) = expected_count {
            if events.len() != expected {
                set_outcome_by_name(
                    outcome_name,
                    Outcome::new(
                        false,
                        Some(format!("Expected {} got {}", expected, events.len())),
                    ),
                );
                return;
            }
        }

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

        set_outcome_by_name(outcome_name, Outcome::new(true, None));
    }
}
