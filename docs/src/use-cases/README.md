# Use Cases

Real-world applications for DataSynth.

## Overview

| Use Case | Description |
|----------|-------------|
| [Fraud Detection ML](fraud-detection.md) | Train supervised fraud models |
| [Audit Analytics](audit-analytics.md) | Test audit procedures |
| [SOX Compliance](sox-compliance.md) | Test control monitoring |
| [Process Mining](process-mining.md) | Generate OCEL 2.0 event logs |
| [ERP Load Testing](erp-testing.md) | Load and stress testing |

## Use Case Summary

| Use Case | Key Features | Output Focus |
|----------|--------------|--------------|
| Fraud Detection | Anomaly injection, graph export | Labels, graphs |
| Audit Analytics | Full document flows, controls | Transactions, controls |
| SOX Compliance | SoD rules, approval workflows | Controls, violations |
| Process Mining | OCEL 2.0 export | Event logs |
| ERP Testing | High volume, realistic patterns | Raw transactions |

## Quick Configuration

### Fraud Detection

```yaml
anomaly_injection:
  enabled: true
  total_rate: 0.02
  generate_labels: true

graph_export:
  enabled: true
  formats:
    - pytorch_geometric
```

### Audit Analytics

```yaml
document_flows:
  p2p:
    enabled: true
  o2c:
    enabled: true

internal_controls:
  enabled: true
```

### SOX Compliance

```yaml
internal_controls:
  enabled: true
  sod_rules: [...]

approval:
  enabled: true
```

### Process Mining

```yaml
document_flows:
  p2p:
    enabled: true
  o2c:
    enabled: true

# Use datasynth-ocpm for OCEL 2.0 export
```

### ERP Testing

```yaml
transactions:
  target_count: 1000000

output:
  format: csv
```

## Selecting a Use Case

**Choose Fraud Detection if:**
- Training ML/AI models
- Building anomaly detection systems
- Need labeled datasets

**Choose Audit Analytics if:**
- Testing audit software
- Validating analytical procedures
- Need complete document trails

**Choose SOX Compliance if:**
- Testing control monitoring systems
- Validating SoD enforcement
- Need control test data

**Choose Process Mining if:**
- Using PM4Py, Celonis, or similar tools
- Need OCEL 2.0 compliant logs
- Analyzing business processes

**Choose ERP Testing if:**
- Load testing financial systems
- Performance benchmarking
- Need high-volume realistic data

## Combining Use Cases

Use cases can be combined:

```yaml
# Fraud detection + audit analytics
anomaly_injection:
  enabled: true
  total_rate: 0.02
  generate_labels: true

document_flows:
  p2p:
    enabled: true
  o2c:
    enabled: true

internal_controls:
  enabled: true

graph_export:
  enabled: true
```

## See Also

- [Configuration](../configuration/README.md)
- [Advanced Topics](../advanced/README.md)
- [Getting Started](../getting-started/README.md)
