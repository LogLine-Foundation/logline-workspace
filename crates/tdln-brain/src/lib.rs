//! `tdln-brain` — Deterministic Cognitive Layer for LogLine OS
//!
//! Render a narrative frame → call an LLM provider → extract **only** JSON →
//! validate into `tdln_ast::SemanticUnit`.
//!
//! # Why
//!
//! - Prevent tool-call hallucinations
//! - Enforce JSON-only outputs
//! - Keep reasoning optional and separated
//! - Make failures machine-legible
//!
//! # Example
//!
//! ```rust,no_run
//! use tdln_brain::{Brain, CognitiveContext, GenerationConfig};
//! use tdln_brain::providers::local::LocalEcho;
//!
//! # async fn example() -> Result<(), tdln_brain::BrainError> {
//! let brain = Brain::new(LocalEcho);
//! let ctx = CognitiveContext {
//!     system_directive: "You are a deterministic planner.".into(),
//!     ..Default::default()
//! };
//! let decision = brain.reason(&ctx, &GenerationConfig::default()).await?;
//! println!("{:?}", decision.intent);
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod parser;
pub mod prompt;
pub mod providers;
pub mod util;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-export core AST type
pub use tdln_ast::SemanticUnit;

// ══════════════════════════════════════════════════════════════════════════════
// Errors
// ══════════════════════════════════════════════════════════════════════════════

/// Errors from the cognitive layer.
#[derive(Debug, Error)]
pub enum BrainError {
    /// Transport or API error from the provider.
    #[error("provider error: {0}")]
    Provider(String),
    /// Model output was not valid TDLN JSON.
    #[error("hallucination: invalid TDLN JSON: {0}")]
    Hallucination(String),
    /// Context window exceeded.
    #[error("context window exceeded")]
    ContextOverflow,
    /// JSON parsing error.
    #[error("parsing error: {0}")]
    Parsing(String),
    /// Prompt rendering error.
    #[error("render error: {0}")]
    Render(String),
}

// ══════════════════════════════════════════════════════════════════════════════
// Types
// ══════════════════════════════════════════════════════════════════════════════

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
    pub fn system(s: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: s.into(),
        }
    }

    /// Create a user message.
    #[must_use]
    pub fn user(s: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: s.into(),
        }
    }

    /// Create an assistant message.
    #[must_use]
    pub fn assistant(s: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: s.into(),
        }
    }
}

/// Cognitive context for prompt rendering.
///
/// Contains the system directive, recall (memory), conversation history,
/// and active constraints (policies) the model must respect.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CognitiveContext {
    /// The system directive (identity + constitution + role).
    pub system_directive: String,
    /// Relevant memories for the current context (long-term recall).
    pub recall: Vec<String>,
    /// Recent conversation history.
    pub history: Vec<Message>,
    /// Active kernel constraints the model must respect.
    pub constraints: Vec<String>,
}

/// Configuration for generation requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Temperature (0.0 = deterministic).
    pub temperature: f32,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Whether to allow reasoning before JSON output.
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

// ══════════════════════════════════════════════════════════════════════════════
// Provider Interface
// ══════════════════════════════════════════════════════════════════════════════

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

// ══════════════════════════════════════════════════════════════════════════════
// Brain: End-to-End Reasoning
// ══════════════════════════════════════════════════════════════════════════════

/// The deterministic cognitive engine.
///
/// Wraps a [`NeuralBackend`] and provides the full pipeline:
/// render → generate → strict-parse → `SemanticUnit`.
pub struct Brain<B: NeuralBackend> {
    backend: B,
}

impl<B: NeuralBackend> Brain<B> {
    /// Create a new Brain with the given backend.
    #[must_use]
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Get a reference to the backend.
    #[must_use]
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Render → Generate → Strict-parse → `SemanticUnit`.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails, the provider fails,
    /// or the output cannot be parsed into a valid `SemanticUnit`.
    pub async fn reason(
        &self,
        ctx: &CognitiveContext,
        cfg: &GenerationConfig,
    ) -> Result<Decision, BrainError> {
        // 1) Render narrative
        let msgs = prompt::render(ctx).map_err(BrainError::Render)?;

        // 2) Call provider
        let raw = self.backend.generate(&msgs, cfg).await?;

        // 3) Parse & validate
        parser::parse_decision(&raw.content, raw.meta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use providers::local::MockBackend;
    use serde_json::json;

    #[test]
    fn message_builders() {
        let sys = Message::system("hello");
        assert_eq!(sys.role, "system");
        assert_eq!(sys.content, "hello");

        let usr = Message::user("hi");
        assert_eq!(usr.role, "user");

        let ast = Message::assistant("ok");
        assert_eq!(ast.role, "assistant");
    }

    #[test]
    fn default_config() {
        let cfg = GenerationConfig::default();
        assert_eq!(cfg.temperature, 0.0);
        assert_eq!(cfg.max_tokens, Some(1024));
        assert!(!cfg.require_reasoning);
    }

    #[test]
    fn cognitive_context_default() {
        let ctx = CognitiveContext::default();
        assert!(ctx.system_directive.is_empty());
        assert!(ctx.recall.is_empty());
        assert!(ctx.history.is_empty());
        assert!(ctx.constraints.is_empty());
    }

    #[tokio::test]
    async fn brain_reason_full_pipeline() {
        // Create a mock backend that returns a valid SemanticUnit
        let backend = MockBackend::with_intent("greet", json!({"name": "world"}));
        let brain = Brain::new(backend);

        // Build context
        let ctx = CognitiveContext {
            system_directive: "You are a helpful agent".into(),
            recall: vec!["Previous: said hello".into()],
            history: vec![Message::user("say hi")],
            constraints: vec!["Be polite".into()],
        };

        let cfg = GenerationConfig::default();

        // Execute reason()
        let decision = brain.reason(&ctx, &cfg).await.expect("reason should succeed");

        // Verify the decision
        assert_eq!(decision.intent.kind, "greet");
        assert_eq!(decision.intent.slots.get("name").and_then(|v| v.as_str()), Some("world"));
        assert!(decision.meta.model_id.contains("mock"));
    }

    #[tokio::test]
    async fn brain_reason_with_reasoning_prefix() {
        // Backend that returns reasoning + JSON
        let response = r#"I should greet the user politely.

{"kind": "respond", "slots": {"message": "Hello!"}}"#;
        let backend = MockBackend::new(response);
        let brain = Brain::new(backend);

        let ctx = CognitiveContext {
            system_directive: "Agent".into(),
            ..Default::default()
        };

        let decision: Decision = brain.reason(&ctx, &GenerationConfig::default()).await.unwrap();

        assert_eq!(decision.intent.kind, "respond");
        assert!(decision.reasoning.is_some());
        assert!(decision.reasoning.unwrap().contains("greet"));
    }

    #[tokio::test]
    async fn brain_reason_rejects_invalid_json() {
        let backend = MockBackend::new("This is not JSON at all");
        let brain = Brain::new(backend);

        let ctx = CognitiveContext::default();
        let result: Result<Decision, BrainError> = brain.reason(&ctx, &GenerationConfig::default()).await;

        assert!(result.is_err());
        match result {
            Err(BrainError::Hallucination(_)) => {} // expected - invalid JSON
            Err(BrainError::Parsing(_)) => {} // also acceptable
            other => panic!("Expected BrainError::Hallucination or Parsing, got {other:?}"),
        }
    }
}
