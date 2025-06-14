use crate::error::Error;
use crate::globals::GLOBALS;
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::{EventKind, Filter, Id, PrivateKey, Unixtime};
use std::time::Duration;

pub async fn supports_eose() -> Result<Outcome, Error> {
    // A very benign filter.
    let filter = {
        let mut filter = Filter::new();
        // Use a random author that should have 0 events
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        filter.add_author(public_key);
        filter.add_event_kind(EventKind::TextNote);
        filter.limit = Some(10);
        filter
    };

    let fresult = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?;

    match (fresult.close_msg, fresult.post_eose_events) {
        (None, None) => Ok(Outcome::fail(Some("Timed out without EOSE".to_owned()))),
        (None, Some(_)) => Ok(Outcome::pass(Some("Timed out after EOSE".to_owned()))),
        (Some(msg), None) => Ok(Outcome::fail(Some(format!(
            "Subscription was closed without EOSE: {}",
            msg
        )))),
        (Some(msg), Some(_)) => Ok(Outcome::pass(Some(format!("Closed after EOSE: {}", msg)))),
    }
}

pub async fn closes_complete_subscriptions_after_eose() -> Result<Outcome, Error> {
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

    let fresult = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?;

    match (fresult.close_msg, fresult.post_eose_events) {
        (None, None) => Ok(Outcome::fail(Some("Timed out without EOSE".to_owned()))),
        (None, Some(_)) => Ok(Outcome::fail(Some("Did not close after EOSE".to_owned()))),
        (Some(msg), None) => Ok(Outcome::fail(Some(format!(
            "Subscription was closed without EOSE: {}",
            msg
        )))),
        (Some(msg), Some(_)) => Ok(Outcome::pass(Some(format!("Closed after EOSE: {}", msg)))),
    }
}

pub async fn keeps_open_incomplete_subscriptions_after_eose() -> Result<Outcome, Error> {
    // Fetch some events of a single author (an incomplete subscription)
    let filter = {
        let mut filter = Filter::new();
        // Use a random author that should have 0 events
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        filter.add_author(public_key);
        filter.add_event_kind(EventKind::TextNote);
        filter.limit = Some(10);
        filter.until = Some(Unixtime(1_700_000_000)); // some time in the past
        filter
    };

    let fresult = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?;

    match (fresult.close_msg, fresult.post_eose_events) {
        (None, None) => Ok(Outcome::fail(Some("Timed out without EOSE".to_owned()))),
        (None, Some(_)) => Ok(Outcome::pass(Some("Did not close after EOSE".to_owned()))),
        (Some(msg), None) => Ok(Outcome::fail(Some(format!(
            "Subscription was closed without EOSE: {}",
            msg
        )))),
        (Some(msg), Some(_)) => Ok(Outcome::fail(Some(format!("Closed after EOSE: {}", msg)))),
    }
}
