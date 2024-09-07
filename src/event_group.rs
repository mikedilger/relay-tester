use crate::error::Error;
use crate::globals::{EventParts, Globals, GLOBALS};
use crate::WAIT;
use nostr_types::Event;
use std::collections::HashMap;
use std::slice::Iter;
use std::time::Duration;

pub struct EventGroup {
    // Events, and whether they can_read_back
    vec: Vec<(Event, bool)>,

    // Map from event name to index in vec
    keys: HashMap<&'static str, usize>,
}

impl EventGroup {
    pub fn new() -> EventGroup {
        EventGroup {
            vec: Vec::new(),
            keys: HashMap::new(),
        }
    }

    pub async fn insert(
        &mut self,
        key: &'static str,
        parts: EventParts,
        can_read_back: bool,
    ) -> Result<(), Error> {
        // Build the event (true = registered user)
        let event = Globals::make_event(parts.clone(), true)?;

        // Submit to the relay
        let (_ok, _reason) = GLOBALS
            .connection
            .write()
            .as_mut()
            .unwrap()
            .post_event(event.clone(), Duration::from_secs(WAIT))
            .await?;

        // Insert into the event group
        let index = self.vec.len();
        self.vec.push((event, can_read_back));

        // Insert into the keys
        self.keys.insert(key, index);

        Ok(())
    }

    pub fn get(&self, key: &'static str) -> Option<(Event, bool)> {
        self.keys.get(key).map(|i| self.vec[*i].clone())
    }

    pub fn iter(&self) -> Iter<'_, (Event, bool)> {
        self.vec.iter()
    }
}
