//! Dunning (Mahnungen) generator.
//!
//! Generates dunning runs and letters for overdue AR invoices,
//! simulating the realistic dunning process including:
//! - Multi-level dunning (reminders, final notices, collection)
//! - Payment responses after dunning
//! - Interest and charge calculations

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_core::accounts::control_accounts;
use datasynth_core::models::subledger::ar::{
    ARInvoice, CustomerDunningSummary, DunningItem, DunningLetter, DunningResponseType, DunningRun,
};
use datasynth_core::models::subledger::SubledgerDocumentStatus;
use datasynth_core::models::{JournalEntry, JournalEntryLine};

/// Configuration for dunning generation.
#[derive(Debug, Clone)]
pub struct DunningGeneratorConfig {
    /// Days overdue for level 1 dunning.
    pub level_1_days_overdue: u32,
    /// Days overdue for level 2 dunning.
    pub level_2_days_overdue: u32,
    /// Days overdue for level 3 dunning.
    pub level_3_days_overdue: u32,
    /// Days overdue for collection handover.
    pub collection_days_overdue: u32,
    /// Payment rate after level 1 reminder.
    pub payment_rate_after_level_1: f64,
    /// Payment rate after level 2 reminder.
    pub payment_rate_after_level_2: f64,
    /// Payment rate after level 3 final notice.
    pub payment_rate_after_level_3: f64,
    /// Payment rate during collection.
    pub payment_rate_during_collection: f64,
    /// Rate that never pays (becomes bad debt).
    pub never_pay_rate: f64,
    /// Rate of invoices blocked from dunning (disputes).
    pub dunning_block_rate: f64,
    /// Annual interest rate for overdue amounts.
    pub interest_rate_per_year: f64,
    /// Fixed charge per dunning letter.
    pub dunning_charge_per_letter: Decimal,
    /// Days between dunning run and payment deadline.
    pub payment_deadline_days: u32,
}

impl Default for DunningGeneratorConfig {
    fn default() -> Self {
        Self {
            level_1_days_overdue: 14,
            level_2_days_overdue: 28,
            level_3_days_overdue: 42,
            collection_days_overdue: 60,
            payment_rate_after_level_1: 0.40,
            payment_rate_after_level_2: 0.30,
            payment_rate_after_level_3: 0.15,
            payment_rate_during_collection: 0.05,
            never_pay_rate: 0.10,
            dunning_block_rate: 0.05,
            interest_rate_per_year: 0.09,
            dunning_charge_per_letter: dec!(25),
            payment_deadline_days: 14,
        }
    }
}

/// Generator for dunning process.
pub struct DunningGenerator {
    config: DunningGeneratorConfig,
    rng: ChaCha8Rng,
    seed: u64,
    run_counter: u64,
    letter_counter: u64,
}

impl DunningGenerator {
    /// Creates a new dunning generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, DunningGeneratorConfig::default())
    }

    /// Creates a new dunning generator with custom configuration.
    pub fn with_config(seed: u64, config: DunningGeneratorConfig) -> Self {
        Self {
            config,
            rng: seeded_rng(seed, 0),
            seed,
            run_counter: 0,
            letter_counter: 0,
        }
    }

    /// Executes a dunning run for a given date.
    ///
    /// This evaluates all open invoices and generates dunning letters
    /// for those that meet the dunning criteria.
    pub fn execute_dunning_run(
        &mut self,
        company_code: &str,
        run_date: NaiveDate,
        invoices: &mut [ARInvoice],
        currency: &str,
    ) -> DunningRunResult {
        self.run_counter += 1;
        let run_id = format!("DR-{}-{:06}", company_code, self.run_counter);

        let mut run = DunningRun::new(run_id.clone(), company_code.to_string(), run_date);
        run.start();

        let mut letters = Vec::new();
        let mut journal_entries = Vec::new();
        let mut payment_simulations = Vec::new();

        // Group invoices by customer
        let mut customer_invoices: std::collections::HashMap<String, Vec<&mut ARInvoice>> =
            std::collections::HashMap::new();

        for invoice in invoices.iter_mut() {
            if invoice.company_code == company_code
                && matches!(
                    invoice.status,
                    SubledgerDocumentStatus::Open | SubledgerDocumentStatus::PartiallyCleared
                )
                && invoice.is_overdue(run_date)
            {
                customer_invoices
                    .entry(invoice.customer_id.clone())
                    .or_default()
                    .push(invoice);
            }
        }

        run.customers_evaluated = customer_invoices.len() as u32;

        // Process each customer
        for (customer_id, customer_invoices) in customer_invoices.iter_mut() {
            let customer_name = customer_invoices
                .first()
                .map(|i| i.customer_name.clone())
                .unwrap_or_default();

            // Determine highest dunning level needed
            let max_days_overdue = customer_invoices
                .iter()
                .map(|i| i.days_overdue(run_date) as u32)
                .max()
                .unwrap_or(0);

            let dunning_level = self.determine_dunning_level(max_days_overdue);

            if dunning_level == 0 {
                continue;
            }

            // Check if blocked from dunning
            if self.rng.random::<f64>() < self.config.dunning_block_rate {
                // Skip this customer - dunning blocked
                continue;
            }

            // Create dunning letter
            self.letter_counter += 1;
            let letter_id = format!("DL-{}-{:08}", company_code, self.letter_counter);

            let payment_deadline =
                run_date + chrono::Duration::days(self.config.payment_deadline_days as i64);

            let mut letter = DunningLetter::new(
                letter_id,
                run_id.clone(),
                company_code.to_string(),
                customer_id.clone(),
                customer_name,
                dunning_level,
                run_date,
                payment_deadline,
                currency.to_string(),
            );

            // Add dunning items
            let mut total_interest = Decimal::ZERO;
            for invoice in customer_invoices.iter_mut() {
                let days_overdue = invoice.days_overdue(run_date) as u32;
                let previous_level = invoice.dunning_info.dunning_level;
                let new_level = self.determine_dunning_level(days_overdue);

                // Calculate interest
                let interest = self.calculate_interest(
                    invoice.amount_remaining,
                    days_overdue,
                    self.config.interest_rate_per_year,
                );
                total_interest += interest;

                let item = DunningItem::new(
                    invoice.invoice_number.clone(),
                    invoice.invoice_date,
                    invoice.due_date,
                    invoice.gross_amount.document_amount,
                    invoice.amount_remaining,
                    days_overdue,
                    previous_level,
                    new_level,
                )
                .with_interest(interest);

                letter.add_item(item);

                // Update invoice dunning info
                invoice.dunning_info.advance_level(run_date, run_id.clone());
            }

            // Set charges and interest
            letter.set_charges(self.config.dunning_charge_per_letter);
            letter.set_interest(total_interest);

            // Mark as sent
            letter.mark_sent(run_date);

            // Generate journal entry for dunning charges and interest
            if letter.dunning_charges > Decimal::ZERO || letter.interest_amount > Decimal::ZERO {
                let je = self.generate_dunning_je(&letter, company_code, currency);
                journal_entries.push(je);
            }

            // Simulate payment response
            let response = self.simulate_payment_response(dunning_level);
            payment_simulations.push(PaymentSimulation {
                customer_id: customer_id.clone(),
                dunning_level,
                response,
                amount: letter.total_dunned_amount,
                expected_payment_date: self.calculate_expected_payment_date(run_date, response),
            });

            run.add_letter(letter.clone());
            letters.push(letter);
        }

        run.complete();

        DunningRunResult {
            dunning_run: run,
            letters,
            journal_entries,
            payment_simulations,
        }
    }

    /// Determines the dunning level based on days overdue.
    fn determine_dunning_level(&self, days_overdue: u32) -> u8 {
        if days_overdue >= self.config.collection_days_overdue {
            4
        } else if days_overdue >= self.config.level_3_days_overdue {
            3
        } else if days_overdue >= self.config.level_2_days_overdue {
            2
        } else if days_overdue >= self.config.level_1_days_overdue {
            1
        } else {
            0
        }
    }

    /// Calculates interest for an overdue amount.
    fn calculate_interest(&self, amount: Decimal, days_overdue: u32, annual_rate: f64) -> Decimal {
        let daily_rate = annual_rate / 365.0;
        let interest_factor = daily_rate * days_overdue as f64;
        (amount * Decimal::try_from(interest_factor).unwrap_or(Decimal::ZERO)).round_dp(2)
    }

    /// Simulates a payment response based on dunning level and configuration.
    fn simulate_payment_response(&mut self, dunning_level: u8) -> DunningResponseType {
        let roll: f64 = self.rng.random();

        // Calculate cumulative probabilities
        let p1 = self.config.payment_rate_after_level_1;
        let p2 = p1 + self.config.payment_rate_after_level_2;
        let p3 = p2 + self.config.payment_rate_after_level_3;
        let p4 = p3 + self.config.payment_rate_during_collection;
        // Remainder is never_pay

        match dunning_level {
            1 => {
                if roll < p1 {
                    DunningResponseType::Paid
                } else if roll < p1 + 0.05 {
                    DunningResponseType::PaymentPromise
                } else if roll < p1 + 0.10 {
                    DunningResponseType::Dispute
                } else {
                    DunningResponseType::NoResponse
                }
            }
            2 => {
                if roll < p2 - p1 {
                    DunningResponseType::Paid
                } else if roll < (p2 - p1) + 0.10 {
                    DunningResponseType::PaymentPromise
                } else if roll < (p2 - p1) + 0.15 {
                    DunningResponseType::PaymentPlan
                } else if roll < (p2 - p1) + 0.20 {
                    DunningResponseType::Dispute
                } else {
                    DunningResponseType::NoResponse
                }
            }
            3 => {
                if roll < p3 - p2 {
                    DunningResponseType::Paid
                } else if roll < (p3 - p2) + 0.05 {
                    DunningResponseType::PaymentPlan
                } else if roll < (p3 - p2) + 0.10 {
                    DunningResponseType::PartialDispute
                } else {
                    DunningResponseType::NoResponse
                }
            }
            4 => {
                if roll < p4 - p3 {
                    DunningResponseType::Paid
                } else if roll < (p4 - p3) + 0.02 {
                    DunningResponseType::Bankruptcy
                } else {
                    DunningResponseType::NoResponse
                }
            }
            _ => DunningResponseType::NoResponse,
        }
    }

    /// Calculates the expected payment date based on response type.
    fn calculate_expected_payment_date(
        &mut self,
        dunning_date: NaiveDate,
        response: DunningResponseType,
    ) -> Option<NaiveDate> {
        match response {
            DunningResponseType::Paid => {
                Some(dunning_date + chrono::Duration::days(self.rng.random_range(1..14) as i64))
            }
            DunningResponseType::PaymentPromise => {
                Some(dunning_date + chrono::Duration::days(self.rng.random_range(7..21) as i64))
            }
            DunningResponseType::PaymentPlan => {
                Some(dunning_date + chrono::Duration::days(self.rng.random_range(30..90) as i64))
            }
            _ => None,
        }
    }

    /// Generates a journal entry for dunning charges and interest.
    fn generate_dunning_je(
        &self,
        letter: &DunningLetter,
        company_code: &str,
        _currency: &str,
    ) -> JournalEntry {
        let mut je = JournalEntry::new_simple(
            format!("JE-DUNN-{}", letter.letter_id),
            company_code.to_string(),
            letter.dunning_date,
            format!("Dunning charges letter {}", letter.letter_id),
        );

        let mut line_num = 1;

        // Debit AR for charges and interest
        let total_receivable = letter.dunning_charges + letter.interest_amount;
        if total_receivable > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: control_accounts::AR_CONTROL.to_string(),
                debit_amount: total_receivable,
                reference: Some(letter.letter_id.clone()),
                assignment: Some(letter.customer_id.clone()),
                ..Default::default()
            });
            line_num += 1;
        }

        // Credit Dunning charges revenue
        if letter.dunning_charges > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: "4800".to_string(), // Other operating income
                credit_amount: letter.dunning_charges,
                reference: Some(letter.letter_id.clone()),
                ..Default::default()
            });
            line_num += 1;
        }

        // Credit Interest income
        if letter.interest_amount > Decimal::ZERO {
            je.add_line(JournalEntryLine {
                line_number: line_num,
                gl_account: "4810".to_string(), // Interest income
                credit_amount: letter.interest_amount,
                reference: Some(letter.letter_id.clone()),
                ..Default::default()
            });
        }

        je
    }

    /// Generates customer dunning summaries.
    pub fn generate_customer_summaries(
        &self,
        letters: &[DunningLetter],
    ) -> Vec<CustomerDunningSummary> {
        let customer_ids: std::collections::HashSet<_> =
            letters.iter().map(|l| l.customer_id.clone()).collect();

        customer_ids
            .into_iter()
            .map(|customer_id| {
                let customer_name = letters
                    .iter()
                    .find(|l| l.customer_id == customer_id)
                    .map(|l| l.customer_name.clone())
                    .unwrap_or_default();

                CustomerDunningSummary::from_letters(customer_id, customer_name, letters)
            })
            .collect()
    }

    /// Generates dunning runs for a period (e.g., monthly dunning).
    pub fn generate_period_dunning_runs(
        &mut self,
        company_code: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        invoices: &mut [ARInvoice],
        currency: &str,
        run_frequency_days: u32,
    ) -> Vec<DunningRunResult> {
        let mut results = Vec::new();
        let mut current_date = start_date;

        while current_date <= end_date {
            let result = self.execute_dunning_run(company_code, current_date, invoices, currency);
            results.push(result);

            current_date += chrono::Duration::days(run_frequency_days as i64);
        }

        results
    }

    /// Resets the generator.
    pub fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.run_counter = 0;
        self.letter_counter = 0;
    }
}

/// Result of a dunning run.
#[derive(Debug, Clone)]
pub struct DunningRunResult {
    /// The dunning run record.
    pub dunning_run: DunningRun,
    /// Letters generated.
    pub letters: Vec<DunningLetter>,
    /// Journal entries for charges and interest.
    pub journal_entries: Vec<JournalEntry>,
    /// Simulated payment responses.
    pub payment_simulations: Vec<PaymentSimulation>,
}

/// Simulated payment response.
#[derive(Debug, Clone)]
pub struct PaymentSimulation {
    /// Customer ID.
    pub customer_id: String,
    /// Dunning level.
    pub dunning_level: u8,
    /// Response type.
    pub response: DunningResponseType,
    /// Amount due.
    pub amount: Decimal,
    /// Expected payment date (if paying).
    pub expected_payment_date: Option<NaiveDate>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::subledger::ar::DunningRunStatus;
    use datasynth_core::models::subledger::{CurrencyAmount, PaymentTerms};

    fn create_test_invoice(
        invoice_number: &str,
        customer_id: &str,
        invoice_date: NaiveDate,
        due_date: NaiveDate,
        amount: Decimal,
    ) -> ARInvoice {
        let mut invoice = ARInvoice::new(
            invoice_number.to_string(),
            "1000".to_string(),
            customer_id.to_string(),
            format!("Customer {}", customer_id),
            invoice_date,
            PaymentTerms::net_30(),
            "USD".to_string(),
        );
        invoice.due_date = due_date;
        invoice.gross_amount = CurrencyAmount::single_currency(amount, "USD".to_string());
        invoice.amount_remaining = amount;
        invoice
    }

    #[test]
    fn test_dunning_level_determination() {
        let gen = DunningGenerator::new(42);

        assert_eq!(gen.determine_dunning_level(10), 0);
        assert_eq!(gen.determine_dunning_level(14), 1);
        assert_eq!(gen.determine_dunning_level(20), 1);
        assert_eq!(gen.determine_dunning_level(28), 2);
        assert_eq!(gen.determine_dunning_level(35), 2);
        assert_eq!(gen.determine_dunning_level(42), 3);
        assert_eq!(gen.determine_dunning_level(50), 3);
        assert_eq!(gen.determine_dunning_level(60), 4);
        assert_eq!(gen.determine_dunning_level(90), 4);
    }

    #[test]
    fn test_interest_calculation() {
        let gen = DunningGenerator::new(42);

        // $1000 at 9% annual for 30 days
        let interest = gen.calculate_interest(dec!(1000), 30, 0.09);
        // Expected: 1000 * (0.09/365) * 30 ≈ 7.40
        assert!(interest > dec!(7) && interest < dec!(8));
    }

    #[test]
    fn test_dunning_run_execution() {
        let mut gen = DunningGenerator::new(42);

        let run_date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let invoice_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let due_date = NaiveDate::from_ymd_opt(2024, 2, 14).unwrap();

        let mut invoices = vec![
            create_test_invoice("INV-001", "CUST001", invoice_date, due_date, dec!(1000)),
            create_test_invoice("INV-002", "CUST001", invoice_date, due_date, dec!(500)),
            create_test_invoice("INV-003", "CUST002", invoice_date, due_date, dec!(2000)),
        ];

        let result = gen.execute_dunning_run("1000", run_date, &mut invoices, "USD");

        assert_eq!(result.dunning_run.status, DunningRunStatus::Completed);
        assert!(!result.letters.is_empty());

        // Check dunning levels are appropriate (invoices are ~30 days overdue)
        for letter in &result.letters {
            assert!(letter.dunning_level >= 1);
            assert!(letter.dunning_level <= 2);
        }
    }

    #[test]
    fn test_dunning_charges_and_interest() {
        let mut gen = DunningGenerator::new(42);

        let run_date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let invoice_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let due_date = NaiveDate::from_ymd_opt(2024, 2, 14).unwrap();

        let mut invoices = vec![create_test_invoice(
            "INV-001",
            "CUST001",
            invoice_date,
            due_date,
            dec!(1000),
        )];

        let result = gen.execute_dunning_run("1000", run_date, &mut invoices, "USD");

        if let Some(letter) = result.letters.first() {
            assert_eq!(letter.dunning_charges, dec!(25)); // Default charge
            assert!(letter.interest_amount > Decimal::ZERO);
            assert!(letter.total_amount_due > letter.total_dunned_amount);
        }
    }

    #[test]
    fn test_deterministic_generation() {
        let run_date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let invoice_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let due_date = NaiveDate::from_ymd_opt(2024, 2, 14).unwrap();

        let create_invoices = || {
            vec![create_test_invoice(
                "INV-001",
                "CUST001",
                invoice_date,
                due_date,
                dec!(1000),
            )]
        };

        let mut gen1 = DunningGenerator::new(42);
        let mut gen2 = DunningGenerator::new(42);

        let mut invoices1 = create_invoices();
        let mut invoices2 = create_invoices();

        let result1 = gen1.execute_dunning_run("1000", run_date, &mut invoices1, "USD");
        let result2 = gen2.execute_dunning_run("1000", run_date, &mut invoices2, "USD");

        assert_eq!(result1.letters.len(), result2.letters.len());
        assert_eq!(
            result1.dunning_run.total_amount_dunned,
            result2.dunning_run.total_amount_dunned
        );
    }
}
