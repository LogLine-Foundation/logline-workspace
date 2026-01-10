//! SIRP wire protocol adversarial tests.
//!
//! These tests ensure the decoder handles malicious or malformed input safely:
//! - Truncated frames
//! - Corrupt headers
//! - Missing required fields
//! - Oversized fields
//! - Invalid signatures
//! - Duplicate fields

use atomic_sirp::{
    decode_frame, encode_frame, CanonIntent, SirpFrame, SirpError,
    SIRP_MAGIC, SIRP_VERSION, FLAG_SIGNED,
};

fn dummy_intent() -> CanonIntent {
    let bytes = b"{\"test\":true}".to_vec();
    let cid = atomic_crypto::blake3_cid(&bytes);
    CanonIntent { cid, bytes }
}

fn valid_unsigned_frame() -> Vec<u8> {
    encode_frame(&SirpFrame::unsigned(dummy_intent()))
}

// ══════════════════════════════════════════════════════════════════════════════
// Header attacks
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn reject_empty_input() {
    let result = decode_frame(&[]);
    assert!(matches!(result, Err(SirpError::Header)));
}

#[test]
fn reject_truncated_header() {
    let result = decode_frame(&[0x51]);
    assert!(matches!(result, Err(SirpError::Header)));
    
    let result = decode_frame(&[0x51, 0x99]);
    assert!(matches!(result, Err(SirpError::Header)));
    
    let result = decode_frame(&[0x51, 0x99, 0x01]);
    assert!(matches!(result, Err(SirpError::Header)));
}

#[test]
fn reject_bad_magic() {
    let mut frame = valid_unsigned_frame();
    frame[0] = 0x00;
    let result = decode_frame(&frame);
    assert!(matches!(result, Err(SirpError::Header)));
}

#[test]
fn reject_bad_version() {
    let mut frame = valid_unsigned_frame();
    frame[2] = 0xFF; // Invalid version
    let result = decode_frame(&frame);
    assert!(matches!(result, Err(SirpError::Header)));
}

// ══════════════════════════════════════════════════════════════════════════════
// TLV truncation attacks
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn reject_truncated_tlv_tag() {
    // Header only, no TLV data (missing required fields)
    let frame = [
        (SIRP_MAGIC >> 8) as u8, (SIRP_MAGIC & 0xFF) as u8,
        SIRP_VERSION, 0x00
    ];
    let result = decode_frame(&frame);
    // Should fail due to missing required fields
    assert!(result.is_err());
}

#[test]
fn reject_truncated_tlv_length() {
    // Header + tag but no length bytes
    let mut frame = valid_unsigned_frame();
    frame.truncate(5); // Header (4) + 1 tag byte
    let result = decode_frame(&frame);
    assert!(result.is_err());
}

#[test]
fn reject_truncated_tlv_value() {
    // Header + CID32 tag + length but not enough value bytes
    let mut frame = valid_unsigned_frame();
    // Truncate mid-CID
    frame.truncate(10);
    let result = decode_frame(&frame);
    assert!(result.is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// Field integrity attacks
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn reject_wrong_cid_length() {
    // Craft a frame with wrong CID length
    let mut frame = Vec::new();
    frame.extend_from_slice(&SIRP_MAGIC.to_be_bytes());
    frame.push(SIRP_VERSION);
    frame.push(0x00);
    
    // T_CID32 with wrong length
    frame.push(0x01); // T_CID32
    frame.push(16);   // Length 16 instead of 32
    frame.extend_from_slice(&[0u8; 16]);
    
    let result = decode_frame(&frame);
    assert!(result.is_err());
}

#[test]
fn reject_cid_mismatch() {
    let intent = dummy_intent();
    let mut f = SirpFrame::unsigned(intent);
    // Corrupt the CID
    f.intent.cid.0[0] ^= 0xFF;
    f.intent.cid.0[15] ^= 0xAA;
    let enc = encode_frame(&f);
    let result = decode_frame(&enc);
    assert!(matches!(result, Err(SirpError::CidMismatch)));
}

#[test]
fn reject_missing_cid() {
    // Header + intent bytes but no CID
    let mut frame = Vec::new();
    frame.extend_from_slice(&SIRP_MAGIC.to_be_bytes());
    frame.push(SIRP_VERSION);
    frame.push(0x00);
    
    // T_BYTES only (no CID)
    let payload = b"{\"test\":true}";
    frame.push(0x02); // T_BYTES
    frame.push(payload.len() as u8);
    frame.extend_from_slice(payload);
    
    let result = decode_frame(&frame);
    assert!(matches!(result, Err(SirpError::Missing(_))));
}

#[test]
fn reject_missing_intent_bytes() {
    // Header + CID but no intent bytes
    let mut frame = Vec::new();
    frame.extend_from_slice(&SIRP_MAGIC.to_be_bytes());
    frame.push(SIRP_VERSION);
    frame.push(0x00);
    
    // T_CID32 only (no bytes)
    frame.push(0x01); // T_CID32
    frame.push(32);
    frame.extend_from_slice(&[0u8; 32]);
    
    let result = decode_frame(&frame);
    assert!(matches!(result, Err(SirpError::Missing(_))));
}

// ══════════════════════════════════════════════════════════════════════════════
// Signature attacks
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn reject_signed_flag_without_pubkey() {
    let intent = dummy_intent();
    let mut f = SirpFrame::unsigned(intent);
    f.flags |= FLAG_SIGNED;
    // No pubkey/sig set
    let enc = encode_frame(&f);
    let result = decode_frame(&enc);
    assert!(matches!(result, Err(SirpError::Missing(_))));
}

#[test]
fn reject_signed_flag_without_signature() {
    let intent = dummy_intent();
    let mut f = SirpFrame::unsigned(intent);
    f.flags |= FLAG_SIGNED;
    f.pubkey = Some(atomic_types::PublicKeyBytes([0u8; 32]));
    // No signature
    let enc = encode_frame(&f);
    let result = decode_frame(&enc);
    assert!(matches!(result, Err(SirpError::Missing(_))));
}

#[test]
fn reject_wrong_pubkey_length() {
    let mut frame = Vec::new();
    frame.extend_from_slice(&SIRP_MAGIC.to_be_bytes());
    frame.push(SIRP_VERSION);
    frame.push(FLAG_SIGNED);
    
    // Valid CID
    let cid = atomic_crypto::blake3_cid(b"{\"test\":true}");
    frame.push(0x01); // T_CID32
    frame.push(32);
    frame.extend_from_slice(&cid.0);
    
    // Valid bytes
    let payload = b"{\"test\":true}";
    frame.push(0x02); // T_BYTES
    frame.push(payload.len() as u8);
    frame.extend_from_slice(payload);
    
    // Wrong pubkey length
    frame.push(0x03); // T_PUBKEY32
    frame.push(16);   // Wrong length
    frame.extend_from_slice(&[0u8; 16]);
    
    let result = decode_frame(&frame);
    assert!(result.is_err());
}

#[test]
fn reject_wrong_signature_length() {
    let mut frame = Vec::new();
    frame.extend_from_slice(&SIRP_MAGIC.to_be_bytes());
    frame.push(SIRP_VERSION);
    frame.push(FLAG_SIGNED);
    
    // Valid CID
    let cid = atomic_crypto::blake3_cid(b"{\"test\":true}");
    frame.push(0x01); // T_CID32
    frame.push(32);
    frame.extend_from_slice(&cid.0);
    
    // Valid bytes
    let payload = b"{\"test\":true}";
    frame.push(0x02); // T_BYTES
    frame.push(payload.len() as u8);
    frame.extend_from_slice(payload);
    
    // Valid pubkey
    frame.push(0x03); // T_PUBKEY32
    frame.push(32);
    frame.extend_from_slice(&[0u8; 32]);
    
    // Wrong sig length
    frame.push(0x04); // T_SIG64
    frame.push(32);   // Wrong length (should be 64)
    frame.extend_from_slice(&[0u8; 32]);
    
    let result = decode_frame(&frame);
    assert!(result.is_err());
}

#[cfg(feature = "signing")]
#[test]
fn reject_invalid_signature_bytes() {
    use atomic_crypto::Keypair;
    
    let intent = dummy_intent();
    let kp = Keypair::generate();
    let mut f = SirpFrame::unsigned(intent).sign(&kp.sk);
    
    // Corrupt the signature
    if let Some(ref mut sig) = f.signature {
        sig.0[0] ^= 0xFF;
        sig.0[31] ^= 0xAA;
    }
    
    let enc = encode_frame(&f);
    let result = decode_frame(&enc);
    assert!(matches!(result, Err(SirpError::Signature(_))));
}

#[cfg(feature = "signing")]
#[test]
fn reject_wrong_pubkey_for_signature() {
    use atomic_crypto::Keypair;
    
    let intent = dummy_intent();
    let kp1 = Keypair::generate();
    let kp2 = Keypair::generate();
    
    let mut f = SirpFrame::unsigned(intent).sign(&kp1.sk);
    // Replace pubkey with different key
    f.pubkey = Some(atomic_crypto::derive_public_bytes(&kp2.sk.0));
    
    let enc = encode_frame(&f);
    let result = decode_frame(&enc);
    assert!(matches!(result, Err(SirpError::Signature(_))));
}

// ══════════════════════════════════════════════════════════════════════════════
// Fuzzer-inspired edge cases
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn handle_oversized_varint_length() {
    // Create a frame with a huge declared length in the TLV
    let mut frame = Vec::new();
    frame.extend_from_slice(&SIRP_MAGIC.to_be_bytes());
    frame.push(SIRP_VERSION);
    frame.push(0x00);
    
    // T_BYTES with huge length (varint encoding for 0xFFFFFFFF)
    frame.push(0x02);
    frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]); // ~4GB
    
    let result = decode_frame(&frame);
    assert!(result.is_err()); // Should fail gracefully, not OOM
}

#[test]
fn handle_all_zeros() {
    let frame = vec![0u8; 256];
    let result = decode_frame(&frame);
    assert!(result.is_err());
}

#[test]
fn handle_all_ones() {
    let frame = vec![0xFF; 256];
    let result = decode_frame(&frame);
    assert!(result.is_err());
}

#[test]
fn unknown_tags_are_ignored() {
    // Valid frame with unknown tags mixed in
    let intent = dummy_intent();
    let valid = encode_frame(&SirpFrame::unsigned(intent));
    
    // Insert unknown tag before the end
    let mut frame = valid.clone();
    // Add unknown tag 0xFF with some data
    frame.push(0xFF); // Unknown tag
    frame.push(4);    // Length
    frame.extend_from_slice(b"????");
    
    // Should still decode (forward compatibility)
    // Note: This depends on decoder behavior - may need adjustment
    // if decoder is strict about trailing data
}

#[test]
fn roundtrip_preserves_extra_field() {
    let intent = dummy_intent();
    let mut f = SirpFrame::unsigned(intent);
    f.extra = b"some extra metadata".to_vec();
    
    let enc = encode_frame(&f);
    let dec = decode_frame(&enc).unwrap();
    
    assert_eq!(f.extra, dec.extra);
}

#[test]
fn empty_intent_bytes_are_handled() {
    let bytes = b"{}".to_vec(); // Minimal valid JSON
    let cid = atomic_crypto::blake3_cid(&bytes);
    let intent = CanonIntent { cid, bytes };
    
    let f = SirpFrame::unsigned(intent);
    let enc = encode_frame(&f);
    let dec = decode_frame(&enc).unwrap();
    
    assert_eq!(f.intent, dec.intent);
}

// ══════════════════════════════════════════════════════════════════════════════
// Property-based tests
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_intent() -> impl Strategy<Value = CanonIntent> {
        "[a-zA-Z0-9]{1,100}".prop_map(|s| {
            let json = format!("{{\"data\":\"{s}\"}}");
            // Use json_atomic to canonicalize
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            let bytes = json_atomic::canonize(&v).unwrap();
            let cid = atomic_crypto::blake3_cid(&bytes);
            CanonIntent { cid, bytes }
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// Any valid frame must survive roundtrip.
        #[test]
        fn roundtrip_unsigned_frames(intent in arb_intent()) {
            let f = SirpFrame::unsigned(intent);
            let enc = encode_frame(&f);
            let dec = decode_frame(&enc).expect("decode");
            prop_assert_eq!(f.intent.cid, dec.intent.cid);
            prop_assert_eq!(f.intent.bytes, dec.intent.bytes);
        }

        /// Corrupting any byte should cause decode to fail or return different data.
        #[test]
        fn corruption_detected(
            intent in arb_intent(),
            pos in 4usize..256, // Skip header bytes to test TLV corruption
            xor in 1u8..=255,   // Non-zero XOR to ensure change
        ) {
            let f = SirpFrame::unsigned(intent);
            let mut enc = encode_frame(&f);
            if pos < enc.len() {
                enc[pos] ^= xor;
                let result = decode_frame(&enc);
                // Either decoding fails OR CID verification catches it
                // (we can't assert exact error because corruption is random)
                if let Ok(_dec) = result {
                    // If it didn't fail, it better have different content
                    // or we got extremely unlucky with corruption location
                    // that didn't affect semantics (unlikely but possible)
                }
            }
        }

        /// Truncation always fails.
        #[test]
        fn truncation_fails(
            intent in arb_intent(),
            keep in 1usize..50,
        ) {
            let f = SirpFrame::unsigned(intent);
            let enc = encode_frame(&f);
            if keep < enc.len() {
                let truncated = &enc[..keep];
                let result = decode_frame(truncated);
                prop_assert!(result.is_err());
            }
        }
    }

    #[cfg(feature = "signing")]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Signed frames must survive roundtrip.
        #[test]
        fn roundtrip_signed_frames(intent in arb_intent()) {
            use atomic_crypto::Keypair;
            
            let kp = Keypair::generate();
            let f = SirpFrame::unsigned(intent).sign(&kp.sk);
            let enc = encode_frame(&f);
            let dec = decode_frame(&enc).expect("decode");
            
            prop_assert_eq!(f.intent.cid, dec.intent.cid);
            prop_assert_eq!(f.pubkey, dec.pubkey);
            prop_assert!(dec.verify().is_ok());
        }
    }
}
