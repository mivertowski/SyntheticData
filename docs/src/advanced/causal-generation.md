# Causal & Counterfactual Generation

> **New in v0.5.0**

DataSynth supports Structural Causal Models (SCMs) for generating data with explicit causal structure, running interventional "what-if" scenarios, and producing counterfactual records.

## Overview

Traditional synthetic data generators capture correlations but not causation. Causal generation lets you:

1. **Define causal relationships** between variables (e.g., "transaction amount causes approval level")
2. **Generate observational data** that follows the causal structure
3. **Run interventions** to answer "what if?" questions (do-calculus)
4. **Produce counterfactuals** — "what would have happened if X were different?"

This is particularly valuable for fraud detection, audit analytics, and regulatory "what-if" scenario testing.

## Causal Graph

A causal graph defines variables and the directed edges (causal mechanisms) between them.

### Variables

```rust
use synth_core::causal::{CausalVariable, CausalVarType};

let var = CausalVariable::new("transaction_amount", CausalVarType::Continuous)
    .with_distribution("lognormal")
    .with_param("mu", 8.0)
    .with_param("sigma", 1.5);
```

| Variable Type | Description | Example |
|---------------|-------------|---------|
| `Continuous` | Real-valued | Transaction amount, revenue |
| `Categorical` | Discrete categories | Industry, department |
| `Count` | Non-negative integers | Line items, approvals |
| `Binary` | Boolean (0/1) | Fraud flag, approval status |

### Causal Mechanisms

Edges between variables define how a parent causally affects a child:

```rust
use synth_core::causal::{CausalEdge, CausalMechanism};

let edge = CausalEdge {
    from: "transaction_amount".into(),
    to: "approval_level".into(),
    mechanism: CausalMechanism::Logistic { scale: 0.001, midpoint: 50000.0 },
    strength: 1.0,
};
```

| Mechanism | Formula | Use Case |
|-----------|---------|----------|
| `Linear { coefficient }` | y += coefficient × parent | Proportional effects |
| `Threshold { cutoff }` | y = 1 if parent > cutoff, else 0 | Binary triggers |
| `Polynomial { coefficients }` | y += Σ coefficients[i] × parent^i | Non-linear effects |
| `Logistic { scale, midpoint }` | y += 1 / (1 + e^(-scale × (parent - midpoint))) | S-curve effects |

### Building a Graph

```rust
use synth_core::causal::{CausalGraph, CausalVariable, CausalVarType, CausalEdge, CausalMechanism};

let mut graph = CausalGraph::new();

// Add variables
graph.add_variable(
    CausalVariable::new("transaction_amount", CausalVarType::Continuous)
        .with_distribution("lognormal")
        .with_param("mu", 8.0)
        .with_param("sigma", 1.5)
);
graph.add_variable(
    CausalVariable::new("approval_level", CausalVarType::Count)
        .with_distribution("normal")
        .with_param("mean", 1.0)
        .with_param("std", 0.5)
);
graph.add_variable(
    CausalVariable::new("fraud_flag", CausalVarType::Binary)
);

// Add causal edges
graph.add_edge(CausalEdge {
    from: "transaction_amount".into(),
    to: "approval_level".into(),
    mechanism: CausalMechanism::Linear { coefficient: 0.00005 },
    strength: 1.0,
});
graph.add_edge(CausalEdge {
    from: "transaction_amount".into(),
    to: "fraud_flag".into(),
    mechanism: CausalMechanism::Logistic { scale: 0.0001, midpoint: 50000.0 },
    strength: 0.8,
});

// Validate (checks for cycles, missing variables)
graph.validate()?;
```

## Built-in Templates

DataSynth includes pre-configured causal graphs for common financial scenarios:

### Fraud Detection Template

```rust
let graph = CausalGraph::fraud_detection_template();
```

Variables: `transaction_amount`, `approval_level`, `vendor_risk`, `fraud_flag`

Causal structure:
- `transaction_amount` → `approval_level` (linear)
- `transaction_amount` → `fraud_flag` (logistic)
- `vendor_risk` → `fraud_flag` (linear)

### Revenue Cycle Template

```rust
let graph = CausalGraph::revenue_cycle_template();
```

Variables: `order_size`, `credit_score`, `payment_delay`, `revenue`

Causal structure:
- `order_size` → `revenue` (linear)
- `credit_score` → `payment_delay` (linear, negative)
- `order_size` → `payment_delay` (linear)

## Structural Causal Model (SCM)

The SCM wraps a causal graph and provides generation capabilities:

```rust
use synth_core::causal::StructuralCausalModel;

let scm = StructuralCausalModel::new(graph)?;

// Generate observational data
let samples = scm.generate(10000, 42)?;
// samples: Vec<HashMap<String, f64>>

for sample in &samples[..3] {
    println!("Amount: {:.2}, Approval: {:.0}, Fraud: {:.0}",
        sample["transaction_amount"],
        sample["approval_level"],
        sample["fraud_flag"],
    );
}
```

Data is generated in topological order — root variables are sampled from their distributions first, then child variables are computed based on their parents' values and the causal mechanisms.

## Interventions (Do-Calculus)

Interventions answer "what would happen if we force variable X to value V?", cutting all incoming causal edges to X.

### Single Intervention

```rust
let intervened = scm.intervene("transaction_amount", 50000.0)?;
let samples = intervened.generate(5000, 42)?;
```

### Multiple Interventions

```rust
let intervened = scm
    .intervene("transaction_amount", 50000.0)?
    .and_intervene("vendor_risk", 0.9);
let samples = intervened.generate(5000, 42)?;
```

### Intervention Engine with Effect Estimation

```rust
use synth_core::causal::InterventionEngine;

let engine = InterventionEngine::new(scm);

let result = engine.do_intervention(
    &[("transaction_amount".into(), 50000.0)],
    5000,  // samples
    42,    // seed
)?;

// Compare baseline vs intervention
println!("Baseline fraud rate: {:.4}",
    result.baseline_samples.iter()
        .map(|s| s["fraud_flag"])
        .sum::<f64>() / result.baseline_samples.len() as f64
);

// Effect estimates with confidence intervals
for (var, effect) in &result.effect_estimates {
    println!("{}: ATE={:.4}, 95% CI=({:.4}, {:.4})",
        var,
        effect.average_treatment_effect,
        effect.confidence_interval.0,
        effect.confidence_interval.1,
    );
}
```

The `InterventionResult` contains:

| Field | Description |
|-------|-------------|
| `baseline_samples` | Data generated without intervention |
| `intervened_samples` | Data generated with the intervention applied |
| `effect_estimates` | Per-variable average treatment effects with confidence intervals |

## Counterfactual Generation

Counterfactuals answer "what would have happened to this specific record if X were different?" using the abduction-action-prediction framework:

1. **Abduction**: Infer the latent noise variables from the factual observation
2. **Action**: Apply the intervention (change X to new value)
3. **Prediction**: Propagate through the SCM with inferred noise

```rust
use synth_core::causal::CounterfactualGenerator;
use std::collections::HashMap;

let cf_gen = CounterfactualGenerator::new(scm);

// Factual record
let factual: HashMap<String, f64> = [
    ("transaction_amount".to_string(), 5000.0),
    ("approval_level".to_string(), 1.0),
    ("fraud_flag".to_string(), 0.0),
].into_iter().collect();

// What if the amount had been 100,000?
let counterfactual = cf_gen.generate_counterfactual(
    &factual,
    "transaction_amount",
    100000.0,
    42,
)?;

println!("Factual fraud_flag: {}", factual["fraud_flag"]);
println!("Counterfactual fraud_flag: {}", counterfactual["fraud_flag"]);
```

### Batch Counterfactuals

```rust
let pairs = cf_gen.generate_batch_counterfactuals(
    &factual_records,
    "transaction_amount",
    100000.0,
    42,
)?;

for pair in &pairs {
    println!("Changed variables: {:?}", pair.changed_variables);
}
```

Each `CounterfactualPair` contains:

| Field | Description |
|-------|-------------|
| `factual` | The original observation |
| `counterfactual` | The counterfactual version |
| `changed_variables` | List of variables that changed |

## Causal Validation

Validate that generated data preserves the specified causal structure:

```rust
use synth_core::causal::CausalValidator;

let report = CausalValidator::validate_causal_structure(&samples, &graph);

println!("Valid: {}", report.valid);
for check in &report.checks {
    println!("{}: {} — {}", check.name, if check.passed { "PASS" } else { "FAIL" }, check.details);
}
if !report.violations.is_empty() {
    println!("Violations: {:?}", report.violations);
}
```

The validator checks:
- Causal edge directions are respected (parent-child correlations)
- Independence constraints hold (non-adjacent variables)
- Intervention effects are consistent with the graph structure

## CLI Usage

### Generate Observational Data

```bash
datasynth-data causal generate \
    --template fraud_detection \
    --samples 10000 \
    --seed 42 \
    --output ./causal_output
```

### Run Interventions

```bash
datasynth-data causal intervene \
    --template fraud_detection \
    --variable transaction_amount \
    --value 50000 \
    --samples 5000 \
    --output ./intervention_output
```

### Validate Causal Structure

```bash
datasynth-data causal validate \
    --data ./causal_output \
    --template fraud_detection
```

## Configuration

```yaml
causal:
  enabled: true
  template: "fraud_detection"   # or "revenue_cycle" or path to custom YAML
  sample_size: 10000
  validate: true                # validate causal structure in output
```

### Custom Causal Graph YAML

```yaml
# custom_graph.yaml
variables:
  - name: order_size
    type: continuous
    distribution: lognormal
    params:
      mu: 7.0
      sigma: 1.2
  - name: discount_rate
    type: continuous
    distribution: beta
    params:
      alpha: 2.0
      beta: 8.0
  - name: revenue
    type: continuous

edges:
  - from: order_size
    to: revenue
    mechanism:
      type: linear
      coefficient: 0.95
  - from: discount_rate
    to: revenue
    mechanism:
      type: linear
      coefficient: -5000.0
```

## See Also

- [AI & ML Configuration](../configuration/ai-ml-features.md)
- [Causal Analysis Use Case](../use-cases/causal-analysis.md)
- [datasynth-core Causal Module](../crates/datasynth-core.md)
- [Generation Pipeline](../architecture/generation-pipeline.md)
