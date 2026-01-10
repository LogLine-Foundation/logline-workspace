//! Quick-start example for atomic-codec binary TLV.

use atomic_codec::{decode_frame, encode_frame, Decoder, Encoder};
use atomic_crypto::blake3_cid;

fn main() {
    // Create a CID from data
    let cid = blake3_cid(b"quickstart example");

    // Encode using TLV
    let mut enc = Encoder::new();
    enc.cid32(&cid);
    enc.str("hello");
    enc.u64(42);
    let payload = enc.finish();

    // Wrap in a frame
    let frame = encode_frame(0xAA, &payload);
    println!("frame len = {} bytes", frame.len());

    // Decode frame
    let (typ, body) = decode_frame(&frame).unwrap();
    assert_eq!(typ, 0xAA);

    // Decode TLV fields
    let mut dec = Decoder::new(body);
    let cid2 = dec.cid32().unwrap();
    let msg = dec.str().unwrap();
    let num = dec.u64().unwrap();

    println!("typ=0x{typ:02X} cid={cid2} msg={msg} num={num}");
    assert!(dec.is_done());
}
