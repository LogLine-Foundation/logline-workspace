//! JSON✯Atomic Seal Example
//!
//! Demonstrates canonical sealing and verification.

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde::Serialize;

#[derive(Serialize)]
struct Document {
    title: String,
    author: String,
    version: u32,
    approved: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 JSON✯Atomic Seal Demo\n");

    // Generate a signing key
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    println!("🔑 Generated Ed25519 keypair");
    println!("   Public key: {}", hex::encode(verifying_key.as_bytes()));

    // Create a document
    let doc = Document {
        title: "Project Proposal".into(),
        author: "alice".into(),
        version: 1,
        approved: true,
    };

    // Canonicalize
    let canonical = serde_json::to_vec(&doc)?;
    println!("\n📄 Document (canonical JSON):");
    println!("   {}", String::from_utf8_lossy(&canonical));

    // Compute CID
    let cid = blake3::hash(&canonical);
    println!("\n🔗 CID (BLAKE3): {}", cid.to_hex());

    // Sign the CID
    use ed25519_dalek::Signer;
    let signature = signing_key.sign(cid.as_bytes());
    println!("✍️  Signature: {}...", &hex::encode(signature.to_bytes())[..32]);

    // Verify
    use ed25519_dalek::Verifier;
    let valid = verifying_key.verify(cid.as_bytes(), &signature).is_ok();
    println!("\n🔍 Verification: {}", if valid { "✅ VALID" } else { "❌ INVALID" });

    // Create sealed fact
    let sealed = serde_json::json!({
        "v": 1,
        "cid": cid.to_hex().to_string(),
        "payload": doc,
        "sig": hex::encode(signature.to_bytes()),
        "pk": hex::encode(verifying_key.as_bytes()),
    });

    println!("\n📦 Sealed Fact:");
    println!("{}", serde_json::to_string_pretty(&sealed)?);

    Ok(())
}
