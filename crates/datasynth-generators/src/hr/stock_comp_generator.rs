//! Stock-based compensation generator — ASC 718 / IFRS 2.
//!
//! Generates equity award grants for executive employees, vesting schedules,
//! period expense recognition records, and the associated journal entries.
//!
//! # Generation logic
//!
//! 1. **Grantees** — The top 10% of employees (by list index, acting as a
//!    proxy for seniority / executive status), subject to a configurable
//!    minimum (`min_grantees`) and maximum (`max_grantees`).
//! 2. **Instrument mix** — 50% RSUs, 30% Options, 20% PSUs (rounded to whole
//!    employees at each threshold).
//! 3. **Fair value**
//!    - RSUs: `share_price` (default $50).
//!    - Options: `share_price × factor` where factor ∈ [0.30, 0.50]
//!      (simplified Black-Scholes proxy).
//!    - PSUs: `share_price × factor` where factor ∈ [0.80, 1.20]
//!      (reflecting performance probability weighting).
//! 4. **Vesting** — Graded over 4 years, 25% per year; one `VestingEntry`
//!    per annual anniversary of the grant date.
//! 5. **Forfeiture rate** — Sampled uniformly in [0.05, 0.15] per grant.
//! 6. **Expense per period** — Straight-line:
//!    `total_grant_value × (1 − forfeiture_rate) / vesting_periods`
//! 7. **Journal entry** — DR Compensation Expense (7200) / CR APIC–Stock
//!    Compensation (3150) for the period expense amount.

use chrono::{Datelike, NaiveDate};
use datasynth_core::models::journal_entry::{JournalEntry, JournalEntryLine, TransactionSource};
use datasynth_core::models::stock_compensation::{
    InstrumentType, StockCompExpense, StockGrant, VestingEntry, VestingSchedule, VestingType,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::debug;

// ---------------------------------------------------------------------------
// GL account codes
// ---------------------------------------------------------------------------

/// Share-based compensation expense — sub-account of operating expenses.
/// Uses 7200 as specified (labour / comp expense range above BENEFITS 6200).
const COMP_EXPENSE: &str = "7200";

/// Additional Paid-In Capital — Stock Compensation sub-account.
/// 3150 is a sub-account of the standard APIC (3100) reserved for
/// equity-settled share-based payments.
const APIC_STOCK_COMP: &str = "3150";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the stock compensation generator.
#[derive(Debug, Clone)]
pub struct StockCompConfig {
    /// Share price used to compute fair value at grant date.
    pub share_price: Decimal,
    /// Minimum quantity of shares granted per executive.
    pub min_grant_quantity: u32,
    /// Maximum quantity of shares granted per executive.
    pub max_grant_quantity: u32,
    /// Number of vesting years (standard: 4).
    pub vesting_years: u32,
    /// Forfeiture rate lower bound (e.g. 0.05 = 5%).
    pub forfeiture_min: Decimal,
    /// Forfeiture rate upper bound (e.g. 0.15 = 15%).
    pub forfeiture_max: Decimal,
}

impl Default for StockCompConfig {
    fn default() -> Self {
        Self {
            share_price: dec!(50.00),
            min_grant_quantity: 500,
            max_grant_quantity: 5000,
            vesting_years: 4,
            forfeiture_min: dec!(0.05),
            forfeiture_max: dec!(0.15),
        }
    }
}

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

/// All outputs from one stock compensation generation run.
#[derive(Debug, Default)]
pub struct StockCompSnapshot {
    /// Stock grants (one per grantee).
    pub grants: Vec<StockGrant>,
    /// Period expense records (one per grant per active vesting period).
    pub expenses: Vec<StockCompExpense>,
    /// Journal entries (DR Comp Expense / CR APIC-Stock Comp).
    pub journal_entries: Vec<JournalEntry>,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates stock-based compensation data for a reporting entity.
pub struct StockCompGenerator {
    #[allow(dead_code)]
    uuid_factory: DeterministicUuidFactory,
    rng: ChaCha8Rng,
    config: StockCompConfig,
}

impl StockCompGenerator {
    /// Create a new generator with a deterministic seed and default config.
    pub fn new(seed: u64) -> Self {
        Self {
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::StockCompensation),
            rng: ChaCha8Rng::seed_from_u64(seed ^ 0x7182_0018_u64),
            config: StockCompConfig::default(),
        }
    }

    /// Override the default configuration.
    pub fn with_config(mut self, config: StockCompConfig) -> Self {
        self.config = config;
        self
    }

    /// Generate stock compensation data for one entity and one reporting period.
    ///
    /// # Parameters
    /// - `entity_code`   : company / entity identifier
    /// - `employee_ids`  : full list of employees (top 10% become grantees)
    /// - `grant_date`    : date on which grants are made (typically fiscal year start)
    /// - `period_label`  : period string used on expense records (e.g. "FY2024")
    /// - `reporting_date`: last day of the reporting period (used for JE posting date)
    /// - `currency`      : reporting currency code
    pub fn generate(
        &mut self,
        entity_code: &str,
        employee_ids: &[String],
        grant_date: NaiveDate,
        period_label: &str,
        reporting_date: NaiveDate,
        currency: &str,
    ) -> StockCompSnapshot {
        let mut snapshot = StockCompSnapshot::default();

        if employee_ids.is_empty() {
            return snapshot;
        }

        // Determine executive pool: top 10% (min 1, max 50)
        let exec_count = ((employee_ids.len() as f64 * 0.10).ceil() as usize).clamp(1, 50);
        let grantees = &employee_ids[..exec_count];

        debug!(
            "StockComp: entity={entity_code}, employees={}, grantees={exec_count}",
            employee_ids.len()
        );

        // Instrument type distribution across grantees
        // 50% RSUs, 30% Options, 20% PSUs
        let rsu_count = ((exec_count as f64 * 0.50).round() as usize).min(exec_count);
        let opt_count = ((exec_count as f64 * 0.30).round() as usize).min(exec_count - rsu_count);
        // PSUs get the remainder
        let _psu_count = exec_count - rsu_count - opt_count;

        for (idx, employee_id) in grantees.iter().enumerate() {
            let instrument_type = if idx < rsu_count {
                InstrumentType::RSUs
            } else if idx < rsu_count + opt_count {
                InstrumentType::Options
            } else {
                InstrumentType::PSUs
            };

            let grant = self.build_grant(
                entity_code,
                employee_id,
                grant_date,
                instrument_type,
                currency,
            );

            // Generate expense records for each vesting period that falls
            // within or before the reporting date
            let expenses = self.build_expenses(&grant, period_label, reporting_date);

            // Generate JEs for this period's expense
            let period_expense: Decimal = expenses.iter().map(|e| e.expense_amount).sum();
            if !period_expense.is_zero() {
                let je = self.build_je(
                    entity_code,
                    reporting_date,
                    &grant.id,
                    period_label,
                    period_expense,
                );
                snapshot.journal_entries.push(je);
            }

            snapshot.expenses.extend(expenses);
            snapshot.grants.push(grant);
        }

        debug!(
            "StockComp generated: entity={entity_code}, grants={}, expenses={}, jes={}",
            snapshot.grants.len(),
            snapshot.expenses.len(),
            snapshot.journal_entries.len()
        );

        snapshot
    }

    // -------------------------------------------------------------------------
    // Grant builder
    // -------------------------------------------------------------------------

    fn build_grant(
        &mut self,
        entity_code: &str,
        employee_id: &str,
        grant_date: NaiveDate,
        instrument_type: InstrumentType,
        currency: &str,
    ) -> StockGrant {
        let grant_id = format!(
            "GRANT-{}-{}-{}",
            entity_code,
            employee_id,
            grant_date.year()
        );

        // Quantity
        let qty_range = (self.config.max_grant_quantity - self.config.min_grant_quantity) as f64;
        let quantity =
            self.config.min_grant_quantity + (self.rng.random::<f64>() * qty_range) as u32;

        // Fair value per share
        let (fair_value_at_grant, exercise_price) = match instrument_type {
            InstrumentType::RSUs => (self.config.share_price, None),
            InstrumentType::Options => {
                // Simplified Black-Scholes proxy: 30%–50% of share price
                let factor = self.rand_rate(dec!(0.30), dec!(0.50));
                let fv = (self.config.share_price * factor).round_dp(2);
                // Exercise price = at-the-money (equal to share price)
                (fv, Some(self.config.share_price))
            }
            InstrumentType::PSUs => {
                // PSUs: 80%–120% of share price reflecting performance probability
                let factor = self.rand_rate(dec!(0.80), dec!(1.20));
                ((self.config.share_price * factor).round_dp(2), None)
            }
        };

        let total_grant_value = (fair_value_at_grant * Decimal::from(quantity)).round_dp(2);

        // Forfeiture rate
        let forfeiture_rate =
            self.rand_rate(self.config.forfeiture_min, self.config.forfeiture_max);

        // Vesting schedule: graded 4-year, 25% per year
        let vesting_schedule = self.build_vesting_schedule(grant_date, self.config.vesting_years);

        // Options expire 10 years from grant date
        let expiration_date = if instrument_type == InstrumentType::Options {
            grant_date.checked_add_signed(chrono::Duration::days(365 * 10))
        } else {
            None
        };

        StockGrant {
            id: grant_id,
            entity_code: entity_code.to_string(),
            employee_id: employee_id.to_string(),
            grant_date,
            instrument_type,
            quantity,
            exercise_price,
            fair_value_at_grant,
            total_grant_value,
            vesting_schedule,
            expiration_date,
            forfeiture_rate,
            currency: currency.to_string(),
        }
    }

    // -------------------------------------------------------------------------
    // Vesting schedule builder
    // -------------------------------------------------------------------------

    fn build_vesting_schedule(&self, grant_date: NaiveDate, years: u32) -> VestingSchedule {
        let pct_per_period = (Decimal::ONE / Decimal::from(years)).round_dp(4);
        let mut cumulative = Decimal::ZERO;
        let mut entries = Vec::with_capacity(years as usize);

        for period in 1..=years {
            // Adjust for rounding: last tranche absorbs any residual
            let pct = if period == years {
                (Decimal::ONE - cumulative).round_dp(4)
            } else {
                pct_per_period
            };
            cumulative = (cumulative + pct).round_dp(4);

            // Vesting date: N-year anniversary of grant date
            let vesting_date = add_years(grant_date, period);

            entries.push(VestingEntry {
                period,
                vesting_date,
                percentage: pct,
                cumulative_percentage: cumulative,
            });
        }

        VestingSchedule {
            vesting_type: VestingType::Graded,
            total_periods: years,
            cliff_periods: None,
            vesting_entries: entries,
        }
    }

    // -------------------------------------------------------------------------
    // Expense builder
    // -------------------------------------------------------------------------

    /// Build period expense records using straight-line expense recognition.
    ///
    /// ASC 718 / IFRS 2 requires recognising compensation cost over the
    /// *requisite service period* (= vesting period), not just at vesting
    /// dates.  For graded vesting with N annual tranches, each tranche
    /// has a service period of 1 year.  We recognise tranche expense
    /// pro-rata based on the fraction of the service period elapsed by
    /// `reporting_date`.
    ///
    /// A tranche's service period begins on `grant_date` (for tranche 1)
    /// or on the previous vesting date (for subsequent tranches).
    /// Any tranche whose service period has started produces an expense record.
    ///
    /// One `StockCompExpense` is emitted per grant summarising the
    /// cumulative expense recognised through `reporting_date`.
    fn build_expenses(
        &self,
        grant: &StockGrant,
        period_label: &str,
        reporting_date: NaiveDate,
    ) -> Vec<StockCompExpense> {
        // Grant must have started service and reporting_date must be on/after grant_date.
        if reporting_date < grant.grant_date {
            return vec![];
        }

        let total_expense =
            (grant.total_grant_value * (Decimal::ONE - grant.forfeiture_rate)).round_dp(2);
        let n = grant.vesting_schedule.total_periods;
        if n == 0 || total_expense.is_zero() {
            return vec![];
        }

        // Per-period expense (straight-line, equal tranche amounts)
        let per_period_base = (total_expense / Decimal::from(n)).round_dp(2);

        let mut cumulative = Decimal::ZERO;

        for (tranche_idx, entry) in grant.vesting_schedule.vesting_entries.iter().enumerate() {
            // Service period start for this tranche
            let service_start = if tranche_idx == 0 {
                grant.grant_date
            } else {
                grant
                    .vesting_schedule
                    .vesting_entries
                    .get(tranche_idx - 1)
                    .map(|prev| prev.vesting_date)
                    .unwrap_or(grant.grant_date)
            };
            let service_end = entry.vesting_date;

            // Skip tranches whose service period has not yet begun
            if service_start > reporting_date {
                break;
            }

            // Expense for this tranche: full period if service_end ≤ reporting_date,
            // otherwise pro-rate by days elapsed / total service days.
            let expense_amount = if service_end <= reporting_date {
                // Tranche fully earned (service period complete)
                if tranche_idx + 1 == n as usize {
                    // Last tranche: absorb any rounding residual
                    (total_expense - cumulative).max(Decimal::ZERO)
                } else {
                    per_period_base
                }
            } else {
                // Tranche partially earned: pro-rate by days elapsed
                let total_days = (service_end - service_start).num_days().max(1) as f64;
                let elapsed_days = (reporting_date - service_start).num_days().max(0) as f64;
                let tranche_max = if tranche_idx + 1 == n as usize {
                    (total_expense - cumulative).max(Decimal::ZERO)
                } else {
                    per_period_base
                };
                let fraction = elapsed_days / total_days;
                let frac_dec = Decimal::try_from(fraction).unwrap_or(Decimal::ZERO);
                (tranche_max * frac_dec).round_dp(2)
            };

            cumulative = (cumulative + expense_amount).round_dp(2);
        }

        if cumulative.is_zero() {
            return vec![];
        }

        let remaining = (total_expense - cumulative).max(Decimal::ZERO);

        vec![StockCompExpense {
            grant_id: grant.id.clone(),
            entity_code: grant.entity_code.clone(),
            period: period_label.to_string(),
            expense_amount: cumulative,
            cumulative_recognized: cumulative,
            remaining_unrecognized: remaining,
            forfeiture_rate: grant.forfeiture_rate,
        }]
    }

    // -------------------------------------------------------------------------
    // Journal entry builder
    // -------------------------------------------------------------------------

    /// DR Compensation Expense (7200) / CR APIC–Stock Compensation (3150).
    fn build_je(
        &mut self,
        entity_code: &str,
        posting_date: NaiveDate,
        grant_id: &str,
        period: &str,
        amount: Decimal,
    ) -> JournalEntry {
        let doc_id = format!("JE-STOCKCOMP-{}-{}", entity_code, period.replace('-', ""));

        let mut je = JournalEntry::new_simple(
            doc_id,
            entity_code.to_string(),
            posting_date,
            format!("Stock-based compensation expense — {period}"),
        );
        je.header.source = TransactionSource::Adjustment;

        je.add_line(JournalEntryLine {
            line_number: 1,
            gl_account: COMP_EXPENSE.to_string(),
            debit_amount: amount,
            reference: Some(grant_id.to_string()),
            text: Some(format!("SBC expense {period}")),
            ..Default::default()
        });
        je.add_line(JournalEntryLine {
            line_number: 2,
            gl_account: APIC_STOCK_COMP.to_string(),
            credit_amount: amount,
            reference: Some(grant_id.to_string()),
            text: Some(format!("APIC stock comp {period}")),
            ..Default::default()
        });

        je
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Sample a uniform decimal in [lo, hi].
    fn rand_rate(&mut self, lo: Decimal, hi: Decimal) -> Decimal {
        let range_f = (hi - lo).to_string().parse::<f64>().unwrap_or(0.0);
        let sample: f64 = self.rng.random::<f64>() * range_f;
        let sample_d = Decimal::try_from(sample).unwrap_or(Decimal::ZERO);
        (lo + sample_d).round_dp(4)
    }
}

// ---------------------------------------------------------------------------
// Date arithmetic helper
// ---------------------------------------------------------------------------

/// Add `years` calendar years to `date`, snapping to end-of-month when the
/// day doesn't exist in the target month (e.g. Feb 29 → Feb 28).
fn add_years(date: NaiveDate, years: u32) -> NaiveDate {
    let target_year = date.year() + years as i32;
    let day = date.day();
    // Try exact day; fall back to last day of month if it doesn't exist.
    NaiveDate::from_ymd_opt(target_year, date.month(), day)
        .or_else(|| NaiveDate::from_ymd_opt(target_year, date.month(), 28))
        .unwrap_or(date)
}
