//! Application context wrapper for emitting UBL events during DIM processing.
use anyhow::Result;
use ubl_ledger::UblWriter;
use serde::Serialize;

/// Contexto do app: escritor do ledger.
pub struct AppCtx {
    writer: UblWriter,
}
impl AppCtx {
    /// Novo contexto
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(writer: UblWriter) -> Self {
        Self { writer }
    }
    /// Log genérico no UBL
    ///
    /// # Errors
    ///
    /// - Propaga falhas de serialização ou I/O do writer
    pub fn log<T: Serialize>(&self, kind: &str, payload: &T) -> Result<()> {
        self.writer.append(kind, payload, None, None)?;
        Ok(())
    }
    /// Eventos padrão
    ///
    /// # Errors
    ///
    /// - Propaga falhas de escrita no UBL
    pub fn received(&self, ev: &crate::events::IntentReceived) -> Result<()> {
        self.log("sirp.intent.received", ev)
    }
    /// Marca completude da intenção.
    ///
    /// # Errors
    ///
    /// - Propaga falhas de escrita no UBL
    pub fn completed(&self, ev: &crate::events::IntentCompleted) -> Result<()> {
        self.log("sirp.intent.completed", ev)
    }
}
