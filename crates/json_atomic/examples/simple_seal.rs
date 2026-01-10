use ed25519_dalek::SigningKey;
use json_atomic::{seal_value, verify_seal};
use serde_json::json;

fn main() {
    let sk = SigningKey::from_bytes(&[7u8; 32]);
    let value = json!({"b":1,"a":"Café"});
    let fact = seal_value(&value, &sk).unwrap();
    verify_seal(&fact).unwrap();
    println!("CID(hex)={}", fact.cid_hex());
}
