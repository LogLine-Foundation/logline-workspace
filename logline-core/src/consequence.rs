#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// Consequência positiva obrigatória (invariant do Paper I).
///
/// Define o que acontece quando a ação é executada com sucesso.
/// O campo `effects` lista as ações secundárias que devem ser executadas.
///
/// # Exemplo
///
/// ```rust
/// use logline_core::Outcome;
///
/// let outcome = Outcome {
///     label: "approved".into(),
///     effects: vec!["emit_receipt".into(), "notify_user".into()],
/// };
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Outcome {
    /// Rótulo descritivo da consequência.
    pub label: String,
    /// Lista de efeitos secundários a serem executados.
    pub effects: Vec<String>,
}

/// Via de dúvida obrigatória (invariant do Paper I).
///
/// Define para onde a ação deve ser encaminhada quando há dúvida sobre sua execução.
/// Pode ser um papel, fila, DID ou outro identificador de destino.
///
/// # Exemplo
///
/// ```rust
/// use logline_core::Escalation;
///
/// let escalation = Escalation {
///     label: "manual_review".into(),
///     route_to: "auditor".into(),
/// };
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Escalation {
    /// Rótulo descritivo da escalação.
    pub label: String,
    /// Destino da escalação (ex: role, queue, DID).
    pub route_to: String,
}

/// Fallback/erro obrigatório (invariant do Paper I).
///
/// Define o que acontece quando a ação falha. O campo `action` especifica
/// a ação de compensação ou notificação a ser executada.
///
/// # Exemplo
///
/// ```rust
/// use logline_core::FailureHandling;
///
/// let failure = FailureHandling {
///     label: "rejected".into(),
///     action: "compensate".into(),
/// };
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FailureHandling {
    /// Rótulo descritivo do tratamento de falha.
    pub label: String,
    /// Ação a ser executada (ex: "compensate", "refund", "notify").
    pub action: String,
}

impl Outcome {
    /// Verifica se o Outcome está vazio (invariant não satisfeito).
    ///
    /// Um Outcome vazio não satisfaz os invariants obrigatórios do Paper I.
    pub fn is_empty(&self) -> bool {
        self.label.is_empty()
    }
}
impl Escalation {
    /// Verifica se a Escalation está vazia (invariant não satisfeito).
    ///
    /// Uma Escalation vazia não satisfaz os invariants obrigatórios do Paper I.
    pub fn is_empty(&self) -> bool {
        self.label.is_empty() || self.route_to.is_empty()
    }
}
impl FailureHandling {
    /// Verifica se o FailureHandling está vazio (invariant não satisfeito).
    ///
    /// Um FailureHandling vazio não satisfaz os invariants obrigatórios do Paper I.
    pub fn is_empty(&self) -> bool {
        self.label.is_empty() || self.action.is_empty()
    }
}
