//! Transport implementations for MCP.
//!
//! Currently provides:
//! - Stdio transport (line-delimited JSON over stdin/stdout)

pub mod stdio;

pub use stdio::StdioTransport;
