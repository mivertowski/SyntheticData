//! Significant Classes of Transactions (SCOTS) per ISA 315 (Revised 2019).
//!
//! ISA 315.26 requires the auditor to identify and understand the significant
//! classes of transactions (SCOTs), account balances and disclosures.  SCOTs
//! drive the design of the auditor's information-technology and internal-control
//! understanding and the nature, timing, and extent of further audit procedures.
//!
//! Each SCOT is characterised by:
//! - Its business process (O2C, P2P, R2R, H2R)
//! - Transaction type (routine, non-routine, estimation)
//! - Processing method (fully automated, semi-automated, manual)
//! - A critical path of four stages (Initiation → Recording → Processing → Reporting)
//! - Relevant financial statement assertions (from the CRA model)
//! - For estimation SCOTs: an estimation complexity rating per ISA 540
//!
//! References:
//! - ISA 315 (Revised 2019) §26 — Significant classes of transactions
//! - ISA 330 §6 — Further audit procedures in response to SCOT assessment
//! - ISA 540 — Accounting estimates (drives estimation complexity)

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enumerations
// ---------------------------------------------------------------------------

/// Significance level of a class of transactions for the audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScotSignificance {
    /// Material individually or in aggregate; requires the most extensive procedures.
    High,
    /// Significant but not individually material; moderate extent of procedures.
    Medium,
    /// Below materiality; limited procedures or analytical only.
    Low,
}

impl std::fmt::Display for ScotSignificance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        };
        write!(f, "{s}")
    }
}

/// Type of transaction within a SCOT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScotTransactionType {
    /// High-volume, standardised, recurring transactions with consistent processing.
    Routine,
    /// Infrequent or unusual transactions requiring significant judgment or
    /// management approval (e.g. asset disposals, significant contracts).
    NonRoutine,
    /// Accounting estimates with inherent measurement uncertainty (ISA 540).
    Estimation,
}

impl std::fmt::Display for ScotTransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Routine => "Routine",
            Self::NonRoutine => "Non-Routine",
            Self::Estimation => "Estimation",
        };
        write!(f, "{s}")
    }
}

/// How transactions within the SCOT are processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingMethod {
    /// System-initiated and system-posted — no manual intervention in normal processing.
    FullyAutomated,
    /// System-initiated but requires manual approval or manual journal entry
    /// for certain steps (e.g. three-way match exception handling).
    SemiAutomated,
    /// Primarily manual processing — spreadsheet-based, clerk-prepared, etc.
    Manual,
}

impl std::fmt::Display for ProcessingMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::FullyAutomated => "Fully Automated",
            Self::SemiAutomated => "Semi-Automated",
            Self::Manual => "Manual",
        };
        write!(f, "{s}")
    }
}

/// Complexity of the underlying estimate per ISA 540.
///
/// Only populated for `ScotTransactionType::Estimation` SCOTs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimationComplexity {
    /// Straightforward estimation with observable inputs (e.g. straight-line depreciation).
    Simple,
    /// Moderate complexity — some unobservable inputs or model uncertainty
    /// (e.g. ECL provisioning with internal historical data).
    Moderate,
    /// Highly complex — significant unobservable inputs, multiple methodologies possible,
    /// or high sensitivity to assumptions (e.g. pension obligations, level-3 fair value).
    Complex,
}

impl std::fmt::Display for EstimationComplexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Simple => "Simple",
            Self::Moderate => "Moderate",
            Self::Complex => "Complex",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Critical path
// ---------------------------------------------------------------------------

/// A single stage in the SCOT's transaction processing critical path.
///
/// The standard four stages are:
/// 1. Initiation — how transactions are initiated (automated trigger or manual request)
/// 2. Recording — how the transaction is recorded in source documents / systems
/// 3. Processing — system processing, matching, posting to the GL
/// 4. Reporting — how the transaction flows into the financial statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPathStage {
    /// Stage name (e.g. "Initiation", "Recording", "Processing", "Reporting").
    pub stage_name: String,
    /// Brief description of how this stage operates for this SCOT.
    pub description: String,
    /// Whether this stage is fully automated (system-driven, no manual input).
    pub is_automated: bool,
    /// ID of the key internal control operating at this stage, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_control_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Main SCOT struct
// ---------------------------------------------------------------------------

/// A Significant Class of Transactions (SCOT) per ISA 315.
///
/// One SCOT is generated per major business process / transaction class.
/// SCOTs drive the scope of the auditor's control and substantive testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificantClassOfTransactions {
    /// Unique identifier for this SCOT (deterministic slug).
    pub id: String,
    /// Entity / company code.
    pub entity_code: String,
    /// Descriptive name (e.g. "Revenue — Product Sales", "Purchases — Raw Materials").
    pub scot_name: String,
    /// Business process code driving this class (O2C, P2P, R2R, H2R, etc.).
    pub business_process: String,
    /// Significance of this SCOT for the audit.
    pub significance_level: ScotSignificance,
    /// Whether the transactions are routine, non-routine, or estimation-based.
    pub transaction_type: ScotTransactionType,
    /// Primary processing method for this class of transactions.
    pub processing_method: ProcessingMethod,
    /// Approximate number of transactions in the period.
    pub volume: usize,
    /// Aggregate monetary value of transactions in the period.
    #[serde(with = "rust_decimal::serde::str")]
    pub monetary_value: Decimal,
    /// The four-stage critical path (Initiation → Recording → Processing → Reporting).
    pub critical_path: Vec<CriticalPathStage>,
    /// Financial statement assertions relevant to this SCOT (links to CRA assertions).
    pub relevant_assertions: Vec<String>,
    /// GL account areas affected by this SCOT.
    pub related_account_areas: Vec<String>,
    /// Estimation complexity — only set for `ScotTransactionType::Estimation` SCOTs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimation_complexity: Option<EstimationComplexity>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn scot_display_impls() {
        assert_eq!(ScotSignificance::High.to_string(), "High");
        assert_eq!(ScotTransactionType::Estimation.to_string(), "Estimation");
        assert_eq!(ProcessingMethod::FullyAutomated.to_string(), "Fully Automated");
        assert_eq!(EstimationComplexity::Complex.to_string(), "Complex");
    }

    #[test]
    fn scot_structure() {
        let scot = SignificantClassOfTransactions {
            id: "SCOT-C001-REVENUE_PRODUCT_SALES".into(),
            entity_code: "C001".into(),
            scot_name: "Revenue — Product Sales".into(),
            business_process: "O2C".into(),
            significance_level: ScotSignificance::High,
            transaction_type: ScotTransactionType::Routine,
            processing_method: ProcessingMethod::SemiAutomated,
            volume: 5_000,
            monetary_value: dec!(10_000_000),
            critical_path: vec![
                CriticalPathStage {
                    stage_name: "Initiation".into(),
                    description: "Sales order created by customer / sales team".into(),
                    is_automated: false,
                    key_control_id: Some("C001".into()),
                },
                CriticalPathStage {
                    stage_name: "Recording".into(),
                    description: "System records SO upon credit approval".into(),
                    is_automated: true,
                    key_control_id: None,
                },
            ],
            relevant_assertions: vec!["Occurrence".into(), "Accuracy".into()],
            related_account_areas: vec!["Revenue".into(), "Trade Receivables".into()],
            estimation_complexity: None,
        };

        assert_eq!(scot.critical_path.len(), 2);
        assert!(scot.estimation_complexity.is_none());
        assert_eq!(scot.significance_level, ScotSignificance::High);
    }

    #[test]
    fn estimation_scot_has_complexity() {
        let scot = SignificantClassOfTransactions {
            id: "SCOT-C001-ECL_BAD_DEBT".into(),
            entity_code: "C001".into(),
            scot_name: "ECL / Bad Debt Provision".into(),
            business_process: "R2R".into(),
            significance_level: ScotSignificance::High,
            transaction_type: ScotTransactionType::Estimation,
            processing_method: ProcessingMethod::Manual,
            volume: 12,
            monetary_value: dec!(250_000),
            critical_path: Vec::new(),
            relevant_assertions: vec!["ValuationAndAllocation".into()],
            related_account_areas: vec!["Trade Receivables".into(), "Provisions".into()],
            estimation_complexity: Some(EstimationComplexity::Moderate),
        };

        assert!(scot.estimation_complexity.is_some());
        assert_eq!(
            scot.estimation_complexity.unwrap(),
            EstimationComplexity::Moderate
        );
    }
}
