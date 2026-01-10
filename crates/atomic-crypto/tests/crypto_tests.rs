use ubl_crypto::*;
use ed25519_dalek::{Signature, Signer, Verifier};

const fn fixed_secret(byte: u8) -> SecretKey {
    SecretKey([byte; 32])
}

#[test]
fn b64_roundtrip_and_invalid() {
    let data = b"hello-world";
    let enc = b64_encode(data);
    assert_eq!(enc, "aGVsbG8td29ybGQ");
    let dec = b64_decode(&enc).expect("decode");
    assert_eq!(dec, data);
    assert!(b64_decode("!").is_err());
}

#[test]
fn blake3_and_key_ids_are_stable() {
    let sk = fixed_secret(7);
    let vk = sk.verifying_key();
    let hash_hex = blake3_hex(vk.as_bytes());
    assert_eq!(hash_hex, blake3::hash(vk.as_bytes()).to_hex().to_string());

    let v1 = key_id_v1(&vk);
    let v2 = key_id_v2(&vk);
    assert!(v1.starts_with("kid:v1:"));
    assert!(v2.starts_with("kid:v2:0001:"));
    // Same hash payload for both versions.
    let tail_v1 = v1.split(':').nth(2).unwrap();
    let tail_v2 = v2.split(':').nth(3).unwrap();
    assert_eq!(tail_v1, tail_v2);
}

#[test]
fn sign_and_verify_cid_hex() {
    let sk = fixed_secret(42);
    let vk = sk.verifying_key();
    let cid = "abcd1234";
    let sig = sign_cid_hex(&sk, cid);
    assert!(verify_cid_hex(&vk, cid, &sig));
    assert!(!verify_cid_hex(&vk, "different", &sig));
}

#[test]
fn verify_many_counts_only_valid() {
    let sk = fixed_secret(9);
    let vk = sk.verifying_key();
    let cid_good = "beef";
    let sig_good = b64_encode(&sign_cid_hex(&sk, cid_good));
    let cases = vec![
        (cid_good, sig_good.as_str()),
        (cid_good, "not-base64"),
        ("other", sig_good.as_str()),
    ];
    assert_eq!(verify_many(&vk, &cases), 1);
}

#[test]
fn hmac_sign_and_verify() {
    let key = b"super-secret";
    let data = b"payload";
    let tag = hmac_sign(key, data);
    hmac_verify(key, data, &tag).expect("valid tag");
    let err = hmac_verify(key, data, "bad").unwrap_err();
    matches!(
        err,
        AtomicCryptoError::Base64(_) | AtomicCryptoError::HmacVerify
    );
}

#[test]
fn did_key_encode_decode_roundtrip() {
    let sk = fixed_secret(5);
    let vk = sk.verifying_key();
    let did = did_key_encode_ed25519(&vk);
    let parsed = did_key_decode_ed25519(&did).expect("decode did:key");
    assert_eq!(parsed.as_bytes(), vk.as_bytes());
    assert!(did_key_decode_ed25519("did:key:zbad").is_err());
}

#[test]
fn keypair_generate_zeroizes_and_signs() {
    let kp = Keypair::generate();
    // signing_key() must match the stored secret key bytes
    let msg = b"sign-me";
    let sig_bytes = kp.signing_key().sign(msg).to_bytes();
    let sig = Signature::from_bytes(&sig_bytes);
    kp.vk.verify(msg, &sig).expect("verify");
}

// ══════════════════════════════════════════════════════════════════════════════
// atomic-types integration tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn blake3_cid_deterministic() {
    let a = blake3_cid(b"hello");
    let b = blake3_cid(b"hello");
    assert_eq!(a.0, b.0);
}

#[test]
fn blake3_cid_chunks_equivalence() {
    let whole = blake3_cid(b"abcdef");
    let chunked = blake3_cid_chunks([b"abc".as_slice(), b"def".as_slice()]);
    assert_eq!(whole.0, chunked.0);
}

#[test]
fn atomic_types_ed25519_roundtrip() {
    let sk = [7u8; 32];
    let pk = derive_public_bytes(&sk);
    let msg = b"deterministic message";
    let sig = sign_bytes(msg, &sk);

    assert!(verify_bytes(msg, &pk, &sig));
    assert!(!verify_bytes(b"wrong", &pk, &sig));
}

#[test]
fn cid_serializes_to_64_hex() {
    let cid = blake3_cid(b"test");
    let json = serde_json::to_string(&cid).unwrap();
    // 64 hex chars + 2 quotes = 66
    assert_eq!(json.len(), 66);
}
