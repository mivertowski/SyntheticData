# EU AI Act Compliance

DataSynth implements technical controls aligned with the EU Artificial Intelligence Act (Regulation 2024/1689), focusing on Article 50 (transparency for synthetic content) and Article 10 (data governance for high-risk AI systems).

## Article 50 — Synthetic Content Marking

Article 50(2) requires that providers of AI systems generating synthetic content shall ensure outputs are marked in a machine-readable format and detectable as artificially generated.

### How DataSynth Complies

DataSynth embeds machine-readable synthetic content credentials in all output files:

- **CSV**: Comment header lines with C2PA-inspired metadata
- **JSON**: `_synthetic_metadata` top-level object with credential fields
- **Parquet**: Key-value metadata pairs in the file footer

### Configuration

```yaml
compliance:
  content_marking:
    enabled: true          # Default: true
    format: embedded       # embedded, sidecar, or both
  article10_report: true   # Generate Article 10 governance report
```

### Marking Formats

| Format | Description |
|--------|-------------|
| `embedded` | Credentials embedded directly in output files (default) |
| `sidecar` | Separate `.synthetic-credential.json` file alongside each output |
| `both` | Both embedded and sidecar credentials |

### Credential Fields

Each synthetic content credential contains:

| Field | Description | Example |
|-------|-------------|---------|
| `generator` | Tool identifier | `"DataSynth"` |
| `version` | Generator version | `"0.5.0"` |
| `timestamp` | ISO 8601 generation time | `"2024-06-15T10:30:00Z"` |
| `content_type` | Output category | `"synthetic_financial_data"` |
| `method` | Generation technique | `"rule_based_statistical"` |
| `config_hash` | SHA-256 of config used | `"a1b2c3..."` |
| `declaration` | Human-readable statement | `"This content is synthetic..."` |

### Programmatic Detection

Third-party systems can detect synthetic DataSynth output by:

1. **CSV**: Checking for `# X-Synthetic-Generator: DataSynth` header lines
2. **JSON**: Checking for `_synthetic_metadata.generator == "DataSynth"`
3. **Parquet**: Reading `synthetic_generator` from file metadata

## Article 10 — Data Governance

Article 10 requires appropriate data governance practices for training datasets used by high-risk AI systems. When synthetic data from DataSynth is used to train such systems, the Article 10 data governance report provides documentation.

### Governance Report Contents

The automated report includes:

- **Data Sources**: Documentation of all inputs (configuration parameters, seed values, statistical distributions)
- **Processing Steps**: Complete pipeline documentation (CoA generation, master data, document flows, anomaly injection, quality validation)
- **Quality Measures**: Statistical validation results (Benford's Law, balance coherence, distribution fitting)
- **Bias Assessment**: Known limitations, demographic representation gaps, and mitigation measures

### Generating the Report

Enable in configuration:

```yaml
compliance:
  article10_report: true
```

The report is written as `article10_governance_report.json` in the output directory.

### Report Structure

```json
{
  "report_version": "1.0",
  "generator": "DataSynth",
  "generated_at": "2024-06-15T10:30:00Z",
  "data_sources": ["configuration_parameters", "statistical_distributions", "deterministic_rng"],
  "processing_steps": [
    "chart_of_accounts_generation",
    "master_data_generation",
    "document_flow_generation",
    "journal_entry_generation",
    "anomaly_injection",
    "quality_validation"
  ],
  "quality_measures": [
    "benfords_law_compliance",
    "balance_sheet_coherence",
    "document_chain_integrity",
    "referential_integrity"
  ],
  "bias_assessment": {
    "known_limitations": [
      "Statistical distributions are parameterized, not learned from real data",
      "Temporal patterns use simplified seasonal models"
    ],
    "mitigation_measures": [
      "Configurable distribution parameters per industry profile",
      "Quality gate validation ensures statistical plausibility"
    ]
  }
}
```

## See Also

- [NIST AI RMF Self-Assessment](nist-ai-rmf.md)
- [GDPR Compliance](gdpr.md)
- [Quality Gates](../advanced/performance.md)
