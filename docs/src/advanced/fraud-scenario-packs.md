# Fraud Scenario Packs

Fraud scenario packs are pre-configured bundles of fraud patterns that can be layered onto any DataSynth configuration. Each pack defines specific anomaly types, rates, and relationships designed for training fraud detection models.

## Available Packs

| Pack | Description | Fraud Types |
|------|-------------|-------------|
| `revenue_fraud` | Revenue manipulation patterns | Fictitious sales, channel stuffing, premature recognition, cookie jar reserves |
| `payroll_ghost` | Payroll and ghost employee fraud | Ghost employees, phantom payroll, timesheet manipulation, unauthorized pay changes |
| `vendor_kickback` | Vendor-related fraud schemes | Shell companies, inflated invoices, bid rigging, duplicate payments, kickbacks |
| `management_override` | Management override of controls | Unauthorized JEs, period-end adjustments, SOD violations, override of approval limits |
| `comprehensive` | All patterns at calibrated rates | Combines all packs with balanced rates for complete model training |

## CLI Usage

```bash
# Apply a single pack
datasynth-data generate --config config.yaml --fraud-scenario revenue_fraud --output ./output

# Apply multiple packs
datasynth-data generate --config config.yaml \
  --fraud-scenario vendor_kickback \
  --fraud-scenario payroll_ghost \
  --output ./output

# Override fraud rate
datasynth-data generate --config config.yaml \
  --fraud-scenario comprehensive \
  --fraud-rate 0.05 \
  --output ./output
```

## YAML Configuration

Fraud packs can also be specified in the YAML config:

```yaml
fraud:
  enabled: true
  fraud_packs:
    - revenue_fraud
    - vendor_kickback
  rate: 0.03  # Optional rate override
```

## Deep Merge Behavior

When a fraud pack is applied, its contents are deep-merged into the existing fraud configuration. This means:
- Pack settings are additive — applying multiple packs combines their fraud types
- Explicit config values take precedence over pack defaults
- The `--fraud-rate` flag overrides the rate from any pack

## Python Usage

```python
from datasynth_py.config import blueprints

# Apply comprehensive pack to a base config
config = blueprints.retail_small()
config = blueprints.with_fraud_packs(config, packs=["comprehensive"], fraud_rate=0.05)

# Or via generate() flags
result = synth.generate(config, fraud_scenario=["revenue_fraud"], fraud_rate=0.03)
```

## Evaluation

Use the fraud pack effectiveness evaluator to measure pack quality:
- Detection rate at various thresholds
- False positive analysis
- Pack coverage metrics
