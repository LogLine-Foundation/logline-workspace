//! Office runtime: the main agent loop.

use crate::{MemorySystem, Narrator, NarratorConfig, OfficeError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tdln_brain::{GenerationConfig, Message, NeuralBackend};
use tokio::sync::watch;

/// Agent lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficeState {
    /// Bootstrapping identity/IO.
    Opening,
    /// Active OODA loop.
    Active,
    /// Dreaming / consolidation / compaction.
    Maintenance,
    /// Shutdown with flush.
    Closing,
}

/// Runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeConfig {
    /// Tenant/agent identity.
    pub tenant_id: String,
    /// Path to constitution / policy bundle.
    pub constitution_path: Option<PathBuf>,
    /// Workspace root.
    pub workspace_root: PathBuf,
    /// Model ID for the brain.
    pub model_id: String,
    /// Steps before entering maintenance (dream).
    pub max_steps_before_dream: u64,
    /// Pause between steps (milliseconds).
    pub step_pause_ms: u64,
    /// Maximum consecutive errors before abort.
    pub max_consecutive_errors: u32,
}

impl Default for OfficeConfig {
    fn default() -> Self {
        Self {
            tenant_id: "agent".into(),
            constitution_path: None,
            workspace_root: PathBuf::from("."),
            model_id: "mock".into(),
            max_steps_before_dream: 50,
            step_pause_ms: 1000,
            max_consecutive_errors: 5,
        }
    }
}

/// Structured metrics for the runtime.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OfficeMetrics {
    /// Total steps executed.
    pub steps_total: u64,
    /// Total decisions made.
    pub decisions_total: u64,
    /// Total denials from Gate.
    pub denials_total: u64,
    /// Total challenges from Gate.
    pub challenges_total: u64,
    /// Total tool errors.
    pub tool_errors_total: u64,
    /// Consecutive errors (resets on success).
    pub consecutive_errors: u32,
}

/// The Office runtime.
pub struct Office<B: NeuralBackend> {
    config: OfficeConfig,
    brain: B,
    memory: MemorySystem,
    narrator: Narrator,
    metrics: OfficeMetrics,
    state: OfficeState,
    state_tx: watch::Sender<OfficeState>,
    history: Vec<Message>,
}

impl<B: NeuralBackend> Office<B> {
    /// Create a new Office with the given config and brain.
    ///
    /// Returns the Office and a state receiver for monitoring.
    pub fn new(config: OfficeConfig, brain: B) -> (Self, watch::Receiver<OfficeState>) {
        let (state_tx, state_rx) = watch::channel(OfficeState::Opening);

        let narrator_config = NarratorConfig {
            system_directive: format!(
                "You are agent '{}'. Output valid TDLN JSON for actions.",
                config.tenant_id
            ),
            constraints: vec![],
            constitution: None,
        };

        let office = Self {
            config,
            brain,
            memory: MemorySystem::new(),
            narrator: Narrator::new(narrator_config),
            metrics: OfficeMetrics::default(),
            state: OfficeState::Opening,
            state_tx,
            history: Vec::new(),
        };

        (office, state_rx)
    }

    /// Get current state.
    #[must_use]
    pub fn state(&self) -> OfficeState {
        self.state
    }

    /// Get metrics.
    #[must_use]
    pub fn metrics(&self) -> &OfficeMetrics {
        &self.metrics
    }

    /// Get config.
    #[must_use]
    pub fn config(&self) -> &OfficeConfig {
        &self.config
    }

    fn set_state(&mut self, state: OfficeState) {
        self.state = state;
        let _ = self.state_tx.send(state);
    }

    /// Open the office: load constitution, initialize.
    pub async fn open(&mut self) -> Result<(), OfficeError> {
        self.set_state(OfficeState::Opening);

        // Load constitution if specified
        if let Some(ref path) = self.config.constitution_path {
            if path.exists() {
                let constitution = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| OfficeError::Config(format!("failed to load constitution: {e}")))?;
                self.narrator.set_constitution(constitution);
            }
        }

        self.set_state(OfficeState::Active);
        Ok(())
    }

    /// Run one OODA step.
    ///
    /// Returns the intent (if any) or an error.
    pub async fn step(&mut self, input: Option<&str>) -> Result<Option<tdln_ast::SemanticUnit>, OfficeError> {
        if self.state != OfficeState::Active {
            return Ok(None);
        }

        self.metrics.steps_total += 1;

        // Check if we need to dream
        if self.metrics.steps_total % self.config.max_steps_before_dream == 0 {
            self.dream().await?;
        }

        // Add user input to history if present
        if let Some(input) = input {
            self.history.push(Message::user(input));
            self.memory.remember(format!("User: {input}"));
        }

        // Orient: build cognitive context
        let ctx = self.narrator.orient(&self.memory, self.history.clone());

        // Decide: call brain
        let messages = ctx.render();
        let raw = self
            .brain
            .generate(&messages, &GenerationConfig::default())
            .await?;

        // Parse decision
        let decision = tdln_brain::parser::parse_decision(&raw.content, raw.meta)?;
        self.metrics.decisions_total += 1;
        self.metrics.consecutive_errors = 0;

        // Remember the decision
        self.memory.remember(format!("Decision: {}", decision.intent.kind));
        self.history.push(Message::assistant(&raw.content));

        Ok(Some(decision.intent))
    }

    /// Enter maintenance mode (dreaming).
    pub async fn dream(&mut self) -> Result<(), OfficeError> {
        let prev_state = self.state;
        self.set_state(OfficeState::Maintenance);

        // Consolidate memory
        let events: Vec<String> = self.memory.recent(20);
        self.memory.consolidate(&events);

        // Trim history
        if self.history.len() > 20 {
            self.history = self.history.split_off(self.history.len() - 10);
        }

        self.set_state(prev_state);
        Ok(())
    }

    /// Run the main loop until shutdown or fatal error.
    pub async fn run(mut self) -> Result<(), OfficeError> {
        self.open().await?;

        loop {
            // Pause between steps
            tokio::time::sleep(Duration::from_millis(self.config.step_pause_ms)).await;

            match self.step(None).await {
                Ok(_) => {}
                Err(OfficeError::Shutdown) => {
                    self.set_state(OfficeState::Closing);
                    break;
                }
                Err(e) => {
                    self.metrics.consecutive_errors += 1;
                    tracing::warn!("step error: {e}");

                    if self.metrics.consecutive_errors >= self.config.max_consecutive_errors {
                        self.set_state(OfficeState::Closing);
                        return Err(e);
                    }

                    // Exponential backoff
                    let delay = Duration::from_millis(
                        100 * (2_u64.pow(self.metrics.consecutive_errors.min(6))),
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Ok(())
    }

    /// Shutdown the office gracefully.
    pub fn shutdown(&mut self) {
        self.set_state(OfficeState::Closing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tdln_brain::MockBackend;

    #[tokio::test]
    async fn state_transitions() {
        let backend = MockBackend::with_intent("noop", json!({}));
        let (mut office, rx) = Office::new(OfficeConfig::default(), backend);

        assert_eq!(office.state(), OfficeState::Opening);
        assert_eq!(*rx.borrow(), OfficeState::Opening);

        office.open().await.unwrap();
        assert_eq!(office.state(), OfficeState::Active);
    }

    #[tokio::test]
    async fn step_produces_intent() {
        let backend = MockBackend::with_intent("greet", json!({"name": "alice"}));
        let (mut office, _) = Office::new(OfficeConfig::default(), backend);
        office.open().await.unwrap();

        let intent = office.step(Some("say hello")).await.unwrap();
        assert!(intent.is_some());
        assert_eq!(intent.unwrap().kind, "greet");
    }

    #[tokio::test]
    async fn dream_consolidates() {
        let backend = MockBackend::with_intent("noop", json!({}));
        let (mut office, _) = Office::new(OfficeConfig::default(), backend);
        office.open().await.unwrap();

        // Add some memory
        for i in 0..80 {
            office.memory.remember(format!("event {i}"));
        }
        let before = office.memory.short_term_len();

        office.dream().await.unwrap();
        
        // Consolidate should reduce memory
        assert!(office.memory.short_term_len() < before);
    }

    #[tokio::test]
    async fn metrics_increment() {
        let backend = MockBackend::with_intent("test", json!({}));
        let (mut office, _) = Office::new(OfficeConfig::default(), backend);
        office.open().await.unwrap();

        office.step(Some("test")).await.unwrap();
        assert_eq!(office.metrics().steps_total, 1);
        assert_eq!(office.metrics().decisions_total, 1);
    }
}
