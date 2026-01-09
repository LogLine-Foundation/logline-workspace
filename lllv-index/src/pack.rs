#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use crate::{
    errors::IndexError,
    evidence::{ProofStepSerde, QueryEvidence, QueryRequest, QueryResult},
    hash::hex32,
    merkle::{leaf_hash_doc, merkle_root, proof_for, verify_path, ProofStep},
    search::cosine,
};
use hex::FromHex;
use lllv_core::Capsule;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct DocEntry {
    id: String,
    payload_cid_hex: String,
}

#[inline]
fn hex_to_32(s: &str) -> Result<[u8; 32], IndexError> {
    let bytes =
        <[u8; 32]>::from_hex(s).map_err(|_| IndexError::Merkle("bad hex (expected 32B)".into()))?;
    Ok(bytes)
}

pub struct IndexPack {
    pub dim: u16,
    ids: Vec<String>,
    vecs: Vec<Vec<f32>>,
    // merkle sobre folhas DOC(id, payload_cid)
    leaves: Vec<[u8; 32]>,
    root: [u8; 32],
    // manifesto opcional (Paper II)
    #[cfg(feature = "manifest")]
    pub manifest: Option<json_atomic::SignedFact>,
}

pub struct IndexPackBuilder {
    dim: u16,
    entries: Vec<(String, Capsule)>,
}

impl IndexPackBuilder {
    pub fn new(dim: u16) -> Self {
        Self {
            dim,
            entries: Vec::new(),
        }
    }

    pub fn add_capsule(&mut self, id: String, cap: Capsule) -> Result<(), IndexError> {
        if cap.header.dim != self.dim {
            return Err(IndexError::DimMismatch);
        }
        self.entries.push((id, cap));
        Ok(())
    }

    pub fn build(&self, sk: Option<&ed25519_dalek::SigningKey>) -> Result<IndexPack, IndexError> {
        if self.entries.is_empty() {
            return Err(IndexError::EmptyIndex);
        }

        // extrair vetores e ids
        let mut ids = Vec::with_capacity(self.entries.len());
        let mut vecs = Vec::with_capacity(self.entries.len());
        let mut leaves = Vec::with_capacity(self.entries.len());

        for (id, cap) in &self.entries {
            let bytes = &cap.payload;
            let need = (self.dim as usize) * core::mem::size_of::<f32>();
            if bytes.len() != need {
                return Err(IndexError::Capsule);
            }
            // bytes -> f32
            let mut v = vec![0f32; self.dim as usize];
            for (i, slot) in v.iter_mut().enumerate() {
                let start = i * 4;
                *slot = f32::from_le_bytes(bytes[start..start + 4].try_into().unwrap());
            }
            let leaf = leaf_hash_doc(id, &cap.header.cid);
            ids.push(id.clone());
            vecs.push(v);
            leaves.push(leaf);
        }

        let root = merkle_root(&leaves);

        #[cfg(feature = "manifest")]
        let manifest = if let Some(sk) = sk {
            // manifesto canônico mínimo do pack
            #[derive(Serialize)]
            struct Manifest<'a> {
                r#type: &'static str,
                dim: u16,
                n: usize,
                merkle_root_hex: String,
                ids: &'a [String],
            }
            let data = Manifest {
                r#type: "LLLV_INDEX_PACK_V1",
                dim: self.dim,
                n: ids.len(),
                merkle_root_hex: hex32(&root),
                ids: &ids,
            };
            Some(
                json_atomic::seal_value(&data, sk)
                    .map_err(|e| IndexError::Serde(format!("seal: {e:?}")))?,
            )
        } else {
            None
        };

        Ok(IndexPack {
            dim: self.dim,
            ids,
            vecs,
            leaves,
            root,
            #[cfg(feature = "manifest")]
            manifest,
        })
    }
}

impl IndexPack {
    pub fn index_pack_cid_hex(&self) -> String {
        hex32(&self.root)
    }

    pub fn query(&self, req: &QueryRequest, topk: usize) -> Result<QueryEvidence, IndexError> {
        if self.vecs.is_empty() {
            return Err(IndexError::EmptyIndex);
        }
        if req.vec.len() != self.dim as usize {
            return Err(IndexError::DimMismatch);
        }

        // scores
        let mut scores: Vec<(usize, f32)> = (0..self.vecs.len())
            .map(|i| (i, cosine(&req.vec, &self.vecs[i]).unwrap()))
            .collect();

        scores.sort_by(|a, b| b.1.total_cmp(&a.1));
        let k = topk.min(scores.len());

        let mut results = Vec::with_capacity(k);
        for (idx, score) in scores.iter().take(k) {
            let id = &self.ids[*idx];
            let leaf = self.leaves[*idx];
            let path_raw: Vec<ProofStep> = proof_for(&self.leaves[..], *idx);
            let path: Vec<ProofStepSerde> = path_raw
                .iter()
                .map(|s| ProofStepSerde {
                    sibling_hex: hex::encode(s.sibling),
                    sibling_is_right: s.sibling_is_right,
                })
                .collect::<Vec<_>>();

            results.push(QueryResult {
                id: id.clone(),
                score: *score,
                leaf_hex: hex32(&leaf),
                path,
            });
        }

        Ok(QueryEvidence {
            index_pack_cid: self.index_pack_cid_hex(),
            results,
            dim: self.dim,
        })
    }

    pub fn verify(&self, ev: &QueryEvidence) -> Result<(), IndexError> {
        if ev.dim != self.dim {
            return Err(IndexError::DimMismatch);
        }
        if ev.index_pack_cid != self.index_pack_cid_hex() {
            return Err(IndexError::Merkle("pack cid mismatch".into()));
        }

        // checa cada prova
        for r in &ev.results {
            // leaf_hex -> bytes
            let leaf = hex_to_32(&r.leaf_hex)?;

            // reconstrói proof
            let path_raw: Vec<ProofStep> = r
                .path
                .iter()
                .map(|s| {
                    let sib = hex_to_32(&s.sibling_hex)?;
                    Ok(ProofStep {
                        sibling: sib,
                        sibling_is_right: s.sibling_is_right,
                    })
                })
                .collect::<Result<Vec<_>, IndexError>>()?;

            verify_path(leaf, &path_raw, self.root)
                .map_err(|e| IndexError::Merkle(format!("verify failed: {:?}", e)))?;
        }
        Ok(())
    }
}
