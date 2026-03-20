//! Defined benefit pension generator — IAS 19 / ASC 715.
//!
//! Generates one pension plan per entity together with a DBO roll-forward,
//! plan asset roll-forward, pension disclosure, and the associated
//! journal entries for the reporting period.
//!
//! # Generation logic
//!
//! 1. **Plan** — one plan per entity; participant count scales with the number
//!    of employees (clamped to 1–500).
//! 2. **Actuarial assumptions** — randomised per plan within standard ranges:
//!    - Discount rate: 3%–5%
//!    - Salary growth: 2%–4%
//!    - Pension increase: 1%–3%
//!    - Expected return on plan assets: 5%–7%
//! 3. **DBO roll-forward**
//!    - Opening DBO seeded from a calibrated funding base (participant count × avg-salary × years).
//!    - Service cost = participant_count × avg_annual_salary × accrual_rate (1%–1.5%).
//!    - Interest cost = dbo_opening × discount_rate.
//!    - Actuarial gains/losses = random (−2% to +2% of dbo_opening).
//!    - Benefits paid = ~4%–6% of dbo_opening (mature plan outflows).
//! 4. **Plan assets**
//!    - Opening fair value calibrated so funding ratio is initially 75%–110%.
//!    - Expected return = opening × expected_return_rate.
//!    - Actuarial gain/loss on assets = random (−1.5% to +1.5% of opening).
//!    - Employer contributions ≈ 80% of service cost.
//! 5. **Journal entries**
//!    - Pension expense JE: DR Pension Expense (6200) / CR Net Pension Liability (2800).
//!    - OCI remeasurement JE (if non-zero): DR/CR OCI (3800) / CR/DR Net Pension Liability (2800).

use chrono::NaiveDate;
use datasynth_core::models::journal_entry::{JournalEntry, JournalEntryLine, TransactionSource};
use datasynth_core::models::pension::{
    ActuarialAssumptions, DefinedBenefitPlan, PensionDisclosure, PensionObligation,
    PensionPlanType, PlanAssets,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::debug;

// ---------------------------------------------------------------------------
// GL accounts (no existing constant for pension — use local strings)
// ---------------------------------------------------------------------------

/// Pension / post-retirement benefit expense (sub-account of Benefits 6200).
const PENSION_EXPENSE: &str = "6205";
/// Net pension liability / asset (balance-sheet line).
const NET_PENSION_LIABILITY: &str = "2800";
/// OCI — remeasurements of defined benefit plans.
const OCI_REMEASUREMENTS: &str = "3800";

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

/// All outputs from one pension generation run.
#[derive(Debug, Default)]
pub struct PensionSnapshot {
    /// Pension plans (one per entity).
    pub plans: Vec<DefinedBenefitPlan>,
    /// DBO roll-forward records.
    pub obligations: Vec<PensionObligation>,
    /// Plan asset roll-forward records.
    pub plan_assets: Vec<PlanAssets>,
    /// Pension disclosures.
    pub disclosures: Vec<PensionDisclosure>,
    /// Journal entries (pension expense + OCI remeasurements).
    pub journal_entries: Vec<JournalEntry>,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates defined benefit pension data for a reporting entity.
pub struct PensionGenerator {
    /// UUID factory for deterministic, collision-free plan IDs.
    #[allow(dead_code)]
    uuid_factory: DeterministicUuidFactory,
    rng: ChaCha8Rng,
}

impl PensionGenerator {
    /// Create a new generator with a deterministic seed.
    pub fn new(seed: u64) -> Self {
        Self {
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::Pension),
            rng: ChaCha8Rng::seed_from_u64(seed ^ 0x19_4500_7150_u64),
        }
    }

    /// Generate pension data for a single entity and period.
    ///
    /// # Parameters
    /// - `entity_code`      : company / entity identifier (e.g. "1000")
    /// - `entity_name`      : human-readable entity name
    /// - `period_label`     : period string (e.g. "2024-12" or "FY2024")
    /// - `reporting_date`   : balance-sheet date
    /// - `employee_count`   : number of employees (used to calibrate participant count)
    /// - `currency`         : reporting currency code
    /// - `avg_salary`       : optional average annual salary from actual payroll data;
    ///                        falls back to $50 000 when `None`
    /// - `period_months`    : number of months in the reporting period (used to prorate
    ///                        annual pension expense; defaults to 12 when 0)
    pub fn generate(
        &mut self,
        entity_code: &str,
        entity_name: &str,
        period_label: &str,
        reporting_date: NaiveDate,
        employee_count: usize,
        currency: &str,
        avg_salary: Option<Decimal>,
        period_months: u32,
    ) -> PensionSnapshot {
        let plan_id = format!("PLAN-{entity_code}-DB");

        // ---- 1. Participant count (1–500, realistic for any company size) ------
        let participant_count = (employee_count.clamp(1, 500)) as u32;

        // ---- 2. Actuarial assumptions -----------------------------------------
        let discount_rate = self.rand_rate(dec!(0.03), dec!(0.05));
        let salary_growth_rate = self.rand_rate(dec!(0.02), dec!(0.04));
        let pension_increase_rate = self.rand_rate(dec!(0.01), dec!(0.03));
        let expected_return_on_plan_assets = self.rand_rate(dec!(0.05), dec!(0.07));

        let assumptions = ActuarialAssumptions {
            discount_rate,
            salary_growth_rate,
            pension_increase_rate,
            expected_return_on_plan_assets,
        };

        let plan = DefinedBenefitPlan {
            id: plan_id.clone(),
            entity_code: entity_code.to_string(),
            plan_name: format!("{entity_name} Retirement Plan"),
            plan_type: PensionPlanType::DefinedBenefit,
            participant_count,
            assumptions: assumptions.clone(),
            currency: currency.to_string(),
        };

        // ---- 3. DBO roll-forward -----------------------------------------------
        // Opening DBO: participant_count × avg_annual_salary × avg_service_years
        // avg_annual_salary: use actual payroll data when available, else $50 000
        // avg_service_years ~ 15
        let avg_annual_salary = avg_salary.unwrap_or(dec!(50000));
        let avg_service_years = dec!(15);
        let dbo_opening = Decimal::from(participant_count) * avg_annual_salary * avg_service_years;

        // Service cost: participant_count × avg_salary × accrual_rate (1%–1.5%)
        let accrual_rate = self.rand_rate(dec!(0.01), dec!(0.015));
        let service_cost =
            (Decimal::from(participant_count) * avg_annual_salary * accrual_rate).round_dp(2);

        // Interest cost: dbo_opening × discount_rate
        let interest_cost = (dbo_opening * discount_rate).round_dp(2);

        // Actuarial gains/losses: −2% to +2% of dbo_opening
        let actuarial_gl_rate = self.rand_rate(dec!(-0.02), dec!(0.02));
        let actuarial_gains_losses = (dbo_opening * actuarial_gl_rate).round_dp(2);

        // Benefits paid: 4%–6% of dbo_opening
        let benefits_pct = self.rand_rate(dec!(0.04), dec!(0.06));
        let benefits_paid = (dbo_opening * benefits_pct).round_dp(2);

        // DBO closing identity
        let dbo_closing = (dbo_opening + service_cost + interest_cost + actuarial_gains_losses
            - benefits_paid)
            .round_dp(2);

        let obligation = PensionObligation {
            plan_id: plan_id.clone(),
            period: period_label.to_string(),
            dbo_opening,
            service_cost,
            interest_cost,
            actuarial_gains_losses,
            benefits_paid,
            dbo_closing,
        };

        // ---- 4. Plan assets roll-forward --------------------------------------
        // Initial funding ratio between 75%–110%
        let initial_funding_ratio = self.rand_rate(dec!(0.75), dec!(1.10));
        let fair_value_opening = (dbo_opening * initial_funding_ratio).round_dp(2);

        let expected_return = (fair_value_opening * expected_return_on_plan_assets).round_dp(2);

        // Actuarial gain/loss on assets: −1.5% to +1.5%
        let asset_al_rate = self.rand_rate(dec!(-0.015), dec!(0.015));
        let actuarial_gain_loss_assets = (fair_value_opening * asset_al_rate).round_dp(2);

        // Employer contributions ≈ 80% of service cost
        let employer_contributions = (service_cost * dec!(0.80)).round_dp(2);

        // Plan assets closing identity
        let fair_value_closing = (fair_value_opening
            + expected_return
            + actuarial_gain_loss_assets
            + employer_contributions
            - benefits_paid)
            .round_dp(2);

        let plan_assets_rec = PlanAssets {
            plan_id: plan_id.clone(),
            period: period_label.to_string(),
            fair_value_opening,
            expected_return,
            actuarial_gain_loss: actuarial_gain_loss_assets,
            employer_contributions,
            benefits_paid,
            fair_value_closing,
        };

        // ---- 5. Pension disclosure --------------------------------------------
        let net_pension_liability = (dbo_closing - fair_value_closing).round_dp(2);
        // Prorate annual pension expense for sub-annual periods.
        // period_months == 0 is treated as a full year (12 months).
        let effective_months = if period_months == 0 {
            12
        } else {
            period_months.min(12)
        };
        let annual_pension_expense = (service_cost + interest_cost - expected_return).round_dp(2);
        let pension_expense = if effective_months < 12 {
            (annual_pension_expense * Decimal::from(effective_months) / Decimal::from(12u32))
                .round_dp(2)
        } else {
            annual_pension_expense
        };
        // OCI = obligation actuarial G/L + asset actuarial G/L (with sign flip for assets)
        let oci_remeasurements = (actuarial_gains_losses - actuarial_gain_loss_assets).round_dp(2);
        let funding_ratio = if dbo_closing.is_zero() {
            Decimal::ZERO
        } else {
            (fair_value_closing / dbo_closing).round_dp(4)
        };

        let disclosure = PensionDisclosure {
            plan_id: plan_id.clone(),
            period: period_label.to_string(),
            net_pension_liability,
            pension_expense,
            oci_remeasurements,
            funding_ratio,
        };

        // ---- 6. Journal entries ----------------------------------------------
        let mut journal_entries = Vec::new();

        if !pension_expense.is_zero() {
            journal_entries.push(self.pension_expense_je(
                entity_code,
                reporting_date,
                &plan_id,
                period_label,
                pension_expense,
            ));
        }

        if !oci_remeasurements.is_zero() {
            journal_entries.push(self.oci_remeasurement_je(
                entity_code,
                reporting_date,
                &plan_id,
                period_label,
                oci_remeasurements,
            ));
        }

        debug!(
            "Pension generated: entity={entity_code}, participants={participant_count}, \
             avg_salary={avg_annual_salary}, period_months={effective_months}/12, \
             DBO closing={dbo_closing}, assets closing={fair_value_closing}, \
             net_liability={net_pension_liability}, expense={pension_expense}"
        );

        PensionSnapshot {
            plans: vec![plan],
            obligations: vec![obligation],
            plan_assets: vec![plan_assets_rec],
            disclosures: vec![disclosure],
            journal_entries,
        }
    }

    // -------------------------------------------------------------------------
    // JE builders
    // -------------------------------------------------------------------------

    /// DR Pension Expense / CR Net Pension Liability for current-period cost.
    fn pension_expense_je(
        &mut self,
        entity_code: &str,
        posting_date: NaiveDate,
        plan_id: &str,
        period: &str,
        pension_expense: Decimal,
    ) -> JournalEntry {
        let doc_id = format!("JE-PENSION-EXP-{}-{}", entity_code, period.replace('-', ""));

        let mut je = JournalEntry::new_simple(
            doc_id,
            entity_code.to_string(),
            posting_date,
            format!("Pension expense — {period}"),
        );
        je.header.source = TransactionSource::Adjustment;

        if pension_expense > Decimal::ZERO {
            // Net expense: DR Pension Expense, CR Net Pension Liability
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: PENSION_EXPENSE.to_string(),
                debit_amount: pension_expense,
                reference: Some(plan_id.to_string()),
                text: Some(format!("Pension expense {period}")),
                ..Default::default()
            });
            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: NET_PENSION_LIABILITY.to_string(),
                credit_amount: pension_expense,
                reference: Some(plan_id.to_string()),
                text: Some(format!("Net pension liability increase {period}")),
                ..Default::default()
            });
        } else {
            // Net income (unusual — expected return exceeds service + interest)
            let abs_expense = pension_expense.abs();
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: NET_PENSION_LIABILITY.to_string(),
                debit_amount: abs_expense,
                reference: Some(plan_id.to_string()),
                text: Some(format!("Net pension liability decrease {period}")),
                ..Default::default()
            });
            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: PENSION_EXPENSE.to_string(),
                credit_amount: abs_expense,
                reference: Some(plan_id.to_string()),
                text: Some(format!("Pension income credit {period}")),
                ..Default::default()
            });
        }

        je
    }

    /// OCI remeasurement JE.
    ///
    /// Positive `oci_remeasurements` = net actuarial loss recognised in OCI:
    ///   DR OCI (3800) / CR Net Pension Liability (2800)
    ///
    /// Negative = net actuarial gain:
    ///   DR Net Pension Liability (2800) / CR OCI (3800)
    fn oci_remeasurement_je(
        &mut self,
        entity_code: &str,
        posting_date: NaiveDate,
        plan_id: &str,
        period: &str,
        oci_remeasurements: Decimal,
    ) -> JournalEntry {
        let doc_id = format!("JE-PENSION-OCI-{}-{}", entity_code, period.replace('-', ""));

        let mut je = JournalEntry::new_simple(
            doc_id,
            entity_code.to_string(),
            posting_date,
            format!("Pension OCI remeasurement — {period}"),
        );
        je.header.source = TransactionSource::Adjustment;

        let abs_amount = oci_remeasurements.abs();
        if oci_remeasurements > Decimal::ZERO {
            // Actuarial loss: DR OCI / CR Net Pension Liability
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: OCI_REMEASUREMENTS.to_string(),
                debit_amount: abs_amount,
                reference: Some(plan_id.to_string()),
                text: Some(format!("OCI actuarial loss {period}")),
                ..Default::default()
            });
            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: NET_PENSION_LIABILITY.to_string(),
                credit_amount: abs_amount,
                reference: Some(plan_id.to_string()),
                text: Some(format!("Net pension liability — actuarial loss {period}")),
                ..Default::default()
            });
        } else {
            // Actuarial gain: DR Net Pension Liability / CR OCI
            je.add_line(JournalEntryLine {
                line_number: 1,
                gl_account: NET_PENSION_LIABILITY.to_string(),
                debit_amount: abs_amount,
                reference: Some(plan_id.to_string()),
                text: Some(format!("Net pension liability — actuarial gain {period}")),
                ..Default::default()
            });
            je.add_line(JournalEntryLine {
                line_number: 2,
                gl_account: OCI_REMEASUREMENTS.to_string(),
                credit_amount: abs_amount,
                reference: Some(plan_id.to_string()),
                text: Some(format!("OCI actuarial gain {period}")),
                ..Default::default()
            });
        }

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
