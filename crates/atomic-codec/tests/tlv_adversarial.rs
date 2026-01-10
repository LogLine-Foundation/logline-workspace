//! Adversarial tests for binary TLV codec.
//!
//! These tests verify that the codec correctly rejects:
//! - Frames with forged/excessive declared lengths (DoS guard)
//! - Varint overflow attacks
//! - Truncated varints
//! - Malformed tag sequences

use atomic_codec::binary::*;

#[test]
fn reject_frame_with_huge_declared_len() {
    // Frame with length forged > MAX_FRAME_LEN
    let mut bad = vec![];
    bad.push(0x42); // frame type
    // Encode varint for (MAX_FRAME_LEN + 1)
    encode_varint_u64((MAX_FRAME_LEN as u64) + 1, &mut bad);
    // Add some garbage payload (doesn't matter - should reject before reading)
    bad.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

    let result = decode_frame(&bad);
    assert!(
        matches!(result, Err(BinaryCodecError::SizeLimit { .. })),
        "Expected SizeLimit error, got: {:?}",
        result
    );
}

#[test]
fn reject_frame_exactly_at_limit_boundary() {
    // Frame with length exactly at MAX_FRAME_LEN should succeed (if data present)
    let mut exactly_at_limit = vec![];
    exactly_at_limit.push(0x42);
    encode_varint_u64(MAX_FRAME_LEN as u64, &mut exactly_at_limit);
    // Don't provide the actual payload - should get EOF, not SizeLimit
    let result = decode_frame(&exactly_at_limit);
    assert!(
        matches!(result, Err(BinaryCodecError::Eof)),
        "Expected Eof (truncated), got: {:?}",
        result
    );

    // Frame with length one over limit
    let mut over_limit = vec![];
    over_limit.push(0x42);
    encode_varint_u64((MAX_FRAME_LEN + 1) as u64, &mut over_limit);
    let result = decode_frame(&over_limit);
    assert!(
        matches!(result, Err(BinaryCodecError::SizeLimit { .. })),
        "Expected SizeLimit, got: {:?}",
        result
    );
}

#[test]
fn reject_varint_with_too_many_bytes() {
    // 20 bytes of 0xFF (continuation bit set) should fail
    // Either VarintOverflow (exceeded MAX_VARINT_BYTES) or Varint (shift > 63)
    let bad: Vec<u8> = std::iter::repeat(0xFF).take(20).collect();
    let mut pos = 0;
    let result = decode_varint_u64(&bad, &mut pos);
    assert!(
        matches!(result, Err(BinaryCodecError::VarintOverflow | BinaryCodecError::Varint)),
        "Expected VarintOverflow or Varint, got: {:?}",
        result
    );
}

#[test]
fn reject_varint_exactly_at_limit() {
    // MAX_VARINT_BYTES (10) bytes of 0x80 followed by proper termination
    // should still fail because continuation never stops within limit
    let mut bad: Vec<u8> = std::iter::repeat(0x80).take(MAX_VARINT_BYTES).collect();
    bad.push(0x01); // terminator beyond limit
    let mut pos = 0;
    let result = decode_varint_u64(&bad, &mut pos);
    assert!(
        matches!(result, Err(BinaryCodecError::VarintOverflow | BinaryCodecError::Varint)),
        "Expected VarintOverflow or Varint, got: {:?}",
        result
    );
}

#[test]
fn reject_truncated_varint() {
    // Single byte with continuation bit - no terminator
    let trunc = vec![0x80];
    let mut pos = 0;
    let result = decode_varint_u64(&trunc, &mut pos);
    assert!(
        matches!(result, Err(BinaryCodecError::Eof)),
        "Expected Eof (truncated varint), got: {:?}",
        result
    );

    // Multiple continuation bytes with no terminator
    let trunc2 = vec![0x80, 0x80, 0x80];
    let mut pos2 = 0;
    let result2 = decode_varint_u64(&trunc2, &mut pos2);
    assert!(
        matches!(result2, Err(BinaryCodecError::Eof)),
        "Expected Eof, got: {:?}",
        result2
    );
}

#[test]
fn reject_empty_frame() {
    let empty: &[u8] = &[];
    let result = decode_frame(empty);
    assert!(
        matches!(result, Err(BinaryCodecError::Eof)),
        "Expected Eof, got: {:?}",
        result
    );
}

#[test]
fn decoder_rejects_oversized_bytes() {
    // Build a valid-looking T_BYTES with huge declared length
    let mut bad = vec![];
    bad.push(T_BYTES);
    encode_varint_u64((MAX_BYTES_LEN + 1) as u64, &mut bad);
    // Add some garbage
    bad.extend_from_slice(&[0x00; 16]);

    let mut dec = Decoder::new(&bad);
    let result = dec.bytes();
    assert!(
        matches!(result, Err(BinaryCodecError::SizeLimit { .. })),
        "Expected SizeLimit, got: {:?}",
        result
    );
}

#[test]
fn decoder_rejects_oversized_string() {
    let mut bad = vec![];
    bad.push(T_STR);
    encode_varint_u64((MAX_BYTES_LEN + 1) as u64, &mut bad);
    bad.extend_from_slice(b"garbage");

    let mut dec = Decoder::new(&bad);
    let result = dec.str();
    assert!(
        matches!(result, Err(BinaryCodecError::SizeLimit { .. })),
        "Expected SizeLimit, got: {:?}",
        result
    );
}

#[test]
fn decoder_rejects_invalid_utf8() {
    let mut bad = vec![];
    bad.push(T_STR);
    encode_varint_u64(4, &mut bad);
    bad.extend_from_slice(&[0xFF, 0xFE, 0x00, 0x00]); // Invalid UTF-8

    let mut dec = Decoder::new(&bad);
    let result = dec.str();
    assert!(
        matches!(result, Err(BinaryCodecError::Utf8)),
        "Expected Utf8 error, got: {:?}",
        result
    );
}

#[test]
fn decoder_rejects_wrong_tag() {
    let mut data = vec![];
    data.push(T_U64); // Encode as U64
    encode_varint_u64(42, &mut data);

    let mut dec = Decoder::new(&data);
    // Try to read as CID32
    let result = dec.cid32();
    assert!(
        matches!(result, Err(BinaryCodecError::Tag { .. })),
        "Expected Tag mismatch, got: {:?}",
        result
    );
}

#[test]
fn decoder_handles_truncated_fixed_size() {
    // T_CID32 but only 16 bytes of data
    let mut data = vec![];
    data.push(T_CID32);
    data.extend_from_slice(&[0xAB; 16]); // Only 16 bytes, need 32

    let mut dec = Decoder::new(&data);
    let result = dec.cid32();
    assert!(
        matches!(result, Err(BinaryCodecError::Eof)),
        "Expected Eof (truncated), got: {:?}",
        result
    );
}
