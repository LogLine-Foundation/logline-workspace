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
    /// Policy violation — action blocked by Gate.
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    /// Tool execution error.
    #[error("tool execution error: {0}")]
    Tool(String),
    /// IO error.
    #[error("io error: {0}")]
    Io(String),
    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),
    /// Context window exceeded.
    #[error("context overflow: {0}")]
    ContextOverflow(String),
    /// Quota exceeded (tokens, decisions, daily limit).
    #[error("quota exceeded: {0}")]
    QuotaExceeded(String),
    /// Ledger append failed.
    #[error("ledger error: {0}")]
    Ledger(String),
    /// Provider error (LLM API).
    #[error("provider error: {0}")]
    Provider(String),
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
        match e {
            tdln_brain::BrainError::ContextOverflow => {
                Self::ContextOverflow("brain context overflow".into())
            }
            tdln_brain::BrainError::Provider(msg) => Self::Provider(msg),
            other => Self::Brain(other.to_string()),
        }
    }
}

impl From<ubl_mcp::McpError> for OfficeError {
    fn from(e: ubl_mcp::McpError) -> Self {
        match e {
            ubl_mcp::McpError::PolicyViolation(msg) => Self::PolicyViolation(msg),
            other => Self::Tool(other.to_string()),
        }
    }
}
