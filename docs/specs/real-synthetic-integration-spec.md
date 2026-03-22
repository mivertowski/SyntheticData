# Real–Synthetic Data Integration — Architecture & Design

> **Status:** Draft
> **Date:** 2026-03-22
> **Scope:** Comprehensive architecture for blending real client data with synthetic ground truth across all knowledge layers to create a unified knowledge system for audit analytics

---

## 1. Executive Summary

DataSynth generates reference knowledge graphs — fully provenanced synthetic datasets where every record traces to a known generative process. This document describes the architecture for **integrating real client data** with the synthetic ground truth to create a **comprehensive knowledge system** that surfaces facts for internal and external audits.

The core insight: synthetic data establishes the **expected baseline** (what normal looks like), real client data represents the **observed reality** (what actually happened), and the **gap between them** is where audit value lives — anomalies, errors, fraud, compliance violations, and process inefficiencies all manifest as deviations from the expected baseline.

### 1.1 The Four Integration Modes

| Mode | Input | Output | Privacy | Use Case |
|------|-------|--------|---------|----------|
| **Fingerprint-Calibrated** | `.dsf` fingerprint | Synthetic data matching real statistics | Full DP guarantees | Training data, benchmarking |
| **Direct Overlay** | Raw client data + synthetic baseline | Blended dataset with provenance tags | Client controls access | Audit support, gap analysis |
| **Gap Analysis** | Real data vs. synthetic expectation | Deviation reports with root causes | Aggregated metrics only | Risk assessment, audit planning |
| **Augmentation** | Sparse real data + synthetic fill | Complete dataset with coverage labels | Per-record provenance | Testing, development, migration |

### 1.2 Relationship to the Three-Layer Knowledge Model

The paper defines three knowledge layers: Structural (K_S), Statistical (K_Σ), and Normative (K_N). Real data integration operates across all three:

- **Structural layer**: Real chart of accounts, entity hierarchies, and document chains overlay the synthetic graph topology
- **Statistical layer**: Real amount distributions, temporal patterns, and correlations calibrate the synthetic baseline
- **Normative layer**: Real control effectiveness, compliance posture, and standards adherence are measured against the synthetic ideal

---

## 2. Problem Statement

### 2.1 The Inverse Problem (from the paper)

Proposition 3.1 proves that recovering ground truth from observed enterprise data is combinatorially infeasible — the configuration space exceeds 10^155,630 for a typical enterprise. Forward generation from a known model is the principled solution.

### 2.2 The Missing Piece

Forward generation alone produces a **self-consistent reference world**. But audit practitioners need to connect this reference world to **their client's actual data**. Without this connection, the synthetic ground truth remains an academic exercise rather than a practical audit tool.

### 2.3 What Integration Enables

When real data meets synthetic baseline:

1. **Anomaly detection with calibrated thresholds** — the synthetic baseline defines "normal," real data deviations are measured against it
2. **Completeness assessment** — synthetic data shows what a complete dataset looks like; gaps in real data become visible
3. **Control effectiveness testing** — synthetic controls have known effectiveness; real control outcomes are compared
4. **Process conformance checking** — synthetic document chains are complete by construction; real chain breaks are surfaced
5. **Statistical profile validation** — Benford compliance, amount distributions, and temporal patterns in real data are scored against the synthetic expectation
6. **Training data with real-world anchoring** — ML models trained on synthetic+real blends generalize better than either alone
7. **Audit evidence generation** — deviations between expected and observed constitute machine-generated audit evidence

---

## 3. Architecture Overview

### 3.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Integration Layer                            │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │  Schema       │  │  Record      │  │  Provenance  │              │
│  │  Harmonizer   │  │  Aligner     │  │  Tracker     │              │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
│         │                  │                  │                      │
│  ┌──────┴──────────────────┴──────────────────┴───────┐             │
│  │              Blending Engine                        │             │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌──────────────┐│             │
│  │  │Overlay │ │Calibr. │ │Gap     │ │Augmentation  ││             │
│  │  │Mode    │ │Mode    │ │Analysis│ │Mode          ││             │
│  │  └────────┘ └────────┘ └────────┘ └──────────────┘│             │
│  └────────────────────────┬───────────────────────────┘             │
│                           │                                         │
│  ┌────────────────────────┴───────────────────────────┐             │
│  │              Knowledge Comparator                   │             │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐         │             │
│  │  │Structural│  │Statistic.│  │Normative │         │             │
│  │  │Comparator│  │Comparator│  │Comparator│         │             │
│  │  └──────────┘  └──────────┘  └──────────┘         │             │
│  └────────────────────────┬───────────────────────────┘             │
│                           │                                         │
│  ┌────────────────────────┴───────────────────────────┐             │
│  │              Audit Evidence Generator               │             │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐         │             │
│  │  │Deviation │  │Finding   │  │Report    │         │             │
│  │  │Scorer    │  │Classifier│  │Generator │         │             │
│  │  └──────────┘  └──────────┘  └──────────┘         │             │
│  └────────────────────────────────────────────────────┘             │
│                                                                     │
├──────────────────────┬──────────────────────────────────────────────┤
│   Synthetic Ground   │          Real Client Data                    │
│   Truth (DataSynth)  │          (Direct or Fingerprint)             │
│                      │                                              │
│  ┌────────────────┐  │  ┌────────────────┐  ┌──────────────────┐   │
│  │ Generated      │  │  │ CSV / Parquet  │  │ ERP Extract      │   │
│  │ Reference KG   │  │  │ Files          │  │ (SAP, Oracle)    │   │
│  └────────────────┘  │  └────────────────┘  └──────────────────┘   │
│  ┌────────────────┐  │  ┌────────────────┐  ┌──────────────────┐   │
│  │ Ground Truth   │  │  │ Database       │  │ API Feed         │   │
│  │ Labels         │  │  │ Connection     │  │ (REST/gRPC)      │   │
│  └────────────────┘  │  └────────────────┘  └──────────────────┘   │
└──────────────────────┴──────────────────────────────────────────────┘
```

### 3.2 Data Flow

```
Real Client Data ──┐
                   ├──→ Schema Harmonizer ──→ Record Aligner ──→ Blending Engine
Synthetic Ground ──┘                                               │
Truth                                                              ├──→ Blended Dataset
                                                                   ├──→ Deviation Report
                                                                   ├──→ Audit Evidence
                                                                   └──→ Knowledge Delta
```

---

## 4. Mode 1: Fingerprint-Calibrated Generation (Existing)

This mode already exists in the codebase and serves as the privacy-preserving foundation.

### 4.1 Pipeline

```
Real Client Data
  → FingerprintExtractor.extract()          # Privacy: ε-DP + k-anonymity
  → .dsf file (ZIP: schema, stats, correlations, rules, anomalies)
  → ConfigSynthesizer.synthesize_full()     # Produces ConfigPatch + CopulaGeneratorSpec
  → EnhancedOrchestrator::from_fingerprint()
  → Synthetic data statistically matching real client
  → FidelityEvaluator.evaluate()            # KS, Wasserstein, JS divergence
  → AutoTuner.analyze()                     # Iterative refinement
```

### 4.2 What It Captures Per Knowledge Layer

| Layer | Captured | Not Captured |
|-------|----------|--------------|
| **Structural** | Table schemas, FK relationships, cardinalities | Actual entity identities, specific account codes |
| **Statistical** | Distributions (fitted params), correlations (copulas), Benford profile, temporal patterns | Individual record values |
| **Normative** | Anomaly rates, balance rules, approval thresholds | Specific control test results, compliance findings |

### 4.3 Privacy Guarantees

Four privacy levels control the extraction:

| Level | ε (epsilon) | k-anonymity | Use Case |
|-------|-------------|-------------|----------|
| Minimal | 5.0 | 3 | Internal analytics team |
| Standard | 1.0 | 5 | Cross-team sharing |
| High | 0.5 | 10 | External sharing |
| Maximum | 0.1 | 20 | Regulatory submission |

Utility bound (Proposition 8.1 from the paper): at n=10,000 records, ε=1.0, expected mean error is $100 on a $1M range — less than 0.01%, negligible for distribution fitting.

### 4.4 Federated Variant

`FederatedFingerprintProtocol` enables multi-site extraction without centralizing raw data. Each site contributes a `PartialFingerprint` with its own DP budget; aggregation uses weighted averaging by record count.

---

## 5. Mode 2: Direct Client Data Integration (New)

This mode operates on raw client data **without** the fingerprint privacy layer. It is designed for engagements where the audit team has authorized access to the client's actual records and needs to blend them directly with synthetic baselines.

### 5.1 When to Use Direct Integration

- **Internal audit teams** working with their own organization's data
- **External audit engagements** where data access agreements are in place
- **System migration testing** where real data must be validated against expected structures
- **Regulatory examinations** where the examiner has legal data access authority
- **Client-side deployments** where data never leaves the client's infrastructure

### 5.2 Direct Data Sources

```rust
/// Supported real data input formats for direct integration
pub enum DirectDataSource {
    /// File-based sources
    CsvDirectory { path: PathBuf, encoding: Encoding },
    ParquetDirectory { path: PathBuf },
    JsonDirectory { path: PathBuf, format: JsonFormat },

    /// Database connections
    Database {
        connection_string: String,       // postgres://, mysql://, mssql://
        schema: String,
        table_filter: Option<Vec<String>>,
    },

    /// ERP-specific extractors
    SapExtract {
        tables: Vec<SapTable>,           // BKPF/BSEG, EKKO/EKPO, VBAK/VBAP
        client: String,
        company_codes: Vec<String>,
    },

    /// In-memory (for API/SDK usage)
    Memory {
        tables: HashMap<String, Vec<HashMap<String, Value>>>,
    },

    /// Streaming source
    Stream {
        endpoint: String,                // Kafka, event hub, etc.
        format: StreamFormat,
    },
}
```

### 5.3 Schema Harmonization

Real client data rarely matches the synthetic schema exactly. The `SchemaHarmonizer` resolves this:

```
Client Schema                          DataSynth Schema
┌─────────────────┐                    ┌─────────────────┐
│ BKPF/BSEG       │ ──→ Mapping ──→  │ JournalEntry     │
│ BUKRS, BELNR,   │     Rules        │ company_code,    │
│ GJAHR, BUZEI... │                   │ document_number, │
│                  │                   │ fiscal_year...   │
└─────────────────┘                    └─────────────────┘
```

#### 5.3.1 Mapping Strategies

| Strategy | Description | Example |
|----------|-------------|---------|
| **Auto-detect** | Semantic type matching via column names and value patterns | `BUKRS` → `company_code`, `DMBTR` → `amount_local` |
| **Template** | Predefined mappings for known ERP systems | SAP FI, Oracle GL, NetSuite templates |
| **Manual** | User-provided YAML mapping file | Custom legacy systems |
| **Hybrid** | Auto-detect with manual overrides | Semi-standardized ERPs |

#### 5.3.2 Mapping Configuration

```yaml
integration:
  mode: direct_overlay
  source:
    type: csv_directory
    path: ./client_data/
    encoding: utf-8

  schema_mapping:
    strategy: template
    template: sap_fi
    overrides:
      - source_column: ZUONR
        target_field: assignment_number
      - source_column: SGTXT
        target_field: line_item_text

  type_coercion:
    date_formats: ["%Y%m%d", "%Y-%m-%d", "%d.%m.%Y"]
    amount_decimal_separator: "."
    amount_thousands_separator: ","
    currency_field: WAERS
```

### 5.4 Record Alignment

Once schemas are harmonized, records must be aligned between real and synthetic datasets for comparison.

#### 5.4.1 Alignment Keys

| Domain | Primary Key | Alignment Strategy |
|--------|-------------|-------------------|
| Journal Entries | company_code + fiscal_year + document_number | Exact match or temporal window |
| Chart of Accounts | account_number | Hierarchical fuzzy match |
| Vendors | vendor_id or tax_id | Deterministic + fuzzy (name, address) |
| Customers | customer_id or tax_id | Deterministic + fuzzy |
| Document Chains | document_reference | Chain-walk matching |
| Trial Balance | account_number + period | Exact match |
| Controls | control_id or control_description | Semantic similarity |

#### 5.4.2 Unmatched Record Handling

Records that exist in one dataset but not the other are classified:

- **Real-only records**: Entities or transactions in the client's data with no synthetic counterpart — these may indicate coverage gaps in the synthetic model or unusual client-specific processes
- **Synthetic-only records**: Expected structures missing from the client's data — these are potential completeness findings (missing controls, incomplete document chains, absent reconciliations)

### 5.5 Direct Overlay Blending

In overlay mode, real records replace their synthetic counterparts while preserving the synthetic ground truth as the reference layer:

```
Blended Record {
    data: RealOrSynthetic,              // The actual field values
    provenance: RecordProvenance {
        source: DataSource,              // Real | Synthetic | Blended
        synthetic_counterpart: Option<SyntheticRef>,  // Link to synthetic twin
        deviations: Vec<FieldDeviation>, // Per-field differences
        confidence: f64,                 // Alignment confidence
    },
    audit_flags: Vec<AuditFlag>,        // Auto-generated findings
}
```

### 5.6 Privacy Controls for Direct Mode

Even without DP fingerprinting, direct mode includes access controls:

```yaml
integration:
  direct_mode:
    access_control:
      # Who can see real data fields
      roles: [auditor, manager]
      # Fields that are masked even in direct mode
      always_mask: [bank_account, ssn, credit_card]
      # Pseudonymize personal identifiers
      pseudonymize: [employee_name, vendor_contact]
      # Audit log all access
      audit_logging: true
      # Data retention policy
      retention_days: 90
      # Geographic restrictions
      allowed_regions: [EU, US]
```

---

## 6. Mode 3: Gap Analysis

Gap analysis is the core audit value proposition. It compares real client data against the synthetic baseline across all three knowledge layers and produces actionable findings.

### 6.1 The Gap Analysis Pipeline

```
                    ┌──────────────────┐
                    │  Real Client     │
                    │  Data            │
                    └────────┬─────────┘
                             │
                    ┌────────▼─────────┐
                    │  Schema          │
                    │  Harmonizer      │
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
    ┌─────────▼────┐  ┌─────▼──────┐  ┌───▼──────────┐
    │  Structural  │  │ Statistical│  │  Normative   │
    │  Gap         │  │ Gap        │  │  Gap         │
    │  Analyzer    │  │ Analyzer   │  │  Analyzer    │
    └─────────┬────┘  └─────┬──────┘  └───┬──────────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
                    ┌────────▼─────────┐
                    │  Gap Report      │
                    │  Generator       │
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
    ┌─────────▼────┐  ┌─────▼──────┐  ┌───▼──────────┐
    │  Deviation   │  │  Finding   │  │  Audit       │
    │  Heatmap     │  │  Registry  │  │  Evidence    │
    └──────────────┘  └────────────┘  └──────────────┘
```

### 6.2 Structural Gap Analysis (K_S)

Compares the **topology** of real vs. synthetic knowledge graphs.

| Check | Synthetic Expectation | Real Finding | Audit Implication |
|-------|----------------------|--------------|-------------------|
| **CoA completeness** | All standard accounts present for industry | Missing accounts | Incomplete financial reporting |
| **Document chain integrity** | PO→GR→Invoice→Payment for every P2P cycle | Broken chains | Potential unauthorized purchases |
| **Entity relationships** | Every vendor has tax ID, bank details, approval | Missing master data fields | Incomplete KYC/vendor management |
| **Intercompany links** | Bilateral IC transactions net to zero | Unmatched IC items | Consolidation risk |
| **Control mapping** | Every significant account has mapped controls | Unmapped accounts | Control coverage gaps |
| **Referential integrity** | All FKs resolve | Orphaned references | Data quality issues |

Output: `StructuralGapReport` with per-entity and per-relationship coverage scores.

### 6.3 Statistical Gap Analysis (K_Σ)

Compares the **quantitative patterns** of real data against the synthetic baseline.

| Check | Method | Threshold | Finding Type |
|-------|--------|-----------|--------------|
| **Benford conformity** | MAD against first-digit law | MAD > 0.015 | Non-conforming amounts |
| **Amount distribution** | KS test vs. fitted log-normal | p < 0.05 | Distribution anomaly |
| **Temporal patterns** | Correlation with expected seasonality | r < 0.5 | Unusual timing patterns |
| **Period-end spikes** | Ratio of last-5-day to mid-month volume | > 4σ from expected | Potential period manipulation |
| **Correlation structure** | Frobenius norm of correlation matrix difference | > 0.3 | Changed dependency patterns |
| **Round number bias** | Proportion of round amounts vs. expected | > 2σ | Potential manual manipulation |
| **Weekend/holiday posting** | Count of non-business-day entries | > expected + 3σ | Process control weakness |
| **Duplicate patterns** | Exact and near-duplicate rates | > expected rate | Duplicate payment risk |

Output: `StatisticalGapReport` with per-metric scores, p-values, and effect sizes.

### 6.4 Normative Gap Analysis (K_N)

Compares **compliance posture** of real data against the standards-based synthetic ideal.

| Check | Standard | Synthetic Baseline | Real Measurement |
|-------|----------|-------------------|------------------|
| **SoD violations** | SOX / COSO | Expected violation rate (e.g., 1%) | Actual violation rate |
| **Approval thresholds** | Internal policy | Configured thresholds respected | Threshold breaches |
| **Revenue recognition** | ASC 606 / IFRS 15 | Performance obligations properly allocated | Allocation anomalies |
| **Lease accounting** | ASC 842 / IFRS 16 | ROU assets and lease liabilities computed | Missing lease entries |
| **Control effectiveness** | COSO 2013 | Known maturity levels per control | Actual test results |
| **Audit trail** | ISA 230 | Complete documentation chain | Trail gaps |
| **Three-way match** | Procurement policy | 98%+ match rate | Actual match rate |
| **Balance sheet equation** | GAAP fundamental | A = L + E always | Equation violations |

Output: `NormativeGapReport` with per-standard compliance scores and specific violation details.

### 6.5 Gap Severity Classification

Each gap is classified by audit severity:

```
Critical  ──→ Material misstatement risk, immediate escalation
             Example: Balance sheet equation violation, >5% of entries fail Benford
High      ──→ Significant deficiency, requires management response
             Example: >10% document chains broken, SoD violation rate >5%
Medium    ──→ Control deficiency, remediation recommended
             Example: Missing controls for 3+ significant accounts
Low       ──→ Observation, process improvement opportunity
             Example: Round number bias 1.5σ above expected
Info      ──→ No action needed, statistical note
             Example: Benford MAD = 0.008 (acceptable but not close conformity)
```

---

## 7. Mode 4: Augmentation

Augmentation fills gaps in sparse real data with synthetic records, creating a complete dataset for testing, development, or migration scenarios.

### 7.1 Use Cases

- **ERP migration testing**: Real data covers 6 months; augment to 24 months for full fiscal-year testing
- **New module rollout**: Real P2P data exists; augment with synthetic O2C, HR, manufacturing data
- **ML training**: Real data has 50 anomalies; augment with 5,000 synthetic anomalies across all types
- **Test environment seeding**: Production data is too sensitive; augment anonymized subset with synthetic fill

### 7.2 Augmentation Strategies

| Strategy | Description | Provenance Tag |
|----------|-------------|----------------|
| **Temporal extension** | Extend real data forward/backward in time using calibrated synthetic generation | `source: synthetic, basis: temporal_extension` |
| **Domain fill** | Add missing domains (e.g., add manufacturing data to a services company's real financials) | `source: synthetic, basis: domain_fill` |
| **Volume scaling** | Multiply real patterns at higher volume for stress testing | `source: synthetic, basis: volume_scaled` |
| **Anomaly enrichment** | Inject labeled anomalies into real data using the anomaly injection framework | `source: synthetic, basis: anomaly_injection` |
| **Coverage completion** | Fill in missing records to complete document chains, reconciliations, or control mappings | `source: synthetic, basis: coverage_fill` |

### 7.3 Augmentation Configuration

```yaml
integration:
  mode: augmentation
  real_data:
    source: ./client_extract/
    coverage:
      temporal: { start: "2025-01-01", end: "2025-06-30" }
      domains: [journal_entries, vendors, customers, purchase_orders]

  augmentation:
    temporal_extension:
      extend_to: "2025-12-31"
      calibration: fingerprint   # Use fingerprint of real data to calibrate extension

    domain_fill:
      add_domains:
        - manufacturing          # Full production order lifecycle
        - hr_payroll             # Payroll runs and time entries
        - treasury               # Cash positions and forecasts

    anomaly_enrichment:
      inject_rate: 0.05          # 5% anomaly rate in augmented data
      types: [all]               # All 33 anomaly types
      difficulty_range: [0.1, 0.9]

    coverage_completion:
      complete_document_chains: true
      add_missing_controls: true
      fill_reconciliations: true

  provenance:
    tag_all_records: true        # Every record tagged as real or synthetic
    export_provenance_map: true  # Separate file mapping record IDs to sources
```

---

## 8. Knowledge Comparator — Cross-Layer Analysis

The Knowledge Comparator operates across all three layers simultaneously, identifying patterns that single-layer analysis would miss.

### 8.1 Cross-Layer Correlation Matrix

| Finding Pattern | Layers Involved | Example |
|----------------|-----------------|---------|
| **Statistical anomaly + normative violation** | K_Σ + K_N | Benford non-conformity concentrated in accounts with weak controls |
| **Structural gap + statistical deviation** | K_S + K_Σ | Missing document chain links correlate with unusual amount distributions |
| **Full-stack anomaly** | K_S + K_Σ + K_N | New vendor (structural) with round-dollar transactions (statistical) bypassing approval (normative) |
| **Compensating pattern** | K_N + K_Σ | Control weakness offset by naturally conservative distribution patterns |

### 8.2 The Knowledge Delta

The `KnowledgeDelta` is the comprehensive output of cross-layer comparison:

```rust
pub struct KnowledgeDelta {
    /// Per-layer gap reports
    pub structural: StructuralGapReport,
    pub statistical: StatisticalGapReport,
    pub normative: NormativeGapReport,

    /// Cross-layer findings
    pub cross_layer_findings: Vec<CrossLayerFinding>,

    /// Overall risk score (0.0 = perfect match, 1.0 = maximum deviation)
    pub overall_risk_score: f64,

    /// Prioritized action items
    pub action_items: Vec<AuditActionItem>,

    /// Metadata
    pub generated_at: DateTime<Utc>,
    pub real_data_coverage: DataCoverage,
    pub synthetic_baseline_config: String,   // Config hash for reproducibility
}

pub struct CrossLayerFinding {
    pub layers: Vec<KnowledgeLayer>,         // Which layers are involved
    pub category: FindingCategory,           // Fraud indicator, control gap, process issue
    pub severity: Severity,                  // Critical, High, Medium, Low, Info
    pub description: String,
    pub affected_entities: Vec<EntityRef>,
    pub affected_records: Vec<RecordRef>,
    pub evidence: Vec<EvidenceItem>,
    pub suggested_procedure: String,         // What the auditor should do next
    pub confidence: f64,
}
```

### 8.3 Risk Scoring Model

The overall risk score combines layer-specific scores with cross-layer amplification:

```
risk_score = w_S * structural_gap_score
           + w_Σ * statistical_gap_score
           + w_N * normative_gap_score
           + w_X * cross_layer_amplification

where:
  w_S = 0.25  (structural weight)
  w_Σ = 0.30  (statistical weight — most granular signal)
  w_N = 0.30  (normative weight — direct compliance impact)
  w_X = 0.15  (cross-layer amplification)

  cross_layer_amplification = count(cross_layer_findings where severity >= High)
                            / max(1, total_finding_count) * severity_factor
```

---

## 9. Audit Evidence Generator

The Audit Evidence Generator transforms gap analysis findings into structured audit evidence that can be used directly in audit workpapers.

### 9.1 Evidence Types

| Evidence Type | Source | ISA Reference | Output Format |
|---------------|--------|---------------|---------------|
| **Analytical procedure results** | Statistical gap analysis | ISA 520 | Expectation vs. actual with threshold |
| **Test of details results** | Record-level comparison | ISA 500 | Sample with pass/fail per attribute |
| **Control test results** | Normative gap analysis | ISA 330 | Control effectiveness assessment |
| **Substantive procedure results** | Cross-layer findings | ISA 330 | Material misstatement risk assessment |
| **Completeness assertions** | Structural gap analysis | ISA 505 | Coverage map with gaps highlighted |
| **Going concern indicators** | Trend analysis across layers | ISA 570 | Financial health trajectory |

### 9.2 Evidence Chain

Every finding traces back through a complete evidence chain:

```
Audit Finding
  ← Cross-Layer Finding
    ← Gap Analysis Result (with p-value / effect size)
      ← Record Comparison (real vs. synthetic)
        ← Schema Alignment (field mapping)
          ← Raw Data Sources (real + synthetic)
            ← Generation Config (synthetic baseline specification)
              ← Seed + Version (exact reproducibility)
```

This chain satisfies ISA 230 (Audit Documentation) requirements: every conclusion is traceable to source evidence, and the synthetic baseline is fully reproducible from its configuration.

### 9.3 Integration with Existing Audit Module

The existing `datasynth-generators/src/audit/` module already generates:
- Engagement letters (ISA 210)
- Risk assessments (ISA 315)
- Materiality calculations (ISA 320)
- Sampling plans (ISA 530)
- Audit opinions (ISA 700/705/706/701)

The Audit Evidence Generator extends this by populating these structures with **real findings** from the gap analysis rather than synthetic placeholders:

```yaml
audit_integration:
  enabled: true
  populate_from_gap_analysis: true
  risk_assessment:
    use_real_risk_scores: true         # Replace synthetic risk levels with gap-derived ones
    materiality_from_real_financials: true
  sampling:
    stratify_by_deviation_score: true  # Focus samples on high-deviation areas
    sample_size_from_population: true  # Use real population sizes
  workpapers:
    auto_generate: true
    include_deviation_heatmaps: true
    include_statistical_exhibits: true
```

---

## 10. Audit Workflow Integration

### 10.1 End-to-End Audit Workflow

```
Phase 1: Planning                    Phase 2: Fieldwork
┌─────────────────────────┐          ┌─────────────────────────┐
│ 1. Obtain client data   │          │ 5. Direct overlay       │
│ 2. Extract fingerprint  │          │    (authorized data)    │
│ 3. Generate calibrated  │          │ 6. Record-level         │
│    synthetic baseline   │          │    gap analysis         │
│ 4. Run gap analysis     │──────▶   │ 7. Cross-layer          │
│    (risk assessment)    │          │    finding generation   │
│                         │          │ 8. Sample selection     │
│ Output: Audit plan with │          │    (deviation-weighted) │
│ risk-weighted scope     │          │                         │
└─────────────────────────┘          │ Output: Audit evidence  │
                                     │ with full provenance    │
                                     └───────────┬─────────────┘
                                                 │
Phase 3: Reporting                               │
┌─────────────────────────┐                      │
│ 9. Generate findings    │◀─────────────────────┘
│ 10. Map to ISA/PCAOB    │
│     requirements        │
│ 11. Classify severity   │
│ 12. Draft opinion       │
│     support             │
│                         │
│ Output: Audit report    │
│ with machine-generated  │
│ evidence                │
└─────────────────────────┘
```

### 10.2 Internal Audit Use Case

For internal audit teams with continuous data access:

```yaml
# Continuous monitoring configuration
integration:
  mode: continuous
  schedule: daily                    # Run gap analysis daily
  source:
    type: database
    connection: ${ERP_CONNECTION_STRING}
    incremental: true                # Only process new/changed records

  baseline:
    refresh: monthly                 # Re-calibrate synthetic baseline monthly
    fingerprint_privacy: minimal     # Internal use, relaxed privacy

  alerts:
    critical_findings: slack         # Immediate notification
    high_findings: email             # Daily digest
    trend_changes: dashboard         # Weekly trend report

  retention:
    findings: 365                    # Keep findings for 1 year
    evidence: 90                     # Keep detailed evidence for 90 days
    raw_comparisons: 30              # Keep record-level comparisons for 30 days
```

### 10.3 External Audit Use Case

For external audit engagements with time-bounded access:

```yaml
integration:
  mode: engagement
  engagement:
    client: "Client Corp"
    period: { start: "2025-01-01", end: "2025-12-31" }
    materiality: 50000.0

  source:
    type: csv_directory
    path: ./client_pbc/              # Prepared-by-client data
    schema_mapping: sap_fi

  baseline:
    industry: manufacturing
    complexity: medium
    calibration: fingerprint         # Extract fingerprint first
    privacy_level: high              # External engagement

  analysis:
    phases:
      - planning_analytics           # ISA 315 risk assessment
      - substantive_analytics        # ISA 520 analytical procedures
      - detail_testing               # ISA 500 sample-based testing
      - completion_analytics         # ISA 560 subsequent events

  output:
    workpapers: ./workpapers/
    format: [html, json, csv]
    include_exhibits: true
```

---

## 11. Implementation Architecture — Codebase Extension Points

### 11.1 New Crate: `datasynth-integration`

The integration layer lives in a new crate to maintain separation of concerns:

```
crates/datasynth-integration/
├── src/
│   ├── lib.rs                      # Public API
│   ├── sources/                    # Real data source adapters
│   │   ├── mod.rs
│   │   ├── csv_source.rs
│   │   ├── parquet_source.rs
│   │   ├── database_source.rs
│   │   ├── sap_source.rs
│   │   └── memory_source.rs
│   ├── harmonizer/                 # Schema harmonization
│   │   ├── mod.rs
│   │   ├── auto_detect.rs
│   │   ├── templates.rs           # ERP-specific mapping templates
│   │   └── coercion.rs            # Type coercion rules
│   ├── aligner/                   # Record alignment
│   │   ├── mod.rs
│   │   ├── key_matcher.rs
│   │   ├── fuzzy_matcher.rs
│   │   └── chain_walker.rs
│   ├── blending/                  # Blending engine
│   │   ├── mod.rs
│   │   ├── overlay.rs
│   │   ├── augmentation.rs
│   │   └── provenance.rs
│   ├── gap_analysis/              # Gap analyzers
│   │   ├── mod.rs
│   │   ├── structural.rs
│   │   ├── statistical.rs
│   │   ├── normative.rs
│   │   └── cross_layer.rs
│   ├── evidence/                  # Audit evidence generation
│   │   ├── mod.rs
│   │   ├── analytical.rs
│   │   ├── detail_testing.rs
│   │   ├── control_testing.rs
│   │   └── chain.rs
│   ├── report/                    # Report generation
│   │   ├── mod.rs
│   │   ├── html_report.rs
│   │   ├── json_report.rs
│   │   └── workpaper.rs
│   └── config.rs                  # Integration configuration
└── Cargo.toml
```

### 11.2 Existing Extension Points Used

| Extension Point | Location | Usage |
|----------------|----------|-------|
| `DataSource::Memory` | `datasynth-fingerprint` | Feed real data for fingerprint extraction |
| `TransformPlugin` | `datasynth-core/src/traits/` | Post-generation real data overlay |
| `PostProcessor` pipeline | `datasynth-core/src/traits/` | Labeled augmentation injection |
| `ComprehensiveEvaluation` | `datasynth-eval` | Gap analysis scoring |
| `AutoTuner` | `datasynth-eval` | Calibration feedback loop |
| `FidelityEvaluator` | `datasynth-fingerprint` | Real vs. synthetic fidelity scoring |
| `PhaseSink` | `datasynth-runtime` | Streaming integration results |
| `ConfigPatch` | `datasynth-fingerprint` / `datasynth-eval` | Dynamic config adjustment |

### 11.3 CLI Extension

```bash
# Fingerprint-calibrated generation (existing)
datasynth-data fingerprint extract --input ./client_data/ --output ./client.dsf
datasynth-data generate --fingerprint ./client.dsf --output ./synthetic/

# Direct gap analysis (new)
datasynth-data integrate gap-analysis \
  --real ./client_data/ \
  --baseline ./synthetic/ \
  --mapping sap_fi \
  --output ./gap_report/

# Direct overlay (new)
datasynth-data integrate overlay \
  --real ./client_data/ \
  --synthetic ./synthetic/ \
  --mapping sap_fi \
  --output ./blended/

# Augmentation (new)
datasynth-data integrate augment \
  --real ./client_data/ \
  --extend-to 2025-12-31 \
  --add-domains manufacturing,hr \
  --anomaly-rate 0.05 \
  --output ./augmented/

# Full audit workflow (new)
datasynth-data integrate audit \
  --real ./client_pbc/ \
  --industry manufacturing \
  --materiality 50000 \
  --output ./workpapers/
```

### 11.4 Server API Extension

```
POST /api/integration/fingerprint    # Upload data for fingerprint extraction
POST /api/integration/gap-analysis   # Run gap analysis
POST /api/integration/overlay        # Create blended dataset
POST /api/integration/augment        # Augment real data
GET  /api/integration/status/{id}    # Check async job status
GET  /api/integration/report/{id}    # Retrieve gap report
WS   /ws/integration/{id}           # Stream integration progress
```

### 11.5 Python SDK Extension

```python
from datasynth_py import DataSynth, Integration

ds = DataSynth()

# Direct gap analysis
report = ds.integration.gap_analysis(
    real_data="./client_data/",
    baseline="./synthetic/",
    mapping="sap_fi",
)
print(f"Overall risk score: {report.risk_score}")
for finding in report.critical_findings:
    print(f"  [{finding.severity}] {finding.description}")

# Direct overlay
blended = ds.integration.overlay(
    real_data="./client_data/",
    synthetic="./synthetic/",
    mapping="sap_fi",
)

# Augmentation
augmented = ds.integration.augment(
    real_data="./client_data/",
    extend_to="2025-12-31",
    add_domains=["manufacturing", "hr"],
    anomaly_rate=0.05,
)

# Full audit workflow
audit = ds.integration.audit_workflow(
    real_data="./client_pbc/",
    industry="manufacturing",
    materiality=50000,
    phases=["planning", "substantive", "detail", "completion"],
)
audit.export_workpapers("./workpapers/")
```

---

## 12. The Comprehensive Knowledge System

This section describes the unified knowledge system that emerges when all integration modes work together.

### 12.1 Conceptual Model

```
┌─────────────────────────────────────────────────────────────┐
│              Comprehensive Knowledge System                  │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Layer 3: Normative Knowledge (K_N)                  │   │
│  │                                                      │   │
│  │  Synthetic: Known control effectiveness, 100%        │   │
│  │  compliance, COSO maturity targets                   │   │
│  │                                     ╲                │   │
│  │  Real: Actual control test results,  ╲ GAP = Audit   │   │
│  │  compliance findings, maturity        ╲  Findings    │   │
│  │  assessments                           ╲             │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Layer 2: Statistical Knowledge (K_Σ)                │   │
│  │                                                      │   │
│  │  Synthetic: Benford-compliant distributions,         │   │
│  │  calibrated correlations, expected temporal patterns  │   │
│  │                                     ╲                │   │
│  │  Real: Actual amount distributions,  ╲ GAP = Risk    │   │
│  │  observed correlations, real timing   ╲  Indicators  │   │
│  │  patterns                              ╲             │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Layer 1: Structural Knowledge (K_S)                 │   │
│  │                                                      │   │
│  │  Synthetic: Complete graph topology, all entities,   │   │
│  │  full document chains, referential integrity         │   │
│  │                                     ╲                │   │
│  │  Real: Client's actual graph, entity ╲ GAP = Data    │   │
│  │  relationships, document chains,      ╲  Quality     │   │
│  │  observed integrity                    ╲  Issues     │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ═══════════════════════════════════════════════════════════ │
│  SYNTHESIS: Cross-layer gaps amplify each other.             │
│  A structural gap (missing control) combined with a          │
│  statistical gap (unusual amounts) in a normatively          │
│  sensitive area (revenue recognition) produces a             │
│  high-confidence compound finding.                           │
│  ═══════════════════════════════════════════════════════════ │
└─────────────────────────────────────────────────────────────┘
```

### 12.2 Knowledge System Properties

The comprehensive knowledge system has properties that neither synthetic nor real data alone can provide:

| Property | Synthetic Alone | Real Alone | Integrated System |
|----------|----------------|------------|-------------------|
| **Ground truth** | Complete (by construction) | Unknown (inverse problem) | Synthetic provides reference; deviations are measured |
| **Completeness** | Guaranteed (all domains generated) | Depends on client systems | Gaps visible by comparison |
| **Consistency** | Guaranteed (balanced, reconciled) | Unknown | Inconsistencies surfaced as findings |
| **Provenance** | Full (every record traced to generator) | Partial (audit trail may be incomplete) | Every record tagged with source and confidence |
| **Anomaly labels** | Known (injected with ground truth) | Unknown (that's what we're looking for) | Synthetic labels calibrate detectors; real deviations scored |
| **Reproducibility** | Deterministic (seed + config) | One-shot (historical reality) | Baseline reproducible; comparison repeatable |
| **Privacy** | No real data exposure | Full exposure | Configurable per mode |

### 12.3 The Feedback Loop

The knowledge system is not static — it improves iteratively:

```
┌─────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ Generate │────▶│ Compare  │────▶│ Evaluate │────▶│ Tune     │
│ Baseline │     │ with Real│     │ Gaps     │     │ Baseline │
└─────────┘     └──────────┘     └──────────┘     └────┬─────┘
     ▲                                                  │
     └──────────────────────────────────────────────────┘
                    AutoTuner ConfigPatch
```

Each iteration brings the synthetic baseline closer to the client's reality while maintaining provenance and ground truth properties. The `AutoTuner` maps evaluation gaps to config patches, the `RecommendationEngine` provides root-cause analysis, and the `FidelityEvaluator` scores convergence.

---

## 13. Practical Scenarios

### 13.1 Scenario: External Audit of a Manufacturing Company

**Phase 1 — Planning (fingerprint mode)**
1. Client provides GL extract (BKPF/BSEG) as CSV files
2. DataSynth extracts fingerprint at `privacy_level: high` (ε=0.5)
3. Fingerprint reveals: 385K journal entries, log-normal amounts (μ=7.2, σ=1.3), Benford MAD=0.009, 62% of entries in Q4
4. Synthetic baseline generated from manufacturing-medium preset, calibrated by fingerprint
5. Preliminary gap analysis identifies: unusual Q4 concentration (1.8σ above expected period-end spike), Benford deviation in 5XXX accounts (revenue), 3 vendor clusters with missing tax IDs

**Phase 2 — Fieldwork (direct overlay mode)**
1. Full client data loaded via SAP FI template mapping
2. Record-level overlay identifies 47 journal entries with amounts just below the €10,000 approval threshold
3. Cross-layer analysis reveals: 12 of these 47 entries posted by the same user, to vendor accounts created in the last 90 days, with round-euro amounts — triggering a **full-stack compound finding** (structural + statistical + normative)
4. Deviation-weighted sampling selects 150 entries for detail testing, concentrated in high-risk areas

**Phase 3 — Reporting**
1. Audit evidence auto-generated with full provenance chains
2. Findings mapped to ISA 240 (fraud risk), ISA 315 (risk assessment), ISA 330 (response to risk)
3. Gap report exported as HTML workpapers with embedded statistical exhibits

### 13.2 Scenario: Internal Continuous Monitoring

**Setup**
1. Internal audit configures daily gap analysis against ERP database
2. Synthetic baseline calibrated monthly via fingerprint extraction
3. Alert thresholds set: Benford MAD > 0.012, duplicate rate > 0.5%, SoD violations > 2%

**Daily Operation**
1. Incremental extraction pulls new/changed records since last run
2. Statistical gap analyzer scores new entries against baseline distributions
3. Structural analyzer checks document chain completion for new P2P/O2C cycles
4. Normative analyzer validates approval workflows and SoD compliance

**Monthly Cycle**
1. Full re-extraction and baseline recalibration
2. Trend analysis: are gaps growing or shrinking?
3. AutoTuner adjusts baseline to reflect genuine business changes (new product lines, seasonal shifts) vs. anomalies

### 13.3 Scenario: ML Model Training for Fraud Detection

**Setup**
1. Client provides 2 years of GL data (no fraud labels)
2. Fingerprint extracted at `privacy_level: standard`
3. Synthetic baseline generated with 5% anomaly injection across all 33 types
4. Augmentation adds 3 multi-stage fraud schemes (vendor kickback, embezzlement, revenue manipulation)

**Training Pipeline**
1. Blended dataset: 95% fingerprint-calibrated synthetic (with labels) + 5% augmented anomalies
2. Graph export to PyTorch Geometric for GNN training
3. Model trained on labeled synthetic data
4. Model validated on held-out synthetic data (known ground truth)
5. Model applied to real client data — findings are scored deviations from the synthetic baseline, not predictions in a vacuum

**Value**: The model's predictions are anchored to the synthetic ground truth. A flagged transaction is not just "unusual" — it deviates from a specific, reproducible expectation in a specific knowledge layer.

---

## 14. Future Extensions

### 14.1 Real-Time Streaming Integration

```yaml
integration:
  mode: streaming
  source:
    type: kafka
    topic: erp.journal_entries
    group_id: datasynth-integration

  baseline:
    type: in_memory
    refresh_trigger: record_count    # Re-calibrate every 10,000 records
    refresh_threshold: 10000

  output:
    alerts: websocket
    findings: elasticsearch
    dashboard: grafana
```

### 14.2 Multi-Client Benchmarking

Using federated fingerprints, compare a client's statistical profile against anonymized industry benchmarks:

```
Client A fingerprint ──┐
Client B fingerprint ──┼──→ Industry Benchmark ──→ Percentile Report
Client C fingerprint ──┘
                              "Your Benford MAD is in the 85th percentile
                               for manufacturing companies — above average
                               deviation warrants investigation"
```

### 14.3 LLM-Enhanced Finding Narration

Integrate with the existing `datasynth-core/src/llm/` module to generate natural-language audit finding narratives from structured gap analysis results:

```
Finding: 47 journal entries with amounts in range [€9,500–€9,999],
         posted by user U042, to vendors V-2024-*, round-euro amounts.

Narrative: "During our analytical procedures over the journal entry population
           for the period ended 31 December 2025, we identified a cluster of
           47 transactions with amounts consistently just below the €10,000
           approval threshold. These transactions share a common posting user
           (employee ID U042) and were directed to vendor accounts created
           within the audit period. The round-euro denomination pattern and
           threshold proximity suggest potential deliberate structuring to
           circumvent approval controls, warranting further investigation
           under ISA 240 paragraph 32(b)."
```

### 14.4 Regulatory Sandbox

Generate compliance-tested datasets for regulatory stress testing:

```
Real Data ──→ Fingerprint ──→ Baseline
                                  │
              ┌───────────────────┼───────────────────┐
              ▼                   ▼                   ▼
         Scenario A          Scenario B          Scenario C
         "Recession"         "Fraud Spike"       "Control Failure"
         (counterfactual)    (anomaly inject)    (normative override)
              │                   │                   │
              └───────────────────┼───────────────────┘
                                  ▼
                          Regulatory Report
                          "How would our controls
                           perform under stress?"
```

This leverages the existing counterfactual simulation engine (`CausalDAG`, `InterventionEngine`, `DiffEngine`) with real-data-calibrated baselines.

---

## 15. Summary

The real–synthetic integration architecture transforms DataSynth from a synthetic data generator into a **comprehensive knowledge system** for enterprise audit analytics. By establishing the synthetic ground truth as an expected baseline and measuring real client data against it across all three knowledge layers, every deviation becomes a measurable, traceable, reproducible audit finding.

| Capability | Before Integration | After Integration |
|-----------|-------------------|-------------------|
| Ground truth | Synthetic only | Synthetic baseline + real deviations measured |
| Anomaly detection | Labeled synthetic anomalies | Calibrated detection against client reality |
| Audit evidence | Synthetic examples | Machine-generated evidence from real gaps |
| Risk assessment | Generic industry profiles | Client-specific, data-driven risk scores |
| Compliance testing | Synthetic control tests | Real controls scored against normative ideal |
| ML training | Synthetic-only training data | Real-anchored training with synthetic labels |
| Continuous monitoring | Not supported | Daily gap analysis with trend tracking |

The four integration modes (fingerprint-calibrated, direct overlay, gap analysis, augmentation) provide a spectrum from full privacy preservation to full data access, enabling deployment across internal audit, external audit, regulatory examination, and ML development contexts.

---

*This specification extends the three-layer knowledge model described in the paper "DataSynth: Reference Knowledge Graphs for Enterprise Audit Analytics through Synthetic Data Generation with Provable Statistical Properties" by adding the critical real-data integration layer that connects the synthetic reference world to observed enterprise reality.*
