**Master Plan do LogLine Workspace (v2 — detalhado de verdade)**, já pronto pra colar no repositório como `WORKSPACE_MASTER_PLAN.md`. Eu incluí estrutura, comandos, YAMLs de CI, scripts, convenções de versionamento/publish e a topologia fina entre as crates atuais e as próximas (TDLN + Chip as Code).

---

# 🚀 LogLine Workspace — Master Plan (v2, detalhado)

## 0) Objetivos (claros e mensuráveis)

* **Unificar** build, testes, docs e publicação do ecossistema LogLine.
* **Parar de recomeçar**: invariantes + gates automatizados para todas as crates.
* **Destravar roadmap**: preparar terreno para **TDLN** e **Chip as Code** sem quebrar nada já publicado.
* **Releases previsíveis**: tagging por crate, trusted publishing, ordem topológica.

**KPIs (Proof-of-Done):**

* `cargo test --workspace` ✅
* `cargo audit` + `cargo deny` (matriz por crate) ✅
* `cargo publish --dry-run` por crate ✅
* `docs.rs` local (`cargo doc --no-deps`) sem erros ✅
* CI verde (fmt, clippy, tests, docs, examples) ✅

---

## 1) Topologia do repositório (monorepo de integração)

**Novo repo GitHub:** `LogLine-Foundation/logline-workspace`

```
logline-workspace/
├─ Cargo.toml                    # workspace virtual
├─ rust-toolchain.toml           # pin do toolchain (stable)
├─ .editorconfig                 # higiene de formatação
├─ .github/workflows/            # CI, audit, deny, sbom, publish
├─ scripts/                      # verificadores e helpers
│  ├─ verify_all_crates.sh
│  ├─ verify_dependencies.sh
│  ├─ verify_quality.sh
│  └─ check_tag_matches_version.sh
├─ external/                     # submodules: crates já existentes
│  ├─ logline-core/              # (repo oficial)
│  ├─ json-atomic/               # (repo oficial)
│  ├─ lllv-core/                 # (repo oficial)
│  └─ lllv-index/                # (repo oficial)
└─ crates/                       # novas crates (TDLN + Chip as Code)
   ├─ tdln-ast/
   ├─ tdln-proof/
   ├─ tdln-compiler/
   ├─ tdln-gate/
   ├─ chip-core/
   ├─ chip-serde/
   ├─ chip-exec/
   └─ chip-ledger/
```

> **Por quê submodules?** Mantêm cada crate **first-class** (issues, releases, badges), mas a gente ganha `cargo test --workspace` e integração unificada. (Se preferir `git subtree`, processo é parecido — escolhemos submodule pela leveza e pinagem por tag.)

### 1.1 Bootstrap (git)

```bash
git init logline-workspace && cd logline-workspace

git submodule add https://github.com/LogLine-Foundation/logline-core external/logline-core
git submodule add https://github.com/LogLine-Foundation/json-atomic external/json-atomic
git submodule add https://github.com/LogLine-Foundation/lllv-core external/lllv-core
git submodule add https://github.com/LogLine-Foundation/lllv-index external/lllv-index

# Fixar submodules em tags estáveis (exemplos)
(cd external/logline-core && git checkout v0.1.0)
(cd external/json-atomic  && git checkout v0.1.0)
(cd external/lllv-core    && git checkout v0.1.0)
(cd external/lllv-index   && git checkout v0.1.0)

git add -A && git commit -m "workspace: add 4 core crates as submodules (pinned)"
```

---

## 2) Root `Cargo.toml` (workspace virtual)

```toml
[workspace]
members = [
  "external/logline-core",
  "external/json-atomic",
  "external/lllv-core",
  "external/lllv-index",
  "crates/tdln-ast",
  "crates/tdln-proof",
  "crates/tdln-compiler",
  "crates/tdln-gate",
  "crates/chip-core",
  "crates/chip-serde",
  "crates/chip-exec",
  "crates/chip-ledger",
]
resolver = "2"

[workspace.package]
edition      = "2021"
rust-version = "1.75"
license      = "MIT"
homepage     = "https://logline.foundation"
repository   = "https://github.com/LogLine-Foundation/logline-workspace"
documentation= "https://docs.rs"

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.dependencies]
blake3  = "1.5"
hex     = "0.4"
serde   = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror  = "1.0"
ed25519-dalek = { version = "2.1", features = ["pkcs8"] }

# Desenvolvimento integrado: as crates locais substituem as do crates.io
[patch.crates-io]
logline-core = { path = "external/logline-core" }
json-atomic  = { path = "external/json-atomic" }
lllv-core    = { path = "external/lllv-core" }
lllv-index   = { path = "external/lllv-index" }
```

### 2.1 `rust-toolchain.toml` (pin do toolchain)

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

### 2.2 `.editorconfig` (higiene)

```
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
indent_style = space
indent_size = 4
trim_trailing_whitespace = true
```

---

## 3) Topologia de dependências (camadas & invariantes)

**Camadas (de baixo pra cima):**

1. **logline-core** (base)
2. **json-atomic** → depende de `logline-core`
3. **lllv-core** → depende de `json-atomic` (+ `logline-core` se preciso)
4. **lllv-index** → depende de `lllv-core` (+ `json-atomic`)

**TDLN (nova família):**

* `tdln-ast` (núcleo sem deps locais)
* `tdln-proof` → pode depender de `json-atomic` (provas em JSON Atomic)
* `tdln-compiler` → depende de `tdln-ast` (+ `tdln-proof` opcional)
* `tdln-gate` → depende de `tdln-compiler` (+ `lllv-core` para verificações opcionais)

**Chip as Code (nova família):**

* `chip-core` (núcleo e primitivas)
* `chip-serde` → pode depender de `serde`/`json-atomic`
* `chip-exec`  → depende de `chip-core`
* `chip-ledger`→ depende de `chip-serde` (+ `json-atomic`)

**Proibições (gates):**

* ❌ Ciclos entre crates
* ❌ “Furar a pirâmide” (ex.: `lllv-index` depender de `logline-core` **sem** precisar)
* ❌ `path`/`git` deps em versões publicadas
* ❌ Versões wildcard (`*`)
* ✅ **Fail se violar**: `scripts/verify_dependencies.sh`

---

## 4) Padrão de Qualidade (aplicado ao workspace inteiro)

* `#![forbid(unsafe_code)]` nas libs públicas (ou justificar explicitamente no README/SECURITY).
* Zero `unwrap`/`expect`/`panic!` em **APIs públicas** (inputs não confiáveis → `Result<_, Error>`).
* `thiserror` p/ erros tipados; **não reduzir granularidade** em patch.
* Testes:

  * Unitários + Integração (≥2 arquivos em `tests/`)
  * `doc-tests` (`cargo test --doc`) e `examples` (≥1, com `cargo test --examples`)
  * Casos de ataque (árvore/cripto/parse): tamanhos ímpares, hex malformado, AAD incorreta, índices fora do range, etc.
* Docs:

  * README por crate (badges, quickstart, Segurança)
  * `[package.metadata.docs.rs]` consistente
  * `#![doc = include_str!("../README.md")]` em `lib.rs` (opcional, recomendado)
* Segurança & SC:

  * `cargo audit`, `cargo deny`, SBOM CycloneDX em releases
  * `exclude` correto p/ empacotar
  * Sem segredos / binários gerados / dumps no repo

---

## 5) CI/CD (workflows prontos)

### 5.1 `ci.yml` (fmt, clippy, tests, docs, examples; matriz por crate)

```yaml
name: CI (workspace)
on:
  push: { branches: ["main"] }
  pull_request: {}

jobs:
  matrix-check:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        crate:
          - external/logline-core
          - external/json-atomic
          - external/lllv-core
          - external/lllv-index
          - crates/tdln-ast
          - crates/tdln-proof
          - crates/tdln-compiler
          - crates/tdln-gate
          - crates/chip-core
          - crates/chip-serde
          - crates/chip-exec
          - crates/chip-ledger
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy --manifest-path ${{ matrix.crate }}/Cargo.toml --all-targets --all-features -- -D warnings
      - run: cargo test   --manifest-path ${{ matrix.crate }}/Cargo.toml --all-features
      - run: cargo test   --manifest-path ${{ matrix.crate }}/Cargo.toml --doc
      - run: cargo test   --manifest-path ${{ matrix.crate }}/Cargo.toml --examples

  workspace-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable
      - run: bash scripts/verify_dependencies.sh
      - run: bash scripts/verify_all_crates.sh
```

### 5.2 `audit.yml` (RustSec)

```yaml
name: Security Audit
on:
  pull_request: {}
  push: {}
  schedule: [{ cron: "0 5 * * 1" }]
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

### 5.3 `deny.yml` (licenças/advisories)

```yaml
name: License/Advisory Deny
on:
  pull_request: {}
  push: {}
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

### 5.4 `sbom.yml` (CycloneDX em releases)

```yaml
name: SBOM
on:
  release: { types: [published] }
jobs:
  sbom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive }
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-cyclonedx
      - run: cargo cyclonedx -o sbom.json
      - uses: softprops/action-gh-release@v2
        with: { files: sbom.json }
```

### 5.5 `publish.yml` (Trusted Publishing por **tag de crate**)

> Política: **não** publicar o workspace inteiro; publicar **por crate** com tag padrão `crate-name-vX.Y.Z`.

```yaml
name: Publish crate
on:
  push:
    tags: ['*-v*.*.*']   # ex.: tdln-ast-v0.1.0, lllv-index-v0.2.1

jobs:
  publish:
    runs-on: ubuntu-latest
    permissions:
      id-token: write  # OIDC para crates.io trusted publishing
      contents: read
    steps:
      - uses: actions/checkout@v4
        with: { submodules: recursive, fetch-depth: 0 }

      - name: Extract crate & version from tag
        id: x
        run: |
          TAG="${GITHUB_REF_NAME}"            # ex.: tdln-ast-v0.1.0
          CRATE="${TAG%-v*}"                  # tdln-ast
          VERSION="${TAG#*-v}"                # 0.1.0
          echo "crate=$CRATE"   >> $GITHUB_OUTPUT
          echo "version=$VERSION" >> $GITHUB_OUTPUT

      - uses: rust-lang/crates-io-auth-action@v1
        id: auth

      - uses: dtolnay/rust-toolchain@stable

      - name: Verify tag matches Cargo.toml version
        run: bash scripts/check_tag_matches_version.sh "${{ steps.x.outputs.crate }}" "${{ steps.x.outputs.version }}"

      - name: Dry-run
        run: cargo publish -p ${{ steps.x.outputs.crate }} --dry-run
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}

      - name: Publish
        run: cargo publish -p ${{ steps.x.outputs.crate }}
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}
```

---

## 6) Scripts (gates operacionais)

### 6.1 `scripts/verify_all_crates.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "== FMT =="
cargo fmt --all -- --check

echo "== CLIPPY =="
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "== TESTS =="
cargo test --workspace --all-features
cargo test --workspace --doc
cargo test --workspace --examples

echo "== PACKAGE DRY LIST (each crate) =="
for mf in $(cargo metadata --no-deps --format-version=1 | jq -r '.packages[].manifest_path'); do
  echo "-> $(dirname "$mf")"
  cargo package --manifest-path "$mf" --list > /dev/null
done

echo "OK"
```

### 6.2 `scripts/verify_dependencies.sh` (camadas & proibições)

```bash
#!/usr/bin/env bash
set -euo pipefail

# Camadas do ecossistema (hard-coded; ajuste conforme adicionar crates novas)
L1="logline-core"
L2="json-atomic"
L3="lllv-core"
L4="lllv-index"
T1="tdln-ast"
T2="tdln-proof"
T3="tdln-compiler"
T4="tdln-gate"
C1="chip-core"
C2="chip-serde"
C3="chip-exec"
C4="chip-ledger"

# ordem válida (base -> topo)
ORDER=($L1 $L2 $L3 $L4 $T1 $T2 $T3 $T4 $C1 $C2 $C3 $C4)

# mapa de posições
declare -A POS
i=0; for c in "${ORDER[@]}"; do POS["$c"]=$i; i=$((i+1)); done

# extrai deps do cargo metadata
MD=$(cargo metadata --format-version=1)
pkgs=$(jq -r '.packages[] | @base64' <<<"$MD")

fail=0
while IFS= read -r p; do
  pkg=$(echo "$p" | base64 -d)
  name=$(jq -r '.name' <<<"$pkg")
  # deps diretas normais (ignorar build/dev)
  deps=$(jq -r '.dependencies[] | select(.kind == null or .kind == "normal") | .name' <<<"$pkg")
  for d in $deps; do
    [[ -n "${POS[$name]:-}" && -n "${POS[$d]:-}" ]] || continue
    if [[ ${POS[$d]} -gt ${POS[$name]} ]]; then
      echo "⛔ Camada inválida: '$name' (nível ${POS[$name]}) depende de '$d' (nível ${POS[$d]})"
      fail=1
    fi
  done
done <<< "$pkgs"

# proibições básicas
# 1) wildcards
if grep -R --include "Cargo.toml" -n 'version = "\\*"' external crates; then
  echo "⛔ Dependência com wildcard (*) detectada"
  fail=1
fi

# 2) path/git em crates publicadas (checado em publish; aqui só alerta)
if grep -R --include "Cargo.toml" -n 'git = ' external crates; then
  echo "⚠️  Dependência git encontrada (ok em dev, proibir ao publicar)"
fi

exit $fail
```

### 6.3 `scripts/verify_quality.sh` (checklist por crate)

```bash
#!/usr/bin/env bash
set -euo pipefail
CRATE_DIR="${1:?usage: $0 <path-to-crate>}"

echo "== Checking $CRATE_DIR =="
cargo fmt --manifest-path "$CRATE_DIR/Cargo.toml" --all -- --check
cargo clippy --manifest-path "$CRATE_DIR/Cargo.toml" --all-targets --all-features -- -D warnings
cargo test   --manifest-path "$CRATE_DIR/Cargo.toml" --all-features
cargo test   --manifest-path "$CRATE_DIR/Cargo.toml" --doc
cargo test   --manifest-path "$CRATE_DIR/Cargo.toml" --examples
cargo deny   check all || true
cargo audit || true
cargo package --manifest-path "$CRATE_DIR/Cargo.toml" --list > /dev/null
echo "OK"
```

### 6.4 `scripts/check_tag_matches_version.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail
CRATENAME="${1:?crate-name}"
EXPECT="${2:?version}"

# encontra manifest por nome do package
MF=$(cargo metadata --no-deps --format-version=1 | jq -r \
  --arg n "$CRATENAME" '.packages[] | select(.name==$n) | .manifest_path' | head -n1)

test -n "$MF" || { echo "crate not found: $CRATENAME"; exit 1; }

FOUND=$(tomlq -f "$MF" -r '.package.version' 2>/dev/null || true)
if [ -z "$FOUND" ]; then
  FOUND=$(grep -E '^\s*version\s*=\s*"[0-9]+\.[0-9]+\.[0-9]+"' -m1 "$MF" | sed -E 's/.*"([^"]+)".*/\1/')
fi

echo "Tag version: $EXPECT ; Cargo.toml version: $FOUND"
test "$FOUND" = "$EXPECT" || { echo "⛔ version mismatch"; exit 1; }
```

> Nota: `tomlq` é opcional (do `yq` moderno). O fallback via `grep` já cobre.

---

## 7) Políticas de versão, MSRV e publicação

* **SemVer**: `MAJOR.MINOR.PATCH`

  * PATCH = correções compatíveis
  * MINOR = novas APIs compatíveis (inclusive bump de MSRV, com nota)
  * MAJOR = breaking (com guia de migração)
* **MSRV**: `1.75` (suporte por ≥ 6 meses)
* **Tagging por crate**: `crate-name-vX.Y.Z`
  Ex.: `tdln-compiler-v0.1.0`, `lllv-index-v0.2.1`
* **Trusted Publishing** (OIDC) com workflow `publish.yml`
* **Ordem de publicação topo-lógica (se houver dependência)**:

  1. `logline-core`
  2. `json-atomic`
  3. `lllv-core`
  4. `lllv-index`
  5. família **TDLN** (ast → proof → compiler → gate)
  6. família **Chip** (core → serde → exec → ledger)

---

## 8) Metadados padronizados por crate

**Cargo.toml (delta mínimo):**

```toml
[package]
name = "crate-name"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
readme = "README.md"
description = "Descrição concisa"
keywords = ["logline","crypto","proofs"]   # ajustar
categories = ["cryptography","encoding"]   # do crates.io
documentation = "https://docs.rs/crate-name"
exclude = [".github/**","deny.toml","SECURITY.md","CODE_OF_CONDUCT.md","CHANGELOG.md"]

[features]
default = ["std"]
std = []
alloc = []  # preparar para no_std

[package.metadata.docs.rs]
features = ["std"]
no-default-features = false
all-features = false
```

**lib.rs cabeçalho recomendado:**

```rust
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
```

**README (mínimo):** badges (crates, docs, CI, MSRV, license), instalação, quickstart, API principal, segurança (ex.: `verify_cid()` vs `verify_with()`), licença.

---

## 9) Roadmap por ondas (com entregáveis)

### Wave 0 — **Import & Baseline** (D0–D2)

* [ ] Criar repo `logline-workspace`
* [ ] Adicionar 4 crates como submodules (pinados)
* [ ] Adicionar root `Cargo.toml`, `rust-toolchain.toml`, `.editorconfig`
* [ ] Subir `ci.yml`, `audit.yml`, `deny.yml`, `sbom.yml`, `publish.yml`
* [ ] Scripts em `scripts/` (4 acima)
* **DoD:** `cargo test --workspace` ok; CI verde

### Wave 1 — **Qualidade & Docs** (D3–D5)

* [ ] READMEs de cada crate com badges padronizados
* [ ] ISSUE/PR templates + CODEOWNERS
* [ ] `deny.toml` padronizado; `SECURITY.md`; `CODE_OF_CONDUCT.md`
* **DoD:** `cargo deny check all` e `cargo audit` sem blockers

### Wave 2 — **TDLN (v0.1.0)** (D6–D12)

* [ ] Criar `tdln-ast`, `tdln-proof`, `tdln-compiler`, `tdln-gate`
* [ ] APIs mínimas + 1 exemplo por crate + 2 testes de integração
* [ ] Tag por crate: `tdln-ast-v0.1.0`, etc. (publicação confiável)
* **DoD:** docs.rs ok, SBOM anexado no release

### Wave 3 — **Chip as Code (v0.1.0)** (D13–D18)

* [ ] Criar `chip-core`, `chip-serde`, `chip-exec`, `chip-ledger`
* [ ] Integração opcional com `json-atomic`
* [ ] Publicar via tags por crate
* **DoD:** exemplos rodando, docs.rs ok

---

## 10) Comandos úteis (dev loop)

**Rodar tudo (local):**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test  --workspace --all-features
cargo test  --workspace --doc
cargo test  --workspace --examples
bash scripts/verify_dependencies.sh
```

**Dry-run publishing por crate:**

```bash
for c in external/logline-core external/json-atomic external/lllv-core external/lllv-index; do
  echo "== $c ==";
  cargo publish --manifest-path "$c/Cargo.toml" --dry-run;
done
```

**Tag & publish (exemplo TDLN):**

```bash
git tag tdln-ast-v0.1.0 && git push origin tdln-ast-v0.1.0
# workflow publish.yml valida versão e publica
```

---

## 11) Riscos & mitigação

* **Divergência submodule vs origem**
  → Sempre pin por tag; mudanças via PR no repo de origem.
* **Publish acidental do workspace**
  → Só dispara por tag `{crate}-v*.*.*`, e `publish.yml` usa `-p <crate>`.
* **Ciclos / violações de camada**
  → Gate em `verify_dependencies.sh` (+ revisão em PR).
* **Quebra docs.rs**
  → `cargo doc --no-deps` roda no CI; doc-tests obrigatórios.

---

## 12) Anexos — Templates rápidos

### 12.1 `deny.toml` (sugestão)

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
ignore = []

[licenses]
allow = ["MIT","Apache-2.0","BSD-3-Clause","ISC","Unicode-DFS-2016","Zlib","CC0-1.0"]
copyleft = "warn"
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "deny"
```

### 12.2 `SECURITY.md` (mini)

```md
# Security Policy

Reporte vulnerabilidades por issue privada ou e-mail da organização.
Evite PoCs destrutivas.
Releases incluem cargo-audit, cargo-deny e SBOM (CycloneDX).
```

### 12.3 `CODEOWNERS` (exemplo)

```
/external/logline-core/   @dcamarilho
/external/json-atomic/    @dcamarilho
/external/lllv-core/      @dcamarilho
/external/lllv-index/     @dcamarilho
/crates/tdln-*            @dcamarilho
/crates/chip-*            @dcamarilho
```

---

## 13) Encerramento (estado-alvo)

* Workspace padronizado, com **gates automatizados**.
* Quatro crates atuais integradas **sem** mudar o fluxo de publicação delas.
* **TDLN** e **Chip as Code** prontos para nascer em **ondas**, versionados, com docs, exemplos, testes e supply-chain hardening desde o dia 1.
* **Zero** “começar do zero”: a base tá fixa; o topo pode ousar. 💪

---

se curtir, eu já te entrego **os arquivos prontos** (root `Cargo.toml`, 5 workflows, 4 scripts). Só colar no repo e dar o primeiro push com os submodules que isso liga o CI e valida tudo 🔧✨
