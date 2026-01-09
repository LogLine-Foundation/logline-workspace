use thiserror::Error;

#[derive(Error, Debug)]
pub enum LllvError {
    #[error("invalid header length")]
    InvalidHeaderLen,
    #[error("invalid magic")]
    InvalidMagic,
    #[error("invalid version")]
    InvalidVersion,
    #[error("mismatched_lengths")]
    MismatchedLengths,
    #[error("signature verification failed")]
    BadSignature,
    #[error("crypto error")]
    Crypto,
    #[error("serde error: {0}")]
    Serde(String),
}
