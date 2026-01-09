# lllv-index — Verifiable Top-K + Merkle Evidence

[![crates.io](https://img.shields.io/crates/v/lllv-index.svg)](https://crates.io/crates/lllv-index)
[![docs.rs](https://docs.rs/lllv-index/badge.svg)](https://docs.rs/lllv-index)
![CI](https://img.shields.io/github/actions/workflow/status/LogLine-Foundation/lllv-index/ci.yml?label=CI)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![no_std](https://img.shields.io/badge/no__std-ready-success)
![license](https://img.shields.io/badge/license-MIT-blue.svg)

**LLLV Index** empacota vetores com **Merkle Evidence** para buscas Top-K verificáveis.
O verificador recomputa o **Merkle root** com *domain separation*:

- folha (documento/cápsula): `H("leaf" || id || cid)`
- nó interno: `H("node" || left || right)`

> **Irmão:** [`lllv-core`](https://github.com/LogLine-Foundation/lllv-core) (cápsulas assinadas).

---

## Instalação

```toml
[dependencies]
lllv-index = "0.1.0"
lllv-core  = "0.1.0"
ed25519-dalek = { version = "2.1", features = ["pkcs8"] }
hex = "0.4"
```

## Quickstart (Top-K + verificação)

```rust
use ed25519_dalek::SigningKey;
use lllv_core::{Capsule, CapsuleFlags};
use lllv_index::{IndexPackBuilder, QueryRequest};

fn f32_to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

fn main() {
    // Dimensão
    let dim = 3u16;
    let sk = SigningKey::from_bytes(&[7u8; 32]);

    // 3 vetores ortogonais
    let a = Capsule::create(dim, &f32_to_bytes(&[1.0, 0.0, 0.0]), CapsuleFlags::NONE, &sk).unwrap();
    let b = Capsule::create(dim, &f32_to_bytes(&[0.0, 1.0, 0.0]), CapsuleFlags::NONE, &sk).unwrap();
    let c = Capsule::create(dim, &f32_to_bytes(&[0.0, 0.0, 1.0]), CapsuleFlags::NONE, &sk).unwrap();

    // monta o pack
    let mut builder = IndexPackBuilder::new(dim);
    builder.add_capsule("a".into(), a).unwrap();
    builder.add_capsule("b".into(), b).unwrap();
    builder.add_capsule("c".into(), c).unwrap();
    let pack = builder.build(None).unwrap();

    // consulta e verifica a evidência
    let ev = pack.query(&QueryRequest::from_vec(&[1.0, 0.0, 0.0]), 2).unwrap();
    pack.verify(&ev).unwrap();
    println!("✅ verificado: root={}", ev.index_pack_cid);
}
```

## Formato da Evidência (JSON)

```json
{
  "index_pack_cid": "a3c2…",
  "dim": 3,
  "results": [
    {
      "id": "a",
      "score": 1.0,
      "leaf_hex": "…32bytes…",
      "path": [
        { "sibling_hex": "…", "sibling_is_right": true },
        { "sibling_hex": "…", "sibling_is_right": false }
      ]
    }
  ]
}
```

## Segurança

- **Integridade:** `Merkle root` com *domain separation*: `"leaf"` e `"node"`.
- **Autenticidade opcional:** combine com `lllv-core` (cápsulas assinadas).
- **Hex robusto:** parsing defensivo em paths/evidências; erros estritos e descritivos.
- **Supply-chain:** CI com `cargo-audit`, `cargo-deny` e **SBOM (CycloneDX)**.

## `no_std` / `alloc`

- `default = ["std", "manifest"]`
- `alloc` disponível (parcial) para ambientes sem `std`.

## MSRV

- Rust **1.75+**

## Licença

MIT © LogLine Foundation

## Links

- Crate: https://crates.io/crates/lllv-index  
- Docs:  https://docs.rs/lllv-index  
- Core:  https://github.com/LogLine-Foundation/lllv-core
