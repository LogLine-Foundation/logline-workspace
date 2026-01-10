//! Narrator: builds CognitiveContext for the brain.

use crate::MemorySystem;
use tdln_brain::{CognitiveContext, Message};

/// Narrator configuration.
#[derive(Clone, Debug)]
pub struct NarratorConfig {
    /// System directive (identity + role).
    pub system_directive: String,
    /// Active constraints (kernel policies).
    pub constraints: Vec<String>,
    /// Constitution text (appended at end).
    pub constitution: Option<String>,
}

impl Default for NarratorConfig {
    fn default() -> Self {
        Self {
            system_directive: "You are a LogLine agent. Output valid TDLN JSON.".into(),
            constraints: vec![],
            constitution: None,
        }
    }
}

/// Narrator builds cognitive context for the brain.
pub struct Narrator {
    config: NarratorConfig,
}

impl Narrator {
    /// Create a new narrator with the given config.
    #[must_use]
    pub fn new(config: NarratorConfig) -> Self {
        Self { config }
    }

    /// Create with default config.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(NarratorConfig::default())
    }

    /// Orient: build a CognitiveContext from current state.
    pub fn orient(&self, memory: &MemorySystem, history: Vec<Message>) -> CognitiveContext {
        let mut constraints = self.config.constraints.clone();

        // Append constitution as final constraint if present
        if let Some(ref constitution) = self.config.constitution {
            constraints.push(format!("CONSTITUTION: {constitution}"));
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
}
