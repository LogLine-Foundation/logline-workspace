use ed25519_dalek::SigningKey;
use json_atomic::{seal_value, verify_seal};
use serde_json::json;

#[test]
fn seal_and_verify_roundtrip() {
    let sk = SigningKey::from_bytes(&[1u8; 32]);
    let value = json!({"a":1,"b":"ok"});
    let fact = seal_value(&value, &sk).unwrap();
    verify_seal(&fact).unwrap();
}
