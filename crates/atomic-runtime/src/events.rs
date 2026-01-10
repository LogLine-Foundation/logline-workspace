//! Structured events emitted to UBL during DIM processing.
use serde::{Deserialize, Serialize};
/// Evento de entrada de intenção.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentReceived {
    /// DIM da intenção.
    pub dim: u16,
    /// CID da cápsula.
    pub capsule_cid_hex: String,
    /// Tamanho da cápsula em bytes.
    pub size: u64,
}

/// Evento de conclusão da intenção.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentCompleted {
    /// DIM da intenção.
    pub dim: u16,
    /// CID da cápsula.
    pub capsule_cid_hex: String,
    /// Indicador de sucesso.
    pub ok: bool,
    /// Tamanho do resultado em bytes.
    pub result_size: u64,
}
