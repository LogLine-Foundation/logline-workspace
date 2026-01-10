//! SIRP Wire Protocol — minimal TLV format for intent routing.
//!
//! Uses `atomic-codec::binary` for TLV encoding with `atomic-types` primitives.
//! Canonicalization via `json_atomic` ensures deterministic CID.

use ubl_codec::binary::{
    decode_varint_u64, encode_varint_u64, BinaryCodecError, T_BYTES, T_CID32, T_PUBKEY32, T_SIG64,
};
use ubl_types::{Cid32, PublicKeyBytes, SignatureBytes};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// SIRP magic bytes (Paper V).
pub const SIRP_MAGIC: u16 = 0x5199;
/// Current wire version.
pub const SIRP_VERSION: u8 = 1;

/// Flags bitmask.
pub const FLAG_SIGNED: u8 = 0b0000_0001;

/// Domain separator for frame signing (contextual binding).
pub const DOMAIN_FRAME_SIGN: &[u8] = b"SIRP:FRAME:v1";

/// SIRP wire errors.
#[derive(Debug, Error)]
pub enum SirpError {
    /// Invalid magic, version, or flags.
    #[error("bad magic/version/flags")]
    Header,
    /// Missing required field.
    #[error("missing required field: {0}")]
    Missing(&'static str),
    /// TLV decode error.
    #[error("decode error: {0}")]
    Decode(String),
    /// Canonicalization error.
    #[error("canonicalization error: {0}")]
    Canon(String),
    /// CID mismatch (content integrity).
    #[error("cid mismatch")]
    CidMismatch,
    /// Signature verification failed.
    #[error("signature error: {0}")]
    Signature(String),
}

impl From<BinaryCodecError> for SirpError {
    fn from(e: BinaryCodecError) -> Self {
        SirpError::Decode(format!("{e}"))
    }
}

/// Canonical intent: bytes + their CID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonIntent {
    /// CID = BLAKE3(canonical bytes).
    pub cid: Cid32,
    /// Canonical JSON bytes.
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

/// SIRP frame (minimal wire format).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SirpFrame {
    /// Protocol version (1).
    pub version: u8,
    /// Flags (FLAG_SIGNED if signed).
    pub flags: u8,
    /// Canonical intent (CID + bytes).
    pub intent: CanonIntent,
    /// Optional extra blob (reserved for future).
    #[serde(with = "serde_bytes", default)]
    pub extra: Vec<u8>,
    /// Public key (if FLAG_SIGNED).
    pub pubkey: Option<PublicKeyBytes>,
    /// Signature (if FLAG_SIGNED).
    pub signature: Option<SignatureBytes>,
}

impl SirpFrame {
    /// Creates an unsigned frame.
    #[must_use]
    pub fn unsigned(intent: CanonIntent) -> Self {
        Self {
            version: SIRP_VERSION,
            flags: 0,
            intent,
            extra: Vec::new(),
            pubkey: None,
            signature: None,
        }
    }

    /// Signs the frame (adds FLAG_SIGNED, pubkey, signature).
    #[cfg(feature = "signing")]
    pub fn sign(mut self, sk: &ubl_crypto::SecretKey) -> Self {
        use ubl_crypto::{derive_public_bytes, sign_bytes};
        self.flags |= FLAG_SIGNED;
        self.pubkey = Some(derive_public_bytes(&sk.0));
        let msg = sign_message(self.version, self.flags, &self.intent.cid);
        let sig = sign_bytes(&msg, &sk.0);
        self.signature = Some(sig);
        self
    }

    /// Verifies: (1) CID == BLAKE3(intent.bytes); (2) signature if FLAG_SIGNED.
    ///
    /// # Errors
    ///
    /// Returns error if CID mismatch or signature invalid.
    pub fn verify(&self) -> Result<(), SirpError> {
        // 1) CID check
        let cid_check = ubl_crypto::blake3_cid(&self.intent.bytes);
        if cid_check != self.intent.cid {
            return Err(SirpError::CidMismatch);
        }

        // 2) Signature check (if signed)
        if self.flags & FLAG_SIGNED == FLAG_SIGNED {
            let pk = self.pubkey.as_ref().ok_or(SirpError::Missing("pubkey"))?;
            let sig = self
                .signature
                .as_ref()
                .ok_or(SirpError::Missing("signature"))?;
            let msg = sign_message(self.version, self.flags, &self.intent.cid);
            #[cfg(feature = "signing")]
            {
                use ubl_crypto::verify_bytes;
                if !verify_bytes(&msg, pk, sig) {
                    return Err(SirpError::Signature("invalid signature".into()));
                }
            }
            #[cfg(not(feature = "signing"))]
            {
                let _ = (pk, sig, msg);
                return Err(SirpError::Signature("signing feature disabled".into()));
            }
        }
        Ok(())
    }
}

/// Builds the message to sign (domain + version + flags + CID).
fn sign_message(version: u8, flags: u8, cid: &Cid32) -> Vec<u8> {
    let mut m = Vec::with_capacity(DOMAIN_FRAME_SIGN.len() + 2 + 32);
    m.extend_from_slice(DOMAIN_FRAME_SIGN);
    m.push(version);
    m.push(flags);
    m.extend_from_slice(&cid.0);
    m
}

// ══════════════════════════════════════════════════════════════════════════════
// TLV helpers (using atomic-codec primitives)
// ══════════════════════════════════════════════════════════════════════════════

/// Push a TLV: tag + varint length + value.
fn push_tlv(buf: &mut Vec<u8>, tag: u8, val: &[u8]) {
    buf.push(tag);
    encode_varint_u64(val.len() as u64, buf);
    buf.extend_from_slice(val);
}

/// Read TLV at position, returns (tag, value_start, value_end).
fn read_tlv(input: &[u8], pos: &mut usize) -> Result<(u8, usize, usize), SirpError> {
    if *pos >= input.len() {
        return Err(SirpError::Decode("eof".into()));
    }
    let tag = input[*pos];
    *pos += 1;
    let len = decode_varint_u64(input, pos)? as usize;
    let start = *pos;
    let end = start
        .checked_add(len)
        .ok_or_else(|| SirpError::Decode("overflow".into()))?;
    if end > input.len() {
        return Err(SirpError::Decode("truncated".into()));
    }
    *pos = end;
    Ok((tag, start, end))
}

// ══════════════════════════════════════════════════════════════════════════════
// Encode/Decode
// ══════════════════════════════════════════════════════════════════════════════

/// Encodes a SIRP frame to wire bytes.
///
/// Format: `[MAGIC:2][VERSION:1][FLAGS:1]` + TLVs:
/// - `T_CID32`: intent.cid
/// - `T_BYTES`: intent.bytes
/// - `T_BYTES`: extra (if non-empty)
/// - `T_PUBKEY32`, `T_SIG64` (if FLAG_SIGNED)
#[must_use]
pub fn encode_frame(f: &SirpFrame) -> Vec<u8> {
    let cap = 4 + 1 + 32 + 1 + 10 + f.intent.bytes.len() + f.extra.len() + 96;
    let mut out = Vec::with_capacity(cap);

    // Header
    out.extend_from_slice(&SIRP_MAGIC.to_be_bytes());
    out.push(f.version);
    out.push(f.flags);

    // Intent CID (fixed 32 bytes, no length prefix needed but we use TLV for uniformity)
    push_tlv(&mut out, T_CID32, &f.intent.cid.0);

    // Intent bytes
    push_tlv(&mut out, T_BYTES, &f.intent.bytes);

    // Extra (if any)
    if !f.extra.is_empty() {
        push_tlv(&mut out, T_BYTES, &f.extra);
    }

    // Signature fields
    if f.flags & FLAG_SIGNED == FLAG_SIGNED {
        if let Some(pk) = &f.pubkey {
            push_tlv(&mut out, T_PUBKEY32, &pk.0);
        }
        if let Some(sig) = &f.signature {
            push_tlv(&mut out, T_SIG64, &sig.0);
        }
    }

    out
}

/// Decodes a SIRP frame from wire bytes.
///
/// # Errors
///
/// Returns error if malformed, CID mismatch, or signature invalid.
pub fn decode_frame(input: &[u8]) -> Result<SirpFrame, SirpError> {
    if input.len() < 4 {
        return Err(SirpError::Header);
    }
    let magic = u16::from_be_bytes([input[0], input[1]]);
    let version = input[2];
    let flags = input[3];
    if magic != SIRP_MAGIC || version != SIRP_VERSION {
        return Err(SirpError::Header);
    }

    let tlv_data = &input[4..];
    let mut pos = 0usize;
    let mut cid: Option<Cid32> = None;
    let mut intent_bytes: Option<Vec<u8>> = None;
    let mut extra = Vec::new();
    let mut pubkey: Option<PublicKeyBytes> = None;
    let mut signature: Option<SignatureBytes> = None;

    while pos < tlv_data.len() {
        let (tag, start, end) = read_tlv(tlv_data, &mut pos)?;
        let val = &tlv_data[start..end];
        match tag {
            T_CID32 => {
                if val.len() != 32 {
                    return Err(SirpError::Decode("cid32 length".into()));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(val);
                cid = Some(Cid32(arr));
            }
            T_BYTES => {
                if intent_bytes.is_none() {
                    intent_bytes = Some(val.to_vec());
                } else {
                    // Second T_BYTES occurrence is extra
                    extra = val.to_vec();
                }
            }
            T_PUBKEY32 => {
                if val.len() != 32 {
                    return Err(SirpError::Decode("pubkey length".into()));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(val);
                pubkey = Some(PublicKeyBytes(arr));
            }
            T_SIG64 => {
                if val.len() != 64 {
                    return Err(SirpError::Decode("sig length".into()));
                }
                let mut arr = [0u8; 64];
                arr.copy_from_slice(val);
                signature = Some(SignatureBytes(arr));
            }
            _ => {
                // Forward-compatible: ignore unknown tags
            }
        }
    }

    let cid = cid.ok_or(SirpError::Missing("intent.cid"))?;
    let intent_bytes = intent_bytes.ok_or(SirpError::Missing("intent.bytes"))?;

    let frame = SirpFrame {
        version,
        flags,
        intent: CanonIntent {
            cid,
            bytes: intent_bytes,
        },
        extra,
        pubkey,
        signature,
    };

    // Validate CID and signature
    frame.verify()?;
    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn canon(v: serde_json::Value) -> CanonIntent {
        let bytes = json_atomic::canonize(&v).unwrap();
        let cid = ubl_crypto::blake3_cid(&bytes);
        CanonIntent { cid, bytes }
    }

    #[test]
    fn roundtrip_unsigned() {
        let a = json!({"intent":"Grant","to":"alice","amount":1});
        let b = serde_json::from_str::<serde_json::Value>(
            r#"{ "amount":1,"intent":"Grant","to":"alice" }"#,
        )
        .unwrap();
        let ca = canon(a);
        let cb = canon(b);
        // Same canonical form regardless of key order
        assert_eq!(ca.cid, cb.cid);
        assert_eq!(ca.bytes, cb.bytes);

        let f = SirpFrame::unsigned(ca);
        let enc = encode_frame(&f);
        let dec = decode_frame(&enc).unwrap();
        assert_eq!(f, dec);
    }

    #[cfg(feature = "signing")]
    #[test]
    fn roundtrip_signed() {
        use ubl_crypto::Keypair;

        let intent = canon(json!({"intent":"Freeze","id":"X"}));
        let kp = Keypair::generate();
        let f = SirpFrame::unsigned(intent).sign(&kp.sk);
        let enc = encode_frame(&f);
        let dec = decode_frame(&enc).unwrap();

        assert_eq!(f.version, dec.version);
        assert_eq!(f.flags, dec.flags);
        assert_eq!(f.intent, dec.intent);
        assert!(dec.verify().is_ok());
    }

    #[test]
    fn cid_mismatch_detected() {
        let intent = canon(json!({"test":"value"}));
        let mut f = SirpFrame::unsigned(intent);
        // Corrupt the CID
        f.intent.cid.0[0] ^= 0xFF;
        let enc = encode_frame(&f);
        let result = decode_frame(&enc);
        assert!(matches!(result, Err(SirpError::CidMismatch)));
    }
}
