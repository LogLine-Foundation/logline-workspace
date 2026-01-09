use logline_core::*;

struct SimpleRegistry;
impl VerbRegistry for SimpleRegistry {
    fn is_allowed(&self, verb: &Verb) -> bool {
        matches!(verb, Verb::Transfer | Verb::Deploy | Verb::Approve)
    }
}

#[test]
fn freeze_with_registry_allowed() {
    let registry = SimpleRegistry;

    let draft = LogLine::builder()
        .who("did:ubl:alice")
        .did(Verb::Transfer)
        .this(Payload::None)
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "ok".into(),
            effects: vec![],
        })
        .if_doubt(Escalation {
            label: "doubt".into(),
            route_to: "auditor".into(),
        })
        .if_not(FailureHandling {
            label: "not".into(),
            action: "compensate".into(),
        })
        .build_draft()
        .unwrap();

    let pending = draft.freeze_with_registry(&registry).unwrap();
    assert_eq!(pending.status, Status::Pending);
}

#[test]
fn freeze_with_registry_rejected() {
    let registry = SimpleRegistry;

    let draft = LogLine::builder()
        .who("did:ubl:bob")
        .did(Verb::Custom("malicious_action".into()))
        .this(Payload::None)
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "ok".into(),
            effects: vec![],
        })
        .if_doubt(Escalation {
            label: "doubt".into(),
            route_to: "auditor".into(),
        })
        .if_not(FailureHandling {
            label: "not".into(),
            action: "compensate".into(),
        })
        .build_draft()
        .unwrap();

    let err = draft.freeze_with_registry(&registry).unwrap_err();
    assert!(matches!(
        err,
        LogLineError::MissingField("did (unknown verb)")
    ));
}
