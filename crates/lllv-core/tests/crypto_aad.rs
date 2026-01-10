use lllv_core::{decrypt_chacha20poly1305, encrypt_chacha20poly1305};

#[test]
fn decrypt_fails_on_aad_mismatch() {
    let pt = b"secret";
    let key = [1u8; 32];
    let nonce = [2u8; 12];
    let aad_ok = b"ID:42";
    let aad_bad = b"ID:41";

    let enc = encrypt_chacha20poly1305(pt, &key, &nonce, aad_ok).unwrap();
    let (n, ct) = enc.split_at(12);
    assert!(decrypt_chacha20poly1305(ct, n.try_into().unwrap(), &key, aad_bad).is_err());
}
