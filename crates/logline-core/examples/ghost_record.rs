use logline_core::{Escalation, FailureHandling, LogLine, Outcome, Payload, Verb};

fn main() {
    let draft = LogLine::builder()
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
        .unwrap();

    let ghost = draft.abandon(Some("timeout".into())).unwrap();
    eprintln!("ghost reason={:?}", ghost.reason);
}
