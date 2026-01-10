use ed25519_dalek::SigningKey;
use json_atomic::seal_logline;
use logline_core::*;

#[test]
fn seal_logline_fact() {
    let sk = SigningKey::from_bytes(&[3u8; 32]);

    let draft = LogLine::builder()
        .who("did:ubl:alice")
        .did(Verb::Approve)
        .this(Payload::Text("purchase:123".into()))
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "approved".into(),
            effects: vec!["emit_receipt".into()],
        })
        .if_doubt(Escalation {
            label: "manual_review".into(),
            route_to: "auditor".into(),
        })
        .if_not(FailureHandling {
            label: "rejected".into(),
            action: "notify".into(),
        })
        .build_draft()
        .unwrap();

    let pending = draft.freeze().unwrap();
    // Nota: a assinatura do LogLine em si é tratada pelo logline-core; aqui selamos o fato canônico
    let fact = seal_logline(&pending, &sk).unwrap();
    assert_eq!(fact.hash_alg, "blake3");
    assert_eq!(fact.sig_alg, "ed25519");
}
