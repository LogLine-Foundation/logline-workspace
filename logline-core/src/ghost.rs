#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};
#[cfg(feature = "std")]
use std::{boxed::Box, string::String};

use crate::{LogLine, Status};

/// Registro forense de um LogLine abandonado.
///
/// Preserva o LogLine original que foi abandonado (não commitado) para análise forense.
/// O status é sempre `Ghost` e pode incluir uma razão opcional do abandono.
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
/// assert_eq!(ghost.reason, Some("user_cancelled".into()));
/// # Ok::<(), LogLineError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GhostRecord {
    /// O LogLine original que foi abandonado.
    pub inner: Box<LogLine>,
    /// Status do registro (sempre `Ghost`).
    pub status: Status,
    /// Razão opcional do abandono (para análise forense).
    pub reason: Option<String>,
}

impl GhostRecord {
    /// Cria um `GhostRecord` a partir de um `LogLine` e uma razão opcional.
    ///
    /// O status do LogLine é alterado para `Ghost` antes de ser encapsulado.
    pub fn from(mut line: LogLine, reason: Option<String>) -> Self {
        line.status = Status::Ghost;
        GhostRecord {
            inner: Box::new(line),
            status: Status::Ghost,
            reason,
        }
    }
}
