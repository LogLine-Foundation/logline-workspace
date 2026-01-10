#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Canonical JSON✯Atomic encoding/decoding helpers + binary TLV codec.
//!
//! This crate provides two complementary codecs:
//!
//! - **JSON✯Atomic** (this module): Canonical JSON serialization with BLAKE3 CIDs
//! - **Binary TLV** ([`binary`] module): Compact binary encoding for SIRP/frames

use serde::{de::DeserializeOwned, Serialize};
use std::io::Read;
use thiserror::Error;

pub mod binary;
pub use binary::{
    decode_frame, decode_varint_u64, encode_frame, encode_varint_u64, BinaryCodecError, Decoder,
    Encoder, T_BYTES, T_CID32, T_PUBKEY32, T_SIG64, T_STR, T_U64,
};

/// Errors returned by the codec helpers.
#[derive(Debug, Error)]
pub enum AtomicCodecError {
    /// Serialization/deserialization error.
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
    /// Canonicalization failure.
    #[error("canon: {0}")]
    Canon(String),
    /// YAML conversion failure.
    #[error("yaml: {0}")]
    Yaml(String),
}

/// Serializa um valor JSON para bytes canônicos JSON✯Atomic.
///
/// # Errors
///
/// - `AtomicCodecError::Serde` se a conversão para `Value` falhar
/// - `AtomicCodecError::Canon` se a canonicalização JSON✯Atomic falhar
pub fn to_canon_vec<T: Serialize>(v: &T) -> Result<Vec<u8>, AtomicCodecError> {
    let val = serde_json::to_value(v)?;
    json_atomic::canonize(&val).map_err(|e| AtomicCodecError::Canon(e.to_string()))
}

/// Desserializa de bytes canônicos para um tipo.
///
/// # Errors
///
/// - `AtomicCodecError::Serde` se o parse ou a desserialização falhar
pub fn from_canon_slice<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, AtomicCodecError> {
    Ok(serde_json::from_slice(bytes)?)
}

/// Converte JSON em string para bytes canônicos.
///
/// # Errors
///
/// - `AtomicCodecError::Serde` se o JSON for inválido
/// - `AtomicCodecError::Canon` se a canonicalização JSON✯Atomic falhar
pub fn from_json_str_canon(s: &str) -> Result<Vec<u8>, AtomicCodecError> {
    let v: serde_json::Value = serde_json::from_str(s)?;
    json_atomic::canonize(&v).map_err(|e| AtomicCodecError::Canon(e.to_string()))
}

/// Calcula o CID hex (BLAKE3) de um valor serializável.
///
/// # Errors
///
/// - Propaga os mesmos erros de [`to_canon_vec`]
pub fn to_cid_hex<T: Serialize>(v: &T) -> Result<String, AtomicCodecError> {
    let b = to_canon_vec(v)?;
    Ok(blake3::hash(&b).to_hex().to_string())
}

/// Valor e seus bytes canônicos já calculados.
pub struct Canonical<T> {
    value: T,
    bytes: Vec<u8>,
}
impl<T> Canonical<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Cria a partir de um valor serializável.
    ///
    /// # Errors
    ///
    /// - `AtomicCodecError::Serde` se a serialização falhar
    /// - `AtomicCodecError::Canon` se a canonicalização JSON✯Atomic falhar
    pub fn new(value: T) -> Result<Self, AtomicCodecError> {
        let bytes = to_canon_vec(&value)?;
        Ok(Self { value, bytes })
    }
    /// Lê de um reader de texto, parseia e canonicaliza.
    ///
    /// # Errors
    ///
    /// - `AtomicCodecError::Canon` se a leitura ou canonicalização falhar
    /// - `AtomicCodecError::Serde` se o JSON for inválido ou desserialização falhar
    pub fn from_reader<R: Read>(mut r: R) -> Result<Self, AtomicCodecError> {
        let mut s = String::new();
        r.read_to_string(&mut s)
            .map_err(|e| AtomicCodecError::Canon(e.to_string()))?;
        let v: serde_json::Value = serde_json::from_str(&s)?;
        let bytes =
            json_atomic::canonize(&v).map_err(|e| AtomicCodecError::Canon(e.to_string()))?;
        Ok(Self {
            value: serde_json::from_value(v)?,
            bytes,
        })
    }
    /// Referência ao valor.
    pub const fn value(&self) -> &T {
        &self.value
    }
    /// Bytes canônicos.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Consome e retorna os bytes canônicos.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// Retorna true se a string JSON já está na forma canônica JSON✯Atomic.
#[must_use]
pub fn is_canonical(s: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(s)
        .ok()
        .and_then(|v| json_atomic::canonize(&v).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .is_some_and(|canon| canon == s.trim())
}

/// Converte YAML (subset) → bytes canônicos JSON✯Atomic.
///
/// # Errors
///
/// - `AtomicCodecError::Yaml` se o YAML for inválido ou não puder ser convertido para JSON
/// - `AtomicCodecError::Canon` se a canonicalização JSON✯Atomic falhar
pub fn yaml_to_canon_vec(yaml: &str) -> Result<Vec<u8>, AtomicCodecError> {
    let v: serde_json::Value = match serde_yaml::from_str::<serde_yaml::Value>(yaml) {
        Ok(doc) => serde_json::to_value(doc).map_err(|e| AtomicCodecError::Yaml(e.to_string()))?,
        Err(e) => return Err(AtomicCodecError::Yaml(e.to_string())),
    };
    json_atomic::canonize(&v).map_err(|e| AtomicCodecError::Canon(e.to_string()))
}
