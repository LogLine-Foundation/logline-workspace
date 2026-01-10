//! Fuzz target for TLV decoder (atomic-codec).
//!
//! Goals:
//! - No panics on arbitrary bytes
//! - No infinite loops
//! - Graceful rejection of malformed input

#![no_main]
use libfuzzer_sys::fuzz_target;
use atomic_codec::binary::{decode_frame, encode_frame, decode_varint_u64};

fuzz_target!(|data: &[u8]| {
    // Test varint decoding
    let mut pos = 0;
    let _ = decode_varint_u64(data, &mut pos);
    
    // Test frame decoding
    let result = decode_frame(data);
    
    // If decode succeeds, verify roundtrip
    if let Ok((tag, value)) = result {
        let reencoded = encode_frame(tag, value);
        
        // Decode the reencoded frame
        let (tag2, value2) = decode_frame(&reencoded)
            .expect("roundtrip must succeed");
        
        assert_eq!(tag, tag2, "tag mismatch on roundtrip");
        assert_eq!(value, value2, "value mismatch on roundtrip");
    }
});
