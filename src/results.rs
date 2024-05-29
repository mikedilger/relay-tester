use colorful::{Color, Colorful};
use lazy_static::lazy_static;
use std::sync::RwLock;
use std::fmt;

#[derive(Debug, Default, Clone)]
pub enum Outcome {
    #[default]
    Untested,
    Pass,
    Fail,
    Fail2(String),
    Info(String),
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Untested => write!(f, "{}", "UNTESTED".color(Color::Grey50)),
            Outcome::Pass => write!(f, "{}", "PASS".color(Color::Green)),
            Outcome::Fail => write!(f, "{}", "FAIL".color(Color::Red)),
            Outcome::Fail2(s) => write!(f, "{} ({})", "FAIL".color(Color::Red), s),
            Outcome::Info(s) => write!(f, "{} ({})", "INFO".color(Color::Gold1), s),
        }
    }
}

pub const NUMTESTS: usize = 60;
pub const TESTNAMES: [&'static str; 60] = [
    "nip11_provided",
    "claimed_support_for_nip1",
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

    // NIP-94 - TBD

    // NIP-96 - TBD
];

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
