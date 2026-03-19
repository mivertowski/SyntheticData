//! Deferred Tax Engine (IAS 12 / ASC 740).
//!
//! Generates temporary differences between book and tax bases, computes
//! DTA and DTL balances, produces ETR reconciliation schedules, and
//! emits balanced GL journal entries for the deferred tax movements.
//!
//! # Journal entry logic
//!
//! For each company the engine generates two JEs:
//!
//! | Scenario | Debit | Credit |
//! |----------|-------|--------|
//! | Net DTL  | Tax Expense (8000) | Deferred Tax Liability (2500) |
//! | Net DTA  | Deferred Tax Asset (1600) | Tax Benefit (8000) |

use chrono::{Datelike, NaiveDate};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_core::accounts::{expense_accounts, tax_accounts};
use datasynth_core::models::deferred_tax::{
    DeferredTaxRollforward, DeferredTaxType, PermanentDifference, TaxRateReconciliation,
    TemporaryDifference,
};
use datasynth_core::models::journal_entry::{
    BusinessProcess, JournalEntry, JournalEntryHeader, JournalEntryLine, TransactionSource,
};
use datasynth_core::utils::seeded_rng;

// ---------------------------------------------------------------------------
// Public snapshot
// ---------------------------------------------------------------------------

/// All deferred tax data generated for a single generation run.
#[derive(Debug, Clone, Default)]
pub struct DeferredTaxSnapshot {
    /// Temporary differences (book vs. tax basis) per company.
    pub temporary_differences: Vec<TemporaryDifference>,
    /// ETR reconciliation schedules per company.
    pub etr_reconciliations: Vec<TaxRateReconciliation>,
    /// Deferred tax rollforward schedules per company.
    pub rollforwards: Vec<DeferredTaxRollforward>,
    /// Balanced GL journal entries recording deferred tax movements.
    pub journal_entries: Vec<JournalEntry>,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates deferred tax data under IAS 12 / ASC 740.
///
/// The generator is deterministic: given the same seed and inputs it always
/// produces the same output.
pub struct DeferredTaxGenerator {
    rng: ChaCha8Rng,
    counter: u64,
}

impl DeferredTaxGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 88),
            counter: 0,
        }
    }

    /// Generate a full deferred tax snapshot for the provided companies.
    ///
    /// # Arguments
    ///
    /// * `companies` – slice of `(company_code, country_code)` tuples.
    /// * `posting_date` – balance sheet / period-end date for JE headers.
    /// * `journal_entries` – existing JEs used to estimate pre-tax income,
    ///   total assets and total revenue.
    pub fn generate(
        &mut self,
        companies: &[(&str, &str)],
        posting_date: NaiveDate,
        journal_entries: &[JournalEntry],
    ) -> DeferredTaxSnapshot {
        let mut snapshot = DeferredTaxSnapshot::default();

        for &(company_code, country_code) in companies {
            let statutory_rate = Self::statutory_rate(country_code);
            let pre_tax_income = self.estimate_pre_tax_income(company_code, journal_entries);

            // Compute total assets and total revenue directly from journal entries.
            // Fall back to income-based heuristics only when no JE data is available.
            let total_assets =
                Self::compute_total_assets(company_code, journal_entries).max(dec!(1_000_000));
            let total_revenue =
                Self::compute_total_revenue(company_code, journal_entries).max(dec!(500_000));

            let period_label = format!("FY{}", posting_date.year());

            // 1. Temporary differences
            let diffs =
                self.generate_temp_diffs(company_code, pre_tax_income, total_assets, total_revenue);
            let (dta, dtl) = compute_dta_dtl(&diffs, statutory_rate);

            // 2. ETR reconciliation
            let etr = self.build_etr_reconciliation(
                company_code,
                &period_label,
                pre_tax_income,
                statutory_rate,
            );

            // 3. Rollforward
            let rollforward = self.build_rollforward(company_code, &period_label, dta, dtl);

            // 4. Journal entries
            let jes = self.build_journal_entries(company_code, posting_date, dta, dtl);

            snapshot.temporary_differences.extend(diffs);
            snapshot.etr_reconciliations.push(etr);
            snapshot.rollforwards.push(rollforward);
            snapshot.journal_entries.extend(jes);
        }

        snapshot
    }

    // -----------------------------------------------------------------------
    // Temporary differences
    // -----------------------------------------------------------------------

    fn generate_temp_diffs(
        &mut self,
        entity_code: &str,
        _pre_tax_income: Decimal,
        total_assets: Decimal,
        revenue_proxy: Decimal,
    ) -> Vec<TemporaryDifference> {
        let templates: Vec<(&str, &str, DeferredTaxType, Option<&str>, Decimal, Decimal)> = vec![
            // (description, account, type, standard, book_basis, tax_basis)
            // Accelerated depreciation – MACRS/capital allowances
            (
                "Accelerated depreciation (MACRS / capital allowances)",
                datasynth_core::accounts::control_accounts::FIXED_ASSETS,
                DeferredTaxType::Liability,
                Some("IAS 16 / ASC 360"),
                // book NBV: 8-12% of total assets
                total_assets * self.rand_decimal(dec!(0.08), dec!(0.12)),
                // tax NBV: lower because accelerated → 1.4-1.6x book depreciation taken
                total_assets * self.rand_decimal(dec!(0.05), dec!(0.09)),
            ),
            // Accrued expenses (deductible when paid)
            (
                "Accrued expenses (deductible when paid)",
                datasynth_core::accounts::liability_accounts::ACCRUED_EXPENSES,
                DeferredTaxType::Asset,
                Some("IAS 37 / ASC 450"),
                revenue_proxy * self.rand_decimal(dec!(0.01), dec!(0.03)),
                Decimal::ZERO,
            ),
            // Allowance for doubtful accounts
            (
                "Allowance for doubtful accounts",
                datasynth_core::accounts::control_accounts::AR_CONTROL,
                DeferredTaxType::Asset,
                Some("IFRS 9 / ASC 310"),
                revenue_proxy * self.rand_decimal(dec!(0.005), dec!(0.015)),
                Decimal::ZERO,
            ),
            // Inventory write-down (lower of cost or NRV)
            (
                "Inventory write-down (LCM / NRV)",
                datasynth_core::accounts::control_accounts::INVENTORY,
                DeferredTaxType::Asset,
                Some("IAS 2 / ASC 330"),
                total_assets * self.rand_decimal(dec!(0.005), dec!(0.015)),
                Decimal::ZERO,
            ),
            // Lease ROU asset (IFRS 16 / ASC 842 – operating for tax)
            (
                "Right-of-use asset – operating lease (tax: rental deduction)",
                expense_accounts::RENT,
                DeferredTaxType::Liability,
                Some("IFRS 16 / ASC 842"),
                total_assets * self.rand_decimal(dec!(0.02), dec!(0.06)),
                Decimal::ZERO,
            ),
            // Warranty provision
            (
                "Warranty provision (deductible when paid)",
                datasynth_core::accounts::liability_accounts::ACCRUED_EXPENSES,
                DeferredTaxType::Asset,
                Some("IAS 37 / ASC 460"),
                revenue_proxy * self.rand_decimal(dec!(0.003), dec!(0.010)),
                Decimal::ZERO,
            ),
            // Share-based compensation (IFRS 2 / ASC 718)
            (
                "Share-based compensation (book > tax until exercise)",
                datasynth_core::accounts::expense_accounts::BENEFITS,
                DeferredTaxType::Asset,
                Some("IFRS 2 / ASC 718"),
                revenue_proxy * self.rand_decimal(dec!(0.005), dec!(0.012)),
                Decimal::ZERO,
            ),
            // Capitalised development costs (book; immediately expensed for tax)
            (
                "Capitalised development costs (expensed for tax)",
                datasynth_core::accounts::control_accounts::FIXED_ASSETS,
                DeferredTaxType::Liability,
                Some("IAS 38 / ASC 730"),
                total_assets * self.rand_decimal(dec!(0.01), dec!(0.04)),
                Decimal::ZERO,
            ),
        ];

        // Pick 5-8 templates
        let n = self.rng.random_range(5usize..=8);
        let mut indices: Vec<usize> = (0..templates.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(n);
        indices.sort();

        indices
            .iter()
            .map(|&i| {
                let (desc, account, dtype, standard, book, tax) = &templates[i];
                let book = book.round_dp(2);
                let tax = tax.round_dp(2);
                let difference = (book - tax).round_dp(2);
                self.counter += 1;
                TemporaryDifference {
                    id: format!("TDIFF-{entity_code}-{:05}", self.counter),
                    entity_code: entity_code.to_string(),
                    account: account.to_string(),
                    description: desc.to_string(),
                    book_basis: book,
                    tax_basis: tax,
                    difference,
                    deferred_type: *dtype,
                    originating_standard: standard.map(|s| s.to_string()),
                }
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // ETR reconciliation
    // -----------------------------------------------------------------------

    fn build_etr_reconciliation(
        &mut self,
        entity_code: &str,
        period: &str,
        pre_tax_income: Decimal,
        statutory_rate: Decimal,
    ) -> TaxRateReconciliation {
        let expected_tax = (pre_tax_income * statutory_rate).round_dp(2);

        // Generate 3-5 permanent differences
        let perm_diff_templates: Vec<(&str, Decimal, Decimal)> = vec![
            // (description, book_amount_as_pct_of_pti, tax_effect_multiplier)
            // Meals & entertainment (50% disallowed)
            (
                "Meals & entertainment (50% non-deductible)",
                pre_tax_income * self.rand_decimal(dec!(0.002), dec!(0.006)),
                // Only 50% is disallowed → tax effect = 50% × amount × rate
                pre_tax_income * self.rand_decimal(dec!(0.001), dec!(0.003)) * statutory_rate,
            ),
            // Municipal bond interest (tax-exempt)
            (
                "Tax-exempt municipal bond interest",
                -(pre_tax_income * self.rand_decimal(dec!(0.005), dec!(0.015))),
                -(pre_tax_income * self.rand_decimal(dec!(0.005), dec!(0.015)) * statutory_rate),
            ),
            // Non-deductible fines & penalties
            (
                "Non-deductible fines & penalties",
                pre_tax_income * self.rand_decimal(dec!(0.001), dec!(0.003)),
                pre_tax_income * self.rand_decimal(dec!(0.001), dec!(0.003)) * statutory_rate,
            ),
            // R&D tax credit (reduces tax below statutory)
            (
                "Research & development tax credits",
                -(pre_tax_income * self.rand_decimal(dec!(0.005), dec!(0.020))),
                -(pre_tax_income * self.rand_decimal(dec!(0.005), dec!(0.020))),
            ),
            // Stock-based compensation (ASC 718 / IFRS 2 excess benefit)
            (
                "Stock-based compensation – excess tax benefit",
                -(pre_tax_income * self.rand_decimal(dec!(0.002), dec!(0.008))),
                -(pre_tax_income * self.rand_decimal(dec!(0.002), dec!(0.008)) * statutory_rate),
            ),
            // Foreign-derived intangible income (US FDII deduction)
            (
                "Foreign-derived intangible income (FDII) deduction",
                -(pre_tax_income * self.rand_decimal(dec!(0.003), dec!(0.010))),
                -(pre_tax_income * self.rand_decimal(dec!(0.003), dec!(0.010)) * statutory_rate),
            ),
            // Officer compensation disallowance (§162(m) US)
            (
                "Officer compensation in excess of §162(m) limit",
                pre_tax_income * self.rand_decimal(dec!(0.001), dec!(0.004)),
                pre_tax_income * self.rand_decimal(dec!(0.001), dec!(0.004)) * statutory_rate,
            ),
        ];

        let n = self.rng.random_range(3usize..=5);
        let mut indices: Vec<usize> = (0..perm_diff_templates.len()).collect();
        indices.shuffle(&mut self.rng);
        indices.truncate(n);
        indices.sort();

        let permanent_differences: Vec<PermanentDifference> = indices
            .iter()
            .map(|&i| {
                let (desc, amount, tax_effect) = &perm_diff_templates[i];
                PermanentDifference {
                    description: desc.to_string(),
                    amount: amount.round_dp(2),
                    tax_effect: tax_effect.round_dp(2),
                }
            })
            .collect();

        let total_perm_effect: Decimal = permanent_differences.iter().map(|p| p.tax_effect).sum();
        let actual_tax = (expected_tax + total_perm_effect).round_dp(2);
        let effective_rate = if pre_tax_income != Decimal::ZERO {
            (actual_tax / pre_tax_income).round_dp(6)
        } else {
            statutory_rate
        };

        TaxRateReconciliation {
            entity_code: entity_code.to_string(),
            period: period.to_string(),
            pre_tax_income: pre_tax_income.round_dp(2),
            statutory_rate,
            expected_tax,
            permanent_differences,
            effective_rate,
            actual_tax,
        }
    }

    // -----------------------------------------------------------------------
    // Rollforward
    // -----------------------------------------------------------------------

    fn build_rollforward(
        &mut self,
        entity_code: &str,
        period: &str,
        closing_dta: Decimal,
        closing_dtl: Decimal,
    ) -> DeferredTaxRollforward {
        // For a first-period run opening balances are zero; movement = closing.
        let opening_dta = Decimal::ZERO;
        let opening_dtl = Decimal::ZERO;
        let current_year_movement = (closing_dta - opening_dta) - (closing_dtl - opening_dtl);

        DeferredTaxRollforward {
            entity_code: entity_code.to_string(),
            period: period.to_string(),
            opening_dta,
            opening_dtl,
            current_year_movement: current_year_movement.round_dp(2),
            closing_dta: closing_dta.round_dp(2),
            closing_dtl: closing_dtl.round_dp(2),
        }
    }

    // -----------------------------------------------------------------------
    // Journal entries
    // -----------------------------------------------------------------------

    fn build_journal_entries(
        &mut self,
        company_code: &str,
        posting_date: NaiveDate,
        dta: Decimal,
        dtl: Decimal,
    ) -> Vec<JournalEntry> {
        let mut jes = Vec::new();

        // DTL entry: DR Tax Expense / CR Deferred Tax Liability
        if dtl > Decimal::ZERO {
            self.counter += 1;
            let mut header = JournalEntryHeader::new(company_code.to_string(), posting_date);
            header.document_type = "TAX_DEFERRED".to_string();
            header.created_by = "DEFERRED_TAX_ENGINE".to_string();
            header.source = TransactionSource::Automated;
            header.business_process = Some(BusinessProcess::R2R);
            header.header_text = Some(format!(
                "Deferred tax liability – period {}",
                posting_date.format("%Y-%m")
            ));

            let doc_id = header.document_id;
            let mut je = JournalEntry::new(header);
            je.add_line(JournalEntryLine::debit(
                doc_id,
                1,
                tax_accounts::TAX_EXPENSE.to_string(),
                dtl,
            ));
            je.add_line(JournalEntryLine::credit(
                doc_id,
                2,
                tax_accounts::DEFERRED_TAX_LIABILITY.to_string(),
                dtl,
            ));
            jes.push(je);
        }

        // DTA entry: DR Deferred Tax Asset / CR Tax Benefit (credit to Tax Expense)
        if dta > Decimal::ZERO {
            self.counter += 1;
            let mut header = JournalEntryHeader::new(company_code.to_string(), posting_date);
            header.document_type = "TAX_DEFERRED".to_string();
            header.created_by = "DEFERRED_TAX_ENGINE".to_string();
            header.source = TransactionSource::Automated;
            header.business_process = Some(BusinessProcess::R2R);
            header.header_text = Some(format!(
                "Deferred tax asset – period {}",
                posting_date.format("%Y-%m")
            ));

            let doc_id = header.document_id;
            let mut je = JournalEntry::new(header);
            je.add_line(JournalEntryLine::debit(
                doc_id,
                1,
                tax_accounts::DEFERRED_TAX_ASSET.to_string(),
                dta,
            ));
            je.add_line(JournalEntryLine::credit(
                doc_id,
                2,
                tax_accounts::TAX_EXPENSE.to_string(),
                dta,
            ));
            jes.push(je);
        }

        jes
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Compute total assets for a company from journal entries.
    ///
    /// Asset accounts start with "1". Net assets = Σ(debit − credit) on asset lines.
    fn compute_total_assets(company_code: &str, journal_entries: &[JournalEntry]) -> Decimal {
        use datasynth_core::accounts::AccountCategory;
        let mut net = Decimal::ZERO;
        for je in journal_entries {
            if je.header.company_code != company_code {
                continue;
            }
            for line in &je.lines {
                if matches!(
                    AccountCategory::from_account(&line.gl_account),
                    AccountCategory::Asset
                ) {
                    net += line.debit_amount - line.credit_amount;
                }
            }
        }
        net.abs()
    }

    /// Compute total revenue for a company from journal entries.
    ///
    /// Revenue accounts start with "4". Revenue is credit-normal.
    fn compute_total_revenue(company_code: &str, journal_entries: &[JournalEntry]) -> Decimal {
        use datasynth_core::accounts::AccountCategory;
        let mut revenue = Decimal::ZERO;
        for je in journal_entries {
            if je.header.company_code != company_code {
                continue;
            }
            for line in &je.lines {
                if matches!(
                    AccountCategory::from_account(&line.gl_account),
                    AccountCategory::Revenue
                ) {
                    revenue += line.credit_amount - line.debit_amount;
                }
            }
        }
        revenue.max(Decimal::ZERO)
    }

    /// Look up the statutory corporate income tax rate for a country.
    fn statutory_rate(country_code: &str) -> Decimal {
        match country_code.to_uppercase().as_str() {
            "US" => dec!(0.21),
            "DE" => dec!(0.30),
            "GB" | "UK" => dec!(0.25),
            "FR" => dec!(0.25),
            "NL" => dec!(0.258),
            "IE" => dec!(0.125),
            "CH" => dec!(0.15),
            "CA" => dec!(0.265),
            "AU" => dec!(0.30),
            "JP" => dec!(0.2928),
            "SG" => dec!(0.17),
            "CN" => dec!(0.25),
            "IN" => dec!(0.2517),
            "BR" => dec!(0.34),
            _ => dec!(0.21), // default to US rate
        }
    }

    /// Estimate pre-tax income for a company from existing journal entries.
    ///
    /// Pre-tax income = Σ revenue account credits − Σ expense account debits.
    /// Falls back to a heuristic if no entries match.
    fn estimate_pre_tax_income(
        &self,
        company_code: &str,
        journal_entries: &[JournalEntry],
    ) -> Decimal {
        use datasynth_core::accounts::AccountCategory;

        let mut revenue = Decimal::ZERO;
        let mut expenses = Decimal::ZERO;

        for je in journal_entries {
            if je.header.company_code != company_code {
                continue;
            }
            for line in &je.lines {
                let cat = AccountCategory::from_account(&line.gl_account);
                match cat {
                    AccountCategory::Revenue => {
                        // Revenue accounts are credit-normal; credits increase revenue
                        revenue += line.credit_amount;
                        revenue -= line.debit_amount;
                    }
                    AccountCategory::Cogs
                    | AccountCategory::OperatingExpense
                    | AccountCategory::OtherIncomeExpense => {
                        // Expense accounts are debit-normal
                        expenses += line.debit_amount;
                        expenses -= line.credit_amount;
                    }
                    _ => {}
                }
            }
        }

        let pti = (revenue - expenses).round_dp(2);
        if pti == Decimal::ZERO {
            // Fallback: synthetic income so we can still generate meaningful ratios
            dec!(1_000_000)
        } else {
            pti
        }
    }

    /// Random decimal in `[min, max]`.
    fn rand_decimal(&mut self, min: Decimal, max: Decimal) -> Decimal {
        let range: f64 = (max - min).to_string().parse().unwrap_or(0.0);
        let min_f: f64 = min.to_string().parse().unwrap_or(0.0);
        let v = min_f + self.rng.random::<f64>() * range;
        Decimal::try_from(v).unwrap_or(min).round_dp(6)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compute total DTA and DTL from a slice of temporary differences.
///
/// `dta = Σ |difference| × statutory_rate` for DTA-type diffs.
/// `dtl = Σ |difference| × statutory_rate` for DTL-type diffs.
pub fn compute_dta_dtl(
    diffs: &[TemporaryDifference],
    statutory_rate: Decimal,
) -> (Decimal, Decimal) {
    let mut dta = Decimal::ZERO;
    let mut dtl = Decimal::ZERO;
    for d in diffs {
        let effect = (d.difference.abs() * statutory_rate).round_dp(2);
        match d.deferred_type {
            DeferredTaxType::Asset => dta += effect,
            DeferredTaxType::Liability => dtl += effect,
        }
    }
    (dta.round_dp(2), dtl.round_dp(2))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
    }

    #[test]
    fn test_generate_returns_data_for_each_company() {
        let mut gen = DeferredTaxGenerator::new(42);
        let companies = vec![("C001", "US"), ("C002", "DE")];
        let snapshot = gen.generate(&companies, sample_date(), &[]);

        // At least 5 temp diffs per company (5–8 each)
        assert!(
            snapshot.temporary_differences.len() >= 2 * 5,
            "Expected ≥10 temp diffs, got {}",
            snapshot.temporary_differences.len()
        );
        assert_eq!(snapshot.etr_reconciliations.len(), 2);
        assert_eq!(snapshot.rollforwards.len(), 2);
    }

    #[test]
    fn test_dta_dtl_computation() {
        let diffs = vec![
            TemporaryDifference {
                id: "T1".into(),
                entity_code: "C001".into(),
                account: "1500".into(),
                description: "Depreciation".into(),
                book_basis: dec!(100_000),
                tax_basis: dec!(80_000),
                difference: dec!(20_000),
                deferred_type: DeferredTaxType::Liability,
                originating_standard: None,
            },
            TemporaryDifference {
                id: "T2".into(),
                entity_code: "C001".into(),
                account: "2200".into(),
                description: "Accruals".into(),
                book_basis: dec!(50_000),
                tax_basis: Decimal::ZERO,
                difference: dec!(50_000),
                deferred_type: DeferredTaxType::Asset,
                originating_standard: None,
            },
        ];
        let (dta, dtl) = compute_dta_dtl(&diffs, dec!(0.21));
        assert_eq!(dtl, (dec!(20_000) * dec!(0.21)).round_dp(2));
        assert_eq!(dta, (dec!(50_000) * dec!(0.21)).round_dp(2));
    }

    #[test]
    fn test_etr_reconciliation_math() {
        let mut gen = DeferredTaxGenerator::new(7);
        let companies = vec![("C001", "US")];
        let snap = gen.generate(&companies, sample_date(), &[]);

        let etr = &snap.etr_reconciliations[0];
        let expected_tax = (etr.pre_tax_income * etr.statutory_rate).round_dp(2);
        assert_eq!(etr.expected_tax, expected_tax, "expected_tax mismatch");

        // actual_tax = expected_tax + Σ permanent_diff.tax_effect
        let total_perm: Decimal = etr.permanent_differences.iter().map(|p| p.tax_effect).sum();
        let expected_actual = (expected_tax + total_perm).round_dp(2);
        assert_eq!(etr.actual_tax, expected_actual, "actual_tax mismatch");

        // effective_rate = actual_tax / pre_tax_income
        if etr.pre_tax_income != Decimal::ZERO {
            let expected_etr = (etr.actual_tax / etr.pre_tax_income).round_dp(6);
            assert_eq!(etr.effective_rate, expected_etr, "effective_rate mismatch");
        }
    }

    #[test]
    fn test_rollforward_opening_plus_movement_equals_closing() {
        let mut gen = DeferredTaxGenerator::new(13);
        let snap = gen.generate(&[("C001", "GB")], sample_date(), &[]);

        let rf = &snap.rollforwards[0];
        // closing_dta = opening_dta + dta_movement
        // closing_dtl = opening_dtl + dtl_movement
        // current_year_movement = (closing_dta - opening_dta) - (closing_dtl - opening_dtl)
        let implied_movement =
            (rf.closing_dta - rf.opening_dta) - (rf.closing_dtl - rf.opening_dtl);
        assert_eq!(
            rf.current_year_movement, implied_movement,
            "Rollforward movement check failed"
        );
    }

    #[test]
    fn test_journal_entries_are_balanced() {
        let mut gen = DeferredTaxGenerator::new(42);
        let snap = gen.generate(&[("C001", "US")], sample_date(), &[]);

        for je in &snap.journal_entries {
            let total_debit: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
            let total_credit: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
            assert_eq!(
                total_debit, total_credit,
                "JE {} is not balanced: debits={}, credits={}",
                je.header.document_id, total_debit, total_credit
            );
        }
    }

    #[test]
    fn test_journal_entries_have_tax_document_type() {
        let mut gen = DeferredTaxGenerator::new(42);
        let snap = gen.generate(&[("C001", "US"), ("C002", "DE")], sample_date(), &[]);

        for je in &snap.journal_entries {
            assert!(
                je.header.document_type.contains("TAX"),
                "Expected document_type to contain 'TAX', got '{}'",
                je.header.document_type
            );
        }
    }

    #[test]
    fn test_deterministic() {
        let companies = vec![("C001", "US")];
        let mut gen1 = DeferredTaxGenerator::new(99);
        let snap1 = gen1.generate(&companies, sample_date(), &[]);

        let mut gen2 = DeferredTaxGenerator::new(99);
        let snap2 = gen2.generate(&companies, sample_date(), &[]);

        assert_eq!(
            snap1.temporary_differences.len(),
            snap2.temporary_differences.len()
        );
        assert_eq!(
            snap1.etr_reconciliations[0].actual_tax,
            snap2.etr_reconciliations[0].actual_tax
        );
        assert_eq!(
            snap1.rollforwards[0].closing_dta,
            snap2.rollforwards[0].closing_dta
        );
    }
}
