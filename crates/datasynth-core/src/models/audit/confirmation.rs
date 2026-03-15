//! External confirmation models per ISA 505.
//!
//! External confirmations are audit evidence obtained as a direct written
//! response to the auditor from a third party (the confirming party), in paper
//! form or by electronic or other medium.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of external confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationType {
    /// Bank balance confirmation
    #[default]
    BankBalance,
    /// Accounts receivable confirmation
    AccountsReceivable,
    /// Accounts payable confirmation
    AccountsPayable,
    /// Investment confirmation
    Investment,
    /// Loan confirmation
    Loan,
    /// Legal letter confirmation
    Legal,
    /// Insurance confirmation
    Insurance,
    /// Inventory confirmation
    Inventory,
}

/// Form of external confirmation (ISA 505.A6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationForm {
    /// Positive confirmation — recipient asked to respond in all cases
    #[default]
    Positive,
    /// Negative confirmation — recipient asked to respond only if they disagree
    Negative,
    /// Blank confirmation — recipient asked to fill in the balance
    Blank,
}

/// Lifecycle status of an external confirmation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationStatus {
    /// Confirmation drafted but not yet sent
    #[default]
    Draft,
    /// Confirmation has been sent to the confirming party
    Sent,
    /// Response has been received
    Received,
    /// No response received by the deadline
    NoResponse,
    /// Alternative procedures performed in lieu of confirmation
    AlternativeProcedures,
    /// Confirmation process completed
    Completed,
}

/// Type of confirming party (recipient).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipientType {
    /// Financial institution / bank
    #[default]
    Bank,
    /// Customer (debtor)
    Customer,
    /// Supplier (creditor)
    Supplier,
    /// Legal counsel
    LegalCounsel,
    /// Insurance company
    Insurer,
    /// Other third party
    Other,
}

/// Type of response received from the confirming party.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    /// Balance confirmed without exception
    #[default]
    Confirmed,
    /// Balance confirmed but with one or more exceptions noted
    ConfirmedWithException,
    /// Confirming party denies the recorded balance
    Denied,
    /// No reply received
    NoReply,
}

/// External confirmation request per ISA 505.
///
/// Tracks the full lifecycle of a confirmation from draft through to completion,
/// including the balance under confirmation and key dates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalConfirmation {
    /// Unique confirmation ID
    pub confirmation_id: Uuid,
    /// Human-readable reference (e.g. "CONF-a1b2c3d4")
    pub confirmation_ref: String,
    /// Engagement this confirmation belongs to
    pub engagement_id: Uuid,
    /// Optional linked workpaper
    pub workpaper_id: Option<Uuid>,
    /// Type of balance or matter being confirmed
    pub confirmation_type: ConfirmationType,
    /// Name of the confirming party
    pub recipient_name: String,
    /// Category of confirming party
    pub recipient_type: RecipientType,
    /// Account or reference number at the confirming party
    pub account_id: Option<String>,
    /// Balance per the client's books
    pub book_balance: Decimal,
    /// Date the balance relates to
    pub confirmation_date: NaiveDate,
    /// Date the request was dispatched
    pub sent_date: Option<NaiveDate>,
    /// Deadline by which a response is required
    pub response_deadline: Option<NaiveDate>,
    /// Current lifecycle status
    pub status: ConfirmationStatus,
    /// Positive, negative, or blank form
    pub positive_negative: ConfirmationForm,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ExternalConfirmation {
    /// Create a new external confirmation request.
    pub fn new(
        engagement_id: Uuid,
        confirmation_type: ConfirmationType,
        recipient_name: &str,
        recipient_type: RecipientType,
        book_balance: Decimal,
        confirmation_date: NaiveDate,
    ) -> Self {
        let id = Uuid::new_v4();
        let now = Utc::now();
        Self {
            confirmation_id: id,
            confirmation_ref: format!("CONF-{}", &id.to_string()[..8]),
            engagement_id,
            workpaper_id: None,
            confirmation_type,
            recipient_name: recipient_name.into(),
            recipient_type,
            account_id: None,
            book_balance,
            confirmation_date,
            sent_date: None,
            response_deadline: None,
            status: ConfirmationStatus::Draft,
            positive_negative: ConfirmationForm::Positive,
            created_at: now,
            updated_at: now,
        }
    }

    /// Link to a workpaper.
    pub fn with_workpaper(mut self, workpaper_id: Uuid) -> Self {
        self.workpaper_id = Some(workpaper_id);
        self
    }

    /// Set the account or reference number at the confirming party.
    pub fn with_account(mut self, account_id: &str) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    /// Mark the confirmation as sent and record the dispatch date and deadline.
    pub fn send(&mut self, sent_date: NaiveDate, deadline: NaiveDate) {
        self.sent_date = Some(sent_date);
        self.response_deadline = Some(deadline);
        self.status = ConfirmationStatus::Sent;
        self.updated_at = Utc::now();
    }
}

/// Response received from a confirming party per ISA 505.
///
/// Records the details of what the confirming party stated, any exceptions,
/// and whether the auditor has reconciled differences to the book balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationResponse {
    /// Unique response ID
    pub response_id: Uuid,
    /// Human-readable reference (e.g. "RESP-a1b2c3d4")
    pub response_ref: String,
    /// The confirmation this response relates to
    pub confirmation_id: Uuid,
    /// Engagement this response belongs to
    pub engagement_id: Uuid,
    /// Date the response was received
    pub response_date: NaiveDate,
    /// Balance stated by the confirming party (None for blank forms not filled in)
    pub confirmed_balance: Option<Decimal>,
    /// Nature of the response
    pub response_type: ResponseType,
    /// Whether the confirming party noted any exceptions
    pub has_exception: bool,
    /// Monetary value of the noted exception, if any
    pub exception_amount: Option<Decimal>,
    /// Description of the exception
    pub exception_description: Option<String>,
    /// Whether differences have been reconciled
    pub reconciled: bool,
    /// Explanation of the reconciliation
    pub reconciliation_explanation: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConfirmationResponse {
    /// Create a new confirmation response.
    pub fn new(
        confirmation_id: Uuid,
        engagement_id: Uuid,
        response_date: NaiveDate,
        response_type: ResponseType,
    ) -> Self {
        let id = Uuid::new_v4();
        let now = Utc::now();
        Self {
            response_id: id,
            response_ref: format!("RESP-{}", &id.to_string()[..8]),
            confirmation_id,
            engagement_id,
            response_date,
            confirmed_balance: None,
            response_type,
            has_exception: false,
            exception_amount: None,
            exception_description: None,
            reconciled: false,
            reconciliation_explanation: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Record the balance confirmed by the third party.
    pub fn with_confirmed_balance(mut self, balance: Decimal) -> Self {
        self.confirmed_balance = Some(balance);
        self
    }

    /// Record an exception noted by the confirming party.
    pub fn with_exception(mut self, amount: Decimal, description: &str) -> Self {
        self.has_exception = true;
        self.exception_amount = Some(amount);
        self.exception_description = Some(description.into());
        self
    }

    /// Mark the response as reconciled and record the explanation.
    pub fn reconcile(&mut self, explanation: &str) {
        self.reconciled = true;
        self.reconciliation_explanation = Some(explanation.into());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_confirmation() -> ExternalConfirmation {
        ExternalConfirmation::new(
            Uuid::new_v4(),
            ConfirmationType::BankBalance,
            "First National Bank",
            RecipientType::Bank,
            dec!(125_000.00),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
    }

    fn sample_response(confirmation_id: Uuid, engagement_id: Uuid) -> ConfirmationResponse {
        ConfirmationResponse::new(
            confirmation_id,
            engagement_id,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            ResponseType::Confirmed,
        )
    }

    // --- ExternalConfirmation tests ---

    #[test]
    fn test_new_confirmation() {
        let conf = sample_confirmation();
        assert_eq!(conf.status, ConfirmationStatus::Draft);
        assert_eq!(conf.positive_negative, ConfirmationForm::Positive);
        assert!(conf.workpaper_id.is_none());
        assert!(conf.account_id.is_none());
        assert!(conf.sent_date.is_none());
        assert!(conf.response_deadline.is_none());
    }

    #[test]
    fn test_send_updates_status() {
        let mut conf = sample_confirmation();
        let sent = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        let deadline = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        conf.send(sent, deadline);
        assert_eq!(conf.status, ConfirmationStatus::Sent);
        assert_eq!(conf.sent_date, Some(sent));
        assert_eq!(conf.response_deadline, Some(deadline));
    }

    #[test]
    fn test_with_workpaper() {
        let wp_id = Uuid::new_v4();
        let conf = sample_confirmation().with_workpaper(wp_id);
        assert_eq!(conf.workpaper_id, Some(wp_id));
    }

    #[test]
    fn test_with_account() {
        let conf = sample_confirmation().with_account("ACC-001");
        assert_eq!(conf.account_id, Some("ACC-001".to_string()));
    }

    // --- ConfirmationResponse tests ---

    #[test]
    fn test_new_response() {
        let conf = sample_confirmation();
        let resp = sample_response(conf.confirmation_id, conf.engagement_id);
        assert!(!resp.has_exception);
        assert!(!resp.reconciled);
        assert!(resp.confirmed_balance.is_none());
        assert!(resp.exception_amount.is_none());
        assert!(resp.reconciliation_explanation.is_none());
    }

    #[test]
    fn test_with_confirmed_balance() {
        let conf = sample_confirmation();
        let resp = sample_response(conf.confirmation_id, conf.engagement_id)
            .with_confirmed_balance(dec!(125_000.00));
        assert_eq!(resp.confirmed_balance, Some(dec!(125_000.00)));
    }

    #[test]
    fn test_with_exception() {
        let conf = sample_confirmation();
        let resp = sample_response(conf.confirmation_id, conf.engagement_id)
            .with_confirmed_balance(dec!(123_500.00))
            .with_exception(dec!(1_500.00), "Unrecorded credit note dated 30 Dec 2025");
        assert!(resp.has_exception);
        assert_eq!(resp.exception_amount, Some(dec!(1_500.00)));
        assert!(resp.exception_description.is_some());
    }

    #[test]
    fn test_reconcile() {
        let conf = sample_confirmation();
        let mut resp = sample_response(conf.confirmation_id, conf.engagement_id)
            .with_exception(dec!(1_500.00), "Timing difference");
        assert!(!resp.reconciled);
        resp.reconcile("Credit note received and posted on 2 Jan 2026 — timing difference only.");
        assert!(resp.reconciled);
        assert!(resp.reconciliation_explanation.is_some());
    }

    // --- Serde tests ---

    #[test]
    fn test_confirmation_status_serde() {
        // CRITICAL: AlternativeProcedures must serialise as "alternative_procedures"
        let val = serde_json::to_value(ConfirmationStatus::AlternativeProcedures).unwrap();
        assert_eq!(val, serde_json::json!("alternative_procedures"));

        // Round-trip all variants
        for status in [
            ConfirmationStatus::Draft,
            ConfirmationStatus::Sent,
            ConfirmationStatus::Received,
            ConfirmationStatus::NoResponse,
            ConfirmationStatus::AlternativeProcedures,
            ConfirmationStatus::Completed,
        ] {
            let serialised = serde_json::to_string(&status).unwrap();
            let deserialised: ConfirmationStatus = serde_json::from_str(&serialised).unwrap();
            assert_eq!(status, deserialised);
        }
    }

    #[test]
    fn test_confirmation_type_serde() {
        for ct in [
            ConfirmationType::BankBalance,
            ConfirmationType::AccountsReceivable,
            ConfirmationType::AccountsPayable,
            ConfirmationType::Investment,
            ConfirmationType::Loan,
            ConfirmationType::Legal,
            ConfirmationType::Insurance,
            ConfirmationType::Inventory,
        ] {
            let serialised = serde_json::to_string(&ct).unwrap();
            let deserialised: ConfirmationType = serde_json::from_str(&serialised).unwrap();
            assert_eq!(ct, deserialised);
        }
    }

    #[test]
    fn test_response_type_serde() {
        for rt in [
            ResponseType::Confirmed,
            ResponseType::ConfirmedWithException,
            ResponseType::Denied,
            ResponseType::NoReply,
        ] {
            let serialised = serde_json::to_string(&rt).unwrap();
            let deserialised: ResponseType = serde_json::from_str(&serialised).unwrap();
            assert_eq!(rt, deserialised);
        }
    }
}
