# Synthetic Data Certificates

> **New in v0.5.0**

Synthetic data certificates provide cryptographic proof of the privacy guarantees and quality metrics associated with generated data.

## Overview

As synthetic data becomes increasingly used in regulated industries, organizations need verifiable assurance that:

1. The data was generated with specific differential privacy guarantees
2. Quality metrics meet documented thresholds
3. The generation configuration hasn't been tampered with
4. The certificate itself is authentic (HMAC-SHA256 signed)

Certificates are produced during generation and can be embedded in output files or distributed alongside them.

## Certificate Structure

```rust
pub struct SyntheticDataCertificate {
    pub certificate_id: String,        // Unique certificate identifier
    pub generation_timestamp: String,  // ISO 8601 timestamp
    pub generator_version: String,     // DataSynth version
    pub config_hash: String,           // SHA-256 hash of generation config
    pub seed: Option<u64>,             // RNG seed for reproducibility
    pub dp_guarantee: Option<DpGuarantee>,
    pub quality_metrics: Option<QualityMetrics>,
    pub fingerprint_hash: Option<String>,  // Source fingerprint hash
    pub issuer: String,                // Certificate issuer
    pub signature: Option<String>,     // HMAC-SHA256 signature
}
```

### DP Guarantee

```rust
pub struct DpGuarantee {
    pub mechanism: String,            // "Laplace" or "Gaussian"
    pub epsilon: f64,                 // Privacy budget spent
    pub delta: Option<f64>,           // For (ε,δ)-DP
    pub composition_method: String,   // "sequential", "advanced", "rdp"
    pub total_queries: u32,           // Number of DP queries made
}
```

### Quality Metrics

```rust
pub struct QualityMetrics {
    pub benford_mad: Option<f64>,             // Mean Absolute Deviation from Benford's Law
    pub correlation_preservation: Option<f64>, // Correlation matrix similarity (0-1)
    pub statistical_fidelity: Option<f64>,    // Overall statistical fidelity score (0-1)
    pub mia_auc: Option<f64>,                 // Membership Inference Attack AUC (closer to 0.5 = better privacy)
}
```

## Building Certificates

Use the `CertificateBuilder` for fluent construction:

```rust
use datasynth_fingerprint::certificates::{
    CertificateBuilder, DpGuarantee, QualityMetrics,
};

let cert = CertificateBuilder::new("DataSynth v0.5.0")
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
    .with_config_hash("sha256:abc123...")
    .with_seed(42)
    .with_fingerprint_hash("sha256:def456...")
    .with_generator_version("0.5.0")
    .build();
```

## Signing and Verification

Certificates are signed using HMAC-SHA256:

```rust
use datasynth_fingerprint::certificates::{sign_certificate, verify_certificate};

// Sign
sign_certificate(&mut cert, "my-secret-key-material");

// Verify
let valid = verify_certificate(&cert, "my-secret-key-material");
assert!(valid);

// Tampered certificate fails verification
cert.dp_guarantee.as_mut().unwrap().epsilon = 0.001; // tamper
assert!(!verify_certificate(&cert, "my-secret-key-material"));
```

## Output Embedding

Certificates can be:

1. **Standalone JSON**: Written as `certificate.json` in the output directory
2. **Parquet metadata**: Embedded in Parquet file metadata under the `datasynth_certificate` key
3. **JSON metadata**: Included in the generation manifest

## CLI Usage

```bash
# Generate data with certificate
datasynth-data generate \
    --config config.yaml \
    --output ./output \
    --certificate

# Certificate is written to ./output/certificate.json
```

## Configuration

```yaml
certificates:
  enabled: true
  issuer: "DataSynth"
  include_quality_metrics: true
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable certificate generation |
| `issuer` | string | `"DataSynth"` | Issuer identity |
| `include_quality_metrics` | bool | `true` | Include quality metrics in certificate |

## Privacy-Utility Pareto Frontier

The `ParetoFrontier` helps find optimal privacy-utility tradeoffs:

```rust
use datasynth_fingerprint::privacy::pareto::{ParetoFrontier, ParetoPoint};

let epsilons = vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0];

let points = ParetoFrontier::explore(&epsilons, |epsilon| {
    // Evaluate utility at this epsilon level
    ParetoPoint {
        epsilon,
        delta: None,
        utility_score: compute_utility(epsilon),
        benford_mad: compute_benford(epsilon),
        correlation_score: compute_correlation(epsilon),
    }
});

// Recommend epsilon for target utility
if let Some(recommended_epsilon) = ParetoFrontier::recommend(&points, 0.90) {
    println!("For 90% utility, use epsilon = {:.2}", recommended_epsilon);
}
```

The frontier identifies non-dominated points where no other configuration achieves both better privacy and better utility.

## See Also

- [Federated Fingerprinting](federated-fingerprinting.md)
- [Fingerprinting Guide](fingerprinting.md)
- [AI & ML Configuration](../configuration/ai-ml-features.md)
- [Compliance & Regulatory](../compliance/README.md)
