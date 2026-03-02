# Evaluation Framework

The `datasynth-eval` crate provides comprehensive evaluation of synthetic data quality across statistical, coherence, and domain-specific dimensions.

## Evaluators

### Statistical Evaluators
- **Benford's Law** -- First-digit distribution compliance (MAD < 0.015)
- **Distribution Fit** -- Kolmogorov-Smirnov and Anderson-Darling tests
- **Temporal Patterns** -- Seasonality and period-end volume verification

### Coherence Evaluators
- **Balance Validation** -- Assets = Liabilities + Equity
- **IC Matching** -- Intercompany transaction pairing completeness
- **Document Chains** -- PO -> GR -> Invoice -> Payment reference integrity

### Quality Evaluators
- **Completeness** -- Missing value rates by field
- **Duplicates** -- Exact and fuzzy duplicate detection
- **Format Validation** -- Date, amount, and identifier format compliance

### ML Evaluators
- **Feature Distributions** -- Training/test split distribution similarity
- **Label Quality** -- Anomaly label accuracy and coverage
- **Class Balance** -- Target variable distribution

## New v0.11 Evaluators

### Multi-Period Coherence
Validates data consistency across multi-period generation sessions:
- Opening balance = prior period closing balance
- Sequential document IDs across periods
- Consistent entity references

### Fraud Pack Effectiveness
Measures the quality of injected fraud patterns:
- Detection rate at configurable thresholds
- False positive analysis per fraud type
- Pack coverage vs. configured rates

### OCEL Enrichment Quality
Validates OCEL 2.0 enrichment completeness:
- State transition coverage percentage
- Correlation event linking accuracy
- Resource pool utilization distribution

### Causal Intervention Magnitude
Validates that interventions produce expected effects:
- KPI delta vs. expected magnitude
- Propagation path verification
- Constraint preservation checks

## Configuration

```yaml
evaluation:
  enabled: true
  thresholds:
    benford_mad: 0.015
    balance_tolerance: 0.01
    multi_period_coherence: 0.99
    fraud_pack_effectiveness: 0.80
    ocel_enrichment_coverage: 0.95
    intervention_magnitude_tolerance: 0.10
```

## AutoTuner Integration

The AutoTuner reads evaluation results and generates config patches to improve data quality:

```bash
datasynth-data evaluate --output ./output --auto-tune
```

This produces a `config_patch.yaml` that can be merged into the generation config.
