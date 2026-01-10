//! MCP Client with Gate enforcement and Audit logging.
//!
//! The client provides a `SecureToolCall` pattern that:
//! 1. Runs the tool call through a `PolicyGate` (permit/deny/challenge)
//! 2. Executes via the transport if permitted
//! 3. Records the call via an `AuditSink`

use crate::audit::{AuditSink, ToolCallRecord};
use crate::gate::{GateDecision, PolicyGate};
use crate::protocol::{
    ContentBlock, JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpError, RequestId, ToolResult,
};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Trait for RPC endpoints (transports).
#[async_trait]
pub trait RpcEndpoint: Send + Sync {
    /// Send a JSON-RPC message and receive a response.
    async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError>;
}

/// MCP Client with integrated gate and audit.
pub struct McpClient<G, A, E>
where
    G: PolicyGate,
    A: AuditSink,
    E: RpcEndpoint,
{
    gate: Arc<G>,
    audit: Arc<A>,
    endpoint: Arc<E>,
    request_id: AtomicI64,
}

impl<G, A, E> McpClient<G, A, E>
where
    G: PolicyGate,
    A: AuditSink,
    E: RpcEndpoint,
{
    /// Create a new MCP client.
    pub fn new(gate: G, audit: A, endpoint: E) -> Self {
        Self {
            gate: Arc::new(gate),
            audit: Arc::new(audit),
            endpoint: Arc::new(endpoint),
            request_id: AtomicI64::new(1),
        }
    }

    fn next_id(&self) -> RequestId {
        RequestId::Number(self.request_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Create a secure tool call.
    ///
    /// Returns a `SecureToolCall` that can be executed with `.execute()`.
    #[must_use]
    pub fn tool<'a>(&'a self, name: impl Into<String>, args: Value) -> SecureToolCall<'a, G, A, E> {
        SecureToolCall {
            client: self,
            tool: name.into(),
            args,
        }
    }

    /// List available tools from the server.
    pub async fn list_tools(&self) -> Result<Vec<crate::ToolDefinition>, McpError> {
        let request = JsonRpcRequest::new(self.next_id(), "tools/list", Value::Null);
        let response = self.endpoint.call(request).await?;

        if let Some(error) = response.error {
            return Err(McpError::Protocol(error.message));
        }

        let tools: Vec<crate::ToolDefinition> = response
            .result
            .and_then(|v| v.get("tools").cloned())
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::Protocol(format!("invalid tools list: {e}")))?
            .unwrap_or_default();

        Ok(tools)
    }

    /// Initialize the MCP connection.
    pub async fn initialize(&self) -> Result<Value, McpError> {
        let request = JsonRpcRequest::new(
            self.next_id(),
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "ubl-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        );
        let response = self.endpoint.call(request).await?;

        if let Some(error) = response.error {
            return Err(McpError::Protocol(error.message));
        }

        Ok(response.result.unwrap_or(Value::Null))
    }
}

/// A secure tool call that goes through gate and audit.
pub struct SecureToolCall<'a, G, A, E>
where
    G: PolicyGate,
    A: AuditSink,
    E: RpcEndpoint,
{
    client: &'a McpClient<G, A, E>,
    /// The tool name.
    pub tool: String,
    /// The tool arguments.
    pub args: Value,
}

impl<'a, G, A, E> SecureToolCall<'a, G, A, E>
where
    G: PolicyGate,
    A: AuditSink,
    E: RpcEndpoint,
{
    /// Execute the tool call.
    ///
    /// 1. Check the policy gate
    /// 2. Execute via transport if permitted
    /// 3. Record via audit sink
    pub async fn execute(self) -> Result<ToolResult, McpError> {
        let start = Instant::now();
        let mut record = ToolCallRecord::new(&self.tool, self.args.clone());

        // 1. Check gate
        let decision = self.client.gate.decide(&self.tool, &self.args).await;
        record = record.with_gate_decision(match &decision {
            GateDecision::Permit => "permit",
            GateDecision::Deny { .. } => "deny",
            GateDecision::Challenge { .. } => "challenge",
        });

        match decision {
            GateDecision::Permit => {}
            GateDecision::Deny { reason } => {
                record = record.with_outcome("blocked").with_error(&reason);
                let _ = self.client.audit.record(record).await;
                return Err(McpError::PolicyViolation(reason));
            }
            GateDecision::Challenge { reason } => {
                record = record.with_outcome("blocked").with_error(&reason);
                let _ = self.client.audit.record(record).await;
                return Err(McpError::PolicyViolation(format!("requires consent: {reason}")));
            }
        }

        // 2. Execute via transport
        let request = JsonRpcRequest::new(
            self.client.next_id(),
            "tools/call",
            serde_json::json!({
                "name": self.tool,
                "arguments": self.args
            }),
        );

        let response = match self.client.endpoint.call(request).await {
            Ok(r) => r,
            Err(e) => {
                record = record
                    .with_outcome("error")
                    .with_error(e.to_string())
                    .with_latency(start.elapsed().as_millis() as u64);
                let _ = self.client.audit.record(record).await;
                return Err(e);
            }
        };

        // 3. Parse response
        if let Some(error) = response.error {
            record = record
                .with_outcome("error")
                .with_error(&error.message)
                .with_latency(start.elapsed().as_millis() as u64);
            let _ = self.client.audit.record(record).await;
            return Err(McpError::ToolFailure(error.message));
        }

        let result_value = response.result.unwrap_or(Value::Null);
        let result: ToolResult = serde_json::from_value(result_value.clone())
            .map_err(|e| McpError::Protocol(format!("invalid result: {e}")))?;

        // Check if tool returned an error
        if result.is_error == Some(true) {
            let msg = result
                .content
                .first()
                .map(|c| match c {
                    ContentBlock::Text { text } => text.clone(),
                    _ => "tool error".into(),
                })
                .unwrap_or_else(|| "unknown error".into());
            record = record
                .with_outcome("error")
                .with_error(&msg)
                .with_latency(start.elapsed().as_millis() as u64);
            let _ = self.client.audit.record(record).await;
            return Err(McpError::ToolFailure(msg));
        }

        // 4. Record success
        record = record
            .with_outcome("success")
            .with_result(result_value)
            .with_latency(start.elapsed().as_millis() as u64);
        let _ = self.client.audit.record(record).await;

        Ok(result)
    }
}

/// Mock endpoint for testing.
pub struct MockEndpoint {
    response: std::sync::Mutex<Option<ToolResult>>,
}

impl MockEndpoint {
    /// Create a mock endpoint that returns the given result.
    #[must_use]
    pub fn with_result(result: ToolResult) -> Self {
        Self {
            response: std::sync::Mutex::new(Some(result)),
        }
    }

    /// Create a mock endpoint that returns a text result.
    #[must_use]
    pub fn with_text(text: impl Into<String>) -> Self {
        Self::with_result(ToolResult::text(text))
    }

    /// Create a mock endpoint that returns an error.
    #[must_use]
    pub fn with_error(msg: impl Into<String>) -> Self {
        Self::with_result(ToolResult::error(msg))
    }
}

#[async_trait]
impl RpcEndpoint for MockEndpoint {
    async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
        match request.method.as_str() {
            "tools/call" => {
                let result = self
                    .response
                    .lock()
                    .unwrap()
                    .clone()
                    .unwrap_or_else(|| ToolResult::text("mock"));
                Ok(JsonRpcResponse::success(
                    request.id,
                    serde_json::to_value(result).unwrap(),
                ))
            }
            "tools/list" => Ok(JsonRpcResponse::success(
                request.id,
                serde_json::json!({"tools": []}),
            )),
            "initialize" => Ok(JsonRpcResponse::success(
                request.id,
                serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "serverInfo": {"name": "mock", "version": "0.1.0"}
                }),
            )),
            _ => Ok(JsonRpcResponse::error(
                request.id,
                JsonRpcError::method_not_found(&request.method),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{MemoryAudit, NoAudit};
    use crate::gate::{AllowAll, DenyAll};

    #[tokio::test]
    async fn client_permit_executes() {
        let client = McpClient::new(AllowAll, NoAudit, MockEndpoint::with_text("hello"));
        let result = client.tool("echo", serde_json::json!({"text": "hi"})).execute().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn client_deny_blocks() {
        let client = McpClient::new(
            DenyAll::new("blocked"),
            NoAudit,
            MockEndpoint::with_text("hello"),
        );
        let result = client.tool("echo", serde_json::json!({})).execute().await;
        assert!(matches!(result, Err(McpError::PolicyViolation(_))));
    }

    #[tokio::test]
    async fn client_records_audit() {
        let audit = Arc::new(MemoryAudit::new());
        let client = McpClient {
            gate: Arc::new(AllowAll),
            audit: audit.clone(),
            endpoint: Arc::new(MockEndpoint::with_text("ok")),
            request_id: AtomicI64::new(1),
        };

        let _ = client.tool("echo", serde_json::json!({})).execute().await;

        let records = audit.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tool, "echo");
        assert_eq!(records[0].gate_decision, "permit");
        assert_eq!(records[0].outcome, "success");
    }

    #[tokio::test]
    async fn client_records_denied_audit() {
        let audit = Arc::new(MemoryAudit::new());
        let client = McpClient {
            gate: Arc::new(DenyAll::new("nope")),
            audit: audit.clone(),
            endpoint: Arc::new(MockEndpoint::with_text("ok")),
            request_id: AtomicI64::new(1),
        };

        let _ = client.tool("danger", serde_json::json!({})).execute().await;

        let records = audit.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].gate_decision, "deny");
        assert_eq!(records[0].outcome, "blocked");
    }
}

