//! Order-to-Cash (O2C) flow generator.
//!
//! Generates complete O2C document chains:
//! SalesOrder → Delivery → CustomerInvoice → CustomerReceipt (Payment)

use chrono::{Datelike, NaiveDate};
use datasynth_core::models::{
    documents::{
        CustomerInvoice, CustomerInvoiceItem, Delivery, DeliveryItem, DocumentReference,
        DocumentType, Payment, PaymentMethod, ReferenceType, SalesOrder, SalesOrderItem,
    },
    subledger::ar::{
        ARCreditMemo, ARCreditMemoLine, CreditMemoReason, OnAccountPayment, OnAccountReason,
        PaymentCorrection, PaymentCorrectionType, ShortPayment, ShortPaymentReasonCode,
    },
    CreditRating, Customer, CustomerPool, Material, MaterialPool, PaymentTerms,
};
use datasynth_core::utils::seeded_rng;
use datasynth_core::CountryPack;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Configuration for O2C flow generation.
#[derive(Debug, Clone)]
pub struct O2CGeneratorConfig {
    /// Credit check failure rate
    pub credit_check_failure_rate: f64,
    /// Rate of partial shipments
    pub partial_shipment_rate: f64,
    /// Average days between SO and Delivery
    pub avg_days_so_to_delivery: u32,
    /// Average days between Delivery and Invoice
    pub avg_days_delivery_to_invoice: u32,
    /// Average days between Invoice and Payment (customer payment)
    pub avg_days_invoice_to_payment: u32,
    /// Late payment rate
    pub late_payment_rate: f64,
    /// Bad debt rate (no payment)
    pub bad_debt_rate: f64,
    /// Rate of sales returns
    pub returns_rate: f64,
    /// Cash discount take rate
    pub cash_discount_take_rate: f64,
    /// Payment method distribution for AR receipts
    pub payment_method_distribution: Vec<(PaymentMethod, f64)>,
    /// Payment behavior configuration
    pub payment_behavior: O2CPaymentBehavior,
}

/// Payment behavior configuration for O2C.
#[derive(Debug, Clone)]
pub struct O2CPaymentBehavior {
    /// Rate of partial payments
    pub partial_payment_rate: f64,
    /// Rate of short payments (unauthorized deductions)
    pub short_payment_rate: f64,
    /// Maximum short payment percentage
    pub max_short_percent: f64,
    /// Rate of on-account payments (unapplied)
    pub on_account_rate: f64,
    /// Rate of payment corrections (NSF, chargebacks)
    pub payment_correction_rate: f64,
    /// Average days until partial payment remainder
    pub avg_days_until_remainder: u32,
}

impl Default for O2CPaymentBehavior {
    fn default() -> Self {
        Self {
            partial_payment_rate: 0.08,
            short_payment_rate: 0.03,
            max_short_percent: 0.10,
            on_account_rate: 0.02,
            payment_correction_rate: 0.02,
            avg_days_until_remainder: 30,
        }
    }
}

impl Default for O2CGeneratorConfig {
    fn default() -> Self {
        Self {
            credit_check_failure_rate: 0.02,
            partial_shipment_rate: 0.08,
            avg_days_so_to_delivery: 5,
            avg_days_delivery_to_invoice: 1,
            avg_days_invoice_to_payment: 30,
            late_payment_rate: 0.15,
            bad_debt_rate: 0.02,
            returns_rate: 0.03,
            cash_discount_take_rate: 0.25,
            payment_method_distribution: vec![
                (PaymentMethod::BankTransfer, 0.50),
                (PaymentMethod::Check, 0.30),
                (PaymentMethod::Wire, 0.15),
                (PaymentMethod::CreditCard, 0.05),
            ],
            payment_behavior: O2CPaymentBehavior::default(),
        }
    }
}

/// A complete O2C document chain.
#[derive(Debug, Clone)]
pub struct O2CDocumentChain {
    /// Sales Order
    pub sales_order: SalesOrder,
    /// Deliveries (may be multiple for partial shipments)
    pub deliveries: Vec<Delivery>,
    /// Customer Invoice
    pub customer_invoice: Option<CustomerInvoice>,
    /// Customer Receipt (Payment)
    pub customer_receipt: Option<Payment>,
    /// Credit memo (if return or adjustment)
    pub credit_memo: Option<ARCreditMemo>,
    /// Chain completion status
    pub is_complete: bool,
    /// Credit check passed
    pub credit_check_passed: bool,
    /// Is this a return/credit memo chain
    pub is_return: bool,
    /// Payment events (partial, short, corrections, etc.)
    pub payment_events: Vec<PaymentEvent>,
    /// Remainder payment receipts (follow-up to partial payments)
    pub remainder_receipts: Vec<Payment>,
}

/// Payment event in an O2C chain.
#[derive(Debug, Clone)]
pub enum PaymentEvent {
    /// Full payment received
    FullPayment(Payment),
    /// Partial payment received
    PartialPayment {
        payment: Payment,
        remaining_amount: Decimal,
        expected_remainder_date: Option<NaiveDate>,
    },
    /// Short payment (deduction)
    ShortPayment {
        payment: Payment,
        short_payment: ShortPayment,
    },
    /// On-account payment (unapplied)
    OnAccountPayment(OnAccountPayment),
    /// Payment correction (NSF, chargeback)
    PaymentCorrection {
        original_payment: Payment,
        correction: PaymentCorrection,
    },
    /// Remainder payment (follow-up to partial)
    RemainderPayment(Payment),
}

/// Generator for O2C document flows.
pub struct O2CGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: O2CGeneratorConfig,
    so_counter: usize,
    dlv_counter: usize,
    ci_counter: usize,
    rec_counter: usize,
    credit_memo_counter: usize,
    short_payment_counter: usize,
    on_account_counter: usize,
    correction_counter: usize,
    country_pack: Option<CountryPack>,
}

impl O2CGenerator {
    /// Create a new O2C generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, O2CGeneratorConfig::default())
    }

    /// Create a new O2C generator with custom configuration.
    pub fn with_config(seed: u64, config: O2CGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            seed,
            config,
            so_counter: 0,
            dlv_counter: 0,
            ci_counter: 0,
            rec_counter: 0,
            credit_memo_counter: 0,
            short_payment_counter: 0,
            on_account_counter: 0,
            correction_counter: 0,
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
                    "sales_order" => &p.document_texts.sales_order,
                    "delivery" => &p.document_texts.delivery,
                    "customer_invoice" => &p.document_texts.customer_invoice,
                    "customer_receipt" => &p.document_texts.customer_receipt,
                    _ => return default_prefix.to_string(),
                };
                if grp.reference_prefix.is_empty() {
                    default_prefix.to_string()
                } else {
                    grp.reference_prefix.clone()
                }
            })
            .unwrap_or_else(|| default_prefix.to_string());
        format!("{}-{}-{:010}", prefix, company_code, counter)
    }

    /// Pick a random line description from the country pack for the given
    /// document type, falling back to the provided default.
    fn pick_line_description(&mut self, pack_key: &str, default: &str) -> String {
        if let Some(pack) = &self.country_pack {
            let descriptions = match pack_key {
                "sales_order" => &pack.document_texts.sales_order.line_descriptions,
                "delivery" => &pack.document_texts.delivery.line_descriptions,
                "customer_invoice" => &pack.document_texts.customer_invoice.line_descriptions,
                "customer_receipt" => &pack.document_texts.customer_receipt.line_descriptions,
                _ => return default.to_string(),
            };
            if !descriptions.is_empty() {
                let idx = self.rng.random_range(0..descriptions.len());
                return descriptions[idx].clone();
            }
        }
        default.to_string()
    }

    /// Generate a complete O2C chain.
    pub fn generate_chain(
        &mut self,
        company_code: &str,
        customer: &Customer,
        materials: &[&Material],
        so_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> O2CDocumentChain {
        // Generate SO
        let mut so = self.generate_sales_order(
            company_code,
            customer,
            materials,
            so_date,
            fiscal_year,
            fiscal_period,
            created_by,
        );

        // Perform credit check
        let credit_check_passed = self.perform_credit_check(customer, so.total_gross_amount);
        so.check_credit(
            credit_check_passed,
            if !credit_check_passed {
                Some("Credit limit exceeded".to_string())
            } else {
                None
            },
        );

        // If credit check fails, the chain may be blocked
        if !credit_check_passed {
            return O2CDocumentChain {
                sales_order: so,
                deliveries: Vec::new(),
                customer_invoice: None,
                customer_receipt: None,
                credit_memo: None,
                is_complete: false,
                credit_check_passed: false,
                is_return: false,
                payment_events: Vec::new(),
                remainder_receipts: Vec::new(),
            };
        }

        // Release for delivery
        so.release_for_delivery();

        // Calculate delivery date
        let delivery_date = self.calculate_delivery_date(so_date);
        let delivery_fiscal_period = self.get_fiscal_period(delivery_date);

        // Generate delivery(s)
        let deliveries = self.generate_deliveries(
            &so,
            company_code,
            customer,
            delivery_date,
            fiscal_year,
            delivery_fiscal_period,
            created_by,
        );

        // Calculate invoice date
        let invoice_date = self.calculate_invoice_date(delivery_date);
        let invoice_fiscal_period = self.get_fiscal_period(invoice_date);

        // Release for billing
        so.release_for_billing();

        // Generate customer invoice
        let customer_invoice = if !deliveries.is_empty() {
            Some(self.generate_customer_invoice(
                &so,
                &deliveries,
                company_code,
                customer,
                invoice_date,
                fiscal_year,
                invoice_fiscal_period,
                created_by,
            ))
        } else {
            None
        };

        // Determine if customer pays
        let will_pay = self.rng.random::<f64>() >= self.config.bad_debt_rate;

        // Calculate payment date and determine payment type
        let mut payment_events = Vec::new();
        let mut customer_receipt = None;
        let mut remainder_receipts = Vec::new();

        if will_pay {
            if let Some(ref invoice) = customer_invoice {
                let payment_date =
                    self.calculate_payment_date(invoice_date, &customer.payment_terms, customer);
                let payment_fiscal_period = self.get_fiscal_period(payment_date);

                let payment_type = self.determine_payment_type();

                match payment_type {
                    PaymentType::Partial => {
                        let payment_percent = self.determine_partial_payment_percent();
                        let (payment, remaining, expected_date) = self.generate_partial_payment(
                            invoice,
                            company_code,
                            customer,
                            payment_date,
                            fiscal_year,
                            payment_fiscal_period,
                            created_by,
                            payment_percent,
                        );

                        payment_events.push(PaymentEvent::PartialPayment {
                            payment: payment.clone(),
                            remaining_amount: remaining,
                            expected_remainder_date: expected_date,
                        });
                        customer_receipt = Some(payment);

                        // Generate remainder payment
                        if remaining > Decimal::ZERO {
                            let remainder_date = expected_date.unwrap_or(
                                payment_date
                                    + chrono::Duration::days(
                                        self.config.payment_behavior.avg_days_until_remainder
                                            as i64,
                                    ),
                            );
                            let remainder_period = self.get_fiscal_period(remainder_date);
                            let remainder_payment = self.generate_remainder_payment(
                                invoice,
                                company_code,
                                customer,
                                remainder_date,
                                fiscal_year,
                                remainder_period,
                                created_by,
                                remaining,
                            );
                            payment_events
                                .push(PaymentEvent::RemainderPayment(remainder_payment.clone()));
                            remainder_receipts.push(remainder_payment);
                        }
                    }
                    PaymentType::Short => {
                        let (payment, short) = self.generate_short_payment(
                            invoice,
                            company_code,
                            customer,
                            payment_date,
                            fiscal_year,
                            payment_fiscal_period,
                            created_by,
                        );

                        payment_events.push(PaymentEvent::ShortPayment {
                            payment: payment.clone(),
                            short_payment: short,
                        });
                        customer_receipt = Some(payment);
                    }
                    PaymentType::OnAccount => {
                        // On-account payment - not tied to this specific invoice
                        let amount = invoice.total_gross_amount
                            * Decimal::from_f64_retain(0.8 + self.rng.random::<f64>() * 0.4)
                                .unwrap_or(Decimal::ONE);
                        let (payment, on_account) = self.generate_on_account_payment(
                            company_code,
                            customer,
                            payment_date,
                            fiscal_year,
                            payment_fiscal_period,
                            created_by,
                            &invoice.header.currency,
                            amount.round_dp(2),
                        );

                        payment_events.push(PaymentEvent::OnAccountPayment(on_account));
                        customer_receipt = Some(payment);
                    }
                    PaymentType::Full => {
                        let payment = self.generate_customer_receipt(
                            invoice,
                            company_code,
                            customer,
                            payment_date,
                            fiscal_year,
                            payment_fiscal_period,
                            created_by,
                        );

                        // Check if this payment will have a correction
                        if self.rng.random::<f64>()
                            < self.config.payment_behavior.payment_correction_rate
                        {
                            let correction_date = payment_date
                                + chrono::Duration::days(self.rng.random_range(3..14) as i64);

                            let correction = self.generate_payment_correction(
                                &payment,
                                company_code,
                                &customer.customer_id,
                                correction_date,
                                &invoice.header.currency,
                            );

                            payment_events.push(PaymentEvent::PaymentCorrection {
                                original_payment: payment.clone(),
                                correction,
                            });
                        } else {
                            payment_events.push(PaymentEvent::FullPayment(payment.clone()));
                        }

                        customer_receipt = Some(payment);
                    }
                }
            }
        }

        let has_partial = payment_events
            .iter()
            .any(|e| matches!(e, PaymentEvent::PartialPayment { .. }));
        let has_remainder = payment_events
            .iter()
            .any(|e| matches!(e, PaymentEvent::RemainderPayment(_)));
        let has_correction = payment_events
            .iter()
            .any(|e| matches!(e, PaymentEvent::PaymentCorrection { .. }));

        let is_complete =
            customer_receipt.is_some() && !has_correction && (!has_partial || has_remainder);

        // Generate credit memo for returns based on returns_rate
        let credit_memo = if let Some(ref invoice) = customer_invoice {
            if self.rng.random_bool(self.config.returns_rate) {
                let return_days = self.rng.random_range(5u32..=30);
                let return_date =
                    invoice.header.document_date + chrono::Duration::days(return_days as i64);
                Some(self.generate_return_credit_memo(
                    invoice,
                    customer,
                    company_code,
                    return_date,
                ))
            } else {
                None
            }
        } else {
            None
        };
        let is_return = credit_memo.is_some();

        O2CDocumentChain {
            sales_order: so,
            deliveries,
            customer_invoice,
            customer_receipt,
            credit_memo,
            is_complete,
            credit_check_passed: true,
            is_return,
            payment_events,
            remainder_receipts,
        }
    }

    /// Generate an AR credit memo for a return against a customer invoice.
    fn generate_return_credit_memo(
        &mut self,
        invoice: &CustomerInvoice,
        customer: &Customer,
        company_code: &str,
        return_date: NaiveDate,
    ) -> ARCreditMemo {
        self.credit_memo_counter += 1;
        let cm_number = format!("CM-{}-{:010}", company_code, self.credit_memo_counter);

        let reason = match self.rng.random_range(0u8..=3) {
            0 => CreditMemoReason::Return,
            1 => CreditMemoReason::Damaged,
            2 => CreditMemoReason::QualityIssue,
            _ => CreditMemoReason::PriceError,
        };

        let reason_desc = match reason {
            CreditMemoReason::Return => "Goods returned by customer",
            CreditMemoReason::Damaged => "Goods damaged in transit",
            CreditMemoReason::QualityIssue => "Quality issue reported",
            CreditMemoReason::PriceError => "Invoice price correction",
            _ => "Credit adjustment",
        };

        let currency = invoice.header.currency.clone();
        let mut memo = ARCreditMemo::for_invoice(
            cm_number,
            company_code.to_string(),
            customer.customer_id.clone(),
            customer.name.clone(),
            return_date,
            invoice.header.document_id.clone(),
            reason,
            reason_desc.to_string(),
            currency.clone(),
        );

        // Credit 10-100% of invoice amount
        let credit_pct = self.rng.random_range(0.10f64..=1.0);
        let credit_amount = (invoice.total_gross_amount
            * Decimal::from_f64_retain(credit_pct).unwrap_or(Decimal::ONE))
        .round_dp(2);

        memo.add_line(ARCreditMemoLine {
            line_number: 1,
            material_id: None,
            description: format!("{:?} - {}", reason, reason_desc),
            quantity: Decimal::ONE,
            unit: "EA".to_string(),
            unit_price: credit_amount,
            net_amount: credit_amount,
            tax_code: None,
            tax_rate: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            gross_amount: credit_amount,
            revenue_account: "4000".to_string(),
            reference_invoice_line: Some(1),
            cost_center: None,
            profit_center: None,
        });

        // Auto-approve if under threshold (e.g., 10,000)
        let threshold = Decimal::from(10_000);
        if !memo.requires_approval(threshold) {
            memo.approve("SYSTEM".to_string(), return_date);
        }

        memo
    }

    /// Generate a sales order.
    pub fn generate_sales_order(
        &mut self,
        company_code: &str,
        customer: &Customer,
        materials: &[&Material],
        so_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> SalesOrder {
        self.so_counter += 1;

        let so_id = self.make_doc_id("SO", "sales_order", company_code, self.so_counter);

        let requested_delivery =
            so_date + chrono::Duration::days(self.config.avg_days_so_to_delivery as i64);

        let mut so = SalesOrder::new(
            so_id,
            company_code,
            &customer.customer_id,
            fiscal_year,
            fiscal_period,
            so_date,
            created_by,
        )
        .with_requested_delivery_date(requested_delivery);

        // Denormalize customer name (DS-011)
        so.customer_name = Some(customer.name.clone());

        // Add line items
        for (idx, material) in materials.iter().enumerate() {
            let quantity = Decimal::from(self.rng.random_range(1..50));
            let unit_price = material.list_price;

            let description = self.pick_line_description("sales_order", &material.description);
            let mut item =
                SalesOrderItem::new((idx + 1) as u16 * 10, &description, quantity, unit_price)
                    .with_material(&material.material_id);

            // Add schedule line
            item.add_schedule_line(requested_delivery, quantity);

            so.add_item(item);
        }

        so
    }

    /// Generate deliveries for a sales order.
    fn generate_deliveries(
        &mut self,
        so: &SalesOrder,
        company_code: &str,
        customer: &Customer,
        delivery_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> Vec<Delivery> {
        let mut deliveries = Vec::new();

        // Determine if partial shipment
        let is_partial = self.rng.random::<f64>() < self.config.partial_shipment_rate;

        if is_partial {
            // First shipment (60-80%)
            let first_pct = 0.6 + self.rng.random::<f64>() * 0.2;
            let dlv1 = self.create_delivery(
                so,
                company_code,
                customer,
                delivery_date,
                fiscal_year,
                fiscal_period,
                created_by,
                first_pct,
            );
            deliveries.push(dlv1);

            // Second shipment
            let second_date =
                delivery_date + chrono::Duration::days(self.rng.random_range(3..7) as i64);
            let second_period = self.get_fiscal_period(second_date);
            let dlv2 = self.create_delivery(
                so,
                company_code,
                customer,
                second_date,
                fiscal_year,
                second_period,
                created_by,
                1.0 - first_pct,
            );
            deliveries.push(dlv2);
        } else {
            // Full shipment
            let dlv = self.create_delivery(
                so,
                company_code,
                customer,
                delivery_date,
                fiscal_year,
                fiscal_period,
                created_by,
                1.0,
            );
            deliveries.push(dlv);
        }

        deliveries
    }

    /// Create a single delivery.
    fn create_delivery(
        &mut self,
        so: &SalesOrder,
        company_code: &str,
        customer: &Customer,
        delivery_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
        quantity_pct: f64,
    ) -> Delivery {
        self.dlv_counter += 1;

        let dlv_id = self.make_doc_id("DLV", "delivery", company_code, self.dlv_counter);

        let mut delivery = Delivery::from_sales_order(
            dlv_id,
            company_code,
            &so.header.document_id,
            &customer.customer_id,
            format!("SP{}", company_code),
            fiscal_year,
            fiscal_period,
            delivery_date,
            created_by,
        );

        // Add items based on SO items
        for so_item in &so.items {
            let ship_qty = (so_item.base.quantity
                * Decimal::from_f64_retain(quantity_pct).unwrap_or(Decimal::ONE))
            .round_dp(0);

            if ship_qty > Decimal::ZERO {
                // Calculate COGS (assume 60-70% of sales price)
                let cogs_pct = 0.60 + self.rng.random::<f64>() * 0.10;
                let cogs = (so_item.base.unit_price
                    * ship_qty
                    * Decimal::from_f64_retain(cogs_pct)
                        .unwrap_or(Decimal::from_f64_retain(0.65).expect("valid decimal literal")))
                .round_dp(2);

                let dlv_description =
                    self.pick_line_description("delivery", &so_item.base.description);
                let mut item = DeliveryItem::from_sales_order(
                    so_item.base.line_number,
                    &dlv_description,
                    ship_qty,
                    so_item.base.unit_price,
                    &so.header.document_id,
                    so_item.base.line_number,
                )
                .with_cogs(cogs);

                if let Some(material_id) = &so_item.base.material_id {
                    item = item.with_material(material_id);
                }

                // Mark as picked
                item.record_pick(ship_qty);

                delivery.add_item(item);
            }
        }

        // Process delivery workflow
        delivery.release_for_picking(created_by);
        delivery.confirm_pick();
        delivery.confirm_pack(self.rng.random_range(1..10));
        delivery.post_goods_issue(created_by, delivery_date);

        delivery
    }

    /// Generate customer invoice.
    fn generate_customer_invoice(
        &mut self,
        so: &SalesOrder,
        deliveries: &[Delivery],
        company_code: &str,
        customer: &Customer,
        invoice_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> CustomerInvoice {
        self.ci_counter += 1;

        let invoice_id = self.make_doc_id("CI", "customer_invoice", company_code, self.ci_counter);

        // Calculate due date based on payment terms
        let due_date = self.calculate_due_date(invoice_date, &customer.payment_terms);

        let mut invoice = CustomerInvoice::from_delivery(
            invoice_id,
            company_code,
            &deliveries[0].header.document_id,
            &customer.customer_id,
            fiscal_year,
            fiscal_period,
            invoice_date,
            due_date,
            created_by,
        )
        .with_payment_terms(
            customer.payment_terms.code(),
            customer.payment_terms.discount_days(),
            customer.payment_terms.discount_percent(),
        );

        // Denormalize customer name (DS-011)
        invoice.customer_name = Some(customer.name.clone());

        // Calculate total delivered quantity per item
        let mut delivered_quantities: std::collections::HashMap<u16, (Decimal, Decimal)> =
            std::collections::HashMap::new();

        for dlv in deliveries {
            for dlv_item in &dlv.items {
                let entry = delivered_quantities
                    .entry(dlv_item.base.line_number)
                    .or_insert((Decimal::ZERO, Decimal::ZERO));
                entry.0 += dlv_item.base.quantity;
                entry.1 += dlv_item.cogs_amount;
            }
        }

        // Add invoice items based on delivered quantities
        for so_item in &so.items {
            if let Some(&(qty, cogs)) = delivered_quantities.get(&so_item.base.line_number) {
                let ci_description =
                    self.pick_line_description("customer_invoice", &so_item.base.description);
                let item = CustomerInvoiceItem::from_delivery(
                    so_item.base.line_number,
                    &ci_description,
                    qty,
                    so_item.base.unit_price,
                    &deliveries[0].header.document_id,
                    so_item.base.line_number,
                )
                .with_cogs(cogs)
                .with_sales_order(&so.header.document_id, so_item.base.line_number);

                invoice.add_item(item);
            }
        }

        // Link to SO
        invoice.header.add_reference(DocumentReference::new(
            DocumentType::SalesOrder,
            &so.header.document_id,
            DocumentType::CustomerInvoice,
            &invoice.header.document_id,
            ReferenceType::FollowOn,
            company_code,
            invoice_date,
        ));

        // Link to all deliveries
        for dlv in deliveries {
            invoice.header.add_reference(DocumentReference::new(
                DocumentType::Delivery,
                &dlv.header.document_id,
                DocumentType::CustomerInvoice,
                &invoice.header.document_id,
                ReferenceType::FollowOn,
                company_code,
                invoice_date,
            ));
        }

        // Post the invoice
        invoice.post(created_by, invoice_date);

        invoice
    }

    /// Generate customer receipt (AR payment).
    fn generate_customer_receipt(
        &mut self,
        invoice: &CustomerInvoice,
        company_code: &str,
        customer: &Customer,
        payment_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> Payment {
        self.rec_counter += 1;

        let receipt_id =
            self.make_doc_id("REC", "customer_receipt", company_code, self.rec_counter);

        // Determine if cash discount taken
        let take_discount = invoice.discount_date_1.is_some_and(|disc_date| {
            payment_date <= disc_date
                && self.rng.random::<f64>() < self.config.cash_discount_take_rate
        });

        let discount_amount = if take_discount {
            invoice.cash_discount_available(payment_date)
        } else {
            Decimal::ZERO
        };

        let payment_amount = invoice.amount_open - discount_amount;

        let mut receipt = Payment::new_ar_receipt(
            receipt_id,
            company_code,
            &customer.customer_id,
            payment_amount,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_payment_method(self.select_payment_method())
        .with_value_date(payment_date);

        // Allocate to invoice
        receipt.allocate_to_invoice(
            &invoice.header.document_id,
            DocumentType::CustomerInvoice,
            payment_amount,
            discount_amount,
        );

        // Add document reference linking receipt to invoice
        receipt.header.add_reference(DocumentReference::new(
            DocumentType::CustomerReceipt,
            &receipt.header.document_id,
            DocumentType::CustomerInvoice,
            &invoice.header.document_id,
            ReferenceType::Payment,
            &receipt.header.company_code,
            payment_date,
        ));

        // Post the receipt
        receipt.post(created_by, payment_date);

        receipt
    }

    /// Generate multiple O2C chains.
    pub fn generate_chains(
        &mut self,
        count: usize,
        company_code: &str,
        customers: &CustomerPool,
        materials: &MaterialPool,
        date_range: (NaiveDate, NaiveDate),
        fiscal_year: u16,
        created_by: &str,
    ) -> Vec<O2CDocumentChain> {
        tracing::debug!(count, company_code, "Generating O2C document chains");
        let mut chains = Vec::new();

        let (start_date, end_date) = date_range;
        let days_range = (end_date - start_date).num_days() as u64;

        for _ in 0..count {
            // Select random customer
            let customer_idx = self.rng.random_range(0..customers.customers.len());
            let customer = &customers.customers[customer_idx];

            // Select random materials (1-5 items per SO)
            let num_items = self.rng.random_range(1..=5).min(materials.materials.len());
            let selected_materials: Vec<&Material> = materials
                .materials
                .iter()
                .choose_multiple(&mut self.rng, num_items)
                .into_iter()
                .collect();

            // Select random SO date
            let so_date =
                start_date + chrono::Duration::days(self.rng.random_range(0..=days_range) as i64);
            let fiscal_period = self.get_fiscal_period(so_date);

            let chain = self.generate_chain(
                company_code,
                customer,
                &selected_materials,
                so_date,
                fiscal_year,
                fiscal_period,
                created_by,
            );

            chains.push(chain);
        }

        chains
    }

    /// Perform credit check for customer.
    fn perform_credit_check(&mut self, customer: &Customer, order_amount: Decimal) -> bool {
        // Check credit limit
        if !customer.can_place_order(order_amount) {
            return false;
        }

        // Additional random failure based on config
        let fail_roll = self.rng.random::<f64>();
        if fail_roll < self.config.credit_check_failure_rate {
            return false;
        }

        // Higher risk customers have higher failure rate
        let additional_fail_rate = match customer.credit_rating {
            CreditRating::CCC | CreditRating::D => 0.20,
            CreditRating::B | CreditRating::BB => 0.05,
            _ => 0.0,
        };

        self.rng.random::<f64>() >= additional_fail_rate
    }

    /// Calculate delivery date from SO date.
    fn calculate_delivery_date(&mut self, so_date: NaiveDate) -> NaiveDate {
        let variance = self.rng.random_range(0..3) as i64;
        so_date + chrono::Duration::days(self.config.avg_days_so_to_delivery as i64 + variance)
    }

    /// Calculate invoice date from delivery date.
    fn calculate_invoice_date(&mut self, delivery_date: NaiveDate) -> NaiveDate {
        let variance = self.rng.random_range(0..2) as i64;
        delivery_date
            + chrono::Duration::days(self.config.avg_days_delivery_to_invoice as i64 + variance)
    }

    /// Calculate payment date based on customer behavior.
    fn calculate_payment_date(
        &mut self,
        invoice_date: NaiveDate,
        payment_terms: &PaymentTerms,
        customer: &Customer,
    ) -> NaiveDate {
        let base_days = payment_terms.net_days() as i64;

        // Adjust based on customer payment behavior
        let behavior_adjustment = match customer.payment_behavior {
            datasynth_core::models::CustomerPaymentBehavior::Excellent
            | datasynth_core::models::CustomerPaymentBehavior::EarlyPayer => {
                -self.rng.random_range(5..15) as i64
            }
            datasynth_core::models::CustomerPaymentBehavior::Good
            | datasynth_core::models::CustomerPaymentBehavior::OnTime => {
                self.rng.random_range(-2..3) as i64
            }
            datasynth_core::models::CustomerPaymentBehavior::Fair
            | datasynth_core::models::CustomerPaymentBehavior::SlightlyLate => {
                self.rng.random_range(5..15) as i64
            }
            datasynth_core::models::CustomerPaymentBehavior::Poor
            | datasynth_core::models::CustomerPaymentBehavior::OftenLate => {
                self.rng.random_range(15..45) as i64
            }
            datasynth_core::models::CustomerPaymentBehavior::VeryPoor
            | datasynth_core::models::CustomerPaymentBehavior::HighRisk => {
                self.rng.random_range(30..90) as i64
            }
        };

        // Additional random late payment
        let late_adjustment = if self.rng.random::<f64>() < self.config.late_payment_rate {
            self.rng.random_range(10..30) as i64
        } else {
            0
        };

        invoice_date + chrono::Duration::days(base_days + behavior_adjustment + late_adjustment)
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

    /// Get fiscal period from date.
    fn get_fiscal_period(&self, date: NaiveDate) -> u8 {
        date.month() as u8
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.so_counter = 0;
        self.dlv_counter = 0;
        self.ci_counter = 0;
        self.rec_counter = 0;
        self.short_payment_counter = 0;
        self.on_account_counter = 0;
        self.correction_counter = 0;
    }

    /// Generate a partial payment for an invoice.
    pub fn generate_partial_payment(
        &mut self,
        invoice: &CustomerInvoice,
        company_code: &str,
        customer: &Customer,
        payment_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
        payment_percent: f64,
    ) -> (Payment, Decimal, Option<NaiveDate>) {
        self.rec_counter += 1;

        let receipt_id =
            self.make_doc_id("REC", "customer_receipt", company_code, self.rec_counter);

        let full_amount = invoice.amount_open;
        let payment_amount = (full_amount
            * Decimal::from_f64_retain(payment_percent).unwrap_or(Decimal::ONE))
        .round_dp(2);
        let remaining_amount = full_amount - payment_amount;

        let mut receipt = Payment::new_ar_receipt(
            receipt_id,
            company_code,
            &customer.customer_id,
            payment_amount,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_payment_method(self.select_payment_method())
        .with_value_date(payment_date);

        // Allocate partial amount to invoice
        receipt.allocate_to_invoice(
            &invoice.header.document_id,
            DocumentType::CustomerInvoice,
            payment_amount,
            Decimal::ZERO, // No discount on partial payments
        );

        // Add document reference
        receipt.header.add_reference(DocumentReference::new(
            DocumentType::CustomerReceipt,
            &receipt.header.document_id,
            DocumentType::CustomerInvoice,
            &invoice.header.document_id,
            ReferenceType::Payment,
            &receipt.header.company_code,
            payment_date,
        ));

        receipt.post(created_by, payment_date);

        // Calculate expected remainder date
        let expected_remainder_date = Some(
            payment_date
                + chrono::Duration::days(
                    self.config.payment_behavior.avg_days_until_remainder as i64,
                )
                + chrono::Duration::days(self.rng.random_range(-7..7) as i64),
        );

        (receipt, remaining_amount, expected_remainder_date)
    }

    /// Generate a remainder payment for a partial payment.
    pub fn generate_remainder_payment(
        &mut self,
        invoice: &CustomerInvoice,
        company_code: &str,
        customer: &Customer,
        payment_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
        amount: Decimal,
    ) -> Payment {
        self.rec_counter += 1;

        let receipt_id =
            self.make_doc_id("REC", "customer_receipt", company_code, self.rec_counter);

        let mut receipt = Payment::new_ar_receipt(
            receipt_id,
            company_code,
            &customer.customer_id,
            amount,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_payment_method(self.select_payment_method())
        .with_value_date(payment_date);

        // Allocate remainder amount to invoice
        receipt.allocate_to_invoice(
            &invoice.header.document_id,
            DocumentType::CustomerInvoice,
            amount,
            Decimal::ZERO, // No discount on remainder payments
        );

        // Add document reference linking receipt to invoice
        receipt.header.add_reference(DocumentReference::new(
            DocumentType::CustomerReceipt,
            &receipt.header.document_id,
            DocumentType::CustomerInvoice,
            &invoice.header.document_id,
            ReferenceType::Payment,
            &receipt.header.company_code,
            payment_date,
        ));

        // Post the receipt
        receipt.post(created_by, payment_date);

        receipt
    }

    /// Generate a short payment for an invoice.
    pub fn generate_short_payment(
        &mut self,
        invoice: &CustomerInvoice,
        company_code: &str,
        customer: &Customer,
        payment_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
    ) -> (Payment, ShortPayment) {
        self.rec_counter += 1;
        self.short_payment_counter += 1;

        let receipt_id =
            self.make_doc_id("REC", "customer_receipt", company_code, self.rec_counter);
        let short_id = format!("SHORT-{}-{:06}", company_code, self.short_payment_counter);

        let full_amount = invoice.amount_open;

        // Calculate short amount (1-10% of invoice)
        let short_percent =
            self.rng.random::<f64>() * self.config.payment_behavior.max_short_percent;
        let short_amount = (full_amount
            * Decimal::from_f64_retain(short_percent).unwrap_or(Decimal::ZERO))
        .round_dp(2)
        .max(Decimal::ONE); // At least $1 short

        let payment_amount = full_amount - short_amount;

        let mut receipt = Payment::new_ar_receipt(
            receipt_id.clone(),
            company_code,
            &customer.customer_id,
            payment_amount,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_payment_method(self.select_payment_method())
        .with_value_date(payment_date);

        // Allocate to invoice
        receipt.allocate_to_invoice(
            &invoice.header.document_id,
            DocumentType::CustomerInvoice,
            payment_amount,
            Decimal::ZERO,
        );

        receipt.header.add_reference(DocumentReference::new(
            DocumentType::CustomerReceipt,
            &receipt.header.document_id,
            DocumentType::CustomerInvoice,
            &invoice.header.document_id,
            ReferenceType::Payment,
            &receipt.header.company_code,
            payment_date,
        ));

        receipt.post(created_by, payment_date);

        // Create short payment record
        let reason_code = self.select_short_payment_reason();
        let short_payment = ShortPayment::new(
            short_id,
            company_code.to_string(),
            customer.customer_id.clone(),
            receipt_id,
            invoice.header.document_id.clone(),
            full_amount,
            payment_amount,
            invoice.header.currency.clone(),
            payment_date,
            reason_code,
        );

        (receipt, short_payment)
    }

    /// Generate an on-account payment.
    pub fn generate_on_account_payment(
        &mut self,
        company_code: &str,
        customer: &Customer,
        payment_date: NaiveDate,
        fiscal_year: u16,
        fiscal_period: u8,
        created_by: &str,
        currency: &str,
        amount: Decimal,
    ) -> (Payment, OnAccountPayment) {
        self.rec_counter += 1;
        self.on_account_counter += 1;

        let receipt_id =
            self.make_doc_id("REC", "customer_receipt", company_code, self.rec_counter);
        let on_account_id = format!("OA-{}-{:06}", company_code, self.on_account_counter);

        let mut receipt = Payment::new_ar_receipt(
            receipt_id.clone(),
            company_code,
            &customer.customer_id,
            amount,
            fiscal_year,
            fiscal_period,
            payment_date,
            created_by,
        )
        .with_payment_method(self.select_payment_method())
        .with_value_date(payment_date);

        // On-account payments are not allocated to any invoice
        receipt.post(created_by, payment_date);

        // Create on-account payment record
        let reason = self.select_on_account_reason();
        let on_account = OnAccountPayment::new(
            on_account_id,
            company_code.to_string(),
            customer.customer_id.clone(),
            receipt_id,
            amount,
            currency.to_string(),
            payment_date,
        )
        .with_reason(reason);

        (receipt, on_account)
    }

    /// Generate a payment correction (NSF or chargeback).
    pub fn generate_payment_correction(
        &mut self,
        original_payment: &Payment,
        company_code: &str,
        customer_id: &str,
        correction_date: NaiveDate,
        currency: &str,
    ) -> PaymentCorrection {
        self.correction_counter += 1;

        let correction_id = format!("CORR-{}-{:06}", company_code, self.correction_counter);

        let correction_type = if self.rng.random::<f64>() < 0.6 {
            PaymentCorrectionType::NSF
        } else {
            PaymentCorrectionType::Chargeback
        };

        let mut correction = PaymentCorrection::new(
            correction_id,
            company_code.to_string(),
            customer_id.to_string(),
            original_payment.header.document_id.clone(),
            correction_type,
            original_payment.amount,
            original_payment.amount, // Full reversal
            currency.to_string(),
            correction_date,
        );

        // Set appropriate details based on type
        match correction_type {
            PaymentCorrectionType::NSF => {
                correction.bank_reference = Some(format!("NSF-{}", self.rng.random::<u32>()));
                correction.fee_amount = Decimal::from(35); // Standard NSF fee
                correction.reason = Some("Payment returned - Insufficient funds".to_string());
            }
            PaymentCorrectionType::Chargeback => {
                correction.chargeback_code =
                    Some(format!("CB{:04}", self.rng.random_range(1000..9999)));
                correction.reason = Some("Credit card chargeback".to_string());
            }
            _ => {}
        }

        // Add affected invoice
        if let Some(allocation) = original_payment.allocations.first() {
            correction.add_affected_invoice(allocation.invoice_id.clone());
        }

        correction
    }

    /// Select a random short payment reason code.
    fn select_short_payment_reason(&mut self) -> ShortPaymentReasonCode {
        let roll: f64 = self.rng.random();
        if roll < 0.30 {
            ShortPaymentReasonCode::PricingDispute
        } else if roll < 0.50 {
            ShortPaymentReasonCode::QualityIssue
        } else if roll < 0.70 {
            ShortPaymentReasonCode::QuantityDiscrepancy
        } else if roll < 0.85 {
            ShortPaymentReasonCode::UnauthorizedDeduction
        } else {
            ShortPaymentReasonCode::IncorrectDiscount
        }
    }

    /// Select a random on-account reason.
    fn select_on_account_reason(&mut self) -> OnAccountReason {
        let roll: f64 = self.rng.random();
        if roll < 0.40 {
            OnAccountReason::NoInvoiceReference
        } else if roll < 0.60 {
            OnAccountReason::Overpayment
        } else if roll < 0.75 {
            OnAccountReason::Prepayment
        } else if roll < 0.90 {
            OnAccountReason::UnclearRemittance
        } else {
            OnAccountReason::Other
        }
    }

    /// Determine the payment type based on configuration.
    fn determine_payment_type(&mut self) -> PaymentType {
        let roll: f64 = self.rng.random();
        let pb = &self.config.payment_behavior;

        let mut cumulative = 0.0;

        cumulative += pb.partial_payment_rate;
        if roll < cumulative {
            return PaymentType::Partial;
        }

        cumulative += pb.short_payment_rate;
        if roll < cumulative {
            return PaymentType::Short;
        }

        cumulative += pb.on_account_rate;
        if roll < cumulative {
            return PaymentType::OnAccount;
        }

        PaymentType::Full
    }

    /// Determine partial payment percentage.
    fn determine_partial_payment_percent(&mut self) -> f64 {
        let roll: f64 = self.rng.random();
        if roll < 0.15 {
            0.25
        } else if roll < 0.65 {
            0.50
        } else if roll < 0.90 {
            0.75
        } else {
            // Random between 30-80%
            0.30 + self.rng.random::<f64>() * 0.50
        }
    }
}

/// Type of payment to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaymentType {
    Full,
    Partial,
    Short,
    OnAccount,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{CustomerPaymentBehavior, MaterialType};

    fn create_test_customer() -> Customer {
        let mut customer = Customer::new(
            "C-000001",
            "Test Customer Inc.",
            datasynth_core::models::CustomerType::Corporate,
        );
        customer.credit_rating = CreditRating::A;
        customer.credit_limit = Decimal::from(1_000_000);
        customer.payment_behavior = CustomerPaymentBehavior::OnTime;
        customer
    }

    fn create_test_materials() -> Vec<Material> {
        let mut mat1 = Material::new("MAT-001", "Test Product 1", MaterialType::FinishedGood);
        mat1.list_price = Decimal::from(100);
        mat1.standard_cost = Decimal::from(60);

        let mut mat2 = Material::new("MAT-002", "Test Product 2", MaterialType::FinishedGood);
        mat2.list_price = Decimal::from(200);
        mat2.standard_cost = Decimal::from(120);

        vec![mat1, mat2]
    }

    #[test]
    fn test_o2c_chain_generation() {
        let mut gen = O2CGenerator::new(42);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert!(!chain.sales_order.items.is_empty());
        assert!(chain.credit_check_passed);
        assert!(!chain.deliveries.is_empty());
        assert!(chain.customer_invoice.is_some());
    }

    #[test]
    fn test_sales_order_generation() {
        let mut gen = O2CGenerator::new(42);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let so = gen.generate_sales_order(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert_eq!(so.customer_id, "C-000001");
        assert_eq!(so.items.len(), 2);
        assert!(so.total_net_amount > Decimal::ZERO);
    }

    #[test]
    fn test_credit_check_failure() {
        let config = O2CGeneratorConfig {
            credit_check_failure_rate: 1.0, // Force failure
            ..Default::default()
        };

        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert!(!chain.credit_check_passed);
        assert!(chain.deliveries.is_empty());
        assert!(chain.customer_invoice.is_none());
    }

    #[test]
    fn test_document_references() {
        let mut gen = O2CGenerator::new(42);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        // Delivery should reference SO
        if let Some(dlv) = chain.deliveries.first() {
            assert!(!dlv.header.document_references.is_empty());
        }

        // Invoice should reference SO and Delivery
        if let Some(invoice) = &chain.customer_invoice {
            assert!(invoice.header.document_references.len() >= 2);
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let mut gen1 = O2CGenerator::new(42);
        let mut gen2 = O2CGenerator::new(42);

        let chain1 = gen1.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );
        let chain2 = gen2.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert_eq!(
            chain1.sales_order.header.document_id,
            chain2.sales_order.header.document_id
        );
        assert_eq!(
            chain1.sales_order.total_net_amount,
            chain2.sales_order.total_net_amount
        );
    }

    #[test]
    fn test_partial_shipment_config() {
        let config = O2CGeneratorConfig {
            partial_shipment_rate: 1.0, // Force partial shipment
            ..Default::default()
        };

        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        // Should have multiple deliveries due to partial shipment
        assert!(chain.deliveries.len() >= 2);
    }

    #[test]
    fn test_gross_margin() {
        let mut gen = O2CGenerator::new(42);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        if let Some(invoice) = &chain.customer_invoice {
            // Gross margin should be positive (revenue > COGS)
            let margin = invoice.gross_margin();
            assert!(margin > Decimal::ZERO, "Gross margin should be positive");
        }
    }

    #[test]
    fn test_partial_payment_generates_remainder() {
        let config = O2CGeneratorConfig {
            bad_debt_rate: 0.0, // Ensure payment happens
            payment_behavior: O2CPaymentBehavior {
                partial_payment_rate: 1.0, // Force partial payment
                short_payment_rate: 0.0,
                on_account_rate: 0.0,
                payment_correction_rate: 0.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        // Should have both PartialPayment and RemainderPayment events
        let has_partial = chain
            .payment_events
            .iter()
            .any(|e| matches!(e, PaymentEvent::PartialPayment { .. }));
        let has_remainder = chain
            .payment_events
            .iter()
            .any(|e| matches!(e, PaymentEvent::RemainderPayment(_)));

        assert!(has_partial, "Should have a PartialPayment event");
        assert!(has_remainder, "Should have a RemainderPayment event");
        assert!(
            chain.payment_events.len() >= 2,
            "Should have at least 2 payment events (partial + remainder)"
        );
    }

    #[test]
    fn test_partial_plus_remainder_equals_invoice_total() {
        let config = O2CGeneratorConfig {
            bad_debt_rate: 0.0,
            payment_behavior: O2CPaymentBehavior {
                partial_payment_rate: 1.0,
                short_payment_rate: 0.0,
                on_account_rate: 0.0,
                payment_correction_rate: 0.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        let invoice = chain
            .customer_invoice
            .as_ref()
            .expect("Should have an invoice");

        // Extract partial payment amount
        let partial_amount = chain
            .payment_events
            .iter()
            .find_map(|e| {
                if let PaymentEvent::PartialPayment { payment, .. } = e {
                    Some(payment.amount)
                } else {
                    None
                }
            })
            .expect("Should have a partial payment");

        // Extract remainder payment amount
        let remainder_amount = chain
            .payment_events
            .iter()
            .find_map(|e| {
                if let PaymentEvent::RemainderPayment(payment) = e {
                    Some(payment.amount)
                } else {
                    None
                }
            })
            .expect("Should have a remainder payment");

        // partial + remainder should equal invoice total
        let total_paid = partial_amount + remainder_amount;
        assert_eq!(
            total_paid, invoice.total_gross_amount,
            "Partial ({}) + remainder ({}) = {} should equal invoice total ({})",
            partial_amount, remainder_amount, total_paid, invoice.total_gross_amount
        );
    }

    #[test]
    fn test_remainder_receipts_vec_populated() {
        let config = O2CGeneratorConfig {
            bad_debt_rate: 0.0,
            payment_behavior: O2CPaymentBehavior {
                partial_payment_rate: 1.0,
                short_payment_rate: 0.0,
                on_account_rate: 0.0,
                payment_correction_rate: 0.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert!(
            !chain.remainder_receipts.is_empty(),
            "remainder_receipts should be populated for partial payment chains"
        );
        assert_eq!(
            chain.remainder_receipts.len(),
            1,
            "Should have exactly one remainder receipt"
        );
    }

    #[test]
    fn test_remainder_date_after_partial_date() {
        let config = O2CGeneratorConfig {
            bad_debt_rate: 0.0,
            payment_behavior: O2CPaymentBehavior {
                partial_payment_rate: 1.0,
                short_payment_rate: 0.0,
                max_short_percent: 0.0,
                on_account_rate: 0.0,
                payment_correction_rate: 0.0,
                avg_days_until_remainder: 30,
            },
            ..Default::default()
        };

        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        // Get partial payment date (use value_date which is always set)
        let partial_date = chain
            .payment_events
            .iter()
            .find_map(|e| {
                if let PaymentEvent::PartialPayment { payment, .. } = e {
                    Some(payment.value_date)
                } else {
                    None
                }
            })
            .expect("Should have a partial payment");

        // Get remainder payment date
        let remainder_date = chain
            .payment_events
            .iter()
            .find_map(|e| {
                if let PaymentEvent::RemainderPayment(payment) = e {
                    Some(payment.value_date)
                } else {
                    None
                }
            })
            .expect("Should have a remainder payment");

        assert!(
            remainder_date > partial_date,
            "Remainder date ({}) should be after partial payment date ({})",
            remainder_date,
            partial_date
        );
    }

    #[test]
    fn test_partial_payment_chain_is_complete() {
        let config = O2CGeneratorConfig {
            bad_debt_rate: 0.0,
            payment_behavior: O2CPaymentBehavior {
                partial_payment_rate: 1.0,
                short_payment_rate: 0.0,
                on_account_rate: 0.0,
                payment_correction_rate: 0.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        // With both partial and remainder, chain should be complete
        assert!(
            chain.is_complete,
            "Chain with partial + remainder payment should be marked complete"
        );
    }

    #[test]
    fn test_non_partial_chain_has_empty_remainder_receipts() {
        let config = O2CGeneratorConfig {
            bad_debt_rate: 0.0,
            payment_behavior: O2CPaymentBehavior {
                partial_payment_rate: 0.0, // No partial payments
                short_payment_rate: 0.0,
                on_account_rate: 0.0,
                payment_correction_rate: 0.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert!(
            chain.remainder_receipts.is_empty(),
            "Non-partial payment chains should have empty remainder_receipts"
        );
    }

    #[test]
    fn test_o2c_returns_rate_generates_credit_memos() {
        let mut config = O2CGeneratorConfig::default();
        config.returns_rate = 1.0; // Force all chains to have returns
        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        assert!(chain.credit_check_passed);
        assert!(chain.is_return);
        assert!(chain.credit_memo.is_some());
    }

    #[test]
    fn test_credit_memo_references_invoice() {
        let mut config = O2CGeneratorConfig::default();
        config.returns_rate = 1.0;
        let mut gen = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        let chain = gen.generate_chain(
            "1000",
            &customer,
            &material_refs,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            2024,
            1,
            "JSMITH",
        );

        let memo = chain.credit_memo.as_ref().unwrap();
        let invoice = chain.customer_invoice.as_ref().unwrap();
        assert_eq!(
            memo.reference_invoice.as_deref(),
            Some(invoice.header.document_id.as_str())
        );
    }

    #[test]
    fn test_credit_memo_amount_bounded() {
        let mut config = O2CGeneratorConfig::default();
        config.returns_rate = 1.0;
        let _ = O2CGenerator::with_config(42, config);
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        for seed in 0..10 {
            let mut gen = O2CGenerator::with_config(seed, {
                let mut c = O2CGeneratorConfig::default();
                c.returns_rate = 1.0;
                c
            });
            let chain = gen.generate_chain(
                "1000",
                &customer,
                &material_refs,
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                2024,
                1,
                "JSMITH",
            );
            if let (Some(memo), Some(invoice)) = (&chain.credit_memo, &chain.customer_invoice) {
                assert!(
                    memo.gross_amount.document_amount <= invoice.total_gross_amount,
                    "Credit memo gross {:?} exceeds invoice gross {}",
                    memo.gross_amount.document_amount,
                    invoice.total_gross_amount
                );
            }
        }
    }

    #[test]
    fn test_zero_returns_rate() {
        let customer = create_test_customer();
        let materials = create_test_materials();
        let material_refs: Vec<&Material> = materials.iter().collect();

        for seed in 0..20 {
            let mut gen = O2CGenerator::with_config(seed, {
                let mut c = O2CGeneratorConfig::default();
                c.returns_rate = 0.0;
                c
            });
            let chain = gen.generate_chain(
                "1000",
                &customer,
                &material_refs,
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                2024,
                1,
                "JSMITH",
            );
            assert!(chain.credit_memo.is_none(), "No credit memos with returns_rate=0");
            assert!(!chain.is_return);
        }
    }
}
