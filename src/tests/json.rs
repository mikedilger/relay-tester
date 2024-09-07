use crate::error::Error;
use crate::globals::{Globals, GLOBALS};
use crate::outcome::Outcome;
use crate::WAIT;
use nostr_types::Unixtime;
use std::time::Duration;

// Try including all nip01 escape sequences
pub async fn nip1() -> Result<Outcome, Error> {
    go(r##"linebreak\ndoublequote\"backslash\\carraigereturn\rtab\tbackspace\bformfeed\fend"##)
        .await
}

// Try including escape sequences not listed in nip01
pub async fn unlisted() -> Result<Outcome, Error> {
    go(r#"\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007 \u000b \u000e \u000f \u0010\u0011\u0012\u0013\u0014\u0015\u0016 \/"#).await
}

// Try including all nip01 escape sequences as literals instead of escapes
// (except we cant use a literal double quote)
pub async fn literals() -> Result<Outcome, Error> {
    go("linebreak\nbackslash\\carraigereturn\rtab\tbackspace\x08formfeed\x0cend").await
}

// Try including non-characters such as FDD1 and 1FFFF
// &[0xef, 0xb7, 0x91, 0xf4, 0x8f, 0xbf, 0xb2];
// https://www.unicode.org/faq/private_use.html#noncharacters
pub async fn utf8non() -> Result<Outcome, Error> {
    go(std::str::from_utf8(&[0xef, 0xb7, 0x91, 0xf4, 0x8f, 0xbf, 0xb2]).unwrap()).await
}

async fn go(content: &str) -> Result<Outcome, Error> {
    let (id, raw_event) =
        Globals::make_raw_event(&format!("{}", Unixtime::now().0), "1", "[]", content, true);

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
