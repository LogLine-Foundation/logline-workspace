//! Strict JSON extraction and validation.
//!
//! Extracts a single JSON object from LLM output (supporting markdown blocks
//! and inline JSON), then validates it into a [`SemanticUnit`].

use crate::{BrainError, Decision, UsageMeta};
use serde_json::Value;
use std::collections::BTreeMap;
use tdln_ast::SemanticUnit;

/// Parse a raw LLM response into a strict [`Decision`].
///
/// # Errors
///
/// Returns `BrainError::Hallucination` if the output cannot be parsed
/// into a valid `SemanticUnit`.
pub fn parse_decision(raw: &str, meta: UsageMeta) -> Result<Decision, BrainError> {
    let (json_str, reasoning) = extract_json_block(raw);

    // Parse as generic JSON first
    let value: Value = serde_json::from_str(&json_str).map_err(|e| {
        BrainError::Hallucination(format!("Invalid JSON: {e}; input: {json_str}"))
    })?;

    // Extract kind (required)
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| BrainError::Hallucination("missing 'kind' field".into()))?
        .to_string();

    // Extract slots (optional, default to empty)
    let slots: BTreeMap<String, Value> = value
        .get("slots")
        .cloned()
        .map(|v| {
            serde_json::from_value(v)
                .map_err(|e| BrainError::Hallucination(format!("invalid slots: {e}")))
        })
        .transpose()?
        .unwrap_or_default();

    // Compute source_hash from the JSON string
    let source_hash: [u8; 32] = blake3::hash(json_str.as_bytes()).into();

    let intent = SemanticUnit {
        kind,
        slots,
        source_hash,
    };

    Ok(Decision {
        reasoning,
        intent,
        meta,
    })
}

/// Extract a JSON block from raw text.
///
/// Supports:
/// - Fenced blocks: ` ```json ... ``` `
/// - Direct JSON objects: `{ ... }`
/// - JSON embedded in prose
///
/// Returns `(json_string, optional_reasoning)`.
fn extract_json_block(text: &str) -> (String, Option<String>) {
    // Try: markdown fenced block
    if let Some(s) = text.find("```json") {
        if let Some(end_rel) = text[s + 7..].find("```") {
            let json = text[s + 7..s + 7 + end_rel].trim().to_string();
            let reasoning = if s > 0 {
                let before = text[..s].trim();
                if before.is_empty() {
                    None
                } else {
                    Some(before.to_string())
                }
            } else {
                None
            };
            return (json, reasoning);
        }
    }

    // Try: find JSON object span
    if let (Some(a), Some(b)) = (text.find('{'), text.rfind('}')) {
        if b > a {
            let json = text[a..=b].to_string();
            let reasoning = if a > 0 {
                let before = text[..a].trim();
                if before.is_empty() {
                    None
                } else {
                    Some(before.to_string())
                }
            } else {
                None
            };
            return (json, reasoning);
        }
    }

    // Fallback: entire text
    (text.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_clean_json() {
        let (j, r) = extract_json_block(r#"{"kind":"test","slots":{}}"#);
        assert_eq!(j, r#"{"kind":"test","slots":{}}"#);
        assert!(r.is_none());
    }

    #[test]
    fn extract_markdown_block() {
        let inp = "thinking...\n```json\n{\"kind\":\"x\",\"slots\":{}}\n```\n";
        let (j, r) = extract_json_block(inp);
        assert!(j.contains(r#""kind":"x""#));
        assert_eq!(r.unwrap(), "thinking...");
    }

    #[test]
    fn extract_json_with_prose() {
        let inp = r#"I'll create a grant. {"kind":"grant","slots":{"to":"bob"}}"#;
        let (j, r) = extract_json_block(inp);
        assert!(j.contains("grant"));
        assert!(r.is_some());
    }

    #[test]
    fn parses_valid_semantic_unit() {
        let raw = r#"{"kind":"noop","slots":{}}"#;
        let dec = parse_decision(raw, UsageMeta::default()).unwrap();
        assert_eq!(dec.intent.kind, "noop");
        assert!(dec.reasoning.is_none());
    }

    #[test]
    fn parses_markdown_wrapped() {
        let raw = "Let me think...\n```json\n{\"kind\":\"grant\",\"slots\":{\"to\":\"alice\"}}\n```\n";
        let dec = parse_decision(raw, UsageMeta::default()).unwrap();
        assert_eq!(dec.intent.kind, "grant");
        assert!(dec.reasoning.is_some());
        assert!(dec.reasoning.unwrap().contains("think"));
    }

    #[test]
    fn rejects_invalid_json() {
        let raw = r#"{"kind":"#;
        let result = parse_decision(raw, UsageMeta::default());
        assert!(matches!(result, Err(BrainError::Hallucination(_))));
    }

    #[test]
    fn rejects_missing_kind() {
        let raw = r#"{"action":"test"}"#;
        let result = parse_decision(raw, UsageMeta::default());
        assert!(matches!(result, Err(BrainError::Hallucination(_))));
    }

    #[test]
    fn handles_empty_slots() {
        let raw = r#"{"kind":"noop"}"#;
        let dec = parse_decision(raw, UsageMeta::default()).unwrap();
        assert_eq!(dec.intent.kind, "noop");
    }

    #[test]
    fn handles_nested_slots() {
        let raw = r#"{"kind":"policy","slots":{"rules":{"max":500}}}"#;
        let dec = parse_decision(raw, UsageMeta::default()).unwrap();
        assert_eq!(dec.intent.kind, "policy");
        assert!(dec.intent.slots.contains_key("rules"));
    }
}
