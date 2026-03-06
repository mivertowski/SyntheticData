//! Three-way match validation for P2P document flows.
//!
//! This module implements proper validation of Purchase Order, Goods Receipt,
//! and Vendor Invoice matching according to standard AP practices.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use datasynth_core::models::documents::{GoodsReceipt, PurchaseOrder, VendorInvoice};

/// Configuration for three-way match validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeWayMatchConfig {
    /// Tolerance for price variance (as decimal percentage, e.g., 0.05 = 5%)
    pub price_tolerance: Decimal,
    /// Tolerance for quantity variance (as decimal percentage, e.g., 0.02 = 2%)
    pub quantity_tolerance: Decimal,
    /// Absolute tolerance for small amounts (to handle rounding)
    pub absolute_amount_tolerance: Decimal,
    /// Whether to allow over-delivery (GR quantity > PO quantity)
    pub allow_over_delivery: bool,
    /// Maximum over-delivery percentage allowed
    pub max_over_delivery_pct: Decimal,
}

impl Default for ThreeWayMatchConfig {
    fn default() -> Self {
        Self {
            price_tolerance: dec!(0.05),           // 5% price variance allowed
            quantity_tolerance: dec!(0.02),        // 2% quantity variance allowed
            absolute_amount_tolerance: dec!(0.01), // $0.01 absolute tolerance
            allow_over_delivery: true,
            max_over_delivery_pct: dec!(0.10), // 10% over-delivery allowed
        }
    }
}

/// Result of three-way match validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeWayMatchResult {
    /// Overall match status
    pub passed: bool,
    /// Quantity match status
    pub quantity_matched: bool,
    /// Price match status
    pub price_matched: bool,
    /// Total amount match status
    pub amount_matched: bool,
    /// List of variances found
    pub variances: Vec<MatchVariance>,
    /// Summary message
    pub message: String,
}

impl ThreeWayMatchResult {
    /// Create a successful match result.
    pub fn success() -> Self {
        Self {
            passed: true,
            quantity_matched: true,
            price_matched: true,
            amount_matched: true,
            variances: Vec::new(),
            message: "Three-way match passed".to_string(),
        }
    }

    /// Create a failed match result with message.
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            passed: false,
            quantity_matched: false,
            price_matched: false,
            amount_matched: false,
            variances: Vec::new(),
            message: message.into(),
        }
    }

    /// Add a variance to the result.
    pub fn with_variance(mut self, variance: MatchVariance) -> Self {
        self.variances.push(variance);
        self
    }
}

/// A specific variance found during three-way match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchVariance {
    /// Line number/item affected
    pub line_number: u16,
    /// Type of variance
    pub variance_type: VarianceType,
    /// Expected value
    pub expected: Decimal,
    /// Actual value
    pub actual: Decimal,
    /// Variance amount
    pub variance: Decimal,
    /// Variance percentage
    pub variance_pct: Decimal,
    /// Description
    pub description: String,
}

/// Type of variance in three-way match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VarianceType {
    /// Quantity variance between PO and GR
    QuantityPoGr,
    /// Quantity variance between GR and Invoice
    QuantityGrInvoice,
    /// Price variance between PO and Invoice
    PricePoInvoice,
    /// Total amount variance
    TotalAmount,
    /// Missing line item
    MissingLine,
    /// Extra line item
    ExtraLine,
}

/// Three-way match validator.
pub struct ThreeWayMatcher {
    config: ThreeWayMatchConfig,
}

impl ThreeWayMatcher {
    /// Create a new three-way matcher with default configuration.
    pub fn new() -> Self {
        Self {
            config: ThreeWayMatchConfig::default(),
        }
    }

    /// Create a three-way matcher with custom configuration.
    pub fn with_config(config: ThreeWayMatchConfig) -> Self {
        Self { config }
    }

    /// Validate three-way match between PO, GR, and Invoice.
    ///
    /// # Arguments
    ///
    /// * `po` - The purchase order
    /// * `grs` - The goods receipts (may be multiple for partial deliveries)
    /// * `invoice` - The vendor invoice
    ///
    /// # Returns
    ///
    /// `ThreeWayMatchResult` indicating whether the match passed and any variances found.
    pub fn validate(
        &self,
        po: &PurchaseOrder,
        grs: &[&GoodsReceipt],
        invoice: &VendorInvoice,
    ) -> ThreeWayMatchResult {
        let mut result = ThreeWayMatchResult::success();
        let mut all_quantity_matched = true;
        let mut all_price_matched = true;
        let mut all_amount_matched = true;

        // Aggregate GR quantities by PO line
        let mut gr_quantities: std::collections::HashMap<u16, Decimal> =
            std::collections::HashMap::new();
        for gr in grs {
            for item in &gr.items {
                if let Some(po_line) = item.po_item {
                    *gr_quantities.entry(po_line).or_insert(Decimal::ZERO) += item.base.quantity;
                }
            }
        }

        // Validate each PO line
        for po_item in &po.items {
            let po_line = po_item.base.line_number;
            let po_qty = po_item.base.quantity;
            let po_price = po_item.base.unit_price;

            // Check GR quantity vs PO quantity
            let gr_qty = gr_quantities
                .get(&po_line)
                .copied()
                .unwrap_or(Decimal::ZERO);
            let qty_variance = gr_qty - po_qty;
            let qty_variance_pct = if po_qty > Decimal::ZERO {
                (qty_variance.abs() / po_qty) * dec!(100)
            } else {
                Decimal::ZERO
            };

            // Check under-delivery
            if qty_variance < Decimal::ZERO
                && qty_variance_pct > self.config.quantity_tolerance * dec!(100)
            {
                all_quantity_matched = false;
                result = result.with_variance(MatchVariance {
                    line_number: po_line,
                    variance_type: VarianceType::QuantityPoGr,
                    expected: po_qty,
                    actual: gr_qty,
                    variance: qty_variance,
                    variance_pct: qty_variance_pct,
                    description: format!("Under-delivery: received {gr_qty} vs ordered {po_qty}"),
                });
            }

            // Check over-delivery
            if qty_variance > Decimal::ZERO
                && (!self.config.allow_over_delivery
                    || qty_variance_pct > self.config.max_over_delivery_pct * dec!(100))
            {
                all_quantity_matched = false;
                result = result.with_variance(MatchVariance {
                    line_number: po_line,
                    variance_type: VarianceType::QuantityPoGr,
                    expected: po_qty,
                    actual: gr_qty,
                    variance: qty_variance,
                    variance_pct: qty_variance_pct,
                    description: format!("Over-delivery: received {gr_qty} vs ordered {po_qty}"),
                });
            }

            // Find matching invoice line
            let invoice_item = invoice.items.iter().find(|i| i.po_item == Some(po_line));

            if let Some(inv_item) = invoice_item {
                // Check price variance
                let price_variance = inv_item.base.unit_price - po_price;
                let price_variance_pct = if po_price > Decimal::ZERO {
                    (price_variance.abs() / po_price) * dec!(100)
                } else {
                    Decimal::ZERO
                };

                if price_variance_pct > self.config.price_tolerance * dec!(100)
                    && price_variance.abs() > self.config.absolute_amount_tolerance
                {
                    all_price_matched = false;
                    result = result.with_variance(MatchVariance {
                        line_number: po_line,
                        variance_type: VarianceType::PricePoInvoice,
                        expected: po_price,
                        actual: inv_item.base.unit_price,
                        variance: price_variance,
                        variance_pct: price_variance_pct,
                        description: format!(
                            "Price variance: invoiced {} vs PO price {}",
                            inv_item.base.unit_price, po_price
                        ),
                    });
                }

                // Check quantity on invoice vs GR
                let inv_qty = inv_item.invoiced_quantity;
                let inv_gr_variance = inv_qty - gr_qty;
                let inv_gr_variance_pct = if gr_qty > Decimal::ZERO {
                    (inv_gr_variance.abs() / gr_qty) * dec!(100)
                } else {
                    Decimal::ZERO
                };

                if inv_gr_variance_pct > self.config.quantity_tolerance * dec!(100)
                    && inv_gr_variance.abs() > self.config.absolute_amount_tolerance
                {
                    all_quantity_matched = false;
                    result = result.with_variance(MatchVariance {
                        line_number: po_line,
                        variance_type: VarianceType::QuantityGrInvoice,
                        expected: gr_qty,
                        actual: inv_qty,
                        variance: inv_gr_variance,
                        variance_pct: inv_gr_variance_pct,
                        description: format!("Invoice qty {inv_qty} doesn't match GR qty {gr_qty}"),
                    });
                }
            } else {
                // Missing invoice line for this PO line
                result = result.with_variance(MatchVariance {
                    line_number: po_line,
                    variance_type: VarianceType::MissingLine,
                    expected: po_qty,
                    actual: Decimal::ZERO,
                    variance: po_qty,
                    variance_pct: dec!(100),
                    description: format!("PO line {po_line} not found on invoice"),
                });
                all_amount_matched = false;
            }
        }

        // Check total amounts
        let po_total = po.total_net_amount;
        let invoice_total = invoice.net_amount;
        let total_variance = invoice_total - po_total;
        let total_variance_pct = if po_total > Decimal::ZERO {
            (total_variance.abs() / po_total) * dec!(100)
        } else {
            Decimal::ZERO
        };

        if total_variance.abs() > self.config.absolute_amount_tolerance
            && total_variance_pct > self.config.price_tolerance * dec!(100)
        {
            all_amount_matched = false;
            result = result.with_variance(MatchVariance {
                line_number: 0,
                variance_type: VarianceType::TotalAmount,
                expected: po_total,
                actual: invoice_total,
                variance: total_variance,
                variance_pct: total_variance_pct,
                description: format!(
                    "Total amount variance: invoice {invoice_total} vs PO {po_total}"
                ),
            });
        }

        // Update result status
        result.quantity_matched = all_quantity_matched;
        result.price_matched = all_price_matched;
        result.amount_matched = all_amount_matched;
        result.passed = all_quantity_matched && all_price_matched && all_amount_matched;

        if !result.passed {
            let issues = result.variances.len();
            result.message = format!("Three-way match failed with {issues} variance(s)");
        }

        result
    }

    /// Quick check if quantities match between PO and GRs.
    pub fn check_quantities(&self, po: &PurchaseOrder, grs: &[&GoodsReceipt]) -> bool {
        // Aggregate GR quantities by PO line
        let mut gr_quantities: std::collections::HashMap<u16, Decimal> =
            std::collections::HashMap::new();
        for gr in grs {
            for item in &gr.items {
                if let Some(po_line) = item.po_item {
                    *gr_quantities.entry(po_line).or_insert(Decimal::ZERO) += item.base.quantity;
                }
            }
        }

        // Check each PO line
        for po_item in &po.items {
            let po_qty = po_item.base.quantity;
            let gr_qty = gr_quantities
                .get(&po_item.base.line_number)
                .copied()
                .unwrap_or(Decimal::ZERO);

            let variance_pct = if po_qty > Decimal::ZERO {
                ((gr_qty - po_qty).abs() / po_qty) * dec!(100)
            } else {
                Decimal::ZERO
            };

            if variance_pct > self.config.quantity_tolerance * dec!(100) {
                return false;
            }
        }

        true
    }
}

impl Default for ThreeWayMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::documents::{
        GoodsReceiptItem, MovementType, PurchaseOrderItem, VendorInvoiceItem,
    };

    fn create_test_po() -> PurchaseOrder {
        let mut po = PurchaseOrder::new(
            "PO-001".to_string(),
            "1000",
            "V-001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "JSMITH",
        );

        let item1 = PurchaseOrderItem::new(
            10,
            "Material A",
            Decimal::from(100),
            Decimal::from(50), // $50/unit
        );

        let item2 = PurchaseOrderItem::new(
            20,
            "Material B",
            Decimal::from(200),
            Decimal::from(25), // $25/unit
        );

        po.add_item(item1);
        po.add_item(item2);
        po
    }

    fn create_matching_gr(po: &PurchaseOrder) -> GoodsReceipt {
        let mut gr = GoodsReceipt::from_purchase_order(
            "GR-001".to_string(),
            "1000",
            &po.header.document_id,
            "V-001",
            "P1000",
            "0001",
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
            "JSMITH",
        );

        // Match PO quantities exactly
        for po_item in &po.items {
            let item = GoodsReceiptItem::from_po(
                po_item.base.line_number,
                &po_item.base.description,
                po_item.base.quantity,
                po_item.base.unit_price,
                &po.header.document_id,
                po_item.base.line_number,
            )
            .with_movement_type(MovementType::GrForPo);

            gr.add_item(item);
        }

        gr
    }

    fn create_matching_invoice(po: &PurchaseOrder, gr: &GoodsReceipt) -> VendorInvoice {
        let mut invoice = VendorInvoice::new(
            "VI-001".to_string(),
            "1000",
            "V-001",
            "INV-001".to_string(),
            2024,
            1,
            NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
            "JSMITH",
        );

        // Match PO/GR exactly
        for po_item in &po.items {
            let item = VendorInvoiceItem::from_po_gr(
                po_item.base.line_number,
                &po_item.base.description,
                po_item.base.quantity,
                po_item.base.unit_price,
                &po.header.document_id,
                po_item.base.line_number,
                Some(gr.header.document_id.clone()),
                Some(po_item.base.line_number),
            );

            invoice.add_item(item);
        }

        invoice
    }

    #[test]
    fn test_perfect_match() {
        let po = create_test_po();
        let gr = create_matching_gr(&po);
        let invoice = create_matching_invoice(&po, &gr);

        let matcher = ThreeWayMatcher::new();
        let result = matcher.validate(&po, &[&gr], &invoice);

        assert!(result.passed, "Perfect match should pass");
        assert!(result.variances.is_empty(), "Should have no variances");
    }

    #[test]
    fn test_price_variance() {
        let po = create_test_po();
        let gr = create_matching_gr(&po);
        let mut invoice = create_matching_invoice(&po, &gr);

        // Increase invoice price by 10%
        for item in &mut invoice.items {
            item.base.unit_price *= dec!(1.10);
        }
        invoice.recalculate_totals();

        let matcher = ThreeWayMatcher::new();
        let result = matcher.validate(&po, &[&gr], &invoice);

        assert!(!result.passed, "Price variance should fail");
        assert!(!result.price_matched, "Price should not match");
        assert!(
            result
                .variances
                .iter()
                .any(|v| v.variance_type == VarianceType::PricePoInvoice),
            "Should have price variance"
        );
    }

    #[test]
    fn test_quantity_under_delivery() {
        let po = create_test_po();
        let mut gr = create_matching_gr(&po);

        // Reduce GR quantity by 20%
        for item in &mut gr.items {
            item.base.quantity *= dec!(0.80);
        }

        let invoice = create_matching_invoice(&po, &gr);

        let matcher = ThreeWayMatcher::new();
        let result = matcher.validate(&po, &[&gr], &invoice);

        assert!(!result.passed, "Under-delivery should fail");
        assert!(!result.quantity_matched, "Quantity should not match");
    }

    #[test]
    fn test_small_variance_within_tolerance() {
        let po = create_test_po();
        let gr = create_matching_gr(&po);
        let mut invoice = create_matching_invoice(&po, &gr);

        // Increase invoice price by 1% (within 5% tolerance)
        for item in &mut invoice.items {
            item.base.unit_price *= dec!(1.01);
        }
        invoice.recalculate_totals();

        let matcher = ThreeWayMatcher::new();
        let result = matcher.validate(&po, &[&gr], &invoice);

        assert!(result.passed, "Small variance within tolerance should pass");
    }
}
