use super::maybe_submit_event_group_a; // tags
                                       //use crate::connection::Connection;
use crate::error::Error;
use crate::globals::GLOBALS; // EventParts, Globals
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::{EventKind, Filter, Id, Signer}; // PublicKeyHex;
use std::time::Duration;

pub async fn accepts_metadata() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    let metadata_older_id: Id = GLOBALS
        .event_group_a
        .read()
        .get("metadata_older")
        .unwrap()
        .0
        .id;
    let metadata_newer_id: Id = GLOBALS
        .event_group_a
        .read()
        .get("metadata_newer")
        .unwrap()
        .0
        .id;

    let filter = {
        let mut filter = Filter::new();
        filter.ids = vec![metadata_older_id.into(), metadata_newer_id.into()];
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

    let have_older = events.iter().any(|e| e.id == metadata_older_id);
    let have_newer = events.iter().any(|e| e.id == metadata_newer_id);

    match (have_older, have_newer) {
        (false, false) => Ok(Outcome::fail(None)),
        _ => Ok(Outcome::pass(None)),
    }
}

pub async fn replaces_metadata() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    let metadata_older_id: Id = GLOBALS
        .event_group_a
        .read()
        .get("metadata_older")
        .unwrap()
        .0
        .id;
    let metadata_newer_id: Id = GLOBALS
        .event_group_a
        .read()
        .get("metadata_newer")
        .unwrap()
        .0
        .id;

    let registered_public_key = GLOBALS.registered_user.read().public_key();

    let filter = {
        let mut filter = Filter::new();
        filter.kinds = vec![EventKind::Metadata];
        filter.authors = vec![registered_public_key.into()];
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

    let have_older = events.iter().any(|e| e.id == metadata_older_id);
    let have_newer = events.iter().any(|e| e.id == metadata_newer_id);

    match (have_older, have_newer) {
        (false, false) => Ok(Outcome::fail(Some("Not accepting metadata".to_owned()))),
        (false, true) => Ok(Outcome::pass(None)),
        (true, false) => Ok(Outcome::fail(Some(
            "Older metadata is returned, new metadata is not returned".to_owned(),
        ))),
        (true, true) => Ok(Outcome::fail(Some(
            "Multiple metadata are returned".to_owned(),
        ))),
    }
}

pub async fn accepts_contact_list() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    let contactlist_older_id: Id = GLOBALS
        .event_group_a
        .read()
        .get("contactlist_older")
        .unwrap()
        .0
        .id;
    let contactlist_newer_id: Id = GLOBALS
        .event_group_a
        .read()
        .get("contactlist_newer")
        .unwrap()
        .0
        .id;

    let filter = {
        let mut filter = Filter::new();
        filter.ids = vec![contactlist_older_id.into(), contactlist_newer_id.into()];
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

    let have_older = events.iter().any(|e| e.id == contactlist_older_id);
    let have_newer = events.iter().any(|e| e.id == contactlist_newer_id);

    match (have_older, have_newer) {
        (false, false) => Ok(Outcome::fail(None)),
        _ => Ok(Outcome::pass(None)),
    }
}

pub async fn replaces_contact_list() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    let contactlist_older_id: Id = GLOBALS
        .event_group_a
        .read()
        .get("contactlist_older")
        .unwrap()
        .0
        .id;
    let contactlist_newer_id: Id = GLOBALS
        .event_group_a
        .read()
        .get("contactlist_newer")
        .unwrap()
        .0
        .id;

    let registered_public_key = GLOBALS.registered_user.read().public_key();

    let filter = {
        let mut filter = Filter::new();
        filter.kinds = vec![EventKind::ContactList];
        filter.authors = vec![registered_public_key.into()];
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

    let have_older = events.iter().any(|e| e.id == contactlist_older_id);
    let have_newer = events.iter().any(|e| e.id == contactlist_newer_id);

    match (have_older, have_newer) {
        (false, false) => Ok(Outcome::fail(Some(
            "Not accepting contact lists".to_owned(),
        ))),
        (false, true) => Ok(Outcome::pass(None)),
        (true, false) => Ok(Outcome::fail(Some(
            "Older contact list is returned, new contact list is not returned".to_owned(),
        ))),
        (true, true) => Ok(Outcome::fail(Some(
            "Multiple contact lists are returned".to_owned(),
        ))),
    }
}

pub async fn replaced_events_still_available_by_id() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    let contactlist_older_id: Id = GLOBALS
        .event_group_a
        .read()
        .get("contactlist_older")
        .unwrap()
        .0
        .id;

    let filter = {
        let mut filter = Filter::new();
        filter.ids = vec![contactlist_older_id.into()];
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

    let have_older = events.iter().any(|e| e.id == contactlist_older_id);

    if have_older {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(None))
    }
}

pub async fn replaceable_event_removes_previous() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}

pub async fn replaceable_event_doesnt_remove_future() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}

pub async fn addressable_event_removes_previous() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}

pub async fn addressable_event_doesnt_remove_future() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}

pub async fn find_replaceable_event() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    let newer_replaceable = GLOBALS
        .event_group_a
        .read()
        .get("newer_replaceable")
        .unwrap()
        .0
        .clone();

    let filter = {
        let mut filter = Filter::new();
        filter.kinds = vec![newer_replaceable.kind];
        filter.authors = vec![newer_replaceable.pubkey.into()];
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

    let have = events.iter().any(|e| e.id == newer_replaceable.id);

    if have {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(None))
    }
}

pub async fn find_addressable_event() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;
    let newer_addressable = GLOBALS
        .event_group_a
        .read()
        .get("newer_param_replaceable")
        .unwrap()
        .0
        .clone();

    let filter = {
        let mut filter = Filter::new();
        filter.kinds = vec![newer_addressable.kind];
        filter.authors = vec![newer_addressable.pubkey.into()];
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

    let have = events.iter().any(|e| e.id == newer_addressable.id);

    if have {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(None))
    }
}