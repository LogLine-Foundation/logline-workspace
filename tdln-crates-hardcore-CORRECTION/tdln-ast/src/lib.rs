//! TDLN — Canonical AST for deterministic translation of NL/DSL into Logical Atoms.
//!
//! Invariants:
//! - Canonical bytes are deterministic for the same semantic content
//! - CID = BLAKE3(canonical_bytes)
//!
//! The canonicalization uses a stable ordering of object keys. If the `json-atomic`
//! feature is enabled, the crate is linked (no API commitment needed here yet).

#![forbid(unsafe_code)]

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
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
    pub fn from_intent(text: &str) -> Self {
        let norm = normalize(text);
        let mut slots = BTreeMap::new();
        slots.insert("utterance".to_string(), Value::String(norm.clone()));
        let source_hash = blake3::hash(text.as_bytes()).into();
        Self {
            kind: "freeform.intent".to_string(),
            slots,
            source_hash,
        }
    }

    /// Canonical JSON bytes with deterministic key ordering.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut root = BTreeMap::new();
        root.insert("kind".to_string(), Value::String(self.kind.clone()));
        // ensure slots sorted deterministically
        let mut slots_sorted = BTreeMap::new();
        for (k, v) in &self.slots {
            slots_sorted.insert(k.clone(), canonicalize_value(v));
        }
        root.insert(
            "slots".to_string(),
            Value::Object(Map::from_iter(slots_sorted.into_iter().map(|(k, v)| (k, v)))),
        );
        root.insert(
            "source_hash".to_string(),
            Value::String(hex::encode(self.source_hash)),
        );
        let stable = Value::Object(Map::from_iter(
            root.into_iter().map(|(k, v)| (k, v)),
        ));
        serde_json::to_vec(&stable).expect("serialize canonical bytes")
    }

    /// CID of canonical bytes = BLAKE3-32.
    pub fn cid_blake3(&self) -> [u8; 32] {
        let mut h = Hasher::new();
        h.update(&self.canonical_bytes());
        h.finalize().into()
    }
}

fn canonicalize_value(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut sorted = BTreeMap::new();
            for (k, v) in m {
                sorted.insert(k.clone(), canonicalize_value(v));
            }
            Value::Object(Map::from_iter(sorted.into_iter().map(|(k, v)| (k, v))))
        }
        Value::Array(arr) => Value::Array(arr.iter().map(canonicalize_value).collect()),
        _ => v.clone(),
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
}
