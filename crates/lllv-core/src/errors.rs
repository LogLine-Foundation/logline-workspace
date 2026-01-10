//! Error type for LLLV capsule parsing, crypto, and serialization helpers.
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
    #[error("timestamp overflow")]
    TimestampOverflow,
    #[error("serde error: {0}")]
    Serde(String),
}
