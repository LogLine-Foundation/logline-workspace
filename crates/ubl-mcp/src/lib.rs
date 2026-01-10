//! `ubl-mcp` — Secure Model Context Protocol for LogLine Agents
//!
//! MCP tools, but with a kernel: policy-first, audit-ready, and boringly predictable.
//!
//! This crate is a clean implementation of the Model Context Protocol (JSON-RPC 2.0)
//! that routes every tool call through a policy gate. It's the "universal IO bus"
//! for your agents — interop with the MCP ecosystem without giving the model a foot-gun.
//!
//! # Security Model
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐     ┌──────────────┐     ┌─────────────┐
//! │ Agent Brain ├────▶│ PolicyGate   ├────▶│ Transport    ├────▶│ MCP Server  │
//! └─────────────┘     │ (permit/deny)│     │ (stdio/http) │     └──────┬──────┘
//!                     └──────────────┘     └──────────────┘            │
//!                            │                                         │
//!                            ▼                                         ▼
//!                     ┌──────────────┐                          ┌─────────────┐
//!                     │ AuditSink    │                          │ Tool Result │
//!                     │ (UBL Ledger) │                          └─────────────┘
//!                     └──────────────┘
//! ```
//!
//! 1. **Gate-before-IO**: tool calls are proposals → Gate decides Permit/Deny/Challenge
//! 2. **Auditable**: every call (success, failure, or blocked) is recorded
//! 3. **Schema-first**: tools declare their input schema (via schemars)
//!
//! # Example
//!
//! ```rust,no_run
//! use ubl_mcp::{McpClient, ToolResult, gate::AllowAll, audit::NoAudit, client::MockEndpoint};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = McpClient::new(AllowAll, NoAudit, MockEndpoint::with_text("hello"));
//!
//! let result = client
//!     .tool("echo", serde_json::json!({"text": "hello"}))
//!     .execute()
//!     .await?;
//!
//! println!("Result: {:?}", result);
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - `client` (default): MCP client with SecureToolCall
//! - `server` (default): MCP server with schema-first tool registration
//! - `transport-stdio` (default): stdio transport (line-delimited JSON)
//! - `transport-http`: HTTP transport (optional)
//! - `gate-tdln`: TDLN Gate integration (optional)
//! - `audit`: UBL Ledger audit sink (optional)

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod protocol;
pub mod gate;
pub mod audit;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "transport-stdio")]
pub mod transport;

pub use protocol::*;

#[cfg(feature = "client")]
pub use client::{McpClient, MockEndpoint, RpcEndpoint, SecureToolCall};
#[cfg(feature = "server")]
pub use server::{McpServer, ServerBuilder};

