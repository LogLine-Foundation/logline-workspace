#!/usr/bin/env bash
set -euo pipefail

echo "== FMT =="
cargo fmt --all -- --check

echo "== CLIPPY =="
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "== TESTS =="
cargo test --workspace --all-features
cargo test --workspace --doc
cargo test --workspace --examples

echo "== PACKAGE DRY LIST (each crate) =="
for mf in $(cargo metadata --no-deps --format-version=1 | jq -r '.packages[].manifest_path'); do
  echo "-> $(dirname "$mf")"
  cargo package --manifest-path "$mf" --list > /dev/null
done

echo "OK"
