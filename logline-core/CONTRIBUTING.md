# Contribuindo para logline-core

Valeu por fortalecer o átomo 🌟

## Como começar

1. Faça um fork e crie um branch:
   ```bash
   git checkout -b feat/minha-feature
   ```

2. Instale toolchain estável:
   ```bash
   rustup default stable
   ```

3. Garanta qualidade:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   ```

## Estilo de commits

Preferimos **Conventional Commits**:

* `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `perf:`

Exemplos:

* `feat: add GhostRecord reason field`
* `fix: forbid committed->ghost transition`

## MSRV

Alvo atual: Rust 1.75+. Se sua mudança exigir MSRV maior, mencione no PR.

## Lints

* `#![forbid(unsafe_code)]` — sem `unsafe`
* Clippy como erro (`-D warnings`) no CI

## Features e compat

* `std` (default) e `serde` opcionais
* Mantenha a compatibilidade `no_std` (use `alloc` quando necessário)

## Testes & Examples

* Cubra invariants, lifecycle e serialização (quando `serde`)
* Exemplos em `examples/` devem compilar com `--no-default-features` quando possível

## Benchmarks

* Use Criterion em `benches/` (dev-only)
* Evite regressões óbvias

## Processo de release (maintainers)

1. Atualize `CHANGELOG.md` (se existir) e versão no `Cargo.toml`
2. Tag:
   ```bash
   git tag -a v0.x.y -m "logline-core v0.x.y"
   git push --tags
   ```
3. Publicação:
   ```bash
   cargo publish
   ```

## Código de Conduta

Seja respeitoso. Incidentes podem ser reportados conforme `SECURITY.md` (abuso também será tratado).
