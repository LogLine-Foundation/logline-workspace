#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// Carga útil da ação (payload).
///
/// Paper I §3.1: o payload deve ser JSON estrito validado por schema do verbo.
/// Em v0.1.0, suporta `None`, `Text`, `Bytes` e `Json` (com feature `serde`).
///
/// # Exemplo
///
/// ```rust
/// use logline_core::Payload;
///
/// let none = Payload::None;
/// let text = Payload::Text("Hello".into());
/// let bytes = Payload::Bytes(vec![1, 2, 3]);
///
/// assert_eq!(none.kind(), "none");
/// assert_eq!(text.kind(), "text");
/// assert_eq!(bytes.kind(), "bytes");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Payload {
    /// Sem payload.
    None,
    /// Payload como string de texto.
    Text(String),
    /// Payload como bytes brutos.
    Bytes(Vec<u8>),
    /// JSON estrito validado por schema do verbo (Paper I §3.1).
    ///
    /// Disponível apenas com feature `serde`.
    #[cfg(feature = "serde")]
    Json(::serde_json::Value),
}

impl Payload {
    /// Retorna o tipo do payload como string ("none", "text", "bytes", "json").
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Text(_) => "text",
            Self::Bytes(_) => "bytes",
            #[cfg(feature = "serde")]
            Self::Json(_) => "json",
        }
    }
}
