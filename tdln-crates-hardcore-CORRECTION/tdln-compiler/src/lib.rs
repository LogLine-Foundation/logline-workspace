//! TDLN Compiler — NL/DSL → AST + Canonical JSON + ProofBundle (deterministic).
//!
//! Determinism: same input + same rule set → same outputs (AST, canon, CID).

#![forbid(unsafe_code)]

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tdln_ast::SemanticUnit;
use tdln_proof::{build_proof, ProofBundle};

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("empty input")]
    Empty,
    #[error("internal")]
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileCtx {
    /// Versioned rule set id; choose different ids to represent tie-breaking evolution.
    pub rule_set: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledIntent {
    pub ast: SemanticUnit,
    pub canon_json: Vec<u8>,
    pub cid: [u8; 32],
    pub proof: ProofBundle,
}

pub fn compile(input: &str, ctx: &CompileCtx) -> Result<CompiledIntent, CompileError> {
    if input.trim().is_empty() {
        return Err(CompileError::Empty);
    }
    let ast = SemanticUnit::from_intent(input);
    let canon = ast.canonical_bytes();
    let mut h = Hasher::new();
    h.update(&canon);
    let cid = h.finalize().into();
    let proof = build_proof(&ast, &canon, &[ctx.rule_set.as_str()]);
    Ok(CompiledIntent { ast, canon_json: canon, cid, proof })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn determinism_whitespace() {
        let ctx = CompileCtx { rule_set: "v1".into() };
        let a = compile("  HELLO   world", &ctx).unwrap();
        let b = compile("hello world", &ctx).unwrap();
        assert_eq!(a.cid, b.cid);
        assert_eq!(a.proof.ast_cid, b.proof.ast_cid);
        assert_eq!(a.proof.canon_cid, b.proof.canon_cid);
    }
}
