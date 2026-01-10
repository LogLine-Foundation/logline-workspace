#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::multiple_crate_versions)]
//! UBL — Universal Business Ledger (NDJSON)
//!
//! This crate provides:
//! - **Canonical bytes**: all intents serialized via `json_atomic::canonize`
//! - **Stable CID**: `Cid32 = BLAKE3(canonical_bytes)`
//! - **Optional signing**: domain `UBL:LEDGER:v1` + CID, via `atomic-crypto`
//! - **Append-only NDJSON**: writers and readers with verification
//!
//! ## Modules
//!
//! - [`ledger`]: Simplified API with `LedgerEntry`, `SimpleLedgerWriter`, `SimpleLedgerReader`
//! - [`writer`], [`reader`]: Full-featured writer/reader with rotation and WAL
//! - [`event`]: Event model
//! - [`verify`]: Verification helpers

/// Signing domain for UBL ledger entries.
pub const UBL_DOMAIN_SIGN: &[u8] = b"UBL:LEDGER:v1";

/// UBL event model.
pub mod event;
/// Simplified ledger API with canonical bytes and optional signing.
pub mod ledger;
/// Path helpers for UBL layout.
pub mod paths;
/// File reader utilities.
pub mod reader;
/// Verification helpers.
pub mod verify;
/// Synchronous writer.
pub mod writer;
#[cfg(feature = "async")]
/// Async writer.
pub mod writer_async;

pub use ledger::{
    AppendResult, FsyncPolicy, LedgerEntry, LedgerError, LedgerWriter, RotatePolicy,
    SimpleLedgerIter, SimpleLedgerReader, SimpleLedgerWriter,
};
pub use reader::{tail_file, UblIter, UblReader};
pub use writer::{Rotation, UblWriter};
#[cfg(feature = "async")]
pub use writer_async::{AsyncConfig, UblWriterAsync};
