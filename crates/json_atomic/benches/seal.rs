use criterion::{criterion_group, criterion_main, Criterion};
use ed25519_dalek::SigningKey;
use json_atomic::seal_value;
use serde_json::json;

fn bench_seal(c: &mut Criterion) {
    let sk = SigningKey::from_bytes(&[5u8; 32]);
    c.bench_function("seal_small_object", |b| {
        b.iter(|| {
            let v = json!({"a":1,"b":"Café"});
            let _ = seal_value(&v, &sk).unwrap();
        });
    });
}

criterion_group!(benches, bench_seal);
criterion_main!(benches);
