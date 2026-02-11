# Causal Analysis

> **New in v0.5.0**

Use DataSynth's causal generation capabilities for "what-if" scenario testing and counterfactual analysis in audit and risk management.

## When to Use Causal Generation

Causal generation is ideal when you need to:

- **Test audit scenarios**: "What would happen to fraud rates if we increased the approval threshold?"
- **Risk assessment**: "How would revenue change if we lost our top vendor?"
- **Policy evaluation**: "What is the causal effect of implementing a new control?"
- **Training causal ML models**: Generate data with known causal structure for model validation

## Setting Up a Fraud Detection SCM

```bash
# Generate causally-structured fraud detection data
datasynth-data causal generate \
    --template fraud_detection \
    --samples 50000 \
    --seed 42 \
    --output ./fraud_causal
```

The `fraud_detection` template models:
- `transaction_amount` → `approval_level` (larger amounts require higher approval)
- `transaction_amount` → `fraud_flag` (larger amounts have higher fraud probability)
- `vendor_risk` → `fraud_flag` (risky vendors associated with more fraud)

## Running Interventions

Answer "what if?" questions by forcing variables to specific values:

```bash
# What if all transactions were $50,000?
datasynth-data causal intervene \
    --template fraud_detection \
    --variable transaction_amount \
    --value 50000 \
    --samples 10000 \
    --output ./intervention_50k

# What if vendor risk were always high (0.9)?
datasynth-data causal intervene \
    --template fraud_detection \
    --variable vendor_risk \
    --value 0.9 \
    --samples 10000 \
    --output ./intervention_high_risk
```

Compare the intervention output against the baseline to estimate causal effects.

## Counterfactual Analysis for Audit

For individual transaction review:

```python
from datasynth_py import DataSynth

synth = DataSynth()

# Load a specific flagged transaction
factual = {
    "transaction_amount": 5000.0,
    "approval_level": 1.0,
    "vendor_risk": 0.3,
    "fraud_flag": 0.0,
}

# What would have happened if the amount were 10x larger?
# The counterfactual preserves the same "noise" (latent factors)
# but propagates the new amount through the causal structure
```

This helps auditors understand which factors most influence risk assessments.

## Configuration Example

```yaml
global:
  seed: 42
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12

causal:
  enabled: true
  template: "fraud_detection"
  sample_size: 50000
  validate: true

# Combine with regular generation
transactions:
  target_count: 100000

fraud:
  enabled: true
  fraud_rate: 0.005
```

## Validating Causal Structure

Verify that generated data preserves the intended causal relationships:

```bash
datasynth-data causal validate \
    --data ./fraud_causal \
    --template fraud_detection
```

The validator checks:
- Parent-child correlations match expected directions
- Independence constraints hold for non-adjacent variables
- Intervention effects are consistent with the graph

## See Also

- [Causal & Counterfactual Generation](../advanced/causal-generation.md)
- [Fraud Detection ML](fraud-detection.md)
- [Audit Analytics](audit-analytics.md)
