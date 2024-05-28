use crate::error::Error;
use crate::probe::{AuthState, Command, Probe};
use crate::results::Results;
use nostr_types::{EventKind, Filter, RelayMessage, SubscriptionId};

pub struct Runner {
    probe: Probe,
    results: Results,
}

impl Runner {
    pub fn new(probe: Probe) -> Runner {
        Runner {
            probe,
            results: Default::default(),
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        // Start with a quick listen. This will process any initial auth,
        // then it should timeout after 1 second.
        match self.probe.wait_for_a_response().await {
            Ok(message) => {
                // We didn't expect anything. Push it back.
                self.probe.push_back_relay_message(message);
            }
            Err(Error::Timeout(_)) => {} // expected,
            Err(e) => {
                // FIXME: This should be recorded in results instead.
                return Err(e);
            }
        }

        // Record results for initial authentication
        self.results.auths_initially = Some(match self.probe.auth_state() {
            AuthState::NotYetRequested => false,
            _ => true,
        });

        // Move on to a very benign filter.
        let our_sub_id = SubscriptionId("fetch_by_filter".to_string());
        let mut filter = Filter::new();
        filter.add_author(&self.probe.public_key().into());
        filter.add_event_kind(EventKind::TextNote);
        filter.limit = Some(10);
        self.probe
            .send(Command::FetchEvents(our_sub_id.clone(), vec![filter]))
            .await?;

        let rm = self.probe.wait_for_a_response().await?;
        match rm {
            RelayMessage::Eose(subid) => {
                if subid == our_sub_id {
                    // GOOD
                } else {
                    // HUH?
                }
            }
            other => {
                // We didn't expect anything else. Push it back.
                self.probe.push_back_relay_message(other);
            }
        }

        self.results.auth = self.probe.auth_state();

        Ok(())
    }

    pub async fn exit(self) -> Result<Results, Error> {
        self.probe.exit().await?;
        Ok(self.results)
    }
}
