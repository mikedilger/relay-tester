use nostr_types::{Event, EventKind, PreEvent, Signer, Tag, Unixtime};
use std::ops::Sub;
use std::time::Duration;

pub fn injected_events(user: &dyn Signer) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    // Basic
    let mut pre_event = PreEvent {
        pubkey: user.public_key(),
        created_at: Unixtime::now().unwrap().sub(Duration::new(3600, 0)), // 1 hour ago
        kind: EventKind::TextNote,
        tags: vec![],
        content: "This is a test.".to_owned(),
    };

    let event = user.sign_event(pre_event.clone()).unwrap();
    events.push(event);

    // Basic Reply
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(3000 * 1, 0)); // 50m ago
    pre_event.tags = vec![
        Tag::from_strings(vec!["e".to_owned(), events[0].id.as_hex_string()]),
        Tag::from_strings(vec!["p".to_owned(), events[0].pubkey.as_hex_string()]),
    ];
    pre_event.content = "Nice test.".to_owned();
    let event = user.sign_event(pre_event.clone()).unwrap();
    events.push(event);

    /*
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(2400 * 1, 0)); // 40m ago

    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(1800 * 1, 0)); // 30m ago

    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(1200 * 1, 0)); // 20m ago

    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(600 * 1, 0)); // 10m ago

    pre_event.created_at = Unixtime::now().unwrap(); // now
     */

    events
}
