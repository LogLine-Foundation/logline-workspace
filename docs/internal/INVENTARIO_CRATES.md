# 📦 Inventário das Crates Publicadas

## 1. logline-core v0.1.1

### 📁 Estrutura
- **src/**: 8 arquivos (builder, consequence, ghost, lib, payload, signature, status, verb)
- **tests/**: 4 arquivos (invariants, lifecycle, serialization, verb_registry)
- **examples/**: 2 arquivos (simple_commit, ghost_record)
- **benches/**: 1 arquivo (creation)
- **docs/**: 1 arquivo (paper-i-logline-protocol.md)

### ✅ Arquivos de Configuração
- `Cargo.toml` ✓
- `README.md` ✓
- `CHANGELOG.md` ✓
- `LICENSE` (MIT) ✓
- `CITATION.cff` ✓
- `SECURITY.md` ✓
- `CODE_OF_CONDUCT.md` ✓
- `CONTRIBUTING.md` ✓
- `.gitignore` ✓

### 🔧 CI/CD
- `.github/workflows/ci.yml` ✓

### 📊 Status
- ✅ Publicado no crates.io
- ✅ Publicado no GitHub
- ✅ Testes: 4 arquivos
- ✅ Exemplos: 2 arquivos
- ✅ Benchmarks: 1 arquivo

### 📝 Features
- `default = ["std"]`
- `serde` (opcional)

---

## 2. json_atomic v0.1.0

### 📁 Estrutura
- **src/**: 7 arquivos (canonicalize, cycle, errors, lib, signed_fact, trajectory, version)
- **tests/**: 5 arquivos (canonicalization, canonicalization_edges, integration, seal_verify, trajectory)
- **examples/**: 3 arquivos (logline_seal, simple_seal, trajectory_match)
- **benches/**: 2 arquivos (canonicalize, seal)
- **docs/**: 1 arquivo (paper-ii-json-atomic.md)

### ✅ Arquivos de Configuração
- `Cargo.toml` ✓
- `README.md` ✓
- `CHANGELOG.md` ✓
- `LICENSE` (MIT) ✓
- `CITATION.cff` ✓
- `.gitignore` ✓

### 🔧 CI/CD
- `.github/workflows/ci.yml` ✓
- `.github/workflows/release-drafter.yml` ✓

### 📋 Templates GitHub
- `.github/ISSUE_TEMPLATE/bug_report.md` ✓
- `.github/ISSUE_TEMPLATE/feature_request.md` ✓
- `.github/ISSUE_TEMPLATE/config.yml` ✓
- `.github/ISSUE_TEMPLATE/v0.1.1-tracking.md` ✓
- `.github/pull_request_template.md` ✓
- `.github/release-drafter.yml` ✓

### 📊 Status
- ✅ Publicado no crates.io
- ✅ Publicado no GitHub
- ✅ Testes: 5 arquivos
- ✅ Exemplos: 3 arquivos
- ✅ Benchmarks: 2 arquivos

### 📝 Features
- `default = ["std", "unicode"]`
- `alloc` (planejado)
- `unicode` (opcional)

---

## 3. lllv-core v0.1.0

### 📁 Estrutura
- **src/**: 7 arquivos (capsule, crypto, errors, header, lib, manifest, version)
- **tests/**: 3 arquivos (capsule_roundtrip, crypto_aad, tamper)
- **benches/**: 1 arquivo (capsule)
- **examples/**: 0 arquivos

### ✅ Arquivos de Configuração
- `Cargo.toml` ✓
- `README.md` ✓
- `CHANGELOG.md` ✓
- `LICENSE` (MIT) ✓
- `CITATION.cff` ✓
- `SECURITY.md` ✓
- `CODE_OF_CONDUCT.md` ✓
- `RELEASE_NOTES.md` ✓
- `deny.toml` ✓
- `.gitignore` ✓

### 🔧 CI/CD
- `.github/workflows/ci.yml` ✓
- `.github/workflows/audit.yml` ✓
- `.github/workflows/deny.yml` ✓
- `.github/workflows/sbom.yml` ✓

### 📊 Status
- ✅ Publicado no crates.io
- ✅ Publicado no GitHub
- ✅ Testes: 3 arquivos (incluindo testes de ataque)
- ✅ Benchmarks: 1 arquivo

### 📝 Features
- `default = ["std", "manifest"]`
- `alloc` (disponível)
- `manifest` (opcional, json_atomic)

### 🔒 Hardening
- ✅ APIs seguras (verify_cid, verify_with)
- ✅ Testes de ataque (tamper, AAD, chaves erradas)
- ✅ Supply-chain (audit, deny, SBOM)

---

## 4. lllv-index v0.1.0

### 📁 Estrutura
- **src/**: 7 arquivos (errors, evidence, hash, lib, merkle, pack, search)
- **tests/**: 2 arquivos (basic, merkle_test)
- **examples/**: 1 arquivo (topk_verify)
- **benches/**: 0 arquivos

### ✅ Arquivos de Configuração
- `Cargo.toml` ✓
- `README.md` ✓
- `CHANGELOG.md` ✓
- `LICENSE` (MIT) ✓
- `CITATION.cff` ✓
- `SECURITY.md` ✓
- `CODE_OF_CONDUCT.md` ✓
- `RELEASE_NOTES.md` ✓
- `deny.toml` ✓
- `.gitignore` ✓

### 🔧 CI/CD
- `.github/workflows/ci.yml` ✓ (com matriz std/alloc)
- `.github/workflows/audit.yml` ✓
- `.github/workflows/deny.yml` ✓
- `.github/workflows/sbom.yml` ✓

### 📋 Templates GitHub
- `.github/ISSUE_TEMPLATE/bug_report.md` ✓
- `.github/ISSUE_TEMPLATE/feature_request.md` ✓
- `.github/ISSUE_TEMPLATE/config.yml` ✓
- `.github/pull_request_template.md` ✓

### 📊 Status
- ✅ Publicado no crates.io
- ✅ Publicado no GitHub
- ✅ Testes: 2 arquivos
- ✅ Exemplos: 1 arquivo

### 📝 Features
- `default = ["std", "manifest"]`
- `alloc` (disponível)
- `manifest` (opcional, json_atomic)

### 🔒 Hardening
- ✅ Merkle hardened (domain separation)
- ✅ Supply-chain (audit, deny, SBOM)
- ✅ CI com matriz std/alloc

---

## 📊 Resumo Comparativo

| Crate | Versão | Tests | Examples | Benches | Workflows | Templates | deny.toml | SECURITY | CoC |
|-------|--------|-------|----------|---------|-----------|-----------|-----------|----------|-----|
| logline-core | 0.1.1 | 4 | 2 | 1 | 1 | ❌ | ❌ | ✅ | ✅ |
| json_atomic | 0.1.0 | 5 | 3 | 2 | 2 | ✅ | ❌ | ❌ | ❌ |
| lllv-core | 0.1.0 | 3 | 0 | 1 | 4 | ❌ | ✅ | ✅ | ✅ |
| lllv-index | 0.1.0 | 2 | 1 | 0 | 4 | ✅ | ✅ | ✅ | ✅ |

### 🎯 Padrão Mais Completo (lllv-index)
- ✅ Todos os workflows (CI, audit, deny, SBOM)
- ✅ Templates GitHub completos
- ✅ deny.toml
- ✅ SECURITY.md
- ✅ CODE_OF_CONDUCT.md
- ✅ RELEASE_NOTES.md
- ✅ CHANGELOG.md
- ✅ README.md com badges
