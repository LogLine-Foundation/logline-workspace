//! Narrator: builds CognitiveContext for the brain.

use crate::runtime::{SessionMode, SessionType};
use crate::MemorySystem;
use serde::{Deserialize, Serialize};
use tdln_brain::{CognitiveContext, Message};

/// Narrator configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NarratorConfig {
    /// System directive (identity + role).
    pub system_directive: String,
    /// Active constraints (kernel policies).
    pub constraints: Vec<String>,
    /// Constitution text (appended at end).
    pub constitution: Option<String>,
    /// Session type.
    pub session_type: SessionType,
    /// Session mode.
    pub session_mode: SessionMode,
}

impl Default for NarratorConfig {
    fn default() -> Self {
        Self {
            system_directive: "You are a LogLine agent. Output valid TDLN JSON.".into(),
            constraints: vec![],
            constitution: None,
            session_type: SessionType::default(),
            session_mode: SessionMode::default(),
        }
    }
}

/// Narrator builds cognitive context for the brain.
pub struct Narrator {
    config: NarratorConfig,
    /// Counter for maintenance/dreaming triggers.
    maintenance_counter: u64,
    /// Max cycles before maintenance suggestion.
    maintenance_threshold: u64,
}

impl Narrator {
    /// Create a new narrator with the given config.
    #[must_use]
    pub fn new(config: NarratorConfig) -> Self {
        Self {
            config,
            maintenance_counter: 0,
            maintenance_threshold: 100,
        }
    }

    /// Create with default config.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(NarratorConfig::default())
    }

    /// Check if maintenance is needed.
    #[must_use]
    pub fn needs_maintenance(&self) -> bool {
        self.maintenance_counter >= self.maintenance_threshold
    }

    /// Increment the maintenance counter.
    pub fn increment_maintenance_counter(&mut self) {
        self.maintenance_counter += 1;
    }

    /// Reset the maintenance counter (after dreaming).
    pub fn reset_maintenance_counter(&mut self) {
        self.maintenance_counter = 0;
    }

    /// Get current maintenance counter value.
    #[must_use]
    pub fn maintenance_counter(&self) -> u64 {
        self.maintenance_counter
    }

    /// Orient: build a CognitiveContext from current state.
    pub fn orient(&self, memory: &MemorySystem, history: Vec<Message>) -> CognitiveContext {
        let mut constraints = self.config.constraints.clone();

        // Session type constraints
        match self.config.session_type {
            SessionType::Work => {
                constraints.push("SESSION: Work mode - may execute and sign actions".into());
            }
            SessionType::Assist => {
                constraints.push("SESSION: Assist mode - propose only, do NOT execute".into());
            }
            SessionType::Deliberate => {
                constraints.push("SESSION: Deliberate mode - think only, do NOT call tools".into());
            }
            SessionType::Research => {
                constraints
                    .push("SESSION: Research mode - read-only tools, cite all sources".into());
            }
        }

        // Session mode constraint
        if self.config.session_mode == SessionMode::Deliberation {
            constraints.push("MODE: Deliberation - nothing will be executed".into());
        }

        // Append constitution as final constraint if present
        if let Some(ref constitution) = self.config.constitution {
            constraints.push(format!("CONSTITUTION: {constitution}"));
        }

        // Maintenance warning
        if self.needs_maintenance() {
            constraints.push("WARNING: Maintenance threshold reached. Consider dreaming.".into());
        }

        CognitiveContext {
            system_directive: self.config.system_directive.clone(),
            recall: memory.recall(""),
            history,
            constraints,
        }
    }

    /// Get the system directive.
    #[must_use]
    pub fn system_directive(&self) -> &str {
        &self.config.system_directive
    }

    /// Add a constraint.
    pub fn add_constraint(&mut self, constraint: impl Into<String>) {
        self.config.constraints.push(constraint.into());
    }

    /// Set the constitution.
    pub fn set_constitution(&mut self, constitution: impl Into<String>) {
        self.config.constitution = Some(constitution.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orient_includes_recall() {
        let mut mem = MemorySystem::new();
        mem.remember("User said hello");
        mem.remember("Balance is 100");

        let narrator = Narrator::with_defaults();
        let ctx = narrator.orient(&mem, vec![]);

        assert!(!ctx.recall.is_empty());
    }

    #[test]
    fn orient_includes_constitution() {
        let mut narrator = Narrator::with_defaults();
        narrator.set_constitution("Always be helpful");

        let mem = MemorySystem::new();
        let ctx = narrator.orient(&mem, vec![]);

        assert!(ctx.constraints.iter().any(|c| c.contains("CONSTITUTION")));
    }

    #[test]
    fn orient_includes_history() {
        let narrator = Narrator::with_defaults();
        let mem = MemorySystem::new();
        let history = vec![Message::user("hello"), Message::assistant("hi")];

        let ctx = narrator.orient(&mem, history);
        assert_eq!(ctx.history.len(), 2);
    }

    #[test]
    fn orient_includes_session_constraints() {
        let config = NarratorConfig {
            session_type: SessionType::Research,
            session_mode: SessionMode::Deliberation,
            ..Default::default()
        };
        let narrator = Narrator::new(config);
        let mem = MemorySystem::new();
        let ctx = narrator.orient(&mem, vec![]);

        assert!(ctx.constraints.iter().any(|c| c.contains("Research")));
        assert!(ctx.constraints.iter().any(|c| c.contains("Deliberation")));
    }

    #[test]
    fn maintenance_counter_works() {
        let mut narrator = Narrator::with_defaults();
        assert!(!narrator.needs_maintenance());

        for _ in 0..100 {
            narrator.increment_maintenance_counter();
        }
        assert!(narrator.needs_maintenance());

        narrator.reset_maintenance_counter();
        assert!(!narrator.needs_maintenance());
    }
}
