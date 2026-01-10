//! Quick-start example for atomic-crypto with atomic-types integration.

use atomic_crypto::{blake3_cid, derive_public_bytes, sign_bytes, verify_bytes};

fn main() {
    // BLAKE3 → Cid32
    let cid = blake3_cid(b"quickstart example");
    println!("cid = {cid}");
    println!("cid json = {}", serde_json::to_string(&cid).unwrap());

    // Ed25519 with atomic-types wrappers
    let sk = [1u8; 32]; // fixed seed for demo
    let pk = derive_public_bytes(&sk);
    let msg = b"hello world";
    let sig = sign_bytes(msg, &sk);

    println!("public key = {pk}");
    println!("signature = {sig}");
    println!("verify = {}", verify_bytes(msg, &pk, &sig));
}
