# json_atomic

[![crates.io](https://img.shields.io/crates/v/json_atomic.svg)](https://crates.io/crates/json_atomic)
[![docs.rs](https://docs.rs/json_atomic/badge.svg)](https://docs.rs/json_atomic)
[![CI](https://github.com/logline-foundation/json_atomic/actions/workflows/ci.yml/badge.svg)](https://github.com/logline-foundation/json_atomic/actions/workflows/ci.yml)
![license](https://img.shields.io/badge/license-MIT-blue.svg)
![no_std](https://img.shields.io/badge/no__std-ready-success)

**JSON✯Atomic** — o **átomo criptográfico** do *Paper II*: canonicalização rigorosa (Same Semantics = Same Bytes = Same Hash), **CID BLAKE3**, e **DV25-Seal (Ed25519)** para **Signed Facts** imutáveis e verificáveis.

> **Projeto irmão**: [`logline-core`](https://github.com/logline-foundation/logline-core) — o **átomo conceitual** (Paper I). Aqui, qualquer valor `Serialize` vira **bytes canônicos**, que viram **CID** e então um **selo Ed25519**.

---

## Instalação

```toml
[dependencies]
json_atomic = "0.1.0"
# Integração opcional (recomendada)
logline-core = { version = "0.1.0", features = ["serde"] }
```

### Features

- `std` (default): experiência completa para dev (tests/examples/benches).
- `alloc`: habilita build **no_std** com alocação.
- `unicode`: normalização **NFC** (aloc-only). Recomendado para confluência semântica de strings.

---

## Quickstart

```rust
use ed25519_dalek::{SigningKey, Signer};
use json_atomic::{seal_value, verify_seal, SignedFact};
use serde::Serialize;

#[derive(Serialize)]
struct Note { title: String, done: bool }

fn main() {
    // chave de demo — em produção, derive de seed/keystore
    let sk = SigningKey::generate(&mut rand::rngs::OsRng);

    let n = Note { title: "Hello, Canon!".into(), done: false };

    // 1) Canonize + hash + seal  → SignedFact
    let fact: SignedFact = seal_value(&n, &sk).expect("sealed");

    // 2) Verificar
    verify_seal(&fact).expect("valid");

    // 3) CID hex (BLAKE3)
    println!("cid={}", fact.cid_hex());
}
```

### Integrando com `logline-core`

```rust
use ed25519_dalek::SigningKey;
use json_atomic::{seal_logline, verify_seal};
use logline_core::{LogLine, Verb, Payload};

fn seal_entire_logline(line: &LogLine, sk: &SigningKey) {
    let fact = seal_logline(line, sk).expect("sealed");
    verify_seal(&fact).expect("valid");
}
```

---

## Conformidade (Paper II)

- **Objetos** → chaves ordenadas **lexicograficamente** (ordem estável).
- **Strings** → **Unicode NFC** (se `feature = "unicode"`).
- **Números** → **inteiros** em forma mínima (sem `+`, sem zeros à esquerda). `float` → **erro**.
- **Boolean / null** → preservados.
- **Arrays** → ordem preservada (sem reordenação).
- **Whitespace** → nenhum fora de strings; encoding estável.
- **Cycle of Truth** → `canonize(value)` → `CID := BLAKE3(bytes)` → `seal := Ed25519.sign(CID)`.
- **Verify** → recalcula canônico + CID e verifica a assinatura estrita.
- **Headers mínimos** em `SignedFact`:
  - `canon_ver = "1"`, `format_id = "json-atomic/1"`
  - `hash_alg = "blake3"`, `sig_alg = "ed25519"`

> Objetivo: **Same Semantics = Same Bytes = Same Hash**. Qualquer representação JSON semanticamente igual deve resultar na **mesma sequência de bytes canônicos** e, portanto, no **mesmo CID**.

📖 **Especificação completa**: `docs/paper-ii-json-atomic.md` (mantenha versionado; excluído do publish)

---

## API (essencial)

```rust
fn canonize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError>;
fn seal_value<T: serde::Serialize>(value: &T, sk: &SigningKey) -> Result<SignedFact, SealError>;
fn verify_seal(fact: &SignedFact) -> Result<(), VerifyError>;
fn seal_logline(line: &logline_core::LogLine, sk: &SigningKey) -> Result<SignedFact, SealError>;
```

```rust
pub struct SignedFact {
    pub canonical: Vec<u8>,   // bytes canônicos (JSON✯Atomic)
    pub cid: [u8; 32],        // BLAKE3(canonical)
    pub signature: [u8; 64],  // Ed25519.sign(CID)
    pub public_key: [u8; 32], // Ed25519 pk
    pub hash_alg:  &'static str,  // "blake3"
    pub sig_alg:   &'static str,  // "ed25519"
    pub canon_ver: &'static str,  // "1"
    pub format_id: &'static str,  // "json-atomic/1"
}
```

---

## `alloc/no_std`

Build **sem `std`**, apenas com `alloc`:

```bash
cargo build --no-default-features --features alloc
cargo build --no-default-features --features "alloc,unicode"
```

> Observação: `tests/examples/benches` usam `std`. Em CI, compilamos a **lib** no modo `alloc` para garantir compatibilidade.

---

## Segurança

- Assinatura **Ed25519** é feita **sobre o CID** (BLAKE3 dos **bytes canônicos**), nunca sobre JSON bruto.
- Mude o `SigningKey`/`VerifyingKey` conforme sua HSM/keystore.
- Persistir somente o **SignedFact** já garante recomputação independente e verificação de integridade.

---

## Testes e Benchmarks

```bash
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo bench --no-run
```

Inclui testes de canto:
- Strings em **NFC** (decomposed vs composed → **iguais** no canônico)
- Inteiros com zeros à esquerda (como string vs inteiro → **diferentes**)
- Objetos aninhados com chaves em ordem diferente (canônico → **igual**)

---

## Roadmap / Changelog

- Veja o [CHANGELOG.md](./CHANGELOG.md) e a seção **[Unreleased]**.
- Planejamento inclui: cabeçalhos canônicos estendidos (content-type + schema-hash), **Merkle chunking** para documentos grandes e vetores determinísticos `no_std`.

---

## Licença

MIT — veja [LICENSE](./LICENSE).

---

## Agradecimentos

Parte do ecossistema **LogLine / JSON✯Atomic** — *verifiable, privacy-first intelligence*.
