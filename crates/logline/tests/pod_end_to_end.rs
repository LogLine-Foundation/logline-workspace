//! Proof of Design (PoD) end-to-end test.
//!
//! This test verifies the complete LogLine pipeline:
//! 
//! 1. **JSON Input** → any JSON object (intent)
//! 2. **Canonicalization** → json_atomic produces deterministic bytes
//! 3. **CID** → BLAKE3 of canonical bytes
//! 4. **SIRP Frame** → TLV wire format with optional signature
//! 5. **UBL Ledger** → Append-only NDJSON with verification
//! 6. **Read Back** → Verify CID matches at every stage
//!
//! This tests cross-crate integration without mocking.

use atomic_crypto::Keypair;
use atomic_sirp::{
    decode_frame, encode_frame, CanonIntent, SirpFrame, FLAG_SIGNED, SIRP_VERSION,
};
use atomic_ubl::{LedgerEntry, SimpleLedgerReader, SimpleLedgerWriter};
use serde_json::{json, Value};
use tempfile::NamedTempFile;

/// Full pipeline test: JSON → Canon → SIRP Frame → Wire → Decode → UBL → Read
#[test]
fn full_pipeline_unsigned() {
    // ═══════════════════════════════════════════════════════════════════════════
    // Stage 1: Original intent (arbitrary key order)
    // ═══════════════════════════════════════════════════════════════════════════
    let original = json!({
        "type": "Grant",
        "to": "alice@example.com",
        "amount": 1000,
        "currency": "UBL",
        "memo": "Monthly grant"
    });
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Stage 2: Canonicalize (json_atomic)
    // ═══════════════════════════════════════════════════════════════════════════
    let canon_bytes = json_atomic::canonize(&original).expect("canonize");
    let cid = atomic_crypto::blake3_cid(&canon_bytes);
    
    // Verify canonicalization is deterministic
    let canon_bytes_2 = json_atomic::canonize(&original).expect("canonize again");
    assert_eq!(canon_bytes, canon_bytes_2, "Canonicalization not deterministic");
    
    // Verify CID is deterministic
    let cid_2 = atomic_crypto::blake3_cid(&canon_bytes_2);
    assert_eq!(cid, cid_2, "CID not deterministic");
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Stage 3: Create SIRP frame (atomic-sirp wire)
    // ═══════════════════════════════════════════════════════════════════════════
    let intent = CanonIntent {
        cid: cid.clone(),
        bytes: canon_bytes.clone(),
    };
    let frame = SirpFrame::unsigned(intent);
    
    assert_eq!(frame.version, SIRP_VERSION);
    assert_eq!(frame.flags, 0);
    assert_eq!(frame.intent.cid, cid);
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Stage 4: Encode to wire format
    // ═══════════════════════════════════════════════════════════════════════════
    let wire_bytes = encode_frame(&frame);
    assert!(!wire_bytes.is_empty(), "Wire bytes empty");
    
    // Wire must start with SIRP magic
    assert_eq!(wire_bytes[0], 0x51);
    assert_eq!(wire_bytes[1], 0x99);
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Stage 5: Decode from wire format (simulates network receive)
    // ═══════════════════════════════════════════════════════════════════════════
    let decoded = decode_frame(&wire_bytes).expect("decode");
    
    // Verify all fields match
    assert_eq!(decoded.version, frame.version);
    assert_eq!(decoded.flags, frame.flags);
    assert_eq!(decoded.intent.cid, frame.intent.cid);
    assert_eq!(decoded.intent.bytes, frame.intent.bytes);
    
    // CID verification happened inside decode_frame
    decoded.verify().expect("frame verify");
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Stage 6: Append to UBL ledger
    // ═══════════════════════════════════════════════════════════════════════════
    let tmp = NamedTempFile::new().expect("tempfile");
    
    // Parse intent bytes back to Value for UBL
    let intent_value: Value = serde_json::from_slice(&decoded.intent.bytes)
        .expect("parse intent");
    
    let entry = LedgerEntry::unsigned(&intent_value, Some("test-actor".into()), b"")
        .expect("create entry");
    
    // CID must match what SIRP decoded
    assert_eq!(entry.cid, decoded.intent.cid, "UBL CID != SIRP CID");
    
    {
        let mut writer = SimpleLedgerWriter::open_append(tmp.path()).expect("open writer");
        writer.append(&entry).expect("append");
        writer.sync().expect("sync");
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Stage 7: Read back from ledger and verify
    // ═══════════════════════════════════════════════════════════════════════════
    let read_entry = SimpleLedgerReader::from_path(tmp.path())
        .expect("open reader")
        .iter()
        .next()
        .expect("has entry")
        .expect("valid entry");
    
    // All CIDs must match the original
    assert_eq!(read_entry.cid, cid, "Read CID != original CID");
    assert_eq!(read_entry.intent, canon_bytes, "Read intent != original");
    read_entry.verify().expect("entry verify");
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Final: Parse back and compare semantically
    // ═══════════════════════════════════════════════════════════════════════════
    let final_value: Value = serde_json::from_slice(&read_entry.intent)
        .expect("parse final");
    assert_eq!(original, final_value, "Semantic content changed");
    
    println!("✅ Full unsigned pipeline passed!");
    println!("   CID: {}", hex::encode(&cid.0[..8]));
    println!("   Wire bytes: {} bytes", wire_bytes.len());
    println!("   Canon bytes: {} bytes", canon_bytes.len());
}

/// Full pipeline with signature
#[test]
fn full_pipeline_signed() {
    let kp = Keypair::generate();
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Intent
    // ═══════════════════════════════════════════════════════════════════════════
    let original = json!({
        "action": "Transfer",
        "from": "alice",
        "to": "bob",
        "amount": 500
    });
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Canonicalize
    // ═══════════════════════════════════════════════════════════════════════════
    let canon_bytes = json_atomic::canonize(&original).expect("canonize");
    let cid = atomic_crypto::blake3_cid(&canon_bytes);
    
    // ═══════════════════════════════════════════════════════════════════════════
    // SIRP Frame (signed)
    // ═══════════════════════════════════════════════════════════════════════════
    let intent = CanonIntent {
        cid: cid.clone(),
        bytes: canon_bytes.clone(),
    };
    let frame = SirpFrame::unsigned(intent).sign(&kp.sk);
    
    assert_eq!(frame.flags & FLAG_SIGNED, FLAG_SIGNED);
    assert!(frame.pubkey.is_some());
    assert!(frame.signature.is_some());
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Wire roundtrip
    // ═══════════════════════════════════════════════════════════════════════════
    let wire_bytes = encode_frame(&frame);
    let decoded = decode_frame(&wire_bytes).expect("decode signed");
    
    assert_eq!(decoded.flags & FLAG_SIGNED, FLAG_SIGNED);
    assert!(decoded.pubkey.is_some());
    decoded.verify().expect("signed frame verify");
    
    // ═══════════════════════════════════════════════════════════════════════════
    // UBL Ledger (also signed)
    // ═══════════════════════════════════════════════════════════════════════════
    let tmp = NamedTempFile::new().expect("tempfile");
    
    let intent_value: Value = serde_json::from_slice(&decoded.intent.bytes)
        .expect("parse intent");
    
    let entry = LedgerEntry::unsigned(&intent_value, None, b"")
        .expect("create entry")
        .sign(&kp.sk);
    
    // CID must match
    assert_eq!(entry.cid, cid);
    assert!(entry.pubkey.is_some());
    assert!(entry.signature.is_some());
    
    {
        let mut writer = SimpleLedgerWriter::open_append(tmp.path()).expect("open writer");
        writer.append(&entry).expect("append");
        writer.sync().expect("sync");
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // Read back
    // ═══════════════════════════════════════════════════════════════════════════
    let read_entry = SimpleLedgerReader::from_path(tmp.path())
        .expect("open reader")
        .iter()
        .next()
        .expect("has entry")
        .expect("valid entry");
    
    assert_eq!(read_entry.cid, cid);
    assert!(read_entry.pubkey.is_some());
    assert!(read_entry.signature.is_some());
    read_entry.verify().expect("entry verify");
    
    println!("✅ Full signed pipeline passed!");
    println!("   CID: {}", hex::encode(&cid.0[..8]));
    println!("   Pubkey: {}", hex::encode(&read_entry.pubkey.unwrap().0[..8]));
}

/// Test that key order doesn't affect CID across the entire pipeline
#[test]
fn key_order_invariance_across_pipeline() {
    // Two JSONs with same content, different key order
    let v1 = json!({
        "z": 3,
        "a": 1,
        "m": 2
    });
    
    let v2 = json!({
        "a": 1,
        "m": 2,
        "z": 3
    });
    
    // Canonicalize both
    let c1 = json_atomic::canonize(&v1).expect("canon v1");
    let c2 = json_atomic::canonize(&v2).expect("canon v2");
    
    // Must be byte-identical
    assert_eq!(c1, c2, "Canonical bytes differ for same content");
    
    // CIDs must match
    let cid1 = atomic_crypto::blake3_cid(&c1);
    let cid2 = atomic_crypto::blake3_cid(&c2);
    assert_eq!(cid1, cid2, "CIDs differ for same content");
    
    // SIRP frames must match
    let frame1 = SirpFrame::unsigned(CanonIntent { cid: cid1, bytes: c1.clone() });
    let frame2 = SirpFrame::unsigned(CanonIntent { cid: cid2, bytes: c2.clone() });
    
    let wire1 = encode_frame(&frame1);
    let wire2 = encode_frame(&frame2);
    assert_eq!(wire1, wire2, "Wire bytes differ for same content");
    
    // UBL entries must match
    let e1 = LedgerEntry::unsigned(&v1, None, b"").expect("entry v1");
    let e2 = LedgerEntry::unsigned(&v2, None, b"").expect("entry v2");
    assert_eq!(e1.cid, e2.cid, "UBL CIDs differ");
    assert_eq!(e1.intent, e2.intent, "UBL intents differ");
    
    println!("✅ Key order invariance verified across entire pipeline!");
}

/// Test nested object key ordering
#[test]
fn nested_objects_sorted_correctly() {
    let original = json!({
        "outer_z": {
            "inner_z": 3,
            "inner_a": 1
        },
        "outer_a": {
            "inner_m": 2,
            "inner_b": 0
        }
    });
    
    let canon = json_atomic::canonize(&original).expect("canonize");
    let canon_str = std::str::from_utf8(&canon).expect("valid utf8");
    
    // Verify key order: outer_a before outer_z
    let a_pos = canon_str.find("\"outer_a\"").expect("outer_a");
    let z_pos = canon_str.find("\"outer_z\"").expect("outer_z");
    assert!(a_pos < z_pos, "outer_a should come before outer_z");
    
    // Round-trip through SIRP and UBL
    let cid = atomic_crypto::blake3_cid(&canon);
    let frame = SirpFrame::unsigned(CanonIntent { cid: cid.clone(), bytes: canon.clone() });
    let wire = encode_frame(&frame);
    let decoded = decode_frame(&wire).expect("decode");
    
    assert_eq!(decoded.intent.bytes, canon);
    assert_eq!(decoded.intent.cid, cid);
    
    println!("✅ Nested objects sorted correctly!");
}

/// Test multiple entries with different content types
#[test]
fn multiple_entry_types() {
    let tmp = NamedTempFile::new().expect("tempfile");
    
    let intents = vec![
        json!({"type": "Grant", "to": "alice", "amount": 100}),
        json!({"type": "Revoke", "id": "grant-123"}),
        json!({"type": "Freeze", "account": "bob"}),
        json!({"type": "Transfer", "from": "alice", "to": "bob", "amount": 50}),
    ];
    
    let mut cids = Vec::new();
    
    // Write all
    {
        let mut writer = SimpleLedgerWriter::open_append(tmp.path()).expect("open writer");
        for intent in &intents {
            let entry = LedgerEntry::unsigned(intent, None, b"").expect("create entry");
            cids.push(entry.cid.clone());
            writer.append(&entry).expect("append");
        }
        writer.sync().expect("sync");
    }
    
    // Read back and verify
    let entries: Vec<_> = SimpleLedgerReader::from_path(tmp.path())
        .expect("open reader")
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("all valid");
    
    assert_eq!(entries.len(), intents.len());
    
    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.cid, cids[i], "CID mismatch at entry {i}");
        entry.verify().expect(&format!("verify entry {i}"));
        
        // Re-create entry and verify CID matches
        let fresh = LedgerEntry::unsigned(&intents[i], None, b"").expect("fresh entry");
        assert_eq!(entry.cid, fresh.cid, "Fresh CID mismatch at entry {i}");
    }
    
    println!("✅ Multiple entry types all verified!");
}

/// Stress test: many entries through complete pipeline
#[test]
fn stress_pipeline() {
    let tmp = NamedTempFile::new().expect("tempfile");
    let count = 100;
    
    let mut cids = Vec::new();
    
    // Generate and write
    {
        let mut writer = SimpleLedgerWriter::open_append(tmp.path()).expect("open writer");
        for i in 0..count {
            let intent = json!({
                "type": "Event",
                "seq": i,
                "data": format!("payload-{i}-{}", i * 17 % 97)
            });
            
            // Full pipeline
            let canon = json_atomic::canonize(&intent).expect("canonize");
            let cid = atomic_crypto::blake3_cid(&canon);
            
            let frame = SirpFrame::unsigned(CanonIntent { cid: cid.clone(), bytes: canon });
            let wire = encode_frame(&frame);
            let decoded = decode_frame(&wire).expect("decode");
            
            let entry = LedgerEntry::unsigned(&intent, None, b"").expect("entry");
            assert_eq!(entry.cid, decoded.intent.cid);
            
            cids.push(entry.cid.clone());
            writer.append(&entry).expect("append");
        }
        writer.sync().expect("sync");
    }
    
    // Read and verify
    let entries: Vec<_> = SimpleLedgerReader::from_path(tmp.path())
        .expect("open reader")
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("all valid");
    
    assert_eq!(entries.len(), count);
    
    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.cid, cids[i], "CID mismatch at {i}");
    }
    
    println!("✅ Stress test passed: {count} entries through full pipeline!");
}
