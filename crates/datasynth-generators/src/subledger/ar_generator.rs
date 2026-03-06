//! AR (Accounts Receivable) generator.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use tracing::debug;

use datasynth_core::accounts::{cash_accounts, control_accounts, revenue_accounts, tax_accounts};
use datasynth_core::models::subledger::ar::{
    ARCreditMemo, ARCreditMemoLine, ARInvoice, ARInvoiceLine, ARReceipt, CreditMemoReason,
    PaymentMethod,
};
use datasynth_core::models::subledger::PaymentTerms;
use datasynth_core::models::{JournalEntry, JournalEntryLine};

/// Configuration for AR generation.
#[derive(Debug, Clone)]
pub struct ARGeneratorConfig {
    /// Average invoice amount.
    pub avg_invoice_amount: Decimal,
    /// Invoice amount variation (0.0 to 1.0).
    pub amount_variation: Decimal,
    /// Percentage of invoices paid on time.
    pub on_time_payment_rate: Decimal,
    /// Average days to payment.
    pub avg_days_to_payment: u32,
    /// Percentage of invoices that get credit memos.
    pub credit_memo_rate: Decimal,
    /// Default tax rate.
    pub tax_rate: Decimal,
    /// Default payment terms.
    pub default_terms: PaymentTerms,
}

impl Default for ARGeneratorConfig {
    fn default() -> Self {
        Self {
            avg_invoice_amount: dec!(5000),
            amount_variation: dec!(0.5),
            avg_days_to_payment: 35,
            on_time_payment_rate: dec!(0.75),
            credit_memo_rate: dec!(0.05),
            tax_rate: dec!(10),
            default_terms: PaymentTerms::net_30(),
        }
    }
}

/// Generator for AR transactions.
pub struct ARGenerator {
    config: ARGeneratorConfig,
    rng: ChaCha8Rng,
    invoice_counter: u64,
    receipt_counter: u64,
    credit_memo_counter: u64,
}

impl ARGenerator {
    /// Creates a new AR generator.
    pub fn new(config: ARGeneratorConfig, rng: ChaCha8Rng) -> Self {
        Self {
            config,
            rng,
            invoice_counter: 0,
            receipt_counter: 0,
            credit_memo_counter: 0,
        }
    }

    /// Creates a new AR generator from a seed, constructing the RNG internally.
    pub fn with_seed(config: ARGeneratorConfig, seed: u64) -> Self {
        Self::new(config, seeded_rng(seed, 0))
    }

    /// Generates an AR invoice.
    pub fn generate_invoice(
        &mut self,
        company_code: &str,
        customer_id: &str,
        customer_name: &str,
        invoice_date: NaiveDate,
        currency: &str,
        line_count: usize,
    ) -> (ARInvoice, JournalEntry) {
        self.invoice_counter += 1;
        let invoice_number = format!("ARINV{:08}", self.invoice_counter);

        let mut invoice = ARInvoice::new(
            invoice_number.clone(),
            company_code.to_string(),
            customer_id.to_string(),
            customer_name.to_string(),
            invoice_date,
            self.config.default_terms.clone(),
            currency.to_string(),
        );

        // Generate invoice lines
        for line_num in 1..=line_count {
            let amount = self.generate_line_amount();
            let line = ARInvoiceLine::new(
                line_num as u32,
                format!("Product/Service {line_num}"),
                dec!(1),
                "EA".to_string(),
                amount,
                revenue_accounts::PRODUCT_REVENUE.to_string(),
            )
            .with_tax("VAT".to_string(), self.config.tax_rate);

            invoice.add_line(line);
        }

        // Generate corresponding journal entry
        let je = self.generate_invoice_je(&invoice);

        (invoice, je)
    }

    /// Generates a receipt for an invoice.
    pub fn generate_receipt(
        &mut self,
        invoice: &ARInvoice,
        receipt_date: NaiveDate,
        amount: Option<Decimal>,
    ) -> (ARReceipt, JournalEntry) {
        self.receipt_counter += 1;
        let receipt_number = format!("ARREC{:08}", self.receipt_counter);

        let payment_amount = amount.unwrap_or(invoice.amount_remaining);
        let discount = invoice.available_discount(receipt_date);
        let net_payment = payment_amount - discount;

        let payment_method = self.random_payment_method();

        let mut receipt = ARReceipt::new(
            receipt_number.clone(),
            invoice.company_code.clone(),
            invoice.customer_id.clone(),
            invoice.customer_name.clone(),
            receipt_date,
            net_payment,
            invoice.gross_amount.document_currency.clone(),
            payment_method,
            cash_accounts::OPERATING_CASH.to_string(),
        );

        receipt.apply_to_invoice(invoice.invoice_number.clone(), payment_amount, discount);

        // Generate corresponding journal entry
        let je = self.generate_receipt_je(&receipt, &invoice.gross_amount.document_currency);

        (receipt, je)
    }

    /// Generates a credit memo.
    pub fn generate_credit_memo(
        &mut self,
        invoice: &ARInvoice,
        memo_date: NaiveDate,
        reason: CreditMemoReason,
        percent_of_invoice: Decimal,
    ) -> (ARCreditMemo, JournalEntry) {
        self.credit_memo_counter += 1;
        let memo_number = format!("ARCM{:08}", self.credit_memo_counter);

        let mut memo = ARCreditMemo::for_invoice(
            memo_number.clone(),
            invoice.company_code.clone(),
            invoice.customer_id.clone(),
            invoice.customer_name.clone(),
            memo_date,
            invoice.invoice_number.clone(),
            reason,
            format!("{reason:?}"),
            invoice.gross_amount.document_currency.clone(),
        );

        // Add credit memo lines proportional to original invoice
        for (idx, inv_line) in invoice.lines.iter().enumerate() {
            let line = ARCreditMemoLine::new(
                (idx + 1) as u32,
                inv_line.description.clone(),
                inv_line.quantity * percent_of_invoice,
                inv_line.unit.clone(),
                inv_line.unit_price,
                inv_line.revenue_account.clone(),
            )
            .with_tax(
                inv_line.tax_code.clone().unwrap_or_default(),
                inv_line.tax_rate,
            )
            .with_invoice_reference(inv_line.line_number);

            memo.add_line(line);
        }

        // Generate corresponding journal entry
        let je = self.generate_credit_memo_je(&memo);

        (memo, je)
    }

    /// Generates a batch of AR transactions for a period.
    pub fn generate_period_transactions(
        &mut self,
        company_code: &str,
        customers: &[(String, String)], // (id, name)
        start_date: NaiveDate,
        end_date: NaiveDate,
        invoices_per_day: u32,
        currency: &str,
    ) -> ARPeriodTransactions {
        debug!(company_code, customer_count = customers.len(), %start_date, %end_date, invoices_per_day, "Generating AR period transactions");
        let mut invoices = Vec::new();
        let mut receipts = Vec::new();
        let mut credit_memos = Vec::new();
        let mut journal_entries = Vec::new();

        let mut current_date = start_date;
        while current_date <= end_date {
            // Generate invoices for this day
            for _ in 0..invoices_per_day {
                if customers.is_empty() {
                    continue;
                }

                let customer_idx = self.rng.random_range(0..customers.len());
                let (customer_id, customer_name) = &customers[customer_idx];

                let line_count = self.rng.random_range(1..=5);
                let (invoice, je) = self.generate_invoice(
                    company_code,
                    customer_id,
                    customer_name,
                    current_date,
                    currency,
                    line_count,
                );

                journal_entries.push(je);
                invoices.push(invoice);
            }

            current_date += chrono::Duration::days(1);
        }

        // Generate receipts for older invoices
        let payment_cutoff =
            end_date - chrono::Duration::days(self.config.avg_days_to_payment as i64);
        for invoice in &invoices {
            if invoice.invoice_date <= payment_cutoff {
                let should_pay: f64 = self.rng.random();
                if should_pay
                    < self
                        .config
                        .on_time_payment_rate
                        .to_string()
                        .parse()
                        .unwrap_or(0.75)
                {
                    let days_to_pay = self.rng.random_range(
                        (self.config.avg_days_to_payment / 2)
                            ..(self.config.avg_days_to_payment * 2),
                    );
                    let receipt_date =
                        invoice.invoice_date + chrono::Duration::days(days_to_pay as i64);

                    if receipt_date <= end_date {
                        let (receipt, je) = self.generate_receipt(invoice, receipt_date, None);
                        journal_entries.push(je);
                        receipts.push(receipt);
                    }
                }
            }
        }

        // Generate some credit memos
        for invoice in &invoices {
            let should_credit: f64 = self.rng.random();
            if should_credit
                < self
                    .config
                    .credit_memo_rate
                    .to_string()
                    .parse()
                    .unwrap_or(0.05)
            {
                let days_after = self.rng.random_range(5..30);
                let memo_date = invoice.invoice_date + chrono::Duration::days(days_after);

                if memo_date <= end_date {
                    let reason = self.random_credit_reason();
                    let percent = Decimal::from(self.rng.random_range(10..50)) / dec!(100);
                    let (memo, je) = self.generate_credit_memo(invoice, memo_date, reason, percent);
                    journal_entries.push(je);
                    credit_memos.push(memo);
                }
            }
        }

        ARPeriodTransactions {
            invoices,
            receipts,
            credit_memos,
            journal_entries,
        }
    }

    /// Generates a random line amount.
    fn generate_line_amount(&mut self) -> Decimal {
        let base = self.config.avg_invoice_amount;
        let variation = base * self.config.amount_variation;
        let random: f64 = self.rng.random_range(-1.0..1.0);
        let amount = base + variation * Decimal::try_from(random).unwrap_or_default();
        amount.max(dec!(100)).round_dp(2)
    }

    /// Generates invoice journal entry.
    fn generate_invoice_je(&mut self, invoice: &ARInvoice) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-{}", invoice.invoice_number),
            invoice.company_code.clone(),
            invoice.posting_date,
            format!("AR Invoice {}", invoice.invoice_number),
        );

        // Debit AR (using centralized control account)
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: control_accounts::AR_CONTROL.to_string(),
            debit_amount: invoice.gross_amount.document_amount,
            reference: Some(invoice.invoice_number.clone()),
            assignment: Some(invoice.customer_id.clone()),
            ..Default::default()
        });

        // Credit Revenue (using centralized revenue account)
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: revenue_accounts::PRODUCT_REVENUE.to_string(),
            credit_amount: invoice.net_amount.document_amount,
            reference: Some(invoice.invoice_number.clone()),
            ..Default::default()
        });

        // Credit Tax Payable (using centralized tax account)
        if invoice.tax_amount.document_amount > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: 3,
                gl_account: tax_accounts::VAT_PAYABLE.to_string(),
                credit_amount: invoice.tax_amount.document_amount,
                reference: Some(invoice.invoice_number.clone()),
                tax_code: Some("VAT".to_string()),
                ..Default::default()
            });
        }

        je
    }

    /// Generates receipt journal entry.
    fn generate_receipt_je(&mut self, receipt: &ARReceipt, _currency: &str) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-{}", receipt.receipt_number),
            receipt.company_code.clone(),
            receipt.posting_date,
            format!("AR Receipt {}", receipt.receipt_number),
        );

        // Debit Cash (using centralized cash account)
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: cash_accounts::OPERATING_CASH.to_string(),
            debit_amount: receipt.amount.document_amount,
            reference: Some(receipt.receipt_number.clone()),
            ..Default::default()
        });

        // Credit AR (using centralized control account)
        let ar_credit = receipt.net_applied + receipt.discount_taken;
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: control_accounts::AR_CONTROL.to_string(),
            credit_amount: ar_credit,
            reference: Some(receipt.receipt_number.clone()),
            assignment: Some(receipt.customer_id.clone()),
            ..Default::default()
        });

        // Debit Discount Expense if discount taken
        if receipt.discount_taken > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: 3,
                gl_account: revenue_accounts::SALES_DISCOUNTS.to_string(),
                debit_amount: receipt.discount_taken,
                reference: Some(receipt.receipt_number.clone()),
                ..Default::default()
            });
        }

        je
    }

    /// Generates credit memo journal entry.
    fn generate_credit_memo_je(&mut self, memo: &ARCreditMemo) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-{}", memo.credit_memo_number),
            memo.company_code.clone(),
            memo.posting_date,
            format!("AR Credit Memo {}", memo.credit_memo_number),
        );

        // Debit Revenue
        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: revenue_accounts::PRODUCT_REVENUE.to_string(),
            debit_amount: memo.net_amount.document_amount,
            reference: Some(memo.credit_memo_number.clone()),
            ..Default::default()
        });

        // Debit Tax
        if memo.tax_amount.document_amount > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: tax_accounts::SALES_TAX_PAYABLE.to_string(),
                debit_amount: memo.tax_amount.document_amount,
                reference: Some(memo.credit_memo_number.clone()),
                tax_code: Some("VAT".to_string()),
                ..Default::default()
            });
        }

        // Credit AR
        je.add_line(JournalEntryLine {
            line_number: 3,
            gl_account: control_accounts::AR_CONTROL.to_string(),
            credit_amount: memo.gross_amount.document_amount,
            reference: Some(memo.credit_memo_number.clone()),
            assignment: Some(memo.customer_id.clone()),
            ..Default::default()
        });

        je
    }

    /// Generates a random payment method.
    fn random_payment_method(&mut self) -> PaymentMethod {
        match self.rng.random_range(0..4) {
            0 => PaymentMethod::WireTransfer,
            1 => PaymentMethod::Check,
            2 => PaymentMethod::ACH,
            _ => PaymentMethod::CreditCard,
        }
    }

    /// Generates a random credit memo reason.
    fn random_credit_reason(&mut self) -> CreditMemoReason {
        match self.rng.random_range(0..5) {
            0 => CreditMemoReason::Return,
            1 => CreditMemoReason::PriceError,
            2 => CreditMemoReason::QualityIssue,
            3 => CreditMemoReason::Promotional,
            _ => CreditMemoReason::Other,
        }
    }
}

/// Result of period AR generation.
#[derive(Debug, Clone)]
pub struct ARPeriodTransactions {
    /// Generated invoices.
    pub invoices: Vec<ARInvoice>,
    /// Generated receipts.
    pub receipts: Vec<ARReceipt>,
    /// Generated credit memos.
    pub credit_memos: Vec<ARCreditMemo>,
    /// Corresponding journal entries.
    pub journal_entries: Vec<JournalEntry>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_generate_invoice() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = ARGenerator::new(ARGeneratorConfig::default(), rng);

        let (invoice, je) = generator.generate_invoice(
            "1000",
            "CUST001",
            "Test Customer",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "USD",
            3,
        );

        assert_eq!(invoice.lines.len(), 3);
        assert!(invoice.gross_amount.document_amount > Decimal::ZERO);
        assert!(je.is_balanced());
    }

    #[test]
    fn test_generate_receipt() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let mut generator = ARGenerator::new(ARGeneratorConfig::default(), rng);

        let (invoice, _) = generator.generate_invoice(
            "1000",
            "CUST001",
            "Test Customer",
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "USD",
            2,
        );

        let (receipt, je) = generator.generate_receipt(
            &invoice,
            NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
            None,
        );

        assert!(receipt.net_applied > Decimal::ZERO);
        assert!(je.is_balanced());
    }
}
