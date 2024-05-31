use colorful::{Color, Colorful};
use lazy_static::lazy_static;
use std::fmt;
use std::sync::RwLock;

#[derive(Debug, Default, Clone)]
pub enum Outcome {
    #[default]
    Untested,
    Pass,
    Yes,
    //No,
    No2(String),
    Fail,
    Fail2(String),
    Info(String),
    //Value(usize),
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Untested => write!(f, "{}", "UNTESTED".color(Color::Grey50)),
            Outcome::Pass => write!(f, "{}", "PASS".color(Color::Green)),
            Outcome::Yes => write!(f, "{}", "YES".color(Color::Green)),
            //Outcome::No => write!(f, "{}", NO".color(Color::Orange)),
            Outcome::No2(s) => write!(f, "{} ({})", "NO".color(Color::DarkGoldenrod), s),
            Outcome::Fail => write!(f, "{}", "FAIL".color(Color::Red3a)),
            Outcome::Fail2(s) => write!(f, "{} ({})", "FAIL".color(Color::Red3a), s),
            Outcome::Info(s) => write!(f, "{} ({})", "INFO".color(Color::DarkGoldenrod), s),
            //Outcome::Value(u) => write!(f, "{} ({})", "VALUE", u),
        }
    }
}

pub const NUMTESTS: usize = 97;
pub const TESTNAMES: [&str; 97] = [
    "nip11_provided",
    "claimed_support_for_nip4",
    "claimed_support_for_nip9",
    "claimed_support_for_nip11",
    "claimed_support_for_nip26",
    "claimed_support_for_nip29",
    "claimed_support_for_nip40",
    "claimed_support_for_nip42",
    "claimed_support_for_nip45",
    "claimed_support_for_nip50",
    "claimed_support_for_nip59",
    "claimed_support_for_nip65",
    "claimed_support_for_nip70",
    "claimed_support_for_nip94",
    "claimed_support_for_nip96",
    // Public permission checks
    "public_can_write",
    "public_can_read_back",
    // NIP-01
    "supports_eose",
    "find_by_id",
    "find_by_pubkey_and_kind",
    "find_by_pubkey_and_tags",
    "find_by_kind_and_tags",
    "find_by_tags",
    "find_by_pubkey",
    "find_by_scrape",
    "find_replaceable_event",
    "find_parameterized_replaceable_event",
    "replaceable_event_removes_previous",
    "replaceable_event_doesnt_remove_future",
    "parameterized_replaceable_event_removes_previous",
    "parameterized_replaceable_event_doesnt_remove_future",
    "since_until_include_equals",
    "limit_zero",
    "event_always_gets_ok_reply",
    "auth_always_gets_ok_reply",
    "limit_works_across_multiple_filter_groups",
    "serves_post_eose_events",
    "no_timeout_while_subscribed",
    // NIP-04
    "nip4_dms_require_auth",
    // NIP-09
    "delete_by_id",
    "delete_by_id_of_others",
    "resubmission_of_deleted_by id",
    "delete_by_npnaddr",
    "delete_by_npnaddr_of_others",
    "delete_by_npnaddr_preserves_newer",
    "resubmission_of_deleted_by_npnaddr",
    "resubmission_of_olderthan_deleted_by_npnaddr",
    "resubmission_of_newerthan_deleted_by_npnaddr",
    "delete_by_pnaddr",
    "delete_by_pnaddr_of_others",
    "delete_by_pnaddr_preserves_newer",
    "delete_by_pnaddr_bound_by_dtag",
    "resubmission_of_deleted_by_pnaddr",
    "resubmission_of_olderthan_deleted_by_pnaddr",
    "resubmission_of_newerthan_deleted_by_pnaddr",
    "deleted_returns_ok_false",
    // NIP-26 - TBD

    // NIP-29 - TBD

    // NIP-40 - TBD

    // NIP-42 (and auth permission checks)
    "prompts_for_auth_initially",
    "can_auth_as_unknown",
    "unknown_can_write_own",
    "unknown_can_readback_own",
    "unknown_can_write_other",
    "unknown_can_readback_other",
    "can_auth_as_known",
    "known_can_write_own",
    "known_can_readback_own",
    "known_can_write_other",
    "known_can_readback_other",
    // NIP-45 - TBD

    // NIP-50 - TBD

    // NIP-59
    "giftwraps_require_auth",
    // NIP-65 - TBD

    // NIP-70 - TBD (protected event)

    // NIP-94 - TBD

    // NIP-96 - TBD
    // other
    "large_contact_lists",
    "preserves_json_field_order",
    "preserves_nonstandard_json_fields",
    "handles_event_kind_larger_than_16bit",
    "handles_filter_kind_larger_than_16bit",
    // created_at limits
    "accepts_events_one_week_old",
    "accepts_events_one_month_old",
    "accepts_events_one_year_old",
    "accepts_events_from_before_nostr",
    "accepts_events_from_before_2000",
    "accepts_events_from_1970",
    "accepts_events_from_before_1970",
    "accepts_events_one_year_into_the_future",
    "accepts_events_in_the_distant_future",
    "accepts_events_with_created_at_greater_than_signed32bit",
    "accepts_events_with_created_at_greater_than_unsigned32bit",
    "accepts_events_with_created_at_in_scientific_notation",
    "accepts_negative_filter_created_at",
    "handles_all_json_escape_codes",
    "handles_surrogate_pairs",
    "verifies_signatures",
    "accepts_invalid_utf8",
    "accepts_null_characters",
    "handles_filter_prefixes",
    "keeps_ephemeral_events",
    "max_subscriptions",
    "allows_immediate_reconnect",
    "idle_timeout_if_unsubscribed",
    "handles_empty_tags",
];

pub fn set_outcome_by_name(name: &'static str, outcome: Outcome) {
    let no = test_no(name);
    (*(*RESULTS).write().unwrap())[no] = outcome;
}

pub fn test_no(name: &'static str) -> usize {
    for (i, thisname) in TESTNAMES.iter().enumerate() {
        if *thisname == name {
            return i;
        }
    }

    panic!("Test \"{}\" was not found", name);
}

lazy_static! {
    pub static ref RESULTS: RwLock<Vec<Outcome>> = {
        let mut v = Vec::with_capacity(NUMTESTS);
        v.resize(NUMTESTS, Outcome::Untested);
        RwLock::new(v)
    };
}
