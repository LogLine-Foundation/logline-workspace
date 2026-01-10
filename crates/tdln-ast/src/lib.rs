//! TDLN — Canonical AST for deterministic translation of NL/DSL into Logical Atoms.
//!
//! Invariants:
//! - Canonical bytes are deterministic for the same semantic content
//! - CID = `BLAKE3(canonical_bytes)`
//!
//! Canonicalization delegates to `json_atomic` as the single source of truth.

#![forbid(unsafe_code)]

mod canon;

use blake3::Hasher;
use canon::to_canon_vec;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Minimal AST node representing a canonical semantic intent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticUnit {
    /// Kind of intent (e.g., "policy.allow", "freeform.intent").
    pub kind: String,
    /// Named slots after resolution (normalized, deterministic order in canon).
    pub slots: BTreeMap<String, Value>,
    /// Hash of the *source* text (not the canonical form). BLAKE3-32.
    pub source_hash: [u8; 32],
}

impl SemanticUnit {
    /// Naive builder from raw input (normalize whitespace to single spaces).
    #[must_use]
    pub fn from_intent(text: &str) -> Self {
        let norm = normalize(text);
        let mut slots = BTreeMap::new();
        slots.insert("utterance".to_string(), Value::String(norm.clone()));
        let source_hash = blake3::hash(norm.as_bytes()).into();
        Self {
            kind: "freeform.intent".to_string(),
            slots,
            source_hash,
        }
    }

    /// Canonical JSON bytes via `json_atomic` — single source of truth.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        to_canon_vec(self)
    }

    /// CID of canonical bytes = BLAKE3-32.
    #[must_use]
    pub fn cid_blake3(&self) -> [u8; 32] {
        let mut h = Hasher::new();
        h.update(&self.canonical_bytes());
        h.finalize().into()
    }
}

fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.trim().chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch.to_ascii_lowercase());
            prev_space = false;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determinism_basic() {
        let a = SemanticUnit::from_intent("  Hello   WORLD ");
        let b = SemanticUnit::from_intent("hello world");
        assert_eq!(a.canonical_bytes(), b.canonical_bytes());
        assert_eq!(a.cid_blake3(), b.cid_blake3());
    }

    #[test]
    fn determinism_whitespace_insensitive() {
        // Same semantic content, different whitespace → identical canonical bytes
        let a = SemanticUnit::from_intent("grant access to alice");
        let b = SemanticUnit::from_intent("  grant   access   to   alice  ");
        assert_eq!(a.canonical_bytes(), b.canonical_bytes());
    }

    #[test]
    fn cid_is_blake3_of_canonical() {
        let unit = SemanticUnit::from_intent("test intent");
        let expected_cid: [u8; 32] = blake3::hash(&unit.canonical_bytes()).into();
        assert_eq!(unit.cid_blake3(), expected_cid);
    }
}
