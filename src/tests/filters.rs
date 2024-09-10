use super::tags;
use crate::error::Error;
use crate::globals::{EventParts, Globals, GLOBALS};
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::{EventKind, Filter, Signer, Unixtime};
use std::time::Duration;

pub async fn since_until_are_inclusive() -> Result<Outcome, Error> {
    let time = Unixtime::now();
    let event = Globals::make_event(
        EventParts::Dated(
            EventKind::JobRequest(5000),
            tags(&[&["test"]]),
            "".to_string(),
            time,
        ),
        true,
    )?;

    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event, Duration::from_secs(WAIT))
        .await?;

    if !ok {
        return Ok(Outcome::err(reason));
    }

    let registered_public_key = GLOBALS.registered_user.read().public_key();

    let base_filter = {
        let mut filter = Filter::new();
        filter.authors = vec![registered_public_key.into()];
        filter.kinds = vec![EventKind::JobRequest(5000)];
        filter
    };
    let mut until_filter = base_filter.clone();
    until_filter.until = Some(time);
    let mut since_filter = base_filter.clone();
    since_filter.since = Some(time);

    let until_events = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![until_filter], Duration::from_secs(WAIT))
        .await?
        .into_events();

    let since_events = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![since_filter], Duration::from_secs(WAIT))
        .await?
        .into_events();

    match (until_events.is_empty(), since_events.is_empty()) {
        (true, true) => Ok(Outcome::fail(Some(
            "since and until are exclusive".to_owned(),
        ))),
        (false, true) => Ok(Outcome::fail(Some(
            "since is exclusive (until is rightly inclusive)".to_owned(),
        ))),
        (true, false) => Ok(Outcome::fail(Some(
            "until is exclusive (since is rightly inclusive)".to_owned(),
        ))),
        (false, false) => Ok(Outcome::pass(None)),
    }
}

pub async fn limit_zero() -> Result<Outcome, Error> {
    let registered_public_key = GLOBALS.registered_user.read().public_key();

    let filter = {
        let mut filter = Filter::new();
        filter.authors = vec![registered_public_key.into()];
        filter.limit = Some(0);
        filter
    };

    let fetch_result = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?;

    if !fetch_result.pre_eose_events.is_empty() {
        Ok(Outcome::fail(Some(
            "Returned events pre-EOSE with limit 0".to_owned(),
        )))
    } else if fetch_result.post_eose_events.is_none() {
        Ok(Outcome::fail(Some(
            "Did not return an EOSE with limit 0, timed out".to_owned(),
        )))
    } else if let Some(msg) = fetch_result.close_msg {
        Ok(Outcome::fail(Some(format!(
            "Did EOSE but then closed the subscription: {msg}"
        ))))
    } else {
        Ok(Outcome::pass(None))
    }
}
