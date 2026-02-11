# Source-to-Pay (S2P) Process Chain — Specification & Implementation Plan

> **Status:** Draft
> **Date:** 2026-02-11
> **Scope:** Extend the existing P2P pipeline with Source-to-Contract (S2C) to model the full S2P chain

---

## 1. Executive Summary

### 1.1 Current State

The codebase provides **comprehensive Procure-to-Pay (P2P)** coverage:

| P2P Step | Models | Generators | Config | OCPM Events |
|----------|--------|------------|--------|-------------|
| Purchase Requisition | `PurchaseRequisition` | p2p_generator | P2PFlowConfig | — |
| Purchase Order | `PurchaseOrder` | p2p_generator | P2PFlowConfig | create_po, approve_po, release_po |
| Goods Receipt | `GoodsReceipt` | p2p_generator | P2PFlowConfig | create_gr, post_gr |
| Vendor Invoice | `VendorInvoice` | p2p_generator | P2PFlowConfig | receive_invoice, verify_invoice, post_invoice |
| Three-Way Match | `ThreeWayMatcher` | three_way_match | P2PFlowConfig | (part of verify_invoice) |
| Payment | `Payment` | p2p_generator | P2PPaymentBehaviorConfig | execute_payment |
| Vendor Network | `VendorNetwork`, `VendorQualityScore`, `VendorCluster` | vendor_generator | VendorNetworkSchemaConfig | — |

### 1.2 Gap: Source-to-Contract (S2C)

**Zero coverage exists** for the upstream strategic-sourcing phase:

| S2C Step | Models | Generators | Config |
|----------|--------|------------|--------|
| Spend Analysis | — | — | — |
| Supplier Identification | — | — | — |
| Supplier Qualification | — | — | — |
| RFx Process (RFP/RFQ/RFI) | — | — | — |
| Bid Evaluation | — | — | — |
| Negotiation | — | — | — |
| Contract Management | — | — | — |
| Catalog / Pricing Agreement | — | — | — |

### 1.3 Goal

Model the **complete S2P chain** as a coherent, interconnected pipeline where:

1. **Spend Analysis** identifies category needs and consolidation opportunities
2. **Sourcing Events** (RFx) invite qualified suppliers to bid
3. **Bid Evaluation** selects winners through weighted scoring
4. **Contracts** capture negotiated terms, pricing, SLAs, and validity periods
5. **Catalog Items** link contract terms to materials/services at agreed prices
6. **Purchase Requisitions** reference catalog items (contract-backed) or are free-text (off-contract)
7. **Purchase Orders** inherit contract terms (price, Incoterms, payment terms) and enforce contract compliance
8. **Three-Way Match** validates against both PO *and* contract price
9. **Vendor Performance** feeds back into supplier scorecards which influence future sourcing decisions

This creates a **closed-loop data model** where every procurement transaction traces back to a sourcing decision, and operational performance informs future strategic sourcing.

---

## 2. Process Chain Architecture

### 2.1 End-to-End S2P Flow

```
┌─────────────────────────────── SOURCE-TO-CONTRACT (S2C) ──────────────────────────────────┐
│                                                                                            │
│  ┌──────────┐    ┌───────────┐    ┌──────────┐    ┌──────────┐    ┌──────────────────────┐ │
│  │  Spend   │───▶│ Supplier  │───▶│   RFx    │───▶│   Bid    │───▶│  Contract / Catalog  │ │
│  │ Analysis │    │ Qualific. │    │ Process  │    │  Eval.   │    │    Management        │ │
│  └──────────┘    └───────────┘    └──────────┘    └──────────┘    └──────────┬───────────┘ │
│                                                                              │             │
└──────────────────────────────────────────────────────────────────────────────┼─────────────┘
                                                                               │
                        ┌──────────────────────────────────────────────────────┘
                        ▼
┌──────────────────── PROCURE-TO-PAY (P2P) — Already Implemented ───────────────────────────┐
│                                                                                            │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐             │
│  │ Purchase │───▶│ Purchase │───▶│  Goods   │───▶│  Vendor  │───▶│ Payment  │             │
│  │  Reqn.   │    │  Order   │    │ Receipt  │    │ Invoice  │    │          │             │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘             │
│                                                                                            │
└──────────────────────────────────────────────────────────────────────────────────────────┬─┘
                                                                                           │
                        ┌──────────────────────────────────────────────────────────────────┘
                        ▼
┌──────────────────── FEEDBACK LOOP ────────────────────────────────────────────────────────┐
│                                                                                            │
│  ┌──────────────┐    ┌───────────────┐    ┌──────────────────┐                             │
│  │   Vendor     │───▶│   Supplier    │───▶│  Sourcing        │                             │
│  │ Performance  │    │  Scorecard    │    │  Decision Input  │                             │
│  └──────────────┘    └───────────────┘    └──────────────────┘                             │
│         ▲                                                                                  │
│         │  (from P2P: delivery, quality, invoice accuracy, payment history)                 │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Cross-Entity Reference Map

Every entity carries foreign keys that tie the chain together:

```
SpendCategory ─────┬──▶ SourcingProject ──▶ RfxEvent ──▶ SupplierBid
                   │                                         │
                   ▼                                         ▼
             SpendAnalysis                            BidEvaluation
                                                          │
                                                          ▼
Vendor ◀── SupplierQualification ◀───────── ProcurementContract
   │                                              │         │
   │           ┌──────────────────────────────────┘         │
   │           ▼                                            ▼
   │     ContractLineItem ──▶ CatalogItem          ContractTerms
   │           │                    │
   │           │                    ▼
   │           │         PurchaseRequisition.catalog_item_id ──(new field)
   │           │                    │
   │           ▼                    ▼
   └──▶ PurchaseOrder.contract_id ──(new field)
              │
              ▼
        GoodsReceipt ──▶ VendorInvoice ──▶ Payment
              │                │
              ▼                ▼
        VendorQualityScore (existing) ──▶ SupplierScorecard (new)
                                                │
                                                ▼
                                    (feeds into next SourcingProject)
```

### 2.3 ID Scheme

Following the existing `uuid_factory.rs` FNV-1a hash-based deterministic UUID pattern, new generator-type discriminators are needed:

| Entity | Discriminator Prefix | Example |
|--------|---------------------|---------|
| SourcingProject | `SRCPRJ` | `SRCPRJ-2024-001` |
| RfxEvent | `RFX` | `RFX-2024-00042` |
| SupplierBid | `BID` | `BID-2024-00042-V001` |
| BidEvaluation | `BEVAL` | `BEVAL-2024-00042` |
| ProcurementContract | `PCTR` | `PCTR-2024-00015` |
| ContractLineItem | `CTRLI` | `CTRLI-2024-00015-010` |
| CatalogItem | `CAT` | `CAT-M001-V001` |
| SupplierQualification | `SQUAL` | `SQUAL-V001-2024` |
| SupplierScorecard | `SCARD` | `SCARD-V001-2024Q1` |
| SpendAnalysis | `SPEND` | `SPEND-2024-IT` |

---

## 3. Data Model Specification

### 3.1 New Models — `datasynth-core/src/models/sourcing/`

#### 3.1.1 `SpendCategory` (extend existing)

The existing `SpendCategory` enum in `templates/realism/vendor_names.rs` (20 categories) is reused. A new **aggregation model** wraps it:

```rust
/// Aggregated spend analysis for a category within a fiscal period.
pub struct SpendAnalysis {
    pub id: String,                         // SPEND-{year}-{category_code}
    pub category: SpendCategory,
    pub fiscal_year: u16,
    pub fiscal_period: Option<u8>,          // None = full year
    pub total_spend: Decimal,
    pub transaction_count: u64,
    pub unique_vendor_count: u32,
    pub top_vendors: Vec<VendorSpendShare>, // vendor_id, spend, share%
    pub concentration_hhi: f64,             // Herfindahl-Hirschman Index
    pub yoy_change_percent: Option<f64>,    // Year-over-year delta
    pub avg_unit_price: Option<Decimal>,
    pub contract_coverage_rate: f64,        // % of spend under contract
    pub maverick_spend_rate: f64,           // % of spend off-contract
    pub consolidation_opportunity: bool,    // flagged if HHI < threshold
    pub analysis_date: NaiveDate,
}

pub struct VendorSpendShare {
    pub vendor_id: String,
    pub spend_amount: Decimal,
    pub share_percent: f64,
    pub transaction_count: u32,
}
```

#### 3.1.2 `SourcingProject`

```rust
/// A strategic sourcing initiative for a spend category.
pub enum SourcingProjectType {
    NewSupplier,          // First-time sourcing for a category
    Resourcing,           // Replacing existing supplier(s)
    ContractRenewal,      // Renewing expiring contracts
    CostReduction,        // Re-bidding for savings
    RiskMitigation,       // Diversifying supply base
    Consolidation,        // Reducing supplier count
}

pub enum SourcingProjectStatus {
    Planning,
    SupplierIdentification,
    QualificationInProgress,
    RfxPublished,
    BidEvaluation,
    NegotiationInProgress,
    Awarded,
    ContractExecution,
    Completed,
    Cancelled,
}

pub struct SourcingProject {
    pub id: String,                         // SRCPRJ-{year}-{seq}
    pub title: String,
    pub description: String,
    pub project_type: SourcingProjectType,
    pub status: SourcingProjectStatus,
    pub category: SpendCategory,
    pub estimated_annual_spend: Decimal,
    pub target_savings_percent: f64,        // e.g. 0.10 = 10% target
    pub owner_employee_id: String,          // Procurement manager
    pub stakeholder_department: String,
    pub start_date: NaiveDate,
    pub target_award_date: NaiveDate,
    pub actual_completion_date: Option<NaiveDate>,
    pub invited_vendor_ids: Vec<String>,
    pub qualified_vendor_ids: Vec<String>,
    pub awarded_vendor_id: Option<String>,
    pub rfx_event_ids: Vec<String>,         // One or more RFx rounds
    pub contract_id: Option<String>,        // Resulting contract
    pub spend_analysis_id: Option<String>,  // Triggering analysis
    pub actual_savings_percent: Option<f64>,
}
```

#### 3.1.3 `SupplierQualification`

```rust
/// Qualification status for a vendor being evaluated for sourcing.
pub enum QualificationStatus {
    Pending,
    InReview,
    Qualified,
    ConditionallyQualified,  // Qualified with conditions/corrective actions
    NotQualified,
    Disqualified,            // Removed from consideration
    Expired,                 // Qualification lapsed
}

pub enum QualificationCriterion {
    FinancialStability,
    ProductionCapacity,
    QualityCertification,    // ISO 9001, etc.
    EnvironmentalCompliance, // ISO 14001, etc.
    RegulatoryCompliance,
    InformationSecurity,     // ISO 27001, SOC2
    InsuranceCoverage,
    ReferencesCheck,
    SiteAudit,
    DeliveryCapability,
    TechnicalCapability,
}

pub struct QualificationScore {
    pub criterion: QualificationCriterion,
    pub weight: f64,                        // 0.0-1.0, sum to 1.0
    pub score: f64,                         // 0.0-1.0
    pub pass_threshold: f64,                // Minimum score to pass
    pub passed: bool,
    pub notes: Option<String>,
    pub evidence_document_id: Option<String>,
}

pub struct SupplierQualification {
    pub id: String,                         // SQUAL-{vendor_id}-{year}
    pub vendor_id: String,
    pub sourcing_project_id: Option<String>,
    pub status: QualificationStatus,
    pub qualification_date: NaiveDate,
    pub expiry_date: NaiveDate,             // Typically qualification_date + 1 year
    pub evaluator_employee_id: String,
    pub scores: Vec<QualificationScore>,
    pub overall_score: f64,                 // Weighted average
    pub overall_passed: bool,               // All mandatory criteria met
    pub conditions: Vec<String>,            // Conditions for conditional qualification
    pub certifications: Vec<SupplierCertification>,
}

pub struct SupplierCertification {
    pub certification_type: String,         // "ISO 9001", "ISO 14001", "SOC2", etc.
    pub issuing_body: String,
    pub certificate_number: Option<String>,
    pub issue_date: NaiveDate,
    pub expiry_date: NaiveDate,
    pub verified: bool,
}
```

#### 3.1.4 `RfxEvent`

```rust
/// Type of RFx solicitation.
pub enum RfxType {
    Rfi,    // Request for Information — non-binding, exploratory
    Rfq,    // Request for Quotation — price-focused, defined spec
    Rfp,    // Request for Proposal — comprehensive, weighted evaluation
}

pub enum RfxStatus {
    Draft,
    Published,
    SubmissionOpen,
    SubmissionClosed,
    UnderEvaluation,
    Awarded,
    Cancelled,
    NoAward,       // No bids met criteria
}

pub struct RfxEvaluationCriterion {
    pub name: String,                       // "Price", "Quality", "Delivery", etc.
    pub weight: f64,                        // Evaluation weight
    pub description: String,
    pub scoring_method: ScoringMethod,
}

pub enum ScoringMethod {
    LowestPriceBest,       // Inverse score: lowest price gets highest score
    HighestScoreBest,      // Direct score
    PassFail,              // Binary pass/fail
    TieredScale,           // 1-5 or 1-10 scale
}

pub struct RfxLineItem {
    pub line_number: u32,
    pub material_id: Option<String>,
    pub description: String,
    pub quantity: Decimal,
    pub uom: String,
    pub target_unit_price: Option<Decimal>, // Budget target (may be hidden from bidders)
    pub delivery_date: NaiveDate,
    pub specifications: Option<String>,
}

pub struct RfxEvent {
    pub id: String,                         // RFX-{year}-{seq}
    pub rfx_type: RfxType,
    pub sourcing_project_id: String,
    pub title: String,
    pub description: String,
    pub status: RfxStatus,
    pub category: SpendCategory,
    pub currency: String,
    pub publish_date: NaiveDate,
    pub submission_deadline: NaiveDate,
    pub evaluation_deadline: NaiveDate,
    pub invited_vendor_ids: Vec<String>,
    pub responded_vendor_ids: Vec<String>,
    pub line_items: Vec<RfxLineItem>,
    pub evaluation_criteria: Vec<RfxEvaluationCriterion>,
    pub bid_ids: Vec<String>,               // SupplierBid IDs received
    pub winning_bid_id: Option<String>,
    pub owner_employee_id: String,
    pub is_sealed_bid: bool,
    pub allow_partial_bids: bool,           // Vendor can bid on subset of lines
    pub previous_rfx_id: Option<String>,    // If this is a follow-up round
}
```

#### 3.1.5 `SupplierBid`

```rust
pub enum BidStatus {
    Submitted,
    UnderReview,
    Shortlisted,
    BestAndFinal,     // Invited to BAFO round
    Awarded,
    Rejected,
    Withdrawn,
    Disqualified,
}

pub struct BidLineItem {
    pub rfx_line_number: u32,               // References RfxLineItem.line_number
    pub unit_price: Decimal,
    pub total_price: Decimal,
    pub lead_time_days: u32,
    pub moq: Option<Decimal>,               // Minimum order quantity
    pub alternative_offered: bool,          // Vendor proposed alternative
    pub notes: Option<String>,
}

pub struct SupplierBid {
    pub id: String,                         // BID-{rfx_seq}-{vendor_code}
    pub rfx_event_id: String,
    pub vendor_id: String,
    pub status: BidStatus,
    pub submission_date: NaiveDate,
    pub total_bid_amount: Decimal,
    pub currency: String,
    pub proposed_payment_terms: String,     // "Net 30", "2/10 Net 30", etc.
    pub proposed_incoterms: Option<String>,
    pub proposed_warranty_months: Option<u32>,
    pub line_items: Vec<BidLineItem>,
    pub validity_days: u32,                 // Bid valid for N days
    pub is_partial: bool,                   // Partial bid (subset of lines)
    pub technical_score: Option<f64>,       // Set during evaluation
    pub commercial_score: Option<f64>,
    pub overall_score: Option<f64>,
    pub rank: Option<u32>,                  // Set after scoring
}
```

#### 3.1.6 `BidEvaluation`

```rust
pub struct BidEvaluationEntry {
    pub bid_id: String,
    pub vendor_id: String,
    pub criterion_name: String,
    pub raw_score: f64,                     // 0.0-1.0
    pub weighted_score: f64,                // raw_score × criterion weight
    pub evaluator_notes: Option<String>,
}

pub struct BidEvaluation {
    pub id: String,                         // BEVAL-{rfx_seq}
    pub rfx_event_id: String,
    pub evaluation_date: NaiveDate,
    pub evaluator_ids: Vec<String>,         // Panel members
    pub entries: Vec<BidEvaluationEntry>,
    pub ranked_bids: Vec<RankedBid>,        // Sorted by total_weighted_score desc
    pub award_recommendation: Option<String>, // Recommended vendor_id
    pub award_justification: String,
    pub savings_vs_budget: Option<f64>,     // % savings vs target price
    pub approved_by: Option<String>,        // Senior approval for award
    pub approval_date: Option<NaiveDate>,
}

pub struct RankedBid {
    pub bid_id: String,
    pub vendor_id: String,
    pub total_weighted_score: f64,
    pub rank: u32,
    pub recommendation: AwardRecommendation,
}

pub enum AwardRecommendation {
    PrimaryAward,
    BackupAward,
    NoAward,
}
```

#### 3.1.7 `ProcurementContract`

```rust
pub enum ContractType {
    FixedPrice,            // Fixed unit prices for the term
    FrameworkAgreement,    // Agreed pricing, quantities released as needed
    BlanketOrder,          // Pre-approved spend ceiling, draw-down
    ServiceAgreement,      // Time & materials or fixed-fee services
    ConsignmentAgreement,  // Vendor-owned inventory at buyer's site
    RateCard,              // Agreed hourly/daily rates (professional services)
}

pub enum ContractStatus {
    Draft,
    UnderNegotiation,
    PendingApproval,
    Active,
    OnHold,
    Expiring,              // Within 90 days of end_date
    Expired,
    Terminated,
    Renewed,
}

pub struct ContractTerms {
    pub payment_terms: String,              // "Net 30", "2/10 Net 30"
    pub incoterms: Option<String>,          // "FOB", "CIF", "DDP"
    pub warranty_months: Option<u32>,
    pub liability_cap: Option<Decimal>,
    pub penalty_late_delivery_percent: Option<f64>,
    pub early_payment_discount_percent: Option<f64>,
    pub early_payment_discount_days: Option<u32>,
    pub minimum_order_value: Option<Decimal>,
    pub maximum_order_value: Option<Decimal>,
    pub auto_renewal: bool,
    pub renewal_notice_days: u32,           // Days before expiry to notify
    pub termination_notice_days: u32,
    pub force_majeure_clause: bool,
    pub sla_definitions: Vec<ContractSla>,
}

pub struct ContractSla {
    pub metric_name: String,                // "on_time_delivery", "defect_rate", etc.
    pub target_value: f64,                  // e.g. 0.95 = 95%
    pub minimum_value: f64,                 // e.g. 0.90 = 90% (breach threshold)
    pub measurement_period: String,         // "monthly", "quarterly"
    pub penalty_per_breach: Option<Decimal>,
    pub consecutive_breach_termination: Option<u32>, // Terminate after N breaches
}

pub struct ContractLineItem {
    pub id: String,                         // CTRLI-{contract_seq}-{line}
    pub line_number: u32,
    pub material_id: Option<String>,
    pub description: String,
    pub unit_price: Decimal,
    pub currency: String,
    pub uom: String,
    pub min_quantity: Option<Decimal>,
    pub max_quantity: Option<Decimal>,       // Ceiling quantity
    pub quantity_released: Decimal,          // Running total of PO quantity
    pub value_released: Decimal,             // Running total of PO value
    pub price_escalation_percent: Option<f64>, // Annual price adjustment
    pub price_escalation_cap: Option<f64>,
}

pub struct ProcurementContract {
    pub id: String,                         // PCTR-{year}-{seq}
    pub contract_type: ContractType,
    pub status: ContractStatus,
    pub vendor_id: String,
    pub category: SpendCategory,
    pub title: String,
    pub description: String,
    pub sourcing_project_id: Option<String>,
    pub rfx_event_id: Option<String>,
    pub winning_bid_id: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_value: Decimal,               // Contract ceiling
    pub total_released: Decimal,            // Sum of PO values
    pub utilization_percent: f64,           // released / ceiling
    pub currency: String,
    pub terms: ContractTerms,
    pub line_items: Vec<ContractLineItem>,
    pub amendment_count: u32,
    pub owner_employee_id: String,
    pub approved_by: Option<String>,
    pub approval_date: Option<NaiveDate>,
    pub predecessor_contract_id: Option<String>, // Renewed from
    pub purchase_order_ids: Vec<String>,     // POs issued against this contract
}
```

#### 3.1.8 `CatalogItem`

```rust
/// A purchasable item linked to a contract at an agreed price.
pub struct CatalogItem {
    pub id: String,                         // CAT-{material}-{vendor}
    pub material_id: String,
    pub vendor_id: String,
    pub contract_id: String,
    pub contract_line_item_id: String,
    pub description: String,
    pub unit_price: Decimal,
    pub currency: String,
    pub uom: String,
    pub lead_time_days: u32,
    pub moq: Option<Decimal>,               // Minimum order quantity
    pub is_preferred: bool,                 // Preferred/primary source
    pub valid_from: NaiveDate,
    pub valid_to: NaiveDate,
    pub category: SpendCategory,
}
```

#### 3.1.9 `SupplierScorecard`

```rust
/// Periodic supplier performance scorecard aggregating operational metrics.
pub struct SupplierScorecard {
    pub id: String,                         // SCARD-{vendor_id}-{period}
    pub vendor_id: String,
    pub evaluation_period: String,          // "2024-Q1", "2024"
    pub evaluation_date: NaiveDate,
    pub quality_score: VendorQualityScore,  // Reuse existing model
    pub contract_compliance: ContractComplianceMetrics,
    pub overall_score: f64,                 // 0.0-1.0
    pub grade: String,                      // A+, A, B+, B, C, D, F
    pub trend: ScoreboardTrend,
    pub recommendation: ScorecardRecommendation,
    pub evaluator_employee_id: Option<String>,
    pub notes: Option<String>,
}

pub struct ContractComplianceMetrics {
    pub sla_breaches: u32,
    pub sla_compliance_rate: f64,           // % of SLAs met
    pub contract_utilization: f64,          // % of contract value used
    pub price_compliance_rate: f64,         // % of invoices at contract price
    pub maverick_order_count: u32,          // Orders placed off-contract
}

pub enum ScoreboardTrend {
    Improving,
    Stable,
    Declining,
}

pub enum ScorecardRecommendation {
    Expand,                // Increase business
    Maintain,              // Continue as-is
    ReviewRequired,        // Performance review meeting needed
    ProbationaryPeriod,    // Formal warning
    PhaseOut,              // Begin transitioning away
}
```

### 3.2 Modifications to Existing Models

#### 3.2.1 `PurchaseRequisition` — New Fields

```rust
// Add to existing PurchaseRequisition struct:
pub catalog_item_id: Option<String>,        // If ordered from catalog
pub contract_id: Option<String>,            // Contract backing this PR
pub is_free_text: bool,                     // true = off-contract / maverick
pub sourcing_justification: Option<String>, // Required if free-text and above threshold
```

#### 3.2.2 `PurchaseOrder` — New Fields

```rust
// Add to existing PurchaseOrder struct:
pub contract_id: Option<String>,            // Governing procurement contract
pub contract_line_ids: Vec<String>,         // Mapped ContractLineItem IDs
pub is_contract_release: bool,              // true = release against framework/blanket
pub contract_price_validated: bool,         // PO price matches contract price
pub catalog_item_ids: Vec<String>,          // Catalog items referenced
```

#### 3.2.3 `VendorInvoice` — New Fields

```rust
// Add to existing VendorInvoice struct:
pub contract_id: Option<String>,            // Contract for price validation
pub contract_price_variance: Option<Decimal>, // Invoice price vs contract price
```

#### 3.2.4 `VendorRelationship` — New Fields

```rust
// Add to existing VendorRelationship struct:
pub qualification_status: Option<QualificationStatus>,
pub qualification_expiry: Option<NaiveDate>,
pub active_contract_ids: Vec<String>,
pub scorecard_grade: Option<String>,        // Latest scorecard grade
```

### 3.3 New Enums for `GraphEntityType` and `RelationshipType`

```rust
// Add to GraphEntityType in relationship.rs:
SourcingProject,
RfxEvent,
Contract,   // Already exists — reuse for procurement contracts

// Add to RelationshipType in relationship.rs:
AwardedTo,          // RfxEvent → Vendor
GovernsOrder,       // Contract → PurchaseOrder
EvaluatedBy,        // SupplierBid → BidEvaluation
QualifiedAs,        // Vendor → SupplierQualification
ScoredBy,           // Vendor → SupplierScorecard
SourcedThrough,     // PurchaseOrder → SourcingProject
CatalogItemOf,      // CatalogItem → Contract
```

---

## 4. Generator Design

### 4.1 New Generator Module: `datasynth-generators/src/sourcing/`

```
crates/datasynth-generators/src/sourcing/
├── mod.rs
├── spend_analysis_generator.rs
├── sourcing_project_generator.rs
├── qualification_generator.rs
├── rfx_generator.rs
├── bid_generator.rs
├── bid_evaluation_generator.rs
├── contract_generator.rs
├── catalog_generator.rs
└── scorecard_generator.rs
```

#### 4.1.1 Generation Order (DAG)

The S2C generators must run **before** P2P generators. Updated orchestration order:

```
Phase 0 (Master Data - existing):
  company_selector → coa_generator → user_generator → vendor_generator
                                                       ↓
Phase 1 (S2C - NEW):                        ┌─────────┘
  spend_analysis_generator                   ▼
       ↓                          qualification_generator
  sourcing_project_generator                 │
       ↓                                     │
  rfx_generator  ◀───────────────────────────┘
       ↓
  bid_generator
       ↓
  bid_evaluation_generator
       ↓
  contract_generator → catalog_generator
                                     │
Phase 2 (P2P - existing, modified):  │
  pr_generator ◀─────────────────────┘  (now references catalog/contracts)
       ↓
  p2p_generator  (PO now references contracts)
       ↓
  three_way_match  (now validates contract price too)
       ↓
  payment processing

Phase 3 (Feedback - NEW):
  scorecard_generator  (aggregates P2P performance into scorecards)
```

#### 4.1.2 Key Generation Logic

**Spend Analysis Generator:**
- Aggregates vendor pool by SpendCategory
- Calculates HHI concentration index
- Flags consolidation opportunities (HHI < 0.15 = fragmented)
- Computes contract coverage rate from active contracts
- Runs once at initialization, produces 1 analysis per active category

**Sourcing Project Generator:**
- Creates projects for categories with low contract coverage or expiring contracts
- `contract_renewal_horizon_days`: default 180 — generate resourcing projects for contracts expiring within window
- `new_category_sourcing_rate`: default 0.1 — 10% of categories trigger new sourcing per year
- Assigns procurement manager from employee pool

**Qualification Generator:**
- Runs qualification for all vendors invited to sourcing projects
- Configurable pass rates per criterion (default: 80-95% pass rate per criterion)
- Vendors in `VendorCluster::ReliableStrategic` get higher base scores
- Vendors in `VendorCluster::Problematic` get lower base scores
- Qualification validity: default 365 days

**RFx Generator:**
- Creates 1-2 RFx events per sourcing project (RFI first if `>$100K`, then RFP/RFQ)
- Invites 3-8 qualified vendors per event
- Response rate: 60-90% of invited vendors respond (varies by cluster)
- Timeline: publish → deadline = 14-45 days depending on RfxType
- Generates evaluation criteria with default weights:
  - Price: 35-45%
  - Quality: 20-30%
  - Delivery: 15-20%
  - Service/Support: 5-15%
  - Innovation/Sustainability: 0-10%

**Bid Generator:**
- Creates bids for responding vendors
- Pricing strategy by cluster:
  - `ReliableStrategic`: competitive pricing (95-105% of target)
  - `StandardOperational`: market pricing (100-115%)
  - `Transactional`: variable pricing (90-130%)
  - `Problematic`: often aggressive low-ball (80-100%) or non-responsive
- Lead times inversely correlated with price (higher price → shorter lead time)
- Partial bids: 5-15% of bids are partial

**Bid Evaluation Generator:**
- Scores each bid against RFx criteria
- Applies ScoringMethod rules (LowestPriceBest normalizes prices)
- Ranks bids by total weighted score
- Selects winner (highest score) and backup
- Computes savings vs. budget target

**Contract Generator:**
- Creates contract from winning bid
- Maps bid line items to contract line items
- Sets contract duration: 12-36 months (configurable)
- Inherits pricing, payment terms, Incoterms from winning bid
- Generates SLAs from vendor cluster profile:
  - `ReliableStrategic`: strict SLAs (95%+ OTD target)
  - `StandardOperational`: standard SLAs (90%+ OTD target)
  - `Transactional`: minimal SLAs
- Sets utilization to 0% initially (grows as POs are released)

**Catalog Generator:**
- Creates CatalogItem entries for each active ContractLineItem
- Links material → vendor → contract at agreed price
- Sets preferred flag for primary vendor per material

**Scorecard Generator:**
- Runs after P2P generation (Phase 3)
- Aggregates `VendorQualityScore` (existing) with contract compliance metrics
- Generates quarterly or annual scorecards
- Trend detection: compares current vs. previous period
- Recommendation based on score bands:
  - 0.85+: Expand
  - 0.70-0.84: Maintain
  - 0.55-0.69: ReviewRequired
  - 0.40-0.54: ProbationaryPeriod
  - <0.40: PhaseOut

### 4.2 Modifications to Existing P2P Generator

The existing `P2PGenerator` in `document_flow/p2p_generator.rs` needs modification to consume S2C outputs:

1. **Contract-backed POs**: When generating a PO, look up active contracts for the vendor+material. If a contract exists, populate `contract_id`, inherit contract price, set `is_contract_release = true`.

2. **Maverick spend**: Configurable `off_contract_rate` (default 15%) controls how many POs bypass contracts.

3. **Contract price validation**: During three-way match, also validate invoice price against contract price (new variance type: `ContractPriceVariance`).

4. **Contract utilization tracking**: After PO creation, increment `ContractLineItem.quantity_released` and `value_released`. Flag when utilization exceeds 80% (approaching ceiling).

### 4.3 OCPM Extension

New OCEL 2.0 activities for S2C process events:

| Activity | Object Types | State Transitions |
|----------|-------------|-------------------|
| create_sourcing_project | sourcing_project | None → planning |
| qualify_supplier | supplier_qualification | None → qualified/not_qualified |
| publish_rfx | rfx_event | draft → published |
| submit_bid | supplier_bid, rfx_event | None → submitted |
| evaluate_bids | bid_evaluation, rfx_event | published → under_evaluation |
| award_rfx | rfx_event, supplier_bid | under_evaluation → awarded |
| create_contract | procurement_contract | None → draft |
| activate_contract | procurement_contract | draft → active |
| renew_contract | procurement_contract | expiring → renewed |

New OCEL object types: `sourcing_project`, `rfx_event`, `supplier_bid`, `procurement_contract`, `supplier_qualification`

---

## 5. Configuration Schema

### 5.1 New Config Section: `source_to_pay`

```yaml
source_to_pay:
  enabled: true                             # Master switch for S2C + integrated S2P

  spend_analysis:
    enabled: true
    consolidation_hhi_threshold: 0.15       # Below = fragmented, flag for consolidation
    high_concentration_threshold: 0.25      # Above = concentrated risk
    contract_coverage_target: 0.80          # Target 80% of spend under contract

  sourcing:
    enabled: true
    projects_per_year: 10                   # Number of sourcing projects per year
    contract_renewal_horizon_days: 180      # Start resourcing when contract expires within
    new_category_sourcing_rate: 0.10        # % of categories triggering new sourcing
    average_project_duration_days: 90       # Average S2C cycle time

  qualification:
    enabled: true
    pass_rate: 0.82                         # Overall pass rate
    validity_days: 365
    criteria_weights:
      financial_stability: 0.15
      production_capacity: 0.15
      quality_certification: 0.20
      regulatory_compliance: 0.15
      delivery_capability: 0.20
      technical_capability: 0.15

  rfx:
    enabled: true
    rfi_threshold_amount: 100000.0          # RFI required if estimated spend > this
    invited_vendors_min: 3
    invited_vendors_max: 8
    response_rate: 0.75                     # % of invited vendors that respond
    allow_partial_bids: true
    partial_bid_rate: 0.10
    submission_window_days:
      rfi: 21
      rfq: 14
      rfp: 30
    evaluation_criteria_defaults:
      price_weight: 0.40
      quality_weight: 0.25
      delivery_weight: 0.20
      service_weight: 0.10
      sustainability_weight: 0.05

  contracts:
    enabled: true
    default_duration_months: 24
    min_duration_months: 12
    max_duration_months: 36
    auto_renewal_rate: 0.40                 # % of contracts with auto-renewal
    amendment_rate: 0.15                    # % of contracts that get amended
    price_escalation_rate: 0.30             # % with annual price escalation
    max_price_escalation_percent: 0.05
    utilization_warning_threshold: 0.80
    type_distribution:
      fixed_price: 0.35
      framework_agreement: 0.30
      blanket_order: 0.15
      service_agreement: 0.10
      consignment_agreement: 0.05
      rate_card: 0.05

  catalog:
    enabled: true
    preferred_vendor_per_material: true     # Flag one vendor as preferred per material
    multi_source_rate: 0.20                 # % of materials with 2+ catalog entries

  scorecards:
    enabled: true
    evaluation_frequency: quarterly         # quarterly, semi_annual, annual
    score_weights:
      delivery: 0.30
      quality: 0.30
      invoice_accuracy: 0.15
      responsiveness: 0.10
      contract_compliance: 0.15
    grade_thresholds:
      expand: 0.85
      maintain: 0.70
      review_required: 0.55
      probationary: 0.40

  # Integration with existing P2P
  p2p_integration:
    off_contract_rate: 0.15                 # % of POs without contract backing
    enforce_contract_pricing: true          # Validate PO price against contract
    contract_price_tolerance: 0.02          # 2% tolerance for contract price
    require_catalog_for_pr: false           # If true, block free-text PRs
    auto_source_from_catalog: true          # PRs auto-fill vendor/price from catalog
```

### 5.2 Additions to `GeneratorConfig`

```rust
// Add to GeneratorConfig struct in schema.rs:
/// Source-to-Pay configuration (S2C + P2P integration)
#[serde(default)]
pub source_to_pay: SourceToPayConfig,
```

---

## 6. Output Files

### 6.1 New Export Files

| Category | File | Format | Description |
|----------|------|--------|-------------|
| Spend Analysis | `spend_analysis.csv` | CSV/JSON | Category-level spend aggregation |
| Sourcing | `sourcing_projects.csv` | CSV/JSON | Sourcing initiatives |
| Qualification | `supplier_qualifications.csv` | CSV/JSON | Vendor qualification records |
| Qualification | `supplier_certifications.csv` | CSV/JSON | Vendor certifications |
| RFx | `rfx_events.csv` | CSV/JSON | RFP/RFQ/RFI events |
| Bids | `supplier_bids.csv` | CSV/JSON | Vendor bid submissions |
| Bids | `bid_evaluations.csv` | CSV/JSON | Evaluation scores and rankings |
| Contracts | `procurement_contracts.csv` | CSV/JSON | Contract headers |
| Contracts | `contract_line_items.csv` | CSV/JSON | Contract line items with pricing |
| Contracts | `contract_slas.csv` | CSV/JSON | SLA definitions per contract |
| Catalog | `catalog_items.csv` | CSV/JSON | Purchasable catalog entries |
| Performance | `supplier_scorecards.csv` | CSV/JSON | Periodic performance scorecards |

### 6.2 Modified Export Files

| Existing File | New Columns Added |
|---------------|-------------------|
| `purchase_orders.csv` | `contract_id`, `is_contract_release`, `contract_price_validated` |
| `purchase_requisitions.csv` | `catalog_item_id`, `contract_id`, `is_free_text` |
| `vendor_invoices.csv` | `contract_id`, `contract_price_variance` |
| `vendors.csv` | `qualification_status`, `qualification_expiry`, `scorecard_grade` |

### 6.3 OCPM Extensions

| Existing File | Changes |
|---------------|---------|
| `event_log.json` | New S2C activities and object types added |
| `objects.json` | New object types: sourcing_project, rfx_event, supplier_bid, procurement_contract |
| `process_variants.json` | New S2C variant patterns |

---

## 7. Coherence Rules & Validation

The following invariants ensure the S2P chain is coherent end-to-end:

### 7.1 Referential Integrity

| Rule ID | Constraint |
|---------|-----------|
| S2P-001 | Every `SourcingProject.awarded_vendor_id` must exist in the vendor pool |
| S2P-002 | Every `RfxEvent.sourcing_project_id` must reference a valid `SourcingProject` |
| S2P-003 | Every `SupplierBid.rfx_event_id` must reference a valid `RfxEvent` |
| S2P-004 | Every `SupplierBid.vendor_id` must appear in the RfxEvent's `invited_vendor_ids` |
| S2P-005 | Every `ProcurementContract.vendor_id` must match the winning bid's vendor |
| S2P-006 | Every `CatalogItem.contract_id` must reference an `Active` contract |
| S2P-007 | Every `PurchaseOrder.contract_id` (when set) must reference an `Active` contract |
| S2P-008 | PO price must be within `contract_price_tolerance` of the contract line price |
| S2P-009 | Contract `total_released` must equal sum of PO values against that contract |
| S2P-010 | Contract `utilization_percent` must equal `total_released / total_value` |

### 7.2 Temporal Consistency

| Rule ID | Constraint |
|---------|-----------|
| S2P-011 | `SpendAnalysis.analysis_date` < `SourcingProject.start_date` |
| S2P-012 | `SupplierQualification.qualification_date` < `RfxEvent.publish_date` |
| S2P-013 | `RfxEvent.publish_date` < `submission_deadline` < `evaluation_deadline` |
| S2P-014 | `SupplierBid.submission_date` ≤ `RfxEvent.submission_deadline` |
| S2P-015 | `ProcurementContract.start_date` > `BidEvaluation.evaluation_date` |
| S2P-016 | `ProcurementContract.start_date` < `PurchaseOrder.document_date` < `Contract.end_date` |
| S2P-017 | `CatalogItem.valid_from` ≥ `ProcurementContract.start_date` |
| S2P-018 | `CatalogItem.valid_to` ≤ `ProcurementContract.end_date` |

### 7.3 Business Logic Constraints

| Rule ID | Constraint |
|---------|-----------|
| S2P-019 | Only `Qualified` or `ConditionallyQualified` vendors can be invited to RFx |
| S2P-020 | Only `Submitted` bids can be evaluated |
| S2P-021 | `BidEvaluation.ranked_bids` must be sorted by `total_weighted_score` descending |
| S2P-022 | Exactly one bid per RfxEvent may have `AwardRecommendation::PrimaryAward` |
| S2P-023 | `off_contract_rate` must match actual ratio of POs without contract_id |
| S2P-024 | `ContractLineItem.quantity_released` must not exceed `max_quantity` (if set) unless over-release tolerance allows it |
| S2P-025 | Vendor with `ScorecardRecommendation::PhaseOut` should not appear in new RFx events |

### 7.4 Statistical Validation (for `datasynth-eval`)

| Metric | Expected Property |
|--------|-------------------|
| Contract coverage rate | Should approximate `contract_coverage_target` (±5%) |
| Off-contract (maverick) spend | Should approximate `off_contract_rate` (±3%) |
| Bid response rate | Should approximate configured `response_rate` (±5%) |
| Qualification pass rate | Should approximate configured `pass_rate` (±5%) |
| Contract utilization | Should follow log-normal distribution (many contracts underutilized, few fully consumed) |
| Bid pricing distribution | Should vary by vendor cluster as specified |

---

## 8. Anomaly Injection Points

Extending the existing anomaly framework for S2C:

| Anomaly Type | Category | Description |
|-------------|----------|-------------|
| `BidRigging` | Fraud | Coordinated bids with suspiciously similar prices or rotating winners |
| `PhantomVendor` | Fraud | Contract awarded to vendor with no qualification record |
| `SplitToAvoidThreshold` | Fraud | Multiple small contracts to avoid approval thresholds |
| `ConflictOfInterest` | Fraud | Sourcing project owner is related to winning vendor |
| `MaverickSpend` | Process | PO issued without contract backing above threshold |
| `ExpiredContractPurchase` | Process | PO references an expired contract |
| `ContractPriceOverride` | Process | PO price deviates from contract price beyond tolerance |
| `SingleBidAward` | Process | RFx with only one bid (competitive concern) |
| `SlaBreachPattern` | Statistical | Vendor repeatedly breaches SLAs but contract is renewed |
| `UnusedContract` | Statistical | Contract with <10% utilization at expiry |
| `QualificationBypass` | Process | Vendor awarded without valid qualification |

---

## 9. Implementation Plan

### Phase 1: Core Models (datasynth-core) — Estimated: ~15 files

| # | Task | Files | Dependencies |
|---|------|-------|-------------|
| 1.1 | Create `models/sourcing/mod.rs` module directory | New module dir | None |
| 1.2 | Implement `SpendAnalysis`, `VendorSpendShare` | `spend_analysis.rs` | SpendCategory (existing) |
| 1.3 | Implement `SourcingProject`, status/type enums | `sourcing_project.rs` | None |
| 1.4 | Implement `SupplierQualification`, `SupplierCertification`, criteria enums | `qualification.rs` | None |
| 1.5 | Implement `RfxEvent`, `RfxLineItem`, `RfxEvaluationCriterion` | `rfx.rs` | None |
| 1.6 | Implement `SupplierBid`, `BidLineItem` | `bid.rs` | None |
| 1.7 | Implement `BidEvaluation`, `RankedBid` | `bid_evaluation.rs` | None |
| 1.8 | Implement `ProcurementContract`, `ContractTerms`, `ContractSla`, `ContractLineItem` | `contract.rs` | None |
| 1.9 | Implement `CatalogItem` | `catalog.rs` | None |
| 1.10 | Implement `SupplierScorecard`, `ContractComplianceMetrics` | `scorecard.rs` | VendorQualityScore (existing) |
| 1.11 | Add new fields to `PurchaseRequisition` | `documents/purchase_requisition.rs` | 1.9 |
| 1.12 | Add new fields to `PurchaseOrder` | `documents/purchase_order.rs` | 1.8 |
| 1.13 | Add new fields to `VendorInvoice` | `documents/vendor_invoice.rs` | 1.8 |
| 1.14 | Extend `GraphEntityType` and `RelationshipType` | `relationship.rs` | None |
| 1.15 | Register new model exports in `models/mod.rs` | `models/mod.rs` | 1.1-1.10 |

### Phase 2: Configuration (datasynth-config) — Estimated: ~2 files

| # | Task | Files | Dependencies |
|---|------|-------|-------------|
| 2.1 | Define `SourceToPayConfig` and all sub-configs | `schema.rs` | Phase 1 |
| 2.2 | Add `source_to_pay` field to `GeneratorConfig` | `schema.rs` | 2.1 |
| 2.3 | Add S2P presets (manufacturing, retail, etc.) | `presets/` | 2.1 |
| 2.4 | Add config validation rules (sum-to-1 checks, range checks) | `validation.rs` | 2.1 |

### Phase 3: Generators (datasynth-generators) — Estimated: ~12 files

| # | Task | Files | Dependencies |
|---|------|-------|-------------|
| 3.1 | Create `sourcing/mod.rs` module | New module | Phase 1, 2 |
| 3.2 | Implement `SpendAnalysisGenerator` | `sourcing/spend_analysis_generator.rs` | Vendor pool, CoA |
| 3.3 | Implement `SourcingProjectGenerator` | `sourcing/sourcing_project_generator.rs` | 3.2 |
| 3.4 | Implement `QualificationGenerator` | `sourcing/qualification_generator.rs` | Vendor pool |
| 3.5 | Implement `RfxGenerator` | `sourcing/rfx_generator.rs` | 3.3, 3.4 |
| 3.6 | Implement `BidGenerator` | `sourcing/bid_generator.rs` | 3.5 |
| 3.7 | Implement `BidEvaluationGenerator` | `sourcing/bid_evaluation_generator.rs` | 3.6 |
| 3.8 | Implement `ContractGenerator` | `sourcing/contract_generator.rs` | 3.7 |
| 3.9 | Implement `CatalogGenerator` | `sourcing/catalog_generator.rs` | 3.8 |
| 3.10 | Implement `ScorecardGenerator` | `sourcing/scorecard_generator.rs` | P2P results |
| 3.11 | Modify `P2PGenerator` for contract integration | `document_flow/p2p_generator.rs` | 3.8, 3.9 |
| 3.12 | Extend `ThreeWayMatcher` for contract price check | `document_flow/three_way_match.rs` | 3.8 |

### Phase 4: Orchestration (datasynth-runtime) — Estimated: ~2 files

| # | Task | Files | Dependencies |
|---|------|-------|-------------|
| 4.1 | Add S2C phase to `GenerationOrchestrator` | `orchestrator.rs` | Phase 3 |
| 4.2 | Wire S2C outputs as inputs to P2P generators | `orchestrator.rs` | 4.1 |
| 4.3 | Add scorecard phase after P2P completion | `orchestrator.rs` | 3.10 |

### Phase 5: Output (datasynth-output) — Estimated: ~2 files

| # | Task | Files | Dependencies |
|---|------|-------|-------------|
| 5.1 | Add CSV/JSON serializers for new S2C models | `sinks/` | Phase 1 |
| 5.2 | Update existing PO/PR/Invoice serializers for new fields | `sinks/` | Phase 1 |

### Phase 6: OCPM Extension (datasynth-ocpm) — Estimated: ~2 files

| # | Task | Files | Dependencies |
|---|------|-------|-------------|
| 6.1 | Add S2C object types and activities | `generator/s2c_generator.rs` | Phase 1 |
| 6.2 | Link S2C events to P2P events in OCEL 2.0 log | `generator/p2p_generator.rs` | 6.1 |

### Phase 7: Anomaly & Evaluation — Estimated: ~3 files

| # | Task | Files | Dependencies |
|---|------|-------|-------------|
| 7.1 | Add S2C anomaly strategies | `datasynth-generators/src/anomaly/` | Phase 3 |
| 7.2 | Add S2P coherence validators | `datasynth-eval/src/coherence/` | Phase 3 |
| 7.3 | Add contract coverage / maverick spend metrics | `datasynth-eval/src/statistical/` | Phase 3 |

### Phase 8: Testing — Estimated: ~5 files

| # | Task | Files | Dependencies |
|---|------|-------|-------------|
| 8.1 | Unit tests for all new models (serde round-trip, invariants) | `datasynth-core/tests/` | Phase 1 |
| 8.2 | Unit tests for each S2C generator | `datasynth-generators/tests/` | Phase 3 |
| 8.3 | Integration test: full S2P chain end-to-end | `datasynth-runtime/tests/` | Phase 4 |
| 8.4 | Coherence validation tests (S2P-001 through S2P-025) | `datasynth-eval/tests/` | Phase 7 |
| 8.5 | Config preset tests for S2P | `datasynth-config/tests/` | Phase 2 |

---

## 10. Industry Preset Profiles

The existing preset system (manufacturing, retail, financial_services, healthcare, technology) should include S2P defaults:

### Manufacturing

```yaml
source_to_pay:
  enabled: true
  sourcing:
    projects_per_year: 20
    contract_renewal_horizon_days: 180
  qualification:
    pass_rate: 0.78        # Stricter due to quality requirements
    criteria_weights:
      quality_certification: 0.25
      production_capacity: 0.20
      delivery_capability: 0.20
  rfx:
    evaluation_criteria_defaults:
      price_weight: 0.30
      quality_weight: 0.35  # Quality paramount in manufacturing
      delivery_weight: 0.25
      service_weight: 0.10
  contracts:
    default_duration_months: 24
    type_distribution:
      framework_agreement: 0.40  # High framework usage
      fixed_price: 0.30
      consignment_agreement: 0.15
      blanket_order: 0.10
      service_agreement: 0.05
  p2p_integration:
    off_contract_rate: 0.10  # Low maverick spend
```

### Retail

```yaml
source_to_pay:
  enabled: true
  sourcing:
    projects_per_year: 30     # More sourcing activity
  qualification:
    pass_rate: 0.85
  rfx:
    evaluation_criteria_defaults:
      price_weight: 0.45      # Price-sensitive industry
      quality_weight: 0.20
      delivery_weight: 0.25
      service_weight: 0.10
  contracts:
    default_duration_months: 12  # Shorter contracts, frequent rebidding
    type_distribution:
      blanket_order: 0.35
      framework_agreement: 0.25
      fixed_price: 0.25
      service_agreement: 0.10
      rate_card: 0.05
  p2p_integration:
    off_contract_rate: 0.20  # Higher maverick spend
```

### Financial Services

```yaml
source_to_pay:
  enabled: true
  sourcing:
    projects_per_year: 12
  qualification:
    pass_rate: 0.75            # Strict due to regulatory requirements
    criteria_weights:
      regulatory_compliance: 0.25
      information_security: 0.25
      financial_stability: 0.20
  rfx:
    evaluation_criteria_defaults:
      price_weight: 0.25
      quality_weight: 0.25
      delivery_weight: 0.15
      service_weight: 0.20    # Service quality matters
      sustainability_weight: 0.15
  contracts:
    default_duration_months: 36  # Longer, stable relationships
    type_distribution:
      service_agreement: 0.40   # Services-heavy
      rate_card: 0.20
      fixed_price: 0.20
      framework_agreement: 0.15
      blanket_order: 0.05
  p2p_integration:
    off_contract_rate: 0.08    # Tightly controlled
```

---

## 11. Risk Considerations

| Risk | Mitigation |
|------|------------|
| S2C generation adds to total runtime | S2C phase generates far fewer records than P2P (hundreds vs. millions); minimal impact expected |
| Memory overhead from cross-references | Use ID strings (not cloned structs) for FK references; contract/catalog pools are small |
| Breaking existing P2P tests | All new PO/PR/Invoice fields are `Option<>` with `None` defaults; backward compatible |
| Config complexity | S2P config is off by default (`enabled: false`); existing users unaffected |
| Coherence validation overhead | Validation is opt-in via `datasynth-eval`; not part of generation hot path |

---

## 12. Summary

| Metric | Value |
|--------|-------|
| New model structs | ~20 |
| New enums | ~15 |
| Modified existing structs | 4 (PR, PO, Invoice, VendorRelationship) |
| New generator files | ~10 |
| Modified generator files | ~2 (P2P generator, three-way match) |
| New config sections | 1 top-level (`source_to_pay`) with 8 sub-sections |
| New export files | 12 |
| Modified export files | 4 |
| New coherence rules | 25 |
| New anomaly types | 11 |
| New OCPM activities | 9 |
| Implementation phases | 8 |
