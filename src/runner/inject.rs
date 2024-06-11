use nostr_types::{Event, EventKind, PreEvent, Signer, Tag, Unixtime};
use std::ops::Sub;
use std::time::Duration;

pub fn injected_events(registered_user: &dyn Signer, stranger: &dyn Signer) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    // Basic
    let mut pre_event = PreEvent {
        pubkey: registered_user.public_key(),
        created_at: Unixtime::now().unwrap().sub(Duration::new(3600, 0)), // 1 hour ago
        kind: EventKind::TextNote,
        tags: vec![],
        content: "This is a test.".to_owned(),
    };

    let event = registered_user.sign_event(pre_event.clone()).unwrap();
    events.push(event);

    // A reaction
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(2400 * 1, 0)); // 40m ago
    pre_event.tags = vec![
        Tag::from_strings(vec!["e".to_owned(), events[0].id.as_hex_string()]),
        Tag::from_strings(vec!["p".to_owned(), events[0].pubkey.as_hex_string()]),
    ];
    pre_event.content = "+".to_owned();
    pre_event.kind = EventKind::Reaction;
    let event = registered_user.sign_event(pre_event.clone()).unwrap();
    events.push(event);

    // Basic Reply
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(3000 * 1, 0)); // 50m ago
    pre_event.tags = vec![
        Tag::from_strings(vec!["e".to_owned(), events[0].id.as_hex_string()]),
        Tag::from_strings(vec!["p".to_owned(), events[0].pubkey.as_hex_string()]),
    ];
    pre_event.content = "Nice test.".to_owned();
    pre_event.kind = EventKind::TextNote;
    let event = registered_user.sign_event(pre_event.clone()).unwrap();
    events.push(event);

    // A reaction from stranger
    pre_event.pubkey = stranger.public_key();
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(2700 * 1, 0)); // 45m ago
    pre_event.tags = vec![
        Tag::from_strings(vec!["e".to_owned(), events[0].id.as_hex_string()]),
        Tag::from_strings(vec!["p".to_owned(), events[0].pubkey.as_hex_string()]),
    ];
    pre_event.content = "+".to_owned();
    pre_event.kind = EventKind::Reaction;
    let event = stranger.sign_event(pre_event.clone()).unwrap();
    events.push(event);

    // Basic Reply from stranger
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(3300 * 1, 0)); // 55m ago
    pre_event.tags = vec![
        Tag::from_strings(vec!["e".to_owned(), events[0].id.as_hex_string()]),
        Tag::from_strings(vec!["p".to_owned(), events[0].pubkey.as_hex_string()]),
    ];
    pre_event.content = "Nice test.".to_owned();
    pre_event.kind = EventKind::TextNote;
    let event = stranger.sign_event(pre_event.clone()).unwrap();
    events.push(event);

    events
}
