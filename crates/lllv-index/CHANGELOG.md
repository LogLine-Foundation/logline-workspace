# Changelog
Todas as mudanças notáveis deste projeto serão documentadas aqui.
Formato: [Keep a Changelog](https://keepachangelog.com/) — SemVer.

## [Unreleased]
- `alloc/no_std` mais amplo; compactação de evidências.
- Suporte a Top-K com múltiplos scorers (cosine/dot/L2).

## [0.1.0] - 2026-01-09
### Adicionado
- **IndexPack + QueryEvidence** com prova Merkle verificável.
- **Domain separation**: `leaf`, `node`.
- API `ProofStep { sibling, sibling_is_right }` + `verify_path() -> Result`.
- Verificador robusto (hex parsing estrito, erros estruturados).
- CI (fmt, clippy, test), audit, deny, **SBOM CycloneDX**.
