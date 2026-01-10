# Changelog
Todas as mudanças notáveis deste projeto serão documentadas aqui.

O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Planejado
- Suporte completo **alloc/no_std** (sem `std`) — builds determinísticos em ambientes restritos
- Vetores determinísticos sem `std` para canonicalização
- Test vectors ampliados (Paper II §3.3)
- Cabeçalhos Canônicos (content-type + version + schema-hash)
- Carimbo temporal canônico
- Merkle chunking para documentos grandes
- HNSW opcional para Trajectory Matching

## [0.1.1] - 2026-01-08
### Melhorado
- **Documentação completa**: cobertura aumentada significativamente
- Adicionados exemplos de código executáveis para todas as funções públicas
- Documentação detalhada para `SignedFact`, `seal_value`, `verify_seal`, `canonize`
- Documentação completa para tipos de erro (`CanonicalError`, `SealError`, `VerifyError`)
- Melhorias na documentação do `docs.rs` com exemplos práticos

## [0.1.0] - 2026-01-08
### Adicionado
- Canonicalização JSON✯Atomic (Same Semantics = Same Bytes = Same Hash)
- Cycle of Truth: `canonize → blake3(CID) → ed25519 (DV25-Seal)`
- `SignedFact { canonical, cid, signature, public_key, algs }`
- Integração com **logline-core 0.1.0**: `seal_logline(LogLine)`
- Trajectory Matching (cosine→[0,1]) básico
- CI (fmt, clippy, test), exemplos e benches

### Segurança
- Assinatura `Ed25519` sobre o **CID** (BLAKE3 dos bytes canônicos)
- Verificação estrita de assinaturas (`verify_strict`)
- Rejeição de floats na canonicalização (apenas inteiros permitidos)
