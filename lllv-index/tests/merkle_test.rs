use lllv_index::merkle::{leaf_hash, merkle_root, proof_for, verify_path};

#[test]
fn merkle_simple() {
    let h1 = leaf_hash(b"A");
    let h2 = leaf_hash(b"B");
    let h3 = leaf_hash(b"C");
    let leaves = vec![h1, h2, h3];

    let root = merkle_root(&leaves);

    let p0 = proof_for(&leaves, 0);
    let p1 = proof_for(&leaves, 1);
    let p2 = proof_for(&leaves, 2);

    assert!(verify_path(h1, &p0, root).is_ok(), "proof0 failed");
    assert!(verify_path(h2, &p1, root).is_ok(), "proof1 failed");
    assert!(verify_path(h3, &p2, root).is_ok(), "proof2 failed");
}

#[test]
fn merkle_tamper_fails() {
    let h1 = leaf_hash(b"A");
    let h2 = leaf_hash(b"B");
    let leaves = vec![h1, h2];

    let root = merkle_root(&leaves);
    let mut p0 = proof_for(&leaves, 0);

    // corrompe o primeiro byte do primeiro step
    p0[0].sibling[0] ^= 0xAA;

    assert!(verify_path(h1, &p0, root).is_err(), "tamper should fail");
}
