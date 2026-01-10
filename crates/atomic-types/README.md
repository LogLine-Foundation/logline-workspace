# ubl-types

[![crates.io](https://img.shields.io/crates/v/ubl-types.svg)](https://crates.io/crates/ubl-types)
[![docs.rs](https://docs.rs/ubl-types/badge.svg)](https://docs.rs/ubl-types)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)

Shared canonical types for **Universal Business Ledger (UBL)**.

> **Note**: UBL = Universal Business Ledger (not OASIS Universal Business Language).

## Features

### Identity Types
- **`AppId`**, **`TenantId`**, **`NodeId`**, **`ActorId`**, **`TraceId`** — String-based newtypes with `Display`/`FromStr`
- **`Dim`** — Protocol dimension (u16) with hex/decimal parsing

### Cryptographic Primitives
- **`Cid32`** — 32-byte content ID (BLAKE3) with hex serialization
- **`PublicKeyBytes`** — 32-byte Ed25519 public key
- **`SignatureBytes`** — 64-byte Ed25519 signature
- **`Intent`** — Textual intent with canonical bytes (whitespace-insensitive)

### Error Types
- **`AtomError`** — Shared basic errors

## Installation

```toml
[dependencies]
ubl-types = "0.1"

# Optional features
ubl-types = { version = "0.1", features = ["ulid", "strict"] }
```

## Quick Example

```rust
use ubl_types::{Cid32, Dim, Intent, AppId};
use core::str::FromStr;

// Dimension parsing
let dim = Dim::parse("0x00A1").unwrap();
assert_eq!(dim.as_u16(), 161);

// CID with hex serialization
let cid = Cid32([0xAB; 32]);
println!("cid = {cid}"); // lowercase hex

// Intent normalization (whitespace-insensitive)
let i1 = Intent::from_raw("  hello   world ");
let i2 = Intent::from_raw("hello world");
assert_eq!(i1.as_bytes(), i2.as_bytes());

// Identity newtypes
let app = AppId::from_str("my-app").unwrap();
println!("app = {app}");
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `std` (default) | Standard library support |
| `ulid` | ULID generators for `TraceId`/`ActorId` |
| `strict` | Regex validation for ID newtypes |

## License

MIT OR Apache-2.0 © LogLine Foundation