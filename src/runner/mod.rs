use crate::error::Error;
use crate::probe::Probe;
use nostr_types::{KeySigner, PrivateKey};
use std::time::Duration;

mod tests;

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

        // Inject events as the registered user
        {
            // Authenticate as the registered user
            if self
                .probe
                .authenticate(&self.registered_user)
                .await
                .is_err()
            {
                eprintln!("Cannot authenticate. Cannot continue testing.");
                return Ok(());
            }

            // Inject events
            // TBD

            // Disconnect and reconnect to revert authentication
            self.probe.reconnect(Duration::new(1, 0)).await?;
        }

        // Authenticate as a stranger
        if self.probe.authenticate(&self.stranger1).await.is_err() {
            eprintln!("Cannot authenticate. Cannot continue testing.");
            return Ok(());
        }

        // Tests that run as a stranger
        // TBD

        // Authenticate as the configured registered user
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
}
