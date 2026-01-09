# lllv-core v0.1.0 — Verifiable Capsules

## Highlights

- `Capsule::create(dim, bytes, flags, &sk)` com CID (BLAKE3) e assinatura Ed25519
- Verificações explícitas:
  - `verify_cid()` → integridade (CID)
  - `verify_with(pk)` → integridade + autenticidade (assinatura)
- `no_std/alloc` ready (design), MSRV 1.75, CI com audit/deny/SBOM
- Testes de tamper e AAD cobrindo cenários críticos

## Security

- Assinatura calculada sobre **CID dos bytes canônicos**; APIs separadas para evitar uso indevido.
- Supply-chain hardening: `cargo-audit`, `cargo-deny`, SBOM CycloneDX nos releases.

## Breaking Changes

- `verify()` está deprecated — use `verify_cid()` para integridade ou `verify_with(pk)` para autenticidade completa.

## Dependencies

- `ed25519-dalek = "2.1"` (com feature `pkcs8`)
- `blake3 = "1.5"`
- `chacha20poly1305 = "0.10"` (para criptografia opcional)
