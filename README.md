# LogLine Crates (TDLN Edition)

## What is this?
- This repository contains the Rust crates used by LogLine.
- Includes the TDLN crates: tdln-ast, tdln-proof, tdln-compiler, tdln-gate.
- Also includes json-atomic and other supporting crates.

## How to build and test
- Requires Rust 1.75+ (stable).
- Run `cargo fmt --all -- --check`
- Run `cargo clippy --all-targets --all-features -- -D warnings`
- Run `cargo test --all-features --locked`

## Publishing
- Use the publish workflow (`.github/workflows/publish.yml`).
- Provide `crate` and `version` inputs.
- Workflow tags the crate (`<crate>-v<version>`) and publishes with `cargo publish`.

## License
- MIT OR Apache-2.0
