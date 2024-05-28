mod error;
mod probe;
mod runner;

use crate::error::Error;
use crate::probe::Probe;
use crate::runner::Runner;
use colorful::{Color, Colorful};
use nostr_types::{KeySigner, PrivateKey};
use lazy_static::lazy_static;
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

    let key_signer = KeySigner::from_private_key(private_key, "", 8)?;

    let probe = Probe::new(relay_url, Box::new(key_signer));

    let mut runner = Runner::new(probe);

    runner.run().await?;

    let results = runner.exit().await?;

    println!("\nRESULTS:");
    for result in results.iter() {
        println!("{}", result);
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
