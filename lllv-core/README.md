# LLLV — The Retrieval Atom (lllv-core)

[![crates.io](https://img.shields.io/crates/v/lllv-core.svg)](https://crates.io/crates/lllv-core)
[![docs.rs](https://docs.rs/lllv-core/badge.svg)](https://docs.rs/lllv-core)
![license](https://img.shields.io/badge/license-MIT-blue.svg)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![no_std](https://img.shields.io/badge/no__std-ready-success)

**lllv-core** define a **Vector Capsule** com cabeçalho binário assinado e manifesto canônico (Paper II), endereçada por **BLAKE3 (CID)** e selada com **Ed25519**. É a base do *Retrieval Atom* (Paper III).

- **Header** (binário, fixo): `MAGIC | VER | FLAGS | TS | CID | DIM | LEN | SIG`
- **CID** = `blake3(payload)` — endereçamento por conteúdo
- **Assinatura** = `Ed25519.sign(header_without_sig || payload)`
- **Manifesto** (JSON✯Atomic): canoniza → CID → DV25-Seal

## Instalação

```toml
[dependencies]
lllv-core = "0.1.0"
# integrações recomendadas
logline-core = { version = "0.1.1", features = ["serde"] }
json_atomic  = "0.1.1" # já incluso via feature "manifest"
ed25519-dalek = "2"
rand = "0.8"
```

## Quickstart

```rust
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use lllv_core::{Capsule, CapsuleHeader, CapsuleFlags, encrypt_chacha20poly1305, decrypt_chacha20poly1305};

fn main() {
    // Chave de assinatura (Ed25519)
    let sk = SigningKey::generate(&mut OsRng);

    // Payload vetorial (pode ser embedding quantizado)
    let dim: u16 = 3;
    let payload = vec![1u8, 2, 3, 4, 5, 6]; // exemplo

    // (Opcional) criptografia do payload com AAD=(vector_id||CID)
    let vector_id = "doc:1";
    let key = [7u8; 32];
    let nonce = [9u8; 12];
    let aad = [vector_id.as_bytes(), &[]].concat(); // simples; use CID real em prod.

    // Cria a cápsula (sem criptografia)
    let cap = Capsule::create(dim, &payload, CapsuleFlags::NONE, &sk).unwrap();
    cap.verify_cid().unwrap(); // verifica integridade
    cap.verify_with(&sk.verifying_key()).unwrap(); // verifica autenticidade

    // Cria a cápsula (criptografada) — payload = nonce || ciphertext
    let enc = encrypt_chacha20poly1305(&payload, &key, &nonce, &aad).unwrap();
    let cap_enc = Capsule::create(dim, &enc, CapsuleFlags::ENCRYPTED, &sk).unwrap();
    cap_enc.verify_with(&sk.verifying_key()).unwrap();

    // Decriptar depois
    let (nonce2, ct2) = enc.split_at(12);
    let pt = decrypt_chacha20poly1305(ct2, nonce2.try_into().unwrap(), &key, &aad).unwrap();
    assert_eq!(pt, payload);
}
```

## API (essencial)

```rust
pub struct CapsuleHeader { /* MAGIC, VER, FLAGS, TS, CID, DIM, LEN, SIG */ }
pub struct Capsule { pub header: CapsuleHeader, pub payload: Vec<u8> }

impl Capsule {
  pub fn create(dim: u16, payload: &[u8], flags: CapsuleFlags, sk: &ed25519_dalek::SigningKey)
    -> Result<Self, LllvError>;
  pub fn to_bytes(&self) -> Vec<u8>;
  pub fn from_bytes(raw: &[u8]) -> Result<Self, LllvError>;
  pub fn verify_cid(&self) -> Result<(), LllvError>;  // integridade (CID)
  pub fn verify_with(&self, pk: &ed25519_dalek::VerifyingKey) -> Result<(), LllvError>;  // integridade + autenticidade
  #[deprecated] pub fn verify(&self) -> Result<(), LllvError>;  // use verify_cid() ou verify_with()
}

pub fn encrypt_chacha20poly1305(plain: &[u8], key: &[u8;32], nonce: &[u8;12], aad: &[u8])
  -> Result<Vec<u8>, LllvError>;
pub fn decrypt_chacha20poly1305(cipher: &[u8], nonce: &[u8;12], key: &[u8;32], aad: &[u8])
  -> Result<Vec<u8>, LllvError>;
```

### Manifesto JSON✯Atomic (Paper II)

Se a feature `manifest` estiver ativa, use `json_atomic` para **canonizar, hashear e selar** o manifesto (DV25-Seal).

```rust
use lllv_core::{CapsuleManifest, seal_manifest};
let mf = CapsuleManifest::minimal("doc:1", "text/plain", dim, "q8");
let fact = seal_manifest(&mf, &sk)?; // SignedFact (json_atomic)
```

## Segurança (como verificar corretamente)

- **Integridade** do payload: `capsule.verify_cid()?` — verifica se o CID corresponde ao payload
- **Autenticidade** (assinatura Ed25519): `capsule.verify_with(&verifying_key)?` — verifica integridade + assinatura

> `verify()` está **deprecated** — use `verify_with(pk)` para checagem completa ou `verify_cid()` apenas para integridade.

### Detalhes de implementação

* Assinatura é calculada sobre **`header_without_sig || payload`**.
* **CID** cobre o payload em repouso (**cifrado** ou não).
* Use AAD com identidade forte (`vector_id || CID`) quando cifrar.

## Supply-chain

- CI roda `cargo-audit` e `cargo-deny` em PRs/merge.
- Releases geram **SBOM CycloneDX** anexado no GitHub Release.

---

MIT • MSRV 1.75+ • pronto para evoluir para `alloc/no_std` no v0.1.1.
