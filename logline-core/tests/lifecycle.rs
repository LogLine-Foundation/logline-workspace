use logline_core::*;

struct NoopSigner;
impl Signer for NoopSigner {
    fn sign(&self, msg: &[u8]) -> Result<Signature, SignError> {
        Ok(Signature {
            alg: "none".into(),
            bytes: msg.to_vec(),
        })
    }
}

#[test]
fn draft_pending_committed() {
    let signer = NoopSigner;

    let draft = LogLine::builder()
        .who("did:ubl:alice")
        .did(Verb::Deploy)
        .this(Payload::Text("artifact:v1".into()))
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "ok".into(),
            effects: vec![],
        })
        .if_doubt(Escalation {
            label: "doubt".into(),
            route_to: "qa".into(),
        })
        .if_not(FailureHandling {
            label: "not".into(),
            action: "rollback".into(),
        })
        .build_draft()
        .unwrap();

    let pending = draft.freeze().unwrap();
    assert_eq!(pending.status, Status::Pending);

    let committed = pending.commit(&signer).unwrap();
    assert_eq!(committed.status, Status::Committed);
}

#[test]
fn draft_to_ghost() {
    let draft = LogLine::builder()
        .who("did:ubl:bob")
        .did(Verb::Custom("approve_budget".into()))
        .this(Payload::None)
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "ok".into(),
            effects: vec![],
        })
        .if_doubt(Escalation {
            label: "doubt".into(),
            route_to: "finance".into(),
        })
        .if_not(FailureHandling {
            label: "not".into(),
            action: "notify".into(),
        })
        .build_draft()
        .unwrap();

    let ghost = draft.abandon(Some("user_cancelled".into())).unwrap();
    assert_eq!(ghost.status, Status::Ghost);
}

#[test]
fn draft_sign_commit() {
    let signer = NoopSigner;

    let draft = LogLine::builder()
        .who("did:ubl:alice")
        .did(Verb::Transfer)
        .this(Payload::Text("100 USD".into()))
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "ok".into(),
            effects: vec!["emit_receipt".into()],
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

    // Paper I: attempt já nasce assinado
    let signed = draft.sign(&signer).unwrap();
    let pending = signed.freeze().unwrap();
    let committed = pending.commit(&signer).unwrap();
    assert_eq!(committed.status, Status::Committed);
}

#[test]
fn abandon_signed() {
    let signer = NoopSigner;

    let draft = LogLine::builder()
        .who("did:ubl:bob")
        .did(Verb::Deploy)
        .this(Payload::Text("service:v2".into()))
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "ok".into(),
            effects: vec![],
        })
        .if_doubt(Escalation {
            label: "doubt".into(),
            route_to: "qa".into(),
        })
        .if_not(FailureHandling {
            label: "not".into(),
            action: "rollback".into(),
        })
        .build_draft()
        .unwrap();

    // Paper I: abandon também deve ser assinado (attempt já nasce assinado)
    let ghost = draft
        .abandon_signed(&signer, Some("timeout".into()))
        .unwrap();
    assert_eq!(ghost.status, Status::Ghost);
}
