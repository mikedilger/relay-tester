pub mod auth;
pub mod eose;
pub mod ephemeral;
pub mod find;
pub mod json;
pub mod misc_events;
pub mod nip11;
pub mod public;
pub mod reg;
pub mod replaceables;
pub mod time;

use crate::error::Error;
use crate::globals::{EventParts, GLOBALS};
use crate::outcome::Outcome;
use nostr_types::{EventKind, Tag, Unixtime};
use std::ops::Sub;
use std::sync::atomic::Ordering;
use std::time::Duration;

pub fn tbd() -> Result<Outcome, Error> {
    Ok(Outcome::err("NOT YET IMPLEMENTED".to_string()))
}

fn tags(intags: &[&[&str]]) -> Vec<Tag> {
    let mut tags: Vec<Tag> = Vec::new();
    for tin in intags.iter() {
        tags.push(Tag::new(tin))
    }
    tags
}

async fn maybe_submit_event_group_a() -> Result<(), Error> {
    if GLOBALS.event_group_a_submitted.load(Ordering::Relaxed) {
        // Already submitted
        return Ok(());
    }

    if GLOBALS.event_group_a_failed.load(Ordering::Relaxed) {
        // Already tried and it failed
        return Err(Error::PrerequisiteEventSubmissionFailed);
    }

    match maybe_submit_event_group_a_inner().await {
        Ok(()) => {
            GLOBALS
                .event_group_a_submitted
                .store(true, Ordering::Relaxed);
            Ok(())
        }
        Err(e) => {
            GLOBALS.event_group_a_failed.store(true, Ordering::Relaxed);
            Err(e)
        }
    }
}

async fn maybe_submit_event_group_a_inner() -> Result<(), Error> {
    let mut lock = GLOBALS.event_group_a.write();

    lock.insert(
        "limit_test_first",
        EventParts::Dated(
            EventKind::TextNote,
            tags(&[&["t", "a"]]),
            "limit_test_first".to_owned(),
            minutes_ago(40),
        ),
        true,
    )
    .await?;

    lock.insert(
        "limit_test_third",
        EventParts::Dated(
            EventKind::TextNote,
            tags(&[&["t", "a"]]),
            "limit_test_third".to_owned(),
            minutes_ago(50),
        ),
        true,
    )
    .await?;

    lock.insert(
        "limit_test_second",
        EventParts::Dated(
            EventKind::TextNote,
            tags(&[&["t", "b"]]),
            "limit_test_second".to_owned(),
            minutes_ago(45),
        ),
        true,
    )
    .await?;

    lock.insert(
        "limit_test_fourth",
        EventParts::Dated(
            EventKind::TextNote,
            tags(&[&["t", "b"]]),
            "limit_test_fourth".to_owned(),
            minutes_ago(55),
        ),
        true,
    )
    .await?;

    lock.insert(
        "metadata_older",
        EventParts::Dated(
            EventKind::Metadata,
            tags(&[]),
            "metadata_older".to_owned(),
            minutes_ago(60),
        ),
        false,
    )
    .await?;

    lock.insert(
        "metadata_newer",
        EventParts::Dated(
            EventKind::Metadata,
            tags(&[]),
            "metadata_newer".to_owned(),
            minutes_ago(0),
        ),
        true,
    )
    .await?;

    lock.insert(
        "contactlist_newer",
        EventParts::Dated(
            EventKind::ContactList,
            tags(&[]),
            "contactlist_newer".to_owned(),
            minutes_ago(10),
        ),
        true,
    )
    .await?;

    lock.insert(
        "contactlist_older",
        EventParts::Dated(
            EventKind::ContactList,
            tags(&[]),
            "contactlist_older".to_owned(),
            minutes_ago(70),
        ),
        false,
    )
    .await?;

    lock.insert(
        "ephemeral",
        EventParts::Dated(
            EventKind::Ephemeral(21212),
            tags(&[]),
            "ephemeral".to_owned(),
            minutes_ago(10),
        ),
        false,
    )
    .await?;

    lock.insert(
        "multipletags",
        EventParts::Dated(
            EventKind::Other(30383),
            tags(&[&["k", "3036"], &["n", "approved"]]),
            "multipletags".to_owned(),
            minutes_ago(10),
        ),
        true,
    )
    .await?;

    lock.insert(
        "multipletags_shouldntmatch",
        EventParts::Dated(
            EventKind::Other(30383),
            tags(&[&["n", "approved"]]),
            "multipletags_shouldntmatch".to_owned(),
            minutes_ago(10),
        ),
        true,
    )
    .await?;

    lock.insert(
        "older_param_replaceable",
        EventParts::Dated(
            EventKind::FollowSets,
            tags(&[&["d", "1"]]),
            "older_param_replaceable".to_owned(),
            minutes_ago(120),
        ),
        false,
    )
    .await?;

    lock.insert(
        "newer_param_replaceable",
        EventParts::Dated(
            EventKind::FollowSets,
            tags(&[&["d", "1"]]),
            "newer_param_replaceable".to_owned(),
            minutes_ago(60),
        ),
        true,
    )
    .await?;

    lock.insert(
        "older_replaceable",
        EventParts::Dated(
            EventKind::BookmarkList,
            tags(&[&[
                "e",
                "65f07794c052916f434d2a40ad4e3c58c1c287d829b999977a7221c0ebadab0a",
            ]]),
            "older_replaceable".to_owned(),
            minutes_ago(80),
        ),
        false,
    )
    .await?;

    lock.insert(
        "newer_replaceable",
        EventParts::Dated(
            EventKind::BookmarkList,
            tags(&[&[
                "e",
                "65f07794c052916f434d2a40ad4e3c58c1c287d829b999977a7221c0ebadab0a",
            ]]),
            "newer_replaceable".to_owned(),
            minutes_ago(60),
        ),
        false,
    )
    .await?;

    Ok(())
}

fn minutes_ago(m: u64) -> Unixtime {
    Unixtime::now().sub(Duration::new(m * 60, 0))
}
