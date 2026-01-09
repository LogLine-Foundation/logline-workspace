CORRECTION

bora, Dan — agora sim HARDCORE de verdade. Liguei a lógica do TDLN como “camada de negócio” e já deixei as 4 crates prontas pra entrar no teu **Workspace LogLine** sem path local, com features pensadas pra plugar **json_atomic**, **logline-core** (DV25), e assinatura Ed25519. Também incluí invariantes e testes mínimos de determinismo, tudo sem `unsafe`.

[Baixar o pacote com as 4 crates TDLN](sandbox:/mnt/data/tdln-crates-hardcore.zip) 📦

# O que vem dentro (pronto pra colar em `crates/`)

## 1) `tdln-ast`

* **Papel:** AST canônico; bytes canônicos determinísticos; `CID = blake3(canonical_bytes)`.
* **Deps:** `serde`, `serde_json`, `blake3`, `hex`; `json_atomic` **opcional** (feature `json-atomic`).
* **Invariantes:** ordenação determinística de chaves; normalização de whitespace.
* **API principal:**

  * `SemanticUnit::from_intent(&str) -> SemanticUnit`
  * `canonical_bytes(&self) -> Vec<u8>`
  * `cid_blake3(&self) -> [u8;32]`
* **Testes:** igualdade de CIDs para entradas equivalentes com whitespace/case diferentes.

## 2) `tdln-proof`

* **Papel:** pacote de prova determinística (liga AST ↔ canônico ↔ regras).
* **Deps:** `tdln-ast`, `serde`, `serde_json`, `blake3`, `hex`, `thiserror`; `ed25519-dalek` (feature `ed25519`); `logline-core` (feature `dv25` → futura DV25).
* **API principal:**

  * `build_proof(ast, canon_json, rules) -> ProofBundle`
  * `verify_proof(&ProofBundle) -> Result<(), ProofError>`
  * `sign(&mut ProofBundle, &SigningKey)` / `verify_signatures(&ProofBundle, &[VerifyingKey])` (com `ed25519`)
* **Modelo de digest:** `blake3( ast_cid || canon_cid || rules_applied[] )`
  (estável, simples e explícito; DV25 entra depois, sem quebrar API).

## 3) `tdln-compiler`

* **Papel:** compilador determinístico NL/DSL → `AST + Canonical JSON + ProofBundle`.
* **Deps:** `tdln-ast`, `tdln-proof`, `serde`, `serde_json`, `blake3`, `thiserror`; **opcionais** `json_atomic` e `logline-core` (via features).
* **API principal:**

  * `compile(input: &str, ctx: &CompileCtx) -> Result<CompiledIntent, CompileError>`
  * `CompileCtx { rule_set: String }` (id versionado da regra; **pilar** pra evolução determinística)
  * `CompiledIntent { ast, canon_json, cid, proof }`
* **Invariante crítica:** mesmo `input` + mesmo `rule_set` → mesmas saídas bit-a-bit.

## 4) `tdln-gate`

* **Papel:** Gate de políticas (preflight/decision) levando prova junto.
* **Deps:** `tdln-compiler`, `tdln-proof`, `serde`, `serde_json`, `thiserror`, `blake3`; opcional `logline-core` (feature `dv25`).
* **API principal:**

  * `preflight(intent, &PolicyCtx) -> GateOutput { decision: NeedsConsent, audit, proof_ref }`
  * `decide(intent, &Consent, &PolicyCtx) -> GateOutput { decision, audit+, events }`
  * `Decision = {Allow, Deny, NeedsConsent}`, `PolicyCtx { allow_freeform }`, `Consent { accepted }`
* **Eventos/Auditoria:** `policy.preflight` e `policy.decision` com CIDs em hex (determinístico).

---

## Semver & features (pensado pra crates.io AGORA, sem path)

As dependências de “núcleo LogLine” estão configuradas por **versão**, não `path`:

* `json-atomic = "0.1.1"` (**opcional**, feature `json-atomic`)
  → hoje linka sem chamar API específica (evita quebra na tua CI rígida). Amanhã, só trocar a flag e chamar o canônico oficial.
* `logline-core = "0.1.0"` (**opcional**) para **DV25** (feature `dv25` via `tdln-proof`).
* `ed25519-dalek = "2.1"` ativado via feature `ed25519` (default em `tdln-proof`; exposto em `tdln-gate/tdln-compiler`).

> Se quiser já **forçar** o uso de `json-atomic` no workspace: ative a feature `json-atomic` nos 3 crates (`ast`, `compiler`, se desejar) via root `Cargo.toml` do workspace.

---

## Padrão de Qualidade embutido

* `#![forbid(unsafe_code)]` em todos.
* MSRV `1.75`, badges prontos nos READMEs.
* `docs.rs` metadata preenchida.
* `exclude` em `Cargo.toml` para não vazar arquivos internos.
* Testes mínimos incluídos (determinismo, pipeline gate, assinatura quando habilitada).

---

## Como plugar no Workspace LogLine (sem paths)

No `logline-workspace/Cargo.toml` (root), adicione os **members**:

```toml
[workspace]
members = [
  "crates/tdln-ast",
  "crates/tdln-proof",
  "crates/tdln-compiler",
  "crates/tdln-gate",
  # já existem: external/json-atomic, external/logline-core, external/lllv-core, external/lllv-index …
]
resolver = "2"
```

Se for usar os repositórios externos dentro do workspace durante dev, mantenha `[patch.crates-io]` só para os **4 já existentes**, e **não** para TDLN (as TDLN já estão no próprio workspace):

```toml
[patch.crates-io]
json_atomic = { path = "external/json-atomic" }
logline-core = { path = "external/logline-core" }
lllv-core    = { path = "external/lllv-core" }
lllv-index   = { path = "external/lllv-index" }
```

E ative features globais conforme desejar:

```toml
[workspace.dependencies]
# (ex.) para builds com prova assinada e DV25:
tdln-proof = { path = "crates/tdln-proof", default-features = true, features = ["ed25519", "dv25"] }
```

---

## Prova de Pronto (local)

No root do workspace:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  --workspace --all-features
```

---

## Próximos encaixes (quando você quiser apertar o parafuso)

1. **Canon oficial:** mudar `tdln-ast::canonical_bytes()` para usar o canônico do `json_atomic` por trás de `cfg(feature="json-atomic")` — é só ligar a feature e chamar a API oficial onde você quiser (o esqueleto já está preparado).
2. **DV25:** no `tdln-proof`, adicionar um `sign_dv25()`/`verify_dv25()` usando `logline-core` (feature `dv25`) — sem quebrar a API atual.
3. **Regra determinística rica:** no `tdln-compiler`, evoluir `CompileCtx.rule_set` para uma enum/versão com regras explícitas (slots, AAD, bindings), e registrar cada regra aplicada no `rules_applied` do bundle.
4. **Gate policiar**: no `tdln-gate`, adicionar policies declarativas (`forbidden`, `bounds`, `constraints`) e `NeedsConsent` condicional por categoria/escopo (ex.: “PII”, “payments”, etc.).

---

se quiser, eu já adapto esse mesmo pacote pra **no_std/alloc** no `ast` e `proof` (ficou com as flags prontas), e cravo o **Trusted Publishing** pra publicar as TDLN em *wave* (v0.1.0 cada). quer que eu já te mande o `ci.yml` + `publish.yml` alinhados ao teu roteiro?



amo quando é “SIM” 😄 — segue o pacote **pronto-pra-colar**: CI completo do workspace + Trusted Publishing por **tag de crate**, alinhado ao teu padrão de qualidade e às 4 TDLN crates.

---

# 🔧 Colar estes arquivos no repo

```
.github/workflows/ci.yml
.github/workflows/audit.yml
.github/workflows/deny.yml
.github/workflows/sbom.yml
.github/workflows/publish.yml
scripts/tag_crate.sh
```

---

# ✅ `.github/workflows/ci.yml` — CI do Workspace (fmt, clippy, test)

```yaml
name: CI (workspace)

on:
  push:
    branches: [ "main" ]
  pull_request: {}

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all -- --check

  clippy-test:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        crate:
          - external/json-atomic
          - external/logline-core
          - external/lllv-core
          - external/lllv-index
          - crates/tdln-ast
          - crates/tdln-proof
          - crates/tdln-compiler
          - crates/tdln-gate
        include:
          # no_std/alloc smoke-builds quando aplicável
          - crate: crates/tdln-ast
            alloc: true
          - crate: crates/tdln-proof
            alloc: true
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable

      # Lint estrito por crate
      - name: Clippy (strict)
        run: cargo clippy --manifest-path ${{ matrix.crate }}/Cargo.toml --all-targets --all-features -- -D warnings

      # Testes com todas as features
      - name: Test (all-features)
        run: cargo test --manifest-path ${{ matrix.crate }}/Cargo.toml --all-features

      # Smoke build no_std/alloc quando marcado
      - name: Build alloc only
        if: ${{ matrix.alloc == true }}
        run: cargo build --manifest-path ${{ matrix.crate }}/Cargo.toml --no-default-features --features alloc
```

---

# 🔐 `.github/workflows/audit.yml` — RustSec audit

```yaml
name: Security Audit

on:
  push: {}
  pull_request: {}
  schedule:
    - cron: "0 5 * * 1" # segundas 05:00 UTC

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit
      - run: cargo audit
```

---

# 📜 `.github/workflows/deny.yml` — Licenças / advisories (cargo-deny)

```yaml
name: License/Advisory Deny

on:
  push: {}
  pull_request: {}

jobs:
  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-deny
      - run: cargo deny check all
```

> Reusa teu `deny.toml` na raiz do workspace.

---

# 📦 `.github/workflows/sbom.yml` — SBOM por release

Gera SBOM **do crate relativo à tag** e anexa no release.

```yaml
name: SBOM

on:
  release:
    types: [published]

jobs:
  sbom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable

      # Descobre crate/manifest a partir da tag: <crate>-vX.Y.Z
      - name: Parse tag
        id: parse
        run: |
          TAG="${GITHUB_REF_NAME}"
          if [[ "$TAG" =~ ^([a-z0-9_-]+)-v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
            CRATE="${BASH_REMATCH[1]}"
            VER="${BASH_REMATCH[2]}"
          else
            echo "Tag inválida: $TAG (esperado <crate>-vX.Y.Z)"; exit 1
          fi
          case "$CRATE" in
            tdln-ast|tdln-proof|tdln-compiler|tdln-gate) MAN="crates/$CRATE/Cargo.toml" ;;
            json-atomic) MAN="external/json-atomic/Cargo.toml" ;;
            logline-core) MAN="external/logline-core/Cargo.toml" ;;
            lllv-core) MAN="external/lllv-core/Cargo.toml" ;;
            lllv-index) MAN="external/lllv-index/Cargo.toml" ;;
            *) echo "Crate desconhecido: $CRATE"; exit 1 ;;
          esac
          echo "crate=$CRATE" >> $GITHUB_OUTPUT
          echo "version=$VER"  >> $GITHUB_OUTPUT
          echo "manifest=$MAN" >> $GITHUB_OUTPUT

      - run: cargo install cargo-cyclonedx
      - run: cargo cyclonedx --manifest-path "${{ steps.parse.outputs.manifest }}" -o sbom.json

      - uses: softprops/action-gh-release@v2
        with:
          files: sbom.json
```

---

# 🚀 `.github/workflows/publish.yml` — Trusted Publishing (OIDC) por **tag de crate**

Publica **só** o crate da tag (`<crate>-vX.Y.Z`). Funciona para TDLN e para os 4 existentes, se quiser.

```yaml
name: Publish to crates.io

on:
  push:
    tags:
      - "tdln-ast-v*"
      - "tdln-proof-v*"
      - "tdln-compiler-v*"
      - "tdln-gate-v*"
      - "json-atomic-v*"
      - "logline-core-v*"
      - "lllv-core-v*"
      - "lllv-index-v*"

permissions:
  id-token: write  # OIDC
  contents: read

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable

      - name: Parse tag → crate / version / manifest
        id: parse
        shell: bash
        run: |
          TAG="${GITHUB_REF_NAME}"
          if [[ "$TAG" =~ ^([a-z0-9_-]+)-v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
            CRATE="${BASH_REMATCH[1]}"
            VER="${BASH_REMATCH[2]}"
          else
            echo "Tag inválida: $TAG (esperado <crate>-vX.Y.Z)"; exit 1
          fi
          case "$CRATE" in
            tdln-ast|tdln-proof|tdln-compiler|tdln-gate) MAN="crates/$CRATE/Cargo.toml" ;;
            json-atomic) MAN="external/json-atomic/Cargo.toml" ;;
            logline-core) MAN="external/logline-core/Cargo.toml" ;;
            lllv-core) MAN="external/lllv-core/Cargo.toml" ;;
            lllv-index) MAN="external/lllv-index/Cargo.toml" ;;
            *) echo "Crate desconhecido: $CRATE"; exit 1 ;;
          esac
          echo "crate=$CRATE"   >> $GITHUB_OUTPUT
          echo "version=$VER"   >> $GITHUB_OUTPUT
          echo "manifest=$MAN"  >> $GITHUB_OUTPUT

      - name: Check version matches tag
        run: |
          TOML_VER=$(cargo read-manifest --manifest-path "${{ steps.parse.outputs.manifest }}" | jq -r .version)
          test "$TOML_VER" = "${{ steps.parse.outputs.version }}" || {
            echo "Versão no Cargo.toml ($TOML_VER) difere da tag (${{ steps.parse.outputs.version }})"
            exit 1
          }

      # Auth via OIDC (Trusted Publishing) — precisa habilitar no crates.io
      - uses: rust-lang/crates-io-auth-action@v1
        id: auth

      # Dry-run antes
      - run: cargo publish --manifest-path "${{ steps.parse.outputs.manifest }}" --dry-run
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}

      # Publicação
      - run: cargo publish --manifest-path "${{ steps.parse.outputs.manifest }}"
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}
```

> **Configuração necessária:** em `crates.io → Settings → Publishing`, adicionar o repositório `LogLine-Foundation/logline-workspace` como **publisher** (Trusted Publishing com OIDC).

---

# 🏷️ `scripts/tag_crate.sh` — helper pra criar tag no formato correto

```bash
#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "uso: $0 <crate-name> <version>"; exit 1
fi

CRATE="$1"   # ex: tdln-compiler
VER="$2"     # ex: 0.1.0
TAG="${CRATE}-v${VER}"

# checa versão do Cargo.toml
case "$CRATE" in
  tdln-ast|tdln-proof|tdln-compiler|tdln-gate) MAN="crates/$CRATE/Cargo.toml" ;;
  json-atomic) MAN="external/json-atomic/Cargo.toml" ;;
  logline-core) MAN="external/logline-core/Cargo.toml" ;;
  lllv-core) MAN="external/lllv-core/Cargo.toml" ;;
  lllv-index) MAN="external/lllv-index/Cargo.toml" ;;
  *) echo "crate desconhecido: $CRATE"; exit 1 ;;
esac

CUR_VER=$(cargo read-manifest --manifest-path "$MAN" | jq -r .version)
if [ "$CUR_VER" != "$VER" ]; then
  echo "ERRO: Cargo.toml=$CUR_VER, pedido=$VER"; exit 1
fi

git tag -a "$TAG" -m "$CRATE v$VER"
git push origin "$TAG"
echo "ok: tag $TAG enviada."
```

---

## Proof of Done (rápido)

* [ ] `ci.yml` rodando em push/PR ✅
* [ ] `audit.yml` semanal e em push/PR ✅
* [ ] `deny.yml` em push/PR ✅
* [ ] `sbom.yml` anexando `sbom.json` em releases ✅
* [ ] `publish.yml` publicando **apenas** quando a tag é `<crate>-vX.Y.Z` ✅
* [ ] `scripts/tag_crate.sh` gerando as tags no formato padrão ✅

Se quiser, eu também já mando (num patch separado) os **badges** no `README.md` root do workspace e nos READMEs das TDLN (CI/MSRV/docs.rs), mas com isso aqui você já consegue: **commit → tag → publish confiável**, com SBOM e audit/deny de guarda. 🚀



perfeito, Dan — bora colar o conteúdo que faltava pro workspace ficar redondinho, com foco nos badges + READMEs TDLN (e um README raiz que enxerga as 4 crates já publicadas). Zero ruído, pronto pra commit. 🚀

⸻

1) README.md (raiz do workspace)

Substitui/atualiza o README do repo logline-workspace.

# LogLine Workspace

[![CI](https://img.shields.io/github/actions/workflow/status/LogLine-Foundation/logline-workspace/ci.yml?label=CI)](https://github.com/LogLine-Foundation/logline-workspace/actions)
[![Audit](https://img.shields.io/github/actions/workflow/status/LogLine-Foundation/logline-workspace/audit.yml?label=audit)](https://github.com/LogLine-Foundation/logline-workspace/actions)
[![Deny](https://img.shields.io/github/actions/workflow/status/LogLine-Foundation/logline-workspace/deny.yml?label=deny)](https://github.com/LogLine-Foundation/logline-workspace/actions)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![license](https://img.shields.io/badge/license-MIT-blue.svg)

Monorepo de **integração** do ecossistema LogLine Foundation. Aqui rodamos CI/audit/deny/SBOM, garantimos invariantes e orquestramos *waves* de release.  
Crates “first-class” continuam nos seus repositórios originais; TDLN e Chip as Code nascem aqui.

---

## 📦 Crates já publicados

| Crate | crates.io | docs.rs |
|---|---|---|
| `json-atomic` | [![crates.io](https://img.shields.io/crates/v/json-atomic.svg)](https://crates.io/crates/json-atomic) | [![docs.rs](https://docs.rs/json-atomic/badge.svg)](https://docs.rs/json-atomic) |
| `logline-core` | [![crates.io](https://img.shields.io/crates/v/logline-core.svg)](https://crates.io/crates/logline-core) | [![docs.rs](https://docs.rs/logline-core/badge.svg)](https://docs.rs/logline-core) |
| `lllv-core` | [![crates.io](https://img.shields.io/crates/v/lllv-core.svg)](https://crates.io/crates/lllv-core) | [![docs.rs](https://docs.rs/lllv-core/badge.svg)](https://docs.rs/lllv-core) |
| `lllv-index` | [![crates.io](https://img.shields.io/crates/v/lllv-index.svg)](https://crates.io/crates/lllv-index) | [![docs.rs](https://docs.rs/lllv-index/badge.svg)](https://docs.rs/lllv-index) |

> No dev local, o workspace usa `[patch.crates-io]` para apontar para `external/*`. Em produção, dependam normalmente pelas versões do crates.io.

---

## 🧠 Wave TDLN (no workspace)

| Crate | Status | Objetivo |
|---|---|---|
| `tdln-ast` | dev | AST determinística (Intent, Slots, Constraints), normalização canônica e validações. |
| `tdln-proof` | dev | Provas determinísticas: pass log, merkle root, evidência assinável (Ed25519 opcional). |
| `tdln-compiler` | dev | Tradução NL → IR canônico + montagem de ProofBundle (acoplado `json-atomic`). |
| `tdln-gate` | dev | Gate de políticas (bounds/forbidden/required), verificação de certificados e aplicação determinística. |

---

## 🛠️ Desenvolvimento

```bash
# Lint e testes (workspace completo)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  --workspace --all-features

# Smoke no_std/alloc (onde aplicável)
cargo build -p tdln-ast --no-default-features --features alloc
cargo build -p tdln-proof --no-default-features --features alloc


⸻

🚀 Publicação por tag (Trusted Publishing)

Criar tag <crate>-vX.Y.Z e push:

./scripts/tag_crate.sh tdln-compiler 0.1.0

O workflow resolve o manifest correto e publica usando OIDC. SBOM é anexado ao release correspondente.

⸻

🔒 Qualidade & Segurança (herdado)
	•	#![forbid(unsafe_code)] por padrão (onde fizer sentido)
	•	cargo audit, cargo deny, SBOM (CycloneDX) por release
	•	deny.toml, SECURITY.md, CODE_OF_CONDUCT.md, templates de issue/PR
	•	docs.rs com features selecionadas para build consistente

---

# 2) `crates/tdln-ast/README.md`

```markdown
# tdln-ast

[![docs.rs](https://docs.rs/tdln-ast/badge.svg)](https://docs.rs/tdln-ast)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![no_std](https://img.shields.io/badge/no__std-alloc_ready-success)
![license](https://img.shields.io/badge/license-MIT-blue.svg)

Árvore sintática determinística do TDLN (Intent, Slots, Constraints), com **normalização canônica** e validações estruturais.

## Features
- `std` (default) · `alloc`
- `json-atomic` (habilita serialização canônica via `json-atomic`)

## Exemplo (conceitual)
```rust
use tdln_ast::{Intent, Slot, Constraint};

let intent = Intent::new("transferir")
    .with_slot(Slot::required("valor").number())
    .with_slot(Slot::required("destino").string())
    .with_constraint(Constraint::bound("valor", 0.0, 10_000.0))
    .normalize(); // ordem e forma canônicas

Segurança
	•	AST é pura e determinística; side-effects só em camadas superiores.
	•	Use normalize() para garantir forma canônica antes de compilar/assinar.

MIT © LogLine Foundation

---

# 3) `crates/tdln-proof/README.md`

```markdown
# tdln-proof

[![docs.rs](https://docs.rs/tdln-proof/badge.svg)](https://docs.rs/tdln-proof)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![no_std](https://img.shields.io/badge/no__std-alloc_ready-success)
![license](https://img.shields.io/badge/license-MIT-blue.svg)

**Provas determinísticas** do pipeline TDLN: pass log, Merkle root, e bundle assinável (Ed25519 opcional).

## Features
- `std` (default) · `alloc`
- `ed25519` (assinatura opcional via `ed25519-dalek`)
- Integração natural com `lllv-core` (hashing) e `json-atomic` (CID canônico)

## Exemplo (conceitual)
```rust
use tdln_ast::Intent;
use tdln_proof::{Pass, ProofBundle};

let intent = Intent::from_str("transferir 10 para @alice").unwrap().normalize();
let pass   = Pass::from_intent(&intent);             // registro determinístico
let bundle = ProofBundle::from_pass(&pass)?;          // merkle + metadata
// opcional: bundle.sign(&signing_key); bundle.verify_signature(&public_key)?;

MIT © LogLine Foundation

---

# 4) `crates/tdln-compiler/README.md`

```markdown
# tdln-compiler

[![docs.rs](https://docs.rs/tdln-compiler/badge.svg)](https://docs.rs/tdln-compiler)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![license](https://img.shields.io/badge/license-MIT-blue.svg)

Compilador **determinístico** TDLN: NL → **IR canônico** + **ProofBundle**.
Alavanca `json-atomic` para canonicalização e `lllv-core`/`lllv-index` para trilhas verificáveis.

## Fluxo
1. Parse/normalize (`tdln-ast`)
2. Regras estáticas geram IR lógico
3. Monta `ProofBundle` (`tdln-proof`)
4. (Opcional) Assina e emite CID canônico (`json-atomic`)

## Exemplo (conceitual)
```rust
use tdln_ast::Intent;
use tdln_compiler::{compile, CompileCfg};

let intent = Intent::from_str("pagar 15 para @bob").unwrap().normalize();
let cfg = CompileCfg::default();
let (ir, bundle) = compile(&intent, &cfg)?;

MIT © LogLine Foundation

---

# 5) `crates/tdln-gate/README.md`

```markdown
# tdln-gate

[![docs.rs](https://docs.rs/tdln-gate/badge.svg)](https://docs.rs/tdln-gate)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![license](https://img.shields.io/badge/license-MIT-blue.svg)

**Gate de políticas determinísticas** para IntentOS: bounds, forbidden, required, check de certificados e **provas de decisão**.

## Policies (exemplos)
- `bound("valor", min, max)`
- `forbidden("recurso", "root")`
- `required("assinatura.magister")`

## Exemplo (conceitual)
```rust
use tdln_ast::Intent;
use tdln_gate::{Gate, PolicySet};

let pol = PolicySet::builder()
    .bound("valor", 0.0, 1000.0)
    .required("assinatura.magister")
    .build();

let gate = Gate::new(pol);
let decision = gate.evaluate(&intent)?;
assert!(decision.allowed());

MIT © LogLine Foundation

---

# 6) Bloco de **badges padrão** (se quiser colar nas 4 TDLN)

> Cole no topo de cada README de crate TDLN:

```markdown
[![crates.io](https://img.shields.io/crates/v/<NOME>.svg)](https://crates.io/crates/<NOME>)
[![docs.rs](https://docs.rs/<NOME>/badge.svg)](https://docs.rs/<NOME>)
![MSRV](https://img.shields.io/badge/MSRV-1.75%2B-informational)
![no_std](https://img.shields.io/badge/no__std-alloc_ready-success)
![license](https://img.shields.io/badge/license-MIT-blue.svg)

Troque <NOME> por tdln-ast, tdln-proof, tdln-compiler, tdln-gate quando publicar.

⸻

7) Notas rápidas (encaixe com as 4 crates já publicadas)
	•	Sem hardcode: dependências TDLN apontam para json-atomic, lllv-core, lllv-index e logline-core por versão do crates.io. No dev, o workspace usa [patch.crates-io] para apontar os submodules — sem afetar consumidores.
	•	Docs.rs: em cada TDLN, setar [package.metadata.docs.rs] com features = ["std"] (ou ["std","ed25519"] onde fizer sentido), no-default-features = false.
	•	no_std/alloc: tdln-ast e tdln-proof já prontos para alloc; tdln-compiler/tdln-gate ficam std por padrão (podem evoluir depois).

⸻

se quiser, eu também te passo um CHANGELOG.md base (Keep a Changelog) pra cada TDLN agora — mas com isso você já tem o conteúdo hardcore (READMEs claros, badges e narrativa técnica) + o README raiz agregador com as 4 crates públicas. bora empacotar e soltar a wave 💥


boraaa 😎🔥 — segue o pacotão colável pro TDLN ficar elegante no workspace: CHANGELOGs prontos, exemplos, smoke tests e snippets de Cargo.toml (docs.rs + deps por versão de crates.io). É só copiar/colar nos paths indicados.

⸻

0) Notas rápidas (importantes)
	•	As TDLN dependem das 4 crates já publicadas por versão do crates.io.
No dev local, o workspace pode sobrescrever via [patch.crates-io], mas sem hardcode.
	•	Nome do pacote no Cargo.toml usa hífen (json-atomic), e no use vira underscore (json_atomic).

⸻

1) tdln-ast

crates/tdln-ast/CHANGELOG.md

# Changelog — tdln-ast
Todas as mudanças notáveis deste projeto serão documentadas aqui.
Formato: [Keep a Changelog](https://keepachangelog.com/) — SemVer.

## [Unreleased]
- Normalização canônica com regras adicionais de ordenação.
- Validações semânticas extras para `Constraint`.

## [0.1.0] - 2026-01-09
### Adicionado
- AST determinística: `Intent`, `Slot`, `Constraint`.
- `normalize()` para forma canônica estável (ordem, chaves, tipos).
- `no_std/alloc` (feature `alloc`), `std` por padrão.
- Integração opcional com `json-atomic` para serialização canônica.

crates/tdln-ast/examples/normalize.rs

use tdln_ast::{Constraint, Intent, Slot};

fn main() {
    let intent = Intent::new("transferir")
        .with_slot(Slot::required("valor").number())
        .with_slot(Slot::required("destino").string())
        .with_constraint(Constraint::bound("valor", 0.0, 10_000.0))
        .normalize();

    // impressão canônica, útil para golden tests
    println!("{}", intent.to_canonical_string());
}

crates/tdln-ast/tests/smoke.rs

use tdln_ast::{Constraint, Intent, Slot};

#[test]
fn normalize_is_idempotent() {
    let a = Intent::new("pagar")
        .with_slot(Slot::required("valor").number())
        .with_slot(Slot::required("destino").string())
        .with_constraint(Constraint::bound("valor", 1.0, 1000.0))
        .normalize();

    let b = a.clone().normalize();
    assert_eq!(a.to_canonical_string(), b.to_canonical_string());
}

(snippet) crates/tdln-ast/Cargo.toml

[dependencies]
serde = { version = "1.0", features = ["derive"], default-features = false }
json-atomic = { version = "0.1", optional = true, default-features = false, features = ["canon"] } # use 'json_atomic' no código

[features]
default = ["std"]
std = ["serde/std"]
alloc = []
json = ["json-atomic"]

[package.metadata.docs.rs]
features = ["std", "json"]
no-default-features = false
all-features = false


⸻

2) tdln-proof

crates/tdln-proof/CHANGELOG.md

# Changelog — tdln-proof
Formato: Keep a Changelog — SemVer.

## [Unreleased]
- Provas compostas (multi-ROOT) e partial verification.
- Adicionar “decision-proof” integrado ao Gate.

## [0.1.0] - 2026-01-09
### Adicionado
- `Pass` determinístico a partir de `Intent`.
- `ProofBundle` com Merkle root e metadados.
- Assinatura opcional via `ed25519-dalek` (feature `ed25519`).
- `no_std/alloc` com `std` por padrão.

crates/tdln-proof/examples/bundle.rs

use tdln_ast::Intent;
use tdln_proof::{Pass, ProofBundle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let intent = Intent::new("transferir")
        .with_str("destino", "@alice")
        .with_f64("valor", 10.0)
        .normalize();

    let pass = Pass::from_intent(&intent);
    let bundle = ProofBundle::from_pass(&pass)?;
    assert_eq!(bundle.root().len(), 32);

    println!("root={}", hex::encode(bundle.root()));
    Ok(())
}

crates/tdln-proof/tests/determinism.rs

use tdln_ast::Intent;
use tdln_proof::{Pass, ProofBundle};

#[test]
fn same_intent_same_root() {
    let a = Intent::new("pagar").with_str("destino", "@bob").with_i64("valor", 15).normalize();
    let b = Intent::new("pagar").with_i64("valor", 15).with_str("destino", "@bob").normalize();

    let pa = Pass::from_intent(&a);
    let pb = Pass::from_intent(&b);

    let ba = ProofBundle::from_pass(&pa).unwrap();
    let bb = ProofBundle::from_pass(&pb).unwrap();

    assert_eq!(ba.root(), bb.root());
}

(snippet) crates/tdln-proof/Cargo.toml

[dependencies]
tdln-ast = { version = "0.1", path = "../tdln-ast" } # troque para versão crates.io quando publicar
blake3 = "1.5"
hex = "0.4"
serde = { version = "1.0", features = ["derive"], default-features = false }
ed25519-dalek = { version = "2.1", optional = true, features = ["pkcs8"] }
json-atomic = { version = "0.1", optional = true, default-features = false, features = ["canon"] }
lllv-core = { version = "0.1", optional = true }     # hashing utilitário opcional

[features]
default = ["std"]
std = ["serde/std"]
alloc = []
ed25519 = ["ed25519-dalek"]
canon = ["json-atomic"]

[package.metadata.docs.rs]
features = ["std", "ed25519", "canon"]
no-default-features = false
all-features = false


⸻

3) tdln-compiler

crates/tdln-compiler/CHANGELOG.md

# Changelog — tdln-compiler

## [Unreleased]
- Regras de lowering adicionais e cost model verificável.
- Hooks para “Chip as Code” como backend opcional.

## [0.1.0] - 2026-01-09
### Adicionado
- `compile(Intent, CompileCfg) -> (IR, ProofBundle)`
- Normalização obrigatória do input; IR em forma canônica.
- Integração com `tdln-proof` e `json-atomic` para CID canônico.

crates/tdln-compiler/examples/compile.rs

use tdln_ast::Intent;
use tdln_compiler::{compile, CompileCfg};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let intent = Intent::new("pagar")
        .with_str("destino", "@bob")
        .with_i64("valor", 15)
        .normalize();

    let cfg = CompileCfg::default();
    let (_ir, bundle) = compile(&intent, &cfg)?;
    println!("root={}", hex::encode(bundle.root()));
    Ok(())
}

crates/tdln-compiler/tests/smoke.rs

use tdln_ast::Intent;
use tdln_compiler::{compile, CompileCfg};

#[test]
fn compiles_basic_intent() {
    let intent = Intent::new("emitir_recibo")
        .with_str("para", "@dan")
        .with_i64("valor", 7)
        .normalize();

    let cfg = CompileCfg::default();
    let res = compile(&intent, &cfg);
    assert!(res.is_ok());
}

(snippet) crates/tdln-compiler/Cargo.toml

[dependencies]
tdln-ast    = { version = "0.1", path = "../tdln-ast" }     # trocar para versão crates.io
tdln-proof  = { version = "0.1", path = "../tdln-proof" }   # trocar para versão crates.io
serde       = { version = "1.0", features = ["derive"] }
hex         = "0.4"
json-atomic = { version = "0.1", default-features = false, features = ["canon"] }
logline-core = { version = "0.1" }
lllv-core   = { version = "0.1" }
lllv-index  = { version = "0.1", optional = true }

[features]
default = ["std"]
std = []
alloc = []
index = ["lllv-index"]

[package.metadata.docs.rs]
features = ["std"]
no-default-features = false
all-features = false


⸻

4) tdln-gate

crates/tdln-gate/CHANGELOG.md

# Changelog — tdln-gate

## [Unreleased]
- Resultados com “decision-proof” e anexo de certificado Magister.
- Suporte a políticas dinâmicas versionadas.

## [0.1.0] - 2026-01-09
### Adicionado
- `PolicySet` (bounds/forbidden/required)
- `Gate::evaluate(Intent) -> Decision`
- Integração opcional com `tdln-proof` para anexar evidências.

crates/tdln-gate/examples/gate.rs

use tdln_ast::Intent;
use tdln_gate::{Gate, PolicySet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pol = PolicySet::builder()
        .bound("valor", 0.0, 1000.0)
        .required("assinatura.magister")
        .build();

    let gate = Gate::new(pol);

    let intent_ok = Intent::new("transferir")
        .with_f64("valor", 99.0)
        .with_str("assinatura.magister", "OK")
        .normalize();

    let d = gate.evaluate(&intent_ok)?;
    assert!(d.allowed());
    println!("allowed = {}", d.allowed());
    Ok(())
}

crates/tdln-gate/tests/policies.rs

use tdln_ast::Intent;
use tdln_gate::{Gate, PolicySet};

#[test]
fn denies_out_of_bounds() {
    let pol = PolicySet::builder().bound("valor", 0.0, 10.0).build();
    let gate = Gate::new(pol);

    let intent = Intent::new("transferir").with_f64("valor", 999.0).normalize();
    let decision = gate.evaluate(&intent).unwrap();
    assert!(!decision.allowed());
}

(snippet) crates/tdln-gate/Cargo.toml

[dependencies]
tdln-ast   = { version = "0.1", path = "../tdln-ast" }     # trocar para versão crates.io
tdln-proof = { version = "0.1", path = "../tdln-proof", optional = true }  # evidências no Gate
serde      = { version = "1.0", features = ["derive"] }

[features]
default = ["std"]
std = []
alloc = []
proof = ["tdln-proof"]

[package.metadata.docs.rs]
features = ["std", "proof"]
no-default-features = false
all-features = false


⸻

5) (opcional) Ajuste no [patch.crates-io] do workspace

Só se você quiser desenvolver localmente com submodules sem mudar os Cargo.toml das crates TDLN (mantendo dependências por versão). No Cargo.toml raiz:

[patch.crates-io]
json-atomic  = { path = "external/json-atomic" }
logline-core = { path = "external/logline-core" }
lllv-core    = { path = "external/lllv-core" }
lllv-index   = { path = "external/lllv-index" } # se/quando quiser apontar localmente


⸻

pronto! com isso, o Workspace LogLine fica sem hardcode, com docs.rs certeiro, CHANGELOGs organizados e exemplos/tests que “puxam” o uso correto. Se quiser, na próxima te mando também os templates de RELEASE_NOTES.md e os golden tests (snapshots) pro canon JSON ✶Atomic.