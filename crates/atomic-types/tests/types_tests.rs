use atomic_types::{AppId, Cid32, Dim, Intent, PublicKeyBytes, SignatureBytes};
use core::str::FromStr;

#[test]
fn dim_parses_hex_and_decimal() {
    assert_eq!(Dim::parse("0x00A1").unwrap().as_u16(), 0x00A1);
    assert_eq!(Dim::parse("161").unwrap().as_u16(), 161);
    assert_eq!(Dim::parse(" 0x0001 ").unwrap().to_hex(), "0x0001");
}

#[test]
fn dim_rejects_bad_inputs() {
    assert_eq!(Dim::parse("0xGG"), Err("bad hex"));
    assert_eq!(Dim::parse(""), Err("bad dec"));
    assert_eq!(Dim::from_hex("xyz"), Err("bad hex"));
}

#[test]
fn newtype_ids_trim_and_display() {
    let id = AppId::from_str("  app-123  ").unwrap();
    assert_eq!(id.0, "app-123");
    assert_eq!(id.to_string(), "app-123");
}

// ══════════════════════════════════════════════════════════════════════════════
// Crypto wrapper tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn cid32_hex_roundtrip() {
    let c = Cid32([0xAB; 32]);
    let j = serde_json::to_string(&c).unwrap();
    // " + 64 hex chars + "
    assert_eq!(j.len(), 66);
    let de: Cid32 = serde_json::from_str(&j).unwrap();
    assert_eq!(c.0, de.0);
}

#[test]
fn cid32_from_hex() {
    let hex_str = "ab".repeat(32);
    let cid = Cid32::from_hex(&hex_str).unwrap();
    assert_eq!(cid.0, [0xAB; 32]);
    assert_eq!(cid.to_hex(), hex_str);
}

#[test]
fn cid32_rejects_wrong_length() {
    assert!(Cid32::from_hex("abcd").is_err());
}

#[test]
fn pk_sig_hex_roundtrip() {
    let pk = PublicKeyBytes([0x22; 32]);
    let sig = SignatureBytes([0x33; 64]);
    let jp = serde_json::to_string(&pk).unwrap();
    let js = serde_json::to_string(&sig).unwrap();
    let dpk: PublicKeyBytes = serde_json::from_str(&jp).unwrap();
    let ds: SignatureBytes = serde_json::from_str(&js).unwrap();
    assert_eq!(pk.0, dpk.0);
    assert_eq!(sig.0, ds.0);
}

#[test]
fn intent_ws_normalization() {
    let i1 = Intent::from_raw("  hello   world ");
    let i2 = Intent::from_raw("hello world");
    assert_eq!(i1.as_bytes(), i2.as_bytes());
    assert_eq!(i1.as_bytes(), b"hello world");
}

#[test]
fn intent_preserves_raw() {
    let i = Intent::from_raw("  original  text ");
    assert_eq!(i.raw, "  original  text ");
}
