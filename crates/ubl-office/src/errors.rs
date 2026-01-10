//! Office errors.

use thiserror::Error;

/// Errors from the Office runtime.
#[derive(Debug, Error)]
pub enum OfficeError {
    /// Cognition error (brain failure).
    #[error("cognition error: {0}")]
    Brain(String),
    /// Policy gate error.
    #[error("policy gate error: {0}")]
    Gate(String),
    /// Tool execution error.
    #[error("tool execution error: {0}")]
    Tool(String),
    /// IO error.
    #[error("io error: {0}")]
    Io(String),
    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),
    /// Shutdown requested.
    #[error("shutdown requested")]
    Shutdown,
}

impl From<std::io::Error> for OfficeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<tdln_brain::BrainError> for OfficeError {
    fn from(e: tdln_brain::BrainError) -> Self {
        Self::Brain(e.to_string())
    }
}

impl From<ubl_mcp::McpError> for OfficeError {
    fn from(e: ubl_mcp::McpError) -> Self {
        Self::Tool(e.to_string())
    }
}
