use super::tags;
use crate::connection::AuthState;
use crate::error::Error;
use crate::globals::{EventParts, Globals, User, GLOBALS};
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::EventKind;
use std::time::Duration;

pub async fn prompts_for_auth_initially() -> Result<Outcome, Error> {
    // Wait for AUTH message first
    // NOTE: auth_state will be internally updated during the wait
    {
        let mut con = GLOBALS.connection.write();
        con.as_mut().unwrap().disconnect().await?;
        con.as_mut().unwrap().reconnect().await?;
        let _ = con
            .as_mut()
            .unwrap()
            .wait_for_message(Duration::from_secs(WAIT))
            .await?;
    }

    let outcome = match &GLOBALS.connection.read().as_ref().unwrap().auth_state {
        AuthState::NotYetRequested => Outcome::fail(Some("Did not prompt us for AUTH".to_owned())),
        AuthState::Challenged(_) => Outcome::pass(None),
        s => Outcome::fail(Some(format!(
            "INTERNAL ERROR auth state beyond reasonable: {:?}",
            s
        ))),
    };

    Ok(outcome)
}

pub async fn can_auth_as_unknown() -> Result<Outcome, Error> {
    // Restart the connection
    {
        let mut con = GLOBALS.connection.write();
        con.as_mut().unwrap().disconnect().await?;
        con.as_mut().unwrap().reconnect().await?;
    }

    // Try to post something (to trigger AUTH if it isn't automatic)
    // but ignore any result/error
    {
        let event = Globals::make_event(
            EventParts::Basic(EventKind::TextNote, tags(&[&["test"]]), "".to_string()),
            User::Stranger,
        )?;
        let _ = GLOBALS
            .connection
            .write()
            .as_mut()
            .unwrap()
            .post_event(event, Duration::from_secs(WAIT))
            .await?;
    }

    // Reply to the AUTH challenge with the Stranger
    GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .authenticate_if_challenged(User::Stranger)
        .await?;

    match &GLOBALS.connection.read().as_ref().unwrap().auth_state {
        AuthState::Success => Ok(Outcome::pass(None)),
        AuthState::Failure(s) => Ok(Outcome::fail(Some(s.to_owned()))),
        AuthState::InProgress(_) => Ok(Outcome::fail(Some("Did not complete AUTH".to_owned()))),
        _ => Ok(Outcome::err("Unexpected auth state".to_owned())),
    }
}
