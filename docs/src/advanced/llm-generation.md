# LLM-Augmented Generation

> **New in v0.5.0**

LLM-augmented generation uses Large Language Models to enrich synthetic data with realistic metadata — vendor names, transaction descriptions, memo fields, and anomaly explanations — that would be difficult to generate with rule-based approaches alone.

## Overview

Traditional synthetic data generators produce structurally correct but often generic-sounding text fields. LLM augmentation addresses this by using language models to generate contextually appropriate text based on the financial domain, industry, and transaction context.

DataSynth provides a pluggable provider abstraction that supports:

| Provider | Description | Use Case |
|----------|-------------|----------|
| **Mock** | Deterministic, no network required | CI/CD, testing, reproducible builds |
| **OpenAI** | OpenAI-compatible APIs (GPT-4o-mini, etc.) | Production enrichment |
| **Anthropic** | Anthropic API (Claude models) | Production enrichment |
| **Custom** | Any OpenAI-compatible endpoint | Self-hosted models, Azure OpenAI |

## Provider Abstraction

All LLM functionality is built around the `LlmProvider` trait:

```rust
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, SynthError>;
    fn complete_batch(&self, requests: &[LlmRequest]) -> Result<Vec<LlmResponse>, SynthError>;
}
```

### LlmRequest

```rust
let request = LlmRequest::new("Generate a vendor name for a German auto parts manufacturer")
    .with_system("You are a business data generator. Return only the company name.")
    .with_seed(42)
    .with_max_tokens(50)
    .with_temperature(0.7);
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `prompt` | String | (required) | The generation prompt |
| `system` | Option\<String\> | None | System message for context |
| `max_tokens` | u32 | 100 | Maximum response tokens |
| `temperature` | f64 | 0.7 | Sampling temperature |
| `seed` | Option\<u64\> | None | Seed for deterministic output |

### LlmResponse

```rust
pub struct LlmResponse {
    pub content: String,       // Generated text
    pub usage: TokenUsage,     // Input/output token counts
    pub cached: bool,          // Whether result came from cache
}
```

## Mock Provider

The `MockLlmProvider` generates deterministic, contextually-aware text without any network calls. It is the default provider and is ideal for:

- CI/CD pipelines where network access is restricted
- Reproducible builds with deterministic output
- Development and testing
- Environments where API costs are a concern

```rust
use synth_core::llm::MockLlmProvider;

let provider = MockLlmProvider::new(42); // seeded for reproducibility
```

The mock provider uses the seed and prompt content to generate plausible-sounding business names and descriptions deterministically.

## HTTP Provider

The `HttpLlmProvider` connects to real LLM APIs:

```rust
use synth_core::llm::{HttpLlmProvider, LlmConfig, LlmProviderType};

let config = LlmConfig {
    provider: LlmProviderType::OpenAi,
    model: "gpt-4o-mini".to_string(),
    api_key_env: "OPENAI_API_KEY".to_string(),
    base_url: None,
    max_retries: 3,
    timeout_secs: 30,
    cache_enabled: true,
};

let provider = HttpLlmProvider::new(config)?;
```

### Configuration

```yaml
# In your generation config
llm:
  provider: openai          # mock, openai, anthropic, custom
  model: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
  base_url: null            # Override for custom endpoints
  max_retries: 3
  timeout_secs: 30
  cache_enabled: true
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `provider` | string | `mock` | Provider type: `mock`, `openai`, `anthropic`, `custom` |
| `model` | string | `gpt-4o-mini` | Model identifier |
| `api_key_env` | string | — | Environment variable containing the API key |
| `base_url` | string | null | Custom API base URL (required for `custom` provider) |
| `max_retries` | integer | `3` | Maximum retry attempts on failure |
| `timeout_secs` | integer | `30` | Request timeout in seconds |
| `cache_enabled` | bool | `true` | Enable prompt-level caching |

## Enrichment Types

### Vendor Name Enrichment

Generates realistic vendor names based on industry, spend category, and country:

```rust
use synth_generators::llm_enrichment::VendorLlmEnricher;

let enricher = VendorLlmEnricher::new(provider.clone());
let name = enricher.enrich_vendor_name("manufacturing", "raw_materials", "DE")?;
// e.g., "Rheinische Stahlwerke GmbH"

// Batch enrichment for efficiency
let names = enricher.enrich_batch(&[
    ("manufacturing".into(), "raw_materials".into(), "DE".into()),
    ("retail".into(), "logistics".into(), "US".into()),
], 42)?;
```

### Transaction Description Enrichment

Generates contextually appropriate journal entry descriptions:

```rust
use synth_generators::llm_enrichment::TransactionLlmEnricher;

let enricher = TransactionLlmEnricher::new(provider.clone());

let desc = enricher.enrich_description(
    "Office Supplies",    // account name
    "1000-5000",          // amount range
    "retail",             // industry
    3,                    // fiscal period
)?;

let memo = enricher.enrich_memo(
    "VendorInvoice",      // document type
    "Acme Corp",          // vendor name
    "2500.00",            // amount
)?;
```

### Anomaly Explanation

Generates natural language explanations for injected anomalies:

```rust
use synth_generators::llm_enrichment::AnomalyLlmExplainer;

let explainer = AnomalyLlmExplainer::new(provider.clone());
let explanation = explainer.explain(
    "DuplicatePayment",           // anomaly type
    3,                             // affected records
    "Same amount, same vendor, 2 days apart",  // statistical details
)?;
```

## Natural Language Configuration

The `NlConfigGenerator` converts natural language descriptions into YAML configuration:

```rust
use synth_core::llm::NlConfigGenerator;

let yaml = NlConfigGenerator::generate(
    "Generate 1 year of retail data for a mid-size US company with fraud patterns",
    &provider,
)?;
```

### CLI Usage

```bash
datasynth-data init \
    --from-description "Generate 1 year of manufacturing data for a German mid-cap with intercompany transactions" \
    -o config.yaml
```

The generator parses intent into structured fields:

```rust
pub struct ConfigIntent {
    pub industry: Option<String>,     // e.g., "manufacturing"
    pub country: Option<String>,      // e.g., "DE"
    pub company_size: Option<String>, // e.g., "mid-cap"
    pub period_months: Option<u32>,   // e.g., 12
    pub features: Vec<String>,        // e.g., ["intercompany"]
}
```

## Caching

The `LlmCache` deduplicates identical prompts using FNV-1a hashing:

```rust
use synth_core::llm::LlmCache;

let cache = LlmCache::new(10000); // max 10,000 entries
let key = LlmCache::cache_key("prompt text", Some("system"), Some(42));

cache.insert(key, "cached response".into());
if let Some(response) = cache.get(key) {
    // Use cached response
}
```

Caching is enabled by default and significantly reduces API costs when generating similar entities.

## Cost and Privacy Considerations

### Cost Management

- Use the **Mock provider** for development and CI/CD (zero cost)
- Enable **caching** to avoid duplicate API calls
- Use **batch enrichment** (`complete_batch`) to reduce per-request overhead
- Set appropriate `max_tokens` limits to control response sizes
- Consider `gpt-4o-mini` or similar efficient models for bulk enrichment

### Privacy

- LLM prompts contain only **synthetic context** (industry, category, amount ranges) — never real data
- No PII or sensitive information is sent to LLM providers
- The Mock provider keeps everything local with no network traffic
- For maximum privacy, use self-hosted models via the `custom` provider type

## See Also

- [AI & ML Configuration](../configuration/ai-ml-features.md)
- [LLM Training Data Use Case](../use-cases/llm-training-data.md)
- [datasynth-core LLM Module](../crates/datasynth-core.md)
- [datasynth-generators LLM Enrichment](../crates/datasynth-generators.md)
