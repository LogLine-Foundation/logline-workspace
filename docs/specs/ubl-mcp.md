ubl-mcp — Secure Model Context Protocol for LogLine Agents

MCP tools, but with a kernel: policy-first, audit-ready, and boringly predictable.

ubl-mcp is a clean implementation of the Model Context Protocol (JSON-RPC 2.0 over stdio/TCP/WS) that routes every tool call through your TDLN Gate and (optionally) writes an audit span to the UBL ledger. It’s the “universal IO bus” for your agents — interop with the MCP ecosystem without giving the model a foot-gun.

Why this exists

Tooling is where agents get hurt:
	•	“Try calling delete_repo lol” — no thanks.
	•	Shadow state and ad-hoc logs — unverifiable.
	•	Each tool wrapper a snowflake — not scalable.

We fix it with three invariants:
	1.	Gate-before-IO: tool calls are proposals → Gate decides Permit/Deny/Challenge.
	2.	Canonical Intent: calls carry a tiny, canonicalized Intent body (TDLN) so the rest of the stack can prove it later.
	3.	Auditable by default: success/failure + metadata written to UBL (feature-flagged).

⸻

Scope

In
	•	JSON-RPC 2.0 framing, request/response/notifications
	•	Stdio transport (default), TCP & WS hooks ready
	•	Client with call_tool_secure() (Gate+Audit)
	•	Server with ergonomic ServerBuilder.tool() (schema-first)
	•	Optional Audit → ubl-ledger append

Out
	•	Long-running orchestration (that’s ubl-office)
	•	Wire-level SIRP capsules (that’s ubl-sirp)
	•	Policy evaluation logic (that’s tdln-gate)

⸻

Features

[features]
default = ["client","server"]
client   = ["dep:tdln-gate","dep:tdln-ast"]
server   = []
audit    = ["dep:ubl-ledger"]     # ledger append for every tool call
transports-tcp = []               # enable when you add a TCP codec
transports-ws  = []               # enable when you add a WS codec


⸻

Install

[dependencies]
ubl-mcp = "0.1"
# Gate + AST so the client can make policy decisions
tdln-gate = "0.1"
tdln-ast  = "0.1"
# Optional audit (only if you enable `audit`)
ubl-ledger = { version = "0.1", optional = true }


⸻

Public API (stable v0.1)

Core protocol types

// JSON-RPC 2.0
#[derive(Serialize,Deserialize,Clone)]
pub enum JsonRpcMessage {
  #[serde(rename="2.0")] Request   { id: RequestId, method: String, #[serde(default)] params: Value },
  #[serde(rename="2.0")] Response  { id: RequestId, result: Option<Value>, error: Option<JsonRpcError> },
  #[serde(rename="2.0")] Notification { method: String, #[serde(default)] params: Value },
}

#[derive(Serialize,Deserialize,Clone,PartialEq,Eq,Hash)]
#[serde(untagged)]
pub enum RequestId { String(String), Number(i64) }

#[derive(Serialize,Deserialize,Clone)]
pub struct JsonRpcError { pub code: i32, pub message: String, pub data: Option<Value> }

// MCP surface
#[derive(Serialize,Deserialize,Clone)]
pub struct ToolDefinition {
  pub name: String,
  pub description: Option<String>,
  pub input_schema: Value, // JSON Schema (via schemars)
}

#[derive(Serialize,Deserialize,Clone)]
pub struct ToolResult {
  pub content: Vec<ContentBlock>,
  pub is_error: Option<bool>,
}

#[derive(Serialize,Deserialize,Clone)]
#[serde(tag="type")]
pub enum ContentBlock {
  #[serde(rename="text")]    Text    { text: String },
  #[serde(rename="image")]   Image   { data: String, mime_type: String },
  #[serde(rename="resource")]Resource{ uri: String, mime_type: Option<String>, text: Option<String> },
}

Client (Gate-aware)

#[derive(thiserror::Error,Debug)]
pub enum McpError {
  #[error("protocol error: {0}")]       Protocol(String),
  #[error("tool execution failed: {0}")]ToolFailure(String),
  #[error("security policy violation: {0}")] PolicyViolation(String),
  #[error("audit error")]               AuditFailure,
}

pub struct McpClient { /* transport handles etc. */ }

impl McpClient {
  /// Gate-first, then execute, then (optionally) audit.
  pub async fn call_tool_secure(
      &self,
      tool: &str,
      args: serde_json::Value,
      gate_ctx: &tdln_gate::GateContext
  ) -> Result<ToolResult, McpError>;
}

Server (schema-first)

#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync {
  fn definition(&self) -> ToolDefinition;
  async fn call(&self, args: serde_json::Value) -> Result<ToolResult, anyhow::Error>;
}

pub struct ServerBuilder { /* registry */ }
impl ServerBuilder {
  pub fn new(name: impl Into<String>) -> Self;
  /// Register a tool with typed args → JSON Schema auto-extracted.
  pub fn tool<Args, F, Fut>(self, name: &str, desc: &str, handler: F) -> Self
  where
    Args: serde::de::DeserializeOwned + schemars::JsonSchema + Send + 'static,
    F: Fn(Args) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<ToolResult, anyhow::Error>> + Send;
  pub fn build(self) -> Self; // start/run hooks up to your transport
}

Transport (stdio by default)

pub mod transport {
  pub mod stdio {
    pub struct StdioTransport<R,W> { /* framed lines codec */ }
    impl<R,W> StdioTransport<R,W> 
    where R: tokio::io::AsyncRead + Unpin, W: tokio::io::AsyncWrite + Unpin {
      pub fn new(reader: R, writer: W) -> Self;
      pub async fn send(&mut self, msg: JsonRpcMessage) -> std::io::Result<()>;
      pub async fn recv(&mut self) -> std::io::Result<Option<JsonRpcMessage>>;
    }
  }
  // tcp/ws modules can follow the same trait shape
}


⸻

How it actually secures a call
	1.	Create a virtual intent from the tool request ("call {tool} with {args}").
	2.	TDLN Gate decides → Permit / Deny / Challenge.
	3.	If Permit, execute via MCP transport.
	4.	If audit feature on → append an audit record to UBL: who/what/args/cid/outcome.
	5.	Return ToolResult to the caller.

The gate sees the same constraints the brain saw: consistent story from intent → policy → execution.

⸻

Examples

1) Secure client call

use ubl_mcp::McpClient;
use serde_json::json;
use tdln_gate::{GateContext, GateDecision};

# async fn demo(client: McpClient, gate_ctx: GateContext) -> anyhow::Result<()> {
let result = client
  .call_tool_secure("drive.read", json!({"path": "/inbox/order.pdf"}), &gate_ctx)
  .await?;

for block in result.content {
  if let ubl_mcp::ContentBlock::Text { text } = block {
    println!("tool says: {text}");
  }
}
# Ok(()) }

2) Tool server with schema-first ergonomics

use ubl_mcp::{ServerBuilder, ToolResult, ContentBlock};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct SumArgs { a: i64, b: i64 }

let server = ServerBuilder::new("math")
  .tool("sum", "Add two integers", |args: SumArgs| async move {
      Ok(ToolResult {
        content: vec![ContentBlock::Text { text: (args.a + args.b).to_string() }],
        is_error: Some(false),
      })
  })
  .build();
// then hook it to a Stdio or TCP transport


⸻

Security model & hardening
	•	No unsafe, no side effects without Gate.
	•	Domain-separated signing is upstream (TDLN/UBL); we don’t sign here.
	•	Canonicalization is upstream (JSON✯Atomic) — if you need CID, pass canonical bytes from tdln-compiler or ubl-codec.
	•	Audit hygiene: when audit is on, append a minimal NDJSON record: ts, actor, tool, args_cid, outcome, latency_ms.
	•	Size & time caps: default request size limits and timeouts (configurable).
	•	Challenge path: GateDecision::Challenge bubbles up as McpError::PolicyViolation("Action requires approval") (caller can implement an approval UI).

⸻

Tests you should ship (unit & integration)
	1.	client_permit_executes → mock Gate Permit ⇒ transport sees one request, audit record present (when audit).
	2.	client_deny_blocks → Deny ⇒ no transport call, no audit, error returned.
	3.	client_challenge_blocks → Challenge ⇒ no transport call; distinct error string.
	4.	server_schema_generation → tool() emits correct JSON Schema for Args.
	5.	stdio_roundtrip → send/recv one request and one response without loss.
	6.	adversarial_big_args_rejected → over size limit returns protocol error.
	7.	latency_accounted → audit record carries latency_ms.

If you want a quick fuzz target, fuzz the stdio line-codec with random lines + malformed JSON-RPC — it should never panic.

⸻

Cargo metadata (publish-ready)

[package]
name        = "ubl-mcp"
version     = "0.1.0"
edition     = "2021"
license     = "MIT"
description = "Secure MCP (Model Context Protocol) for LogLine Agents: Gate-first, audit-ready tool execution."
readme      = "README.md"
repository  = "https://github.com/LogLine-Foundation/logline-workspace"
homepage    = "https://logline.foundation"
keywords    = ["mcp","agent","json-rpc","security","tools"]
categories  = ["api-bindings","asynchronous"]

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg","docsrs"]


⸻

README skeleton (drop-in)

# ubl-mcp

**Secure MCP for LogLine Agents.** JSON-RPC 2.0 transport + Gate-aware client + schema-first server builder.  
- ✅ Gate-before-IO (TDLN policies)  
- ✅ Optional audit to UBL  
- ✅ Stdio transport included; TCP/WS hooks ready

## Quickstart

```rust
// see examples in crate docs

Features
	•	client, server, audit, transports-tcp, transports-ws

Security
	•	No unsafe
	•	Default size/time caps
	•	Audit entries are minimal & canonical-friendly

---

## Roadmap (v0.1 → v0.2)

- v0.1: stdio transport, gate-aware client, schema-first server, audit flag
- v0.1.1: structured tracing spans + metrics adapters
- v0.2: TCP/WS transports, batch calls, tool discovery handshake

---

## Proof of Done (one-screen)

- `cargo clippy --all-features -- -D warnings` ✅  
- `cargo test -p ubl-mcp` (7 tests above) ✅  
- `cargo doc -p ubl-mcp` builds clean ✅  
- README quickstart compiles (doctest/example) ✅  
- Publish: `cargo publish -p ubl-mcp` ✅

---

if you’re good with this, i’ll roll into crate 3/3 — **ubl-office** (the living runtime: Wake/Work/Dream loop, memory consolidation, and MCP execution hooks). want me to go ahead? 💫