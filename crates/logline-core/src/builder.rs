#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

use crate::{Escalation, FailureHandling, LogLine, LogLineError, Outcome, Payload, Status, Verb};

/// Builder para construir um `LogLine` passo a passo.
///
/// Use `LogLine::builder()` para criar um novo builder. Todos os campos
/// obrigatórios devem ser fornecidos antes de chamar `build_draft()`.
///
/// # Exemplo
///
/// ```rust
/// use logline_core::*;
///
/// let line = LogLine::builder()
///     .who("did:ubl:alice")
///     .did(Verb::Transfer)
///     .this(Payload::Text("100 USD".into()))
///     .when(1_735_671_234)
///     .if_ok(Outcome { label: "transferred".into(), effects: vec!["emit_receipt".into()] })
///     .if_doubt(Escalation { label: "verify".into(), route_to: "auditor".into() })
///     .if_not(FailureHandling { label: "failed".into(), action: "compensate".into() })
///     .build_draft()?;
/// # Ok::<(), LogLineError>(())
/// ```
#[derive(Default)]
#[must_use]
pub struct LogLineBuilder {
    who: Option<String>,
    did: Option<Verb>,
    this: Option<Payload>,
    when: Option<u64>,
    confirmed_by: Option<String>,
    if_ok: Option<Outcome>,
    if_doubt: Option<Escalation>,
    if_not: Option<FailureHandling>,
}

impl LogLineBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Define o agente que executa a ação (DID futuro, ex: `did:ubl:...`).
    pub fn who(mut self, v: impl Into<String>) -> Self {
        self.who = Some(v.into());
        self
    }
    /// Define o verbo da ação (canônico ou custom).
    pub fn did(mut self, v: Verb) -> Self {
        self.did = Some(v);
        self
    }
    /// Define a carga útil da ação (payload).
    pub fn this(mut self, v: Payload) -> Self {
        self.this = Some(v);
        self
    }
    /// Define o timestamp Unix em nanosegundos.
    pub const fn when(mut self, v: u64) -> Self {
        self.when = Some(v);
        self
    }
    /// Define a identidade que confirma a ação (opcional).
    ///
    /// Paper I: obrigatório para ações de Risk Level 3+ (L3+).
    pub fn confirmed_by(mut self, v: impl Into<String>) -> Self {
        self.confirmed_by = Some(v.into());
        self
    }
    /// Define a consequência positiva obrigatória (invariant).
    pub fn if_ok(mut self, v: Outcome) -> Self {
        self.if_ok = Some(v);
        self
    }
    /// Define a via de dúvida obrigatória (invariant).
    pub fn if_doubt(mut self, v: Escalation) -> Self {
        self.if_doubt = Some(v);
        self
    }
    /// Define o fallback/erro obrigatório (invariant).
    pub fn if_not(mut self, v: FailureHandling) -> Self {
        self.if_not = Some(v);
        self
    }

    /// Constrói um DRAFT válido (invariants obrigatórios presentes).
    ///
    /// Valida que todos os campos obrigatórios estão presentes e cria um `LogLine`
    /// com status `Draft`. Os invariants são verificados antes de retornar.
    ///
    /// # Errors
    ///
    /// - `LogLineError::MissingField` se algum campo obrigatório estiver faltando
    /// - `LogLineError::MissingInvariant` se algum invariant estiver faltando ou vazio
    ///
    /// # Campos obrigatórios
    ///
    /// - `who`: identidade do agente
    /// - `did`: verbo da ação
    /// - `when`: timestamp
    /// - `if_ok`: consequência positiva
    /// - `if_doubt`: via de dúvida
    /// - `if_not`: fallback/erro
    pub fn build_draft(self) -> Result<LogLine, LogLineError> {
        let line = LogLine {
            who: self.who.ok_or(LogLineError::MissingField("who"))?,
            did: self.did.ok_or(LogLineError::MissingField("did"))?,
            this: self.this.unwrap_or(Payload::None),
            when: self.when.ok_or(LogLineError::MissingField("when"))?,
            confirmed_by: self.confirmed_by,
            if_ok: self.if_ok.ok_or(LogLineError::MissingInvariant("if_ok"))?,
            if_doubt: self
                .if_doubt
                .ok_or(LogLineError::MissingInvariant("if_doubt"))?,
            if_not: self
                .if_not
                .ok_or(LogLineError::MissingInvariant("if_not"))?,
            status: Status::Draft,
        };
        line.verify_invariants()?;
        Ok(line)
    }
}
