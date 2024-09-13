use super::tags;
use crate::error::Error;
use crate::globals::{EventParts, Globals, User, GLOBALS};
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::EventKind;
use std::time::Duration;

pub async fn public_can_write() -> Result<Outcome, Error> {
    let event = Globals::make_event(
        EventParts::Basic(EventKind::TextNote, tags(&[&["test"]]), "".to_string()),
        User::Stranger,
    )?;

    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event, Duration::from_secs(WAIT))
        .await?;

    if ok {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(Some(reason)))
    }
}

pub async fn accepts_relay_lists_from_public() -> Result<Outcome, Error> {
    let event = Globals::make_event(
        EventParts::Basic(EventKind::RelayList, tags(&[&["test"]]), "".to_string()),
        User::Stranger,
    )?;

    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event, Duration::from_secs(WAIT))
        .await?;

    if ok {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(Some(reason)))
    }
}

pub async fn accepts_dm_relay_lists_from_public() -> Result<Outcome, Error> {
    let event = Globals::make_event(
        EventParts::Basic(EventKind::DmRelayList, tags(&[&["test"]]), "".to_string()),
        User::Stranger,
    )?;

    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event, Duration::from_secs(WAIT))
        .await?;

    if ok {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(Some(reason)))
    }
}

pub async fn accepts_ephemeral_events_from_public() -> Result<Outcome, Error> {
    let event = Globals::make_event(
        EventParts::Basic(
            EventKind::WalletResponse,
            tags(&[&["test"]]),
            "".to_string(),
        ),
        User::Stranger,
    )?;

    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event, Duration::from_secs(WAIT))
        .await?;

    if ok {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(Some(reason)))
    }
}
