use crate::error::Error;
use crate::globals::{Globals, User, GLOBALS};
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::Unixtime;
use std::time::Duration;

pub async fn empty_tags() -> Result<Outcome, Error> {
    let (id, raw_event) = Globals::make_raw_event(
        &format!("{}", Unixtime::now().0),
        "1",
        "[[],[]]",
        "",
        User::Registered1,
    );

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
