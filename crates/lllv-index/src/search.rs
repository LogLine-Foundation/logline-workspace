use crate::errors::IndexError;

pub fn cosine(a: &[f32], b: &[f32]) -> Result<f32, IndexError> {
    if a.len() != b.len() {
        return Err(IndexError::DimMismatch);
    }
    let mut dot = 0f32;
    let mut na = 0f32;
    let mut nb = 0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = (na.sqrt() * nb.sqrt()).max(1e-12);
    Ok(dot / denom)
}
