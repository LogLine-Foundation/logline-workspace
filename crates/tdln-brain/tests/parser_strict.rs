//! Strict parser tests.

use tdln_brain::{parser, UsageMeta};

#[test]
fn parses_markdown_block() {
    let raw = "think\n```json\n{\"kind\":\"noop\",\"slots\":{}}\n```\n";
    let d = parser::parse_decision(raw, UsageMeta::default()).unwrap();
    assert_eq!(d.intent.kind, "noop");
}

#[test]
fn rejects_bad_json() {
    let raw = "```json\n{\"kind\":\n```\n";
    let err = parser::parse_decision(raw, UsageMeta::default())
        .unwrap_err()
        .to_string();
    assert!(err.contains("Invalid structure") || err.contains("hallucination"));
}

#[test]
fn parses_inline_json() {
    let raw = r#"Here's the plan: {"kind":"grant","slots":{"to":"bob","amount":50}}"#;
    let d = parser::parse_decision(raw, UsageMeta::default()).unwrap();
    assert_eq!(d.intent.kind, "grant");
    assert!(d.reasoning.is_some());
}

#[test]
fn parses_clean_json() {
    let raw = r#"{"kind":"transfer","slots":{"from":"alice","to":"bob"}}"#;
    let d = parser::parse_decision(raw, UsageMeta::default()).unwrap();
    assert_eq!(d.intent.kind, "transfer");
    assert!(d.reasoning.is_none());
}

#[test]
fn rejects_missing_kind() {
    let raw = r#"{"slots":{"x":1}}"#;
    let result = parser::parse_decision(raw, UsageMeta::default());
    assert!(result.is_err());
}

#[test]
fn handles_complex_slots() {
    let raw = r#"{"kind":"policy","slots":{"rules":{"max":500,"min":10},"enabled":true}}"#;
    let d = parser::parse_decision(raw, UsageMeta::default()).unwrap();
    assert_eq!(d.intent.kind, "policy");
    assert!(d.intent.slots.contains_key("rules"));
    assert!(d.intent.slots.contains_key("enabled"));
}
