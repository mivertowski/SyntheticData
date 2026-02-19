//! Tax Provision Generator (ASC 740 / IAS 12).
//!
//! Computes current and deferred income tax provisions for a reporting period.
//! Generates rate reconciliation items that bridge from the statutory rate to
//! the effective rate, and produces realistic deferred tax asset/liability
//! balances from temporary differences.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_core::models::TaxProvision;

// ---------------------------------------------------------------------------
// Rate reconciliation catalogue
// ---------------------------------------------------------------------------

/// A candidate reconciliation item with its description and rate-impact range.
struct ReconciliationCandidate {
    description: &'static str,
    /// Minimum rate impact (inclusive).
    min_impact: Decimal,
    /// Maximum rate impact (inclusive).
    max_impact: Decimal,
}

/// Pool of possible rate reconciliation items.
const CANDIDATES: &[ReconciliationCandidate] = &[
    ReconciliationCandidate {
        description: "State and local taxes",
        min_impact: dec!(0.01),
        max_impact: dec!(0.04),
    },
    ReconciliationCandidate {
        description: "Permanent differences",
        min_impact: dec!(-0.01),
        max_impact: dec!(0.02),
    },
    ReconciliationCandidate {
        description: "R&D tax credits",
        min_impact: dec!(-0.02),
        max_impact: dec!(-0.005),
    },
    ReconciliationCandidate {
        description: "Foreign rate differential",
        min_impact: dec!(-0.03),
        max_impact: dec!(0.03),
    },
    ReconciliationCandidate {
        description: "Stock compensation",
        min_impact: dec!(-0.01),
        max_impact: dec!(0.01),
    },
    ReconciliationCandidate {
        description: "Valuation allowance change",
        min_impact: dec!(-0.02),
        max_impact: dec!(0.05),
    },
];

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates income tax provisions under ASC 740 / IAS 12.
///
/// Given pre-tax income and a statutory rate, the generator:
/// 1. Selects 2-5 rate reconciliation items from the candidate pool.
/// 2. Computes the effective rate as `statutory_rate + sum(reconciliation impacts)`.
/// 3. Computes `current_tax_expense = pre_tax_income * effective_rate`.
/// 4. Generates realistic deferred tax asset and liability balances.
pub struct TaxProvisionGenerator {
    rng: ChaCha8Rng,
    counter: u64,
}

impl TaxProvisionGenerator {
    /// Creates a new tax provision generator with the given deterministic seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            counter: 0,
        }
    }

    /// Generate a tax provision for a period.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Legal entity identifier.
    /// * `period` - Period end date.
    /// * `pre_tax_income` - Pre-tax income from financial statements.
    /// * `statutory_rate` - Statutory corporate tax rate (e.g., `0.21` for US).
    pub fn generate(
        &mut self,
        entity_id: &str,
        period: NaiveDate,
        pre_tax_income: Decimal,
        statutory_rate: Decimal,
    ) -> TaxProvision {
        self.counter += 1;
        let provision_id = format!("TXPROV-{:06}", self.counter);

        // Select 2-5 reconciliation items
        let num_items = self.rng.gen_range(2..=5);
        let mut selected_indices: Vec<usize> = (0..CANDIDATES.len()).collect();
        selected_indices.shuffle(&mut self.rng);
        selected_indices.truncate(num_items);
        selected_indices.sort(); // stable ordering for determinism after shuffle

        let mut total_impact = Decimal::ZERO;
        let mut reconciliation_items: Vec<(&str, Decimal)> = Vec::new();

        for &idx in &selected_indices {
            let candidate = &CANDIDATES[idx];
            let impact = self.random_decimal(candidate.min_impact, candidate.max_impact);
            total_impact += impact;
            reconciliation_items.push((candidate.description, impact));
        }

        let effective_rate = (statutory_rate + total_impact).round_dp(6);
        let current_tax_expense = (pre_tax_income * effective_rate).round_dp(2);

        // Generate deferred tax balances (random realistic amounts)
        // DTA: typically 1-8% of pre_tax_income (from timing differences, NOL carryforwards)
        let dta_pct = self.random_decimal(dec!(0.01), dec!(0.08));
        let deferred_tax_asset = (pre_tax_income.abs() * dta_pct).round_dp(2);

        // DTL: typically 1-6% of pre_tax_income (from depreciation timing, etc.)
        let dtl_pct = self.random_decimal(dec!(0.01), dec!(0.06));
        let deferred_tax_liability = (pre_tax_income.abs() * dtl_pct).round_dp(2);

        let mut provision = TaxProvision::new(
            provision_id,
            entity_id,
            period,
            current_tax_expense,
            deferred_tax_asset,
            deferred_tax_liability,
            statutory_rate,
            effective_rate,
        );

        for (desc, impact) in &reconciliation_items {
            provision = provision.with_reconciliation_item(*desc, *impact);
        }

        provision
    }

    /// Generates a random decimal between `min` and `max` (inclusive).
    fn random_decimal(&mut self, min: Decimal, max: Decimal) -> Decimal {
        let range_f64 = (max - min).to_string().parse::<f64>().unwrap_or(0.0);
        let min_f64 = min.to_string().parse::<f64>().unwrap_or(0.0);
        let val = min_f64 + self.rng.gen::<f64>() * range_f64;
        Decimal::try_from(val).unwrap_or(min).round_dp(6)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn period_end() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
    }

    #[test]
    fn test_provision_calculation() {
        let mut gen = TaxProvisionGenerator::new(42);
        let provision = gen.generate("ENT-001", period_end(), dec!(1000000), dec!(0.21));

        // Effective rate should be statutory_rate + sum of reconciliation impacts
        let total_impact: Decimal = provision
            .rate_reconciliation
            .iter()
            .map(|r| r.rate_impact)
            .sum();
        let expected_effective = (dec!(0.21) + total_impact).round_dp(6);
        assert_eq!(provision.effective_rate, expected_effective);

        // Current tax expense = pre_tax_income * effective_rate
        let expected_expense = (dec!(1000000) * provision.effective_rate).round_dp(2);
        assert_eq!(provision.current_tax_expense, expected_expense);

        // Statutory rate preserved
        assert_eq!(provision.statutory_rate, dec!(0.21));
    }

    #[test]
    fn test_rate_reconciliation() {
        let mut gen = TaxProvisionGenerator::new(42);
        let provision = gen.generate("ENT-001", period_end(), dec!(500000), dec!(0.21));

        // Should have 2-5 reconciliation items
        assert!(
            provision.rate_reconciliation.len() >= 2,
            "Should have at least 2 items, got {}",
            provision.rate_reconciliation.len()
        );
        assert!(
            provision.rate_reconciliation.len() <= 5,
            "Should have at most 5 items, got {}",
            provision.rate_reconciliation.len()
        );

        // Sum of impacts should equal effective_rate - statutory_rate
        let total_impact: Decimal = provision
            .rate_reconciliation
            .iter()
            .map(|r| r.rate_impact)
            .sum();
        let diff = (provision.effective_rate - provision.statutory_rate).round_dp(6);
        let impact_rounded = total_impact.round_dp(6);

        // Allow small tolerance for floating-point → decimal conversion
        let tolerance = dec!(0.000002);
        assert!(
            (diff - impact_rounded).abs() <= tolerance,
            "Reconciliation items should sum to effective - statutory: diff={}, impact={}",
            diff,
            impact_rounded
        );
    }

    #[test]
    fn test_deferred_tax() {
        let mut gen = TaxProvisionGenerator::new(42);
        let provision = gen.generate("ENT-001", period_end(), dec!(2000000), dec!(0.21));

        // Deferred tax asset and liability should both be positive
        assert!(
            provision.deferred_tax_asset > Decimal::ZERO,
            "DTA should be positive: {}",
            provision.deferred_tax_asset
        );
        assert!(
            provision.deferred_tax_liability > Decimal::ZERO,
            "DTL should be positive: {}",
            provision.deferred_tax_liability
        );

        // DTA should be between 1-8% of pre_tax_income
        let pti = dec!(2000000);
        assert!(
            provision.deferred_tax_asset >= (pti * dec!(0.01)).round_dp(2),
            "DTA too small"
        );
        assert!(
            provision.deferred_tax_asset <= (pti * dec!(0.08)).round_dp(2),
            "DTA too large"
        );

        // DTL should be between 1-6% of pre_tax_income
        assert!(
            provision.deferred_tax_liability >= (pti * dec!(0.01)).round_dp(2),
            "DTL too small"
        );
        assert!(
            provision.deferred_tax_liability <= (pti * dec!(0.06)).round_dp(2),
            "DTL too large"
        );
    }

    #[test]
    fn test_deterministic() {
        let mut gen1 = TaxProvisionGenerator::new(999);
        let p1 = gen1.generate("ENT-001", period_end(), dec!(750000), dec!(0.21));

        let mut gen2 = TaxProvisionGenerator::new(999);
        let p2 = gen2.generate("ENT-001", period_end(), dec!(750000), dec!(0.21));

        assert_eq!(p1.id, p2.id);
        assert_eq!(p1.current_tax_expense, p2.current_tax_expense);
        assert_eq!(p1.effective_rate, p2.effective_rate);
        assert_eq!(p1.statutory_rate, p2.statutory_rate);
        assert_eq!(p1.deferred_tax_asset, p2.deferred_tax_asset);
        assert_eq!(p1.deferred_tax_liability, p2.deferred_tax_liability);
        assert_eq!(p1.rate_reconciliation.len(), p2.rate_reconciliation.len());
        for (r1, r2) in p1
            .rate_reconciliation
            .iter()
            .zip(p2.rate_reconciliation.iter())
        {
            assert_eq!(r1.description, r2.description);
            assert_eq!(r1.rate_impact, r2.rate_impact);
        }
    }
}
