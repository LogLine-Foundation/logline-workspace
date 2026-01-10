#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::multiple_crate_versions)]
//! SIRP network capsule + receipt handling (HTTP/server optional).
//!
//! Also provides minimal wire protocol via the `wire` module.

/// Capsule encode/decode.
pub mod capsule;
/// Idempotency store (sqlite).
#[cfg(feature = "sqlite")]
pub mod idempotency;
/// Receipt encode/verify.
pub mod receipt;
/// Minimal server/router.
#[cfg(feature = "server")]
pub mod server;
/// HTTP transport helpers.
#[cfg(feature = "http")]
pub mod transport_http;
/// SIRP Wire Protocol (TLV-based minimal frames).
pub mod wire;

pub use capsule::{build_capsule, parse_capsule, CapsuleBytes};
pub use receipt::{sign_receipt, verify_receipt, Receipt};
pub use wire::{
    decode_frame, encode_frame, CanonIntent, SirpError, SirpFrame, DOMAIN_FRAME_SIGN, FLAG_SIGNED,
    SIRP_MAGIC, SIRP_VERSION,
};

/// Convenience: build a `CanonIntent` from any JSON value.
///
/// Internally uses `json_atomic::canonize` for deterministic bytes
/// and `atomic_crypto::blake3_cid` for the CID.
///
/// # Errors
///
/// Returns error if canonicalization fails.
pub fn canon_intent_from_value(v: &serde_json::Value) -> Result<CanonIntent, SirpError> {
    let bytes = json_atomic::canonize(v).map_err(|e| SirpError::Canon(format!("{e:?}")))?;
    let cid = atomic_crypto::blake3_cid(&bytes);
    Ok(CanonIntent { cid, bytes })
}
