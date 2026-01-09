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

## CLI

```bash
logline compile "book a table for two"
logline info
logline version
```

- `compile <text>`: produces canonical JSON, AST CID, canon CID.
- `info`/`version`: prints build info.

## License
MIT
