//! Merkle tree hardened: domínio explícito, provas com lado, checagens completas,
//! API compatível com nomes antigos (leaf_hash, merkle_root, proof_for, verify_path).

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::errors::IndexError;
use blake3::Hasher;

// --- Domínio explícito p/ evitar colisões entre folhas e nós ---
const NODE_PREFIX: &[u8] = b"\x01node";
const LEAF_PREFIX: &[u8] = b"\x00leaf";
const DOC_PREFIX: &[u8] = b"\x00doc";

/// Passo da prova: hash do irmão + indicador se o irmão está à direita.
/// - `sibling_is_right = true`  => parent = H(node || cur || sibling)
/// - `sibling_is_right = false` => parent = H(node || sibling || cur)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofStep {
    pub sibling: [u8; 32],
    pub sibling_is_right: bool,
}

/// Prova Merkle como uma sequência de passos.
pub type Proof = Vec<ProofStep>;

// ----------------- Primitivas internas -----------------

#[inline]
fn blake3_bytes(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Hasher::new();
    for p in parts {
        h.update(p);
    }
    *h.finalize().as_bytes()
}

#[inline]
fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    blake3_bytes(&[NODE_PREFIX, left, right])
}

/// Hash de folha com domínio.
#[inline]
pub fn leaf_hash(data: &[u8]) -> [u8; 32] {
    blake3_bytes(&[LEAF_PREFIX, data])
}

/// Hash de folha para documento: H(LEAF_PREFIX || DOC_PREFIX || id || payload_cid)
#[inline]
pub fn leaf_hash_doc(id: &str, cid: &[u8; 32]) -> [u8; 32] {
    blake3_bytes(&[LEAF_PREFIX, DOC_PREFIX, id.as_bytes(), cid])
}

/// Constrói os níveis da árvore, duplicando a última folha quando necessário.
/// Retorna erro se `leaves` estiver vazio.
fn build_levels(leaves: &[[u8; 32]]) -> Result<Vec<Vec<[u8; 32]>>, IndexError> {
    if leaves.is_empty() {
        return Err(IndexError::Merkle("no leaves".into()));
    }
    let mut levels: Vec<Vec<[u8; 32]>> = Vec::new();
    levels.push(leaves.to_vec());
    while levels.last().unwrap().len() > 1 {
        let cur = levels.last().unwrap();
        let mut next = Vec::with_capacity(cur.len().div_ceil(2));
        for chunk in cur.chunks(2) {
            let l = chunk[0];
            let r = *chunk.get(1).unwrap_or(&l);
            next.push(node_hash(&l, &r));
        }
        levels.push(next);
    }
    Ok(levels)
}

// ----------------- API hardened (Result) -----------------

/// Raiz Merkle (checada).
pub fn root_checked(leaves: &[[u8; 32]]) -> Result<[u8; 32], IndexError> {
    let levels = build_levels(leaves)?;
    Ok(levels.last().unwrap()[0])
}

/// Prova Merkle (checada) para `idx`.
pub fn prove_checked(leaves: &[[u8; 32]], mut idx: usize) -> Result<([u8; 32], Proof), IndexError> {
    let levels = build_levels(leaves)?;
    if idx >= levels[0].len() {
        return Err(IndexError::Merkle("index out of range".into()));
    }

    let mut proof: Proof = Vec::new();
    for cur in levels.iter().take(levels.len() - 1) {
        let is_left = idx % 2 == 0;
        let sibling = if is_left {
            if idx + 1 < cur.len() {
                cur[idx + 1]
            } else {
                cur[idx]
            }
        } else {
            cur[idx - 1]
        };
        proof.push(ProofStep {
            sibling,
            sibling_is_right: is_left,
        });
        idx /= 2;
    }

    Ok((levels.last().unwrap()[0], proof))
}

/// Verifica caminho Merkle contra a raiz esperada.
pub fn verify_path_checked(
    mut cur: [u8; 32],
    path: &[ProofStep],
    expected_root: [u8; 32],
) -> Result<(), IndexError> {
    for step in path {
        cur = if step.sibling_is_right {
            node_hash(&cur, &step.sibling)
        } else {
            node_hash(&step.sibling, &cur)
        };
    }
    if cur == expected_root {
        Ok(())
    } else {
        Err(IndexError::Merkle("invalid proof path".into()))
    }
}

// ----------------- Camada de compatibilidade (nomes antigos) -----------------

/// Compat: raiz Merkle (panica se vazio) — use `root_checked` em código novo.
pub fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    root_checked(leaves).expect("merkle_root: empty leaves")
}

/// Compat: prova (panica se erro) — use `prove_checked` em código novo.
pub fn proof_for(leaves: &[[u8; 32]], idx: usize) -> Vec<ProofStep> {
    let (_root, p) = prove_checked(leaves, idx).expect("proof_for: invalid input");
    p
}

/// Compat: verificação (mantém Result) — idêntico ao antigo.
pub fn verify_path(
    leaf: [u8; 32],
    path: &[ProofStep],
    expected_root: [u8; 32],
) -> Result<(), IndexError> {
    verify_path_checked(leaf, path, expected_root)
}

// leaf_hash e leaf_hash_doc já estão exportadas acima

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_leaf() {
        let h = leaf_hash(b"A");
        let r = merkle_root(&[h]);
        assert_eq!(h, r);

        let (_root, p) = prove_checked(&[h], 0).unwrap();
        verify_path_checked(h, &p, r).unwrap();
    }

    #[test]
    fn three_leaves_dup_semantics() {
        let h1 = leaf_hash(b"A");
        let h2 = leaf_hash(b"B");
        let h3 = leaf_hash(b"C");
        let leaves = vec![h1, h2, h3];

        let r = merkle_root(&leaves);
        let (r0, p0) = prove_checked(&leaves, 0).unwrap();
        let (r1, p1) = prove_checked(&leaves, 1).unwrap();
        let (r2, p2) = prove_checked(&leaves, 2).unwrap();

        assert_eq!(r, r0);
        assert_eq!(r, r1);
        assert_eq!(r, r2);

        verify_path(h1, &p0, r).unwrap();
        verify_path(h2, &p1, r).unwrap();
        verify_path(h3, &p2, r).unwrap();
    }

    #[test]
    fn tamper_detection() {
        let h1 = leaf_hash(b"A");
        let h2 = leaf_hash(b"B");
        let leaves = vec![h1, h2];

        let r = merkle_root(&leaves);
        let (_r0, p0) = prove_checked(&leaves, 0).unwrap();

        // corrompe um byte
        let mut bad = p0.clone();
        bad[0].sibling[0] ^= 0xFF;

        assert!(verify_path(h1, &bad, r).is_err());
    }
}
