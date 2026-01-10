alright dan—kicking off with crate 1/3: tdln-brain. this is the “mind” layer that turns messy NL into a strictly-parsable, proof-friendly intent. i’m giving you a full, drop-in README + API map + examples + test plan so your IDE agent can scaffold it perfectly. 🧠✨

⸻

tdln-brain — Deterministic Cognitive Layer for LogLine OS

NL → TDLN SemanticUnit → canonical bytes (via json_atomic) → happy Gate → verifiable execution.

tdln-brain is the cognitive shim between LLMs and the LogLine kernel. It renders a typed cognitive context, asks a model for an intent, and guarantees you can parse it into a tdln_ast::SemanticUnit with zero ambiguity. Reasoning (free-form text) is separated from action (strict JSON). Same semantics → same bytes → same CID.

Why this exists

Typical “agent” libraries push strings around and pray. We want:
	•	Strict output: JSON that parses into a SemanticUnit or it’s a hard error.
	•	Kernel awareness: constraints (policies) visible before generation, reducing Gate rejections.
	•	Deterministic canon: one source of truth for canonical bytes (delegates to json_atomic).
	•	Simple drivers: plug any model (cloud or local) via a tiny NeuralBackend trait.

TL;DR

use tdln_brain::{CognitiveContext, NeuralBackend, GenerationConfig, parser::parse_decision, Message, RawOutput, UsageMeta};
use tdln_ast::SemanticUnit;

// 1) Prepare context
let ctx = CognitiveContext {
    system_directive: "You’re LogLine’s TDLN brain. Output VALID JSON for a SemanticUnit.".into(),
    recall: vec!["User balance: 420".into()],
    history: vec![Message::user("grant to alice amount 100")],
    constraints: vec!["Never transfer > 500 without second approval".into()],
};

// 2) Render messages for the model
let messages = ctx.render();

// 3) Call any NeuralBackend you like (example: a mock)
struct Mock; impl NeuralBackend for Mock {
    fn model_id(&self) -> &str { "mock-tdln" }
    async fn generate(&self, _m: &[Message], _c: &GenerationConfig) -> Result<RawOutput, tdln_brain::BrainError> {
        Ok(RawOutput {
            content: r#"```json
{ "kind": "grant", "slots": { "to": "alice", "amount": 100 } }
```"#.into(),
            meta: UsageMeta { model_id: "mock-tdln".into(), ..Default::default() }
        })
    }
}

// 4) Generate & parse
let backend = Mock;
let raw = backend.generate(&messages, &GenerationConfig::default()).await?;
let decision = parse_decision(&raw.content, raw.meta)?;
let intent: SemanticUnit = decision.intent;

// 5) Done: `intent` is canonicalizable, provable, and Gate-friendly.


⸻

Crate scope

In
	•	Typed cognitive context (CognitiveContext) → rendered prompt (Vec<Message>)
	•	Model integration via NeuralBackend trait
	•	Strict parsing (parse_decision) with reasoning extraction
	•	Output shape: Decision { reasoning?, intent: SemanticUnit, meta }
	•	Helpful error model (BrainError)

Out (by design)
	•	Policy decisions (that’s tdln-gate)
	•	Transport/wire (that’s ubl-sirp)
	•	Ledger writes (that’s ubl-ledger)
	•	Full agent loop (that’s ubl-office)

⸻

Features
	•	http-drivers (optional): includes a minimal reqwest-based driver you can adapt.
	•	std only (no no_std targets here).
	•	No unsafe.

[features]
default = []
http-drivers = ["dep:reqwest"]


⸻

Public API (stable v0.1)

Data types

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message { pub role: String, pub content: String }
impl Message {
  pub fn system(s: impl Into<String>) -> Self; 
  pub fn user(s: impl Into<String>) -> Self; 
  pub fn assistant(s: impl Into<String>) -> Self;
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct CognitiveContext {
  pub system_directive: String,
  pub recall: Vec<String>,
  pub history: Vec<Message>,
  pub constraints: Vec<String>,
}
impl CognitiveContext { pub fn render(&self) -> Vec<Message>; }

#[derive(Clone, Debug, Default)]
pub struct UsageMeta { pub input_tokens: u32, pub output_tokens: u32, pub model_id: String }

pub struct RawOutput { pub content: String, pub meta: UsageMeta }

#[derive(Debug)]
pub struct Decision {
  pub reasoning: Option<String>,
  pub intent: tdln_ast::SemanticUnit,
  pub meta: UsageMeta,
}

#[derive(Debug, thiserror::Error)]
pub enum BrainError {
  #[error("provider error: {0}")] Provider(String),
  #[error("hallucination: output was not valid TDLN: {0}")] Hallucination(String),
  #[error("context window exceeded")] ContextOverflow,
  #[error("parsing error: {0}")] Parsing(String),
}

Trait for model providers

#[async_trait::async_trait]
pub trait NeuralBackend: Send + Sync {
  fn model_id(&self) -> &str;
  async fn generate(&self, messages: &[Message], cfg: &GenerationConfig)
    -> Result<RawOutput, BrainError>;
}

#[derive(Clone, Debug)]
pub struct GenerationConfig {
  pub temperature: f32,
  pub max_tokens: Option<u32>,
  pub require_reasoning: bool,
}
impl Default for GenerationConfig { /* temp=0.0, max_tokens=Some(1024) */ }

The parser

pub mod parser {
  use super::*;
  /// Extracts a JSON block (supports ```json fences) and parses into SemanticUnit.
  pub fn parse_decision(raw: &str, meta: UsageMeta) -> Result<Decision, BrainError>;
}


⸻

Prompting model (how CognitiveContext::render() works)

render() builds a single system message packing:
	•	Your directive (role + tone + boundaries)
	•	Constraints (kernel policies the model must respect)
	•	Relevant memory (recall)
	•	Then appends your recent history

This cuts Gate rejections because the model knows the rules before proposing an action.

Example of system scaffold rendered:

IDENTITY: agent/logline
You output VALID JSON for a TDLN SemanticUnit. No extra prose.

### SYSTEM PROTOCOL ###
- Output a single JSON object with fields: kind, slots
- Example: {"kind":"grant","slots":{"to":"alice","amount":100}}

### ACTIVE KERNEL CONSTRAINTS ###
- Never transfer > 500 without secondary approval
- Read-only on /invoices/*

### RELEVANT MEMORY (RECALL) ###
- User balance: 420


⸻

Canon bytes & CID (interoperability invariant)

tdln-brain does not compute CIDs itself; it produces a SemanticUnit that downstream crates (compiler/proof/gate) can:
	•	Canonicalize via json_atomic::canonize
	•	Hash via ubl_crypto::blake3_cid
	•	Prove via tdln-proof

This keeps one source of truth for canonical bytes across the whole stack.

⸻

Examples

1) Mock end-to-end (no network)

# async fn demo() -> anyhow::Result<()> {
use tdln_brain::*;
use tdln_ast::SemanticUnit;

let ctx = CognitiveContext {
  system_directive: "Emit a valid TDLN SemanticUnit JSON.".into(),
  recall: vec![],
  history: vec![Message::user("grant to alice amount 100")],
  constraints: vec!["Never exceed 500 without approval".into()],
};
let messages = ctx.render();

struct Mock; impl NeuralBackend for Mock {
  fn model_id(&self)->&str{"mock"}
  async fn generate(&self,_:&[Message],_:&GenerationConfig)->Result<RawOutput,BrainError>{
    Ok(RawOutput{content:r#"{"kind":"grant","slots":{"to":"alice","amount":100}}"#.into(),meta:UsageMeta::default()})
  }
}
let backend = Mock;
let raw = backend.generate(&messages,&GenerationConfig::default()).await?;
let decision = parser::parse_decision(&raw.content, raw.meta)?;
assert_eq!(decision.intent.kind(), "grant");
# Ok(()) }

2) OpenAI driver (optional, http-drivers)

Provide a simple providers::openai::OpenAiDriver you can wire like:

#[cfg(feature="http-drivers")]
use tdln_brain::providers::openai::OpenAiDriver;

#[cfg(feature="http-drivers")]
# async fn demo(api_key:String)->anyhow::Result<()> {
let driver = OpenAiDriver::new(api_key, "gpt-4o-mini".into());
let decision = {
  let ctx = CognitiveContext { system_directive: "...".into(), ..Default::default() };
  let msgs = ctx.render();
  let raw = driver.generate(&msgs, &GenerationConfig::default()).await?;
  parser::parse_decision(&raw.content, raw.meta)?
};
// decision.intent → pass to gate/ledger
# Ok(()) }

If the model supports JSON mode, the driver requests it. Otherwise, the parser tolerates prose + fenced blocks and still extracts the JSON.

⸻

Error model
	•	Provider(..): transport, API, timeouts, HTTP.
	•	Hallucination(..): we got text, but not a valid SemanticUnit. (You’ll see a precise serde error + a short context.)
	•	Parsing(..): rare—malformed JSON we couldn’t recover from.
	•	ContextOverflow: use this to signal your agent runtime to “dream” / compress memory (ubl-office).

⸻

Tests you should have in tests/
	•	parses_clean_json: raw {"kind":...} works.
	•	parses_fenced_json: markdown-fenced JSON works, reasoning extracted.
	•	rejects_invalid_shape: wrong field types → Hallucination.
	•	constraints_appear_in_system_prompt: ensure render() includes constraints.
	•	stable_render: rendering same context yields identical messages (idempotence).
	•	(optional) fuzz parser with tiny corpus—already covered at workspace level; not mandatory here.

⸻

Security & invariants
	•	#![forbid(unsafe_code)]
	•	No implicit decisions: if we can’t parse strict JSON → hard error.
	•	Never insert model output directly to IO/FS/network—tdln-brain only emits AST; the Gate decides.
	•	Canon chain remains downstream; don’t hash here.

⸻

Cargo metadata (publish-ready)

[package]
name = "tdln-brain"
version = "0.1.0"
license = "MIT"
edition = "2021"
description = "Cognitive layer for LogLine OS. Strict TDLN intent generation with reasoning separation."
readme = "README.md"
repository = "https://github.com/LogLine-Foundation/logline-workspace"
homepage  = "https://logline.foundation"
keywords  = ["llm","agent","json","tdln","security"]
categories= ["science::robotics","api-bindings"]

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg","docsrs"]


⸻

Roadmap (v0.1 → v0.2)
	•	v0.1 (now): cognitive context + parser + mock driver + docs, tests.
	•	v0.1.1: usage telemetry hooks (tracing spans, counters).
	•	v0.2: multi-turn plan mode (structured scratchpad) and red-team “self-check” prompt add-on (pre-Gate sanity).

⸻

One-screen checklist (Proof of Done)
	•	cargo clippy --all-features -- -D warnings ✅
	•	cargo test (the 5 tests above) ✅
	•	cargo doc -p tdln-brain builds clean ✅
	•	README quickstart compiles (doctest or example) ✅
	•	Publish: cargo publish -p tdln-brain ✅

⸻

that’s the brain. want me to drop the complete README.md (with badges and copy-pasteable quickstart) as the next message, or jump straight to crate 2 (ubl-mcp) in the same level of detail?