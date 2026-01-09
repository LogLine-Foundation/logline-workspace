#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use ed25519_dalek::{Signer, SigningKey};
use logline_core::LogLine;

use crate::errors::{SealError, VerifyError};
use crate::{
    canonize,
    version::{CANON_VERSION, FORMAT_ID},
    SignedFact,
};

/// Sela um valor serializável como Signed Fact (Paper II: Cycle of Truth).
///
/// Executa o ciclo completo: `canonize → blake3 → ed25519`:
/// 1. Canoniza o valor em bytes determinísticos
/// 2. Calcula o CID (BLAKE3 hash dos bytes canônicos)
/// 3. Assina o CID com Ed25519
///
/// O resultado é um `SignedFact` imutável que pode ser verificado por qualquer pessoa
/// que tenha acesso ao fato, sem necessidade de confiar em terceiros.
///
/// # Erros
///
/// - `SealError::Canonical` se a canonicalização falhar
///
/// # Exemplo
///
/// ```rust
/// use ed25519_dalek::SigningKey;
/// use json_atomic::{seal_value, errors::SealError};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Fact {
///     message: String,
///     timestamp: u64,
/// }
///
/// // Chave de exemplo (em produção, derive de seed/keystore)
/// let sk = SigningKey::from_bytes(&[0u8; 32]);
/// let fact = Fact {
///     message: "Hello, World!".into(),
///     timestamp: 1735671234,
/// };
///
/// let signed = seal_value(&fact, &sk)?;
/// println!("CID: {}", signed.cid_hex());
/// # Ok::<(), SealError>(())
/// ```
pub fn seal_value<T: serde::Serialize>(
    value: &T,
    sk: &SigningKey,
) -> Result<SignedFact, SealError> {
    let canonical = canonize(value)?;
    let cid = blake3::hash(&canonical);
    let sig = sk.sign(cid.as_bytes());
    let vk = sk.verifying_key();

    Ok(SignedFact {
        canonical,
        cid: *cid.as_bytes(),
        signature: sig.to_bytes(),
        public_key: vk.to_bytes(),
        hash_alg: "blake3",
        sig_alg: "ed25519",
        canon_ver: CANON_VERSION,
        format_id: FORMAT_ID,
    })
}

/// Verifica a integridade e autenticidade de um Signed Fact.
///
/// Valida que:
/// 1. O CID corresponde aos bytes canônicos (recalcula BLAKE3)
/// 2. A assinatura Ed25519 é válida para o CID e a chave pública
///
/// Retorna `Ok(())` se o fato estiver íntegro e autenticado, ou um erro
/// se houver qualquer problema de integridade ou assinatura.
///
/// # Erros
///
/// - `VerifyError::CanonicalMismatch` se o CID recalculado não corresponder
/// - `VerifyError::BadSignature` se a assinatura for inválida
///
/// # Exemplo
///
/// ```rust
/// use ed25519_dalek::SigningKey;
/// use json_atomic::{seal_value, verify_seal, errors::SealError};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Fact {
///     data: String,
/// }
///
/// // Chave de exemplo (em produção, derive de seed/keystore)
/// let sk = SigningKey::from_bytes(&[0u8; 32]);
/// let fact = Fact { data: "test".into() };
///
/// let signed = seal_value(&fact, &sk)?;
/// verify_seal(&signed).unwrap(); // Verifica integridade e autenticidade
/// # Ok::<(), SealError>(())
/// ```
pub fn verify_seal(f: &SignedFact) -> Result<(), VerifyError> {
    // Recalcula CID do canonical e compara
    let recomputed = blake3::hash(&f.canonical);
    if recomputed.as_bytes() != &f.cid {
        return Err(VerifyError::CanonicalMismatch);
    }
    let vk = f.verifying_key();
    vk.verify_strict(recomputed.as_bytes(), &f.signature_obj())
        .map_err(|_| VerifyError::BadSignature)
}

/// Sela um LogLine completo como Signed Fact (Paper II: Signed Fact de ação verificada).
///
/// Conveniência para selar um `LogLine` do `logline-core` diretamente.
/// Equivalente a `seal_value(line, sk)`, mas com tipo específico para LogLine.
///
/// # Exemplo
///
/// ```rust
/// use ed25519_dalek::SigningKey;
/// use json_atomic::{seal_logline, errors::SealError};
/// use logline_core::*;
///
/// // Chave de exemplo (em produção, derive de seed/keystore)
/// let sk = SigningKey::from_bytes(&[0u8; 32]);
/// let line = LogLine::builder()
///     .who("did:ubl:alice")
///     .did(Verb::Approve)
///     .when(1735671234)
///     .if_ok(Outcome { label: "ok".into(), effects: vec![] })
///     .if_doubt(Escalation { label: "doubt".into(), route_to: "auditor".into() })
///     .if_not(FailureHandling { label: "not".into(), action: "notify".into() })
///     .build_draft()
///     .unwrap();
///
/// let signed = seal_logline(&line, &sk)?;
/// println!("LogLine CID: {}", signed.cid_hex());
/// # Ok::<(), SealError>(())
/// ```
pub fn seal_logline(line: &LogLine, sk: &SigningKey) -> Result<SignedFact, SealError> {
    // Reaproveita `serde` derivado do logline-core
    seal_value(line, sk)
}
