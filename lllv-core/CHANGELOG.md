# Changelog

Todas as mudanças notáveis deste projeto serão documentadas aqui.

O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Planejado
- Suporte completo `alloc/no_std` (sem `std`)
- Merkle chunking para cápsulas grandes
- Trajectory matching integrado
- Indexação otimizada (lllv-index)

## [0.1.0] - 2026-01-09
### Adicionado
- Vector Capsule: header binário assinado (BLAKE3 CID + Ed25519)
- Payload cifrável (ChaCha20-Poly1305) com AAD
- Manifesto JSON✯Atomic (Paper II) — selagem DV25 (feature `manifest`)
- Testes básicos + CI
