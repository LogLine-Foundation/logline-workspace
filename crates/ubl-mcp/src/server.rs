//! MCP Server with schema-first tool registration.

use crate::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpError, RequestId, ToolDefinition, ToolResult,
};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for async tool handler function.
pub type BoxedToolHandler = Arc<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<ToolResult, anyhow::Error>> + Send>>
        + Send
        + Sync,
>;

/// A registered tool with its definition and handler.
struct RegisteredTool {
    definition: ToolDefinition,
    handler: BoxedToolHandler,
}

/// MCP Server builder for schema-first tool registration.
pub struct ServerBuilder {
    name: String,
    tools: HashMap<String, RegisteredTool>,
}

impl ServerBuilder {
    /// Create a new server builder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tools: HashMap::new(),
        }
    }

    /// Register a tool with typed arguments.
    ///
    /// The input schema is automatically derived from the `Args` type using schemars.
    pub fn tool<Args, F, Fut>(mut self, name: &str, description: &str, handler: F) -> Self
    where
        Args: DeserializeOwned + JsonSchema + Send + 'static,
        F: Fn(Args) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ToolResult, anyhow::Error>> + Send + 'static,
    {
        let schema = schemars::schema_for!(Args);
        let input_schema = serde_json::to_value(schema).unwrap_or(Value::Object(Default::default()));

        let definition = ToolDefinition {
            name: name.to_string(),
            description: Some(description.to_string()),
            input_schema,
        };

        let handler = Arc::new(handler);
        let boxed_handler: BoxedToolHandler = Arc::new(move |args: Value| {
            let handler = handler.clone();
            Box::pin(async move {
                let typed_args: Args = serde_json::from_value(args)
                    .map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))?;
                handler(typed_args).await
            })
        });

        self.tools.insert(
            name.to_string(),
            RegisteredTool {
                definition,
                handler: boxed_handler,
            },
        );

        self
    }

    /// Build the server.
    pub fn build(self) -> McpServer {
        McpServer {
            name: self.name,
            tools: self.tools,
        }
    }
}

/// MCP Server instance.
pub struct McpServer {
    name: String,
    tools: HashMap<String, RegisteredTool>,
}

impl McpServer {
    /// Get the server name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get list of tool definitions.
    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition.clone()).collect()
    }

    /// Handle a JSON-RPC request.
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_tools_list(request.id),
            "tools/call" => self.handle_tools_call(request.id, request.params).await,
            _ => JsonRpcResponse::error(
                request.id,
                JsonRpcError::method_not_found(&request.method),
            ),
        }
    }

    fn handle_initialize(&self, id: RequestId) -> JsonRpcResponse {
        JsonRpcResponse::success(
            id,
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": self.name,
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )
    }

    fn handle_tools_list(&self, id: RequestId) -> JsonRpcResponse {
        let tools: Vec<_> = self.list_tools();
        JsonRpcResponse::success(id, serde_json::json!({ "tools": tools }))
    }

    async fn handle_tools_call(&self, id: RequestId, params: Value) -> JsonRpcResponse {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let args = params.get("arguments").cloned().unwrap_or(Value::Null);

        let Some(tool) = self.tools.get(name) else {
            return JsonRpcResponse::error(id, JsonRpcError::method_not_found(name));
        };

        match (tool.handler)(args).await {
            Ok(result) => {
                JsonRpcResponse::success(id, serde_json::to_value(result).unwrap_or(Value::Null))
            }
            Err(e) => JsonRpcResponse::success(
                id,
                serde_json::to_value(ToolResult::error(e.to_string())).unwrap(),
            ),
        }
    }
}

/// Trait for server transport implementations.
#[async_trait]
pub trait ServerTransport: Send + Sync {
    /// Receive the next request.
    async fn recv_request(&mut self) -> Result<Option<JsonRpcRequest>, McpError>;

    /// Send a response.
    async fn send_response(&mut self, response: JsonRpcResponse) -> Result<(), McpError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::Deserialize;

    #[derive(Deserialize, JsonSchema)]
    struct EchoArgs {
        text: String,
    }

    #[derive(Deserialize, JsonSchema)]
    struct SumArgs {
        a: i64,
        b: i64,
    }

    #[tokio::test]
    async fn server_schema_generation() {
        let server = ServerBuilder::new("test")
            .tool("echo", "Echo text back", |args: EchoArgs| async move {
                Ok(ToolResult::text(args.text))
            })
            .build();

        let tools = server.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "echo");
        assert!(tools[0].input_schema.is_object());
    }

    #[tokio::test]
    async fn server_handles_tool_call() {
        let server = ServerBuilder::new("math")
            .tool("sum", "Add two numbers", |args: SumArgs| async move {
                Ok(ToolResult::text((args.a + args.b).to_string()))
            })
            .build();

        let request = JsonRpcRequest::new(
            1i64,
            "tools/call",
            serde_json::json!({
                "name": "sum",
                "arguments": {"a": 5, "b": 3}
            }),
        );

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());

        let result: ToolResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn server_handles_unknown_tool() {
        let server = ServerBuilder::new("test").build();

        let request = JsonRpcRequest::new(
            1i64,
            "tools/call",
            serde_json::json!({"name": "unknown", "arguments": {}}),
        );

        let response = server.handle_request(request).await;
        // Returns success with error result, not JSON-RPC error
        assert!(response.result.is_none() || response.error.is_some());
    }

    #[tokio::test]
    async fn server_lists_tools() {
        let server = ServerBuilder::new("multi")
            .tool("echo", "Echo", |args: EchoArgs| async move {
                Ok(ToolResult::text(args.text))
            })
            .tool("sum", "Sum", |args: SumArgs| async move {
                Ok(ToolResult::text((args.a + args.b).to_string()))
            })
            .build();

        let request = JsonRpcRequest::new(1i64, "tools/list", Value::Null);
        let response = server.handle_request(request).await;

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools.len(), 2);
    }
}
