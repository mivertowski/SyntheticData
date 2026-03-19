//! Engagement letter models per ISA 210.
//!
//! ISA 210 requires the auditor to agree the terms of the audit engagement with
//! management or those charged with governance. The engagement letter documents
//! the scope, responsibilities, and fee arrangement for the engagement.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An engagement letter issued under ISA 210.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementLetter {
    /// Unique engagement letter ID
    pub id: String,
    /// Reference to the parent engagement
    pub engagement_id: String,
    /// Addressee (management / those charged with governance)
    pub addressee: String,
    /// Date the letter was issued
    pub date: NaiveDate,
    /// Scope of the engagement
    pub scope: EngagementScope,
    /// Responsibilities of the auditor
    pub responsibilities_auditor: Vec<String>,
    /// Responsibilities of management
    pub responsibilities_management: Vec<String>,
    /// Fee arrangement
    pub fee_arrangement: FeeArrangement,
    /// Expected reporting deadline
    pub reporting_deadline: NaiveDate,
    /// Applicable accounting framework (e.g., "IFRS", "US GAAP")
    pub applicable_framework: String,
    /// Any special terms or conditions
    pub special_terms: Vec<String>,
}

/// Scope classification of an audit engagement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EngagementScope {
    /// Full statutory audit of the financial statements of a single entity
    #[default]
    StatutoryAudit,
    /// Group audit spanning multiple entities (ISA 600)
    GroupAudit,
    /// Limited assurance engagement
    LimitedAssurance,
    /// Agreed-upon procedures engagement
    AgreedUponProcedures,
}

/// Fee arrangement for the engagement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeArrangement {
    /// Basis on which fees are charged (e.g., "Fixed", "Time and materials")
    pub basis: String,
    /// Agreed fee amount
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    /// Currency code (ISO 4217)
    pub currency: String,
}

impl EngagementLetter {
    /// Create a new engagement letter.
    pub fn new(
        engagement_id: impl Into<String>,
        addressee: impl Into<String>,
        date: NaiveDate,
        scope: EngagementScope,
        fee_arrangement: FeeArrangement,
        reporting_deadline: NaiveDate,
        applicable_framework: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            engagement_id: engagement_id.into(),
            addressee: addressee.into(),
            date,
            scope,
            responsibilities_auditor: Vec::new(),
            responsibilities_management: Vec::new(),
            fee_arrangement,
            reporting_deadline,
            applicable_framework: applicable_framework.into(),
            special_terms: Vec::new(),
        }
    }
}

impl FeeArrangement {
    /// Create a new fee arrangement.
    pub fn new(basis: impl Into<String>, amount: Decimal, currency: impl Into<String>) -> Self {
        Self {
            basis: basis.into(),
            amount,
            currency: currency.into(),
        }
    }
}
