//! Journal Entry data structures for General Ledger postings.
//!
//! This module defines the core journal entry structures that form the basis
//! of double-entry bookkeeping. Each journal entry consists of a header and
//! one or more line items that must balance (total debits = total credits).

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::anomaly::FraudType;
use super::approval::ApprovalWorkflow;

/// Source of a journal entry transaction.
///
/// Distinguishes between manual human entries and automated system postings,
/// which is critical for audit trail analysis and fraud detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransactionSource {
    /// Manual entry by human user during working hours
    #[default]
    Manual,
    /// Automated system posting (interfaces, batch jobs, EDI)
    Automated,
    /// Recurring scheduled posting (depreciation, amortization)
    Recurring,
    /// Reversal of a previous entry
    Reversal,
    /// Period-end adjustment entry
    Adjustment,
    /// Statistical posting (memo only, no financial impact)
    Statistical,
}

// Note: FraudType is defined in anomaly.rs and re-exported from mod.rs
// Use `crate::models::FraudType` for fraud type classification.

/// Business process that originated the transaction.
///
/// Aligns with standard enterprise process frameworks for process mining
/// and analytics integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum BusinessProcess {
    /// Order-to-Cash: sales, billing, accounts receivable
    O2C,
    /// Procure-to-Pay: purchasing, accounts payable
    P2P,
    /// Record-to-Report: GL, consolidation, reporting
    #[default]
    R2R,
    /// Hire-to-Retire: payroll, HR accounting
    H2R,
    /// Acquire-to-Retire: fixed assets, depreciation
    A2R,
    /// Source-to-Contract: sourcing, supplier qualification, RFx
    S2C,
    /// Manufacturing: production orders, quality, cycle counts
    #[serde(rename = "MFG")]
    Mfg,
    /// Banking operations: KYC/AML, accounts, transactions
    #[serde(rename = "BANK")]
    Bank,
    /// Audit engagement lifecycle
    #[serde(rename = "AUDIT")]
    Audit,
    /// Treasury operations
    Treasury,
    /// Tax accounting
    Tax,
    /// Intercompany transactions
    Intercompany,
    /// Project accounting lifecycle
    #[serde(rename = "PROJECT")]
    ProjectAccounting,
    /// ESG / Sustainability reporting
    #[serde(rename = "ESG")]
    Esg,
}

/// Document type classification for journal entries.
///
/// Standard SAP-compatible document type codes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentType {
    /// Two-character document type code (e.g., "SA", "KR", "DR")
    pub code: String,
    /// Human-readable description
    pub description: String,
    /// Associated business process
    pub business_process: BusinessProcess,
    /// Is this a reversal document type
    pub is_reversal: bool,
}

impl DocumentType {
    /// Standard GL account document
    pub fn gl_account() -> Self {
        Self {
            code: "SA".to_string(),
            description: "G/L Account Document".to_string(),
            business_process: BusinessProcess::R2R,
            is_reversal: false,
        }
    }

    /// Vendor invoice
    pub fn vendor_invoice() -> Self {
        Self {
            code: "KR".to_string(),
            description: "Vendor Invoice".to_string(),
            business_process: BusinessProcess::P2P,
            is_reversal: false,
        }
    }

    /// Customer invoice
    pub fn customer_invoice() -> Self {
        Self {
            code: "DR".to_string(),
            description: "Customer Invoice".to_string(),
            business_process: BusinessProcess::O2C,
            is_reversal: false,
        }
    }

    /// Vendor payment
    pub fn vendor_payment() -> Self {
        Self {
            code: "KZ".to_string(),
            description: "Vendor Payment".to_string(),
            business_process: BusinessProcess::P2P,
            is_reversal: false,
        }
    }

    /// Customer payment
    pub fn customer_payment() -> Self {
        Self {
            code: "DZ".to_string(),
            description: "Customer Payment".to_string(),
            business_process: BusinessProcess::O2C,
            is_reversal: false,
        }
    }

    /// Asset posting
    pub fn asset_posting() -> Self {
        Self {
            code: "AA".to_string(),
            description: "Asset Posting".to_string(),
            business_process: BusinessProcess::A2R,
            is_reversal: false,
        }
    }

    /// Payroll posting
    pub fn payroll() -> Self {
        Self {
            code: "PR".to_string(),
            description: "Payroll Document".to_string(),
            business_process: BusinessProcess::H2R,
            is_reversal: false,
        }
    }
}

/// Header information for a journal entry document.
///
/// Contains all metadata about the posting including timing, user, and
/// organizational assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryHeader {
    /// Unique identifier for this journal entry (UUID v7 for time-ordering)
    pub document_id: Uuid,

    /// Company code this entry belongs to
    pub company_code: String,

    /// Fiscal year (4-digit)
    pub fiscal_year: u16,

    /// Fiscal period (1-12, or 13-16 for special periods)
    pub fiscal_period: u8,

    /// Posting date (when the entry affects the books)
    pub posting_date: NaiveDate,

    /// Document date (date on source document)
    pub document_date: NaiveDate,

    /// Entry timestamp (when created in system)
    pub created_at: DateTime<Utc>,

    /// Document type code
    pub document_type: String,

    /// Transaction currency (ISO 4217)
    pub currency: String,

    /// Exchange rate to local currency (1.0 if same currency)
    pub exchange_rate: Decimal,

    /// Reference document number (external reference)
    pub reference: Option<String>,

    /// Header text/description
    pub header_text: Option<String>,

    /// User who created the entry
    pub created_by: String,

    /// User persona classification for behavioral analysis
    pub user_persona: String,

    /// Transaction source (manual vs automated)
    pub source: TransactionSource,

    /// Business process reference
    pub business_process: Option<BusinessProcess>,

    /// Ledger (0L = Leading Ledger)
    pub ledger: String,

    /// Is this entry part of a fraud scenario
    pub is_fraud: bool,

    /// Fraud type if applicable
    pub fraud_type: Option<FraudType>,

    // --- Anomaly Tracking Fields ---
    /// Whether this entry has an injected anomaly
    #[serde(default)]
    pub is_anomaly: bool,

    /// Unique anomaly identifier for label linkage
    #[serde(default)]
    pub anomaly_id: Option<String>,

    /// Type of anomaly if applicable (serialized enum name)
    #[serde(default)]
    pub anomaly_type: Option<String>,

    /// Simulation batch ID for traceability
    pub batch_id: Option<Uuid>,

    // --- Internal Controls / SOX Compliance Fields ---
    /// Internal control IDs that apply to this transaction
    #[serde(default)]
    pub control_ids: Vec<String>,

    /// Whether this is a SOX-relevant transaction
    #[serde(default)]
    pub sox_relevant: bool,

    /// Control status for this transaction
    #[serde(default)]
    pub control_status: super::internal_control::ControlStatus,

    /// Whether a Segregation of Duties violation occurred
    #[serde(default)]
    pub sod_violation: bool,

    /// Type of SoD conflict if violation occurred
    #[serde(default)]
    pub sod_conflict_type: Option<super::sod::SodConflictType>,

    // --- Approval Workflow ---
    /// Approval workflow for high-value transactions
    #[serde(default)]
    pub approval_workflow: Option<ApprovalWorkflow>,

    // --- OCPM (Object-Centric Process Mining) Traceability ---
    /// OCPM event IDs that triggered this journal entry
    #[serde(default)]
    pub ocpm_event_ids: Vec<Uuid>,

    /// OCPM object IDs involved in this journal entry
    #[serde(default)]
    pub ocpm_object_ids: Vec<Uuid>,

    /// OCPM case ID for process instance tracking
    #[serde(default)]
    pub ocpm_case_id: Option<Uuid>,
}

impl JournalEntryHeader {
    /// Create a new journal entry header with default values.
    pub fn new(company_code: String, posting_date: NaiveDate) -> Self {
        Self {
            document_id: Uuid::now_v7(),
            company_code,
            fiscal_year: posting_date.year() as u16,
            fiscal_period: posting_date.month() as u8,
            posting_date,
            document_date: posting_date,
            created_at: Utc::now(),
            document_type: "SA".to_string(),
            currency: "USD".to_string(),
            exchange_rate: Decimal::ONE,
            reference: None,
            header_text: None,
            created_by: "SYSTEM".to_string(),
            user_persona: "automated_system".to_string(),
            source: TransactionSource::Automated,
            business_process: Some(BusinessProcess::R2R),
            ledger: "0L".to_string(),
            is_fraud: false,
            fraud_type: None,
            // Anomaly tracking
            is_anomaly: false,
            anomaly_id: None,
            anomaly_type: None,
            batch_id: None,
            // Internal Controls / SOX fields
            control_ids: Vec::new(),
            sox_relevant: false,
            control_status: super::internal_control::ControlStatus::default(),
            sod_violation: false,
            sod_conflict_type: None,
            // Approval workflow
            approval_workflow: None,
            // OCPM traceability
            ocpm_event_ids: Vec::new(),
            ocpm_object_ids: Vec::new(),
            ocpm_case_id: None,
        }
    }

    /// Create a new journal entry header with a deterministic document ID.
    ///
    /// Used for reproducible generation where the document ID is derived
    /// from a seed and counter.
    pub fn with_deterministic_id(
        company_code: String,
        posting_date: NaiveDate,
        document_id: Uuid,
    ) -> Self {
        Self {
            document_id,
            company_code,
            fiscal_year: posting_date.year() as u16,
            fiscal_period: posting_date.month() as u8,
            posting_date,
            document_date: posting_date,
            created_at: Utc::now(),
            document_type: "SA".to_string(),
            currency: "USD".to_string(),
            exchange_rate: Decimal::ONE,
            reference: None,
            header_text: None,
            created_by: "SYSTEM".to_string(),
            user_persona: "automated_system".to_string(),
            source: TransactionSource::Automated,
            business_process: Some(BusinessProcess::R2R),
            ledger: "0L".to_string(),
            is_fraud: false,
            fraud_type: None,
            // Anomaly tracking
            is_anomaly: false,
            anomaly_id: None,
            anomaly_type: None,
            batch_id: None,
            // Internal Controls / SOX fields
            control_ids: Vec::new(),
            sox_relevant: false,
            control_status: super::internal_control::ControlStatus::default(),
            sod_violation: false,
            sod_conflict_type: None,
            // Approval workflow
            approval_workflow: None,
            // OCPM traceability
            ocpm_event_ids: Vec::new(),
            ocpm_object_ids: Vec::new(),
            ocpm_case_id: None,
        }
    }
}

use chrono::Datelike;

/// Individual line item within a journal entry.
///
/// Each line represents a debit or credit posting to a specific GL account.
/// Line items must be balanced within a journal entry (sum of debits = sum of credits).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryLine {
    /// Parent document ID (matches header)
    pub document_id: Uuid,

    /// Line item number within document (1-based)
    pub line_number: u32,

    /// GL account number
    pub gl_account: String,

    /// Account code (alias for gl_account for compatibility)
    #[serde(default)]
    pub account_code: String,

    /// Account description (for display)
    #[serde(default)]
    pub account_description: Option<String>,

    /// Debit amount in transaction currency (positive or zero)
    #[serde(with = "rust_decimal::serde::str")]
    pub debit_amount: Decimal,

    /// Credit amount in transaction currency (positive or zero)
    #[serde(with = "rust_decimal::serde::str")]
    pub credit_amount: Decimal,

    /// Amount in local/company currency
    #[serde(with = "rust_decimal::serde::str")]
    pub local_amount: Decimal,

    /// Amount in group currency (for consolidation)
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub group_amount: Option<Decimal>,

    /// Cost center assignment
    pub cost_center: Option<String>,

    /// Profit center assignment
    pub profit_center: Option<String>,

    /// Segment for segment reporting
    pub segment: Option<String>,

    /// Functional area
    pub functional_area: Option<String>,

    /// Line item text/description
    pub line_text: Option<String>,

    /// Text field (alias for line_text for compatibility)
    #[serde(default)]
    pub text: Option<String>,

    /// Reference field
    #[serde(default)]
    pub reference: Option<String>,

    /// Value date (for interest calculations)
    #[serde(default)]
    pub value_date: Option<NaiveDate>,

    /// Tax code
    pub tax_code: Option<String>,

    /// Tax amount
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub tax_amount: Option<Decimal>,

    /// Assignment field (for account assignment)
    pub assignment: Option<String>,

    /// Reference to offsetting account (for network generation)
    pub offsetting_account: Option<String>,

    /// Is this posting to a suspense/clearing account
    pub is_suspense: bool,

    /// Trading partner company code (for intercompany)
    pub trading_partner: Option<String>,

    /// Quantity (for quantity-based postings)
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub quantity: Option<Decimal>,

    /// Unit of measure
    pub unit_of_measure: Option<String>,

    /// Unit (alias for unit_of_measure for compatibility)
    #[serde(default)]
    pub unit: Option<String>,

    /// Project code
    #[serde(default)]
    pub project_code: Option<String>,
}

impl JournalEntryLine {
    /// Create a new debit line item.
    pub fn debit(document_id: Uuid, line_number: u32, gl_account: String, amount: Decimal) -> Self {
        Self {
            document_id,
            line_number,
            gl_account: gl_account.clone(),
            account_code: gl_account,
            account_description: None,
            debit_amount: amount,
            credit_amount: Decimal::ZERO,
            local_amount: amount,
            group_amount: None,
            cost_center: None,
            profit_center: None,
            segment: None,
            functional_area: None,
            line_text: None,
            text: None,
            reference: None,
            value_date: None,
            tax_code: None,
            tax_amount: None,
            assignment: None,
            offsetting_account: None,
            is_suspense: false,
            trading_partner: None,
            quantity: None,
            unit_of_measure: None,
            unit: None,
            project_code: None,
        }
    }

    /// Create a new credit line item.
    pub fn credit(
        document_id: Uuid,
        line_number: u32,
        gl_account: String,
        amount: Decimal,
    ) -> Self {
        Self {
            document_id,
            line_number,
            gl_account: gl_account.clone(),
            account_code: gl_account,
            account_description: None,
            debit_amount: Decimal::ZERO,
            credit_amount: amount,
            local_amount: -amount,
            group_amount: None,
            cost_center: None,
            profit_center: None,
            segment: None,
            functional_area: None,
            line_text: None,
            text: None,
            reference: None,
            value_date: None,
            tax_code: None,
            tax_amount: None,
            assignment: None,
            offsetting_account: None,
            is_suspense: false,
            trading_partner: None,
            quantity: None,
            unit_of_measure: None,
            unit: None,
            project_code: None,
        }
    }

    /// Check if this is a debit posting.
    pub fn is_debit(&self) -> bool {
        self.debit_amount > Decimal::ZERO
    }

    /// Check if this is a credit posting.
    pub fn is_credit(&self) -> bool {
        self.credit_amount > Decimal::ZERO
    }

    /// Get the signed amount (positive for debit, negative for credit).
    pub fn signed_amount(&self) -> Decimal {
        self.debit_amount - self.credit_amount
    }

    // Convenience accessors for compatibility

    /// Get the account code (alias for gl_account).
    #[allow(clippy::misnamed_getters)]
    pub fn account_code(&self) -> &str {
        &self.gl_account
    }

    /// Get the account description (currently returns empty string as not stored).
    pub fn account_description(&self) -> &str {
        // Account descriptions are typically looked up from CoA, not stored per line
        ""
    }
}

impl Default for JournalEntryLine {
    fn default() -> Self {
        Self {
            document_id: Uuid::nil(),
            line_number: 0,
            gl_account: String::new(),
            account_code: String::new(),
            account_description: None,
            debit_amount: Decimal::ZERO,
            credit_amount: Decimal::ZERO,
            local_amount: Decimal::ZERO,
            group_amount: None,
            cost_center: None,
            profit_center: None,
            segment: None,
            functional_area: None,
            line_text: None,
            text: None,
            reference: None,
            value_date: None,
            tax_code: None,
            tax_amount: None,
            assignment: None,
            offsetting_account: None,
            is_suspense: false,
            trading_partner: None,
            quantity: None,
            unit_of_measure: None,
            unit: None,
            project_code: None,
        }
    }
}

/// Complete journal entry with header and line items.
///
/// Represents a balanced double-entry bookkeeping transaction where
/// total debits must equal total credits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Header with document metadata
    pub header: JournalEntryHeader,
    /// Line items (debit and credit postings)
    pub lines: Vec<JournalEntryLine>,
}

impl JournalEntry {
    /// Create a new journal entry with header and empty lines.
    pub fn new(header: JournalEntryHeader) -> Self {
        Self {
            header,
            lines: Vec::new(),
        }
    }

    /// Create a new journal entry with basic parameters (convenience constructor).
    ///
    /// This is a simplified constructor for backwards compatibility that creates
    /// a journal entry with the specified document number, company code, posting date,
    /// and description.
    pub fn new_simple(
        _document_number: String,
        company_code: String,
        posting_date: NaiveDate,
        description: String,
    ) -> Self {
        let mut header = JournalEntryHeader::new(company_code, posting_date);
        header.header_text = Some(description);
        Self {
            header,
            lines: Vec::new(),
        }
    }

    /// Add a line item to the journal entry.
    pub fn add_line(&mut self, line: JournalEntryLine) {
        self.lines.push(line);
    }

    /// Get the total debit amount.
    pub fn total_debit(&self) -> Decimal {
        self.lines.iter().map(|l| l.debit_amount).sum()
    }

    /// Get the total credit amount.
    pub fn total_credit(&self) -> Decimal {
        self.lines.iter().map(|l| l.credit_amount).sum()
    }

    /// Check if the journal entry is balanced (debits = credits).
    pub fn is_balanced(&self) -> bool {
        self.total_debit() == self.total_credit()
    }

    /// Get the out-of-balance amount (should be zero for valid entries).
    pub fn balance_difference(&self) -> Decimal {
        self.total_debit() - self.total_credit()
    }

    /// Get the number of line items.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Check if the line count is even.
    pub fn has_even_line_count(&self) -> bool {
        self.lines.len().is_multiple_of(2)
    }

    /// Get the count of debit and credit lines.
    pub fn debit_credit_counts(&self) -> (usize, usize) {
        let debits = self.lines.iter().filter(|l| l.is_debit()).count();
        let credits = self.lines.iter().filter(|l| l.is_credit()).count();
        (debits, credits)
    }

    /// Check if debit and credit line counts are equal.
    pub fn has_equal_debit_credit_counts(&self) -> bool {
        let (d, c) = self.debit_credit_counts();
        d == c
    }

    /// Get unique GL accounts used in this entry.
    pub fn unique_accounts(&self) -> Vec<&str> {
        let mut accounts: Vec<&str> = self.lines.iter().map(|l| l.gl_account.as_str()).collect();
        accounts.sort();
        accounts.dedup();
        accounts
    }

    /// Check if any line posts to a suspense account.
    pub fn has_suspense_posting(&self) -> bool {
        self.lines.iter().any(|l| l.is_suspense)
    }

    // Convenience accessors for header fields

    /// Get the company code.
    pub fn company_code(&self) -> &str {
        &self.header.company_code
    }

    /// Get the document number (document_id as string).
    pub fn document_number(&self) -> String {
        self.header.document_id.to_string()
    }

    /// Get the posting date.
    pub fn posting_date(&self) -> NaiveDate {
        self.header.posting_date
    }

    /// Get the document date.
    pub fn document_date(&self) -> NaiveDate {
        self.header.document_date
    }

    /// Get the fiscal year.
    pub fn fiscal_year(&self) -> u16 {
        self.header.fiscal_year
    }

    /// Get the fiscal period.
    pub fn fiscal_period(&self) -> u8 {
        self.header.fiscal_period
    }

    /// Get the currency.
    pub fn currency(&self) -> &str {
        &self.header.currency
    }

    /// Check if this entry is marked as fraud.
    pub fn is_fraud(&self) -> bool {
        self.header.is_fraud
    }

    /// Check if this entry has a SOD violation.
    pub fn has_sod_violation(&self) -> bool {
        self.header.sod_violation
    }

    /// Get the description (header text).
    pub fn description(&self) -> Option<&str> {
        self.header.header_text.as_deref()
    }

    /// Set the description (header text).
    pub fn set_description(&mut self, description: String) {
        self.header.header_text = Some(description);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_entry() {
        let header = JournalEntryHeader::new(
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );
        let mut entry = JournalEntry::new(header);

        entry.add_line(JournalEntryLine::debit(
            entry.header.document_id,
            1,
            "100000".to_string(),
            Decimal::from(1000),
        ));
        entry.add_line(JournalEntryLine::credit(
            entry.header.document_id,
            2,
            "200000".to_string(),
            Decimal::from(1000),
        ));

        assert!(entry.is_balanced());
        assert_eq!(entry.line_count(), 2);
        assert!(entry.has_even_line_count());
        assert!(entry.has_equal_debit_credit_counts());
    }

    #[test]
    fn test_unbalanced_entry() {
        let header = JournalEntryHeader::new(
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );
        let mut entry = JournalEntry::new(header);

        entry.add_line(JournalEntryLine::debit(
            entry.header.document_id,
            1,
            "100000".to_string(),
            Decimal::from(1000),
        ));
        entry.add_line(JournalEntryLine::credit(
            entry.header.document_id,
            2,
            "200000".to_string(),
            Decimal::from(500),
        ));

        assert!(!entry.is_balanced());
        assert_eq!(entry.balance_difference(), Decimal::from(500));
    }
}
