#[allow(dead_code)]
pub fn blake3_bytes(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

pub fn hex32(h: &[u8; 32]) -> String {
    hex::encode(h)
}
