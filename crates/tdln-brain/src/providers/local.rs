//! Local echo backend for testing.
//!
//! Returns a fixed `noop` intent without calling any external service.

use crate::{BrainError, GenerationConfig, Message, NeuralBackend, RawOutput, UsageMeta};
use async_trait::async_trait;

/// A local echo backend that returns a fixed `noop` intent.
///
/// Useful for testing the pipeline without network calls.
pub struct LocalEcho;

#[async_trait]
impl NeuralBackend for LocalEcho {
    fn model_id(&self) -> &str {
        "local-echo"
    }

    async fn generate(
        &self,
        _messages: &[Message],
        _config: &GenerationConfig,
    ) -> Result<RawOutput, BrainError> {
        let content = r#"{"kind":"noop","slots":{}}"#.to_string();
        Ok(RawOutput {
            content,
            meta: UsageMeta {
                input_tokens: 0,
                output_tokens: 3,
                model_id: "local-echo".into(),
            },
        })
    }
}

/// A mock backend that returns a configurable response.
pub struct MockBackend {
    response: String,
    model_id: String,
}

impl MockBackend {
    /// Create a mock backend with a fixed JSON response.
    #[must_use]
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            model_id: "mock-tdln".into(),
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

    /// Set a custom model ID.
    #[must_use]
    pub fn with_model_id(mut self, id: impl Into<String>) -> Self {
        self.model_id = id.into();
        self
    }
}

#[async_trait]
impl NeuralBackend for MockBackend {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn generate(
        &self,
        _messages: &[Message],
        _config: &GenerationConfig,
    ) -> Result<RawOutput, BrainError> {
        Ok(RawOutput {
            content: self.response.clone(),
            meta: UsageMeta {
                model_id: self.model_id.clone(),
                input_tokens: 0,
                output_tokens: self.response.len() as u32 / 4,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_echo_returns_noop() {
        let backend = LocalEcho;
        let result = backend
            .generate(&[], &GenerationConfig::default())
            .await
            .unwrap();
        assert!(result.content.contains("noop"));
    }

    #[tokio::test]
    async fn mock_backend_returns_configured() {
        let backend = MockBackend::with_intent("grant", serde_json::json!({"to": "alice"}));
        let result = backend
            .generate(&[], &GenerationConfig::default())
            .await
            .unwrap();
        assert!(result.content.contains("grant"));
        assert!(result.content.contains("alice"));
    }
}
