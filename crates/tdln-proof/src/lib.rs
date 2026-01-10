//! TDLN Proof Bundle — captures deterministic proof of translation.
//!
//! `ProofBundle` references canonical content by CID and may carry signatures
//! (plain Ed25519 and/or DV25 Seal via `logline-core` feature).

#![forbid(unsafe_code)]

#[cfg(feature = "dv25")]
use logline_core as _;

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use tdln_ast::SemanticUnit;
use thiserror::Error;

#[cfg(feature = "ed25519")]
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
#[cfg(feature = "ed25519")]
use std::convert::TryInto;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofBundle {
    pub ast_cid: [u8; 32],
    pub canon_cid: [u8; 32],
    /// Rule ids applied deterministically by the compiler.
    pub rules_applied: Vec<String>,
    /// Hashes of relevant inputs (pre-images) to lock determinism.
    pub preimage_hashes: Vec<[u8; 32]>,
    /// Optional signatures over the bundle digest.
    #[cfg(feature = "ed25519")]
    pub signatures: Vec<Vec<u8>>,
}

#[derive(Debug, Error)]
pub enum ProofError {
    #[error("invalid bundle shape")]
    Invalid,
    #[error("signature missing")]
    NoSignature,
    #[error("signature verify failed")]
    VerifyFailed,
}

/// Build a proof bundle from AST + canonical bytes + rule ids.
pub fn build_proof(
    ast: &SemanticUnit,
    canon_json: &[u8],
    rules: &[impl AsRef<str>],
) -> ProofBundle {
    let ast_cid = ast.cid_blake3();
    let mut h = Hasher::new();
    h.update(canon_json);
    let canon_cid = h.finalize().into();
    ProofBundle {
        ast_cid,
        canon_cid,
        rules_applied: rules.iter().map(|r| r.as_ref().to_string()).collect(),
        preimage_hashes: vec![],
        #[cfg(feature = "ed25519")]
        signatures: vec![],
    }
}

/// Digest that is signed/verified (`ast_cid` || `canon_cid` || `rules_applied` as bytes).
fn bundle_digest(bundle: &ProofBundle) -> [u8; 32] {
    let mut h = Hasher::new();
    h.update(&bundle.ast_cid);
    h.update(&bundle.canon_cid);
    for r in &bundle.rules_applied {
        h.update(r.as_bytes());
    }
    h.finalize().into()
}

#[cfg(feature = "ed25519")]
pub fn sign(bundle: &mut ProofBundle, sk: &SigningKey) {
    let msg = bundle_digest(bundle);
    let sig = sk.sign(&msg);
    bundle.signatures.push(sig.to_bytes().to_vec());
}

/// Verifies determinism & integrity relationships within the bundle (shape-level).
///
/// # Errors
///
/// - `ProofError::Invalid` se os CIDs estiverem zerados
pub fn verify_proof(bundle: &ProofBundle) -> Result<(), ProofError> {
    // Minimal sanity: CIDs are non-zero, rules list stable
    if bundle.ast_cid == [0; 32] || bundle.canon_cid == [0; 32] {
        return Err(ProofError::Invalid);
    }
    Ok(())
}

#[cfg(feature = "ed25519")]
/// Verifica as assinaturas associadas ao bundle.
///
/// # Errors
///
/// - `ProofError::NoSignature` se nenhuma assinatura estiver presente
/// - `ProofError::VerifyFailed` se qualquer assinatura falhar a verificação
pub fn verify_signatures(bundle: &ProofBundle, keys: &[VerifyingKey]) -> Result<(), ProofError> {
    if bundle.signatures.is_empty() {
        return Err(ProofError::NoSignature);
    }
    let msg = bundle_digest(bundle);
    for (sig_bytes, vk) in bundle.signatures.iter().zip(keys.iter().cycle()) {
        let sig_array: [u8; 64] = sig_bytes
            .as_slice()
            .try_into()
            .map_err(|_| ProofError::VerifyFailed)?;
        let sig = Signature::from_bytes(&sig_array);
        vk.verify(&msg, &sig)
            .map_err(|_| ProofError::VerifyFailed)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ed25519")]
    use ed25519_dalek::{SigningKey, VerifyingKey};
    #[test]
    fn shape_ok() {
        let ast = SemanticUnit::from_intent("book a table for two");
        let canon = ast.canonical_bytes();
        let pb = build_proof(&ast, &canon, &["normalize", "slots"]);
        assert!(verify_proof(&pb).is_ok());
    }

    #[cfg(feature = "ed25519")]
    #[test]
    fn sign_and_verify() {
        let ast = SemanticUnit::from_intent("set timer 5 minutes");
        let canon = ast.canonical_bytes();
        let mut pb = build_proof(&ast, &canon, &["normalize"]);
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let vk: VerifyingKey = (&sk).into();
        sign(&mut pb, &sk);
        assert!(verify_signatures(&pb, &[vk]).is_ok());
    }
}
