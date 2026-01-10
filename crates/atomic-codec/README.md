# ubl-codec

[![crates.io](https://img.shields.io/crates/v/ubl-codec.svg)](https://crates.io/crates/ubl-codec)
[![docs.rs](https://docs.rs/ubl-codec/badge.svg)](https://docs.rs/ubl-codec)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)

Canonical encoding for the **LogLine Workspace** — two complementary codecs:

## JSON✯Atomic (canonical JSON)

- `to_canon_vec` / `from_canon_slice` — Serialize/deserialize with key sorting
- `to_cid_hex` — BLAKE3 CID of canonical bytes
- `Canonical<T>` — Value + precomputed canonical bytes
- `is_canonical` — Check if JSON string is already canonical
- `yaml_to_canon_vec` — YAML → canonical JSON

## Binary TLV (compact wire format)

- **Varint (u64)**: Base-128 with MSB continuation bit
- **Tags**: `CID32`, `PUBKEY32`, `SIG64`, `BYTES`, `STR`, `U64`
- **Frames**: `typ (u8) + len (varint) + payload`

```rust
use ubl_codec::{Encoder, Decoder, encode_frame, decode_frame};
use ubl_crypto::blake3_cid;

let cid = blake3_cid(b"hello");
let mut enc = Encoder::new();
enc.cid32(&cid);
enc.str("hi");
let payload = enc.finish();

let frame = encode_frame(0x42, &payload);
let (typ, body) = decode_frame(&frame).unwrap();

let mut dec = Decoder::new(body);
let cid2 = dec.cid32().unwrap();
let msg = dec.str().unwrap();
```

## Installation

```toml
[dependencies]
ubl-codec = "0.3"
```

## Segurança & Limites

Proteções DoS integradas:

| Limite | Valor | Erro |
|--------|-------|------|
| `MAX_FRAME_LEN` | 1 MiB | `BinaryCodecError::SizeLimit` |
| `MAX_VARINT_BYTES` | 10 | `BinaryCodecError::VarintOverflow` |

- **Fuzzing**: cargo-fuzz com >860k execuções sem crashes
- **Testes adversariais**: 11 testes cobrindo frames oversized, varints malformados, truncação
- **CI**: hardening.yml com property tests, fuzz bursts e verificação de limites

```bash
cargo test --test tlv_adversarial
```

## License

MIT OR Apache-2.0 © LogLine Foundation