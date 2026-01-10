# ubl-crypto

[![crates.io](https://img.shields.io/crates/v/ubl-crypto.svg)](https://crates.io/crates/ubl-crypto)
[![docs.rs](https://docs.rs/ubl-crypto/badge.svg)](https://docs.rs/ubl-crypto)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)

Crypto primitives for the **LogLine Workspace**.

## Features

### Hashing
- **`blake3_hex`** — BLAKE3 hash → hex string
- **`blake3_cid`** — BLAKE3 hash → `Cid32` (ubl-types)
- **`blake3_cid_chunks`** — Incremental BLAKE3 → `Cid32`

### Ed25519 Signatures
- **`SecretKey`**, **`Keypair`** — Key management with zeroize
- **`sign_cid_hex`**, **`verify_cid_hex`** — Sign/verify CID hex strings
- **`sign_bytes`**, **`verify_bytes`** — Sign/verify with `ubl-types` wrappers
- **`derive_public_bytes`** — Derive `PublicKeyBytes` from secret seed

### HMAC
- **`hmac_sign`**, **`hmac_verify`** — HMAC-SHA256 with base64url

### DID:key
- **`did_key_encode_ed25519`**, **`did_key_decode_ed25519`** — Ed25519 DID encoding

### Key IDs
- **`key_id_v1`**, **`key_id_v2`** — Key identifier formats

## Installation

```toml
[dependencies]
ubl-crypto = "0.3"
```

## Quick Example

```rust
use ubl_crypto::{blake3_cid, derive_public_bytes, sign_bytes, verify_bytes};

// BLAKE3 → Cid32
let cid = blake3_cid(b"hello");
println!("cid = {cid}");

// Ed25519 with fixed seed (deterministic)
let sk = [7u8; 32];
let pk = derive_public_bytes(&sk);
let sig = sign_bytes(b"message", &sk);
assert!(verify_bytes(b"message", &pk, &sig));
```

## Roadmap

- Key rotation (KID + grace period)
- AEAD for evidence encryption
- KMS adapters

## License

MIT OR Apache-2.0 © LogLine Foundation