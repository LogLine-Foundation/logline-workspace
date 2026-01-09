//! # logline-core
//!
//! The Conceptual Atom of Verifiable Action — Paper I §3 (9-field tuple, lifecycle, invariants, Ghost Records)
//!
//! See [README.md](https://github.com/LogLine-Foundation/logline-core/blob/main/README.md) for full documentation.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

mod builder;
mod consequence;
mod ghost;
mod payload;
mod signature;
mod status;
mod verb;

pub use builder::LogLineBuilder;
pub use consequence::{Escalation, FailureHandling, Outcome};
pub use ghost::GhostRecord;
pub use payload::Payload;
pub use signature::{SignError, Signable, Signature, Signer};
pub use status::Status;
pub use verb::{Verb, VerbRegistry};

use thiserror::Error;

/// Número rígido de campos do átomo conceitual (Paper I §3).
pub const TUPLE_FIELD_COUNT: usize = 9;

/// Erros que podem ocorrer ao manipular um `LogLine`.
///
/// Todos os erros são retornados como `LogLineError` e podem ser convertidos
/// em strings legíveis usando `Display` (via `thiserror`).
#[derive(Error, Debug, PartialEq)]
pub enum LogLineError {
    /// Um invariant obrigatório está faltando ou vazio.
    ///
    /// Os invariants obrigatórios são: `if_ok`, `if_doubt`, `if_not`.
    #[error("Missing consequence invariant: {0}")]
    MissingInvariant(&'static str),
    /// Um campo obrigatório está faltando.
    ///
    /// Campos obrigatórios: `who`, `did`, `when`.
    #[error("Missing field: {0}")]
    MissingField(&'static str),
    /// Tentativa de transição de status inválida.
    ///
    /// O lifecycle permite apenas: `DRAFT → PENDING → COMMITTED` ou `DRAFT/PENDING → GHOST`.
    #[error("Invalid status transition: {from:?} → {to:?}")]
    InvalidTransition { from: Status, to: Status },
    /// Tentativa de abandonar um LogLine que já está `Committed`.
    ///
    /// LogLines `Committed` são imutáveis e não podem ser abandonados.
    #[error("Cannot ghost a committed LogLine")]
    AlreadyCommitted,
    /// Erro durante a assinatura do LogLine.
    #[error("Signing error")]
    Signing,
}

/// O "átomo" LogLine — 9-field tuple rígido.
///
/// Representa uma ação verificável com lifecycle determinístico e invariants obrigatórios.
/// Conforme Paper I §3, cada LogLine deve ter exatamente 9 campos e seguir o lifecycle
/// `DRAFT → PENDING → COMMITTED | GHOST`.
///
/// # Exemplo
///
/// ```rust
/// use logline_core::*;
///
/// let line = LogLine::builder()
///     .who("did:ubl:alice")
///     .did(Verb::Approve)
///     .this(Payload::Text("purchase:123".into()))
///     .when(1_735_671_234)
///     .if_ok(Outcome { label: "approved".into(), effects: vec!["emit_receipt".into()] })
///     .if_doubt(Escalation { label: "manual_review".into(), route_to: "auditor".into() })
///     .if_not(FailureHandling { label: "rejected".into(), action: "notify".into() })
///     .build_draft()?;
/// # Ok::<(), LogLineError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LogLine {
    /// Identidade do agente que executa a ação (DID futuro, ex: `did:ubl:...`).
    pub who: String,
    /// Verbo canônico ou custom (Paper I: validar contra ALLOWED_ACTIONS via VerbRegistry).
    pub did: Verb,
    /// Carga útil mínima/typed (Paper I: JSON estrito validado por schema do verbo).
    pub this: Payload,
    /// Unix timestamp em nanosegundos (interno). Na serialização canônica JSON✯Atomic será ISO8601.
    pub when: u64,
    /// Identidade que confirma a ação. Paper I: obrigatório para ações de Risk Level 3+ (L3+).
    pub confirmed_by: Option<String>,
    /// Consequência positiva obrigatória (Paper I: invariante obrigatório).
    pub if_ok: Outcome,
    /// Via de dúvida obrigatória (Paper I: invariante obrigatório).
    pub if_doubt: Escalation,
    /// Fallback/erro obrigatório (Paper I: invariante obrigatório).
    pub if_not: FailureHandling,
    /// Estado do lifecycle rígido: `DRAFT → PENDING → COMMITTED | GHOST`.
    pub status: Status,
    // pub signature: Option<Signature> // (futuro)
}

impl LogLine {
    /// Cria um novo builder para construir um `LogLine` passo a passo.
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use logline_core::*;
    ///
    /// let builder = LogLine::builder()
    ///     .who("did:ubl:alice")
    ///     .did(Verb::Transfer);
    /// ```
    pub fn builder() -> LogLineBuilder {
        LogLineBuilder::new()
    }

    /// Verifica invariants do 9-tuple (Paper I §3).
    ///
    /// Valida que todos os campos obrigatórios estão presentes e não vazios:
    /// - `who` não pode ser vazio
    /// - `when` deve ser > 0
    /// - `if_ok`, `if_doubt`, `if_not` devem estar presentes e não vazios
    ///
    /// # Erros
    ///
    /// Retorna `LogLineError::MissingField` ou `LogLineError::MissingInvariant`
    /// se algum invariant não for satisfeito.
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use logline_core::*;
    ///
    /// let line = LogLine::builder()
    ///     .who("did:ubl:alice")
    ///     .did(Verb::Approve)
    ///     .when(1_735_671_234)
    ///     .if_ok(Outcome { label: "ok".into(), effects: vec![] })
    ///     .if_doubt(Escalation { label: "doubt".into(), route_to: "auditor".into() })
    ///     .if_not(FailureHandling { label: "not".into(), action: "notify".into() })
    ///     .build_draft()?;
    ///
    /// assert!(line.verify_invariants().is_ok());
    /// # Ok::<(), LogLineError>(())
    /// ```
    pub fn verify_invariants(&self) -> Result<(), LogLineError> {
        if self.who.is_empty() {
            return Err(LogLineError::MissingField("who"));
        }
        if self.when == 0 {
            return Err(LogLineError::MissingField("when"));
        }
        if self.if_ok.is_empty() {
            return Err(LogLineError::MissingInvariant("if_ok"));
        }
        if self.if_doubt.is_empty() {
            return Err(LogLineError::MissingInvariant("if_doubt"));
        }
        if self.if_not.is_empty() {
            return Err(LogLineError::MissingInvariant("if_not"));
        }
        Ok(())
    }

    /// Assina o LogLine (DRAFT ou PENDING). Paper I: "nada acontece sem estar assinado".
    ///
    /// A assinatura é calculada sobre os bytes determinísticos retornados por
    /// `to_signable_bytes()`. Retorna `self` para method chaining.
    ///
    /// # Erros
    ///
    /// - `LogLineError::InvalidTransition` se o status não for `Draft` ou `Pending`
    /// - `LogLineError::Signing` se a assinatura falhar
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use logline_core::*;
    ///
    /// struct NoopSigner;
    /// impl Signer for NoopSigner {
    ///     fn sign(&self, _msg: &[u8]) -> Result<Signature, SignError> {
    ///         Ok(Signature { alg: "none".into(), bytes: vec![] })
    ///     }
    /// }
    ///
    /// let signer = NoopSigner;
    /// let line = LogLine::builder()
    ///     .who("did:ubl:alice")
    ///     .did(Verb::Approve)
    ///     .when(1_735_671_234)
    ///     .if_ok(Outcome { label: "ok".into(), effects: vec![] })
    ///     .if_doubt(Escalation { label: "doubt".into(), route_to: "auditor".into() })
    ///     .if_not(FailureHandling { label: "not".into(), action: "notify".into() })
    ///     .build_draft()?
    ///     .sign(&signer)?;
    /// # Ok::<(), LogLineError>(())
    /// ```
    pub fn sign(self, signer: &dyn Signer) -> Result<Self, LogLineError> {
        match self.status {
            Status::Draft | Status::Pending => {
                let _sig = signer
                    .sign(&self.to_signable_bytes())
                    .map_err(|_| LogLineError::Signing)?;
                // self.signature = Some(sig); // quando ativarmos
                Ok(self)
            }
            _ => Err(LogLineError::InvalidTransition {
                from: self.status,
                to: Status::Committed,
            }),
        }
    }

    /// Congela o DRAFT em PENDING (pronto para sign/commit/ghost).
    ///
    /// Valida os invariants antes de fazer a transição. Após `freeze()`, o LogLine
    /// está pronto para ser assinado e commitado, ou abandonado como Ghost.
    ///
    /// # Erros
    ///
    /// - `LogLineError::InvalidTransition` se o status não for `Draft`
    /// - `LogLineError::MissingField` ou `LogLineError::MissingInvariant` se os invariants falharem
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use logline_core::*;
    ///
    /// let line = LogLine::builder()
    ///     .who("did:ubl:alice")
    ///     .did(Verb::Approve)
    ///     .when(1_735_671_234)
    ///     .if_ok(Outcome { label: "ok".into(), effects: vec![] })
    ///     .if_doubt(Escalation { label: "doubt".into(), route_to: "auditor".into() })
    ///     .if_not(FailureHandling { label: "not".into(), action: "notify".into() })
    ///     .build_draft()?
    ///     .freeze()?;
    ///
    /// assert_eq!(line.status, Status::Pending);
    /// # Ok::<(), LogLineError>(())
    /// ```
    pub fn freeze(mut self) -> Result<Self, LogLineError> {
        status::ensure(Status::Draft, Status::Pending, self.status)?;
        self.verify_invariants()?;
        self.status = Status::Pending;
        Ok(self)
    }

    /// Congela com validação de verbo contra ALLOWED_ACTIONS (Paper I: verbo deve estar no registro).
    ///
    /// Equivalente a `freeze()`, mas valida primeiro se o verbo (`did`) está permitido
    /// no sistema através do `VerbRegistry`. Útil para implementar políticas de segurança
    /// onde apenas verbos específicos são permitidos.
    ///
    /// # Erros
    ///
    /// - `LogLineError::MissingField("did (unknown verb)")` se o verbo não estiver no registro
    /// - Erros de `freeze()` se os invariants falharem
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use logline_core::*;
    ///
    /// struct SimpleRegistry;
    /// impl VerbRegistry for SimpleRegistry {
    ///     fn is_allowed(&self, verb: &Verb) -> bool {
    ///         matches!(verb, Verb::Transfer | Verb::Approve)
    ///     }
    /// }
    ///
    /// let registry = SimpleRegistry;
    /// let line = LogLine::builder()
    ///     .who("did:ubl:alice")
    ///     .did(Verb::Approve)
    ///     .when(1_735_671_234)
    ///     .if_ok(Outcome { label: "ok".into(), effects: vec![] })
    ///     .if_doubt(Escalation { label: "doubt".into(), route_to: "auditor".into() })
    ///     .if_not(FailureHandling { label: "not".into(), action: "notify".into() })
    ///     .build_draft()?
    ///     .freeze_with_registry(&registry)?;
    /// # Ok::<(), LogLineError>(())
    /// ```
    pub fn freeze_with_registry(self, registry: &dyn VerbRegistry) -> Result<Self, LogLineError> {
        if !registry.is_allowed(&self.did) {
            return Err(LogLineError::MissingField("did (unknown verb)"));
        }
        self.freeze()
    }

    /// Commit final (PENDING → COMMITTED). Paper I: requer assinatura obrigatória.
    ///
    /// Transiciona o LogLine de `Pending` para `Committed`, assinando os bytes determinísticos.
    /// Uma vez `Committed`, o LogLine não pode mais ser modificado ou abandonado.
    ///
    /// # Erros
    ///
    /// - `LogLineError::InvalidTransition` se o status não for `Pending`
    /// - `LogLineError::Signing` se a assinatura falhar
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use logline_core::*;
    ///
    /// struct NoopSigner;
    /// impl Signer for NoopSigner {
    ///     fn sign(&self, _msg: &[u8]) -> Result<Signature, SignError> {
    ///         Ok(Signature { alg: "none".into(), bytes: vec![] })
    ///     }
    /// }
    ///
    /// let signer = NoopSigner;
    /// let line = LogLine::builder()
    ///     .who("did:ubl:alice")
    ///     .did(Verb::Approve)
    ///     .when(1_735_671_234)
    ///     .if_ok(Outcome { label: "ok".into(), effects: vec![] })
    ///     .if_doubt(Escalation { label: "doubt".into(), route_to: "auditor".into() })
    ///     .if_not(FailureHandling { label: "not".into(), action: "notify".into() })
    ///     .build_draft()?
    ///     .freeze()?
    ///     .commit(&signer)?;
    ///
    /// assert_eq!(line.status, Status::Committed);
    /// # Ok::<(), LogLineError>(())
    /// ```
    pub fn commit(mut self, signer: &dyn Signer) -> Result<Self, LogLineError> {
        status::ensure(Status::Pending, Status::Committed, self.status)?;
        // Futuro: bytes canônicos JSON✯Atomic; por ora, ordem estável de campos:
        let _sig = signer
            .sign(&self.to_signable_bytes())
            .map_err(|_| LogLineError::Signing)?;
        // self.signature = Some(sig); // quando ativarmos
        self.status = Status::Committed;
        Ok(self)
    }

    /// Abandona intenção: DRAFT/PENDING → GHOST (forensics).
    ///
    /// Versão sem assinatura (compatibilidade). Para versão assinada, use `abandon_signed()`.
    /// Cria um `GhostRecord` que preserva o LogLine original para análise forense.
    ///
    /// # Erros
    ///
    /// - `LogLineError::AlreadyCommitted` se o LogLine já estiver `Committed`
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use logline_core::*;
    ///
    /// let line = LogLine::builder()
    ///     .who("did:ubl:alice")
    ///     .did(Verb::Deploy)
    ///     .when(1_735_671_234)
    ///     .if_ok(Outcome { label: "ok".into(), effects: vec![] })
    ///     .if_doubt(Escalation { label: "doubt".into(), route_to: "qa".into() })
    ///     .if_not(FailureHandling { label: "not".into(), action: "rollback".into() })
    ///     .build_draft()?;
    ///
    /// let ghost = line.abandon(Some("user_cancelled".into()))?;
    /// assert_eq!(ghost.status, Status::Ghost);
    /// # Ok::<(), LogLineError>(())
    /// ```
    pub fn abandon(self, reason: Option<String>) -> Result<GhostRecord, LogLineError> {
        match self.status {
            Status::Committed => Err(LogLineError::AlreadyCommitted),
            Status::Draft | Status::Pending => Ok(GhostRecord::from(self, reason)),
            _ => Ok(GhostRecord::from(self, reason)),
        }
    }

    /// Abandona intenção assinada: DRAFT/PENDING → GHOST (forensics).
    ///
    /// Paper I: attempt já nasce assinado, então o abandon também deve ser assinado.
    /// Versão recomendada que assina o LogLine antes de criar o GhostRecord.
    ///
    /// # Erros
    ///
    /// - `LogLineError::AlreadyCommitted` se o LogLine já estiver `Committed`
    /// - `LogLineError::Signing` se a assinatura falhar
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use logline_core::*;
    ///
    /// struct NoopSigner;
    /// impl Signer for NoopSigner {
    ///     fn sign(&self, _msg: &[u8]) -> Result<Signature, SignError> {
    ///         Ok(Signature { alg: "none".into(), bytes: vec![] })
    ///     }
    /// }
    ///
    /// let signer = NoopSigner;
    /// let line = LogLine::builder()
    ///     .who("did:ubl:alice")
    ///     .did(Verb::Deploy)
    ///     .when(1_735_671_234)
    ///     .if_ok(Outcome { label: "ok".into(), effects: vec![] })
    ///     .if_doubt(Escalation { label: "doubt".into(), route_to: "qa".into() })
    ///     .if_not(FailureHandling { label: "not".into(), action: "rollback".into() })
    ///     .build_draft()?;
    ///
    /// let ghost = line.abandon_signed(&signer, Some("timeout".into()))?;
    /// assert_eq!(ghost.status, Status::Ghost);
    /// # Ok::<(), LogLineError>(())
    /// ```
    pub fn abandon_signed(
        self,
        signer: &dyn Signer,
        reason: Option<String>,
    ) -> Result<GhostRecord, LogLineError> {
        match self.status {
            Status::Committed => Err(LogLineError::AlreadyCommitted),
            Status::Draft | Status::Pending => {
                let _sig = signer
                    .sign(&self.to_signable_bytes())
                    .map_err(|_| LogLineError::Signing)?;
                Ok(GhostRecord::from(self, reason))
            }
            _ => Ok(GhostRecord::from(self, reason)),
        }
    }

    /// Bytes determinísticos "suficientes" para v0.1 (placeholder).
    ///
    /// Gera uma representação determinística dos campos principais do LogLine
    /// para assinatura. Em versões futuras, isso será substituído por bytes canônicos
    /// JSON✯Atomic (via `json_atomic`).
    ///
    /// Formato atual: `who|verb|when|status|confirmed_by|this.kind`
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use logline_core::*;
    ///
    /// let line = LogLine::builder()
    ///     .who("did:ubl:alice")
    ///     .did(Verb::Approve)
    ///     .when(1_735_671_234)
    ///     .if_ok(Outcome { label: "ok".into(), effects: vec![] })
    ///     .if_doubt(Escalation { label: "doubt".into(), route_to: "auditor".into() })
    ///     .if_not(FailureHandling { label: "not".into(), action: "notify".into() })
    ///     .build_draft()?;
    ///
    /// let bytes = line.to_signable_bytes();
    /// assert!(!bytes.is_empty());
    /// # Ok::<(), LogLineError>(())
    /// ```
    pub fn to_signable_bytes(&self) -> Vec<u8> {
        // Ordem fixa: who|verb|when|status|confirmed_by|this.kind
        let mut out = Vec::new();
        out.extend_from_slice(self.who.as_bytes());
        out.extend_from_slice(b"|");
        out.extend_from_slice(self.did.as_str().as_bytes());
        out.extend_from_slice(b"|");
        out.extend_from_slice(self.when.to_string().as_bytes());
        out.extend_from_slice(b"|");
        out.extend_from_slice(self.status.as_str().as_bytes());
        out.extend_from_slice(b"|");
        if let Some(c) = &self.confirmed_by {
            out.extend_from_slice(c.as_bytes());
        }
        out.extend_from_slice(b"|");
        out.extend_from_slice(self.this.kind().as_bytes());
        out
    }
}
