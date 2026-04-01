# Generic Process Blueprint Engine

**Date**: 2026-04-01
**Status**: Draft
**Scope**: Extract a domain-agnostic process FSM engine from `datasynth-audit-fsm`, enabling custom YAML-defined business processes with an artifact registry pattern and process mining readiness.

## Problem

DataSynth currently has two fundamentally different approaches to process generation:

1. **Hardcoded transactional processes** (P2P, O2C, manufacturing, HR) — state progression baked into Rust generator code. Changing the flow (adding an approval gate, inserting a quality step, reordering phases) requires Rust code changes.
2. **YAML-driven audit processes** (`datasynth-audit-fsm`) — explicit state machines defined in YAML blueprints with configurable phases, procedures, transitions, guards, and event trails.

This split creates several problems:

- **No customization** — enterprises can't model their actual processes without forking the codebase
- **No process mining feedback loop** — captured real-world process variants can't feed back into generation
- **Duplicated concepts** — both approaches implement state progression, event emission, and artifact generation independently
- **Rigid transactional flows** — a three-way match is always a three-way match; there's no way to express a four-way match with quality inspection, or a two-way match for low-value POs

The audit FSM crate already solved this for audit workflows. The engine, loader, validator, and export infrastructure are genuinely domain-agnostic — the audit-specific parts are limited to command dispatch and engagement context.

## Solution

Extract the generic FSM engine into a new `datasynth-process-engine` crate. Refactor `datasynth-audit-fsm` to become a thin domain layer on top. Introduce an artifact registry with a trait-based plugin interface and a schema-driven fallback generator for unknown artifact types.

### Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Artifact generation | Hybrid registry — built-in generators for known types, schema-driven fallback for custom types | Preserves domain richness (Benford compliance, deterministic UUIDs, 50+ field models) while allowing arbitrary custom artifacts |
| Crate structure | Layered — new generic engine crate, audit crate becomes thin domain layer on top | Preserves audit crate stability (10 battle-tested blueprints), clean separation of concerns |
| Process mining | Design schema for ingestion readiness, don't build integration layer | Schema becomes the contract; integration layer (separate repo, future) outputs blueprint + overlay + schemas that DataSynth consumes directly |
| Artifact schema location | Both inline and external file references | Small/one-off artifacts inline in blueprint, complex/shared schemas as separate YAML files |
| Migration of existing generators | Blueprint facades dispatching to existing Rust generators | P2P/O2C/manufacturing expressed as blueprint YAMLs that control flow, existing generators produce data. Validates engine against known-good output |
| External plugin support | Trait-based registry now, design interface for future Python FFI / WASM | YAGNI on plugin runtime, but `ArtifactGenerator` trait is clean enough for adapters later |

## Architecture

### Layer Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      User / Integration Layer                    │
│                                                                  │
│  CLI: datasynth-data generate --blueprint ./my_process.yaml      │
│  Config YAML: process_blueprints: [{ blueprint: ..., overlay: }] │
│  Python: DataSynth().generate(blueprint="./my_process.yaml")     │
│  Future: Process Mining → Blueprint YAML + Overlay + Schemas     │
└──────────────────────────┬──────────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│              datasynth-process-engine (NEW CRATE)                │
│                                                                  │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────────┐  │
│  │  Blueprint   │  │   Loader /   │  │     FSM Engine         │  │
│  │  Schema      │  │  Validator   │  │  Topological walk      │  │
│  │  (phases,    │  │  (DAG, xref, │  │  State transitions     │  │
│  │  procedures, │  │  reachability │  │  Step execution        │  │
│  │  steps,      │  │  checks)     │  │  Event emission        │  │
│  │  artifacts)  │  └──────────────┘  │  Anomaly injection     │  │
│  └─────────────┘                     │  Deterministic (ChaCha8│  │
│                                      └────────────┬───────────┘  │
│  ┌─────────────────────────────┐                  │              │
│  │    Artifact Registry        │◄─────────────────┘              │
│  │  ┌─────────────────────┐   │                                  │
│  │  │ Built-in generators │   │  Trait lookup by artifact_type   │
│  │  │ (registered by      │   │  Known type → domain generator   │
│  │  │  domain crates)     │   │  Unknown type → SchemaGenerator  │
│  │  ├─────────────────────┤   │                                  │
│  │  │ SchemaGenerator     │   │  Reads ArtifactSchema (inline    │
│  │  │ (fallback)          │   │  or external YAML) → produces    │
│  │  │                     │   │  typed records with distributions │
│  │  └─────────────────────┘   │                                  │
│  └─────────────────────────────┘                                 │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  Export: JSON | OCEL 2.0 | CSV | XES | Celonis | Parquet │    │
│  └──────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
         ▲                    ▲                    ▲
         │                    │                    │
┌────────┴───────┐  ┌────────┴───────┐  ┌────────┴───────────────┐
│ datasynth-     │  │ datasynth-     │  │ datasynth-generators   │
│ audit-fsm      │  │ banking        │  │                        │
│                │  │                │  │ Registers:             │
│ Registers:     │  │ Registers:     │  │  purchase_order        │
│  risk_assess   │  │  kyc_check     │  │  goods_receipt         │
│  materiality   │  │  aml_screen    │  │  vendor_invoice        │
│  sampling      │  │  transaction   │  │  payment               │
│  audit_opinion │  │  alert         │  │  sales_order           │
│  ...           │  │  ...           │  │  journal_entry         │
│                │  │                │  │  ...                   │
│ Includes:      │  │ Includes:      │  │                        │
│  10 audit      │  │  KYC/AML       │  │ Includes:              │
│  blueprints    │  │  blueprints    │  │  P2P, O2C, mfg        │
│  7 overlays    │  │                │  │  blueprint facades     │
└────────────────┘  └────────────────┘  └────────────────────────┘
```

### Blueprint Schema

The generic blueprint schema generalizes from the audit-specific schema. All types derive `Debug, Clone, Serialize, Deserialize`.

```rust
/// Root blueprint definition — describes any business process
pub struct ProcessBlueprint {
    // --- Metadata ---
    pub id: String,
    pub name: String,
    pub version: String,
    pub schema_version: String,             // "2.0" for generic process schema
    pub description: Option<String>,

    // --- Process Definition ---
    pub domain: ProcessDomain,              // audit, procurement, sales, banking, custom
    pub methodology: Option<BlueprintMethodology>,  // Optional: ISA, IIA-GIAS, Six Sigma, etc.
    pub depth: DepthLevel,                  // simplified | standard | full

    // --- Filtering ---
    pub discriminators: HashMap<String, Vec<String>>,

    // --- Catalogs ---
    pub actors: Vec<ProcessActor>,
    pub artifact_schemas: Vec<ArtifactSchema>,      // NEW: typed record definitions
    pub evidence_templates: Vec<EvidenceTemplate>,   // Reusable evidence definitions
    pub standards: Vec<StandardReference>,            // Optional: regulatory references
    pub external_schemas: Vec<ExternalSchemaRef>,     // NEW: references to external YAML files

    // --- Process Structure ---
    pub phases: Vec<ProcessPhase>,
}

pub enum ProcessDomain {
    Audit,
    Procurement,
    Sales,
    Manufacturing,
    Banking,
    HumanResources,
    Custom(String),
}

pub struct ProcessPhase {
    pub id: String,
    pub name: String,
    pub order: i32,
    pub description: Option<String>,
    pub gate: Option<PhaseGate>,
    pub procedures: Vec<ProcessProcedure>,
}

pub struct ProcessProcedure {
    pub id: String,
    pub title: String,
    pub discriminators: Option<HashMap<String, Vec<String>>>,

    // FSM definition
    pub aggregate: ProcedureAggregate,

    // Work steps
    pub steps: Vec<ProcessStep>,

    // DAG ordering
    pub preconditions: Vec<String>,
}

pub struct ProcedureAggregate {
    pub initial_state: String,
    pub states: Vec<String>,
    pub transitions: Vec<StateTransition>,
}

pub struct StateTransition {
    pub from_state: String,
    pub to_state: String,
    pub command: String,
    pub emits: String,
    pub guards: Vec<String>,
}

pub struct ProcessStep {
    pub id: String,
    pub order: i32,
    pub action: String,                     // Generic action verb (not enum-restricted)
    pub actor: String,
    pub description: String,
    pub binding: Option<BindingLevel>,       // requirement | guidance | optional

    pub command: String,
    pub emits: String,

    // Artifact production
    pub artifact_type: Option<String>,       // NEW: registry lookup key
    pub artifact_overrides: Option<HashMap<String, serde_yaml::Value>>,  // Field overrides

    // Evidence traceability
    pub evidence: Option<EvidenceSpec>,

    // Standards mapping (optional — not all processes are regulated)
    pub standards: Vec<StandardMapping>,

    // Conditional branching
    pub decisions: Vec<Decision>,

    // Guards
    pub guards: Vec<StepGuard>,
}
```

### Artifact Registry

```rust
/// Trait for domain-specific artifact generators
pub trait ArtifactGenerator: Send + Sync {
    /// The artifact type name this generator handles (e.g., "purchase_order")
    fn artifact_type(&self) -> &str;

    /// Generate one or more records for a step execution
    fn generate(
        &self,
        context: &StepContext,
        schema: Option<&ArtifactSchema>,
        rng: &mut ChaCha8Rng,
    ) -> Result<Vec<GeneratedRecord>, SynthError>;

    /// Describe the fields this generator produces (for introspection/validation)
    fn output_fields(&self) -> Vec<FieldDescriptor>;
}

/// Registry that resolves artifact_type → generator
pub struct ArtifactRegistry {
    generators: HashMap<String, Box<dyn ArtifactGenerator>>,
    schema_generator: SchemaGenerator,  // Fallback for unknown types
}

impl ArtifactRegistry {
    pub fn new() -> Self { ... }

    /// Register a domain-specific generator
    pub fn register(&mut self, generator: Box<dyn ArtifactGenerator>) { ... }

    /// Resolve and generate — falls back to SchemaGenerator if no match
    pub fn generate(
        &self,
        artifact_type: &str,
        context: &StepContext,
        schema: Option<&ArtifactSchema>,
        rng: &mut ChaCha8Rng,
    ) -> Result<Vec<GeneratedRecord>, SynthError> {
        if let Some(gen) = self.generators.get(artifact_type) {
            gen.generate(context, schema, rng)
        } else if let Some(schema) = schema {
            self.schema_generator.generate(context, schema, rng)
        } else {
            // No generator and no schema — emit a generic event record
            Ok(vec![GeneratedRecord::event_only(context)])
        }
    }
}
```

### StepContext

The context passed to artifact generators — domain-agnostic but rich enough for domain generators to extract what they need.

```rust
pub struct StepContext {
    // Process identity
    pub case_id: String,                    // Process instance ID
    pub blueprint_id: String,
    pub phase_id: String,
    pub procedure_id: String,
    pub step_id: String,

    // Temporal
    pub timestamp: NaiveDateTime,
    pub fiscal_period: Option<FiscalPeriod>,

    // Actor
    pub actor_id: String,
    pub actor_role: String,

    // Organizational
    pub company_code: String,
    pub entity_code: Option<String>,
    pub department: Option<String>,
    pub currency: String,

    // Process state
    pub procedure_state: String,            // Current FSM state
    pub iteration: u32,                     // Revision loop count
    pub prior_artifacts: Vec<ArtifactRef>,  // Artifacts from earlier steps (for chaining)

    // Domain-specific context (opaque to engine, populated by domain layers)
    pub domain_context: HashMap<String, serde_json::Value>,

    // Upstream data (from orchestrator — CoA, master data, etc.)
    pub shared_data: Arc<SharedGenerationData>,
}
```

### Artifact Schema (for Schema-Driven Generation)

```yaml
# Inline in blueprint YAML under artifact_schemas:
artifact_schemas:
  - id: loan_application
    description: "Consumer loan application"
    fields:
      - name: application_id
        type: string
        generator: uuid
      - name: applicant_name
        type: string
        generator: person_name
      - name: loan_amount
        type: decimal
        distribution:
          type: lognormal
          mu: 10.2
          sigma: 1.4
          min: 1000.0
          max: 500000.0
        benford_compliance: true
      - name: interest_rate
        type: decimal
        distribution:
          type: beta
          alpha: 2.0
          beta: 5.0
          scale: 0.15          # Maps [0,1] → [0, 15%]
      - name: credit_score
        type: integer
        distribution:
          type: normal
          mean: 720
          std: 80
          min: 300
          max: 850
      - name: loan_type
        type: enum
        values: [mortgage, auto, personal, student, business]
        weights: [0.35, 0.25, 0.20, 0.12, 0.08]
      - name: status
        type: enum
        values: [pending, approved, denied, withdrawn]
        weights: [0.30, 0.45, 0.20, 0.05]
      - name: submission_date
        type: date
        generator: step_timestamp           # Inherit from step context
      - name: branch_code
        type: string
        generator: from_context             # Pull from domain_context
        context_key: branch_code

    # Relationships between fields
    constraints:
      - type: conditional
        when: { field: loan_type, equals: mortgage }
        then: { field: loan_amount, distribution: { type: lognormal, mu: 12.5, sigma: 0.6 } }
      - type: correlation
        fields: [loan_amount, credit_score]
        coefficient: -0.3                   # Higher loans → slightly lower credit scores
```

```yaml
# External schema reference in blueprint:
external_schemas:
  - id: insurance_claim
    path: ./schemas/insurance_claim.yaml    # Relative to blueprint file
```

### Rust Types for Schema-Driven Generation

```rust
pub struct ArtifactSchema {
    pub id: String,
    pub description: Option<String>,
    pub fields: Vec<FieldSchema>,
    pub constraints: Vec<FieldConstraint>,
}

pub struct FieldSchema {
    pub name: String,
    pub field_type: FieldType,
    pub generator: Option<FieldGenerator>,
    pub distribution: Option<DistributionSpec>,
    pub nullable: bool,
    pub null_rate: Option<f64>,
}

pub enum FieldType {
    String,
    Integer,
    Decimal,
    Boolean,
    Date,
    DateTime,
    Enum(Vec<EnumVariant>),
}

pub enum FieldGenerator {
    Uuid,
    PersonName,
    CompanyName,
    Address,
    Email,
    Phone,
    StepTimestamp,
    FromContext(String),
    Sequential(String),      // Prefix + counter: "LN-000001"
    Pattern(String),         // Regex-like pattern: "[A-Z]{3}-[0-9]{6}"
    Reference(String),       // Foreign key to another artifact's field
}

pub struct DistributionSpec {
    pub dist_type: DistributionType,
    pub params: HashMap<String, f64>,
    pub benford_compliance: bool,
}

pub enum DistributionType {
    Normal,
    LogNormal,
    Beta,
    Uniform,
    Pareto,
    Weibull,
    ZeroInflated,
    Mixture(Vec<MixtureComponent>),
}

pub enum FieldConstraint {
    Conditional {
        when: FieldCondition,
        then: Vec<FieldOverride>,
    },
    Correlation {
        fields: Vec<String>,
        coefficient: f64,
    },
    UniqueWithin {
        field: String,
        scope: String,         // "case" | "global"
    },
    ForeignKey {
        field: String,
        references: ArtifactFieldRef,
    },
}
```

### GeneratedRecord (Output)

```rust
/// Domain-agnostic output record
pub struct GeneratedRecord {
    pub artifact_type: String,
    pub record_id: String,
    pub fields: IndexMap<String, RecordValue>,
    pub metadata: RecordMetadata,
}

pub enum RecordValue {
    String(String),
    Integer(i64),
    Decimal(Decimal),
    Boolean(bool),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    Null,
    Array(Vec<RecordValue>),
    Object(IndexMap<String, RecordValue>),
}

pub struct RecordMetadata {
    pub case_id: String,
    pub step_id: String,
    pub procedure_id: String,
    pub phase_id: String,
    pub timestamp: NaiveDateTime,
    pub actor_id: String,
    pub blueprint_id: String,
}
```

When a built-in generator produces a domain struct (e.g., `PurchaseOrder`), the registry adapter converts it to `GeneratedRecord` via a `ToGeneratedRecord` trait. This means the export layer only deals with one type.

### Generation Overlay (Process Mining Ready)

The overlay schema generalizes from the audit-specific overlay to support process mining discovered parameters:

```yaml
# overlay.yaml — controls HOW the process generates, not WHAT happens
overlay_version: "2.0"

# Process variant weighting (from process mining discovery)
variants:
  - id: happy_path
    weight: 0.70
    description: "Standard flow, no rework"
    skip_procedures: []
    
  - id: with_rework
    weight: 0.20
    description: "Revision loop triggered"
    force_transitions:
      - procedure: quality_check
        transition: reject_to_rework
        
  - id: expedited
    weight: 0.10
    description: "Skip optional steps"
    skip_procedures: [detailed_review, secondary_approval]

# Transition behavior
transitions:
  defaults:
    revision_probability: 0.15
    timing:
      mu_hours: 24.0
      sigma_hours: 8.0
      distribution: lognormal           # lognormal | normal | exponential | weibull
  per_procedure:
    approve_purchase_order:
      revision_probability: 0.05
      timing:
        mu_hours: 4.0
        sigma_hours: 2.0
    receive_goods:
      timing:
        mu_hours: 72.0
        sigma_hours: 24.0

# Resource allocation (from process mining resource profiling)
actor_profiles:
  approver:
    availability_hours: [8, 17]
    timezone: "America/New_York"
    concurrent_cases: 15
    batch_processing: true              # Tends to process multiple cases at once
    batch_size: { min: 3, max: 8 }
  clerk:
    availability_hours: [9, 18]
    concurrent_cases: 5

# Anomaly injection
anomalies:
  skipped_step: 0.02
  out_of_order: 0.01
  duplicate_execution: 0.005
  unauthorized_actor: 0.01
  late_execution: 0.03                  # Breaches SLA timing
  missing_artifact: 0.02

# Volume and batching
volume:
  cases_per_period: 1000                # How many process instances to generate
  period: month
  seasonality:
    enabled: true
    pattern: [0.8, 0.9, 1.0, 1.1, 1.2, 1.0, 0.9, 0.8, 1.0, 1.1, 1.3, 1.5]

# Iteration limits (safety bounds)
iteration_limits:
  default: 50
  per_procedure: {}
```

### Process Mining Readiness Matrix

| Process Mining Output | Blueprint/Overlay Location | Notes |
|---|---|---|
| Discovered process model | `phases`, `procedures`, `aggregate.transitions` | Direct mapping from discovered Petri net / BPMN |
| Transition probabilities | `overlay.transitions.per_procedure` | Frequency analysis → probability weights |
| Timing distributions (per activity) | `overlay.transitions.*.timing` | Fitted from event log timestamps |
| Resource pools and allocation | `actors` + `overlay.actor_profiles` | From resource profiling |
| Case attributes and distributions | `artifact_schemas` field definitions | Column profiling → distribution fitting |
| Process variants | `overlay.variants` | Variant analysis with frequency weighting |
| Conformance deviations | `overlay.anomalies` | Conformance checking deviation rates |
| Bottleneck patterns | `overlay.transitions.*.timing` | Bottleneck analysis → inflated timing |
| Batch processing patterns | `overlay.actor_profiles.*.batch_processing` | Batch detection from event correlation |
| SLA definitions | `overlay.transitions.*.sla_hours` (new) | From performance analysis |

The integration layer (separate repo, future) would run discovery algorithms on real event logs and output: blueprint YAML + overlay YAML + artifact schema files. DataSynth consumes these directly.

## Example: Custom Loan Origination Blueprint

```yaml
id: loan_origination
name: "Consumer Loan Origination"
version: "1.0.0"
schema_version: "2.0"
description: "End-to-end consumer loan process from application to disbursement"

domain: banking
depth: standard

actors:
  - id: loan_officer
    name: "Loan Officer"
    responsibilities: [application_intake, initial_assessment]
  - id: credit_analyst
    name: "Credit Analyst"
    responsibilities: [credit_assessment, risk_scoring]
  - id: underwriter
    name: "Underwriter"
    responsibilities: [underwriting_decision, condition_review]
  - id: closer
    name: "Loan Closer"
    responsibilities: [document_preparation, disbursement]

artifact_schemas:
  - id: loan_application
    fields:
      - { name: application_id, type: string, generator: sequential, prefix: "LA-" }
      - { name: applicant_name, type: string, generator: person_name }
      - { name: loan_amount, type: decimal, distribution: { type: lognormal, mu: 10.2, sigma: 1.4 } }
      - { name: loan_type, type: enum, values: [mortgage, auto, personal], weights: [0.4, 0.35, 0.25] }
      - { name: credit_score, type: integer, distribution: { type: normal, mean: 720, std: 80, min: 300, max: 850 } }

  - id: credit_report
    fields:
      - { name: report_id, type: string, generator: uuid }
      - { name: score, type: integer, generator: from_context, context_key: credit_score }
      - { name: delinquencies, type: integer, distribution: { type: zero_inflated, zero_prob: 0.6, lambda: 1.5 } }
      - { name: debt_to_income, type: decimal, distribution: { type: beta, alpha: 2, beta: 5, scale: 0.8 } }

  - id: underwriting_decision
    fields:
      - { name: decision_id, type: string, generator: uuid }
      - { name: decision, type: enum, values: [approved, denied, conditional], weights: [0.55, 0.15, 0.30] }
      - { name: conditions, type: string, nullable: true, null_rate: 0.55 }
      - { name: approved_amount, type: decimal, generator: from_context, context_key: loan_amount }

phases:
  - id: intake
    name: "Application Intake"
    order: 1
    procedures:
      - id: receive_application
        title: "Receive and Register Application"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, submitted, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: start_intake, emits: IntakeStarted }
            - { from_state: in_progress, to_state: submitted, command: submit_application, emits: ApplicationSubmitted, guards: [all_steps_complete] }
            - { from_state: submitted, to_state: completed, command: confirm_receipt, emits: ReceiptConfirmed }
        steps:
          - id: collect_info
            order: 1
            action: collect
            actor: loan_officer
            description: "Collect applicant information and loan requirements"
            command: collect_application_data
            emits: ApplicationDataCollected
            artifact_type: loan_application
          - id: verify_identity
            order: 2
            action: verify
            actor: loan_officer
            description: "Verify applicant identity documents"
            command: verify_applicant_identity
            emits: IdentityVerified
        preconditions: []

  - id: assessment
    name: "Credit Assessment"
    order: 2
    gate:
      all_of:
        - { procedure: receive_application, state: completed }
    procedures:
      - id: credit_check
        title: "Perform Credit Assessment"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, under_review, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: start_credit_check, emits: CreditCheckStarted }
            - { from_state: in_progress, to_state: under_review, command: submit_assessment, emits: AssessmentSubmitted }
            - { from_state: under_review, to_state: in_progress, command: request_revision, emits: RevisionRequested }
            - { from_state: under_review, to_state: completed, command: approve_assessment, emits: AssessmentApproved }
        steps:
          - id: pull_credit
            order: 1
            action: retrieve
            actor: credit_analyst
            description: "Pull credit report from bureau"
            command: retrieve_credit_report
            emits: CreditReportRetrieved
            artifact_type: credit_report
          - id: score_risk
            order: 2
            action: evaluate
            actor: credit_analyst
            description: "Calculate risk score based on credit profile"
            command: evaluate_credit_risk
            emits: RiskScoreCalculated
        preconditions:
          - receive_application

  - id: underwriting
    name: "Underwriting Decision"
    order: 3
    gate:
      all_of:
        - { procedure: credit_check, state: completed }
    procedures:
      - id: underwrite
        title: "Make Underwriting Decision"
        aggregate:
          initial_state: not_started
          states: [not_started, in_progress, decided, completed]
          transitions:
            - { from_state: not_started, to_state: in_progress, command: start_underwriting, emits: UnderwritingStarted }
            - { from_state: in_progress, to_state: decided, command: make_decision, emits: DecisionMade }
            - { from_state: decided, to_state: completed, command: finalize_decision, emits: DecisionFinalized }
        steps:
          - id: review_package
            order: 1
            action: review
            actor: underwriter
            description: "Review complete application package"
            command: review_application_package
            emits: PackageReviewed
          - id: decide
            order: 2
            action: decide
            actor: underwriter
            description: "Make underwriting decision"
            command: make_underwriting_decision
            emits: UnderwritingDecided
            artifact_type: underwriting_decision
        preconditions:
          - credit_check
```

## Example: P2P Blueprint Facade

Demonstrates how the existing P2P generator wraps into a blueprint, making the hardcoded flow customizable:

```yaml
id: procure_to_pay
name: "Procure to Pay (Standard)"
version: "1.0.0"
schema_version: "2.0"
description: "Standard three-way match P2P process"

domain: procurement
depth: standard

actors:
  - { id: requester, name: "Requester" }
  - { id: buyer, name: "Procurement Buyer" }
  - { id: warehouse, name: "Warehouse Clerk" }
  - { id: ap_clerk, name: "AP Clerk" }
  - { id: approver, name: "Approver" }

# No artifact_schemas needed — all types are built-in registry matches
# purchase_order, goods_receipt, vendor_invoice, payment all resolve to
# existing generators in datasynth-generators

phases:
  - id: requisition
    name: "Requisition & Approval"
    order: 1
    procedures:
      - id: create_purchase_order
        title: "Create and Approve Purchase Order"
        aggregate:
          initial_state: draft
          states: [draft, pending_approval, approved, rejected]
          transitions:
            - { from_state: draft, to_state: pending_approval, command: submit_po, emits: POSubmitted }
            - { from_state: pending_approval, to_state: approved, command: approve_po, emits: POApproved, guards: [within_budget] }
            - { from_state: pending_approval, to_state: rejected, command: reject_po, emits: PORejected }
        steps:
          - id: create_po
            order: 1
            action: create
            actor: requester
            description: "Create purchase order from requisition"
            command: create_purchase_order
            emits: PurchaseOrderCreated
            artifact_type: purchase_order        # → Resolves to existing PO generator
          - id: approve_po
            order: 2
            action: approve
            actor: approver
            description: "Review and approve purchase order"
            command: approve_purchase_order
            emits: PurchaseOrderApproved
        preconditions: []

  - id: receipt
    name: "Goods Receipt"
    order: 2
    gate:
      all_of:
        - { procedure: create_purchase_order, state: approved }
    procedures:
      - id: receive_goods
        title: "Receive and Inspect Goods"
        aggregate:
          initial_state: awaiting
          states: [awaiting, received, inspected, completed]
          transitions:
            - { from_state: awaiting, to_state: received, command: receive_delivery, emits: DeliveryReceived }
            - { from_state: received, to_state: inspected, command: inspect_goods, emits: GoodsInspected }
            - { from_state: inspected, to_state: completed, command: confirm_receipt, emits: ReceiptConfirmed }
        steps:
          - id: record_receipt
            order: 1
            action: record
            actor: warehouse
            description: "Record goods receipt against PO"
            command: record_goods_receipt
            emits: GoodsReceiptRecorded
            artifact_type: goods_receipt         # → Resolves to existing GR generator
        preconditions:
          - create_purchase_order

  - id: invoice_and_payment
    name: "Invoice Processing & Payment"
    order: 3
    gate:
      all_of:
        - { procedure: receive_goods, state: completed }
    procedures:
      - id: process_invoice
        title: "Three-Way Match and Process Invoice"
        aggregate:
          initial_state: received
          states: [received, matching, matched, disputed, posted]
          transitions:
            - { from_state: received, to_state: matching, command: start_matching, emits: MatchingStarted }
            - { from_state: matching, to_state: matched, command: confirm_match, emits: ThreeWayMatchConfirmed }
            - { from_state: matching, to_state: disputed, command: flag_discrepancy, emits: DiscrepancyFlagged }
            - { from_state: disputed, to_state: matching, command: resolve_dispute, emits: DisputeResolved }
            - { from_state: matched, to_state: posted, command: post_invoice, emits: InvoicePosted }
        steps:
          - id: receive_invoice
            order: 1
            action: receive
            actor: ap_clerk
            description: "Receive vendor invoice"
            command: receive_vendor_invoice
            emits: VendorInvoiceReceived
            artifact_type: vendor_invoice        # → Resolves to existing VI generator
          - id: three_way_match
            order: 2
            action: match
            actor: ap_clerk
            description: "Perform three-way match (PO/GR/Invoice)"
            command: perform_three_way_match
            emits: ThreeWayMatchPerformed
        preconditions:
          - receive_goods

      - id: execute_payment
        title: "Execute Payment"
        aggregate:
          initial_state: pending
          states: [pending, approved, executed, confirmed]
          transitions:
            - { from_state: pending, to_state: approved, command: approve_payment, emits: PaymentApproved }
            - { from_state: approved, to_state: executed, command: execute_payment, emits: PaymentExecuted }
            - { from_state: executed, to_state: confirmed, command: confirm_payment, emits: PaymentConfirmed }
        steps:
          - id: create_payment
            order: 1
            action: create
            actor: ap_clerk
            description: "Create payment for matched invoice"
            command: create_payment
            emits: PaymentCreated
            artifact_type: payment               # → Resolves to existing Payment generator
        preconditions:
          - process_invoice
```

An enterprise could take this reference blueprint, add a quality inspection step between receipt and invoice, change the approval to require dual sign-off, or remove three-way matching for low-value POs — all in YAML.

## Crate Structure

### datasynth-process-engine

```
crates/datasynth-process-engine/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── schema.rs              # ProcessBlueprint, ProcessPhase, ProcessProcedure, ProcessStep
│   ├── artifact_schema.rs     # ArtifactSchema, FieldSchema, FieldType, FieldGenerator
│   ├── loader.rs              # YAML loading, validation, DAG checks (extracted from audit-fsm)
│   ├── engine.rs              # FSM execution engine (extracted from audit-fsm, generalized)
│   ├── registry.rs            # ArtifactRegistry, ArtifactGenerator trait
│   ├── schema_generator.rs    # SchemaGenerator — fallback for custom artifact types
│   ├── context.rs             # StepContext, SharedGenerationData
│   ├── event.rs               # ProcessEvent (generalized from AuditEvent)
│   ├── overlay.rs             # GenerationOverlay, variant weighting, timing
│   ├── record.rs              # GeneratedRecord, RecordValue, RecordMetadata
│   └── export/
│       ├── mod.rs
│       ├── json.rs            # Flat JSON event log
│       ├── ocel.rs            # OCEL 2.0 projection
│       ├── csv.rs             # Flattened CSV
│       ├── xes.rs             # XES 2.0
│       └── parquet.rs         # Columnar Parquet
└── tests/
    ├── schema_tests.rs
    ├── loader_tests.rs
    ├── engine_tests.rs
    ├── registry_tests.rs
    └── schema_generator_tests.rs
```

### datasynth-audit-fsm (refactored — thin layer)

```
crates/datasynth-audit-fsm/
├── Cargo.toml                 # Now depends on datasynth-process-engine
├── blueprints/                # 10 audit YAML blueprints (unchanged)
├── overlays/                  # 7 overlay presets (unchanged)
├── src/
│   ├── lib.rs
│   ├── audit_registry.rs      # Registers audit-specific artifact generators
│   ├── audit_context.rs       # EngagementContext → StepContext.domain_context adapter
│   ├── dispatch.rs            # AuditStepDispatcher (wraps existing generators as ArtifactGenerator impls)
│   └── compat.rs              # Re-exports for backward compatibility with existing API
└── tests/
```

### Registration Flow

```rust
// In datasynth-audit-fsm/src/audit_registry.rs
pub fn register_audit_generators(registry: &mut ArtifactRegistry) {
    registry.register(Box::new(RiskAssessmentGenerator));
    registry.register(Box::new(MaterialityGenerator));
    registry.register(Box::new(SamplingGenerator));
    registry.register(Box::new(AuditOpinionGenerator));
    // ... all 14 audit generators
}

// In datasynth-generators (future — P2P facade)
pub fn register_procurement_generators(registry: &mut ArtifactRegistry) {
    registry.register(Box::new(PurchaseOrderGenerator));
    registry.register(Box::new(GoodsReceiptGenerator));
    registry.register(Box::new(VendorInvoiceGenerator));
    registry.register(Box::new(PaymentGenerator));
}
```

## Integration with Orchestrator

### Configuration

```yaml
# In main config YAML
process_blueprints:
  enabled: true
  processes:
    - name: "Procurement"
      blueprint: builtin:p2p                    # or /path/to/custom.yaml
      overlay: builtin:default                  # or /path/to/overlay.yaml
      cases: 500                                # Number of process instances
      seed: null                                # null = derive from global seed
      discriminators:
        complexity: [standard]

    - name: "Loan Origination"
      blueprint: ./blueprints/loan_origination.yaml
      overlay: ./overlays/loan_overlay.yaml
      artifact_schemas:                         # Additional external schemas
        - ./schemas/loan_application.yaml
        - ./schemas/credit_report.yaml
      cases: 200

    - name: "Financial Statement Audit"
      blueprint: builtin:fsa                    # Routes through audit-fsm layer
      overlay: builtin:thorough
      cases: 5
```

### CLI

```bash
# Generate from a custom blueprint
datasynth-data generate --blueprint ./my_process.yaml --overlay ./my_overlay.yaml --output ./output

# Generate from built-in with customization
datasynth-data generate --blueprint builtin:p2p --cases 1000 --output ./output

# Validate a blueprint without generating
datasynth-data validate --blueprint ./my_process.yaml
```

### Output Structure

Process blueprint output lives alongside existing output:

```
output/
├── processes/
│   ├── procurement/
│   │   ├── event_log.json              # Flat process event log
│   │   ├── event_log_ocel.json         # OCEL 2.0 projection
│   │   ├── artifacts/
│   │   │   ├── purchase_orders.json    # Generated from built-in PO generator
│   │   │   ├── goods_receipts.json
│   │   │   ├── vendor_invoices.json
│   │   │   └── payments.json
│   │   └── summary.json               # Process statistics
│   ├── loan_origination/
│   │   ├── event_log.json
│   │   ├── artifacts/
│   │   │   ├── loan_applications.json  # Generated from schema-driven generator
│   │   │   ├── credit_reports.json
│   │   │   └── underwriting_decisions.json
│   │   └── summary.json
│   └── financial_statement_audit/
│       ├── event_log.json              # From audit-fsm via process engine
│       ├── artifacts/                  # Audit-specific artifacts
│       └── summary.json
├── journal_entries.csv                 # Existing output (unchanged)
├── master_data/                        # Existing output (unchanged)
└── ...
```

## Migration Path

### Phase 1: Extract Generic Engine
- Create `datasynth-process-engine` crate
- Extract schema, loader, validator, engine, event, export from `datasynth-audit-fsm`
- Generalize types (remove audit-specific fields, add `domain_context`)
- Implement `ArtifactRegistry` trait and `SchemaGenerator`
- Write comprehensive tests for the generic engine
- `datasynth-audit-fsm` depends on new crate, delegates engine work

### Phase 2: Validate with Reference Blueprints
- Write P2P and O2C blueprint facades
- Register existing generators as `ArtifactGenerator` implementations
- Validate generated output matches current hardcoded generator output
- Write integration tests comparing blueprint-driven vs. direct generation

### Phase 3: CLI and Config Integration
- Add `process_blueprints` config section
- Add `--blueprint` CLI flag
- Wire into `EnhancedOrchestrator`
- Output routing to `processes/` directory

### Phase 4: Schema-Driven Generator
- Implement `SchemaGenerator` with full distribution support
- Leverage existing `datasynth-core` distributions (AmountSampler, BenfordSampler, etc.)
- Test with the loan origination example blueprint
- Field constraint evaluation (conditional distributions, correlations)

### Phase 5: Audit Crate Refactor
- Refactor `datasynth-audit-fsm` to thin layer
- Move engine delegation to `datasynth-process-engine`
- Verify all 10 audit blueprints produce identical output
- Backward compatibility for existing audit config YAML

## Testing Strategy

| Test Type | Scope | Details |
|-----------|-------|---------|
| Unit | Schema parsing | Round-trip YAML → Rust → YAML for all blueprint types |
| Unit | Loader validation | Invalid blueprints caught: cycles, missing refs, unreachable states |
| Unit | Engine FSM | Transition logic, guard evaluation, topological ordering |
| Unit | SchemaGenerator | Each field type and distribution produces valid output |
| Unit | ArtifactRegistry | Built-in lookup, schema fallback, unknown type handling |
| Integration | P2P facade | Blueprint-driven P2P matches direct generator output (statistical) |
| Integration | Loan origination | Custom blueprint with schema-driven artifacts produces valid records |
| Integration | Audit compat | Refactored audit crate produces identical output to pre-refactor |
| Property | Determinism | Same seed → identical output across runs |
| Property | Benford compliance | Schema-driven decimal fields with benford_compliance pass MAD test |
| Benchmark | Throughput | Process engine overhead vs. direct generator performance |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Audit crate regression during extraction | High — 10 production blueprints | Phase 5 is last; comprehensive before/after comparison tests |
| Schema-driven generator quality | Medium — may produce unrealistic data | Leverage existing distribution infrastructure from datasynth-core; validate against known domains |
| Performance overhead of registry dispatch | Low — one hashmap lookup per step | Benchmark; registry lookup is O(1), negligible vs. generation cost |
| Blueprint YAML complexity for users | Medium — steep learning curve | Reference blueprints (P2P, O2C) as starting templates; validation errors with clear messages |
| Process mining schema drift | Low — integration layer is future | Schema versioning (`schema_version: "2.0"`); overlay format designed from process mining output |
