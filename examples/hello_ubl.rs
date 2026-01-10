//! Hello UBL Example
//!
//! Demonstrates creating and appending a signed ledger entry.

use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Hello UBL — Ledger Entry Demo\n");

    // Create a sample intent
    let intent = json!({
        "intent": "Grant",
        "to": "alice",
        "amount": 3,
        "reason": "quarterly bonus"
    });

    println!("📝 Intent: {}", serde_json::to_string_pretty(&intent)?);

    // Compute CID using BLAKE3
    let canonical = serde_json::to_vec(&intent)?;
    let cid = blake3::hash(&canonical);

    println!("🔗 CID (BLAKE3): {}", cid.to_hex());

    // Create ledger entry
    let entry = json!({
        "v": 1,
        "cid": cid.to_hex().to_string(),
        "ts": chrono::Utc::now().to_rfc3339(),
        "intent": intent,
        "signed": false
    });

    // Append to NDJSON ledger
    let ledger_path = "ledger.ndjson";
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger_path)?;

    writeln!(file, "{}", serde_json::to_string(&entry)?)?;

    println!("\n✅ Entry appended to {}", ledger_path);
    println!("📦 Entry: {}", serde_json::to_string_pretty(&entry)?);

    Ok(())
}
