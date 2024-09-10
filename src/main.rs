#![allow(clippy::await_holding_lock)] // we aren't really parallel, doesn't matter.

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
use strum::IntoEnumIterator;

const WAIT: u64 = 2;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut args = env::args();
    let _ = args.next(); // program name

    let relay_url = match args.next() {
        Some(u) => u,
        None => return usage(),
    };

    let private_key = match args.next() {
        Some(s) => PrivateKey::try_from_bech32_string(&s)?,
        None => return usage(),
    };

    // post-static init of global variables
    Globals::init(relay_url, private_key).await?;

    // deadlock detection thread
    {
        use parking_lot::deadlock;
        use std::thread;
        use std::time::Duration;

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(10));
            let deadlocks = deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            println!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                println!("Deadlock #{}", i);
                for t in threads {
                    println!("Threadn Id {:#?}", t.thread_id());
                    println!("{:#?}", t.backtrace());
                }
            }
        });
    }

    // Run the tests in stages
    for stage in Stage::iter() {
        eprintln!("-----------------------------------------------------");
        eprintln!(
            "*** Stage: {} ***",
            format!("{:?}", stage).color(Color::Green3a)
        );
        stage.init().await?;
        for test_item in TestItem::iter() {
            if test_item.stage() == stage {
                eprintln!("  * TEST: {}", test_item.name());

                let outcome = if stage == Stage::Unknown {
                    Outcome::err("Test has not been assigned to a stage yet.".to_owned())
                } else {
                    test_item.run().await
                };

                GLOBALS.test_results.write().insert(test_item, outcome);
            }
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
    eprintln!("====================================================");
    println!("SUMMARY RESULTS\n");
    for (test_item, outcome) in GLOBALS.test_results.read().iter() {
        // Don't print the tests that are not yet implemented
        if let Some(s) = &outcome.info {
            if s.contains("NOT YET IMPLEMENTED") {
                continue;
            }
        }

        println!(
            "{}: {}",
            test_item.name(),
            outcome.display(test_item.required())
        );
    }

    Ok(())
}

fn usage() -> Result<(), Error> {
    eprintln!(
        "{}: relay-tester <relay_url> <allowed_nsec>",
        "Usage".color(Color::Gold1)
    );
    Ok(())
}
