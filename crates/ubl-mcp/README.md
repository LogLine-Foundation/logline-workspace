# ubl-mcp

**Secure Model Context Protocol for LogLine Agents**

MCP tools, but with a kernel: policy-first, audit-ready, and boringly predictable.

[![Crates.io](https://img.shields.io/crates/v/ubl-mcp.svg)](https://crates.io/crates/ubl-mcp)
[![Documentation](https://docs.rs/ubl-mcp/badge.svg)](https://docs.rs/ubl-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## What is this?

`ubl-mcp` is a clean implementation of the Model Context Protocol (JSON-RPC 2.0) that routes every tool call through your TDLN Gate. It's the "universal IO bus" for your agents — interop with the MCP ecosystem without giving the model a foot-gun.

### Why this exists

Tooling is where agents get hurt:
- "Try calling delete_repo lol" — no thanks.
- Shadow state and ad-hoc logs — unverifiable.
- Each tool wrapper a snowflake — not scalable.

We fix it with three invariants:
1. **Gate-before-IO**: tool calls are proposals → Gate decides Permit/Deny/Challenge
2. **Canonical Intent**: calls carry a tiny, canonicalized Intent body (TDLN)
3. **Schema-first**: tools declare their input schema (via schemars)

## Quickstart

### Client (Gate-aware)

```rust
use ubl_mcp::{McpClient, MockTransport, GateContext, ToolResult};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let transport = MockTransport::with_result(ToolResult::text("Hello!"));
    let client = McpClient::new(transport);
    
    let gate_ctx = GateContext {
        allow_freeform: true,
        pre_consented: true, // Skip consent for this call
    };

    let result = client
        .call_tool_secure("echo", json!({"msg": "hello"}), &gate_ctx)
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

#[derive(Deserialize, JsonSchema)]
struct SumArgs {
    a: i64,
    b: i64,
}

let server = ServerBuilder::new("my-tools")
    .tool("echo", "Echo text back", |args: EchoArgs| async move {
        Ok(ToolResult::text(args.text))
    })
    .tool("sum", "Add two integers", |args: SumArgs| async move {
        Ok(ToolResult::text((args.a + args.b).to_string()))
    })
    .build();

// Handle requests
let tools = server.list_tools();
println!("Available tools: {}", tools.len());
```

## API Overview

### Protocol Types

```rust
// JSON-RPC 2.0
struct JsonRpcRequest { id, method, params }
struct JsonRpcResponse { id, result?, error? }

// MCP
struct ToolDefinition { name, description?, inputSchema }
struct ToolResult { content: Vec<ContentBlock>, isError? }
enum ContentBlock { Text { text }, Image { data, mimeType }, Resource { uri, ... } }
```

### Client

```rust
impl McpClient<T: McpTransport> {
    // Gate-first tool execution
    async fn call_tool_secure(&self, tool: &str, args: Value, gate_ctx: &GateContext) 
        -> Result<ToolResult, McpError>;
    
    // List available tools
    async fn list_tools(&self) -> Result<Vec<ToolDefinition>, McpError>;
}
```

### Server

```rust
impl ServerBuilder {
    fn tool<Args: JsonSchema + DeserializeOwned, F, Fut>(
        self, name: &str, description: &str, handler: F
    ) -> Self;
    
    fn build(self) -> McpServer;
}
```

## Error Model

| Error | Meaning |
|-------|---------|
| `Protocol(msg)` | JSON-RPC or MCP protocol error |
| `ToolFailure(msg)` | Tool returned an error |
| `PolicyViolation(msg)` | Gate denied the call |
| `Transport(msg)` | IO or connection error |

## Features

- `client` — MCP client with Gate enforcement (default)
- `server` — MCP server with schema-first tools (default)

## Security

- `#![forbid(unsafe_code)]`
- Gate-before-IO: every call goes through TDLN Gate
- Schema validation via schemars
- Size and time caps (defaults are conservative)

## License

MIT — See [LICENSE](LICENSE)
