//! Basic loop example for ubl-office
//!
//! Demonstrates: Office creation, open, step, dream cycle

use serde_json::json;
use tdln_brain::MockBackend;
use ubl_office::{DreamConfig, Office, OfficeConfig, SessionMode, SessionType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) Create mock brain that always returns a "respond" intent
    let brain = MockBackend::with_intent(
        "respond",
        json!({
            "message": "Hello from the agent!"
        }),
    );

    // 2) Configure the office
    let config = OfficeConfig {
        tenant_id: "demo-agent".into(),
        session_type: SessionType::Work,
        session_mode: SessionMode::Commitment,
        dream: DreamConfig {
            dream_every_n_cycles: 5,
            dream_min_interval_secs: 1,
        },
        step_pause_ms: 100,
        max_consecutive_errors: 3,
        ..Default::default()
    };

    // 3) Create office with state receiver
    let (mut office, mut state_rx) = Office::new(config, brain);

    println!("Initial state: {:?}", office.state());

    // 4) Open the office
    office.open().await?;
    println!("After open: {:?}", office.state());

    // 5) Run a few steps
    for i in 1..=3 {
        println!("\n=== Step {i} ===");

        let intent = office.step(Some(&format!("User message {i}"))).await?;

        if let Some(intent) = intent {
            println!("Intent kind: {}", intent.kind);
            println!("Intent slots: {:?}", intent.slots);
        }

        println!("Metrics: steps={}, decisions={}", 
            office.metrics().steps_total,
            office.metrics().decisions_total
        );
    }

    // 6) Manually trigger dream
    println!("\n=== Dreaming ===");
    office.dream().await?;
    println!("Memory consolidated");

    // 7) Check state via receiver
    if state_rx.changed().await.is_ok() {
        println!("State changed to: {:?}", *state_rx.borrow());
    }

    // 8) Shutdown
    office.shutdown();
    println!("\nFinal state: {:?}", office.state());

    Ok(())
}
