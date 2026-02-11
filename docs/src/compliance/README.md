# Compliance & Regulatory Overview

DataSynth generates synthetic financial data for testing, training, and analytics. This section documents how DataSynth aligns with key regulatory frameworks and provides self-assessment artifacts for compliance teams.

## Regulatory Landscape

Synthetic data generation sits at the intersection of several regulatory domains. While pure synthetic data (generated without real-world data as input) generally faces fewer regulatory constraints than real data processing, organizations deploying DataSynth should understand the applicable frameworks.

## EU AI Act

The EU AI Act (Regulation 2024/1689) introduces obligations for AI systems and their training data. DataSynth addresses two key articles:

**Article 50 -- Transparency for Synthetic Content**: All DataSynth output includes machine-readable content credentials indicating that the data is synthetically generated. This is implemented through the `ContentCredential` system in `datasynth-core`, which embeds markers in CSV headers, JSON metadata, and Parquet file metadata. Content marking is enabled by default and can be configured via the `marking` section in the configuration YAML.

**Article 10 -- Data Governance**: DataSynth generates automated `DataGovernanceReport` documents that describe data sources (synthetic generation, no real data used), processing steps (COA generation through quality validation), quality measures applied (Benford's Law compliance, balance coherence, referential integrity), and bias assessments. These reports provide the documentation trail required under Article 10.

For full details, see [EU AI Act Compliance](eu-ai-act.md).

## NIST AI Risk Management Framework

The NIST AI RMF (AI 100-1) provides a voluntary framework for managing risks in AI systems. DataSynth has completed a self-assessment across all four core functions:

| Function | Focus Area | DataSynth Alignment |
|----------|------------|---------------------|
| **MAP** | Context and use cases | Documented intended uses, users, and known limitations |
| **MEASURE** | Metrics and evaluation | Quality gates, privacy metrics (MIA, linkage), statistical validation |
| **MANAGE** | Risk mitigation | Deterministic reproducibility, audit logging, content marking |
| **GOVERN** | Policies and oversight | Access control (API key + JWT/RBAC), configuration management, quality gate governance |

For the complete self-assessment, see [NIST AI RMF Self-Assessment](nist-ai-rmf.md).

## GDPR

The General Data Protection Regulation applies differently depending on the DataSynth workflow:

**Pure Synthetic Generation** (no real data input): GDPR obligations are minimal because no personal data is processed. The generated output contains no data subjects. Article 30 records should still document the processing activity for audit completeness.

**Fingerprint Extraction** (real data as input): When DataSynth's fingerprint module extracts statistical profiles from real datasets, GDPR applies in full. The fingerprint module includes differential privacy (Laplace mechanism with configurable epsilon/delta budgets), k-anonymity suppression of rare values, and a complete privacy audit trail. A Data Protection Impact Assessment (DPIA) template is provided for this scenario.

For templates and detailed guidance, see [GDPR Compliance](gdpr.md).

## SOC 2 Readiness

DataSynth's architecture supports SOC 2 Type II controls across the Trust Services Criteria:

| Criteria | DataSynth Controls |
|----------|--------------------|
| Security | API key authentication with Argon2id hashing, JWT/OIDC support, TLS termination, CORS lockdown |
| Availability | Graceful degradation under resource pressure, health/readiness endpoints |
| Processing Integrity | Deterministic RNG (ChaCha8), balanced journal entries enforced at construction, quality gates |
| Confidentiality | Content marking prevents synthetic data from being mistaken for real data |
| Privacy | Differential privacy in fingerprint extraction, no real PII in standard generation |

For deployment security controls, see [Security Hardening](../deployment/security-hardening.md).

## ISO 27001 Alignment

DataSynth supports ISO 27001:2022 Annex A controls relevant to data processing tools:

| Control | Implementation |
|---------|----------------|
| A.5.12 Classification of information | Content credentials classify all output as synthetic |
| A.8.10 Information deletion | Deterministic generation eliminates data retention concerns for pure synthetic workflows |
| A.8.11 Data masking | Fingerprint extraction applies differential privacy and k-anonymity |
| A.8.12 Data leakage prevention | Quality gates include privacy metrics (MIA AUC-ROC, linkage attack assessment) |
| A.8.25 Secure development lifecycle | Deterministic builds, dependency auditing (`cargo audit`), SBOM generation |

For access control configuration, see [Security Hardening](../deployment/security-hardening.md).

## Quick Reference

| Framework | Status | Documentation |
|-----------|--------|---------------|
| EU AI Act Article 50 | Implemented (content marking) | [EU AI Act](eu-ai-act.md) |
| EU AI Act Article 10 | Implemented (governance reports) | [EU AI Act](eu-ai-act.md) |
| NIST AI RMF | Self-assessment complete | [NIST AI RMF](nist-ai-rmf.md) |
| GDPR | Templates provided | [GDPR](gdpr.md) |
| SOC 2 | Readiness documented | [SOC 2 Readiness](soc2.md) |
| ISO 27001 | Annex A alignment documented | [ISO 27001 Alignment](iso27001.md) |

## See Also

- [Security Hardening](../deployment/security-hardening.md)
- [SOX Compliance Testing](../use-cases/sox-compliance.md)
- [Compliance Configuration](../configuration/compliance.md)
- [Fingerprinting](../advanced/fingerprinting.md)
