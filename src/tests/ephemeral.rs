use super::tags;
use crate::connection::Connection;
use crate::error::Error;
use crate::globals::{EventParts, Globals, User, GLOBALS};
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::{EventKind, Filter, Signer};
use std::time::Duration;

pub async fn ephemeral_subscriptions_work() -> Result<Outcome, Error> {
    let filter = {
        let mut filter = Filter::new();
        filter.kinds = vec![EventKind::Ephemeral(25000)];
        filter.add_author(GLOBALS.registered1.read().public_key());
        filter
    };

    // On global connection - subscribe to the filter and wait for EOSE and a timeout
    // but keep the subscription open
    let fresult = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events_keep_open(vec![filter], Duration::from_secs(WAIT))
        .await?;
    let sub_id = fresult.sub_id.unwrap();

    // If the relay closed the sub, this test fails
    if let Some(msg) = fresult.close_msg {
        return Ok(Outcome::fail(Some(format!(
            "Relay closed our ephemeral subscription: {}",
            msg
        ))));
    }

    // Create a second parallel connection to the relay for injecting events
    let relay_url = GLOBALS.relay_url.read().to_owned();
    let mut injector = Connection::new(relay_url, 1000).await?;

    // Inject an ephemeral event
    let event = Globals::make_event(
        EventParts::Basic(
            EventKind::Ephemeral(25000),
            tags(&[&["test"]]),
            "".to_string(),
        ),
        User::Registered1,
    )?;
    let (ok, reason) = injector
        .post_event(event.clone(), Duration::from_secs(WAIT))
        .await?;
    if !ok {
        return Ok(Outcome::fail(Some(format!(
            "Relay rejected our ephemeral event: {}",
            reason
        ))));
    }

    // On global connection, collect events
    let collected_events = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .collect_events(sub_id, Duration::from_secs(WAIT))
        .await?;

    if collected_events.is_empty() {
        return Ok(Outcome::fail(Some(
            "Ephemeral event did not come through".to_owned(),
        )));
    }
    if collected_events[0] != event {
        return Ok(Outcome::fail(Some(
            "Ephemeral event does not match what we sent".to_owned(),
        )));
    }
    Ok(Outcome::pass(None))
}

pub async fn persists_ephemeral_events() -> Result<Outcome, Error> {
    // Inject an ephemeral event
    let event = Globals::make_event(
        EventParts::Basic(
            EventKind::Ephemeral(25001),
            tags(&[&["test"]]),
            "".to_string(),
        ),
        User::Registered1,
    )?;
    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event.clone(), Duration::from_secs(WAIT))
        .await?;
    if !ok {
        return Ok(Outcome::fail(Some(format!(
            "Relay rejected our ephemeral event: {}",
            reason
        ))));
    }

    // Now search for it
    let filter = {
        let mut filter = Filter::new();
        filter.kinds = vec![EventKind::Ephemeral(25001)];
        filter.add_author(GLOBALS.registered1.read().public_key());
        filter
    };

    let events = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?
        .into_events();

    for e in events.iter() {
        if event == *e {
            return Ok(Outcome::pass(None));
        }
    }

    Ok(Outcome::fail(None))
}
