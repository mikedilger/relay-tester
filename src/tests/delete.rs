use super::{minutes_ago, tags};
use crate::error::Error;
use crate::globals::{EventParts, Globals, GLOBALS};
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::{Event, EventKind, Filter, NAddr};
use std::time::Duration;

pub async fn delete_by_id() -> Result<Outcome, Error> {
    // Make an event
    let event = Globals::make_event(
        EventParts::Basic(
            EventKind::TextNote,
            tags(&[&["test"]]),
            "I say wrong thing".to_string(),
        ),
        true,
    )?;
    let event_id = event.id;

    // Submit it
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

    // Make a deletion event, e-tag
    let delete_event = Globals::make_event(
        EventParts::Basic(
            EventKind::EventDeletion,
            tags(&[&["e", &event_id.as_hex_string()]]),
            "".to_string(),
        ),
        true,
    )?;

    // Submit it
    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(delete_event, Duration::from_secs(WAIT))
        .await?;
    if !ok {
        return Ok(Outcome::err(reason));
    }

    // Fetch back the original event
    let mut filter = Filter::new();
    filter.ids = vec![event_id.into()];
    let events = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?
        .into_events();

    if events.is_empty() {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(Some(
            "Deleted event did not get deleted".to_owned(),
        )))
    }
}

pub async fn delete_by_addr() -> Result<Outcome, Error> {
    // Make an event
    let event = Globals::make_event(
        EventParts::Basic(
            EventKind::LongFormContent,
            tags(&[&["d", "delete_by_addr_test"]]),
            "I say wrong thing".to_string(),
        ),
        true,
    )?;
    let event_id = event.id;

    // Compute event group address
    let naddr = NAddr {
        d: "delete_by_addr_test".to_owned(),
        relays: vec![],
        kind: EventKind::LongFormContent,
        author: event.pubkey,
    };
    let a_tag = format!(
        "{}:{}:{}",
        Into::<u32>::into(naddr.kind),
        naddr.author.as_hex_string(),
        naddr.d
    );

    // Submit it
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

    // Make a deletion event, a-tag
    let delete_event = Globals::make_event(
        EventParts::Basic(
            EventKind::EventDeletion,
            tags(&[&["a", &a_tag]]),
            "".to_string(),
        ),
        true,
    )?;

    // Submit it
    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(delete_event, Duration::from_secs(WAIT))
        .await?;
    if !ok {
        return Ok(Outcome::err(reason));
    }

    // Fetch back the original event (by ID this time)
    let mut filter = Filter::new();
    filter.ids = vec![event_id.into()];
    let events = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?
        .into_events();

    if events.is_empty() {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(Some(
            "Deleted event did not get deleted".to_owned(),
        )))
    }
}

pub async fn delete_by_addr_only_older() -> Result<Outcome, Error> {
    // Prepare some times
    let time1 = minutes_ago(5);
    let time2 = minutes_ago(3);
    let time3 = minutes_ago(1);

    // Make an event, time1
    let event1 = Globals::make_event(
        EventParts::Dated(
            EventKind::LongFormContent,
            tags(&[&["d", "delete_by_addr_only_older_test"]]),
            "I say wrong thing".to_string(),
            time1,
        ),
        true,
    )?;
    let event1_id = event1.id;

    // Compute event group address
    let naddr = NAddr {
        d: "delete_by_addr_only_older_test".to_owned(),
        relays: vec![],
        kind: EventKind::LongFormContent,
        author: event1.pubkey,
    };
    let a_tag = format!(
        "{}:{}:{}",
        Into::<u32>::into(naddr.kind),
        naddr.author.as_hex_string(),
        naddr.d
    );

    // Submit it
    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event1, Duration::from_secs(WAIT))
        .await?;
    if !ok {
        return Ok(Outcome::err(reason));
    }

    // Make an event, time3
    let event3 = Globals::make_event(
        EventParts::Dated(
            EventKind::LongFormContent,
            tags(&[&["d", "delete_by_addr_only_older_test"]]),
            "I say right thing".to_string(),
            time3,
        ),
        true,
    )?;
    let event3_id = event3.id;

    // Submit it
    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event3, Duration::from_secs(WAIT))
        .await?;
    if !ok {
        return Ok(Outcome::err(reason));
    }

    // Make a deletion event, a-tag
    let delete_event = Globals::make_event(
        EventParts::Dated(
            EventKind::EventDeletion,
            tags(&[&["a", &a_tag]]),
            "".to_string(),
            time2,
        ),
        true,
    )?;

    // Submit it
    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(delete_event, Duration::from_secs(WAIT))
        .await?;
    if !ok {
        return Ok(Outcome::err(reason));
    }

    // Fetch back the events in this event group
    let mut filter = Filter::new();
    filter.authors.push(naddr.author.into());
    filter.add_event_kind(naddr.kind);
    filter.add_tag_value('d', naddr.d);
    let events = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?
        .into_events();

    // We SHOULD have just event3, not event1
    if events.iter().any(|e| e.id == event1_id) {
        Ok(Outcome::fail(Some(
            "Failed to delete addressable event older than delete event".to_owned(),
        )))
    } else if !events.iter().any(|e| e.id == event3_id) {
        Ok(Outcome::fail(Some(
            "Wrongly deleted addressable event newer than delete event".to_owned(),
        )))
    } else {
        Ok(Outcome::pass(None))
    }
}

pub async fn delete_by_addr_bound_by_tag() -> Result<Outcome, Error> {
    let mut events: Vec<Event> = vec![];


    // Make 4 events, the final 3 differing by the first by just one factor
    events.push(Globals::make_event(
        EventParts::Basic(
            EventKind::LongFormContent,
            tags(&[&["d", "delete_by_addr_test_bound"]]),
            "I say wrong thing".to_string(),
        ),
        true,
    )?);
    events.push(Globals::make_event(
        EventParts::Basic(
            EventKind::LongFormContent,
            tags(&[&["d", "delete_by_addr_test_bound"]]),
            "I say wrong thing".to_string(),
        ),
        false, // different author
    )?);
    events.push(Globals::make_event(
        EventParts::Basic(
            EventKind::LongFormContent,
            tags(&[&["d", "delete_by_addr_test_bound_x"]]), // different d-tag
            "I say wrong thing".to_string(),
        ),
        true,
    )?);
    events.push(Globals::make_event(
        EventParts::Basic(
            EventKind::DraftLongFormContent, // different kind
            tags(&[&["d", "delete_by_addr_test_bound"]]),
            "I say wrong thing".to_string(),
        ),
        true,
    )?);

    // Submit all events
    {
        let mut lock = GLOBALS.connection.write();
        for event in &events {
            let (ok, reason) = lock
                .as_mut()
                .unwrap()
                .post_event(event.clone(), Duration::from_secs(WAIT))
                .await?;
            if !ok {
                return Ok(Outcome::err(reason));
            }
        }
    }

    // Make a deletion event, a-tag
    let delete_event = {
        let a_tag = format!(
            "{}:{}:{}",
            Into::<u32>::into(EventKind::LongFormContent),
            events[0].pubkey.as_hex_string(),
            "delete_by_addr_test_bound"
        );
        Globals::make_event(
            EventParts::Basic(
                EventKind::EventDeletion,
                tags(&[&["a", &a_tag]]),
                "".to_string(),
            ),
            true,
        )?
    };

    // Submit it
    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(delete_event, Duration::from_secs(WAIT))
        .await?;
    if !ok {
        return Ok(Outcome::err(reason));
    }

    // Fetch all original events by id
    let mut filter = Filter::new();
    filter.ids = events.iter().map(|e| e.id.into()).collect();
    let events_back = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?
        .into_events();

    if events_back.iter().any(|e| e.id == events[0].id) {
        Ok(Outcome::fail(Some("Failed to delete".to_string())))
    } else if ! events_back.iter().any(|e| e.id == events[1].id) {
        Ok(Outcome::fail(Some("Also deleted event of a different author!".to_string())))
    } else if ! events_back.iter().any(|e| e.id == events[2].id) {
        Ok(Outcome::fail(Some("Also deleted event of a different d-tag!".to_string())))
    } else if ! events_back.iter().any(|e| e.id == events[3].id) {
        Ok(Outcome::fail(Some("Also deleted event of a different kind!".to_string())))
    } else {
        Ok(Outcome::pass(None))
    }
}

pub async fn delete_by_id_of_others() -> Result<Outcome, Error> {
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}

pub async fn delete_by_addr_of_others() -> Result<Outcome, Error> {
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}

pub async fn resubmission_of_delete_by_id() -> Result<Outcome, Error> {
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}

pub async fn resubmission_of_older_delete_by_addr() -> Result<Outcome, Error> {
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}

pub async fn resubmission_of_newer_delete_by_addr() -> Result<Outcome, Error> {
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}
