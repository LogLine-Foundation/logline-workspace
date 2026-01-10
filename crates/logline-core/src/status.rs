/// Estado do lifecycle do `LogLine`.
///
/// O lifecycle é determinístico e permite apenas as seguintes transições:
/// - `Draft → Pending` (via `freeze()`)
/// - `Pending → Committed` (via `commit()`)
/// - `Draft/Pending → Ghost` (via `abandon()` ou `abandon_signed()`)
///
/// Uma vez `Committed`, o `LogLine` é imutável e não pode mais ser modificado.
///
/// # Exemplo
///
/// ```rust
/// use logline_core::Status;
///
/// let status = Status::Draft;
/// assert_eq!(status.as_str(), "DRAFT");
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Status {
    /// Estado inicial: `LogLine` em construção, ainda não validado.
    Draft,
    /// Estado intermediário: `LogLine` validado, pronto para commit ou abandon.
    Pending,
    /// Estado final: `LogLine` commitado e imutável.
    Committed,
    /// Estado forense: `LogLine` abandonado, preservado para análise.
    Ghost,
}

impl Status {
    /// Retorna a representação string do status (ex: "DRAFT", "PENDING").
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Pending => "PENDING",
            Self::Committed => "COMMITTED",
            Self::Ghost => "GHOST",
        }
    }
}

use crate::{LogLineError, Status as S};

pub fn ensure(expected_from: S, to: S, current: S) -> Result<(), LogLineError> {
    if current != expected_from {
        return Err(LogLineError::InvalidTransition { from: current, to });
    }
    Ok(())
}
