//! Protocol roundtrip tests

use ubl_mcp::{JsonRpcRequest, JsonRpcResponse, RequestId, ToolResult};

#[test]
fn jsonrpc_request_roundtrip() {
    let request = JsonRpcRequest::new(
        RequestId::Number(42),
        "tools/call",
        serde_json::json!({"name": "echo", "arguments": {"text": "hello"}}),
    );

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"method\":\"tools/call\""));
    assert!(json.contains("\"id\":42"));

    let back: JsonRpcRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.method, "tools/call");
}

#[test]
fn jsonrpc_response_roundtrip() {
    let response = JsonRpcResponse::success(
        RequestId::String("req-1".into()),
        serde_json::json!({"content": [{"type": "text", "text": "ok"}]}),
    );

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"id\":\"req-1\""));

    let back: JsonRpcResponse = serde_json::from_str(&json).unwrap();
    assert!(back.result.is_some());
    assert!(back.error.is_none());
}

#[test]
fn tool_result_roundtrip() {
    let result = ToolResult::text("Hello from test");
    let json = serde_json::to_string(&result).unwrap();

    assert!(json.contains("\"type\":\"text\""));
    assert!(json.contains("Hello from test"));

    let back: ToolResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.content.len(), 1);
    assert_eq!(back.is_error, Some(false));
}

#[test]
fn error_response_roundtrip() {
    let response = JsonRpcResponse::error(
        RequestId::Number(1),
        ubl_mcp::JsonRpcError::method_not_found("unknown_method"),
    );

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"code\":-32601"));

    let back: JsonRpcResponse = serde_json::from_str(&json).unwrap();
    assert!(back.error.is_some());
    assert_eq!(back.error.as_ref().unwrap().code, -32601);
}
