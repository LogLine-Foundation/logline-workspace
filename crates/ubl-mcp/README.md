# ubl-mcp

**Secure, Audited Model Context Protocol for LogLine Agents**

MCP tools, but with a kernel: policy-first, audit-ready, and boringly predictable.

[![Crates.io](https://img.shields.io/crates/v/ubl-mcp.svg)](https://crates.io/crates/ubl-mcp)
[![Documentation](https://docs.rs/ubl-mcp/badge.svg)](https://docs.rs/ubl-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## What is this?

`ubl-mcp` is a clean implementation of the Model Context Protocol (JSON-RPC 2.0) with integrated policy gates and audit logging. It's the "universal IO bus" for your agents — interop with the MCP ecosystem without giving the model a foot-gun.

### Security Model

```text
┌─────────────┐     ┌──────────────┐     ┌──────────────┐     ┌─────────────┐
│ Agent Brain ├────▶│ PolicyGate   ├────▶│ Transport    ├────▶│ MCP Server  │
└─────────────┘     │ (permit/deny)│     │ (stdio/http) │     └──────┬──────┘
                    └──────────────┘     └──────────────┘            │
                           │                                         │
                           ▼                                         ▼
                    ┌──────────────┐                          ┌─────────────┐
                    │ AuditSink    │                          │ Tool Result │
                    │ (UBL Ledger) │                          └─────────────┘
                    └──────────────┘
```

1. **Gate-before-IO**: tool calls are proposals → Gate decides Permit/Deny/Challenge
2. **Auditable**: every call (success, failure, or blocked) is recorded
3. **Schema-first**: tools declare their input schema (via schemars)

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `client` | ✅ | MCP client with SecureToolCall |
| `server` | ✅ | MCP server with schema-first tools |
| `transport-stdio` | ✅ | stdio transport (line-delimited JSON) |
| `transport-http` | ❌ | HTTP transport |
| `gate-tdln` | ❌ | TDLN Gate integration |
| `audit` | ❌ | UBL Ledger audit sink |

## Quickstart

### Client (Gate-aware)

```rust
use ubl_mcp::{McpClient, gate::AllowAll, audit::NoAudit, client::MockEndpoint};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = McpClient::new(
        AllowAll,                           // Gate: permit all (use custom for production)
        NoAudit,                            // Audit: disabled (use TracingAudit or UblAudit)
        MockEndpoint::with_text("Hello!"),  // Endpoint: mock for testing
    );

    let result = client
        .tool("echo", json!({"text": "hello"}))
        .execute()
        .await?;
    
    println!("Result: {:?}", result);
    Ok(())
}
```

### Server (schema-first)

```rust
use ubl_mcp::{ServerBuilder, ToolResult};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct EchoArgs {
    text: String,
}

let server = ServerBuilder::new("my-tools")
    .tool("echo", "Echo text back", |args: EchoArgs| async move {
        Ok(ToolResult::text(args.text))
    })
    .build();

// Handle requests
let tools = server.list_tools();
println!("Available tools: {}", tools.len());
```

### Custom Gate (block destructive operations)

```rust
use ubl_mcp::gate::{PolicyGate, GateDecision};
use async_trait::async_trait;
use serde_json::Value;

struct SafeGate;

#[async_trait]
impl PolicyGate for SafeGate {
    async fn decide(&self, tool: &str, _args: &Value) -> GateDecision {
        if tool.starts_with("delete") || tool.starts_with("drop") {
            GateDecision::Deny { reason: "destructive operations blocked".into() }
        } else {
            GateDecision::Permit
        }
    }
}
```

## API Overview

### Gate Types

| Gate | Behavior |
|------|----------|
| `AllowAll` | Permits everything (testing) |
| `DenyAll` | Blocks everything (testing) |
| `AllowlistGate` | Permits only listed tools |
| `DenylistGate` | Blocks only listed tools |
| `TdlnGate` | TDLN policy evaluation (requires `gate-tdln`) |

### Audit Sinks

| Sink | Behavior |
|------|----------|
| `NoAudit` | Discards all records |
| `TracingAudit` | Logs via tracing |
| `MemoryAudit` | Stores in memory (testing) |
| `UblAudit` | Writes to UBL Ledger (requires `audit`) |

### Error Model

| Error | Meaning |
|-------|---------|
| `Protocol(msg)` | JSON-RPC or MCP protocol error |
| `ToolFailure(msg)` | Tool returned an error |
| `PolicyViolation(msg)` | Gate denied the call |
| `Transport(msg)` | IO or connection error |

## Examples

```bash
# Run echo example
cargo run -p ubl-mcp --example echo
```

## Security

- `#![forbid(unsafe_code)]`
- Gate-before-IO: every call goes through PolicyGate
- Audit trail: all calls recorded via AuditSink
- Schema validation via schemars

## Status

- **v0.2.0**: PolicyGate trait, AuditSink trait, SecureToolCall pattern
- **v0.1.0**: Basic client/server, TDLN Gate

## License

MIT — See [LICENSE](LICENSE)
