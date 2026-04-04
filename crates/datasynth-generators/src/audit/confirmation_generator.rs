//! Confirmation generator for audit engagements.
//!
//! Generates external confirmations and their responses per ISA 505.
//! Supports bank balance, accounts receivable, accounts payable, and
//! other confirmation types with realistic response distributions.

use std::collections::HashMap;

use chrono::Duration;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use datasynth_core::models::audit::{
    AuditEngagement, ConfirmationResponse, ConfirmationStatus, ConfirmationType,
    ExternalConfirmation, RecipientType, ResponseType, Workpaper, WorkpaperSection,
};

/// Configuration for external confirmation generation (ISA 505).
#[derive(Debug, Clone)]
pub struct ConfirmationGeneratorConfig {
    /// Number of confirmations per engagement (min, max)
    pub confirmations_per_engagement: (u32, u32),
    /// Fraction of confirmations that are bank balance type
    pub bank_balance_ratio: f64,
    /// Fraction of confirmations that are accounts receivable type
    pub accounts_receivable_ratio: f64,
    /// Fraction of confirmations that receive a clean "Confirmed" response
    pub confirmed_response_ratio: f64,
    /// Fraction of confirmations that receive a "ConfirmedWithException" response
    pub exception_response_ratio: f64,
    /// Fraction of confirmations that receive no reply
    pub no_response_ratio: f64,
    /// Of the exception responses, fraction that are subsequently reconciled
    pub exception_reconciled_ratio: f64,
}

impl Default for ConfirmationGeneratorConfig {
    fn default() -> Self {
        Self {
            confirmations_per_engagement: (5, 15),
            bank_balance_ratio: 0.25,
            accounts_receivable_ratio: 0.40,
            confirmed_response_ratio: 0.70,
            exception_response_ratio: 0.15,
            no_response_ratio: 0.10,
            exception_reconciled_ratio: 0.80,
        }
    }
}

/// Generator for external confirmations and responses per ISA 505.
pub struct ConfirmationGenerator {
    /// Seeded random number generator
    rng: ChaCha8Rng,
    /// Configuration
    config: ConfirmationGeneratorConfig,
    /// Monotone counter used for human-readable references
    confirmation_counter: u32,
}

impl ConfirmationGenerator {
    /// Create a new generator with the given seed and default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: ConfirmationGeneratorConfig::default(),
            confirmation_counter: 0,
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: ConfirmationGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            confirmation_counter: 0,
        }
    }

    /// Generate external confirmations and responses for an engagement.
    ///
    /// Returns a pair of vecs: `(confirmations, responses)`.  A response is
    /// generated for every confirmation that does *not* remain in `NoResponse`
    /// status, so the response vec may be shorter than the confirmation vec.
    ///
    /// # Arguments
    /// * `engagement` — The audit engagement these confirmations belong to.
    /// * `workpapers`  — Workpapers already generated for the engagement.  The
    ///   generator links each confirmation to a randomly chosen substantive
    ///   workpaper (if one exists).
    /// * `account_codes` — GL account codes available in the client data.  Each
    ///   confirmation will reference one of these codes when the slice is
    ///   non-empty.
    pub fn generate_confirmations(
        &mut self,
        engagement: &AuditEngagement,
        workpapers: &[Workpaper],
        account_codes: &[String],
    ) -> (Vec<ExternalConfirmation>, Vec<ConfirmationResponse>) {
        let count = self.rng.random_range(
            self.config.confirmations_per_engagement.0..=self.config.confirmations_per_engagement.1,
        ) as usize;

        // Collect substantive workpapers as candidate link targets.
        let substantive_wps: Vec<&Workpaper> = workpapers
            .iter()
            .filter(|wp| wp.section == WorkpaperSection::SubstantiveTesting)
            .collect();

        let mut confirmations = Vec::with_capacity(count);
        let mut responses = Vec::with_capacity(count);

        for i in 0..count {
            let (conf_type, recipient_type, recipient_name) =
                self.choose_confirmation_type(i, count);

            // Pick a random account code if available.
            let account_code: Option<String> = if account_codes.is_empty() {
                None
            } else {
                let idx = self.rng.random_range(0..account_codes.len());
                Some(account_codes[idx].clone())
            };

            // Realistic book balance: $10k – $5M
            let balance_units: i64 = self.rng.random_range(10_000_i64..=5_000_000_i64);
            let book_balance = Decimal::new(balance_units * 100, 2); // cents → dollars

            // Confirmation date = period end date (the balance date being confirmed).
            let confirmation_date = engagement.period_end_date;

            // Sent during fieldwork: random day in fieldwork window.
            let fieldwork_days = (engagement.fieldwork_end - engagement.fieldwork_start)
                .num_days()
                .max(1);
            let sent_offset = self.rng.random_range(0..fieldwork_days);
            let sent_date = engagement.fieldwork_start + Duration::days(sent_offset);
            let deadline = sent_date + Duration::days(30);

            self.confirmation_counter += 1;

            let mut confirmation = ExternalConfirmation::new(
                engagement.engagement_id,
                conf_type,
                &recipient_name,
                recipient_type,
                book_balance,
                confirmation_date,
            );

            // Override ref to include a sequential counter for readability.
            confirmation.confirmation_ref = format!(
                "CONF-{}-{:04}",
                engagement.fiscal_year, self.confirmation_counter
            );

            // Link to a substantive workpaper if one exists.
            if !substantive_wps.is_empty() {
                let wp_idx = self.rng.random_range(0..substantive_wps.len());
                confirmation = confirmation.with_workpaper(substantive_wps[wp_idx].workpaper_id);
            }

            // Attach account code.
            if let Some(ref code) = account_code {
                confirmation = confirmation.with_account(code);
            }

            // Mark as sent.
            confirmation.send(sent_date, deadline);

            // Determine response outcome using configured ratios.
            let roll: f64 = self.rng.random();
            let no_response_cutoff = self.config.no_response_ratio;
            let exception_cutoff = no_response_cutoff + self.config.exception_response_ratio;
            let confirmed_cutoff = exception_cutoff + self.config.confirmed_response_ratio;
            // Anything above confirmed_cutoff → Denied.

            if roll < no_response_cutoff {
                // No reply — confirmation stays without a response record.
                confirmation.status = ConfirmationStatus::NoResponse;
            } else {
                // A response was received.
                let response_days = self.rng.random_range(5_i64..=25_i64);
                let response_date = sent_date + Duration::days(response_days);

                let response_type = if roll < exception_cutoff {
                    ResponseType::ConfirmedWithException
                } else if roll < confirmed_cutoff {
                    ResponseType::Confirmed
                } else {
                    ResponseType::Denied
                };

                let mut response = ConfirmationResponse::new(
                    confirmation.confirmation_id,
                    engagement.engagement_id,
                    response_date,
                    response_type,
                );

                match response_type {
                    ResponseType::Confirmed => {
                        // Confirming party agrees with book balance exactly.
                        response = response.with_confirmed_balance(book_balance);
                        confirmation.status = ConfirmationStatus::Completed;
                    }
                    ResponseType::ConfirmedWithException => {
                        // Exception: confirmed balance differs by 1–8% of book.
                        let exception_pct: f64 = self.rng.random_range(0.01..0.08);
                        let exception_units = (balance_units as f64 * exception_pct).round() as i64;
                        let exception_amount = Decimal::new(exception_units.max(1) * 100, 2);
                        let confirmed_balance = book_balance - exception_amount;

                        response = response
                            .with_confirmed_balance(confirmed_balance)
                            .with_exception(
                                exception_amount,
                                self.exception_description(conf_type),
                            );

                        // Optionally reconcile the exception.
                        if self.rng.random::<f64>() < self.config.exception_reconciled_ratio {
                            response.reconcile(
                                "Difference investigated and reconciled to timing items \
								 — no audit adjustment required.",
                            );
                        }

                        confirmation.status = ConfirmationStatus::Completed;
                    }
                    ResponseType::Denied => {
                        // Confirming party disagrees; no confirmed balance set.
                        confirmation.status = ConfirmationStatus::AlternativeProcedures;
                    }
                    ResponseType::NoReply => {
                        // Handled above — should not reach here.
                        confirmation.status = ConfirmationStatus::NoResponse;
                    }
                }

                responses.push(response);
            }

            confirmations.push(confirmation);
        }

        (confirmations, responses)
    }

    /// Generate confirmations using real account balances for book values.
    ///
    /// When a confirmation is of type `BankBalance`, `AccountsReceivable`, or
    /// `AccountsPayable`, the book balance is derived from matching GL accounts
    /// in `account_balances` (keyed by GL account code).  If no matching
    /// balance is found, falls back to the existing synthetic generation.
    ///
    /// The response logic (confirmed, exception, no reply) is identical to the
    /// base method.
    pub fn generate_confirmations_with_balances(
        &mut self,
        engagement: &AuditEngagement,
        workpapers: &[Workpaper],
        account_codes: &[String],
        account_balances: &HashMap<String, f64>,
    ) -> (Vec<ExternalConfirmation>, Vec<ConfirmationResponse>) {
        let count = self.rng.random_range(
            self.config.confirmations_per_engagement.0..=self.config.confirmations_per_engagement.1,
        ) as usize;

        let substantive_wps: Vec<&Workpaper> = workpapers
            .iter()
            .filter(|wp| wp.section == WorkpaperSection::SubstantiveTesting)
            .collect();

        let mut confirmations = Vec::with_capacity(count);
        let mut responses = Vec::with_capacity(count);

        // Pre-compute aggregate balances for each confirmation category.
        let bank_balance: f64 = account_balances
            .iter()
            .filter(|(code, _)| code.starts_with("10"))
            .map(|(_, bal)| bal.abs())
            .sum();
        let ar_balance: f64 = account_balances
            .iter()
            .filter(|(code, _)| code.starts_with("11"))
            .map(|(_, bal)| bal.abs())
            .sum();
        let ap_balance: f64 = account_balances
            .iter()
            .filter(|(code, _)| code.starts_with("20"))
            .map(|(_, bal)| bal.abs())
            .sum();

        for i in 0..count {
            let (conf_type, recipient_type, recipient_name) =
                self.choose_confirmation_type(i, count);

            let account_code: Option<String> = if account_codes.is_empty() {
                None
            } else {
                let idx = self.rng.random_range(0..account_codes.len());
                Some(account_codes[idx].clone())
            };

            // Use real balances when available for the matching confirmation type.
            let real_balance = match conf_type {
                ConfirmationType::BankBalance | ConfirmationType::Loan => bank_balance,
                ConfirmationType::AccountsReceivable => ar_balance,
                ConfirmationType::AccountsPayable => ap_balance,
                _ => 0.0,
            };

            // Synthetic fallback: $10k - $5M (same as original generate_confirmations).
            let synthetic_units: i64 = self.rng.random_range(10_000_i64..=5_000_000_i64);
            let synthetic_balance = Decimal::new(synthetic_units * 100, 2);

            let book_balance = if real_balance > 0.0 {
                Decimal::from_f64(real_balance).unwrap_or(synthetic_balance)
            } else {
                synthetic_balance
            };
            let balance_units_for_exception = if real_balance > 0.0 {
                real_balance as i64
            } else {
                synthetic_units
            };

            let confirmation_date = engagement.period_end_date;

            let fieldwork_days = (engagement.fieldwork_end - engagement.fieldwork_start)
                .num_days()
                .max(1);
            let sent_offset = self.rng.random_range(0..fieldwork_days);
            let sent_date = engagement.fieldwork_start + Duration::days(sent_offset);
            let deadline = sent_date + Duration::days(30);

            self.confirmation_counter += 1;

            let mut confirmation = ExternalConfirmation::new(
                engagement.engagement_id,
                conf_type,
                &recipient_name,
                recipient_type,
                book_balance,
                confirmation_date,
            );

            confirmation.confirmation_ref = format!(
                "CONF-{}-{:04}",
                engagement.fiscal_year, self.confirmation_counter
            );

            if !substantive_wps.is_empty() {
                let wp_idx = self.rng.random_range(0..substantive_wps.len());
                confirmation = confirmation.with_workpaper(substantive_wps[wp_idx].workpaper_id);
            }

            if let Some(ref code) = account_code {
                confirmation = confirmation.with_account(code);
            }

            confirmation.send(sent_date, deadline);

            // Determine response outcome (identical logic to generate_confirmations).
            let roll: f64 = self.rng.random();
            let no_response_cutoff = self.config.no_response_ratio;
            let exception_cutoff = no_response_cutoff + self.config.exception_response_ratio;
            let confirmed_cutoff = exception_cutoff + self.config.confirmed_response_ratio;

            if roll < no_response_cutoff {
                confirmation.status = ConfirmationStatus::NoResponse;
            } else {
                let response_days = self.rng.random_range(5_i64..=25_i64);
                let response_date = sent_date + Duration::days(response_days);

                let response_type = if roll < exception_cutoff {
                    ResponseType::ConfirmedWithException
                } else if roll < confirmed_cutoff {
                    ResponseType::Confirmed
                } else {
                    ResponseType::Denied
                };

                let mut response = ConfirmationResponse::new(
                    confirmation.confirmation_id,
                    engagement.engagement_id,
                    response_date,
                    response_type,
                );

                match response_type {
                    ResponseType::Confirmed => {
                        response = response.with_confirmed_balance(book_balance);
                        confirmation.status = ConfirmationStatus::Completed;
                    }
                    ResponseType::ConfirmedWithException => {
                        let exception_pct: f64 = self.rng.random_range(0.01..0.08);
                        let exception_units =
                            (balance_units_for_exception as f64 * exception_pct).round() as i64;
                        let exception_amount = Decimal::new(exception_units.max(1) * 100, 2);
                        let confirmed_balance = book_balance - exception_amount;

                        response = response
                            .with_confirmed_balance(confirmed_balance)
                            .with_exception(
                                exception_amount,
                                self.exception_description(conf_type),
                            );

                        if self.rng.random::<f64>() < self.config.exception_reconciled_ratio {
                            response.reconcile(
                                "Difference investigated and reconciled to timing items \
                                 — no audit adjustment required.",
                            );
                        }

                        confirmation.status = ConfirmationStatus::Completed;
                    }
                    ResponseType::Denied => {
                        confirmation.status = ConfirmationStatus::AlternativeProcedures;
                    }
                    ResponseType::NoReply => {
                        confirmation.status = ConfirmationStatus::NoResponse;
                    }
                }

                responses.push(response);
            }

            confirmations.push(confirmation);
        }

        (confirmations, responses)
    }

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    /// Choose a confirmation type, recipient type, and a realistic name based
    /// on configured ratios.  The `index` / `total` args allow even spread
    /// across types rather than pure random (avoids clustering at small counts).
    fn choose_confirmation_type(
        &mut self,
        index: usize,
        total: usize,
    ) -> (ConfirmationType, RecipientType, String) {
        // Compute cumulative thresholds.
        let bank_cutoff = self.config.bank_balance_ratio;
        let ar_cutoff = bank_cutoff + self.config.accounts_receivable_ratio;
        // Remaining split evenly between AP, Investment, Loan, Legal, Insurance, Inventory.
        let remaining = 1.0 - ar_cutoff;
        let other_each = remaining / 6.0;

        // Spread confirmations evenly across types.
        let fraction = (index as f64 + self.rng.random::<f64>()) / total.max(1) as f64;

        if fraction < bank_cutoff {
            let name = self.bank_name();
            (ConfirmationType::BankBalance, RecipientType::Bank, name)
        } else if fraction < ar_cutoff {
            let name = self.customer_name();
            (
                ConfirmationType::AccountsReceivable,
                RecipientType::Customer,
                name,
            )
        } else if fraction < ar_cutoff + other_each {
            let name = self.supplier_name();
            (
                ConfirmationType::AccountsPayable,
                RecipientType::Supplier,
                name,
            )
        } else if fraction < ar_cutoff + 2.0 * other_each {
            let name = self.investment_firm_name();
            (ConfirmationType::Investment, RecipientType::Other, name)
        } else if fraction < ar_cutoff + 3.0 * other_each {
            let name = self.bank_name();
            (ConfirmationType::Loan, RecipientType::Bank, name)
        } else if fraction < ar_cutoff + 4.0 * other_each {
            let name = self.legal_firm_name();
            (ConfirmationType::Legal, RecipientType::LegalCounsel, name)
        } else if fraction < ar_cutoff + 5.0 * other_each {
            let name = self.insurer_name();
            (ConfirmationType::Insurance, RecipientType::Insurer, name)
        } else {
            let name = self.supplier_name();
            (ConfirmationType::Inventory, RecipientType::Other, name)
        }
    }

    fn bank_name(&mut self) -> String {
        let banks = [
            "First National Bank",
            "City Commerce Bank",
            "Meridian Federal Credit Union",
            "Pacific Trust Bank",
            "Atlantic Financial Corp",
            "Heritage Savings Bank",
            "Sunrise Bank plc",
            "Continental Banking Group",
        ];
        let idx = self.rng.random_range(0..banks.len());
        banks[idx].to_string()
    }

    fn customer_name(&mut self) -> String {
        let names = [
            "Acme Industries Ltd",
            "Beacon Holdings PLC",
            "Crestwood Manufacturing",
            "Delta Retail Group",
            "Epsilon Logistics Inc",
            "Falcon Distribution SA",
            "Global Supplies Corp",
            "Horizon Trading Ltd",
            "Irongate Wholesale",
            "Jupiter Services LLC",
        ];
        let idx = self.rng.random_range(0..names.len());
        names[idx].to_string()
    }

    fn supplier_name(&mut self) -> String {
        let names = [
            "Allied Components GmbH",
            "BestSource Procurement",
            "Cornerstone Supplies",
            "Direct Parts Ltd",
            "Eagle Procurement SA",
            "Foundation Materials Inc",
            "Granite Supply Co",
        ];
        let idx = self.rng.random_range(0..names.len());
        names[idx].to_string()
    }

    fn investment_firm_name(&mut self) -> String {
        let names = [
            "Summit Asset Management",
            "Veritas Capital Partners",
            "Pinnacle Investment Trust",
            "Apex Securities Ltd",
        ];
        let idx = self.rng.random_range(0..names.len());
        names[idx].to_string()
    }

    fn legal_firm_name(&mut self) -> String {
        let names = [
            "Harrison & Webb LLP",
            "Morrison Clarke Solicitors",
            "Pemberton Legal Group",
            "Sterling Advocates LLP",
        ];
        let idx = self.rng.random_range(0..names.len());
        names[idx].to_string()
    }

    fn insurer_name(&mut self) -> String {
        let names = [
            "Centennial Insurance Co",
            "Landmark Re Ltd",
            "Prudential Assurance PLC",
            "Shield Underwriters Ltd",
        ];
        let idx = self.rng.random_range(0..names.len());
        names[idx].to_string()
    }

    fn exception_description(&self, conf_type: ConfirmationType) -> &'static str {
        match conf_type {
            ConfirmationType::BankBalance => {
                "Outstanding cheque issued before year-end not yet presented for clearing"
            }
            ConfirmationType::AccountsReceivable => {
                "Credit note raised before period end not yet reflected in client ledger"
            }
            ConfirmationType::AccountsPayable => {
                "Goods received before year-end; supplier invoice recorded in following period"
            }
            ConfirmationType::Investment => {
                "Accrued income on securities differs due to day-count convention"
            }
            ConfirmationType::Loan => {
                "Accrued interest calculation basis differs from bank statement"
            }
            ConfirmationType::Legal => {
                "Matter description differs from client disclosure — wording to be aligned"
            }
            ConfirmationType::Insurance => {
                "Policy premium allocation differs by one month due to renewal date"
            }
            ConfirmationType::Inventory => {
                "Consignment stock included in third-party count but excluded from client records"
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::audit::test_helpers::create_test_engagement;

    fn make_gen(seed: u64) -> ConfirmationGenerator {
        ConfirmationGenerator::new(seed)
    }

    fn empty_workpapers() -> Vec<Workpaper> {
        Vec::new()
    }

    fn empty_accounts() -> Vec<String> {
        Vec::new()
    }

    // -------------------------------------------------------------------------

    /// Count is always within the configured (min, max) range.
    #[test]
    fn test_generates_expected_count() {
        let engagement = create_test_engagement();
        let mut gen = make_gen(42);
        let (confs, _) =
            gen.generate_confirmations(&engagement, &empty_workpapers(), &empty_accounts());

        let min = ConfirmationGeneratorConfig::default()
            .confirmations_per_engagement
            .0 as usize;
        let max = ConfirmationGeneratorConfig::default()
            .confirmations_per_engagement
            .1 as usize;
        assert!(
            confs.len() >= min && confs.len() <= max,
            "expected {min}..={max}, got {}",
            confs.len()
        );
    }

    /// With many runs, roughly 70% of confirmations should get a "Confirmed" response.
    #[test]
    fn test_response_distribution() {
        let engagement = create_test_engagement();
        // Use a config that generates a large fixed count per run so we accumulate quickly.
        let config = ConfirmationGeneratorConfig {
            confirmations_per_engagement: (100, 100),
            ..Default::default()
        };
        let mut gen = ConfirmationGenerator::with_config(99, config);
        let (confs, responses) =
            gen.generate_confirmations(&engagement, &empty_workpapers(), &empty_accounts());

        let total = confs.len() as f64;
        let confirmed_count = responses
            .iter()
            .filter(|r| r.response_type == ResponseType::Confirmed)
            .count() as f64;

        // Expect within ±15% of the 70% target (i.e., 55–85%).
        let ratio = confirmed_count / total;
        assert!(
            (0.55..=0.85).contains(&ratio),
            "confirmed ratio {ratio:.2} outside expected 55–85%"
        );
    }

    /// Exception amounts should be a small fraction (1–8%) of the book balance.
    #[test]
    fn test_exception_amounts() {
        let engagement = create_test_engagement();
        let config = ConfirmationGeneratorConfig {
            confirmations_per_engagement: (200, 200),
            exception_response_ratio: 0.50, // inflate exceptions so we get enough samples
            confirmed_response_ratio: 0.40,
            no_response_ratio: 0.05,
            ..Default::default()
        };
        let mut gen = ConfirmationGenerator::with_config(77, config);
        let (confs, responses) =
            gen.generate_confirmations(&engagement, &empty_workpapers(), &empty_accounts());

        // Build a lookup from confirmation_id → book_balance.
        let book_map: std::collections::HashMap<uuid::Uuid, Decimal> = confs
            .iter()
            .map(|c| (c.confirmation_id, c.book_balance))
            .collect();

        let exceptions: Vec<&ConfirmationResponse> =
            responses.iter().filter(|r| r.has_exception).collect();

        assert!(
            !exceptions.is_empty(),
            "expected at least some exception responses"
        );

        for resp in &exceptions {
            let book = *book_map.get(&resp.confirmation_id).unwrap();
            let exc = resp.exception_amount.unwrap();
            // exc should be 1–8% of book balance (plus small rounding).
            let ratio = (exc / book).to_string().parse::<f64>().unwrap_or(1.0);
            assert!(
                ratio > 0.0 && ratio <= 0.09,
                "exception ratio {ratio:.4} out of expected 0–9% for book={book}, exc={exc}"
            );
        }
    }

    /// Same seed must produce identical output (deterministic PRNG).
    #[test]
    fn test_deterministic_with_seed() {
        let engagement = create_test_engagement();
        let accounts = vec!["1010".to_string(), "1200".to_string(), "2100".to_string()];

        let (confs_a, resp_a) = {
            let mut gen = make_gen(1234);
            gen.generate_confirmations(&engagement, &empty_workpapers(), &accounts)
        };
        let (confs_b, resp_b) = {
            let mut gen = make_gen(1234);
            gen.generate_confirmations(&engagement, &empty_workpapers(), &accounts)
        };

        assert_eq!(
            confs_a.len(),
            confs_b.len(),
            "confirmation counts differ across identical seeds"
        );
        assert_eq!(
            resp_a.len(),
            resp_b.len(),
            "response counts differ across identical seeds"
        );

        for (a, b) in confs_a.iter().zip(confs_b.iter()) {
            assert_eq!(a.confirmation_ref, b.confirmation_ref);
            assert_eq!(a.book_balance, b.book_balance);
            assert_eq!(a.status, b.status);
            assert_eq!(a.confirmation_type, b.confirmation_type);
        }
    }

    /// Confirmations link to the provided account codes.
    #[test]
    fn test_account_codes_linked() {
        let engagement = create_test_engagement();
        let accounts = vec!["ACC-001".to_string(), "ACC-002".to_string()];
        let mut gen = make_gen(55);
        let (confs, _) = gen.generate_confirmations(&engagement, &empty_workpapers(), &accounts);

        // Every confirmation should have an account_id from our list.
        for conf in &confs {
            assert!(
                conf.account_id.as_deref().is_some(),
                "confirmation {} should have an account_id",
                conf.confirmation_ref
            );
            assert!(
                accounts.contains(conf.account_id.as_ref().unwrap()),
                "account_id '{}' not in provided list",
                conf.account_id.as_ref().unwrap()
            );
        }
    }

    /// When a substantive workpaper is provided, confirmations should link to it.
    #[test]
    fn test_workpaper_linking() {
        use datasynth_core::models::audit::WorkpaperSection;

        let engagement = create_test_engagement();
        // Build a minimal substantive workpaper so we can test linking.
        let wp = Workpaper::new(
            engagement.engagement_id,
            "D-001",
            "Test Workpaper",
            WorkpaperSection::SubstantiveTesting,
        );
        let wp_id = wp.workpaper_id;

        let mut gen = make_gen(71);
        let (confs, _) = gen.generate_confirmations(&engagement, &[wp], &empty_accounts());

        // All confirmations should be linked to the single substantive workpaper.
        for conf in &confs {
            assert_eq!(
                conf.workpaper_id,
                Some(wp_id),
                "confirmation {} should link to workpaper {wp_id}",
                conf.confirmation_ref
            );
        }
    }

    /// Confirmations with real balances use the supplied AR/AP/Cash amounts.
    #[test]
    fn test_balance_weighted_confirmations_use_real_balances() {
        use datasynth_core::models::audit::ConfirmationType;

        let engagement = create_test_engagement();
        let accounts = vec!["1100".to_string(), "2000".to_string(), "1010".to_string()];
        let balances = HashMap::from([
            ("1100".into(), 1_250_000.0), // AR
            ("2000".into(), 875_000.0),   // AP
            ("1010".into(), 500_000.0),   // Cash/Bank
        ]);

        let config = ConfirmationGeneratorConfig {
            confirmations_per_engagement: (30, 30),
            ..Default::default()
        };
        let mut gen = ConfirmationGenerator::with_config(42, config);
        let (confs, _) = gen.generate_confirmations_with_balances(
            &engagement,
            &empty_workpapers(),
            &accounts,
            &balances,
        );

        assert!(!confs.is_empty());

        // AR confirmations should have book_balance equal to the AR total (1,250,000).
        let ar_confs: Vec<_> = confs
            .iter()
            .filter(|c| c.confirmation_type == ConfirmationType::AccountsReceivable)
            .collect();
        for conf in &ar_confs {
            let expected = Decimal::from_f64(1_250_000.0).unwrap();
            assert_eq!(
                conf.book_balance, expected,
                "AR confirmation should use real AR balance"
            );
        }

        // Bank confirmations should use Cash balance (500,000).
        let bank_confs: Vec<_> = confs
            .iter()
            .filter(|c| c.confirmation_type == ConfirmationType::BankBalance)
            .collect();
        for conf in &bank_confs {
            let expected = Decimal::from_f64(500_000.0).unwrap();
            assert_eq!(
                conf.book_balance, expected,
                "Bank confirmation should use real Cash balance"
            );
        }

        // AP confirmations should use AP balance (875,000).
        let ap_confs: Vec<_> = confs
            .iter()
            .filter(|c| c.confirmation_type == ConfirmationType::AccountsPayable)
            .collect();
        for conf in &ap_confs {
            let expected = Decimal::from_f64(875_000.0).unwrap();
            assert_eq!(
                conf.book_balance, expected,
                "AP confirmation should use real AP balance"
            );
        }
    }

    /// When balances are empty, the balance-weighted method falls back to synthetic values.
    #[test]
    fn test_balance_weighted_empty_balances_uses_synthetic() {
        let engagement = create_test_engagement();
        let accounts = vec!["1100".to_string()];
        let empty_balances: HashMap<String, f64> = HashMap::new();

        let mut gen = make_gen(42);
        let (confs, _) = gen.generate_confirmations_with_balances(
            &engagement,
            &empty_workpapers(),
            &accounts,
            &empty_balances,
        );

        assert!(!confs.is_empty());
        // All book balances should be in the synthetic $10k-$5M range.
        for conf in &confs {
            let bal = conf.book_balance;
            assert!(
                bal >= Decimal::new(10_000_00, 2) && bal <= Decimal::new(5_000_000_00, 2),
                "expected synthetic balance in 10k-5M range, got {bal}"
            );
        }
    }
}
