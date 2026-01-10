//! Gate integration tests

use async_trait::async_trait;
use serde_json::json;
use ubl_mcp::{
    audit::{MemoryAudit, NoAudit},
    client::MockEndpoint,
    gate::{AllowAll, AllowlistGate, DenyAll, GateDecision, PolicyGate},
    McpClient, McpError,
};
use std::sync::Arc;

/// Custom gate that denies destructive operations
struct SafeGate;

#[async_trait]
impl PolicyGate for SafeGate {
    async fn decide(&self, tool: &str, _args: &serde_json::Value) -> GateDecision {
        if tool.contains("delete") || tool.contains("drop") || tool.contains("rm") {
            GateDecision::Deny {
                reason: format!("destructive operation '{tool}' is not allowed"),
            }
        } else if tool.contains("write") || tool.contains("exec") {
            GateDecision::Challenge {
                reason: format!("'{tool}' requires approval"),
            }
        } else {
            GateDecision::Permit
        }
    }
}

#[tokio::test]
async fn deny_all_blocks_everything() {
    let client = McpClient::new(
        DenyAll::new("test: all blocked"),
        NoAudit,
        MockEndpoint::with_text("should not reach"),
    );

    let result = client.tool("echo", json!({"text": "hi"})).execute().await;

    assert!(matches!(result, Err(McpError::PolicyViolation(ref msg)) if msg.contains("all blocked")));
}

#[tokio::test]
async fn allow_all_permits_everything() {
    let client = McpClient::new(AllowAll, NoAudit, MockEndpoint::with_text("hello"));

    let result = client.tool("echo", json!({"text": "hi"})).execute().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn allowlist_blocks_unlisted() {
    let gate = AllowlistGate::new(["echo", "read", "list"]);
    let client = McpClient::new(gate, NoAudit, MockEndpoint::with_text("ok"));

    // Listed tools should work
    assert!(client.tool("echo", json!({})).execute().await.is_ok());
    assert!(client.tool("read", json!({})).execute().await.is_ok());

    // Unlisted should be blocked
    let result = client.tool("delete", json!({})).execute().await;
    assert!(matches!(result, Err(McpError::PolicyViolation(_))));
}

#[tokio::test]
async fn custom_gate_blocks_destructive() {
    let client = McpClient::new(SafeGate, NoAudit, MockEndpoint::with_text("ok"));

    // Safe operations pass
    assert!(client.tool("read", json!({})).execute().await.is_ok());
    assert!(client.tool("list", json!({})).execute().await.is_ok());

    // Destructive operations blocked
    let result = client.tool("delete_file", json!({})).execute().await;
    assert!(matches!(result, Err(McpError::PolicyViolation(ref msg)) if msg.contains("destructive")));

    let result = client.tool("drop_table", json!({})).execute().await;
    assert!(matches!(result, Err(McpError::PolicyViolation(ref msg)) if msg.contains("destructive")));
}

#[tokio::test]
async fn challenge_blocks_until_approved() {
    let client = McpClient::new(SafeGate, NoAudit, MockEndpoint::with_text("ok"));

    // Operations requiring approval are blocked (challenge = block in simple client)
    let result = client.tool("write_file", json!({})).execute().await;
    assert!(matches!(result, Err(McpError::PolicyViolation(ref msg)) if msg.contains("requires consent")));

    let result = client.tool("exec_command", json!({})).execute().await;
    assert!(matches!(result, Err(McpError::PolicyViolation(ref msg)) if msg.contains("requires consent")));
}

/// Helper to create client with shared audit for testing
fn make_client_with_audit<G: PolicyGate + 'static>(
    gate: G,
    audit: Arc<MemoryAudit>,
) -> McpClient<G, Arc<MemoryAudit>, MockEndpoint> {
    // Use a wrapper to access Arc<MemoryAudit> as AuditSink
    McpClient::new(gate, audit, MockEndpoint::with_text("ok"))
}

#[tokio::test]
async fn audit_records_blocked_calls() {
    let audit = Arc::new(MemoryAudit::new());
    let client = make_client_with_audit(DenyAll::new("blocked"), audit.clone());

    let _ = client.tool("danger", json!({})).execute().await;

    let records = audit.records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].tool, "danger");
    assert_eq!(records[0].gate_decision, "deny");
    assert_eq!(records[0].outcome, "blocked");
}

#[tokio::test]
async fn audit_records_successful_calls() {
    let audit = Arc::new(MemoryAudit::new());
    let client = make_client_with_audit(AllowAll, audit.clone());

    let _ = client.tool("echo", json!({"text": "hello"})).execute().await;

    let records = audit.records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].tool, "echo");
    assert_eq!(records[0].gate_decision, "permit");
    assert_eq!(records[0].outcome, "success");
    assert!(records[0].latency_ms.is_some());
}
