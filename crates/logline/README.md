# logline

One-stop LogLine bundle and CLI. Includes:
- TDLN stack (ast/proof/compiler/gate) with dv25 + ed25519 and JSON✯Atomic enforced
- LogLine core
- LLLV core/index
- JSON✯Atomic strict rules by default

## Library usage

```toml
[dependencies]
logline = "0.1"
```

```rust
use logline::gate::decide;
use logline::compiler::compile;
use logline::json_atomic;
```

## CLI (single binary: `logline`)

```bash
# TDLN compile/info/version
logline compile "book a table for two"
logline info
logline version

# Atomic utilities
logline send --url https://... --json payload.json --dim 0x0001
logline tail --path ./ubl_dir --pretty --kind payment
logline bench --url https://... --dim 0x0001 --json payload.json --concurrency 8 --count 100
logline keygen
logline ubl verify --path ./events.ubl
logline completions --shell zsh
```

- `compile <text>`: produces canonical JSON, AST CID, canon CID.
- `send`/`tail`/`bench`/`ubl verify`/`keygen`/`completions` mirror the atomic toolkit; `dev-server` stays behind feature `server`.

## License
MIT
