//! JSON-RPC 2.0 protocol types for MCP.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request/response ID.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    /// String ID
    String(String),
    /// Numeric ID
    Number(i64),
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

/// JSON-RPC 2.0 error object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional additional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Parse error (-32700)
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: msg.into(),
            data: None,
        }
    }

    /// Invalid request (-32600)
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: msg.into(),
            data: None,
        }
    }

    /// Method not found (-32601)
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {method}"),
            data: None,
        }
    }

    /// Invalid params (-32602)
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }

    /// Internal error (-32603)
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: msg.into(),
            data: None,
        }
    }
}

/// JSON-RPC 2.0 request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (always "2.0")
    pub jsonrpc: String,
    /// Request ID
    pub id: RequestId,
    /// Method name
    pub method: String,
    /// Method parameters
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub params: Value,
}

impl JsonRpcRequest {
    /// Create a new request.
    pub fn new(id: impl Into<RequestId>, method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version (always "2.0")
    pub jsonrpc: String,
    /// Request ID this is responding to
    pub id: RequestId,
    /// Result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response.
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: RequestId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 notification (no ID, no response expected).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// Protocol version
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Parameters
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub params: Value,
}

/// MCP tool definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP tool execution result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    /// Content blocks in the result
    pub content: Vec<ContentBlock>,
    /// Whether this result represents an error
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolResult {
    /// Create a text result.
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text { text: s.into() }],
            is_error: Some(false),
        }
    }

    /// Create an error result.
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text { text: msg.into() }],
            is_error: Some(true),
        }
    }
}

/// Content block in a tool result.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
    },
    /// Image content (base64)
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded image data
        data: String,
        /// MIME type
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Resource reference
    #[serde(rename = "resource")]
    Resource {
        /// Resource URI
        uri: String,
        /// MIME type
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        /// Optional text content
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
}

/// MCP errors specific to this crate.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// Protocol-level error
    #[error("protocol error: {0}")]
    Protocol(String),
    /// Tool execution failed
    #[error("tool execution failed: {0}")]
    ToolFailure(String),
    /// Security policy violation
    #[error("security policy violation: {0}")]
    PolicyViolation(String),
    /// Transport error
    #[error("transport error: {0}")]
    Transport(String),
    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for McpError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl From<std::io::Error> for McpError {
    fn from(e: std::io::Error) -> Self {
        Self::Transport(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serialization() {
        let req = JsonRpcRequest::new(1i64, "tools/call", serde_json::json!({"name": "echo"}));
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tools/call\""));
    }

    #[test]
    fn response_success() {
        let resp = JsonRpcResponse::success(RequestId::Number(1), serde_json::json!({"ok": true}));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn response_error() {
        let resp = JsonRpcResponse::error(
            RequestId::Number(1),
            JsonRpcError::method_not_found("unknown"),
        );
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.as_ref().unwrap().code, -32601);
    }

    #[test]
    fn tool_result_text() {
        let result = ToolResult::text("hello");
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.is_error, Some(false));
    }

    #[test]
    fn content_block_serialization() {
        let block = ContentBlock::Text {
            text: "hi".into(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"text\""));
    }
}
