//! HTTP transport for MCP.
//!
//! Requires the `transport-http` feature.

use crate::client::RpcEndpoint;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse, McpError};
use async_trait::async_trait;
use std::sync::Arc;

/// HTTP transport configuration.
#[derive(Clone, Debug)]
pub struct HttpConfig {
    /// Base URL of the MCP server.
    pub base_url: String,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Optional bearer token for authentication.
    pub bearer_token: Option<String>,
    /// Optional custom headers.
    pub headers: Vec<(String, String)>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".into(),
            timeout_secs: 30,
            bearer_token: None,
            headers: Vec::new(),
        }
    }
}

impl HttpConfig {
    /// Create a new config with the given base URL.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Default::default()
        }
    }

    /// Set the request timeout.
    #[must_use]
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Set the bearer token for authentication.
    #[must_use]
    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.bearer_token = Some(token.into());
        self
    }

    /// Add a custom header.
    #[must_use]
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

/// HTTP transport for MCP communication.
///
/// Uses reqwest to send JSON-RPC requests over HTTP POST.
pub struct HttpTransport {
    client: reqwest::Client,
    config: HttpConfig,
}

impl HttpTransport {
    /// Create a new HTTP transport with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the reqwest client cannot be built.
    pub fn new(config: HttpConfig) -> Result<Self, McpError> {
        let builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs));

        // Build client
        let client = builder
            .build()
            .map_err(|e| McpError::Transport(format!("failed to build HTTP client: {e}")))?;

        Ok(Self { client, config })
    }

    /// Create with just a base URL (uses defaults for everything else).
    pub fn with_url(base_url: impl Into<String>) -> Result<Self, McpError> {
        Self::new(HttpConfig::new(base_url))
    }

    /// Get the JSON-RPC endpoint URL.
    fn endpoint_url(&self) -> String {
        format!("{}/jsonrpc", self.config.base_url.trim_end_matches('/'))
    }
}

#[async_trait]
impl RpcEndpoint for HttpTransport {
    async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
        let mut req_builder = self
            .client
            .post(self.endpoint_url())
            .header("Content-Type", "application/json");

        // Add bearer token if present
        if let Some(ref token) = self.config.bearer_token {
            req_builder = req_builder.header("Authorization", format!("Bearer {token}"));
        }

        // Add custom headers
        for (name, value) in &self.config.headers {
            req_builder = req_builder.header(name, value);
        }

        // Send request
        let response = req_builder
            .json(&request)
            .send()
            .await
            .map_err(|e| McpError::Transport(format!("HTTP request failed: {e}")))?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(McpError::Transport(format!(
                "HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }

        // Parse response
        let json_response: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| McpError::Protocol(format!("failed to parse response: {e}")))?;

        Ok(json_response)
    }
}

/// HTTP transport wrapped in Arc for shared use.
pub type SharedHttpTransport = Arc<HttpTransport>;

impl HttpTransport {
    /// Wrap in Arc for shared use across tasks.
    #[must_use]
    pub fn shared(self) -> SharedHttpTransport {
        Arc::new(self)
    }
}

#[async_trait]
impl RpcEndpoint for Arc<HttpTransport> {
    async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
        (**self).call(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_builder_works() {
        let config = HttpConfig::new("https://api.example.com")
            .with_timeout(60)
            .with_bearer_token("secret123")
            .with_header("X-Custom", "value");

        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.bearer_token, Some("secret123".into()));
        assert_eq!(config.headers.len(), 1);
    }

    #[test]
    fn endpoint_url_formation() {
        let transport = HttpTransport::with_url("https://api.example.com").unwrap();
        assert_eq!(transport.endpoint_url(), "https://api.example.com/jsonrpc");

        let transport = HttpTransport::with_url("https://api.example.com/").unwrap();
        assert_eq!(transport.endpoint_url(), "https://api.example.com/jsonrpc");
    }

    #[test]
    fn request_serialization_for_http() {
        use crate::protocol::{JsonRpcRequest, RequestId};
        use serde_json::json;

        // Test that requests serialize correctly for HTTP transport
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            "tools/call",
            json!({
                "name": "get_weather",
                "arguments": {"city": "NYC"}
            }),
        );

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("tools/call"));
        assert!(serialized.contains("get_weather"));
        assert!(serialized.contains("NYC"));

        // Verify roundtrip
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.method, "tools/call");
    }

    #[test]
    fn response_deserialization_for_http() {
        use crate::protocol::JsonRpcResponse;
        use serde_json::json;

        // Simulate a successful response from HTTP
        let json_response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "content": [
                    {"type": "text", "text": "Weather in NYC: Sunny, 72°F"}
                ]
            }
        });

        let response: JsonRpcResponse = serde_json::from_value(json_response).unwrap();
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn error_response_handling() {
        use crate::protocol::JsonRpcResponse;
        use serde_json::json;

        // Simulate an error response
        let json_response = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32600,
                "message": "Invalid Request"
            }
        });

        let response: JsonRpcResponse = serde_json::from_value(json_response).unwrap();
        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32600);
        assert_eq!(response.error.as_ref().unwrap().message, "Invalid Request");
    }

    #[test]
    fn shared_transport_creation() {
        let transport = HttpTransport::with_url("https://api.example.com").unwrap();
        let shared = transport.shared();
        
        // Should be able to clone Arc
        let shared2 = Arc::clone(&shared);
        assert_eq!(shared.endpoint_url(), shared2.endpoint_url());
    }
}
