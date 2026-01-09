use logline_core::*;

#[test]
fn invariants_missing_fail() {
    let b = LogLine::builder()
        .who("did:ubl:alice")
        .did(Verb::Approve)
        .this(Payload::None)
        .when(1_735_671_234)
        // falta if_ok / if_doubt / if_not
        ;

    let err = b.build_draft().unwrap_err();
    assert!(matches!(err, LogLineError::MissingInvariant(_)));
}

#[test]
fn invariants_ok() {
    let line = LogLine::builder()
        .who("did:ubl:alice")
        .did(Verb::Transfer)
        .this(Payload::Text("x".into()))
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "done".into(),
            effects: vec!["emit_receipt".into()],
        })
        .if_doubt(Escalation {
            label: "escalate".into(),
            route_to: "auditor".into(),
        })
        .if_not(FailureHandling {
            label: "fail".into(),
            action: "compensate".into(),
        })
        .build_draft()
        .unwrap();
    assert_eq!(line.status, Status::Draft);
    assert!(line.verify_invariants().is_ok());
}
