# GDPR Compliance

This document provides GDPR (General Data Protection Regulation) compliance guidance for DataSynth deployments. DataSynth generates purely synthetic data by default, but certain workflows (fingerprint extraction) may process real personal data.

## Synthetic Data and GDPR

### Pure Synthetic Generation

When DataSynth generates data from configuration alone (no real data input):

- **No personal data is processed**: All names, identifiers, and transactions are algorithmically generated
- **No data subjects exist**: Synthetic entities have no real-world counterparts
- **GDPR does not apply** to the generated output, as it contains no personal data per Article 4(1)

This is the default operating mode for all `datasynth-data generate` workflows.

### Fingerprint Extraction Workflows

When using `datasynth-data fingerprint extract` with real data as input:

- **Real personal data may be processed** during statistical fingerprint extraction
- **GDPR obligations apply** to the extraction phase
- **Differential privacy** controls limit information retained in the fingerprint
- **The output fingerprint** (.dsf file) contains only aggregate statistics, not individual records

## Article 30 — Records of Processing Activities

### Template for Pure Synthetic Generation

| Field | Value |
|-------|-------|
| **Purpose** | Generation of synthetic financial data for testing, training, and validation |
| **Categories of data subjects** | None (no real data subjects) |
| **Categories of personal data** | None (all data is synthetic) |
| **Recipients** | Internal development, QA, and data science teams |
| **Transfers to third countries** | Not applicable (no personal data) |
| **Retention period** | Per project requirements |
| **Technical measures** | Seed-based deterministic generation, content marking |

### Template for Fingerprint Extraction

| Field | Value |
|-------|-------|
| **Purpose** | Statistical fingerprint extraction for privacy-preserving data synthesis |
| **Legal basis** | Legitimate interest (Article 6(1)(f)) or consent |
| **Categories of data subjects** | As per source dataset (e.g., customers, vendors, employees) |
| **Categories of personal data** | As per source dataset (aggregate statistics only retained) |
| **Recipients** | Data engineering team operating DataSynth |
| **Transfers to third countries** | Assess per deployment topology |
| **Retention period** | Fingerprint files: per project; source data: minimize retention |
| **Technical measures** | Differential privacy (configurable epsilon/delta), k-anonymity |

## Data Protection Impact Assessment (DPIA)

A DPIA under Article 35 is recommended when fingerprint extraction processes:
- Large-scale datasets (>100,000 records)
- Special categories of data (Article 9)
- Data relating to vulnerable persons

### DPIA Template for Fingerprint Extraction

**1. Description of Processing**

DataSynth extracts statistical fingerprints from source data. The fingerprint captures distribution parameters (means, variances, correlations) without retaining individual records. Differential privacy noise is added with configurable epsilon/delta parameters.

**2. Necessity and Proportionality**

- Purpose: Enable realistic synthetic data generation without accessing source data repeatedly
- Minimization: Only aggregate statistics are retained
- Privacy controls: Differential privacy with user-specified budget

**3. Risks to Data Subjects**

| Risk | Likelihood | Severity | Mitigation |
|------|-----------|----------|------------|
| Re-identification from fingerprint | Low | High | Differential privacy, k-anonymity enforcement |
| Membership inference | Low | Medium | MIA AUC-ROC testing in evaluation framework |
| Fingerprint file compromise | Medium | Low | Aggregate statistics only, no individual records |

**4. Measures to Address Risks**

- Configure `fingerprint_privacy.level: high` or `maximum` for sensitive data
- Set `fingerprint_privacy.epsilon` to 0.1-1.0 range (lower = stronger privacy)
- Enable k-anonymity with `fingerprint_privacy.k_anonymity >= 5`
- Use evaluation framework MIA testing to verify privacy guarantees

## Privacy Configuration

```yaml
fingerprint_privacy:
  level: high             # minimal, standard, high, maximum, custom
  epsilon: 0.5            # Privacy budget (lower = stronger)
  delta: 1.0e-5           # Failure probability
  k_anonymity: 10         # Minimum group size
  composition_method: renyi_dp  # naive, advanced, renyi_dp, zcdp
```

### Privacy Level Presets

| Level | Epsilon | Delta | k-Anonymity | Use Case |
|-------|---------|-------|-------------|----------|
| `minimal` | 10.0 | 1e-3 | 2 | Non-sensitive aggregates |
| `standard` | 1.0 | 1e-5 | 5 | General business data |
| `high` | 0.5 | 1e-6 | 10 | Sensitive financial data |
| `maximum` | 0.1 | 1e-8 | 20 | Regulated personal data |

## Data Subject Rights

### Pure Synthetic Mode

Articles 15-22 (access, rectification, erasure, etc.) do not apply as no real data subjects exist in synthetic output.

### Fingerprint Extraction Mode

- **Right of access (Art. 15)**: Fingerprints contain only aggregate statistics; individual records cannot be extracted
- **Right to erasure (Art. 17)**: Delete source data and fingerprint files; regenerate synthetic data with new parameters
- **Right to restriction (Art. 18)**: Suspend fingerprint extraction pipeline
- **Right to object (Art. 21)**: Remove individual from source dataset before extraction

## International Transfers

- **Synthetic output**: Generally not subject to Chapter V transfer restrictions (no personal data)
- **Fingerprint files**: Assess whether aggregate statistics constitute personal data in your jurisdiction
- **Source data**: Standard GDPR transfer rules apply during fingerprint extraction

## NIST SP 800-226 Alignment

DataSynth's evaluation framework includes NIST SP 800-226 alignment reporting for synthetic data privacy assessment. Enable via:

```yaml
privacy:
  nist_alignment_enabled: true
```

## See Also

- [EU AI Act Compliance](eu-ai-act.md)
- [NIST AI RMF Self-Assessment](nist-ai-rmf.md)
- [Fingerprinting Guide](../advanced/fingerprinting.md)
