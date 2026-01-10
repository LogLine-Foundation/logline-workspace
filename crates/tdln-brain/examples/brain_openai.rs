//! OpenAI provider example — real LLM call.
//!
//! Run with: `OPENAI_API_KEY=... cargo run -p tdln-brain --example brain_openai`

use tdln_brain::providers::openai::OpenAiDriver;
use tdln_brain::{Brain, CognitiveContext, GenerationConfig, Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());

    let driver = OpenAiDriver::new(api_key, &model);
    let brain = Brain::new(driver);

    let ctx = CognitiveContext {
        system_directive: "You are a finance agent; output ONLY a TDLN JSON intent.".into(),
        recall: vec!["Client: alice; risk_tolerance: low".into()],
        history: vec![Message::user("Plan a weekly budget of 100")],
        constraints: vec!["No transfers; analysis only".into()],
    };

    let cfg = GenerationConfig {
        temperature: 0.0,
        max_tokens: Some(400),
        require_reasoning: false,
    };

    println!("Calling {} ...", model);
    let decision = brain.reason(&ctx, &cfg).await?;

    println!("Tokens: in={}, out={}", decision.meta.input_tokens, decision.meta.output_tokens);
    println!("Intent: {:?}", decision.intent);

    if let Some(reasoning) = &decision.reasoning {
        println!("Reasoning: {reasoning}");
    }

    Ok(())
}
