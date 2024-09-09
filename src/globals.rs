use crate::connection::Connection;
use crate::error::Error;
use crate::event_group::EventGroup;
use crate::outcome::Outcome;
use crate::test_item::TestItem;
use colorful::{Color, Colorful};
use lazy_static::lazy_static;
use nostr_types::{Event, EventKind, Id, KeySigner, PreEvent, PrivateKey, Signer, Tag, Unixtime};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use strum::IntoEnumIterator;

lazy_static! {
    pub static ref GLOBALS: Globals = Globals::new();
}

pub struct Globals {
    pub relay_url: Arc<RwLock<String>>,
    pub connection: Arc<RwLock<Option<Connection>>>,
    pub disconnected: AtomicBool,
    pub stranger: Arc<RwLock<KeySigner>>,
    pub registered_user: Arc<RwLock<KeySigner>>,
    pub test_results: Arc<RwLock<BTreeMap<TestItem, Outcome>>>,
    pub nip11: Arc<RwLock<Option<serde_json::Value>>>,
    pub saw_ok_after_event: AtomicBool,
    pub event_group_a: Arc<RwLock<EventGroup>>,
    pub event_group_a_submitted: AtomicBool,
    pub event_group_a_failed: AtomicBool,
}

impl Globals {
    fn new() -> Globals {
        let mut test_results = BTreeMap::new();
        for test_item in TestItem::iter() {
            test_results.insert(test_item, Default::default());
        }

        Globals {
            relay_url: Arc::new(RwLock::new("".to_owned())),
            connection: Arc::new(RwLock::new(None)),
            disconnected: AtomicBool::new(false),
            stranger: Arc::new(RwLock::new(KeySigner::generate("stranger", 2).unwrap())),
            registered_user: Arc::new(RwLock::new(KeySigner::generate("fixme", 2).unwrap())),
            test_results: Arc::new(RwLock::new(test_results)),
            nip11: Arc::new(RwLock::new(None)),
            saw_ok_after_event: AtomicBool::new(false),
            event_group_a: Arc::new(RwLock::new(EventGroup::new())),
            event_group_a_submitted: AtomicBool::new(false),
            event_group_a_failed: AtomicBool::new(false),
        }
    }

    pub async fn init(relay_url: String, private_key: PrivateKey) -> Result<(), Error> {
        *GLOBALS.relay_url.write() = relay_url;
        *GLOBALS.registered_user.write() = KeySigner::from_private_key(private_key, "", 8).unwrap();
        eprintln!("{}", "*** CONNECTING ***".color(Color::Red));
        let relay_url = GLOBALS.relay_url.read().clone();
        let connection = Connection::new(relay_url).await?;
        *GLOBALS.connection.write() = Some(connection);
        Ok(())
    }

    pub fn make_event(parts: EventParts, registered: bool) -> Result<Event, Error> {
        let (kind, tags, content, created_at) = match parts {
            EventParts::Basic(k, t, c) => (k, t, c, Unixtime::now()),
            EventParts::Dated(k, t, c, d) => (k, t, c, d),
        };

        let user = if registered {
            &GLOBALS.registered_user.read()
        } else {
            &GLOBALS.stranger.read()
        };

        let pre_event = PreEvent {
            pubkey: user.public_key(),
            created_at,
            kind,
            tags,
            content,
        };

        Ok(user.sign_event(pre_event)?)
    }

    pub fn make_raw_event(
        created_at: &str,
        kind: &str,
        tags: &str,
        content: &str,
        registered: bool,
    ) -> (Id, String) {
        let public_key_hex = if registered {
            GLOBALS.registered_user.read().public_key().as_hex_string()
        } else {
            GLOBALS.stranger.read().public_key().as_hex_string()
        };

        let serial_for_sig = format!(
            "[0,\"{}\",{},{},{},\"{}\"]",
            &public_key_hex, created_at, kind, tags, content
        );
        use secp256k1::hashes::Hash;
        let hash = secp256k1::hashes::sha256::Hash::hash(serial_for_sig.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();
        let id = Id(id);
        let signature = GLOBALS.registered_user.read().sign_id(id).unwrap();

        let raw_event = format!(
            r##"{{"id":"{}","pubkey":"{}","created_at":{},"kind":{},"tags":{},"content":"{}","sig":"{}"}}"##,
            id.as_hex_string(),
            &public_key_hex,
            created_at,
            kind,
            tags,
            content,
            signature.as_hex_string()
        );

        (id, raw_event)
    }
}

#[derive(Debug, Clone)]
pub enum EventParts {
    Basic(EventKind, Vec<Tag>, String),
    Dated(EventKind, Vec<Tag>, String, Unixtime),
}