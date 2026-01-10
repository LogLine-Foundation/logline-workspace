//! MCP Client with Gate enforcement.

use crate::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpError, RequestId, ToolResult,
};
use serde_json::Value;
use std::sync::atomic::{AtomicI64, Ordering};
use tdln_compiler::{compile, CompileCtx};
use tdln_gate::{decide, preflight, Consent, Decision as GateDecision, PolicyCtx};

/// Gate context for policy decisions.
#[derive(Clone, Debug)]
pub struct GateContext {
    /// Whether freeform intents are allowed
    pub allow_freeform: bool,
    /// Pre-consented (skip consent check)
    pub pre_consented: bool,
}

impl Default for GateContext {
    fn default() -> Self {
        Self {
            allow_freeform: true,
            pre_consented: false,
        }
    }
}

/// MCP Client that enforces Gate policies before tool execution.
pub struct McpClient<T> {
    transport: T,
    request_id: AtomicI64,
}

impl<T> McpClient<T>
where
    T: McpTransport,
{
    /// Create a new MCP client with the given transport.
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            request_id: AtomicI64::new(1),
        }
    }

    fn next_id(&self) -> RequestId {
        RequestId::Number(self.request_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Call a tool with Gate enforcement.
    ///
    /// 1. Create a virtual intent from the tool call
    /// 2. Run through TDLN Gate
    /// 3. If Permit, execute via transport
    /// 4. Return result or policy error
    ///
    /// # Errors
    ///
    /// - `McpError::PolicyViolation` if Gate denies the call
    /// - `McpError::ToolFailure` if the tool returns an error
    /// - `McpError::Transport` for transport-level errors
    pub async fn call_tool_secure(
        &self,
        tool: &str,
        args: Value,
        gate_ctx: &GateContext,
    ) -> Result<ToolResult, McpError> {
        // 1. Create virtual intent text
        let intent_text = format!("call {tool} with {args}");

        // 2. Compile to TDLN
        let compile_ctx = CompileCtx {
            rule_set: "v1".into(),
        };
        let compiled = compile(&intent_text, &compile_ctx)
            .map_err(|e| McpError::Protocol(format!("compile error: {e}")))?;

        // 3. Gate preflight
        let policy_ctx = PolicyCtx {
            allow_freeform: gate_ctx.allow_freeform,
        };
        let preflight_result = preflight(&compiled, &policy_ctx)
            .map_err(|e| McpError::Protocol(format!("gate error: {e}")))?;

        // 4. Check gate decision
        let consent = Consent {
            accepted: gate_ctx.pre_consented || preflight_result.decision == GateDecision::Allow,
        };
        let gate_output = decide(&compiled, &consent, &policy_ctx)
            .map_err(|e| McpError::Protocol(format!("gate error: {e}")))?;

        match gate_output.decision {
            GateDecision::Allow => {
                // Proceed with tool call
            }
            GateDecision::Deny => {
                return Err(McpError::PolicyViolation(format!(
                    "Gate denied call to '{tool}'"
                )));
            }
            GateDecision::NeedsConsent => {
                return Err(McpError::PolicyViolation(format!(
                    "Call to '{tool}' requires consent"
                )));
            }
        }

        // 5. Execute via transport
        let request = JsonRpcRequest::new(
            self.next_id(),
            "tools/call",
            serde_json::json!({
                "name": tool,
                "arguments": args
            }),
        );

        let response = self.transport.send_request(request).await?;

        // 6. Parse response
        if let Some(error) = response.error {
            return Err(McpError::ToolFailure(error.message));
        }

        let result: ToolResult = response
            .result
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::Protocol(format!("invalid result: {e}")))?
            .unwrap_or_else(|| ToolResult::text(""));

        if result.is_error == Some(true) {
            let msg = result
                .content
                .first()
                .map(|c| match c {
                    crate::ContentBlock::Text { text } => text.clone(),
                    _ => "tool error".into(),
                })
                .unwrap_or_else(|| "unknown error".into());
            return Err(McpError::ToolFailure(msg));
        }

        Ok(result)
    }

    /// List available tools from the server.
    pub async fn list_tools(&self) -> Result<Vec<crate::ToolDefinition>, McpError> {
        let request = JsonRpcRequest::new(self.next_id(), "tools/list", Value::Null);
        let response = self.transport.send_request(request).await?;

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
}

/// Transport trait for MCP communication.
#[async_trait::async_trait]
pub trait McpTransport: Send + Sync {
    /// Send a request and wait for response.
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError>;

    /// Send a notification (no response expected).
    async fn send_notification(&self, method: &str, params: Value) -> Result<(), McpError>;
}

/// Mock transport for testing.
pub struct MockTransport {
    response: std::sync::Mutex<Option<ToolResult>>,
}

impl MockTransport {
    /// Create a mock transport that returns the given result.
    pub fn with_result(result: ToolResult) -> Self {
        Self {
            response: std::sync::Mutex::new(Some(result)),
        }
    }

    /// Create a mock transport that returns an error.
    pub fn with_error(msg: impl Into<String>) -> Self {
        Self::with_result(ToolResult::error(msg))
    }
}

#[async_trait::async_trait]
impl McpTransport for MockTransport {
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
        if request.method == "tools/call" {
            let result = self
                .response
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| ToolResult::text("mock response"));
            Ok(JsonRpcResponse::success(
                request.id,
                serde_json::to_value(result).unwrap(),
            ))
        } else if request.method == "tools/list" {
            Ok(JsonRpcResponse::success(
                request.id,
                serde_json::json!({"tools": []}),
            ))
        } else {
            Ok(JsonRpcResponse::error(
                request.id,
                JsonRpcError::method_not_found(&request.method),
            ))
        }
    }

    async fn send_notification(&self, _method: &str, _params: Value) -> Result<(), McpError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn client_permit_executes() {
        let transport = MockTransport::with_result(ToolResult::text("hello"));
        let client = McpClient::new(transport);
        let gate_ctx = GateContext {
            allow_freeform: true,
            pre_consented: true,
        };

        let result = client
            .call_tool_secure("echo", serde_json::json!({"msg": "hi"}), &gate_ctx)
            .await
            .unwrap();

        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn client_needs_consent_blocks() {
        let transport = MockTransport::with_result(ToolResult::text("hello"));
        let client = McpClient::new(transport);
        let gate_ctx = GateContext {
            allow_freeform: true,
            pre_consented: false, // No consent
        };

        let result = client
            .call_tool_secure("dangerous", serde_json::json!({}), &gate_ctx)
            .await;

        assert!(matches!(result, Err(McpError::PolicyViolation(_))));
    }
}
