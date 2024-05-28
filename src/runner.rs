use crate::error::Error;
use crate::probe::{AuthState, Command, Probe};
use colorful::{Color, Colorful};
use nostr_types::{
    EventKind, Filter, KeySigner, PreEvent, PrivateKey, RelayMessage, Signer, SubscriptionId, Tag,
    Unixtime
};
use paste::paste;
use std::fmt;

#[derive(Debug)]
pub enum Outcome {
    Untested,
    Pass,
    Fail,
    Fail2(String),
    Info(String),
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Untested => write!(f, "????"),
            Outcome::Pass => write!(f, "{}", "PASS".color(Color::Green)),
            Outcome::Fail => write!(f, "{}", "FAIL".color(Color::Red)),
            Outcome::Fail2(s) => write!(f, "{} ({})", "FAIL".color(Color::Red), s),
            Outcome::Info(s) => write!(f, "{} ({})", "INFO".color(Color::Gold1), s),
        }
    }
}

#[derive(Debug)]
pub struct Test {
    pub name: String,
    pub outcome: Outcome,
}

impl fmt::Display for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.outcome)
    }
}

macro_rules! run_test {
    ($self:ident, $name:ident) => {
        paste! {
            let outcome = $self.[<test_ $name>]().await?;
            $self.results.push(Test {
                name: stringify!($name).to_string(),
                outcome
            });
        }
    };
}

pub struct Runner {
    probe: Probe,
    results: Vec<Test>,
}

impl Runner {
    pub fn new(probe: Probe) -> Runner {
        Runner {
            probe,
            results: Default::default(),
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        run_test!(self, initial_prompt_for_auth);
        run_test!(self, eose);
        run_test!(self, public_write);

        // Authenticate if we can before continuing
        self.probe.authenticate().await?;


        run_test!(self, auth);
        Ok(())
    }

    pub async fn exit(self) -> Result<Vec<Test>, Error> {
        self.probe.exit().await?;
        Ok(self.results)
    }

    async fn test_initial_prompt_for_auth(&mut self) -> Result<Outcome, Error> {
        // Start with a quick listen. This will process any initial auth,
        // then it should timeout after 1 second.
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
            _ => Outcome::Pass,
        })
    }

    async fn test_eose(&mut self) -> Result<Outcome, Error> {
        // Move on to a very benign filter.
        let our_sub_id = SubscriptionId("fetch_by_filter".to_string());
        let mut filter = Filter::new();
        filter.add_author(&self.probe.public_key().into());
        filter.add_event_kind(EventKind::TextNote);
        filter.limit = Some(10);
        self.probe
            .send(Command::FetchEvents(our_sub_id.clone(), vec![filter]))
            .await?;

        let outcome;
        loop {
            let rm = match self.probe.wait_for_a_response().await {
                Ok(rm) => rm,
                Err(Error::Timeout(_)) => {
                    outcome = Outcome::Fail;
                    break;
                },
                Err(e) => return Err(e)
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
        };

        Ok(outcome)
    }

    async fn test_public_write(&mut self) -> Result<Outcome, Error> {
        // Generate a random keypair
        let private_key = PrivateKey::generate();
        let signer = KeySigner::from_private_key(private_key, "", 8)?;
        let public_key = signer.public_key();

        // Generate an event from them
        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now().unwrap(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["test"]),
            ],
            content: "This is a test from a random keypair. Feel free to delete.".to_string(),
        };
        let event = signer.sign_event(pre_event)?;
        let initial_id = event.id;

        // Post the event
        self.probe.send(Command::PostEvent(event)).await?;

        // Wait for an Ok response
        let outcome;
        loop {
            let rm = match self.probe.wait_for_a_response().await {
                Ok(rm) => rm,
                Err(Error::Timeout(_)) => {
                    outcome = Outcome::Fail2("No response to an EVENT submission".to_owned());
                    break;
                },
                Err(e) => return Err(e)
            };

            match rm {
                RelayMessage::Ok(id, ok, reason) => {
                    if id == initial_id {
                        if ok {
                            outcome = Outcome::Info("Accepts events from the public".to_owned());
                        } else {
                            outcome = Outcome::Info(reason);
                        }
                    } else {
                        outcome = Outcome::Fail2("Responded to EVENT with OK with a different id".to_owned());
                    }
                    break;
                },
                _ => {
                    // We didn't expect that
                    continue;
                }
            }
        };

        // FIXME this isn't good enough, we need to read it back and make sure it is there.

        Ok(outcome)
    }

    async fn test_auth(&mut self) -> Result<Outcome, Error> {
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
            AuthState::Challenged(_) => Outcome::Fail2("Challenged but we failed to AUTH back".to_string()),
            AuthState::InProgress(_) => Outcome::Fail2("Did not OK the AUTH".to_string()),
            AuthState::Success => Outcome::Pass,
            AuthState::Failure(s) => Outcome::Fail2(s),
            AuthState::Duplicate => Outcome::Fail2("AUTHed multiple times".to_string()),
        })
    }


    /*
    // authed submission of other people's events
    async fn test_public_write(&mut self) -> Result<Outcome, Error> {
    async fn test_write_and_read_back(&mut self) -> Result<Outcome, Error> {
    async fn test_find_by_id(&mut self) -> Result<Outcome, Error> {
    async fn test_find_by_pubkey_and_kind(&mut self) -> Result<Outcome, Error> {
    async fn test_find_by_pubkey_and_tags(&mut self) -> Result<Outcome, Error> {
    async fn test_find_by_kind_and_tags(&mut self) -> Result<Outcome, Error> {
    async fn test_find_by_tags(&mut self) -> Result<Outcome, Error> {
    async fn test_find_by_pubkey(&mut self) -> Result<Outcome, Error> {
    async fn test_find_by_scrape(&mut self) -> Result<Outcome, Error> {
    async fn test_find_replaceable_event(&mut self) -> Result<Outcome, Error> {
    async fn test_find_parameterized_replaceable_event(&mut self) -> Result<Outcome, Error> {
    async fn test_delete_by_id_event_is_deleted(&mut self) -> Result<Outcome, Error> {
    async fn test_cannot_delete_by_id_events_of_others(&mut self) -> Result<Outcome, Error> {
    async fn test_resubmission_of_deleted_by_id_event_is_rejected(&mut self) -> Result<Outcome, Error> {
    async fn test_deleted_by_npnaddr_event_is_deleted(&mut self) -> Result<Outcome, Error> {
    async fn test_cannot_delete_by_npnaddr_events_of_others(&mut self) -> Result<Outcome, Error> {
    async fn test_resubmission_of_deleted_by_npnaddr_event_is_rejected(&mut self) -> Result<Outcome, Error> {
    async fn test_submission_of_any_older_deleted_by_npnaddr_event_is_rejected(&mut self) -> Result<Outcome, Error> {
    async fn test_submission_of_any_newer_deleted_by_npnaddr_event_is_accepted(&mut self) -> Result<Outcome, Error> {
    async fn test_deleted_by_npnaddr_doesnt_affect_newer_events(&mut self) -> Result<Outcome, Error> {
    async fn test_deleted_by_pnaddr_event_is_deleted(&mut self) -> Result<Outcome, Error> {
    async fn test_cannot_delete_by_pnaddr_events_of_others(&mut self) -> Result<Outcome, Error> {
    async fn test_resubmission_of_deleted_by_pnaddr_event_is_rejected(&mut self) -> Result<Outcome, Error> {
    async fn test_submission_of_any_older_deleted_by_pnaddr_event_is_rejected(&mut self) -> Result<Outcome, Error> {
    async fn test_submission_of_any_newer_deleted_by_pnaddr_event_is_accepted(&mut self) -> Result<Outcome, Error> {
    async fn test_deleted_by_pnaddr_doesnt_affect_newer_events(&mut self) -> Result<Outcome, Error> {
    async fn test_deleted_by_pnaddr_is_bound_by_d_tag(&mut self) -> Result<Outcome, Error> {
    async fn test_replaceable_event_removes_previous(&mut self) -> Result<Outcome, Error> {
    async fn test_parameterized_replaceable_event_removes_previous(&mut self) -> Result<Outcome, Error> {
    async fn test_naddr_is_deleted_asof(&mut self) -> Result<Outcome, Error> {
    async fn test_(&mut self) -> Result<Outcome, Error> {
    async fn test_(&mut self) -> Result<Outcome, Error> {
    // NIP-04 DM tests TBD
    // NIP-11 relay info doc tests TBD
    // NIP-26 delegated events ests TBD
    // NIP-29 relay based group tests TBD
    // NIP-40 expiration timestamp tests TBD
    // NIP-45 count tests TBD
    // NIP-50 search tests TBD
    // NIP-59 giftwrap tests TBD
    // NIP-65 relay list tests TBD
    // NIP-94 file metadata tests TBD
    // NIP-96 http file storage tests TBD
    // test large contact lists
    */
}
