//! HTTP helpers for DIM capsules encoded as octet streams.
use anyhow::{ensure, Result};
use ubl_types::Dim;
/// Extrai DIM do corpo HTTP (octet-stream DIM+payload)
///
/// # Errors
///
/// - Retorna erro se o corpo for menor que 2 bytes
pub fn parse_http_octets(body: &[u8]) -> Result<(Dim, &[u8])> {
    ensure!(body.len() >= 2, "body too small");
    let dim = (u16::from(body[0]) << 8) | u16::from(body[1]);
    Ok((Dim(dim), &body[2..]))
}
