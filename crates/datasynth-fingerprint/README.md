# datasynth-fingerprint

Privacy-preserving synthetic data fingerprinting for DataSynth.

## Overview

The `datasynth-fingerprint` crate provides functionality to:

1. **Extract** statistical fingerprints from real data while preserving privacy
2. **Store** fingerprints in portable `.dsf` (DataSynth Fingerprint) files
3. **Synthesize** generator configurations that produce matching synthetic data
4. **Evaluate** fidelity between synthetic data and source fingerprints

## Quick Start

```rust
use datasynth_fingerprint::{
    extraction::{FingerprintExtractor, DataSource, CsvDataSource},
    io::{FingerprintWriter, FingerprintReader},
    synthesis::ConfigSynthesizer,
};

// Extract fingerprint from real data
let extractor = FingerprintExtractor::new();
let fingerprint = extractor.extract_from_csv("data.csv")?;

// Save to .dsf file
let writer = FingerprintWriter::new();
writer.write_to_file(&fingerprint, "fingerprint.dsf")?;

// Later: Load and synthesize config
let reader = FingerprintReader::new();
let fingerprint = reader.read_from_file("fingerprint.dsf")?;

let synthesizer = ConfigSynthesizer::new();
let config_patch = synthesizer.synthesize(&fingerprint)?;
```

## Privacy Features

The crate implements multiple privacy-preserving mechanisms:

### Differential Privacy
- Laplace noise is added to statistics based on epsilon budget
- Configurable privacy levels: Minimal, Standard, High, Maximum

```rust
use datasynth_fingerprint::extraction::{FingerprintExtractor, ExtractionConfig};
use datasynth_fingerprint::models::PrivacyLevel;

let config = ExtractionConfig::with_privacy_level(PrivacyLevel::High);
let extractor = FingerprintExtractor::with_config(config);
```

### K-Anonymity
- Rare categorical values are suppressed if they appear fewer than k times
- Default k=5 for Standard privacy level

### Privacy Audit Trail
- All privacy decisions are logged in the fingerprint
- Tracks epsilon spent, suppressions, generalizations

## Supported Data Sources

### CSV Files
```rust
let fingerprint = extractor.extract_from_csv("data.csv")?;
```

### Parquet Files
```rust
let source = DataSource::Parquet(ParquetDataSource::new("data.parquet"));
let fingerprint = extractor.extract(&source)?;
```

### JSON/JSONL Files
```rust
// JSON array format
let source = DataSource::Json(JsonDataSource::json_array("data.json"));

// JSONL (newline-delimited) format
let source = DataSource::Json(JsonDataSource::jsonl("data.jsonl"));
```

### Directories (Multi-table)
```rust
// Extract from all supported files in a directory
let fingerprint = extractor.extract_from_directory("./data_folder/")?;
```

### Streaming Extraction (Large Files)
```rust
// Memory-efficient extraction for large CSV files
let fingerprint = extractor.extract_streaming_csv("large_data.csv")?;
```

## Fingerprint Components

A fingerprint contains:

| Component | Description |
|-----------|-------------|
| `manifest` | Metadata, version, checksums, privacy config |
| `schema` | Table structures, column types, relationships |
| `statistics` | Distributions, percentiles, Benford analysis |
| `correlations` | Correlation matrices, copulas (optional) |
| `integrity` | Unique constraints, foreign keys (optional) |
| `rules` | Business rules, balance equations (optional) |
| `anomalies` | Anomaly patterns and rates (optional) |
| `privacy_audit` | Privacy actions and epsilon tracking |

## DSF File Format

The `.dsf` format is a ZIP archive containing:

```
fingerprint.dsf
├── manifest.json      # Version, checksums, privacy config
├── schema.yaml        # Table and column definitions
├── statistics.yaml    # Distribution parameters
├── correlations.yaml  # Correlation matrices (optional)
├── integrity.yaml     # Integrity constraints (optional)
├── rules.yaml         # Business rules (optional)
├── anomalies.yaml     # Anomaly profiles (optional)
└── privacy_audit.json # Privacy audit trail
```

## Digital Signatures

DSF files can be signed for authenticity verification:

```rust
use datasynth_fingerprint::io::{SigningKey, DsfSigner, DsfVerifier};

// Generate a signing key
let key = SigningKey::generate("my-key-id");

// Sign when writing
let signer = DsfSigner::new(key.clone());
writer.write_to_file_signed(&fingerprint, "signed.dsf", &signer)?;

// Verify when reading
let verifier = DsfVerifier::new(key);
let fingerprint = reader.read_from_file_verified("signed.dsf", &verifier)?;
```

## Config Synthesis

Convert fingerprints to generator configurations:

```rust
use datasynth_fingerprint::synthesis::{ConfigSynthesizer, SynthesisOptions};

let options = SynthesisOptions {
    scale: 2.0,              // Generate 2x the original row count
    seed: Some(42),          // Set random seed
    preserve_correlations: true,
    inject_anomalies: true,
};

let synthesizer = ConfigSynthesizer::with_options(options);
let result = synthesizer.synthesize_full(&fingerprint, seed)?;

// result.config_patch - configuration values to apply
// result.copula_generators - for preserving correlations
```

## Fidelity Evaluation

Evaluate how well synthetic data matches the original fingerprint:

```rust
use datasynth_fingerprint::evaluation::FidelityEvaluator;

let evaluator = FidelityEvaluator::new();
let report = evaluator.evaluate(&original_fingerprint, &synthetic_fingerprint)?;

println!("Overall fidelity: {:.2}", report.overall_score);
println!("Statistical fidelity: {:.2}", report.statistical_fidelity);
println!("Correlation fidelity: {:.2}", report.correlation_fidelity);
```

### Statistical Distance Metrics

The fidelity evaluator computes per-column distance metrics:

| Metric | Description |
|--------|-------------|
| KS Statistic | Kolmogorov-Smirnov two-sample test statistic |
| Wasserstein-1 | Earth Mover's Distance via inverse CDF integration (9 percentile knots) |
| JS Divergence | Jensen-Shannon divergence from percentile-bin PMFs (bounded by ln(2)) |

### Distribution CDFs

The distribution fitter supports CDF computation for fitted distributions:

| Distribution | CDF Method |
|-------------|------------|
| Normal | Standard error function |
| LogNormal | Transform to normal CDF |
| Gamma | Regularized incomplete gamma (Lanczos + Lentz CF) |
| Pareto | `1 - (x_m/x)^alpha` |
| PointMass | Step function |
| Mixture | Weighted sum of component CDFs |

## Privacy Levels

| Level | Epsilon | K | Use Case |
|-------|---------|---|----------|
| Minimal | 5.0 | 3 | Low privacy requirements |
| Standard | 1.0 | 5 | Balanced (default) |
| High | 0.5 | 10 | Sensitive data |
| Maximum | 0.1 | 20 | Highly sensitive data |

## API Reference

### Core Types

- `Fingerprint` - Root fingerprint structure
- `SchemaFingerprint` - Table and column schemas
- `StatisticsFingerprint` - Numeric and categorical statistics
- `CorrelationFingerprint` - Correlation matrices and copulas
- `PrivacyAudit` - Privacy action tracking

### Extraction

- `FingerprintExtractor` - Main extraction coordinator
- `DataSource` - Data source types (CSV, Parquet, JSON, Directory, Memory)
- `ExtractionConfig` - Extraction configuration
- `StreamingNumericStats` / `StreamingCategoricalStats` - Online statistics

### I/O

- `FingerprintWriter` - Write .dsf files
- `FingerprintReader` - Read .dsf files
- `SigningKey` / `DsfSigner` / `DsfVerifier` - Digital signatures
- `validate_dsf()` - Validate .dsf file integrity

### Synthesis

- `ConfigSynthesizer` - Convert fingerprints to configs
- `ConfigPatch` - Configuration patch values
- `CopulaGenerator` - Generate correlated samples
- `DistributionFitter` - Fit distributions to data

### Evaluation

- `FidelityEvaluator` - Compare fingerprints
- `FidelityReport` - Evaluation results

## CLI Integration

The fingerprint crate integrates with the `datasynth-data` CLI:

```bash
# Extract fingerprint from data
datasynth-data fingerprint extract \
    --input ./real_data/ \
    --output ./fingerprint.dsf \
    --privacy-level standard

# Validate fingerprint file
datasynth-data fingerprint validate ./fingerprint.dsf

# Generate from fingerprint
datasynth-data generate \
    --fingerprint ./fingerprint.dsf \
    --output ./synthetic/ \
    --scale 1.0

# Evaluate fidelity
datasynth-data fingerprint evaluate \
    --fingerprint ./fingerprint.dsf \
    --synthetic ./synthetic/
```

## License

Same as the parent DataSynth project.
