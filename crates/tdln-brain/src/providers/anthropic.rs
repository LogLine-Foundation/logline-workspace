//! Anthropic Claude provider implementation.
//!
//! Supports Claude 3 family (Opus, Sonnet, Haiku) and Claude 2.

use crate::{BrainError, GenerationConfig, Message, NeuralBackend, RawOutput, UsageMeta};
use async_trait::async_trait;
use serde_json::json;

/// Anthropic API driver.
pub struct AnthropicDriver {
    api_key: String,
    model: String,
    client: reqwest::Client,
    base_url: String,
}

impl AnthropicDriver {
    /// Create a new Anthropic driver with the given API key and model.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            client: reqwest::Client::new(),
            base_url: "https://api.anthropic.com/v1".into(),
        }
    }

    /// Use a custom base URL.
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

#[async_trait]
impl NeuralBackend for AnthropicDriver {
    fn model_id(&self) -> &str {
        &self.model
    }

    async fn generate(
        &self,
        messages: &[Message],
        cfg: &GenerationConfig,
    ) -> Result<RawOutput, BrainError> {
        let url = format!("{}/messages", self.base_url);

        // Anthropic uses a different message format
        // System message is separate, not in messages array
        let (system_msg, user_messages): (Option<&Message>, Vec<&Message>) = {
            let mut sys = None;
            let mut msgs = Vec::new();
            for m in messages {
                if m.role == "system" {
                    sys = Some(m);
                } else {
                    msgs.push(m);
                }
            }
            (sys, msgs)
        };

        // Convert to Anthropic format
        let anthropic_messages: Vec<serde_json::Value> = user_messages
            .iter()
            .map(|m| {
                json!({
                    "role": m.role,
                    "content": m.content
                })
            })
            .collect();

        let mut body = json!({
            "model": self.model,
            "messages": anthropic_messages,
            "max_tokens": cfg.max_tokens.unwrap_or(1024),
        });

        if let Some(sys) = system_msg {
            body["system"] = json!(sys.content);
        }

        // Anthropic doesn't have temperature 0, use very low
        if cfg.temperature > 0.0 {
            body["temperature"] = json!(cfg.temperature);
        }

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
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

        // Anthropic returns content as array of blocks
        let content = val["content"][0]["text"]
            .as_str()
            .ok_or_else(|| BrainError::Provider("empty content in response".into()))?
            .to_string();

        let usage = &val["usage"];
        let meta = UsageMeta {
            input_tokens: usage["input_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: usage["output_tokens"].as_u64().unwrap_or(0) as u32,
            model_id: self.model.clone(),
        };

        Ok(RawOutput { content, meta })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_creation() {
        let driver = AnthropicDriver::new("key", "claude-3-sonnet-20240229");
        assert_eq!(driver.model_id(), "claude-3-sonnet-20240229");
    }

    #[test]
    fn custom_base_url() {
        let driver = AnthropicDriver::new("key", "claude-3-haiku-20240307")
            .with_base_url("https://custom.api.com");
        assert_eq!(driver.base_url, "https://custom.api.com");
    }
}
