use ubl_codec::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Sample {
    b: u32,
    a: String,
}

#[test]
fn canonical_bytes_roundtrip() {
    let value = Sample {
        b: 7,
        a: "hi".to_string(),
    };
    let bytes = to_canon_vec(&value).expect("canon bytes");
    // Canonical JSON✯Atomic should be sorted by keys.
    assert_eq!(bytes, br#"{"a":"hi","b":7}"#.to_vec());

    let decoded: Sample = from_canon_slice(&bytes).expect("decode");
    assert_eq!(decoded, value);
}

#[test]
fn cid_hex_matches_blake3_hash() {
    let value = json!({"z":1,"x":2});
    let canon = to_canon_vec(&value).expect("canon");
    let cid = to_cid_hex(&value).expect("cid");
    let expected = blake3::hash(&canon).to_hex().to_string();
    assert_eq!(cid, expected);
}

#[test]
fn canonical_reader_and_json_string() {
    let json_text = b"{\n  \"b\": 2,\n  \"a\": 1\n}\n";
    let canonical =
        Canonical::<serde_json::Value>::from_reader(&json_text[..]).expect("from reader");
    assert_eq!(canonical.as_bytes(), br#"{"a":1,"b":2}"#);
    assert!(is_canonical(r#"{"a":1,"b":2}"#));
    assert!(!is_canonical("{ \"b\":2, \"a\":1 }"));
    assert!(!is_canonical("not json"));

    let canon_from_str = from_json_str_canon("{\"b\":2,\"a\":1}").expect("canon from str");
    assert_eq!(canon_from_str, canonical.as_bytes());
}

#[test]
fn yaml_is_converted_to_canonical() {
    let yaml = "a: 1\nb: 2\n";
    let canon = yaml_to_canon_vec(yaml).expect("yaml to canon");
    assert_eq!(canon, br#"{"a":1,"b":2}"#.to_vec());
}

#[test]
fn errors_surface_cleanly() {
    let err = from_json_str_canon("not-json").unwrap_err();
    matches!(err, AtomicCodecError::Serde(_));

    let err = yaml_to_canon_vec("a: [unterminated").unwrap_err();
    matches!(err, AtomicCodecError::Yaml(_));
}

// ══════════════════════════════════════════════════════════════════════════════
// Binary TLV codec tests
// ══════════════════════════════════════════════════════════════════════════════

use ubl_codec::binary::{
    decode_frame, decode_varint_u64, encode_frame, encode_varint_u64, Decoder, Encoder,
};
use ubl_types::{Cid32, PublicKeyBytes, SignatureBytes};

#[test]
fn binary_varint_roundtrip() {
    let vals = [0u64, 1, 127, 128, 255, 256, 16_384, u32::MAX as u64, u64::MAX];
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
fn binary_frame_roundtrip() {
    let payload = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];
    let f = encode_frame(0x42, &payload);
    let (t, p) = decode_frame(&f).unwrap();
    assert_eq!(t, 0x42);
    assert_eq!(p, &payload);
}

#[test]
fn binary_tlv_full_roundtrip() {
    let cid = Cid32([0xAB; 32]);
    let pk = PublicKeyBytes([0x22; 32]);
    let sig = SignatureBytes([0x33; 64]);

    let mut enc = Encoder::new();
    enc.cid32(&cid);
    enc.public_key(&pk);
    enc.signature(&sig);
    enc.str("hello");
    enc.u64(999);
    enc.bytes(b"raw data");
    let buf = enc.finish();

    let mut dec = Decoder::new(&buf);
    assert_eq!(dec.cid32().unwrap().0, cid.0);
    assert_eq!(dec.public_key().unwrap().0, pk.0);
    assert_eq!(dec.signature().unwrap().0, sig.0);
    assert_eq!(dec.str().unwrap(), "hello");
    assert_eq!(dec.u64().unwrap(), 999);
    assert_eq!(dec.bytes().unwrap(), b"raw data");
    assert!(dec.is_done());
}
