use criterion::{black_box, criterion_group, criterion_main, Criterion};
use logline_core::*;

fn bench_build_commit(c: &mut Criterion) {
    struct NoopSigner;
    impl Signer for NoopSigner {
        fn sign(&self, msg: &[u8]) -> Result<Signature, SignError> {
            Ok(Signature {
                alg: "none".into(),
                bytes: msg.to_vec(),
            })
        }
    }

    let signer = NoopSigner;

    c.bench_function("logline_build_freeze_commit", |b| {
        b.iter(|| {
            let draft = LogLine::builder()
                .who("did:ubl:bench")
                .did(Verb::Deploy)
                .this(Payload::Text("artifact:v1".into()))
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

            let pending = draft.freeze().unwrap();
            let committed = pending.commit(&signer).unwrap();

            black_box(committed);
        })
    });
}

criterion_group!(benches, bench_build_commit);
criterion_main!(benches);
