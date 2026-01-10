//! Quickstart example for SIRP wire protocol.

use atomic_sirp::{canon_intent_from_value, decode_frame, encode_frame, SirpFrame};
use serde_json::json;

fn main() {
    // Create a canonical intent from JSON
    let v = json!({"intent":"Grant","to":"bob","amount":7});
    let ci = canon_intent_from_value(&v).expect("canonicalization failed");

    // Build an unsigned frame
    let frame = SirpFrame::unsigned(ci);

    // Encode to wire format
    let wire_bytes = encode_frame(&frame);
    println!("Encoded {} bytes", wire_bytes.len());

    // Decode back
    let decoded = decode_frame(&wire_bytes).expect("decode failed");

    // Verify CID matches
    assert_eq!(frame.intent.cid, decoded.intent.cid);
    println!("CID: {}", hex::encode(decoded.intent.cid.0));
    println!("OK!");
}
