use nostr_types::{Event, EventKind, PreEvent, Signer, Tag, Unixtime};
use std::ops::Sub;
use std::time::Duration;

pub fn injected_events(registered_user: &dyn Signer) -> Vec<Event> {

    // NOTE: all injected events are the registered user, because a stranger may not
    //       have the permissions to inject events.

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

    // Here we create FOUR events in TWO groups of TWO, and request a LIMIT 2 on them
    //
    // 1: kind 1 hashtag "a"  40 minutes ago
    // 2: kind 1 hashtag "b"  50 minutes ago
    // 3: kind 7 hashtag "b"  45 minutes ago
    // 4: kind 7 hashtag "a"  55 minutes ago
    //
    // We should get [1] and then [3] in that order
    //
    // Event 1:
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(2400 * 1, 0)); // 40m ago
    pre_event.kind = EventKind::TextNote;
    pre_event.tags = vec![
        Tag::from_strings(vec!["t".to_owned(), "a".to_owned()]),
    ];
    let event = registered_user.sign_event(pre_event.clone()).unwrap();
    events.push(event);
    //
    // Event 2:
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(3000 * 1, 0)); // 50m ago
    pre_event.kind = EventKind::TextNote;
    pre_event.tags = vec![
        Tag::from_strings(vec!["t".to_owned(), "b".to_owned()]),
    ];
    let event = registered_user.sign_event(pre_event.clone()).unwrap();
    events.push(event);
    //
    // Event 3:
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(2700 * 1, 0)); // 45m ago
    pre_event.kind = EventKind::Reaction;
    pre_event.content = "+".to_string();
    pre_event.tags = vec![
        Tag::from_strings(vec!["t".to_owned(), "b".to_owned()]),
    ];
    let event = registered_user.sign_event(pre_event.clone()).unwrap();
    events.push(event);
    //
    // Event 4:
    pre_event.created_at = Unixtime::now().unwrap().sub(Duration::new(3300 * 1, 0)); // 55m ago
    pre_event.kind = EventKind::Reaction;
    pre_event.tags = vec![
        Tag::from_strings(vec!["t".to_owned(), "a".to_owned()]),
    ];
    let event = registered_user.sign_event(pre_event.clone()).unwrap();
    events.push(event);


    // 1000-10000:          regular
    // 10000-19999, 0, 3:   replaceable (pubkey+kind, MAYBE only the latest is available SHOULD only return latest)
    //    test 0 - metadata, 3 - contact list, 10002 relay list metadata
    // 20000-29999:         ephemeral   (MAY not persist)
    //    test a made up number like 21212
    // 30000-39000:         parameterized replaceable  (pubkey+kind+d  MAYBE only the latest is available SHOULD only return latest,
    //                         but may keep others accessible by id)
    //    test follow sets 30000

    events
}
