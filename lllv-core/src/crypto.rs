use crate::errors::LllvError;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Key, Nonce,
};

pub fn encrypt_chacha20poly1305(
    plain: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
    aad: &[u8],
) -> Result<Vec<u8>, LllvError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let mut out = Vec::with_capacity(12 + plain.len() + 16);
    out.extend_from_slice(nonce);
    let ct = cipher
        .encrypt(Nonce::from_slice(nonce), Payload { msg: plain, aad })
        .map_err(|_| LllvError::Crypto)?;
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn decrypt_chacha20poly1305(
    cipher: &[u8],
    nonce: &[u8; 12],
    key: &[u8; 32],
    aad: &[u8],
) -> Result<Vec<u8>, LllvError> {
    let cipher_impl = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher_impl
        .decrypt(Nonce::from_slice(nonce), Payload { msg: cipher, aad })
        .map_err(|_| LllvError::Crypto)
}
