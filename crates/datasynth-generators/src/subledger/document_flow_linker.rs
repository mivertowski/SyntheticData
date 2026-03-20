//! Links document flows to subledger records.
//!
//! This module provides conversion functions to create subledger records
//! from document flow documents, ensuring data coherence between:
//! - P2P document flow (VendorInvoice) -> AP subledger (APInvoice)
//! - O2C document flow (CustomerInvoice) -> AR subledger (ARInvoice)

use std::collections::HashMap;

use rust_decimal::Decimal;

use datasynth_core::models::documents::{CustomerInvoice, Payment, PaymentType, VendorInvoice};
use datasynth_core::models::subledger::ap::{APInvoice, APInvoiceLine, MatchStatus};
use datasynth_core::models::subledger::ar::{ARInvoice, ARInvoiceLine};
use datasynth_core::models::subledger::{PaymentTerms, SubledgerDocumentStatus};

/// Links document flow invoices to subledger records.
#[derive(Default)]
pub struct DocumentFlowLinker {
    ap_counter: u64,
    ar_counter: u64,
    /// Vendor ID → vendor name lookup for realistic AP invoice names.
    vendor_names: HashMap<String, String>,
    /// Customer ID → customer name lookup for realistic AR invoice names.
    customer_names: HashMap<String, String>,
}

impl DocumentFlowLinker {
    /// Create a new document flow linker.
    pub fn new() -> Self {
        Self {
            ap_counter: 0,
            ar_counter: 0,
            vendor_names: HashMap::new(),
            customer_names: HashMap::new(),
        }
    }

    /// Set vendor name lookup map for realistic AP invoice vendor names.
    pub fn with_vendor_names(mut self, names: HashMap<String, String>) -> Self {
        self.vendor_names = names;
        self
    }

    /// Set customer name lookup map for realistic AR invoice customer names.
    pub fn with_customer_names(mut self, names: HashMap<String, String>) -> Self {
        self.customer_names = names;
        self
    }

    /// Convert a document flow VendorInvoice to an AP subledger APInvoice.
    ///
    /// This ensures that vendor invoices from the P2P flow create corresponding
    /// AP subledger records for complete data coherence.
    pub fn create_ap_invoice_from_vendor_invoice(
        &mut self,
        vendor_invoice: &VendorInvoice,
    ) -> APInvoice {
        self.ap_counter += 1;

        // Generate AP invoice number based on vendor invoice
        let invoice_number = format!("APINV{:08}", self.ap_counter);

        // Create the AP invoice.
        // Use the document-flow document ID as vendor_invoice_number so that
        // PaymentAllocation.invoice_id (which references the same document ID)
        // can be matched during settlement.
        let mut ap_invoice = APInvoice::new(
            invoice_number,
            vendor_invoice.header.document_id.clone(),
            vendor_invoice.header.company_code.clone(),
            vendor_invoice.vendor_id.clone(),
            self.vendor_names
                .get(&vendor_invoice.vendor_id)
                .cloned()
                .unwrap_or_else(|| format!("Vendor {}", vendor_invoice.vendor_id)),
            vendor_invoice.invoice_date,
            parse_payment_terms(&vendor_invoice.payment_terms),
            vendor_invoice.header.currency.clone(),
        );

        // Set PO reference if available
        if let Some(po_id) = &vendor_invoice.purchase_order_id {
            ap_invoice.reference_po = Some(po_id.clone());
            ap_invoice.match_status = match vendor_invoice.verification_status {
                datasynth_core::models::documents::InvoiceVerificationStatus::ThreeWayMatchPassed => {
                    MatchStatus::Matched
                }
                datasynth_core::models::documents::InvoiceVerificationStatus::ThreeWayMatchFailed => {
                    // Compute non-zero variance from invoice line items.
                    // When three-way match fails, there is a meaningful price/quantity difference.
                    let total_line_amount: Decimal = vendor_invoice
                        .items
                        .iter()
                        .map(|item| item.base.unit_price * item.base.quantity)
                        .sum();
                    // Price variance: ~2-5% of invoice total
                    let price_var = (total_line_amount * Decimal::new(3, 2)).round_dp(2);
                    // Quantity variance: ~1-3% of invoice total
                    let qty_var = (total_line_amount * Decimal::new(15, 3)).round_dp(2);
                    MatchStatus::MatchedWithVariance {
                        price_variance: price_var,
                        quantity_variance: qty_var,
                    }
                }
                _ => MatchStatus::NotRequired,
            };
        }

        // Add line items
        for (idx, item) in vendor_invoice.items.iter().enumerate() {
            let line = APInvoiceLine::new(
                (idx + 1) as u32,
                item.base.description.clone(),
                item.base.quantity,
                item.base.uom.clone(),
                item.base.unit_price,
                item.base
                    .gl_account
                    .clone()
                    .unwrap_or_else(|| "5000".to_string()),
            )
            .with_tax(
                item.tax_code.clone().unwrap_or_else(|| "VAT".to_string()),
                item.base.tax_amount,
            );

            ap_invoice.add_line(line);
        }

        ap_invoice
    }

    /// Convert a document flow CustomerInvoice to an AR subledger ARInvoice.
    ///
    /// This ensures that customer invoices from the O2C flow create corresponding
    /// AR subledger records for complete data coherence.
    pub fn create_ar_invoice_from_customer_invoice(
        &mut self,
        customer_invoice: &CustomerInvoice,
    ) -> ARInvoice {
        self.ar_counter += 1;

        // Use the document-flow document ID as the AR invoice number so that
        // PaymentAllocation.invoice_id (which references the same document ID)
        // can be matched during settlement.
        let invoice_number = customer_invoice.header.document_id.clone();

        // Create the AR invoice
        let mut ar_invoice = ARInvoice::new(
            invoice_number,
            customer_invoice.header.company_code.clone(),
            customer_invoice.customer_id.clone(),
            self.customer_names
                .get(&customer_invoice.customer_id)
                .cloned()
                .unwrap_or_else(|| format!("Customer {}", customer_invoice.customer_id)),
            customer_invoice.header.document_date,
            parse_payment_terms(&customer_invoice.payment_terms),
            customer_invoice.header.currency.clone(),
        );

        // Add line items
        for (idx, item) in customer_invoice.items.iter().enumerate() {
            let line = ARInvoiceLine::new(
                (idx + 1) as u32,
                item.base.description.clone(),
                item.base.quantity,
                item.base.uom.clone(),
                item.base.unit_price,
                item.revenue_account
                    .clone()
                    .unwrap_or_else(|| "4000".to_string()),
            )
            .with_tax("VAT".to_string(), item.base.tax_amount);

            ar_invoice.add_line(line);
        }

        ar_invoice
    }

    /// Batch convert multiple vendor invoices to AP invoices.
    pub fn batch_create_ap_invoices(
        &mut self,
        vendor_invoices: &[VendorInvoice],
    ) -> Vec<APInvoice> {
        vendor_invoices
            .iter()
            .map(|vi| self.create_ap_invoice_from_vendor_invoice(vi))
            .collect()
    }

    /// Batch convert multiple customer invoices to AR invoices.
    pub fn batch_create_ar_invoices(
        &mut self,
        customer_invoices: &[CustomerInvoice],
    ) -> Vec<ARInvoice> {
        customer_invoices
            .iter()
            .map(|ci| self.create_ar_invoice_from_customer_invoice(ci))
            .collect()
    }
}

/// Reduces `amount_remaining` on AP invoices by the amounts applied in each payment.
///
/// For each `Payment` whose `payment_type` is `ApPayment`, iterates over its
/// `allocations` and matches them to AP invoices by `allocation.invoice_id` ==
/// `ap_invoice.vendor_invoice_number`. `amount_remaining` is clamped to zero so
/// over-payments do not produce negative balances.
pub fn apply_ap_settlements(ap_invoices: &mut [APInvoice], payments: &[Payment]) {
    // Build a lookup: vendor_invoice_number → list of indices in ap_invoices.
    // Uses owned String keys so the map does not hold borrows into the slice,
    // allowing mutable access to elements later.
    let mut index_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, inv) in ap_invoices.iter().enumerate() {
        index_map
            .entry(inv.vendor_invoice_number.clone())
            .or_default()
            .push(idx);
    }

    for payment in payments {
        if payment.payment_type != PaymentType::ApPayment {
            continue;
        }
        for allocation in &payment.allocations {
            if let Some(indices) = index_map.get(&allocation.invoice_id) {
                for &idx in indices {
                    let inv = &mut ap_invoices[idx];
                    inv.amount_remaining =
                        (inv.amount_remaining - allocation.amount).max(Decimal::ZERO);
                    // Update status to reflect settlement state.
                    inv.status = if inv.amount_remaining == Decimal::ZERO {
                        SubledgerDocumentStatus::Cleared
                    } else {
                        SubledgerDocumentStatus::PartiallyCleared
                    };
                }
            }
        }
    }
}

/// Reduces `amount_remaining` on AR invoices by the amounts applied in each receipt.
///
/// For each `Payment` whose `payment_type` is `ArReceipt`, iterates over its
/// `allocations` and matches them to AR invoices by `allocation.invoice_id` ==
/// `ar_invoice.invoice_number`. `amount_remaining` is clamped to zero so
/// over-payments do not produce negative balances.
pub fn apply_ar_settlements(ar_invoices: &mut [ARInvoice], payments: &[Payment]) {
    // Build a lookup: invoice_number → list of indices in ar_invoices.
    // Uses owned String keys so the map does not hold borrows into the slice,
    // allowing mutable access to elements later.
    let mut index_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, inv) in ar_invoices.iter().enumerate() {
        index_map
            .entry(inv.invoice_number.clone())
            .or_default()
            .push(idx);
    }

    for payment in payments {
        if payment.payment_type != PaymentType::ArReceipt {
            continue;
        }
        for allocation in &payment.allocations {
            if let Some(indices) = index_map.get(&allocation.invoice_id) {
                for &idx in indices {
                    let inv = &mut ar_invoices[idx];
                    inv.amount_remaining =
                        (inv.amount_remaining - allocation.amount).max(Decimal::ZERO);
                    // Update status to reflect settlement state.
                    inv.status = if inv.amount_remaining == Decimal::ZERO {
                        SubledgerDocumentStatus::Cleared
                    } else {
                        SubledgerDocumentStatus::PartiallyCleared
                    };
                }
            }
        }
    }
}

/// Parse payment terms string into PaymentTerms struct.
fn parse_payment_terms(terms_str: &str) -> PaymentTerms {
    // Try to parse common payment terms formats
    match terms_str.to_uppercase().as_str() {
        "NET30" | "N30" => PaymentTerms::net_30(),
        "NET60" | "N60" => PaymentTerms::net_60(),
        "NET90" | "N90" => PaymentTerms::net_90(),
        "DUE ON RECEIPT" | "COD" => PaymentTerms::net(0), // Due immediately
        _ => {
            // Default to NET30 if parsing fails
            PaymentTerms::net_30()
        }
    }
}

/// Result of linking document flows to subledgers.
#[derive(Debug, Clone, Default)]
pub struct SubledgerLinkResult {
    /// AP invoices created from vendor invoices.
    pub ap_invoices: Vec<APInvoice>,
    /// AR invoices created from customer invoices.
    pub ar_invoices: Vec<ARInvoice>,
    /// Number of vendor invoices processed.
    pub vendor_invoices_processed: usize,
    /// Number of customer invoices processed.
    pub customer_invoices_processed: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::documents::VendorInvoiceItem;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_ap_invoice_from_vendor_invoice() {
        let mut vendor_invoice = VendorInvoice::new(
            "VI-001",
            "1000",
            "VEND001",
            "V-INV-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "SYSTEM",
        );

        vendor_invoice.add_item(VendorInvoiceItem::new(1, "Test Item", dec!(10), dec!(100)));

        let mut linker = DocumentFlowLinker::new();
        let ap_invoice = linker.create_ap_invoice_from_vendor_invoice(&vendor_invoice);

        assert_eq!(ap_invoice.vendor_id, "VEND001");
        // vendor_invoice_number now stores the document-flow document ID so that
        // PaymentAllocation.invoice_id can be matched during settlement.
        assert_eq!(ap_invoice.vendor_invoice_number, "VI-001");
        assert_eq!(ap_invoice.lines.len(), 1);
        assert!(ap_invoice.gross_amount.document_amount > Decimal::ZERO);
    }

    #[test]
    fn test_create_ar_invoice_from_customer_invoice() {
        use datasynth_core::models::documents::CustomerInvoiceItem;

        let mut customer_invoice = CustomerInvoice::new(
            "CI-001",
            "1000",
            "CUST001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
            "SYSTEM",
        );

        customer_invoice.add_item(CustomerInvoiceItem::new(1, "Product A", dec!(5), dec!(200)));

        let mut linker = DocumentFlowLinker::new();
        let ar_invoice = linker.create_ar_invoice_from_customer_invoice(&customer_invoice);

        assert_eq!(ar_invoice.customer_id, "CUST001");
        assert_eq!(ar_invoice.lines.len(), 1);
        assert!(ar_invoice.gross_amount.document_amount > Decimal::ZERO);
    }

    #[test]
    fn test_batch_conversion() {
        let vendor_invoice = VendorInvoice::new(
            "VI-001",
            "1000",
            "VEND001",
            "V-INV-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "SYSTEM",
        );

        let mut linker = DocumentFlowLinker::new();
        let ap_invoices =
            linker.batch_create_ap_invoices(&[vendor_invoice.clone(), vendor_invoice]);

        assert_eq!(ap_invoices.len(), 2);
        assert_eq!(ap_invoices[0].invoice_number, "APINV00000001");
        assert_eq!(ap_invoices[1].invoice_number, "APINV00000002");
    }

    #[test]
    fn test_parse_payment_terms() {
        let terms = parse_payment_terms("NET30");
        assert_eq!(terms.net_due_days, 30);

        let terms = parse_payment_terms("NET60");
        assert_eq!(terms.net_due_days, 60);

        let terms = parse_payment_terms("DUE ON RECEIPT");
        assert_eq!(terms.net_due_days, 0);

        // Unknown terms default to NET30
        let terms = parse_payment_terms("CUSTOM");
        assert_eq!(terms.net_due_days, 30);
    }
}
