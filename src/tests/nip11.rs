use crate::error::Error;
use crate::globals::GLOBALS;
use crate::outcome::Outcome;
use serde_json::Value;

pub async fn nip11_provided() -> Result<Outcome, Error> {
    let nip11 = crate::connection::fetch_nip11().await?;

    *GLOBALS.nip11.write() = Some(nip11);

    Ok(Outcome::pass(None))
}

pub async fn claimed_support_for_nip(number: u64) -> Result<Outcome, Error> {
    let nip11 = GLOBALS.nip11.read().clone();
    if nip11.is_none() {
        return Ok(Outcome::fail(Some(
            "NIP-11 document was not found".to_owned(),
        )));
    }
    let nip11 = nip11.unwrap();

    if let Value::Object(map) = nip11 {
        if let Some(Value::Array(vec)) = map.get("supported_nips") {
            for valelem in vec.iter() {
                if let Value::Number(vnum) = valelem {
                    if let Some(u) = vnum.as_u64() {
                        if u == number {
                            return Ok(Outcome::pass(None));
                        }
                    }
                }
            }
        }
    }

    Ok(Outcome::fail(None))
}
