use json_atomic::canonize;
use serde_json::json;

#[test]
fn integers_with_leading_zeros_normalize() {
    // JSON padrão não permite 01, mas se vier via string -> ao reparse não vira número.
    // Aqui validamos que "001" (string) ≠ 1 (inteiro). O canônico é estável por tipo.
    let a = json!({"x": "001"});
    let b = json!({"x": 1});
    let ca = canonize(&a).unwrap();
    let cb = canonize(&b).unwrap();
    assert_ne!(ca, cb);
}

#[test]
fn key_order_with_nested_objects() {
    let a = json!({"o":{"b":1,"a":1},"z":0});
    let b = json!({"z":0,"o":{"a":1,"b":1}});
    let ca = canonize(&a).unwrap();
    let cb = canonize(&b).unwrap();
    assert_eq!(ca, cb);
}

#[test]
fn nfc_equivalence_accented() {
    let a = json!({"t":"Cafe\u{301}"}); // decomposed
    let b = json!({"t":"Café"}); // composed
    assert_eq!(canonize(&a).unwrap(), canonize(&b).unwrap());
}
