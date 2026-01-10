//! Fuzz target for SIRP wire protocol decoding.
//!
//! Goals:
//! - No panics on arbitrary bytes
//! - No memory allocation bombs
//! - Graceful rejection of malformed frames

#![no_main]
use libfuzzer_sys::fuzz_target;
use atomic_sirp::{decode_frame, encode_frame};

fuzz_target!(|data: &[u8]| {
    // Try to decode arbitrary bytes
    let result = decode_frame(data);
    
    // If decode succeeds, the frame is valid (CID + sig verified)
    if let Ok(frame) = result {
        // Verify method should also pass
        if frame.verify().is_ok() {
            // Roundtrip: encode and decode again
            let wire = encode_frame(&frame);
            let decoded = decode_frame(&wire)
                .expect("roundtrip must succeed");
            
            assert_eq!(frame.version, decoded.version);
            assert_eq!(frame.flags, decoded.flags);
            assert_eq!(frame.intent.cid, decoded.intent.cid);
            assert_eq!(frame.intent.bytes, decoded.intent.bytes);
        }
    }
    // Errors are fine - we're testing that bad input is handled gracefully
});
