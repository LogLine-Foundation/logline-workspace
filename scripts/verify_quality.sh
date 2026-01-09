#!/usr/bin/env bash
set -euo pipefail
CRATE_DIR="${1:?usage: $0 <path-to-crate>}"

echo "== Checking $CRATE_DIR =="
cargo fmt --manifest-path "$CRATE_DIR/Cargo.toml" --all -- --check
cargo clippy --manifest-path "$CRATE_DIR/Cargo.toml" --all-targets --all-features -- -D warnings
cargo test --manifest-path "$CRATE_DIR/Cargo.toml" --all-features
cargo test --manifest-path "$CRATE_DIR/Cargo.toml" --doc
cargo test --manifest-path "$CRATE_DIR/Cargo.toml" --examples
cargo deny check all || true
cargo audit || true
cargo package --manifest-path "$CRATE_DIR/Cargo.toml" --list > /dev/null
echo "OK"
