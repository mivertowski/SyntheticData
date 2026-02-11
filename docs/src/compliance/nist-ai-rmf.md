# NIST AI Risk Management Framework Self-Assessment

This document provides a self-assessment of DataSynth against the NIST AI Risk Management Framework (AI 100-1, January 2023). The framework defines four core functions -- MAP, MEASURE, MANAGE, and GOVERN -- each with categories and subcategories. This assessment covers all four functions as they apply to a synthetic data generation tool.

## Assessment Scope

- **System**: DataSynth synthetic financial data generator
- **Version**: 0.5.x
- **Assessment Date**: 2025
- **Assessor**: Development team (self-assessment)
- **AI System Type**: Data generation tool (not a decision-making AI system)
- **Risk Classification**: The generated synthetic data may be used as training data for AI/ML systems. DataSynth itself does not make autonomous decisions, but the quality of its output can affect downstream AI system performance.

---

## MAP: Context and Framing

The MAP function establishes the context for AI risk management by identifying intended use cases, users, and known limitations.

### MAP 1: Intended Use Cases

DataSynth is designed for the following use cases:

| Use Case | Description | Risk Level |
|----------|-------------|------------|
| ML Training Data | Generate labeled datasets for fraud detection, anomaly detection, and audit analytics models | Medium |
| Software Testing | Provide realistic test data for ERP systems, accounting platforms, and audit tools | Low |
| Privacy-Preserving Analytics | Replace real financial data with synthetic equivalents that preserve statistical properties | Medium |
| Compliance Testing | Generate SOX control test evidence, COSO framework data, and SoD violation scenarios | Low |
| Process Mining | Create OCEL 2.0 event logs for process analysis without exposing real business processes | Low |
| Education and Research | Provide realistic financial datasets for academic research and training | Low |

**Not intended for**: Replacement of real financial records in regulatory filings, direct use as evidence in audit engagements, or any scenario where the synthetic nature of the data is concealed.

### MAP 2: Intended Users

| User Group | Typical Use | Access Level |
|------------|-------------|--------------|
| Data Scientists | Training ML models for fraud/anomaly detection | API or CLI |
| QA Engineers | ERP and accounting system load/integration testing | CLI or Python wrapper |
| Auditors | Testing audit analytics tools against known-labeled data | CLI output files |
| Compliance Teams | SOX control testing, COSO framework validation | CLI or server API |
| Researchers | Academic study of financial data patterns | Python wrapper |

### MAP 3: Known Limitations

DataSynth users should understand the following limitations:

1. **No Real PII**: Generated names, identifiers, and addresses are synthetic. They do not correspond to real individuals or organizations. This is a design feature, not a limitation, but downstream systems should not treat synthetic identities as real.

2. **Statistical Approximation**: Generated data follows configurable statistical distributions (log-normal, Benford's Law, Gaussian mixtures) that approximate real-world patterns. They are not derived from actual transaction populations unless fingerprint extraction is used.

3. **Industry Profile Approximations**: Pre-configured industry profiles (retail, manufacturing, financial services, healthcare, technology) are based on published research and general knowledge. They may not match specific organizations within an industry.

4. **Temporal Pattern Simplification**: Business day calendars, holiday schedules, and intraday patterns are modeled but may not capture all regional or organizational nuances.

5. **Anomaly Injection Boundaries**: Injected fraud patterns follow configurable typologies (ACFE taxonomy) but do not represent the full diversity of real-world fraud schemes.

6. **Fingerprint Extraction Privacy**: When extracting fingerprints from real data, differential privacy noise and k-anonymity are applied. The privacy guarantees depend on correct epsilon/delta parameter selection.

### MAP 4: Deployment Context

DataSynth can be deployed as:

- A CLI tool on developer workstations
- A server (REST/gRPC/WebSocket) in cloud or on-premises environments
- A Python library embedded in data pipelines
- A desktop application (Tauri/SvelteKit)

Each deployment context has different risk profiles. Server deployments require authentication, TLS, and rate limiting. CLI usage on trusted workstations has fewer access control requirements.

---

## MEASURE: Metrics and Evaluation

The MEASURE function establishes metrics, methods, and benchmarks for evaluating AI system trustworthiness.

### MEASURE 1: Quality Gate Metrics

DataSynth includes a comprehensive evaluation framework (`datasynth-eval`) with configurable quality gates. Each metric has defined thresholds and automated pass/fail checking.

#### Statistical Quality

| Metric | Gate Name | Threshold | Comparison | Purpose |
|--------|-----------|-----------|------------|---------|
| Benford's Law MAD | `benford_compliance` | 0.015 | LTE | First-digit distribution follows Benford's Law |
| Balance Coherence | `balance_sheet_valid` | 1.0 | GTE | Assets = Liabilities + Equity |
| Document Chain Integrity | `doc_chain_complete` | 0.95 | GTE | P2P/O2C chains are complete |
| Temporal Consistency | `temporal_valid` | 0.90 | GTE | Temporal patterns match configuration |
| Correlation Preservation | `correlation_check` | 0.80 | GTE | Cross-field correlations preserved |

#### Data Quality

| Metric | Gate Name | Threshold | Comparison | Purpose |
|--------|-----------|-----------|------------|---------|
| Completion Rate | `completeness` | 0.95 | GTE | Required fields are populated |
| Duplicate Rate | `uniqueness` | 0.05 | LTE | Acceptable duplicate rate |
| Referential Integrity | `ref_integrity` | 0.99 | GTE | Foreign key references valid |
| IC Match Rate | `ic_matching` | 0.95 | GTE | Intercompany transactions match |

#### Gate Profiles

Quality gates are organized into profiles with configurable strictness:

```yaml
evaluation:
  quality_gates:
    profile: strict    # strict, default, lenient
    fail_strategy: collect_all
    gates:
      - name: benford_compliance
        metric: benford_mad
        threshold: 0.015
        comparison: lte
      - name: balance_valid
        metric: balance_coherence
        threshold: 1.0
        comparison: gte
      - name: completeness
        metric: completion_rate
        threshold: 0.95
        comparison: gte
```

### MEASURE 2: Privacy Evaluation

DataSynth evaluates privacy risk through empirical attacks on generated data.

#### Membership Inference Attack (MIA)

The MIA module (`datasynth-eval/src/privacy/membership_inference.rs`) implements a distance-based classifier that attempts to determine whether a specific record was part of the generation configuration. Key metrics:

| Metric | Threshold | Interpretation |
|--------|-----------|----------------|
| AUC-ROC | <= 0.60 | Near-random classification indicates strong privacy |
| Accuracy | <= 0.55 | Low accuracy means synthetic data does not memorize patterns |
| Precision/Recall | Balanced | No systematic bias toward members or non-members |

#### Linkage Attack Assessment

The linkage module (`datasynth-eval/src/privacy/linkage.rs`) evaluates re-identification risk using quasi-identifier combinations:

| Metric | Threshold | Interpretation |
|--------|-----------|----------------|
| Re-identification Rate | <= 0.05 | Less than 5% of synthetic records can be linked to originals |
| K-Anonymity Achieved | >= 5 | Each quasi-identifier combination appears at least 5 times |
| Unique QI Overlap | Reported | Number of overlapping quasi-identifier combinations |

#### NIST SP 800-226 Alignment

The evaluation framework includes self-assessment against NIST SP 800-226 criteria for de-identification. The `NistAlignmentReport` evaluates:

- Data transformation adequacy
- Re-identification risk assessment
- Documentation completeness
- Privacy control effectiveness

Overall alignment score must meet >= 71% for a passing grade.

#### Fingerprint Module Privacy

When fingerprint extraction is used with real data input, the `datasynth-fingerprint` privacy engine provides:

| Mechanism | Parameter | Default (Standard Level) |
|-----------|-----------|--------------------------|
| Differential Privacy (Laplace) | Epsilon | 1.0 |
| K-Anonymity | K threshold | 5 |
| Outlier Protection | Winsorization percentile | 95th |
| Composition | Method | Naive (RDP/zCDP available) |

Privacy levels provide pre-configured parameter sets:

| Level | Epsilon | K | Use Case |
|-------|---------|---|----------|
| Minimal | 5.0 | 3 | Low sensitivity |
| Standard | 1.0 | 5 | Balanced (default) |
| High | 0.5 | 10 | Sensitive data |
| Maximum | 0.1 | 20 | Highly sensitive data |

### MEASURE 3: Completeness and Uniqueness

The evaluation module tracks data completeness and uniqueness metrics:

- **Completeness**: Measures the percentage of non-null values across all required fields. Reported as `overall_completeness` in the evaluation output.
- **Uniqueness**: Measures the duplicate rate across primary key fields. Collision-free UUIDs (FNV-1a hash-based with generator-type discriminators) ensure deterministic uniqueness.

### MEASURE 4: Distribution Validation

Statistical validation tests verify that generated data matches configured distributions:

| Test | Implementation | Purpose |
|------|----------------|---------|
| Benford First Digit | Chi-squared against Benford distribution | Transaction amounts follow expected first-digit distribution |
| Distribution Fit | Anderson-Darling test | Amount distributions match configured log-normal parameters |
| Correlation Check | Pearson/Spearman correlation | Cross-field correlations preserved via copula models |
| Temporal Patterns | Autocorrelation analysis | Seasonality and period-end patterns present |

---

## MANAGE: Risk Mitigation

The MANAGE function addresses risk response and mitigation strategies.

### MANAGE 1: Deterministic Reproducibility

DataSynth uses ChaCha8 CSPRNG with configurable seeds. Given the same configuration and seed, the output is identical across runs and platforms. This provides:

- **Auditability**: Any generated dataset can be exactly reproduced by preserving the configuration YAML and seed value.
- **Debugging**: Anomalous output can be reproduced for investigation.
- **Regression Testing**: Changes to generation logic can be detected by comparing output hashes.

```yaml
global:
  seed: 42                    # Deterministic seed
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12
```

### MANAGE 2: Audit Logging

DataSynth provides audit trails at multiple levels:

**Generation Audit**: The runtime emits structured JSON logs for every generation phase, including timing, record counts, and resource utilization.

**Privacy Audit**: The fingerprint module maintains a `PrivacyAudit` record of every privacy-related action (noise additions with epsilon spent, value suppressions, generalizations, winsorizations). This audit is embedded in the `.dsf` fingerprint file.

**Server Audit**: The REST/gRPC server logs authentication attempts, configuration changes, stream operations, and rate limit events with request correlation IDs (`X-Request-Id`).

**Run Manifest**: Each generation run produces a manifest documenting the configuration hash, seed, crate versions, start/end times, record counts, and quality gate results.

### MANAGE 3: Data Lineage Tracking

DataSynth tracks data lineage through:

- **Configuration Hashing**: SHA-256 hash of the input configuration is embedded in all output metadata.
- **Content Credentials**: Every output file includes a `ContentCredential` linking back to the generator version, configuration hash, and seed.
- **Document Reference Chains**: Generated document flows maintain explicit reference chains (PO -> GR -> Invoice -> Payment) with `DocumentReference` records.
- **Data Governance Reports**: Automated Article 10 governance reports document all processing steps from COA generation through quality validation.

### MANAGE 4: Content Marking

All synthetic output is marked to prevent confusion with real data:

- **CSV**: Comment headers with `# SYNTHETIC DATA - Generated by DataSynth v{version}`
- **JSON**: `_metadata.content_credential` object with generator, timestamp, config hash, and EU AI Act article reference
- **Parquet**: Custom metadata key-value pairs with full credential JSON
- **Sidecar Files**: Optional `.credential.json` files alongside output files

Content marking is enabled by default and can be configured:

```yaml
marking:
  enabled: true
  format: embedded    # embedded, sidecar, both
```

### MANAGE 5: Graceful Degradation

The resource guard system (`datasynth-core`) monitors memory, disk, and CPU usage, applying progressive degradation:

| Level | Memory Threshold | Response |
|-------|------------------|----------|
| Normal | < 70% | Full feature generation |
| Reduced | 70-85% | Disable optional features |
| Minimal | 85-95% | Core generation only |
| Emergency | > 95% | Graceful shutdown |

This prevents resource exhaustion from affecting other systems in shared environments.

---

## GOVERN: Policies and Oversight

The GOVERN function establishes organizational policies and structures for AI risk management.

### GOVERN 1: Access Control

DataSynth implements layered access control for the server deployment:

**API Key Authentication**: Keys are hashed with Argon2id at startup. Verification uses timing-safe comparison with a short-lived cache to prevent side-channel attacks. Keys are provided via the `X-API-Key` header or `Authorization: Bearer` header.

**JWT/OIDC Integration** (optional `jwt` feature): Supports external identity providers (Keycloak, Auth0, Entra ID) with RS256 token validation. JWT claims include subject, roles, and tenant ID for multi-tenancy.

**RBAC**: Role-based access control via JWT claims enables differentiated access:

| Role | Permissions |
|------|-------------|
| `operator` | Start/stop/pause generation streams |
| `admin` | Configuration changes, API key management |
| `viewer` | Read-only access to status and metrics |

**Exempt Paths**: Health (`/health`), readiness (`/ready`), liveness (`/live`), and metrics (`/metrics`) endpoints are exempt from authentication for infrastructure integration.

### GOVERN 2: Configuration Management

DataSynth configuration is managed through:

- **YAML Schema Validation**: All configuration is validated against a typed schema before generation begins. Invalid configurations produce descriptive error messages.
- **Industry Presets**: Pre-validated configuration presets for common industries (retail, manufacturing, financial services, healthcare, technology) reduce misconfiguration risk.
- **Complexity Levels**: Small (~100 accounts), medium (~400), and large (~2500) complexity levels provide validated scaling parameters.
- **Template System**: YAML/JSON templates with merge strategies enable configuration reuse while allowing overrides.

### GOVERN 3: Quality Gates as Governance Controls

Quality gates serve as automated governance controls:

```yaml
evaluation:
  quality_gates:
    profile: strict
    fail_strategy: fail_fast    # Stop on first failure
    gates:
      - name: benford_compliance
        metric: benford_mad
        threshold: 0.015
        comparison: lte
      - name: privacy_mia
        metric: privacy_mia_auc
        threshold: 0.60
        comparison: lte
      - name: balance_coherence
        metric: balance_coherence
        threshold: 1.0
        comparison: gte
```

Gate profiles can enforce:

- **Fail-fast**: Stop generation on first quality failure
- **Collect-all**: Run all checks and report all failures
- **Custom thresholds**: Organization-specific quality requirements

The `GateEngine` evaluates all configured gates against the `ComprehensiveEvaluation` and produces a `GateResult` with per-gate pass/fail status, actual values, and summary messages.

### GOVERN 4: Audit Trail Completeness

The following audit artifacts are produced for each generation run:

| Artifact | Location | Contents |
|----------|----------|----------|
| Run Manifest | `output/_manifest.json` | Config hash, seed, timestamps, record counts, gate results |
| Content Credentials | Embedded in each output file | Generator version, config hash, seed, EU AI Act reference |
| Data Governance Report | `output/_governance_report.json` | Article 10 data sources, processing steps, quality measures, bias assessment |
| Privacy Audit | Embedded in `.dsf` files | Epsilon spent, actions taken, composition method, remaining budget |
| Server Logs | Structured JSON to stdout/log aggregator | Request traces, auth events, config changes, stream operations |
| Quality Gate Results | `output/_evaluation.json` | Per-gate pass/fail, actual vs threshold, summary |

### GOVERN 5: Incident Response

For scenarios where generated data is mistakenly used as real data:

1. **Detection**: Content credentials in output files identify synthetic origin
2. **Containment**: Deterministic generation means the exact dataset can be reproduced and identified
3. **Remediation**: All output files carry machine-readable markers that downstream systems can check programmatically
4. **Prevention**: Content marking is enabled by default and requires explicit configuration to disable

---

## Assessment Summary

| Function | Category Count | Addressed | Notes |
|----------|---------------|-----------|-------|
| MAP | 4 | 4 | Use cases, users, limitations, and deployment documented |
| MEASURE | 4 | 4 | Quality gates, privacy metrics, completeness, distribution validation |
| MANAGE | 5 | 5 | Reproducibility, audit logging, lineage, content marking, degradation |
| GOVERN | 5 | 5 | Access control, config management, quality gates, audit trails, incident response |

**Overall Assessment**: DataSynth provides comprehensive risk management controls appropriate for a synthetic data generation tool. The primary residual risks relate to (1) parameter misconfiguration leading to unrealistic output, mitigated by quality gates and industry presets, and (2) privacy leakage during fingerprint extraction from real data, mitigated by differential privacy with configurable epsilon/delta budgets and empirical privacy evaluation.

## See Also

- [Quality Gate Configuration](../configuration/compliance.md)
- [Security Hardening](../deployment/security-hardening.md)
- [Fingerprinting](../advanced/fingerprinting.md)
- [GDPR Compliance](gdpr.md)
- [EU AI Act Compliance](eu-ai-act.md)
