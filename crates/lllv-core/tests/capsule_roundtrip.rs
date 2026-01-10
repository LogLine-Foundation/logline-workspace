use ed25519_dalek::SigningKey;
use lllv_core::*;

#[test]
fn capsule_create_verify_plain() {
    let sk = SigningKey::from_bytes(&[1u8; 32]);
    let payload = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let cap = Capsule::create(3, &payload, CapsuleFlags::NONE, &sk).unwrap();
    cap.verify_cid().unwrap();
    // Verificação com chave pública
    let pk = sk.verifying_key();
    cap.verify_with(&pk).unwrap();
}

#[test]
fn capsule_encrypt_decrypt() {
    let payload = (0..64u8).collect::<Vec<_>>();
    let key = [1u8; 32];
    let nonce = [2u8; 12];
    let aad = b"doc:1";
    let enc = encrypt_chacha20poly1305(&payload, &key, &nonce, aad).unwrap();
    let (n, ct) = enc.split_at(12);
    let plain = decrypt_chacha20poly1305(ct, n.try_into().unwrap(), &key, aad).unwrap();
    assert_eq!(plain, payload);
}

#[test]
fn capsule_roundtrip() {
    let sk = SigningKey::from_bytes(&[3u8; 32]);
    let payload = vec![10u8, 20, 30, 40, 50];
    let cap = Capsule::create(5, &payload, CapsuleFlags::NONE, &sk).unwrap();

    let bytes = cap.to_bytes();
    let cap2 = Capsule::from_bytes(&bytes).unwrap();

    assert_eq!(cap.header.cid, cap2.header.cid);
    assert_eq!(cap.payload, cap2.payload);

    let pk = sk.verifying_key();
    cap2.verify_with(&pk).unwrap();
}
