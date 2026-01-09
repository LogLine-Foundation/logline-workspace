# Changelog
Todas as mudanças notáveis deste projeto serão documentadas aqui.

O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Planejado
- Assinatura Ed25519 opcional via `ed25519-dalek`
- Bytes canônicos com `json-atomic`
- Ghost forensics enriquecido (razões padronizadas, timestamps extras)
- Registry de verbos publicamente versionado

## [0.1.1] - 2026-01-08
### Melhorado
- **Documentação completa**: cobertura aumentada de ~28% para ~80%+
- Adicionados exemplos de código executáveis para todos os métodos públicos
- Documentação detalhada para todos os tipos, traits e enums
- Melhorias na documentação do `docs.rs` com exemplos práticos

## [0.1.0] - 2026-01-08
### Adicionado
- Átomo **9-field tuple** (`who, did, this, when, confirmed_by, if_ok, if_doubt, if_not, status`)
- **Lifecycle** determinístico: `DRAFT → PENDING → COMMITTED` ou `GHOST`
- **Invariants** obrigatórios: `if_ok`, `if_doubt`, `if_not`
- **GhostRecord** e `abandon()` / `abandon_signed()`
- **Assinatura obrigatória**: `sign()` e `commit(&Signer)`
- **VerbRegistry** e `freeze_with_registry()` para validar `did`
- `Payload::Json(serde_json::Value)` quando `feature = "serde"`
- Compatibilidade **no_std** (usa `alloc` sem `std`)
- Exemplos (`simple_commit`, `ghost_record`), benchmark (Criterion) e suíte de testes

### Mudado
- `commit()` passou a exigir `&dyn Signer` (não mais `Option`)

### Segurança
- Política de segurança documentada em `SECURITY.md` (report responsável)
