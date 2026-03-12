//! Procure-to-Pay (P2P) flow generator.
//!
//! Generates complete P2P document chains:
//! PurchaseOrder → GoodsReceipt → VendorInvoice → Payment

use chrono::{Datelike, NaiveDate};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use datasynth_core::models::{
    documents::{
        DocumentReference, DocumentType, GoodsReceipt, GoodsReceiptItem, MovementType, Payment,
        PaymentMethod, PurchaseOrder, PurchaseOrderItem, ReferenceType, VendorInvoice,
        VendorInvoiceItem,
    },
    Material, MaterialPool, PaymentTerms, Vendor, VendorPool,
};
use datasynth_core::CountryPack;

use super::three_way_match::ThreeWayMatcher;

/// Configuration for P2P flow generation.
#[derive(Debug, Clone)]
pub struct P2PGeneratorConfig {
    /// Three-way match success rate (PO-GR-Invoice match)
    pub three_way_match_rate: f64,
    /// Rate of partial deliveries
    pub partial_delivery_rate: f64,
    /// Rate of over-delivery (quantity exceeds PO)
    pub over_delivery_rate: f64,
    /// Rate of price variance (invoice price differs from PO)
    pub price_variance_rate: f64,
    /// Max price variance percentage
    pub max_price_variance_percent: f64,
    /// Average days between PO and GR
    pub avg_days_po_to_gr: u32,
    /// Average days between GR and Invoice
    pub avg_days_gr_to_invoice: u32,
    /// Average days between Invoice and Payment
    pub avg_days_invoice_to_payment: u32,
    /// Payment method distribution
    pub payment_method_distribution: Vec<(PaymentMethod, f64)>,
    /// Probability of early payment discount being taken
    pub early_payment_discount_rate: f64,
    /// Payment behavior configuration
    pub payment_behavior: P2PPaymentBehavior,
}

/// Payment behavior configuration for P2P.
#[derive(Debug, Clone)]
pub struct P2PPaymentBehavior {
    /// Rate of late payments (beyond due date)
    pub late_payment_rate: f64,
    /// Distribution of late payment days
    pub late_payment_distribution: LatePaymentDistribution,
    /// Rate of partial payments
    pub partial_payment_rate: f64,
    /// Rate of payment corrections
    pub payment_correction_rate: f64,
    /// Average days until partial payment remainder is paid
    pub avg_days_until_remainder: u32,
}

impl Default for P2PPaymentBehavior {
    fn default() -> Self {
        Self {
            late_payment_rate: 0.15,
            late_payment_distribution: LatePaymentDistribution::default(),
            partial_payment_rate: 0.05,
            payment_correction_rate: 0.02,
            avg_days_until_remainder: 30,
        }
    }
}

/// Distribution of late payment days.
#[derive(Debug, Clone)]
pub struct LatePaymentDistribution {
    /// 1-7 days late
    pub slightly_late_1_to_7: f64,
    /// 8-14 days late
    pub late_8_to_14: f64,
    /// 15-30 days late
    pub very_late_15_to_30: f64,
    /// 31-60 days late
    pub severely_late_31_to_60: f64,
    /// Over 60 days late
    pub extremely_late_over_60: f64,
}

impl Default for LatePaymentDistribution {
    fn default() -> Self {
        Self {
            slightly_late_1_to_7: 0.50,
            late_8_to_14: 0.25,
            very_late_15_to_30: 0.15,
            severely_late_31_to_60: 0.07,
            extremely_late_over_60: 0.03,
        }
    }
}

impl Default for P2PGeneratorConfig {
    fn default() -> Self {
        Self {
            three_way_match_rate: 0.95,
            partial_delivery_rate: 0.10,
            over_delivery_rate: 0.02,
            price_variance_rate: 0.05,
            max_price_variance_percent: 0.05,
            avg_days_po_to_gr: 7,
            avg_days_gr_to_invoice: 5,
            avg_days_invoice_to_payment: 30,
            payment_method_distribution: vec![
                (PaymentMethod::BankTransfer, 0.60),
                (PaymentMethod::Check, 0.25),
                (PaymentMethod::Wire, 0.10),
                (PaymentMethod::CreditCard, 0.05),
            ],
            early_payment_discount_rate: 0.30,
            payment_behavior: P2PPaymentBehavior::default(),
        }
    }
}

/// A complete P2P document chain.
#[derive(Debug, Clone)]
pub struct P2PDocumentChain {
    /// Purchase Order
    pub purchase_order: PurchaseOrder,
    /// Goods Receipts (may be multiple for partial deliveries)
    pub goods_receipts: Vec<GoodsReceipt>,
    /// Vendor Invoice
    pub vendor_invoice: Option<VendorInvoice>,
    /// Payment
    pub payment: Option<Payment>,
    /// Remainder payments (follow-up to partial payments)
    pub remainder_payments: Vec<Payment>,
    /// Chain completion status
    pub is_complete: bool,
    /// Three-way match status
    pub three_way_match_passed: bool,
    /// Payment timing information
    pub payment_timing: Option<PaymentTimingInfo>,
}

/// Information about payment timing.
#[derive(Debug, Clone)]
pub struct PaymentTimingInfo {
    /// Invoice due date
    pub due_date: NaiveDate,
    /// Actual payment date
    pub payment_date: NaiveDate,
    /// Days late (0 if on time or early)
    pub days_late: i32,
    /// Whether payment was late
    pub is_late: bool,
    /// Whether early payment discount was taken
    pub discount_taken: bool,
}

/// Generator for P2P document flows.
pub struct P2PGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: P2PGeneratorConfig,
    po_counter: usize,
    gr_counter: usize,
    vi_counter: usize,
    pay_counter: usize,
    three_way_matcher: ThreeWayMatcher,
    country_pack: Option<CountryPack>,
}

impl P2PGenerator {
    /// Create a new P2P generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, P2PGeneratorConfig::default())
    }

    /// Create a new P2P generator with custom configuration.
    pub fn with_config(seed: u64, config: P2PGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            seed,
            config,
            po_counter: 0,
            gr_counter: 0,
            vi_counter: 0,
            pay_counter: 0,
            three_way_matcher: ThreeWayMatcher::new(),
            country_pack: None,
        }
    }

    /// Set the country pack for locale-aware document texts.
    pub fn set_country_pack(&mut self, pack: CountryPack) {
        self.country_pack = Some(pack);
    }

    /// Build a document ID, preferring the country pack `reference_prefix` when set.
    fn make_doc_id(
        &self,
        default_prefix: &str,
        pack_key: &str,
        company_code: &str,
        counter: usize,
    ) -> String {
        let prefix = self
            .country_pack
            .as_ref()
            .map(|p| {
                let grp = match pack_key {
                    "purchase_order" => &p.document_texts.purchase_order,
                    "goods_receipt" => &p.document_texts.goods_receipt,
                    "vendor_invoice" => &p.document_texts.vendor_invoice,
                    "payment" => &p.document_texts.payment,
                    _ => return default_prefix.to_string(),
                };
                if grp.reference_prefix.is_empty() {
                    default_prefix.to_string()
                } else {
                    grp.reference_prefix.clone()
                }
            })
            .unwrap_or_else(|| default_prefix.to_string());
        format!("{prefix}-{company_code}-{counter:010}")
    }

    /// Pick a random line description from the country pack for the given
    /// document type, falling back to the provided default.
    fn pick_line_description(&mut self, pack_key: &str, default: &str) -> String {
        if let Some(pack) = &self.country_pack {
            let descriptions = match pack_key {
                "purchase_order" => &pack.document_texts.purchase_order.line_descriptions,
                "goods_receipt" => &pack.document_texts.goods_receipt.line_descriptions,
                "vendor_invoice" => &pack.document_texts.vendor_invoice.line_descriptions,
                "payment" => &pack.document_texts.payment.line_descriptions,
                _ => return default.to_string(),
            };
            if !descriptions.is_empty() {
                let idx = self.rng.random_range(0..descriptions.len());
                return descriptions[idx].clone();
            }
        }
        default.to_string()
    }

    /// Generate a complete P2P chain.
    pub fn generate_chain(
        &mut self,
        company_code: &str,
        vendor: &Vendor,
        materials: &[&Material],
        po_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> P2PDocumentChain {
        // Generate PO
        let po = self.generate_purchase_order(
            company_code,
            vendor,
            materials,
            po_date,
            fiscal_year,
            fiscal_period,
            created_by,
        );

        // Calculate GR date
        let gr_date = self.calculate_gr_date(po_date);
        let gr_fiscal_period = self.get_fiscal_period(gr_date);

        // Generate GR(s)
        let goods_receipts = self.generate_goods_receipts(
            &po,
            company_code,
            gr_date,
            fiscal_year,
            gr_fiscal_period,
            created_by,
        );

        // Calculate invoice date
        let invoice_date = self.calculate_invoice_date(gr_date);
        let invoice_fiscal_period = self.get_fiscal_period(invoice_date);

        // Determine if we should introduce variances based on configuration
        // This simulates real-world scenarios where not all invoices match perfectly
        let should_have_variance = self.rng.random::<f64>() >= self.config.three_way_match_rate;

        // Generate invoice (may introduce variances based on config)
        let vendor_invoice = self.generate_vendor_invoice(
            &po,
            &goods_receipts,
            company_code,
            vendor,
            invoice_date,
            fiscal_year,
            invoice_fiscal_period,
            created_by,
            !should_have_variance, // Pass whether this should be a clean match
        );

        // Perform actual three-way match validation
        let three_way_match_passed = if let Some(ref invoice) = vendor_invoice {
            let gr_refs: Vec<&GoodsReceipt> = goods_receipts.iter().collect();
            let match_result = self.three_way_matcher.validate(&po, &gr_refs, invoice);
            match_result.passed
        } else {
            false
        };

        // Calculate payment date based on payment terms
        let payment_date = self.calculate_payment_date(invoice_date, &vendor.payment_terms);
        let payment_fiscal_period = self.get_fiscal_period(payment_date);

        // Calculate due date for timing info
        let due_date = self.calculate_due_date(invoice_date, &vendor.payment_terms);

        // Determine if this is a partial payment
        let is_partial_payment =
            self.rng.random::<f64>() < self.config.payment_behavior.partial_payment_rate;

        // Generate payment (possibly partial)
        let (payment, remainder_payments) = if let Some(ref invoice) = vendor_invoice {
            if is_partial_payment {
                // Partial payment: 50-75% of invoice amount
                let partial_pct = 0.50 + self.rng.random::<f64>() * 0.25;
                let partial_amount = (invoice.payable_amount
                    * Decimal::from_f64_retain(partial_pct).unwrap_or(Decimal::ONE))
                .round_dp(2);

                let initial_payment = self.generate_payment_for_amount(
                    invoice,
                    company_code,
                    vendor,
                    payment_date,
                    fiscal_year,
                    payment_fiscal_period,
                    created_by,
                    partial_amount,
                );

                // Generate remainder payment
                let remainder_amount = invoice.payable_amount - partial_amount;
                let remainder_days_variance = self.rng.random_range(0..10) as i64;
                let remainder_date = payment_date
                    + chrono::Duration::days(
                        self.config.payment_behavior.avg_days_until_remainder as i64
                            + remainder_days_variance,
                    );
                let remainder_fiscal_period = self.get_fiscal_period(remainder_date);

                let remainder_payment = self.generate_remainder_payment(
                    invoice,
                    company_code,
                    vendor,
                    remainder_date,
                    fiscal_year,
                    remainder_fiscal_period,
                    created_by,
                    remainder_amount,
                    &initial_payment,
                );

                (Some(initial_payment), vec![remainder_payment])
            } else {
                // Full payment
                let full_payment = self.generate_payment(
                    invoice,
                    company_code,
                    vendor,
                    payment_date,
                    fiscal_year,
                    payment_fiscal_period,
                    created_by,
                );
                (Some(full_payment), Vec::new())
            }
        } else {
            (None, Vec::new())
        };

        let is_complete = payment.is_some();

        // Calculate payment timing information
        let payment_timing = if payment.is_some() {
            let days_diff = (payment_date - due_date).num_days() as i32;
            let is_late = days_diff > 0;
            let discount_taken = payment
                .as_ref()
                .map(|p| {
                    p.allocations
                        .iter()
                        .any(|a| a.discount_taken > Decimal::ZERO)
                })
                .unwrap_or(false);

            Some(PaymentTimingInfo {
                due_date,
                payment_date,
                days_late: days_diff.max(0),
                is_late,
                discount_taken,
            })
        } else {
            None
        };

        P2PDocumentChain {
            purchase_order: po,
            goods_receipts,
            vendor_invoice,
            payment,
            remainder_payments,
            is_complete,
            three_way_match_passed,
            payment_timing,
        }
    }

    /// Generate a purchase order.
    pub fn generate_purchase_order(
        &mut self,
        company_code: &str,
        vendor: &Vendor,
        materials: &[&Material],
        po_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> PurchaseOrder {
        self.po_counter += 1;

        let po_id = self.make_doc_id("PO", "purchase_order", company_code, self.po_counter);

        let mut po = PurchaseOrder::new(
            po_id,
            company_code,
            &vendor.vendor_id,
            fiscal_year,
            fiscal_period,
            po_date,
            created_by,
        )
        .with_payment_terms(vendor.payment_terms.code());

        // Denormalize vendor name (DS-011)
        po.vendor_name = Some(vendor.name.clone());

        // Add line items
        for (idx, material) in materials.iter().enumerate() {
            let quantity = Decimal::from(self.rng.random_range(1..100));
            let unit_price = material.standard_cost;

            let description = self.pick_line_description("purchase_order", &material.description);
            let item =
                PurchaseOrderItem::new((idx + 1) as u16 * 10, &description, quantity, unit_price)
                    .with_material(&material.material_id);

            po.add_item(item);
        }

        // Release the PO
        po.release(created_by);

        po
    }

    /// Generate goods receipt(s) for a PO.
    fn generate_goods_receipts(
        &mut self,
        po: &PurchaseOrder,
        company_code: &str,
        gr_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> Vec<GoodsReceipt> {
        let mut receipts = Vec::new();

        // Determine if partial delivery
        let is_partial = self.rng.random::<f64>() < self.config.partial_delivery_rate;

        if is_partial {
            // First partial delivery (60-80% of quantity)
            let first_pct = 0.6 + self.rng.random::<f64>() * 0.2;
            let gr1 = self.create_goods_receipt(
                po,
                company_code,
                gr_date,
                fiscal_year,
                fiscal_period,
                created_by,
                first_pct,
            );
            receipts.push(gr1);

            // Second delivery (remaining quantity)
            let second_date = gr_date + chrono::Duration::days(self.rng.random_range(3..10) as i64);
            let second_period = self.get_fiscal_period(second_date);
            let gr2 = self.create_goods_receipt(
                po,
                company_code,
                second_date,
                fiscal_year,
                second_period,
                created_by,
                1.0 - first_pct,
            );
            receipts.push(gr2);
        } else {
            // Full delivery
            let delivery_pct = if self.rng.random::<f64>() < self.config.over_delivery_rate {
                1.0 + self.rng.random::<f64>() * 0.1 // Up to 10% over
            } else {
                1.0
            };

            let gr = self.create_goods_receipt(
                po,
                company_code,
                gr_date,
                fiscal_year,
                fiscal_period,
                created_by,
                delivery_pct,
            );
            receipts.push(gr);
        }

        receipts
    }

    /// Create a single goods receipt.
    fn create_goods_receipt(
        &mut self,
        po: &PurchaseOrder,
        company_code: &str,
        gr_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
        quantity_pct: f64,
    ) -> GoodsReceipt {
        self.gr_counter += 1;

        let gr_id = self.make_doc_id("GR", "goods_receipt", company_code, self.gr_counter);

        let mut gr = GoodsReceipt::from_purchase_order(
            gr_id,
            company_code,
            &po.header.document_id,
            &po.vendor_id,
            format!("P{company_code}"),
            "0001",
            fiscal_year,
            fiscal_period,
            gr_date,
            created_by,
        );

        // Add items based on PO items
        for po_item in &po.items {
            let received_qty = (po_item.base.quantity
                * Decimal::from_f64_retain(quantity_pct).unwrap_or(Decimal::ONE))
            .round_dp(0);

            if received_qty > Decimal::ZERO {
                let description =
                    self.pick_line_description("goods_receipt", &po_item.base.description);
                let mut gr_item = GoodsReceiptItem::from_po(
                    po_item.base.line_number,
                    &description,
                    received_qty,
                    po_item.base.unit_price,
                    &po.header.document_id,
                    po_item.base.line_number,
                )
                .with_movement_type(MovementType::GrForPo);

                // Carry material_id from PO item to GR item
                if let Some(ref mat_id) = po_item.base.material_id {
                    gr_item = gr_item.with_material(mat_id);
                }

                gr.add_item(gr_item);
            }
        }

        // Post the GR
        gr.post(created_by, gr_date);

        gr
    }

    /// Generate vendor invoice.
    fn generate_vendor_invoice(
        &mut self,
        po: &PurchaseOrder,
        goods_receipts: &[GoodsReceipt],
        company_code: &str,
        vendor: &Vendor,
        invoice_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
        three_way_match_passed: bool,
    ) -> Option<VendorInvoice> {
        if goods_receipts.is_empty() {
            return None;
        }

        self.vi_counter += 1;

        let invoice_id = self.make_doc_id("VI", "vendor_invoice", company_code, self.vi_counter);
        let vendor_invoice_number = format!("INV-{:08}", self.rng.random_range(10000000..99999999));

        // Calculate due date based on payment terms
        let _due_date = self.calculate_due_date(invoice_date, &vendor.payment_terms);

        let net_days = vendor.payment_terms.net_days() as i64;

        let mut invoice = VendorInvoice::new(
            invoice_id,
            company_code,
            &vendor.vendor_id,
            vendor_invoice_number,
            fiscal_year,
            fiscal_period,
            invoice_date,
            created_by,
        )
        .with_payment_terms(vendor.payment_terms.code(), net_days);

        // Populate top-level FK fields (DS-GEP-004)
        invoice.purchase_order_id = Some(po.header.document_id.clone());
        invoice.goods_receipt_id = goods_receipts
            .first()
            .map(|gr| gr.header.document_id.clone());

        // Denormalize vendor name (DS-011)
        invoice.vendor_name = Some(vendor.name.clone());

        // Apply cash discount if payment terms have one
        if let (Some(discount_days), Some(discount_percent)) = (
            vendor.payment_terms.discount_days(),
            vendor.payment_terms.discount_percent(),
        ) {
            invoice = invoice.with_cash_discount(discount_percent, discount_days as i64);
        }

        // Calculate total received quantity per item
        let mut received_quantities: std::collections::HashMap<u16, Decimal> =
            std::collections::HashMap::new();

        for gr in goods_receipts {
            for gr_item in &gr.items {
                *received_quantities
                    .entry(gr_item.base.line_number)
                    .or_insert(Decimal::ZERO) += gr_item.base.quantity;
            }
        }

        // Add invoice items based on received quantities
        for po_item in &po.items {
            if let Some(&qty) = received_quantities.get(&po_item.base.line_number) {
                // Apply price variance if configured
                let unit_price = if !three_way_match_passed
                    && self.rng.random::<f64>() < self.config.price_variance_rate
                {
                    let variance = Decimal::from_f64_retain(
                        1.0 + (self.rng.random::<f64>() - 0.5)
                            * 2.0
                            * self.config.max_price_variance_percent,
                    )
                    .unwrap_or(Decimal::ONE);
                    (po_item.base.unit_price * variance).round_dp(2)
                } else {
                    po_item.base.unit_price
                };

                let vi_description =
                    self.pick_line_description("vendor_invoice", &po_item.base.description);
                let item = VendorInvoiceItem::from_po_gr(
                    po_item.base.line_number,
                    &vi_description,
                    qty,
                    unit_price,
                    &po.header.document_id,
                    po_item.base.line_number,
                    goods_receipts
                        .first()
                        .map(|gr| gr.header.document_id.clone()),
                    Some(po_item.base.line_number),
                );

                invoice.add_item(item);
            }
        }

        // Link to PO
        invoice.header.add_reference(DocumentReference::new(
            DocumentType::PurchaseOrder,
            &po.header.document_id,
            DocumentType::VendorInvoice,
            &invoice.header.document_id,
            ReferenceType::FollowOn,
            company_code,
            invoice_date,
        ));

        // Link to GRs
        for gr in goods_receipts {
            invoice.header.add_reference(DocumentReference::new(
                DocumentType::GoodsReceipt,
                &gr.header.document_id,
                DocumentType::VendorInvoice,
                &invoice.header.document_id,
                ReferenceType::FollowOn,
                company_code,
                invoice_date,
            ));
        }

        // Verify three-way match
        if three_way_match_passed {
            invoice.verify(true);
        }

        // Post the invoice
        invoice.post(created_by, invoice_date);

        Some(invoice)
    }

    /// Generate payment for an invoice.
    fn generate_payment(
        &mut self,
        invoice: &VendorInvoice,
        company_code: &str,
        vendor: &Vendor,
        payment_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> Payment {
        self.pay_counter += 1;

        let payment_id = self.make_doc_id("PAY", "payment", company_code, self.pay_counter);

        // Determine if early payment discount applies
        let take_discount = invoice.discount_due_date.is_some_and(|disc_date| {
            payment_date <= disc_date
                && self.rng.random::<f64>() < self.config.early_payment_discount_rate
        });

        let discount_amount = if take_discount {
            invoice.cash_discount_amount
        } else {
            Decimal::ZERO
        };

        let payment_amount = invoice.payable_amount - discount_amount;

        let mut payment = Payment::new_ap_payment(
            payment_id,
            company_code,
            &vendor.vendor_id,
            payment_amount,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_payment_method(self.select_payment_method())
        .with_value_date(payment_date + chrono::Duration::days(1));

        // Allocate to invoice
        payment.allocate_to_invoice(
            &invoice.header.document_id,
            DocumentType::VendorInvoice,
            payment_amount,
            discount_amount,
        );

        // Add document reference linking payment to invoice
        payment.header.add_reference(DocumentReference::new(
            DocumentType::ApPayment,
            &payment.header.document_id,
            DocumentType::VendorInvoice,
            &invoice.header.document_id,
            ReferenceType::Payment,
            &payment.header.company_code,
            payment_date,
        ));

        // Approve and send to bank
        payment.approve(created_by);
        payment.send_to_bank(created_by);

        // Post the payment
        payment.post(created_by, payment_date);

        payment
    }

    /// Generate a payment for a specific amount (used for partial payments).
    fn generate_payment_for_amount(
        &mut self,
        invoice: &VendorInvoice,
        company_code: &str,
        vendor: &Vendor,
        payment_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
        amount: Decimal,
    ) -> Payment {
        self.pay_counter += 1;

        let payment_id = self.make_doc_id("PAY", "payment", company_code, self.pay_counter);

        let mut payment = Payment::new_ap_payment(
            payment_id,
            company_code,
            &vendor.vendor_id,
            amount,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_payment_method(self.select_payment_method())
        .with_value_date(payment_date + chrono::Duration::days(1));

        // Allocate to invoice (partial amount, no discount on partial)
        payment.allocate_to_invoice(
            &invoice.header.document_id,
            DocumentType::VendorInvoice,
            amount,
            Decimal::ZERO,
        );

        // Add document reference linking payment to invoice
        payment.header.add_reference(DocumentReference::new(
            DocumentType::ApPayment,
            &payment.header.document_id,
            DocumentType::VendorInvoice,
            &invoice.header.document_id,
            ReferenceType::Payment,
            &payment.header.company_code,
            payment_date,
        ));

        // Approve and send to bank
        payment.approve(created_by);
        payment.send_to_bank(created_by);

        // Post the payment
        payment.post(created_by, payment_date);

        payment
    }

    /// Generate a remainder payment for the balance after a partial payment.
    fn generate_remainder_payment(
        &mut self,
        invoice: &VendorInvoice,
        company_code: &str,
        vendor: &Vendor,
        payment_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
        amount: Decimal,
        initial_payment: &Payment,
    ) -> Payment {
        self.pay_counter += 1;

        let payment_id = self.make_doc_id("PAY", "payment", company_code, self.pay_counter);

        let mut payment = Payment::new_ap_payment(
            payment_id,
            company_code,
            &vendor.vendor_id,
            amount,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_payment_method(self.select_payment_method())
        .with_value_date(payment_date + chrono::Duration::days(1));

        // Allocate remainder to the same invoice
        payment.allocate_to_invoice(
            &invoice.header.document_id,
            DocumentType::VendorInvoice,
            amount,
            Decimal::ZERO,
        );

        // Add document reference linking remainder payment to invoice
        payment.header.add_reference(DocumentReference::new(
            DocumentType::ApPayment,
            &payment.header.document_id,
            DocumentType::VendorInvoice,
            &invoice.header.document_id,
            ReferenceType::Payment,
            &payment.header.company_code,
            payment_date,
        ));

        // Add document reference linking remainder payment to initial payment
        payment.header.add_reference(DocumentReference::new(
            DocumentType::ApPayment,
            &payment.header.document_id,
            DocumentType::ApPayment,
            &initial_payment.header.document_id,
            ReferenceType::FollowOn,
            &payment.header.company_code,
            payment_date,
        ));

        // Approve and send to bank
        payment.approve(created_by);
        payment.send_to_bank(created_by);

        // Post the payment
        payment.post(created_by, payment_date);

        payment
    }

    /// Generate multiple P2P chains.
    pub fn generate_chains(
        &mut self,
        count: usize,
        company_code: &str,
        vendors: &VendorPool,
        materials: &MaterialPool,
        date_range: (NaiveDate, NaiveDate),
        fiscal_year: u16,
        created_by: &str,
    ) -> Vec<P2PDocumentChain> {
        tracing::debug!(count, company_code, "Generating P2P document chains");
        let mut chains = Vec::new();

        let (start_date, end_date) = date_range;
        let days_range = (end_date - start_date).num_days() as u64;

        for _ in 0..count {
            // Select random vendor
            let vendor_idx = self.rng.random_range(0..vendors.vendors.len());
            let vendor = &vendors.vendors[vendor_idx];

            // Select random materials (1-5 items per PO)
            let num_items = self.rng.random_range(1..=5).min(materials.materials.len());
            let selected_materials: Vec<&Material> = materials
                .materials
                .iter()
                .choose_multiple(&mut self.rng, num_items)
                .into_iter()
                .collect();

            // Select random PO date
            let po_date =
                start_date + chrono::Duration::days(self.rng.random_range(0..=days_range) as i64);
            let fiscal_period = self.get_fiscal_period(po_date);

            let chain = self.generate_chain(
                company_code,
                vendor,
                &selected_materials,
                po_date,
                fiscal_year,
                fiscal_period,
                created_by,
            );

            chains.push(chain);
        }

        chains
    }

    /// Calculate GR date based on PO date.
    fn calculate_gr_date(&mut self, po_date: NaiveDate) -> NaiveDate {
        let variance = self.rng.random_range(0..5) as i64;
        po_date + chrono::Duration::days(self.config.avg_days_po_to_gr as i64 + variance)
    }

    /// Calculate invoice date based on GR date.
    fn calculate_invoice_date(&mut self, gr_date: NaiveDate) -> NaiveDate {
        let variance = self.rng.random_range(0..3) as i64;
        gr_date + chrono::Duration::days(self.config.avg_days_gr_to_invoice as i64 + variance)
    }

    /// Calculate payment date based on invoice date and payment terms.
    fn calculate_payment_date(
        &mut self,
        invoice_date: NaiveDate,
        payment_terms: &PaymentTerms,
    ) -> NaiveDate {
        let due_days = payment_terms.net_days() as i64;
        let due_date = invoice_date + chrono::Duration::days(due_days);

        // Determine if this is a late payment
        if self.rng.random::<f64>() < self.config.payment_behavior.late_payment_rate {
            // Calculate late days based on distribution
            let late_days = self.calculate_late_days();
            due_date + chrono::Duration::days(late_days as i64)
        } else {
            // On-time or slightly early payment (-5 to +5 days variance)
            let variance = self.rng.random_range(-5..=5) as i64;
            due_date + chrono::Duration::days(variance)
        }
    }

    /// Calculate late payment days based on the distribution.
    fn calculate_late_days(&mut self) -> u32 {
        let roll: f64 = self.rng.random();
        let dist = &self.config.payment_behavior.late_payment_distribution;

        let mut cumulative = 0.0;

        cumulative += dist.slightly_late_1_to_7;
        if roll < cumulative {
            return self.rng.random_range(1..=7);
        }

        cumulative += dist.late_8_to_14;
        if roll < cumulative {
            return self.rng.random_range(8..=14);
        }

        cumulative += dist.very_late_15_to_30;
        if roll < cumulative {
            return self.rng.random_range(15..=30);
        }

        cumulative += dist.severely_late_31_to_60;
        if roll < cumulative {
            return self.rng.random_range(31..=60);
        }

        // Extremely late: 61-120 days
        self.rng.random_range(61..=120)
    }

    /// Calculate due date based on payment terms.
    fn calculate_due_date(
        &self,
        invoice_date: NaiveDate,
        payment_terms: &PaymentTerms,
    ) -> NaiveDate {
        invoice_date + chrono::Duration::days(payment_terms.net_days() as i64)
    }

    /// Select payment method based on distribution.
    fn select_payment_method(&mut self) -> PaymentMethod {
        let roll: f64 = self.rng.random();
        let mut cumulative = 0.0;

        for (method, prob) in &self.config.payment_method_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *method;
            }
        }

        PaymentMethod::BankTransfer
    }

    /// Get fiscal period from date (simple month-based).
    fn get_fiscal_period(&self, date: NaiveDate) -> u8 {
        date.month() as u8
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.po_counter = 0;
        self.gr_counter = 0;
        self.vi_counter = 0;
        self.pay_counter = 0;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::documents::DocumentStatus;
    use datasynth_core::models::MaterialType;

    fn create_test_vendor() -> Vendor {
        Vendor::new(
            "V-000001",
            "Test Vendor Inc.",
            datasynth_core::models::VendorType::Supplier,
        )
    }

    fn create_test_materials() -> Vec<Material> {
        vec![
            Material::new("MAT-001", "Test Material 1", MaterialType::RawMaterial)
                .with_standard_cost(Decimal::from(100)),
            Material::new("MAT-002", "Test Material 2", MaterialType::RawMaterial)
                .with_standard_cost(Decimal::from(50)),
        ]
    }

    #[test]
    fn test_p2p_chain_generation() {
        let mut gen = P2PGenerator::new(42);
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert!(!chain.purchase_order.items.is_empty());
        assert!(!chain.goods_receipts.is_empty());
        assert!(chain.vendor_invoice.is_some());
        assert!(chain.payment.is_some());
        assert!(chain.is_complete);
    }

    #[test]
    fn test_purchase_order_generation() {
        let mut gen = P2PGenerator::new(42);
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let po = gen.generate_purchase_order(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert_eq!(po.vendor_id, "V-000001");
        assert_eq!(po.items.len(), 2);
        assert!(po.total_net_amount > Decimal::ZERO);
        assert_eq!(po.header.status, DocumentStatus::Released);
    }

    #[test]
    fn test_document_references() {
        let mut gen = P2PGenerator::new(42);
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        // GR should reference PO
        let gr = &chain.goods_receipts[0];
        assert!(!gr.header.document_references.is_empty());

        // Invoice should reference PO and GR
        if let Some(invoice) = &chain.vendor_invoice {
            assert!(invoice.header.document_references.len() >= 2);
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let mut gen1 = P2PGenerator::new(42);
        let mut gen2 = P2PGenerator::new(42);

        let chain1 = gen1.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );
        let chain2 = gen2.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert_eq!(
            chain1.purchase_order.header.document_id,
            chain2.purchase_order.header.document_id
        );
        assert_eq!(
            chain1.purchase_order.total_net_amount,
            chain2.purchase_order.total_net_amount
        );
    }

    #[test]
    fn test_partial_delivery_config() {
        let config = P2PGeneratorConfig {
            partial_delivery_rate: 1.0, // Force partial delivery
            ..Default::default()
        };

        let mut gen = P2PGenerator::with_config(42, config);
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        // Should have multiple goods receipts due to partial delivery
        assert!(chain.goods_receipts.len() >= 2);
    }

    #[test]
    fn test_partial_payment_produces_remainder() {
        let config = P2PGeneratorConfig {
            payment_behavior: P2PPaymentBehavior {
                partial_payment_rate: 1.0, // Force partial payment
                avg_days_until_remainder: 30,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = P2PGenerator::with_config(42, config);
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        // With 100% partial_payment_rate, chain must have both payment and remainder
        assert!(
            chain.payment.is_some(),
            "Chain should have an initial payment"
        );
        assert_eq!(
            chain.remainder_payments.len(),
            1,
            "Chain should have exactly one remainder payment"
        );
    }

    #[test]
    fn test_partial_payment_amounts_sum_to_invoice() {
        let config = P2PGeneratorConfig {
            payment_behavior: P2PPaymentBehavior {
                partial_payment_rate: 1.0, // Force partial payment
                avg_days_until_remainder: 30,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = P2PGenerator::with_config(42, config);
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        let invoice = chain.vendor_invoice.as_ref().unwrap();
        let initial_payment = chain.payment.as_ref().unwrap();
        let remainder = &chain.remainder_payments[0];

        // payment amount + remainder amount = invoice payable_amount
        let total_paid = initial_payment.amount + remainder.amount;
        assert_eq!(
            total_paid, invoice.payable_amount,
            "Initial payment ({}) + remainder ({}) = {} but invoice payable is {}",
            initial_payment.amount, remainder.amount, total_paid, invoice.payable_amount
        );
    }

    #[test]
    fn test_remainder_payment_date_after_initial() {
        let config = P2PGeneratorConfig {
            payment_behavior: P2PPaymentBehavior {
                partial_payment_rate: 1.0, // Force partial payment
                avg_days_until_remainder: 30,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = P2PGenerator::with_config(42, config);
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        let initial_payment = chain.payment.as_ref().unwrap();
        let remainder = &chain.remainder_payments[0];

        // Remainder date should be after initial payment date
        assert!(
            remainder.header.document_date > initial_payment.header.document_date,
            "Remainder date ({}) should be after initial payment date ({})",
            remainder.header.document_date,
            initial_payment.header.document_date
        );
    }

    #[test]
    fn test_no_partial_payment_means_no_remainder() {
        let config = P2PGeneratorConfig {
            payment_behavior: P2PPaymentBehavior {
                partial_payment_rate: 0.0, // Never partial payment
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = P2PGenerator::with_config(42, config);
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert!(chain.payment.is_some(), "Chain should have a full payment");
        assert!(
            chain.remainder_payments.is_empty(),
            "Chain should have no remainder payments when partial_payment_rate is 0"
        );
    }

    #[test]
    fn test_partial_payment_amount_in_expected_range() {
        let config = P2PGeneratorConfig {
            payment_behavior: P2PPaymentBehavior {
                partial_payment_rate: 1.0, // Force partial payment
                avg_days_until_remainder: 30,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = P2PGenerator::with_config(42, config);
        let vendor = create_test_vendor();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &vendor,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        let invoice = chain.vendor_invoice.as_ref().unwrap();
        let initial_payment = chain.payment.as_ref().unwrap();

        // Partial payment should be 50-75% of invoice amount
        let min_pct = Decimal::from_f64_retain(0.50).unwrap();
        let max_pct = Decimal::from_f64_retain(0.75).unwrap();
        let min_amount = (invoice.payable_amount * min_pct).round_dp(2);
        let max_amount = (invoice.payable_amount * max_pct).round_dp(2);

        assert!(
            initial_payment.amount >= min_amount && initial_payment.amount <= max_amount,
            "Partial payment {} should be between {} and {} (50-75% of {})",
            initial_payment.amount,
            min_amount,
            max_amount,
            invoice.payable_amount
        );
    }
}
