use super::maybe_submit_event_group_a;
use crate::error::Error;
use crate::globals::GLOBALS;
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::{Event, EventKind, Filter, IdHex, PublicKeyHex, Signer, Unixtime};
use std::time::Duration;

pub async fn newest_to_oldest() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    // Collect all the Ids from Event Group A
    let ids: Vec<IdHex> = GLOBALS
        .event_group_a
        .read()
        .iter()
        .map(|rm| rm.0.id.into())
        .collect();

    // Filter to read them all back
    let filter = {
        let mut filter = Filter::new();
        filter.ids = ids;
        filter
    };

    let events = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?
        .into_events();

    if events.len() < 3 {
        Ok(Outcome::fail(Some(
            "Could not fetch enough events to test".to_owned(),
        )))
    } else {
        let mut last = Unixtime(i64::MAX);
        for event in events.iter() {
            if event.created_at <= last {
                last = event.created_at;
            } else {
                return Ok(Outcome::fail(Some("Order is wrong".to_owned())));
            }
        }
        Ok(Outcome::pass(None))
    }
}

pub async fn newest_events_when_limited() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    let registered_public_key = GLOBALS.registered_user.read().public_key();

    let filter = {
        let mut filter = Filter::new();
        filter.authors = vec![registered_public_key.into()];
        filter.add_tag_value('t', "a".to_string());
        filter.add_tag_value('t', "b".to_string());
        filter.kinds = vec![EventKind::TextNote, EventKind::Reaction];
        filter.limit = Some(2);
        filter
    };

    let events = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter], Duration::from_secs(WAIT))
        .await?
        .into_events();

    let limit_test_first_id = GLOBALS
        .event_group_a
        .read()
        .get("limit_test_first")
        .unwrap()
        .0
        .id;
    let limit_test_second_id = GLOBALS
        .event_group_a
        .read()
        .get("limit_test_second")
        .unwrap()
        .0
        .id;

    if events.len() != 2 {
        Ok(Outcome::fail(Some(format!(
            "Got {} events, expected 2",
            events.len()
        ))))
    } else if events[0].id != limit_test_first_id || events[1].id != limit_test_second_id {
        Ok(Outcome::fail(None))
    } else {
        Ok(Outcome::pass(None))
    }
}

pub async fn find_by_id() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    // Collect all the Ids from Event Group A that are findable
    let ids: Vec<IdHex> = GLOBALS
        .event_group_a
        .read()
        .iter()
        .filter(|v| v.1)
        .map(|rm| rm.0.id.into())
        .collect();

    let num = ids.len();

    let filter = {
        let mut filter = Filter::new();
        filter.ids = ids;
        filter
    };

    find(filter, Some(num)).await
}

pub async fn find_by_pubkey_and_kind() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    let registered_public_key = GLOBALS.registered_user.read().public_key();
    let stranger_public_key = GLOBALS.stranger.read().public_key();

    let filter = {
        let mut filter = Filter::new();
        let pkh1: PublicKeyHex = registered_public_key.into();
        let pkh2: PublicKeyHex = stranger_public_key.into();
        filter.authors = vec![pkh1, pkh2];
        filter.kinds = vec![EventKind::TextNote, EventKind::ContactList];
        filter
    };

    find(filter, None).await
}

pub async fn find_by_pubkey_and_tags() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    let registered_public_key = GLOBALS.registered_user.read().public_key();

    let filter = {
        let mut filter = Filter::new();
        let pkh: PublicKeyHex = registered_public_key.into();
        filter.add_author(&pkh);
        filter.add_tag_value('p', pkh.to_string());
        filter
    };

    find(filter, None).await
}

pub async fn find_by_kind_and_tags() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    let filter = {
        let mut filter = Filter::new();
        filter.kinds = vec![
            EventKind::TextNote,
            EventKind::Other(40383),
            EventKind::ContactList,
        ];
        filter.add_tag_value('n', "approved".to_string());
        filter
    };

    find(filter, None).await
}

pub async fn find_by_tags() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    let filter = {
        let mut filter = Filter::new();
        filter.add_tag_value('k', "3036".to_string());
        filter
    };

    find(filter, None).await
}

pub async fn find_by_multiple_tags() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    let registered_public_key = GLOBALS.registered_user.read().public_key();

    let filter = {
        let mut filter = Filter::new();
        let pkh: PublicKeyHex = registered_public_key.into();
        filter.add_event_kind(EventKind::Other(40383));
        filter.add_author(&pkh);
        filter.add_tag_value('k', "3036".to_string());
        filter.add_tag_value('n', "approved".to_string());
        filter.limit = Some(20);
        filter
    };

    find(filter, None).await
}

pub async fn find_by_pubkey() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    let registered_public_key = GLOBALS.registered_user.read().public_key();

    let filter = {
        let mut filter = Filter::new();
        let pkh: PublicKeyHex = registered_public_key.into();
        filter.add_author(&pkh);
        filter
    };

    find(filter, None).await
}

pub async fn find_by_scrape() -> Result<Outcome, Error> {
    maybe_submit_event_group_a().await?;

    let filter = Filter::new();

    find(filter, None).await
}

async fn find(filter: Filter, num_matches_expected: Option<usize>) -> Result<Outcome, Error> {
    let findable: Vec<Event> = GLOBALS
        .event_group_a
        .read()
        .iter()
        .filter(|v| v.1)
        .map(|v| v.0.clone())
        .collect();

    let fresult = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .fetch_events(vec![filter.clone()], Duration::from_secs(WAIT))
        .await?;
    let maybe_error = fresult.close_msg.clone();
    let fetched = fresult.into_events();

    // Verify all fetched events match the filter
    for event in fetched.iter() {
        if !filter.event_matches(event) {
            return Ok(Outcome::fail(Some(
                "Returned an event that does not match the filter".to_owned(),
            )));
        }
    }

    // Verify all findable events that also match the filter were fetched,
    // and count the matches as we go
    let mut matches: usize = 0;
    for findable_event in findable {
        if filter.event_matches(&findable_event) {
            if !fetched.iter().any(|e| e.id == findable_event.id) {
                if let Some(e) = maybe_error {
                    return Ok(Outcome::fail(Some(format!(
                        "Expected event is missing: {}",
                        e
                    ))));
                } else {
                    return Ok(Outcome::fail(Some(format!("Expected event is missing: {}", findable_event.id.as_hex_string()))));
                }
            }
            matches += 1;
        }
    }

    // Check that we got the expected number of matches
    if let Some(expected) = num_matches_expected {
        if matches != expected {
            if let Some(e) = maybe_error {
                Ok(Outcome::fail(Some(format!(
                    "matched {} events but expected {}: {}",
                    matches, expected, e
                ))))
            } else {
                Ok(Outcome::fail(Some(format!(
                    "matched {} events but expected {}",
                    matches, expected
                ))))
            }
        } else {
            Ok(Outcome::pass(Some(format!(
                "matched {} events, as expected",
                matches
            ))))
        }
    } else {
        Ok(Outcome::pass(None))
    }
}
