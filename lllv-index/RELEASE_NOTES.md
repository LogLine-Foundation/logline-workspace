# lllv-index v0.1.0 — Verifiable Top-K + Merkle Evidence

## Highlights

- **IndexPack + QueryEvidence** com prova Merkle verificável.
- **Domain separation**: `leaf`, `node`.
- API `ProofStep { sibling, sibling_is_right }` + `verify_path() -> Result`.
- Verificador robusto (hex parsing estrito, erros estruturados).
- CI (fmt, clippy, test), audit, deny, **SBOM CycloneDX**.

## Security

- **Integridade:** `Merkle root` com *domain separation*: `"leaf"` e `"node"`.
- **Autenticidade opcional:** combine com `lllv-core` (cápsulas assinadas).
- **Hex robusto:** parsing defensivo em paths/evidências; erros estritos e descritivos.
- **Supply-chain:** CI com `cargo-audit`, `cargo-deny` e **SBOM (CycloneDX)**.

## Dependencies

- `lllv-core = "0.1.0"`
- `blake3 = "1.5"`
- `ed25519-dalek = "2.1"` (com feature `pkcs8`)
