//! Audit sink for recording tool calls.
//!
//! Every tool call (whether permitted, denied, or challenged) can be
//! recorded via an `AuditSink` for compliance and debugging.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A record of a tool call for audit purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// Timestamp (Unix epoch seconds).
    pub ts: i64,
    /// Tool name.
    pub tool: String,
    /// Arguments (canonicalized).
    pub args: Value,
    /// Gate decision ("permit", "deny", "challenge").
    pub gate_decision: String,
    /// Outcome ("success", "error", "blocked").
    pub outcome: String,
    /// Result (if executed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error message (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Latency in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl ToolCallRecord {
    /// Create a new tool call record.
    #[must_use]
    pub fn new(tool: impl Into<String>, args: Value) -> Self {
        Self {
            ts: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            tool: tool.into(),
            args,
            gate_decision: "unknown".into(),
            outcome: "unknown".into(),
            result: None,
            error: None,
            latency_ms: None,
        }
    }

    /// Set the gate decision.
    #[must_use]
    pub fn with_gate_decision(mut self, decision: impl Into<String>) -> Self {
        self.gate_decision = decision.into();
        self
    }

    /// Set the outcome.
    #[must_use]
    pub fn with_outcome(mut self, outcome: impl Into<String>) -> Self {
        self.outcome = outcome.into();
        self
    }

    /// Set the result.
    #[must_use]
    pub fn with_result(mut self, result: Value) -> Self {
        self.result = Some(result);
        self
    }

    /// Set the error.
    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Set the latency.
    #[must_use]
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }
}

/// Trait for audit sinks that record tool calls.
///
/// Implement this trait to create custom audit backends
/// (file, database, UBL Ledger, etc.).
#[async_trait]
pub trait AuditSink: Send + Sync {
    /// Record a tool call.
    ///
    /// This is called after every tool call attempt (whether it succeeded,
    /// failed, or was blocked by the gate).
    async fn record(&self, record: ToolCallRecord) -> Result<(), anyhow::Error>;
}

/// A no-op audit sink (discards all records).
pub struct NoAudit;

#[async_trait]
impl AuditSink for NoAudit {
    async fn record(&self, _record: ToolCallRecord) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

/// A tracing-based audit sink (logs records via tracing).
pub struct TracingAudit;

#[async_trait]
impl AuditSink for TracingAudit {
    async fn record(&self, record: ToolCallRecord) -> Result<(), anyhow::Error> {
        tracing::info!(
            tool = record.tool,
            gate = record.gate_decision,
            outcome = record.outcome,
            latency_ms = ?record.latency_ms,
            "mcp.tool_call"
        );
        Ok(())
    }
}

/// An in-memory audit sink (for testing).
pub struct MemoryAudit {
    records: std::sync::Mutex<Vec<ToolCallRecord>>,
}

impl MemoryAudit {
    /// Create a new in-memory audit sink.
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Get all recorded tool calls.
    #[must_use]
    pub fn records(&self) -> Vec<ToolCallRecord> {
        self.records.lock().unwrap().clone()
    }

    /// Clear all records.
    pub fn clear(&self) {
        self.records.lock().unwrap().clear();
    }
}

impl Default for MemoryAudit {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditSink for MemoryAudit {
    async fn record(&self, record: ToolCallRecord) -> Result<(), anyhow::Error> {
        self.records.lock().unwrap().push(record);
        Ok(())
    }
}

// Implement for Arc<T> where T: AuditSink to allow shared audit sinks
#[async_trait]
impl<T: AuditSink> AuditSink for std::sync::Arc<T> {
    async fn record(&self, record: ToolCallRecord) -> Result<(), anyhow::Error> {
        (**self).record(record).await
    }
}

/// UBL Ledger audit sink (requires `audit` feature).
#[cfg(feature = "audit")]
pub mod ubl_impl {
    use super::*;

    /// Audit sink that writes to a UBL Ledger.
    pub struct UblAudit {
        // In a real implementation, this would hold a LedgerWriter
        // For now we just log
        _path: std::path::PathBuf,
    }

    impl UblAudit {
        /// Create a new UBL audit sink.
        #[must_use]
        pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
            Self { _path: path.into() }
        }
    }

    #[async_trait]
    impl AuditSink for UblAudit {
        async fn record(&self, record: ToolCallRecord) -> Result<(), anyhow::Error> {
            // In a real implementation, this would append to the ledger
            // with canonical JSON and CID computation
            let json = serde_json::to_string(&record)?;
            tracing::debug!(record = json, "ubl_audit.record");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_audit_accepts_all() {
        let sink = NoAudit;
        let record = ToolCallRecord::new("test", serde_json::json!({}));
        assert!(sink.record(record).await.is_ok());
    }

    #[tokio::test]
    async fn memory_audit_stores_records() {
        let sink = MemoryAudit::new();

        let record1 = ToolCallRecord::new("echo", serde_json::json!({"text": "hello"}))
            .with_gate_decision("permit")
            .with_outcome("success");
        sink.record(record1).await.unwrap();

        let record2 = ToolCallRecord::new("delete", serde_json::json!({}))
            .with_gate_decision("deny")
            .with_outcome("blocked");
        sink.record(record2).await.unwrap();

        let records = sink.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].tool, "echo");
        assert_eq!(records[1].tool, "delete");
    }

    #[test]
    fn record_builder_works() {
        let record = ToolCallRecord::new("test", serde_json::json!({}))
            .with_gate_decision("permit")
            .with_outcome("success")
            .with_result(serde_json::json!({"ok": true}))
            .with_latency(42);

        assert_eq!(record.gate_decision, "permit");
        assert_eq!(record.outcome, "success");
        assert!(record.result.is_some());
        assert_eq!(record.latency_ms, Some(42));
    }
}
