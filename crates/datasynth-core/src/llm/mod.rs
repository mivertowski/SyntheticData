//! LLM provider abstraction for AI-augmented data generation.
//!
//! This module provides a trait-based LLM integration that supports:
//! - Deterministic mock provider for testing (always available)
//! - HTTP-based providers for OpenAI/Anthropic (requires `llm` feature)
//! - Response caching for efficiency

pub mod cache;
#[cfg(feature = "llm")]
pub mod http_provider;
pub mod mock_provider;
pub mod nl_config;
pub mod provider;

pub use cache::LlmCache;
#[cfg(feature = "llm")]
pub use http_provider::HttpLlmProvider;
pub use mock_provider::MockLlmProvider;
pub use provider::*;
