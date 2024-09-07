use super::tags;
use crate::error::Error;
use crate::globals::{EventParts, Globals, GLOBALS};
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::{EventKind, Id, Signature, Signer};
use std::sync::atomic::Ordering;
use std::time::Duration;

pub async fn sends_ok_after_event() -> Result<Outcome, Error> {
    Ok(match GLOBALS.saw_ok_after_event.load(Ordering::Relaxed) {
        true => Outcome::pass(None),
        false => Outcome::fail(None),
    })
}

pub async fn verifies_signatures() -> Result<Outcome, Error> {
    let mut event = Globals::make_event(
        EventParts::Basic(EventKind::TextNote, tags(&[&["test"]]), "".to_string()),
        false,
    )?;

    event.sig = Signature::zeroes();

    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event, Duration::from_secs(WAIT))
        .await?;

    if ok {
        Ok(Outcome::fail(Some(
            "Accepted event with a bad sig".to_owned(),
        )))
    } else {
        Ok(Outcome::pass(Some(reason)))
    }
}

pub async fn verifies_id_hashes() -> Result<Outcome, Error> {
    let mut event = Globals::make_event(
        EventParts::Basic(EventKind::TextNote, tags(&[&["test"]]), "".to_string()),
        false,
    )?;

    event.id =
        Id::try_from_hex_string("cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe")
            .unwrap();
    event.sig = GLOBALS.registered_user.read().sign_id(event.id).unwrap();

    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event, Duration::from_secs(WAIT))
        .await?;

    if ok {
        Ok(Outcome::fail(Some(
            "Accepted event with a bad id hash".to_owned(),
        )))
    } else {
        Ok(Outcome::pass(Some(reason)))
    }
}
