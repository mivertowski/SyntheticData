# RustGraph & DataSynth Round 2 Feature Requests — Design Document

**Date:** 2026-03-01
**Scope:** DS-001 through DS-012 (12 feature requests, 51 new entity types, edge registry)
**Status:** Approved

---

## Problem Statement

AssureTwin (downstream graph consumer) queries DataSynth-generated graph nodes with
specific property keys, entity type codes, and edge relationships. The current graph
export layer is ML-feature-focused (numeric vectors for PyG/DGL) rather than
semantic-property-focused (key-value pairs for Neo4j/AssureTwin). This creates three
gaps:

1. **Property gap**: GraphNode.properties HashMap is never populated by builders
2. **Entity type gap**: New domain entity types (Tax, Treasury, ESG, etc.) lack graph
   registration with numeric codes
3. **Edge gap**: No formal registry of edge types with source→target constraints

## Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Property mapping location | `ToNodeProperties` trait on each model struct | Clean, discoverable, type-safe |
| Edge type definition | Extend existing `RelationshipType` enum | Builds on 38 existing variants |
| Entity naming (DS-001) | `uncertain_tax_position` (snake_case) | Matches AssureTwin queries |
| Debt covenant (DS-003) | Split into standalone `DebtCovenant` model | AssureTwin expects separate node type |
| ESG scope (DS-004) | All 13 entity types | Prevents follow-up round |
| New models (DS-008) | Full typed structs for BomComponent, InventoryMovement, BenefitEnrollment | Matches codebase convention |

---

## Section 1: ToNodeProperties Trait

### Location
`crates/datasynth-core/src/models/graph_properties.rs` (new file)

### Definition
```rust
use std::collections::HashMap;

/// Property value for graph node export.
/// Mirrors datasynth-graph NodeProperty but lives in core to avoid circular deps.
pub enum GraphPropertyValue {
    String(String),
    Int(i64),
    Float(f64),
    Decimal(rust_decimal::Decimal),
    Bool(bool),
    Date(chrono::NaiveDate),
    StringList(Vec<String>),
}

/// Trait for converting typed model structs to graph node property maps.
///
/// Implementations map struct fields to camelCase property keys matching
/// downstream consumer (AssureTwin) DTO expectations.
pub trait ToNodeProperties {
    /// Entity type name (snake_case), e.g. "uncertain_tax_position"
    fn node_type_name(&self) -> &'static str;

    /// Numeric entity type code for registry, e.g. 416
    fn node_type_code(&self) -> u16;

    /// Convert all fields to a property map with camelCase keys.
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue>;
}
```

### Implementation Pattern
Each model struct gets a straightforward field-to-property mapping:
```rust
impl ToNodeProperties for TaxReturn {
    fn node_type_name(&self) -> &'static str { "tax_return" }
    fn node_type_code(&self) -> u16 { 413 }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut props = HashMap::new();
        props.insert("entityCode".into(), GraphPropertyValue::String(self.entity_code.clone()));
        props.insert("period".into(), GraphPropertyValue::String(self.period.clone()));
        // ... all fields
        props
    }
}
```

### Entity Types Implementing the Trait

| Family | Entity Types (code) |
|--------|-------------------|
| TAX | TaxJurisdiction (410), TaxCode (411), TaxLine (412), TaxReturn (413), TaxProvision (414), WithholdingTaxRecord (415), UncertainTaxPosition (416) |
| TREASURY | CashPosition (420), CashForecast (421), CashPool (422), CashPoolSweep (423), HedgingInstrument (424), HedgeRelationship (425), DebtInstrument (426), DebtCovenant (427) |
| ESG | EmissionRecord (430), EnergyConsumption (431), WaterUsage (432), WasteRecord (433), WorkforceDiversityMetric (434), PayEquityMetric (435), SafetyIncident (436), SafetyMetric (437), GovernanceMetric (438), SupplierEsgAssessment (439), MaterialityAssessment (440), EsgDisclosure (441), ClimateScenario (442) |
| PROJECT | Project (450), ProjectCostLine (451), ProjectRevenue (452), EarnedValueMetric (453), ChangeOrder (454), ProjectMilestone (455) |
| S2C | SourcingProject (320), RfxEvent (321), SupplierBid (322), BidEvaluation (323), ProcurementContract (324), SupplierQualification (325) |
| H2R | PayrollRun (330), TimeEntry (331), ExpenseReport (332), BenefitEnrollment (333) |
| MFG | ProductionOrder (340), QualityInspection (341), CycleCount (342), BomComponent (343), Material (102), InventoryMovement (105) |
| GOV | CosoComponent (500), CosoPrinciple (501), SoxAssertion (502), AuditEngagement (360), ProfessionalJudgment (365) |

**Total: ~51 implementations**

---

## Section 2: Entity Type Registry

### Location
`crates/datasynth-core/src/models/relationship.rs` — extend `GraphEntityType` enum

### New Variants
Add ~35 new variants to the existing 20-variant enum. Each gets:
- `code()` → 2-letter string
- `numeric_code()` → u16 (new method)
- `node_type_name()` → snake_case string (new method)
- Category helpers: `is_tax()`, `is_treasury()`, `is_esg()`, `is_project()`, `is_h2r()`, `is_mfg()`, `is_governance()`

### Static Registry
```rust
impl GraphEntityType {
    pub fn from_numeric_code(code: u16) -> Option<Self> { ... }
    pub fn from_node_type_name(name: &str) -> Option<Self> { ... }
    pub fn all_types() -> &'static [GraphEntityType] { ... }
}
```

---

## Section 3: Edge Type Registry (DS-010)

### Location
`crates/datasynth-core/src/models/relationship.rs` — extend `RelationshipType` enum

### New Variants (~30)

| Family | Edge Type | Source → Target | Cardinality |
|--------|-----------|----------------|-------------|
| P2P | PlacedWith | PurchaseOrder → Vendor | N:1 |
| P2P | MatchesOrder | VendorInvoice → PurchaseOrder | N:1 |
| P2P | PaysInvoice | Payment → VendorInvoice | N:N |
| O2C | PlacedBy | SalesOrder → Customer | N:1 |
| O2C | BillsOrder | CustomerInvoice → SalesOrder | N:1 |
| S2C | RfxBelongsToProject | RfxEvent → SourcingProject | N:1 |
| S2C | RespondsTo | SupplierBid → RfxEvent | N:1 |
| S2C | AwardedFrom | ProcurementContract → BidEvaluation | 1:1 |
| H2R | RecordedBy | TimeEntry → Employee | N:1 |
| H2R | PayrollIncludes | PayrollRun → Employee | N:N |
| H2R | SubmittedBy | ExpenseReport → Employee | N:1 |
| MFG | Produces | ProductionOrder → Material | N:1 |
| MFG | Inspects | QualityInspection → ProductionOrder | N:1 |
| MFG | PartOf | BomComponent → Material | N:1 |
| TAX | TaxLineBelongsTo | TaxLine → TaxReturn | N:1 |
| TAX | ProvisionAppliesTo | TaxProvision → TaxJurisdiction | N:1 |
| TAX | WithheldFrom | WithholdingTaxRecord → Vendor | N:1 |
| TREASURY | SweepsTo | CashPoolSweep → CashPool | N:1 |
| TREASURY | HedgesInstrument | HedgeRelationship → HedgingInstrument | N:1 |
| TREASURY | GovernsInstrument | DebtCovenant → DebtInstrument | N:1 |
| ESG | EmissionReportedBy | EmissionRecord → Company | N:1 |
| ESG | AssessesSupplier | SupplierEsgAssessment → Vendor | N:1 |
| PROJECT | CostChargedTo | ProjectCostLine → Project | N:1 |
| PROJECT | MilestoneOf | ProjectMilestone → Project | N:1 |
| PROJECT | ModifiesProject | ChangeOrder → Project | N:1 |
| GOV | PrincipleUnder | CosoPrinciple → CosoComponent | N:1 |
| GOV | AssertionCovers | SoxAssertion → GlAccount | N:N |
| GOV | JudgmentWithin | ProfessionalJudgment → AuditEngagement | N:1 |

### EdgeConstraint Struct
```rust
pub struct EdgeConstraint {
    pub relationship_type: RelationshipType,
    pub source_type: GraphEntityType,
    pub target_type: GraphEntityType,
    pub cardinality: Cardinality,
    pub edge_properties: &'static [&'static str],
}

pub enum Cardinality { OneToOne, OneToMany, ManyToMany }

impl RelationshipType {
    pub fn constraint(&self) -> Option<EdgeConstraint> { ... }
    pub fn all_constraints() -> Vec<EdgeConstraint> { ... }
}
```

---

## Section 4: New Model Structs

### 4a. BomComponent
**Location:** `crates/datasynth-core/src/models/manufacturing.rs` (new file)

Fields: `id`, `entity_code`, `parent_material`, `component_material`,
`component_description`, `level` (u32), `quantity_per` (Decimal), `unit`,
`scrap_rate` (Decimal), `is_phantom` (bool)

### 4b. InventoryMovement
**Location:** Same file

Fields: `id`, `entity_code`, `material_code`, `material_description`,
`movement_date`, `period`, `movement_type` (enum: GoodsReceipt, GoodsIssue,
Transfer, Return, Scrap, Adjustment), `quantity`, `unit`, `value`, `currency`,
`storage_location`, `reference_doc`

### 4c. BenefitEnrollment
**Location:** `crates/datasynth-core/src/models/hr.rs` (extend existing)

Fields: `id`, `entity_code`, `employee_id`, `employee_name`, `plan_type`
(enum: Health, Dental, Vision, Retirement401k, StockPurchase, LifeInsurance,
Disability), `plan_name`, `enrollment_date`, `effective_date`, `period`,
`employee_contribution`, `employer_contribution`, `currency`, `status`
(enum: Active, Pending, Terminated, OnLeave), `is_active`

### 4d. DebtCovenant (extracted from treasury.rs)
**Location:** `crates/datasynth-core/src/models/treasury.rs` (refactor)

Extract the existing covenant sub-struct into a standalone model:
Fields: `id`, `facility_id`, `entity_code`, `facility_name`, `covenant_type`,
`period`, `threshold` (Decimal), `actual_value` (Decimal), `headroom` (Decimal),
`compliance_status`, `outstanding_principal` (Decimal), `currency`, `test_date`

---

## Section 5: Denormalization (DS-011)

Add `Option<String>` name fields to existing transaction models:

| Model | New Field | Populated By |
|-------|-----------|-------------|
| PurchaseOrder | `vendor_name` | p2p_generator looks up vendor pool |
| VendorInvoice | `vendor_name` | Same |
| SalesOrder | `customer_name` | o2c_generator looks up customer pool |
| CustomerInvoice | `customer_name` | Same |
| SupplierBid | `vendor_name` | Sourcing generator (may already exist) |
| BidEvaluation | `vendor_name` | Same |
| ProcurementContract | `vendor_name` | Same |
| SupplierQualification | `vendor_name` | Same |
| TimeEntry | `employee_name` | HR generator looks up employee pool |
| ExpenseReport | `employee_name` | Same |
| ProductionOrder | `material_description` | MFG generator looks up material pool |
| CycleCount | `material_description` | Same |

Generator changes: pass name alongside ID during construction. All generators
already hold references to the relevant master data pools.

---

## Section 6: Boolean Flags (DS-012)

### Verification & Addition
Audit all model structs for boolean fields. Where missing, add the field.
Where present but not generated, add generation logic.

### Probability Table

| Flag | Models | True % | Logic |
|------|--------|--------|-------|
| `is_approved` | PurchaseOrder, SalesOrder, PayrollRun, ExpenseReport | 85/90/90/80 | Status-derived |
| `is_complete` | GoodsReceipt, Delivery, ProductionOrder | 75/70/65 | Status-derived |
| `is_matched` | VendorInvoice | 90 | Three-way match engine |
| `is_paid` | CustomerInvoice, ProjectMilestone | 80/60 | Payment linkage |
| `is_reconciled` | Payment | 85 | Reconciliation engine |
| `is_active` | TaxCode, TaxJurisdiction, Material, ProcurementContract, BenefitEnrollment | 95 | Master data lifecycle |
| `is_effective` | HedgingInstrument | 80 | Effectiveness test |
| `is_qualified` | SupplierQualification | 90 | Score threshold |
| `is_phantom` | BomComponent | 10 | BOM structure |
| `is_adjusted` | CycleCount | 30 | Variance > threshold |
| `is_passed` | QualityInspection | 95 | QC result |
| `is_invoiced` | ProjectMilestone | 60 | Billing status |
| `is_material` | EsgDisclosure | 40 | Materiality assessment |
| `has_corrective_action` | SupplierEsgAssessment | 25 | Risk tier |
| `treaty_applied` | WithholdingTaxRecord | 30 | Treaty eligibility |
| `billable` | TimeEntry | 70 | Activity code |
| `is_compliant` | SupplierBid | 85 | Compliance check |

---

## Section 7: Testing Strategy

### Unit Tests — ToNodeProperties implementations
For each of the ~51 entity types, verify:
- `to_node_properties()` returns all expected keys
- Values have correct types (String, Decimal, Bool, etc.)
- No unexpected None/empty values for required fields

**Location:** `crates/datasynth-core/tests/graph_properties_tests.rs`

### Unit Tests — Entity Type Registry
- `from_numeric_code()` round-trips for all codes
- `from_node_type_name()` round-trips for all names
- `all_types()` returns all registered types
- Category helpers (`is_tax()`, etc.) are correct

**Location:** `crates/datasynth-core/tests/entity_registry_tests.rs`

### Unit Tests — Edge Constraints
- All edges connect valid source→target types
- `constraint()` returns Some for all domain edges
- `all_constraints()` is complete
- Inverse relationships are consistent

**Location:** `crates/datasynth-core/tests/edge_constraint_tests.rs`

### Integration Tests — Property Completeness
Generate a small dataset and export to graph format. For each entity type:
- Assert all AssureTwin-expected property keys are present
- Assert no null/empty values for required properties
- Assert boolean flags have reasonable distributions

**Location:** `crates/datasynth-generators/tests/graph_property_completeness_tests.rs`

### Integration Tests — Boolean Distribution
Generate N=1000 entities per type. Assert boolean flags fall within expected
probability ranges (±15% tolerance for statistical variance).

**Location:** `crates/datasynth-generators/tests/boolean_distribution_tests.rs`

### Integration Tests — Denormalization
Generate P2P/O2C/S2C flows and verify:
- `vendor_name` is present and non-empty on POs, invoices
- `customer_name` is present on sales orders, customer invoices
- `employee_name` is present on time entries, expense reports
- `material_description` is present on production orders, cycle counts

**Location:** `crates/datasynth-generators/tests/denormalization_tests.rs`

---

## File Change Summary

| Category | Files | Action |
|----------|-------|--------|
| New files | `datasynth-core/src/models/graph_properties.rs`, `datasynth-core/src/models/manufacturing.rs` | Create |
| New test files | 6 test files across core and generators | Create |
| Modified models | `relationship.rs`, `treasury.rs`, `hr.rs`, `tax.rs`, `esg.rs`, `project_accounting.rs`, `internal_control.rs`, `coso.rs` | Add ToNodeProperties impls |
| Modified models | `purchase_order.rs`, `vendor_invoice.rs`, `sales_order.rs`, `customer_invoice.rs`, `time_entry.rs`, `expense_report.rs`, `production_order.rs`, `cycle_count.rs` | Add denormalized name fields |
| Modified generators | `p2p_generator.rs`, `o2c_generator.rs`, sourcing generators, HR generators, MFG generators | Populate name fields, boolean flags |
| New generators | `bom_generator.rs`, `inventory_movement_generator.rs`, `benefit_enrollment_generator.rs` | Create |
| Modified lib.rs | `datasynth-core/src/models/mod.rs` | Re-export new modules |

**Estimated: ~15 new files, ~25 modified files**
