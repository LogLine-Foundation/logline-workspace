# Contributing to LogLine Workspace

Thank you for helping strengthen the protocol! 🌟

## Quick Start

1. Fork and create a branch:
   ```bash
   git checkout -b feat/my-feature
   ```

2. Install toolchain:
   ```bash
   rustup default stable
   ```

3. Run quality checks:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo deny check
   ```

## MSRV

**Minimum Supported Rust Version: 1.75**

If your change requires a higher MSRV, mention it in the PR.

## Commit Style

We follow **Conventional Commits**:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation only
- `chore:` Maintenance, deps
- `refactor:` Code restructuring
- `test:` Test additions
- `perf:` Performance improvements

Examples:
- `feat(ubl): add rotation policy`
- `fix(sirp): domain separation in frame signing`

## Code Standards

### Lints

- `#![forbid(unsafe_code)]` — No unsafe code
- Clippy as error (`-D warnings`) in CI
- `cargo deny` for dependency auditing

### Feature Flags

Each crate may have:

- `std` (default): Standard library support
- `signing`: Ed25519 signatures
- `async`: Tokio-based async IO
- `metrics`: Prometheus-style metrics

Maintain `no_std` compatibility where possible.

### Documentation

- All public items must have doc comments
- Examples should be tested (`cargo test --doc`)
- Use `# Errors` section for fallible functions

## Testing

### Unit Tests

Cover invariants, edge cases, and error conditions.

### Integration Tests

Located in `tests/` directory. Test cross-crate interactions.

### Golden Vectors

Use `tests/vectors/` for canonical test fixtures:

- `*.json` → expected canonical bytes
- `*.bin` → expected TLV frames
- `*.ndjson` → expected ledger entries

### Benchmarks

Use Criterion in `benches/`. Avoid obvious regressions.

## Crate Guidelines

### Naming

- `atomic-*`: Protocol primitives (types, crypto, codec)
- `tdln-*`: Natural language → AST → policy
- `lllv-*`: Storage and indexing
- `logline-*`: Integration and bundles

### Dependencies

- Minimize dependencies
- No duplicates across workspace
- Use workspace dependencies when possible

## Release Process (Maintainers)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Tag:
   ```bash
   git tag -a v0.x.y -m "crate-name v0.x.y"
   git push --tags
   ```
4. Publish:
   ```bash
   cargo publish -p crate-name
   ```

## Code of Conduct

Be respectful. See `CODE_OF_CONDUCT.md`. Incidents can be reported via `SECURITY.md`.
