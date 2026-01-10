#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
//! Shared atomic identifiers, DIM helpers, and parsers reused across the stack.

extern crate alloc;
use alloc::string::{String, ToString};
use serde::{Deserialize, Serialize};

#[cfg(feature = "strict")]
use regex::Regex;

/// Macro para newtypes de IDs simples (String-based).
#[macro_export]
macro_rules! newtype_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        #[doc = concat!(stringify!($name), " identifier newtype (string, trimmed; strict regex when feature=\"strict\").")]
        pub struct $name(pub String);
        impl core::fmt::Display for $name {
            fn fmt(&self, f:&mut core::fmt::Formatter<'_>)->core::fmt::Result{ f.write_str(&self.0) }
        }
        impl core::str::FromStr for $name {
            type Err = &'static str;
            fn from_str(s:&str)->Result<Self,Self::Err>{
                let s=s.trim();
                if s.is_empty(){return Err("empty");}
                #[cfg(feature="strict")]
                {
                    // ids alfanuméricos, -, _, :, . (simples e estável)
                    static PAT: &str = r"^[A-Za-z0-9._:-]{1,128}$";
                    let re = Regex::new(PAT).unwrap();
                    if !re.is_match(s){ return Err("invalid chars"); }
                }
                Ok(Self(s.to_string()))
            }
        }
    };
}

newtype_id!(AppId);
newtype_id!(TenantId);
newtype_id!(NodeId);
newtype_id!(ActorId);
newtype_id!(TraceId);

/// Dimensão (DIM) do protocolo (u16, big-endian no fio).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dim(pub u16);
impl Dim {
    /// Converte "0x00A1" ou "161" para `Dim`.
    ///
    /// # Errors
    ///
    /// Retorna erro se o valor não for hexadecimal ou decimal válido.
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        let s = s.trim();
        s.strip_prefix("0x").map_or_else(
            || s.parse::<u16>().map(Dim).map_err(|_| "bad dec"),
            |h| u16::from_str_radix(h, 16).map(Dim).map_err(|_| "bad hex"),
        )
    }
    /// Cria a partir de string hexa (sem "0x").
    ///
    /// # Errors
    ///
    /// Retorna erro se o valor não for hexadecimal válido.
    pub fn from_hex(h: &str) -> Result<Self, &'static str> {
        u16::from_str_radix(h, 16).map(Dim).map_err(|_| "bad hex")
    }
    /// Representação "0xHHHH".
    #[must_use]
    pub fn to_hex(self) -> alloc::string::String {
        alloc::format!("0x{:04X}", self.0)
    }
    /// Valor bruto.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

#[cfg(feature = "ulid")]
/// Geradores ULID para IDs comuns.
pub mod gen {
    use super::{ActorId, TraceId};
    /// Gera `TraceId` com ULID.
    #[must_use]
    pub fn new_ulid_trace() -> TraceId {
        TraceId(ulid::Ulid::new().to_string())
    }
    /// Gera `ActorId` com ULID.
    #[must_use]
    pub fn new_ulid_actor() -> ActorId {
        ActorId(ulid::Ulid::new().to_string())
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Cryptographic primitive wrappers (hex-serialized)
// ══════════════════════════════════════════════════════════════════════════════

use core::fmt;
use serde::{Deserializer, Serializer};
use thiserror::Error;

/// 32-byte Content ID (BLAKE3 hash). Serializes as lowercase hex (no `0x` prefix).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Cid32(pub [u8; 32]);

impl fmt::Debug for Cid32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cid32({})", hex::encode(self.0))
    }
}
impl fmt::Display for Cid32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}
impl Serialize for Cid32 {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(self.0))
    }
}
impl<'de> Deserialize<'de> for Cid32 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = <&str>::deserialize(d)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Cid32 must be 32 bytes"));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(Cid32(out))
    }
}
impl Cid32 {
    /// Creates a `Cid32` from a hex string.
    ///
    /// # Errors
    ///
    /// Returns `AtomError::Hex` if decoding fails or `AtomError::Length` if not 32 bytes.
    pub fn from_hex(s: &str) -> Result<Self, AtomError> {
        let bytes = hex::decode(s).map_err(|_| AtomError::Hex)?;
        if bytes.len() != 32 {
            return Err(AtomError::Length { expected: 32, actual: bytes.len() });
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(Cid32(out))
    }
    /// Returns the hex representation.
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

/// 32-byte Ed25519 public key. Serializes as lowercase hex.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PublicKeyBytes(pub [u8; 32]);

impl fmt::Debug for PublicKeyBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKeyBytes({})", hex::encode(self.0))
    }
}
impl fmt::Display for PublicKeyBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}
impl Serialize for PublicKeyBytes {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(self.0))
    }
}
impl<'de> Deserialize<'de> for PublicKeyBytes {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = <&str>::deserialize(d)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("PublicKeyBytes must be 32 bytes"));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(PublicKeyBytes(out))
    }
}

/// 64-byte Ed25519 signature. Serializes as lowercase hex.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignatureBytes(pub [u8; 64]);

impl Default for SignatureBytes {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl fmt::Debug for SignatureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignatureBytes({}..)", &hex::encode(self.0)[..16])
    }
}
impl fmt::Display for SignatureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}
impl Serialize for SignatureBytes {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(self.0))
    }
}
impl<'de> Deserialize<'de> for SignatureBytes {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = <&str>::deserialize(d)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("SignatureBytes must be 64 bytes"));
        }
        let mut out = [0u8; 64];
        out.copy_from_slice(&bytes);
        Ok(SignatureBytes(out))
    }
}

/// Textual intent with canonical bytes (whitespace-insensitive).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intent {
    /// Original text received (for audit trail).
    pub raw: String,
    /// Canonical bytes (e.g., NFC + collapsed whitespace).
    pub canon: alloc::vec::Vec<u8>,
}

impl Intent {
    /// Creates an intent, collapsing whitespace deterministically.
    #[must_use]
    pub fn from_raw(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let canon = normalize_ws(&raw).into_bytes();
        Self { raw, canon }
    }
    /// Returns canonical bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.canon
    }
}

/// Deterministic whitespace normalization (ASCII-first, collapse runs).
fn normalize_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        let is_space = ch.is_whitespace();
        if is_space {
            if !prev_space {
                out.push(' ');
            }
        } else {
            out.push(ch);
        }
        prev_space = is_space;
    }
    out.trim().to_string()
}

/// Shared basic errors for atomic types.
#[derive(Debug, Error)]
pub enum AtomError {
    /// Hex decoding failed.
    #[error("hex decode error")]
    Hex,
    /// Length mismatch.
    #[error("length mismatch: expected {expected}, got {actual}")]
    Length {
        /// Expected length.
        expected: usize,
        /// Actual length.
        actual: usize,
    },
    /// Invalid intent format.
    #[error("invalid intent")]
    InvalidIntent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cid32_roundtrip() {
        let c = Cid32([0xAB; 32]);
        let j = serde_json::to_string(&c).unwrap();
        assert_eq!(j.len(), 66); // " + 64 hex + "
        let de: Cid32 = serde_json::from_str(&j).unwrap();
        assert_eq!(c.0, de.0);
    }

    #[test]
    fn intent_ws_canonical() {
        let i1 = Intent::from_raw("  hello   world ");
        let i2 = Intent::from_raw("hello world");
        assert_eq!(i1.canon, i2.canon);
    }

    #[test]
    fn pk_sig_roundtrip() {
        let pk = PublicKeyBytes([0x22; 32]);
        let sig = SignatureBytes([0x33; 64]);
        let jp = serde_json::to_string(&pk).unwrap();
        let js = serde_json::to_string(&sig).unwrap();
        let _dpk: PublicKeyBytes = serde_json::from_str(&jp).unwrap();
        let _ds: SignatureBytes = serde_json::from_str(&js).unwrap();
    }
}
