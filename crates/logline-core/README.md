# logline-core

[![crates.io](https://img.shields.io/crates/v/logline-core.svg)](https://crates.io/crates/logline-core)
[![docs.rs](https://docs.rs/logline-core/badge.svg)](https://docs.rs/logline-core)
[![CI](https://github.com/LogLine-Foundation/logline-core/actions/workflows/ci.yml/badge.svg)](https://github.com/LogLine-Foundation/logline-core/actions/workflows/ci.yml)
![license](https://img.shields.io/badge/license-MIT-blue.svg)
![no_std](https://img.shields.io/badge/no__std-compatible-informational)

> The Conceptual Atom of Verifiable Action — Paper I §3 (9-field tuple, lifecycle, invariants, Ghost Records)

`logline-core` implementa o átomo básico do **LogLine Protocol**:
- **9-field tuple rígido** (`who, did, this, when, confirmed_by, if_ok, if_doubt, if_not, status`)
- **Lifecycle determinístico**: `DRAFT → PENDING → COMMITTED` ou `GHOST`
- **Invariants obrigatórios**: `if_ok`, `if_doubt`, `if_not`
- **Ghost Records**: trilha forense para intents abandonadas
- **Signing-ready**: `Signer`/`Signature` + bytes determinísticos (placeholder v0.1)
- **no_std-friendly**: funciona com `--no-default-features` (usa apenas `alloc`)

---

## Instalação

```toml
[dependencies]
logline-core = "0.1"
```

### Features

* `std` (default) — ergonomia com `std`
* `serde` — `Serialize/Deserialize` para todas as estruturas

Sem `std`:

```bash
cargo build --no-default-features --features serde
```

---

## Quickstart

```rust
use logline_core::*;

struct NoopSigner;
impl Signer for NoopSigner {
    fn sign(&self, msg: &[u8]) -> Result<Signature, SignError> {
        Ok(Signature { alg: "none".into(), bytes: msg.to_vec() })
    }
}

fn main() {
    let signer = NoopSigner;

    // Paper I: "nada acontece sem estar assinado" — attempt já nasce assinado
    let draft = LogLine::builder()
        .who("did:ubl:alice")
        .did(Verb::Approve)
        .this(Payload::Text("purchase:123".into()))
        .when(1_735_671_234) // unix ns (canônico: ISO8601 na serialização JSON✯Atomic)
        .if_ok(Outcome { label: "approved".into(), effects: vec!["emit_receipt".into()] })
        .if_doubt(Escalation { label: "manual_review".into(), route_to: "auditor".into() })
        .if_not(FailureHandling { label: "rejected".into(), action: "notify".into() })
        .build_draft().unwrap();

    // Paper I: assinar antes de freeze (attempt já nasce assinado)
    let signed = draft.sign(&signer).unwrap();
    let pending = signed.freeze().unwrap();
    
    // Paper I: commit requer assinatura obrigatória
    let committed = pending.commit(&signer).unwrap();
    assert!(matches!(committed.status, Status::Committed));
}
```

---

## Modelo de dados (9-field tuple)

| Campo          | Tipo              | Notas                                        |
| -------------- | ----------------- | -------------------------------------------- |
| `who`          | `String`          | DID futuro (ex.: `did:ubl:...`)              |
| `did`          | `Verb`            | `Transfer`, `Deploy`, `Approve`, ou `Custom` |
| `this`         | `Payload`         | `None` \| `Text` \| `Bytes`                    |
| `when`         | `u64`             | Timestamp unix (ns)                          |
| `confirmed_by` | `Option<String>`  | Identidade de confirmação (opcional)         |
| `if_ok`        | `Outcome`         | Consequência positiva (obrigatória)          |
| `if_doubt`     | `Escalation`      | Rota de dúvida (obrigatória)                 |
| `if_not`       | `FailureHandling` | Tratamento de falha (obrigatório)            |
| `status`       | `Status`          | `Draft` \| `Pending` \| `Committed` \| `Ghost`  |

Invariants (checadas em `build_draft()` e `verify_invariants()`):

* `who` não vazio
* `when` > 0
* `if_ok`, `if_doubt`, `if_not` sempre presentes e não vazios

---

## Lifecycle

```
DRAFT --freeze()--> PENDING --commit()--> COMMITTED
   \                               
    \--abandon()------------------> GHOST
```

* `Committed → Ghost` é proibido.
* `abandon(reason)` produz `GhostRecord` com forense opcional.

---

## Assinaturas (Paper I conformant)

* **Assinatura obrigatória**: Paper I exige que "nada acontece sem estar assinado". 
  - `sign()` assina DRAFT/PENDING antes de commit
  - `commit()` requer signer obrigatório (não mais opcional)
  - `abandon_signed()` permite abandon assinado (attempt já nasce assinado)
* Trait `Signer`/`Signature` expostos
* `to_signable_bytes()` com ordem fixa (v0.1 placeholder)
* Próximos passos: `ed25519-dalek` + canonicidade JSON✯Atomic

---

## `no_std`

A crate é `no_std`-compatible: desative `std` e ela usa apenas `alloc`.

---

## Exemplos

* `examples/simple_commit.rs` — create → freeze → commit
* `examples/ghost_record.rs` — draft → abandon → ghost

Rode:

```bash
cargo run --example simple_commit
cargo run --example ghost_record
```

---

## Benchmarks

Criterion (dev-only):

```bash
cargo bench
```

---

## Testes

```bash
cargo test
cargo test --features serde
```

---

## Conformidade com Paper I

Esta implementação está **alinhada com Paper I** (The LogLine Protocol — The Conceptual Atom of Verifiable Action):

✅ **9-field tuple rígido** — especificação completa implementada  
✅ **Lifecycle determinístico** — `DRAFT → PENDING → COMMITTED | GHOST` com enforcement  
✅ **Consequence invariants** — `if_ok`, `if_doubt`, `if_not` obrigatórios  
✅ **Ghost Records** — trilha forense para intents abandonadas  
✅ **Assinatura obrigatória** — Paper I: "nada acontece sem estar assinado"  
✅ **VerbRegistry** — validação de verbos contra ALLOWED_ACTIONS (Paper I §3.1)  
✅ **Payload tipado** — suporte a `Payload::Json` com feature `serde` (Paper I: JSON estrito)  
✅ **Tempo canônico** — `when` como unix-ns interno; ISO8601 na serialização canônica (Paper I §3.1)

**Notas de implementação:**
- `confirmed_by`: obrigatório para ações L3+ (regra de runtime/policy)
- `to_signable_bytes()`: placeholder v0.1 (ordem fixa); JSON✯Atomic canônico em `json-atomic` (v0.3.0)
- Hash-chain e Merkle Tree: implementados no ledger (fora do escopo desta crate core)

---

## MSRV

Rust stable 1.75+ (alvo). PRs para suportar versões anteriores são bem-vindos.

---

## Roadmap

* **v0.1.1** — `to_signable_bytes()` estável (separadores ASCII, doc do formato); docs.rs melhoradas
* **v0.2.0** — feature `ed25519` (opcional) com `ed25519-dalek`; `Signature` real
* **v0.3.0** — `json-atomic` (crate irmã) para bytes canônicos JSON✯Atomic + prova de assinatura
* **v0.3.x** — enriquecimento de **Ghost forensics** (razões padronizadas, timestamps extras)
* **v0.4.0** — verbos canônicos publicamente versionados; `Payload::Map` (serde) opcional
* **v0.5.0** — integração com recibos UBL (adapters) e exemplos "agent decisions"
* **v1.0.0** — freeze de API (compat longa), MSRV declarada, guia de migração

---

## Contribuindo

Contribuições são bem-vindas! Veja [`CONTRIBUTING.md`](CONTRIBUTING.md) para detalhes.

---

## Segurança

Relatos de vulnerabilidades devem seguir [`SECURITY.md`](SECURITY.md).

---

---

## Ecossistema

**Criptografia & canônica**: [`json_atomic`](https://github.com/logline-foundation/json_atomic) — o **átomo criptográfico** (Paper II): canonicalização JSON✯Atomic, BLAKE3 CID e DV25-Seal (Ed25519) para Signed Facts imutáveis.

---

## Licença

MIT — veja [`LICENSE`](LICENSE).

---

## Repository

https://github.com/LogLine-Foundation/logline-core
