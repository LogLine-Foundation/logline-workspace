use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("dimension mismatch")]
    DimMismatch,
    #[error("empty index")]
    EmptyIndex,
    #[error("capsule parse error")]
    Capsule,
    #[error("serde error: {0}")]
    Serde(String),
    #[error("merkle error: {0}")]
    Merkle(String),
}
