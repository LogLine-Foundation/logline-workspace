//! Capsule encode/decode helpers: DIM (u16 BE) + canonical payload bytes.
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

/// Bytes da cápsula: DIM (u16, BE) + payload canônico.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleBytes(pub Vec<u8>);
impl AsRef<[u8]> for CapsuleBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Monta cápsula (P3: layout simples).
#[must_use]
pub fn build_capsule(dim: u16, payload: &[u8]) -> CapsuleBytes {
    let mut v = Vec::with_capacity(2 + payload.len());
    v.push(((dim >> 8) & 0xFF) as u8);
    v.push((dim & 0xFF) as u8);
    v.extend_from_slice(payload);
    CapsuleBytes(v)
}
/// Parseia cápsula (P3).
///
/// # Errors
///
/// - Retorna erro se os bytes forem menores que 2 ou inválidos
pub fn parse_capsule(bytes: &[u8]) -> Result<(u16, &[u8])> {
    ensure!(bytes.len() >= 2, "capsule too small");
    let dim = (u16::from(bytes[0]) << 8) | u16::from(bytes[1]);
    Ok((dim, &bytes[2..]))
}
