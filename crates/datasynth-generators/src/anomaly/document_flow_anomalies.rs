//! Document flow anomaly injection for 3-way match fraud patterns.
//!
//! This module provides anomaly injection specifically for document flows,
//! simulating common procurement fraud patterns:
//! - Quantity mismatches between PO, GR, and Invoice
//! - Maverick buying (Invoice without PO)
//! - Unbilled goods (GR without Invoice)
//! - Unauthorized disbursements (Payment without Invoice)

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use datasynth_core::models::documents::{GoodsReceipt, Payment, PurchaseOrder, VendorInvoice};
use datasynth_core::{AnomalyType, FraudType, LabeledAnomaly, ProcessIssueType};

/// Types of document flow anomalies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentFlowAnomalyType {
    /// GR quantity doesn't match PO quantity
    QuantityMismatch,
    /// Price on invoice doesn't match PO price
    PriceMismatch,
    /// Invoice received without corresponding PO (maverick buying)
    InvoiceWithoutPO,
    /// Goods received but never invoiced
    GoodsReceivedNotBilled,
    /// Payment issued without valid invoice
    PaymentWithoutInvoice,
    /// Duplicate invoice for same PO
    DuplicateInvoice,
    /// Invoice date before goods receipt
    InvoiceBeforeReceipt,
    /// Payment before invoice approval
    EarlyPayment,
}

/// Result of injecting a document flow anomaly.
#[derive(Debug, Clone)]
pub struct DocumentFlowAnomalyResult {
    /// Type of anomaly injected
    pub anomaly_type: DocumentFlowAnomalyType,
    /// Description of what was modified
    pub description: String,
    /// Original value (if applicable)
    pub original_value: Option<String>,
    /// Modified value (if applicable)
    pub modified_value: Option<String>,
    /// Associated document IDs
    pub document_ids: Vec<String>,
    /// Severity (1-5)
    pub severity: u8,
}

impl DocumentFlowAnomalyResult {
    /// Convert to a labeled anomaly for ML training.
    pub fn to_labeled_anomaly(
        &self,
        anomaly_id: &str,
        document_id: &str,
        company_code: &str,
        date: NaiveDate,
    ) -> LabeledAnomaly {
        // Map document flow anomaly types to existing AnomalyType variants
        let anomaly_type = match self.anomaly_type {
            // Quantity/price mismatches are invoice manipulation fraud
            DocumentFlowAnomalyType::QuantityMismatch => {
                AnomalyType::Fraud(FraudType::InvoiceManipulation)
            }
            DocumentFlowAnomalyType::PriceMismatch => {
                AnomalyType::Fraud(FraudType::InvoiceManipulation)
            }
            // Invoice without PO is a process issue (missing documentation/control bypass)
            DocumentFlowAnomalyType::InvoiceWithoutPO => {
                AnomalyType::ProcessIssue(ProcessIssueType::MissingDocumentation)
            }
            // Goods received but not billed could indicate asset misappropriation
            DocumentFlowAnomalyType::GoodsReceivedNotBilled => {
                AnomalyType::Fraud(FraudType::AssetMisappropriation)
            }
            // Payment without invoice is unauthorized approval
            DocumentFlowAnomalyType::PaymentWithoutInvoice => {
                AnomalyType::Fraud(FraudType::UnauthorizedApproval)
            }
            // Duplicate invoice is duplicate payment fraud
            DocumentFlowAnomalyType::DuplicateInvoice => {
                AnomalyType::Fraud(FraudType::DuplicatePayment)
            }
            // Invoice before receipt is process timing issue
            DocumentFlowAnomalyType::InvoiceBeforeReceipt => {
                AnomalyType::ProcessIssue(ProcessIssueType::MissingDocumentation)
            }
            // Early payment bypasses normal approval
            DocumentFlowAnomalyType::EarlyPayment => {
                AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval)
            }
        };

        LabeledAnomaly::new(
            anomaly_id.to_string(),
            anomaly_type,
            document_id.to_string(),
            "DocumentFlow".to_string(),
            company_code.to_string(),
            date,
        )
        .with_description(&self.description)
    }
}

/// Configuration for document flow anomaly injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFlowAnomalyConfig {
    /// Probability of quantity mismatch (0.0-1.0)
    pub quantity_mismatch_rate: f64,
    /// Probability of price mismatch
    pub price_mismatch_rate: f64,
    /// Probability of invoice without PO
    pub maverick_buying_rate: f64,
    /// Probability of GR without invoice
    pub unbilled_receipt_rate: f64,
    /// Probability of payment without invoice
    pub unauthorized_payment_rate: f64,
    /// Probability of duplicate invoice
    pub duplicate_invoice_rate: f64,
    /// Probability of invoice before receipt
    pub early_invoice_rate: f64,
    /// Probability of early payment
    pub early_payment_rate: f64,
    /// Maximum quantity variance percentage (e.g., 0.2 = 20%)
    pub max_quantity_variance: f64,
    /// Maximum price variance percentage
    pub max_price_variance: f64,
}

impl Default for DocumentFlowAnomalyConfig {
    fn default() -> Self {
        Self {
            quantity_mismatch_rate: 0.02,     // 2% of receipts
            price_mismatch_rate: 0.015,       // 1.5% of invoices
            maverick_buying_rate: 0.01,       // 1% maverick buying
            unbilled_receipt_rate: 0.005,     // 0.5% unbilled
            unauthorized_payment_rate: 0.002, // 0.2% unauthorized
            duplicate_invoice_rate: 0.008,    // 0.8% duplicates
            early_invoice_rate: 0.01,         // 1% early invoices
            early_payment_rate: 0.005,        // 0.5% early payments
            max_quantity_variance: 0.25,      // Up to 25% variance
            max_price_variance: 0.15,         // Up to 15% variance
        }
    }
}

/// Injector for document flow anomalies.
pub struct DocumentFlowAnomalyInjector {
    config: DocumentFlowAnomalyConfig,
    rng: ChaCha8Rng,
    results: Vec<DocumentFlowAnomalyResult>,
}

impl DocumentFlowAnomalyInjector {
    /// Create a new document flow anomaly injector.
    pub fn new(config: DocumentFlowAnomalyConfig, seed: u64) -> Self {
        Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
            results: Vec::new(),
        }
    }

    /// Create with default configuration.
    pub fn with_seed(seed: u64) -> Self {
        Self::new(DocumentFlowAnomalyConfig::default(), seed)
    }

    /// Get the results of anomaly injection.
    pub fn get_results(&self) -> &[DocumentFlowAnomalyResult] {
        &self.results
    }

    /// Clear results.
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    /// Maybe inject a quantity mismatch into a goods receipt.
    ///
    /// Returns true if an anomaly was injected.
    pub fn maybe_inject_quantity_mismatch(
        &mut self,
        gr: &mut GoodsReceipt,
        po: &PurchaseOrder,
    ) -> bool {
        if self.rng.gen::<f64>() >= self.config.quantity_mismatch_rate {
            return false;
        }

        // Find a matching item to modify
        if let Some(gr_item) = gr.items.first_mut() {
            let original_qty = gr_item.base.quantity;

            // Generate variance (either over or under)
            let variance = if self.rng.gen::<bool>() {
                // Over-receipt (more common in fraud)
                Decimal::from_f64_retain(
                    1.0 + self.rng.gen::<f64>() * self.config.max_quantity_variance,
                )
                .unwrap_or(Decimal::ONE)
            } else {
                // Under-receipt
                Decimal::from_f64_retain(
                    1.0 - self.rng.gen::<f64>() * self.config.max_quantity_variance,
                )
                .unwrap_or(Decimal::ONE)
            };

            gr_item.base.quantity = (original_qty * variance).round_dp(2);

            let result = DocumentFlowAnomalyResult {
                anomaly_type: DocumentFlowAnomalyType::QuantityMismatch,
                description: format!(
                    "GR quantity {} doesn't match PO, expected based on PO line",
                    gr_item.base.quantity
                ),
                original_value: Some(original_qty.to_string()),
                modified_value: Some(gr_item.base.quantity.to_string()),
                document_ids: vec![gr.header.document_id.clone(), po.header.document_id.clone()],
                severity: if variance > Decimal::from_f64_retain(1.1).expect("valid f64 to decimal")
                {
                    4
                } else {
                    3
                },
            };

            self.results.push(result);
            true
        } else {
            false
        }
    }

    /// Maybe inject a price mismatch into a vendor invoice.
    ///
    /// Returns true if an anomaly was injected.
    pub fn maybe_inject_price_mismatch(
        &mut self,
        invoice: &mut VendorInvoice,
        po: &PurchaseOrder,
    ) -> bool {
        if self.rng.gen::<f64>() >= self.config.price_mismatch_rate {
            return false;
        }

        // Find a matching item to modify
        if let Some(inv_item) = invoice.items.first_mut() {
            let original_price = inv_item.base.unit_price;

            // Usually invoices are higher than PO (vendor overcharging)
            let variance = if self.rng.gen::<f64>() < 0.8 {
                // 80% chance of overcharge
                Decimal::from_f64_retain(
                    1.0 + self.rng.gen::<f64>() * self.config.max_price_variance,
                )
                .unwrap_or(Decimal::ONE)
            } else {
                // 20% chance of undercharge (rare, could be error)
                Decimal::from_f64_retain(
                    1.0 - self.rng.gen::<f64>() * self.config.max_price_variance * 0.5,
                )
                .unwrap_or(Decimal::ONE)
            };

            inv_item.base.unit_price = (original_price * variance).round_dp(2);

            let result = DocumentFlowAnomalyResult {
                anomaly_type: DocumentFlowAnomalyType::PriceMismatch,
                description: format!(
                    "Invoice price {} doesn't match PO agreed price",
                    inv_item.base.unit_price
                ),
                original_value: Some(original_price.to_string()),
                modified_value: Some(inv_item.base.unit_price.to_string()),
                document_ids: vec![
                    invoice.header.document_id.clone(),
                    po.header.document_id.clone(),
                ],
                severity: if variance > Decimal::from_f64_retain(1.1).expect("valid f64 to decimal")
                {
                    4
                } else {
                    3
                },
            };

            self.results.push(result);
            true
        } else {
            false
        }
    }

    /// Create an invoice without PO reference (maverick buying).
    ///
    /// Removes the PO reference from an invoice to simulate maverick buying.
    pub fn inject_maverick_buying(&mut self, invoice: &mut VendorInvoice) -> bool {
        if self.rng.gen::<f64>() >= self.config.maverick_buying_rate {
            return false;
        }

        // Only inject if there's a PO to remove
        if invoice.purchase_order_id.is_none() {
            return false;
        }

        let original_po = invoice.purchase_order_id.take();

        let result = DocumentFlowAnomalyResult {
            anomaly_type: DocumentFlowAnomalyType::InvoiceWithoutPO,
            description: "Invoice submitted without purchase order (maverick buying)".to_string(),
            original_value: original_po,
            modified_value: None,
            document_ids: vec![invoice.header.document_id.clone()],
            severity: 4, // Significant control bypass
        };

        self.results.push(result);
        true
    }

    /// Mark a goods receipt as having invoice timing anomaly.
    ///
    /// Returns a result indicating invoice came before goods receipt.
    pub fn create_early_invoice_anomaly(
        &mut self,
        invoice: &VendorInvoice,
        gr: &GoodsReceipt,
    ) -> Option<DocumentFlowAnomalyResult> {
        if self.rng.gen::<f64>() >= self.config.early_invoice_rate {
            return None;
        }

        // Check if invoice date is before GR date
        if invoice.invoice_date < gr.header.document_date {
            let result = DocumentFlowAnomalyResult {
                anomaly_type: DocumentFlowAnomalyType::InvoiceBeforeReceipt,
                description: format!(
                    "Invoice dated {} before goods receipt dated {}",
                    invoice.invoice_date, gr.header.document_date
                ),
                original_value: Some(gr.header.document_date.to_string()),
                modified_value: Some(invoice.invoice_date.to_string()),
                document_ids: vec![
                    invoice.header.document_id.clone(),
                    gr.header.document_id.clone(),
                ],
                severity: 3,
            };

            self.results.push(result.clone());
            return Some(result);
        }

        None
    }

    /// Check for potential unauthorized payment (payment without proper invoice).
    pub fn check_unauthorized_payment(
        &mut self,
        payment: &Payment,
        has_valid_invoice: bool,
    ) -> Option<DocumentFlowAnomalyResult> {
        if has_valid_invoice {
            return None;
        }

        if self.rng.gen::<f64>() >= self.config.unauthorized_payment_rate {
            return None;
        }

        let result = DocumentFlowAnomalyResult {
            anomaly_type: DocumentFlowAnomalyType::PaymentWithoutInvoice,
            description: "Payment issued without valid approved invoice".to_string(),
            original_value: None,
            modified_value: None,
            document_ids: vec![payment.header.document_id.clone()],
            severity: 5, // Critical - potential fraud
        };

        self.results.push(result.clone());
        Some(result)
    }

    /// Get statistics about injected anomalies.
    pub fn get_statistics(&self) -> DocumentFlowAnomalyStats {
        let mut stats = DocumentFlowAnomalyStats::default();

        for result in &self.results {
            match result.anomaly_type {
                DocumentFlowAnomalyType::QuantityMismatch => stats.quantity_mismatches += 1,
                DocumentFlowAnomalyType::PriceMismatch => stats.price_mismatches += 1,
                DocumentFlowAnomalyType::InvoiceWithoutPO => stats.maverick_buying += 1,
                DocumentFlowAnomalyType::GoodsReceivedNotBilled => stats.unbilled_receipts += 1,
                DocumentFlowAnomalyType::PaymentWithoutInvoice => stats.unauthorized_payments += 1,
                DocumentFlowAnomalyType::DuplicateInvoice => stats.duplicate_invoices += 1,
                DocumentFlowAnomalyType::InvoiceBeforeReceipt => stats.early_invoices += 1,
                DocumentFlowAnomalyType::EarlyPayment => stats.early_payments += 1,
            }
        }

        stats.total = self.results.len();
        stats
    }
}

/// Statistics about document flow anomalies.
#[derive(Debug, Clone, Default)]
pub struct DocumentFlowAnomalyStats {
    pub total: usize,
    pub quantity_mismatches: usize,
    pub price_mismatches: usize,
    pub maverick_buying: usize,
    pub unbilled_receipts: usize,
    pub unauthorized_payments: usize,
    pub duplicate_invoices: usize,
    pub early_invoices: usize,
    pub early_payments: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::documents::{
        GoodsReceiptItem, PurchaseOrderItem, VendorInvoiceItem,
    };
    use rust_decimal_macros::dec;

    fn create_test_po() -> PurchaseOrder {
        let mut po = PurchaseOrder::new(
            "PO-001",
            "1000",
            "VEND001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "USER001",
        );
        po.add_item(PurchaseOrderItem::new(
            1,
            "Test Item",
            dec!(100),
            dec!(10.00),
        ));
        po
    }

    fn create_test_gr(_po_id: &str) -> GoodsReceipt {
        let mut gr = GoodsReceipt::new(
            "GR-001",
            "1000",
            "PLANT01",
            "STOR01",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            "USER001",
        );
        gr.add_item(GoodsReceiptItem::new(
            1,
            "Test Item",
            dec!(100),
            dec!(10.00),
        ));
        gr
    }

    fn create_test_invoice(po_id: Option<&str>) -> VendorInvoice {
        let mut inv = VendorInvoice::new(
            "VI-001",
            "1000",
            "VEND001",
            "INV-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
            "USER001",
        );
        inv.purchase_order_id = po_id.map(|s| s.to_string());
        inv.add_item(VendorInvoiceItem::new(
            1,
            "Test Item",
            dec!(100),
            dec!(10.00),
        ));
        inv
    }

    #[test]
    fn test_quantity_mismatch_injection() {
        // Use high rate to ensure injection
        let config = DocumentFlowAnomalyConfig {
            quantity_mismatch_rate: 1.0, // Always inject
            ..Default::default()
        };

        let mut injector = DocumentFlowAnomalyInjector::new(config, 42);
        let po = create_test_po();
        let mut gr = create_test_gr(&po.header.document_id);

        let original_qty = gr.items[0].base.quantity;
        let injected = injector.maybe_inject_quantity_mismatch(&mut gr, &po);

        assert!(injected);
        assert_ne!(gr.items[0].base.quantity, original_qty);
        assert_eq!(injector.get_results().len(), 1);
        assert_eq!(
            injector.get_results()[0].anomaly_type,
            DocumentFlowAnomalyType::QuantityMismatch
        );
    }

    #[test]
    fn test_maverick_buying_injection() {
        let config = DocumentFlowAnomalyConfig {
            maverick_buying_rate: 1.0, // Always inject
            ..Default::default()
        };

        let mut injector = DocumentFlowAnomalyInjector::new(config, 42);
        let mut invoice = create_test_invoice(Some("PO-001"));

        assert!(invoice.purchase_order_id.is_some());
        let injected = injector.inject_maverick_buying(&mut invoice);

        assert!(injected);
        assert!(invoice.purchase_order_id.is_none());
        assert_eq!(
            injector.get_results()[0].anomaly_type,
            DocumentFlowAnomalyType::InvoiceWithoutPO
        );
    }

    #[test]
    fn test_statistics() {
        let config = DocumentFlowAnomalyConfig {
            quantity_mismatch_rate: 1.0,
            maverick_buying_rate: 1.0,
            ..Default::default()
        };

        let mut injector = DocumentFlowAnomalyInjector::new(config, 42);

        // Inject quantity mismatch
        let po = create_test_po();
        let mut gr = create_test_gr(&po.header.document_id);
        injector.maybe_inject_quantity_mismatch(&mut gr, &po);

        // Inject maverick buying
        let mut invoice = create_test_invoice(Some("PO-001"));
        injector.inject_maverick_buying(&mut invoice);

        let stats = injector.get_statistics();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.quantity_mismatches, 1);
        assert_eq!(stats.maverick_buying, 1);
    }

    #[test]
    fn test_labeled_anomaly_conversion() {
        let result = DocumentFlowAnomalyResult {
            anomaly_type: DocumentFlowAnomalyType::QuantityMismatch,
            description: "Test mismatch".to_string(),
            original_value: Some("100".to_string()),
            modified_value: Some("120".to_string()),
            document_ids: vec!["DOC-001".to_string()],
            severity: 3,
        };

        let labeled = result.to_labeled_anomaly(
            "ANO-001",
            "DOC-001",
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        assert_eq!(labeled.document_id, "DOC-001");
        assert_eq!(labeled.company_code, "1000");
        // QuantityMismatch maps to InvoiceManipulation
        assert!(matches!(
            labeled.anomaly_type,
            AnomalyType::Fraud(FraudType::InvoiceManipulation)
        ));
    }
}
