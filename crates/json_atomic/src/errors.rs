use thiserror::Error;

/// Erros que podem ocorrer durante a canonicalização.
///
/// A canonicalização JSON✯Atomic tem regras rígidas que garantem
/// determinismo: mesmo valor sempre produz os mesmos bytes.
#[derive(Debug, Error)]
pub enum CanonicalError {
    /// Números de ponto flutuante não são permitidos em JSON canônico.
    ///
    /// Apenas inteiros são suportados para garantir determinismo.
    /// Use `u64` ou `i64` em vez de `f32` ou `f64`.
    #[error("floating numbers are not allowed in canonical JSON")]
    FloatNotAllowed,
    /// Erro na normalização Unicode (quando `feature = "unicode"`).
    #[error("invalid unicode normalization")]
    Unicode,
    /// Erro de serialização do serde.
    #[error("serde error: {0}")]
    Serde(String),
}

/// Erros que podem ocorrer ao selar um valor.
///
/// O processo de selagem envolve canonicalização, hash e assinatura.
#[derive(Debug, Error)]
pub enum SealError {
    /// A canonicalização falhou.
    ///
    /// Veja `CanonicalError` para detalhes sobre o erro específico.
    #[error("canonicalization failed: {0}")]
    Canonical(#[from] CanonicalError),
}

/// Erros que podem ocorrer ao verificar um Signed Fact.
///
/// A verificação valida tanto a integridade (CID) quanto a autenticidade (assinatura).
#[derive(Debug, Error)]
pub enum VerifyError {
    /// Os bytes canônicos não correspondem ao CID armazenado.
    ///
    /// Isso indica que o fato foi corrompido ou modificado após a assinatura.
    #[error("signed fact canonical bytes mismatch (recomputed CID differs)")]
    CanonicalMismatch,
    /// A assinatura Ed25519 é inválida.
    ///
    /// Isso indica que o fato não foi assinado pela chave pública correspondente,
    /// ou que foi modificado após a assinatura.
    #[error("signature verification failed")]
    BadSignature,
}
