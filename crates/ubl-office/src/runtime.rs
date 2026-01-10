//! Office runtime: the main agent loop.

use crate::{MemorySystem, Narrator, NarratorConfig, OfficeError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};
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

/// Session type determines allowed behaviors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SessionType {
    /// Full autonomous work: may sign & act.
    #[default]
    Work,
    /// Propose only, never act.
    Assist,
    /// Think-only; no tool calls.
    Deliberate,
    /// Read-only tools allowed; summarize with citations.
    Research,
}

/// Session mode determines commitment level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SessionMode {
    /// Actions are binding; receipts written.
    #[default]
    Commitment,
    /// Proposals only; nothing executed.
    Deliberation,
}

/// Token budget configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// Max input tokens per decision.
    pub max_input_tokens: u32,
    /// Max output tokens per decision.
    pub max_output_tokens: u32,
    /// Daily token quota per entity.
    pub daily_token_quota: u64,
    /// Max decisions per cycle.
    pub max_decisions_per_cycle: u32,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            max_input_tokens: 4000,
            max_output_tokens: 1024,
            daily_token_quota: 200_000,
            max_decisions_per_cycle: 1,
        }
    }
}

/// Dreaming (maintenance) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamConfig {
    /// Dream every N cycles.
    pub dream_every_n_cycles: u64,
    /// Minimum interval between dreams (seconds).
    pub dream_min_interval_secs: u64,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            dream_every_n_cycles: 100,
            dream_min_interval_secs: 900, // 15 minutes
        }
    }
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
    /// Path to ledger file (NDJSON).
    pub ledger_path: Option<PathBuf>,
    /// Model ID for the brain.
    pub model_id: String,
    /// Session type.
    pub session_type: SessionType,
    /// Session mode.
    pub session_mode: SessionMode,
    /// Token budget.
    pub budget: TokenBudget,
    /// Dreaming configuration.
    pub dream: DreamConfig,
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
            ledger_path: None,
            model_id: "mock".into(),
            session_type: SessionType::default(),
            session_mode: SessionMode::default(),
            budget: TokenBudget::default(),
            dream: DreamConfig::default(),
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
    /// Total input tokens consumed today.
    pub input_tokens_today: u64,
    /// Total output tokens consumed today.
    pub output_tokens_today: u64,
    /// Dreams completed.
    pub dreams_total: u64,
    /// Decisions since last dream.
    pub decisions_since_dream: u64,
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
    last_dream_time: Option<Instant>,
}

impl<B: NeuralBackend> Office<B> {
    /// Create a new Office with the given config and brain.
    ///
    /// Returns the Office and a state receiver for monitoring.
    pub fn new(config: OfficeConfig, brain: B) -> (Self, watch::Receiver<OfficeState>) {
        let (state_tx, state_rx) = watch::channel(OfficeState::Opening);

        let narrator_config = NarratorConfig {
            system_directive: format!(
                "IDENTITY: {}\nSESSION: {:?}/{:?}\nYou are an autonomous professional agent in LogLine OS.\nOutput MUST be a valid TDLN SemanticUnit JSON.",
                config.tenant_id, config.session_type, config.session_mode
            ),
            constraints: Self::build_constraints(&config),
            constitution: None,
            session_type: config.session_type,
            session_mode: config.session_mode,
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
            last_dream_time: None,
        };

        (office, state_rx)
    }

    fn build_constraints(config: &OfficeConfig) -> Vec<String> {
        let mut constraints = vec![
            "Never execute write tools unless Gate:Permit".into(),
            "Prefer simulate() for risk_score ≥ 0.7".into(),
        ];

        match config.session_type {
            SessionType::Work => {
                constraints.push("May sign & act; write receipts for all actions".into());
            }
            SessionType::Assist => {
                constraints.push("Propose only, never act; include remediation steps".into());
            }
            SessionType::Deliberate => {
                constraints.push("Think-only; do NOT call tools".into());
            }
            SessionType::Research => {
                constraints.push("Read-only tools allowed; summarize sources with citations".into());
            }
        }

        match config.session_mode {
            SessionMode::Commitment => {
                constraints.push("Actions are binding; all decisions produce receipts".into());
            }
            SessionMode::Deliberation => {
                constraints.push("Proposals only; nothing is executed".into());
            }
        }

        constraints
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

    /// Get mutable metrics.
    pub fn metrics_mut(&mut self) -> &mut OfficeMetrics {
        &mut self.metrics
    }

    /// Get config.
    #[must_use]
    pub fn config(&self) -> &OfficeConfig {
        &self.config
    }

    /// Get memory system.
    #[must_use]
    pub fn memory(&self) -> &MemorySystem {
        &self.memory
    }

    /// Get mutable memory system.
    pub fn memory_mut(&mut self) -> &mut MemorySystem {
        &mut self.memory
    }

    fn set_state(&mut self, state: OfficeState) {
        self.state = state;
        let _ = self.state_tx.send(state);
    }

    /// Check if dreaming is needed.
    fn needs_dream(&self) -> bool {
        // Check cycle count
        if self.metrics.decisions_since_dream >= self.config.dream.dream_every_n_cycles {
            // Check minimum interval
            if let Some(last) = self.last_dream_time {
                let min_interval = Duration::from_secs(self.config.dream.dream_min_interval_secs);
                return last.elapsed() >= min_interval;
            }
            return true;
        }
        false
    }

    /// Check token budget before a decision.
    fn check_budget(&self) -> Result<(), OfficeError> {
        // Check daily quota
        let total_today = self.metrics.input_tokens_today + self.metrics.output_tokens_today;
        if total_today >= self.config.budget.daily_token_quota {
            return Err(OfficeError::QuotaExceeded(format!(
                "daily quota exceeded: {} >= {}",
                total_today, self.config.budget.daily_token_quota
            )));
        }
        Ok(())
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
        if self.needs_dream() {
            self.dream().await?;
        }

        // Check budget
        self.check_budget()?;

        // Add user input to history if present
        if let Some(input) = input {
            self.history.push(Message::user(input));
            self.memory.remember(format!("User: {input}"));
        }

        // Orient: build cognitive context
        let ctx = self.narrator.orient(&self.memory, self.history.clone());

        // Build generation config with budget limits
        let gen_config = GenerationConfig {
            max_tokens: Some(self.config.budget.max_output_tokens),
            ..GenerationConfig::default()
        };

        // Decide: call brain
        let messages = ctx.render();
        let raw = self.brain.generate(&messages, &gen_config).await?;

        // Track token usage
        self.metrics.input_tokens_today += u64::from(raw.meta.input_tokens);
        self.metrics.output_tokens_today += u64::from(raw.meta.output_tokens);

        // Parse decision
        let decision = tdln_brain::parser::parse_decision(&raw.content, raw.meta)?;
        self.metrics.decisions_total += 1;
        self.metrics.decisions_since_dream += 1;
        self.metrics.consecutive_errors = 0;

        // Remember the decision
        self.memory.remember(format!("Decision: {}", decision.intent.kind));
        self.history.push(Message::assistant(&raw.content));

        // Increment narrator maintenance counter
        self.narrator.increment_maintenance_counter();

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

        // Reset counters
        self.metrics.decisions_since_dream = 0;
        self.metrics.dreams_total += 1;
        self.last_dream_time = Some(Instant::now());
        self.narrator.reset_maintenance_counter();

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
                Err(OfficeError::QuotaExceeded(msg)) => {
                    tracing::warn!("quota exceeded: {msg}");
                    // Don't increment error counter for quota; just skip
                    continue;
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

    /// Check if session allows tool execution.
    #[must_use]
    pub fn can_execute_tools(&self) -> bool {
        match self.config.session_type {
            SessionType::Work => true,
            SessionType::Research => true, // read-only
            SessionType::Assist | SessionType::Deliberate => false,
        }
    }

    /// Check if session allows write operations.
    #[must_use]
    pub fn can_write(&self) -> bool {
        matches!(
            (&self.config.session_type, &self.config.session_mode),
            (SessionType::Work, SessionMode::Commitment)
        )
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
