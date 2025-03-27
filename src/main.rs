#![allow(clippy::await_holding_lock)] // we aren't really parallel, doesn't matter.

macro_rules! log {
    ($($arg:tt)*) => {{
        if ! $crate::globals::GLOBALS.script_mode.load(std::sync::atomic::Ordering::Relaxed) {
            std::eprintln!($($arg)*);
        }
    }};
}

mod connection;
mod error;
mod event_group;
mod globals;
mod outcome;
mod stage;
mod test_item;
mod tests;

use crate::error::Error;
use crate::globals::{Globals, GLOBALS};
use crate::outcome::Outcome;
use crate::stage::Stage;
use crate::test_item::TestItem;
use colorful::{Color, Colorful};
use nostr_types::PrivateKey;
use std::env;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use strum::IntoEnumIterator;

const WAIT: u64 = 2;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Install crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let mut args = env::args();
    let _ = args.next(); // program name

    let mut relay_url_opt: Option<String> = None;
    let mut private_key1_opt: Option<String> = None;
    let mut private_key2_opt: Option<String> = None;
    loop {
        if let Some(a) = args.next() {
            if a.starts_with("--") {
                match &*a {
                    "--script" => GLOBALS.script_mode.store(true, Ordering::Relaxed),
                    _ => return usage(),
                }
            } else if relay_url_opt.is_none() {
                relay_url_opt = Some(a);
            } else if private_key1_opt.is_none() {
                private_key1_opt = Some(a);
            } else if private_key2_opt.is_none() {
                private_key2_opt = Some(a);
            } else {
                return usage();
            }
        } else {
            break;
        }
    }

    let relay_url = match relay_url_opt {
        Some(u) => u,
        None => return usage(),
    };

    let private_key1 = match private_key1_opt {
        Some(s) => PrivateKey::try_from_bech32_string(&s)?,
        None => return usage(),
    };

    let private_key2 = match private_key2_opt {
        Some(s) => PrivateKey::try_from_bech32_string(&s)?,
        None => return usage(),
    };

    // post-static init of global variables
    Globals::init(relay_url, private_key1, private_key2).await?;

    // deadlock detection thread
    {
        use parking_lot::deadlock;

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(10));
            let deadlocks = deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            log!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                log!("Deadlock #{}", i);
                for t in threads {
                    log!("Threadn Id {:#?}", t.thread_id());
                    log!("{:#?}", t.backtrace());
                }
            }
        });
    }

    // Run the tests in stages
    for stage in Stage::iter() {
        log!("-----------------------------------------------------");
        log!(
            "*** Stage: {} ***",
            format!("{:?}", stage).color(Color::Green3a)
        );
        stage.init().await?;

        let mut old_next_sub_id = GLOBALS
            .connection
            .read()
            .as_ref()
            .unwrap()
            .next_sub_id
            .load(Ordering::Relaxed);

        for test_item in TestItem::iter() {
            if test_item.stage() == stage {
                log!("\n--* TEST: {} *--------", test_item.name());

                let mut outcome = if stage == Stage::Unknown {
                    Outcome::err("Test has not been assigned to a stage yet.".to_owned())
                } else {
                    test_item.run().await
                };

                let new_next_sub_id = GLOBALS
                    .connection
                    .read()
                    .as_ref()
                    .unwrap()
                    .next_sub_id
                    .load(Ordering::Relaxed);

                // old=5, new=7:   answer=(5,6)
                for i in old_next_sub_id..new_next_sub_id {
                    outcome.subs.push(i);
                }

                GLOBALS.test_results.write().insert(test_item, outcome);

                old_next_sub_id = new_next_sub_id;
            }

            thread::sleep(Duration::new(0, 100));
        }
    }

    GLOBALS
        .connection
        .write()
        .as_mut()
        .unwrap()
        .disconnect()
        .await?;

    // Display the results
    log!("====================================================");
    log!("SUMMARY RESULTS\n");

    let mut not_implemented: usize = 0;
    let mut untested: usize = 0;
    let mut fail: usize = 0;
    let mut total: usize = 0;

    for (test_item, outcome) in GLOBALS.test_results.read().iter() {
        total += 1;

        // Don't print the tests that are not yet implemented
        if let Some(s) = &outcome.info {
            if s.contains("NOT YET IMPLEMENTED") {
                not_implemented += 1;
                continue;
            }
        }

        if test_item.required() && matches!(outcome.pass, Some(false)) {
            fail += 1;
        }

        if outcome.pass.is_none() {
            untested += 1;
        }

        log!(
            "{}: {}",
            test_item.name(),
            outcome.display(test_item.required())
        );

        if GLOBALS.script_mode.load(Ordering::Relaxed) {
            let value = serde_json::json!({
                "test": test_item.name(),
                "required": test_item.required(),
                "pass": outcome.pass,
                "info": outcome.info
            });
            println!("{}", value);
        }
    }

    log!(
        "FAIL: {}, UNTESTED: {}, NOT_IMPLEMENTED: {}, TOTAL: {}",
        fail,
        untested,
        not_implemented,
        total
    );

    Ok(())
}

fn usage() -> Result<(), Error> {
    log!(
        "{}: relay-tester <relay_url> <allowed_nsec1> <allowed_nsec2>",
        "Usage".color(Color::Gold1)
    );
    Ok(())
}
