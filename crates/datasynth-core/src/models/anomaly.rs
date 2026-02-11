//! Anomaly types and labels for synthetic data generation.
//!
//! This module provides comprehensive anomaly classification for:
//! - Fraud detection training
//! - Error detection systems
//! - Process compliance monitoring
//! - Statistical anomaly detection
//! - Graph-based anomaly detection

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Causal reason explaining why an anomaly was injected.
///
/// This enables provenance tracking for understanding the "why" behind each anomaly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalyCausalReason {
    /// Injected due to random rate selection.
    RandomRate {
        /// Base rate used for selection.
        base_rate: f64,
    },
    /// Injected due to temporal pattern matching.
    TemporalPattern {
        /// Name of the temporal pattern (e.g., "year_end_spike", "month_end").
        pattern_name: String,
    },
    /// Injected based on entity targeting rules.
    EntityTargeting {
        /// Type of entity targeted (e.g., "vendor", "user", "account").
        target_type: String,
        /// ID of the targeted entity.
        target_id: String,
    },
    /// Part of an anomaly cluster.
    ClusterMembership {
        /// ID of the cluster this anomaly belongs to.
        cluster_id: String,
    },
    /// Part of a multi-step scenario.
    ScenarioStep {
        /// Type of scenario (e.g., "kickback_scheme", "round_tripping").
        scenario_type: String,
        /// Step number within the scenario.
        step_number: u32,
    },
    /// Injected based on data quality profile.
    DataQualityProfile {
        /// Profile name (e.g., "noisy", "legacy", "clean").
        profile: String,
    },
    /// Injected for ML training balance.
    MLTrainingBalance {
        /// Target class being balanced.
        target_class: String,
    },
}

/// Structured injection strategy with captured parameters.
///
/// Unlike the string-based `injection_strategy` field, this enum captures
/// the exact parameters used during injection for full reproducibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InjectionStrategy {
    /// Amount was manipulated by a factor.
    AmountManipulation {
        /// Original amount before manipulation.
        original: Decimal,
        /// Multiplication factor applied.
        factor: f64,
    },
    /// Amount adjusted to avoid a threshold.
    ThresholdAvoidance {
        /// Threshold being avoided.
        threshold: Decimal,
        /// Final amount after adjustment.
        adjusted_amount: Decimal,
    },
    /// Date was backdated or forward-dated.
    DateShift {
        /// Number of days shifted (negative = backdated).
        days_shifted: i32,
        /// Original date before shift.
        original_date: NaiveDate,
    },
    /// User approved their own transaction.
    SelfApproval {
        /// User who created and approved.
        user_id: String,
    },
    /// Segregation of duties violation.
    SoDViolation {
        /// First duty involved.
        duty1: String,
        /// Second duty involved.
        duty2: String,
        /// User who performed both duties.
        violating_user: String,
    },
    /// Exact duplicate of another document.
    ExactDuplicate {
        /// ID of the original document.
        original_doc_id: String,
    },
    /// Near-duplicate with small variations.
    NearDuplicate {
        /// ID of the original document.
        original_doc_id: String,
        /// Fields that were varied.
        varied_fields: Vec<String>,
    },
    /// Circular flow of funds/goods.
    CircularFlow {
        /// Chain of entities involved.
        entity_chain: Vec<String>,
    },
    /// Split transaction to avoid threshold.
    SplitTransaction {
        /// Original total amount.
        original_amount: Decimal,
        /// Number of splits.
        split_count: u32,
        /// IDs of the split documents.
        split_doc_ids: Vec<String>,
    },
    /// Round number manipulation.
    RoundNumbering {
        /// Original precise amount.
        original_amount: Decimal,
        /// Rounded amount.
        rounded_amount: Decimal,
    },
    /// Timing manipulation (weekend, after-hours, etc.).
    TimingManipulation {
        /// Type of timing issue.
        timing_type: String,
        /// Original timestamp.
        original_time: Option<NaiveDateTime>,
    },
    /// Account misclassification.
    AccountMisclassification {
        /// Correct account.
        correct_account: String,
        /// Incorrect account used.
        incorrect_account: String,
    },
    /// Missing required field.
    MissingField {
        /// Name of the missing field.
        field_name: String,
    },
    /// Custom injection strategy.
    Custom {
        /// Strategy name.
        name: String,
        /// Additional parameters.
        parameters: HashMap<String, String>,
    },
}

impl InjectionStrategy {
    /// Returns a human-readable description of the strategy.
    pub fn description(&self) -> String {
        match self {
            InjectionStrategy::AmountManipulation { factor, .. } => {
                format!("Amount multiplied by {:.2}", factor)
            }
            InjectionStrategy::ThresholdAvoidance { threshold, .. } => {
                format!("Amount adjusted to avoid {} threshold", threshold)
            }
            InjectionStrategy::DateShift { days_shifted, .. } => {
                if *days_shifted < 0 {
                    format!("Date backdated by {} days", days_shifted.abs())
                } else {
                    format!("Date forward-dated by {} days", days_shifted)
                }
            }
            InjectionStrategy::SelfApproval { user_id } => {
                format!("Self-approval by user {}", user_id)
            }
            InjectionStrategy::SoDViolation { duty1, duty2, .. } => {
                format!("SoD violation: {} and {}", duty1, duty2)
            }
            InjectionStrategy::ExactDuplicate { original_doc_id } => {
                format!("Exact duplicate of {}", original_doc_id)
            }
            InjectionStrategy::NearDuplicate {
                original_doc_id,
                varied_fields,
            } => {
                format!(
                    "Near-duplicate of {} (varied: {:?})",
                    original_doc_id, varied_fields
                )
            }
            InjectionStrategy::CircularFlow { entity_chain } => {
                format!("Circular flow through {} entities", entity_chain.len())
            }
            InjectionStrategy::SplitTransaction { split_count, .. } => {
                format!("Split into {} transactions", split_count)
            }
            InjectionStrategy::RoundNumbering { .. } => "Amount rounded to even number".to_string(),
            InjectionStrategy::TimingManipulation { timing_type, .. } => {
                format!("Timing manipulation: {}", timing_type)
            }
            InjectionStrategy::AccountMisclassification {
                correct_account,
                incorrect_account,
            } => {
                format!(
                    "Misclassified from {} to {}",
                    correct_account, incorrect_account
                )
            }
            InjectionStrategy::MissingField { field_name } => {
                format!("Missing required field: {}", field_name)
            }
            InjectionStrategy::Custom { name, .. } => format!("Custom: {}", name),
        }
    }

    /// Returns the strategy type name.
    pub fn strategy_type(&self) -> &'static str {
        match self {
            InjectionStrategy::AmountManipulation { .. } => "AmountManipulation",
            InjectionStrategy::ThresholdAvoidance { .. } => "ThresholdAvoidance",
            InjectionStrategy::DateShift { .. } => "DateShift",
            InjectionStrategy::SelfApproval { .. } => "SelfApproval",
            InjectionStrategy::SoDViolation { .. } => "SoDViolation",
            InjectionStrategy::ExactDuplicate { .. } => "ExactDuplicate",
            InjectionStrategy::NearDuplicate { .. } => "NearDuplicate",
            InjectionStrategy::CircularFlow { .. } => "CircularFlow",
            InjectionStrategy::SplitTransaction { .. } => "SplitTransaction",
            InjectionStrategy::RoundNumbering { .. } => "RoundNumbering",
            InjectionStrategy::TimingManipulation { .. } => "TimingManipulation",
            InjectionStrategy::AccountMisclassification { .. } => "AccountMisclassification",
            InjectionStrategy::MissingField { .. } => "MissingField",
            InjectionStrategy::Custom { .. } => "Custom",
        }
    }
}

/// Primary anomaly classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Fraudulent activity.
    Fraud(FraudType),
    /// Data entry or processing error.
    Error(ErrorType),
    /// Process or control issue.
    ProcessIssue(ProcessIssueType),
    /// Statistical anomaly.
    Statistical(StatisticalAnomalyType),
    /// Relational/graph anomaly.
    Relational(RelationalAnomalyType),
    /// Custom anomaly type.
    Custom(String),
}

impl AnomalyType {
    /// Returns the category name.
    pub fn category(&self) -> &'static str {
        match self {
            AnomalyType::Fraud(_) => "Fraud",
            AnomalyType::Error(_) => "Error",
            AnomalyType::ProcessIssue(_) => "ProcessIssue",
            AnomalyType::Statistical(_) => "Statistical",
            AnomalyType::Relational(_) => "Relational",
            AnomalyType::Custom(_) => "Custom",
        }
    }

    /// Returns the specific type name.
    pub fn type_name(&self) -> String {
        match self {
            AnomalyType::Fraud(t) => format!("{:?}", t),
            AnomalyType::Error(t) => format!("{:?}", t),
            AnomalyType::ProcessIssue(t) => format!("{:?}", t),
            AnomalyType::Statistical(t) => format!("{:?}", t),
            AnomalyType::Relational(t) => format!("{:?}", t),
            AnomalyType::Custom(s) => s.clone(),
        }
    }

    /// Returns the severity level (1-5, 5 being most severe).
    pub fn severity(&self) -> u8 {
        match self {
            AnomalyType::Fraud(t) => t.severity(),
            AnomalyType::Error(t) => t.severity(),
            AnomalyType::ProcessIssue(t) => t.severity(),
            AnomalyType::Statistical(t) => t.severity(),
            AnomalyType::Relational(t) => t.severity(),
            AnomalyType::Custom(_) => 3,
        }
    }

    /// Returns whether this anomaly is typically intentional.
    pub fn is_intentional(&self) -> bool {
        matches!(self, AnomalyType::Fraud(_))
    }
}

/// Fraud types for detection training.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FraudType {
    // Journal Entry Fraud
    /// Fictitious journal entry with no business purpose.
    FictitiousEntry,
    /// Fictitious transaction (alias for FictitiousEntry).
    FictitiousTransaction,
    /// Round-dollar amounts suggesting manual manipulation.
    RoundDollarManipulation,
    /// Entry posted just below approval threshold.
    JustBelowThreshold,
    /// Revenue recognition manipulation.
    RevenueManipulation,
    /// Expense capitalization fraud.
    ImproperCapitalization,
    /// Improperly capitalizing expenses as assets.
    ExpenseCapitalization,
    /// Cookie jar reserves manipulation.
    ReserveManipulation,
    /// Round-tripping funds through suspense/clearing accounts.
    SuspenseAccountAbuse,
    /// Splitting transactions to stay below approval thresholds.
    SplitTransaction,
    /// Unusual timing (weekend, holiday, after-hours postings).
    TimingAnomaly,
    /// Posting to unauthorized accounts.
    UnauthorizedAccess,

    // Approval Fraud
    /// User approving their own request.
    SelfApproval,
    /// Approval beyond authorized limit.
    ExceededApprovalLimit,
    /// Segregation of duties violation.
    SegregationOfDutiesViolation,
    /// Approval by unauthorized user.
    UnauthorizedApproval,
    /// Collusion between approver and requester.
    CollusiveApproval,

    // Vendor/Payment Fraud
    /// Fictitious vendor.
    FictitiousVendor,
    /// Duplicate payment to vendor.
    DuplicatePayment,
    /// Payment to shell company.
    ShellCompanyPayment,
    /// Kickback scheme.
    Kickback,
    /// Kickback scheme (alias).
    KickbackScheme,
    /// Invoice manipulation.
    InvoiceManipulation,

    // Asset Fraud
    /// Misappropriation of assets.
    AssetMisappropriation,
    /// Inventory theft.
    InventoryTheft,
    /// Ghost employee.
    GhostEmployee,

    // Financial Statement Fraud
    /// Premature revenue recognition.
    PrematureRevenue,
    /// Understated liabilities.
    UnderstatedLiabilities,
    /// Overstated assets.
    OverstatedAssets,
    /// Channel stuffing.
    ChannelStuffing,

    // Accounting Standards Violations (ASC 606 / IFRS 15 - Revenue)
    /// Improper revenue recognition timing (ASC 606/IFRS 15).
    ImproperRevenueRecognition,
    /// Multiple performance obligations not properly separated.
    ImproperPoAllocation,
    /// Variable consideration not properly estimated.
    VariableConsiderationManipulation,
    /// Contract modifications not properly accounted for.
    ContractModificationMisstatement,

    // Accounting Standards Violations (ASC 842 / IFRS 16 - Leases)
    /// Lease classification manipulation (operating vs finance).
    LeaseClassificationManipulation,
    /// Off-balance sheet lease fraud.
    OffBalanceSheetLease,
    /// Lease liability understatement.
    LeaseLiabilityUnderstatement,
    /// ROU asset misstatement.
    RouAssetMisstatement,

    // Accounting Standards Violations (ASC 820 / IFRS 13 - Fair Value)
    /// Fair value hierarchy misclassification.
    FairValueHierarchyManipulation,
    /// Level 3 input manipulation.
    Level3InputManipulation,
    /// Valuation technique manipulation.
    ValuationTechniqueManipulation,

    // Accounting Standards Violations (ASC 360 / IAS 36 - Impairment)
    /// Delayed impairment recognition.
    DelayedImpairment,
    /// Improperly avoiding impairment testing.
    ImpairmentTestAvoidance,
    /// Cash flow projection manipulation for impairment.
    CashFlowProjectionManipulation,
    /// Improper impairment reversal (IFRS only).
    ImproperImpairmentReversal,
}

impl FraudType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            FraudType::RoundDollarManipulation => 2,
            FraudType::JustBelowThreshold => 3,
            FraudType::SelfApproval => 3,
            FraudType::ExceededApprovalLimit => 3,
            FraudType::DuplicatePayment => 3,
            FraudType::FictitiousEntry => 4,
            FraudType::RevenueManipulation => 5,
            FraudType::FictitiousVendor => 5,
            FraudType::ShellCompanyPayment => 5,
            FraudType::AssetMisappropriation => 5,
            FraudType::SegregationOfDutiesViolation => 4,
            FraudType::CollusiveApproval => 5,
            // Accounting Standards Violations (Revenue - ASC 606/IFRS 15)
            FraudType::ImproperRevenueRecognition => 5,
            FraudType::ImproperPoAllocation => 4,
            FraudType::VariableConsiderationManipulation => 4,
            FraudType::ContractModificationMisstatement => 3,
            // Accounting Standards Violations (Leases - ASC 842/IFRS 16)
            FraudType::LeaseClassificationManipulation => 4,
            FraudType::OffBalanceSheetLease => 5,
            FraudType::LeaseLiabilityUnderstatement => 4,
            FraudType::RouAssetMisstatement => 3,
            // Accounting Standards Violations (Fair Value - ASC 820/IFRS 13)
            FraudType::FairValueHierarchyManipulation => 4,
            FraudType::Level3InputManipulation => 5,
            FraudType::ValuationTechniqueManipulation => 4,
            // Accounting Standards Violations (Impairment - ASC 360/IAS 36)
            FraudType::DelayedImpairment => 4,
            FraudType::ImpairmentTestAvoidance => 4,
            FraudType::CashFlowProjectionManipulation => 5,
            FraudType::ImproperImpairmentReversal => 3,
            _ => 4,
        }
    }
}

/// Error types for error detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    // Data Entry Errors
    /// Duplicate document entry.
    DuplicateEntry,
    /// Reversed debit/credit amounts.
    ReversedAmount,
    /// Transposed digits in amount.
    TransposedDigits,
    /// Wrong decimal placement.
    DecimalError,
    /// Missing required field.
    MissingField,
    /// Invalid account code.
    InvalidAccount,

    // Timing Errors
    /// Posted to wrong period.
    WrongPeriod,
    /// Backdated entry.
    BackdatedEntry,
    /// Future-dated entry.
    FutureDatedEntry,
    /// Cutoff error.
    CutoffError,

    // Classification Errors
    /// Wrong account classification.
    MisclassifiedAccount,
    /// Wrong cost center.
    WrongCostCenter,
    /// Wrong company code.
    WrongCompanyCode,

    // Calculation Errors
    /// Unbalanced journal entry.
    UnbalancedEntry,
    /// Rounding error.
    RoundingError,
    /// Currency conversion error.
    CurrencyError,
    /// Tax calculation error.
    TaxCalculationError,

    // Accounting Standards Errors (Non-Fraudulent)
    /// Wrong revenue recognition timing (honest mistake).
    RevenueTimingError,
    /// Performance obligation allocation error.
    PoAllocationError,
    /// Lease classification error (operating vs finance).
    LeaseClassificationError,
    /// Lease calculation error (PV, amortization).
    LeaseCalculationError,
    /// Fair value measurement error.
    FairValueError,
    /// Impairment calculation error.
    ImpairmentCalculationError,
    /// Discount rate error.
    DiscountRateError,
    /// Framework application error (IFRS vs GAAP).
    FrameworkApplicationError,
}

impl ErrorType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            ErrorType::RoundingError => 1,
            ErrorType::MissingField => 2,
            ErrorType::TransposedDigits => 2,
            ErrorType::DecimalError => 3,
            ErrorType::DuplicateEntry => 3,
            ErrorType::ReversedAmount => 3,
            ErrorType::WrongPeriod => 4,
            ErrorType::UnbalancedEntry => 5,
            ErrorType::CurrencyError => 4,
            // Accounting Standards Errors
            ErrorType::RevenueTimingError => 4,
            ErrorType::PoAllocationError => 3,
            ErrorType::LeaseClassificationError => 3,
            ErrorType::LeaseCalculationError => 3,
            ErrorType::FairValueError => 4,
            ErrorType::ImpairmentCalculationError => 4,
            ErrorType::DiscountRateError => 3,
            ErrorType::FrameworkApplicationError => 4,
            _ => 3,
        }
    }
}

/// Process issue types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessIssueType {
    // Approval Issues
    /// Approval skipped entirely.
    SkippedApproval,
    /// Late approval (after posting).
    LateApproval,
    /// Missing supporting documentation.
    MissingDocumentation,
    /// Incomplete approval chain.
    IncompleteApprovalChain,

    // Timing Issues
    /// Late posting.
    LatePosting,
    /// Posting outside business hours.
    AfterHoursPosting,
    /// Weekend/holiday posting.
    WeekendPosting,
    /// Rushed period-end posting.
    RushedPeriodEnd,

    // Control Issues
    /// Manual override of system control.
    ManualOverride,
    /// Unusual user access pattern.
    UnusualAccess,
    /// System bypass.
    SystemBypass,
    /// Batch processing anomaly.
    BatchAnomaly,

    // Documentation Issues
    /// Vague or missing description.
    VagueDescription,
    /// Changed after posting.
    PostFactoChange,
    /// Incomplete audit trail.
    IncompleteAuditTrail,
}

impl ProcessIssueType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            ProcessIssueType::VagueDescription => 1,
            ProcessIssueType::LatePosting => 2,
            ProcessIssueType::AfterHoursPosting => 2,
            ProcessIssueType::WeekendPosting => 2,
            ProcessIssueType::SkippedApproval => 4,
            ProcessIssueType::ManualOverride => 4,
            ProcessIssueType::SystemBypass => 5,
            ProcessIssueType::IncompleteAuditTrail => 4,
            _ => 3,
        }
    }
}

/// Statistical anomaly types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatisticalAnomalyType {
    // Amount Anomalies
    /// Amount significantly above normal.
    UnusuallyHighAmount,
    /// Amount significantly below normal.
    UnusuallyLowAmount,
    /// Violates Benford's Law distribution.
    BenfordViolation,
    /// Exact duplicate amount (suspicious).
    ExactDuplicateAmount,
    /// Repeating pattern in amounts.
    RepeatingAmount,

    // Frequency Anomalies
    /// Unusual transaction frequency.
    UnusualFrequency,
    /// Burst of transactions.
    TransactionBurst,
    /// Unusual time of day.
    UnusualTiming,

    // Trend Anomalies
    /// Break in historical trend.
    TrendBreak,
    /// Sudden level shift.
    LevelShift,
    /// Seasonal pattern violation.
    SeasonalAnomaly,

    // Distribution Anomalies
    /// Outlier in distribution.
    StatisticalOutlier,
    /// Change in variance.
    VarianceChange,
    /// Distribution shift.
    DistributionShift,
}

impl StatisticalAnomalyType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            StatisticalAnomalyType::UnusualTiming => 1,
            StatisticalAnomalyType::UnusualFrequency => 2,
            StatisticalAnomalyType::BenfordViolation => 2,
            StatisticalAnomalyType::UnusuallyHighAmount => 3,
            StatisticalAnomalyType::TrendBreak => 3,
            StatisticalAnomalyType::TransactionBurst => 4,
            StatisticalAnomalyType::ExactDuplicateAmount => 3,
            _ => 3,
        }
    }
}

/// Relational/graph anomaly types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationalAnomalyType {
    // Transaction Pattern Anomalies
    /// Circular transaction pattern.
    CircularTransaction,
    /// Unusual account combination.
    UnusualAccountPair,
    /// New trading partner.
    NewCounterparty,
    /// Dormant account suddenly active.
    DormantAccountActivity,

    // Network Anomalies
    /// Unusual network centrality.
    CentralityAnomaly,
    /// Isolated transaction cluster.
    IsolatedCluster,
    /// Bridge node anomaly.
    BridgeNodeAnomaly,
    /// Community structure change.
    CommunityAnomaly,

    // Relationship Anomalies
    /// Missing expected relationship.
    MissingRelationship,
    /// Unexpected relationship.
    UnexpectedRelationship,
    /// Relationship strength change.
    RelationshipStrengthChange,

    // Intercompany Anomalies
    /// Unmatched intercompany transaction.
    UnmatchedIntercompany,
    /// Circular intercompany flow.
    CircularIntercompany,
    /// Transfer pricing anomaly.
    TransferPricingAnomaly,
}

impl RelationalAnomalyType {
    /// Returns severity level (1-5).
    pub fn severity(&self) -> u8 {
        match self {
            RelationalAnomalyType::NewCounterparty => 1,
            RelationalAnomalyType::DormantAccountActivity => 2,
            RelationalAnomalyType::UnusualAccountPair => 2,
            RelationalAnomalyType::CircularTransaction => 4,
            RelationalAnomalyType::CircularIntercompany => 4,
            RelationalAnomalyType::TransferPricingAnomaly => 4,
            RelationalAnomalyType::UnmatchedIntercompany => 3,
            _ => 3,
        }
    }
}

/// A labeled anomaly for supervised learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledAnomaly {
    /// Unique anomaly identifier.
    pub anomaly_id: String,
    /// Type of anomaly.
    pub anomaly_type: AnomalyType,
    /// Document or entity that contains the anomaly.
    pub document_id: String,
    /// Document type (JE, PO, Invoice, etc.).
    pub document_type: String,
    /// Company code.
    pub company_code: String,
    /// Date the anomaly occurred.
    pub anomaly_date: NaiveDate,
    /// Timestamp when detected/injected.
    pub detection_timestamp: NaiveDateTime,
    /// Confidence score (0.0 - 1.0) for injected anomalies.
    pub confidence: f64,
    /// Severity (1-5).
    pub severity: u8,
    /// Description of the anomaly.
    pub description: String,
    /// Related entities (user IDs, account codes, etc.).
    pub related_entities: Vec<String>,
    /// Monetary impact if applicable.
    pub monetary_impact: Option<Decimal>,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
    /// Whether this was injected (true) or naturally occurring (false).
    pub is_injected: bool,
    /// Injection strategy used (if injected) - legacy string field.
    pub injection_strategy: Option<String>,
    /// Cluster ID if part of an anomaly cluster.
    pub cluster_id: Option<String>,

    // ========================================
    // PROVENANCE TRACKING FIELDS (Phase 1.2)
    // ========================================
    /// Hash of the original document before modification.
    /// Enables tracking what the document looked like pre-injection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_document_hash: Option<String>,

    /// Causal reason explaining why this anomaly was injected.
    /// Provides "why" tracking for each anomaly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causal_reason: Option<AnomalyCausalReason>,

    /// Structured injection strategy with parameters.
    /// More detailed than the legacy string-based injection_strategy field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_strategy: Option<InjectionStrategy>,

    /// Parent anomaly ID if this was derived from another anomaly.
    /// Enables anomaly transformation chains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_anomaly_id: Option<String>,

    /// Child anomaly IDs that were derived from this anomaly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub child_anomaly_ids: Vec<String>,

    /// Scenario ID if this anomaly is part of a multi-step scenario.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario_id: Option<String>,

    /// Generation run ID that produced this anomaly.
    /// Enables tracing anomalies back to their generation run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,

    /// Seed used for RNG during generation.
    /// Enables reproducibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_seed: Option<u64>,
}

impl LabeledAnomaly {
    /// Creates a new labeled anomaly.
    pub fn new(
        anomaly_id: String,
        anomaly_type: AnomalyType,
        document_id: String,
        document_type: String,
        company_code: String,
        anomaly_date: NaiveDate,
    ) -> Self {
        let severity = anomaly_type.severity();
        let description = format!(
            "{} - {} in document {}",
            anomaly_type.category(),
            anomaly_type.type_name(),
            document_id
        );

        Self {
            anomaly_id,
            anomaly_type,
            document_id,
            document_type,
            company_code,
            anomaly_date,
            detection_timestamp: chrono::Local::now().naive_local(),
            confidence: 1.0,
            severity,
            description,
            related_entities: Vec::new(),
            monetary_impact: None,
            metadata: HashMap::new(),
            is_injected: true,
            injection_strategy: None,
            cluster_id: None,
            // Provenance fields
            original_document_hash: None,
            causal_reason: None,
            structured_strategy: None,
            parent_anomaly_id: None,
            child_anomaly_ids: Vec::new(),
            scenario_id: None,
            run_id: None,
            generation_seed: None,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Sets the monetary impact.
    pub fn with_monetary_impact(mut self, impact: Decimal) -> Self {
        self.monetary_impact = Some(impact);
        self
    }

    /// Adds a related entity.
    pub fn with_related_entity(mut self, entity: &str) -> Self {
        self.related_entities.push(entity.to_string());
        self
    }

    /// Adds metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Sets the injection strategy (legacy string).
    pub fn with_injection_strategy(mut self, strategy: &str) -> Self {
        self.injection_strategy = Some(strategy.to_string());
        self
    }

    /// Sets the cluster ID.
    pub fn with_cluster(mut self, cluster_id: &str) -> Self {
        self.cluster_id = Some(cluster_id.to_string());
        self
    }

    // ========================================
    // PROVENANCE BUILDER METHODS (Phase 1.2)
    // ========================================

    /// Sets the original document hash for provenance tracking.
    pub fn with_original_document_hash(mut self, hash: &str) -> Self {
        self.original_document_hash = Some(hash.to_string());
        self
    }

    /// Sets the causal reason for this anomaly.
    pub fn with_causal_reason(mut self, reason: AnomalyCausalReason) -> Self {
        self.causal_reason = Some(reason);
        self
    }

    /// Sets the structured injection strategy.
    pub fn with_structured_strategy(mut self, strategy: InjectionStrategy) -> Self {
        // Also set the legacy string field for backward compatibility
        self.injection_strategy = Some(strategy.strategy_type().to_string());
        self.structured_strategy = Some(strategy);
        self
    }

    /// Sets the parent anomaly ID (for anomaly derivation chains).
    pub fn with_parent_anomaly(mut self, parent_id: &str) -> Self {
        self.parent_anomaly_id = Some(parent_id.to_string());
        self
    }

    /// Adds a child anomaly ID.
    pub fn with_child_anomaly(mut self, child_id: &str) -> Self {
        self.child_anomaly_ids.push(child_id.to_string());
        self
    }

    /// Sets the scenario ID for multi-step scenario tracking.
    pub fn with_scenario(mut self, scenario_id: &str) -> Self {
        self.scenario_id = Some(scenario_id.to_string());
        self
    }

    /// Sets the generation run ID.
    pub fn with_run_id(mut self, run_id: &str) -> Self {
        self.run_id = Some(run_id.to_string());
        self
    }

    /// Sets the generation seed for reproducibility.
    pub fn with_generation_seed(mut self, seed: u64) -> Self {
        self.generation_seed = Some(seed);
        self
    }

    /// Sets multiple provenance fields at once for convenience.
    pub fn with_provenance(
        mut self,
        run_id: Option<&str>,
        seed: Option<u64>,
        causal_reason: Option<AnomalyCausalReason>,
    ) -> Self {
        if let Some(id) = run_id {
            self.run_id = Some(id.to_string());
        }
        self.generation_seed = seed;
        self.causal_reason = causal_reason;
        self
    }

    /// Converts to a feature vector for ML.
    ///
    /// Returns a vector of 15 features:
    /// - 6 features: Category one-hot encoding (Fraud, Error, ProcessIssue, Statistical, Relational, Custom)
    /// - 1 feature: Severity (normalized 0-1)
    /// - 1 feature: Confidence
    /// - 1 feature: Has monetary impact (0/1)
    /// - 1 feature: Monetary impact (log-scaled)
    /// - 1 feature: Is intentional (0/1)
    /// - 1 feature: Number of related entities
    /// - 1 feature: Is part of cluster (0/1)
    /// - 1 feature: Is part of scenario (0/1)
    /// - 1 feature: Has parent anomaly (0/1) - indicates derivation
    pub fn to_features(&self) -> Vec<f64> {
        let mut features = Vec::new();

        // Category one-hot encoding
        let categories = [
            "Fraud",
            "Error",
            "ProcessIssue",
            "Statistical",
            "Relational",
            "Custom",
        ];
        for cat in &categories {
            features.push(if self.anomaly_type.category() == *cat {
                1.0
            } else {
                0.0
            });
        }

        // Severity (normalized)
        features.push(self.severity as f64 / 5.0);

        // Confidence
        features.push(self.confidence);

        // Has monetary impact
        features.push(if self.monetary_impact.is_some() {
            1.0
        } else {
            0.0
        });

        // Monetary impact (log-scaled)
        if let Some(impact) = self.monetary_impact {
            let impact_f64: f64 = impact.try_into().unwrap_or(0.0);
            features.push((impact_f64.abs() + 1.0).ln());
        } else {
            features.push(0.0);
        }

        // Is intentional
        features.push(if self.anomaly_type.is_intentional() {
            1.0
        } else {
            0.0
        });

        // Number of related entities
        features.push(self.related_entities.len() as f64);

        // Is part of cluster
        features.push(if self.cluster_id.is_some() { 1.0 } else { 0.0 });

        // Provenance features
        // Is part of scenario
        features.push(if self.scenario_id.is_some() { 1.0 } else { 0.0 });

        // Has parent anomaly (indicates this is a derived anomaly)
        features.push(if self.parent_anomaly_id.is_some() {
            1.0
        } else {
            0.0
        });

        features
    }

    /// Returns the number of features in the feature vector.
    pub fn feature_count() -> usize {
        15 // 6 category + 9 other features
    }

    /// Returns feature names for documentation/ML metadata.
    pub fn feature_names() -> Vec<&'static str> {
        vec![
            "category_fraud",
            "category_error",
            "category_process_issue",
            "category_statistical",
            "category_relational",
            "category_custom",
            "severity_normalized",
            "confidence",
            "has_monetary_impact",
            "monetary_impact_log",
            "is_intentional",
            "related_entity_count",
            "is_clustered",
            "is_scenario_part",
            "is_derived",
        ]
    }
}

/// Summary of anomalies for reporting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnomalySummary {
    /// Total anomaly count.
    pub total_count: usize,
    /// Count by category.
    pub by_category: HashMap<String, usize>,
    /// Count by specific type.
    pub by_type: HashMap<String, usize>,
    /// Count by severity.
    pub by_severity: HashMap<u8, usize>,
    /// Count by company.
    pub by_company: HashMap<String, usize>,
    /// Total monetary impact.
    pub total_monetary_impact: Decimal,
    /// Date range.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Number of clusters.
    pub cluster_count: usize,
}

impl AnomalySummary {
    /// Creates a summary from a list of anomalies.
    pub fn from_anomalies(anomalies: &[LabeledAnomaly]) -> Self {
        let mut summary = AnomalySummary {
            total_count: anomalies.len(),
            ..Default::default()
        };

        let mut min_date: Option<NaiveDate> = None;
        let mut max_date: Option<NaiveDate> = None;
        let mut clusters = std::collections::HashSet::new();

        for anomaly in anomalies {
            // By category
            *summary
                .by_category
                .entry(anomaly.anomaly_type.category().to_string())
                .or_insert(0) += 1;

            // By type
            *summary
                .by_type
                .entry(anomaly.anomaly_type.type_name())
                .or_insert(0) += 1;

            // By severity
            *summary.by_severity.entry(anomaly.severity).or_insert(0) += 1;

            // By company
            *summary
                .by_company
                .entry(anomaly.company_code.clone())
                .or_insert(0) += 1;

            // Monetary impact
            if let Some(impact) = anomaly.monetary_impact {
                summary.total_monetary_impact += impact;
            }

            // Date range
            match min_date {
                None => min_date = Some(anomaly.anomaly_date),
                Some(d) if anomaly.anomaly_date < d => min_date = Some(anomaly.anomaly_date),
                _ => {}
            }
            match max_date {
                None => max_date = Some(anomaly.anomaly_date),
                Some(d) if anomaly.anomaly_date > d => max_date = Some(anomaly.anomaly_date),
                _ => {}
            }

            // Clusters
            if let Some(cluster_id) = &anomaly.cluster_id {
                clusters.insert(cluster_id.clone());
            }
        }

        summary.date_range = min_date.zip(max_date);
        summary.cluster_count = clusters.len();

        summary
    }
}

// ============================================================================
// ENHANCED ANOMALY TAXONOMY (FR-003)
// ============================================================================

/// High-level anomaly category for multi-class classification.
///
/// These categories provide a more granular classification than the base
/// AnomalyType enum, enabling better ML model training and audit reporting.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyCategory {
    // Vendor-related anomalies
    /// Fictitious or shell vendor.
    FictitiousVendor,
    /// Kickback or collusion with vendor.
    VendorKickback,
    /// Related party vendor transactions.
    RelatedPartyVendor,

    // Transaction-related anomalies
    /// Duplicate payment or invoice.
    DuplicatePayment,
    /// Unauthorized transaction.
    UnauthorizedTransaction,
    /// Structured transactions to avoid thresholds.
    StructuredTransaction,

    // Pattern-based anomalies
    /// Circular flow of funds.
    CircularFlow,
    /// Behavioral anomaly (deviation from normal patterns).
    BehavioralAnomaly,
    /// Timing-based anomaly.
    TimingAnomaly,

    // Journal entry anomalies
    /// Manual journal entry anomaly.
    JournalAnomaly,
    /// Manual override of controls.
    ManualOverride,
    /// Missing approval in chain.
    MissingApproval,

    // Statistical anomalies
    /// Statistical outlier.
    StatisticalOutlier,
    /// Distribution anomaly (Benford, etc.).
    DistributionAnomaly,

    // Custom category
    /// User-defined category.
    Custom(String),
}

impl AnomalyCategory {
    /// Derives an AnomalyCategory from an AnomalyType.
    pub fn from_anomaly_type(anomaly_type: &AnomalyType) -> Self {
        match anomaly_type {
            AnomalyType::Fraud(fraud_type) => match fraud_type {
                FraudType::FictitiousVendor | FraudType::ShellCompanyPayment => {
                    AnomalyCategory::FictitiousVendor
                }
                FraudType::Kickback | FraudType::KickbackScheme => AnomalyCategory::VendorKickback,
                FraudType::DuplicatePayment => AnomalyCategory::DuplicatePayment,
                FraudType::SplitTransaction | FraudType::JustBelowThreshold => {
                    AnomalyCategory::StructuredTransaction
                }
                FraudType::SelfApproval
                | FraudType::UnauthorizedApproval
                | FraudType::CollusiveApproval => AnomalyCategory::UnauthorizedTransaction,
                FraudType::TimingAnomaly
                | FraudType::RoundDollarManipulation
                | FraudType::SuspenseAccountAbuse => AnomalyCategory::JournalAnomaly,
                _ => AnomalyCategory::BehavioralAnomaly,
            },
            AnomalyType::Error(error_type) => match error_type {
                ErrorType::DuplicateEntry => AnomalyCategory::DuplicatePayment,
                ErrorType::WrongPeriod
                | ErrorType::BackdatedEntry
                | ErrorType::FutureDatedEntry => AnomalyCategory::TimingAnomaly,
                _ => AnomalyCategory::JournalAnomaly,
            },
            AnomalyType::ProcessIssue(process_type) => match process_type {
                ProcessIssueType::SkippedApproval | ProcessIssueType::IncompleteApprovalChain => {
                    AnomalyCategory::MissingApproval
                }
                ProcessIssueType::ManualOverride | ProcessIssueType::SystemBypass => {
                    AnomalyCategory::ManualOverride
                }
                ProcessIssueType::AfterHoursPosting | ProcessIssueType::WeekendPosting => {
                    AnomalyCategory::TimingAnomaly
                }
                _ => AnomalyCategory::BehavioralAnomaly,
            },
            AnomalyType::Statistical(stat_type) => match stat_type {
                StatisticalAnomalyType::BenfordViolation
                | StatisticalAnomalyType::DistributionShift => AnomalyCategory::DistributionAnomaly,
                _ => AnomalyCategory::StatisticalOutlier,
            },
            AnomalyType::Relational(rel_type) => match rel_type {
                RelationalAnomalyType::CircularTransaction
                | RelationalAnomalyType::CircularIntercompany => AnomalyCategory::CircularFlow,
                _ => AnomalyCategory::BehavioralAnomaly,
            },
            AnomalyType::Custom(s) => AnomalyCategory::Custom(s.clone()),
        }
    }

    /// Returns the category name as a string.
    pub fn name(&self) -> &str {
        match self {
            AnomalyCategory::FictitiousVendor => "fictitious_vendor",
            AnomalyCategory::VendorKickback => "vendor_kickback",
            AnomalyCategory::RelatedPartyVendor => "related_party_vendor",
            AnomalyCategory::DuplicatePayment => "duplicate_payment",
            AnomalyCategory::UnauthorizedTransaction => "unauthorized_transaction",
            AnomalyCategory::StructuredTransaction => "structured_transaction",
            AnomalyCategory::CircularFlow => "circular_flow",
            AnomalyCategory::BehavioralAnomaly => "behavioral_anomaly",
            AnomalyCategory::TimingAnomaly => "timing_anomaly",
            AnomalyCategory::JournalAnomaly => "journal_anomaly",
            AnomalyCategory::ManualOverride => "manual_override",
            AnomalyCategory::MissingApproval => "missing_approval",
            AnomalyCategory::StatisticalOutlier => "statistical_outlier",
            AnomalyCategory::DistributionAnomaly => "distribution_anomaly",
            AnomalyCategory::Custom(s) => s.as_str(),
        }
    }

    /// Returns the ordinal value for ML encoding.
    pub fn ordinal(&self) -> u8 {
        match self {
            AnomalyCategory::FictitiousVendor => 0,
            AnomalyCategory::VendorKickback => 1,
            AnomalyCategory::RelatedPartyVendor => 2,
            AnomalyCategory::DuplicatePayment => 3,
            AnomalyCategory::UnauthorizedTransaction => 4,
            AnomalyCategory::StructuredTransaction => 5,
            AnomalyCategory::CircularFlow => 6,
            AnomalyCategory::BehavioralAnomaly => 7,
            AnomalyCategory::TimingAnomaly => 8,
            AnomalyCategory::JournalAnomaly => 9,
            AnomalyCategory::ManualOverride => 10,
            AnomalyCategory::MissingApproval => 11,
            AnomalyCategory::StatisticalOutlier => 12,
            AnomalyCategory::DistributionAnomaly => 13,
            AnomalyCategory::Custom(_) => 14,
        }
    }

    /// Returns the total number of categories (excluding Custom).
    pub fn category_count() -> usize {
        15 // 14 fixed categories + Custom
    }
}

/// Type of contributing factor for anomaly confidence/severity calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactorType {
    /// Amount deviation from expected value.
    AmountDeviation,
    /// Proximity to approval/reporting threshold.
    ThresholdProximity,
    /// Timing-related anomaly indicator.
    TimingAnomaly,
    /// Entity risk score contribution.
    EntityRisk,
    /// Pattern match confidence.
    PatternMatch,
    /// Frequency deviation from normal.
    FrequencyDeviation,
    /// Relationship-based anomaly indicator.
    RelationshipAnomaly,
    /// Control bypass indicator.
    ControlBypass,
    /// Benford's Law violation.
    BenfordViolation,
    /// Duplicate indicator.
    DuplicateIndicator,
    /// Approval chain issue.
    ApprovalChainIssue,
    /// Documentation gap.
    DocumentationGap,
    /// Custom factor type.
    Custom,
}

impl FactorType {
    /// Returns the factor type name.
    pub fn name(&self) -> &'static str {
        match self {
            FactorType::AmountDeviation => "amount_deviation",
            FactorType::ThresholdProximity => "threshold_proximity",
            FactorType::TimingAnomaly => "timing_anomaly",
            FactorType::EntityRisk => "entity_risk",
            FactorType::PatternMatch => "pattern_match",
            FactorType::FrequencyDeviation => "frequency_deviation",
            FactorType::RelationshipAnomaly => "relationship_anomaly",
            FactorType::ControlBypass => "control_bypass",
            FactorType::BenfordViolation => "benford_violation",
            FactorType::DuplicateIndicator => "duplicate_indicator",
            FactorType::ApprovalChainIssue => "approval_chain_issue",
            FactorType::DocumentationGap => "documentation_gap",
            FactorType::Custom => "custom",
        }
    }
}

/// Evidence supporting a contributing factor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorEvidence {
    /// Source of the evidence (e.g., "transaction_history", "entity_registry").
    pub source: String,
    /// Raw evidence data.
    pub data: HashMap<String, String>,
}

/// A contributing factor to anomaly confidence/severity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributingFactor {
    /// Type of factor.
    pub factor_type: FactorType,
    /// Observed value.
    pub value: f64,
    /// Threshold or expected value.
    pub threshold: f64,
    /// Direction of comparison (true = value > threshold is anomalous).
    pub direction_greater: bool,
    /// Weight of this factor in overall calculation (0.0 - 1.0).
    pub weight: f64,
    /// Human-readable description.
    pub description: String,
    /// Optional supporting evidence.
    pub evidence: Option<FactorEvidence>,
}

impl ContributingFactor {
    /// Creates a new contributing factor.
    pub fn new(
        factor_type: FactorType,
        value: f64,
        threshold: f64,
        direction_greater: bool,
        weight: f64,
        description: &str,
    ) -> Self {
        Self {
            factor_type,
            value,
            threshold,
            direction_greater,
            weight,
            description: description.to_string(),
            evidence: None,
        }
    }

    /// Adds evidence to the factor.
    pub fn with_evidence(mut self, source: &str, data: HashMap<String, String>) -> Self {
        self.evidence = Some(FactorEvidence {
            source: source.to_string(),
            data,
        });
        self
    }

    /// Calculates the factor's contribution to anomaly score.
    pub fn contribution(&self) -> f64 {
        let deviation = if self.direction_greater {
            (self.value - self.threshold).max(0.0)
        } else {
            (self.threshold - self.value).max(0.0)
        };

        // Normalize by threshold to get relative deviation
        let relative_deviation = if self.threshold.abs() > 0.001 {
            deviation / self.threshold.abs()
        } else {
            deviation
        };

        // Apply weight and cap at 1.0
        (relative_deviation * self.weight).min(1.0)
    }
}

/// Enhanced anomaly label with dynamic confidence and severity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedAnomalyLabel {
    /// Base labeled anomaly (backward compatible).
    pub base: LabeledAnomaly,
    /// Enhanced category classification.
    pub category: AnomalyCategory,
    /// Dynamically calculated confidence (0.0 - 1.0).
    pub enhanced_confidence: f64,
    /// Contextually calculated severity (0.0 - 1.0).
    pub enhanced_severity: f64,
    /// Factors contributing to confidence/severity.
    pub contributing_factors: Vec<ContributingFactor>,
    /// Secondary categories (for multi-label classification).
    pub secondary_categories: Vec<AnomalyCategory>,
}

impl EnhancedAnomalyLabel {
    /// Creates an enhanced label from a base labeled anomaly.
    pub fn from_base(base: LabeledAnomaly) -> Self {
        let category = AnomalyCategory::from_anomaly_type(&base.anomaly_type);
        let enhanced_confidence = base.confidence;
        let enhanced_severity = base.severity as f64 / 5.0;

        Self {
            base,
            category,
            enhanced_confidence,
            enhanced_severity,
            contributing_factors: Vec::new(),
            secondary_categories: Vec::new(),
        }
    }

    /// Sets the enhanced confidence.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.enhanced_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Sets the enhanced severity.
    pub fn with_severity(mut self, severity: f64) -> Self {
        self.enhanced_severity = severity.clamp(0.0, 1.0);
        self
    }

    /// Adds a contributing factor.
    pub fn with_factor(mut self, factor: ContributingFactor) -> Self {
        self.contributing_factors.push(factor);
        self
    }

    /// Adds a secondary category.
    pub fn with_secondary_category(mut self, category: AnomalyCategory) -> Self {
        if !self.secondary_categories.contains(&category) && category != self.category {
            self.secondary_categories.push(category);
        }
        self
    }

    /// Converts to an extended feature vector.
    ///
    /// Returns base features (15) + enhanced features (10) = 25 features.
    pub fn to_features(&self) -> Vec<f64> {
        let mut features = self.base.to_features();

        // Enhanced features
        features.push(self.enhanced_confidence);
        features.push(self.enhanced_severity);
        features.push(self.category.ordinal() as f64 / AnomalyCategory::category_count() as f64);
        features.push(self.secondary_categories.len() as f64);
        features.push(self.contributing_factors.len() as f64);

        // Max factor weight
        let max_weight = self
            .contributing_factors
            .iter()
            .map(|f| f.weight)
            .fold(0.0, f64::max);
        features.push(max_weight);

        // Factor type indicators (binary flags for key factor types)
        let has_control_bypass = self
            .contributing_factors
            .iter()
            .any(|f| f.factor_type == FactorType::ControlBypass);
        features.push(if has_control_bypass { 1.0 } else { 0.0 });

        let has_amount_deviation = self
            .contributing_factors
            .iter()
            .any(|f| f.factor_type == FactorType::AmountDeviation);
        features.push(if has_amount_deviation { 1.0 } else { 0.0 });

        let has_timing = self
            .contributing_factors
            .iter()
            .any(|f| f.factor_type == FactorType::TimingAnomaly);
        features.push(if has_timing { 1.0 } else { 0.0 });

        let has_pattern_match = self
            .contributing_factors
            .iter()
            .any(|f| f.factor_type == FactorType::PatternMatch);
        features.push(if has_pattern_match { 1.0 } else { 0.0 });

        features
    }

    /// Returns the number of features in the enhanced feature vector.
    pub fn feature_count() -> usize {
        25 // 15 base + 10 enhanced
    }

    /// Returns feature names for the enhanced feature vector.
    pub fn feature_names() -> Vec<&'static str> {
        let mut names = LabeledAnomaly::feature_names();
        names.extend(vec![
            "enhanced_confidence",
            "enhanced_severity",
            "category_ordinal",
            "secondary_category_count",
            "contributing_factor_count",
            "max_factor_weight",
            "has_control_bypass",
            "has_amount_deviation",
            "has_timing_factor",
            "has_pattern_match",
        ]);
        names
    }
}

// ============================================================================
// MULTI-DIMENSIONAL LABELING (Anomaly Pattern Enhancements)
// ============================================================================

/// Severity level classification for anomalies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SeverityLevel {
    /// Minor issue, low impact.
    Low,
    /// Moderate issue, noticeable impact.
    #[default]
    Medium,
    /// Significant issue, substantial impact.
    High,
    /// Critical issue, severe impact requiring immediate attention.
    Critical,
}

impl SeverityLevel {
    /// Returns the numeric value (1-4) for the severity level.
    pub fn numeric(&self) -> u8 {
        match self {
            SeverityLevel::Low => 1,
            SeverityLevel::Medium => 2,
            SeverityLevel::High => 3,
            SeverityLevel::Critical => 4,
        }
    }

    /// Creates a severity level from a numeric value.
    pub fn from_numeric(value: u8) -> Self {
        match value {
            1 => SeverityLevel::Low,
            2 => SeverityLevel::Medium,
            3 => SeverityLevel::High,
            _ => SeverityLevel::Critical,
        }
    }

    /// Creates a severity level from a normalized score (0.0-1.0).
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.25 => SeverityLevel::Low,
            s if s < 0.50 => SeverityLevel::Medium,
            s if s < 0.75 => SeverityLevel::High,
            _ => SeverityLevel::Critical,
        }
    }

    /// Returns a normalized score (0.0-1.0) for this severity level.
    pub fn to_score(&self) -> f64 {
        match self {
            SeverityLevel::Low => 0.125,
            SeverityLevel::Medium => 0.375,
            SeverityLevel::High => 0.625,
            SeverityLevel::Critical => 0.875,
        }
    }
}

/// Structured severity scoring for anomalies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalySeverity {
    /// Severity level classification.
    pub level: SeverityLevel,
    /// Continuous severity score (0.0-1.0).
    pub score: f64,
    /// Absolute financial impact amount.
    pub financial_impact: Decimal,
    /// Whether this exceeds materiality threshold.
    pub is_material: bool,
    /// Materiality threshold used for determination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materiality_threshold: Option<Decimal>,
}

impl AnomalySeverity {
    /// Creates a new severity assessment.
    pub fn new(level: SeverityLevel, financial_impact: Decimal) -> Self {
        Self {
            level,
            score: level.to_score(),
            financial_impact,
            is_material: false,
            materiality_threshold: None,
        }
    }

    /// Creates severity from a score, auto-determining level.
    pub fn from_score(score: f64, financial_impact: Decimal) -> Self {
        Self {
            level: SeverityLevel::from_score(score),
            score: score.clamp(0.0, 1.0),
            financial_impact,
            is_material: false,
            materiality_threshold: None,
        }
    }

    /// Sets the materiality assessment.
    pub fn with_materiality(mut self, threshold: Decimal) -> Self {
        self.materiality_threshold = Some(threshold);
        self.is_material = self.financial_impact.abs() >= threshold;
        self
    }
}

impl Default for AnomalySeverity {
    fn default() -> Self {
        Self {
            level: SeverityLevel::Medium,
            score: 0.5,
            financial_impact: Decimal::ZERO,
            is_material: false,
            materiality_threshold: None,
        }
    }
}

/// Detection difficulty classification for anomalies.
///
/// Categorizes how difficult an anomaly is to detect, which is useful
/// for ML model benchmarking and audit procedure selection.
///
/// Note: This is distinct from `drift_events::AnomalyDetectionDifficulty` which
/// is used for drift event classification and has different variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AnomalyDetectionDifficulty {
    /// Obvious anomaly, easily caught by basic rules (expected detection rate: 99%).
    Trivial,
    /// Relatively easy to detect with standard procedures (expected detection rate: 90%).
    Easy,
    /// Requires moderate effort or specialized analysis (expected detection rate: 70%).
    #[default]
    Moderate,
    /// Difficult to detect, requires advanced techniques (expected detection rate: 40%).
    Hard,
    /// Expert-level difficulty, requires forensic analysis (expected detection rate: 15%).
    Expert,
}

impl AnomalyDetectionDifficulty {
    /// Returns the expected detection rate for this difficulty level.
    pub fn expected_detection_rate(&self) -> f64 {
        match self {
            AnomalyDetectionDifficulty::Trivial => 0.99,
            AnomalyDetectionDifficulty::Easy => 0.90,
            AnomalyDetectionDifficulty::Moderate => 0.70,
            AnomalyDetectionDifficulty::Hard => 0.40,
            AnomalyDetectionDifficulty::Expert => 0.15,
        }
    }

    /// Returns a numeric difficulty score (0.0-1.0).
    pub fn difficulty_score(&self) -> f64 {
        match self {
            AnomalyDetectionDifficulty::Trivial => 0.05,
            AnomalyDetectionDifficulty::Easy => 0.25,
            AnomalyDetectionDifficulty::Moderate => 0.50,
            AnomalyDetectionDifficulty::Hard => 0.75,
            AnomalyDetectionDifficulty::Expert => 0.95,
        }
    }

    /// Creates a difficulty level from a score (0.0-1.0).
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.15 => AnomalyDetectionDifficulty::Trivial,
            s if s < 0.35 => AnomalyDetectionDifficulty::Easy,
            s if s < 0.55 => AnomalyDetectionDifficulty::Moderate,
            s if s < 0.75 => AnomalyDetectionDifficulty::Hard,
            _ => AnomalyDetectionDifficulty::Expert,
        }
    }

    /// Returns the name of this difficulty level.
    pub fn name(&self) -> &'static str {
        match self {
            AnomalyDetectionDifficulty::Trivial => "trivial",
            AnomalyDetectionDifficulty::Easy => "easy",
            AnomalyDetectionDifficulty::Moderate => "moderate",
            AnomalyDetectionDifficulty::Hard => "hard",
            AnomalyDetectionDifficulty::Expert => "expert",
        }
    }
}

/// Ground truth certainty level for anomaly labels.
///
/// Indicates how certain we are that the label is correct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GroundTruthCertainty {
    /// Definitively known (injected anomaly with full provenance).
    #[default]
    Definite,
    /// Highly probable based on strong evidence.
    Probable,
    /// Possibly an anomaly based on indirect evidence.
    Possible,
}

impl GroundTruthCertainty {
    /// Returns a certainty score (0.0-1.0).
    pub fn certainty_score(&self) -> f64 {
        match self {
            GroundTruthCertainty::Definite => 1.0,
            GroundTruthCertainty::Probable => 0.8,
            GroundTruthCertainty::Possible => 0.5,
        }
    }

    /// Returns the name of this certainty level.
    pub fn name(&self) -> &'static str {
        match self {
            GroundTruthCertainty::Definite => "definite",
            GroundTruthCertainty::Probable => "probable",
            GroundTruthCertainty::Possible => "possible",
        }
    }
}

/// Detection method classification.
///
/// Indicates which detection methods are recommended or effective for an anomaly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// Simple rule-based detection (thresholds, filters).
    RuleBased,
    /// Statistical analysis (distributions, outlier detection).
    Statistical,
    /// Machine learning models (classification, anomaly detection).
    MachineLearning,
    /// Graph-based analysis (network patterns, relationships).
    GraphBased,
    /// Manual forensic audit procedures.
    ForensicAudit,
    /// Combination of multiple methods.
    Hybrid,
}

impl DetectionMethod {
    /// Returns the name of this detection method.
    pub fn name(&self) -> &'static str {
        match self {
            DetectionMethod::RuleBased => "rule_based",
            DetectionMethod::Statistical => "statistical",
            DetectionMethod::MachineLearning => "machine_learning",
            DetectionMethod::GraphBased => "graph_based",
            DetectionMethod::ForensicAudit => "forensic_audit",
            DetectionMethod::Hybrid => "hybrid",
        }
    }

    /// Returns a description of this detection method.
    pub fn description(&self) -> &'static str {
        match self {
            DetectionMethod::RuleBased => "Simple threshold and filter rules",
            DetectionMethod::Statistical => "Statistical distribution analysis",
            DetectionMethod::MachineLearning => "ML classification models",
            DetectionMethod::GraphBased => "Network and relationship analysis",
            DetectionMethod::ForensicAudit => "Manual forensic procedures",
            DetectionMethod::Hybrid => "Combined multi-method approach",
        }
    }
}

/// Extended anomaly label with comprehensive multi-dimensional classification.
///
/// This extends the base `EnhancedAnomalyLabel` with additional fields for
/// severity scoring, detection difficulty, recommended methods, and ground truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedAnomalyLabel {
    /// Base labeled anomaly.
    pub base: LabeledAnomaly,
    /// Enhanced category classification.
    pub category: AnomalyCategory,
    /// Structured severity assessment.
    pub severity: AnomalySeverity,
    /// Detection difficulty classification.
    pub detection_difficulty: AnomalyDetectionDifficulty,
    /// Recommended detection methods for this anomaly.
    pub recommended_methods: Vec<DetectionMethod>,
    /// Key indicators that should trigger detection.
    pub key_indicators: Vec<String>,
    /// Ground truth certainty level.
    pub ground_truth_certainty: GroundTruthCertainty,
    /// Contributing factors to confidence/severity.
    pub contributing_factors: Vec<ContributingFactor>,
    /// Related entity IDs (vendors, customers, employees, etc.).
    pub related_entity_ids: Vec<String>,
    /// Secondary categories for multi-label classification.
    pub secondary_categories: Vec<AnomalyCategory>,
    /// Scheme ID if part of a multi-stage fraud scheme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme_id: Option<String>,
    /// Stage number within a scheme (1-indexed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme_stage: Option<u32>,
    /// Whether this is a near-miss (suspicious but legitimate).
    #[serde(default)]
    pub is_near_miss: bool,
    /// Explanation if this is a near-miss.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub near_miss_explanation: Option<String>,
}

impl ExtendedAnomalyLabel {
    /// Creates an extended label from a base labeled anomaly.
    pub fn from_base(base: LabeledAnomaly) -> Self {
        let category = AnomalyCategory::from_anomaly_type(&base.anomaly_type);
        let severity = AnomalySeverity {
            level: SeverityLevel::from_numeric(base.severity),
            score: base.severity as f64 / 5.0,
            financial_impact: base.monetary_impact.unwrap_or(Decimal::ZERO),
            is_material: false,
            materiality_threshold: None,
        };

        Self {
            base,
            category,
            severity,
            detection_difficulty: AnomalyDetectionDifficulty::Moderate,
            recommended_methods: vec![DetectionMethod::RuleBased],
            key_indicators: Vec::new(),
            ground_truth_certainty: GroundTruthCertainty::Definite,
            contributing_factors: Vec::new(),
            related_entity_ids: Vec::new(),
            secondary_categories: Vec::new(),
            scheme_id: None,
            scheme_stage: None,
            is_near_miss: false,
            near_miss_explanation: None,
        }
    }

    /// Sets the severity assessment.
    pub fn with_severity(mut self, severity: AnomalySeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Sets the detection difficulty.
    pub fn with_difficulty(mut self, difficulty: AnomalyDetectionDifficulty) -> Self {
        self.detection_difficulty = difficulty;
        self
    }

    /// Adds a recommended detection method.
    pub fn with_method(mut self, method: DetectionMethod) -> Self {
        if !self.recommended_methods.contains(&method) {
            self.recommended_methods.push(method);
        }
        self
    }

    /// Sets the recommended detection methods.
    pub fn with_methods(mut self, methods: Vec<DetectionMethod>) -> Self {
        self.recommended_methods = methods;
        self
    }

    /// Adds a key indicator.
    pub fn with_indicator(mut self, indicator: impl Into<String>) -> Self {
        self.key_indicators.push(indicator.into());
        self
    }

    /// Sets the ground truth certainty.
    pub fn with_certainty(mut self, certainty: GroundTruthCertainty) -> Self {
        self.ground_truth_certainty = certainty;
        self
    }

    /// Adds a contributing factor.
    pub fn with_factor(mut self, factor: ContributingFactor) -> Self {
        self.contributing_factors.push(factor);
        self
    }

    /// Adds a related entity ID.
    pub fn with_entity(mut self, entity_id: impl Into<String>) -> Self {
        self.related_entity_ids.push(entity_id.into());
        self
    }

    /// Adds a secondary category.
    pub fn with_secondary_category(mut self, category: AnomalyCategory) -> Self {
        if category != self.category && !self.secondary_categories.contains(&category) {
            self.secondary_categories.push(category);
        }
        self
    }

    /// Sets scheme information.
    pub fn with_scheme(mut self, scheme_id: impl Into<String>, stage: u32) -> Self {
        self.scheme_id = Some(scheme_id.into());
        self.scheme_stage = Some(stage);
        self
    }

    /// Marks this as a near-miss with explanation.
    pub fn as_near_miss(mut self, explanation: impl Into<String>) -> Self {
        self.is_near_miss = true;
        self.near_miss_explanation = Some(explanation.into());
        self
    }

    /// Converts to an extended feature vector for ML.
    ///
    /// Returns base features (15) + extended features (15) = 30 features.
    pub fn to_features(&self) -> Vec<f64> {
        let mut features = self.base.to_features();

        // Extended features
        features.push(self.severity.score);
        features.push(self.severity.level.to_score());
        features.push(if self.severity.is_material { 1.0 } else { 0.0 });
        features.push(self.detection_difficulty.difficulty_score());
        features.push(self.detection_difficulty.expected_detection_rate());
        features.push(self.ground_truth_certainty.certainty_score());
        features.push(self.category.ordinal() as f64 / AnomalyCategory::category_count() as f64);
        features.push(self.secondary_categories.len() as f64);
        features.push(self.contributing_factors.len() as f64);
        features.push(self.key_indicators.len() as f64);
        features.push(self.recommended_methods.len() as f64);
        features.push(self.related_entity_ids.len() as f64);
        features.push(if self.scheme_id.is_some() { 1.0 } else { 0.0 });
        features.push(self.scheme_stage.unwrap_or(0) as f64);
        features.push(if self.is_near_miss { 1.0 } else { 0.0 });

        features
    }

    /// Returns the number of features in the extended feature vector.
    pub fn feature_count() -> usize {
        30 // 15 base + 15 extended
    }

    /// Returns feature names for the extended feature vector.
    pub fn feature_names() -> Vec<&'static str> {
        let mut names = LabeledAnomaly::feature_names();
        names.extend(vec![
            "severity_score",
            "severity_level_score",
            "is_material",
            "difficulty_score",
            "expected_detection_rate",
            "ground_truth_certainty",
            "category_ordinal",
            "secondary_category_count",
            "contributing_factor_count",
            "key_indicator_count",
            "recommended_method_count",
            "related_entity_count",
            "is_part_of_scheme",
            "scheme_stage",
            "is_near_miss",
        ]);
        names
    }
}

// ============================================================================
// MULTI-STAGE FRAUD SCHEME TYPES
// ============================================================================

/// Type of multi-stage fraud scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchemeType {
    /// Gradual embezzlement over time.
    GradualEmbezzlement,
    /// Revenue manipulation across periods.
    RevenueManipulation,
    /// Vendor kickback scheme.
    VendorKickback,
    /// Round-tripping funds through multiple entities.
    RoundTripping,
    /// Ghost employee scheme.
    GhostEmployee,
    /// Expense reimbursement fraud.
    ExpenseReimbursement,
    /// Inventory theft scheme.
    InventoryTheft,
    /// Custom scheme type.
    Custom,
}

impl SchemeType {
    /// Returns the name of this scheme type.
    pub fn name(&self) -> &'static str {
        match self {
            SchemeType::GradualEmbezzlement => "gradual_embezzlement",
            SchemeType::RevenueManipulation => "revenue_manipulation",
            SchemeType::VendorKickback => "vendor_kickback",
            SchemeType::RoundTripping => "round_tripping",
            SchemeType::GhostEmployee => "ghost_employee",
            SchemeType::ExpenseReimbursement => "expense_reimbursement",
            SchemeType::InventoryTheft => "inventory_theft",
            SchemeType::Custom => "custom",
        }
    }

    /// Returns the typical number of stages for this scheme type.
    pub fn typical_stages(&self) -> u32 {
        match self {
            SchemeType::GradualEmbezzlement => 4, // testing, escalation, acceleration, desperation
            SchemeType::RevenueManipulation => 4, // Q4->Q1->Q2->Q4
            SchemeType::VendorKickback => 4,      // setup, inflation, kickback, concealment
            SchemeType::RoundTripping => 3,       // setup, execution, reversal
            SchemeType::GhostEmployee => 3,       // creation, payroll, concealment
            SchemeType::ExpenseReimbursement => 3, // submission, approval, payment
            SchemeType::InventoryTheft => 3,      // access, theft, cover-up
            SchemeType::Custom => 4,
        }
    }
}

/// Status of detection for a fraud scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SchemeDetectionStatus {
    /// Scheme is undetected.
    #[default]
    Undetected,
    /// Under investigation but not confirmed.
    UnderInvestigation,
    /// Partially detected (some transactions flagged).
    PartiallyDetected,
    /// Fully detected and confirmed.
    FullyDetected,
}

/// Reference to a transaction within a scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeTransactionRef {
    /// Document ID of the transaction.
    pub document_id: String,
    /// Transaction date.
    pub date: chrono::NaiveDate,
    /// Transaction amount.
    pub amount: Decimal,
    /// Stage this transaction belongs to.
    pub stage: u32,
    /// Anomaly ID if labeled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anomaly_id: Option<String>,
}

/// Concealment technique used in fraud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConcealmentTechnique {
    /// Document manipulation or forgery.
    DocumentManipulation,
    /// Circumventing approval processes.
    ApprovalCircumvention,
    /// Exploiting timing (period-end, holidays).
    TimingExploitation,
    /// Transaction splitting to avoid thresholds.
    TransactionSplitting,
    /// Account misclassification.
    AccountMisclassification,
    /// Collusion with other employees.
    Collusion,
    /// Data alteration or deletion.
    DataAlteration,
    /// Creating false documentation.
    FalseDocumentation,
}

impl ConcealmentTechnique {
    /// Returns the difficulty bonus this technique adds.
    pub fn difficulty_bonus(&self) -> f64 {
        match self {
            ConcealmentTechnique::DocumentManipulation => 0.20,
            ConcealmentTechnique::ApprovalCircumvention => 0.15,
            ConcealmentTechnique::TimingExploitation => 0.10,
            ConcealmentTechnique::TransactionSplitting => 0.15,
            ConcealmentTechnique::AccountMisclassification => 0.10,
            ConcealmentTechnique::Collusion => 0.25,
            ConcealmentTechnique::DataAlteration => 0.20,
            ConcealmentTechnique::FalseDocumentation => 0.15,
        }
    }
}

// ============================================================================
// ACFE-ALIGNED FRAUD TAXONOMY
// ============================================================================
//
// Based on the Association of Certified Fraud Examiners (ACFE) Report to the
// Nations: Occupational Fraud Classification System. This taxonomy provides
// ACFE-aligned categories, schemes, and calibration data.

/// ACFE-aligned fraud categories based on the Occupational Fraud Tree.
///
/// ACFE Report to the Nations statistics (typical):
/// - Asset Misappropriation: 86% of cases, $100k median loss
/// - Corruption: 33% of cases, $150k median loss
/// - Financial Statement Fraud: 10% of cases, $954k median loss
///
/// Note: Percentages sum to >100% because some schemes fall into multiple categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AcfeFraudCategory {
    /// Theft of organizational assets (cash, inventory, equipment).
    /// Most common (86% of cases) but typically lowest median loss ($100k).
    #[default]
    AssetMisappropriation,
    /// Abuse of position for personal gain through bribery, kickbacks, conflicts of interest.
    /// Medium frequency (33% of cases), medium median loss ($150k).
    Corruption,
    /// Intentional misstatement of financial statements.
    /// Least common (10% of cases) but highest median loss ($954k).
    FinancialStatementFraud,
}

impl AcfeFraudCategory {
    /// Returns the name of this category.
    pub fn name(&self) -> &'static str {
        match self {
            AcfeFraudCategory::AssetMisappropriation => "asset_misappropriation",
            AcfeFraudCategory::Corruption => "corruption",
            AcfeFraudCategory::FinancialStatementFraud => "financial_statement_fraud",
        }
    }

    /// Returns the typical percentage of occupational fraud cases (from ACFE reports).
    pub fn typical_occurrence_rate(&self) -> f64 {
        match self {
            AcfeFraudCategory::AssetMisappropriation => 0.86,
            AcfeFraudCategory::Corruption => 0.33,
            AcfeFraudCategory::FinancialStatementFraud => 0.10,
        }
    }

    /// Returns the typical median loss amount (from ACFE reports).
    pub fn typical_median_loss(&self) -> Decimal {
        match self {
            AcfeFraudCategory::AssetMisappropriation => Decimal::new(100_000, 0),
            AcfeFraudCategory::Corruption => Decimal::new(150_000, 0),
            AcfeFraudCategory::FinancialStatementFraud => Decimal::new(954_000, 0),
        }
    }

    /// Returns the typical detection time in months (from ACFE reports).
    pub fn typical_detection_months(&self) -> u32 {
        match self {
            AcfeFraudCategory::AssetMisappropriation => 12,
            AcfeFraudCategory::Corruption => 18,
            AcfeFraudCategory::FinancialStatementFraud => 24,
        }
    }
}

/// Cash-based fraud schemes under Asset Misappropriation.
///
/// Organized according to the ACFE Fraud Tree:
/// - Theft of Cash on Hand
/// - Theft of Cash Receipts
/// - Fraudulent Disbursements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CashFraudScheme {
    // ========== Theft of Cash on Hand ==========
    /// Stealing cash from cash drawers or safes after it has been recorded.
    Larceny,
    /// Stealing cash before it is recorded in the books (intercepts receipts).
    Skimming,

    // ========== Theft of Cash Receipts ==========
    /// Skimming from sales transactions before recording.
    SalesSkimming,
    /// Intercepting customer payments on accounts receivable.
    ReceivablesSkimming,
    /// Creating false refunds to pocket the difference.
    RefundSchemes,

    // ========== Fraudulent Disbursements - Billing Schemes ==========
    /// Creating fictitious vendors to invoice and pay.
    ShellCompany,
    /// Manipulating payments to legitimate vendors for personal gain.
    NonAccompliceVendor,
    /// Using company funds for personal purchases.
    PersonalPurchases,

    // ========== Fraudulent Disbursements - Payroll Schemes ==========
    /// Creating fake employees to collect wages.
    GhostEmployee,
    /// Falsifying hours worked, sales commissions, or salary rates.
    FalsifiedWages,
    /// Manipulating commission calculations.
    CommissionSchemes,

    // ========== Fraudulent Disbursements - Expense Reimbursement ==========
    /// Claiming non-business expenses as business expenses.
    MischaracterizedExpenses,
    /// Inflating legitimate expense amounts.
    OverstatedExpenses,
    /// Creating completely fictitious expenses.
    FictitiousExpenses,

    // ========== Fraudulent Disbursements - Check/Payment Tampering ==========
    /// Forging the signature of an authorized check signer.
    ForgedMaker,
    /// Intercepting and altering the endorsement on legitimate checks.
    ForgedEndorsement,
    /// Altering the payee on a legitimate check.
    AlteredPayee,
    /// Authorized signer writing checks for personal benefit.
    AuthorizedMaker,

    // ========== Fraudulent Disbursements - Register/POS Schemes ==========
    /// Creating false voided transactions.
    FalseVoids,
    /// Processing fictitious refunds.
    FalseRefunds,
}

impl CashFraudScheme {
    /// Returns the ACFE category this scheme belongs to.
    pub fn category(&self) -> AcfeFraudCategory {
        AcfeFraudCategory::AssetMisappropriation
    }

    /// Returns the subcategory within the ACFE Fraud Tree.
    pub fn subcategory(&self) -> &'static str {
        match self {
            CashFraudScheme::Larceny | CashFraudScheme::Skimming => "theft_of_cash_on_hand",
            CashFraudScheme::SalesSkimming
            | CashFraudScheme::ReceivablesSkimming
            | CashFraudScheme::RefundSchemes => "theft_of_cash_receipts",
            CashFraudScheme::ShellCompany
            | CashFraudScheme::NonAccompliceVendor
            | CashFraudScheme::PersonalPurchases => "billing_schemes",
            CashFraudScheme::GhostEmployee
            | CashFraudScheme::FalsifiedWages
            | CashFraudScheme::CommissionSchemes => "payroll_schemes",
            CashFraudScheme::MischaracterizedExpenses
            | CashFraudScheme::OverstatedExpenses
            | CashFraudScheme::FictitiousExpenses => "expense_reimbursement",
            CashFraudScheme::ForgedMaker
            | CashFraudScheme::ForgedEndorsement
            | CashFraudScheme::AlteredPayee
            | CashFraudScheme::AuthorizedMaker => "check_tampering",
            CashFraudScheme::FalseVoids | CashFraudScheme::FalseRefunds => "register_schemes",
        }
    }

    /// Returns the typical severity (1-5) for this scheme.
    pub fn severity(&self) -> u8 {
        match self {
            // Lower severity - often small amounts, easier to detect
            CashFraudScheme::FalseVoids
            | CashFraudScheme::FalseRefunds
            | CashFraudScheme::MischaracterizedExpenses => 3,
            // Medium severity
            CashFraudScheme::OverstatedExpenses
            | CashFraudScheme::Skimming
            | CashFraudScheme::Larceny
            | CashFraudScheme::PersonalPurchases
            | CashFraudScheme::FalsifiedWages => 4,
            // Higher severity - larger amounts, harder to detect
            CashFraudScheme::ShellCompany
            | CashFraudScheme::GhostEmployee
            | CashFraudScheme::FictitiousExpenses
            | CashFraudScheme::ForgedMaker
            | CashFraudScheme::AuthorizedMaker => 5,
            _ => 4,
        }
    }

    /// Returns the typical detection difficulty.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        match self {
            // Easy to detect with basic controls
            CashFraudScheme::FalseVoids | CashFraudScheme::FalseRefunds => {
                AnomalyDetectionDifficulty::Easy
            }
            // Moderate - requires reconciliation
            CashFraudScheme::Larceny | CashFraudScheme::OverstatedExpenses => {
                AnomalyDetectionDifficulty::Moderate
            }
            // Hard - requires sophisticated analysis
            CashFraudScheme::Skimming
            | CashFraudScheme::ShellCompany
            | CashFraudScheme::GhostEmployee => AnomalyDetectionDifficulty::Hard,
            // Expert level
            CashFraudScheme::SalesSkimming | CashFraudScheme::ReceivablesSkimming => {
                AnomalyDetectionDifficulty::Expert
            }
            _ => AnomalyDetectionDifficulty::Moderate,
        }
    }

    /// Returns all variants for iteration.
    pub fn all_variants() -> &'static [CashFraudScheme] {
        &[
            CashFraudScheme::Larceny,
            CashFraudScheme::Skimming,
            CashFraudScheme::SalesSkimming,
            CashFraudScheme::ReceivablesSkimming,
            CashFraudScheme::RefundSchemes,
            CashFraudScheme::ShellCompany,
            CashFraudScheme::NonAccompliceVendor,
            CashFraudScheme::PersonalPurchases,
            CashFraudScheme::GhostEmployee,
            CashFraudScheme::FalsifiedWages,
            CashFraudScheme::CommissionSchemes,
            CashFraudScheme::MischaracterizedExpenses,
            CashFraudScheme::OverstatedExpenses,
            CashFraudScheme::FictitiousExpenses,
            CashFraudScheme::ForgedMaker,
            CashFraudScheme::ForgedEndorsement,
            CashFraudScheme::AlteredPayee,
            CashFraudScheme::AuthorizedMaker,
            CashFraudScheme::FalseVoids,
            CashFraudScheme::FalseRefunds,
        ]
    }
}

/// Inventory and Other Asset fraud schemes under Asset Misappropriation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetFraudScheme {
    // ========== Inventory Schemes ==========
    /// Misusing or converting inventory for personal benefit.
    InventoryMisuse,
    /// Stealing physical inventory items.
    InventoryTheft,
    /// Manipulating purchasing to facilitate theft.
    InventoryPurchasingScheme,
    /// Manipulating receiving/shipping to steal inventory.
    InventoryReceivingScheme,

    // ========== Other Asset Schemes ==========
    /// Misusing company equipment or vehicles.
    EquipmentMisuse,
    /// Theft of company equipment, tools, or supplies.
    EquipmentTheft,
    /// Unauthorized access to or theft of intellectual property.
    IntellectualPropertyTheft,
    /// Using company time/resources for personal business.
    TimeTheft,
}

impl AssetFraudScheme {
    /// Returns the ACFE category this scheme belongs to.
    pub fn category(&self) -> AcfeFraudCategory {
        AcfeFraudCategory::AssetMisappropriation
    }

    /// Returns the subcategory within the ACFE Fraud Tree.
    pub fn subcategory(&self) -> &'static str {
        match self {
            AssetFraudScheme::InventoryMisuse
            | AssetFraudScheme::InventoryTheft
            | AssetFraudScheme::InventoryPurchasingScheme
            | AssetFraudScheme::InventoryReceivingScheme => "inventory",
            _ => "other_assets",
        }
    }

    /// Returns the typical severity (1-5) for this scheme.
    pub fn severity(&self) -> u8 {
        match self {
            AssetFraudScheme::TimeTheft | AssetFraudScheme::EquipmentMisuse => 2,
            AssetFraudScheme::InventoryMisuse | AssetFraudScheme::EquipmentTheft => 3,
            AssetFraudScheme::InventoryTheft
            | AssetFraudScheme::InventoryPurchasingScheme
            | AssetFraudScheme::InventoryReceivingScheme => 4,
            AssetFraudScheme::IntellectualPropertyTheft => 5,
        }
    }
}

/// Corruption schemes under the ACFE Fraud Tree.
///
/// Corruption schemes involve the wrongful use of influence in a business
/// transaction to procure personal benefit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorruptionScheme {
    // ========== Conflicts of Interest ==========
    /// Employee has undisclosed financial interest in purchasing decisions.
    PurchasingConflict,
    /// Employee has undisclosed relationship with customer/vendor.
    SalesConflict,
    /// Employee owns or has interest in competing business.
    OutsideBusinessInterest,
    /// Employee makes decisions benefiting family members.
    NepotismConflict,

    // ========== Bribery ==========
    /// Kickback payments from vendors for favorable treatment.
    InvoiceKickback,
    /// Collusion among vendors to inflate prices.
    BidRigging,
    /// Other cash payments for favorable decisions.
    CashBribery,
    /// Bribery of government officials.
    PublicOfficial,

    // ========== Illegal Gratuities ==========
    /// Gifts given after favorable decisions (not agreed in advance).
    IllegalGratuity,

    // ========== Economic Extortion ==========
    /// Demanding payment under threat of adverse action.
    EconomicExtortion,
}

impl CorruptionScheme {
    /// Returns the ACFE category this scheme belongs to.
    pub fn category(&self) -> AcfeFraudCategory {
        AcfeFraudCategory::Corruption
    }

    /// Returns the subcategory within the ACFE Fraud Tree.
    pub fn subcategory(&self) -> &'static str {
        match self {
            CorruptionScheme::PurchasingConflict
            | CorruptionScheme::SalesConflict
            | CorruptionScheme::OutsideBusinessInterest
            | CorruptionScheme::NepotismConflict => "conflicts_of_interest",
            CorruptionScheme::InvoiceKickback
            | CorruptionScheme::BidRigging
            | CorruptionScheme::CashBribery
            | CorruptionScheme::PublicOfficial => "bribery",
            CorruptionScheme::IllegalGratuity => "illegal_gratuities",
            CorruptionScheme::EconomicExtortion => "economic_extortion",
        }
    }

    /// Returns the typical severity (1-5) for this scheme.
    pub fn severity(&self) -> u8 {
        match self {
            // Lower severity conflicts of interest
            CorruptionScheme::NepotismConflict => 3,
            // Medium severity
            CorruptionScheme::PurchasingConflict
            | CorruptionScheme::SalesConflict
            | CorruptionScheme::OutsideBusinessInterest
            | CorruptionScheme::IllegalGratuity => 4,
            // High severity - active corruption
            CorruptionScheme::InvoiceKickback
            | CorruptionScheme::BidRigging
            | CorruptionScheme::CashBribery
            | CorruptionScheme::EconomicExtortion => 5,
            // Highest severity - involves public officials
            CorruptionScheme::PublicOfficial => 5,
        }
    }

    /// Returns the typical detection difficulty.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        match self {
            // Easier to detect with proper disclosure requirements
            CorruptionScheme::NepotismConflict | CorruptionScheme::OutsideBusinessInterest => {
                AnomalyDetectionDifficulty::Moderate
            }
            // Hard - requires transaction pattern analysis
            CorruptionScheme::PurchasingConflict
            | CorruptionScheme::SalesConflict
            | CorruptionScheme::BidRigging => AnomalyDetectionDifficulty::Hard,
            // Expert level - deliberate concealment
            CorruptionScheme::InvoiceKickback
            | CorruptionScheme::CashBribery
            | CorruptionScheme::PublicOfficial
            | CorruptionScheme::IllegalGratuity
            | CorruptionScheme::EconomicExtortion => AnomalyDetectionDifficulty::Expert,
        }
    }

    /// Returns all variants for iteration.
    pub fn all_variants() -> &'static [CorruptionScheme] {
        &[
            CorruptionScheme::PurchasingConflict,
            CorruptionScheme::SalesConflict,
            CorruptionScheme::OutsideBusinessInterest,
            CorruptionScheme::NepotismConflict,
            CorruptionScheme::InvoiceKickback,
            CorruptionScheme::BidRigging,
            CorruptionScheme::CashBribery,
            CorruptionScheme::PublicOfficial,
            CorruptionScheme::IllegalGratuity,
            CorruptionScheme::EconomicExtortion,
        ]
    }
}

/// Financial Statement Fraud schemes under the ACFE Fraud Tree.
///
/// Financial statement fraud involves the intentional misstatement or omission
/// of material information in financial reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FinancialStatementScheme {
    // ========== Asset/Revenue Overstatement ==========
    /// Recording revenue before it is earned.
    PrematureRevenue,
    /// Deferring expenses to future periods.
    DelayedExpenses,
    /// Recording revenue for transactions that never occurred.
    FictitiousRevenues,
    /// Failing to record known liabilities.
    ConcealedLiabilities,
    /// Overstating the value of assets.
    ImproperAssetValuations,
    /// Omitting or misstating required disclosures.
    ImproperDisclosures,
    /// Manipulating timing of revenue recognition (channel stuffing).
    ChannelStuffing,
    /// Recognizing bill-and-hold revenue improperly.
    BillAndHold,
    /// Capitalizing expenses that should be expensed.
    ImproperCapitalization,

    // ========== Asset/Revenue Understatement ==========
    /// Understating revenue (often for tax purposes).
    UnderstatedRevenues,
    /// Recording excessive expenses.
    OverstatedExpenses,
    /// Recording excessive liabilities or reserves.
    OverstatedLiabilities,
    /// Undervaluing assets for writedowns/reserves.
    ImproperAssetWritedowns,
}

impl FinancialStatementScheme {
    /// Returns the ACFE category this scheme belongs to.
    pub fn category(&self) -> AcfeFraudCategory {
        AcfeFraudCategory::FinancialStatementFraud
    }

    /// Returns the subcategory within the ACFE Fraud Tree.
    pub fn subcategory(&self) -> &'static str {
        match self {
            FinancialStatementScheme::UnderstatedRevenues
            | FinancialStatementScheme::OverstatedExpenses
            | FinancialStatementScheme::OverstatedLiabilities
            | FinancialStatementScheme::ImproperAssetWritedowns => "understatement",
            _ => "overstatement",
        }
    }

    /// Returns the typical severity (1-5) for this scheme.
    pub fn severity(&self) -> u8 {
        // All financial statement fraud is high severity
        5
    }

    /// Returns the typical detection difficulty.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        match self {
            // Easier to detect with good analytics
            FinancialStatementScheme::ChannelStuffing
            | FinancialStatementScheme::DelayedExpenses => AnomalyDetectionDifficulty::Moderate,
            // Hard - requires deep analysis
            FinancialStatementScheme::PrematureRevenue
            | FinancialStatementScheme::ImproperCapitalization
            | FinancialStatementScheme::ImproperAssetWritedowns => AnomalyDetectionDifficulty::Hard,
            // Expert level
            FinancialStatementScheme::FictitiousRevenues
            | FinancialStatementScheme::ConcealedLiabilities
            | FinancialStatementScheme::ImproperAssetValuations
            | FinancialStatementScheme::ImproperDisclosures
            | FinancialStatementScheme::BillAndHold => AnomalyDetectionDifficulty::Expert,
            _ => AnomalyDetectionDifficulty::Hard,
        }
    }

    /// Returns all variants for iteration.
    pub fn all_variants() -> &'static [FinancialStatementScheme] {
        &[
            FinancialStatementScheme::PrematureRevenue,
            FinancialStatementScheme::DelayedExpenses,
            FinancialStatementScheme::FictitiousRevenues,
            FinancialStatementScheme::ConcealedLiabilities,
            FinancialStatementScheme::ImproperAssetValuations,
            FinancialStatementScheme::ImproperDisclosures,
            FinancialStatementScheme::ChannelStuffing,
            FinancialStatementScheme::BillAndHold,
            FinancialStatementScheme::ImproperCapitalization,
            FinancialStatementScheme::UnderstatedRevenues,
            FinancialStatementScheme::OverstatedExpenses,
            FinancialStatementScheme::OverstatedLiabilities,
            FinancialStatementScheme::ImproperAssetWritedowns,
        ]
    }
}

/// Unified ACFE scheme type that encompasses all fraud schemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AcfeScheme {
    /// Cash-based fraud schemes.
    Cash(CashFraudScheme),
    /// Inventory and other asset fraud schemes.
    Asset(AssetFraudScheme),
    /// Corruption schemes.
    Corruption(CorruptionScheme),
    /// Financial statement fraud schemes.
    FinancialStatement(FinancialStatementScheme),
}

impl AcfeScheme {
    /// Returns the ACFE category this scheme belongs to.
    pub fn category(&self) -> AcfeFraudCategory {
        match self {
            AcfeScheme::Cash(s) => s.category(),
            AcfeScheme::Asset(s) => s.category(),
            AcfeScheme::Corruption(s) => s.category(),
            AcfeScheme::FinancialStatement(s) => s.category(),
        }
    }

    /// Returns the severity (1-5) for this scheme.
    pub fn severity(&self) -> u8 {
        match self {
            AcfeScheme::Cash(s) => s.severity(),
            AcfeScheme::Asset(s) => s.severity(),
            AcfeScheme::Corruption(s) => s.severity(),
            AcfeScheme::FinancialStatement(s) => s.severity(),
        }
    }

    /// Returns the detection difficulty for this scheme.
    pub fn detection_difficulty(&self) -> AnomalyDetectionDifficulty {
        match self {
            AcfeScheme::Cash(s) => s.detection_difficulty(),
            AcfeScheme::Asset(_) => AnomalyDetectionDifficulty::Moderate,
            AcfeScheme::Corruption(s) => s.detection_difficulty(),
            AcfeScheme::FinancialStatement(s) => s.detection_difficulty(),
        }
    }
}

/// How a fraud was detected (from ACFE statistics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AcfeDetectionMethod {
    /// Tip from employee, customer, vendor, or anonymous source.
    Tip,
    /// Internal audit procedures.
    InternalAudit,
    /// Management review and oversight.
    ManagementReview,
    /// External audit procedures.
    ExternalAudit,
    /// Account reconciliation discrepancies.
    AccountReconciliation,
    /// Document examination.
    DocumentExamination,
    /// Discovered by accident.
    ByAccident,
    /// Automated monitoring/IT controls.
    ItControls,
    /// Surveillance or investigation.
    Surveillance,
    /// Confession by perpetrator.
    Confession,
    /// Law enforcement notification.
    LawEnforcement,
    /// Other detection method.
    Other,
}

impl AcfeDetectionMethod {
    /// Returns the typical percentage of frauds detected by this method (from ACFE reports).
    pub fn typical_detection_rate(&self) -> f64 {
        match self {
            AcfeDetectionMethod::Tip => 0.42,
            AcfeDetectionMethod::InternalAudit => 0.16,
            AcfeDetectionMethod::ManagementReview => 0.12,
            AcfeDetectionMethod::ExternalAudit => 0.04,
            AcfeDetectionMethod::AccountReconciliation => 0.05,
            AcfeDetectionMethod::DocumentExamination => 0.04,
            AcfeDetectionMethod::ByAccident => 0.06,
            AcfeDetectionMethod::ItControls => 0.03,
            AcfeDetectionMethod::Surveillance => 0.02,
            AcfeDetectionMethod::Confession => 0.02,
            AcfeDetectionMethod::LawEnforcement => 0.01,
            AcfeDetectionMethod::Other => 0.03,
        }
    }

    /// Returns all variants for iteration.
    pub fn all_variants() -> &'static [AcfeDetectionMethod] {
        &[
            AcfeDetectionMethod::Tip,
            AcfeDetectionMethod::InternalAudit,
            AcfeDetectionMethod::ManagementReview,
            AcfeDetectionMethod::ExternalAudit,
            AcfeDetectionMethod::AccountReconciliation,
            AcfeDetectionMethod::DocumentExamination,
            AcfeDetectionMethod::ByAccident,
            AcfeDetectionMethod::ItControls,
            AcfeDetectionMethod::Surveillance,
            AcfeDetectionMethod::Confession,
            AcfeDetectionMethod::LawEnforcement,
            AcfeDetectionMethod::Other,
        ]
    }
}

/// Department/position of perpetrator (from ACFE statistics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerpetratorDepartment {
    /// Accounting, finance, or bookkeeping.
    Accounting,
    /// Operations or manufacturing.
    Operations,
    /// Executive/upper management.
    Executive,
    /// Sales.
    Sales,
    /// Customer service.
    CustomerService,
    /// Purchasing/procurement.
    Purchasing,
    /// Information technology.
    It,
    /// Human resources.
    HumanResources,
    /// Administrative/clerical.
    Administrative,
    /// Warehouse/inventory.
    Warehouse,
    /// Board of directors.
    BoardOfDirectors,
    /// Other department.
    Other,
}

impl PerpetratorDepartment {
    /// Returns the typical percentage of frauds by department (from ACFE reports).
    pub fn typical_occurrence_rate(&self) -> f64 {
        match self {
            PerpetratorDepartment::Accounting => 0.21,
            PerpetratorDepartment::Operations => 0.17,
            PerpetratorDepartment::Executive => 0.12,
            PerpetratorDepartment::Sales => 0.11,
            PerpetratorDepartment::CustomerService => 0.07,
            PerpetratorDepartment::Purchasing => 0.06,
            PerpetratorDepartment::It => 0.05,
            PerpetratorDepartment::HumanResources => 0.04,
            PerpetratorDepartment::Administrative => 0.04,
            PerpetratorDepartment::Warehouse => 0.03,
            PerpetratorDepartment::BoardOfDirectors => 0.02,
            PerpetratorDepartment::Other => 0.08,
        }
    }

    /// Returns the typical median loss by perpetrator department.
    pub fn typical_median_loss(&self) -> Decimal {
        match self {
            PerpetratorDepartment::Executive => Decimal::new(600_000, 0),
            PerpetratorDepartment::BoardOfDirectors => Decimal::new(500_000, 0),
            PerpetratorDepartment::Sales => Decimal::new(150_000, 0),
            PerpetratorDepartment::Accounting => Decimal::new(130_000, 0),
            PerpetratorDepartment::Purchasing => Decimal::new(120_000, 0),
            PerpetratorDepartment::Operations => Decimal::new(100_000, 0),
            PerpetratorDepartment::It => Decimal::new(100_000, 0),
            _ => Decimal::new(80_000, 0),
        }
    }
}

/// Perpetrator position level (from ACFE statistics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerpetratorLevel {
    /// Entry-level employee.
    Employee,
    /// Manager or supervisor.
    Manager,
    /// Owner, executive, or C-level.
    OwnerExecutive,
}

impl PerpetratorLevel {
    /// Returns the typical percentage of frauds by position level.
    pub fn typical_occurrence_rate(&self) -> f64 {
        match self {
            PerpetratorLevel::Employee => 0.42,
            PerpetratorLevel::Manager => 0.36,
            PerpetratorLevel::OwnerExecutive => 0.22,
        }
    }

    /// Returns the typical median loss by position level.
    pub fn typical_median_loss(&self) -> Decimal {
        match self {
            PerpetratorLevel::Employee => Decimal::new(50_000, 0),
            PerpetratorLevel::Manager => Decimal::new(125_000, 0),
            PerpetratorLevel::OwnerExecutive => Decimal::new(337_000, 0),
        }
    }
}

/// ACFE Calibration data for fraud generation.
///
/// Contains statistical parameters based on ACFE Report to the Nations
/// for realistic fraud pattern generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcfeCalibration {
    /// Overall median loss for occupational fraud ($117,000 typical).
    pub median_loss: Decimal,
    /// Median duration in months before detection (12 months typical).
    pub median_duration_months: u32,
    /// Distribution of fraud by category.
    pub category_distribution: HashMap<String, f64>,
    /// Distribution of detection methods.
    pub detection_method_distribution: HashMap<String, f64>,
    /// Distribution by perpetrator department.
    pub department_distribution: HashMap<String, f64>,
    /// Distribution by perpetrator level.
    pub level_distribution: HashMap<String, f64>,
    /// Average number of red flags per fraud case.
    pub avg_red_flags_per_case: f64,
    /// Percentage of frauds involving collusion.
    pub collusion_rate: f64,
}

impl Default for AcfeCalibration {
    fn default() -> Self {
        let mut category_distribution = HashMap::new();
        category_distribution.insert("asset_misappropriation".to_string(), 0.86);
        category_distribution.insert("corruption".to_string(), 0.33);
        category_distribution.insert("financial_statement_fraud".to_string(), 0.10);

        let mut detection_method_distribution = HashMap::new();
        for method in AcfeDetectionMethod::all_variants() {
            detection_method_distribution.insert(
                format!("{:?}", method).to_lowercase(),
                method.typical_detection_rate(),
            );
        }

        let mut department_distribution = HashMap::new();
        department_distribution.insert("accounting".to_string(), 0.21);
        department_distribution.insert("operations".to_string(), 0.17);
        department_distribution.insert("executive".to_string(), 0.12);
        department_distribution.insert("sales".to_string(), 0.11);
        department_distribution.insert("customer_service".to_string(), 0.07);
        department_distribution.insert("purchasing".to_string(), 0.06);
        department_distribution.insert("other".to_string(), 0.26);

        let mut level_distribution = HashMap::new();
        level_distribution.insert("employee".to_string(), 0.42);
        level_distribution.insert("manager".to_string(), 0.36);
        level_distribution.insert("owner_executive".to_string(), 0.22);

        Self {
            median_loss: Decimal::new(117_000, 0),
            median_duration_months: 12,
            category_distribution,
            detection_method_distribution,
            department_distribution,
            level_distribution,
            avg_red_flags_per_case: 2.8,
            collusion_rate: 0.50,
        }
    }
}

impl AcfeCalibration {
    /// Creates a new ACFE calibration with the given parameters.
    pub fn new(median_loss: Decimal, median_duration_months: u32) -> Self {
        Self {
            median_loss,
            median_duration_months,
            ..Self::default()
        }
    }

    /// Returns the median loss for a specific category.
    pub fn median_loss_for_category(&self, category: AcfeFraudCategory) -> Decimal {
        category.typical_median_loss()
    }

    /// Returns the median duration for a specific category.
    pub fn median_duration_for_category(&self, category: AcfeFraudCategory) -> u32 {
        category.typical_detection_months()
    }

    /// Validates the calibration data.
    pub fn validate(&self) -> Result<(), String> {
        if self.median_loss <= Decimal::ZERO {
            return Err("Median loss must be positive".to_string());
        }
        if self.median_duration_months == 0 {
            return Err("Median duration must be at least 1 month".to_string());
        }
        if self.collusion_rate < 0.0 || self.collusion_rate > 1.0 {
            return Err("Collusion rate must be between 0.0 and 1.0".to_string());
        }
        Ok(())
    }
}

/// Fraud Triangle components (Pressure, Opportunity, Rationalization).
///
/// The fraud triangle is a model for explaining the factors that cause
/// someone to commit occupational fraud.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudTriangle {
    /// Pressure or incentive to commit fraud.
    pub pressure: PressureType,
    /// Opportunity factors that enable fraud.
    pub opportunities: Vec<OpportunityFactor>,
    /// Rationalization used to justify the fraud.
    pub rationalization: Rationalization,
}

impl FraudTriangle {
    /// Creates a new fraud triangle.
    pub fn new(
        pressure: PressureType,
        opportunities: Vec<OpportunityFactor>,
        rationalization: Rationalization,
    ) -> Self {
        Self {
            pressure,
            opportunities,
            rationalization,
        }
    }

    /// Returns a risk score based on the fraud triangle components.
    pub fn risk_score(&self) -> f64 {
        let pressure_score = self.pressure.risk_weight();
        let opportunity_score: f64 = self
            .opportunities
            .iter()
            .map(|o| o.risk_weight())
            .sum::<f64>()
            / self.opportunities.len().max(1) as f64;
        let rationalization_score = self.rationalization.risk_weight();

        (pressure_score + opportunity_score + rationalization_score) / 3.0
    }
}

/// Types of pressure/incentive that can lead to fraud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PressureType {
    // Financial Pressures
    /// Personal financial difficulties (debt, lifestyle beyond means).
    PersonalFinancialDifficulties,
    /// Pressure to meet financial targets/earnings expectations.
    FinancialTargets,
    /// Market or analyst expectations.
    MarketExpectations,
    /// Debt covenant compliance requirements.
    CovenantCompliance,
    /// Credit rating maintenance.
    CreditRatingMaintenance,
    /// Acquisition/merger valuation pressure.
    AcquisitionValuation,

    // Non-Financial Pressures
    /// Fear of job loss.
    JobSecurity,
    /// Pressure to maintain status or image.
    StatusMaintenance,
    /// Gambling addiction.
    GamblingAddiction,
    /// Substance abuse issues.
    SubstanceAbuse,
    /// Family pressure or obligations.
    FamilyPressure,
    /// Greed or desire for more.
    Greed,
}

impl PressureType {
    /// Returns the risk weight (0.0-1.0) for this pressure type.
    pub fn risk_weight(&self) -> f64 {
        match self {
            PressureType::PersonalFinancialDifficulties => 0.80,
            PressureType::FinancialTargets => 0.75,
            PressureType::MarketExpectations => 0.70,
            PressureType::CovenantCompliance => 0.85,
            PressureType::CreditRatingMaintenance => 0.70,
            PressureType::AcquisitionValuation => 0.75,
            PressureType::JobSecurity => 0.65,
            PressureType::StatusMaintenance => 0.55,
            PressureType::GamblingAddiction => 0.90,
            PressureType::SubstanceAbuse => 0.85,
            PressureType::FamilyPressure => 0.60,
            PressureType::Greed => 0.70,
        }
    }
}

/// Opportunity factors that enable fraud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OpportunityFactor {
    /// Weak internal controls.
    WeakInternalControls,
    /// Lack of segregation of duties.
    LackOfSegregation,
    /// Override capability.
    ManagementOverride,
    /// Complex or unusual transactions.
    ComplexTransactions,
    /// Related party transactions.
    RelatedPartyTransactions,
    /// Poor tone at the top.
    PoorToneAtTop,
    /// Inadequate supervision.
    InadequateSupervision,
    /// Access to assets without accountability.
    AssetAccess,
    /// Inadequate record keeping.
    PoorRecordKeeping,
    /// Failure to discipline fraud perpetrators.
    LackOfDiscipline,
    /// Lack of independent checks.
    LackOfIndependentChecks,
}

impl OpportunityFactor {
    /// Returns the risk weight (0.0-1.0) for this opportunity factor.
    pub fn risk_weight(&self) -> f64 {
        match self {
            OpportunityFactor::WeakInternalControls => 0.85,
            OpportunityFactor::LackOfSegregation => 0.80,
            OpportunityFactor::ManagementOverride => 0.90,
            OpportunityFactor::ComplexTransactions => 0.70,
            OpportunityFactor::RelatedPartyTransactions => 0.75,
            OpportunityFactor::PoorToneAtTop => 0.85,
            OpportunityFactor::InadequateSupervision => 0.75,
            OpportunityFactor::AssetAccess => 0.70,
            OpportunityFactor::PoorRecordKeeping => 0.65,
            OpportunityFactor::LackOfDiscipline => 0.60,
            OpportunityFactor::LackOfIndependentChecks => 0.75,
        }
    }
}

/// Rationalizations used by fraud perpetrators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rationalization {
    /// "I'm just borrowing; I'll pay it back."
    TemporaryBorrowing,
    /// "Everyone does it."
    EveryoneDoesIt,
    /// "It's for the good of the company."
    ForTheCompanyGood,
    /// "I deserve this; the company owes me."
    Entitlement,
    /// "I was just following orders."
    FollowingOrders,
    /// "They won't miss it; they have plenty."
    TheyWontMissIt,
    /// "I need it more than they do."
    NeedItMore,
    /// "It's not really stealing."
    NotReallyStealing,
    /// "I'm underpaid for what I do."
    Underpaid,
    /// "It's a victimless crime."
    VictimlessCrime,
}

impl Rationalization {
    /// Returns the risk weight (0.0-1.0) for this rationalization.
    pub fn risk_weight(&self) -> f64 {
        match self {
            // More dangerous rationalizations
            Rationalization::Entitlement => 0.85,
            Rationalization::EveryoneDoesIt => 0.80,
            Rationalization::NotReallyStealing => 0.80,
            Rationalization::TheyWontMissIt => 0.75,
            // Medium risk
            Rationalization::Underpaid => 0.70,
            Rationalization::ForTheCompanyGood => 0.65,
            Rationalization::NeedItMore => 0.65,
            // Lower risk (still indicates fraud)
            Rationalization::TemporaryBorrowing => 0.60,
            Rationalization::FollowingOrders => 0.55,
            Rationalization::VictimlessCrime => 0.60,
        }
    }
}

// ============================================================================
// NEAR-MISS TYPES
// ============================================================================

/// Type of near-miss pattern (suspicious but legitimate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NearMissPattern {
    /// Transaction very similar to another (possible duplicate but legitimate).
    NearDuplicate {
        /// Date difference from similar transaction.
        date_difference_days: u32,
        /// Original transaction ID.
        similar_transaction_id: String,
    },
    /// Amount just below approval threshold (but legitimate).
    ThresholdProximity {
        /// The threshold being approached.
        threshold: Decimal,
        /// Percentage of threshold (0.0-1.0).
        proximity: f64,
    },
    /// Unusual but legitimate business pattern.
    UnusualLegitimate {
        /// Type of legitimate pattern.
        pattern_type: LegitimatePatternType,
        /// Business justification.
        justification: String,
    },
    /// Error that was caught and corrected.
    CorrectedError {
        /// Days until correction.
        correction_lag_days: u32,
        /// Correction document ID.
        correction_document_id: String,
    },
}

/// Types of unusual but legitimate business patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LegitimatePatternType {
    /// Year-end bonus payment.
    YearEndBonus,
    /// Contract prepayment.
    ContractPrepayment,
    /// Settlement payment.
    SettlementPayment,
    /// Insurance claim.
    InsuranceClaim,
    /// One-time vendor payment.
    OneTimePayment,
    /// Asset disposal.
    AssetDisposal,
    /// Seasonal inventory buildup.
    SeasonalInventory,
    /// Promotional spending.
    PromotionalSpending,
}

impl LegitimatePatternType {
    /// Returns a description of this pattern type.
    pub fn description(&self) -> &'static str {
        match self {
            LegitimatePatternType::YearEndBonus => "Year-end bonus payment",
            LegitimatePatternType::ContractPrepayment => "Contract prepayment per terms",
            LegitimatePatternType::SettlementPayment => "Legal settlement payment",
            LegitimatePatternType::InsuranceClaim => "Insurance claim reimbursement",
            LegitimatePatternType::OneTimePayment => "One-time vendor payment",
            LegitimatePatternType::AssetDisposal => "Fixed asset disposal",
            LegitimatePatternType::SeasonalInventory => "Seasonal inventory buildup",
            LegitimatePatternType::PromotionalSpending => "Promotional campaign spending",
        }
    }
}

/// What might trigger a false positive for this near-miss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FalsePositiveTrigger {
    /// Amount is near threshold.
    AmountNearThreshold,
    /// Timing is unusual.
    UnusualTiming,
    /// Similar to existing transaction.
    SimilarTransaction,
    /// New counterparty.
    NewCounterparty,
    /// Account combination unusual.
    UnusualAccountCombination,
    /// Volume spike.
    VolumeSpike,
    /// Round amount.
    RoundAmount,
}

/// Label for a near-miss case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearMissLabel {
    /// Document ID.
    pub document_id: String,
    /// The near-miss pattern.
    pub pattern: NearMissPattern,
    /// How suspicious it appears (0.0-1.0).
    pub suspicion_score: f64,
    /// What would trigger a false positive.
    pub false_positive_trigger: FalsePositiveTrigger,
    /// Why this is actually legitimate.
    pub explanation: String,
}

impl NearMissLabel {
    /// Creates a new near-miss label.
    pub fn new(
        document_id: impl Into<String>,
        pattern: NearMissPattern,
        suspicion_score: f64,
        trigger: FalsePositiveTrigger,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            pattern,
            suspicion_score: suspicion_score.clamp(0.0, 1.0),
            false_positive_trigger: trigger,
            explanation: explanation.into(),
        }
    }
}

/// Configuration for anomaly rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRateConfig {
    /// Overall anomaly rate (0.0 - 1.0).
    pub total_rate: f64,
    /// Fraud rate as proportion of anomalies.
    pub fraud_rate: f64,
    /// Error rate as proportion of anomalies.
    pub error_rate: f64,
    /// Process issue rate as proportion of anomalies.
    pub process_issue_rate: f64,
    /// Statistical anomaly rate as proportion of anomalies.
    pub statistical_rate: f64,
    /// Relational anomaly rate as proportion of anomalies.
    pub relational_rate: f64,
}

impl Default for AnomalyRateConfig {
    fn default() -> Self {
        Self {
            total_rate: 0.02,         // 2% of transactions are anomalous
            fraud_rate: 0.25,         // 25% of anomalies are fraud
            error_rate: 0.35,         // 35% of anomalies are errors
            process_issue_rate: 0.20, // 20% are process issues
            statistical_rate: 0.15,   // 15% are statistical
            relational_rate: 0.05,    // 5% are relational
        }
    }
}

impl AnomalyRateConfig {
    /// Validates that rates sum to approximately 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.fraud_rate
            + self.error_rate
            + self.process_issue_rate
            + self.statistical_rate
            + self.relational_rate;

        if (sum - 1.0).abs() > 0.01 {
            return Err(format!(
                "Anomaly category rates must sum to 1.0, got {}",
                sum
            ));
        }

        if self.total_rate < 0.0 || self.total_rate > 1.0 {
            return Err(format!(
                "Total rate must be between 0.0 and 1.0, got {}",
                self.total_rate
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_anomaly_type_category() {
        let fraud = AnomalyType::Fraud(FraudType::SelfApproval);
        assert_eq!(fraud.category(), "Fraud");
        assert!(fraud.is_intentional());

        let error = AnomalyType::Error(ErrorType::DuplicateEntry);
        assert_eq!(error.category(), "Error");
        assert!(!error.is_intentional());
    }

    #[test]
    fn test_labeled_anomaly() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_description("User approved their own expense report")
        .with_related_entity("USER001");

        assert_eq!(anomaly.severity, 3);
        assert!(anomaly.is_injected);
        assert_eq!(anomaly.related_entities.len(), 1);
    }

    #[test]
    fn test_labeled_anomaly_with_provenance() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_run_id("run-123")
        .with_generation_seed(42)
        .with_causal_reason(AnomalyCausalReason::RandomRate { base_rate: 0.02 })
        .with_structured_strategy(InjectionStrategy::SelfApproval {
            user_id: "USER001".to_string(),
        })
        .with_scenario("scenario-001")
        .with_original_document_hash("abc123");

        assert_eq!(anomaly.run_id, Some("run-123".to_string()));
        assert_eq!(anomaly.generation_seed, Some(42));
        assert!(anomaly.causal_reason.is_some());
        assert!(anomaly.structured_strategy.is_some());
        assert_eq!(anomaly.scenario_id, Some("scenario-001".to_string()));
        assert_eq!(anomaly.original_document_hash, Some("abc123".to_string()));

        // Check that legacy injection_strategy is also set
        assert_eq!(anomaly.injection_strategy, Some("SelfApproval".to_string()));
    }

    #[test]
    fn test_labeled_anomaly_derivation_chain() {
        let parent = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::DuplicatePayment),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let child = LabeledAnomaly::new(
            "ANO002".to_string(),
            AnomalyType::Error(ErrorType::DuplicateEntry),
            "JE002".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_parent_anomaly(&parent.anomaly_id);

        assert_eq!(child.parent_anomaly_id, Some("ANO001".to_string()));
    }

    #[test]
    fn test_injection_strategy_description() {
        let strategy = InjectionStrategy::AmountManipulation {
            original: dec!(1000),
            factor: 2.5,
        };
        assert_eq!(strategy.description(), "Amount multiplied by 2.50");
        assert_eq!(strategy.strategy_type(), "AmountManipulation");

        let strategy = InjectionStrategy::ThresholdAvoidance {
            threshold: dec!(10000),
            adjusted_amount: dec!(9999),
        };
        assert_eq!(
            strategy.description(),
            "Amount adjusted to avoid 10000 threshold"
        );

        let strategy = InjectionStrategy::DateShift {
            days_shifted: -5,
            original_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        };
        assert_eq!(strategy.description(), "Date backdated by 5 days");

        let strategy = InjectionStrategy::DateShift {
            days_shifted: 3,
            original_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        };
        assert_eq!(strategy.description(), "Date forward-dated by 3 days");
    }

    #[test]
    fn test_causal_reason_variants() {
        let reason = AnomalyCausalReason::RandomRate { base_rate: 0.02 };
        if let AnomalyCausalReason::RandomRate { base_rate } = reason {
            assert!((base_rate - 0.02).abs() < 0.001);
        }

        let reason = AnomalyCausalReason::TemporalPattern {
            pattern_name: "year_end_spike".to_string(),
        };
        if let AnomalyCausalReason::TemporalPattern { pattern_name } = reason {
            assert_eq!(pattern_name, "year_end_spike");
        }

        let reason = AnomalyCausalReason::ScenarioStep {
            scenario_type: "kickback".to_string(),
            step_number: 3,
        };
        if let AnomalyCausalReason::ScenarioStep {
            scenario_type,
            step_number,
        } = reason
        {
            assert_eq!(scenario_type, "kickback");
            assert_eq!(step_number, 3);
        }
    }

    #[test]
    fn test_feature_vector_length() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let features = anomaly.to_features();
        assert_eq!(features.len(), LabeledAnomaly::feature_count());
        assert_eq!(features.len(), LabeledAnomaly::feature_names().len());
    }

    #[test]
    fn test_feature_vector_with_provenance() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_scenario("scenario-001")
        .with_parent_anomaly("ANO000");

        let features = anomaly.to_features();

        // Last two features should be 1.0 (has scenario, has parent)
        assert_eq!(features[features.len() - 2], 1.0); // is_scenario_part
        assert_eq!(features[features.len() - 1], 1.0); // is_derived
    }

    #[test]
    fn test_anomaly_summary() {
        let anomalies = vec![
            LabeledAnomaly::new(
                "ANO001".to_string(),
                AnomalyType::Fraud(FraudType::SelfApproval),
                "JE001".to_string(),
                "JE".to_string(),
                "1000".to_string(),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            ),
            LabeledAnomaly::new(
                "ANO002".to_string(),
                AnomalyType::Error(ErrorType::DuplicateEntry),
                "JE002".to_string(),
                "JE".to_string(),
                "1000".to_string(),
                NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            ),
        ];

        let summary = AnomalySummary::from_anomalies(&anomalies);

        assert_eq!(summary.total_count, 2);
        assert_eq!(summary.by_category.get("Fraud"), Some(&1));
        assert_eq!(summary.by_category.get("Error"), Some(&1));
    }

    #[test]
    fn test_rate_config_validation() {
        let config = AnomalyRateConfig::default();
        assert!(config.validate().is_ok());

        let bad_config = AnomalyRateConfig {
            fraud_rate: 0.5,
            error_rate: 0.5,
            process_issue_rate: 0.5, // Sum > 1.0
            ..Default::default()
        };
        assert!(bad_config.validate().is_err());
    }

    #[test]
    fn test_injection_strategy_serialization() {
        let strategy = InjectionStrategy::SoDViolation {
            duty1: "CreatePO".to_string(),
            duty2: "ApprovePO".to_string(),
            violating_user: "USER001".to_string(),
        };

        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: InjectionStrategy = serde_json::from_str(&json).unwrap();

        assert_eq!(strategy, deserialized);
    }

    #[test]
    fn test_labeled_anomaly_serialization_with_provenance() {
        let anomaly = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_run_id("run-123")
        .with_generation_seed(42)
        .with_causal_reason(AnomalyCausalReason::RandomRate { base_rate: 0.02 });

        let json = serde_json::to_string(&anomaly).unwrap();
        let deserialized: LabeledAnomaly = serde_json::from_str(&json).unwrap();

        assert_eq!(anomaly.run_id, deserialized.run_id);
        assert_eq!(anomaly.generation_seed, deserialized.generation_seed);
    }

    // ========================================
    // FR-003 ENHANCED TAXONOMY TESTS
    // ========================================

    #[test]
    fn test_anomaly_category_from_anomaly_type() {
        // Fraud mappings
        let fraud_vendor = AnomalyType::Fraud(FraudType::FictitiousVendor);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&fraud_vendor),
            AnomalyCategory::FictitiousVendor
        );

        let fraud_kickback = AnomalyType::Fraud(FraudType::KickbackScheme);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&fraud_kickback),
            AnomalyCategory::VendorKickback
        );

        let fraud_structured = AnomalyType::Fraud(FraudType::SplitTransaction);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&fraud_structured),
            AnomalyCategory::StructuredTransaction
        );

        // Error mappings
        let error_duplicate = AnomalyType::Error(ErrorType::DuplicateEntry);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&error_duplicate),
            AnomalyCategory::DuplicatePayment
        );

        // Process issue mappings
        let process_skip = AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&process_skip),
            AnomalyCategory::MissingApproval
        );

        // Relational mappings
        let relational_circular =
            AnomalyType::Relational(RelationalAnomalyType::CircularTransaction);
        assert_eq!(
            AnomalyCategory::from_anomaly_type(&relational_circular),
            AnomalyCategory::CircularFlow
        );
    }

    #[test]
    fn test_anomaly_category_ordinal() {
        assert_eq!(AnomalyCategory::FictitiousVendor.ordinal(), 0);
        assert_eq!(AnomalyCategory::VendorKickback.ordinal(), 1);
        assert_eq!(AnomalyCategory::Custom("test".to_string()).ordinal(), 14);
    }

    #[test]
    fn test_contributing_factor() {
        let factor = ContributingFactor::new(
            FactorType::AmountDeviation,
            15000.0,
            10000.0,
            true,
            0.5,
            "Amount exceeds threshold",
        );

        assert_eq!(factor.factor_type, FactorType::AmountDeviation);
        assert_eq!(factor.value, 15000.0);
        assert_eq!(factor.threshold, 10000.0);
        assert!(factor.direction_greater);

        // Contribution: (15000 - 10000) / 10000 * 0.5 = 0.25
        let contribution = factor.contribution();
        assert!((contribution - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_contributing_factor_with_evidence() {
        let mut data = HashMap::new();
        data.insert("expected".to_string(), "10000".to_string());
        data.insert("actual".to_string(), "15000".to_string());

        let factor = ContributingFactor::new(
            FactorType::AmountDeviation,
            15000.0,
            10000.0,
            true,
            0.5,
            "Amount deviation detected",
        )
        .with_evidence("transaction_history", data);

        assert!(factor.evidence.is_some());
        let evidence = factor.evidence.unwrap();
        assert_eq!(evidence.source, "transaction_history");
        assert_eq!(evidence.data.get("expected"), Some(&"10000".to_string()));
    }

    #[test]
    fn test_enhanced_anomaly_label() {
        let base = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::DuplicatePayment),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let enhanced = EnhancedAnomalyLabel::from_base(base)
            .with_confidence(0.85)
            .with_severity(0.7)
            .with_factor(ContributingFactor::new(
                FactorType::DuplicateIndicator,
                1.0,
                0.5,
                true,
                0.4,
                "Duplicate payment detected",
            ))
            .with_secondary_category(AnomalyCategory::StructuredTransaction);

        assert_eq!(enhanced.category, AnomalyCategory::DuplicatePayment);
        assert_eq!(enhanced.enhanced_confidence, 0.85);
        assert_eq!(enhanced.enhanced_severity, 0.7);
        assert_eq!(enhanced.contributing_factors.len(), 1);
        assert_eq!(enhanced.secondary_categories.len(), 1);
    }

    #[test]
    fn test_enhanced_anomaly_label_features() {
        let base = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let enhanced = EnhancedAnomalyLabel::from_base(base)
            .with_confidence(0.9)
            .with_severity(0.8)
            .with_factor(ContributingFactor::new(
                FactorType::ControlBypass,
                1.0,
                0.0,
                true,
                0.5,
                "Control bypass detected",
            ));

        let features = enhanced.to_features();

        // Should have 25 features (15 base + 10 enhanced)
        assert_eq!(features.len(), EnhancedAnomalyLabel::feature_count());
        assert_eq!(features.len(), 25);

        // Check enhanced confidence is in features
        assert_eq!(features[15], 0.9); // enhanced_confidence

        // Check has_control_bypass flag
        assert_eq!(features[21], 1.0); // has_control_bypass
    }

    #[test]
    fn test_enhanced_anomaly_label_feature_names() {
        let names = EnhancedAnomalyLabel::feature_names();
        assert_eq!(names.len(), 25);
        assert!(names.contains(&"enhanced_confidence"));
        assert!(names.contains(&"enhanced_severity"));
        assert!(names.contains(&"has_control_bypass"));
    }

    #[test]
    fn test_factor_type_names() {
        assert_eq!(FactorType::AmountDeviation.name(), "amount_deviation");
        assert_eq!(FactorType::ThresholdProximity.name(), "threshold_proximity");
        assert_eq!(FactorType::ControlBypass.name(), "control_bypass");
    }

    #[test]
    fn test_anomaly_category_serialization() {
        let category = AnomalyCategory::CircularFlow;
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: AnomalyCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, deserialized);

        let custom = AnomalyCategory::Custom("custom_type".to_string());
        let json = serde_json::to_string(&custom).unwrap();
        let deserialized: AnomalyCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(custom, deserialized);
    }

    #[test]
    fn test_enhanced_label_secondary_category_dedup() {
        let base = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::DuplicatePayment),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let enhanced = EnhancedAnomalyLabel::from_base(base)
            // Try to add the primary category as secondary (should be ignored)
            .with_secondary_category(AnomalyCategory::DuplicatePayment)
            // Add a valid secondary
            .with_secondary_category(AnomalyCategory::TimingAnomaly)
            // Try to add duplicate secondary (should be ignored)
            .with_secondary_category(AnomalyCategory::TimingAnomaly);

        // Should only have 1 secondary category (TimingAnomaly)
        assert_eq!(enhanced.secondary_categories.len(), 1);
        assert_eq!(
            enhanced.secondary_categories[0],
            AnomalyCategory::TimingAnomaly
        );
    }

    // ==========================================================================
    // Accounting Standards Fraud Type Tests
    // ==========================================================================

    #[test]
    fn test_revenue_recognition_fraud_types() {
        // Test ASC 606/IFRS 15 related fraud types
        let fraud_types = [
            FraudType::ImproperRevenueRecognition,
            FraudType::ImproperPoAllocation,
            FraudType::VariableConsiderationManipulation,
            FraudType::ContractModificationMisstatement,
        ];

        for fraud_type in fraud_types {
            let anomaly_type = AnomalyType::Fraud(fraud_type);
            assert_eq!(anomaly_type.category(), "Fraud");
            assert!(anomaly_type.is_intentional());
            assert!(anomaly_type.severity() >= 3);
        }
    }

    #[test]
    fn test_lease_accounting_fraud_types() {
        // Test ASC 842/IFRS 16 related fraud types
        let fraud_types = [
            FraudType::LeaseClassificationManipulation,
            FraudType::OffBalanceSheetLease,
            FraudType::LeaseLiabilityUnderstatement,
            FraudType::RouAssetMisstatement,
        ];

        for fraud_type in fraud_types {
            let anomaly_type = AnomalyType::Fraud(fraud_type);
            assert_eq!(anomaly_type.category(), "Fraud");
            assert!(anomaly_type.is_intentional());
            assert!(anomaly_type.severity() >= 3);
        }

        // Off-balance sheet lease fraud should be high severity
        assert_eq!(FraudType::OffBalanceSheetLease.severity(), 5);
    }

    #[test]
    fn test_fair_value_fraud_types() {
        // Test ASC 820/IFRS 13 related fraud types
        let fraud_types = [
            FraudType::FairValueHierarchyManipulation,
            FraudType::Level3InputManipulation,
            FraudType::ValuationTechniqueManipulation,
        ];

        for fraud_type in fraud_types {
            let anomaly_type = AnomalyType::Fraud(fraud_type);
            assert_eq!(anomaly_type.category(), "Fraud");
            assert!(anomaly_type.is_intentional());
            assert!(anomaly_type.severity() >= 4);
        }

        // Level 3 manipulation is highest severity (unobservable inputs)
        assert_eq!(FraudType::Level3InputManipulation.severity(), 5);
    }

    #[test]
    fn test_impairment_fraud_types() {
        // Test ASC 360/IAS 36 related fraud types
        let fraud_types = [
            FraudType::DelayedImpairment,
            FraudType::ImpairmentTestAvoidance,
            FraudType::CashFlowProjectionManipulation,
            FraudType::ImproperImpairmentReversal,
        ];

        for fraud_type in fraud_types {
            let anomaly_type = AnomalyType::Fraud(fraud_type);
            assert_eq!(anomaly_type.category(), "Fraud");
            assert!(anomaly_type.is_intentional());
            assert!(anomaly_type.severity() >= 3);
        }

        // Cash flow manipulation has highest severity
        assert_eq!(FraudType::CashFlowProjectionManipulation.severity(), 5);
    }

    // ==========================================================================
    // Accounting Standards Error Type Tests
    // ==========================================================================

    #[test]
    fn test_standards_error_types() {
        // Test non-fraudulent accounting standards errors
        let error_types = [
            ErrorType::RevenueTimingError,
            ErrorType::PoAllocationError,
            ErrorType::LeaseClassificationError,
            ErrorType::LeaseCalculationError,
            ErrorType::FairValueError,
            ErrorType::ImpairmentCalculationError,
            ErrorType::DiscountRateError,
            ErrorType::FrameworkApplicationError,
        ];

        for error_type in error_types {
            let anomaly_type = AnomalyType::Error(error_type);
            assert_eq!(anomaly_type.category(), "Error");
            assert!(!anomaly_type.is_intentional());
            assert!(anomaly_type.severity() >= 3);
        }
    }

    #[test]
    fn test_framework_application_error() {
        // Test IFRS vs GAAP confusion errors
        let error_type = ErrorType::FrameworkApplicationError;
        assert_eq!(error_type.severity(), 4);

        let anomaly = LabeledAnomaly::new(
            "ERR001".to_string(),
            AnomalyType::Error(error_type),
            "JE100".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
        )
        .with_description("LIFO inventory method used under IFRS (not permitted)")
        .with_metadata("framework", "IFRS")
        .with_metadata("standard_violated", "IAS 2");

        assert_eq!(anomaly.anomaly_type.category(), "Error");
        assert_eq!(
            anomaly.metadata.get("standard_violated"),
            Some(&"IAS 2".to_string())
        );
    }

    #[test]
    fn test_standards_anomaly_serialization() {
        // Test that new fraud types serialize/deserialize correctly
        let fraud_types = [
            FraudType::ImproperRevenueRecognition,
            FraudType::LeaseClassificationManipulation,
            FraudType::FairValueHierarchyManipulation,
            FraudType::DelayedImpairment,
        ];

        for fraud_type in fraud_types {
            let json = serde_json::to_string(&fraud_type).expect("Failed to serialize");
            let deserialized: FraudType =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(fraud_type, deserialized);
        }

        // Test error types
        let error_types = [
            ErrorType::RevenueTimingError,
            ErrorType::LeaseCalculationError,
            ErrorType::FairValueError,
            ErrorType::FrameworkApplicationError,
        ];

        for error_type in error_types {
            let json = serde_json::to_string(&error_type).expect("Failed to serialize");
            let deserialized: ErrorType =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(error_type, deserialized);
        }
    }

    #[test]
    fn test_standards_labeled_anomaly() {
        // Test creating a labeled anomaly for a standards violation
        let anomaly = LabeledAnomaly::new(
            "STD001".to_string(),
            AnomalyType::Fraud(FraudType::ImproperRevenueRecognition),
            "CONTRACT-2024-001".to_string(),
            "Revenue".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        )
        .with_description("Revenue recognized before performance obligation satisfied (ASC 606)")
        .with_monetary_impact(dec!(500000))
        .with_metadata("standard", "ASC 606")
        .with_metadata("paragraph", "606-10-25-1")
        .with_metadata("contract_id", "C-2024-001")
        .with_related_entity("CONTRACT-2024-001")
        .with_related_entity("CUSTOMER-500");

        assert_eq!(anomaly.severity, 5); // ImproperRevenueRecognition has severity 5
        assert!(anomaly.is_injected);
        assert_eq!(anomaly.monetary_impact, Some(dec!(500000)));
        assert_eq!(anomaly.related_entities.len(), 2);
        assert_eq!(
            anomaly.metadata.get("standard"),
            Some(&"ASC 606".to_string())
        );
    }

    // ==========================================================================
    // Multi-Dimensional Labeling Tests
    // ==========================================================================

    #[test]
    fn test_severity_level() {
        assert_eq!(SeverityLevel::Low.numeric(), 1);
        assert_eq!(SeverityLevel::Critical.numeric(), 4);

        assert_eq!(SeverityLevel::from_numeric(1), SeverityLevel::Low);
        assert_eq!(SeverityLevel::from_numeric(4), SeverityLevel::Critical);

        assert_eq!(SeverityLevel::from_score(0.1), SeverityLevel::Low);
        assert_eq!(SeverityLevel::from_score(0.9), SeverityLevel::Critical);

        assert!((SeverityLevel::Medium.to_score() - 0.375).abs() < 0.01);
    }

    #[test]
    fn test_anomaly_severity() {
        let severity =
            AnomalySeverity::new(SeverityLevel::High, dec!(50000)).with_materiality(dec!(10000));

        assert_eq!(severity.level, SeverityLevel::High);
        assert!(severity.is_material);
        assert_eq!(severity.materiality_threshold, Some(dec!(10000)));

        // Not material
        let low_severity =
            AnomalySeverity::new(SeverityLevel::Low, dec!(5000)).with_materiality(dec!(10000));
        assert!(!low_severity.is_material);
    }

    #[test]
    fn test_detection_difficulty() {
        assert!(
            (AnomalyDetectionDifficulty::Trivial.expected_detection_rate() - 0.99).abs() < 0.01
        );
        assert!((AnomalyDetectionDifficulty::Expert.expected_detection_rate() - 0.15).abs() < 0.01);

        assert_eq!(
            AnomalyDetectionDifficulty::from_score(0.05),
            AnomalyDetectionDifficulty::Trivial
        );
        assert_eq!(
            AnomalyDetectionDifficulty::from_score(0.90),
            AnomalyDetectionDifficulty::Expert
        );

        assert_eq!(AnomalyDetectionDifficulty::Moderate.name(), "moderate");
    }

    #[test]
    fn test_ground_truth_certainty() {
        assert_eq!(GroundTruthCertainty::Definite.certainty_score(), 1.0);
        assert_eq!(GroundTruthCertainty::Probable.certainty_score(), 0.8);
        assert_eq!(GroundTruthCertainty::Possible.certainty_score(), 0.5);
    }

    #[test]
    fn test_detection_method() {
        assert_eq!(DetectionMethod::RuleBased.name(), "rule_based");
        assert_eq!(DetectionMethod::MachineLearning.name(), "machine_learning");
    }

    #[test]
    fn test_extended_anomaly_label() {
        let base = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::FictitiousVendor),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        )
        .with_monetary_impact(dec!(100000));

        let extended = ExtendedAnomalyLabel::from_base(base)
            .with_severity(AnomalySeverity::new(SeverityLevel::Critical, dec!(100000)))
            .with_difficulty(AnomalyDetectionDifficulty::Hard)
            .with_method(DetectionMethod::GraphBased)
            .with_method(DetectionMethod::ForensicAudit)
            .with_indicator("New vendor with no history")
            .with_indicator("Large first transaction")
            .with_certainty(GroundTruthCertainty::Definite)
            .with_entity("V001")
            .with_secondary_category(AnomalyCategory::BehavioralAnomaly)
            .with_scheme("SCHEME001", 2);

        assert_eq!(extended.severity.level, SeverityLevel::Critical);
        assert_eq!(
            extended.detection_difficulty,
            AnomalyDetectionDifficulty::Hard
        );
        // from_base adds RuleBased, then we add 2 more (GraphBased, ForensicAudit)
        assert_eq!(extended.recommended_methods.len(), 3);
        assert_eq!(extended.key_indicators.len(), 2);
        assert_eq!(extended.scheme_id, Some("SCHEME001".to_string()));
        assert_eq!(extended.scheme_stage, Some(2));
    }

    #[test]
    fn test_extended_anomaly_label_features() {
        let base = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        );

        let extended =
            ExtendedAnomalyLabel::from_base(base).with_difficulty(AnomalyDetectionDifficulty::Hard);

        let features = extended.to_features();
        assert_eq!(features.len(), ExtendedAnomalyLabel::feature_count());
        assert_eq!(features.len(), 30);

        // Check difficulty score is in features
        let difficulty_idx = 18; // Position of difficulty_score
        assert!((features[difficulty_idx] - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_extended_label_near_miss() {
        let base = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Statistical(StatisticalAnomalyType::UnusuallyHighAmount),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        );

        let extended = ExtendedAnomalyLabel::from_base(base)
            .as_near_miss("Year-end bonus payment, legitimately high");

        assert!(extended.is_near_miss);
        assert!(extended.near_miss_explanation.is_some());
    }

    #[test]
    fn test_scheme_type() {
        assert_eq!(
            SchemeType::GradualEmbezzlement.name(),
            "gradual_embezzlement"
        );
        assert_eq!(SchemeType::GradualEmbezzlement.typical_stages(), 4);
        assert_eq!(SchemeType::VendorKickback.typical_stages(), 4);
    }

    #[test]
    fn test_concealment_technique() {
        assert!(ConcealmentTechnique::Collusion.difficulty_bonus() > 0.0);
        assert!(
            ConcealmentTechnique::Collusion.difficulty_bonus()
                > ConcealmentTechnique::TimingExploitation.difficulty_bonus()
        );
    }

    #[test]
    fn test_near_miss_label() {
        let near_miss = NearMissLabel::new(
            "JE001",
            NearMissPattern::ThresholdProximity {
                threshold: dec!(10000),
                proximity: 0.95,
            },
            0.7,
            FalsePositiveTrigger::AmountNearThreshold,
            "Transaction is 95% of threshold but business justified",
        );

        assert_eq!(near_miss.document_id, "JE001");
        assert_eq!(near_miss.suspicion_score, 0.7);
        assert_eq!(
            near_miss.false_positive_trigger,
            FalsePositiveTrigger::AmountNearThreshold
        );
    }

    #[test]
    fn test_legitimate_pattern_type() {
        assert_eq!(
            LegitimatePatternType::YearEndBonus.description(),
            "Year-end bonus payment"
        );
        assert_eq!(
            LegitimatePatternType::InsuranceClaim.description(),
            "Insurance claim reimbursement"
        );
    }

    #[test]
    fn test_severity_detection_difficulty_serialization() {
        let severity = AnomalySeverity::new(SeverityLevel::High, dec!(50000));
        let json = serde_json::to_string(&severity).expect("Failed to serialize");
        let deserialized: AnomalySeverity =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(severity.level, deserialized.level);

        let difficulty = AnomalyDetectionDifficulty::Hard;
        let json = serde_json::to_string(&difficulty).expect("Failed to serialize");
        let deserialized: AnomalyDetectionDifficulty =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(difficulty, deserialized);
    }

    // ========================================
    // ACFE Taxonomy Tests
    // ========================================

    #[test]
    fn test_acfe_fraud_category() {
        let asset = AcfeFraudCategory::AssetMisappropriation;
        assert_eq!(asset.name(), "asset_misappropriation");
        assert!((asset.typical_occurrence_rate() - 0.86).abs() < 0.01);
        assert_eq!(asset.typical_median_loss(), Decimal::new(100_000, 0));
        assert_eq!(asset.typical_detection_months(), 12);

        let corruption = AcfeFraudCategory::Corruption;
        assert_eq!(corruption.name(), "corruption");
        assert!((corruption.typical_occurrence_rate() - 0.33).abs() < 0.01);

        let fs_fraud = AcfeFraudCategory::FinancialStatementFraud;
        assert_eq!(fs_fraud.typical_median_loss(), Decimal::new(954_000, 0));
        assert_eq!(fs_fraud.typical_detection_months(), 24);
    }

    #[test]
    fn test_cash_fraud_scheme() {
        let shell = CashFraudScheme::ShellCompany;
        assert_eq!(shell.category(), AcfeFraudCategory::AssetMisappropriation);
        assert_eq!(shell.subcategory(), "billing_schemes");
        assert_eq!(shell.severity(), 5);
        assert_eq!(
            shell.detection_difficulty(),
            AnomalyDetectionDifficulty::Hard
        );

        let ghost = CashFraudScheme::GhostEmployee;
        assert_eq!(ghost.subcategory(), "payroll_schemes");
        assert_eq!(ghost.severity(), 5);

        // Test all variants exist
        assert_eq!(CashFraudScheme::all_variants().len(), 20);
    }

    #[test]
    fn test_asset_fraud_scheme() {
        let ip_theft = AssetFraudScheme::IntellectualPropertyTheft;
        assert_eq!(
            ip_theft.category(),
            AcfeFraudCategory::AssetMisappropriation
        );
        assert_eq!(ip_theft.subcategory(), "other_assets");
        assert_eq!(ip_theft.severity(), 5);

        let inv_theft = AssetFraudScheme::InventoryTheft;
        assert_eq!(inv_theft.subcategory(), "inventory");
        assert_eq!(inv_theft.severity(), 4);
    }

    #[test]
    fn test_corruption_scheme() {
        let kickback = CorruptionScheme::InvoiceKickback;
        assert_eq!(kickback.category(), AcfeFraudCategory::Corruption);
        assert_eq!(kickback.subcategory(), "bribery");
        assert_eq!(kickback.severity(), 5);
        assert_eq!(
            kickback.detection_difficulty(),
            AnomalyDetectionDifficulty::Expert
        );

        let bid_rigging = CorruptionScheme::BidRigging;
        assert_eq!(bid_rigging.subcategory(), "bribery");
        assert_eq!(
            bid_rigging.detection_difficulty(),
            AnomalyDetectionDifficulty::Hard
        );

        let purchasing = CorruptionScheme::PurchasingConflict;
        assert_eq!(purchasing.subcategory(), "conflicts_of_interest");

        // Test all variants exist
        assert_eq!(CorruptionScheme::all_variants().len(), 10);
    }

    #[test]
    fn test_financial_statement_scheme() {
        let fictitious = FinancialStatementScheme::FictitiousRevenues;
        assert_eq!(
            fictitious.category(),
            AcfeFraudCategory::FinancialStatementFraud
        );
        assert_eq!(fictitious.subcategory(), "overstatement");
        assert_eq!(fictitious.severity(), 5);
        assert_eq!(
            fictitious.detection_difficulty(),
            AnomalyDetectionDifficulty::Expert
        );

        let understated = FinancialStatementScheme::UnderstatedRevenues;
        assert_eq!(understated.subcategory(), "understatement");

        // Test all variants exist
        assert_eq!(FinancialStatementScheme::all_variants().len(), 13);
    }

    #[test]
    fn test_acfe_scheme_unified() {
        let cash_scheme = AcfeScheme::Cash(CashFraudScheme::ShellCompany);
        assert_eq!(
            cash_scheme.category(),
            AcfeFraudCategory::AssetMisappropriation
        );
        assert_eq!(cash_scheme.severity(), 5);

        let corruption_scheme = AcfeScheme::Corruption(CorruptionScheme::BidRigging);
        assert_eq!(corruption_scheme.category(), AcfeFraudCategory::Corruption);

        let fs_scheme = AcfeScheme::FinancialStatement(FinancialStatementScheme::PrematureRevenue);
        assert_eq!(
            fs_scheme.category(),
            AcfeFraudCategory::FinancialStatementFraud
        );
    }

    #[test]
    fn test_acfe_detection_method() {
        let tip = AcfeDetectionMethod::Tip;
        assert!((tip.typical_detection_rate() - 0.42).abs() < 0.01);

        let internal_audit = AcfeDetectionMethod::InternalAudit;
        assert!((internal_audit.typical_detection_rate() - 0.16).abs() < 0.01);

        let external_audit = AcfeDetectionMethod::ExternalAudit;
        assert!((external_audit.typical_detection_rate() - 0.04).abs() < 0.01);

        // Test all variants exist
        assert_eq!(AcfeDetectionMethod::all_variants().len(), 12);
    }

    #[test]
    fn test_perpetrator_department() {
        let accounting = PerpetratorDepartment::Accounting;
        assert!((accounting.typical_occurrence_rate() - 0.21).abs() < 0.01);
        assert_eq!(accounting.typical_median_loss(), Decimal::new(130_000, 0));

        let executive = PerpetratorDepartment::Executive;
        assert_eq!(executive.typical_median_loss(), Decimal::new(600_000, 0));
    }

    #[test]
    fn test_perpetrator_level() {
        let employee = PerpetratorLevel::Employee;
        assert!((employee.typical_occurrence_rate() - 0.42).abs() < 0.01);
        assert_eq!(employee.typical_median_loss(), Decimal::new(50_000, 0));

        let exec = PerpetratorLevel::OwnerExecutive;
        assert_eq!(exec.typical_median_loss(), Decimal::new(337_000, 0));
    }

    #[test]
    fn test_acfe_calibration() {
        let cal = AcfeCalibration::default();
        assert_eq!(cal.median_loss, Decimal::new(117_000, 0));
        assert_eq!(cal.median_duration_months, 12);
        assert!((cal.collusion_rate - 0.50).abs() < 0.01);
        assert!(cal.validate().is_ok());

        // Test custom calibration
        let custom_cal = AcfeCalibration::new(Decimal::new(200_000, 0), 18);
        assert_eq!(custom_cal.median_loss, Decimal::new(200_000, 0));
        assert_eq!(custom_cal.median_duration_months, 18);

        // Test validation failure
        let bad_cal = AcfeCalibration {
            collusion_rate: 1.5,
            ..Default::default()
        };
        assert!(bad_cal.validate().is_err());
    }

    #[test]
    fn test_fraud_triangle() {
        let triangle = FraudTriangle::new(
            PressureType::FinancialTargets,
            vec![
                OpportunityFactor::WeakInternalControls,
                OpportunityFactor::ManagementOverride,
            ],
            Rationalization::ForTheCompanyGood,
        );

        // Risk score should be between 0 and 1
        let risk = triangle.risk_score();
        assert!((0.0..=1.0).contains(&risk));
        // Should be relatively high given the components
        assert!(risk > 0.5);
    }

    #[test]
    fn test_pressure_types() {
        let financial = PressureType::FinancialTargets;
        assert!(financial.risk_weight() > 0.5);

        let gambling = PressureType::GamblingAddiction;
        assert_eq!(gambling.risk_weight(), 0.90);
    }

    #[test]
    fn test_opportunity_factors() {
        let override_factor = OpportunityFactor::ManagementOverride;
        assert_eq!(override_factor.risk_weight(), 0.90);

        let weak_controls = OpportunityFactor::WeakInternalControls;
        assert!(weak_controls.risk_weight() > 0.8);
    }

    #[test]
    fn test_rationalizations() {
        let entitlement = Rationalization::Entitlement;
        assert!(entitlement.risk_weight() > 0.8);

        let borrowing = Rationalization::TemporaryBorrowing;
        assert!(borrowing.risk_weight() < entitlement.risk_weight());
    }

    #[test]
    fn test_acfe_scheme_serialization() {
        let scheme = AcfeScheme::Corruption(CorruptionScheme::BidRigging);
        let json = serde_json::to_string(&scheme).expect("Failed to serialize");
        let deserialized: AcfeScheme = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(scheme, deserialized);

        let calibration = AcfeCalibration::default();
        let json = serde_json::to_string(&calibration).expect("Failed to serialize");
        let deserialized: AcfeCalibration =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(calibration.median_loss, deserialized.median_loss);
    }
}
