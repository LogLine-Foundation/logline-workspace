//! `ubl-mcp` — Secure Model Context Protocol for LogLine Agents
//!
//! MCP tools, but with a kernel: policy-first, audit-ready, and boringly predictable.
//!
//! This crate is a clean implementation of the Model Context Protocol (JSON-RPC 2.0)
//! that routes every tool call through your TDLN Gate. It's the "universal IO bus"
//! for your agents — interop with the MCP ecosystem without giving the model a foot-gun.
//!
//! # Why this exists
//!
//! Tooling is where agents get hurt:
//! - "Try calling delete_repo lol" — no thanks.
//! - Shadow state and ad-hoc logs — unverifiable.
//! - Each tool wrapper a snowflake — not scalable.
//!
//! We fix it with three invariants:
//! 1. **Gate-before-IO**: tool calls are proposals → Gate decides Permit/Deny/Challenge
//! 2. **Canonical Intent**: calls carry a tiny, canonicalized Intent body (TDLN)
//! 3. **Schema-first**: tools declare their input schema (via schemars)
//!
//! # Example
//!
//! ```rust,no_run
//! use ubl_mcp::{ToolResult, ContentBlock};
//!
//! let result = ToolResult {
//!     content: vec![ContentBlock::Text { text: "Hello!".into() }],
//!     is_error: Some(false),
//! };
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod protocol;
#[cfg(feature = "client")]
mod client;
#[cfg(feature = "server")]
mod server;
pub mod transport;

pub use protocol::*;
#[cfg(feature = "client")]
pub use client::*;
#[cfg(feature = "server")]
pub use server::*;
