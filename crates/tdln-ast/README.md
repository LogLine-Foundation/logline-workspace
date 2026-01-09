# tdln-ast

[![docs.rs](https://docs.rs/tdln-ast/badge.svg)](https://docs.rs/tdln-ast)
![license](https://img.shields.io/badge/license-MIT-blue.svg)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)

Canonical AST for **TDLN** — deterministic, proof-carrying translation of NL/DSL into a Logical Atom.

```rust
use tdln_ast::SemanticUnit;

let su = SemanticUnit::from_intent("Turn on the lights in the kitchen");
let cid = su.cid_blake3(); // BLAKE3 of canonical bytes
```

- Deterministic canonical bytes (sorted keys)
- CID = `BLAKE3(canonical_bytes)`
- `json-atomic` can be enabled via feature for strict canonicalization
