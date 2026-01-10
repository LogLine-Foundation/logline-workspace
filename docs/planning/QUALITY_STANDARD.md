# 🧱 Padrão de Qualidade — Workspace LogLine (v1.1)

> Garantir **qualidade consistente**, **segurança de supply-chain** e **publicação previsível** para todas as crates do ecossistema LogLine (monorepo).

## 🔎 O que mudou nesta versão (refino)

* Regras **operacionais** com “Gates” (Hard/Soft fail).
* **Política de versão & MSRV** claras (bump, suporte, deprecações).
* **Checklist de publicação** enxuta e executável.
* **Invariantes de API** (sem `unwrap`/`panic!` em caminhos públicos).
* Linhas-guia de **segurança criptográfica** (fail-closed, inputs não confiáveis).
* **Docs e exemplos** com *doc-tests* e `cargo test --examples`.
* **Topologia de dependências** consolidada (camadas) + script de verificação.

---

## 0) Escopo & Princípios

* **Escopo**: `logline-workspace/` e **todas** as crates membros:
  `logline-core`, `json_atomic`, `lllv-core`, `lllv-index` e futuras.
* **Princípios**:

  1. **Invariantes explícitos** (MSRV, SemVer, “no unsafe”, sem `panic!` público).
  2. **Automação decide** (CI + scripts: verificação é gate).
  3. **Publicação determinística** (dry-run obrigatório + ordem topológica).
  4. **Docs & exemplos sempre verdes** (docs.rs, doctests, `examples/`).

---

## 1) Layout do Workspace

```
logline-workspace/
├─ Cargo.toml                 # [workspace], [workspace.package], [workspace.dependencies]
├─ rust-toolchain.toml        # (pin do channel stable; MSRV documentada no Cargo.toml)
├─ scripts/                   # verificadores e gates
│  ├─ verify_all_crates.sh
│  ├─ verify_quality.sh
│  ├─ verify_quality_python.py
│  └─ verify_dependencies.sh
├─ .github/
│  ├─ workflows/{ci.yml,audit.yml,deny.yml,sbom.yml,publish.yml}
│  └─ ISSUE_TEMPLATE/* , pull_request_template.md , CODEOWNERS
├─ logline-core/
├─ json_atomic/
├─ lllv-core/
└─ lllv-index/
```

### 1.1 `Cargo.toml` (raiz)

```toml
[workspace]
members = ["logline-core", "json_atomic", "lllv-core", "lllv-index"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.75"        # MSRV
license = "MIT"
repository = "https://github.com/LogLine-Foundation/logline-workspace"
homepage = "https://logline.foundation"
documentation = "https://docs.rs"

[workspace.dependencies]
blake3 = "1.5"
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
hex = "0.4"
```

> Cada crate herda metadados via `*.workspace = true`.

---

## 2) Topologia & Regras de Dependência

### 2.1 Camadas (pirâmide)

1. **Base**: `logline-core`
2. **Formato/Protocolo**: `json_atomic` → depende de `logline-core`
3. **Cripto/Verificação**: `lllv-core` → depende de `json_atomic`
4. **Indexação/Árvores**: `lllv-index` → depende de `lllv-core` e `json_atomic`

### 2.2 Proibições (enforced)

* ❌ Ciclos entre crates
* ❌ Dependência “para trás” (furar a pirâmide)
* ❌ `path`/`git` deps **em versões publicadas**
* ❌ Versões `*` (wildcards)

`./scripts/verify_dependencies.sh` falha o build se violar.

---

## 3) Estrutura Mínima por Crate

**Arquivos obrigatórios**

```
Cargo.toml
README.md
LICENSE (MIT)
.gitignore
```

**Recomendados**

```
CHANGELOG.md      # Keep a Changelog
SECURITY.md
CODE_OF_CONDUCT.md
deny.toml         # cargo-deny
CITATION.cff      # opcional
```

**Diretórios**

```
src/              # código
tests/            # ≥2 arquivos de integração
examples/         # ≥1 exemplo funcional
benches/          # opcional
```

---

## 4) Cargo.toml (por crate)

### 4.1 Metadados

```toml
[package]
name = "NOME"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
readme = "README.md"
description = "Descrição concisa"
keywords = ["keyword1","keyword2"]
categories = ["cryptography","encoding"]
documentation = "https://docs.rs/NOME"
exclude = [".github/**","deny.toml","SECURITY.md","CODE_OF_CONDUCT.md","CHANGELOG.md"]
```

### 4.2 Features & docs.rs

```toml
[features]
default = ["std"]
std = []
alloc = []  # preparação para no_std quando aplicável

[package.metadata.docs.rs]
features = ["std"]
no-default-features = false
all-features = false
```

---

## 5) Invariantes de Código & Testes

### 5.1 Código

* **Sem `unsafe` não justificado**
  Em `src/lib.rs`: `#![forbid(unsafe_code)]` (exceções precisam de justificativa no README/SECURITY).
* **Sem `unwrap`/`expect`/`panic!`** em **APIs públicas**.
  Use `Result<_, Error>` com `thiserror`; falhas são **fail-closed** e **diagnosticadas**.
* **Erros tipados** (`thiserror`) e mensagens úteis.
* **Compatibilidade de erros**: nunca reduzir granularidade em patch.
* **Doc-comments `///`** em itens públicos; exemplos compiláveis.
* **Const-correctness** e *zero allocation* quando possível em paths quentes.

### 5.2 Testes

* Unitários + Integração (≥2 arquivos).
* **Doc-tests**: `cargo test --doc` deve passar.
* **Exemplos**: `cargo test --examples` deve passar.
* **Edge & ataque** (quando cripto/árvore/parse): inputs inválidos, tamanhos ímpares, overflow de índice, hex malformado, AAD incorreta, etc.
* **no_std (quando aplicável)**: `cargo build --no-default-features --features alloc`.

---

## 6) Segurança & Supply-chain

### 6.1 Auditorias

* **cargo-audit** (workflow `audit.yml`) — push/PR + semanal.
* **cargo-deny** (workflow `deny.yml`)

  * `advisories.vulnerability = "deny"`
  * `licenses.allow = ["MIT","Apache-2.0","BSD-3-Clause","ISC","Unicode-DFS-2016","Zlib","CC0-1.0"]`
  * `bans.wildcards = "deny"`

### 6.2 SBOM

* **CycloneDX** no `sbom.yml`, anexado no release.

### 6.3 Higiene

* Sem `git`/`path` em versões publicadas.
* Sem `*` (wildcards) em dependências.
* **`cargo tree`** antes de publicar.
* (Opcional) **`cargo geiger`** para mapear `unsafe` em deps.
* Itens proibidos no repo: segredos, dados sensíveis, binários gerados, dumps, licenças incompatíveis.

### 6.4 Linhas-guia para cripto/árvores

* **Fail-closed** ao validar provas/assinaturas.
* **Nenhum `panic!` com input não confiável**.
* Checagem de **tamanho** e **limites** antes de indexar buffers.
* **Ordem de concatenação** de Merkle explícita e testada (par/ímpar).
* **Constância** de domínios (ex.: `H("node"||L||R)`, `H("leaf"||data)`).

---

## 7) Documentação

**README por crate** (mínimo):

* Badges: crates.io, docs.rs, CI, MSRV, license
* Instalação (`[dependencies]`)
* Quickstart (mínimo funcional)
* API principal
* Segurança (ex.: `verify_cid()` vs `verify_with()`)
* Licença

**docs.rs**:

* Mesmas features do `Cargo.toml` (consistência)
* Exemplos em `examples/` referenciados

---

## 8) CI/CD (Gates)

**CI raiz (`ci.yml`)** — *Hard Gates*:

* `cargo fmt --all -- --check`
* `cargo clippy --all-targets --all-features -- -D warnings`
* `cargo test --all-features`
* `cargo test --doc`
* `cargo test --examples`
* `bash scripts/verify_all_crates.sh`
* `bash scripts/verify_dependencies.sh`

**Segurança**:

* `audit.yml` (cargo-audit)
* `deny.yml` (cargo-deny)
* `sbom.yml` (CycloneDX em releases)

**Publicação**:

* `publish.yml` (Trusted Publishing via OIDC em tag `v*`)

---

## 9) Política de Versão & MSRV

* **SemVer**: `MAJOR.MINOR.PATCH`

  * *PATCH*: correções compatíveis
  * *MINOR*: novas APIs compatíveis
  * *MAJOR*: breaking changes (doc + migração)
* **MSRV = 1.75**

  * Bump de MSRV ⇒ *MINOR* (mínimo) + nota de release.
  * Suporte a MSRV vigente por ≥ 6 meses.
* **Deprecações**: `#[deprecated]` + entrada no CHANGELOG + substituto indicado.

---

## 10) Publicação (ordem & checklist)

### 10.1 Ordem

1. `logline-core`
2. `json_atomic`
3. `lllv-core`
4. `lllv-index`

### 10.2 Checklist executável (por crate)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --doc
cargo test --examples
cargo deny check all
cargo audit
cargo package --list
cargo publish --dry-run
```

> Em crates dependentes, substituir `path = "../X"` por `X = "x.y.z"` antes do publish.

---

## 11) Templates & Governança

* **Issue Templates**: `bug_report.md`, `feature_request.md`, `config.yml`
* **PR Template**: checklist (fmt, clippy, test, changelog)
* **CODEOWNERS**: donos por crate/módulo
* **Mudanças no Padrão**: PR com impacto nos scripts e CI, versão do padrão incrementada (ex.: `v1.1`).

---

## 12) Aceite (Proof of Done)

**Workspace**

* `verify_all_crates.sh` → exit 0
* `verify_dependencies.sh` → sem ciclos/violação de camadas
* CI (`ci.yml`) verde em todos os jobs

**Por crate**

* Estrutura obrigatória presente
* README com badges + quickstart + segurança (quando aplicável)
* `#![forbid(unsafe_code)]` (ou justificativa explícita)
* Zero `unwrap`/`expect`/`panic!` em API pública
* `clippy -D warnings` **limpo**
* Testes (incluindo doc/examples) **ok**
* `deny`/`audit` **ok**
* `package --list` **higienizado**
* `publish --dry-run` **ok**

---

## 13) Pós-Publicação

* Conferir crates.io e docs.rs
* Release no GitHub com `RELEASE_NOTES.md` + SBOM anexado
* Monitorar downloads, dependentes, issues, advisories
* Manter CHANGELOG e dependências em dia

---

## Apêndice A — Snippets úteis

**Cabeçalho lib**

```rust
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
```

**Badges (README)**

```md
[![crates.io](https://img.shields.io/crates/v/NOME.svg)](https://crates.io/crates/NOME)
[![docs.rs](https://docs.rs/NOME/badge.svg)](https://docs.rs/NOME)
![CI](https://img.shields.io/github/actions/workflow/status/LogLine-Foundation/logline-workspace/ci.yml?label=CI)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![license](https://img.shields.io/badge/license-MIT-blue.svg)
```

**docs.rs (Cargo.toml)**

```toml
[package.metadata.docs.rs]
features = ["std"]
no-default-features = false
all-features = false
```

---

**Fim.** Se quiser, já te mando também o `publish.yml` de Trusted Publishing alinhado a estes gates pra fechar o ciclo de ponta a ponta. 🚀
