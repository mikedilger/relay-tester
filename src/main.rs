mod error;
mod probe;
mod results;
mod runner;

use crate::error::Error;
use crate::probe::Probe;
use crate::runner::Runner;
use colorful::{Color, Colorful};
use nostr_types::{KeySigner, PrivateKey};
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

    println!("{:?}", results);

    Ok(())
}

fn usage() -> Result<(), Error> {
    eprintln!(
        "{}: relay-tester <relay_url> <allowed_nsec>",
        "Usage".color(Color::Gold1)
    );
    Ok(())
}
