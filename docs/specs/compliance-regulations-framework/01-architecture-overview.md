# Part 1: Architecture Overview & Design Principles

> **Parent:** [Compliance & Regulations Framework](00-index.md)
> **Status:** Draft | **Date:** 2026-03-09

---

## 1.1 Executive Summary

The Compliance & Regulations Framework introduces a **unified regulatory abstraction layer** that sits between DataSynth's configuration system and its generation/graph pipelines. Instead of individual generators knowing about specific standards, all compliance knowledge flows through a registry-based architecture where:

1. **Standards are data, not code** — Each standard is a registry entry with metadata, versions, and cross-references
2. **Countries compose standards** — Jurisdiction profiles declare which standards apply, with what local overrides
3. **Time flows through everything** — Every artifact carries temporal context; generation targets a specific date range
4. **Graphs connect compliance** — Every compliance artifact participates in a typed graph with semantic edges
5. **Templates make it extensible** — Users define custom audit procedures without writing Rust code

---

## 1.2 Core Abstractions

### 1.2.1 StandardId — Canonical Standard Identifier

```rust
/// Canonical identifier for a compliance standard.
/// Format: "{BODY}-{NUMBER}" (e.g., "IFRS-16", "ISA-315", "SOX-404", "ASC-606")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StandardId(pub String);

impl StandardId {
    pub fn new(body: &str, number: &str) -> Self {
        Self(format!("{body}-{number}"))
    }

    /// Returns the issuing body (e.g., "IFRS", "ISA", "SOX", "ASC")
    pub fn body(&self) -> &str {
        self.0.split('-').next().unwrap_or("")
    }

    /// Returns the standard number (e.g., "16", "315", "404", "606")
    pub fn number(&self) -> &str {
        self.0.split('-').nth(1).unwrap_or("")
    }
}
```

### 1.2.2 ComplianceStandard — Standard Metadata

```rust
/// A compliance standard with full metadata.
pub struct ComplianceStandard {
    /// Canonical identifier
    pub id: StandardId,
    /// Human-readable title
    pub title: String,
    /// Issuing body (IASB, IAASB, SEC, PCAOB, FASB, etc.)
    pub issuing_body: IssuingBody,
    /// Standard category
    pub category: StandardCategory,
    /// Domain area
    pub domain: ComplianceDomain,
    /// All known versions with effective dates
    pub versions: Vec<TemporalVersion>,
    /// Standards this one supersedes
    pub supersedes: Vec<StandardId>,
    /// Standards this one is superseded by
    pub superseded_by: Option<StandardId>,
    /// Cross-references to related standards
    pub cross_references: Vec<CrossReference>,
    /// Jurisdictions where this standard is mandatory
    pub mandatory_jurisdictions: Vec<JurisdictionCode>,
    /// Jurisdictions where this standard is permitted but not required
    pub permitted_jurisdictions: Vec<JurisdictionCode>,
    /// Key requirements that map to audit assertions
    pub requirements: Vec<StandardRequirement>,
}

/// Category of compliance standard.
pub enum StandardCategory {
    AccountingStandard,     // IFRS, US GAAP, local GAAP
    AuditingStandard,       // ISA, PCAOB AS
    RegulatoryRequirement,  // SOX, EU Audit Regulation, MiFID II
    ReportingStandard,      // XBRL, ESEF, iXBRL
    PrudentialRegulation,   // Basel III/IV, Solvency II
    TaxRegulation,          // BEPS, CRS, FATCA
    DataProtection,         // GDPR, CCPA
    SustainabilityStandard, // CSRD, ISSB, GRI
}

/// Domain within financial compliance.
pub enum ComplianceDomain {
    FinancialReporting,
    InternalControl,
    ExternalAudit,
    TaxCompliance,
    RegulatoryReporting,
    RiskManagement,
    DataGovernance,
    Sustainability,
    AntiMoneyLaundering,
    PrudentialCapital,
}
```

### 1.2.3 TemporalVersion — Time-Aware Versioning

```rust
/// A specific version of a standard with temporal bounds.
pub struct TemporalVersion {
    /// Version identifier (e.g., "2018", "2020-amended", "2023-revised")
    pub version_id: String,
    /// Date this version becomes effective
    pub effective_from: NaiveDate,
    /// Date this version is superseded (None = currently active)
    pub superseded_at: Option<NaiveDate>,
    /// Transition period end (for early adoption)
    pub early_adoption_from: Option<NaiveDate>,
    /// Key changes from the previous version
    pub change_summary: Vec<String>,
    /// Impact level on generated data
    pub impact: ChangeImpact,
}

pub enum ChangeImpact {
    /// Cosmetic/disclosure-only changes
    Low,
    /// Changes to recognition or measurement
    Medium,
    /// Fundamental restructuring (e.g., IAS 39 → IFRS 9)
    High,
    /// Complete replacement of a standard
    Replacement,
}
```

### 1.2.4 JurisdictionProfile — Country Compliance Composition

```rust
/// A jurisdiction's complete compliance profile.
pub struct JurisdictionProfile {
    /// ISO 3166-1 alpha-2 country code
    pub country_code: String,
    /// Jurisdiction display name
    pub name: String,
    /// Supranational memberships (EU, EEA, ASEAN, etc.)
    pub memberships: Vec<SupranationalBody>,
    /// Applicable accounting framework
    pub accounting_framework: AccountingFramework,
    /// Local GAAP standard (if different from IFRS)
    pub local_gaap: Option<StandardId>,
    /// Mandatory standards with local effective dates
    pub mandatory_standards: Vec<JurisdictionStandard>,
    /// Local regulatory requirements
    pub local_regulations: Vec<LocalRegulation>,
    /// Audit framework (ISA-based, PCAOB, or local)
    pub audit_framework: AuditFramework,
    /// Required reporting formats
    pub reporting_formats: Vec<ReportingFormat>,
    /// Filing requirements and deadlines
    pub filing_requirements: Vec<FilingRequirement>,
    /// Local tax compliance rules
    pub tax_rules: Vec<TaxRule>,
    /// Currency and locale settings
    pub locale: JurisdictionLocale,
}

/// How a standard applies in a specific jurisdiction.
pub struct JurisdictionStandard {
    pub standard_id: StandardId,
    /// Local effective date (may differ from global)
    pub local_effective_date: NaiveDate,
    /// Local standard name/number (e.g., "Ind AS 116" for IFRS 16 in India)
    pub local_designation: Option<String>,
    /// Local modifications or carve-outs
    pub modifications: Vec<LocalModification>,
    /// Applicability criteria (e.g., "public interest entities only")
    pub applicability: ApplicabilityCriteria,
}
```

---

## 1.3 Architecture Layers

The framework operates in four layers:

### Layer 1: Standard Registry (datasynth-core)

The **Standard Registry** is the source of truth for all compliance knowledge. It is a static, in-memory catalog built at compile time from embedded data and extended at runtime from user-provided YAML/JSON files.

```
StandardRegistry
├── standards: HashMap<StandardId, ComplianceStandard>
├── jurisdictions: HashMap<String, JurisdictionProfile>
├── cross_reference_index: HashMap<StandardId, Vec<StandardId>>
├── supersession_chains: HashMap<StandardId, Vec<StandardId>>
└── temporal_index: BTreeMap<NaiveDate, Vec<StandardEvent>>
```

**Responsibilities:**
- Resolve which version of a standard is active at a given date
- Look up cross-references (e.g., ISA 315 ↔ SOX 404 ↔ COSO)
- Determine supersession chains (IAS 39 → IFRS 9)
- Query by jurisdiction (what standards apply in Germany on 2025-01-01?)

### Layer 2: Compliance Generators (datasynth-generators)

Generators consume registry data to produce compliance artifacts:

| Generator | Input | Output |
|-----------|-------|--------|
| `RegulationGenerator` | Registry + config | `Regulation` records with version-specific requirements |
| `ComplianceProcedureGenerator` | Templates + standards | Audit procedures mapped to assertions and standards |
| `ComplianceFindingGenerator` | Procedures + anomalies | Findings with severity, remediation, and standard references |
| `ComplianceAssertionGenerator` | Standards + accounts | Assertions (existence, completeness, valuation, etc.) per account |
| `FilingGenerator` | Jurisdiction profile + data | Regulatory filing records (10-K, 20-F, annual return, etc.) |
| `ComplianceTestGenerator` | Controls + procedures | Test-of-controls and substantive test results |

### Layer 3: Graph Builder (datasynth-graph)

The **Compliance Graph Builder** constructs a heterogeneous graph from compliance artifacts:

```
ComplianceGraphBuilder
├── add_standard_nodes()      → Standard, Regulation, Requirement nodes
├── add_control_edges()       → Control ──maps_to──▶ Standard
├── add_procedure_edges()     → Procedure ──tests──▶ Control
├── add_assertion_edges()     → Assertion ──covers──▶ Account
├── add_finding_edges()       → Finding ──identified_by──▶ Procedure
├── add_jurisdiction_edges()  → Entity ──subject_to──▶ Jurisdiction
└── add_temporal_edges()      → Standard(v1) ──supersedes──▶ Standard(v2)
```

### Layer 4: Runtime Orchestration (datasynth-runtime)

The **ComplianceOrchestrator** coordinates the end-to-end flow:

```
ComplianceOrchestrator::generate(config, registry)
  1. Resolve jurisdiction profiles for all companies
  2. Determine applicable standards at target date
  3. Generate regulation records
  4. Execute audit procedure templates
  5. Generate compliance assertions
  6. Run compliance test generation
  7. Produce findings and deficiency classifications
  8. Wire all artifacts into the graph builder
  9. Export compliance-specific output files
```

---

## 1.4 Design Principles

### Principle 1: Separation of Knowledge and Logic

Compliance knowledge (what standards exist, what they require) is separated from generation logic (how to produce synthetic data that embodies those requirements). Knowledge lives in the registry and templates; logic lives in generators.

**Implication:** Adding a new standard means adding a registry entry and optionally a template — not writing new Rust code for common patterns.

### Principle 2: Compositional Jurisdiction Profiles

A country's compliance posture is a composition, not a monolith. Germany's profile composes: HGB (local GAAP) + IFRS (for listed entities) + ISA (audit) + EU Audit Regulation + HGB audit requirements + GoBD (tax) + CSRD (sustainability). Any piece can be independently updated.

**Implication:** Adding Japan support means composing J-GAAP + Company Act + JICPA standards, reusing the ISA base and adding local overrides.

### Principle 3: Temporal Determinism

Given the same seed and target date, the framework must produce identical output. Temporal versioning is deterministic: the registry resolves which standard version applies at date `T`, and generators use that version consistently.

**Implication:** Generating data for "2020-06-30" produces pre-IFRS 17 output; "2023-06-30" produces post-IFRS 17 output. The seed controls randomness; the date controls regulatory state.

### Principle 4: Graph-Native Compliance

Every compliance artifact is a first-class graph citizen. Controls, procedures, findings, and standards are nodes; their relationships are typed edges. This enables:
- **Compliance coverage analysis** via graph traversal
- **Anomaly detection** where missing edges (untested controls) are the signal
- **ML feature engineering** using graph embeddings that capture compliance structure

### Principle 5: Progressive Disclosure

The framework has three tiers of engagement:

| Tier | User | Configuration |
|------|------|--------------|
| **Simple** | "I want SOX-compliant data" | `compliance: { preset: sox_us }` |
| **Custom** | "I want ISA + local German requirements" | Jurisdiction profile + standard selection |
| **Expert** | "I need custom audit procedures for revenue recognition" | YAML template DSL |

---

## 1.5 Data Flow

```
                    ┌──────────────┐
                    │   Config     │
                    │  (YAML)      │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │  Registry    │◄─── Built-in standards
                    │  Resolution  │◄─── User-defined standards (YAML)
                    └──────┬───────┘
                           │
              ┌────────────▼────────────┐
              │  Jurisdiction Resolver   │
              │  For each company:       │
              │  - country → profile     │
              │  - date → active stds    │
              └────────────┬────────────┘
                           │
         ┌─────────────────▼─────────────────┐
         │       Compliance Generators        │
         │                                    │
         │  ┌──────────┐ ┌──────────────────┐ │
         │  │Regulation│ │ Audit Procedure  │ │
         │  │Generator │ │ Template Engine  │ │
         │  └────┬─────┘ └───────┬──────────┘ │
         │       │               │             │
         │  ┌────▼─────┐ ┌──────▼───────────┐ │
         │  │Assertion │ │ Finding          │ │
         │  │Generator │ │ Generator        │ │
         │  └────┬─────┘ └───────┬──────────┘ │
         │       │               │             │
         │  ┌────▼─────┐ ┌──────▼───────────┐ │
         │  │Filing    │ │ Compliance Test  │ │
         │  │Generator │ │ Generator        │ │
         │  └────┬─────┘ └───────┬──────────┘ │
         └───────┼───────────────┼────────────┘
                 │               │
         ┌───────▼───────────────▼────────────┐
         │        Compliance Graph Builder     │
         │  Nodes: Standard, Control, Finding  │
         │  Edges: maps_to, tests, covers      │
         └───────────────────┬────────────────┘
                             │
                    ┌────────▼────────┐
                    │   Output Sinks   │
                    │  CSV/JSON/Parquet│
                    │  + Graph formats │
                    └─────────────────┘
```

---

## 1.6 Integration Points with Existing Crates

### datasynth-core

**New models in `models/compliance/`:**
- `StandardId`, `ComplianceStandard`, `TemporalVersion`
- `JurisdictionProfile`, `JurisdictionStandard`
- `ComplianceAssertion`, `ComplianceFinding`
- `RegulatoryFiling`, `FilingRequirement`
- `AuditProcedureTemplate`, `ProcedureStep`

**Extended models:**
- `InternalControl` gains `applicable_standards: Vec<StandardId>`
- `ControlMapping` gains `standard_version: Option<String>`
- `AuditEngagement` gains `jurisdiction_profile: JurisdictionCode`

### datasynth-standards

**New modules:**
- `registry.rs` — Standard registry with temporal resolution
- `jurisdiction.rs` — Jurisdiction profile loader and resolver
- `templates/` — Audit procedure template engine
- `regulatory/` extended with Basel, CSRD, MiFID II, AML directives

**Extended modules:**
- `accounting/` — Version-aware generation respecting temporal bounds
- `audit/` — Template-driven procedure generation
- `regulatory/sox.rs` — Enhanced with PCAOB AS 2201 integration

### datasynth-generators

**New module: `compliance/`**
- `regulation_generator.rs`
- `procedure_generator.rs`
- `finding_generator.rs`
- `assertion_generator.rs`
- `filing_generator.rs`
- `compliance_test_generator.rs`
- `country_orchestrator.rs`

### datasynth-graph

**New node types:**
- `NodeType::Standard` — Compliance standard
- `NodeType::Regulation` — Regulatory requirement
- `NodeType::AuditProcedure` — Audit procedure instance
- `NodeType::ComplianceFinding` — Audit finding
- `NodeType::Jurisdiction` — Legal jurisdiction
- `NodeType::Assertion` — Audit assertion

**New edge types:**
- `EdgeType::MapsTo` — Control → Standard
- `EdgeType::Tests` — Procedure → Control
- `EdgeType::Covers` — Assertion → Account
- `EdgeType::IdentifiedBy` — Finding → Procedure
- `EdgeType::SubjectTo` — Entity → Jurisdiction
- `EdgeType::Supersedes` — Standard(v2) → Standard(v1)
- `EdgeType::RequiredBy` — Standard → Regulation
- `EdgeType::CrossReferences` — Standard ↔ Standard

### datasynth-config

**New configuration section:**
```yaml
compliance_regulations:
  enabled: true
  # ... (see Part 7 for full schema)
```

### datasynth-runtime

**New orchestrator:** `ComplianceOrchestrator` integrates into the existing `GenerationOrchestrator` pipeline, running after core generators and before graph export.

---

## 1.7 Non-Goals (v1.1.0)

To keep scope manageable, the following are explicitly deferred:

| Non-Goal | Reason | Future Version |
|----------|--------|----------------|
| Real-time regulatory feed integration | Requires external API subscriptions | v1.3+ |
| Natural language regulation text generation | Requires LLM integration | v1.2+ |
| Automated compliance gap analysis | Requires ML inference at generation time | v1.2+ |
| Cross-border transfer pricing scenarios | Complex; needs dedicated module | v1.2+ |
| XBRL/iXBRL taxonomy generation | Specialized format; low initial demand | v1.3+ |
