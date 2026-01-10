#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
//! LLLV capsule format primitives: header, manifest, crypto, and helpers.

#[cfg(not(feature = "std"))]
extern crate alloc;

mod capsule;
mod crypto;
mod errors;
mod header;
mod manifest;
mod version;

pub use capsule::Capsule;
pub use crypto::{decrypt_chacha20poly1305, encrypt_chacha20poly1305};
pub use errors::LllvError;
pub use header::{CapsuleFlags, CapsuleHeader};
pub use manifest::{seal_manifest, CapsuleManifest};
pub use version::{CAP_MAGIC, CAP_VER, HEADER_LEN};

#[cfg_attr(docsrs, doc = include_str!("../README.md"))]
const _: &str = "";
