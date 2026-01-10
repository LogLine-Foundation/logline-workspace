use criterion::{criterion_group, criterion_main, Criterion};
use json_atomic::canonize;
use serde_json::json;

fn bench_canonize(c: &mut Criterion) {
    c.bench_function("canonize_small_object", |b| {
        b.iter(|| {
            let v = json!({"b":1,"a":"Café","arr":[1,2,3], "o":{"y":2,"x":1}});
            let _ = canonize(&v).unwrap();
        });
    });
}

criterion_group!(benches, bench_canonize);
criterion_main!(benches);
