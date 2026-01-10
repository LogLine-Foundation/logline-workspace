# ubl-office

**The Agent Runtime (Wake · Work · Dream)**

Run agents like reliable services, not fragile notebooks.

[![Crates.io](https://img.shields.io/crates/v/ubl-office.svg)](https://crates.io/crates/ubl-office)
[![Documentation](https://docs.rs/ubl-office/badge.svg)](https://docs.rs/ubl-office)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## What is this?

`ubl-office` is the execution environment for LogLine agents. It coordinates thinking (TDLN), acting (MCP tools), memory, and policy (Gate) under one tight loop. No root access, no mystery state, no shrug emojis.

### The Loop

1. **Boot**: load identity/constitution, attach transports, warm caches
2. **Orient**: build a typed `CognitiveContext` (system directive, recall, constraints)
3. **Decide**: call `tdln-brain` to produce a strict `SemanticUnit` (TDLN AST)
4. **Gate**: run `tdln-gate` → Permit | Deny | Challenge
5. **Act**: execute via `ubl-mcp` (MCP tools)
6. **Dream**: consolidate short-term into durable memory; compact context
7. **Repeat**, with backpressure, watchdog timers, exponential backoff on failure

## Quickstart

```rust
use ubl_office::{Office, OfficeConfig, OfficeState};
use tdln_brain::MockBackend;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) Create brain (mock for testing)
    let brain = MockBackend::with_intent("greet", json!({"name": "world"}));
    
    // 2) Configure office
    let config = OfficeConfig {
        tenant_id: "my-agent".into(),
        max_steps_before_dream: 10,
        step_pause_ms: 100,
        ..Default::default()
    };
    
    // 3) Create office
    let (mut office, state_rx) = Office::new(config, brain);
    
    // 4) Open and run a step
    office.open().await?;
    assert_eq!(office.state(), OfficeState::Active);
    
    let intent = office.step(Some("hello")).await?;
    println!("Got intent: {:?}", intent);
    
    // 5) Check metrics
    println!("Steps: {}", office.metrics().steps_total);
    println!("Decisions: {}", office.metrics().decisions_total);
    
    Ok(())
}
```

## API Overview

### States

```rust
enum OfficeState {
    Opening,     // Bootstrapping
    Active,      // OODA loop running
    Maintenance, // Dreaming / consolidation
    Closing,     // Shutdown with flush
}
```

### Configuration

```rust
struct OfficeConfig {
    tenant_id: String,           // Agent identity
    constitution_path: Option<PathBuf>, // Policy file
    workspace_root: PathBuf,     // Working directory
    model_id: String,            // Brain model ID
    max_steps_before_dream: u64, // Steps before maintenance
    step_pause_ms: u64,          // Delay between steps
    max_consecutive_errors: u32, // Error threshold
}
```

### Office

```rust
impl Office<B: NeuralBackend> {
    fn new(config: OfficeConfig, brain: B) -> (Self, Receiver<OfficeState>);
    async fn open(&mut self) -> Result<(), OfficeError>;
    async fn step(&mut self, input: Option<&str>) -> Result<Option<SemanticUnit>, OfficeError>;
    async fn dream(&mut self) -> Result<(), OfficeError>;
    async fn run(self) -> Result<(), OfficeError>;
    fn shutdown(&mut self);
}
```

### Memory

```rust
impl MemorySystem {
    fn remember(&mut self, note: impl Into<String>);
    fn recall(&self, signal: &str) -> Vec<String>;
    fn consolidate(&mut self, events: &[String]);
}
```

## Error Model

| Error | Meaning |
|-------|---------|
| `Brain(msg)` | Cognition/LLM error |
| `Gate(msg)` | Policy violation |
| `Tool(msg)` | MCP tool error |
| `Io(msg)` | File/network error |
| `Config(msg)` | Configuration error |
| `Shutdown` | Clean shutdown requested |

## Reliability

- **Exponential backoff**: consecutive errors increase delay (capped)
- **Automatic dreaming**: consolidates memory every N steps
- **State broadcasting**: watch channel for external monitoring
- **Graceful shutdown**: flush and close cleanly

## Security

- `#![forbid(unsafe_code)]`
- Gate-first execution (no action without policy check)
- Constitution as immutable constraint
- Memory hygiene (consolidation, trimming)

## License

MIT — See [LICENSE](LICENSE)
