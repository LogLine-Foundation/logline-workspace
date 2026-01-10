amo esse pedido, Dan — bora dar o “cérebro” do TDLN pro teu agente. Abaixo está o **esqueleto funcional e as invariantes de negócio** pra ele gerar as crates certas, com APIs públicas, tipos, fluxo de execução, prova criptográfica e integração com o Workspace. É plug-and-play com `logline-core`, `json-atomic`, `lllv-core`, `lllv-index`.

---

# TDLN — Business Logic do Workspace (v1)

## Visão em 20s

* **TDLN** = pipeline determinístico NL → **Logical Atom** (estrutura canônica), com **prova criptográfica** de cada passo.
* **Gate** = verificador de políticas de altíssima velocidade (HFT-style): aceita/nega/desafia uma intenção **só** se o átomo e a prova passarem.
* **JSON✯Atomic** = formato canônico e assinável de tudo (entrada, átomo, passos, hashes, assinatura).
* **LLLV** = árvore de evidências e indexação (Merkle) para consulta, auditoria e replay.

---

## Crates (mantemos 4, com módulos internos)

```
tdln-ast/        # gramática, AST inicial, normalização canônica
tdln-proof/      # objetos de prova, hashing BLAKE3, trilha de reescrita, Merkle
tdln-compiler/   # fases NL->AST->IR->Atom, checagens de tipo/constraints, emissão
tdln-gate/       # Policy Engine (HFT Gate): Permit | Deny | Challenge, latency budget
```

> Cada crate expõe **APIs estáveis**, sem dependência circular. `tdln-gate` depende de `tdln-compiler` e `tdln-proof`. `tdln-compiler` depende de `tdln-ast` e `tdln-proof`. Todas usam `json-atomic` para canônicos/assinaturas e integram com `lllv-*` via traits leves (sem acoplamento pesado).

---

## Invariantes (não negociar)

1. **Determinismo**: mesma entrada + mesmo contexto ⇒ mesmo átomo + mesma prova (hash idêntico).
2. **Canon**: toda estrutura serializada via **JSON✯Atomic** canônico (ordem de campos, tipos explícitos).
3. **Hash-first**: cada fase emite `pre_hash` → `post_hash` (BLAKE3) e registra a **regra aplicada**.
4. **Sem `unsafe`**: `#![forbid(unsafe_code)]`.
5. **Sem panics** na API pública: erros via `Result<_, TdlnError>`.
6. **No_std ready** (`alloc`) e **WASM**-compat em `tdln-ast`, `tdln-proof`, `tdln-compiler` (gate pode ter `std` por clock/telemetria).
7. **Tempo HFT Gate**: decisão `O(ms baixos)` com zero I/O síncrono (políticas puras + caches imutáveis).

---

## Tipos essenciais (compartilhados)

```rust
// tdln-types (módulo interno reexportado)
pub type Hash32 = [u8; 32];          // BLAKE3
pub type CanonJson = Vec<u8>;         // JSON✯Atomic (bytes canônicos)
pub type SpanId = String;             // id NDJSON/ledger (se usado)
pub type PolicyId = String;
pub type ModelId = String;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct IntentionNL {
    pub ts_ms: u64,
    pub locale: String,        // ex: "pt-BR"
    pub text: String,          // entrada natural
    pub context: serde_json::Value, // hints/slots (opcional, canônico antes da fase 1)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LogicalAtom {
    // átomo mínimo porém expressivo (intent + slots + constraints canônicas)
    pub kind: String,            // ex: "transfer", "fetch", "schedule", etc.
    pub slots: serde_json::Value, // ex: { "amount": "100", "to":"acct:123" }
    pub constraints: serde_json::Value, // ex: { "currency":"USD", "limits":{...} }
    pub version: u32,            // schema version do átomo
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TdlnProofBundle {
    pub model: ModelId,           // quem traduziu (modelo/versão)
    pub policy: PolicyId,         // política usada no gate (id canônico)
    pub steps_root: Hash32,       // Merkle root dos steps
    pub atom_hash: Hash32,        // hash canônico do LogicalAtom
    pub trace_hash: Hash32,       // hash de todo o pipeline (RX hash)
    pub signature: Option<Vec<u8>>, // Ed25519 opcional (assinado pelo gateway)
}
```

---

## `tdln-ast` (gramática + normalização)

**Responsabilidade**: tokenizar/parsear o NL para uma **AST** simples e estável; normalizar **datas, números, unidades**, entidades evidentes; **não resolve** políticas.

### API

```rust
pub struct Ast;
pub struct AstNormalized; // datas/valores em forma canônica

pub fn tokenize(nl: &IntentionNL) -> Result<Vec<Token>, TdlnError>;
pub fn parse(tokens: &[Token]) -> Result<Ast, TdlnError>;
pub fn normalize(ast: &Ast) -> Result<AstNormalized, TdlnError>;

// Hash/canon helpers
pub fn canon_json<T: serde::Serialize>(t: &T) -> Result<CanonJson, TdlnError>;
pub fn blake3_bytes(parts: &[&[u8]]) -> Hash32;
```

**Regras de normalização (exemplos):**

* `“amanhã 3pm”` → `UTC ts` + `tz`.
* `“cem dólares”` → `{ "amount":"100.00", "currency":"USD" }`.
* `“pra João”` → `entity:contact:joao` (se vier no `context`).

**Proof step (emite em `tdln-proof`)**: `Rule("normalize_datetime:v1")`, `Rule("normalize_amount:v1")`…

---

## `tdln-proof` (trilha, hash, Merkle, assinatura)

**Responsabilidade**: modelo de **passos de prova** e composição em Merkle tree, **trace hash** e **assinatura** opcional.

### API

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProofStep {
    pub rule_id: String,       // ex: "normalize_amount:v1"
    pub input_hash: Hash32,    // BLAKE3 do JSON canônico de entrada do passo
    pub output_hash: Hash32,   // BLAKE3 do JSON canônico de saída
    pub aux: serde_json::Value // metadados (ex: locale, tz)
}

pub fn steps_merkle_root(steps: &[ProofStep]) -> Hash32;

// pacote de prova completo (usado no bundle)
pub fn build_proof_bundle(
    model: &ModelId,
    policy: &PolicyId,
    steps: &[ProofStep],
    atom_canon: &CanonJson,
    priv_key: Option<&ed25519_dalek::SigningKey>,
) -> Result<TdlnProofBundle, TdlnError>;

pub fn verify_bundle(
    bundle: &TdlnProofBundle,
    steps: &[ProofStep],
    atom_canon: &CanonJson,
    pub_key: Option<&ed25519_dalek::VerifyingKey>,
) -> Result<(), TdlnError>;
```

**Invariantes**:

* `steps_root = Merkle(ProofStep)` com **ordem estável**.
* `atom_hash = blake3(atom_canon)`.
* `trace_hash = blake3([steps_root || atom_hash])`.
* Se `signature` existir → Ed25519 sobre `trace_hash` (domínio: `"tdln_trace_v1"` prefix).

---

## `tdln-compiler` (fases NL→Atom com checagens)

**Responsabilidade**: pipeline **determinístico**:

1. `tokenize` → 2) `parse` → 3) `normalize` → 4) `infer_kind/slots` → 5) `apply_constraints` → 6) **emit LogicalAtom**.

Cada etapa **emite `ProofStep`** com `rule_id` e os hashes `input/output`.

### API

```rust
pub struct CompileConfig {
    pub model: ModelId,       // id do tradutor (p.ex. “tdln-static:v1” ou “gpt-xyz@prompt:v3”)
    pub locale: String,
    pub strict: bool,         // true = erro em ambiguidade
}

pub struct CompileOutput {
    pub atom: LogicalAtom,
    pub atom_canon: CanonJson,
    pub steps: Vec<ProofStep>,
    pub bundle: TdlnProofBundle,
}

pub fn compile(
    input: &IntentionNL,
    cfg: &CompileConfig,
    sign_with: Option<&ed25519_dalek::SigningKey>,
    policy_id_hint: Option<&str>, // opcional: já preencher no bundle
) -> Result<CompileOutput, TdlnError>;
```

### Regras de negócio (exemplos de `infer_kind/slots`)

* Se contém `“transferir”|“enviar”` + `amount` + `destinatário` → `kind="transfer"`.
* Se contém `“buscar”|“get”` + `resource` → `kind="fetch"`.
* Ambiguidade com `strict=true` ⇒ `Err(TdlnError::Ambiguous(...))` com **caminhos alternativos** (opcional retornar `Challenge` no gate).

### Constraints (normalize → constraints)

* Limites (ex.: amount ≤ policy.limit).
* Tipagem/min-max/regex por slot.
* **Nunca** faz I/O; lookup vem do `context` canônico entregue pelo chamador.

---

## `tdln-gate` (HFT Gate — Policy Engine)

**Responsabilidade**: dado **IntentionNL** + **Policy** determinística + **ModelId**, decide:

* `Permit(Atom, Evidence)` se **prova e constraints** ok,
* `Deny(reason)` se viola,
* `Challenge(question)` se ambíguo (com **hint** do que falta).

### Modelo de Política (puro, serializável)

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GatePolicy {
    pub id: PolicyId,
    pub version: u32,
    pub allowed_kinds: Vec<String>,           // ["transfer","fetch",...]
    pub slot_bounds: serde_json::Value,       // ex: { "amount": { "max":"1000.00","currency":"USD" } }
    pub forbidden: serde_json::Value,         // ex: vendor ids, targets
    pub constraints: serde_json::Value,       // regras extras específicas
    pub decision_budget_ms: u32,              // p99 alvo (só para telemetria)
}
```

### API

```rust
pub enum GateDecision {
    Permit { atom: LogicalAtom, evidence: TdlnProofBundle },
    Deny { reason: String, code: &'static str },
    Challenge { missing: Vec<String>, message: String },
}

pub struct GateContext<'a> {
    pub policy: &'a GatePolicy,
    pub model: &'a ModelId,
    pub pub_key: Option<&'a ed25519_dalek::VerifyingKey>, // se exigir assinatura do bundle
}

pub fn decide(
    nl: &IntentionNL,
    cfg: &CompileConfig,
    gate: &GateContext,
) -> Result<GateDecision, TdlnError>;
```

**Fluxo (determinístico, sem I/O):**

1. `compile(nl, cfg, sign_with=None, policy_id_hint=Some(policy.id))`
2. `verify_bundle(bundle, steps, atom_canon, pub_key)` (se `pub_key` presente)
3. `enforce(policy, atom)`:

   * `kind ∈ allowed_kinds`
   * `slots` obedecem `slot_bounds` e `constraints`
4. **Retorno**: `Permit` com `evidence=bundle` ou `Deny/Challenge`.

**Latency**: tudo é local (pure), sem rede. O **HFT Gate** fica livre para orquestrar *N* políticas/micro-regras numa shot só.

---

## Integrações do Workspace

### JSON✯Atomic (canon e spans)

* Todas as estruturas de entrada/saída **serializadas** com o codec **canônico** do `json-atomic`.
* Opcional: emitir **spans NDJSON**:

  * `register_intention`
  * `tdln_ast_normalized`
  * `tdln_compiled_atom`
  * `tdln_proof_bundle`
  * `gate_decision`
* Cada span leva `hash`, `prev`, `ts_ms`, `tenant`, `sign`.

### LLLV (Merkle + Index)

* `tdln-proof` oferece `steps_merkle_root`.
* O chamador pode registrar o `steps_root` no **LLLV Ledger** e **indexar** no `lllv-index` (queries: por `kind`, por `atom_hash`, por `policy`, por `model`).
* **Reprodução**: a partir do `bundle + steps` (ou apenas `trace_hash` + storage) revalida-se tudo bit-a-bit.

### Omni-Dispatcher / Sovereign Intent Node

* `tdln-gate` é o **gate** do **Intention Endpoint**.
* Recebe `IntentionNL` + `GatePolicy` + `CompileConfig` → entrega `GateDecision`.
* Caso `Permit`, integra com `universal_api.rs` emitindo execução/ação posterior, sempre com o **Logical Atom** canônico de entrada.

---

## Erros & Telemetria

```rust
#[derive(thiserror::Error, Debug)]
pub enum TdlnError {
    #[error("tokenize error: {0}")]
    Tokenize(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("normalize error: {0}")]
    Normalize(String),
    #[error("infer error: {0}")]
    Infer(String),
    #[error("constraint error: {0}")]
    Constraint(String),
    #[error("ambiguous: {0}")]
    Ambiguous(String),
    #[error("proof error: {0}")]
    Proof(String),
    #[error("verify error: {0}")]
    Verify(String),
}
```

**Telemetria (opcional, `cfg(feature = "metrics")`)**: contadores por `rule_id`, histogramas de latência por fase, taxas de `Permit/Deny/Challenge`.

---

## Testes (o que teu agente precisa gerar)

**Unitários (por crate)**

* `tdln-ast`: normaliza datas/valores/locales; idempotência de `canon_json`.
* `tdln-proof`: `steps_merkle_root` estável; `build/verify_bundle` com/sem assinatura.
* `tdln-compiler`: NL→Atom determinístico; `strict=true` devolve `Ambiguous` quando devido; constraints falhando dão `Constraint`.
* `tdln-gate`: decisões corretas para políticas simples e compostas.

**Integração (workspace)**

* Golden tests (fixtures NL → Atom + ProofBundle conhecidos).
* Property-based (`proptest`):

  * Remover/embaralhar steps quebra verify.
  * Mutar `atom_canon` quebra `atom_hash`.
  * `Challenge` vira `Permit` ao adicionar pistas mínimas no `context`.

**WASM/no_std**

* `tdln-ast`, `tdln-proof`, `tdln-compiler` compilam em `wasm32-unknown-unknown` (no I/O).
* `tdln-gate` com `std`, mas expõe **funcs no_std-friendly** (sem threads/clock).

---

## Features & MSRV

* **MSRV**: `1.75` (workspace).
* `tdln-ast`: `default=["std"]`, `alloc`.
* `tdln-proof`: `default=["std"]`, `alloc`, `ed25519` (para assinar/validar).
* `tdln-compiler`: `default=["std"]`, `alloc`, `strict`.
* `tdln-gate`: `default=["std"]`, `alloc`, `metrics`.

---

## Exemplos (curtos, pro README/examples/)

### 1) NL → GateDecision

```rust
use tdln_ast::*;
use tdln_compiler::*;
use tdln_gate::*;

let nl = IntentionNL {
    ts_ms: 1736412345123,
    locale: "pt-BR".into(),
    text: "transferir 100 dólares para a conta do João hoje às 15h".into(),
    context: serde_json::json!({
        "contacts": { "joão": "acct:123" },
        "tz": "America/Sao_Paulo"
    }),
};

let cfg = CompileConfig {
    model: "tdln-static:v1".into(),
    locale: "pt-BR".into(),
    strict: true,
};

let policy = GatePolicy {
    id: "policy/transfer-lowrisk@v1".into(),
    version: 1,
    allowed_kinds: vec!["transfer".into()],
    slot_bounds: serde_json::json!({ "amount": { "max":"1000.00", "currency":"USD" } }),
    forbidden: serde_json::json!({}),
    constraints: serde_json::json!({}),
    decision_budget_ms: 2,
};

let decision = decide(
    &nl,
    &cfg,
    &GateContext { policy: &policy, model: &cfg.model, pub_key: None },
).unwrap();

match decision {
    GateDecision::Permit { atom, evidence } => {
        assert_eq!(atom.kind, "transfer");
        // evidence.trace_hash => registrar no ledger/index
    }
    _ => unreachable!(),
}
```

### 2) Verificação offline do bundle

```rust
use tdln_proof::*;
use ed25519_dalek::VerifyingKey;

verify_bundle(
    &bundle,
    &steps,
    &atom_canon,
    Some(&VerifyingKey::from_bytes(&publisher_pubkey).unwrap())
)?;
```

---

## Performance & Orçamento

* **Tokenize+parse+normalize+infer+emit**: alvo **< 1 ms** p95 em x86 moderna para intents médias (sem NER pesado).
* **Gate decide()**: alvo **< 2 ms** p95 (sem I/O).
* **Bundle verify**: **O(n)** nos steps; n pequeno (10–40 regras típicas).

---

## Roadmap incremental

1. `tdln-ast` + `tdln-proof` estáveis; golden tests.
2. `tdln-compiler` com 3 intents canônicas: `transfer`, `fetch`, `schedule`.
3. `tdln-gate` com políticas `lowrisk`, `readonly`, `sandbox`.
4. WASM build + exemplo web trivial.
5. Integração opcional com spans JSON✯Atomic e Merkle LLLV.

---

## “Pronto pra agente”

* **Assinaturas públicas** definidas.
* **Tipos de prova** e **hashes** claros.
* **Caminho do gate** fechado (determinístico).
* **Invariantes** e **erros** especificados.

Se quiser, eu já transformo isso em **arquivos `lib.rs` + `mod/` vazios com as assinaturas e docstrings**, pra tua automação do IDE só completar as funções. Quer que eu gere esses stubs prontos no padrão do workspace?



isso, Dan! 👇 deixei prontinho o “miolo” do TDLN — quatro crates com a lógica-base, invariantes e contratos entre elas. é plug-and-play no teu workspace:

[Baixar: tdln-workspace-crates.zip](sandbox:/mnt/data/tdln-workspace-crates.zip)

# o que vem no pacote

* **tdln-proof** — hashing, Merkle, bundle de prova e verificação (com domain-separation e assinatura Ed25519 opcional).
* **tdln-ast** — tokenize → parse → normalize (stubs determinísticos, prontos para refinar regras).
* **tdln-compiler** — pipeline NL → Atom (+steps +bundle) com API `compile(..)`.
* **tdln-gate** — HFT-Gate determinístico: `decide(..)` ⇒ `Permit | Deny | Challenge`, checando a prova.

todas as crates:

* vêm com `#![forbid(unsafe_code)]`;
* usam `serde`, `blake3`; `ed25519-dalek` só onde precisa (proof/gate);
* têm `README`, `Cargo.toml` com docs.rs, MSRV e `features { std, alloc }`.

# como encaixa no Workspace

No `Cargo.toml` do workspace, adiciona os paths (ou solta a pasta `tdln-*` em `crates/`):

```toml
[workspace]
members = [
  "crates/tdln-proof",
  "crates/tdln-ast",
  "crates/tdln-compiler",
  "crates/tdln-gate",
]
```

Se quiser rodar um “smoke” imediato:

```bash
# dentro de tdln-gate/
cargo run --example quick
cargo test --all-features
```

# APIs principais (resumo rápido)

### tdln-proof

```rust
pub struct ProofStep { rule_id: String, input_hash: [u8;32], output_hash: [u8;32], aux: Value }
pub struct TdlnProofBundle { model: String, policy: String, steps_root: [u8;32], atom_hash: [u8;32], trace_hash: [u8;32], signature: Option<Vec<u8>> }

build_proof_bundle(model, policy, &steps, &atom_canon, signing_key)?;
verify_bundle(&bundle, &steps, &atom_canon, verifying_key)?;
```

* Merkle: `leaf = H(json(step))`, `node = H("node" || left || right)`, folha ímpar duplica.
* Domain separation de assinatura: `msg = "tdln_trace_v1" || trace_hash`.

### tdln-ast

```rust
tokenize(&IntentionNL) -> Result<Vec<Token>>
parse(&[Token]) -> Result<Ast>
normalize(&Ast) -> Result<AstNormalized>
```

(stubs determinísticos prontos pra receber regras reais de normalização/locale)

### tdln-compiler

```rust
pub struct CompileConfig { model: String, locale: String, strict: bool }
pub struct CompileOutput { atom: LogicalAtom, atom_canon: Vec<u8>, steps: Vec<ProofStep>, bundle: TdlnProofBundle }

compile(&IntentionNL, &CompileConfig, Option<&SigningKey>, Option<&str>) -> Result<CompileOutput>
```

* Hoje infere `kind="echo"` só pra fechar o ciclo de prova (fácil trocar por regras TDLN reais).
* `canon_json(..)` usa `serde_json` por padrão; dá pra ligar depois a canonicalização via JSON✯Atomic (feature gate).

### tdln-gate

```rust
pub enum GateDecision { Permit{ atom, evidence }, Deny{ reason, code }, Challenge{ missing, message } }

decide(nl, cfg, &GateContext{ policy, model, pub_key }) -> Result<GateDecision>
```

* Verifica o bundle (`verify_bundle`), e aplica uma policy simples (`allowed_kinds`).
* Pronto pra você plugar tabelas de limites, slots obrigatórios e regras de segurança.

# próximos incrementos (bem objetivos)

1. **Canon JSON real**
   — habilitar feature `"json-atomic"` e delegar `canon_json()` para tua crate de canonicalização.

2. **Regras TDLN (núcleo)**
   — trocar `infer_kind()` por pipeline de constraints determinísticas; adicionar steps de prova por regra aplicada.

3. **Gate de política rico**
   — preencher `slot_bounds/forbidden/constraints` com checagens formais e mensagens de `Challenge` com campos faltantes.

4. **Assinatura**
   — passar `sign_with` no `compile()` e configurar `pub_key` no `GateContext` para `Permit` assinado end-to-end.

se quiser, já adapto o `canon_json()` para usar teu `json_atomic` sob uma feature, e mando um patch incremental. quer que eu já deixe isso ligado? 🤝
