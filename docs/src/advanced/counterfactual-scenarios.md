# Counterfactual Scenarios

The counterfactual scenario engine enables what-if analysis by generating paired baseline and counterfactual datasets using a causal directed acyclic graph (DAG).

## Causal DAG

The default DAG contains 17 financial process nodes:

| Node | Description |
|------|-------------|
| gdp_growth | GDP growth rate |
| interest_rates | Market interest rates |
| consumer_confidence | Consumer confidence index |
| transaction_volume | Overall transaction volume |
| revenue | Company revenue |
| expenses | Operating expenses |
| fraud_rate | Rate of fraudulent transactions |
| control_effectiveness | Internal control effectiveness |
| audit_risk | Audit risk level |
| misstatement_risk | Financial misstatement risk |
| detection_rate | Fraud detection rate |
| approval_threshold | Approval threshold amount |
| sod_violations | Segregation of duties violations |
| late_postings | Late posting frequency |
| period_end_adjustments | Period-end adjustment volume |
| intercompany_volume | Intercompany transaction volume |
| cash_flow | Operating cash flow |

## Transfer Functions

8 transfer function types model causal relationships:

| Function | Description |
|----------|-------------|
| Linear | `y = strength * x + offset` |
| Exponential | `y = strength * e^(rate * x)` |
| Logistic | S-curve saturation |
| InverseLogistic | Inverse S-curve |
| Step | Binary threshold |
| Threshold | Activation above/below value |
| Decay | Exponential decay |
| Piecewise | Multi-segment linear |

## DAG Presets

| Preset | Nodes | Description |
|--------|-------|-------------|
| `minimal` | 6 | Core accounting relationships only |
| `financial_process` | 12 | Includes document flows and period close |
| `full` | 17 | Complete causal graph |

## Interventions

Interventions modify node values to create counterfactual scenarios:

```yaml
scenarios:
  - name: "recession_impact"
    interventions:
      - type: ParameterShift
        target_node: gdp_growth
        magnitude: -0.03
        timing: immediate
      - type: MacroShock
        target_node: interest_rates
        magnitude: 0.02
        timing: gradual
```

## ConfigMutator Constraints

The ConfigMutator applies interventions while preserving data integrity:
- `preserve_accounting_identity` -- Assets = Liabilities + Equity
- `preserve_document_chains` -- PO -> GR -> Invoice -> Payment integrity
- `preserve_period_close` -- Fiscal period boundaries maintained
- `preserve_balance_coherence` -- Trial balance consistency

## CLI Usage

```bash
# List available scenarios
datasynth-data scenario list

# Generate baseline + counterfactual pair
datasynth-data scenario generate --config config.yaml --scenario recession_impact --output ./output

# Compute diff between baseline and counterfactual
datasynth-data scenario diff --baseline ./output/baseline --counterfactual ./output/counterfactual
```

## Python Usage

```python
from datasynth_py.config import blueprints

config = blueprints.retail_small()
config = blueprints.with_scenarios(config, template="fraud_detection", with_interventions=True)
```
