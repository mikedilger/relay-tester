mod error;
mod probe;
mod results;
mod runner;

use crate::error::Error;
use crate::results::{TestDef, NUMTESTS, RESULTS, TESTDEFS};
use crate::runner::Runner;
use colorful::{Color, Colorful};
use lazy_static::lazy_static;
use nostr_types::PrivateKey;
use std::env;

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

    let mut runner = Runner::new(relay_url, private_key);

    runner.run().await?;

    runner.exit().await?;

    println!("\nRESULTS:");
    let results = &(*(*RESULTS).read().unwrap());
    for i in 0..NUMTESTS {
        let testdef = TestDef {
            required: TESTDEFS[i].0,
            name: TESTDEFS[i].1,
            outcome: results[i].clone(),
        };
        println!("{}", testdef);
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

// Colorful prefixes for terminal output
pub struct Prefixes {
    from_relay: String,
    sending: String,
}
lazy_static! {
    pub static ref PREFIXES: Prefixes = Prefixes {
        from_relay: "Relay".color(Color::Blue).to_string(),
        sending: "Sending".color(Color::MediumPurple).to_string(),
    };
}
