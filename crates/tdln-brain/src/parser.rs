//! Parser module for extracting TDLN intents from LLM output.
//!
//! Handles JSON extraction from raw model responses, including:
//! - Clean JSON objects
//! - Markdown-fenced JSON blocks
//! - Reasoning + JSON combinations

use crate::{BrainError, Decision, UsageMeta};
use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;
use tdln_ast::SemanticUnit;

/// Extract a JSON block from raw text.
///
/// Supports:
/// - Direct JSON objects: `{"kind": ...}`
/// - Fenced blocks: ` ```json ... ``` `
/// - Mixed text with embedded JSON
fn extract_json_block(raw: &str) -> Option<(Option<String>, String)> {
    let trimmed = raw.trim();

    // Try: direct JSON object
    if trimmed.starts_with('{') {
        if let Some(end) = find_matching_brace(trimmed) {
            let json_str = &trimmed[..=end];
            return Some((None, json_str.to_string()));
        }
    }

    // Try: markdown fenced block
    let fence_re = Regex::new(r"```(?:json)?\s*\n?([\s\S]*?)```").ok()?;
    if let Some(caps) = fence_re.captures(raw) {
        let json_str = caps.get(1)?.as_str().trim();
        // Extract reasoning before the fence
        let fence_start = caps.get(0)?.start();
        let reasoning = if fence_start > 0 {
            let before = raw[..fence_start].trim();
            if !before.is_empty() {
                Some(before.to_string())
            } else {
                None
            }
        } else {
            None
        };
        return Some((reasoning, json_str.to_string()));
    }

    // Try: find JSON object anywhere
    if let Some(start) = raw.find('{') {
        if let Some(end) = find_matching_brace(&raw[start..]) {
            let json_str = &raw[start..=start + end];
            let reasoning = if start > 0 {
                let before = raw[..start].trim();
                if !before.is_empty() {
                    Some(before.to_string())
                } else {
                    None
                }
            } else {
                None
            };
            return Some((reasoning, json_str.to_string()));
        }
    }

    None
}

/// Find the index of the matching closing brace.
fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse a raw LLM response into a strict Decision.
///
/// # Errors
///
/// Returns `BrainError::Hallucination` if the output cannot be parsed
/// into a valid `SemanticUnit`.
pub fn parse_decision(raw: &str, meta: UsageMeta) -> Result<Decision, BrainError> {
    let (reasoning, json_str) = extract_json_block(raw)
        .ok_or_else(|| BrainError::Hallucination("no JSON block found".into()))?;

    let value: Value = serde_json::from_str(&json_str)
        .map_err(|e| BrainError::Parsing(format!("invalid JSON: {e}")))?;

    // Extract kind
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| BrainError::Hallucination("missing 'kind' field".into()))?
        .to_string();

    // Extract slots (default to empty)
    let slots: BTreeMap<String, Value> = value
        .get("slots")
        .cloned()
        .map(|v| {
            serde_json::from_value(v)
                .map_err(|e| BrainError::Hallucination(format!("invalid slots: {e}")))
        })
        .transpose()?
        .unwrap_or_default();

    // Build SemanticUnit with proper source_hash
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_json() {
        let raw = r#"{"kind":"grant","slots":{"to":"alice","amount":100}}"#;
        let dec = parse_decision(raw, UsageMeta::default()).unwrap();
        assert_eq!(dec.intent.kind, "grant");
        assert!(dec.reasoning.is_none());
    }

    #[test]
    fn parses_fenced_json() {
        let raw = r#"Let me think about this...

```json
{"kind":"transfer","slots":{"from":"bob","to":"alice"}}
```
"#;
        let dec = parse_decision(raw, UsageMeta::default()).unwrap();
        assert_eq!(dec.intent.kind, "transfer");
        assert!(dec.reasoning.is_some());
        assert!(dec.reasoning.unwrap().contains("think about"));
    }

    #[test]
    fn parses_json_with_prose() {
        let raw = r#"I'll create a grant action. {"kind":"grant","slots":{"to":"bob"}}"#;
        let dec = parse_decision(raw, UsageMeta::default()).unwrap();
        assert_eq!(dec.intent.kind, "grant");
    }

    #[test]
    fn rejects_invalid_shape() {
        let raw = r#"{"action":"grant"}"#; // missing "kind"
        let result = parse_decision(raw, UsageMeta::default());
        assert!(matches!(result, Err(BrainError::Hallucination(_))));
    }

    #[test]
    fn rejects_no_json() {
        let raw = "I don't know what to do.";
        let result = parse_decision(raw, UsageMeta::default());
        assert!(matches!(result, Err(BrainError::Hallucination(_))));
    }

    #[test]
    fn handles_nested_json() {
        let raw = r#"{"kind":"policy","slots":{"rules":{"max":500,"min":10}}}"#;
        let dec = parse_decision(raw, UsageMeta::default()).unwrap();
        assert_eq!(dec.intent.kind, "policy");
        assert!(dec.intent.slots.contains_key("rules"));
    }

    #[test]
    fn handles_empty_slots() {
        let raw = r#"{"kind":"noop"}"#;
        let dec = parse_decision(raw, UsageMeta::default()).unwrap();
        assert_eq!(dec.intent.kind, "noop");
        assert!(dec.intent.slots.is_empty());
    }
}
