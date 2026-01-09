use ed25519_dalek::SigningKey;
use lllv_core::{Capsule, CapsuleFlags};
use lllv_index::{IndexPackBuilder, QueryRequest};

fn f32_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

#[test]
fn build_query_verify_ok() {
    let dim: u16 = 3;
    let sk = SigningKey::from_bytes(&[1u8; 32]);
    let v1 = vec![1.0, 0.0, 0.0]; // a
    let v2 = vec![0.0, 1.0, 0.0]; // b
    let v3 = vec![0.0, 0.0, 1.0]; // c

    let cap_a = Capsule::create(dim, &f32_to_bytes(&v1), CapsuleFlags::NONE, &sk).unwrap();
    let cap_b = Capsule::create(dim, &f32_to_bytes(&v2), CapsuleFlags::NONE, &sk).unwrap();
    let cap_c = Capsule::create(dim, &f32_to_bytes(&v3), CapsuleFlags::NONE, &sk).unwrap();

    let mut b = IndexPackBuilder::new(dim);
    b.add_capsule("a".into(), cap_a).unwrap();
    b.add_capsule("b".into(), cap_b).unwrap();
    b.add_capsule("c".into(), cap_c).unwrap();

    let pack = b.build(None).unwrap();
    let ev = pack.query(&QueryRequest::from_vec(&v1), 2).unwrap();
    pack.verify(&ev).unwrap();

    assert_eq!(ev.results[0].id, "a");
    assert!(ev.results[0].score > ev.results[1].score);
}
