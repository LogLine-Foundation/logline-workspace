<div align="center">

# 🔐 LogLine Workspace

**Verifiable, privacy-first intelligence — data and actions that prove themselves.**

[![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)](https://www.rust-lang.org/)
[![no_std](https://img.shields.io/badge/no__std-ready-success)](#)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](#license)
[![Tests](https://github.com/LogLine-Foundation/logline-workspace/actions/workflows/ci.yml/badge.svg)](https://github.com/LogLine-Foundation/logline-workspace/actions)

<sub>The complete Rust workspace for the LogLine Protocol & JSON✯Atomic ecosystem</sub>

---

[📖 Docs](https://docs.rs/logline-core) · [📦 Crates.io](https://crates.io/crates/logline-core) · [🌐 Website](https://logline.foundation)

</div>

---

## 🧬 The Two Atoms

> **Same Semantics = Same Bytes = Same Hash.**

LogLine is built on two cryptographic primitives that form the foundation for verifiable, auditable systems:

| Atom | Crate | Description |
|------|-------|-------------|
| **Conceptual** | [`logline-core`](https://crates.io/crates/logline-core) | 9-field tuple describing **who did what, when, with what consequences** |
| **Cryptographic** | [`json_atomic`](https://crates.io/crates/json_atomic) | Canonical JSON + BLAKE3 CID + Ed25519 sealing for **Signed Facts** |

Together: describe an action with `logline-core`, seal the fact with `json_atomic`. **Auditable. Verifiable. Immutable.**

---

## 📦 Crate Ecosystem (18 crates)

```
logline-workspace/
├── 🧠 Protocol Core
│   ├── logline-core      — The Conceptual Atom (Paper I)
│   ├── json_atomic       — The Cryptographic Atom (Paper II)  
│   ├── lllv-core         — LLLV Capsule format (Paper III)
│   └── lllv-index        — Capsule indexing & retrieval
│
├── 🔧 TDLN (Typed Declarative Logic Notation)
│   ├── tdln-ast          — Abstract Syntax Tree
│   ├── tdln-brain        — AI reasoning engine with LLM integration
│   ├── tdln-compiler     — TDLN → bytecode compilation
│   ├── tdln-gate         — Policy gates & validation
│   └── tdln-proof        — Proof generation & verification
│
├── ⚛️ Atomic Family
│   ├── atomic-types      — Shared IDs, time, error helpers
│   ├── atomic-crypto     — BLAKE3, Ed25519, HMAC, key management
│   ├── atomic-codec      — JSON✯Atomic canonical encode/decode
│   ├── atomic-sirp       — Network capsule + receipt flow (HTTP)
│   └── atomic-runtime    — DIM router/handlers with UBL logging
│
└── 🏢 UBL (Unified Business Ledger)
    ├── ubl-ledger        — NDJSON writer with rotation & signing
    ├── ubl-mcp           — Model Context Protocol server
    └── ubl-office        — Business automation with AI agents
```

---

## 🚀 Quick Start

```toml
# Cargo.toml
[dependencies]
logline-core = "0.1"
json_atomic  = "0.1"
```

```rust
use json_atomic::{seal_value, verify_seal};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde::Serialize;

#[derive(Serialize)]
struct Action { actor: String, verb: String }

fn main() {
    let key = SigningKey::generate(&mut OsRng);
    
    // Seal any serializable value → Signed Fact
    let fact = seal_value(&Action { 
        actor: "alice".into(), 
        verb: "approved".into() 
    }, &key).unwrap();
    
    // Anyone can verify
    verify_seal(&fact).unwrap();
    
    println!("CID: {}", fact.cid_hex());  // BLAKE3 of canonical bytes
    println!("Sig: {}", fact.signature_hex());
}
```

---

## 🔒 Security Model

- **Canonical Serialization** — Same semantics = same bytes = same hash
- **BLAKE3 CIDs** — Fast, secure content addressing
- **Ed25519 Signatures** — Curve25519 EdDSA on CID, not raw JSON
- **no_std Support** — Runs in constrained environments
- **Lifecycle Invariants** — `DRAFT → PENDING → COMMITTED | GHOST`

---

## 🎯 Use Cases

| Domain | How LogLine Helps |
|--------|-------------------|
| **Audit Trails** | Signed action logs with ex-ante consequences |
| **Immutable Documents** | Every doc/message becomes a Signed Fact |
| **Computable Contracts** | Policies that explain and prove themselves |
| **AI/Data Provenance** | End-to-end verifiable content chains |
| **Supply Chain** | Tamper-evident records with cryptographic proofs |

---

## 🛠️ Development

```bash
# Clone
git clone https://github.com/LogLine-Foundation/logline-workspace
cd logline-workspace

# Build & test
cargo build --all-features
cargo test --all-features

# Quality checks
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Run benchmarks
cargo bench -p json_atomic
```

### MSRV Policy
Minimum Supported Rust Version: **1.75+**

---

## 📚 Papers & Documentation

| Paper | Topic | Crate |
|-------|-------|-------|
| **Paper I** | The Conceptual Atom — Verifiable Actions | `logline-core` |
| **Paper II** | JSON✯Atomic — Cryptographic Sealing | `json_atomic` |
| **Paper III** | LLLV — The Retrieval Atom | `lllv-core` |

Full papers available in [`docs/papers/`](docs/papers/).

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

- 🐛 **Bug Reports**: Open an issue with reproduction steps
- 💡 **Feature Ideas**: Discuss in issues first
- 🔧 **Pull Requests**: Fork → branch → PR → review

---

## 📄 License

Dual-licensed under your choice of:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

---

## 🔐 Security

Report vulnerabilities to: **[security@logline.foundation](mailto:security@logline.foundation)**

See [SECURITY.md](SECURITY.md) for our security policy.

---

<div align="center">

**LogLine Foundation** — *data and actions that prove themselves* ✨

[Website](https://logline.foundation) · [Crates.io](https://crates.io/crates/logline-core) · [Docs](https://docs.rs/logline-core)

</div>
