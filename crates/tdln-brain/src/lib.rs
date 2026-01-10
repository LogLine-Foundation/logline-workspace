//! `tdln-brain` — Deterministic Cognitive Layer for LogLine OS
//!
//! NL → TDLN SemanticUnit → canonical bytes (via json_atomic) → happy Gate → verifiable execution.
//!
//! This crate is the cognitive shim between LLMs and the LogLine kernel. It renders
//! a typed cognitive context, asks a model for an intent, and guarantees you can parse
//! it into a [`tdln_ast::SemanticUnit`] with zero ambiguity.
//!
//! # Invariants
//!
//! - **Strict output**: JSON that parses into a `SemanticUnit` or it's a hard error
//! - **Kernel awareness**: constraints (policies) visible before generation
//! - **Deterministic canon**: one source of truth for canonical bytes (delegates to `json_atomic`)
//!
//! # Example
//!
//! ```rust
//! use tdln_brain::{CognitiveContext, Message, parser};
//!
//! let ctx = CognitiveContext {
//!     system_directive: "You output VALID JSON for a TDLN SemanticUnit.".into(),
//!     recall: vec!["User balance: 420".into()],
//!     history: vec![Message::user("grant to alice amount 100")],
//!     constraints: vec!["Never transfer > 500 without approval".into()],
//! };
//!
//! let messages = ctx.render();
//! assert!(!messages.is_empty());
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod parser;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Re-export core AST type
pub use tdln_ast::SemanticUnit;

/// A chat message with role and content.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    /// Role: "system", "user", or "assistant"
    pub role: String,
    /// Message content
    pub content: String,
}

impl Message {
    /// Create a system message.
    #[must_use]
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }

    /// Create a user message.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }

    /// Create an assistant message.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
        }
    }
}

/// Cognitive context for prompt rendering.
///
/// Contains the system directive, recall (memory), conversation history,
/// and active constraints (policies) the model must respect.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CognitiveContext {
    /// The system directive (role + tone + boundaries).
    pub system_directive: String,
    /// Relevant memories for the current context.
    pub recall: Vec<String>,
    /// Recent conversation history.
    pub history: Vec<Message>,
    /// Active kernel constraints the model must respect.
    pub constraints: Vec<String>,
}

impl CognitiveContext {
    /// Render the context into a vector of messages suitable for LLM input.
    ///
    /// The system message packs:
    /// - Your directive
    /// - Constraints (kernel policies)
    /// - Relevant memory (recall)
    ///
    /// Then appends conversation history.
    #[must_use]
    pub fn render(&self) -> Vec<Message> {
        let mut system_parts = vec![self.system_directive.clone()];

        // Add output format instructions
        system_parts.push(String::from(
            "\n### OUTPUT FORMAT ###\n\
             Output a single JSON object with fields: kind, slots\n\
             Example: {\"kind\":\"grant\",\"slots\":{\"to\":\"alice\",\"amount\":100}}\n\
             Do NOT include any text before or after the JSON.",
        ));

        // Add constraints
        if !self.constraints.is_empty() {
            system_parts.push(String::from("\n### ACTIVE KERNEL CONSTRAINTS ###"));
            for c in &self.constraints {
                system_parts.push(format!("- {c}"));
            }
        }

        // Add recall
        if !self.recall.is_empty() {
            system_parts.push(String::from("\n### RELEVANT MEMORY (RECALL) ###"));
            for r in &self.recall {
                system_parts.push(format!("- {r}"));
            }
        }

        let system_content = system_parts.join("\n");
        let mut messages = vec![Message::system(system_content)];

        // Append history
        messages.extend(self.history.clone());

        messages
    }
}

/// Usage metadata from a generation request.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UsageMeta {
    /// Input token count (if available).
    pub input_tokens: u32,
    /// Output token count (if available).
    pub output_tokens: u32,
    /// Model identifier used.
    pub model_id: String,
}

/// Raw output from a neural backend.
#[derive(Clone, Debug)]
pub struct RawOutput {
    /// The raw content returned by the model.
    pub content: String,
    /// Usage metadata.
    pub meta: UsageMeta,
}

/// A parsed decision containing reasoning and a strict intent.
#[derive(Debug)]
pub struct Decision {
    /// Optional reasoning text extracted from the response.
    pub reasoning: Option<String>,
    /// The strictly-parsed TDLN intent.
    pub intent: SemanticUnit,
    /// Usage metadata from generation.
    pub meta: UsageMeta,
}

/// Configuration for generation requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Temperature (0.0 = deterministic).
    pub temperature: f32,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Whether to require reasoning in output.
    pub require_reasoning: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            max_tokens: Some(1024),
            require_reasoning: false,
        }
    }
}

/// Errors from the cognitive layer.
#[derive(Debug, thiserror::Error)]
pub enum BrainError {
    /// Transport or API error from the provider.
    #[error("provider error: {0}")]
    Provider(String),
    /// Model output was not valid TDLN JSON.
    #[error("hallucination: output was not valid TDLN: {0}")]
    Hallucination(String),
    /// Context window exceeded.
    #[error("context window exceeded")]
    ContextOverflow,
    /// JSON parsing error.
    #[error("parsing error: {0}")]
    Parsing(String),
}

/// Trait for model providers (LLM backends).
///
/// Implement this trait to plug in any LLM (cloud or local).
#[async_trait]
pub trait NeuralBackend: Send + Sync {
    /// Returns the model identifier.
    fn model_id(&self) -> &str;

    /// Generate a response from the given messages.
    async fn generate(
        &self,
        messages: &[Message],
        config: &GenerationConfig,
    ) -> Result<RawOutput, BrainError>;
}

/// A mock backend for testing that returns a fixed response.
pub struct MockBackend {
    response: String,
}

impl MockBackend {
    /// Create a mock backend with a fixed JSON response.
    #[must_use]
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }

    /// Create a mock backend with a valid SemanticUnit response.
    #[must_use]
    pub fn with_intent(kind: &str, slots: serde_json::Value) -> Self {
        let json = serde_json::json!({
            "kind": kind,
            "slots": slots
        });
        Self::new(json.to_string())
    }
}

#[async_trait]
impl NeuralBackend for MockBackend {
    fn model_id(&self) -> &str {
        "mock-tdln"
    }

    async fn generate(
        &self,
        _messages: &[Message],
        _config: &GenerationConfig,
    ) -> Result<RawOutput, BrainError> {
        Ok(RawOutput {
            content: self.response.clone(),
            meta: UsageMeta {
                model_id: "mock-tdln".into(),
                input_tokens: 0,
                output_tokens: self.response.len() as u32 / 4,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_includes_constraints() {
        let ctx = CognitiveContext {
            system_directive: "You are a TDLN brain.".into(),
            recall: vec![],
            history: vec![],
            constraints: vec!["No transfers over 500".into()],
        };
        let msgs = ctx.render();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].content.contains("No transfers over 500"));
    }

    #[test]
    fn render_includes_recall() {
        let ctx = CognitiveContext {
            system_directive: "Brain".into(),
            recall: vec!["User balance: 420".into()],
            history: vec![Message::user("hi")],
            constraints: vec![],
        };
        let msgs = ctx.render();
        assert_eq!(msgs.len(), 2);
        assert!(msgs[0].content.contains("User balance: 420"));
    }

    #[test]
    fn stable_render() {
        let ctx = CognitiveContext {
            system_directive: "Test".into(),
            recall: vec!["mem".into()],
            history: vec![Message::user("hello")],
            constraints: vec!["rule".into()],
        };
        let a = ctx.render();
        let b = ctx.render();
        assert_eq!(a, b);
    }
}
