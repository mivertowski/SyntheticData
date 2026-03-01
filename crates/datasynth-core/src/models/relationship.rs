//! Entity relationship graph models.
//!
//! Provides comprehensive relationship modeling including:
//! - Entity graph with typed nodes and edges
//! - Relationship strength calculation
//! - Cross-process linkages (P2P ↔ O2C via inventory)
//! - Network analysis support

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Type of entity in the relationship graph.
///
/// This is separate from `entity_registry::EntityType` as it represents
/// the entity types specifically used in graph/network analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEntityType {
    /// Company/legal entity
    Company,
    /// Vendor/supplier
    Vendor,
    /// Customer
    Customer,
    /// Employee
    Employee,
    /// Department
    Department,
    /// Cost center
    CostCenter,
    /// Project
    Project,
    /// Contract
    Contract,
    /// Fixed asset
    Asset,
    /// Bank account
    BankAccount,
    /// Material/inventory item
    Material,
    /// GL account
    GlAccount,
    /// Purchase order
    PurchaseOrder,
    /// Sales order
    SalesOrder,
    /// Invoice
    Invoice,
    /// Payment
    Payment,
    /// Sourcing project
    SourcingProject,
    /// RFx event
    RfxEvent,
    /// Production order
    ProductionOrder,
    /// Bank reconciliation
    BankReconciliation,

    // ===== Tax entities (DS-001, DS-002) =====
    /// Tax jurisdiction
    TaxJurisdiction,
    /// Tax code
    TaxCode,
    /// Tax line item
    TaxLine,
    /// Tax return
    TaxReturn,
    /// Tax provision
    TaxProvision,
    /// Withholding tax record
    WithholdingTaxRecord,
    /// Uncertain tax position
    UncertainTaxPosition,

    // ===== Treasury entities (DS-003) =====
    /// Cash position
    CashPosition,
    /// Cash forecast
    CashForecast,
    /// Cash pool
    CashPool,
    /// Cash pool sweep
    CashPoolSweep,
    /// Hedging instrument
    HedgingInstrument,
    /// Hedge relationship
    HedgeRelationship,
    /// Debt instrument
    DebtInstrument,
    /// Debt covenant
    DebtCovenant,

    // ===== ESG entities (DS-004) =====
    /// Emission record
    EmissionRecord,
    /// Energy consumption
    EnergyConsumption,
    /// Water usage
    WaterUsage,
    /// Waste record
    WasteRecord,
    /// Workforce diversity metric
    WorkforceDiversityMetric,
    /// Pay equity metric
    PayEquityMetric,
    /// Safety incident
    SafetyIncident,
    /// Safety metric
    SafetyMetric,
    /// Governance metric
    GovernanceMetric,
    /// Supplier ESG assessment
    SupplierEsgAssessment,
    /// Materiality assessment
    MaterialityAssessment,
    /// ESG disclosure
    EsgDisclosure,
    /// Climate scenario
    ClimateScenario,

    // ===== Project entities (DS-005) =====
    /// Project cost line
    ProjectCostLine,
    /// Project revenue
    ProjectRevenue,
    /// Earned value metric
    EarnedValueMetric,
    /// Change order
    ChangeOrder,
    /// Project milestone
    ProjectMilestone,

    // ===== S2C entities (DS-006) =====
    /// Supplier bid
    SupplierBid,
    /// Bid evaluation
    BidEvaluation,
    /// Procurement contract
    ProcurementContract,
    /// Supplier qualification
    SupplierQualification,

    // ===== H2R entities (DS-007) =====
    /// Payroll run
    PayrollRun,
    /// Time entry
    TimeEntry,
    /// Expense report
    ExpenseReport,
    /// Benefit enrollment
    BenefitEnrollment,

    // ===== MFG entities (DS-008) =====
    /// Quality inspection
    QualityInspection,
    /// Cycle count
    CycleCount,
    /// BOM component
    BomComponent,
    /// Inventory movement
    InventoryMovement,

    // ===== GOV entities (DS-009) =====
    /// COSO component
    CosoComponent,
    /// COSO principle
    CosoPrinciple,
    /// SOX assertion
    SoxAssertion,
    /// Audit engagement
    AuditEngagement,
    /// Professional judgment
    ProfessionalJudgment,
}

impl GraphEntityType {
    /// Get the entity type code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Company => "CO",
            Self::Vendor => "VN",
            Self::Customer => "CU",
            Self::Employee => "EM",
            Self::Department => "DP",
            Self::CostCenter => "CC",
            Self::Project => "PJ",
            Self::Contract => "CT",
            Self::Asset => "AS",
            Self::BankAccount => "BA",
            Self::Material => "MT",
            Self::GlAccount => "GL",
            Self::PurchaseOrder => "PO",
            Self::SalesOrder => "SO",
            Self::Invoice => "IV",
            Self::Payment => "PM",
            Self::SourcingProject => "SP",
            Self::RfxEvent => "RX",
            Self::ProductionOrder => "PR",
            Self::BankReconciliation => "BR",
            // Tax
            Self::TaxJurisdiction => "TJ",
            Self::TaxCode => "TC",
            Self::TaxLine => "TL",
            Self::TaxReturn => "TR",
            Self::TaxProvision => "TP",
            Self::WithholdingTaxRecord => "WH",
            Self::UncertainTaxPosition => "UT",
            // Treasury
            Self::CashPosition => "CP",
            Self::CashForecast => "CF",
            Self::CashPool => "CL",
            Self::CashPoolSweep => "CS",
            Self::HedgingInstrument => "HI",
            Self::HedgeRelationship => "HR",
            Self::DebtInstrument => "DI",
            Self::DebtCovenant => "DC",
            // ESG
            Self::EmissionRecord => "ER",
            Self::EnergyConsumption => "EC",
            Self::WaterUsage => "WU",
            Self::WasteRecord => "WR",
            Self::WorkforceDiversityMetric => "WD",
            Self::PayEquityMetric => "PE",
            Self::SafetyIncident => "SI",
            Self::SafetyMetric => "SM",
            Self::GovernanceMetric => "GM",
            Self::SupplierEsgAssessment => "SE",
            Self::MaterialityAssessment => "MA",
            Self::EsgDisclosure => "ED",
            Self::ClimateScenario => "CZ",
            // Project
            Self::ProjectCostLine => "PL",
            Self::ProjectRevenue => "PV",
            Self::EarnedValueMetric => "EV",
            Self::ChangeOrder => "CR",
            Self::ProjectMilestone => "MS",
            // S2C
            Self::SupplierBid => "BD",
            Self::BidEvaluation => "BE",
            Self::ProcurementContract => "PC",
            Self::SupplierQualification => "SQ",
            // H2R
            Self::PayrollRun => "PY",
            Self::TimeEntry => "TE",
            Self::ExpenseReport => "EX",
            Self::BenefitEnrollment => "BN",
            // MFG
            Self::QualityInspection => "QI",
            Self::CycleCount => "CY",
            Self::BomComponent => "BC",
            Self::InventoryMovement => "IM",
            // GOV
            Self::CosoComponent => "GC",
            Self::CosoPrinciple => "GP",
            Self::SoxAssertion => "SA",
            Self::AuditEngagement => "AE",
            Self::ProfessionalJudgment => "JD",
        }
    }

    /// Numeric entity type code for registry.
    pub fn numeric_code(&self) -> u16 {
        match self {
            // Existing types (100-series)
            Self::Company => 100,
            Self::Vendor => 101,
            Self::Material => 102,
            Self::Customer => 103,
            Self::Employee => 104,
            Self::InventoryMovement => 105,
            Self::GlAccount => 106,
            Self::Department => 107,
            Self::CostCenter => 108,
            Self::Project => 109,
            Self::Contract => 110,
            Self::Asset => 111,
            Self::BankAccount => 112,
            Self::PurchaseOrder => 200,
            Self::SalesOrder => 201,
            Self::Invoice => 202,
            Self::Payment => 203,
            Self::BankReconciliation => 210,
            // S2C
            Self::SourcingProject => 320,
            Self::RfxEvent => 321,
            Self::SupplierBid => 322,
            Self::BidEvaluation => 323,
            Self::ProcurementContract => 324,
            Self::SupplierQualification => 325,
            // H2R
            Self::PayrollRun => 330,
            Self::TimeEntry => 331,
            Self::ExpenseReport => 332,
            Self::BenefitEnrollment => 333,
            // MFG
            Self::ProductionOrder => 340,
            Self::QualityInspection => 341,
            Self::CycleCount => 342,
            Self::BomComponent => 343,
            // Tax
            Self::TaxJurisdiction => 410,
            Self::TaxCode => 411,
            Self::TaxLine => 412,
            Self::TaxReturn => 413,
            Self::TaxProvision => 414,
            Self::WithholdingTaxRecord => 415,
            Self::UncertainTaxPosition => 416,
            // Treasury
            Self::CashPosition => 420,
            Self::CashForecast => 421,
            Self::CashPool => 422,
            Self::CashPoolSweep => 423,
            Self::HedgingInstrument => 424,
            Self::HedgeRelationship => 425,
            Self::DebtInstrument => 426,
            Self::DebtCovenant => 427,
            // ESG
            Self::EmissionRecord => 430,
            Self::EnergyConsumption => 431,
            Self::WaterUsage => 432,
            Self::WasteRecord => 433,
            Self::WorkforceDiversityMetric => 434,
            Self::PayEquityMetric => 435,
            Self::SafetyIncident => 436,
            Self::SafetyMetric => 437,
            Self::GovernanceMetric => 438,
            Self::SupplierEsgAssessment => 439,
            Self::MaterialityAssessment => 440,
            Self::EsgDisclosure => 441,
            Self::ClimateScenario => 442,
            // Project
            Self::ProjectCostLine => 451,
            Self::ProjectRevenue => 452,
            Self::EarnedValueMetric => 453,
            Self::ChangeOrder => 454,
            Self::ProjectMilestone => 455,
            // GOV
            Self::CosoComponent => 500,
            Self::CosoPrinciple => 501,
            Self::SoxAssertion => 502,
            Self::AuditEngagement => 360,
            Self::ProfessionalJudgment => 365,
        }
    }

    /// Entity type name (snake_case) for graph export.
    pub fn node_type_name(&self) -> &'static str {
        match self {
            Self::Company => "company",
            Self::Vendor => "vendor",
            Self::Customer => "customer",
            Self::Employee => "employee",
            Self::Department => "department",
            Self::CostCenter => "cost_center",
            Self::Project => "project",
            Self::Contract => "contract",
            Self::Asset => "asset",
            Self::BankAccount => "bank_account",
            Self::Material => "material",
            Self::GlAccount => "gl_account",
            Self::PurchaseOrder => "purchase_order",
            Self::SalesOrder => "sales_order",
            Self::Invoice => "invoice",
            Self::Payment => "payment",
            Self::SourcingProject => "sourcing_project",
            Self::RfxEvent => "rfx_event",
            Self::ProductionOrder => "production_order",
            Self::BankReconciliation => "bank_reconciliation",
            // Tax
            Self::TaxJurisdiction => "tax_jurisdiction",
            Self::TaxCode => "tax_code",
            Self::TaxLine => "tax_line",
            Self::TaxReturn => "tax_return",
            Self::TaxProvision => "tax_provision",
            Self::WithholdingTaxRecord => "withholding_tax_record",
            Self::UncertainTaxPosition => "uncertain_tax_position",
            // Treasury
            Self::CashPosition => "cash_position",
            Self::CashForecast => "cash_forecast",
            Self::CashPool => "cash_pool",
            Self::CashPoolSweep => "cash_pool_sweep",
            Self::HedgingInstrument => "hedging_instrument",
            Self::HedgeRelationship => "hedge_relationship",
            Self::DebtInstrument => "debt_instrument",
            Self::DebtCovenant => "debt_covenant",
            // ESG
            Self::EmissionRecord => "emission_record",
            Self::EnergyConsumption => "energy_consumption",
            Self::WaterUsage => "water_usage",
            Self::WasteRecord => "waste_record",
            Self::WorkforceDiversityMetric => "workforce_diversity_metric",
            Self::PayEquityMetric => "pay_equity_metric",
            Self::SafetyIncident => "safety_incident",
            Self::SafetyMetric => "safety_metric",
            Self::GovernanceMetric => "governance_metric",
            Self::SupplierEsgAssessment => "supplier_esg_assessment",
            Self::MaterialityAssessment => "materiality_assessment",
            Self::EsgDisclosure => "esg_disclosure",
            Self::ClimateScenario => "climate_scenario",
            // Project
            Self::ProjectCostLine => "project_cost_line",
            Self::ProjectRevenue => "project_revenue",
            Self::EarnedValueMetric => "earned_value_metric",
            Self::ChangeOrder => "change_order",
            Self::ProjectMilestone => "project_milestone",
            // S2C
            Self::SupplierBid => "supplier_bid",
            Self::BidEvaluation => "bid_evaluation",
            Self::ProcurementContract => "procurement_contract",
            Self::SupplierQualification => "supplier_qualification",
            // H2R
            Self::PayrollRun => "payroll_run",
            Self::TimeEntry => "time_entry",
            Self::ExpenseReport => "expense_report",
            Self::BenefitEnrollment => "benefit_enrollment",
            // MFG
            Self::QualityInspection => "quality_inspection",
            Self::CycleCount => "cycle_count",
            Self::BomComponent => "bom_component",
            Self::InventoryMovement => "inventory_movement",
            // GOV
            Self::CosoComponent => "coso_component",
            Self::CosoPrinciple => "coso_principle",
            Self::SoxAssertion => "sox_assertion",
            Self::AuditEngagement => "audit_engagement",
            Self::ProfessionalJudgment => "professional_judgment",
        }
    }

    /// Lookup entity type by numeric code.
    pub fn from_numeric_code(code: u16) -> Option<Self> {
        Self::all_types().iter().find(|t| t.numeric_code() == code).copied()
    }

    /// Lookup entity type by snake_case name.
    pub fn from_node_type_name(name: &str) -> Option<Self> {
        Self::all_types().iter().find(|t| t.node_type_name() == name).copied()
    }

    /// All registered entity types.
    pub fn all_types() -> &'static [GraphEntityType] {
        &[
            // Original
            Self::Company,
            Self::Vendor,
            Self::Customer,
            Self::Employee,
            Self::Department,
            Self::CostCenter,
            Self::Project,
            Self::Contract,
            Self::Asset,
            Self::BankAccount,
            Self::Material,
            Self::GlAccount,
            Self::PurchaseOrder,
            Self::SalesOrder,
            Self::Invoice,
            Self::Payment,
            Self::SourcingProject,
            Self::RfxEvent,
            Self::ProductionOrder,
            Self::BankReconciliation,
            // Tax
            Self::TaxJurisdiction,
            Self::TaxCode,
            Self::TaxLine,
            Self::TaxReturn,
            Self::TaxProvision,
            Self::WithholdingTaxRecord,
            Self::UncertainTaxPosition,
            // Treasury
            Self::CashPosition,
            Self::CashForecast,
            Self::CashPool,
            Self::CashPoolSweep,
            Self::HedgingInstrument,
            Self::HedgeRelationship,
            Self::DebtInstrument,
            Self::DebtCovenant,
            // ESG
            Self::EmissionRecord,
            Self::EnergyConsumption,
            Self::WaterUsage,
            Self::WasteRecord,
            Self::WorkforceDiversityMetric,
            Self::PayEquityMetric,
            Self::SafetyIncident,
            Self::SafetyMetric,
            Self::GovernanceMetric,
            Self::SupplierEsgAssessment,
            Self::MaterialityAssessment,
            Self::EsgDisclosure,
            Self::ClimateScenario,
            // Project
            Self::ProjectCostLine,
            Self::ProjectRevenue,
            Self::EarnedValueMetric,
            Self::ChangeOrder,
            Self::ProjectMilestone,
            // S2C
            Self::SupplierBid,
            Self::BidEvaluation,
            Self::ProcurementContract,
            Self::SupplierQualification,
            // H2R
            Self::PayrollRun,
            Self::TimeEntry,
            Self::ExpenseReport,
            Self::BenefitEnrollment,
            // MFG
            Self::QualityInspection,
            Self::CycleCount,
            Self::BomComponent,
            Self::InventoryMovement,
            // GOV
            Self::CosoComponent,
            Self::CosoPrinciple,
            Self::SoxAssertion,
            Self::AuditEngagement,
            Self::ProfessionalJudgment,
        ]
    }

    /// Check if this is a tax entity.
    pub fn is_tax(&self) -> bool {
        matches!(
            self,
            Self::TaxJurisdiction
                | Self::TaxCode
                | Self::TaxLine
                | Self::TaxReturn
                | Self::TaxProvision
                | Self::WithholdingTaxRecord
                | Self::UncertainTaxPosition
        )
    }

    /// Check if this is a treasury entity.
    pub fn is_treasury(&self) -> bool {
        matches!(
            self,
            Self::CashPosition
                | Self::CashForecast
                | Self::CashPool
                | Self::CashPoolSweep
                | Self::HedgingInstrument
                | Self::HedgeRelationship
                | Self::DebtInstrument
                | Self::DebtCovenant
        )
    }

    /// Check if this is an ESG entity.
    pub fn is_esg(&self) -> bool {
        matches!(
            self,
            Self::EmissionRecord
                | Self::EnergyConsumption
                | Self::WaterUsage
                | Self::WasteRecord
                | Self::WorkforceDiversityMetric
                | Self::PayEquityMetric
                | Self::SafetyIncident
                | Self::SafetyMetric
                | Self::GovernanceMetric
                | Self::SupplierEsgAssessment
                | Self::MaterialityAssessment
                | Self::EsgDisclosure
                | Self::ClimateScenario
        )
    }

    /// Check if this is a project entity.
    pub fn is_project(&self) -> bool {
        matches!(
            self,
            Self::Project
                | Self::ProjectCostLine
                | Self::ProjectRevenue
                | Self::EarnedValueMetric
                | Self::ChangeOrder
                | Self::ProjectMilestone
        )
    }

    /// Check if this is an H2R (hire-to-retire) entity.
    pub fn is_h2r(&self) -> bool {
        matches!(
            self,
            Self::PayrollRun | Self::TimeEntry | Self::ExpenseReport | Self::BenefitEnrollment
        )
    }

    /// Check if this is a manufacturing entity.
    pub fn is_mfg(&self) -> bool {
        matches!(
            self,
            Self::ProductionOrder
                | Self::QualityInspection
                | Self::CycleCount
                | Self::BomComponent
                | Self::Material
                | Self::InventoryMovement
        )
    }

    /// Check if this is a governance entity.
    pub fn is_governance(&self) -> bool {
        matches!(
            self,
            Self::CosoComponent
                | Self::CosoPrinciple
                | Self::SoxAssertion
                | Self::AuditEngagement
                | Self::ProfessionalJudgment
        )
    }

    /// Check if this is a master data entity.
    pub fn is_master_data(&self) -> bool {
        matches!(
            self,
            Self::Company
                | Self::Vendor
                | Self::Customer
                | Self::Employee
                | Self::Department
                | Self::CostCenter
                | Self::Material
                | Self::GlAccount
                | Self::TaxJurisdiction
                | Self::TaxCode
                | Self::BankAccount
        )
    }

    /// Check if this is a transactional entity.
    pub fn is_transactional(&self) -> bool {
        matches!(
            self,
            Self::PurchaseOrder
                | Self::SalesOrder
                | Self::Invoice
                | Self::Payment
                | Self::TaxLine
                | Self::TaxReturn
                | Self::CashPoolSweep
                | Self::InventoryMovement
        )
    }
}

/// Type of relationship between entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    // ===== Transactional relationships =====
    /// Entity buys from another entity
    BuysFrom,
    /// Entity sells to another entity
    SellsTo,
    /// Entity pays to another entity
    PaysTo,
    /// Entity receives payment from another entity
    ReceivesFrom,
    /// Supplies goods to
    SuppliesTo,
    /// Sources goods from
    SourcesFrom,

    // ===== Organizational relationships =====
    /// Employee reports to manager
    ReportsTo,
    /// Manager manages employee
    Manages,
    /// Entity belongs to parent entity
    BelongsTo,
    /// Entity owned by another entity
    OwnedBy,
    /// Works in department/cost center
    WorksIn,
    /// Responsible for
    ResponsibleFor,

    // ===== Network relationships =====
    /// Referred by another entity
    ReferredBy,
    /// Partners with another entity
    PartnersWith,
    /// Affiliated with
    AffiliatedWith,
    /// Intercompany relationship
    Intercompany,

    // ===== Document relationships =====
    /// Document references another document
    References,
    /// Document is referenced by another document
    ReferencedBy,
    /// Fulfills (e.g., delivery fulfills sales order)
    Fulfills,
    /// Fulfilled by
    FulfilledBy,
    /// Applies to (e.g., payment applies to invoice)
    AppliesTo,
    /// Applied by
    AppliedBy,

    // ===== Process relationships =====
    /// Inventory links P2P to O2C
    InventoryLink,
    /// Material used in
    UsedIn,
    /// Material sourced via
    SourcedVia,

    // ===== Sourcing/procurement relationships =====
    /// RFx awarded to vendor
    AwardedTo,
    /// Contract governs a purchase order
    GovernsOrder,
    /// Bid evaluated by evaluator
    EvaluatedBy,
    /// Vendor qualified as (status)
    QualifiedAs,
    /// Vendor scored by scorecard
    ScoredBy,
    /// Order sourced through contract
    SourcedThrough,
    /// Item belongs to catalog
    CatalogItemOf,

    // ===== Manufacturing relationships =====
    /// Material produced by production order
    ProducedBy,

    // ===== Banking relationships =====
    /// Payment reconciled with bank statement line
    ReconciledWith,

    // ===== P2P domain edges (DS-010) =====
    /// PurchaseOrder → Vendor
    PlacedWith,
    /// VendorInvoice → PurchaseOrder
    MatchesOrder,
    /// Payment → VendorInvoice
    PaysInvoice,

    // ===== O2C domain edges =====
    /// SalesOrder → Customer
    PlacedBy,
    /// CustomerInvoice → SalesOrder
    BillsOrder,

    // ===== S2C domain edges =====
    /// RfxEvent → SourcingProject
    RfxBelongsToProject,
    /// SupplierBid → RfxEvent
    RespondsTo,
    /// ProcurementContract → BidEvaluation
    AwardedFrom,

    // ===== H2R domain edges =====
    /// TimeEntry → Employee
    RecordedBy,
    /// PayrollRun → Employee
    PayrollIncludes,
    /// ExpenseReport → Employee
    SubmittedBy,
    /// BenefitEnrollment → Employee
    EnrolledBy,

    // ===== MFG domain edges =====
    /// ProductionOrder → Material
    Produces,
    /// QualityInspection → ProductionOrder
    Inspects,
    /// BomComponent → Material
    PartOf,

    // ===== Tax domain edges =====
    /// TaxLine → TaxReturn
    TaxLineBelongsTo,
    /// TaxProvision → TaxJurisdiction
    ProvisionAppliesTo,
    /// WithholdingTaxRecord → Vendor
    WithheldFrom,

    // ===== Treasury domain edges =====
    /// CashPoolSweep → CashPool
    SweepsTo,
    /// HedgeRelationship → HedgingInstrument
    HedgesInstrument,
    /// DebtCovenant → DebtInstrument
    GovernsInstrument,

    // ===== ESG domain edges =====
    /// EmissionRecord → Company
    EmissionReportedBy,
    /// SupplierEsgAssessment → Vendor
    AssessesSupplier,

    // ===== Project domain edges =====
    /// ProjectCostLine → Project
    CostChargedTo,
    /// ProjectMilestone → Project
    MilestoneOf,
    /// ChangeOrder → Project
    ModifiesProject,

    // ===== GOV domain edges =====
    /// CosoPrinciple → CosoComponent
    PrincipleUnder,
    /// SoxAssertion → GlAccount
    AssertionCovers,
    /// ProfessionalJudgment → AuditEngagement
    JudgmentWithin,
}

impl RelationshipType {
    /// Get the relationship type code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::BuysFrom => "BF",
            Self::SellsTo => "ST",
            Self::PaysTo => "PT",
            Self::ReceivesFrom => "RF",
            Self::SuppliesTo => "SP",
            Self::SourcesFrom => "SF",
            Self::ReportsTo => "RT",
            Self::Manages => "MG",
            Self::BelongsTo => "BT",
            Self::OwnedBy => "OB",
            Self::WorksIn => "WI",
            Self::ResponsibleFor => "RS",
            Self::ReferredBy => "RB",
            Self::PartnersWith => "PW",
            Self::AffiliatedWith => "AW",
            Self::Intercompany => "IC",
            Self::References => "REF",
            Self::ReferencedBy => "RBY",
            Self::Fulfills => "FL",
            Self::FulfilledBy => "FLB",
            Self::AppliesTo => "AP",
            Self::AppliedBy => "APB",
            Self::InventoryLink => "INV",
            Self::UsedIn => "UI",
            Self::SourcedVia => "SV",
            Self::AwardedTo => "AT",
            Self::GovernsOrder => "GO",
            Self::EvaluatedBy => "EB",
            Self::QualifiedAs => "QA",
            Self::ScoredBy => "SB",
            Self::SourcedThrough => "STH",
            Self::CatalogItemOf => "CIO",
            Self::ProducedBy => "PB",
            Self::ReconciledWith => "RW",
            // P2P
            Self::PlacedWith => "PWI",
            Self::MatchesOrder => "MO",
            Self::PaysInvoice => "PI",
            // O2C
            Self::PlacedBy => "PLB",
            Self::BillsOrder => "BO",
            // S2C
            Self::RfxBelongsToProject => "RBP",
            Self::RespondsTo => "RTO",
            Self::AwardedFrom => "AFR",
            // H2R
            Self::RecordedBy => "RCB",
            Self::PayrollIncludes => "PYI",
            Self::SubmittedBy => "SUB",
            Self::EnrolledBy => "ENB",
            // MFG
            Self::Produces => "PRD",
            Self::Inspects => "INS",
            Self::PartOf => "POF",
            // Tax
            Self::TaxLineBelongsTo => "TLB",
            Self::ProvisionAppliesTo => "PAT",
            Self::WithheldFrom => "WHF",
            // Treasury
            Self::SweepsTo => "SWT",
            Self::HedgesInstrument => "HDG",
            Self::GovernsInstrument => "GVI",
            // ESG
            Self::EmissionReportedBy => "ERB",
            Self::AssessesSupplier => "ASS",
            // Project
            Self::CostChargedTo => "CCT",
            Self::MilestoneOf => "MLO",
            Self::ModifiesProject => "MPJ",
            // GOV
            Self::PrincipleUnder => "PUN",
            Self::AssertionCovers => "ACO",
            Self::JudgmentWithin => "JWI",
        }
    }

    /// Get the inverse relationship type.
    pub fn inverse(&self) -> Self {
        match self {
            Self::BuysFrom => Self::SellsTo,
            Self::SellsTo => Self::BuysFrom,
            Self::PaysTo => Self::ReceivesFrom,
            Self::ReceivesFrom => Self::PaysTo,
            Self::SuppliesTo => Self::SourcesFrom,
            Self::SourcesFrom => Self::SuppliesTo,
            Self::ReportsTo => Self::Manages,
            Self::Manages => Self::ReportsTo,
            Self::BelongsTo => Self::OwnedBy,
            Self::OwnedBy => Self::BelongsTo,
            Self::References => Self::ReferencedBy,
            Self::ReferencedBy => Self::References,
            Self::Fulfills => Self::FulfilledBy,
            Self::FulfilledBy => Self::Fulfills,
            Self::AppliesTo => Self::AppliedBy,
            Self::AppliedBy => Self::AppliesTo,
            // Symmetric relationships
            Self::WorksIn => Self::WorksIn,
            Self::ResponsibleFor => Self::ResponsibleFor,
            Self::ReferredBy => Self::ReferredBy,
            Self::PartnersWith => Self::PartnersWith,
            Self::AffiliatedWith => Self::AffiliatedWith,
            Self::Intercompany => Self::Intercompany,
            Self::InventoryLink => Self::InventoryLink,
            Self::UsedIn => Self::UsedIn,
            Self::SourcedVia => Self::SourcedVia,
            // Sourcing/procurement (symmetric or self-inverse)
            Self::AwardedTo => Self::AwardedTo,
            Self::GovernsOrder => Self::GovernsOrder,
            Self::EvaluatedBy => Self::EvaluatedBy,
            Self::QualifiedAs => Self::QualifiedAs,
            Self::ScoredBy => Self::ScoredBy,
            Self::SourcedThrough => Self::SourcedThrough,
            Self::CatalogItemOf => Self::CatalogItemOf,
            Self::ProducedBy => Self::ProducedBy,
            Self::ReconciledWith => Self::ReconciledWith,
            // New domain edges (self-inverse, as they are directed)
            Self::PlacedWith => Self::PlacedWith,
            Self::MatchesOrder => Self::MatchesOrder,
            Self::PaysInvoice => Self::PaysInvoice,
            Self::PlacedBy => Self::PlacedBy,
            Self::BillsOrder => Self::BillsOrder,
            Self::RfxBelongsToProject => Self::RfxBelongsToProject,
            Self::RespondsTo => Self::RespondsTo,
            Self::AwardedFrom => Self::AwardedFrom,
            Self::RecordedBy => Self::RecordedBy,
            Self::PayrollIncludes => Self::PayrollIncludes,
            Self::SubmittedBy => Self::SubmittedBy,
            Self::EnrolledBy => Self::EnrolledBy,
            Self::Produces => Self::Produces,
            Self::Inspects => Self::Inspects,
            Self::PartOf => Self::PartOf,
            Self::TaxLineBelongsTo => Self::TaxLineBelongsTo,
            Self::ProvisionAppliesTo => Self::ProvisionAppliesTo,
            Self::WithheldFrom => Self::WithheldFrom,
            Self::SweepsTo => Self::SweepsTo,
            Self::HedgesInstrument => Self::HedgesInstrument,
            Self::GovernsInstrument => Self::GovernsInstrument,
            Self::EmissionReportedBy => Self::EmissionReportedBy,
            Self::AssessesSupplier => Self::AssessesSupplier,
            Self::CostChargedTo => Self::CostChargedTo,
            Self::MilestoneOf => Self::MilestoneOf,
            Self::ModifiesProject => Self::ModifiesProject,
            Self::PrincipleUnder => Self::PrincipleUnder,
            Self::AssertionCovers => Self::AssertionCovers,
            Self::JudgmentWithin => Self::JudgmentWithin,
        }
    }

    /// Check if this is a transactional relationship.
    pub fn is_transactional(&self) -> bool {
        matches!(
            self,
            Self::BuysFrom
                | Self::SellsTo
                | Self::PaysTo
                | Self::ReceivesFrom
                | Self::SuppliesTo
                | Self::SourcesFrom
        )
    }

    /// Check if this is an organizational relationship.
    pub fn is_organizational(&self) -> bool {
        matches!(
            self,
            Self::ReportsTo
                | Self::Manages
                | Self::BelongsTo
                | Self::OwnedBy
                | Self::WorksIn
                | Self::ResponsibleFor
        )
    }

    /// Check if this is a document relationship.
    pub fn is_document(&self) -> bool {
        matches!(
            self,
            Self::References
                | Self::ReferencedBy
                | Self::Fulfills
                | Self::FulfilledBy
                | Self::AppliesTo
                | Self::AppliedBy
        )
    }

    /// Get the edge constraint for this domain-specific relationship type.
    pub fn constraint(&self) -> Option<EdgeConstraint> {
        let c = |src: GraphEntityType, tgt: GraphEntityType, card: Cardinality| EdgeConstraint {
            relationship_type: *self,
            source_type: src,
            target_type: tgt,
            cardinality: card,
            edge_properties: &[],
        };
        use Cardinality::*;
        use GraphEntityType as E;
        match self {
            // P2P
            Self::PlacedWith => Some(c(E::PurchaseOrder, E::Vendor, ManyToOne)),
            Self::MatchesOrder => Some(c(E::Invoice, E::PurchaseOrder, ManyToOne)),
            Self::PaysInvoice => Some(c(E::Payment, E::Invoice, ManyToMany)),
            // O2C
            Self::PlacedBy => Some(c(E::SalesOrder, E::Customer, ManyToOne)),
            Self::BillsOrder => Some(c(E::Invoice, E::SalesOrder, ManyToOne)),
            // S2C
            Self::RfxBelongsToProject => Some(c(E::RfxEvent, E::SourcingProject, ManyToOne)),
            Self::RespondsTo => Some(c(E::SupplierBid, E::RfxEvent, ManyToOne)),
            Self::AwardedFrom => Some(c(E::ProcurementContract, E::BidEvaluation, OneToOne)),
            // H2R
            Self::RecordedBy => Some(c(E::TimeEntry, E::Employee, ManyToOne)),
            Self::PayrollIncludes => Some(c(E::PayrollRun, E::Employee, ManyToMany)),
            Self::SubmittedBy => Some(c(E::ExpenseReport, E::Employee, ManyToOne)),
            Self::EnrolledBy => Some(c(E::BenefitEnrollment, E::Employee, ManyToOne)),
            // MFG
            Self::Produces => Some(c(E::ProductionOrder, E::Material, ManyToOne)),
            Self::Inspects => Some(c(E::QualityInspection, E::ProductionOrder, ManyToOne)),
            Self::PartOf => Some(c(E::BomComponent, E::Material, ManyToOne)),
            // Tax
            Self::TaxLineBelongsTo => Some(c(E::TaxLine, E::TaxReturn, ManyToOne)),
            Self::ProvisionAppliesTo => Some(c(E::TaxProvision, E::TaxJurisdiction, ManyToOne)),
            Self::WithheldFrom => Some(c(E::WithholdingTaxRecord, E::Vendor, ManyToOne)),
            // Treasury
            Self::SweepsTo => Some(c(E::CashPoolSweep, E::CashPool, ManyToOne)),
            Self::HedgesInstrument => Some(c(E::HedgeRelationship, E::HedgingInstrument, ManyToOne)),
            Self::GovernsInstrument => Some(c(E::DebtCovenant, E::DebtInstrument, ManyToOne)),
            // ESG
            Self::EmissionReportedBy => Some(c(E::EmissionRecord, E::Company, ManyToOne)),
            Self::AssessesSupplier => Some(c(E::SupplierEsgAssessment, E::Vendor, ManyToOne)),
            // Project
            Self::CostChargedTo => Some(c(E::ProjectCostLine, E::Project, ManyToOne)),
            Self::MilestoneOf => Some(c(E::ProjectMilestone, E::Project, ManyToOne)),
            Self::ModifiesProject => Some(c(E::ChangeOrder, E::Project, ManyToOne)),
            // GOV
            Self::PrincipleUnder => Some(c(E::CosoPrinciple, E::CosoComponent, ManyToOne)),
            Self::AssertionCovers => Some(c(E::SoxAssertion, E::GlAccount, ManyToMany)),
            Self::JudgmentWithin => Some(c(E::ProfessionalJudgment, E::AuditEngagement, ManyToOne)),
            // Existing relationship types don't have formal constraints
            _ => None,
        }
    }

    /// All domain-specific edge constraints.
    pub fn all_constraints() -> Vec<EdgeConstraint> {
        let all_types = [
            Self::PlacedWith,
            Self::MatchesOrder,
            Self::PaysInvoice,
            Self::PlacedBy,
            Self::BillsOrder,
            Self::RfxBelongsToProject,
            Self::RespondsTo,
            Self::AwardedFrom,
            Self::RecordedBy,
            Self::PayrollIncludes,
            Self::SubmittedBy,
            Self::EnrolledBy,
            Self::Produces,
            Self::Inspects,
            Self::PartOf,
            Self::TaxLineBelongsTo,
            Self::ProvisionAppliesTo,
            Self::WithheldFrom,
            Self::SweepsTo,
            Self::HedgesInstrument,
            Self::GovernsInstrument,
            Self::EmissionReportedBy,
            Self::AssessesSupplier,
            Self::CostChargedTo,
            Self::MilestoneOf,
            Self::ModifiesProject,
            Self::PrincipleUnder,
            Self::AssertionCovers,
            Self::JudgmentWithin,
        ];
        all_types.iter().filter_map(|t| t.constraint()).collect()
    }
}

/// Edge cardinality constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cardinality {
    OneToOne,
    ManyToOne,
    ManyToMany,
}

/// Constraint on an edge type: valid source/target entity types and cardinality.
#[derive(Debug, Clone)]
pub struct EdgeConstraint {
    pub relationship_type: RelationshipType,
    pub source_type: GraphEntityType,
    pub target_type: GraphEntityType,
    pub cardinality: Cardinality,
    pub edge_properties: &'static [&'static str],
}

/// Unique identifier for an entity in the relationship graph.
///
/// This is separate from `entity_registry::EntityId` as it represents
/// the entity identifiers specifically used in graph/network analysis.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphEntityId {
    /// Entity type
    pub entity_type: GraphEntityType,
    /// Entity identifier (e.g., "V-001234")
    pub id: String,
}

impl GraphEntityId {
    /// Create a new entity ID.
    pub fn new(entity_type: GraphEntityType, id: impl Into<String>) -> Self {
        Self {
            entity_type,
            id: id.into(),
        }
    }

    /// Get the composite key for this entity.
    pub fn key(&self) -> String {
        format!("{}:{}", self.entity_type.code(), self.id)
    }
}

/// Node in the entity graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityNode {
    /// Entity identifier
    pub entity_id: GraphEntityId,
    /// Display name
    pub name: String,
    /// Entity attributes (flexible key-value)
    pub attributes: HashMap<String, String>,
    /// Creation date
    pub created_date: NaiveDate,
    /// Is entity active
    pub is_active: bool,
    /// Company code (if applicable)
    pub company_code: Option<String>,
}

impl EntityNode {
    /// Create a new entity node.
    pub fn new(entity_id: GraphEntityId, name: impl Into<String>, created_date: NaiveDate) -> Self {
        Self {
            entity_id,
            name: name.into(),
            attributes: HashMap::new(),
            created_date,
            is_active: true,
            company_code: None,
        }
    }

    /// Add an attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Set company code.
    pub fn with_company(mut self, company_code: impl Into<String>) -> Self {
        self.company_code = Some(company_code.into());
        self
    }
}

/// Edge in the entity graph representing a relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEdge {
    /// Source entity ID
    pub from_id: GraphEntityId,
    /// Target entity ID
    pub to_id: GraphEntityId,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Relationship strength (0.0 to 1.0)
    pub strength: f64,
    /// Relationship start date
    pub start_date: NaiveDate,
    /// Relationship end date (if terminated)
    pub end_date: Option<NaiveDate>,
    /// Edge attributes
    pub attributes: HashMap<String, String>,
    /// Strength components (for analysis)
    pub strength_components: Option<StrengthComponents>,
}

impl RelationshipEdge {
    /// Create a new relationship edge.
    pub fn new(
        from_id: GraphEntityId,
        to_id: GraphEntityId,
        relationship_type: RelationshipType,
        start_date: NaiveDate,
    ) -> Self {
        Self {
            from_id,
            to_id,
            relationship_type,
            strength: 0.5, // Default medium strength
            start_date,
            end_date: None,
            attributes: HashMap::new(),
            strength_components: None,
        }
    }

    /// Set relationship strength.
    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    /// Set strength with components.
    pub fn with_strength_components(mut self, components: StrengthComponents) -> Self {
        self.strength = components.total();
        self.strength_components = Some(components);
        self
    }

    /// Add an attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Check if relationship is active.
    pub fn is_active(&self) -> bool {
        self.end_date.is_none()
    }

    /// Get the edge key (for deduplication).
    pub fn key(&self) -> String {
        format!(
            "{}->{}:{}",
            self.from_id.key(),
            self.to_id.key(),
            self.relationship_type.code()
        )
    }
}

/// Components of relationship strength calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthComponents {
    /// Transaction volume component (log scale, 0.0-1.0)
    pub transaction_volume: f64,
    /// Transaction count component (sqrt scale, 0.0-1.0)
    pub transaction_count: f64,
    /// Relationship duration component (0.0-1.0)
    pub duration: f64,
    /// Recency component (exp decay, 0.0-1.0)
    pub recency: f64,
    /// Mutual connections component (Jaccard, 0.0-1.0)
    pub mutual_connections: f64,
}

impl StrengthComponents {
    /// Create new strength components.
    pub fn new(
        transaction_volume: f64,
        transaction_count: f64,
        duration: f64,
        recency: f64,
        mutual_connections: f64,
    ) -> Self {
        Self {
            transaction_volume: transaction_volume.clamp(0.0, 1.0),
            transaction_count: transaction_count.clamp(0.0, 1.0),
            duration: duration.clamp(0.0, 1.0),
            recency: recency.clamp(0.0, 1.0),
            mutual_connections: mutual_connections.clamp(0.0, 1.0),
        }
    }

    /// Calculate total strength with default weights.
    pub fn total(&self) -> f64 {
        self.total_weighted(RelationshipStrengthCalculator::default_weights())
    }

    /// Calculate total strength with custom weights.
    pub fn total_weighted(&self, weights: &StrengthWeights) -> f64 {
        let total = self.transaction_volume * weights.transaction_volume_weight
            + self.transaction_count * weights.transaction_count_weight
            + self.duration * weights.duration_weight
            + self.recency * weights.recency_weight
            + self.mutual_connections * weights.mutual_connections_weight;

        total.clamp(0.0, 1.0)
    }
}

/// Weights for relationship strength calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrengthWeights {
    /// Weight for transaction volume (default: 0.30)
    pub transaction_volume_weight: f64,
    /// Weight for transaction count (default: 0.25)
    pub transaction_count_weight: f64,
    /// Weight for relationship duration (default: 0.20)
    pub duration_weight: f64,
    /// Weight for recency (default: 0.15)
    pub recency_weight: f64,
    /// Weight for mutual connections (default: 0.10)
    pub mutual_connections_weight: f64,
}

impl Default for StrengthWeights {
    fn default() -> Self {
        Self {
            transaction_volume_weight: 0.30,
            transaction_count_weight: 0.25,
            duration_weight: 0.20,
            recency_weight: 0.15,
            mutual_connections_weight: 0.10,
        }
    }
}

impl StrengthWeights {
    /// Validate that weights sum to 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.transaction_volume_weight
            + self.transaction_count_weight
            + self.duration_weight
            + self.recency_weight
            + self.mutual_connections_weight;

        if (sum - 1.0).abs() > 0.01 {
            Err(format!("Strength weights must sum to 1.0, got {}", sum))
        } else {
            Ok(())
        }
    }
}

/// Calculator for relationship strength.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipStrengthCalculator {
    /// Strength weights
    pub weights: StrengthWeights,
    /// Recency half-life in days (default: 90)
    pub recency_half_life_days: u32,
    /// Max transaction volume for normalization
    pub max_transaction_volume: Decimal,
    /// Max transaction count for normalization
    pub max_transaction_count: u32,
    /// Max relationship duration in days for normalization
    pub max_duration_days: u32,
}

impl Default for RelationshipStrengthCalculator {
    fn default() -> Self {
        Self {
            weights: StrengthWeights::default(),
            recency_half_life_days: 90,
            max_transaction_volume: Decimal::from(10_000_000),
            max_transaction_count: 1000,
            max_duration_days: 3650, // 10 years
        }
    }
}

impl RelationshipStrengthCalculator {
    /// Get default weights.
    pub fn default_weights() -> &'static StrengthWeights {
        static WEIGHTS: std::sync::OnceLock<StrengthWeights> = std::sync::OnceLock::new();
        WEIGHTS.get_or_init(StrengthWeights::default)
    }

    /// Calculate relationship strength.
    pub fn calculate(
        &self,
        transaction_volume: Decimal,
        transaction_count: u32,
        relationship_days: u32,
        days_since_last_transaction: u32,
        mutual_connections: usize,
        total_possible_connections: usize,
    ) -> StrengthComponents {
        // Transaction volume (log scale)
        let volume_normalized = if transaction_volume > Decimal::ZERO
            && self.max_transaction_volume > Decimal::ZERO
        {
            let log_vol = (transaction_volume.to_string().parse::<f64>().unwrap_or(1.0) + 1.0).ln();
            let log_max = (self
                .max_transaction_volume
                .to_string()
                .parse::<f64>()
                .unwrap_or(1.0)
                + 1.0)
                .ln();
            (log_vol / log_max).min(1.0)
        } else {
            0.0
        };

        // Transaction count (sqrt scale)
        let count_normalized = if self.max_transaction_count > 0 {
            let sqrt_count = (transaction_count as f64).sqrt();
            let sqrt_max = (self.max_transaction_count as f64).sqrt();
            (sqrt_count / sqrt_max).min(1.0)
        } else {
            0.0
        };

        // Duration (linear scale)
        let duration_normalized = if self.max_duration_days > 0 {
            (relationship_days as f64 / self.max_duration_days as f64).min(1.0)
        } else {
            0.0
        };

        // Recency (exponential decay)
        let recency_normalized = if self.recency_half_life_days > 0 {
            let decay_rate = 0.693 / self.recency_half_life_days as f64; // ln(2) / half_life
            (-decay_rate * days_since_last_transaction as f64).exp()
        } else {
            1.0
        };

        // Mutual connections (Jaccard-like)
        let mutual_normalized = if total_possible_connections > 0 {
            mutual_connections as f64 / total_possible_connections as f64
        } else {
            0.0
        };

        StrengthComponents::new(
            volume_normalized,
            count_normalized,
            duration_normalized,
            recency_normalized,
            mutual_normalized,
        )
    }
}

/// Relationship strength classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipStrength {
    /// Strong relationship (>= 0.7)
    Strong,
    /// Moderate relationship (>= 0.4)
    Moderate,
    /// Weak relationship (>= 0.1)
    Weak,
    /// Dormant relationship (< 0.1)
    Dormant,
}

impl RelationshipStrength {
    /// Classify a strength value.
    pub fn from_value(strength: f64) -> Self {
        if strength >= 0.7 {
            Self::Strong
        } else if strength >= 0.4 {
            Self::Moderate
        } else if strength >= 0.1 {
            Self::Weak
        } else {
            Self::Dormant
        }
    }

    /// Get the minimum threshold for this classification.
    pub fn min_threshold(&self) -> f64 {
        match self {
            Self::Strong => 0.7,
            Self::Moderate => 0.4,
            Self::Weak => 0.1,
            Self::Dormant => 0.0,
        }
    }
}

/// Indexes for efficient graph lookups.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphIndexes {
    /// Edges from each node
    pub outgoing_edges: HashMap<String, Vec<usize>>,
    /// Edges to each node
    pub incoming_edges: HashMap<String, Vec<usize>>,
    /// Edges by relationship type
    pub edges_by_type: HashMap<RelationshipType, Vec<usize>>,
    /// Nodes by entity type
    pub nodes_by_type: HashMap<GraphEntityType, Vec<String>>,
}

/// Entity relationship graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityGraph {
    /// All nodes in the graph
    pub nodes: HashMap<String, EntityNode>,
    /// All edges in the graph
    pub edges: Vec<RelationshipEdge>,
    /// Graph indexes for efficient lookups
    #[serde(skip)]
    pub indexes: GraphIndexes,
    /// Graph metadata
    pub metadata: GraphMetadata,
}

/// Metadata about the graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Company code (if single-company graph)
    pub company_code: Option<String>,
    /// Creation date
    pub created_date: Option<NaiveDate>,
    /// Total transaction volume
    #[serde(with = "rust_decimal::serde::str")]
    pub total_transaction_volume: Decimal,
    /// Date range covered
    pub date_range: Option<(NaiveDate, NaiveDate)>,
}

impl EntityGraph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: EntityNode) {
        let key = node.entity_id.key();
        let entity_type = node.entity_id.entity_type;

        self.nodes.insert(key.clone(), node);
        self.indexes
            .nodes_by_type
            .entry(entity_type)
            .or_default()
            .push(key);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: RelationshipEdge) {
        let edge_idx = self.edges.len();
        let from_key = edge.from_id.key();
        let to_key = edge.to_id.key();
        let rel_type = edge.relationship_type;

        self.indexes
            .outgoing_edges
            .entry(from_key)
            .or_default()
            .push(edge_idx);
        self.indexes
            .incoming_edges
            .entry(to_key)
            .or_default()
            .push(edge_idx);
        self.indexes
            .edges_by_type
            .entry(rel_type)
            .or_default()
            .push(edge_idx);

        self.edges.push(edge);
    }

    /// Get a node by entity ID.
    pub fn get_node(&self, entity_id: &GraphEntityId) -> Option<&EntityNode> {
        self.nodes.get(&entity_id.key())
    }

    /// Get outgoing edges from a node.
    pub fn get_outgoing_edges(&self, entity_id: &GraphEntityId) -> Vec<&RelationshipEdge> {
        self.indexes
            .outgoing_edges
            .get(&entity_id.key())
            .map(|indices| indices.iter().map(|&idx| &self.edges[idx]).collect())
            .unwrap_or_default()
    }

    /// Get incoming edges to a node.
    pub fn get_incoming_edges(&self, entity_id: &GraphEntityId) -> Vec<&RelationshipEdge> {
        self.indexes
            .incoming_edges
            .get(&entity_id.key())
            .map(|indices| indices.iter().map(|&idx| &self.edges[idx]).collect())
            .unwrap_or_default()
    }

    /// Get edges by relationship type.
    pub fn get_edges_by_type(&self, rel_type: RelationshipType) -> Vec<&RelationshipEdge> {
        self.indexes
            .edges_by_type
            .get(&rel_type)
            .map(|indices| indices.iter().map(|&idx| &self.edges[idx]).collect())
            .unwrap_or_default()
    }

    /// Get all nodes of a specific type.
    pub fn get_nodes_by_type(&self, entity_type: GraphEntityType) -> Vec<&EntityNode> {
        self.indexes
            .nodes_by_type
            .get(&entity_type)
            .map(|keys| keys.iter().filter_map(|k| self.nodes.get(k)).collect())
            .unwrap_or_default()
    }

    /// Find neighbors of a node (nodes connected by edges).
    pub fn get_neighbors(&self, entity_id: &GraphEntityId) -> Vec<&EntityNode> {
        let mut neighbor_ids: HashSet<String> = HashSet::new();

        // Outgoing edges
        for edge in self.get_outgoing_edges(entity_id) {
            neighbor_ids.insert(edge.to_id.key());
        }

        // Incoming edges
        for edge in self.get_incoming_edges(entity_id) {
            neighbor_ids.insert(edge.from_id.key());
        }

        neighbor_ids
            .iter()
            .filter_map(|key| self.nodes.get(key))
            .collect()
    }

    /// Calculate the degree of a node (total edges in + out).
    pub fn node_degree(&self, entity_id: &GraphEntityId) -> usize {
        let key = entity_id.key();
        let out_degree = self
            .indexes
            .outgoing_edges
            .get(&key)
            .map(|v| v.len())
            .unwrap_or(0);
        let in_degree = self
            .indexes
            .incoming_edges
            .get(&key)
            .map(|v| v.len())
            .unwrap_or(0);
        out_degree + in_degree
    }

    /// Rebuild indexes (call after deserialization).
    pub fn rebuild_indexes(&mut self) {
        self.indexes = GraphIndexes::default();

        // Rebuild node type index
        for (key, node) in &self.nodes {
            self.indexes
                .nodes_by_type
                .entry(node.entity_id.entity_type)
                .or_default()
                .push(key.clone());
        }

        // Rebuild edge indexes
        for (idx, edge) in self.edges.iter().enumerate() {
            self.indexes
                .outgoing_edges
                .entry(edge.from_id.key())
                .or_default()
                .push(idx);
            self.indexes
                .incoming_edges
                .entry(edge.to_id.key())
                .or_default()
                .push(idx);
            self.indexes
                .edges_by_type
                .entry(edge.relationship_type)
                .or_default()
                .push(idx);
        }
    }

    /// Get graph statistics.
    pub fn statistics(&self) -> GraphStatistics {
        let node_count = self.nodes.len();
        let edge_count = self.edges.len();

        // Calculate average degree
        let avg_degree = if node_count > 0 {
            (2.0 * edge_count as f64) / node_count as f64
        } else {
            0.0
        };

        // Calculate average strength
        let avg_strength = if edge_count > 0 {
            self.edges.iter().map(|e| e.strength).sum::<f64>() / edge_count as f64
        } else {
            0.0
        };

        // Count nodes by type
        let mut node_counts: HashMap<String, usize> = HashMap::new();
        for node in self.nodes.values() {
            *node_counts
                .entry(format!("{:?}", node.entity_id.entity_type))
                .or_insert(0) += 1;
        }

        // Count edges by type
        let mut edge_counts: HashMap<String, usize> = HashMap::new();
        for edge in &self.edges {
            *edge_counts
                .entry(format!("{:?}", edge.relationship_type))
                .or_insert(0) += 1;
        }

        // Count strength distribution
        let mut strength_distribution: HashMap<String, usize> = HashMap::new();
        for edge in &self.edges {
            let classification = RelationshipStrength::from_value(edge.strength);
            *strength_distribution
                .entry(format!("{:?}", classification))
                .or_insert(0) += 1;
        }

        GraphStatistics {
            node_count,
            edge_count,
            avg_degree,
            avg_strength,
            node_counts,
            edge_counts,
            strength_distribution,
        }
    }
}

/// Statistics about the graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Average degree (edges per node)
    pub avg_degree: f64,
    /// Average edge strength
    pub avg_strength: f64,
    /// Node counts by type
    pub node_counts: HashMap<String, usize>,
    /// Edge counts by relationship type
    pub edge_counts: HashMap<String, usize>,
    /// Edge counts by strength classification
    pub strength_distribution: HashMap<String, usize>,
}

/// Cross-process link connecting P2P and O2C via inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossProcessLink {
    /// Material ID linking the processes
    pub material_id: String,
    /// Source process (e.g., P2P)
    pub source_process: String,
    /// Source document ID
    pub source_document_id: String,
    /// Target process (e.g., O2C)
    pub target_process: String,
    /// Target document ID
    pub target_document_id: String,
    /// Link type
    pub link_type: CrossProcessLinkType,
    /// Quantity involved
    #[serde(with = "rust_decimal::serde::str")]
    pub quantity: Decimal,
    /// Link date
    pub link_date: NaiveDate,
}

/// Type of cross-process link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossProcessLinkType {
    /// Inventory movement links GR to delivery
    InventoryMovement,
    /// Return flow from O2C back to P2P
    ReturnFlow,
    /// Payment reconciliation
    PaymentReconciliation,
    /// Intercompany bilateral matching
    IntercompanyBilateral,
}

impl CrossProcessLink {
    /// Create a new cross-process link.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        material_id: impl Into<String>,
        source_process: impl Into<String>,
        source_document_id: impl Into<String>,
        target_process: impl Into<String>,
        target_document_id: impl Into<String>,
        link_type: CrossProcessLinkType,
        quantity: Decimal,
        link_date: NaiveDate,
    ) -> Self {
        Self {
            material_id: material_id.into(),
            source_process: source_process.into(),
            source_document_id: source_document_id.into(),
            target_process: target_process.into(),
            target_document_id: target_document_id.into(),
            link_type,
            quantity,
            link_date,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id() {
        let id = GraphEntityId::new(GraphEntityType::Vendor, "V-001234");
        assert_eq!(id.key(), "VN:V-001234");
    }

    #[test]
    fn test_relationship_type_inverse() {
        assert_eq!(
            RelationshipType::BuysFrom.inverse(),
            RelationshipType::SellsTo
        );
        assert_eq!(
            RelationshipType::SellsTo.inverse(),
            RelationshipType::BuysFrom
        );
        assert_eq!(
            RelationshipType::ReportsTo.inverse(),
            RelationshipType::Manages
        );
    }

    #[test]
    fn test_strength_weights_validation() {
        let valid_weights = StrengthWeights::default();
        assert!(valid_weights.validate().is_ok());

        let invalid_weights = StrengthWeights {
            transaction_volume_weight: 0.5,
            transaction_count_weight: 0.5,
            duration_weight: 0.5,
            recency_weight: 0.5,
            mutual_connections_weight: 0.5,
        };
        assert!(invalid_weights.validate().is_err());
    }

    #[test]
    fn test_strength_calculator() {
        let calc = RelationshipStrengthCalculator::default();
        let components = calc.calculate(Decimal::from(100000), 50, 365, 30, 5, 20);

        assert!(components.transaction_volume > 0.0);
        assert!(components.transaction_count > 0.0);
        assert!(components.duration > 0.0);
        assert!(components.recency > 0.0);
        assert!(components.mutual_connections > 0.0);
        assert!(components.total() <= 1.0);
    }

    #[test]
    fn test_relationship_strength_classification() {
        assert_eq!(
            RelationshipStrength::from_value(0.8),
            RelationshipStrength::Strong
        );
        assert_eq!(
            RelationshipStrength::from_value(0.5),
            RelationshipStrength::Moderate
        );
        assert_eq!(
            RelationshipStrength::from_value(0.2),
            RelationshipStrength::Weak
        );
        assert_eq!(
            RelationshipStrength::from_value(0.05),
            RelationshipStrength::Dormant
        );
    }

    #[test]
    fn test_entity_graph() {
        let mut graph = EntityGraph::new();

        let vendor_id = GraphEntityId::new(GraphEntityType::Vendor, "V-001");
        let customer_id = GraphEntityId::new(GraphEntityType::Customer, "C-001");

        graph.add_node(EntityNode::new(
            vendor_id.clone(),
            "Acme Supplies",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));

        graph.add_node(EntityNode::new(
            customer_id.clone(),
            "Contoso Corp",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));

        graph.add_edge(
            RelationshipEdge::new(
                vendor_id.clone(),
                customer_id.clone(),
                RelationshipType::SellsTo,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )
            .with_strength(0.7),
        );

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);

        let neighbors = graph.get_neighbors(&vendor_id);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].entity_id.id, "C-001");

        assert_eq!(graph.node_degree(&vendor_id), 1);
        assert_eq!(graph.node_degree(&customer_id), 1);
    }

    #[test]
    fn test_graph_statistics() {
        let mut graph = EntityGraph::new();

        for i in 0..10 {
            let id = GraphEntityId::new(GraphEntityType::Vendor, format!("V-{:03}", i));
            graph.add_node(EntityNode::new(
                id,
                format!("Vendor {}", i),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ));
        }

        for i in 0..5 {
            let from_id = GraphEntityId::new(GraphEntityType::Vendor, format!("V-{:03}", i));
            let to_id = GraphEntityId::new(GraphEntityType::Vendor, format!("V-{:03}", i + 5));
            graph.add_edge(
                RelationshipEdge::new(
                    from_id,
                    to_id,
                    RelationshipType::PartnersWith,
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                )
                .with_strength(0.6),
            );
        }

        let stats = graph.statistics();
        assert_eq!(stats.node_count, 10);
        assert_eq!(stats.edge_count, 5);
        assert!((stats.avg_degree - 1.0).abs() < 0.01);
        assert!((stats.avg_strength - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_numeric_code_roundtrip() {
        for &entity_type in GraphEntityType::all_types() {
            let code = entity_type.numeric_code();
            let recovered = GraphEntityType::from_numeric_code(code);
            assert_eq!(
                recovered,
                Some(entity_type),
                "Failed roundtrip for {:?} with code {}",
                entity_type,
                code
            );
        }
    }

    #[test]
    fn test_node_type_name_roundtrip() {
        for &entity_type in GraphEntityType::all_types() {
            let name = entity_type.node_type_name();
            let recovered = GraphEntityType::from_node_type_name(name);
            assert_eq!(
                recovered,
                Some(entity_type),
                "Failed roundtrip for {:?} with name {}",
                entity_type,
                name
            );
        }
    }

    #[test]
    fn test_all_types_unique_codes() {
        let mut codes = std::collections::HashSet::new();
        for &entity_type in GraphEntityType::all_types() {
            assert!(
                codes.insert(entity_type.numeric_code()),
                "Duplicate numeric code {} for {:?}",
                entity_type.numeric_code(),
                entity_type
            );
        }
    }

    #[test]
    fn test_all_types_unique_names() {
        let mut names = std::collections::HashSet::new();
        for &entity_type in GraphEntityType::all_types() {
            assert!(
                names.insert(entity_type.node_type_name()),
                "Duplicate name {} for {:?}",
                entity_type.node_type_name(),
                entity_type
            );
        }
    }

    #[test]
    fn test_all_types_unique_letter_codes() {
        let mut codes = std::collections::HashSet::new();
        for &entity_type in GraphEntityType::all_types() {
            assert!(
                codes.insert(entity_type.code()),
                "Duplicate letter code {} for {:?}",
                entity_type.code(),
                entity_type
            );
        }
    }

    #[test]
    fn test_category_helpers() {
        assert!(GraphEntityType::TaxJurisdiction.is_tax());
        assert!(GraphEntityType::UncertainTaxPosition.is_tax());
        assert!(!GraphEntityType::CashPosition.is_tax());

        assert!(GraphEntityType::CashPosition.is_treasury());
        assert!(GraphEntityType::DebtCovenant.is_treasury());
        assert!(!GraphEntityType::EmissionRecord.is_treasury());

        assert!(GraphEntityType::EmissionRecord.is_esg());
        assert!(GraphEntityType::ClimateScenario.is_esg());
        assert!(!GraphEntityType::TaxCode.is_esg());

        assert!(GraphEntityType::Project.is_project());
        assert!(GraphEntityType::ProjectMilestone.is_project());

        assert!(GraphEntityType::PayrollRun.is_h2r());
        assert!(GraphEntityType::BenefitEnrollment.is_h2r());

        assert!(GraphEntityType::ProductionOrder.is_mfg());
        assert!(GraphEntityType::BomComponent.is_mfg());

        assert!(GraphEntityType::CosoComponent.is_governance());
        assert!(GraphEntityType::ProfessionalJudgment.is_governance());
    }

    #[test]
    fn test_edge_constraint_validity() {
        let constraints = RelationshipType::all_constraints();
        assert_eq!(constraints.len(), 29, "Expected 29 domain-specific edge constraints");
        for constraint in &constraints {
            // Source and target types should have valid numeric codes
            assert!(constraint.source_type.numeric_code() > 0);
            assert!(constraint.target_type.numeric_code() > 0);
        }
    }

    #[test]
    fn test_all_process_families_have_edges() {
        let constraints = RelationshipType::all_constraints();
        // P2P
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::PlacedWith));
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::PaysInvoice));
        // O2C
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::PlacedBy));
        // Tax
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::TaxLineBelongsTo));
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::WithheldFrom));
        // Treasury
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::SweepsTo));
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::GovernsInstrument));
        // ESG
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::EmissionReportedBy));
        // Project
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::CostChargedTo));
        // GOV
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::PrincipleUnder));
        assert!(constraints.iter().any(|c| c.relationship_type == RelationshipType::JudgmentWithin));
    }

    #[test]
    fn test_edge_source_target_types() {
        let placed_with = RelationshipType::PlacedWith.constraint().unwrap();
        assert_eq!(placed_with.source_type, GraphEntityType::PurchaseOrder);
        assert_eq!(placed_with.target_type, GraphEntityType::Vendor);
        assert_eq!(placed_with.cardinality, Cardinality::ManyToOne);

        let awarded_from = RelationshipType::AwardedFrom.constraint().unwrap();
        assert_eq!(awarded_from.source_type, GraphEntityType::ProcurementContract);
        assert_eq!(awarded_from.target_type, GraphEntityType::BidEvaluation);
        assert_eq!(awarded_from.cardinality, Cardinality::OneToOne);

        let pays_invoice = RelationshipType::PaysInvoice.constraint().unwrap();
        assert_eq!(pays_invoice.cardinality, Cardinality::ManyToMany);
    }

    #[test]
    fn test_existing_types_no_constraint() {
        // Pre-existing relationship types should return None for constraint
        assert!(RelationshipType::BuysFrom.constraint().is_none());
        assert!(RelationshipType::SellsTo.constraint().is_none());
        assert!(RelationshipType::ReportsTo.constraint().is_none());
    }

    #[test]
    fn test_specific_entity_codes() {
        assert_eq!(GraphEntityType::UncertainTaxPosition.numeric_code(), 416);
        assert_eq!(
            GraphEntityType::UncertainTaxPosition.node_type_name(),
            "uncertain_tax_position"
        );
        assert_eq!(GraphEntityType::DebtCovenant.numeric_code(), 427);
        assert_eq!(GraphEntityType::BenefitEnrollment.numeric_code(), 333);
        assert_eq!(GraphEntityType::BomComponent.numeric_code(), 343);
    }

    #[test]
    fn test_all_types_count() {
        // 20 original + 7 tax + 8 treasury + 13 ESG + 5 project
        // + 4 S2C + 4 H2R + 4 MFG + 5 GOV = 70
        assert_eq!(GraphEntityType::all_types().len(), 70);
    }

    #[test]
    fn test_cross_process_link() {
        let link = CrossProcessLink::new(
            "MAT-001",
            "P2P",
            "GR-12345",
            "O2C",
            "DEL-67890",
            CrossProcessLinkType::InventoryMovement,
            Decimal::from(100),
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
        );

        assert_eq!(link.material_id, "MAT-001");
        assert_eq!(link.link_type, CrossProcessLinkType::InventoryMovement);
    }
}
