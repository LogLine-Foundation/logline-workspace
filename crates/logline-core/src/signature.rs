#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// Trait para tipos que podem ser assinados.
///
/// Define como um tipo gera bytes determinísticos para assinatura.
/// `LogLine` implementa este trait.
pub trait Signable {
    /// Gera bytes determinísticos que serão assinados.
    fn to_signable_bytes(&self) -> Vec<u8>;
}

/// Assinatura digital de uma mensagem.
///
/// Contém o algoritmo usado e os bytes da assinatura.
/// Em versões futuras, será integrado com `ed25519-dalek` para assinaturas reais.
///
/// # Exemplo
///
/// ```rust
/// use logline_core::Signature;
///
/// let sig = Signature {
///     alg: "ed25519".into(),
///     bytes: vec![0u8; 64],
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    /// Algoritmo de assinatura (ex: "ed25519", "none").
    pub alg: String,
    /// Bytes da assinatura.
    pub bytes: Vec<u8>,
}

/// Placeholder error type for signing operations (v0.1.0).
/// Future versions will use proper error types (e.g., ed25519-dalek errors).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignError;

/// Trait para tipos que podem assinar mensagens.
///
/// Implementações devem assinar os bytes fornecidos e retornar uma `Signature`.
/// Em produção, use implementações reais como `ed25519-dalek::SigningKey`.
///
/// # Exemplo
///
/// ```rust
/// use logline_core::{Signer, Signature, SignError};
///
/// struct NoopSigner;
/// impl Signer for NoopSigner {
///     fn sign(&self, msg: &[u8]) -> Result<Signature, SignError> {
///         Ok(Signature { alg: "none".into(), bytes: msg.to_vec() })
///     }
/// }
/// ```
pub trait Signer {
    /// Assina a mensagem fornecida e retorna uma `Signature`.
    ///
    /// # Errors
    ///
    /// Retorna `SignError` se a assinatura falhar.
    fn sign(&self, msg: &[u8]) -> Result<Signature, SignError>;
}
