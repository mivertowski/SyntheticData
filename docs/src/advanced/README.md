# Advanced Topics

Advanced features for specialized use cases.

## Overview

| Topic | Description |
|-------|-------------|
| [Anomaly Injection](anomaly-injection.md) | Fraud, errors, and process issues |
| [Data Quality Variations](data-quality.md) | Missing values, typos, duplicates |
| [Graph Export](graph-export.md) | ML-ready graph formats |
| [Intercompany Processing](intercompany.md) | Multi-entity transactions |
| [Period Close Engine](period-close.md) | Month/quarter/year-end processes |
| [Performance Tuning](performance.md) | Optimization strategies |
| [Fraud Scenario Packs](fraud-scenario-packs.md) | Pre-configured fraud pattern bundles |
| [Counterfactual Scenarios](counterfactual-scenarios.md) | Causal DAG what-if analysis |
| [OCEL 2.0 Enrichment](ocel-enrichment.md) | Lifecycle, correlation, resource pools |
| [Streaming Pipeline](streaming-pipeline.md) | Phase-aware real-time output |
| [Evaluation Framework](evaluation-framework.md) | Data quality evaluation suite |

## Feature Matrix

| Feature | Use Case | Output |
|---------|----------|--------|
| Anomaly Injection | ML training | Labels (CSV) |
| Data Quality | Testing robustness | Varied data |
| Graph Export | GNN training | PyG, Neo4j |
| Intercompany | Consolidation testing | IC pairs |
| Period Close | Full cycle testing | Closing entries |

## Enabling Advanced Features

### In Configuration

```yaml
# Anomaly injection
anomaly_injection:
  enabled: true
  total_rate: 0.02
  generate_labels: true

# Data quality variations
data_quality:
  enabled: true
  missing_values:
    rate: 0.01

# Graph export
graph_export:
  enabled: true
  formats:
    - pytorch_geometric
    - neo4j

# Intercompany
intercompany:
  enabled: true

# Period close
period_close:
  enabled: true
  monthly:
    accruals: true
    depreciation: true
```

### Via CLI

Most advanced features are controlled through configuration. Use init to create a base config, then customize:

```bash
datasynth-data init --industry manufacturing --complexity medium -o config.yaml
# Edit config.yaml to enable advanced features
datasynth-data generate --config config.yaml --output ./output
```

## Prerequisites

Some advanced features have dependencies:

| Feature | Requires |
|---------|----------|
| Intercompany | Multiple companies defined |
| Period Close | `period_months` ≥ 1 |
| Graph Export | Transactions generated |
| FX | Multiple currencies |

## Output Files

Advanced features produce additional outputs:

```
output/
├── labels/                      # Anomaly injection
│   ├── anomaly_labels.csv
│   ├── fraud_labels.csv
│   └── quality_issues.csv
├── graphs/                      # Graph export
│   ├── pytorch_geometric/
│   └── neo4j/
├── consolidation/               # Intercompany
│   ├── eliminations.csv
│   └── ic_pairs.csv
└── period_close/                # Period close
    ├── trial_balances/
    ├── accruals.csv
    └── closing_entries.csv
```

## Performance Impact

| Feature | Impact | Mitigation |
|---------|--------|------------|
| Anomaly Injection | Low | Post-processing |
| Data Quality | Low | Post-processing |
| Graph Export | Medium | Separate phase |
| Intercompany | Medium | Per-transaction |
| Period Close | Low | Per-period |

## See Also

- [Configuration](../configuration/README.md)
- [Use Cases](../use-cases/README.md)
- [datasynth-generators](../crates/datasynth-generators.md)
