#!/usr/bin/env bash
set -euo pipefail

require() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required tool: $1" >&2
    exit 1
  fi
}

require cargo
require jq
require rustup
require cargo-deny
require cargo-llvm-cov
require cargo-semver-checks
require cargo-udeps
require cargo-hack
require cargo-geiger
require cargo-spellcheck

MSRV_TOOLCHAIN="1.75.0"

echo "== FMT =="
cargo fmt --all -- --check

echo "== CLIPPY (strict) =="
# Pedantic + nursery + deny warnings across all targets/features.
cargo clippy --workspace --all-targets --all-features -- \
  -D warnings \
  -W clippy::pedantic \
  -W clippy::nursery \
  -W clippy::cargo

echo "== DOCS (all features, docs.rs cfg, warnings as errors) =="
# Ensure docs build with all features and docs.rs cfg; deny warnings.
RUSTDOCFLAGS="--cfg docsrs -Dwarnings" cargo doc --workspace --all-features --no-deps

echo "== TESTS =="
cargo test --workspace --all-features
cargo test --workspace --doc
cargo test --workspace --examples

echo "== RELEASE BUILD (all features) =="
cargo build --workspace --release --all-features

echo "== SECURITY / DEPS (cargo-deny) =="
cargo deny check licenses bans sources advisories

echo "== UNUSED DEPS (cargo-udeps) =="
# udeps needs nightly for -Z binary-dep-depinfo support
cargo +nightly udeps --workspace --all-targets --all-features

echo "== COVERAGE (cargo-llvm-cov, fail-under=85%) =="
cargo llvm-cov --workspace --all-features --fail-under-lines 85

echo "== SEMVER CHECKS (cargo-semver-checks) =="
cargo semver-checks --workspace --all-features

echo "== FEATURE MATRIX (cargo-hack) =="
cargo hack check --workspace --all-targets --each-feature --no-dev-deps

echo "== MSRV BUILD/TEST (${MSRV_TOOLCHAIN}) =="
rustup run "$MSRV_TOOLCHAIN" cargo test --workspace --all-features

echo "== UNSAFE AUDIT (cargo-geiger) =="
cargo geiger --workspace --all-features --print-summary

echo "== SPELLCHECK (cargo-spellcheck) =="
cargo spellcheck -m exhaustive

echo "== DEP GRAPH DUPLICATES (cargo tree -d) =="
cargo tree -d --workspace --all-features

echo "== PACKAGE DRY LIST (each crate) =="
while IFS= read -r mf; do
  echo "-> $(dirname \"$mf\")"
  cargo package --manifest-path "$mf" --list --allow-dirty > /dev/null
done < <(cargo metadata --no-deps --format-version=1 | jq -r '.packages[].manifest_path')

echo "== PUBLISH DRY-RUN (each crate) =="
while IFS= read -r mf; do
  echo "-> $(dirname \"$mf\")"
  cargo publish --manifest-path "$mf" --dry-run --allow-dirty > /dev/null
done < <(cargo metadata --no-deps --format-version=1 | jq -r '.packages[].manifest_path')

echo "== OK =="
