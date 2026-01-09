#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Calcula a confiança de trajetória usando similaridade de cosseno mapeada para \[0,1\].
///
/// Compara dois vetores de trajetória e retorna um valor de confiança:
/// - `1.0`: vetores idênticos (mesma direção e magnitude)
/// - `~0.5`: vetores ortogonais (sem correlação)
/// - `~0.0`: vetores opostos (direções opostas)
///
/// # Panics
///
/// Em modo debug, panics se os vetores tiverem tamanhos diferentes.
///
/// # Exemplo
///
/// ```rust
/// use json_atomic::trajectory_confidence;
///
/// let a = vec![1.0, 0.0, 0.0];
/// let b = vec![1.0, 0.0, 0.0];
/// assert_eq!(trajectory_confidence(&a, &b), 1.0);
///
/// let c = vec![1.0, 0.0];
/// let d = vec![0.0, 1.0];
/// let conf = trajectory_confidence(&c, &d);
/// assert!((conf - 0.5).abs() < 0.01); // ~0.5 para ortogonais
/// ```
pub fn trajectory_confidence(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    let cos = dot / (na.sqrt() * nb.sqrt());
    (cos + 1.0) * 0.5
}
