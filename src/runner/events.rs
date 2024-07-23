use nostr_types::{Event, EventKind, PreEvent, Signer, Tag, Unixtime};
use std::ops::Sub;
use std::time::Duration;

pub fn build_event_ago(
    user: &dyn Signer,
    minutes_ago: u64,
    kind: EventKind,
    intags: &[&[&str]],
) -> Event {
    let created_at = Unixtime::now()
        .sub(Duration::new(minutes_ago * 60, 0));
    build_event(user, created_at, kind, intags)
}

pub fn build_event(
    user: &dyn Signer,
    created_at: Unixtime,
    kind: EventKind,
    intags: &[&[&str]],
) -> Event {
    let mut tags: Vec<Tag> = Vec::new();
    for tin in intags.iter() {
        tags.push(Tag::from_strings(
            tin.iter().map(|s| (*s).to_owned()).collect(),
        ));
    }
    let pre_event = PreEvent {
        pubkey: user.public_key(),
        created_at,
        kind,
        tags,
        content: textnonce::TextNonce::new().to_string(), // to ensure we have no dups
    };
    let event = user.sign_event(pre_event).unwrap();
    event
}

pub struct EventData {
    pub name: &'static str,
    pub minutes_ago: u64,
    pub kind: EventKind,
    pub can_read_back: bool,
    pub tags: &'static [&'static [&'static str]],
}

impl EventData {
    pub const fn new(
        data: (
            &'static str,
            u64,
            EventKind,
            bool,
            &'static [&'static [&'static str]],
        ),
    ) -> EventData {
        EventData {
            name: data.0,
            minutes_ago: data.1,
            kind: data.2,
            can_read_back: data.3,
            tags: data.4,
        }
    }
}

pub const GROUP_A: [EventData; 15] = [
    EventData::new((
        "limit_test_first",
        40,
        EventKind::TextNote,
        true,
        &[&["t", "a"]],
    )),
    EventData::new((
        "limit_test_third",
        50,
        EventKind::TextNote,
        true,
        &[&["t", "a"]],
    )),
    EventData::new((
        "limit_test_second",
        45,
        EventKind::TextNote,
        true,
        &[&["t", "b"]],
    )),
    EventData::new((
        "limit_test_fourth",
        55,
        EventKind::TextNote,
        true,
        &[&["t", "b"]],
    )),
    EventData::new(("metadata_older", 60, EventKind::Metadata, false, &[])),
    EventData::new(("metadata_newer", 0, EventKind::Metadata, true, &[])),
    EventData::new(("contactlist_newer", 10, EventKind::ContactList, true, &[])),
    EventData::new(("contactlist_older", 70, EventKind::ContactList, false, &[])),
    EventData::new(("ephemeral", 10, EventKind::Ephemeral(21212), false, &[])),
    EventData::new((
        "multipletags",
        10,
        EventKind::Other(30383),
        true,
        &[&["k", "3036"], &["n", "approved"]],
    )),
    EventData::new((
        "multipletags_shouldntmatch",
        10,
        EventKind::Other(30383),
        true,
        &[&["n", "approved"]],
    )),
    EventData::new((
        "older_param_replaceable",
        120,
        EventKind::FollowSets,
        false,
        &[&["d", "1"]],
    )),
    EventData::new((
        "newer_param_replaceable",
        60,
        EventKind::FollowSets,
        true,
        &[&["d", "1"]],
    )),
    EventData::new((
        "older_replaceable",
        80,
        EventKind::BookmarkList,
        false,
        &[&[
            "e",
            "65f07794c052916f434d2a40ad4e3c58c1c287d829b999977a7221c0ebadab0a",
        ]],
    )),
    EventData::new((
        "newer_replaceable",
        60,
        EventKind::BookmarkList,
        true,
        &[&[
            "e",
            "65f07794c052916f434d2a40ad4e3c58c1c287d829b999977a7221c0ebadab0a",
        ]],
    )),
];
