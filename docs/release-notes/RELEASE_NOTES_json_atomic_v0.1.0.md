# json_atomic v0.1.0 — The Cryptographic Atom (Paper II)

## What's new

* Canonicalização JSON✯Atomic: **Same Semantics = Same Bytes = Same Hash**
* Cycle of Truth: `canonize(value) → blake3(CID) → ed25519 (DV25-Seal)`
* `SignedFact { canonical, cid, signature, public_key, hash_alg, sig_alg, canon_ver, format_id }`
* Integração com **logline-core 0.1.0**: `seal_logline(LogLine)`
* Trajectory Matching básico (`trajectory_confidence`)
* `alloc/no_std` pronto (matriz no CI)
* Testes de canto (NFC, zeros à esquerda, ordem de chaves)
* README com conformidade, quickstart e API

## Docs & crate

* **crates.io**: `json_atomic = "0.1.0"`
* **docs.rs**: https://docs.rs/json_atomic/0.1.0 (com badges e README incorporado)

## Security

* Assinatura **Ed25519** calculada **sobre o CID** (BLAKE3 dos **bytes canônicos**).

## Links

* 📦 [crates.io](https://crates.io/crates/json_atomic)
* 📚 [docs.rs](https://docs.rs/json_atomic)
* 🔗 [Projeto irmão: logline-core](https://github.com/logline-foundation/logline-core)
* 📖 [Paper II: JSON✯Atomic](https://github.com/logline-foundation/json-atomic/blob/main/docs/paper-ii-json-atomic.md)
