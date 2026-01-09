use ed25519_dalek::SigningKey;
use lllv_core::{Capsule, CapsuleFlags};

#[test]
fn tamper_payload_fails_cid() {
    let sk = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let payload = vec![1u8, 2, 3, 4];
    let cap = Capsule::create(2, &payload, CapsuleFlags::NONE, &sk).unwrap();
    assert!(cap.verify_cid().is_ok());

    // tamper
    let mut raw = cap.to_bytes();
    *raw.last_mut().unwrap() ^= 0xFF;
    let cap2 = Capsule::from_bytes(&raw).unwrap();
    assert!(cap2.verify_cid().is_err());
}

#[test]
fn wrong_pubkey_rejects_signature() {
    let sk1 = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let sk2 = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let other = sk2.verifying_key();
    let payload = vec![0u8; 8];
    let cap = Capsule::create(2, &payload, CapsuleFlags::NONE, &sk1).unwrap();
    assert!(cap.verify_with(&other).is_err());
}

#[test]
fn header_len_mismatch_trips_error() {
    let sk = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let payload = vec![0u8; 8];
    let mut cap = Capsule::create(2, &payload, CapsuleFlags::NONE, &sk).unwrap();
    // quebra coerência intencionalmente
    cap.header.len = 999;
    let raw = cap.to_bytes();
    assert!(lllv_core::Capsule::from_bytes(&raw).is_err());
}
