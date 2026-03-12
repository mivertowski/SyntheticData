//! Document reference chain for tracking document relationships.
//!
//! Provides structures for tracking the relationships between documents
//! in business processes (e.g., PO -> GR -> Invoice -> Payment).

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of business document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    // P2P Documents
    /// Purchase Requisition
    PurchaseRequisition,
    /// Purchase Order
    PurchaseOrder,
    /// Goods Receipt
    GoodsReceipt,
    /// Vendor Invoice
    VendorInvoice,
    /// AP Payment
    ApPayment,
    /// Debit Memo (AP)
    DebitMemo,

    // O2C Documents
    /// Sales Quote
    SalesQuote,
    /// Sales Order
    SalesOrder,
    /// Delivery
    Delivery,
    /// Customer Invoice
    CustomerInvoice,
    /// Customer Receipt
    CustomerReceipt,
    /// Credit Memo (AR)
    CreditMemo,

    // Financial Documents
    /// Journal Entry
    JournalEntry,
    /// Asset Acquisition
    AssetAcquisition,
    /// Depreciation Run
    DepreciationRun,
    /// Intercompany Document
    IntercompanyDocument,

    // Other
    /// General document
    General,
}

impl DocumentType {
    /// Get the document type prefix for ID generation.
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::PurchaseRequisition => "PR",
            Self::PurchaseOrder => "PO",
            Self::GoodsReceipt => "GR",
            Self::VendorInvoice => "VI",
            Self::ApPayment => "AP",
            Self::DebitMemo => "DM",
            Self::SalesQuote => "SQ",
            Self::SalesOrder => "SO",
            Self::Delivery => "DL",
            Self::CustomerInvoice => "CI",
            Self::CustomerReceipt => "CR",
            Self::CreditMemo => "CM",
            Self::JournalEntry => "JE",
            Self::AssetAcquisition => "AA",
            Self::DepreciationRun => "DR",
            Self::IntercompanyDocument => "IC",
            Self::General => "GN",
        }
    }

    /// Check if this document type generates GL entries.
    pub fn creates_gl_entry(&self) -> bool {
        !matches!(
            self,
            Self::PurchaseRequisition | Self::PurchaseOrder | Self::SalesQuote | Self::SalesOrder
        )
    }

    /// Get the business process this document belongs to.
    pub fn business_process(&self) -> &'static str {
        match self {
            Self::PurchaseRequisition
            | Self::PurchaseOrder
            | Self::GoodsReceipt
            | Self::VendorInvoice
            | Self::ApPayment
            | Self::DebitMemo => "P2P",

            Self::SalesQuote
            | Self::SalesOrder
            | Self::Delivery
            | Self::CustomerInvoice
            | Self::CustomerReceipt
            | Self::CreditMemo => "O2C",

            Self::JournalEntry => "R2R",
            Self::AssetAcquisition | Self::DepreciationRun => "A2R",
            Self::IntercompanyDocument => "IC",
            Self::General => "GEN",
        }
    }
}

/// Type of reference relationship between documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceType {
    /// Follow-on document (normal flow: PO -> GR)
    FollowOn,
    /// Payment for invoice
    Payment,
    /// Reversal/correction of document
    Reversal,
    /// Partial fulfillment (partial GR, partial payment)
    Partial,
    /// Credit memo related to invoice
    CreditMemo,
    /// Debit memo related to invoice
    DebitMemo,
    /// Return related to delivery
    Return,
    /// Intercompany matching document
    IntercompanyMatch,
    /// Manual reference (user-defined)
    Manual,
}

/// Reference between two documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReference {
    /// Reference ID
    pub reference_id: Uuid,

    /// Source document type
    pub source_doc_type: DocumentType,

    /// Source document ID
    pub source_doc_id: String,

    /// Target document type
    pub target_doc_type: DocumentType,

    /// Target document ID
    pub target_doc_id: String,

    /// Type of reference relationship
    pub reference_type: ReferenceType,

    /// Company code
    pub company_code: String,

    /// Date the reference was created
    pub reference_date: NaiveDate,

    /// Description/notes
    pub description: Option<String>,

    /// Amount covered by this reference (for partial references)
    pub reference_amount: Option<rust_decimal::Decimal>,
}

impl DocumentReference {
    /// Create a new document reference.
    pub fn new(
        source_type: DocumentType,
        source_id: impl Into<String>,
        target_type: DocumentType,
        target_id: impl Into<String>,
        ref_type: ReferenceType,
        company_code: impl Into<String>,
        date: NaiveDate,
    ) -> Self {
        Self {
            reference_id: Uuid::new_v4(),
            source_doc_type: source_type,
            source_doc_id: source_id.into(),
            target_doc_type: target_type,
            target_doc_id: target_id.into(),
            reference_type: ref_type,
            company_code: company_code.into(),
            reference_date: date,
            description: None,
            reference_amount: None,
        }
    }

    /// Create a follow-on reference.
    pub fn follow_on(
        source_type: DocumentType,
        source_id: impl Into<String>,
        target_type: DocumentType,
        target_id: impl Into<String>,
        company_code: impl Into<String>,
        date: NaiveDate,
    ) -> Self {
        Self::new(
            source_type,
            source_id,
            target_type,
            target_id,
            ReferenceType::FollowOn,
            company_code,
            date,
        )
    }

    /// Create a payment reference.
    pub fn payment(
        invoice_type: DocumentType,
        invoice_id: impl Into<String>,
        payment_id: impl Into<String>,
        company_code: impl Into<String>,
        date: NaiveDate,
        amount: rust_decimal::Decimal,
    ) -> Self {
        let payment_type = match invoice_type {
            DocumentType::VendorInvoice => DocumentType::ApPayment,
            DocumentType::CustomerInvoice => DocumentType::CustomerReceipt,
            _ => DocumentType::ApPayment,
        };

        let mut reference = Self::new(
            invoice_type,
            invoice_id,
            payment_type,
            payment_id,
            ReferenceType::Payment,
            company_code,
            date,
        );
        reference.reference_amount = Some(amount);
        reference
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set reference amount.
    pub fn with_amount(mut self, amount: rust_decimal::Decimal) -> Self {
        self.reference_amount = Some(amount);
        self
    }
}

/// Document status in workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    /// Draft/not yet released
    #[default]
    Draft,
    /// Submitted for approval
    Submitted,
    /// Pending approval
    PendingApproval,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
    /// Released for processing
    Released,
    /// Partially processed
    PartiallyProcessed,
    /// Fully processed/completed
    Completed,
    /// Cancelled/voided
    Cancelled,
    /// Posted to GL
    Posted,
    /// Cleared (for open items)
    Cleared,
}

impl DocumentStatus {
    /// Check if document can be modified.
    pub fn is_editable(&self) -> bool {
        matches!(self, Self::Draft | Self::Rejected)
    }

    /// Check if document can be cancelled.
    pub fn can_cancel(&self) -> bool {
        !matches!(self, Self::Cancelled | Self::Cleared | Self::Completed)
    }

    /// Check if document needs approval.
    pub fn needs_approval(&self) -> bool {
        matches!(self, Self::Submitted | Self::PendingApproval)
    }
}

/// Common document header fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHeader {
    /// Unique document ID
    pub document_id: String,

    /// Document type
    pub document_type: DocumentType,

    /// Company code
    pub company_code: String,

    /// Fiscal year
    pub fiscal_year: u16,

    /// Fiscal period
    pub fiscal_period: u8,

    /// Document date
    pub document_date: NaiveDate,

    /// Posting date (if applicable)
    pub posting_date: Option<NaiveDate>,

    /// Entry date (when document was created)
    pub entry_date: NaiveDate,

    /// Entry timestamp
    pub entry_timestamp: NaiveDateTime,

    /// Document status
    pub status: DocumentStatus,

    /// Created by user
    pub created_by: String,

    /// Last changed by user
    pub changed_by: Option<String>,

    /// Last change timestamp
    pub changed_at: Option<NaiveDateTime>,

    /// Employee ID of the creator (bridges user_id ↔ employee_id)
    ///
    /// `created_by` stores the user login (e.g. "JSMITH") while
    /// employee nodes use `employee_id` (e.g. "E-001234"). This
    /// field stores the employee_id when it is known at generation
    /// time, allowing the export pipeline to emit
    /// `DOC_CREATED_BY` edges directly without an expensive lookup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by_employee_id: Option<String>,

    /// Currency
    pub currency: String,

    /// Reference number (external)
    pub reference: Option<String>,

    /// Header text
    pub header_text: Option<String>,

    /// Related journal entry ID (if posted to GL)
    pub journal_entry_id: Option<String>,

    /// References to other documents
    pub document_references: Vec<DocumentReference>,
}

impl DocumentHeader {
    /// Create a new document header.
    pub fn new(
        document_id: impl Into<String>,
        document_type: DocumentType,
        company_code: impl Into<String>,
        fiscal_year: u16,
        fiscal_period: u8,
        document_date: NaiveDate,
        created_by: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            document_id: document_id.into(),
            document_type,
            company_code: company_code.into(),
            fiscal_year,
            fiscal_period,
            document_date,
            posting_date: None,
            entry_date: document_date,
            entry_timestamp: now,
            status: DocumentStatus::Draft,
            created_by: created_by.into(),
            changed_by: None,
            changed_at: None,
            created_by_employee_id: None,
            currency: "USD".to_string(),
            reference: None,
            header_text: None,
            journal_entry_id: None,
            document_references: Vec::new(),
        }
    }

    /// Set the employee ID of the document creator.
    pub fn with_created_by_employee_id(mut self, employee_id: impl Into<String>) -> Self {
        self.created_by_employee_id = Some(employee_id.into());
        self
    }

    /// Set posting date.
    pub fn with_posting_date(mut self, date: NaiveDate) -> Self {
        self.posting_date = Some(date);
        self
    }

    /// Set currency.
    pub fn with_currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = currency.into();
        self
    }

    /// Set reference.
    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    /// Set header text.
    pub fn with_header_text(mut self, text: impl Into<String>) -> Self {
        self.header_text = Some(text.into());
        self
    }

    /// Add a document reference.
    pub fn add_reference(&mut self, reference: DocumentReference) {
        self.document_references.push(reference);
    }

    /// Update status and record change.
    pub fn update_status(&mut self, new_status: DocumentStatus, user: impl Into<String>) {
        self.status = new_status;
        self.changed_by = Some(user.into());
        self.changed_at = Some(chrono::Utc::now().naive_utc());
    }

    /// Generate a deterministic document ID.
    pub fn generate_id(doc_type: DocumentType, company_code: &str, sequence: u64) -> String {
        format!("{}-{}-{:010}", doc_type.prefix(), company_code, sequence)
    }
}

/// Document line item common fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLineItem {
    /// Line item number
    pub line_number: u16,

    /// Material/service ID (if applicable)
    pub material_id: Option<String>,

    /// Description
    pub description: String,

    /// Quantity
    pub quantity: rust_decimal::Decimal,

    /// Unit of measure
    pub uom: String,

    /// Unit price
    pub unit_price: rust_decimal::Decimal,

    /// Net amount (quantity * unit_price)
    pub net_amount: rust_decimal::Decimal,

    /// Tax amount
    pub tax_amount: rust_decimal::Decimal,

    /// Gross amount (net + tax)
    pub gross_amount: rust_decimal::Decimal,

    /// GL account (for posting)
    pub gl_account: Option<String>,

    /// Cost center
    pub cost_center: Option<String>,

    /// Profit center
    pub profit_center: Option<String>,

    /// Internal order
    pub internal_order: Option<String>,

    /// WBS element
    pub wbs_element: Option<String>,

    /// Delivery date (for scheduling)
    pub delivery_date: Option<NaiveDate>,

    /// Plant/location
    pub plant: Option<String>,

    /// Storage location
    pub storage_location: Option<String>,

    /// Line text
    pub line_text: Option<String>,

    /// Is this line cancelled?
    pub is_cancelled: bool,
}

impl DocumentLineItem {
    /// Create a new line item.
    pub fn new(
        line_number: u16,
        description: impl Into<String>,
        quantity: rust_decimal::Decimal,
        unit_price: rust_decimal::Decimal,
    ) -> Self {
        let net_amount = quantity * unit_price;
        Self {
            line_number,
            material_id: None,
            description: description.into(),
            quantity,
            uom: "EA".to_string(),
            unit_price,
            net_amount,
            tax_amount: rust_decimal::Decimal::ZERO,
            gross_amount: net_amount,
            gl_account: None,
            cost_center: None,
            profit_center: None,
            internal_order: None,
            wbs_element: None,
            delivery_date: None,
            plant: None,
            storage_location: None,
            line_text: None,
            is_cancelled: false,
        }
    }

    /// Set material ID.
    pub fn with_material(mut self, material_id: impl Into<String>) -> Self {
        self.material_id = Some(material_id.into());
        self
    }

    /// Set GL account.
    pub fn with_gl_account(mut self, account: impl Into<String>) -> Self {
        self.gl_account = Some(account.into());
        self
    }

    /// Set cost center.
    pub fn with_cost_center(mut self, cost_center: impl Into<String>) -> Self {
        self.cost_center = Some(cost_center.into());
        self
    }

    /// Set tax amount and recalculate gross.
    pub fn with_tax(mut self, tax_amount: rust_decimal::Decimal) -> Self {
        self.tax_amount = tax_amount;
        self.gross_amount = self.net_amount + tax_amount;
        self
    }

    /// Set UOM.
    pub fn with_uom(mut self, uom: impl Into<String>) -> Self {
        self.uom = uom.into();
        self
    }

    /// Set delivery date.
    pub fn with_delivery_date(mut self, date: NaiveDate) -> Self {
        self.delivery_date = Some(date);
        self
    }

    /// Recalculate amounts.
    pub fn recalculate(&mut self) {
        self.net_amount = self.quantity * self.unit_price;
        self.gross_amount = self.net_amount + self.tax_amount;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_document_type_prefix() {
        assert_eq!(DocumentType::PurchaseOrder.prefix(), "PO");
        assert_eq!(DocumentType::VendorInvoice.prefix(), "VI");
        assert_eq!(DocumentType::CustomerInvoice.prefix(), "CI");
    }

    #[test]
    fn test_document_reference() {
        let reference = DocumentReference::follow_on(
            DocumentType::PurchaseOrder,
            "PO-1000-0000000001",
            DocumentType::GoodsReceipt,
            "GR-1000-0000000001",
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        assert_eq!(reference.reference_type, ReferenceType::FollowOn);
        assert_eq!(reference.source_doc_type, DocumentType::PurchaseOrder);
    }

    #[test]
    fn test_document_header() {
        let header = DocumentHeader::new(
            "PO-1000-0000000001",
            DocumentType::PurchaseOrder,
            "1000",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        )
        .with_currency("EUR")
        .with_reference("EXT-REF-123");

        assert_eq!(header.currency, "EUR");
        assert_eq!(header.reference, Some("EXT-REF-123".to_string()));
        assert_eq!(header.status, DocumentStatus::Draft);
    }

    #[test]
    fn test_document_line_item() {
        let item = DocumentLineItem::new(
            1,
            "Office Supplies",
            rust_decimal::Decimal::from(10),
            rust_decimal::Decimal::from(25),
        )
        .with_tax(rust_decimal::Decimal::from(25));

        assert_eq!(item.net_amount, rust_decimal::Decimal::from(250));
        assert_eq!(item.gross_amount, rust_decimal::Decimal::from(275));
    }

    #[test]
    fn test_document_status() {
        assert!(DocumentStatus::Draft.is_editable());
        assert!(!DocumentStatus::Posted.is_editable());
        assert!(DocumentStatus::Released.can_cancel());
        assert!(!DocumentStatus::Cancelled.can_cancel());
    }
}
