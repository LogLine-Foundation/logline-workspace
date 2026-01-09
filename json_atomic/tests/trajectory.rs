use json_atomic::trajectory_confidence;

#[test]
fn cosine_identity_and_orthogonal() {
    let a = [1.0, 0.0, 0.0];
    let b = [1.0, 0.0, 0.0];
    let c = [0.0, 1.0, 0.0];

    let id = trajectory_confidence(&a, &b);
    let ort = trajectory_confidence(&a, &c);

    assert!((id - 1.0).abs() < 1e-6);
    assert!((ort - 0.5).abs() < 1e-6);
}
