//! Echo server example for ubl-mcp
//!
//! Demonstrates: ServerBuilder with typed tool, client call with Gate

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use ubl_mcp::{
    audit::NoAudit, client::MockEndpoint, gate::AllowAll, McpClient, ServerBuilder, ToolResult,
};

#[derive(Deserialize, JsonSchema)]
struct EchoArgs {
    text: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) Build server with echo tool
    let server = ServerBuilder::new("echo-server")
        .tool("echo", "Echo the input text", |args: EchoArgs| async move {
            Ok(ToolResult::text(format!("Echo: {}", args.text)))
        })
        .build();

    println!("=== Server Tools ===");
    for tool in server.list_tools() {
        println!("- {} : {:?}", tool.name, tool.description);
    }

    // 2) Simulate a tool call via mock endpoint
    let client = McpClient::new(
        AllowAll,
        NoAudit,
        MockEndpoint::with_text("Echo: hello world"),
    );

    println!("\n=== Client Call ===");
    let result = client
        .tool("echo", json!({"text": "hello world"}))
        .execute()
        .await?;

    for block in result.content {
        match block {
            ubl_mcp::ContentBlock::Text { text } => println!("Result: {text}"),
            _ => println!("(other content)"),
        }
    }

    Ok(())
}
