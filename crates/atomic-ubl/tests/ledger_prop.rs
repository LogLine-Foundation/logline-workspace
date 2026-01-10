//! Property-based and adversarial tests for UBL ledger.
//!
//! These tests ensure the ledger handles:
//! - Arbitrary valid intents roundtrip correctly
//! - CID verification catches any corruption
//! - Signature verification (when enabled)
//! - Multiple concurrent readers
//! - Large batches of entries

use atomic_ubl::{LedgerEntry, LedgerError, SimpleLedgerReader, SimpleLedgerWriter};
use proptest::prelude::*;
use serde_json::{json, Value};
use std::io::{BufReader, Cursor};
use tempfile::NamedTempFile;

// ══════════════════════════════════════════════════════════════════════════════
// Strategy: Generate valid JSON intents
// ══════════════════════════════════════════════════════════════════════════════

fn arb_intent_value() -> impl Strategy<Value = Value> {
    // Simple intent-like JSON objects (no floats!)
    (
        "[a-zA-Z]{3,10}",                          // intent type
        "[a-zA-Z0-9]{1,20}",                       // actor
        proptest::option::of("[a-zA-Z0-9]{1,30}"), // target
        any::<i32>(),                              // amount (as integer)
    )
        .prop_map(|(intent_type, actor, target, amount)| {
            let mut obj = serde_json::Map::new();
            obj.insert("intent".to_string(), Value::String(intent_type));
            obj.insert("actor".to_string(), Value::String(actor));
            if let Some(t) = target {
                obj.insert("target".to_string(), Value::String(t));
            }
            obj.insert("amount".to_string(), Value::Number(amount.into()));
            Value::Object(obj)
        })
}

fn arb_actor() -> impl Strategy<Value = Option<String>> {
    proptest::option::of("[a-zA-Z0-9_]{1,32}")
}

fn arb_extra() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), 0..64)
}

// ══════════════════════════════════════════════════════════════════════════════
// Property tests
// ══════════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Any valid entry can be created and verified.
    #[test]
    fn unsigned_entry_verifies(
        intent in arb_intent_value(),
        actor in arb_actor(),
        extra in arb_extra(),
    ) {
        let e = LedgerEntry::unsigned(&intent, actor, &extra).unwrap();
        prop_assert!(e.verify().is_ok());
        prop_assert_eq!(e.pubkey, None);
        prop_assert_eq!(e.signature, None);
    }

    /// CID is deterministic: same intent produces same CID.
    #[test]
    fn cid_is_deterministic(intent in arb_intent_value()) {
        let e1 = LedgerEntry::unsigned(&intent, None, b"").unwrap();
        let e2 = LedgerEntry::unsigned(&intent, None, b"").unwrap();
        prop_assert_eq!(e1.cid, e2.cid);
        prop_assert_eq!(e1.intent, e2.intent);
    }

    /// Corrupting CID causes verification to fail.
    #[test]
    fn corrupted_cid_fails_verify(
        intent in arb_intent_value(),
        byte_idx in 0usize..32,
        xor in 1u8..=255,
    ) {
        let mut e = LedgerEntry::unsigned(&intent, None, b"").unwrap();
        e.cid.0[byte_idx] ^= xor;
        prop_assert!(matches!(e.verify(), Err(LedgerError::CidMismatch)));
    }

    /// Corrupting intent bytes causes verification to fail.
    #[test]
    fn corrupted_intent_fails_verify(
        intent in arb_intent_value(),
        xor in 1u8..=255,
    ) {
        let mut e = LedgerEntry::unsigned(&intent, None, b"").unwrap();
        if !e.intent.is_empty() {
            let idx = e.intent.len() / 2;
            e.intent[idx] ^= xor;
            prop_assert!(matches!(e.verify(), Err(LedgerError::CidMismatch)));
        }
    }

    /// Write and read back multiple entries.
    #[test]
    fn roundtrip_multiple_entries(
        intents in proptest::collection::vec(arb_intent_value(), 1..10),
    ) {
        let tmp = NamedTempFile::new().unwrap();
        
        // Write
        {
            let mut w = SimpleLedgerWriter::open_append(tmp.path()).unwrap();
            for intent in &intents {
                let e = LedgerEntry::unsigned(intent, None, b"").unwrap();
                w.append(&e).unwrap();
            }
            w.sync().unwrap();
        }
        
        // Read back
        let entries: Vec<_> = SimpleLedgerReader::from_path(tmp.path())
            .unwrap()
            .iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        
        prop_assert_eq!(entries.len(), intents.len());
        
        // Verify each entry's CID matches what we'd compute fresh
        for (entry, intent) in entries.iter().zip(intents.iter()) {
            let fresh = LedgerEntry::unsigned(intent, None, b"").unwrap();
            prop_assert_eq!(entry.cid, fresh.cid);
        }
    }

    /// Same intent with different key order produces same CID.
    #[test]
    fn key_order_does_not_affect_cid(
        keys in prop::collection::hash_set("[a-z]{1,8}", 2..5),
    ) {
        let keys: Vec<String> = keys.into_iter().collect();
        let values: Vec<i32> = (0..keys.len() as i32).collect();
        
        // Build two objects with different insertion order
        let mut map1 = serde_json::Map::new();
        for (k, v) in keys.iter().zip(values.iter()) {
            map1.insert(k.clone(), Value::Number((*v).into()));
        }
        
        let mut map2 = serde_json::Map::new();
        for (k, v) in keys.iter().rev().zip(values.iter().rev()) {
            map2.insert(k.clone(), Value::Number((*v).into()));
        }
        
        let v1 = Value::Object(map1);
        let v2 = Value::Object(map2);
        
        let e1 = LedgerEntry::unsigned(&v1, None, b"").unwrap();
        let e2 = LedgerEntry::unsigned(&v2, None, b"").unwrap();
        
        prop_assert_eq!(e1.cid, e2.cid);
        prop_assert_eq!(e1.intent, e2.intent);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Signature tests (when signing feature enabled)
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "signing")]
mod signing_tests {
    use super::*;
    use atomic_crypto::Keypair;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Signed entries verify correctly.
        #[test]
        fn signed_entry_verifies(intent in arb_intent_value()) {
            let kp = Keypair::generate();
            let e = LedgerEntry::unsigned(&intent, None, b"")
                .unwrap()
                .sign(&kp.sk);
            
            prop_assert!(e.verify().is_ok());
            prop_assert!(e.pubkey.is_some());
            prop_assert!(e.signature.is_some());
        }

        /// Corrupting signature fails verification.
        #[test]
        fn corrupted_signature_fails(
            intent in arb_intent_value(),
            byte_idx in 0usize..64,
            xor in 1u8..=255,
        ) {
            let kp = Keypair::generate();
            let mut e = LedgerEntry::unsigned(&intent, None, b"")
                .unwrap()
                .sign(&kp.sk);
            
            if let Some(ref mut sig) = e.signature {
                sig.0[byte_idx] ^= xor;
            }
            
            prop_assert!(matches!(e.verify(), Err(LedgerError::SigInvalid)));
        }

        /// Wrong public key fails verification.
        #[test]
        fn wrong_pubkey_fails(intent in arb_intent_value()) {
            let kp1 = Keypair::generate();
            let kp2 = Keypair::generate();
            
            let mut e = LedgerEntry::unsigned(&intent, None, b"")
                .unwrap()
                .sign(&kp1.sk);
            
            // Replace pubkey with different key
            e.pubkey = Some(atomic_crypto::derive_public_bytes(&kp2.sk.0));
            
            prop_assert!(matches!(e.verify(), Err(LedgerError::SigInvalid)));
        }

        /// Signed entries roundtrip through file.
        #[test]
        fn signed_roundtrip_file(intent in arb_intent_value()) {
            let kp = Keypair::generate();
            let e = LedgerEntry::unsigned(&intent, None, b"")
                .unwrap()
                .sign(&kp.sk);
            
            let tmp = NamedTempFile::new().unwrap();
            {
                let mut w = SimpleLedgerWriter::open_append(tmp.path()).unwrap();
                w.append(&e).unwrap();
                w.sync().unwrap();
            }
            
            let read = SimpleLedgerReader::from_path(tmp.path())
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .unwrap();
            
            prop_assert_eq!(e.cid, read.cid);
            prop_assert_eq!(e.pubkey, read.pubkey);
            prop_assert!(read.verify().is_ok());
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Adversarial tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn reject_malformed_json_line() {
    let data = b"not valid json\n";
    let reader = SimpleLedgerReader::new(BufReader::new(Cursor::new(data)));
    let result = reader.iter().next().unwrap();
    assert!(matches!(result, Err(LedgerError::Serde(_))));
}

#[test]
fn reject_incomplete_entry() {
    // Missing required field
    let data = br#"{"ts":"2024-01-01T00:00:00Z","intent":[1,2,3]}"#;
    let reader = SimpleLedgerReader::new(BufReader::new(Cursor::new(&data[..])));
    let result = reader.iter().next().unwrap();
    assert!(matches!(result, Err(LedgerError::Serde(_))));
}

#[test]
fn reject_partial_signature_pubkey_only() {
    // Pubkey without signature
    let intent = json!({"test": "value"});
    let mut e = LedgerEntry::unsigned(&intent, None, b"").unwrap();
    e.pubkey = Some(atomic_types::PublicKeyBytes([0u8; 32]));
    
    let result = e.verify();
    assert!(matches!(result, Err(LedgerError::SigMissing)));
}

#[test]
fn reject_partial_signature_sig_only() {
    // Signature without pubkey
    let intent = json!({"test": "value"});
    let mut e = LedgerEntry::unsigned(&intent, None, b"").unwrap();
    e.signature = Some(atomic_types::SignatureBytes([0u8; 64]));
    
    let result = e.verify();
    assert!(matches!(result, Err(LedgerError::SigMissing)));
}

#[test]
fn empty_ledger_file() {
    let data = b"";
    let reader = SimpleLedgerReader::new(BufReader::new(Cursor::new(data)));
    let count = reader.iter().count();
    assert_eq!(count, 0);
}

#[test]
fn ledger_with_blank_lines() {
    // Some blank lines mixed in
    let intent = json!({"intent":"test"});
    let e = LedgerEntry::unsigned(&intent, None, b"").unwrap();
    let entry_json = serde_json::to_string(&e).unwrap();
    
    // Valid entry, then blank line
    let data = format!("{entry_json}\n\n");
    let reader = SimpleLedgerReader::new(BufReader::new(Cursor::new(data.as_bytes())));
    let results: Vec<_> = reader.iter().collect();
    
    // First should succeed, second (blank) should fail
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
    assert!(results[1].is_err()); // Empty line is not valid JSON
}

#[test]
fn extra_field_preserved() {
    let intent = json!({"intent":"test"});
    let extra = b"some metadata blob";
    let e = LedgerEntry::unsigned(&intent, None, extra).unwrap();
    
    let tmp = NamedTempFile::new().unwrap();
    {
        let mut w = SimpleLedgerWriter::open_append(tmp.path()).unwrap();
        w.append(&e).unwrap();
        w.sync().unwrap();
    }
    
    let read = SimpleLedgerReader::from_path(tmp.path())
        .unwrap()
        .iter()
        .next()
        .unwrap()
        .unwrap();
    
    assert_eq!(read.extra, extra);
}

#[test]
fn actor_field_preserved() {
    let intent = json!({"intent":"test"});
    let actor = Some("alice@example.com".to_string());
    let e = LedgerEntry::unsigned(&intent, actor.clone(), b"").unwrap();
    
    let tmp = NamedTempFile::new().unwrap();
    {
        let mut w = SimpleLedgerWriter::open_append(tmp.path()).unwrap();
        w.append(&e).unwrap();
        w.sync().unwrap();
    }
    
    let read = SimpleLedgerReader::from_path(tmp.path())
        .unwrap()
        .iter()
        .next()
        .unwrap()
        .unwrap();
    
    assert_eq!(read.actor, actor);
}

#[test]
fn large_batch_stress() {
    let tmp = NamedTempFile::new().unwrap();
    let count = 1000;
    
    // Write many entries
    {
        let mut w = SimpleLedgerWriter::open_append(tmp.path()).unwrap();
        for i in 0..count {
            let intent = json!({"seq": i, "data": format!("entry-{i}")});
            let e = LedgerEntry::unsigned(&intent, None, b"").unwrap();
            w.append(&e).unwrap();
        }
        w.sync().unwrap();
    }
    
    // Read and verify all
    let entries: Vec<_> = SimpleLedgerReader::from_path(tmp.path())
        .unwrap()
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    
    assert_eq!(entries.len(), count);
    
    // Verify each entry
    for (i, e) in entries.iter().enumerate() {
        assert!(e.verify().is_ok(), "Entry {i} failed verification");
    }
}

#[test]
fn different_actors_produce_same_cid() {
    // CID is only of intent, not actor
    let intent = json!({"intent":"test"});
    let e1 = LedgerEntry::unsigned(&intent, Some("alice".into()), b"").unwrap();
    let e2 = LedgerEntry::unsigned(&intent, Some("bob".into()), b"").unwrap();
    let e3 = LedgerEntry::unsigned(&intent, None, b"").unwrap();
    
    // All should have same CID (same intent)
    assert_eq!(e1.cid, e2.cid);
    assert_eq!(e2.cid, e3.cid);
}

#[test]
fn different_extra_produces_same_cid() {
    // CID is only of intent, not extra
    let intent = json!({"intent":"test"});
    let e1 = LedgerEntry::unsigned(&intent, None, b"extra1").unwrap();
    let e2 = LedgerEntry::unsigned(&intent, None, b"extra2").unwrap();
    
    assert_eq!(e1.cid, e2.cid);
}
