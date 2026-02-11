use serde::{Deserialize, Serialize};

use crate::error::SynthError;

/// LLM provider trait for AI-augmented generation.
pub trait LlmProvider: Send + Sync {
    /// Provider name.
    fn name(&self) -> &str;
    /// Complete a single request.
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, SynthError>;
    /// Complete a batch of requests.
    fn complete_batch(&self, requests: &[LlmRequest]) -> Result<Vec<LlmResponse>, SynthError> {
        requests.iter().map(|r| self.complete(r)).collect()
    }
}

/// A request to an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// The user prompt.
    pub prompt: String,
    /// Optional system prompt.
    pub system: Option<String>,
    /// Maximum tokens in the response.
    pub max_tokens: u32,
    /// Sampling temperature.
    pub temperature: f64,
    /// Optional seed for deterministic output.
    pub seed: Option<u64>,
}

impl LlmRequest {
    /// Create a new request with the given prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            system: None,
            max_tokens: 1024,
            temperature: 0.7,
            seed: None,
        }
    }

    /// Set the system prompt.
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set the seed for deterministic output.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the maximum number of tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the sampling temperature.
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }
}

/// A response from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// The generated content.
    pub content: String,
    /// Token usage statistics.
    pub usage: TokenUsage,
    /// Whether this response was served from cache.
    pub cached: bool,
}

/// Token usage statistics for an LLM request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of input (prompt) tokens.
    pub input_tokens: u32,
    /// Number of output (completion) tokens.
    pub output_tokens: u32,
}

/// LLM provider type selection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmProviderType {
    /// Deterministic mock provider (no network calls).
    #[default]
    Mock,
    /// OpenAI-compatible API provider.
    OpenAi,
    /// Anthropic API provider.
    Anthropic,
    /// Custom provider with user-specified base URL.
    Custom,
}

/// LLM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Which provider type to use.
    pub provider: LlmProviderType,
    /// Model name/ID.
    #[serde(default = "default_llm_model")]
    pub model: String,
    /// Environment variable containing the API key.
    #[serde(default)]
    pub api_key_env: String,
    /// Custom API base URL (overrides provider default).
    #[serde(default)]
    pub base_url: Option<String>,
    /// Maximum retry attempts for failed requests.
    #[serde(default = "default_max_retries")]
    pub max_retries: u8,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    /// Whether to cache responses.
    #[serde(default = "default_true_val")]
    pub cache_enabled: bool,
}

fn default_llm_model() -> String {
    "gpt-4o-mini".to_string()
}
fn default_max_retries() -> u8 {
    3
}
fn default_timeout_secs() -> u64 {
    30
}
fn default_true_val() -> bool {
    true
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProviderType::default(),
            model: default_llm_model(),
            api_key_env: String::new(),
            base_url: None,
            max_retries: default_max_retries(),
            timeout_secs: default_timeout_secs(),
            cache_enabled: true,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_request_builder() {
        let req = LlmRequest::new("test prompt")
            .with_system("system prompt")
            .with_seed(42)
            .with_max_tokens(512)
            .with_temperature(0.5);
        assert_eq!(req.prompt, "test prompt");
        assert_eq!(req.system, Some("system prompt".to_string()));
        assert_eq!(req.seed, Some(42));
        assert_eq!(req.max_tokens, 512);
        assert!((req.temperature - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_llm_request_serde_roundtrip() {
        let req = LlmRequest::new("test").with_seed(42);
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: LlmRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.prompt, "test");
        assert_eq!(deserialized.seed, Some(42));
    }

    #[test]
    fn test_llm_response_serde_roundtrip() {
        let resp = LlmResponse {
            content: "output".to_string(),
            usage: TokenUsage {
                input_tokens: 10,
                output_tokens: 20,
            },
            cached: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: LlmResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content, "output");
        assert_eq!(deserialized.usage.input_tokens, 10);
    }

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert!(matches!(config.provider, LlmProviderType::Mock));
        assert!(config.cache_enabled);
        assert_eq!(config.max_retries, 3);
    }
}
