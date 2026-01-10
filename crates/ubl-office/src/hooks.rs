//! Hook traits for Office event callbacks.
//!
//! Implement these traits to receive notifications about agent lifecycle events.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Receipt for an agent decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionReceipt {
    /// Timestamp (Unix seconds).
    pub ts: i64,
    /// Decision CID (BLAKE3 of canonical bytes).
    pub decision_cid: String,
    /// Intent kind.
    pub intent_kind: String,
    /// Model used.
    pub model_id: String,
    /// Input tokens consumed.
    pub input_tokens: u32,
    /// Output tokens generated.
    pub output_tokens: u32,
}

/// Receipt for a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallReceipt {
    /// Timestamp (Unix seconds).
    pub ts: i64,
    /// Tool name.
    pub tool: String,
    /// CID of canonical args.
    pub args_cid: String,
    /// Gate decision ("permit", "deny", "challenge").
    pub gate_decision: String,
    /// Outcome ("ok", "error", "skipped").
    pub outcome: String,
    /// Latency in milliseconds.
    pub latency_ms: u64,
}

/// Receipt for dreaming/maintenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamReceipt {
    /// Timestamp (Unix seconds).
    pub ts: i64,
    /// Events consolidated.
    pub events_consolidated: u32,
    /// Memory items before.
    pub memory_before: usize,
    /// Memory items after.
    pub memory_after: usize,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

/// Receipt for handover (session pause/end).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoverReceipt {
    /// Timestamp (Unix seconds).
    pub ts: i64,
    /// Session summary.
    pub summary: String,
    /// Pending tasks (if any).
    pub pending_tasks: Vec<String>,
    /// Total decisions this session.
    pub decisions_count: u64,
}

/// Receipt for quota breach.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaBreachReceipt {
    /// Timestamp (Unix seconds).
    pub ts: i64,
    /// Quota type ("input_tokens", "output_tokens", "daily_quota", "decisions").
    pub quota_type: String,
    /// Limit value.
    pub limit: u64,
    /// Attempted value.
    pub attempted: u64,
}

/// Trait for receiving Office lifecycle events.
///
/// Implement this to write receipts to a ledger, emit metrics, etc.
#[async_trait]
pub trait OfficeHooks: Send + Sync {
    /// Called when the office starts.
    async fn on_start(&self, tenant_id: &str) {
        let _ = tenant_id;
    }

    /// Called after each decision.
    async fn on_decision(&self, receipt: DecisionReceipt) {
        let _ = receipt;
    }

    /// Called after each tool call (or attempted call).
    async fn on_tool_call(&self, receipt: ToolCallReceipt) {
        let _ = receipt;
    }

    /// Called after dreaming/maintenance.
    async fn on_dream(&self, receipt: DreamReceipt) {
        let _ = receipt;
    }

    /// Called on session handover.
    async fn on_handover(&self, receipt: HandoverReceipt) {
        let _ = receipt;
    }

    /// Called when a quota is breached.
    async fn on_quota_breach(&self, receipt: QuotaBreachReceipt) {
        let _ = receipt;
    }

    /// Called on office shutdown.
    async fn on_shutdown(&self, tenant_id: &str) {
        let _ = tenant_id;
    }

    /// Called on any error.
    async fn on_error(&self, error: &str) {
        let _ = error;
    }
}

/// No-op hooks implementation.
pub struct NoopHooks;

#[async_trait]
impl OfficeHooks for NoopHooks {}

/// Tracing-based hooks (logs events via tracing).
pub struct TracingHooks;

#[async_trait]
impl OfficeHooks for TracingHooks {
    async fn on_start(&self, tenant_id: &str) {
        tracing::info!(tenant_id, "office.start");
    }

    async fn on_decision(&self, receipt: DecisionReceipt) {
        tracing::info!(
            intent = receipt.intent_kind,
            model = receipt.model_id,
            input_tokens = receipt.input_tokens,
            output_tokens = receipt.output_tokens,
            "office.decision"
        );
    }

    async fn on_tool_call(&self, receipt: ToolCallReceipt) {
        tracing::info!(
            tool = receipt.tool,
            gate = receipt.gate_decision,
            outcome = receipt.outcome,
            latency_ms = receipt.latency_ms,
            "office.tool_call"
        );
    }

    async fn on_dream(&self, receipt: DreamReceipt) {
        tracing::info!(
            events = receipt.events_consolidated,
            memory_before = receipt.memory_before,
            memory_after = receipt.memory_after,
            duration_ms = receipt.duration_ms,
            "office.dream"
        );
    }

    async fn on_handover(&self, receipt: HandoverReceipt) {
        tracing::info!(
            decisions = receipt.decisions_count,
            pending = receipt.pending_tasks.len(),
            "office.handover"
        );
    }

    async fn on_quota_breach(&self, receipt: QuotaBreachReceipt) {
        tracing::warn!(
            quota_type = receipt.quota_type,
            limit = receipt.limit,
            attempted = receipt.attempted,
            "office.quota_breach"
        );
    }

    async fn on_shutdown(&self, tenant_id: &str) {
        tracing::info!(tenant_id, "office.shutdown");
    }

    async fn on_error(&self, error: &str) {
        tracing::error!(error, "office.error");
    }
}
