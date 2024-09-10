use super::tags;
use crate::error::Error;
use crate::globals::{EventParts, Globals, GLOBALS};
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::{EventKind, Unixtime};
use std::ops::{Add, Sub};
use std::time::Duration;

pub async fn one_week_ago() -> Result<Outcome, Error> {
    doit(minutes_ago(60 * 24 * 7)).await
}

pub async fn one_month_ago() -> Result<Outcome, Error> {
    doit(minutes_ago(60 * 24 * 7 * 4)).await
}

pub async fn one_year_ago() -> Result<Outcome, Error> {
    doit(minutes_ago(60 * 24 * 365)).await
}

pub async fn before_nostr() -> Result<Outcome, Error> {
    // 2015, Thursday, January 1, 2015 12:01:01 AM GMT
    doit(Unixtime(1420070461)).await
}

pub async fn before_2000() -> Result<Outcome, Error> {
    // 1999, Friday, January 1, 1999 12:01:01 AM
    doit(Unixtime(915148861)).await
}

pub async fn from_1970() -> Result<Outcome, Error> {
    // 1970, Thursday, January 1, 1970 12:00:00 AM
    doit(Unixtime(0)).await
}

pub async fn before_1970() -> Result<Outcome, Error> {
    // sometime in 1969, negative date
    doit_raw("-200").await
}

pub async fn one_year_hence() -> Result<Outcome, Error> {
    doit(Unixtime::now().add(Duration::new(86400 * 365, 0))).await
}

pub async fn distant_future() -> Result<Outcome, Error> {
    doit(Unixtime(i64::MAX)).await
}

pub async fn greater_than_signed_32bit() -> Result<Outcome, Error> {
    // 2^31 + 1
    doit_raw("2147483649").await
}

pub async fn greater_than_unsigned_32bit() -> Result<Outcome, Error> {
    // 2^32 + 1
    doit_raw("4294967297").await
}

pub async fn scientific_notation() -> Result<Outcome, Error> {
    doit_raw("1e+10").await
}

fn minutes_ago(m: u64) -> Unixtime {
    Unixtime::now().sub(Duration::new(m * 60, 0))
}

async fn doit(u: Unixtime) -> Result<Outcome, Error> {
    let event = Globals::make_event(
        EventParts::Dated(EventKind::TextNote, tags(&[]), "".to_owned(), u),
        true,
    )?;

    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_event(event, Duration::from_secs(WAIT))
        .await?;

    if ok {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(Some(reason)))
    }
}

async fn doit_raw(date: &str) -> Result<Outcome, Error> {
    let (id, raw_event) = Globals::make_raw_event(date, "1", "[]", "", true);

    let (ok, reason) = GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .post_raw_event(id, raw_event, Duration::from_secs(WAIT))
        .await?;

    if ok {
        Ok(Outcome::pass(None))
    } else {
        Ok(Outcome::fail(Some(reason)))
    }
}
