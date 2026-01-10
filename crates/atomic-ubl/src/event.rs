//! UBL NDJSON event schema with signatures and linkage.
use atomic_types::{ActorId, AppId, NodeId, TenantId, TraceId};
use serde::{Deserialize, Serialize};

/// Evento do UBL (NDJSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UblEvent {
    /// Timestamp (RFC3339, UTC)
    pub ts: String,
    /// App / Tenant / Node / Actor
    pub app: AppId,
    /// Tenant que originou o evento.
    pub tenant: TenantId,
    /// Nó emissor.
    pub node: NodeId,
    /// Ator (ex.: microserviço ou usuário).
    pub actor: ActorId,
    /// Tipo (família.evento)
    #[serde(rename = "type")]
    pub kind: String,
    /// Trace opcional
    pub trace_id: Option<TraceId>,
    /// Referências soltas
    pub refs: Vec<String>,
    /// Payload (canônico, em base64)
    pub payload: serde_json::Value,
    /// Payload canônico (base64url).
    pub canon_b64: String,
    /// CID BLAKE3 em hex.
    pub cid_hex: String,
    /// Assinatura Ed25519 em base64url.
    pub sig_b64: String,
    /// Chave pública em base64url.
    pub pk_b64: String,
    /// Identificador da chave.
    pub key_id: String,
    /// Encadeamento (CID anterior no mesmo arquivo)
    pub prev_cid_hex: Option<String>,
}
