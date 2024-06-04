use colorful::{Color, Colorful};
use lazy_static::lazy_static;
use std::fmt;
use std::sync::RwLock;

#[derive(Debug, Default, Clone)]
pub struct Outcome {
    pub pass: Option<bool>,
    pub info: Option<String>,
}

impl Outcome {
    pub fn new(pass: bool, info: Option<String>) -> Outcome {
        Outcome {
            pass: Some(pass),
            info,
        }
    }

    pub fn err(info: String) -> Outcome {
        Outcome {
            pass: None,
            info: Some(info),
        }
    }
}

pub struct TestDef {
    pub required: bool,
    pub name: &'static str,
    pub outcome: Outcome,
}

impl fmt::Display for TestDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.name)?;
        match self.required {
            false => match self.outcome.pass {
                None => match self.outcome.info {
                    None => write!(f, "{}", "UNTESTED".color(Color::Grey50)),
                    Some(ref s) => write!(f, "{} ({})", "UNTESTED".color(Color::Grey50), s),
                },
                Some(false) => match self.outcome.info {
                    None => write!(f, "{}", "NO".color(Color::DarkGoldenrod)),
                    Some(ref s) => write!(f, "{} ({})", "NO".color(Color::DarkGoldenrod), s),
                },
                Some(true) => match self.outcome.info {
                    None => write!(f, "{}", "YES".color(Color::Green)),
                    Some(ref s) => write!(f, "{} ({})", "YES".color(Color::Green), s),
                },
            },
            true => match self.outcome.pass {
                None => match self.outcome.info {
                    None => write!(f, "{}", "UNTESTED".color(Color::Grey50)),
                    Some(ref s) => write!(f, "{} ({})", "UNTESTED".color(Color::Grey50), s),
                },
                Some(false) => match self.outcome.info {
                    None => write!(f, "{}", "FAIL".color(Color::Red3a)),
                    Some(ref s) => write!(f, "{} ({})", "FAIL".color(Color::Red3a), s),
                },
                Some(true) => match self.outcome.info {
                    None => write!(f, "{}", "PASS".color(Color::Green)),
                    Some(ref s) => write!(f, "{} ({})", "PASS".color(Color::Green), s),
                },
            },
        }
    }
}

pub const NUMTESTS: usize = 96;
pub const TESTDEFS: [(bool, &str); 96] = [
    (false, "nip11_provided"),
    (false, "claimed_support_for_nip4"),
    (false, "claimed_support_for_nip9"),
    (false, "claimed_support_for_nip11"),
    (false, "claimed_support_for_nip26"),
    (false, "claimed_support_for_nip29"),
    (false, "claimed_support_for_nip40"),
    (false, "claimed_support_for_nip42"),
    (false, "claimed_support_for_nip45"),
    (false, "claimed_support_for_nip50"),
    (false, "claimed_support_for_nip59"),
    (false, "claimed_support_for_nip65"),
    (false, "claimed_support_for_nip94"),
    (false, "claimed_support_for_nip96"),
    // Public permission checks
    (false, "public_can_write"),
    // NIP-01
    (true, "supports_eose"),
    (true, "find_by_id"),
    (true, "find_by_pubkey_and_kind"),
    (true, "find_by_pubkey_and_tags"),
    (true, "find_by_kind_and_tags"),
    (true, "find_by_tags"),
    (true, "find_by_pubkey"),
    (true, "find_by_scrape"),
    (true, "find_replaceable_event"),
    (true, "find_parameterized_replaceable_event"),
    (true, "replaceable_event_removes_previous"),
    (true, "replaceable_event_doesnt_remove_future"),
    (true, "parameterized_replaceable_event_removes_previous"),
    (true, "parameterized_replaceable_event_doesnt_remove_future"),
    (true, "since_until_include_equals"),
    (true, "limit_zero"),
    (true, "event_always_gets_ok_reply"),
    (false, "auth_always_gets_ok_reply"),
    (true, "limit_works_across_multiple_filter_groups"),
    (true, "serves_post_eose_events"),
    (true, "no_timeout_while_subscribed"),
    // NIP-04
    (false, "nip4_dms_require_auth"),
    // NIP-09
    (false, "delete_by_id"),
    (false, "delete_by_id_of_others"),
    (false, "resubmission_of_deleted_by id"),
    (false, "delete_by_npnaddr"),
    (false, "delete_by_npnaddr_of_others"),
    (false, "delete_by_npnaddr_preserves_newer"),
    (false, "resubmission_of_deleted_by_npnaddr"),
    (false, "resubmission_of_olderthan_deleted_by_npnaddr"),
    (false, "resubmission_of_newerthan_deleted_by_npnaddr"),
    (false, "delete_by_pnaddr"),
    (false, "delete_by_pnaddr_of_others"),
    (false, "delete_by_pnaddr_preserves_newer"),
    (false, "delete_by_pnaddr_bound_by_dtag"),
    (false, "resubmission_of_deleted_by_pnaddr"),
    (false, "resubmission_of_olderthan_deleted_by_pnaddr"),
    (false, "resubmission_of_newerthan_deleted_by_pnaddr"),
    (false, "deleted_returns_ok_false"),
    // NIP-26 - TBD

    // NIP-29 - TBD

    // NIP-40 - TBD

    // NIP-42 (and auth permission checks)
    (false, "prompts_for_auth_initially"),
    (false, "can_auth_as_unknown"),
    (false, "unknown_can_write_own"),
    (false, "unknown_can_readback_own"),
    (false, "unknown_can_write_other"),
    (false, "unknown_can_readback_other"),
    (false, "can_auth_as_known"),
    (false, "known_can_write_own"),
    (false, "known_can_readback_own"),
    (false, "known_can_write_other"),
    (false, "known_can_readback_other"),
    // NIP-45 - TBD

    // NIP-50 - TBD

    // NIP-59
    (false, "giftwraps_require_auth"),
    // NIP-65 - TBD

    // NIP-70 - TBD (protected event)

    // NIP-94 - TBD

    // NIP-96 - TBD
    // other
    (false, "large_contact_lists"),
    (false, "preserves_json_field_order"),
    (false, "preserves_nonstandard_json_fields"),
    (false, "handles_event_kind_larger_than_16bit"),
    (false, "handles_filter_kind_larger_than_16bit"),
    // created_at limits
    (true, "accepts_events_one_week_old"),
    (false, "accepts_events_one_month_old"),
    (false, "accepts_events_one_year_old"),
    (false, "accepts_events_from_before_nostr"),
    (false, "accepts_events_from_before_2000"),
    (false, "accepts_events_from_1970"),
    (false, "accepts_events_from_before_1970"),
    (false, "accepts_events_one_year_into_the_future"),
    (false, "accepts_events_in_the_distant_future"),
    (
        false,
        "accepts_events_with_created_at_greater_than_signed32bit",
    ),
    (
        false,
        "accepts_events_with_created_at_greater_than_unsigned32bit",
    ),
    (
        false,
        "accepts_events_with_created_at_in_scientific_notation",
    ),
    (false, "accepts_negative_filter_created_at"),
    (true, "handles_all_json_escape_codes"),
    (true, "handles_surrogate_pairs"),
    (true, "verifies_signatures"),
    (true, "verifies_id_hashes"),
    (false, "accepts_invalid_utf8"),
    (false, "accepts_null_characters"),
    (false, "handles_filter_prefixes"),
    (false, "keeps_ephemeral_events"),
    (false, "max_subscriptions"),
    (false, "allows_immediate_reconnect"),
    (false, "idle_timeout_if_unsubscribed"),
    (false, "handles_empty_tags"),
];

pub fn set_outcome_by_name(name: &'static str, outcome: Outcome) {
    let no = test_no(name);
    (*(*RESULTS).write().unwrap())[no] = outcome;
}

pub fn test_no(name: &'static str) -> usize {
    for (i, (_, thisname)) in TESTDEFS.iter().enumerate() {
        if *thisname == name {
            return i;
        }
    }

    panic!("Test \"{}\" was not found", name);
}

lazy_static! {
    pub static ref RESULTS: RwLock<Vec<Outcome>> = {
        let mut v = Vec::with_capacity(NUMTESTS);
        v.resize(
            NUMTESTS,
            Outcome {
                pass: None,
                info: None,
            },
        );
        RwLock::new(v)
    };
}
