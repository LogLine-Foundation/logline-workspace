//! Capsule header layout, serialization, and parsing utilities.
use crate::{
    errors::LllvError,
    version::{CAP_MAGIC, CAP_VER, HEADER_LEN},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CapsuleHeader {
    pub magic: u16,    // 2
    pub ver: u8,       // 1
    pub flags: u8,     // 1
    pub ts_ms: u64,    // 8
    pub cid: [u8; 32], // 32 (blake3(payload))
    pub dim: u16,      // 2
    pub len: u32,      // 4 (payload bytes)
    pub sig: [u8; 64], // 64 (ed25519(header_wo_sig||payload))
}

bitflags::bitflags! {
    #[derive(Default, Copy, Clone)]
    pub struct CapsuleFlags: u8 {
        const NONE       = 0b0000_0000;
        const ENCRYPTED  = 0b0000_0001; // payload = nonce(12) || ciphertext
        const RECEIPTREQ = 0b0000_0010; // reserva
    }
}

impl CapsuleHeader {
    #[must_use]
    pub const fn empty(dim: u16, flags: CapsuleFlags, cid: [u8; 32], len: u32, ts_ms: u64) -> Self {
        Self {
            magic: CAP_MAGIC,
            ver: CAP_VER,
            flags: flags.bits(),
            ts_ms,
            cid,
            dim,
            len,
            sig: [0u8; 64],
        }
    }

    #[must_use]
    pub fn to_bytes_wo_sig(&self) -> [u8; HEADER_LEN - 64] {
        let mut out = [0u8; HEADER_LEN - 64];
        let mut i = 0usize;
        out[i..=i + 1].copy_from_slice(&self.magic.to_le_bytes());
        i += 2;
        out[i..=i].copy_from_slice(&[self.ver]);
        i += 1;
        out[i..=i].copy_from_slice(&[self.flags]);
        i += 1;
        out[i..=i + 7].copy_from_slice(&self.ts_ms.to_le_bytes());
        i += 8;
        out[i..=i + 31].copy_from_slice(&self.cid);
        i += 32;
        out[i..=i + 1].copy_from_slice(&self.dim.to_le_bytes());
        i += 2;
        out[i..=i + 3].copy_from_slice(&self.len.to_le_bytes());
        i += 4;
        debug_assert_eq!(i, HEADER_LEN - 64);
        out
    }

    #[must_use]
    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut out = [0u8; HEADER_LEN];
        let (head, tail) = out.split_at_mut(HEADER_LEN - 64);
        head.copy_from_slice(&self.to_bytes_wo_sig());
        tail.copy_from_slice(&self.sig);
        out
    }

    /// Parse header from raw bytes.
    ///
    /// # Errors
    ///
    /// - `LllvError::InvalidHeaderLen` se o buffer for menor que `HEADER_LEN`
    /// - `LllvError::InvalidMagic` ou `InvalidVersion` se os campos não baterem
    pub fn from_bytes(raw: &[u8]) -> Result<Self, LllvError> {
        if raw.len() < HEADER_LEN {
            return Err(LllvError::InvalidHeaderLen);
        }
        let mut i = 0usize;
        let magic = u16::from_le_bytes(
            raw[i..i + 2]
                .try_into()
                .map_err(|_| LllvError::InvalidHeaderLen)?,
        );
        i += 2;
        let ver = raw[i];
        i += 1;
        let flags = raw[i];
        i += 1;
        let ts_ms = u64::from_le_bytes(
            raw[i..i + 8]
                .try_into()
                .map_err(|_| LllvError::InvalidHeaderLen)?,
        );
        i += 8;
        let mut cid = [0u8; 32];
        cid.copy_from_slice(&raw[i..i + 32]);
        i += 32;
        let dim = u16::from_le_bytes(
            raw[i..i + 2]
                .try_into()
                .map_err(|_| LllvError::InvalidHeaderLen)?,
        );
        i += 2;
        let len = u32::from_le_bytes(
            raw[i..i + 4]
                .try_into()
                .map_err(|_| LllvError::InvalidHeaderLen)?,
        );
        i += 4;
        let mut sig = [0u8; 64];
        sig.copy_from_slice(&raw[i..i + 64]);

        if magic != CAP_MAGIC {
            return Err(LllvError::InvalidMagic);
        }
        if ver != CAP_VER {
            return Err(LllvError::InvalidVersion);
        }

        Ok(Self {
            magic,
            ver,
            flags,
            ts_ms,
            cid,
            dim,
            len,
            sig,
        })
    }
}
