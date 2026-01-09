#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

/// Verbo que descreve a ação executada.
///
/// Paper I §3.1: verbos devem ser validados contra ALLOWED_ACTIONS via `VerbRegistry`.
/// Verbos canônicos são `Transfer`, `Deploy`, `Approve`. Verbos customizados são permitidos
/// via `Custom(String)`, mas devem passar pela validação do registry.
///
/// # Exemplo
///
/// ```rust
/// use logline_core::Verb;
///
/// let canonical = Verb::Transfer;
/// let custom = Verb::Custom("approve_budget".into());
///
/// assert_eq!(canonical.as_str(), "transfer");
/// assert_eq!(custom.as_str(), "approve_budget");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Verb {
    /// Transferência de recursos (ex: dinheiro, tokens, dados).
    Transfer,
    /// Deploy de artefato (ex: código, configuração, serviço).
    Deploy,
    /// Aprovação de ação ou decisão.
    Approve,
    /// Verbo customizado (deve ser validado via `VerbRegistry`).
    Custom(String),
}

impl Verb {
    /// Retorna a representação string do verbo.
    ///
    /// Verbos canônicos retornam suas strings fixas, enquanto `Custom` retorna
    /// a string fornecida.
    pub fn as_str(&self) -> &str {
        match self {
            Verb::Transfer => "transfer",
            Verb::Deploy => "deploy",
            Verb::Approve => "approve",
            Verb::Custom(s) => s.as_str(),
        }
    }
}

/// Registry para validar verbos contra ALLOWED_ACTIONS (Paper I §3.1).
/// Implementações devem verificar se o verbo está no registro permitido.
pub trait VerbRegistry {
    /// Retorna `true` se o verbo está permitido no sistema.
    fn is_allowed(&self, verb: &Verb) -> bool;
}
