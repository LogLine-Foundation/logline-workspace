//! Receipt signing and verification for SIRP capsules.
use ubl_crypto::{b64_decode, b64_encode, sign_cid_hex, verify_cid_hex, SecretKey};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

/// Recibo de processamento SIRP.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    /// CID da cápsula (hex de blake3)
    pub capsule_cid_hex: String,
    /// Resultado do processamento
    pub ok: bool,
    /// Mensagem (erro, "`already_done`", etc.)
    pub msg: Option<String>,
    /// Assinatura e chave pública em base64url
    pub sig_b64: String,
    /// Chave pública em base64url.
    pub pk_b64: String,
    /// Identificador da chave (kid)
    pub kid: String,
    /// Indica se foi deduplicação (idempotência)
    pub already_done: bool,
}
/// Assina recibo.
#[must_use]
pub fn sign_receipt(
    sk: &SecretKey,
    vk: &VerifyingKey,
    kid: &str,
    cid_hex: &str,
    ok: bool,
    msg: Option<String>,
    already_done: bool,
) -> Receipt {
    let sig = sign_cid_hex(sk, cid_hex);
    Receipt {
        capsule_cid_hex: cid_hex.into(),
        ok,
        msg,
        sig_b64: b64_encode(&sig),
        pk_b64: b64_encode(vk.as_bytes()),
        kid: kid.into(),
        already_done,
    }
}
/// Verifica recibo.
#[must_use]
pub fn verify_receipt(vk: &VerifyingKey, r: &Receipt) -> bool {
    b64_decode(&r.sig_b64).is_ok_and(|sig| verify_cid_hex(vk, &r.capsule_cid_hex, &sig))
}
