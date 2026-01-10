# 🚀 lllv-core v0.1.0 — Publish Checklist

## ✅ Pré-requisitos (já feitos)

- [x] Cargo.toml configurado corretamente
- [x] `cargo fmt --all` ✓
- [x] `cargo clippy --all-targets --all-features -- -D warnings` ✓
- [x] `cargo test --all-features` ✓ (7 testes passando)
- [x] `cargo package` ✓ (22 arquivos, 54.3KiB)

## 📦 Comandos de Publicação

### 1. Verificar dry-run
```bash
cd "/Users/ubl-ops/Crates LogLine/lllv-core"
cargo publish --dry-run
```

### 2. Publicar no crates.io
```bash
# Se ainda não fez login:
# cargo login

cargo publish
```

### 3. Criar tag e release no GitHub
```bash
cd "/Users/ubl-ops/Crates LogLine/lllv-core"
git add -A
git commit -m "lllv-core v0.1.0: Verifiable Capsules with hardening"
git tag -a v0.1.0 -m "lllv-core v0.1.0"
git push origin main --tags
```

### 4. Criar GitHub Release
- Vá para: https://github.com/LogLine-Foundation/lllv-core/releases/new
- Tag: `v0.1.0`
- Title: `lllv-core v0.1.0 — Verifiable Capsules`
- Description: copiar conteúdo de `RELEASE_NOTES.md`

---

## 🔗 Desbloquear lllv-index

Após `lllv-core v0.1.0` aparecer no crates.io (pode levar alguns minutos):

```bash
cd "/Users/ubl-ops/Crates LogLine/lllv-index"
cargo update
cargo test --all-features
cargo publish --dry-run
cargo publish
```

---

## 📋 Release Notes (copiar no GitHub Release)

```markdown
## lllv-core v0.1.0 — Verifiable Capsules

### Highlights
- `Capsule::create(dim, bytes, flags, &sk)` com CID (BLAKE3) e assinatura Ed25519
- Verificações explícitas:
  - `verify_cid()` → integridade (CID)
  - `verify_with(pk)` → integridade + autenticidade (assinatura)
- `no_std/alloc` ready (design), MSRV 1.75, CI com audit/deny/SBOM
- Testes de tamper e AAD cobrindo cenários críticos

### Security
- Assinatura calculada sobre **CID dos bytes canônicos**; APIs separadas para evitar uso indevido.
- Supply-chain hardening: `cargo-audit`, `cargo-deny`, SBOM CycloneDX nos releases.

### Breaking Changes
- `verify()` está deprecated — use `verify_cid()` para integridade ou `verify_with(pk)` para autenticidade completa.
```
