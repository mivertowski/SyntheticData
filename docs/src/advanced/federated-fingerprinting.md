# Federated Fingerprinting

> **New in v0.5.0**

Federated fingerprinting enables extracting statistical fingerprints from multiple distributed data sources and combining them without centralizing the raw data.

## Overview

In many enterprise environments, data is distributed across multiple systems, departments, or legal entities that cannot share raw data due to privacy regulations or data governance policies. Federated fingerprinting addresses this by:

1. **Local extraction**: Each data source extracts a partial fingerprint with its own differential privacy budget
2. **Secure aggregation**: Partial fingerprints are combined using a configurable aggregation strategy
3. **Privacy composition**: The total privacy budget is tracked across all sources

```
Source A → [Extract + Local DP] → Partial FP A ─┐
Source B → [Extract + Local DP] → Partial FP B ─┼→ [Aggregate] → Combined FP → [Generate]
Source C → [Extract + Local DP] → Partial FP C ─┘
```

## Partial Fingerprints

Each data source produces a `PartialFingerprint` containing noised statistics:

```rust
pub struct PartialFingerprint {
    pub source_id: String,         // Identifier for this data source
    pub local_epsilon: f64,        // DP epsilon budget spent locally
    pub record_count: u64,         // Number of records in source
    pub column_names: Vec<String>, // Column identifiers
    pub means: Vec<f64>,           // Per-column means (noised)
    pub stds: Vec<f64>,            // Per-column standard deviations (noised)
    pub mins: Vec<f64>,            // Per-column minimums (noised)
    pub maxs: Vec<f64>,            // Per-column maximums (noised)
    pub correlations: Vec<f64>,    // Flat row-major correlation matrix (noised)
}
```

### Creating a Partial Fingerprint

```rust
use datasynth_fingerprint::federated::FederatedFingerprintProtocol;

let partial = FederatedFingerprintProtocol::create_partial(
    "department_finance",                        // source ID
    vec!["amount".into(), "line_items".into()],  // columns
    50000,                                       // record count
    vec![8500.0, 3.2],                           // means
    vec![4200.0, 1.8],                           // standard deviations
    1.0,                                         // local epsilon budget
);
```

## Aggregation Methods

| Method | Description | Properties |
|--------|-------------|------------|
| **WeightedAverage** | Weighted by record count | Best for balanced sources |
| **Median** | Median across sources | Robust to outlier sources |
| **TrimmedMean** | Mean after removing extremes | Balances robustness and efficiency |

## Protocol Usage

```rust
use datasynth_fingerprint::federated::{
    FederatedFingerprintProtocol, FederatedConfig, AggregationMethod,
};

// Configure the protocol
let config = FederatedConfig {
    min_sources: 2,                                // Minimum sources required
    max_epsilon_per_source: 5.0,                   // Max DP budget per source
    aggregation_method: AggregationMethod::WeightedAverage,
};

let protocol = FederatedFingerprintProtocol::new(config);

// Collect partial fingerprints from each source
let partial_a = FederatedFingerprintProtocol::create_partial(
    "source_a", vec!["amount".into(), "count".into()],
    10000, vec![5000.0, 3.0], vec![2000.0, 1.5], 1.0,
);
let partial_b = FederatedFingerprintProtocol::create_partial(
    "source_b", vec!["amount".into(), "count".into()],
    8000, vec![4500.0, 2.8], vec![1800.0, 1.2], 1.0,
);
let partial_c = FederatedFingerprintProtocol::create_partial(
    "source_c", vec!["amount".into(), "count".into()],
    12000, vec![5500.0, 3.3], vec![2200.0, 1.7], 1.0,
);

// Aggregate
let aggregated = protocol.aggregate(&[partial_a, partial_b, partial_c])?;

println!("Total records: {}", aggregated.total_record_count);  // 30000
println!("Total epsilon: {}", aggregated.total_epsilon);        // 3.0 (sum)
println!("Sources: {}", aggregated.source_count);               // 3
```

## Aggregated Fingerprint

The `AggregatedFingerprint` contains the combined statistics:

```rust
pub struct AggregatedFingerprint {
    pub column_names: Vec<String>,
    pub means: Vec<f64>,            // Aggregated means
    pub stds: Vec<f64>,             // Aggregated standard deviations
    pub mins: Vec<f64>,             // Aggregated minimums
    pub maxs: Vec<f64>,             // Aggregated maximums
    pub correlations: Vec<f64>,     // Aggregated correlation matrix
    pub total_record_count: u64,    // Sum across all sources
    pub total_epsilon: f64,         // Sum of local epsilons
    pub source_count: usize,        // Number of contributing sources
}
```

## Privacy Budget Composition

The total privacy budget is the sum of local epsilons across all sources. This follows sequential composition — each source's local DP guarantee composes with the others.

For example, if three sources each spend ε=1.0 locally, the total privacy cost of the aggregated fingerprint is ε=3.0 under sequential composition.

To minimize total budget:
- Use the lowest `local_epsilon` that provides sufficient utility
- Prefer fewer sources with more records over many sources with few records
- Use `max_epsilon_per_source` to enforce per-source budget caps

## CLI Usage

```bash
# Aggregate fingerprints from multiple sources
datasynth-data fingerprint federated \
    --sources ./finance.dsf ./operations.dsf ./sales.dsf \
    --output ./aggregated.dsf \
    --method weighted_average \
    --max-epsilon 5.0

# Then generate from the aggregated fingerprint
datasynth-data generate \
    --fingerprint ./aggregated.dsf \
    --output ./synthetic_output
```

## Configuration

```yaml
# Federated config is specified per-invocation via CLI flags
# The aggregation method and privacy budget are controlled at execution time
```

| CLI Flag | Default | Description |
|----------|---------|-------------|
| `--sources` | (required) | Two or more .dsf fingerprint files |
| `--output` | (required) | Output path for aggregated fingerprint |
| `--method` | `weighted_average` | Aggregation strategy |
| `--max-epsilon` | `5.0` | Maximum epsilon per source |

## See Also

- [Fingerprinting Guide](fingerprinting.md)
- [Synthetic Data Certificates](certificates.md)
- [datasynth-fingerprint Crate](../crates/datasynth-fingerprint.md)
- [Privacy & Regulatory Compliance](../compliance/README.md)
