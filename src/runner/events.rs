use nostr_types::{Event, EventKind, PreEvent, Signer, Tag, Unixtime};
use std::collections::HashMap;
use std::ops::Sub;
use std::time::Duration;

pub fn build_event_ago(
    user: &dyn Signer,
    minutes_ago: u64,
    kind: EventKind,
    intags: &[&[&str]],
) -> Event {
    let created_at = Unixtime::now()
        .unwrap()
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

const GROUP_A: [(&str, u64, EventKind, &[&[&str]]); 13] = [
    ("limit_test_first", 40, EventKind::TextNote, &[&["t", "a"]]),
    ("limit_test_third", 50, EventKind::TextNote, &[&["t", "a"]]),
    ("limit_test_second", 45, EventKind::TextNote, &[&["t", "b"]]),
    ("limit_test_fourth", 55, EventKind::TextNote, &[&["t", "b"]]),
    ("metadata_older", 60, EventKind::Metadata, &[]),
    ("metadata_newer", 0, EventKind::Metadata, &[]),
    ("contactlist_newer", 10, EventKind::ContactList, &[]),
    ("contactlist_older", 70, EventKind::ContactList, &[]),
    ("ephemeral", 10, EventKind::Ephemeral(21212), &[]),
    (
        "multipletags",
        10,
        EventKind::Other(30383),
        &[&["k", "3036"], &["n", "approved"]],
    ),
    (
        "multipletags_shouldntmatch",
        10,
        EventKind::Other(30383),
        &[&["n", "approved"]],
    ),
    (
        "older_param_replaceable",
        120,
        EventKind::FollowSets,
        &[&["d", "1"]],
    ),
    (
        "newer_param_replaceable",
        60,
        EventKind::FollowSets,
        &[&["d", "1"]],
    ),
];

pub fn build_event_group_a(user: &dyn Signer) -> HashMap<&'static str, Event> {
    let mut map: HashMap<&'static str, Event> = HashMap::new();
    for (s, m, k, t) in GROUP_A.iter() {
        map.insert(s, build_event_ago(user, *m, *k, *t));
    }
    map
}
