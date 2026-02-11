use std::collections::HashMap;
use std::sync::RwLock;

use serde_json::json;
use sha2::{Digest, Sha256};

use super::provider::{LlmConfig, LlmProvider, LlmRequest, LlmResponse, TokenUsage};
use crate::error::SynthError;

/// HTTP-based LLM provider for OpenAI-compatible APIs.
///
/// Supports any API following the `/v1/chat/completions` format.
pub struct HttpLlmProvider {
    config: LlmConfig,
    client: reqwest::blocking::Client,
    cache: RwLock<HashMap<u64, String>>,
}

impl HttpLlmProvider {
    /// Create a new HTTP provider with the given configuration.
    pub fn new(config: LlmConfig) -> Result<Self, SynthError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| SynthError::generation(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config,
            client,
            cache: RwLock::new(HashMap::new()),
        })
    }

    fn cache_key(request: &LlmRequest) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(request.prompt.as_bytes());
        if let Some(ref system) = request.system {
            hasher.update(system.as_bytes());
        }
        if let Some(seed) = request.seed {
            hasher.update(seed.to_le_bytes());
        }
        let hash = hasher.finalize();
        u64::from_le_bytes(hash[..8].try_into().unwrap_or([0u8; 8]))
    }

    fn base_url(&self) -> &str {
        self.config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com")
    }
}

impl LlmProvider for HttpLlmProvider {
    fn name(&self) -> &str {
        "http"
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, SynthError> {
        // Check cache
        if self.config.cache_enabled {
            let key = Self::cache_key(request);
            if let Ok(cache) = self.cache.read() {
                if let Some(cached_content) = cache.get(&key) {
                    return Ok(LlmResponse {
                        content: cached_content.clone(),
                        usage: TokenUsage::default(),
                        cached: true,
                    });
                }
            }
        }

        let url = format!("{}/v1/chat/completions", self.base_url());

        let mut messages = Vec::new();
        if let Some(ref system) = request.system {
            messages.push(json!({"role": "system", "content": system}));
        }
        messages.push(json!({"role": "user", "content": &request.prompt}));

        let mut body = json!({
            "model": &self.config.model,
            "messages": messages,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
        });

        if let Some(seed) = request.seed {
            body["seed"] = json!(seed);
        }

        let api_key = if !self.config.api_key_env.is_empty() {
            std::env::var(&self.config.api_key_env).ok()
        } else {
            None
        };

        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let backoff = std::time::Duration::from_millis(100 * 2u64.pow(attempt as u32));
                std::thread::sleep(backoff);
            }

            let mut req_builder = self
                .client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body);

            if let Some(ref key) = api_key {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", key));
            }

            match req_builder.send() {
                Ok(response) => {
                    if response.status().is_success() {
                        let resp_json: serde_json::Value = response.json().map_err(|e| {
                            SynthError::generation(format!("Failed to parse LLM response: {}", e))
                        })?;

                        let content = resp_json["choices"][0]["message"]["content"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();

                        let usage = TokenUsage {
                            input_tokens: resp_json["usage"]["prompt_tokens"].as_u64().unwrap_or(0)
                                as u32,
                            output_tokens: resp_json["usage"]["completion_tokens"]
                                .as_u64()
                                .unwrap_or(0) as u32,
                        };

                        // Cache the result
                        if self.config.cache_enabled {
                            let key = Self::cache_key(request);
                            if let Ok(mut cache) = self.cache.write() {
                                cache.insert(key, content.clone());
                            }
                        }

                        return Ok(LlmResponse {
                            content,
                            usage,
                            cached: false,
                        });
                    } else if response.status().as_u16() == 429
                        || response.status().is_server_error()
                    {
                        last_error = Some(format!("HTTP {}", response.status()));
                        continue;
                    } else {
                        return Err(SynthError::generation(format!(
                            "LLM API error: HTTP {}",
                            response.status()
                        )));
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    continue;
                }
            }
        }

        Err(SynthError::generation(format!(
            "LLM request failed after {} retries: {}",
            self.config.max_retries,
            last_error.unwrap_or_else(|| "unknown error".to_string())
        )))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_http_provider_cache_key_deterministic() {
        let req = LlmRequest::new("test prompt").with_seed(42);
        let k1 = HttpLlmProvider::cache_key(&req);
        let k2 = HttpLlmProvider::cache_key(&req);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_http_provider_cache_key_differs() {
        let req1 = LlmRequest::new("prompt 1");
        let req2 = LlmRequest::new("prompt 2");
        assert_ne!(
            HttpLlmProvider::cache_key(&req1),
            HttpLlmProvider::cache_key(&req2)
        );
    }
}
