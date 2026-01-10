//! TDLN Policy Gate — preflight & decision, proof-carrying, deterministic.
//!
//! Event sequence (high-level): nl.utterance → plan.proposed → policy.preflight
//! → user.consent → policy.decision → effect.exec → state.update

#![forbid(unsafe_code)]

#[cfg(feature = "dv25")]
use logline_core as _;

use serde::{Deserialize, Serialize};
use tdln_compiler::CompiledIntent;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
    NeedsConsent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateOutput {
    pub decision: Decision,
    pub audit: serde_json::Value,
    pub proof_ref: [u8; 32],
    pub events: Vec<LogEvent>,
}

#[derive(Debug, Error)]
pub enum GateError {
    #[error("invalid input")]
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCtx {
    pub allow_freeform: bool,
}

/// Pré-validação determinística.
///
/// # Errors
///
/// - Retorna `GateError::Invalid` para entradas inválidas (reservado)
pub fn preflight(intent: &CompiledIntent, _ctx: &PolicyCtx) -> Result<GateOutput, GateError> {
    // Minimal deterministic audit
    let audit = serde_json::json!({
        "ast_cid": hex::encode(intent.proof.ast_cid),
        "canon_cid": hex::encode(intent.proof.canon_cid),
    });
    let events = vec![LogEvent {
        kind: "policy.preflight".into(),
        payload: audit.clone(),
    }];
    Ok(GateOutput {
        decision: Decision::NeedsConsent,
        audit,
        proof_ref: intent.cid,
        events,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consent {
    pub accepted: bool,
}

/// Decide a partir do consentimento e contexto.
///
/// # Errors
///
/// - Propaga `GateError::Invalid` para entradas inválidas (reservado)
pub fn decide(
    intent: &CompiledIntent,
    consent: &Consent,
    ctx: &PolicyCtx,
) -> Result<GateOutput, GateError> {
    // Deterministic decision with audit trail
    let mut out = preflight(intent, ctx)?;
    let decision = if consent.accepted && ctx.allow_freeform {
        Decision::Allow
    } else if !consent.accepted {
        Decision::NeedsConsent
    } else {
        Decision::Deny
    };
    out.events.push(LogEvent {
        kind: "policy.decision".into(),
        payload: serde_json::json!({ "decision": format!("{:?}", decision) }),
    });
    out.decision = decision;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tdln_compiler::{compile, CompileCtx};
    #[test]
    fn pipeline() {
        let ctx = CompileCtx {
            rule_set: "v1".into(),
        };
        let compiled = compile("hello world", &ctx).unwrap();
        let gctx = PolicyCtx {
            allow_freeform: true,
        };
        let pf = preflight(&compiled, &gctx).unwrap();
        assert_eq!(pf.decision, Decision::NeedsConsent);
        let dec = decide(&compiled, &Consent { accepted: true }, &gctx).unwrap();
        assert_eq!(dec.decision, Decision::Allow);
        assert!(dec.events.len() >= 2);
    }
}
