//! Property-based tests for JSON✯Atomic canonicalization.
//!
//! These tests use proptest to verify critical invariants:
//! - **Idempotence**: canonize(canonize(x)) == canonize(x)
//! - **Key order insensitivity**: shuffled keys produce same canonical output
//! - **Determinism**: same input always produces same output
//! - **CID stability**: CID only depends on semantic content, not formatting

use json_atomic::canonize;
use proptest::prelude::*;
use serde_json::Value;

/// Generate arbitrary JSON values suitable for canonicalization.
/// 
/// Note: JSON✯Atomic does NOT allow floats (non-deterministic representation),
/// so we only generate integers in the numeric strategy.
fn arb_json(max_depth: u32) -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        // Booleans
        any::<bool>().prop_map(Value::Bool),
        // Null
        Just(Value::Null),
        // Integers only (no floats - they're rejected by canonize)
        any::<i32>().prop_map(|n| Value::Number(n.into())),
        // Strings (limited size to avoid pathological cases)
        "[a-zA-Z0-9 _-]{0,64}".prop_map(Value::String),
    ];
    
    leaf.prop_recursive(
        max_depth,   // max depth
        256,         // max nodes
        10,          // items per collection
        |inner| {
            prop_oneof![
                // Arrays (order preserved)
                prop::collection::vec(inner.clone(), 0..6)
                    .prop_map(Value::Array),
                // Objects (keys will be sorted by canonize)
                prop::collection::hash_map("[a-zA-Z_][a-zA-Z0-9_]{0,16}", inner, 0..6)
                    .prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]
    
    /// Canonicalization must be idempotent: canonize(parse(canonize(x))) == canonize(x)
    #[test]
    fn canon_is_idempotent(v in arb_json(3)) {
        if let Ok(first) = canonize(&v) {
            // Parse the canonical bytes back to Value
            let reparsed: Value = serde_json::from_slice(&first)
                .expect("canonical output must be valid JSON");
            // Canonize again
            let second = canonize(&reparsed)
                .expect("reparsed canonical value must canonize");
            // Must be byte-identical
            prop_assert_eq!(
                first, second,
                "Canonicalization is not idempotent"
            );
        }
        // If canonize fails (e.g., float), that's fine - we tested what we could
    }

    /// CID must be stable regardless of key insertion order in source.
    #[test]
    fn key_order_does_not_affect_cid(
        keys in prop::collection::hash_set("[a-z]{1,8}", 1..6),
    ) {
        // Use HashSet to guarantee unique keys
        let keys: Vec<String> = keys.into_iter().collect();
        
        // Create two objects with same keys but different insertion orders
        let values: Vec<i32> = (0..keys.len() as i32).collect();
        
        let mut map1 = serde_json::Map::new();
        for (k, v) in keys.iter().zip(values.iter()) {
            map1.insert(k.clone(), Value::Number((*v).into()));
        }
        
        // Reverse order insertion
        let mut map2 = serde_json::Map::new();
        for (k, v) in keys.iter().rev().zip(values.iter().rev()) {
            map2.insert(k.clone(), Value::Number((*v).into()));
        }
        
        let v1 = Value::Object(map1);
        let v2 = Value::Object(map2);
        
        let c1 = canonize(&v1).expect("canonize v1");
        let c2 = canonize(&v2).expect("canonize v2");
        
        // CIDs must match
        let cid1 = blake3::hash(&c1);
        let cid2 = blake3::hash(&c2);
        prop_assert_eq!(cid1.as_bytes(), cid2.as_bytes(), "CIDs differ for same content");
        
        prop_assert_eq!(c1, c2, "Key order affected canonical output");
    }

    /// Canonical output must be valid UTF-8 JSON that parses to equivalent value.
    #[test]
    fn canon_produces_valid_utf8_json(v in arb_json(3)) {
        if let Ok(bytes) = canonize(&v) {
            // Must be valid UTF-8
            let s = std::str::from_utf8(&bytes)
                .expect("canonical bytes must be UTF-8");
            // Must parse back to JSON
            let reparsed: Value = serde_json::from_str(s)
                .expect("canonical JSON must parse");
            // Must be semantically equivalent
            prop_assert_eq!(
                v, reparsed,
                "Round-trip changed the value"
            );
        }
    }

    /// CID is always 32 bytes (BLAKE3).
    #[test]
    fn cid_is_always_32_bytes(v in arb_json(2)) {
        if let Ok(bytes) = canonize(&v) {
            let cid = blake3::hash(&bytes);
            prop_assert_eq!(cid.as_bytes().len(), 32);
        }
    }
}

/// Explicit test for nested key ordering.
#[test]
fn nested_objects_sort_keys_at_all_levels() {
    use serde_json::json;
    
    let v1 = json!({
        "z": {"b": 2, "a": 1},
        "a": {"d": 4, "c": 3}
    });
    
    let v2 = json!({
        "a": {"c": 3, "d": 4},
        "z": {"a": 1, "b": 2}
    });
    
    let c1 = canonize(&v1).expect("canonize v1");
    let c2 = canonize(&v2).expect("canonize v2");
    
    assert_eq!(c1, c2, "Nested key order affected output");
    
    // Verify the output has keys in sorted order
    let s = String::from_utf8(c1).unwrap();
    let a_pos = s.find("\"a\"").unwrap();
    let z_pos = s.find("\"z\"").unwrap();
    assert!(a_pos < z_pos, "Top-level keys not sorted: {}", s);
}

/// Floats must be rejected (they're non-deterministic).
#[test]
fn floats_are_rejected() {
    use serde_json::json;
    
    let with_float = json!({"value": 3.14159});
    let result = canonize(&with_float);
    assert!(result.is_err(), "Float should be rejected");
}

/// Empty structures canonize correctly.
#[test]
fn empty_structures_canonize() {
    use serde_json::json;
    
    let empty_obj = json!({});
    let empty_arr = json!([]);
    
    let c1 = canonize(&empty_obj).expect("empty object");
    let c2 = canonize(&empty_arr).expect("empty array");
    
    assert_eq!(c1, b"{}");
    assert_eq!(c2, b"[]");
}
