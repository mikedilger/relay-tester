use nostr_types::PrivateKey;

fn main() {
    let mut private_key = PrivateKey::generate();
    let public_key = private_key.public_key();
    println!("PUBLIC: {}", public_key.as_bech32_string());

    println!("        {}", public_key.as_hex_string());
    println!("PRIVATE: {}", private_key.as_bech32_string());
    println!("        {}", private_key.as_hex_string());
}
