//! Fuzz target for UBL ledger entry parsing.
//!
//! Goals:
//! - No panics on arbitrary NDJSON lines
//! - CID verification catches any corruption
//! - Graceful rejection of malformed entries

#![no_main]
use libfuzzer_sys::fuzz_target;
use atomic_ubl::LedgerEntry;

fuzz_target!(|data: &[u8]| {
    // Try to parse as NDJSON line
    let line = match std::str::from_utf8(data) {
        Ok(s) => s.trim(),
        Err(_) => return, // Not valid UTF-8
    };
    
    if line.is_empty() {
        return;
    }
    
    // Try to parse as LedgerEntry
    let entry: Result<LedgerEntry, _> = serde_json::from_str(line);
    
    if let Ok(entry) = entry {
        // If parsing succeeded, verify the entry
        // This checks CID and signature
        let _ = entry.verify();
        
        // Even if verify fails, we shouldn't panic
    }
    // Parse errors are fine - we're testing robustness
});
