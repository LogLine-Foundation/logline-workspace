use ed25519_dalek::SigningKey;
use json_atomic::seal_logline;
use logline_core::*;

fn main() {
    let sk = SigningKey::from_bytes(&[9u8; 32]);
    let line = LogLine::builder()
        .who("did:ubl:bob")
        .did(Verb::Deploy)
        .this(Payload::Text("service:v2".into()))
        .when(1_735_671_234)
        .if_ok(Outcome {
            label: "ok".into(),
            effects: vec!["emit_receipt".into()],
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
        .unwrap()
        .freeze()
        .unwrap();

    let fact = seal_logline(&line, &sk).unwrap();
    println!("SignedFact CID(hex)={}", fact.cid_hex());
}
