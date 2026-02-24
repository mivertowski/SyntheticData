//! Generate journal entries from document flows.
//!
//! This module creates proper GL entries from P2P and O2C document chains,
//! ensuring that document flow activity is reflected in the general ledger.
//!
//! # P2P Flow JE Mappings
//! - GoodsReceipt → DR Inventory, CR GR/IR Clearing
//! - VendorInvoice → DR GR/IR Clearing, CR AP
//! - Payment → DR AP, CR Cash
//!
//! # O2C Flow JE Mappings
//! - Delivery → DR COGS, CR Inventory
//! - CustomerInvoice → DR AR, CR Revenue
//! - CustomerReceipt → DR Cash, CR AR

use rust_decimal::Decimal;

use datasynth_core::accounts::{
    cash_accounts, control_accounts, expense_accounts, revenue_accounts,
};
use datasynth_core::models::{
    documents::{CustomerInvoice, Delivery, GoodsReceipt, Payment, VendorInvoice},
    BusinessProcess, JournalEntry, JournalEntryHeader, JournalEntryLine, TransactionSource,
};
use datasynth_core::pcg::{
    cash_accounts as pcg_cash, control_accounts as pcg_control, expense_accounts as pcg_expense,
    revenue_accounts as pcg_revenue,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};

use super::{O2CDocumentChain, P2PDocumentChain};

/// Configuration for document flow JE generation.
#[derive(Debug, Clone)]
pub struct DocumentFlowJeConfig {
    /// Inventory account (default: 1200 from control_accounts::INVENTORY)
    pub inventory_account: String,
    /// GR/IR clearing account (default: 2900 from control_accounts::GR_IR_CLEARING)
    pub gr_ir_clearing_account: String,
    /// Accounts payable control account (default: 2000 from control_accounts::AP_CONTROL)
    pub ap_account: String,
    /// Cash/bank account (default: 1000 from cash_accounts::OPERATING_CASH)
    pub cash_account: String,
    /// Accounts receivable control account (default: 1100 from control_accounts::AR_CONTROL)
    pub ar_account: String,
    /// Revenue account (default: 4000 from revenue_accounts::PRODUCT_REVENUE)
    pub revenue_account: String,
    /// COGS account (default: 5000 from expense_accounts::COGS)
    pub cogs_account: String,
}

impl Default for DocumentFlowJeConfig {
    fn default() -> Self {
        Self {
            inventory_account: control_accounts::INVENTORY.to_string(),
            gr_ir_clearing_account: control_accounts::GR_IR_CLEARING.to_string(),
            ap_account: control_accounts::AP_CONTROL.to_string(),
            cash_account: cash_accounts::OPERATING_CASH.to_string(),
            ar_account: control_accounts::AR_CONTROL.to_string(),
            revenue_account: revenue_accounts::PRODUCT_REVENUE.to_string(),
            cogs_account: expense_accounts::COGS.to_string(),
        }
    }
}

/// French GAAP (PCG 2024): use only PCG accounts, no extra accounts.
impl DocumentFlowJeConfig {
    pub fn french_gaap() -> Self {
        Self {
            inventory_account: pcg_control::INVENTORY.to_string(),
            gr_ir_clearing_account: pcg_control::GR_IR_CLEARING.to_string(),
            ap_account: pcg_control::AP_CONTROL.to_string(),
            cash_account: pcg_cash::OPERATING_CASH.to_string(),
            ar_account: pcg_control::AR_CONTROL.to_string(),
            revenue_account: pcg_revenue::PRODUCT_REVENUE.to_string(),
            cogs_account: pcg_expense::COGS.to_string(),
        }
    }
}

/// Generator for creating JEs from document flows.
pub struct DocumentFlowJeGenerator {
    config: DocumentFlowJeConfig,
    uuid_factory: DeterministicUuidFactory,
}

impl DocumentFlowJeGenerator {
    /// Create a new document flow JE generator with default config and seed 0.
    pub fn new() -> Self {
        Self::with_config_and_seed(DocumentFlowJeConfig::default(), 0)
    }

    /// Create with custom account configuration and seed.
    pub fn with_config_and_seed(config: DocumentFlowJeConfig, seed: u64) -> Self {
        Self {
            config,
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::DocumentFlow),
        }
    }

    /// Generate all JEs from a P2P document chain.
    pub fn generate_from_p2p_chain(&mut self, chain: &P2PDocumentChain) -> Vec<JournalEntry> {
        let mut entries = Vec::new();

        // Generate JEs for goods receipts
        for gr in &chain.goods_receipts {
            if let Some(je) = self.generate_from_goods_receipt(gr) {
                entries.push(je);
            }
        }

        // Generate JE for vendor invoice
        if let Some(ref invoice) = chain.vendor_invoice {
            if let Some(je) = self.generate_from_vendor_invoice(invoice) {
                entries.push(je);
            }
        }

        // Generate JE for payment
        if let Some(ref payment) = chain.payment {
            if let Some(je) = self.generate_from_ap_payment(payment) {
                entries.push(je);
            }
        }

        entries
    }

    /// Generate all JEs from an O2C document chain.
    pub fn generate_from_o2c_chain(&mut self, chain: &O2CDocumentChain) -> Vec<JournalEntry> {
        let mut entries = Vec::new();

        // Generate JEs for deliveries
        for delivery in &chain.deliveries {
            if let Some(je) = self.generate_from_delivery(delivery) {
                entries.push(je);
            }
        }

        // Generate JE for customer invoice
        if let Some(ref invoice) = chain.customer_invoice {
            if let Some(je) = self.generate_from_customer_invoice(invoice) {
                entries.push(je);
            }
        }

        // Generate JE for customer receipt
        if let Some(ref receipt) = chain.customer_receipt {
            if let Some(je) = self.generate_from_ar_receipt(receipt) {
                entries.push(je);
            }
        }

        entries
    }

    /// Generate JE from Goods Receipt.
    /// DR Inventory, CR GR/IR Clearing
    pub fn generate_from_goods_receipt(&mut self, gr: &GoodsReceipt) -> Option<JournalEntry> {
        if gr.items.is_empty() {
            return None;
        }

        let document_id = self.uuid_factory.next();

        // Use the total_value from the GR, or calculate from line items
        let total_amount = if gr.total_value > Decimal::ZERO {
            gr.total_value
        } else {
            gr.items
                .iter()
                .map(|item| item.base.net_amount)
                .sum::<Decimal>()
        };

        if total_amount == Decimal::ZERO {
            return None;
        }

        // Use posting_date or fall back to document_date
        let posting_date = gr.header.posting_date.unwrap_or(gr.header.document_date);

        let mut header = JournalEntryHeader::with_deterministic_id(
            gr.header.company_code.clone(),
            posting_date,
            document_id,
        );
        header.source = TransactionSource::Automated;
        header.business_process = Some(BusinessProcess::P2P);
        header.reference = Some(format!("GR:{}", gr.header.document_id));
        header.header_text = Some(format!(
            "Goods Receipt {} - {}",
            gr.header.document_id,
            gr.vendor_id.as_deref().unwrap_or("Unknown")
        ));

        let mut entry = JournalEntry::new(header);

        // DR Inventory
        let debit_line = JournalEntryLine::debit(
            entry.header.document_id,
            1,
            self.config.inventory_account.clone(),
            total_amount,
        );
        entry.add_line(debit_line);

        // CR GR/IR Clearing
        let credit_line = JournalEntryLine::credit(
            entry.header.document_id,
            2,
            self.config.gr_ir_clearing_account.clone(),
            total_amount,
        );
        entry.add_line(credit_line);

        Some(entry)
    }

    /// Generate JE from Vendor Invoice.
    /// DR GR/IR Clearing, CR AP
    pub fn generate_from_vendor_invoice(
        &mut self,
        invoice: &VendorInvoice,
    ) -> Option<JournalEntry> {
        if invoice.payable_amount == Decimal::ZERO {
            return None;
        }

        let document_id = self.uuid_factory.next();

        // Use posting_date or fall back to document_date
        let posting_date = invoice
            .header
            .posting_date
            .unwrap_or(invoice.header.document_date);

        let mut header = JournalEntryHeader::with_deterministic_id(
            invoice.header.company_code.clone(),
            posting_date,
            document_id,
        );
        header.source = TransactionSource::Automated;
        header.business_process = Some(BusinessProcess::P2P);
        header.reference = Some(format!("VI:{}", invoice.header.document_id));
        header.header_text = Some(format!(
            "Vendor Invoice {} - {}",
            invoice.vendor_invoice_number, invoice.vendor_id
        ));

        let mut entry = JournalEntry::new(header);

        // DR GR/IR Clearing (or expense if no PO)
        let debit_line = JournalEntryLine::debit(
            entry.header.document_id,
            1,
            self.config.gr_ir_clearing_account.clone(),
            invoice.payable_amount,
        );
        entry.add_line(debit_line);

        // CR Accounts Payable
        let credit_line = JournalEntryLine::credit(
            entry.header.document_id,
            2,
            self.config.ap_account.clone(),
            invoice.payable_amount,
        );
        entry.add_line(credit_line);

        Some(entry)
    }

    /// Generate JE from AP Payment.
    /// DR AP, CR Cash
    pub fn generate_from_ap_payment(&mut self, payment: &Payment) -> Option<JournalEntry> {
        if payment.amount == Decimal::ZERO {
            return None;
        }

        let document_id = self.uuid_factory.next();

        // Use posting_date or fall back to document_date
        let posting_date = payment
            .header
            .posting_date
            .unwrap_or(payment.header.document_date);

        let mut header = JournalEntryHeader::with_deterministic_id(
            payment.header.company_code.clone(),
            posting_date,
            document_id,
        );
        header.source = TransactionSource::Automated;
        header.business_process = Some(BusinessProcess::P2P);
        header.reference = Some(format!("PAY:{}", payment.header.document_id));
        header.header_text = Some(format!(
            "Payment {} - {}",
            payment.header.document_id, payment.business_partner_id
        ));

        let mut entry = JournalEntry::new(header);

        // DR Accounts Payable
        let debit_line = JournalEntryLine::debit(
            entry.header.document_id,
            1,
            self.config.ap_account.clone(),
            payment.amount,
        );
        entry.add_line(debit_line);

        // CR Cash/Bank
        let credit_line = JournalEntryLine::credit(
            entry.header.document_id,
            2,
            self.config.cash_account.clone(),
            payment.amount,
        );
        entry.add_line(credit_line);

        Some(entry)
    }

    /// Generate JE from Delivery.
    /// DR COGS, CR Inventory
    pub fn generate_from_delivery(&mut self, delivery: &Delivery) -> Option<JournalEntry> {
        if delivery.items.is_empty() {
            return None;
        }

        let document_id = self.uuid_factory.next();

        // Calculate total cost from line items
        let total_cost = delivery
            .items
            .iter()
            .map(|item| item.base.net_amount)
            .sum::<Decimal>();

        if total_cost == Decimal::ZERO {
            return None;
        }

        // Use posting_date or fall back to document_date
        let posting_date = delivery
            .header
            .posting_date
            .unwrap_or(delivery.header.document_date);

        let mut header = JournalEntryHeader::with_deterministic_id(
            delivery.header.company_code.clone(),
            posting_date,
            document_id,
        );
        header.source = TransactionSource::Automated;
        header.business_process = Some(BusinessProcess::O2C);
        header.reference = Some(format!("DEL:{}", delivery.header.document_id));
        header.header_text = Some(format!(
            "Delivery {} - {}",
            delivery.header.document_id, delivery.customer_id
        ));

        let mut entry = JournalEntry::new(header);

        // DR COGS
        let debit_line = JournalEntryLine::debit(
            entry.header.document_id,
            1,
            self.config.cogs_account.clone(),
            total_cost,
        );
        entry.add_line(debit_line);

        // CR Inventory
        let credit_line = JournalEntryLine::credit(
            entry.header.document_id,
            2,
            self.config.inventory_account.clone(),
            total_cost,
        );
        entry.add_line(credit_line);

        Some(entry)
    }

    /// Generate JE from Customer Invoice.
    /// DR AR, CR Revenue
    pub fn generate_from_customer_invoice(
        &mut self,
        invoice: &CustomerInvoice,
    ) -> Option<JournalEntry> {
        if invoice.total_gross_amount == Decimal::ZERO {
            return None;
        }

        let document_id = self.uuid_factory.next();

        // Use posting_date or fall back to document_date
        let posting_date = invoice
            .header
            .posting_date
            .unwrap_or(invoice.header.document_date);

        let mut header = JournalEntryHeader::with_deterministic_id(
            invoice.header.company_code.clone(),
            posting_date,
            document_id,
        );
        header.source = TransactionSource::Automated;
        header.business_process = Some(BusinessProcess::O2C);
        header.reference = Some(format!("CI:{}", invoice.header.document_id));
        header.header_text = Some(format!(
            "Customer Invoice {} - {}",
            invoice.header.document_id, invoice.customer_id
        ));

        let mut entry = JournalEntry::new(header);

        // DR Accounts Receivable
        let debit_line = JournalEntryLine::debit(
            entry.header.document_id,
            1,
            self.config.ar_account.clone(),
            invoice.total_gross_amount,
        );
        entry.add_line(debit_line);

        // CR Revenue
        let credit_line = JournalEntryLine::credit(
            entry.header.document_id,
            2,
            self.config.revenue_account.clone(),
            invoice.total_gross_amount,
        );
        entry.add_line(credit_line);

        Some(entry)
    }

    /// Generate JE from AR Receipt (Customer Payment).
    /// DR Cash, CR AR
    pub fn generate_from_ar_receipt(&mut self, payment: &Payment) -> Option<JournalEntry> {
        if payment.amount == Decimal::ZERO {
            return None;
        }

        let document_id = self.uuid_factory.next();

        // Use posting_date or fall back to document_date
        let posting_date = payment
            .header
            .posting_date
            .unwrap_or(payment.header.document_date);

        let mut header = JournalEntryHeader::with_deterministic_id(
            payment.header.company_code.clone(),
            posting_date,
            document_id,
        );
        header.source = TransactionSource::Automated;
        header.business_process = Some(BusinessProcess::O2C);
        header.reference = Some(format!("RCP:{}", payment.header.document_id));
        header.header_text = Some(format!(
            "Customer Receipt {} - {}",
            payment.header.document_id, payment.business_partner_id
        ));

        let mut entry = JournalEntry::new(header);

        // DR Cash/Bank
        let debit_line = JournalEntryLine::debit(
            entry.header.document_id,
            1,
            self.config.cash_account.clone(),
            payment.amount,
        );
        entry.add_line(debit_line);

        // CR Accounts Receivable
        let credit_line = JournalEntryLine::credit(
            entry.header.document_id,
            2,
            self.config.ar_account.clone(),
            payment.amount,
        );
        entry.add_line(credit_line);

        Some(entry)
    }
}

impl Default for DocumentFlowJeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::documents::{GoodsReceiptItem, MovementType};

    fn create_test_gr() -> GoodsReceipt {
        let mut gr = GoodsReceipt::from_purchase_order(
            "GR-001".to_string(),
            "1000",
            "PO-001",
            "V-001",
            "P1000",
            "0001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        let item = GoodsReceiptItem::from_po(
            10,
            "Test Material",
            Decimal::from(100),
            Decimal::from(50),
            "PO-001",
            10,
        )
        .with_movement_type(MovementType::GrForPo);

        gr.add_item(item);
        gr.post("JSMITH", NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        gr
    }

    fn create_test_vendor_invoice() -> VendorInvoice {
        use datasynth_core::models::documents::VendorInvoiceItem;

        let mut invoice = VendorInvoice::new(
            "VI-001".to_string(),
            "1000",
            "V-001",
            "INV-12345".to_string(),
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            "JSMITH",
        );

        let item = VendorInvoiceItem::from_po_gr(
            10,
            "Test Material",
            Decimal::from(100),
            Decimal::from(50),
            "PO-001",
            10,
            Some("GR-001".to_string()),
            Some(10),
        );

        invoice.add_item(item);
        invoice.post("JSMITH", NaiveDate::from_ymd_opt(2024, 1, 20).unwrap());

        invoice
    }

    fn create_test_payment() -> Payment {
        let mut payment = Payment::new_ap_payment(
            "PAY-001".to_string(),
            "1000",
            "V-001",
            Decimal::from(5000),
            2024,
            2,
            NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
            "JSMITH",
        );

        payment.post("JSMITH", NaiveDate::from_ymd_opt(2024, 2, 15).unwrap());

        payment
    }

    #[test]
    fn test_generate_from_goods_receipt() {
        let mut generator = DocumentFlowJeGenerator::new();
        let gr = create_test_gr();

        let je = generator.generate_from_goods_receipt(&gr);

        assert!(je.is_some());
        let je = je.unwrap();

        // Should be balanced
        assert!(je.is_balanced());

        // Should have 2 lines
        assert_eq!(je.line_count(), 2);

        // DR should be inventory, CR should be GR/IR
        assert!(je.total_debit() > Decimal::ZERO);
        assert_eq!(je.total_debit(), je.total_credit());

        // Should reference source document
        assert!(je.header.reference.is_some());
        assert!(je.header.reference.as_ref().unwrap().contains("GR:"));
    }

    #[test]
    fn test_generate_from_vendor_invoice() {
        let mut generator = DocumentFlowJeGenerator::new();
        let invoice = create_test_vendor_invoice();

        let je = generator.generate_from_vendor_invoice(&invoice);

        assert!(je.is_some());
        let je = je.unwrap();

        assert!(je.is_balanced());
        assert_eq!(je.line_count(), 2);
        assert!(je.header.reference.as_ref().unwrap().contains("VI:"));
    }

    #[test]
    fn test_generate_from_ap_payment() {
        let mut generator = DocumentFlowJeGenerator::new();
        let payment = create_test_payment();

        let je = generator.generate_from_ap_payment(&payment);

        assert!(je.is_some());
        let je = je.unwrap();

        assert!(je.is_balanced());
        assert_eq!(je.line_count(), 2);
        assert!(je.header.reference.as_ref().unwrap().contains("PAY:"));
    }

    #[test]
    fn test_all_entries_are_balanced() {
        let mut generator = DocumentFlowJeGenerator::new();

        let gr = create_test_gr();
        let invoice = create_test_vendor_invoice();
        let payment = create_test_payment();

        let entries = vec![
            generator.generate_from_goods_receipt(&gr),
            generator.generate_from_vendor_invoice(&invoice),
            generator.generate_from_ap_payment(&payment),
        ];

        for entry in entries.into_iter().flatten() {
            assert!(
                entry.is_balanced(),
                "Entry {} is not balanced",
                entry.header.document_id
            );
        }
    }
}
