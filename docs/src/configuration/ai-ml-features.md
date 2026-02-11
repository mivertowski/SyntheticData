# AI & ML Features Configuration

> **New in v0.5.0**

This page documents the configuration for DataSynth's AI and ML-powered generation features: LLM-augmented generation, diffusion models, causal generation, and synthetic data certificates.

## LLM Configuration

Configure the LLM provider for metadata enrichment and natural language configuration.

```yaml
llm:
  provider: mock              # Provider type
  model: "gpt-4o-mini"       # Model identifier
  api_key_env: "OPENAI_API_KEY"  # Environment variable for API key
  base_url: null              # Custom API endpoint (for 'custom' provider)
  max_retries: 3              # Retry attempts on failure
  timeout_secs: 30            # Request timeout
  cache_enabled: true         # Enable prompt-level caching
```

### Provider Types

| Provider | Value | Requirements | Description |
|----------|-------|--------------|-------------|
| Mock | `mock` | None | Deterministic, no network. Default for CI/CD |
| OpenAI | `openai` | `OPENAI_API_KEY` env var | OpenAI API (GPT-4o, GPT-4o-mini, etc.) |
| Anthropic | `anthropic` | `ANTHROPIC_API_KEY` env var | Anthropic API (Claude models) |
| Custom | `custom` | `base_url` + API key env var | Any OpenAI-compatible endpoint |

### Field Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `provider` | string | `"mock"` | LLM provider type |
| `model` | string | `"gpt-4o-mini"` | Model identifier passed to the API |
| `api_key_env` | string | `""` | Environment variable name containing the API key |
| `base_url` | string | null | Custom API base URL (required for `custom` provider) |
| `max_retries` | integer | `3` | Maximum retry attempts on transient failures |
| `timeout_secs` | integer | `30` | Per-request timeout in seconds |
| `cache_enabled` | bool | `true` | Cache responses to avoid duplicate API calls |

### Examples

**Mock provider (default, no config needed):**
```yaml
# LLM enrichment uses mock provider by default
# No configuration required
```

**OpenAI:**
```yaml
llm:
  provider: openai
  model: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
```

**Anthropic:**
```yaml
llm:
  provider: anthropic
  model: "claude-sonnet-4-5-20250929"
  api_key_env: "ANTHROPIC_API_KEY"
```

**Self-hosted (e.g., vLLM, Ollama):**
```yaml
llm:
  provider: custom
  model: "llama-3-8b"
  api_key_env: "LOCAL_API_KEY"
  base_url: "http://localhost:8000/v1"
```

**Azure OpenAI:**
```yaml
llm:
  provider: custom
  model: "gpt-4o-mini"
  api_key_env: "AZURE_OPENAI_KEY"
  base_url: "https://my-resource.openai.azure.com/openai/deployments/gpt-4o-mini"
```

## Diffusion Configuration

Configure the statistical diffusion model backend for learned distribution capture.

```yaml
diffusion:
  enabled: false              # Enable diffusion generation
  n_steps: 1000               # Number of diffusion steps
  schedule: "linear"          # Noise schedule type
  sample_size: 1000           # Number of samples to generate
```

### Field Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable diffusion model generation |
| `n_steps` | integer | `1000` | Number of forward/reverse diffusion steps. Higher values improve quality but increase compute time |
| `schedule` | string | `"linear"` | Noise schedule: `"linear"`, `"cosine"`, `"sigmoid"` |
| `sample_size` | integer | `1000` | Number of diffusion-generated samples |

### Noise Schedules

| Schedule | Characteristics | Best For |
|----------|-----------------|----------|
| `linear` | Uniform noise addition, simple and robust | General purpose |
| `cosine` | Slower noise addition, preserves fine details | Financial amounts with precise distributions |
| `sigmoid` | Smooth transition between linear and cosine | Balanced quality and compute |

### Examples

**Basic diffusion:**
```yaml
diffusion:
  enabled: true
  n_steps: 1000
  schedule: "cosine"
  sample_size: 5000
```

**Fast diffusion (fewer steps):**
```yaml
diffusion:
  enabled: true
  n_steps: 200
  schedule: "linear"
  sample_size: 1000
```

## Causal Configuration

Configure causal graph-based data generation with Structural Causal Models.

```yaml
causal:
  enabled: false              # Enable causal generation
  template: "fraud_detection" # Built-in template or custom YAML path
  sample_size: 1000           # Number of samples
  validate: true              # Validate causal structure in output
```

### Field Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable causal/counterfactual generation |
| `template` | string | `"fraud_detection"` | Template name or path to custom YAML graph |
| `sample_size` | integer | `1000` | Number of causal samples to generate |
| `validate` | bool | `true` | Run causal structure validation on output |

### Built-in Templates

| Template | Variables | Use Case |
|----------|-----------|----------|
| `fraud_detection` | transaction_amount, approval_level, vendor_risk, fraud_flag | Fraud risk modeling |
| `revenue_cycle` | order_size, credit_score, payment_delay, revenue | Revenue and credit analysis |

### Custom Causal Graph

Point `template` to a YAML file defining a custom causal graph:

```yaml
causal:
  enabled: true
  template: "./graphs/custom_fraud.yaml"
  sample_size: 10000
  validate: true
```

Custom graph format:

```yaml
# custom_fraud.yaml
variables:
  - name: transaction_amount
    type: continuous
    distribution: lognormal
    params:
      mu: 8.0
      sigma: 1.5
  - name: approval_level
    type: count
    distribution: normal
    params:
      mean: 1.0
      std: 0.5
  - name: fraud_flag
    type: binary

edges:
  - from: transaction_amount
    to: approval_level
    mechanism:
      type: linear
      coefficient: 0.00005
  - from: transaction_amount
    to: fraud_flag
    mechanism:
      type: logistic
      scale: 0.0001
      midpoint: 50000.0
    strength: 0.8
```

### Causal Mechanism Types

| Type | Parameters | Description |
|------|------------|-------------|
| `linear` | `coefficient` | y += coefficient × parent |
| `threshold` | `cutoff` | y = 1 if parent > cutoff, else 0 |
| `polynomial` | `coefficients` (list) | y += Σ c[i] × parent^i |
| `logistic` | `scale`, `midpoint` | y += 1 / (1 + e^(-scale × (parent - midpoint))) |

## Certificate Configuration

Configure synthetic data certificates for provenance and privacy attestation.

```yaml
certificates:
  enabled: false              # Enable certificate generation
  issuer: "DataSynth"        # Certificate issuer identity
  include_quality_metrics: true  # Include quality metrics
```

### Field Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Generate a certificate with each output |
| `issuer` | string | `"DataSynth"` | Issuer identity embedded in the certificate |
| `include_quality_metrics` | bool | `true` | Include Benford MAD, correlation, fidelity, MIA AUC metrics |

### Certificate Contents

When enabled, a `certificate.json` is produced containing:

| Section | Contents |
|---------|----------|
| **Identity** | certificate_id, generation_timestamp, generator_version |
| **Reproducibility** | config_hash (SHA-256), seed, fingerprint_hash |
| **Privacy** | DP mechanism, epsilon, delta, composition method, total queries |
| **Quality** | Benford MAD, correlation preservation, statistical fidelity, MIA AUC |
| **Integrity** | HMAC-SHA256 signature |

## Combined Example

A complete configuration using all AI/ML features:

```yaml
global:
  seed: 42
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12

companies:
  - code: "1000"
    name: "Manufacturing Corp"
    currency: USD
    country: US

transactions:
  target_count: 50000

# LLM enrichment for realistic metadata
llm:
  provider: mock

# Diffusion for learned distributions
diffusion:
  enabled: true
  n_steps: 1000
  schedule: "cosine"
  sample_size: 5000

# Causal structure for fraud scenarios
causal:
  enabled: true
  template: "fraud_detection"
  sample_size: 10000
  validate: true

# Certificate for provenance
certificates:
  enabled: true
  issuer: "DataSynth v0.5.0"
  include_quality_metrics: true

fraud:
  enabled: true
  fraud_rate: 0.005

anomaly_injection:
  enabled: true
  total_rate: 0.02

output:
  format: csv
```

## CLI Flags

Several AI/ML features can also be controlled via CLI flags:

```bash
# Generate with certificate
datasynth-data generate --config config.yaml --output ./output --certificate

# Initialize from natural language
datasynth-data init --from-description "1 year of retail data with fraud" -o config.yaml

# Train diffusion model
datasynth-data diffusion train --fingerprint ./fp.dsf --output ./model.json

# Generate causal data
datasynth-data causal generate --template fraud_detection --samples 10000 --output ./causal/
```

## See Also

- [LLM-Augmented Generation](../advanced/llm-generation.md)
- [Diffusion Models](../advanced/diffusion-models.md)
- [Causal & Counterfactual Generation](../advanced/causal-generation.md)
- [Synthetic Data Certificates](../advanced/certificates.md)
- [YAML Schema Reference](yaml-schema.md)
