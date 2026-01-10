//! Quickstart example for tdln-brain
//!
//! Demonstrates: CognitiveContext → MockBackend → parse_decision

use serde_json::json;
use tdln_brain::{parser, CognitiveContext, GenerationConfig, Message, MockBackend, NeuralBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) Build cognitive context
    let ctx = CognitiveContext {
        system_directive: "You are LogLine's TDLN brain. Output VALID JSON for a SemanticUnit."
            .into(),
        recall: vec!["User balance: 420".into(), "Last action: deposit 100".into()],
        history: vec![Message::user("grant to alice amount 100")],
        constraints: vec![
            "Never transfer > 500 without second approval".into(),
            "Read-only on /invoices/*".into(),
        ],
    };

    // 2) Render messages
    let messages = ctx.render();
    println!("=== Rendered Messages ===");
    for msg in &messages {
        println!("[{}]: {}", msg.role, &msg.content[..msg.content.len().min(100)]);
    }
    println!();

    // 3) Create mock backend with valid response
    let backend = MockBackend::with_intent(
        "grant",
        json!({
            "to": "alice",
            "amount": 100
        }),
    );

    // 4) Generate
    let raw = backend
        .generate(&messages, &GenerationConfig::default())
        .await?;
    println!("=== Raw Output ===");
    println!("{}", raw.content);
    println!();

    // 5) Parse into strict Decision
    let decision = parser::parse_decision(&raw.content, raw.meta)?;

    println!("=== Parsed Decision ===");
    println!("Intent kind: {}", decision.intent.kind);
    println!("Intent slots: {:?}", decision.intent.slots);
    println!("Intent CID: {}", hex::encode(decision.intent.cid_blake3()));
    if let Some(reasoning) = &decision.reasoning {
        println!("Reasoning: {}", reasoning);
    }

    Ok(())
}
