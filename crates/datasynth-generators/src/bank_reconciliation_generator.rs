//! Bank reconciliation generator.
//!
//! Generates realistic bank reconciliations with statement lines, matched/unmatched
//! items, and reconciling items (outstanding checks, deposits in transit, bank
//! charges, interest). The reconciliation balances such that:
//!
//! ```text
//! closing_balance = opening_balance + total_credits - total_debits
//! adjusted_bank_balance == adjusted_book_balance  (when Completed)
//! ```

use chrono::NaiveDate;
use datasynth_core::models::{
    BankReconciliation, BankStatementLine, Direction, MatchStatus, ReconciliationStatus,
    ReconcilingItem, ReconcilingItemType,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A reference to an internal payment/receipt to be matched against bank statement lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentReference {
    /// Internal payment/receipt document ID.
    pub id: String,
    /// Payment amount (positive for outflows, negative for inflows).
    pub amount: Decimal,
    /// Date the payment was recorded in the books.
    pub date: NaiveDate,
    /// Human-readable reference (e.g., check number, wire ref).
    pub reference: String,
}

/// Configuration knobs for bank reconciliation generation.
#[derive(Debug, Clone)]
pub struct BankReconciliationConfig {
    /// Fraction of statement lines auto-matched to payments (0.0 - 1.0).
    pub auto_match_rate: f64,
    /// Fraction manually matched (0.0 - 1.0).
    pub manual_match_rate: f64,
    /// Minimum opening balance.
    pub min_opening_balance: f64,
    /// Maximum opening balance.
    pub max_opening_balance: f64,
    /// Number of extra bank-only lines (charges, interest, misc) to generate.
    pub extra_bank_lines: usize,
    /// Probability that a reconciliation completes without exceptions.
    pub completion_rate: f64,
}

impl Default for BankReconciliationConfig {
    fn default() -> Self {
        Self {
            auto_match_rate: 0.70,
            manual_match_rate: 0.15,
            min_opening_balance: 50_000.0,
            max_opening_balance: 500_000.0,
            extra_bank_lines: 5,
            completion_rate: 0.80,
        }
    }
}

/// Generates [`BankReconciliation`] instances with realistic statement lines,
/// matching behaviour, and reconciling items.
pub struct BankReconciliationGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    line_uuid_factory: DeterministicUuidFactory,
    recon_item_uuid_factory: DeterministicUuidFactory,
    config: BankReconciliationConfig,
}

impl BankReconciliationGenerator {
    /// Create a new generator with default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::BankReconciliation),
            line_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::BankReconciliation,
                1,
            ),
            recon_item_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::BankReconciliation,
                2,
            ),
            config: BankReconciliationConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: BankReconciliationConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::BankReconciliation),
            line_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::BankReconciliation,
                1,
            ),
            recon_item_uuid_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::BankReconciliation,
                2,
            ),
            config,
        }
    }

    /// Generate a bank reconciliation for the given account and period.
    ///
    /// # Arguments
    ///
    /// * `company_code` - The company code owning the bank account.
    /// * `bank_account_id` - The bank account identifier.
    /// * `period_start` - Start of the reconciliation period (inclusive).
    /// * `period_end` - End of the reconciliation period (inclusive).
    /// * `currency` - ISO 4217 currency code.
    /// * `payments` - Internal payment/receipt records to match against.
    pub fn generate(
        &mut self,
        company_code: &str,
        bank_account_id: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
        currency: &str,
        payments: &[PaymentReference],
    ) -> BankReconciliation {
        let reconciliation_id = self.uuid_factory.next().to_string();

        // --- Opening balance ---
        let opening_balance = self.random_opening_balance();

        // --- Build statement lines from payments ---
        let mut statement_lines: Vec<BankStatementLine> = Vec::new();
        let mut reconciling_items: Vec<ReconcilingItem> = Vec::new();

        for payment in payments {
            let roll: f64 = self.rng.gen();
            let auto_threshold = self.config.auto_match_rate;
            let manual_threshold = auto_threshold + self.config.manual_match_rate;

            if roll < auto_threshold {
                // Auto-matched: appears on the bank statement, auto-linked
                statement_lines.push(self.payment_to_statement_line(
                    payment,
                    bank_account_id,
                    company_code,
                    MatchStatus::AutoMatched,
                ));
            } else if roll < manual_threshold {
                // Manually matched: appears on the bank statement, manually linked
                statement_lines.push(self.payment_to_statement_line(
                    payment,
                    bank_account_id,
                    company_code,
                    MatchStatus::ManuallyMatched,
                ));
            } else {
                // Unmatched: payment is in the books but not on the bank statement.
                // This creates a reconciling item (outstanding check or deposit in transit).
                let item_type = if payment.amount > Decimal::ZERO {
                    ReconcilingItemType::OutstandingCheck
                } else {
                    ReconcilingItemType::DepositInTransit
                };
                let clearing_days = self.rng.gen_range(1..=10);
                reconciling_items.push(ReconcilingItem {
                    item_id: self.recon_item_uuid_factory.next().to_string(),
                    item_type,
                    document_id: Some(payment.id.clone()),
                    amount: payment.amount.abs(),
                    date: payment.date,
                    description: format!(
                        "{}: {} ({})",
                        match item_type {
                            ReconcilingItemType::OutstandingCheck => "Outstanding check",
                            ReconcilingItemType::DepositInTransit => "Deposit in transit",
                            _ => "Item",
                        },
                        payment.reference,
                        currency,
                    ),
                    expected_clearing_date: Some(
                        period_end + chrono::Duration::days(clearing_days),
                    ),
                });
            }
        }

        // --- Extra bank-only lines (charges, interest, misc) ---
        let extra_count = self.config.extra_bank_lines;
        for _ in 0..extra_count {
            let (line, maybe_recon) = self.generate_bank_only_line(
                bank_account_id,
                company_code,
                period_start,
                period_end,
                currency,
            );
            statement_lines.push(line);
            if let Some(ri) = maybe_recon {
                reconciling_items.push(ri);
            }
        }

        // Sort statement lines by date for realism.
        statement_lines.sort_by_key(|l| l.statement_date);

        // --- Compute balances ---
        // closing_balance = opening_balance + total_credits - total_debits
        let mut total_credits = Decimal::ZERO;
        let mut total_debits = Decimal::ZERO;
        for line in &statement_lines {
            match line.direction {
                Direction::Inflow => total_credits += line.amount,
                Direction::Outflow => total_debits += line.amount,
            }
        }
        let bank_ending_balance = opening_balance + total_credits - total_debits;

        // Adjusted bank balance: bank_ending_balance
        //   - outstanding checks (subtract, because bank hasn't paid them yet)
        //   + deposits in transit (add, because bank hasn't credited them yet)
        let mut bank_adjustment = Decimal::ZERO;
        for ri in &reconciling_items {
            match ri.item_type {
                ReconcilingItemType::OutstandingCheck => bank_adjustment -= ri.amount,
                ReconcilingItemType::DepositInTransit => bank_adjustment += ri.amount,
                _ => {}
            }
        }
        let adjusted_bank_balance = bank_ending_balance + bank_adjustment;

        // Adjusted book balance: book_ending_balance
        //   = adjusted_bank_balance (when the reconciliation completes cleanly)
        //   For that we derive book_ending_balance by working backwards through
        //   book-side reconciling items (bank charges, interest not yet booked).
        let mut book_adjustment = Decimal::ZERO;
        for ri in &reconciling_items {
            match ri.item_type {
                ReconcilingItemType::BankCharge | ReconcilingItemType::ReturnedCheck => {
                    book_adjustment -= ri.amount;
                }
                ReconcilingItemType::InterestEarned => {
                    book_adjustment += ri.amount;
                }
                _ => {}
            }
        }
        // book_ending_balance + book_adjustment = adjusted_bank_balance
        // so book_ending_balance = adjusted_bank_balance - book_adjustment
        let book_ending_balance = adjusted_bank_balance - book_adjustment;

        // --- Status ---
        let has_unmatched = statement_lines
            .iter()
            .any(|l| l.match_status == MatchStatus::Unmatched);

        let status = if has_unmatched {
            ReconciliationStatus::CompletedWithExceptions
        } else if self.rng.gen_bool(self.config.completion_rate) {
            ReconciliationStatus::Completed
        } else {
            ReconciliationStatus::InProgress
        };

        // Net difference should be zero when fully reconciled.
        let net_difference = adjusted_bank_balance - (book_ending_balance + book_adjustment);

        // Preparer / reviewer
        let preparer_id = format!("USR-{:04}", self.rng.gen_range(1..=200));
        let reviewer_id = if status == ReconciliationStatus::Completed {
            Some(format!("USR-{:04}", self.rng.gen_range(201..=400)))
        } else {
            None
        };

        BankReconciliation {
            reconciliation_id,
            bank_account_id: bank_account_id.to_string(),
            company_code: company_code.to_string(),
            reconciliation_date: period_end,
            status,
            bank_ending_balance,
            book_ending_balance,
            statement_lines,
            reconciling_items,
            net_difference,
            opening_balance,
            preparer_id,
            reviewer_id,
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Generate a random opening balance using the configured range.
    fn random_opening_balance(&mut self) -> Decimal {
        let raw: f64 = self
            .rng
            .gen_range(self.config.min_opening_balance..=self.config.max_opening_balance);
        Decimal::from_f64_retain(raw)
            .unwrap_or(Decimal::ZERO)
            .round_dp(2)
    }

    /// Convert an internal payment reference into a bank statement line.
    fn payment_to_statement_line(
        &mut self,
        payment: &PaymentReference,
        bank_account_id: &str,
        company_code: &str,
        match_status: MatchStatus,
    ) -> BankStatementLine {
        // Positive payment amount = outflow (company pays), negative = inflow (receipt).
        let (direction, amount) = if payment.amount >= Decimal::ZERO {
            (Direction::Outflow, payment.amount)
        } else {
            (Direction::Inflow, payment.amount.abs())
        };

        // Value date may lag statement date by 0-2 business days.
        let lag_days = self.rng.gen_range(0..=2);
        let value_date = payment.date + chrono::Duration::days(lag_days);

        let bank_ref = format!(
            "BNK-{}-{:06}",
            payment.date.format("%Y%m%d"),
            self.rng.gen_range(1..=999_999)
        );

        BankStatementLine {
            line_id: self.line_uuid_factory.next().to_string(),
            bank_account_id: bank_account_id.to_string(),
            statement_date: payment.date,
            value_date,
            amount,
            direction,
            description: format!("Payment ref {}", payment.reference),
            bank_reference: bank_ref,
            match_status,
            matched_document_id: Some(payment.id.clone()),
            company_code: company_code.to_string(),
        }
    }

    /// Generate a bank-originated line (charge, interest, or miscellaneous)
    /// that does not correspond to an internal payment.
    /// Returns the statement line and an optional reconciling item for amounts
    /// not yet recorded in the books.
    fn generate_bank_only_line(
        &mut self,
        bank_account_id: &str,
        company_code: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
        currency: &str,
    ) -> (BankStatementLine, Option<ReconcilingItem>) {
        let days_in_period = (period_end - period_start).num_days().max(1);
        let offset = self.rng.gen_range(0..=days_in_period);
        let statement_date = period_start + chrono::Duration::days(offset);

        // Pick a bank-only category.
        let category_roll: f64 = self.rng.gen();
        let (match_status, direction, amount_range, desc, recon_type) = if category_roll < 0.40 {
            // Bank service charge
            (
                MatchStatus::BankCharge,
                Direction::Outflow,
                (15.0, 150.0),
                "Monthly service charge",
                Some(ReconcilingItemType::BankCharge),
            )
        } else if category_roll < 0.70 {
            // Interest earned
            (
                MatchStatus::Interest,
                Direction::Inflow,
                (5.0, 500.0),
                "Interest earned",
                Some(ReconcilingItemType::InterestEarned),
            )
        } else if category_roll < 0.85 {
            // NSF / returned check
            (
                MatchStatus::Unmatched,
                Direction::Outflow,
                (100.0, 5000.0),
                "Returned check / NSF",
                Some(ReconcilingItemType::ReturnedCheck),
            )
        } else {
            // Miscellaneous unmatched debit/credit
            let is_debit = self.rng.gen_bool(0.5);
            if is_debit {
                (
                    MatchStatus::Unmatched,
                    Direction::Outflow,
                    (50.0, 2000.0),
                    "Miscellaneous bank debit",
                    None,
                )
            } else {
                (
                    MatchStatus::Unmatched,
                    Direction::Inflow,
                    (50.0, 2000.0),
                    "Miscellaneous bank credit",
                    None,
                )
            }
        };

        let raw_amount: f64 = self.rng.gen_range(amount_range.0..=amount_range.1);
        let amount = Decimal::from_f64_retain(raw_amount)
            .unwrap_or(Decimal::ONE)
            .round_dp(2);

        let bank_ref = format!(
            "BNK-{}-{:06}",
            statement_date.format("%Y%m%d"),
            self.rng.gen_range(1..=999_999)
        );

        let line = BankStatementLine {
            line_id: self.line_uuid_factory.next().to_string(),
            bank_account_id: bank_account_id.to_string(),
            statement_date,
            value_date: statement_date,
            amount,
            direction,
            description: desc.to_string(),
            bank_reference: bank_ref,
            match_status,
            matched_document_id: None,
            company_code: company_code.to_string(),
        };

        let recon_item = recon_type.map(|rt| ReconcilingItem {
            item_id: self.recon_item_uuid_factory.next().to_string(),
            item_type: rt,
            document_id: None,
            amount,
            date: statement_date,
            description: format!("{} ({})", desc, currency),
            expected_clearing_date: if rt == ReconcilingItemType::ReturnedCheck {
                Some(period_end + chrono::Duration::days(self.rng.gen_range(3..=14) as i64))
            } else {
                // Charges/interest settle immediately on the bank side.
                None
            },
        });

        (line, recon_item)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Helper: build a small set of payment references for testing.
    fn sample_payments(period_start: NaiveDate) -> Vec<PaymentReference> {
        vec![
            PaymentReference {
                id: "PAY-001".to_string(),
                amount: Decimal::new(1_500_00, 2), // 1500.00 outflow
                date: period_start + chrono::Duration::days(3),
                reference: "CHK-10001".to_string(),
            },
            PaymentReference {
                id: "PAY-002".to_string(),
                amount: Decimal::new(-2_000_00, 2), // -2000.00 inflow (receipt)
                date: period_start + chrono::Duration::days(7),
                reference: "WIRE-20001".to_string(),
            },
            PaymentReference {
                id: "PAY-003".to_string(),
                amount: Decimal::new(750_00, 2), // 750.00 outflow
                date: period_start + chrono::Duration::days(12),
                reference: "CHK-10002".to_string(),
            },
            PaymentReference {
                id: "PAY-004".to_string(),
                amount: Decimal::new(-3_500_00, 2), // -3500.00 inflow
                date: period_start + chrono::Duration::days(18),
                reference: "WIRE-20002".to_string(),
            },
            PaymentReference {
                id: "PAY-005".to_string(),
                amount: Decimal::new(4_200_00, 2), // 4200.00 outflow
                date: period_start + chrono::Duration::days(22),
                reference: "CHK-10003".to_string(),
            },
            PaymentReference {
                id: "PAY-006".to_string(),
                amount: Decimal::new(-1_800_00, 2), // -1800.00 inflow
                date: period_start + chrono::Duration::days(25),
                reference: "ACH-30001".to_string(),
            },
            PaymentReference {
                id: "PAY-007".to_string(),
                amount: Decimal::new(600_00, 2),
                date: period_start + chrono::Duration::days(28),
                reference: "CHK-10004".to_string(),
            },
            PaymentReference {
                id: "PAY-008".to_string(),
                amount: Decimal::new(-900_00, 2),
                date: period_start + chrono::Duration::days(5),
                reference: "WIRE-20003".to_string(),
            },
            PaymentReference {
                id: "PAY-009".to_string(),
                amount: Decimal::new(2_100_00, 2),
                date: period_start + chrono::Duration::days(15),
                reference: "CHK-10005".to_string(),
            },
            PaymentReference {
                id: "PAY-010".to_string(),
                amount: Decimal::new(-6_000_00, 2),
                date: period_start + chrono::Duration::days(20),
                reference: "WIRE-20004".to_string(),
            },
        ]
    }

    #[test]
    fn test_basic_generation_produces_valid_reconciliation() {
        let mut gen = BankReconciliationGenerator::new(42);
        let period_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let payments = sample_payments(period_start);

        let recon = gen.generate("C001", "BA-001", period_start, period_end, "USD", &payments);

        // Basic field checks
        assert!(!recon.reconciliation_id.is_empty());
        assert_eq!(recon.company_code, "C001");
        assert_eq!(recon.bank_account_id, "BA-001");
        assert_eq!(recon.reconciliation_date, period_end);
        assert!(recon.opening_balance > Decimal::ZERO);
        assert!(!recon.statement_lines.is_empty());
        assert!(!recon.preparer_id.is_empty());

        // Statement lines should include at least some of the payments plus extras.
        // (Some payments become reconciling items instead of statement lines.)
        let matched_count = recon
            .statement_lines
            .iter()
            .filter(|l| l.matched_document_id.is_some())
            .count();
        assert!(
            matched_count > 0,
            "Expected at least one matched statement line"
        );

        // Should have reconciling items (from unmatched payments + bank-only lines).
        assert!(
            !recon.reconciling_items.is_empty(),
            "Expected at least one reconciling item"
        );
    }

    #[test]
    fn test_statement_lines_balance() {
        let mut gen = BankReconciliationGenerator::new(99);
        let period_start = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let payments = sample_payments(period_start);

        let recon = gen.generate("C001", "BA-002", period_start, period_end, "USD", &payments);

        // closing_balance = opening_balance + total_credits - total_debits
        let mut total_credits = Decimal::ZERO;
        let mut total_debits = Decimal::ZERO;
        for line in &recon.statement_lines {
            match line.direction {
                Direction::Inflow => total_credits += line.amount,
                Direction::Outflow => total_debits += line.amount,
            }
        }

        let expected_closing = recon.opening_balance + total_credits - total_debits;
        assert_eq!(
            recon.bank_ending_balance,
            expected_closing,
            "Bank ending balance must equal opening + credits - debits. \
             opening={}, credits={}, debits={}, expected={}, actual={}",
            recon.opening_balance,
            total_credits,
            total_debits,
            expected_closing,
            recon.bank_ending_balance,
        );
    }

    #[test]
    fn test_deterministic_output_with_same_seed() {
        let period_start = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let payments = sample_payments(period_start);

        let mut gen1 = BankReconciliationGenerator::new(12345);
        let recon1 = gen1.generate("C001", "BA-003", period_start, period_end, "EUR", &payments);

        let mut gen2 = BankReconciliationGenerator::new(12345);
        let recon2 = gen2.generate("C001", "BA-003", period_start, period_end, "EUR", &payments);

        assert_eq!(recon1.reconciliation_id, recon2.reconciliation_id);
        assert_eq!(recon1.opening_balance, recon2.opening_balance);
        assert_eq!(recon1.bank_ending_balance, recon2.bank_ending_balance);
        assert_eq!(recon1.book_ending_balance, recon2.book_ending_balance);
        assert_eq!(recon1.statement_lines.len(), recon2.statement_lines.len());
        assert_eq!(
            recon1.reconciling_items.len(),
            recon2.reconciling_items.len()
        );

        // Verify line-by-line determinism.
        for (l1, l2) in recon1
            .statement_lines
            .iter()
            .zip(recon2.statement_lines.iter())
        {
            assert_eq!(l1.line_id, l2.line_id);
            assert_eq!(l1.amount, l2.amount);
            assert_eq!(l1.direction, l2.direction);
            assert_eq!(l1.match_status, l2.match_status);
        }
    }

    #[test]
    fn test_mix_of_matched_and_unmatched_items() {
        // Use a seed that will exercise the probability buckets with enough payments.
        let mut gen = BankReconciliationGenerator::new(777);
        let period_start = NaiveDate::from_ymd_opt(2024, 9, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();

        // Generate 20 payments to increase the chance of hitting all buckets.
        let mut payments: Vec<PaymentReference> = Vec::new();
        for i in 0..20 {
            let day_offset = (i % 28) + 1;
            let sign = if i % 3 == 0 {
                Decimal::NEGATIVE_ONE
            } else {
                Decimal::ONE
            };
            payments.push(PaymentReference {
                id: format!("PAY-{:03}", i + 1),
                amount: sign * Decimal::new((1000 + i * 500) as i64, 0),
                date: period_start + chrono::Duration::days(day_offset),
                reference: format!("REF-{:05}", i + 1),
            });
        }

        let recon = gen.generate("C002", "BA-010", period_start, period_end, "USD", &payments);

        let auto_count = recon
            .statement_lines
            .iter()
            .filter(|l| l.match_status == MatchStatus::AutoMatched)
            .count();
        let manual_count = recon
            .statement_lines
            .iter()
            .filter(|l| l.match_status == MatchStatus::ManuallyMatched)
            .count();
        let bank_charge_count = recon
            .statement_lines
            .iter()
            .filter(|l| l.match_status == MatchStatus::BankCharge)
            .count();
        let interest_count = recon
            .statement_lines
            .iter()
            .filter(|l| l.match_status == MatchStatus::Interest)
            .count();
        let unmatched_line_count = recon
            .statement_lines
            .iter()
            .filter(|l| l.match_status == MatchStatus::Unmatched)
            .count();

        // With 20 payments at default rates (70/15/15) we expect a mix.
        assert!(
            auto_count > 0,
            "Expected at least one auto-matched line, got 0"
        );
        // manual or unmatched (reconciling items) should exist
        assert!(
            manual_count > 0 || !recon.reconciling_items.is_empty(),
            "Expected manual matches or reconciling items from unmatched payments"
        );
        // Bank-only lines should contribute charges or interest.
        assert!(
            bank_charge_count + interest_count + unmatched_line_count > 0,
            "Expected at least one bank-only line"
        );

        // Reconciling items should include outstanding checks or deposits in transit
        // from the ~15% unmatched payment bucket.
        let outstanding_or_transit = recon
            .reconciling_items
            .iter()
            .filter(|ri| {
                ri.item_type == ReconcilingItemType::OutstandingCheck
                    || ri.item_type == ReconcilingItemType::DepositInTransit
            })
            .count();
        assert!(
            outstanding_or_transit > 0,
            "Expected outstanding checks or deposits in transit from unmatched payments"
        );
    }

    #[test]
    fn test_empty_payments_still_produces_reconciliation() {
        let mut gen = BankReconciliationGenerator::new(55);
        let period_start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();

        let recon = gen.generate("C001", "BA-005", period_start, period_end, "GBP", &[]);

        // Even with no payments, we should get bank-only lines.
        assert!(
            !recon.statement_lines.is_empty(),
            "Extra bank-only lines should be generated even with no payments"
        );
        assert!(recon.opening_balance > Decimal::ZERO);
    }

    #[test]
    fn test_statement_lines_sorted_by_date() {
        let mut gen = BankReconciliationGenerator::new(101);
        let period_start = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let period_end = NaiveDate::from_ymd_opt(2024, 4, 30).unwrap();
        let payments = sample_payments(period_start);

        let recon = gen.generate("C001", "BA-006", period_start, period_end, "USD", &payments);

        for window in recon.statement_lines.windows(2) {
            assert!(
                window[0].statement_date <= window[1].statement_date,
                "Statement lines should be sorted by date: {} > {}",
                window[0].statement_date,
                window[1].statement_date,
            );
        }
    }
}
