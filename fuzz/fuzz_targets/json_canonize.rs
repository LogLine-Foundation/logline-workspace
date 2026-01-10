//! Fuzz target for JSON✯Atomic canonicalization.
//!
//! Goals:
//! - No panics on arbitrary input
//! - No memory corruption
//! - Idempotence: canonize(parse(canonize(x))) == canonize(x)

#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    // Try to parse as JSON
    let Ok(value) = serde_json::from_slice::<Value>(data) else {
        return; // Invalid JSON, nothing to test
    };
    
    // Try to canonize
    let Ok(canon) = json_atomic::canonize(&value) else {
        return; // Contains floats or other unsupported, that's fine
    };
    
    // Verify idempotence: parse canonical output and re-canonize
    let reparsed: Value = serde_json::from_slice(&canon)
        .expect("canonical output must be valid JSON");
    
    let canon2 = json_atomic::canonize(&reparsed)
        .expect("canonical value must re-canonize");
    
    // Must be byte-identical
    assert_eq!(canon, canon2, "Canonicalization not idempotent");
    
    // CID must be stable
    let cid1 = atomic_crypto::blake3_cid(&canon);
    let cid2 = atomic_crypto::blake3_cid(&canon2);
    assert_eq!(cid1, cid2, "CID not stable");
});
