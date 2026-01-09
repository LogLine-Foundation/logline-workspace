# logline-core v0.1.0 — The Conceptual Atom of Verifiable Action

## What's new

* **9-field tuple rígido**: `who, did, this, when, confirmed_by, if_ok, if_doubt, if_not, status`
* **Lifecycle determinístico**: `DRAFT → PENDING → COMMITTED | GHOST`
* **Consequence invariants obrigatórios** (`if_ok`, `if_doubt`, `if_not`)
* **Ghost Records** (`abandon` / `abandon_signed`) para trilha forense
* **Assinatura obrigatória** (`sign()` e `commit(&Signer)`)
* **VerbRegistry** + `freeze_with_registry()` (validação de `did`)
* **Payload::Json** (feature `serde`) + compat `no_std` (usa `alloc`)
* Exemplos, testes, benchmark e CI

## Docs & crate

* **crates.io**: `logline-core = "0.1.0"`
* **docs.rs**: https://docs.rs/logline-core/0.1.0 (com badges e README incorporado)

## Security

* Assinatura obrigatória em todas as transições críticas
* Verificação estrita de invariants
* Ghost Records para auditoria forense

## Links

* 📦 [crates.io](https://crates.io/crates/logline-core)
* 📚 [docs.rs](https://docs.rs/logline-core)
* 🔗 [Projeto irmão: json_atomic](https://github.com/logline-foundation/json-atomic)
* 📖 [Paper I: The LogLine Protocol](https://github.com/logline-foundation/logline-core/blob/main/docs/paper-i-logline-protocol.md)

## MSRV

Rust stable 1.75+
