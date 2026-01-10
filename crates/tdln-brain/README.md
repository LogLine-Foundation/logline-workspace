# tdln-brain

**Deterministic Cognitive Layer for LogLine OS**

NL → TDLN `SemanticUnit` → canonical bytes (via `json_atomic`) → happy Gate → verifiable execution.

[![Crates.io](https://img.shields.io/crates/v/tdln-brain.svg)](https://crates.io/crates/tdln-brain)
[![Documentation](https://docs.rs/tdln-brain/badge.svg)](https://docs.rs/tdln-brain)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## What is this?

`tdln-brain` is the cognitive shim between LLMs and the LogLine kernel. It:

1. **Renders** a typed `CognitiveContext` (system directive, recall, constraints) into LLM-ready messages
2. **Parses** model output into a strict `SemanticUnit` — or returns a hard error
3. **Separates** reasoning from action (free-form text vs. strict JSON)

### Invariants

- **Strict output**: JSON that parses into a `SemanticUnit` or it's a `BrainError::Hallucination`
- **Kernel awareness**: constraints (policies) visible before generation, reducing Gate rejections
- **Deterministic canon**: one source of truth for canonical bytes (delegates to `json_atomic`)

## Quickstart

```rust
use tdln_brain::{CognitiveContext, Message, MockBackend, NeuralBackend, GenerationConfig, parser};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) Prepare cognitive context
    let ctx = CognitiveContext {
        system_directive: "You output VALID JSON for a TDLN SemanticUnit.".into(),
        recall: vec!["User balance: 420".into()],
        history: vec![Message::user("grant to alice amount 100")],
        constraints: vec!["Never transfer > 500 without approval".into()],
    };

    // 2) Render messages for the model
    let messages = ctx.render();

    // 3) Use any NeuralBackend (here: mock for testing)
    let backend = MockBackend::with_intent("grant", json!({"to": "alice", "amount": 100}));
    let raw = backend.generate(&messages, &GenerationConfig::default()).await?;

    // 4) Parse into strict Decision
    let decision = parser::parse_decision(&raw.content, raw.meta)?;
    
    println!("Intent kind: {}", decision.intent.kind);
    println!("Intent CID: {}", hex::encode(decision.intent.cid_blake3()));
    
    Ok(())
}
```

## API Overview

### Core Types

```rust
// Cognitive context for prompt rendering
struct CognitiveContext {
    system_directive: String,
    recall: Vec<String>,
    history: Vec<Message>,
    constraints: Vec<String>,
}

// Chat message
struct Message { role: String, content: String }

// Parsed decision
struct Decision {
    reasoning: Option<String>,
    intent: SemanticUnit,
    meta: UsageMeta,
}
```

### NeuralBackend Trait

Implement this to plug in any LLM:

```rust
#[async_trait]
trait NeuralBackend: Send + Sync {
    fn model_id(&self) -> &str;
    async fn generate(&self, messages: &[Message], config: &GenerationConfig) 
        -> Result<RawOutput, BrainError>;
}
```

### Parser

```rust
// Extract JSON from raw LLM output, parse into SemanticUnit
fn parse_decision(raw: &str, meta: UsageMeta) -> Result<Decision, BrainError>;
```

Handles:
- Clean JSON: `{"kind":"grant",...}`
- Fenced blocks: ` ```json {...} ``` `
- Mixed prose + JSON

## Error Model

| Error | Meaning |
|-------|---------|
| `Provider(msg)` | Transport/API error |
| `Hallucination(msg)` | Output not valid TDLN JSON |
| `ContextOverflow` | Context window exceeded |
| `Parsing(msg)` | Malformed JSON |

## Features

- `default` — Core functionality
- `http-drivers` — Includes `reqwest` for HTTP-based backends

## Security

- `#![forbid(unsafe_code)]`
- No implicit decisions — invalid output = hard error
- Canon chain downstream (hash at compile/proof stage, not here)

## License

MIT — See [LICENSE](LICENSE)
