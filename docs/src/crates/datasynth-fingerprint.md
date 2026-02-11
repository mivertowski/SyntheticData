# datasynth-fingerprint

Privacy-preserving fingerprint extraction from real data and synthesis of matching synthetic data.

## Overview

The `datasynth-fingerprint` crate provides tools for extracting statistical fingerprints from real datasets while preserving privacy through differential privacy mechanisms and k-anonymity. These fingerprints can then be used to generate synthetic data that matches the statistical properties of the original data without exposing sensitive information.

## Architecture

```
Real Data → Extract → .dsf File → Generate → Synthetic Data → Evaluate
```

The fingerprinting workflow consists of three main stages:

1. **Extraction**: Analyze real data and extract statistical properties
2. **Synthesis**: Generate configuration and synthetic data from fingerprints
3. **Evaluation**: Validate synthetic data fidelity against fingerprints

## Key Components

### Models (`models/`)

| Model | Description |
|-------|-------------|
| **Fingerprint** | Root container with manifest, schema, statistics, correlations, integrity, rules, anomalies, privacy_audit |
| **Manifest** | Version, format, created_at, source metadata, privacy metadata, checksums, optional signature |
| **SchemaFingerprint** | Tables with columns, data types, cardinalities, relationships |
| **StatisticsFingerprint** | Numeric stats (distribution, percentiles, Benford), categorical stats (frequencies, entropy) |
| **CorrelationFingerprint** | Correlation matrices with copula parameters |
| **IntegrityFingerprint** | Foreign key definitions, cardinality rules |
| **RulesFingerprint** | Balance rules, approval thresholds |
| **AnomalyFingerprint** | Anomaly rates, type distributions, temporal patterns |
| **PrivacyAudit** | Actions log, epsilon spent, k-anonymity, warnings |

### Privacy Engine (`privacy/`)

| Component | Description |
|-----------|-------------|
| **LaplaceMechanism** | Differential privacy with configurable epsilon |
| **GaussianMechanism** | Alternative DP mechanism for (ε,δ)-privacy |
| **KAnonymity** | Suppression of rare categorical values below k threshold |
| **PrivacyEngine** | Unified interface combining DP, k-anonymity, winsorization |
| **PrivacyAuditBuilder** | Build privacy audit with actions and warnings |

#### Privacy Levels

| Level | Epsilon | k | Outlier % | Use Case |
|-------|---------|---|-----------|----------|
| Minimal | 5.0 | 3 | 99% | Low privacy, high utility |
| Standard | 1.0 | 5 | 95% | Balanced (default) |
| High | 0.5 | 10 | 90% | Higher privacy |
| Maximum | 0.1 | 20 | 85% | Maximum privacy |

### Extraction Engine (`extraction/`)

| Extractor | Description |
|-----------|-------------|
| **FingerprintExtractor** | Main coordinator for all extraction |
| **SchemaExtractor** | Infer data types, cardinalities, relationships |
| **StatsExtractor** | Compute distributions, percentiles, Benford analysis |
| **CorrelationExtractor** | Pearson correlations, copula fitting |
| **IntegrityExtractor** | Detect foreign key relationships |
| **RulesExtractor** | Detect balance rules, approval patterns |
| **AnomalyExtractor** | Analyze anomaly rates and patterns |

### I/O (`io/`)

| Component | Description |
|-----------|-------------|
| **FingerprintWriter** | Write .dsf files (ZIP with YAML/JSON components) |
| **FingerprintReader** | Read .dsf files with checksum verification |
| **FingerprintValidator** | Validate DSF structure and integrity |
| **validate_dsf()** | Convenience function for CLI validation |

### Synthesis (`synthesis/`)

| Component | Description |
|-----------|-------------|
| **ConfigSynthesizer** | Convert fingerprint to GeneratorConfig |
| **DistributionFitter** | Fit AmountSampler parameters from statistics |
| **GaussianCopula** | Generate correlated values preserving multivariate structure |

### Evaluation (`evaluation/`)

| Component | Description |
|-----------|-------------|
| **FidelityEvaluator** | Compare synthetic data against fingerprint |
| **FidelityReport** | Overall score, component scores, pass/fail status |
| **FidelityConfig** | Thresholds and weights for evaluation |

### Federated Fingerprinting (`federated/`) — v0.5.0

| Component | Description |
|-----------|-------------|
| **FederatedFingerprintProtocol** | Orchestrates multi-source fingerprint aggregation |
| **PartialFingerprint** | Per-source fingerprint with local DP (epsilon, means, stds, correlations) |
| **AggregatedFingerprint** | Combined fingerprint with total epsilon and source count |
| **AggregationMethod** | WeightedAverage, Median, or TrimmedMean strategies |
| **FederatedConfig** | min_sources, max_epsilon_per_source, aggregation_method |

### Certificates (`certificates/`) — v0.5.0

| Component | Description |
|-----------|-------------|
| **SyntheticDataCertificate** | Certificate with DP guarantees, quality metrics, config hash, signature |
| **CertificateBuilder** | Builder pattern for constructing certificates |
| **DpGuarantee** | DP mechanism, epsilon, delta, composition method, total queries |
| **QualityMetrics** | Benford MAD, correlation preservation, statistical fidelity, MIA AUC |
| **sign_certificate()** | HMAC-SHA256 signing |
| **verify_certificate()** | Signature verification |

### Privacy-Utility Frontier (`privacy/pareto.rs`) — v0.5.0

| Component | Description |
|-----------|-------------|
| **ParetoFrontier** | Explore privacy-utility tradeoff space |
| **ParetoPoint** | Epsilon, utility_score, benford_mad, correlation_score |
| **recommend()** | Recommend optimal epsilon for target utility |

## DSF File Format

The DataSynth Fingerprint (`.dsf`) file is a ZIP archive containing:

```
fingerprint.dsf (ZIP)
├── manifest.json       # Version, checksums, privacy config
├── schema.yaml         # Tables, columns, relationships
├── statistics.yaml     # Distributions, percentiles, Benford
├── correlations.yaml   # Correlation matrices, copulas
├── integrity.yaml      # FK relationships, cardinality
├── rules.yaml          # Balance constraints, approval thresholds
├── anomalies.yaml      # Anomaly rates, type distribution
└── privacy_audit.json  # Privacy decisions, epsilon spent
```

## Usage

### Extracting a Fingerprint

```rust
use datasynth_fingerprint::{
    extraction::FingerprintExtractor,
    privacy::{PrivacyEngine, PrivacyLevel},
    io::FingerprintWriter,
};

// Create privacy engine with standard level
let privacy = PrivacyEngine::new(PrivacyLevel::Standard);

// Extract fingerprint from CSV data
let extractor = FingerprintExtractor::new(privacy);
let fingerprint = extractor.extract_from_csv("data.csv")?;

// Write to DSF file
let writer = FingerprintWriter::new();
writer.write(&fingerprint, "fingerprint.dsf")?;
```

### Reading a Fingerprint

```rust
use datasynth_fingerprint::io::FingerprintReader;

let reader = FingerprintReader::new();
let fingerprint = reader.read("fingerprint.dsf")?;

println!("Tables: {:?}", fingerprint.schema.tables.len());
println!("Privacy epsilon spent: {}", fingerprint.privacy_audit.epsilon_spent);
```

### Validating a Fingerprint

```rust
use datasynth_fingerprint::io::validate_dsf;

match validate_dsf("fingerprint.dsf") {
    Ok(report) => println!("Valid: {:?}", report),
    Err(e) => eprintln!("Invalid: {}", e),
}
```

### Synthesizing Configuration

```rust
use datasynth_fingerprint::synthesis::ConfigSynthesizer;

let synthesizer = ConfigSynthesizer::new();
let config = synthesizer.synthesize(&fingerprint)?;

// Use config with datasynth-generators
```

### Evaluating Fidelity

```rust
use datasynth_fingerprint::evaluation::{FidelityEvaluator, FidelityConfig};

let config = FidelityConfig::default();
let evaluator = FidelityEvaluator::new(config);

let report = evaluator.evaluate(&fingerprint, "./synthetic_data/")?;

println!("Overall score: {:.2}", report.overall_score);
println!("Pass: {}", report.passed);

for (metric, score) in &report.component_scores {
    println!("  {}: {:.2}", metric, score);
}
```

### Federated Fingerprinting

```rust
use datasynth_fingerprint::federated::{
    FederatedFingerprintProtocol, FederatedConfig, AggregationMethod,
};

let config = FederatedConfig {
    min_sources: 2,
    max_epsilon_per_source: 5.0,
    aggregation_method: AggregationMethod::WeightedAverage,
};

let protocol = FederatedFingerprintProtocol::new(config);

// Create partial fingerprints from each data source
let partial1 = FederatedFingerprintProtocol::create_partial(
    "source_a", vec!["amount".into(), "count".into()], 10000,
    vec![5000.0, 3.0], vec![2000.0, 1.5], 1.0,
);
let partial2 = FederatedFingerprintProtocol::create_partial(
    "source_b", vec!["amount".into(), "count".into()], 8000,
    vec![4500.0, 2.8], vec![1800.0, 1.2], 1.0,
);

// Aggregate without centralizing raw data
let aggregated = protocol.aggregate(&[partial1, partial2])?;
println!("Total epsilon: {}", aggregated.total_epsilon);
```

### Synthetic Data Certificates

```rust
use datasynth_fingerprint::certificates::{
    CertificateBuilder, DpGuarantee, QualityMetrics,
    sign_certificate, verify_certificate,
};

let mut cert = CertificateBuilder::new("DataSynth v0.5.0")
    .with_dp_guarantee(DpGuarantee {
        mechanism: "Laplace".into(),
        epsilon: 1.0,
        delta: None,
        composition_method: "sequential".into(),
        total_queries: 50,
    })
    .with_quality_metrics(QualityMetrics {
        benford_mad: Some(0.008),
        correlation_preservation: Some(0.95),
        statistical_fidelity: Some(0.92),
        mia_auc: Some(0.52),
    })
    .with_seed(42)
    .build();

// Sign and verify
sign_certificate(&mut cert, "my-secret-key");
assert!(verify_certificate(&cert, "my-secret-key"));
```

## Fidelity Metrics

| Category | Metrics |
|----------|---------|
| **Statistical** | KS statistic, Wasserstein distance, Benford MAD |
| **Correlation** | Correlation matrix RMSE |
| **Schema** | Column type match, row count ratio |
| **Rules** | Balance equation compliance rate |

## Privacy Guarantees

The fingerprint extraction process provides the following privacy guarantees:

1. **Differential Privacy**: Numeric statistics are perturbed using Laplace or Gaussian mechanisms with configurable epsilon budget
2. **K-Anonymity**: Categorical values appearing fewer than k times are suppressed
3. **Winsorization**: Outliers are clipped to prevent identification of extreme values
4. **Audit Trail**: All privacy decisions are logged for compliance verification

## CLI Commands

```bash
# Extract fingerprint
datasynth-data fingerprint extract \
    --input ./data.csv \
    --output ./fp.dsf \
    --privacy-level standard

# Validate
datasynth-data fingerprint validate ./fp.dsf

# Show info
datasynth-data fingerprint info ./fp.dsf --detailed

# Compare
datasynth-data fingerprint diff ./fp1.dsf ./fp2.dsf

# Evaluate fidelity
datasynth-data fingerprint evaluate \
    --fingerprint ./fp.dsf \
    --synthetic ./synthetic/ \
    --threshold 0.8

# Federated fingerprinting
datasynth-data fingerprint federated \
    --sources ./source_a.dsf ./source_b.dsf \
    --output ./aggregated.dsf \
    --method weighted_average

# Generate with certificate
datasynth-data generate --config config.yaml --output ./output --certificate
```

## Dependencies

```toml
[dependencies]
datasynth-core = { path = "../datasynth-core" }
datasynth-config = { path = "../datasynth-config" }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
zip = "0.6"
sha2 = "0.10"
rand = "0.8"
statrs = "0.16"
```

## See Also

- [Fingerprinting Guide](../advanced/fingerprinting.md)
- [CLI Reference](../user-guide/cli-reference.md#fingerprint)
- [Privacy Model](../../fingerprint/concepts/03-privacy-model.md)
- [Fidelity Model](../../fingerprint/concepts/04-fidelity-model.md)
