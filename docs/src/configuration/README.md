# Configuration

SyntheticData uses YAML configuration files to control all aspects of data generation.

## Quick Start

```bash
# Create configuration from preset
datasynth-data init --industry manufacturing --complexity medium -o config.yaml

# Validate configuration
datasynth-data validate --config config.yaml

# Generate with configuration
datasynth-data generate --config config.yaml --output ./output
```

## Configuration Sections

| Section | Description |
|---------|-------------|
| [Global Settings](global-settings.md) | Industry, dates, seed, performance |
| [Companies](companies.md) | Company codes, currencies, volume weights |
| [Transactions](transactions.md) | Line items, amounts, sources |
| [Master Data](master-data.md) | Vendors, customers, materials, assets |
| [Document Flows](document-flows.md) | P2P, O2C configuration |
| [Financial Settings](financial-settings.md) | Balance, subledger, FX, period close |
| [Compliance](compliance.md) | Fraud, controls, approval |
| [AI & ML Features](ai-ml-features.md) | LLM, diffusion, causal, certificates |
| [Output Settings](output-settings.md) | Format, compression |

## Reference

- [Complete YAML Schema](yaml-schema.md)
- [Industry Presets](industry-presets.md)

## Minimal Configuration

```yaml
global:
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12

transactions:
  target_count: 10000

output:
  format: csv
```

## Full Configuration Example

```yaml
global:
  seed: 42
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12
  group_currency: USD

companies:
  - code: "1000"
    name: "Headquarters"
    currency: USD
    country: US
    volume_weight: 0.6
  - code: "2000"
    name: "European Subsidiary"
    currency: EUR
    country: DE
    volume_weight: 0.4

chart_of_accounts:
  complexity: medium

transactions:
  target_count: 100000
  line_items:
    distribution: empirical
  amounts:
    min: 100
    max: 1000000

master_data:
  vendors:
    count: 200
  customers:
    count: 500
  materials:
    count: 1000

document_flows:
  p2p:
    enabled: true
    flow_rate: 0.3
  o2c:
    enabled: true
    flow_rate: 0.3

fraud:
  enabled: true
  fraud_rate: 0.005

anomaly_injection:
  enabled: true
  total_rate: 0.02
  generate_labels: true

graph_export:
  enabled: true
  formats:
    - pytorch_geometric
    - neo4j

# AI & ML Features (v0.5.0)
diffusion:
  enabled: true
  n_steps: 1000
  schedule: "cosine"
  sample_size: 1000

causal:
  enabled: true
  template: "fraud_detection"
  sample_size: 1000
  validate: true

certificates:
  enabled: true
  issuer: "DataSynth"
  include_quality_metrics: true

output:
  format: csv
  compression: none
```

## Configuration Loading

Configuration can be loaded from:

1. **YAML file** (recommended):
   ```bash
   datasynth-data generate --config config.yaml --output ./output
   ```

2. **JSON file**:
   ```bash
   datasynth-data generate --config config.json --output ./output
   ```

3. **Demo preset**:
   ```bash
   datasynth-data generate --demo --output ./output
   ```

## Validation

The configuration is validated for:

| Rule | Description |
|------|-------------|
| Required fields | All mandatory fields must be present |
| Value ranges | Numbers within valid bounds |
| Distributions | Weights sum to 1.0 (±0.01 tolerance) |
| Dates | Valid date ranges |
| Uniqueness | Company codes must be unique |
| Consistency | Cross-field validations |

Run validation:
```bash
datasynth-data validate --config config.yaml
```

## Overriding Values

Command-line options override configuration file values:

```bash
# Override seed
datasynth-data generate --config config.yaml --seed 12345 --output ./output

# Override format
datasynth-data generate --config config.yaml --format json --output ./output
```

## Environment Variables

Some settings can be controlled via environment variables:

| Variable | Configuration Equivalent |
|----------|--------------------------|
| `SYNTH_DATA_SEED` | `global.seed` |
| `SYNTH_DATA_THREADS` | `global.worker_threads` |
| `SYNTH_DATA_MEMORY_LIMIT` | `global.memory_limit` |

## See Also

- [Quick Start](../getting-started/quick-start.md)
- [CLI Reference](../user-guide/cli-reference.md)
- [datasynth-config Crate](../crates/datasynth-config.md)
