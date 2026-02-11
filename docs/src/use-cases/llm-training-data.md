# LLM Training Data

> **New in v0.5.0**

Generate LLM-enriched synthetic financial data for training and fine-tuning language models on domain-specific tasks.

## When to Use LLM-Enriched Data

- **Fine-tuning**: Train financial document understanding models on realistic data
- **RAG evaluation**: Test retrieval-augmented generation with known-truth synthetic documents
- **Classification training**: Generate labeled financial text for transaction categorization
- **Anomaly explanation**: Train models to explain financial anomalies in natural language

## Quality vs Cost Tradeoffs

| Provider | Quality | Cost | Latency | Reproducibility |
|----------|---------|------|---------|-----------------|
| **Mock** | Good (template-based) | Free | Instant | Fully deterministic |
| **gpt-4o-mini** | High | ~$0.15/1M tokens | ~200ms/req | Seed-based |
| **gpt-4o** | Very High | ~$2.50/1M tokens | ~500ms/req | Seed-based |
| **Claude (Anthropic)** | Very High | Varies | ~300ms/req | Seed-based |
| **Self-hosted** | Varies | Infrastructure cost | Varies | Full control |

## Using the Mock Provider for CI/CD

The mock provider generates deterministic, contextually-aware text without any API calls:

```bash
# Default: uses mock provider (no API key needed)
datasynth-data generate --config config.yaml --output ./output
```

```yaml
# Explicit mock configuration
llm:
  provider: mock
```

The mock provider is suitable for:
- CI/CD pipelines
- Automated testing
- Reproducible research
- Development environments

## Using Real LLM Providers

For production-quality enrichment:

```yaml
llm:
  provider: openai
  model: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
  cache_enabled: true       # Avoid duplicate API calls
  max_retries: 3
  timeout_secs: 30
```

```bash
export OPENAI_API_KEY="sk-..."
datasynth-data generate --config config.yaml --output ./output
```

## Batch Generation for Large Datasets

For large-scale enrichment, use batch mode to minimize API overhead:

```python
from datasynth_py import DataSynth, Config
from datasynth_py.config import blueprints

# Generate base data first (fast, rule-based)
config = blueprints.manufacturing_large(transactions=100000)
synth = DataSynth()
result = synth.generate(config=config, output={"format": "csv", "sink": "temp_dir"})

# Then enrich with LLM in a separate pass if needed
```

## Example: Financial Document Understanding

Generate training data for a document understanding model:

```yaml
global:
  seed: 42
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12

transactions:
  target_count: 50000

document_flows:
  p2p:
    enabled: true
    flow_rate: 0.4
  o2c:
    enabled: true
    flow_rate: 0.3

anomaly_injection:
  enabled: true
  total_rate: 0.03
  generate_labels: true

# LLM enrichment adds realistic descriptions
llm:
  provider: mock     # or openai for higher quality
```

The generated data includes:
- Vendor names appropriate for the industry and spend category
- Transaction descriptions that read like real GL entries
- Memo fields on invoices and payments
- Natural language explanations for flagged anomalies

## See Also

- [LLM-Augmented Generation](../advanced/llm-generation.md)
- [AI & ML Configuration](../configuration/ai-ml-features.md)
- [Fraud Detection ML](fraud-detection.md)
