//! Local echo example — instant sanity check.
//!
//! Run with: `cargo run -p tdln-brain --example brain_local`

use tdln_brain::providers::local::LocalEcho;
use tdln_brain::{Brain, CognitiveContext, GenerationConfig, Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let brain = Brain::new(LocalEcho);

    let ctx = CognitiveContext {
        system_directive: "You are a deterministic planner.".into(),
        recall: vec!["Last week we shipped v0.1".into()],
        history: vec![Message::user("Plan next steps")],
        constraints: vec!["Never spend funds".into()],
    };

    let decision = brain.reason(&ctx, &GenerationConfig::default()).await?;

    println!("Model: {}", decision.meta.model_id);
    println!("Parsed intent: {:?}", decision.intent);

    if let Some(reasoning) = &decision.reasoning {
        println!("Reasoning: {reasoning}");
    }

    Ok(())
}
