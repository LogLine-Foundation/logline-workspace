#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Crypto helpers: Ed25519 keypairs/KID, BLAKE3 hashing, and HMAC utilities.

use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use rand_core::OsRng;
use sha2::Sha256;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Erros de crypto.
#[derive(Debug, Error)]
pub enum AtomicCryptoError {
    /// Base64 decoding error.
    #[error("base64 decode: {0}")]
    Base64(#[from] base64::DecodeError),
    /// Signature verification failed.
    #[error("invalid signature")]
    InvalidSig,
    /// HMAC verification failed.
    #[error("hmac verify failed")]
    HmacVerify,
    /// did:key payload is malformed.
    #[error("bad did:key material")]
    BadDidKey,
}

/// Chave secreta (zera memória ao sair).
#[derive(Zeroize, ZeroizeOnDrop, Clone)]
pub struct SecretKey(pub [u8; 32]);

impl SecretKey {
    /// Derives the Ed25519 verifying key from this secret key.
    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        SigningKey::from_bytes(&self.0).verifying_key()
    }
}

/// Par de chaves (conveniência).
pub struct Keypair {
    /// Secret key; zeroized on drop.
    pub sk: SecretKey,
    /// Verifying/public key.
    pub vk: VerifyingKey,
}
impl Keypair {
    /// Gera par de chaves.
    #[must_use]
    pub fn generate() -> Self {
        let sk = SigningKey::generate(&mut OsRng);
        Self {
            sk: SecretKey(sk.to_bytes()),
            vk: sk.verifying_key(),
        }
    }
    /// Signing key helper.
    #[must_use]
    pub fn signing_key(&self) -> SigningKey {
        SigningKey::from_bytes(&self.sk.0)
    }
}

/// Base64 URL-safe (sem padding).
#[must_use]
pub fn b64_encode(b: &[u8]) -> String {
    B64.encode(b)
}
/// Decodifica Base64 URL-safe (sem padding).
///
/// # Errors
///
/// Retorna erro se a string não estiver em Base64 URL-safe sem padding.
pub fn b64_decode(s: &str) -> Result<Vec<u8>, AtomicCryptoError> {
    Ok(B64.decode(s)?)
}
/// Hash BLAKE3 → hex.
#[must_use]
pub fn blake3_hex(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Identificador de chave (v1).
#[must_use]
pub fn key_id_v1(vk: &VerifyingKey) -> String {
    let h = blake3::hash(vk.as_bytes());
    format!("kid:v1:{}", B64.encode(h.as_bytes()))
}
/// Identificador de chave (v2 com versão dummy).
#[must_use]
pub fn key_id_v2(vk: &VerifyingKey) -> String {
    let h = blake3::hash(vk.as_bytes());
    format!("kid:v2:0001:{}", B64.encode(h.as_bytes()))
}

/// Assina um CID (hex) usando Ed25519.
#[must_use]
pub fn sign_cid_hex(sk: &SecretKey, cid_hex: &str) -> [u8; 64] {
    let sig: Signature = SigningKey::from_bytes(&sk.0).sign(cid_hex.as_bytes());
    sig.to_bytes()
}
/// Verifica assinatura de CID (hex).
#[must_use]
pub fn verify_cid_hex(vk: &VerifyingKey, cid_hex: &str, sig: &[u8]) -> bool {
    Signature::from_slice(sig).is_ok_and(|parsed| vk.verify(cid_hex.as_bytes(), &parsed).is_ok())
}

/// HMAC (SHA-256) - retorna base64url sem padding.
///
/// # Panics
///
/// Pode entrar em pânico se a chave HMAC tiver tamanho inválido para SHA-256.
#[must_use]
pub fn hmac_sign(key: &[u8], data: &[u8]) -> String {
    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(key).expect("HMAC key");
    mac.update(data);
    let out = mac.finalize().into_bytes();
    b64_encode(&out)
}
/// Verifica HMAC (base64url sem padding).
///
/// # Errors
///
/// Retorna `AtomicCryptoError::Base64` se a tag não for base64 válida ou `AtomicCryptoError::HmacVerify` se a verificação falhar.
///
/// # Panics
///
/// Pode entrar em pânico se a chave HMAC tiver tamanho inválido para SHA-256.
pub fn hmac_verify(key: &[u8], data: &[u8], tag_b64: &str) -> Result<(), AtomicCryptoError> {
    let want = b64_decode(tag_b64)?;
    let mut mac: Hmac<Sha256> = Hmac::new_from_slice(key).expect("HMAC key");
    mac.update(data);
    mac.verify_slice(&want)
        .map_err(|_| AtomicCryptoError::HmacVerify)
}

/// did:key (ed25519) encoding: returns did:key:z.... (multibase z + multicodec 0xED01 + pk).
#[must_use]
pub fn did_key_encode_ed25519(vk: &VerifyingKey) -> String {
    // multicodec for Ed25519 public key: 0xED 0x01
    let mut data = vec![0xED, 0x01];
    data.extend_from_slice(vk.as_bytes());
    let mut out = String::from("did:key:z");
    // base58btc; minimal inline implementation via bs58
    out.push_str(&bs58::encode(data).into_string());
    out
}
/// did:key decode → `VerifyingKey`
///
/// # Errors
///
/// Retorna erro se o DID não estiver no formato esperado ou o material não for válido.
pub fn did_key_decode_ed25519(did: &str) -> Result<VerifyingKey, AtomicCryptoError> {
    let p = did
        .strip_prefix("did:key:z")
        .ok_or(AtomicCryptoError::BadDidKey)?;
    let bytes = bs58::decode(p)
        .into_vec()
        .map_err(|_| AtomicCryptoError::BadDidKey)?;
    if bytes.len() != 34 || bytes[0] != 0xED || bytes[1] != 0x01 {
        return Err(AtomicCryptoError::BadDidKey);
    }
    let pk: [u8; 32] = bytes[2..]
        .try_into()
        .map_err(|_| AtomicCryptoError::BadDidKey)?;
    VerifyingKey::from_bytes(&pk).map_err(|_| AtomicCryptoError::BadDidKey)
}

/// Verifica vários pares (vk, `cid_hex`, `sig_b64`).
#[must_use]
pub fn verify_many(vk: &VerifyingKey, items: &[(&str, &str)]) -> usize {
    items
        .iter()
        .filter(|(cid, sig_b64)| b64_decode(sig_b64).is_ok_and(|sig| verify_cid_hex(vk, cid, &sig)))
        .count()
}

// ══════════════════════════════════════════════════════════════════════════════
// atomic-types integration: Cid32, PublicKeyBytes, SignatureBytes
// ══════════════════════════════════════════════════════════════════════════════

use ubl_types::{Cid32, PublicKeyBytes, SignatureBytes};

/// Hash BLAKE3 de 32 bytes → `Cid32`.
///
/// Wrapper determinístico que retorna o tipo canônico `Cid32`.
#[must_use]
#[inline]
pub fn blake3_cid(data: &[u8]) -> Cid32 {
    let hash = blake3::hash(data);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(hash.as_bytes());
    Cid32(bytes)
}

/// Hash BLAKE3 de múltiplos chunks → `Cid32`.
///
/// Útil para hashing incremental/streaming.
#[must_use]
#[inline]
pub fn blake3_cid_chunks<'a, I>(chunks: I) -> Cid32
where
    I: IntoIterator<Item = &'a [u8]>,
{
    let mut hasher = blake3::Hasher::new();
    for c in chunks {
        hasher.update(c);
    }
    let out = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(out.as_bytes());
    Cid32(bytes)
}

/// Deriva `PublicKeyBytes` a partir de secret seed Ed25519 (32B).
#[must_use]
#[inline]
pub fn derive_public_bytes(secret_key: &[u8; 32]) -> PublicKeyBytes {
    let sk = SigningKey::from_bytes(secret_key);
    PublicKeyBytes(sk.verifying_key().to_bytes())
}

/// Assina `msg` com secret seed Ed25519 (32B) → `SignatureBytes`.
#[must_use]
#[inline]
pub fn sign_bytes(msg: &[u8], secret_key: &[u8; 32]) -> SignatureBytes {
    let sk = SigningKey::from_bytes(secret_key);
    let sig: Signature = sk.sign(msg);
    SignatureBytes(sig.to_bytes())
}

/// Verifica assinatura Ed25519 usando tipos `atomic-types`.
///
/// Retorna `true` se a assinatura for válida.
#[must_use]
#[inline]
pub fn verify_bytes(msg: &[u8], pk: &PublicKeyBytes, sig: &SignatureBytes) -> bool {
    match VerifyingKey::from_bytes(&pk.0) {
        Ok(vk) => {
            let s = Signature::from_bytes(&sig.0);
            vk.verify_strict(msg, &s).is_ok()
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod atomic_types_tests {
    use super::*;

    #[test]
    fn blake3_cid_deterministic() {
        let a = blake3_cid(b"hello");
        let b = blake3_cid(b"hello");
        assert_eq!(a.0, b.0);

        let c = blake3_cid_chunks([b"hel".as_slice(), b"lo".as_slice()]);
        assert_eq!(a.0, c.0);
    }

    #[test]
    fn ed25519_atomic_types_roundtrip() {
        let sk = [7u8; 32]; // fixed seed for determinism
        let pk = derive_public_bytes(&sk);
        let msg = b"deterministic message";
        let sig = sign_bytes(msg, &sk);

        assert!(verify_bytes(msg, &pk, &sig));
        assert!(!verify_bytes(b"wrong", &pk, &sig));
    }

    #[test]
    fn cid_serializes_to_hex() {
        let cid = blake3_cid(b"test");
        let json = serde_json::to_string(&cid).unwrap();
        // Should be 64 hex chars + 2 quotes
        assert_eq!(json.len(), 66);
    }
}
