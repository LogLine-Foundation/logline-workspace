//! # json_atomic
//!
//! The Cryptographic Atom — JSON✯Atomic: canonicalization + BLAKE3 + DV25-Seal (Ed25519)
//!
//! This crate implements the **Cryptographic Atom** from Paper II, providing:
//! - **Canonical JSON✯Atomic** serialization (Same Semantics = Same Bytes = Same Hash)
//! - **BLAKE3** content addressing (CID)
//! - **DV25-Seal (Ed25519)** for Signed Facts
//! - **Trajectory Matching** for similarity calculations
//!
//! See [README.md](https://github.com/LogLine-Foundation/json_atomic/blob/main/README.md) for full documentation.

#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod canonicalize;
mod cycle;
pub mod errors;
mod signed_fact;
mod trajectory;
mod version;

pub use canonicalize::canonize;
pub use cycle::{seal_logline, seal_value, verify_seal};
pub use signed_fact::SignedFact;
pub use trajectory::trajectory_confidence;
pub use version::{CANON_VERSION, FORMAT_ID};
