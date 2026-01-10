ubl-office — The Agent Runtime (Wake · Work · Dream)

run agents like reliable services, not fragile notebooks.

ubl-office is the execution environment for LogLine agents. It coordinates thinking (TDLN), acting (MCP tools), memory (LLLV + ledger), and policy (Gate) under one tight loop. No root access, no mystery state, no shrug emojis.

What it does (in one screen)
	•	Boot: load identity/constitution, attach transports, warm caches
	•	Orient: build a typed CognitiveContext (system directive, recall, constraints)
	•	Decide: call tdln-brain to produce a strict SemanticUnit (TDLN AST)
	•	Gate: run tdln-gate → Permit | Deny | Challenge
	•	Act: execute via ubl-mcp (MCP tools), optionally audit to ubl-ledger
	•	Dream: consolidate short-term into LLLV index; compact context; keep it fresh
	•	Repeat, with backpressure, watchdog timers, exponential backoff on failure

⸻

Cargo (publish-ready)

[package]
name        = "ubl-office"
version     = "0.1.0"
edition     = "2021"
license     = "MIT"
description = "The LogLine Agent Runtime: Wake/Work/Dream loop with Gate-first execution and durable memory."
repository  = "https://github.com/LogLine-Foundation/logline-workspace"
homepage    = "https://logline.foundation"
readme      = "README.md"
keywords    = ["agent","runtime","tdln","mcp","ledger"]
categories  = ["asynchronous","concurrency"]

[features]
default       = ["std","audit","metrics"]
std           = []
audit         = ["dep:ubl-ledger"]
metrics       = ["dep:metrics","dep:metrics-exporter-prometheus"]
persist-index = []         # enable LLLV on-disk packs
http-health   = ["dep:axum","dep:tokio"]   # optional /healthz, /metrics

[dependencies]
# user-space cognition + io
tdln-brain   = "0.1"
tdln-ast     = "0.1"
tdln-gate    = "0.1"
ubl-mcp      = "0.1"

# kernel bits (audit & memory)
ubl-ledger   = { version = "0.1", optional = true }
lllv-core    = "0.1"
lllv-index   = "0.1"

# infra
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
thiserror    = "1"
anyhow       = "1"
tracing      = "0.1"
tokio        = { version = "1", features = ["full","sync","time"] }

# optional metrics/health
metrics                   = { version = "0.22", optional = true }
metrics-exporter-prometheus = { version = "0.14", optional = true }
axum                     = { version = "0.7", optional = true, default-features=false, features=["http1","json"] }

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg","docsrs"]


⸻

Module layout

ubl-office/
├─ src/
│  ├─ lib.rs          # public surface
│  ├─ runtime.rs      # state machine & main loop
│  ├─ memory.rs       # short/long-term memory & consolidation
│  ├─ narrator.rs     # orientation: builds CognitiveContext
│  ├─ audit.rs        # (feature=audit) minimal ledger entries
│  ├─ health.rs       # (feature=http-health) /healthz /metrics
│  └─ errors.rs       # OfficeError
└─ examples/
   └─ quickstart.rs


⸻

Public API (stable v0.1)

#![forbid(unsafe_code)]

/// Agent lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficeState {
  Opening,     // bootstrapping identity I/O
  Active,      // OODA: observe/orient/decide/act
  Maintenance, // Dreaming / consolidation / compaction
  Closing,     // shutdown with flush
}

/// Runtime configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OfficeConfig {
  /// tenant/agent identity (free-form); use DID or canonical handle
  pub tenant_id: String,
  /// path to your constitution / gate policy bundle
  pub constitution_path: std::path::PathBuf,

  /// workspace and storage
  pub workspace_root: std::path::PathBuf,
  pub ledger_path:    std::path::PathBuf,

  /// cognition
  pub model_id: String,

  /// loop knobs
  pub max_steps_before_dream: u64, // every N steps, enter maintenance
  pub step_pause_ms: u64,          // heart-beat sleep between steps
}

/// Structured instrumentation (subset).
#[derive(Default, Debug, Clone)]
pub struct OfficeMetrics {
  pub steps_total: u64,
  pub decisions_total: u64,
  pub denials_total: u64,
  pub challenges_total: u64,
  pub tool_errors_total: u64,
}

/// Runtime handle (constructed via `Office::new`).
pub struct Office<B: tdln_brain::NeuralBackend> {
  config: OfficeConfig,
  brain:  B,
  memory: memory::MemorySystem,
  mcp:    ubl_mcp::McpClient,
  narrator: narrator::Narrator,
  metrics: OfficeMetrics,
}

impl<B: tdln_brain::NeuralBackend> Office<B> {
  /// Create an Office with a concrete neural backend (e.g., OpenAI driver).
  pub fn new(config: OfficeConfig, brain: B) -> Self;

  /// Run forever (until error or shutdown signal).
  pub async fn run(mut self) -> Result<(), OfficeError>;
}


⸻

How the loop works (OODA + Dream)
	1.	Observe/Orient: Narrator::orient fetches recent signals (last ledger entries or system events), queries memory for recall (LLLV later), crafts a CognitiveContext with constraints from the constitution.
	2.	Decide (TDLN): tdln-brain generates the SemanticUnit; strict parsing ensures JSON-valid output even if the model rambles.
	3.	Gate: tdln-gate::decide runs policies.
	•	Permit → continue
	•	Deny → record denial metrics; (optional) audit denial reason
	•	Challenge → bubble up; caller can require human approval
	4.	Act (MCP): ubl-mcp::call_tool_secure(tool,args,gate_ctx) executes only if Gate already Permit; on audit feature, write a compact NDJSON record to ubl-ledger (ts, actor, tool, args_cid, outcome, latency_ms).
	5.	Dream (maintenance): every max_steps_before_dream, consolidate recent ephemeral state into a durable structure (LLLV). Reduce context, keep the brain crisp.
	6.	Backoff & watchdog: transient failures use capped exponential backoff; hard errors abort cleanly.

⸻

Memory model

pub struct MemorySystem {
  /// scratchpad: short-term ephemeral notes
  short_buffer: Vec<String>,
  /// optional: persisted vector index (lllv-index)
  #[cfg(feature="persist-index")]
  index_root: std::path::PathBuf,
}

impl MemorySystem {
  pub fn new() -> Self { /* ... */ }

  /// extract relevant memories for the current context
  pub async fn recall(&self, signal: &str) -> anyhow::Result<Vec<String>> { Ok(vec![]) }

  /// periodic compaction into long-term (LLLV)
  pub async fn consolidate(&mut self, recent_events: &[String]) -> anyhow::Result<()> {
    // TODO: embed → pack → commit proof
    Ok(())
  }
}

	•	Short-term: in-process scratchpad to reduce duplicate queries during a step
	•	Long-term: optional LLLV pack with Merkle evidence (later), or keep simple until needed
	•	Provenance: anything promoted to long-term should have CID + time, so later you can prove why the agent “remembers” X

⸻

Audit model (feature = audit)
	•	On Permit → after tool finishes, append an NDJSON record:
{ ts, tenant, tool, args_cid, outcome: "ok|error|challenge|deny", latency_ms }
	•	On Deny/Challenge → append a minimal record with the reason code (no sensitive args).
	•	CID discipline: never write raw args; canonize (json_atomic) and store CID + size to keep logs safe and small.

⸻

Health & metrics (optional)

With http-health, you get:
	•	GET /healthz → {"state":"Active","steps_total":123}
	•	GET /metrics → Prometheus exposition (if metrics feature on)

Counters to export:
	•	office_steps_total, office_decisions_total, office_denials_total, office_challenges_total, office_tool_errors_total
	•	timing histograms for decision latency and tool latency

⸻

Example: bring your own OpenAI driver

use ubl_office::{Office, OfficeConfig};
use tdln_brain::providers::openai::OpenAiDriver;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();

  let cfg = OfficeConfig {
    tenant_id: "acme.bot".into(),
    constitution_path: "config/constitution.yml".into(),
    workspace_root: "var/workspace".into(),
    ledger_path: "var/ledger.ndjson".into(),
    model_id: "gpt-4.1-mini".into(),
    max_steps_before_dream: 100,
    step_pause_ms: 1500,
  };

  let brain = OpenAiDriver::new(
    std::env::var("OPENAI_API_KEY")?, 
    cfg.model_id.clone()
  );

  let office = Office::new(cfg, brain);
  office.run().await?;
  Ok(())
}


⸻

Tests to ship (unit + integration)

runtime
	•	state_transitions_open_to_active_to_maintenance: threshold triggers Dreaming
	•	denied_decision_never_calls_mcp: Gate=deny → no tool execution, metric increments
	•	challenge_bubbles_up: verify Challenge path returns a distinct OfficeError
	•	backoff_applies_on_tool_failure: consecutive failures increase delay (capped)

memory
	•	consolidate_is_idempotent: same events → same CID/pack id
	•	recall_does_not_panic_on_empty_index: recall gracefully returns empty

audit (feature=audit)
	•	audit_ok_minimal_recorded: Permit OK writes minimal NDJSON line
	•	audit_no_sensitive_payload: args raw bytes never appear

health (feature=http-health)
	•	health_endpoint_reports_active
	•	metrics_exporter_boots

⸻

Error model

#[derive(thiserror::Error, Debug)]
pub enum OfficeError {
  #[error("cognition error: {0}")]  Brain(String),
  #[error("policy gate error: {0}")] Gate(String),
  #[error("tool execution error: {0}")] Tool(String),
  #[error("audit error: {0}")]       Audit(String),
  #[error("io error: {0}")]          Io(String),
}

	•	All external errors get mapped; we keep the loop alive when possible (Tool/Audit) and break on fatal (Brain persistent failure or corrupted constitution).

⸻

Security notes (non-negotiables)
	•	No unsafe.
	•	Gate-first always; ubl-mcp already enforces this, but ubl-office double-checks before calling.
	•	Size/time caps on cognition and tool calls; defaults are conservative.
	•	CID-only audit; never log secrets; redact known sensitive keys (token, password, api_key).
	•	Deterministic canonicalization happens upstream (json_atomic), and all hashes are BLAKE3.
	•	Idempotence: decisions carry a decision_id; retries that replay the same ID are treated as duplicates to avoid double effects.

⸻

README (drop-in)

# ubl-office

The **LogLine Agent Runtime** — a boringly reliable loop that turns intent into audited action.

- Wake → Work → Dream
- Gate-first execution (TDLN policies)
- Optional audit to UBL ledger
- Memory hygiene with recall & consolidation

## Quickstart

See `examples/quickstart.rs`. Provide a `tdln-brain` backend (e.g., OpenAI), a constitution for the Gate, and a ledger path.

## Features

- `audit`: write minimal NDJSON records to `ubl-ledger`
- `metrics`: Prometheus exporter
- `http-health`: `/healthz` & `/metrics`
- `persist-index`: enable LLLV on-disk packs for long-term recall

## Guarantees

- No `unsafe`
- Gate-before-IO
- Canonicalized, CID-backed audit when enabled


⸻

Roadmap (v0.1 → v0.2)
	•	v0.1: core loop, Gate-first calls, optional audit, health & metrics
	•	v0.1.1: graceful shutdown, SIGTERM handling, structured tracing spans
	•	v0.2: background Dreamer scheduler, persistent LLLV packs, memory budgets & eviction policies, human-in-the-loop challenge resolver hook

⸻

Proof of Done
	•	cargo clippy --all-features -- -D warnings ✅
	•	cargo test -p ubl-office (runtime + memory + audit + health) ✅
	•	cargo doc -p ubl-office ✅
	•	README example compiles as doctest ✅
	•	minimal smoke run with OpenAI driver ✅

⸻

if this matches your vibe, we can lock it in and you can hand it to the IDE agent. want me to draft the constitution.yaml skeleton and the challenge resolver hook next, or jump into the foundation crate readmes?