//! Binary TLV codec (varint + frames) for LogLine Workspace.
//!
//! Provides:
//! - **Varint (u64)**: base-128 encoding with MSB as continuation bit
//! - **TLV**: Tag (u8) + optional Length (varint) + Value (bytes)
//! - **Frames**: `typ (u8) + len (varint) + payload`
//!
//! Fixed-size types (`CID32`, `PUBKEY32`, `SIG64`) don't carry length (known size).
//!
//! ## Security Limits
//!
//! - [`MAX_FRAME_LEN`]: Hard limit on frame payload (~1 MiB) to prevent DoS
//! - [`MAX_VARINT_BYTES`]: Maximum bytes for varint encoding (10) to prevent overflow

use atomic_types::{Cid32, PublicKeyBytes, SignatureBytes};
use thiserror::Error;

// ══════════════════════════════════════════════════════════════════════════════
// Security Limits (DoS guards)
// ══════════════════════════════════════════════════════════════════════════════

/// Hard limit for frame payload size (~1 MiB). Prevents memory exhaustion DoS.
pub const MAX_FRAME_LEN: usize = 1 << 20;

/// Maximum bytes for a varint-encoded u64 (ceil(64/7) = 10).
pub const MAX_VARINT_BYTES: usize = 10;

/// Maximum bytes for variable-length TLV fields (same as frame limit).
pub const MAX_BYTES_LEN: usize = MAX_FRAME_LEN;

// ══════════════════════════════════════════════════════════════════════════════
// Canonical Tags (0x00–0x3F: primitives; 0x40–0x7F: reserved; 0x80+ vendor)
// ══════════════════════════════════════════════════════════════════════════════

/// Tag for variable-length bytes.
pub const T_BYTES: u8 = 0x01;
/// Tag for UTF-8 string.
pub const T_STR: u8 = 0x02;
/// Tag for varint-encoded u64.
pub const T_U64: u8 = 0x03;
/// Tag for 32-byte CID (BLAKE3).
pub const T_CID32: u8 = 0x10;
/// Tag for 32-byte Ed25519 public key.
pub const T_PUBKEY32: u8 = 0x11;
/// Tag for 64-byte Ed25519 signature.
pub const T_SIG64: u8 = 0x12;

// ══════════════════════════════════════════════════════════════════════════════
// Errors
// ══════════════════════════════════════════════════════════════════════════════

/// Errors from binary codec operations.
#[derive(Debug, Error)]
pub enum BinaryCodecError {
    /// Unexpected end of input.
    #[error("unexpected EOF")]
    Eof,
    /// Malformed varint encoding.
    #[error("malformed varint")]
    Varint,
    /// Varint exceeds maximum allowed bytes.
    #[error("varint overflow: exceeded {MAX_VARINT_BYTES} bytes")]
    VarintOverflow,
    /// Declared size exceeds maximum allowed.
    #[error("size limit exceeded: {declared} > {MAX_FRAME_LEN}")]
    SizeLimit {
        /// Declared size that exceeded the limit.
        declared: usize,
    },
    /// Invalid length for fixed-size field.
    #[error("invalid length")]
    Length,
    /// Unexpected tag encountered.
    #[error("unexpected tag: got {got:#04x}, expected {expected:#04x}")]
    Tag {
        /// Tag that was found.
        got: u8,
        /// Tag that was expected.
        expected: u8,
    },
    /// Invalid UTF-8 in string.
    #[error("invalid UTF-8")]
    Utf8,
}

// ══════════════════════════════════════════════════════════════════════════════
// Varint encoding/decoding
// ══════════════════════════════════════════════════════════════════════════════

/// Encodes a `u64` as a base-128 varint, appending to `out`.
#[inline]
pub fn encode_varint_u64(mut x: u64, out: &mut Vec<u8>) {
    while x >= 0x80 {
        out.push(((x as u8) & 0x7F) | 0x80);
        x >>= 7;
    }
    out.push(x as u8);
}

/// Decodes a base-128 varint from `input` starting at `pos`, advancing `pos`.
///
/// # Errors
///
/// - `BinaryCodecError::Eof` if input ends prematurely
/// - `BinaryCodecError::VarintOverflow` if exceeds [`MAX_VARINT_BYTES`]
/// - `BinaryCodecError::Varint` if encoding is malformed (shift > 63)
#[inline]
pub fn decode_varint_u64(input: &[u8], pos: &mut usize) -> Result<u64, BinaryCodecError> {
    let mut shift = 0u32;
    let mut result: u64 = 0;
    let mut bytes_read = 0usize;
    loop {
        // Guard: EOF check
        if *pos >= input.len() {
            return Err(BinaryCodecError::Eof);
        }
        // Guard: varint byte limit
        bytes_read += 1;
        if bytes_read > MAX_VARINT_BYTES {
            return Err(BinaryCodecError::VarintOverflow);
        }
        
        let b = input[*pos];
        *pos += 1;
        let val = (b & 0x7F) as u64;
        result |= val << shift;
        if (b & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift > 63 {
            return Err(BinaryCodecError::Varint);
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Encoder
// ══════════════════════════════════════════════════════════════════════════════

/// Binary TLV encoder.
#[derive(Default)]
pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    /// Creates a new empty encoder.
    #[must_use]
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Creates an encoder with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buf: Vec::with_capacity(cap),
        }
    }

    /// Returns the encoded bytes, consuming the encoder.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.buf
    }

    /// Returns the encoded bytes as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    /// Clears the buffer for reuse.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Returns current length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns true if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    #[inline]
    fn tag(&mut self, t: u8) {
        self.buf.push(t);
    }

    #[inline]
    fn write_len(&mut self, n: usize) {
        encode_varint_u64(n as u64, &mut self.buf);
    }

    /// Encodes a `u64` as tagged varint.
    pub fn u64(&mut self, v: u64) {
        self.tag(T_U64);
        encode_varint_u64(v, &mut self.buf);
    }

    /// Encodes raw bytes with length prefix.
    pub fn bytes(&mut self, b: &[u8]) {
        self.tag(T_BYTES);
        self.write_len(b.len());
        self.buf.extend_from_slice(b);
    }

    /// Encodes a UTF-8 string with length prefix.
    pub fn str(&mut self, s: &str) {
        self.tag(T_STR);
        self.write_len(s.len());
        self.buf.extend_from_slice(s.as_bytes());
    }

    /// Encodes a 32-byte CID (no length prefix, fixed size).
    pub fn cid32(&mut self, cid: &Cid32) {
        self.tag(T_CID32);
        self.buf.extend_from_slice(&cid.0);
    }

    /// Encodes a 32-byte public key (no length prefix, fixed size).
    pub fn public_key(&mut self, pk: &PublicKeyBytes) {
        self.tag(T_PUBKEY32);
        self.buf.extend_from_slice(&pk.0);
    }

    /// Encodes a 64-byte signature (no length prefix, fixed size).
    pub fn signature(&mut self, sig: &SignatureBytes) {
        self.tag(T_SIG64);
        self.buf.extend_from_slice(&sig.0);
    }
}

impl core::fmt::Debug for Encoder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Encoder(len={})", self.buf.len())
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Decoder
// ══════════════════════════════════════════════════════════════════════════════

/// Binary TLV decoder.
pub struct Decoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    /// Creates a new decoder from a byte slice.
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Returns the current position.
    #[must_use]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns true if all data has been consumed.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// Returns remaining bytes.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    #[inline]
    fn need(&self, n: usize) -> Result<(), BinaryCodecError> {
        if self.pos + n <= self.data.len() {
            Ok(())
        } else {
            Err(BinaryCodecError::Eof)
        }
    }

    #[inline]
    fn take(&mut self, n: usize) -> Result<&'a [u8], BinaryCodecError> {
        self.need(n)?;
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    #[inline]
    fn read_tag(&mut self, expected: u8) -> Result<(), BinaryCodecError> {
        let got = *self.data.get(self.pos).ok_or(BinaryCodecError::Eof)?;
        self.pos += 1;
        if got == expected {
            Ok(())
        } else {
            Err(BinaryCodecError::Tag { got, expected })
        }
    }

    /// Decodes a tagged `u64`.
    ///
    /// # Errors
    ///
    /// Returns error if tag mismatch or malformed varint.
    pub fn u64(&mut self) -> Result<u64, BinaryCodecError> {
        self.read_tag(T_U64)?;
        decode_varint_u64(self.data, &mut self.pos)
    }

    /// Decodes tagged bytes.
    ///
    /// # Errors
    ///
    /// - `BinaryCodecError::Tag` if tag mismatch
    /// - `BinaryCodecError::SizeLimit` if length exceeds [`MAX_BYTES_LEN`]
    /// - `BinaryCodecError::Eof` if insufficient data
    pub fn bytes(&mut self) -> Result<&'a [u8], BinaryCodecError> {
        self.read_tag(T_BYTES)?;
        let len = decode_varint_u64(self.data, &mut self.pos)? as usize;
        if len > MAX_BYTES_LEN {
            return Err(BinaryCodecError::SizeLimit { declared: len });
        }
        self.take(len)
    }

    /// Decodes a tagged UTF-8 string.
    ///
    /// # Errors
    ///
    /// - `BinaryCodecError::Tag` if tag mismatch
    /// - `BinaryCodecError::SizeLimit` if length exceeds [`MAX_BYTES_LEN`]
    /// - `BinaryCodecError::Eof` if insufficient data
    /// - `BinaryCodecError::Utf8` if invalid UTF-8
    pub fn str(&mut self) -> Result<&'a str, BinaryCodecError> {
        self.read_tag(T_STR)?;
        let len = decode_varint_u64(self.data, &mut self.pos)? as usize;
        if len > MAX_BYTES_LEN {
            return Err(BinaryCodecError::SizeLimit { declared: len });
        }
        let b = self.take(len)?;
        core::str::from_utf8(b).map_err(|_| BinaryCodecError::Utf8)
    }

    /// Decodes a 32-byte CID.
    ///
    /// # Errors
    ///
    /// Returns error if tag mismatch or insufficient data.
    pub fn cid32(&mut self) -> Result<Cid32, BinaryCodecError> {
        self.read_tag(T_CID32)?;
        let b = self.take(32)?;
        let mut out = [0u8; 32];
        out.copy_from_slice(b);
        Ok(Cid32(out))
    }

    /// Decodes a 32-byte public key.
    ///
    /// # Errors
    ///
    /// Returns error if tag mismatch or insufficient data.
    pub fn public_key(&mut self) -> Result<PublicKeyBytes, BinaryCodecError> {
        self.read_tag(T_PUBKEY32)?;
        let b = self.take(32)?;
        let mut out = [0u8; 32];
        out.copy_from_slice(b);
        Ok(PublicKeyBytes(out))
    }

    /// Decodes a 64-byte signature.
    ///
    /// # Errors
    ///
    /// Returns error if tag mismatch or insufficient data.
    pub fn signature(&mut self) -> Result<SignatureBytes, BinaryCodecError> {
        self.read_tag(T_SIG64)?;
        let b = self.take(64)?;
        let mut out = [0u8; 64];
        out.copy_from_slice(b);
        Ok(SignatureBytes(out))
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Frame encoding/decoding
// ══════════════════════════════════════════════════════════════════════════════

/// Encodes a frame: `typ (u8) + len (varint) + payload`.
#[must_use]
pub fn encode_frame(typ: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + 10 + payload.len());
    out.push(typ);
    encode_varint_u64(payload.len() as u64, &mut out);
    out.extend_from_slice(payload);
    out
}

/// Decodes a frame, returning `(typ, payload)`.
///
/// # Errors
///
/// - `BinaryCodecError::Eof` if input is empty or truncated
/// - `BinaryCodecError::SizeLimit` if declared length exceeds [`MAX_FRAME_LEN`]
pub fn decode_frame(buf: &[u8]) -> Result<(u8, &[u8]), BinaryCodecError> {
    if buf.is_empty() {
        return Err(BinaryCodecError::Eof);
    }
    let typ = buf[0];
    let mut pos = 1usize;
    let len = decode_varint_u64(buf, &mut pos)? as usize;
    
    // Security: reject frames exceeding size limit
    if len > MAX_FRAME_LEN {
        return Err(BinaryCodecError::SizeLimit { declared: len });
    }
    
    if pos + len > buf.len() {
        return Err(BinaryCodecError::Eof);
    }
    Ok((typ, &buf[pos..pos + len]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_roundtrip() {
        let vals = [
            0u64,
            1,
            127,
            128,
            255,
            256,
            16_384,
            u32::MAX as u64,
            u64::MAX,
        ];
        for &v in &vals {
            let mut b = Vec::new();
            encode_varint_u64(v, &mut b);
            let mut p = 0usize;
            let got = decode_varint_u64(&b, &mut p).unwrap();
            assert_eq!(got, v);
            assert_eq!(p, b.len());
        }
    }

    #[test]
    fn frame_roundtrip() {
        let payload = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];
        let f = encode_frame(0x42, &payload);
        let (t, p) = decode_frame(&f).unwrap();
        assert_eq!(t, 0x42);
        assert_eq!(p, &payload);
    }

    #[test]
    fn encoder_decoder_roundtrip() {
        let cid = Cid32([0xAB; 32]);
        let pk = PublicKeyBytes([0x22; 32]);
        let sig = SignatureBytes([0x33; 64]);

        let mut enc = Encoder::new();
        enc.cid32(&cid);
        enc.public_key(&pk);
        enc.signature(&sig);
        enc.str("hello");
        enc.u64(42);
        enc.bytes(b"raw");
        let buf = enc.finish();

        let mut dec = Decoder::new(&buf);
        let cid2 = dec.cid32().unwrap();
        let pk2 = dec.public_key().unwrap();
        let sig2 = dec.signature().unwrap();
        let s = dec.str().unwrap();
        let n = dec.u64().unwrap();
        let raw = dec.bytes().unwrap();

        assert!(dec.is_done());
        assert_eq!(cid2.0, cid.0);
        assert_eq!(pk2.0, pk.0);
        assert_eq!(sig2.0, sig.0);
        assert_eq!(s, "hello");
        assert_eq!(n, 42);
        assert_eq!(raw, b"raw");
    }
}
