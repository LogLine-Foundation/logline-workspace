//! OpenAI provider implementation.
//!
//! Supports GPT-4, GPT-3.5-turbo, and other OpenAI chat models.

use crate::{BrainError, GenerationConfig, Message, NeuralBackend, RawOutput, UsageMeta};
use async_trait::async_trait;
use serde_json::json;

/// OpenAI API driver.
pub struct OpenAiDriver {
    api_key: String,
    model: String,
    client: reqwest::Client,
    base_url: String,
}

impl OpenAiDriver {
    /// Create a new OpenAI driver with the given API key and model.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            client: reqwest::Client::new(),
            base_url: "https://api.openai.com/v1".into(),
        }
    }

    /// Use a custom base URL (for proxies or Azure OpenAI).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Check if the model supports JSON mode.
    fn supports_json_mode(&self) -> bool {
        self.model.contains("gpt-4")
            || self.model.contains("gpt-3.5-turbo-1106")
            || self.model.contains("gpt-3.5-turbo-0125")
    }
}

#[async_trait]
impl NeuralBackend for OpenAiDriver {
    fn model_id(&self) -> &str {
        &self.model
    }

    async fn generate(
        &self,
        messages: &[Message],
        cfg: &GenerationConfig,
    ) -> Result<RawOutput, BrainError> {
        let url = format!("{}/chat/completions", self.base_url);

        // Build request body
        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "temperature": cfg.temperature,
        });

        if let Some(max) = cfg.max_tokens {
            body["max_tokens"] = json!(max);
        }

        // Enable JSON mode if supported
        if self.supports_json_mode() {
            body["response_format"] = json!({"type": "json_object"});
        }

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| BrainError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrainError::Provider(format!("{status}: {text}")));
        }

        let val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BrainError::Provider(e.to_string()))?;

        let content = val["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| BrainError::Provider("empty content in response".into()))?
            .to_string();

        let usage = &val["usage"];
        let meta = UsageMeta {
            input_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
            model_id: self.model.clone(),
        };

        Ok(RawOutput { content, meta })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_json_mode_for_gpt4() {
        let driver = OpenAiDriver::new("key", "gpt-4o");
        assert!(driver.supports_json_mode());
    }

    #[test]
    fn supports_json_mode_for_turbo_1106() {
        let driver = OpenAiDriver::new("key", "gpt-3.5-turbo-1106");
        assert!(driver.supports_json_mode());
    }

    #[test]
    fn no_json_mode_for_old_models() {
        let driver = OpenAiDriver::new("key", "gpt-3.5-turbo-0613");
        assert!(!driver.supports_json_mode());
    }
}
