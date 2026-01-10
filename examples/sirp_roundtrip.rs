//! SIRP Roundtrip Example
//!
//! Demonstrates capsule creation and verification flow.

use serde::{Deserialize, Serialize};
use serde_json::json;

/// A simple capsule structure
#[derive(Debug, Serialize, Deserialize)]
struct Capsule {
    id: String,
    payload: serde_json::Value,
    cid: String,
}

/// A receipt acknowledging capsule processing
#[derive(Debug, Serialize, Deserialize)]
struct Receipt {
    capsule_id: String,
    status: String,
    processed_at: String,
}

impl Capsule {
    fn new(payload: serde_json::Value) -> Self {
        let canonical = serde_json::to_vec(&payload).unwrap();
        let cid = blake3::hash(&canonical);
        let id = format!("cap_{}", &cid.to_hex()[..16]);

        Self {
            id,
            payload,
            cid: cid.to_hex().to_string(),
        }
    }

    fn verify(&self) -> bool {
        let canonical = serde_json::to_vec(&self.payload).unwrap();
        let computed_cid = blake3::hash(&canonical);
        self.cid == computed_cid.to_hex().to_string()
    }
}

impl Receipt {
    fn for_capsule(capsule: &Capsule) -> Self {
        Self {
            capsule_id: capsule.id.clone(),
            status: "processed".into(),
            processed_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 SIRP Roundtrip — Capsule + Receipt Demo\n");

    // Create a capsule with some payload
    let payload = json!({
        "action": "transfer",
        "from": "alice",
        "to": "bob",
        "asset": "document_v1",
        "nonce": 42
    });

    let capsule = Capsule::new(payload);
    println!("📦 Capsule created:");
    println!("   ID:  {}", capsule.id);
    println!("   CID: {}", capsule.cid);
    println!("   Payload: {}", serde_json::to_string(&capsule.payload)?);

    // Verify capsule integrity
    let valid = capsule.verify();
    println!("\n🔍 Verification: {}", if valid { "✅ VALID" } else { "❌ INVALID" });

    // Simulate processing and generate receipt
    let receipt = Receipt::for_capsule(&capsule);
    println!("\n🧾 Receipt generated:");
    println!("   Capsule ID: {}", receipt.capsule_id);
    println!("   Status: {}", receipt.status);
    println!("   Processed at: {}", receipt.processed_at);

    // Full roundtrip JSON
    println!("\n📋 Full roundtrip:");
    println!("Capsule: {}", serde_json::to_string_pretty(&capsule)?);
    println!("Receipt: {}", serde_json::to_string_pretty(&receipt)?);

    Ok(())
}
