//! Transport implementations for MCP.
//!
//! Transports handle the low-level communication with MCP servers.
//!
//! Available transports:
//! - `stdio` (default): line-delimited JSON over stdin/stdout
//! - `http` (optional): HTTP transport (requires `transport-http` feature)

pub mod stdio;

#[cfg(feature = "transport-http")]
pub mod http;

pub use stdio::StdioTransport;

#[cfg(feature = "transport-http")]
pub use http::{HttpConfig, HttpTransport, SharedHttpTransport};
