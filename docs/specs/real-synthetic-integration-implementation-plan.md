# Real–Synthetic Data Integration — Implementation Plan

> **Status:** Draft
> **Date:** 2026-03-22
> **Spec Reference:** `docs/specs/real-synthetic-integration-spec.md`
> **Target Crate:** `datasynth-integration` (new) + extensions to existing crates

---

## Overview

This plan implements the architecture described in the integration spec across **8 phases**, each independently testable and shippable. Phases are ordered by dependency — later phases build on earlier ones, but each phase produces a usable deliverable.

### Phase Summary

| Phase | Name | Deliverable | Est. Files | Dependencies |
|-------|------|-------------|------------|--------------|
| **0** | Foundation | Crate scaffold, core types, config schema | ~15 | None |
| **1** | Data Sources & Schema Harmonization | CSV/JSON/Parquet ingest + ERP mapping templates | ~20 | Phase 0 |
| **2** | Record Alignment & Provenance | Key matching, fuzzy alignment, provenance tracking | ~12 | Phase 1 |
| **3** | Gap Analysis Engine | Structural, statistical, normative analyzers + cross-layer | ~18 | Phase 2 |
| **4** | Blending Engine | Overlay mode + augmentation mode | ~10 | Phase 2 |
| **5** | Audit Evidence & Reporting | Evidence generator, workpaper export, HTML/JSON reports | ~12 | Phase 3 |
| **6** | CLI, Server, Python SDK | `integrate` subcommand, REST endpoints, Python bindings | ~15 | Phase 3, 4, 5 |
| **7** | End-to-End Workflows & Quality Gates | Audit workflow orchestration, continuous monitoring, evaluation | ~10 | Phase 6 |

### Guiding Principles

1. **Each phase compiles and passes `cargo test` independently** — no broken intermediate states
2. **Lean on existing infrastructure** — reuse `datasynth-eval` for statistical tests, `datasynth-fingerprint` for privacy, `datasynth-core` for models/traits
3. **Feature-flag optional dependencies** — database connectors, Parquet, Kafka behind Cargo features
4. **No over-abstraction** — build concrete implementations first, extract traits when a second implementation appears
5. **Test-driven** — each module has unit tests; each phase ends with integration tests

---

## Phase 0: Foundation — Crate Scaffold & Core Types

**Goal:** Create `datasynth-integration` crate with core domain types, error handling, and configuration schema. Everything compiles, nothing does real work yet.

### 0.1 Create Crate

**File:** `crates/datasynth-integration/Cargo.toml`

```toml
[package]
name = "datasynth-integration"
version = "0.1.0"
edition = "2021"

[dependencies]
datasynth-core = { path = "../datasynth-core" }
datasynth-config = { path = "../datasynth-config" }
datasynth-eval = { path = "../datasynth-eval" }
datasynth-fingerprint = { path = "../datasynth-fingerprint" }

serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
chrono = { workspace = true, features = ["serde"] }
rust_decimal = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

# Optional: heavy dependencies behind features
csv = { workspace = true }
arrow = { workspace = true, optional = true }
parquet = { workspace = true, optional = true }

[features]
default = []
parquet = ["dep:parquet", "dep:arrow"]
database = []       # Future: sqlx
streaming = []      # Future: rdkafka

[dev-dependencies]
datasynth-test-utils = { path = "../datasynth-test-utils" }
tempfile = { workspace = true }
```

**Action:** Add `"crates/datasynth-integration"` to workspace members in root `Cargo.toml`.

### 0.2 Core Types

**File:** `crates/datasynth-integration/src/types.rs`

Define the fundamental domain types that all modules share:

```rust
// DataSource provenance tag
pub enum DataSource {
    Real { source_id: String, timestamp: DateTime<Utc> },
    Synthetic { config_hash: String, seed: u64 },
    Blended { real_ref: String, synthetic_ref: String },
}

// Knowledge layers (from the paper)
pub enum KnowledgeLayer {
    Structural,
    Statistical,
    Normative,
}

// Finding severity (5-level, spec §6.5)
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

// Finding category
pub enum FindingCategory {
    FraudIndicator,
    ControlGap,
    DataQualityIssue,
    ComplianceViolation,
    ProcessAnomaly,
    StatisticalDeviation,
}

// Record provenance (spec §5.5)
pub struct RecordProvenance {
    pub source: DataSource,
    pub synthetic_counterpart: Option<String>,  // UUID of synthetic twin
    pub deviations: Vec<FieldDeviation>,
    pub confidence: f64,
}

pub struct FieldDeviation {
    pub field_name: String,
    pub expected: String,     // Synthetic value (as string for generality)
    pub observed: String,     // Real value
    pub deviation_score: f64, // Normalized 0.0–1.0
}

// Alignment result
pub enum AlignmentResult {
    Matched { real_id: String, synthetic_id: String, confidence: f64 },
    RealOnly { real_id: String, classification: UnmatchedClassification },
    SyntheticOnly { synthetic_id: String, classification: UnmatchedClassification },
}

pub enum UnmatchedClassification {
    CoverageGap,          // Synthetic model doesn't cover this entity type
    MissingInReal,        // Expected entity absent from real data
    ClientSpecific,       // Client-specific entity with no synthetic analog
}
```

### 0.3 Error Types

**File:** `crates/datasynth-integration/src/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Schema harmonization failed: {0}")]
    SchemaError(String),
    #[error("Source read error: {0}")]
    SourceError(String),
    #[error("Alignment failed: {0}")]
    AlignmentError(String),
    #[error("Gap analysis error: {0}")]
    GapAnalysisError(String),
    #[error("Report generation error: {0}")]
    ReportError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type IntegrationResult<T> = Result<T, IntegrationError>;
```

### 0.4 Configuration Schema

**File:** `crates/datasynth-integration/src/config.rs`

Deserializable YAML config matching spec §5.3.2, §7.3, §10.2, §10.3:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub mode: IntegrationMode,
    pub source: SourceConfig,
    pub schema_mapping: SchemaMappingConfig,
    pub type_coercion: TypeCoercionConfig,
    pub baseline: Option<BaselineConfig>,
    pub augmentation: Option<AugmentationConfig>,
    pub gap_analysis: Option<GapAnalysisConfig>,
    pub access_control: Option<AccessControlConfig>,
    pub provenance: ProvenanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationMode {
    FingerprintCalibrated,
    DirectOverlay,
    GapAnalysis,
    Augmentation,
    Continuous,
    Engagement,
}

// ... remaining config structs for each section
```

### 0.5 Module Skeleton

**File:** `crates/datasynth-integration/src/lib.rs`

```rust
pub mod types;
pub mod error;
pub mod config;
pub mod sources;      // Phase 1
pub mod harmonizer;   // Phase 1
pub mod aligner;      // Phase 2
pub mod blending;     // Phase 4
pub mod gap_analysis; // Phase 3
pub mod evidence;     // Phase 5
pub mod report;       // Phase 5
```

Each submodule starts as an empty `mod.rs` with a doc comment describing its purpose. Modules are conditionally compiled only when their contents exist (use `#[cfg]` or just empty mods).

### 0.6 Tasks

| # | Task | Files | Test |
|---|------|-------|------|
| 0.1 | Create crate directory and `Cargo.toml` | `Cargo.toml`, root `Cargo.toml` | `cargo check -p datasynth-integration` |
| 0.2 | Define core types in `types.rs` | `src/types.rs` | Unit tests for `Display`, `Serialize`/`Deserialize` |
| 0.3 | Define error types | `src/error.rs` | Compiles, error conversions work |
| 0.4 | Define config schema | `src/config.rs` | Round-trip YAML serialization test |
| 0.5 | Create module skeleton (`lib.rs` + empty submodules) | `src/lib.rs`, `src/sources/mod.rs`, etc. | `cargo test -p datasynth-integration` passes |
| 0.6 | Add integration config section to `datasynth-config` | `datasynth-config/src/schema.rs` | Existing config tests still pass |

**Exit Criteria:** `cargo build -p datasynth-integration` succeeds. `cargo test -p datasynth-integration` passes. All existing tests unaffected.

---

## Phase 1: Data Sources & Schema Harmonization

**Goal:** Read real client data from files (CSV, JSON, Parquet) and transform it into DataSynth's internal model representation through schema mapping.

**Depends on:** Phase 0

### 1.1 Data Source Trait

**File:** `crates/datasynth-integration/src/sources/mod.rs`

Define a unified trait for reading real data regardless of source format:

```rust
/// A row of real client data — column name → value
pub type RawRecord = HashMap<String, serde_json::Value>;

/// Metadata about a data source table
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: Option<u64>,
}

pub struct ColumnInfo {
    pub name: String,
    pub inferred_type: InferredType,  // String, Integer, Decimal, Date, Boolean, Null
    pub null_count: u64,
    pub sample_values: Vec<String>,   // First 5 non-null values
}

/// Trait for reading real data from any source
pub trait DataSourceReader: Send {
    /// List available tables/files in this source
    fn list_tables(&self) -> IntegrationResult<Vec<TableInfo>>;

    /// Read all records from a table (small datasets)
    fn read_table(&self, table_name: &str) -> IntegrationResult<Vec<RawRecord>>;

    /// Stream records from a table (large datasets)
    fn stream_table(&self, table_name: &str) -> IntegrationResult<Box<dyn Iterator<Item = IntegrationResult<RawRecord>> + '_>>;

    /// Read table schema without loading data
    fn inspect_table(&self, table_name: &str) -> IntegrationResult<TableInfo>;
}
```

### 1.2 CSV Source

**File:** `crates/datasynth-integration/src/sources/csv_source.rs`

```rust
pub struct CsvDirectorySource {
    path: PathBuf,
    encoding: Encoding,
    delimiter: u8,
    has_headers: bool,
}
```

Implementation details:
- Enumerate `*.csv` files in directory; each file = one table (filename minus extension = table name)
- Use `csv::Reader` with configurable delimiter and encoding
- `inspect_table`: read first 1000 rows to infer column types via pattern matching (date patterns, numeric detection, etc.)
- `stream_table`: return lazy iterator over `csv::Reader` records
- Handle BOM markers, mixed line endings, and common CSV quirks

Tests:
- Read a sample SAP BKPF export (fixture file in `tests/fixtures/`)
- Verify column type inference on mixed-type columns
- Test empty files, single-row files, files with only headers

### 1.3 JSON Source

**File:** `crates/datasynth-integration/src/sources/json_source.rs`

```rust
pub struct JsonDirectorySource {
    path: PathBuf,
    format: JsonFormat,  // JsonLines | JsonArray | NestedObject { record_path: String }
}
```

- Support JSON Lines (one JSON object per line) — common for DataSynth's own output
- Support JSON array (top-level `[{...}, {...}]`)
- Support nested object with configurable record path (e.g., `"data.items"`)

### 1.4 Parquet Source (Feature-Gated)

**File:** `crates/datasynth-integration/src/sources/parquet_source.rs`

```rust
#[cfg(feature = "parquet")]
pub struct ParquetDirectorySource {
    path: PathBuf,
}
```

- Behind `parquet` feature flag to avoid mandatory Arrow dependency
- Use `parquet::arrow::arrow_reader::ParquetRecordBatchReader`
- Convert Arrow RecordBatch → `Vec<RawRecord>` for unified downstream processing
- Leverage Parquet schema for zero-inference column typing

### 1.5 Memory Source

**File:** `crates/datasynth-integration/src/sources/memory_source.rs`

```rust
pub struct MemorySource {
    tables: HashMap<String, Vec<RawRecord>>,
}
```

- For API/SDK usage and testing
- Trivial implementation of `DataSourceReader`
- Primary source for unit tests in later phases

### 1.6 Schema Harmonizer

**File:** `crates/datasynth-integration/src/harmonizer/mod.rs`

The harmonizer maps client column names to DataSynth model fields:

```rust
pub struct SchemaHarmonizer {
    strategy: MappingStrategy,
    overrides: Vec<ColumnOverride>,
    coercion: TypeCoercionRules,
}

pub enum MappingStrategy {
    AutoDetect,
    Template(TemplateId),  // sap_fi, oracle_gl, netsuite, etc.
    Manual(MappingFile),
    Hybrid { template: TemplateId, overrides: Vec<ColumnOverride> },
}

/// Result of harmonizing one source table
pub struct HarmonizedTable {
    pub target_model: String,          // e.g., "JournalEntry", "Vendor"
    pub field_mappings: Vec<FieldMapping>,
    pub unmapped_source_columns: Vec<String>,
    pub unmapped_target_fields: Vec<String>,
    pub coercion_warnings: Vec<CoercionWarning>,
}

pub struct FieldMapping {
    pub source_column: String,
    pub target_field: String,
    pub coercion: Option<CoercionRule>,
    pub confidence: f64,               // 1.0 for template/manual, 0.0–1.0 for auto-detect
}

impl SchemaHarmonizer {
    /// Analyze a source table and produce a mapping plan (does not transform data)
    pub fn plan(&self, table: &TableInfo) -> IntegrationResult<HarmonizedTable>;

    /// Apply the mapping plan to transform raw records into model-aligned records
    pub fn apply(&self, plan: &HarmonizedTable, records: Vec<RawRecord>)
        -> IntegrationResult<Vec<RawRecord>>;
}
```

### 1.7 Auto-Detection Engine

**File:** `crates/datasynth-integration/src/harmonizer/auto_detect.rs`

Column-name-to-field matching using:
1. **Exact name match** (case-insensitive): `company_code` → `company_code` (confidence: 1.0)
2. **Known alias match**: `BUKRS` → `company_code`, `BELNR` → `document_number` (confidence: 0.95)
3. **Semantic pattern match**: column named `*_amt` or `*_amount` → amount field (confidence: 0.7)
4. **Value pattern match**: column with values matching `^\d{4}-\d{2}-\d{2}$` → date field (confidence: 0.6)

The alias dictionary is built from a static table covering SAP, Oracle, and common ERP naming conventions:

```rust
static KNOWN_ALIASES: &[(&str, &str, &str)] = &[
    // (source_pattern, target_field, target_model)
    ("BUKRS",  "company_code",    "JournalEntry"),
    ("BELNR",  "document_number", "JournalEntry"),
    ("GJAHR",  "fiscal_year",     "JournalEntry"),
    ("BUZEI",  "line_item",       "JournalEntry"),
    ("DMBTR",  "amount_local",    "JournalEntry"),
    ("WRBTR",  "amount_doc",      "JournalEntry"),
    ("WAERS",  "currency",        "JournalEntry"),
    ("BUDAT",  "posting_date",    "JournalEntry"),
    ("BLDAT",  "document_date",   "JournalEntry"),
    ("HKONT",  "account_number",  "JournalEntry"),
    ("LIFNR",  "vendor_id",       "Vendor"),
    ("KUNNR",  "customer_id",     "Customer"),
    // ... 100+ aliases covering SAP FI/CO/MM/SD, Oracle GL/AP/AR
];
```

### 1.8 ERP Mapping Templates

**File:** `crates/datasynth-integration/src/harmonizer/templates.rs`

Predefined complete mappings for major ERP systems:

| Template | Source Tables | Target Models |
|----------|-------------|---------------|
| `sap_fi` | BKPF, BSEG, SKA1, SKB1, LFA1, KNA1 | JournalEntry, ChartOfAccounts, Vendor, Customer |
| `sap_mm` | EKKO, EKPO, MKPF, MSEG, RBKP, RSEG | PurchaseOrder, GoodsReceipt, VendorInvoice |
| `sap_sd` | VBAK, VBAP, LIKP, LIPS, VBRK, VBRP | SalesOrder, Delivery, CustomerInvoice |
| `oracle_gl` | GL_JE_HEADERS, GL_JE_LINES, GL_CODE_COMBINATIONS | JournalEntry, ChartOfAccounts |
| `netsuite` | transaction, transactionline, account, vendor, customer | JournalEntry, ChartOfAccounts, Vendor, Customer |

Each template is a `Vec<FieldMapping>` with confidence 1.0, loaded from embedded YAML:

**File:** `crates/datasynth-integration/src/harmonizer/templates/sap_fi.yaml`

```yaml
source_system: sap_fi
version: "1.0"
tables:
  BKPF:
    target_model: JournalEntryHeader
    fields:
      BUKRS: { target: company_code, type: string }
      BELNR: { target: document_number, type: string }
      GJAHR: { target: fiscal_year, type: integer }
      BLART: { target: document_type, type: string }
      BUDAT: { target: posting_date, type: date, format: "%Y%m%d" }
      BLDAT: { target: document_date, type: date, format: "%Y%m%d" }
      MONAT: { target: fiscal_period, type: integer }
      USNAM: { target: created_by, type: string }
      # ...
  BSEG:
    target_model: JournalEntryLine
    join_key: [BUKRS, BELNR, GJAHR]
    fields:
      BUZEI: { target: line_item, type: integer }
      HKONT: { target: account_number, type: string }
      DMBTR: { target: amount_local, type: decimal }
      WRBTR: { target: amount_doc, type: decimal }
      SHKZG: { target: debit_credit, type: string, mapping: { S: debit, H: credit } }
      # ...
```

### 1.9 Type Coercion

**File:** `crates/datasynth-integration/src/harmonizer/coercion.rs`

Convert real data values to DataSynth's expected types:

| Source Type | Target Type | Coercion Logic |
|-------------|-------------|----------------|
| String `"20250315"` | `NaiveDate` | Try configured date formats in order |
| String `"1,234.56"` | `Decimal` | Strip thousands separator, parse |
| String `"S"` / `"H"` | `DebitCredit` | Template-defined value mapping |
| String `"1100"` | `String` (account) | Left-pad to configured width |
| Integer `20250315` | `NaiveDate` | Convert to string, then parse |
| Float `1234.56` | `Decimal` | `Decimal::from_f64_retain()` |

```rust
pub struct TypeCoercionRules {
    pub date_formats: Vec<String>,
    pub amount_decimal_separator: char,
    pub amount_thousands_separator: Option<char>,
    pub boolean_true_values: Vec<String>,   // ["1", "true", "yes", "X"]
    pub boolean_false_values: Vec<String>,  // ["0", "false", "no", ""]
    pub null_values: Vec<String>,           // ["", "NULL", "N/A", "#"]
}

impl TypeCoercionRules {
    pub fn coerce(&self, value: &serde_json::Value, target_type: &TargetType)
        -> Result<serde_json::Value, CoercionWarning>;
}
```

### 1.10 Tasks

| # | Task | Files | Test |
|---|------|-------|------|
| 1.1 | Define `DataSourceReader` trait | `src/sources/mod.rs` | Trait compiles |
| 1.2 | Implement `CsvDirectorySource` | `src/sources/csv_source.rs` | Read fixture CSV, verify record count and types |
| 1.3 | Implement `JsonDirectorySource` | `src/sources/json_source.rs` | Read JSON Lines and JSON Array fixtures |
| 1.4 | Implement `ParquetDirectorySource` | `src/sources/parquet_source.rs` | Feature-gated test with sample Parquet file |
| 1.5 | Implement `MemorySource` | `src/sources/memory_source.rs` | In-memory round-trip test |
| 1.6 | Build `SchemaHarmonizer` core | `src/harmonizer/mod.rs` | Plan generation for known columns |
| 1.7 | Build auto-detection with alias dictionary | `src/harmonizer/auto_detect.rs` | SAP column names resolved correctly |
| 1.8 | Create SAP FI mapping template | `src/harmonizer/templates/sap_fi.yaml`, `templates.rs` | Template loads, all fields map |
| 1.9 | Build type coercion engine | `src/harmonizer/coercion.rs` | Date, decimal, boolean coercion tests |
| 1.10 | Integration test: CSV → harmonize → model-aligned records | `tests/harmonization_integration.rs` | End-to-end with SAP-like fixture data |

**Test Fixtures:**
- `tests/fixtures/sap_bkpf.csv` — 50 rows of realistic BKPF data
- `tests/fixtures/sap_bseg.csv` — 200 rows of corresponding BSEG data
- `tests/fixtures/simple_gl.json` — 100 journal entries in DataSynth JSON format

**Exit Criteria:** Can read CSV/JSON files, auto-detect or template-map columns to DataSynth fields, and produce `Vec<RawRecord>` with correct types. `cargo test -p datasynth-integration` passes.

---

## Phase 2: Record Alignment & Provenance

**Goal:** Match records between harmonized real data and synthetic baseline data. Track provenance for every record. Classify unmatched records.

**Depends on:** Phase 1

### 2.1 Record Alignment Engine

**File:** `crates/datasynth-integration/src/aligner/mod.rs`

```rust
pub struct RecordAligner {
    strategies: HashMap<String, AlignmentStrategy>,  // Per model type
    default_strategy: AlignmentStrategy,
}

pub enum AlignmentStrategy {
    /// Exact match on composite key fields
    ExactKey { key_fields: Vec<String> },

    /// Temporal window: match within ±N days on date fields
    TemporalWindow { key_fields: Vec<String>, date_field: String, window_days: i64 },

    /// Fuzzy match using string similarity
    FuzzyMatch { key_fields: Vec<String>, threshold: f64 },

    /// Hierarchical: try exact first, fall back to fuzzy
    Hierarchical(Vec<AlignmentStrategy>),
}

impl RecordAligner {
    pub fn align(
        &self,
        real_records: &[RawRecord],
        synthetic_records: &[RawRecord],
        model_type: &str,
    ) -> IntegrationResult<AlignmentReport>;
}

pub struct AlignmentReport {
    pub matched: Vec<MatchedPair>,
    pub real_only: Vec<UnmatchedRecord>,
    pub synthetic_only: Vec<UnmatchedRecord>,
    pub statistics: AlignmentStatistics,
}

pub struct MatchedPair {
    pub real_record: RawRecord,
    pub synthetic_record: RawRecord,
    pub alignment_key: String,
    pub confidence: f64,
    pub field_deviations: Vec<FieldDeviation>,   // Per-field comparison
}

pub struct AlignmentStatistics {
    pub total_real: usize,
    pub total_synthetic: usize,
    pub matched_count: usize,
    pub match_rate: f64,
    pub avg_confidence: f64,
    pub deviation_summary: DeviationSummary,
}
```

### 2.2 Key Matcher

**File:** `crates/datasynth-integration/src/aligner/key_matcher.rs`

Fast exact-key matching using `HashMap` join:

```rust
pub struct KeyMatcher {
    key_fields: Vec<String>,
}

impl KeyMatcher {
    /// Build composite key from record fields
    fn build_key(&self, record: &RawRecord) -> String;

    /// Match records by exact composite key
    pub fn match_exact(
        &self,
        real: &[RawRecord],
        synthetic: &[RawRecord],
    ) -> (Vec<(usize, usize)>, Vec<usize>, Vec<usize>);
    // Returns: (matched pairs as index tuples, unmatched real indices, unmatched synthetic indices)
}
```

Default key configurations per model type (from spec §5.4.1):

```rust
fn default_keys() -> HashMap<String, Vec<String>> {
    hashmap! {
        "JournalEntry" => vec!["company_code", "fiscal_year", "document_number"],
        "ChartOfAccounts" => vec!["account_number"],
        "Vendor" => vec!["vendor_id"],
        "Customer" => vec!["customer_id"],
        "PurchaseOrder" => vec!["po_number"],
        "TrialBalance" => vec!["account_number", "period"],
        // ...
    }
}
```

### 2.3 Fuzzy Matcher

**File:** `crates/datasynth-integration/src/aligner/fuzzy_matcher.rs`

For entities without exact key matches (e.g., vendors matched by name + address):

```rust
pub struct FuzzyMatcher {
    fields: Vec<FuzzyField>,
    threshold: f64,           // Minimum combined score to consider a match
}

pub struct FuzzyField {
    pub field_name: String,
    pub weight: f64,          // Relative importance (sums to 1.0)
    pub algorithm: SimilarityAlgorithm,
}

pub enum SimilarityAlgorithm {
    Levenshtein,              // Edit distance (normalized)
    JaroWinkler,              // Good for names
    TokenSet,                 // Good for addresses (order-independent)
    Numeric { tolerance: f64 }, // For amounts (relative tolerance)
}
```

Implementation:
- For small datasets (<10K × 10K): brute-force pairwise comparison
- For larger datasets: blocking strategy — group by first 3 chars of primary field, then compare within blocks
- Return top match per real record if above threshold, with confidence score

### 2.4 Document Chain Walker

**File:** `crates/datasynth-integration/src/aligner/chain_walker.rs`

For document flow alignment (P2P: PO→GR→Invoice→Payment, O2C: SO→Delivery→Invoice→Receipt):

```rust
pub struct ChainWalker {
    chain_definitions: Vec<ChainDefinition>,
}

pub struct ChainDefinition {
    pub name: String,                        // "P2P", "O2C"
    pub steps: Vec<ChainStep>,
}

pub struct ChainStep {
    pub model_type: String,                  // "PurchaseOrder", "GoodsReceipt", etc.
    pub reference_field: String,             // Field linking to next step
    pub referenced_model: String,
    pub referenced_field: String,
}

impl ChainWalker {
    /// Walk both real and synthetic chains, align at chain level
    pub fn align_chains(
        &self,
        real_records: &HashMap<String, Vec<RawRecord>>,     // model_type → records
        synthetic_records: &HashMap<String, Vec<RawRecord>>,
    ) -> IntegrationResult<ChainAlignmentReport>;
}

pub struct ChainAlignmentReport {
    pub complete_real_chains: Vec<DocumentChain>,
    pub incomplete_real_chains: Vec<DocumentChain>,     // Missing links
    pub synthetic_only_chains: Vec<DocumentChain>,      // Expected but absent
    pub chain_completion_rate: f64,
}
```

### 2.5 Provenance Tracker

**File:** `crates/datasynth-integration/src/blending/provenance.rs`

Maintains provenance metadata for every record in the blended dataset:

```rust
pub struct ProvenanceTracker {
    records: HashMap<String, RecordProvenance>,  // record_id → provenance
    access_log: Vec<AccessLogEntry>,
}

impl ProvenanceTracker {
    pub fn tag_real(&mut self, record_id: &str, source_id: &str);
    pub fn tag_synthetic(&mut self, record_id: &str, config_hash: &str, seed: u64);
    pub fn tag_blended(&mut self, record_id: &str, real_ref: &str, synthetic_ref: &str);
    pub fn add_deviations(&mut self, record_id: &str, deviations: Vec<FieldDeviation>);

    /// Export provenance map as JSON (spec §7.3 provenance.export_provenance_map)
    pub fn export(&self, path: &Path) -> IntegrationResult<()>;

    /// Summary statistics
    pub fn summary(&self) -> ProvenanceSummary;
}

pub struct ProvenanceSummary {
    pub total_records: usize,
    pub real_count: usize,
    pub synthetic_count: usize,
    pub blended_count: usize,
    pub avg_alignment_confidence: f64,
}
```

### 2.6 Field-Level Deviation Scoring

**File:** `crates/datasynth-integration/src/aligner/deviation.rs`

Compare individual fields between matched real and synthetic records:

```rust
pub fn compute_field_deviation(
    field_name: &str,
    real_value: &serde_json::Value,
    synthetic_value: &serde_json::Value,
    field_type: &FieldType,
) -> FieldDeviation;
```

Scoring logic by type:
- **Numeric**: `|real - synthetic| / max(|synthetic|, 1.0)` — relative deviation
- **String**: 1.0 - Jaro-Winkler similarity
- **Date**: `|real - synthetic|` in days, normalized by period length
- **Boolean**: 0.0 if equal, 1.0 if different
- **Categorical**: 0.0 if equal, 1.0 if different (could extend to ordinal distance)

### 2.7 Tasks

| # | Task | Files | Test |
|---|------|-------|------|
| 2.1 | Define `RecordAligner` and `AlignmentReport` | `src/aligner/mod.rs` | Types compile |
| 2.2 | Implement `KeyMatcher` with default keys per model | `src/aligner/key_matcher.rs` | 100 real + 100 synthetic JEs, verify match count |
| 2.3 | Implement `FuzzyMatcher` (Levenshtein + JaroWinkler) | `src/aligner/fuzzy_matcher.rs` | Vendor name matching with known typos |
| 2.4 | Implement `ChainWalker` for P2P and O2C chains | `src/aligner/chain_walker.rs` | Complete chain detection, broken chain identification |
| 2.5 | Implement `ProvenanceTracker` | `src/blending/provenance.rs` | Tag + export round-trip test |
| 2.6 | Implement field-level deviation scoring | `src/aligner/deviation.rs` | Numeric, string, date deviation tests |
| 2.7 | Integration test: harmonized data → align → provenance-tagged output | `tests/alignment_integration.rs` | Full pipeline with fixture data |

**Exit Criteria:** Given harmonized real records and synthetic records, can match them by key/fuzzy/chain, compute per-field deviations, tag provenance, and report match rates. All tests pass.

---

## Phase 3: Gap Analysis Engine

**Goal:** Implement the three knowledge-layer gap analyzers (structural, statistical, normative) plus the cross-layer comparator. This is the core audit value — where real data deviations from the synthetic baseline become measurable findings.

**Depends on:** Phase 2

### 3.1 Gap Analysis Orchestrator

**File:** `crates/datasynth-integration/src/gap_analysis/mod.rs`

```rust
pub struct GapAnalyzer {
    structural: StructuralAnalyzer,
    statistical: StatisticalAnalyzer,
    normative: NormativeAnalyzer,
    cross_layer: CrossLayerAnalyzer,
    severity_config: SeverityConfig,
}

impl GapAnalyzer {
    /// Run all three layer-specific analyses + cross-layer
    pub fn analyze(
        &self,
        alignment: &AlignmentReport,
        real_data: &HashMap<String, Vec<RawRecord>>,
        synthetic_data: &HashMap<String, Vec<RawRecord>>,
    ) -> IntegrationResult<KnowledgeDelta>;
}

/// The comprehensive output (spec §8.2)
pub struct KnowledgeDelta {
    pub structural: StructuralGapReport,
    pub statistical: StatisticalGapReport,
    pub normative: NormativeGapReport,
    pub cross_layer_findings: Vec<CrossLayerFinding>,
    pub overall_risk_score: f64,
    pub action_items: Vec<AuditActionItem>,
    pub generated_at: DateTime<Utc>,
    pub real_data_coverage: DataCoverage,
    pub synthetic_baseline_config: String,
}
```

### 3.2 Structural Gap Analyzer (K_S)

**File:** `crates/datasynth-integration/src/gap_analysis/structural.rs`

Checks (from spec §6.2):

```rust
pub struct StructuralAnalyzer {
    checks: Vec<Box<dyn StructuralCheck>>,
}

trait StructuralCheck: Send + Sync {
    fn name(&self) -> &str;
    fn check(
        &self,
        real: &HashMap<String, Vec<RawRecord>>,
        synthetic: &HashMap<String, Vec<RawRecord>>,
        alignment: &AlignmentReport,
    ) -> Vec<StructuralFinding>;
}
```

Concrete checks implemented as separate structs:

| Check Struct | What It Does |
|-------------|--------------|
| `CoaCompletenessCheck` | Compare real vs. synthetic chart of accounts — find missing account categories |
| `DocumentChainIntegrityCheck` | Use `ChainWalker` results to find broken P2P/O2C chains in real data |
| `EntityRelationshipCheck` | Verify master data completeness (vendors have tax IDs, bank details, etc.) |
| `IntercompanyMatchCheck` | Verify bilateral IC transactions net to zero |
| `ControlMappingCheck` | Check that all significant accounts have mapped controls |
| `ReferentialIntegrityCheck` | Verify all FK references resolve (e.g., JE account numbers exist in CoA) |

Output:

```rust
pub struct StructuralGapReport {
    pub findings: Vec<StructuralFinding>,
    pub coverage_scores: HashMap<String, f64>,  // Per entity type: 0.0–1.0
    pub overall_structural_score: f64,
}

pub struct StructuralFinding {
    pub check_name: String,
    pub severity: Severity,
    pub description: String,
    pub affected_entities: Vec<String>,
    pub expected: String,
    pub observed: String,
}
```

### 3.3 Statistical Gap Analyzer (K_Σ)

**File:** `crates/datasynth-integration/src/gap_analysis/statistical.rs`

Reuse existing `datasynth-eval` statistical infrastructure wherever possible:

```rust
pub struct StatisticalAnalyzer {
    checks: Vec<Box<dyn StatisticalCheck>>,
}

trait StatisticalCheck: Send + Sync {
    fn name(&self) -> &str;
    fn check(
        &self,
        real_amounts: &[Decimal],
        synthetic_amounts: &[Decimal],
        real_dates: &[NaiveDate],
        synthetic_dates: &[NaiveDate],
    ) -> Vec<StatisticalFinding>;
}
```

Concrete checks (from spec §6.3):

| Check Struct | Method | Reuse From |
|-------------|--------|------------|
| `BenfordCheck` | MAD against first-digit law | `datasynth-eval::statistical::benford` |
| `DistributionCheck` | KS test, real vs. synthetic amounts | `datasynth-eval::statistical::distributions` |
| `TemporalPatternCheck` | Correlation of monthly volumes | `datasynth-eval::statistical::temporal` |
| `PeriodEndSpikeCheck` | Last-5-day / mid-month volume ratio | New, but uses `datasynth-eval` temporal |
| `CorrelationStructureCheck` | Frobenius norm of correlation matrix difference | `datasynth-eval::statistical::distributions` |
| `RoundNumberCheck` | Proportion of round amounts vs. expected | New |
| `WeekendHolidayCheck` | Non-business-day posting count | Uses `datasynth-core::distributions::holidays` |
| `DuplicatePatternCheck` | Exact and near-duplicate rates | `datasynth-eval::quality::uniqueness` |

Output:

```rust
pub struct StatisticalGapReport {
    pub findings: Vec<StatisticalFinding>,
    pub per_metric_scores: HashMap<String, MetricScore>,
    pub overall_statistical_score: f64,
}

pub struct StatisticalFinding {
    pub check_name: String,
    pub severity: Severity,
    pub description: String,
    pub test_statistic: f64,
    pub p_value: Option<f64>,
    pub effect_size: Option<f64>,
    pub threshold: f64,
    pub passed: bool,
}

pub struct MetricScore {
    pub metric_name: String,
    pub score: f64,           // 0.0 = perfect match, 1.0 = maximum deviation
    pub p_value: Option<f64>,
    pub sample_size: usize,
}
```

### 3.4 Normative Gap Analyzer (K_N)

**File:** `crates/datasynth-integration/src/gap_analysis/normative.rs`

Checks (from spec §6.4):

| Check Struct | Standard | What It Compares |
|-------------|----------|-----------------|
| `SodViolationCheck` | SOX / COSO | Synthetic violation rate vs. actual |
| `ApprovalThresholdCheck` | Internal policy | Entries exceeding thresholds without proper approval |
| `ThreeWayMatchCheck` | Procurement | PO/GR/Invoice match rate vs. synthetic baseline |
| `BalanceSheetEquationCheck` | GAAP | A = L + E validation on real trial balance |
| `ControlEffectivenessCheck` | COSO 2013 | Synthetic maturity levels vs. real test results |
| `AuditTrailCheck` | ISA 230 | Documentation chain completeness |

```rust
pub struct NormativeGapReport {
    pub findings: Vec<NormativeFinding>,
    pub per_standard_scores: HashMap<String, ComplianceScore>,
    pub overall_normative_score: f64,
}

pub struct NormativeFinding {
    pub check_name: String,
    pub standard: String,         // "SOX", "COSO 2013", "ISA 230", etc.
    pub severity: Severity,
    pub description: String,
    pub expected_rate: f64,       // From synthetic baseline
    pub observed_rate: f64,       // From real data
    pub population_size: usize,
    pub violation_count: usize,
}
```

### 3.5 Cross-Layer Analyzer

**File:** `crates/datasynth-integration/src/gap_analysis/cross_layer.rs`

Combines findings from all three layers to identify compound patterns (spec §8.1):

```rust
pub struct CrossLayerAnalyzer {
    patterns: Vec<Box<dyn CrossLayerPattern>>,
}

trait CrossLayerPattern: Send + Sync {
    fn name(&self) -> &str;
    fn detect(
        &self,
        structural: &StructuralGapReport,
        statistical: &StatisticalGapReport,
        normative: &NormativeGapReport,
        alignment: &AlignmentReport,
    ) -> Vec<CrossLayerFinding>;
}
```

Concrete patterns:

| Pattern | Layers | Detection Logic |
|---------|--------|-----------------|
| `StatisticalNormativeCorrelation` | K_Σ + K_N | Benford deviations concentrated in accounts with weak controls |
| `StructuralStatisticalCorrelation` | K_S + K_Σ | Missing chain links correlate with unusual amounts |
| `FullStackAnomaly` | K_S + K_Σ + K_N | New entity + unusual statistics + control bypass |
| `ThresholdStructuring` | K_Σ + K_N | Amounts clustered just below approval thresholds |
| `CompensatingControl` | K_N + K_Σ | Control weakness offset by conservative distribution |

### 3.6 Risk Scoring

**File:** `crates/datasynth-integration/src/gap_analysis/scoring.rs`

Implements the weighted scoring model from spec §8.3:

```rust
pub struct RiskScorer {
    pub structural_weight: f64,     // Default: 0.25
    pub statistical_weight: f64,    // Default: 0.30
    pub normative_weight: f64,      // Default: 0.30
    pub cross_layer_weight: f64,    // Default: 0.15
}

impl RiskScorer {
    pub fn score(&self, delta: &KnowledgeDelta) -> f64;
}
```

### 3.7 Severity Classification

**File:** `crates/datasynth-integration/src/gap_analysis/severity.rs`

Configurable severity thresholds (spec §6.5):

```rust
pub struct SeverityConfig {
    pub benford_mad_critical: f64,          // Default: 0.05
    pub benford_mad_high: f64,              // Default: 0.025
    pub benford_mad_medium: f64,            // Default: 0.015
    pub chain_break_critical_pct: f64,      // Default: 0.10
    pub sod_violation_critical_pct: f64,    // Default: 0.05
    pub balance_equation_any: Severity,     // Always Critical
    // ...
}

impl SeverityConfig {
    pub fn classify(&self, finding: &dyn Finding) -> Severity;
}
```

### 3.8 Tasks

| # | Task | Files | Test |
|---|------|-------|------|
| 3.1 | Define `GapAnalyzer` orchestrator and `KnowledgeDelta` | `src/gap_analysis/mod.rs` | Types compile |
| 3.2 | Implement `CoaCompletenessCheck` | `src/gap_analysis/structural.rs` | Missing account detection |
| 3.3 | Implement `DocumentChainIntegrityCheck` | `src/gap_analysis/structural.rs` | Broken chain detection |
| 3.4 | Implement remaining structural checks | `src/gap_analysis/structural.rs` | Entity, IC, control, FK checks |
| 3.5 | Implement `BenfordCheck` (wrapping `datasynth-eval`) | `src/gap_analysis/statistical.rs` | Real amounts with known Benford deviation |
| 3.6 | Implement `DistributionCheck` (KS test) | `src/gap_analysis/statistical.rs` | Real vs. synthetic distribution comparison |
| 3.7 | Implement remaining statistical checks | `src/gap_analysis/statistical.rs` | Temporal, round number, duplicate checks |
| 3.8 | Implement `SodViolationCheck` | `src/gap_analysis/normative.rs` | Known SoD violations detected |
| 3.9 | Implement remaining normative checks | `src/gap_analysis/normative.rs` | Threshold, three-way match, balance equation |
| 3.10 | Implement cross-layer patterns | `src/gap_analysis/cross_layer.rs` | Full-stack anomaly detected in synthetic test data |
| 3.11 | Implement risk scoring | `src/gap_analysis/scoring.rs` | Score computation matches manual calculation |
| 3.12 | Implement severity classification | `src/gap_analysis/severity.rs` | Threshold-based classification tests |
| 3.13 | Integration test: aligned data → gap analysis → KnowledgeDelta | `tests/gap_analysis_integration.rs` | Full pipeline with planted anomalies |

**Key Design Decision:** The statistical checks should **delegate** to `datasynth-eval` wherever possible rather than reimplementing. The gap analyzer wraps eval metrics with comparison logic (real vs. expected) and severity classification.

**Exit Criteria:** Given aligned real + synthetic data, produces a `KnowledgeDelta` with per-layer findings, cross-layer patterns, risk scores, and severity classifications. Integration test with planted anomalies detects all Critical and High findings.

---

## Phase 4: Blending Engine — Overlay & Augmentation

**Goal:** Implement the two data-producing integration modes: overlay (real replaces synthetic with provenance) and augmentation (synthetic fills gaps in real data).

**Depends on:** Phase 2

### 4.1 Overlay Engine

**File:** `crates/datasynth-integration/src/blending/overlay.rs`

Overlay mode replaces synthetic records with their real counterparts, preserving the synthetic original as a reference:

```rust
pub struct OverlayEngine {
    provenance: ProvenanceTracker,
    deviation_threshold: f64,     // Only flag deviations above this score
}

impl OverlayEngine {
    /// Create blended dataset from alignment results
    pub fn overlay(
        &mut self,
        alignment: &AlignmentReport,
    ) -> IntegrationResult<BlendedDataset>;
}

pub struct BlendedDataset {
    pub records: HashMap<String, Vec<BlendedRecord>>,  // model_type → records
    pub provenance: ProvenanceTracker,
    pub statistics: BlendingStatistics,
}

pub struct BlendedRecord {
    pub id: String,
    pub data: RawRecord,
    pub provenance: RecordProvenance,
    pub audit_flags: Vec<AuditFlag>,
}

pub struct AuditFlag {
    pub flag_type: AuditFlagType,
    pub description: String,
    pub severity: Severity,
    pub field_name: Option<String>,
}

pub enum AuditFlagType {
    HighDeviation,           // Field value deviates significantly from synthetic
    MissingExpectedField,    // Field present in synthetic but absent in real
    UnexpectedValue,         // Value outside expected range
    PatternAnomaly,          // Value pattern differs from synthetic (e.g., format)
}
```

Blending rules:
1. **Matched records**: Real data is primary; synthetic data stored as reference. Per-field deviations computed. Audit flags generated for deviations above threshold.
2. **Real-only records**: Included with `DataSource::Real` provenance. Flagged as `CoverageGap` if no synthetic model exists for that type, or `ClientSpecific` otherwise.
3. **Synthetic-only records**: Included with `DataSource::Synthetic` provenance. Flagged as potential completeness findings.

### 4.2 Augmentation Engine

**File:** `crates/datasynth-integration/src/blending/augmentation.rs`

Augmentation fills gaps in real data with calibrated synthetic records:

```rust
pub struct AugmentationEngine {
    provenance: ProvenanceTracker,
}

impl AugmentationEngine {
    /// Augment real data with synthetic fill
    pub fn augment(
        &mut self,
        real_data: &HashMap<String, Vec<RawRecord>>,
        synthetic_data: &HashMap<String, Vec<RawRecord>>,
        config: &AugmentationConfig,
    ) -> IntegrationResult<AugmentedDataset>;
}

pub struct AugmentedDataset {
    pub records: HashMap<String, Vec<BlendedRecord>>,
    pub provenance: ProvenanceTracker,
    pub augmentation_summary: AugmentationSummary,
}

pub struct AugmentationSummary {
    pub real_record_count: usize,
    pub synthetic_fill_count: usize,
    pub fill_by_strategy: HashMap<String, usize>,  // strategy_name → count
    pub domains_added: Vec<String>,
    pub temporal_range_extended: Option<(NaiveDate, NaiveDate)>,
}
```

Augmentation strategies (spec §7.2):

| Strategy | Implementation |
|----------|---------------|
| **Temporal extension** | Filter synthetic records by date range outside real data coverage; tag with `temporal_extension` basis |
| **Domain fill** | Include all synthetic records for model types not present in real data; tag with `domain_fill` basis |
| **Volume scaling** | Duplicate real records with randomized perturbation for stress testing (not in initial implementation — defer to Phase 7) |
| **Anomaly enrichment** | Use existing `datasynth-generators::anomaly::injector` to inject labeled anomalies into the augmented dataset |
| **Coverage completion** | Use `ChainWalker` to find incomplete document chains in real data; fill missing links from synthetic data |

### 4.3 Anomaly Injection Bridge

**File:** `crates/datasynth-integration/src/blending/anomaly_bridge.rs`

Bridge between the existing anomaly injection framework and the augmentation engine:

```rust
pub struct AnomalyBridge;

impl AnomalyBridge {
    /// Inject labeled anomalies into augmented dataset
    pub fn inject(
        records: &mut Vec<BlendedRecord>,
        config: &AnomalyEnrichmentConfig,
        seed: u64,
    ) -> IntegrationResult<Vec<LabeledAnomaly>>;
}
```

This delegates to `datasynth-generators::anomaly::injector` but wraps the output with provenance tags (`source: synthetic, basis: anomaly_injection`).

### 4.4 Blended Dataset Export

**File:** `crates/datasynth-integration/src/blending/export.rs`

Export blended/augmented datasets in standard formats:

```rust
pub struct BlendedExporter;

impl BlendedExporter {
    /// Export blended records as CSV with provenance column
    pub fn to_csv(dataset: &BlendedDataset, output_dir: &Path) -> IntegrationResult<()>;

    /// Export as JSON with embedded provenance
    pub fn to_json(dataset: &BlendedDataset, output_dir: &Path) -> IntegrationResult<()>;

    /// Export provenance map as separate file
    pub fn export_provenance(dataset: &BlendedDataset, path: &Path) -> IntegrationResult<()>;
}
```

Output structure:
```
output/
├── blended/
│   ├── journal_entries.csv          # Real + synthetic records
│   ├── vendors.csv
│   ├── ...
├── provenance/
│   ├── provenance_map.json          # record_id → source + confidence
│   ├── deviation_summary.json       # Per-field deviation statistics
│   └── augmentation_summary.json    # What was filled and why
```

### 4.5 Tasks

| # | Task | Files | Test |
|---|------|-------|------|
| 4.1 | Implement `OverlayEngine` | `src/blending/overlay.rs` | Overlay 50 real + 100 synthetic → 100 blended with correct provenance |
| 4.2 | Implement audit flag generation for high-deviation fields | `src/blending/overlay.rs` | Known deviations produce correct flags |
| 4.3 | Implement `AugmentationEngine` — temporal extension | `src/blending/augmentation.rs` | 6-month real data extended to 12 months |
| 4.4 | Implement domain fill strategy | `src/blending/augmentation.rs` | Manufacturing domain added to services-only real data |
| 4.5 | Implement coverage completion (chain fill) | `src/blending/augmentation.rs` | Incomplete P2P chains completed from synthetic |
| 4.6 | Build anomaly injection bridge | `src/blending/anomaly_bridge.rs` | Injected anomalies have correct provenance tags |
| 4.7 | Implement blended dataset CSV/JSON export | `src/blending/export.rs` | Export → re-read → verify provenance preserved |
| 4.8 | Integration test: full overlay pipeline | `tests/overlay_integration.rs` | End-to-end overlay with deviation detection |
| 4.9 | Integration test: full augmentation pipeline | `tests/augmentation_integration.rs` | End-to-end augmentation with all strategies |

**Exit Criteria:** Can produce blended and augmented datasets with full provenance tracking. Every record tagged as real, synthetic, or blended. Export to CSV/JSON preserves provenance. All tests pass.

---

## Phase 5: Audit Evidence & Reporting

**Goal:** Transform gap analysis findings into structured audit evidence and exportable reports (HTML, JSON, workpapers).

**Depends on:** Phase 3

### 5.1 Evidence Generator

**File:** `crates/datasynth-integration/src/evidence/mod.rs`

```rust
pub struct EvidenceGenerator {
    generators: Vec<Box<dyn EvidenceProducer>>,
}

trait EvidenceProducer: Send + Sync {
    fn name(&self) -> &str;
    fn isa_reference(&self) -> &str;
    fn produce(
        &self,
        delta: &KnowledgeDelta,
        alignment: &AlignmentReport,
    ) -> Vec<AuditEvidence>;
}

pub struct AuditEvidence {
    pub evidence_type: EvidenceType,
    pub isa_reference: String,
    pub title: String,
    pub description: String,
    pub findings: Vec<String>,            // References to specific findings
    pub data_exhibits: Vec<DataExhibit>,  // Tables, charts, statistics
    pub evidence_chain: EvidenceChain,    // Full provenance chain (spec §9.2)
    pub generated_at: DateTime<Utc>,
}

pub enum EvidenceType {
    AnalyticalProcedure,     // ISA 520
    TestOfDetails,           // ISA 500
    ControlTest,             // ISA 330
    SubstantiveProcedure,    // ISA 330
    CompletenessAssertion,   // ISA 505
    GoingConcernIndicator,   // ISA 570
}

pub struct EvidenceChain {
    pub steps: Vec<EvidenceChainStep>,
}

pub struct EvidenceChainStep {
    pub level: String,       // "Audit Finding", "Gap Analysis Result", "Record Comparison", etc.
    pub description: String,
    pub reference: String,   // ID or hash linking to source
}
```

### 5.2 Concrete Evidence Producers

| Producer | ISA | Input | Output |
|----------|-----|-------|--------|
| `AnalyticalProcedureEvidence` | ISA 520 | `StatisticalGapReport` | Expectation vs. actual tables with thresholds |
| `DetailTestingEvidence` | ISA 500 | `AlignmentReport` + matched pairs | Sample with pass/fail per attribute |
| `ControlTestEvidence` | ISA 330 | `NormativeGapReport` | Control effectiveness assessments |
| `CompletenessEvidence` | ISA 505 | `StructuralGapReport` | Coverage maps with gap highlighting |
| `GoingConcernEvidence` | ISA 570 | Cross-layer trend data | Financial health trajectory indicators |

### 5.3 Data Exhibits

**File:** `crates/datasynth-integration/src/evidence/exhibits.rs`

```rust
pub enum DataExhibit {
    /// Statistical table (e.g., Benford digit distribution)
    Table {
        title: String,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        footer: Option<String>,
    },

    /// Key-value summary (e.g., KS test results)
    Summary {
        title: String,
        metrics: Vec<(String, String)>,  // (label, value)
    },

    /// Deviation heatmap data (rendered by report generator)
    Heatmap {
        title: String,
        x_labels: Vec<String>,  // e.g., account numbers
        y_labels: Vec<String>,  // e.g., check names
        values: Vec<Vec<f64>>,  // Deviation scores
    },

    /// Distribution comparison (real vs. expected)
    DistributionComparison {
        title: String,
        real_histogram: Vec<(f64, f64)>,       // (bin_start, count)
        synthetic_histogram: Vec<(f64, f64)>,
        test_statistic: f64,
        p_value: f64,
    },
}
```

### 5.4 HTML Report Generator

**File:** `crates/datasynth-integration/src/report/html_report.rs`

Generates self-contained HTML reports with embedded CSS and inline SVG charts:

```rust
pub struct HtmlReportGenerator {
    template: String,    // Embedded HTML template
}

impl HtmlReportGenerator {
    pub fn generate(
        &self,
        delta: &KnowledgeDelta,
        evidence: &[AuditEvidence],
        config: &ReportConfig,
    ) -> IntegrationResult<String>;  // Returns HTML string
}
```

Report sections:
1. **Executive Summary** — Overall risk score, finding counts by severity, data coverage
2. **Structural Analysis** — CoA completeness, document chain integrity, entity coverage
3. **Statistical Analysis** — Benford results, distribution comparisons, temporal patterns
4. **Normative Analysis** — Compliance scores by standard, control effectiveness
5. **Cross-Layer Findings** — Compound patterns with full evidence chains
6. **Action Items** — Prioritized list of recommended audit procedures
7. **Data Exhibits** — All statistical tables, heatmaps, and distribution comparisons
8. **Methodology** — Description of synthetic baseline, comparison methods, thresholds used

Use the same approach as `datasynth-eval::report::html` — embedded template with `{placeholder}` substitution, no external template engine dependency.

### 5.5 JSON Report Generator

**File:** `crates/datasynth-integration/src/report/json_report.rs`

Serialize `KnowledgeDelta` + `Vec<AuditEvidence>` as structured JSON for programmatic consumption:

```rust
pub struct JsonReportGenerator;

impl JsonReportGenerator {
    pub fn generate(
        &self,
        delta: &KnowledgeDelta,
        evidence: &[AuditEvidence],
    ) -> IntegrationResult<serde_json::Value>;

    pub fn write_to_file(
        &self,
        delta: &KnowledgeDelta,
        evidence: &[AuditEvidence],
        path: &Path,
    ) -> IntegrationResult<()>;
}
```

### 5.6 Workpaper Generator

**File:** `crates/datasynth-integration/src/report/workpaper.rs`

Generate audit workpaper files organized by ISA standard:

```rust
pub struct WorkpaperGenerator;

impl WorkpaperGenerator {
    pub fn generate(
        &self,
        delta: &KnowledgeDelta,
        evidence: &[AuditEvidence],
        output_dir: &Path,
    ) -> IntegrationResult<WorkpaperManifest>;
}

pub struct WorkpaperManifest {
    pub files: Vec<WorkpaperFile>,
    pub engagement_info: EngagementInfo,
}
```

Output structure:
```
workpapers/
├── manifest.json
├── WP-100_executive_summary.html
├── WP-200_risk_assessment.html        # ISA 315
├── WP-300_materiality.html            # ISA 320
├── WP-400_analytical_procedures.html  # ISA 520
├── WP-500_detail_testing.html         # ISA 500
├── WP-600_control_testing.html        # ISA 330
├── WP-700_findings.html               # Cross-layer findings
├── exhibits/
│   ├── benford_analysis.html
│   ├── distribution_comparison.html
│   ├── deviation_heatmap.html
│   └── document_chain_coverage.html
└── data/
    ├── knowledge_delta.json            # Machine-readable full results
    ├── provenance_map.json
    └── evidence_chain.json
```

### 5.7 Tasks

| # | Task | Files | Test |
|---|------|-------|------|
| 5.1 | Define evidence types and `EvidenceGenerator` | `src/evidence/mod.rs` | Types compile |
| 5.2 | Implement `AnalyticalProcedureEvidence` | `src/evidence/analytical.rs` | Benford exhibit generated from statistical gap |
| 5.3 | Implement `DetailTestingEvidence` | `src/evidence/detail_testing.rs` | Sample with pass/fail from alignment |
| 5.4 | Implement `ControlTestEvidence` | `src/evidence/control_testing.rs` | Control assessment from normative gap |
| 5.5 | Implement evidence chain construction | `src/evidence/chain.rs` | Full chain from finding to source |
| 5.6 | Implement data exhibits | `src/evidence/exhibits.rs` | Table, heatmap, distribution rendering |
| 5.7 | Implement HTML report generator | `src/report/html_report.rs` | Generate valid HTML, open in browser |
| 5.8 | Implement JSON report generator | `src/report/json_report.rs` | Round-trip serialization test |
| 5.9 | Implement workpaper generator | `src/report/workpaper.rs` | Correct directory structure and manifest |
| 5.10 | Integration test: KnowledgeDelta → evidence → HTML report | `tests/reporting_integration.rs` | Full pipeline produces valid report |

**Exit Criteria:** Given a `KnowledgeDelta`, produces structured audit evidence with full provenance chains, and exports to HTML workpapers, JSON, and organized workpaper directories. HTML report is self-contained and renders correctly in a browser.

---

## Phase 6: CLI, Server & Python SDK Integration

**Goal:** Expose all integration capabilities through the existing CLI (`datasynth-data integrate`), REST API, and Python SDK.

**Depends on:** Phase 3, Phase 4, Phase 5

### 6.1 CLI: `integrate` Subcommand

**File:** `crates/datasynth-cli/src/main.rs` (extend `Commands` enum)

Add a new top-level subcommand with sub-subcommands:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing: Generate, Validate, Init, Info, Verify, Fingerprint, Scenario
    /// Integrate real client data with synthetic baseline
    Integrate {
        #[command(subcommand)]
        command: IntegrateCommands,
    },
}

#[derive(Subcommand)]
enum IntegrateCommands {
    /// Run gap analysis: compare real data against synthetic baseline
    GapAnalysis {
        /// Path to real client data (CSV/JSON/Parquet directory)
        #[arg(long)]
        real: PathBuf,

        /// Path to synthetic baseline data (DataSynth output directory)
        #[arg(long)]
        baseline: PathBuf,

        /// Schema mapping template (sap_fi, oracle_gl, netsuite, auto)
        #[arg(long, default_value = "auto")]
        mapping: String,

        /// Output directory for gap report
        #[arg(short, long, default_value = "./gap_report")]
        output: PathBuf,

        /// Report formats to generate
        #[arg(long, value_delimiter = ',', default_value = "html,json")]
        format: Vec<String>,

        /// Materiality threshold for severity classification
        #[arg(long)]
        materiality: Option<f64>,
    },

    /// Create blended dataset: real data overlaid on synthetic baseline
    Overlay {
        #[arg(long)]
        real: PathBuf,
        #[arg(long)]
        synthetic: PathBuf,
        #[arg(long, default_value = "auto")]
        mapping: String,
        #[arg(short, long, default_value = "./blended")]
        output: PathBuf,
    },

    /// Augment sparse real data with synthetic fill
    Augment {
        #[arg(long)]
        real: PathBuf,
        /// Extend temporal coverage to this date
        #[arg(long)]
        extend_to: Option<String>,
        /// Add synthetic data for these domains
        #[arg(long, value_delimiter = ',')]
        add_domains: Option<Vec<String>>,
        /// Anomaly injection rate (0.0–1.0)
        #[arg(long)]
        anomaly_rate: Option<f64>,
        #[arg(short, long, default_value = "./augmented")]
        output: PathBuf,
        /// Industry preset for synthetic fill calibration
        #[arg(long, default_value = "manufacturing")]
        industry: String,
    },

    /// Run full audit workflow (planning → fieldwork → reporting)
    Audit {
        #[arg(long)]
        real: PathBuf,
        #[arg(long, default_value = "manufacturing")]
        industry: String,
        #[arg(long)]
        materiality: f64,
        #[arg(short, long, default_value = "./workpapers")]
        output: PathBuf,
        /// Audit phases to run
        #[arg(long, value_delimiter = ',', default_value = "planning,substantive,detail,completion")]
        phases: Vec<String>,
        /// Schema mapping template
        #[arg(long, default_value = "auto")]
        mapping: String,
    },

    /// Inspect a real data source (show tables, columns, inferred types)
    Inspect {
        #[arg(long)]
        source: PathBuf,
        /// Show detailed column analysis
        #[arg(long)]
        detailed: bool,
    },
}
```

### 6.2 CLI Execution Logic

**File:** `crates/datasynth-cli/src/integrate.rs` (new file)

Each subcommand follows the same pattern:

```rust
pub fn run_gap_analysis(args: GapAnalysisArgs) -> Result<()> {
    // 1. Create source reader
    let source = detect_source_type(&args.real)?;

    // 2. Read and harmonize
    let harmonizer = SchemaHarmonizer::new(args.mapping)?;
    let real_data = harmonize_all_tables(&source, &harmonizer)?;

    // 3. Load synthetic baseline
    let synthetic_data = load_synthetic_baseline(&args.baseline)?;

    // 4. Align records
    let aligner = RecordAligner::with_defaults();
    let alignment = aligner.align_all(&real_data, &synthetic_data)?;

    // 5. Run gap analysis
    let analyzer = GapAnalyzer::with_defaults();
    let delta = analyzer.analyze(&alignment, &real_data, &synthetic_data)?;

    // 6. Generate evidence
    let evidence_gen = EvidenceGenerator::with_defaults();
    let evidence = evidence_gen.produce(&delta, &alignment)?;

    // 7. Generate reports
    for format in &args.format {
        match format.as_str() {
            "html" => HtmlReportGenerator::new().write(&delta, &evidence, &args.output)?,
            "json" => JsonReportGenerator.write_to_file(&delta, &evidence, &args.output.join("report.json"))?,
            _ => warn!("Unknown format: {}", format),
        }
    }

    // 8. Print summary
    println!("Gap Analysis Complete");
    println!("  Overall risk score: {:.2}", delta.overall_risk_score);
    println!("  Findings: {} critical, {} high, {} medium",
        count_severity(&delta, Severity::Critical),
        count_severity(&delta, Severity::High),
        count_severity(&delta, Severity::Medium),
    );
    println!("  Report: {}", args.output.display());

    Ok(())
}
```

### 6.3 Server API Extension

**File:** `crates/datasynth-server/src/routes/integration.rs` (new file)

Add REST endpoints under `/api/integration/`:

```rust
pub fn integration_routes() -> Router {
    Router::new()
        .route("/gap-analysis", post(run_gap_analysis))
        .route("/overlay", post(run_overlay))
        .route("/augment", post(run_augment))
        .route("/status/:id", get(get_status))
        .route("/report/:id", get(get_report))
}
```

Request/response models:

```rust
#[derive(Deserialize)]
pub struct GapAnalysisRequest {
    pub real_data_path: String,
    pub baseline_path: String,
    pub mapping: String,
    pub materiality: Option<f64>,
    pub output_formats: Vec<String>,
}

#[derive(Serialize)]
pub struct GapAnalysisResponse {
    pub job_id: String,
    pub status: JobStatus,
}

#[derive(Serialize)]
pub enum JobStatus {
    Pending,
    Running { progress: f64 },
    Complete { report_url: String, risk_score: f64, finding_count: usize },
    Failed { error: String },
}
```

Integration jobs run async (Tokio task), status polled via `/status/{id}` or streamed via WebSocket `/ws/integration/{id}`.

### 6.4 Server WebSocket Streaming

**File:** `crates/datasynth-server/src/ws/integration.rs` (new file)

Stream integration progress events to connected WebSocket clients:

```rust
pub enum IntegrationEvent {
    PhaseStarted { phase: String },
    Progress { phase: String, percent: f64, message: String },
    FindingDetected { severity: String, description: String },
    PhaseComplete { phase: String, duration_ms: u64 },
    Complete { risk_score: f64, report_url: String },
    Error { message: String },
}
```

### 6.5 Python SDK Extension

**File:** `python/datasynth_py/integration.py` (new file)

```python
class Integration:
    """Real-synthetic data integration capabilities."""

    def __init__(self, datasynth: "DataSynth"):
        self._ds = datasynth

    def gap_analysis(
        self,
        real_data: str,
        baseline: str,
        mapping: str = "auto",
        materiality: Optional[float] = None,
    ) -> "GapAnalysisReport":
        """Run gap analysis comparing real data against synthetic baseline."""
        # Calls datasynth-data integrate gap-analysis via subprocess
        # Parses JSON report output
        ...

    def overlay(
        self,
        real_data: str,
        synthetic: str,
        mapping: str = "auto",
    ) -> "BlendedDataset":
        """Create blended dataset with real data overlaid on synthetic."""
        ...

    def augment(
        self,
        real_data: str,
        extend_to: Optional[str] = None,
        add_domains: Optional[List[str]] = None,
        anomaly_rate: float = 0.0,
        industry: str = "manufacturing",
    ) -> "AugmentedDataset":
        """Augment sparse real data with synthetic fill."""
        ...

    def audit_workflow(
        self,
        real_data: str,
        industry: str = "manufacturing",
        materiality: float = 50000.0,
        phases: Optional[List[str]] = None,
    ) -> "AuditWorkflow":
        """Run full audit workflow."""
        ...


class GapAnalysisReport:
    """Results of a gap analysis."""
    risk_score: float
    structural_findings: List[Finding]
    statistical_findings: List[Finding]
    normative_findings: List[Finding]
    cross_layer_findings: List[Finding]
    critical_findings: List[Finding]

    def to_html(self, path: str) -> None: ...
    def to_json(self) -> dict: ...
    def export_workpapers(self, path: str) -> None: ...
```

**File:** `python/datasynth_py/__init__.py` (extend)

Add `integration` property to `DataSynth` class:

```python
@property
def integration(self) -> Integration:
    return Integration(self)
```

### 6.6 Tasks

| # | Task | Files | Test |
|---|------|-------|------|
| 6.1 | Add `Integrate` subcommand to CLI | `datasynth-cli/src/main.rs` | `datasynth-data integrate --help` shows subcommands |
| 6.2 | Implement `gap-analysis` CLI handler | `datasynth-cli/src/integrate.rs` | CLI runs against fixture data, produces HTML report |
| 6.3 | Implement `overlay` CLI handler | `datasynth-cli/src/integrate.rs` | CLI produces blended output directory |
| 6.4 | Implement `augment` CLI handler | `datasynth-cli/src/integrate.rs` | CLI produces augmented dataset |
| 6.5 | Implement `audit` CLI handler | `datasynth-cli/src/integrate.rs` | CLI produces workpaper directory |
| 6.6 | Implement `inspect` CLI handler | `datasynth-cli/src/integrate.rs` | CLI shows table/column analysis |
| 6.7 | Add integration REST routes to server | `datasynth-server/src/routes/integration.rs` | POST endpoint returns job ID |
| 6.8 | Add WebSocket streaming for integration | `datasynth-server/src/ws/integration.rs` | Progress events streamed to client |
| 6.9 | Add Python `Integration` class | `python/datasynth_py/integration.py` | `ds.integration.gap_analysis()` returns report |
| 6.10 | Add Python result classes | `python/datasynth_py/integration.py` | Report properties accessible |
| 6.11 | CLI integration test: end-to-end gap analysis | `datasynth-cli/tests/integrate_test.rs` | Full CLI invocation with fixture data |
| 6.12 | Python integration test | `python/tests/test_integration.py` | Python round-trip test |

**Exit Criteria:** All four integration modes accessible via CLI, REST API, and Python SDK. `datasynth-data integrate gap-analysis` produces a valid HTML report from real CSV data. Server endpoints return async job status. Python SDK returns typed result objects.

---

## Phase 7: End-to-End Workflows & Quality Gates

**Goal:** Implement the high-level audit workflows (external engagement, internal continuous monitoring) and add quality gates to ensure integration results meet configurable standards.

**Depends on:** Phase 6

### 7.1 Audit Workflow Orchestrator

**File:** `crates/datasynth-integration/src/workflow/mod.rs`

Coordinates the multi-phase audit workflow (spec §10.1):

```rust
pub struct AuditWorkflow {
    config: WorkflowConfig,
    phases: Vec<AuditPhase>,
}

pub enum AuditPhase {
    /// Phase 1: Extract fingerprint, generate baseline, run preliminary gap analysis
    Planning {
        fingerprint_privacy: PrivacyLevel,
        industry: String,
        complexity: String,
    },
    /// Phase 2: Direct overlay, record-level gap analysis, sample selection
    Substantive {
        mapping: String,
        materiality: f64,
    },
    /// Phase 3: Detail testing on deviation-weighted samples
    DetailTesting {
        sample_strategy: SampleStrategy,
        sample_size: usize,
    },
    /// Phase 4: Subsequent events, going concern, completion procedures
    Completion {
        subsequent_event_cutoff: NaiveDate,
    },
}

pub enum SampleStrategy {
    /// Random sampling
    Random,
    /// Weighted by deviation score (high-deviation records sampled more)
    DeviationWeighted,
    /// Stratified by account category
    Stratified { strata_field: String },
    /// Monetary Unit Sampling
    MonetaryUnit,
}

impl AuditWorkflow {
    pub fn run(&self) -> IntegrationResult<AuditWorkflowResult>;
}

pub struct AuditWorkflowResult {
    pub phases: Vec<PhaseResult>,
    pub final_delta: KnowledgeDelta,
    pub evidence: Vec<AuditEvidence>,
    pub workpaper_path: PathBuf,
    pub summary: WorkflowSummary,
}
```

### 7.2 Deviation-Weighted Sampling

**File:** `crates/datasynth-integration/src/workflow/sampling.rs`

Select audit samples biased toward high-risk records:

```rust
pub struct DeviationWeightedSampler;

impl DeviationWeightedSampler {
    /// Select sample from matched pairs, weighted by aggregate deviation score
    pub fn select(
        &self,
        matched_pairs: &[MatchedPair],
        sample_size: usize,
        seed: u64,
    ) -> Vec<&MatchedPair>;
}
```

The aggregate deviation score for a record is the sum of its field deviation scores, weighted by field importance (amount fields weighted higher than description fields).

### 7.3 Continuous Monitoring Mode

**File:** `crates/datasynth-integration/src/workflow/continuous.rs`

For internal audit teams running daily/weekly analysis (spec §10.2):

```rust
pub struct ContinuousMonitor {
    config: ContinuousConfig,
    last_run: Option<DateTime<Utc>>,
    baseline_hash: String,
    finding_history: Vec<FindingSnapshot>,
}

impl ContinuousMonitor {
    /// Run incremental analysis on new/changed records since last run
    pub fn run_incremental(
        &mut self,
        source: &dyn DataSourceReader,
    ) -> IntegrationResult<IncrementalReport>;

    /// Full recalibration: re-extract fingerprint, regenerate baseline
    pub fn recalibrate(
        &mut self,
        source: &dyn DataSourceReader,
    ) -> IntegrationResult<()>;

    /// Trend analysis: compare current findings with historical
    pub fn trend_analysis(&self) -> TrendReport;
}

pub struct IncrementalReport {
    pub new_findings: Vec<CrossLayerFinding>,
    pub resolved_findings: Vec<String>,       // Finding IDs no longer present
    pub trend_direction: TrendDirection,       // Improving, Stable, Deteriorating
    pub delta_risk_score: f64,                // Change in overall risk score
}

pub enum TrendDirection {
    Improving,
    Stable,
    Deteriorating,
}
```

### 7.4 Quality Gates

**File:** `crates/datasynth-integration/src/workflow/quality_gate.rs`

Configurable quality gates that must pass before integration results are accepted:

```rust
pub struct QualityGate {
    pub checks: Vec<QualityCheck>,
    pub fail_mode: FailMode,  // Warn | Error
}

pub enum QualityCheck {
    /// Minimum match rate between real and synthetic
    MinMatchRate { threshold: f64 },          // e.g., 0.80

    /// Maximum percentage of coercion warnings
    MaxCoercionWarnings { threshold: f64 },   // e.g., 0.05

    /// Schema mapping coverage (% of real columns mapped)
    MinMappingCoverage { threshold: f64 },    // e.g., 0.90

    /// Minimum records processed
    MinRecordCount { threshold: usize },

    /// Maximum risk score (for "known good" baselines)
    MaxRiskScore { threshold: f64 },          // e.g., for regression testing

    /// Required evidence types present
    RequiredEvidence { types: Vec<EvidenceType> },
}

impl QualityGate {
    pub fn evaluate(
        &self,
        alignment: &AlignmentReport,
        delta: &KnowledgeDelta,
        evidence: &[AuditEvidence],
    ) -> QualityGateResult;
}

pub struct QualityGateResult {
    pub passed: bool,
    pub check_results: Vec<(QualityCheck, bool, String)>,  // (check, passed, message)
}
```

### 7.5 Integration with Existing Eval Module

**File:** `crates/datasynth-integration/src/workflow/eval_bridge.rs`

Bridge to `datasynth-eval` for running the comprehensive evaluation on blended/augmented datasets:

```rust
pub struct EvalBridge;

impl EvalBridge {
    /// Run standard DataSynth evaluation on the blended dataset
    /// to verify it still meets quality thresholds
    pub fn evaluate_blended(
        dataset: &BlendedDataset,
    ) -> IntegrationResult<ComprehensiveEvaluation>;

    /// Compare evaluation scores: synthetic-only vs. blended
    pub fn compare_evaluations(
        synthetic_eval: &ComprehensiveEvaluation,
        blended_eval: &ComprehensiveEvaluation,
    ) -> EvalComparison;
}
```

### 7.6 Tasks

| # | Task | Files | Test |
|---|------|-------|------|
| 7.1 | Implement `AuditWorkflow` orchestrator | `src/workflow/mod.rs` | Multi-phase workflow runs end-to-end |
| 7.2 | Implement deviation-weighted sampling | `src/workflow/sampling.rs` | High-deviation records selected preferentially |
| 7.3 | Implement `ContinuousMonitor` | `src/workflow/continuous.rs` | Incremental analysis detects new findings |
| 7.4 | Implement trend analysis | `src/workflow/continuous.rs` | Trend direction computed from finding history |
| 7.5 | Implement quality gates | `src/workflow/quality_gate.rs` | Gate fails when match rate below threshold |
| 7.6 | Implement eval bridge | `src/workflow/eval_bridge.rs` | Blended dataset passes standard eval |
| 7.7 | End-to-end integration test: audit workflow | `tests/workflow_integration.rs` | Planning → substantive → detail → completion |
| 7.8 | End-to-end integration test: continuous monitoring | `tests/continuous_integration.rs` | Two incremental runs, trend computed |

**Exit Criteria:** Full audit workflow runs end-to-end (planning through completion), producing workpapers with deviation-weighted samples. Continuous monitoring detects new findings incrementally. Quality gates enforce configurable thresholds. All tests pass.

---

## Phase & Dependency Graph

```
Phase 0: Foundation (core types, config, crate scaffold)
   │
   ├──→ Phase 1: Data Sources & Schema Harmonization
   │       │
   │       └──→ Phase 2: Record Alignment & Provenance
   │               │
   │               ├──→ Phase 3: Gap Analysis Engine
   │               │       │
   │               │       └──→ Phase 5: Audit Evidence & Reporting
   │               │               │
   │               │               └─────┐
   │               │                     │
   │               ├──→ Phase 4: Blending (Overlay & Augmentation)
   │               │       │             │
   │               │       └─────────────┤
   │               │                     │
   │               └─────────────────────┤
   │                                     │
   │                                     ▼
   │                              Phase 6: CLI, Server, Python SDK
   │                                     │
   │                                     ▼
   │                              Phase 7: Workflows & Quality Gates
   │
   └──→ (Phase 3 and 4 can be developed in parallel after Phase 2)
```

**Critical path:** Phase 0 → 1 → 2 → 3 → 5 → 6 → 7

**Parallel opportunities:**
- Phase 3 (Gap Analysis) and Phase 4 (Blending) can be developed in parallel after Phase 2
- Phase 5 depends only on Phase 3; Phase 4 is independent of Phase 5
- Within Phase 1, data sources (1.2–1.5) can be developed in parallel
- Within Phase 3, the three layer analyzers (3.2–3.4) can be developed in parallel

---

## Cross-Crate Modifications Summary

Most work happens in the new `datasynth-integration` crate, but some existing crates need modifications:

| Crate | Modification | Phase |
|-------|-------------|-------|
| **Root `Cargo.toml`** | Add `datasynth-integration` to workspace members | 0 |
| **`datasynth-config`** | Add `integration` section to `GeneratorConfig` schema | 0 |
| **`datasynth-cli`** | Add `Integrate` subcommand with sub-subcommands | 6 |
| **`datasynth-server`** | Add `/api/integration/*` routes and `/ws/integration/*` WebSocket | 6 |
| **`python/datasynth_py`** | Add `Integration` class and result types | 6 |
| **`datasynth-eval`** | No modification — used as dependency only | — |
| **`datasynth-fingerprint`** | No modification — used as dependency only | — |
| **`datasynth-core`** | No modification — types and traits used as-is | — |
| **`datasynth-generators`** | No modification — anomaly injector used via bridge | — |

**Principle:** Minimize changes to existing crates. The integration crate depends on them; they should not depend on it (no circular dependencies).

---

## Risk Matrix

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| **Schema auto-detection accuracy too low** | Users get wrong mappings, bad analysis | Medium | Start with template-based mapping (SAP FI); auto-detect is a bonus. Quality gate catches low mapping coverage. |
| **Fuzzy matching performance on large datasets** | Slow alignment, O(n²) complexity | Medium | Implement blocking strategy early. Set maximum dataset size for fuzzy matching; recommend key-based matching for large datasets. |
| **Statistical tests produce false positives** | Users distrust gap analysis | Medium | Calibrate thresholds conservatively. Use Bonferroni correction for multiple comparisons. Allow per-check threshold overrides in config. |
| **Real data variety exceeds template coverage** | Users with non-SAP ERPs can't onboard easily | High | Prioritize auto-detect + manual YAML mapping as fallback. SAP/Oracle/NetSuite templates cover ~70% of market. |
| **Cross-layer pattern detection too noisy** | Too many low-confidence compound findings | Medium | Require minimum confidence threshold for cross-layer findings. Start with conservative patterns (full-stack only). |
| **HTML report rendering issues** | Reports look broken in some browsers | Low | Use minimal CSS, inline everything, test in Chrome/Firefox/Safari. Follow same approach as existing eval HTML reports. |
| **Parquet/Arrow dependency bloat** | Binary size increases significantly | Low | Feature-gate Parquet support. CSV and JSON are the default; Parquet is opt-in. |
| **Integration config schema too complex** | Users overwhelmed by configuration options | Medium | Provide sensible defaults for everything. The `inspect` command helps users understand their data before configuring. CLI flags override YAML config. |

---

## Testing Strategy

### Unit Tests (per module)

Every module has co-located unit tests (`#[cfg(test)]` modules) covering:
- Core logic (type coercion, key matching, deviation scoring, severity classification)
- Edge cases (empty inputs, single records, all-null columns)
- Error handling (malformed data, missing columns, type mismatches)

### Integration Tests (per phase)

Each phase has a dedicated integration test file in `crates/datasynth-integration/tests/`:

| Test File | What It Tests | Fixture Data |
|-----------|---------------|--------------|
| `harmonization_integration.rs` | CSV → harmonize → model-aligned records | SAP-like CSV fixtures |
| `alignment_integration.rs` | Harmonized data → align → provenance-tagged | Pre-harmonized real + synthetic |
| `gap_analysis_integration.rs` | Aligned data → gap analysis → KnowledgeDelta | Data with planted anomalies |
| `overlay_integration.rs` | Full overlay pipeline with deviation detection | Real + synthetic with known differences |
| `augmentation_integration.rs` | Full augmentation with all strategies | Sparse real data + synthetic fill |
| `reporting_integration.rs` | KnowledgeDelta → evidence → HTML/JSON reports | Pre-computed KnowledgeDelta |
| `workflow_integration.rs` | End-to-end audit workflow | Full fixture dataset |
| `continuous_integration.rs` | Incremental monitoring with trend detection | Two time-sliced datasets |

### CLI Tests

End-to-end CLI tests in `crates/datasynth-cli/tests/`:

```rust
#[test]
fn test_integrate_gap_analysis_cli() {
    let output = Command::new(cargo_bin("datasynth-data"))
        .args(&["integrate", "gap-analysis",
                "--real", "tests/fixtures/client_data/",
                "--baseline", "tests/fixtures/synthetic_baseline/",
                "--mapping", "sap_fi",
                "--output", &tempdir.path().to_string_lossy()])
        .output()
        .expect("CLI execution failed");

    assert!(output.status.success());
    assert!(tempdir.path().join("report.html").exists());
    assert!(tempdir.path().join("report.json").exists());
}
```

### Fixture Data Generation

Create a dedicated fixture generation script that uses DataSynth itself to produce the synthetic baseline, then manually creates "real" CSV files with known differences:

```bash
# Generate synthetic baseline for fixtures
datasynth-data generate --demo --output tests/fixtures/synthetic_baseline/

# Real data fixtures are hand-crafted CSV files with known:
# - Missing accounts (structural gap)
# - Benford-violating amounts (statistical gap)
# - SoD violations (normative gap)
# - Broken document chains (structural gap)
# - Threshold-proximate amounts (cross-layer pattern)
```

---

## Milestone Summary

| Milestone | Phase(s) | Deliverable | Key Metric |
|-----------|----------|-------------|------------|
| **M0: Scaffold** | 0 | Crate compiles, types defined, config parses | `cargo check` passes |
| **M1: Read Real Data** | 1 | CSV/JSON ingested and harmonized to DataSynth schema | 3+ source formats, SAP FI template working |
| **M2: Compare Data** | 2 | Records aligned, provenance tracked, deviations scored | Match rate > 80% on fixture data |
| **M3: Find Gaps** | 3 | Three-layer gap analysis produces KnowledgeDelta | All planted anomalies detected in tests |
| **M4: Blend Data** | 4 | Overlay and augmentation produce provenance-tagged datasets | 100% records have provenance tags |
| **M5: Generate Evidence** | 5 | Audit evidence with ISA references, HTML workpapers | Self-contained HTML report renders correctly |
| **M6: User-Facing** | 6 | CLI, REST API, and Python SDK fully functional | `datasynth-data integrate gap-analysis` works end-to-end |
| **M7: Production Workflows** | 7 | Audit workflow orchestration, continuous monitoring, quality gates | Full audit workflow produces workpapers |

---

## Appendix A: Complete File Manifest

```
crates/datasynth-integration/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── types.rs
│   ├── error.rs
│   ├── config.rs
│   ├── sources/
│   │   ├── mod.rs                     # DataSourceReader trait
│   │   ├── csv_source.rs             # CsvDirectorySource
│   │   ├── json_source.rs            # JsonDirectorySource
│   │   ├── parquet_source.rs         # ParquetDirectorySource (feature-gated)
│   │   └── memory_source.rs          # MemorySource (testing + API)
│   ├── harmonizer/
│   │   ├── mod.rs                     # SchemaHarmonizer
│   │   ├── auto_detect.rs            # Column name/pattern matching
│   │   ├── templates.rs              # ERP template loader
│   │   ├── coercion.rs               # Type coercion rules
│   │   └── templates/
│   │       ├── sap_fi.yaml
│   │       ├── sap_mm.yaml
│   │       ├── sap_sd.yaml
│   │       ├── oracle_gl.yaml
│   │       └── netsuite.yaml
│   ├── aligner/
│   │   ├── mod.rs                     # RecordAligner, AlignmentReport
│   │   ├── key_matcher.rs            # Exact composite key matching
│   │   ├── fuzzy_matcher.rs          # Levenshtein/JaroWinkler fuzzy matching
│   │   ├── chain_walker.rs           # Document chain alignment (P2P, O2C)
│   │   └── deviation.rs              # Field-level deviation scoring
│   ├── gap_analysis/
│   │   ├── mod.rs                     # GapAnalyzer, KnowledgeDelta
│   │   ├── structural.rs             # Structural gap checks (K_S)
│   │   ├── statistical.rs            # Statistical gap checks (K_Σ)
│   │   ├── normative.rs              # Normative gap checks (K_N)
│   │   ├── cross_layer.rs            # Cross-layer pattern detection
│   │   ├── scoring.rs                # Risk scoring model
│   │   └── severity.rs               # Severity classification
│   ├── blending/
│   │   ├── mod.rs                     # BlendedDataset, BlendedRecord
│   │   ├── overlay.rs                # OverlayEngine
│   │   ├── augmentation.rs           # AugmentationEngine
│   │   ├── anomaly_bridge.rs         # Bridge to existing anomaly injector
│   │   ├── provenance.rs             # ProvenanceTracker
│   │   └── export.rs                 # CSV/JSON export with provenance
│   ├── evidence/
│   │   ├── mod.rs                     # EvidenceGenerator, AuditEvidence
│   │   ├── analytical.rs             # ISA 520 analytical procedure evidence
│   │   ├── detail_testing.rs         # ISA 500 detail testing evidence
│   │   ├── control_testing.rs        # ISA 330 control testing evidence
│   │   ├── chain.rs                  # Evidence chain construction
│   │   └── exhibits.rs               # Data exhibits (tables, heatmaps)
│   ├── report/
│   │   ├── mod.rs
│   │   ├── html_report.rs            # Self-contained HTML report generator
│   │   ├── json_report.rs            # Structured JSON report
│   │   └── workpaper.rs              # Audit workpaper directory generator
│   └── workflow/
│       ├── mod.rs                     # AuditWorkflow orchestrator
│       ├── sampling.rs               # Deviation-weighted audit sampling
│       ├── continuous.rs             # ContinuousMonitor for internal audit
│       ├── quality_gate.rs           # Configurable quality gates
│       └── eval_bridge.rs            # Bridge to datasynth-eval
├── tests/
│   ├── fixtures/
│   │   ├── sap_bkpf.csv
│   │   ├── sap_bseg.csv
│   │   ├── simple_gl.json
│   │   └── synthetic_baseline/       # Generated via datasynth --demo
│   ├── harmonization_integration.rs
│   ├── alignment_integration.rs
│   ├── gap_analysis_integration.rs
│   ├── overlay_integration.rs
│   ├── augmentation_integration.rs
│   ├── reporting_integration.rs
│   ├── workflow_integration.rs
│   └── continuous_integration.rs
```

**Files modified in other crates:**

```
Cargo.toml                                          # Add workspace member
crates/datasynth-config/src/schema.rs               # Add integration config section
crates/datasynth-cli/src/main.rs                    # Add Integrate subcommand
crates/datasynth-cli/src/integrate.rs               # NEW: CLI handlers
crates/datasynth-server/src/routes/integration.rs   # NEW: REST endpoints
crates/datasynth-server/src/routes/mod.rs           # Register integration routes
crates/datasynth-server/src/ws/integration.rs       # NEW: WebSocket streaming
python/datasynth_py/__init__.py                     # Add integration property
python/datasynth_py/integration.py                  # NEW: Python bindings
python/tests/test_integration.py                    # NEW: Python tests
```

**Total new files:** ~50
**Total modified files:** ~6

---

## Appendix B: Task Count Summary

| Phase | Task Count | New Files | Modified Files |
|-------|-----------|-----------|----------------|
| Phase 0 | 6 | ~8 | 2 |
| Phase 1 | 10 | ~12 | 0 |
| Phase 2 | 7 | ~6 | 0 |
| Phase 3 | 13 | ~8 | 0 |
| Phase 4 | 9 | ~6 | 0 |
| Phase 5 | 10 | ~8 | 0 |
| Phase 6 | 12 | ~8 | 4 |
| Phase 7 | 8 | ~6 | 0 |
| **Total** | **75** | **~62** | **~6** |

---

*This implementation plan derives from the architecture spec at `docs/specs/real-synthetic-integration-spec.md`. Each phase is independently testable and shippable, building incrementally toward the full integration capability.*
