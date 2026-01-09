use json_atomic::canonize;
use serde_json::json;

#[test]
fn objects_with_different_key_order_are_equal() {
    let a = json!({"a":1,"b":1});
    let b = json!({"b":1,"a":1});
    let ca = canonize(&a).unwrap();
    let cb = canonize(&b).unwrap();
    assert_eq!(ca, cb);
}

#[test]
fn nfc_normalization() {
    // "Café": forma decomposta vs composta
    let a = json!({"t":"Cafe\u{301}"});
    let b = json!({"t":"Café"});
    let ca = canonize(&a).unwrap();
    let cb = canonize(&b).unwrap();
    assert_eq!(ca, cb);
}

#[test]
fn floats_are_rejected() {
    let v = json!({"x": 1.23});
    let err = canonize(&v).unwrap_err().to_string();
    assert!(err.contains("floating numbers"));
}
