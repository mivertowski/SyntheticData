//! External Confirmations (ISA 505).
//!
//! Implements external confirmation procedures for obtaining audit evidence:
//! - Bank confirmations
//! - Accounts receivable confirmations
//! - Accounts payable confirmations
//! - Legal confirmations

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// External confirmation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalConfirmation {
    /// Unique confirmation identifier.
    pub confirmation_id: Uuid,

    /// Engagement ID.
    pub engagement_id: Uuid,

    /// Type of confirmation.
    pub confirmation_type: ConfirmationType,

    /// Form of confirmation.
    pub confirmation_form: ConfirmationForm,

    /// Name of confirming party.
    pub confirmee_name: String,

    /// Address of confirming party.
    pub confirmee_address: String,

    /// Contact information.
    pub confirmee_contact: String,

    /// Account or item being confirmed.
    pub item_description: String,

    /// Amount per client records.
    #[serde(with = "rust_decimal::serde::str")]
    pub client_amount: Decimal,

    /// Currency.
    pub currency: String,

    /// Date sent.
    pub date_sent: NaiveDate,

    /// Follow-up date.
    pub follow_up_date: Option<NaiveDate>,

    /// Response status.
    pub response_status: ConfirmationResponseStatus,

    /// Response details (if received).
    pub response: Option<ConfirmationResponse>,

    /// Reconciliation of differences.
    pub reconciliation: Option<ConfirmationReconciliation>,

    /// Alternative procedures (if no response).
    pub alternative_procedures: Option<AlternativeProcedures>,

    /// Conclusion.
    pub conclusion: ConfirmationConclusion,

    /// Workpaper reference.
    pub workpaper_reference: Option<String>,

    /// Prepared by.
    pub prepared_by: String,

    /// Reviewed by.
    pub reviewed_by: Option<String>,
}

impl ExternalConfirmation {
    /// Create a new external confirmation.
    pub fn new(
        engagement_id: Uuid,
        confirmation_type: ConfirmationType,
        confirmee_name: impl Into<String>,
        item_description: impl Into<String>,
        client_amount: Decimal,
        currency: impl Into<String>,
    ) -> Self {
        Self {
            confirmation_id: Uuid::now_v7(),
            engagement_id,
            confirmation_type,
            confirmation_form: ConfirmationForm::Positive,
            confirmee_name: confirmee_name.into(),
            confirmee_address: String::new(),
            confirmee_contact: String::new(),
            item_description: item_description.into(),
            client_amount,
            currency: currency.into(),
            date_sent: chrono::Utc::now().date_naive(),
            follow_up_date: None,
            response_status: ConfirmationResponseStatus::Pending,
            response: None,
            reconciliation: None,
            alternative_procedures: None,
            conclusion: ConfirmationConclusion::NotCompleted,
            workpaper_reference: None,
            prepared_by: String::new(),
            reviewed_by: None,
        }
    }

    /// Check if confirmation is complete.
    pub fn is_complete(&self) -> bool {
        !matches!(self.conclusion, ConfirmationConclusion::NotCompleted)
    }

    /// Check if alternative procedures are needed.
    pub fn needs_alternative_procedures(&self) -> bool {
        matches!(
            self.response_status,
            ConfirmationResponseStatus::NoResponse | ConfirmationResponseStatus::Returned
        )
    }

    /// Calculate difference between client and confirmed amounts.
    pub fn difference(&self) -> Option<Decimal> {
        self.response
            .as_ref()
            .map(|r| self.client_amount - r.confirmed_amount)
    }
}

/// Type of confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationType {
    /// Bank account confirmation.
    Bank,
    /// Trade receivable confirmation.
    AccountsReceivable,
    /// Trade payable confirmation.
    AccountsPayable,
    /// Loan/debt confirmation.
    Loan,
    /// Legal confirmation (lawyers' letters).
    Legal,
    /// Investment confirmation.
    Investment,
    /// Insurance confirmation.
    Insurance,
    /// Related party confirmation.
    RelatedParty,
    /// Other confirmation.
    Other,
}

impl std::fmt::Display for ConfirmationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bank => write!(f, "Bank Confirmation"),
            Self::AccountsReceivable => write!(f, "AR Confirmation"),
            Self::AccountsPayable => write!(f, "AP Confirmation"),
            Self::Loan => write!(f, "Loan Confirmation"),
            Self::Legal => write!(f, "Legal Confirmation"),
            Self::Investment => write!(f, "Investment Confirmation"),
            Self::Insurance => write!(f, "Insurance Confirmation"),
            Self::RelatedParty => write!(f, "Related Party Confirmation"),
            Self::Other => write!(f, "Other Confirmation"),
        }
    }
}

/// Form of confirmation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationForm {
    /// Positive confirmation - requests response in all cases.
    #[default]
    Positive,
    /// Negative confirmation - requests response only if disagrees.
    Negative,
    /// Blank confirmation - confirmee fills in the amount.
    Blank,
}

impl std::fmt::Display for ConfirmationForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Positive => write!(f, "Positive"),
            Self::Negative => write!(f, "Negative"),
            Self::Blank => write!(f, "Blank"),
        }
    }
}

/// Confirmation response status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationResponseStatus {
    /// Confirmation not yet sent.
    #[default]
    NotSent,
    /// Sent, awaiting response.
    Pending,
    /// Response received - agrees.
    ReceivedAgrees,
    /// Response received - disagrees.
    ReceivedDisagrees,
    /// Response received - partial information.
    ReceivedPartial,
    /// No response after follow-up.
    NoResponse,
    /// Returned undeliverable.
    Returned,
    /// Response received (for blank confirmations).
    ReceivedBlank,
}

impl std::fmt::Display for ConfirmationResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSent => write!(f, "Not Sent"),
            Self::Pending => write!(f, "Pending"),
            Self::ReceivedAgrees => write!(f, "Received - Agrees"),
            Self::ReceivedDisagrees => write!(f, "Received - Disagrees"),
            Self::ReceivedPartial => write!(f, "Received - Partial"),
            Self::NoResponse => write!(f, "No Response"),
            Self::Returned => write!(f, "Returned"),
            Self::ReceivedBlank => write!(f, "Received - Blank"),
        }
    }
}

/// Confirmation response details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationResponse {
    /// Date response received.
    pub date_received: NaiveDate,

    /// Confirmed amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub confirmed_amount: Decimal,

    /// Response agrees with client records.
    pub agrees: bool,

    /// Comments from confirmee.
    pub comments: String,

    /// Differences noted by confirmee.
    pub differences_noted: Vec<ConfirmedDifference>,

    /// Respondent name/title.
    pub respondent_name: String,

    /// Whether response appears authentic.
    pub appears_authentic: bool,

    /// Reliability assessment.
    pub reliability_assessment: ResponseReliability,
}

impl ConfirmationResponse {
    /// Create a new confirmation response.
    pub fn new(date_received: NaiveDate, confirmed_amount: Decimal, agrees: bool) -> Self {
        Self {
            date_received,
            confirmed_amount,
            agrees,
            comments: String::new(),
            differences_noted: Vec::new(),
            respondent_name: String::new(),
            appears_authentic: true,
            reliability_assessment: ResponseReliability::Reliable,
        }
    }
}

/// Reliability assessment of confirmation response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseReliability {
    /// Response appears reliable.
    #[default]
    Reliable,
    /// Some concerns about reliability.
    QuestionableReliability,
    /// Response is unreliable.
    Unreliable,
}

/// Difference noted in confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmedDifference {
    /// Description of difference.
    pub description: String,

    /// Amount of difference.
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,

    /// Type of difference.
    pub difference_type: DifferenceType,
}

/// Type of confirmation difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceType {
    /// Timing difference (e.g., payment in transit).
    Timing,
    /// Actual error in client records.
    Error,
    /// Disputed amount.
    Dispute,
    /// Cutoff difference.
    Cutoff,
    /// Classification difference.
    Classification,
    /// Unknown/unexplained.
    Unknown,
}

/// Reconciliation of confirmation differences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationReconciliation {
    /// Client balance.
    #[serde(with = "rust_decimal::serde::str")]
    pub client_balance: Decimal,

    /// Confirmed balance.
    #[serde(with = "rust_decimal::serde::str")]
    pub confirmed_balance: Decimal,

    /// Total difference.
    #[serde(with = "rust_decimal::serde::str")]
    pub total_difference: Decimal,

    /// Reconciling items.
    pub reconciling_items: Vec<ReconcilingItem>,

    /// Unreconciled difference.
    #[serde(with = "rust_decimal::serde::str")]
    pub unreconciled_difference: Decimal,

    /// Conclusion on reconciliation.
    pub conclusion: ReconciliationConclusion,
}

impl ConfirmationReconciliation {
    /// Create a new reconciliation.
    pub fn new(client_balance: Decimal, confirmed_balance: Decimal) -> Self {
        let total_difference = client_balance - confirmed_balance;
        Self {
            client_balance,
            confirmed_balance,
            total_difference,
            reconciling_items: Vec::new(),
            unreconciled_difference: total_difference,
            conclusion: ReconciliationConclusion::NotCompleted,
        }
    }

    /// Add a reconciling item and update unreconciled difference.
    pub fn add_reconciling_item(&mut self, item: ReconcilingItem) {
        self.unreconciled_difference -= item.amount;
        self.reconciling_items.push(item);
    }
}

/// Reconciling item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcilingItem {
    /// Description.
    pub description: String,

    /// Amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,

    /// Type of reconciling item.
    pub item_type: ReconcilingItemType,

    /// Supporting evidence obtained.
    pub evidence: String,
}

/// Type of reconciling item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcilingItemType {
    /// Cash/payment in transit.
    CashInTransit,
    /// Deposit in transit.
    DepositInTransit,
    /// Outstanding check.
    OutstandingCheck,
    /// Bank charges not recorded.
    BankCharges,
    /// Interest not recorded.
    InterestNotRecorded,
    /// Cutoff adjustment.
    CutoffAdjustment,
    /// Error correction.
    ErrorCorrection,
    /// Other reconciling item.
    Other,
}

/// Reconciliation conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationConclusion {
    /// Reconciliation not completed.
    #[default]
    NotCompleted,
    /// Fully reconciled, no issues.
    FullyReconciled,
    /// Reconciled with timing differences only.
    ReconciledTimingOnly,
    /// Potential misstatement identified.
    PotentialMisstatement,
    /// Misstatement identified.
    MisstatementIdentified,
}

/// Alternative procedures when confirmation not received.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeProcedures {
    /// Reason alternative procedures were needed.
    pub reason: AlternativeProcedureReason,

    /// Procedures performed.
    pub procedures: Vec<AlternativeProcedure>,

    /// Evidence obtained.
    pub evidence_obtained: Vec<String>,

    /// Conclusion.
    pub conclusion: AlternativeProcedureConclusion,
}

impl AlternativeProcedures {
    /// Create new alternative procedures.
    pub fn new(reason: AlternativeProcedureReason) -> Self {
        Self {
            reason,
            procedures: Vec::new(),
            evidence_obtained: Vec::new(),
            conclusion: AlternativeProcedureConclusion::NotCompleted,
        }
    }
}

/// Reason for alternative procedures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlternativeProcedureReason {
    /// No response received.
    NoResponse,
    /// Response unreliable.
    UnreliableResponse,
    /// Confirmation returned undeliverable.
    Undeliverable,
    /// Management refused to allow.
    ManagementRefused,
}

/// Alternative audit procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeProcedure {
    /// Procedure description.
    pub description: String,

    /// Type of procedure.
    pub procedure_type: AlternativeProcedureType,

    /// Result of procedure.
    pub result: String,
}

/// Type of alternative procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlternativeProcedureType {
    /// Examine subsequent cash receipts.
    SubsequentCashReceipts,
    /// Examine subsequent cash disbursements.
    SubsequentCashDisbursements,
    /// Examine shipping documents.
    ShippingDocuments,
    /// Examine receiving reports.
    ReceivingReports,
    /// Examine customer purchase orders.
    PurchaseOrders,
    /// Examine sales contracts.
    SalesContracts,
    /// Examine bank statements.
    BankStatements,
    /// Other procedure.
    Other,
}

/// Conclusion from alternative procedures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlternativeProcedureConclusion {
    /// Procedures not completed.
    #[default]
    NotCompleted,
    /// Sufficient evidence obtained.
    SufficientEvidence,
    /// Insufficient evidence obtained.
    InsufficientEvidence,
    /// Misstatement identified.
    MisstatementIdentified,
}

/// Confirmation conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationConclusion {
    /// Confirmation not completed.
    #[default]
    NotCompleted,
    /// Satisfactory response received, balance confirmed.
    Confirmed,
    /// Exception noted and resolved.
    ExceptionResolved,
    /// Exception noted, potential misstatement.
    PotentialMisstatement,
    /// Misstatement identified.
    MisstatementIdentified,
    /// Alternative procedures satisfactory.
    AlternativesSatisfactory,
    /// Unable to obtain sufficient evidence.
    InsufficientEvidence,
}

impl std::fmt::Display for ConfirmationConclusion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotCompleted => write!(f, "Not Completed"),
            Self::Confirmed => write!(f, "Confirmed"),
            Self::ExceptionResolved => write!(f, "Exception Resolved"),
            Self::PotentialMisstatement => write!(f, "Potential Misstatement"),
            Self::MisstatementIdentified => write!(f, "Misstatement Identified"),
            Self::AlternativesSatisfactory => write!(f, "Alternative Procedures Satisfactory"),
            Self::InsufficientEvidence => write!(f, "Insufficient Evidence"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_confirmation_creation() {
        let confirmation = ExternalConfirmation::new(
            Uuid::now_v7(),
            ConfirmationType::AccountsReceivable,
            "Customer Corp",
            "Trade receivable balance",
            dec!(50000),
            "USD",
        );

        assert_eq!(confirmation.confirmee_name, "Customer Corp");
        assert_eq!(confirmation.client_amount, dec!(50000));
        assert_eq!(
            confirmation.response_status,
            ConfirmationResponseStatus::Pending
        );
    }

    #[test]
    fn test_confirmation_difference() {
        let mut confirmation = ExternalConfirmation::new(
            Uuid::now_v7(),
            ConfirmationType::AccountsReceivable,
            "Customer Corp",
            "Trade receivable balance",
            dec!(50000),
            "USD",
        );

        confirmation.response = Some(ConfirmationResponse::new(
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            dec!(48000),
            false,
        ));

        assert_eq!(confirmation.difference(), Some(dec!(2000)));
    }

    #[test]
    fn test_reconciliation() {
        let mut recon = ConfirmationReconciliation::new(dec!(50000), dec!(48000));

        assert_eq!(recon.total_difference, dec!(2000));
        assert_eq!(recon.unreconciled_difference, dec!(2000));

        recon.add_reconciling_item(ReconcilingItem {
            description: "Payment in transit".to_string(),
            amount: dec!(2000),
            item_type: ReconcilingItemType::CashInTransit,
            evidence: "Examined subsequent receipt".to_string(),
        });

        assert_eq!(recon.unreconciled_difference, dec!(0));
    }

    #[test]
    fn test_alternative_procedures_needed() {
        let mut confirmation = ExternalConfirmation::new(
            Uuid::now_v7(),
            ConfirmationType::AccountsReceivable,
            "Customer Corp",
            "Trade receivable balance",
            dec!(50000),
            "USD",
        );

        confirmation.response_status = ConfirmationResponseStatus::NoResponse;
        assert!(confirmation.needs_alternative_procedures());

        confirmation.response_status = ConfirmationResponseStatus::ReceivedAgrees;
        assert!(!confirmation.needs_alternative_procedures());
    }
}
