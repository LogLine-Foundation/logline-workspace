# tdln-brain

**Deterministic Cognitive Layer for LogLine OS**

Render a narrative frame → call an LLM → extract **only** JSON → validate into `tdln_ast::SemanticUnit`.

[![Crates.io](https://img.shields.io/crates/v/tdln-brain.svg)](https://crates.io/crates/tdln-brain)
[![Documentation](https://docs.rs/tdln-brain/badge.svg)](https://docs.rs/tdln-brain)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Why

- **Prevent tool-call hallucinations**: strict JSON-only outputs
- **Enforce determinism**: temperature 0, JSON mode when available
- **Separate reasoning**: optional thinking goes before JSON, not mixed
- **Machine-legible failures**: invalid output → `BrainError::Hallucination`

## Quickstart

```rust
use tdln_brain::{Brain, CognitiveContext, GenerationConfig};
use tdln_brain::providers::local::LocalEcho;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let brain = Brain::new(LocalEcho);
    let ctx = CognitiveContext {
        system_directive: "You are a planner".into(),
        ..Default::default()
    };
    let decision = brain.reason(&ctx, &GenerationConfig::default()).await?;
    println!("{:?}", decision.intent);
    Ok(())
}
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `parsing` | ✅ | JSON extraction and validation |
| `render` | ✅ | Narrative prompt rendering |
| `providers-openai` | ✅ | OpenAI GPT driver |
| `providers-anthropic` | ❌ | Anthropic Claude driver |
| `providers-local` | ❌ | Local/mock backends |

## Providers

### OpenAI

```rust
use tdln_brain::providers::openai::OpenAiDriver;

let driver = OpenAiDriver::new(api_key, "gpt-4o-mini");
let brain = Brain::new(driver);
```

### Anthropic

```rust
use tdln_brain::providers::anthropic::AnthropicDriver;

let driver = AnthropicDriver::new(api_key, "claude-3-sonnet-20240229");
let brain = Brain::new(driver);
```

### Local/Mock

```rust
use tdln_brain::providers::local::{LocalEcho, MockBackend};

// Fixed noop response
let brain = Brain::new(LocalEcho);

// Custom response
let mock = MockBackend::with_intent("grant", serde_json::json!({"to": "alice"}));
let brain = Brain::new(mock);
```

## CognitiveContext

The context controls what the model sees:

```rust
let ctx = CognitiveContext {
    // Identity + role + boundaries
    system_directive: "You are a finance agent.".into(),
    
    // Long-term memory (RAG results, user facts)
    recall: vec!["User balance: 420".into()],
    
    // Recent conversation
    history: vec![Message::user("Plan a budget")],
    
    // Kernel constraints (violations → gate rejection)
    constraints: vec!["No transfers > 500".into()],
};
```

## Error Model

| Error | Meaning |
|-------|---------|
| `Provider(msg)` | Transport/API error |
| `Hallucination(msg)` | Output not valid TDLN JSON |
| `ContextOverflow` | Context window exceeded |
| `Parsing(msg)` | Malformed JSON |
| `Render(msg)` | Prompt rendering error |

## Guarantees

- ✅ `#![forbid(unsafe_code)]`
- ✅ Strict parse: invalid JSON → `BrainError::Hallucination`
- ✅ Optional reasoning: model can think, but JSON wins
- ✅ Usage metadata: token counts for budgeting

## OS Integration

- **With `ubl-mcp`**: feed `Decision.intent` into tool selection; preflight via Gate; audit via Ledger
- **With `ubl-office`**: run `Brain::reason` in the OODA loop; persist to ledger; schedule Dreaming separately
- **Token budget**: use `util::clamp_budget` to manage context windows

## License

MIT — See [LICENSE](LICENSE)
