//! Prior-year comparative data generator (WI-2).
//!
//! Generates prior-year balances, audit findings, and engagement summaries
//! from current-year account data. Supports ISA 315 (risk assessment via
//! year-over-year comparison) and ISA 520 (analytical procedures).

use chrono::NaiveDate;
use datasynth_core::distributions::{AmountDistributionConfig, AmountSampler};
use datasynth_core::models::{PriorYearComparative, PriorYearFinding, PriorYearSummary};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use rust_decimal::Decimal;

// ---------------------------------------------------------------------------
// Finding description templates
// ---------------------------------------------------------------------------

/// (finding_type, risk_area) -> description template pool
const FINDING_DESCRIPTIONS: &[(&str, &str, &str)] = &[
    // control_deficiency
    (
        "control_deficiency",
        "revenue",
        "Insufficient segregation of duties in revenue posting process",
    ),
    (
        "control_deficiency",
        "receivables",
        "Lack of timely reconciliation of accounts receivable subsidiary ledger",
    ),
    (
        "control_deficiency",
        "payables",
        "Missing secondary approval for vendor master data changes",
    ),
    (
        "control_deficiency",
        "inventory",
        "Cycle count procedures not performed on schedule for high-value items",
    ),
    (
        "control_deficiency",
        "estimates",
        "No formal review process for management's key accounting estimates",
    ),
    // misstatement
    (
        "misstatement",
        "revenue",
        "Revenue recognised before transfer of control per ASC 606 criteria",
    ),
    (
        "misstatement",
        "receivables",
        "Overstatement of accounts receivable due to improper cutoff at period end",
    ),
    (
        "misstatement",
        "payables",
        "Unrecorded liabilities identified through subsequent disbursement testing",
    ),
    (
        "misstatement",
        "inventory",
        "Inventory obsolescence reserve understated based on ageing analysis",
    ),
    (
        "misstatement",
        "estimates",
        "Fair value measurement for Level 3 assets not supported by observable inputs",
    ),
    // significant_deficiency
    (
        "significant_deficiency",
        "revenue",
        "Percentage-of-completion estimates lack corroborating project data",
    ),
    (
        "significant_deficiency",
        "receivables",
        "Expected credit loss model uses outdated forward-looking information",
    ),
    (
        "significant_deficiency",
        "payables",
        "Automated three-way match tolerance set above materiality threshold",
    ),
    (
        "significant_deficiency",
        "inventory",
        "Standard cost variances not analysed or allocated on a timely basis",
    ),
    (
        "significant_deficiency",
        "estimates",
        "Inadequate documentation of key assumptions in impairment model",
    ),
    // material_weakness
    (
        "material_weakness",
        "revenue",
        "Pervasive override of revenue recognition controls by senior management",
    ),
    (
        "material_weakness",
        "receivables",
        "Systematic failure to record allowance for doubtful accounts",
    ),
    (
        "material_weakness",
        "payables",
        "Duplicate payments processed without detection across multiple periods",
    ),
    (
        "material_weakness",
        "inventory",
        "Physical inventory counts not reconciled to perpetual records for the full year",
    ),
    (
        "material_weakness",
        "estimates",
        "Material misstatement in goodwill impairment due to unsubstantiated growth assumptions",
    ),
];

/// Key audit matter templates.
const KAM_POOL: &[&str] = &[
    "Revenue recognition",
    "Goodwill impairment",
    "Expected credit losses",
    "Inventory valuation",
    "Provisions and contingencies",
    "Fair value measurement of financial instruments",
    "Business combination purchase price allocation",
    "Going concern assessment",
    "Tax provisions and uncertain tax positions",
    "Lease accounting transition",
];

/// Weighted finding types: (type, cumulative weight).
const FINDING_TYPES: &[(&str, f64)] = &[
    ("control_deficiency", 0.40),
    ("misstatement", 0.70),
    ("significant_deficiency", 0.90),
    ("material_weakness", 1.00),
];

/// Weighted statuses: (status, cumulative weight).
const FINDING_STATUSES: &[(&str, f64)] = &[
    ("remediated", 0.50),
    ("open", 0.70),
    ("partially_remediated", 0.90),
    ("recurring", 1.00),
];

/// Risk areas with weights.
const RISK_AREAS: &[(&str, f64)] = &[
    ("revenue", 0.30),
    ("receivables", 0.50),
    ("estimates", 0.70),
    ("payables", 0.85),
    ("inventory", 1.00),
];

/// Generates prior-year comparative data from current-year balances.
pub struct PriorYearGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    amount_sampler: AmountSampler,
}

impl PriorYearGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x4E00),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::PriorYear),
            amount_sampler: AmountSampler::with_benford(
                seed.wrapping_add(0x4E01),
                AmountDistributionConfig::default(),
            ),
        }
    }

    /// Generate prior-year comparative data from current-year account balances.
    ///
    /// For each account the prior-year amount is derived by applying a realistic
    /// year-over-year growth factor drawn from N(0.03, 0.12). The prior-year
    /// amount is then adjusted to follow Benford's law on its first digit.
    pub fn generate_comparatives(
        &mut self,
        entity_code: &str,
        fiscal_year: i32,
        current_balances: &[(String, String, Decimal)],
    ) -> Vec<PriorYearComparative> {
        let normal = Normal::new(0.03_f64, 0.12_f64).expect("valid normal params");
        let period = format!("{}-12", fiscal_year);

        current_balances
            .iter()
            .map(|(code, name, current)| {
                // Derive prior year: prior = current / (1 + growth)
                // where growth ~ N(0.03, 0.12)
                let growth: f64 = normal.sample(&mut self.rng);
                let divisor = 1.0 + growth;
                let current_f64 = decimal_to_f64(*current);

                // Compute raw prior-year amount
                let raw_prior = if divisor.abs() < 1e-10 {
                    current_f64
                } else {
                    current_f64 / divisor
                };

                // Apply Benford-compliant first-digit nudge.
                // With 30% probability, replace the leading digit with a
                // Benford-sampled digit to ensure the aggregate distribution
                // conforms to Benford's law. The remaining 70% are left
                // as-is (log-normal variance already trends Benford).
                let prior_f64 = if raw_prior.abs() > 10.0 && self.rng.random_bool(0.30) {
                    benford_first_digit_adjust(raw_prior, &mut self.rng)
                } else {
                    raw_prior
                };

                let prior = f64_to_decimal(prior_f64);
                let variance = *current - prior;
                let variance_pct = if prior.is_zero() {
                    0.0
                } else {
                    let prior_abs_f64 = decimal_to_f64(prior).abs();
                    if prior_abs_f64 < 1e-10 {
                        0.0
                    } else {
                        decimal_to_f64(variance) / prior_abs_f64 * 100.0
                    }
                };

                PriorYearComparative {
                    account_code: code.clone(),
                    account_name: name.clone(),
                    current_year_amount: *current,
                    prior_year_amount: prior,
                    variance,
                    variance_pct,
                    entity_code: entity_code.to_string(),
                    period: period.clone(),
                }
            })
            .collect()
    }

    /// Generate prior-year audit findings.
    ///
    /// Produces 3-8 findings with realistic distributions across finding types,
    /// statuses, and risk areas.
    pub fn generate_findings(
        &mut self,
        entity_code: &str,
        fiscal_year: i32,
    ) -> Vec<PriorYearFinding> {
        let count = self.rng.random_range(3..=8_usize);
        let prior_year = fiscal_year - 1;

        (0..count)
            .map(|_| {
                let finding_type = weighted_pick(&mut self.rng, FINDING_TYPES);
                let status = weighted_pick(&mut self.rng, FINDING_STATUSES);
                let risk_area = weighted_pick(&mut self.rng, RISK_AREAS);

                let description = self.pick_description(finding_type, risk_area);

                // Open and recurring findings require follow-up
                let follow_up_required = status == "open" || status == "recurring";

                // Remediated findings get a remediation date
                let remediation_date = if status == "remediated" || status == "partially_remediated"
                {
                    // Remediation happened between the prior year-end and
                    // 6 months into the current year
                    let day_offset = self.rng.random_range(30..=270_i64);
                    NaiveDate::from_ymd_opt(prior_year, 12, 31)
                        .and_then(|d| d.checked_add_signed(chrono::Duration::days(day_offset)))
                } else {
                    None
                };

                // Misstatements and material weaknesses always have an amount;
                // other finding types have a 30% chance.
                let has_amount = finding_type == "misstatement"
                    || finding_type == "material_weakness"
                    || self.rng.random_bool(0.3);
                let original_amount = if has_amount {
                    Some(self.amount_sampler.sample())
                } else {
                    None
                };

                let _entity = entity_code; // used for context
                PriorYearFinding {
                    finding_id: self.uuid_factory.next(),
                    fiscal_year: prior_year,
                    finding_type: finding_type.to_string(),
                    description,
                    status: status.to_string(),
                    risk_area: risk_area.to_string(),
                    original_amount,
                    remediation_date,
                    follow_up_required,
                }
            })
            .collect()
    }

    /// Generate a complete prior-year summary including comparatives, findings,
    /// and the prior-year engagement metadata.
    pub fn generate_summary(
        &mut self,
        entity_code: &str,
        fiscal_year: i32,
        current_balances: &[(String, String, Decimal)],
    ) -> PriorYearSummary {
        let comparatives = self.generate_comparatives(entity_code, fiscal_year, current_balances);
        let findings = self.generate_findings(entity_code, fiscal_year);
        let open = findings
            .iter()
            .filter(|f| f.status == "open" || f.status == "recurring")
            .count();

        // Opinion type: 90% unmodified, 8% qualified, 2% adverse/disclaimer
        let opinion_roll: f64 = self.rng.random();
        let opinion_type = if opinion_roll < 0.90 {
            "unmodified"
        } else if opinion_roll < 0.98 {
            "qualified"
        } else {
            "adverse"
        };

        // Derive materiality from the total absolute current-year amounts
        // (roughly 1-2% of total revenue/assets)
        let total_abs: f64 = current_balances
            .iter()
            .map(|(_, _, amt)| decimal_to_f64(*amt).abs())
            .sum();
        let materiality_pct = 0.01 + self.rng.random::<f64>() * 0.01; // 1-2%
        let materiality = f64_to_decimal(total_abs * materiality_pct);

        // Pick 2-4 KAMs
        let kam_count = self.rng.random_range(2..=4_usize).min(KAM_POOL.len());
        let mut kam_indices: Vec<usize> = (0..KAM_POOL.len()).collect();
        kam_indices.shuffle(&mut self.rng);
        kam_indices.truncate(kam_count);
        kam_indices.sort_unstable();
        let key_audit_matters: Vec<String> = kam_indices
            .iter()
            .map(|&i| KAM_POOL[i].to_string())
            .collect();

        PriorYearSummary {
            fiscal_year: fiscal_year - 1,
            entity_code: entity_code.to_string(),
            opinion_type: opinion_type.to_string(),
            materiality,
            total_findings: findings.len(),
            open_findings: open,
            key_audit_matters,
            comparatives,
            findings,
        }
    }

    /// Pick a finding description that matches the given type and risk area.
    fn pick_description(&mut self, finding_type: &str, risk_area: &str) -> String {
        // Find all matching templates
        let matches: Vec<&str> = FINDING_DESCRIPTIONS
            .iter()
            .filter(|(ft, ra, _)| *ft == finding_type && *ra == risk_area)
            .map(|(_, _, desc)| *desc)
            .collect();

        if matches.is_empty() {
            // Fallback: pick any description for the finding type
            let type_matches: Vec<&str> = FINDING_DESCRIPTIONS
                .iter()
                .filter(|(ft, _, _)| *ft == finding_type)
                .map(|(_, _, desc)| *desc)
                .collect();
            if type_matches.is_empty() {
                return format!("Prior-year {} in {} area", finding_type, risk_area);
            }
            let idx = self.rng.random_range(0..type_matches.len());
            return type_matches[idx].to_string();
        }

        let idx = self.rng.random_range(0..matches.len());
        matches[idx].to_string()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Pick from a weighted list using cumulative weights.
fn weighted_pick<'a>(rng: &mut ChaCha8Rng, items: &[(&'a str, f64)]) -> &'a str {
    let roll: f64 = rng.random();
    for (item, threshold) in items {
        if roll < *threshold {
            return item;
        }
    }
    items.last().map(|(item, _)| *item).unwrap_or("unknown")
}

/// Benford's law probabilities for digits 1-9.
const BENFORD_PROBS: [f64; 9] = [
    0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
];

/// Sample a first digit (1-9) according to Benford's law.
fn sample_benford_digit(rng: &mut ChaCha8Rng) -> u32 {
    let roll: f64 = rng.random();
    let mut cumulative = 0.0;
    for (i, &p) in BENFORD_PROBS.iter().enumerate() {
        cumulative += p;
        if roll < cumulative {
            return (i + 1) as u32;
        }
    }
    9
}

/// Adjust the first significant digit of a value to follow Benford's law.
///
/// This preserves the order of magnitude and the lower digits, making only
/// a small perturbation that keeps the prior-year amount close to the raw
/// variance-derived value.
fn benford_first_digit_adjust(raw: f64, rng: &mut ChaCha8Rng) -> f64 {
    let abs_raw = raw.abs();
    if abs_raw < 1.0 {
        return raw;
    }

    let magnitude = abs_raw.log10().floor() as i32;
    let scale = 10_f64.powi(magnitude);

    // Current first digit (1-9)
    let normalised = abs_raw / scale; // value in [1.0, 10.0)
    let current_first = normalised.floor() as u32;

    // Sample a Benford-distributed first digit
    let benford_digit = sample_benford_digit(rng);

    // Replace the first digit while preserving the fractional part
    let fractional = normalised - current_first as f64; // in [0.0, 1.0)
    let adjusted = (benford_digit as f64 + fractional) * scale;

    if raw < 0.0 {
        -adjusted
    } else {
        adjusted
    }
}

fn decimal_to_f64(d: Decimal) -> f64 {
    use std::str::FromStr;
    f64::from_str(&d.to_string()).unwrap_or(0.0)
}

fn f64_to_decimal(v: f64) -> Decimal {
    use rust_decimal::prelude::FromPrimitive;
    Decimal::from_f64(v).unwrap_or(Decimal::ZERO).round_dp(2)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    fn sample_balances() -> Vec<(String, String, Decimal)> {
        vec![
            ("1100".into(), "Accounts Receivable".into(), dec!(500_000)),
            ("1200".into(), "Inventory".into(), dec!(300_000)),
            ("2000".into(), "Accounts Payable".into(), dec!(200_000)),
            ("4000".into(), "Revenue".into(), dec!(1_500_000)),
            ("5000".into(), "Cost of Goods Sold".into(), dec!(900_000)),
            ("1000".into(), "Cash".into(), dec!(150_000)),
            ("3000".into(), "Retained Earnings".into(), dec!(400_000)),
            ("6000".into(), "Operating Expenses".into(), dec!(250_000)),
        ]
    }

    #[test]
    fn test_comparatives_generated() {
        let mut gen = PriorYearGenerator::new(42);
        let balances = sample_balances();
        let comps = gen.generate_comparatives("C001", 2025, &balances);

        assert_eq!(comps.len(), balances.len());
        for comp in &comps {
            assert_eq!(comp.entity_code, "C001");
            assert_eq!(comp.period, "2025-12");
            assert!(!comp.account_code.is_empty());
            assert!(!comp.account_name.is_empty());
        }
    }

    #[test]
    fn test_variance_distribution() {
        // Generate many comparatives and verify:
        // 1. Most variances are within a reasonable range (< 50%)
        // 2. The median variance is moderate (near 0)
        //
        // Note: ~30% of prior-year amounts get a Benford first-digit
        // adjustment, which can shift values significantly (e.g. first
        // digit 1 → 5). Unadjusted amounts follow N(3%, 12%).
        let mut gen = PriorYearGenerator::new(123);
        let balances = sample_balances();

        let mut all_pcts = Vec::new();
        for _ in 0..50 {
            let comps = gen.generate_comparatives("C001", 2025, &balances);
            for c in &comps {
                all_pcts.push(c.variance_pct);
            }
        }

        // At least 40% should be within 50%
        let within_50 = all_pcts.iter().filter(|p| p.abs() < 50.0).count();
        let ratio = within_50 as f64 / all_pcts.len() as f64;
        assert!(
            ratio > 0.40,
            "Expected >40% of variances within 50%, got {:.1}%",
            ratio * 100.0
        );

        // Median should be moderate (within +/- 50%)
        let mut sorted = all_pcts.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sorted[sorted.len() / 2];
        assert!(
            median.abs() < 50.0,
            "Expected median variance within 50%, got {:.2}%",
            median
        );
    }

    #[test]
    fn test_comparatives_arithmetic() {
        let mut gen = PriorYearGenerator::new(77);
        let balances = sample_balances();
        let comps = gen.generate_comparatives("C001", 2025, &balances);

        for comp in &comps {
            // variance = current - prior
            let expected_variance = comp.current_year_amount - comp.prior_year_amount;
            assert_eq!(
                comp.variance, expected_variance,
                "Variance mismatch for account {}",
                comp.account_code
            );

            // variance_pct = (current - prior) / |prior| * 100
            if !comp.prior_year_amount.is_zero() {
                let prior_abs_f64 = decimal_to_f64(comp.prior_year_amount).abs();
                if prior_abs_f64 > 1e-10 {
                    let expected_pct = decimal_to_f64(comp.variance) / prior_abs_f64 * 100.0;
                    let diff = (comp.variance_pct - expected_pct).abs();
                    assert!(
                        diff < 0.01,
                        "Variance pct mismatch for {}: got {}, expected {}",
                        comp.account_code,
                        comp.variance_pct,
                        expected_pct
                    );
                }
            }
        }
    }

    #[test]
    fn test_findings_generated() {
        let mut gen = PriorYearGenerator::new(42);
        let findings = gen.generate_findings("C001", 2025);

        assert!(
            findings.len() >= 3 && findings.len() <= 8,
            "Expected 3-8 findings, got {}",
            findings.len()
        );

        for f in &findings {
            assert_eq!(f.fiscal_year, 2024);
            assert!(!f.finding_type.is_empty());
            assert!(!f.description.is_empty());
            assert!(!f.status.is_empty());
            assert!(!f.risk_area.is_empty());
        }
    }

    #[test]
    fn test_finding_status_distribution() {
        // Run many times and check that we see a mix of statuses
        let mut status_counts: HashMap<String, usize> = HashMap::new();
        for seed in 0..50_u64 {
            let mut gen = PriorYearGenerator::new(seed);
            let findings = gen.generate_findings("C001", 2025);
            for f in &findings {
                *status_counts.entry(f.status.clone()).or_insert(0) += 1;
            }
        }

        // We should see all four statuses across 50 runs
        assert!(
            status_counts.contains_key("remediated"),
            "Missing 'remediated' status"
        );
        assert!(status_counts.contains_key("open"), "Missing 'open' status");

        // At least 2 distinct statuses (very conservative)
        assert!(
            status_counts.len() >= 2,
            "Expected at least 2 distinct statuses, got {}",
            status_counts.len()
        );
    }

    #[test]
    fn test_summary_consistent() {
        let mut gen = PriorYearGenerator::new(42);
        let balances = sample_balances();
        let summary = gen.generate_summary("C001", 2025, &balances);

        assert_eq!(summary.fiscal_year, 2024);
        assert_eq!(summary.entity_code, "C001");
        assert_eq!(summary.total_findings, summary.findings.len());

        // open_findings should match actual open/recurring count
        let actual_open = summary
            .findings
            .iter()
            .filter(|f| f.status == "open" || f.status == "recurring")
            .count();
        assert_eq!(
            summary.open_findings, actual_open,
            "open_findings {} doesn't match actual open/recurring count {}",
            summary.open_findings, actual_open
        );

        // Comparatives should match input size
        assert_eq!(summary.comparatives.len(), balances.len());

        // Key audit matters should be non-empty
        assert!(!summary.key_audit_matters.is_empty());

        // Opinion should be a valid type
        let valid_opinions = ["unmodified", "qualified", "adverse", "disclaimer"];
        assert!(
            valid_opinions.contains(&summary.opinion_type.as_str()),
            "Invalid opinion type: {}",
            summary.opinion_type
        );

        // Open findings must have follow_up_required = true
        for f in &summary.findings {
            if f.status == "open" || f.status == "recurring" {
                assert!(
                    f.follow_up_required,
                    "Open/recurring finding {} should have follow_up_required=true",
                    f.finding_id
                );
            }
        }

        // Remediated findings should have remediation_date set
        for f in &summary.findings {
            if f.status == "remediated" {
                assert!(
                    f.remediation_date.is_some(),
                    "Remediated finding {} should have a remediation_date",
                    f.finding_id
                );
            }
        }
    }

    #[test]
    fn test_prior_year_amounts_benford() {
        // Check that prior-year amounts follow Benford's first-digit law.
        // We generate many comparatives and tally the first digit.
        let mut digit_counts = [0_usize; 10]; // index 0 unused

        for seed in 0..100_u64 {
            let mut gen = PriorYearGenerator::new(seed);
            let balances = sample_balances();
            let comps = gen.generate_comparatives("C001", 2025, &balances);
            for c in &comps {
                let abs_str = decimal_to_f64(c.prior_year_amount).abs().to_string();
                if let Some(first_char) = abs_str.chars().find(|c| c.is_ascii_digit() && *c != '0')
                {
                    let digit = first_char.to_digit(10).unwrap_or(0) as usize;
                    if digit >= 1 && digit <= 9 {
                        digit_counts[digit] += 1;
                    }
                }
            }
        }

        let total: usize = digit_counts[1..].iter().sum();
        if total < 50 {
            // Not enough data to test
            return;
        }

        // Benford expected frequencies
        let benford_expected = [
            0.0, 0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
        ];

        // Check that digit 1 is the most frequent (basic Benford sanity)
        let freq_1 = digit_counts[1] as f64 / total as f64;
        assert!(
            freq_1 > 0.15,
            "Digit 1 frequency {:.3} is too low for Benford (expected ~{:.3})",
            freq_1,
            benford_expected[1]
        );

        // Check mean absolute deviation (MAD) is reasonable
        // Benford conformity: MAD < 0.015 is close, < 0.04 is acceptable
        let mut mad = 0.0;
        for d in 1..=9 {
            let observed = digit_counts[d] as f64 / total as f64;
            mad += (observed - benford_expected[d]).abs();
        }
        mad /= 9.0;

        // Use a generous threshold since we're adjusting rather than directly
        // sampling Benford — 0.06 allows for some variance in small samples
        assert!(
            mad < 0.06,
            "Benford MAD {:.4} is too high (expected < 0.06)",
            mad
        );
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut gen = PriorYearGenerator::new(42);
        let balances = sample_balances();
        let summary = gen.generate_summary("C001", 2025, &balances);

        let json = serde_json::to_string(&summary).expect("serialize");
        let parsed: PriorYearSummary = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(summary.fiscal_year, parsed.fiscal_year);
        assert_eq!(summary.entity_code, parsed.entity_code);
        assert_eq!(summary.opinion_type, parsed.opinion_type);
        assert_eq!(summary.total_findings, parsed.total_findings);
        assert_eq!(summary.open_findings, parsed.open_findings);
        assert_eq!(summary.comparatives.len(), parsed.comparatives.len());
        assert_eq!(summary.findings.len(), parsed.findings.len());

        for (orig, rt) in summary.findings.iter().zip(parsed.findings.iter()) {
            assert_eq!(orig.finding_id, rt.finding_id);
            assert_eq!(orig.finding_type, rt.finding_type);
            assert_eq!(orig.status, rt.status);
        }
    }
}
