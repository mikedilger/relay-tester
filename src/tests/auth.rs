use crate::connection::AuthState;
use crate::error::Error;
use crate::globals::GLOBALS;
use crate::outcome::Outcome;
use crate::WAIT;
use std::time::Duration;

pub async fn prompts_for_auth_initially() -> Result<Outcome, Error> {
    // Wait for AUTH message first
    // NOTE: auth_state will be internally updated during the wait
    {
        let mut con = GLOBALS.connection.write();
        //con.as_mut().unwrap().disconnect().await?;
        //con.as_mut().unwrap().reconnect().await?;
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
