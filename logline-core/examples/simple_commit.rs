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

fn main() {
    let signer = NoopSigner;
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
    let committed = pending.commit(&signer).unwrap();
    println!("status={}", committed.status.as_str());
}
