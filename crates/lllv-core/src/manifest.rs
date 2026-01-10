//! Capsule manifest struct and sealing helper for signed facts.
use crate::errors::LllvError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleManifest {
    pub vector_id: String,
    pub source_uri: String,
    pub mime: String,
    pub content_hash: String, // blake3 do conteúdo de origem (não a cápsula)
    pub dim: u16,
    pub quant: String,      // ex.: "q8"
    pub encoder: String,    // ex.: "text-embedding-3-large"
    pub policy_ref: String, // ex.: "tdln://policy/default"
    pub ts_ingest: String,  // ISO8601
}

impl CapsuleManifest {
    #[must_use]
    pub fn minimal(id: &str, mime: &str, dim: u16, quant: &str) -> Self {
        Self {
            vector_id: id.into(),
            source_uri: String::new(),
            mime: mime.into(),
            content_hash: String::new(),
            dim,
            quant: quant.into(),
            encoder: String::new(),
            policy_ref: String::new(),
            ts_ingest: String::new(),
        }
    }
}

#[cfg(feature = "manifest")]
/// Sela um `CapsuleManifest` em um `SignedFact`.
///
/// # Errors
///
/// - `LllvError::Serde` se a selagem falhar
pub fn seal_manifest(
    mf: &CapsuleManifest,
    sk: &ed25519_dalek::SigningKey,
) -> Result<json_atomic::SignedFact, LllvError> {
    use json_atomic::seal_value;
    seal_value(mf, sk).map_err(|_| LllvError::Serde("seal".into()))
}
