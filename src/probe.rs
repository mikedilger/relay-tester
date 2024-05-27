use nostr_types::{Event, Filter, SubscriptionId};

/// These are things we can ask the relay probe to do.
/// Mostly they become messages to the relay.
#[derive(Debug)]
pub enum Command {
    Auth(Event),
    PostEvent(Event),
    FetchEvents(SubscriptionId, Vec<Filter>),
    Exit,
}
