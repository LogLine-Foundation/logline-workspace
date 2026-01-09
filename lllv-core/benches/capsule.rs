use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ed25519_dalek::SigningKey;
use lllv_core::*;

fn bench_capsule_create(c: &mut Criterion) {
    let sk = SigningKey::from_bytes(&[1u8; 32]);
    let payload = vec![1u8; 1024];

    c.bench_function("capsule_create", |b| {
        b.iter(|| {
            let cap = Capsule::create(128, &payload, CapsuleFlags::NONE, &sk).unwrap();
            black_box(cap);
        })
    });
}

criterion_group!(benches, bench_capsule_create);
criterion_main!(benches);
