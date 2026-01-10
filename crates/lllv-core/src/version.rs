//! Versioning constants for the LLLV capsule binary format.
pub const CAP_MAGIC: u16 = 0x4C56; // "LV"
pub const CAP_VER: u8 = 1;
pub const HEADER_LEN: usize = 114; // bytes do header binário fixo (sem payload)
