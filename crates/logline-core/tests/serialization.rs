#[cfg(feature = "serde")]
#[test]
fn serde_roundtrip() {
    use logline_core::*;
    let draft = LogLine::builder()
        .who("did:ubl:alice")
        .did(Verb::Transfer)
        .this(Payload::Text("100 USD".into()))
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "ok".into(),
            effects: vec!["receipt".into()],
        })
        .if_doubt(Escalation {
            label: "doubt".into(),
            route_to: "ops".into(),
        })
        .if_not(FailureHandling {
            label: "not".into(),
            action: "compensate".into(),
        })
        .build_draft()
        .unwrap();

    let s = serde_json::to_string(&draft).unwrap();
    let back: LogLine = serde_json::from_str(&s).unwrap();
    assert_eq!(back.who, "did:ubl:alice");
}
