//! Audit sampling plan generator per ISA 530.
//!
//! For each Combined Risk Assessment (CRA) at Moderate or High level, this
//! generator produces a complete `SamplingPlan` and the corresponding
//! `SampledItem` records that document the actual sample drawn.
//!
//! # Sample-size logic (ISA 530 guidance)
//!
//! | CRA level | Representative items | Methodology |
//! |-----------|---------------------|-------------|
//! | Minimal   | 0 (analytical only) | — |
//! | Low       | 10–15               | MUS (balance) / Systematic (transaction) |
//! | Moderate  | 20–30               | MUS (balance) / Systematic (transaction) |
//! | High      | 40–60               | MUS (balance) / Systematic (transaction) |
//!
//! Misstatement rates are correlated with CRA level:
//! - Low: 2–5% of sampled items
//! - Moderate: 5–10%
//! - High: 10–20%
//!
//! # Key-item identification
//!
//! Key items are populated from the supplied JE amounts > tolerable error.
//! When no JE data is available, synthetic key items are generated based on
//! a fraction of the population size.

use std::collections::HashSet;

use datasynth_core::models::audit::risk_assessment_cra::{
    AuditAssertion, CombinedRiskAssessment, CraLevel,
};
use datasynth_core::models::audit::sampling_plan::{
    KeyItem, KeyItemReason, SampledItem, SamplingMethodology, SamplingPlan, SelectionType,
};
use datasynth_core::models::journal_entry::{JournalEntry, JournalEntryLine};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Choose the appropriate methodology for an assertion type.
///
/// Balance-testing assertions (Existence, Valuation) → MUS.
/// Transaction-testing assertions (Occurrence, Completeness, Accuracy, Cutoff) → Systematic.
/// Low-risk (no sampling plan generated) → HaphazardSelection.
fn methodology_for_assertion(assertion: AuditAssertion, cra: CraLevel) -> SamplingMethodology {
    use AuditAssertion::*;
    if cra == CraLevel::Minimal {
        return SamplingMethodology::HaphazardSelection;
    }
    match assertion {
        // Balance assertions → MUS
        Existence | ValuationAndAllocation | RightsAndObligations | CompletenessBalance => {
            SamplingMethodology::MonetaryUnitSampling
        }
        // Presentation → Random
        PresentationAndDisclosure => SamplingMethodology::RandomSelection,
        // Transaction assertions → Systematic
        Occurrence | Completeness | Accuracy | Cutoff | Classification => {
            SamplingMethodology::SystematicSelection
        }
    }
}

/// Derive representative sample size from CRA level (with random jitter).
fn sample_size_for_cra(rng: &mut ChaCha8Rng, cra: CraLevel) -> usize {
    match cra {
        CraLevel::Minimal => 0,
        CraLevel::Low => rng.random_range(10usize..=15),
        CraLevel::Moderate => rng.random_range(20usize..=30),
        CraLevel::High => rng.random_range(40usize..=60),
    }
}

/// Misstatement rate for a given CRA level (probability a sampled item has error).
fn misstatement_rate(cra: CraLevel) -> f64 {
    match cra {
        CraLevel::Minimal => 0.02,
        CraLevel::Low => 0.04,
        CraLevel::Moderate => 0.08,
        CraLevel::High => 0.15,
    }
}

/// Map an audit account area name to GL account code prefixes.
fn account_area_to_prefixes(account_area: &str) -> Vec<&'static str> {
    let lower = account_area.to_lowercase();
    if lower.contains("revenue") || lower.contains("sales") {
        vec!["4"]
    } else if lower.contains("receivable") {
        vec!["11"]
    } else if lower.contains("payable") {
        vec!["20"]
    } else if lower.contains("inventory") || lower.contains("stock") {
        vec!["12", "13"]
    } else if lower.contains("cash") || lower.contains("bank") {
        vec!["10"]
    } else if lower.contains("fixed asset") || lower.contains("ppe") || lower.contains("property") {
        vec!["14", "15", "16"]
    } else if lower.contains("equity") || lower.contains("capital") {
        vec!["3"]
    } else if lower.contains("expense") || lower.contains("cost") {
        vec!["5", "6"]
    } else if lower.contains("debt") || lower.contains("loan") || lower.contains("borrow") {
        vec!["23", "24"]
    } else if lower.contains("tax") {
        vec!["17", "25"]
    } else if lower.contains("provision") {
        vec!["26"]
    } else if lower.contains("intangible") || lower.contains("goodwill") {
        vec!["19"]
    } else if lower.contains("interest") {
        vec!["71"]
    } else if lower.contains("other income") || lower.contains("other expense") {
        vec!["7"]
    } else if lower.contains("depreciation") || lower.contains("amortization") {
        vec!["60"]
    } else if lower.contains("salary") || lower.contains("wages") || lower.contains("payroll") {
        vec!["61"]
    } else if lower.contains("rent") || lower.contains("lease") {
        vec!["63"]
    } else {
        vec![] // Empty = use all JE lines as fallback
    }
}

/// Filter JE lines matching the account area's GL prefixes.
/// Returns (JournalEntry ref, JournalEntryLine ref, absolute amount) tuples.
fn filter_je_lines_for_area<'a>(
    entries: &'a [JournalEntry],
    account_area: &str,
) -> Vec<(&'a JournalEntry, &'a JournalEntryLine, Decimal)> {
    let prefixes = account_area_to_prefixes(account_area);
    let mut results = Vec::new();

    for je in entries {
        for line in &je.lines {
            let matches = if prefixes.is_empty() {
                true
            } else {
                prefixes.iter().any(|p| line.account_code.starts_with(p))
            };
            if matches {
                let amount = (line.debit_amount - line.credit_amount).abs();
                if amount > Decimal::ZERO {
                    results.push((je, line, amount));
                }
            }
        }
    }
    results
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the sampling plan generator.
#[derive(Debug, Clone)]
pub struct SamplingPlanGeneratorConfig {
    /// Fraction of the population that consists of key items (0.0–1.0).
    /// Applied when no external JE data is supplied.
    pub key_item_fraction: f64,
    /// Minimum population size assumed when no JE data is available.
    pub min_population_size: usize,
    /// Maximum population size assumed when no JE data is available.
    pub max_population_size: usize,
    /// Base population value (monetary) when no JE data is available.
    pub base_population_value: Decimal,
}

impl Default for SamplingPlanGeneratorConfig {
    fn default() -> Self {
        Self {
            key_item_fraction: 0.05, // 5% of items selected as key items
            min_population_size: 100,
            max_population_size: 2_000,
            base_population_value: dec!(5_000_000),
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for ISA 530 sampling plans and sampled items.
pub struct SamplingPlanGenerator {
    rng: ChaCha8Rng,
    config: SamplingPlanGeneratorConfig,
}

impl SamplingPlanGenerator {
    /// Create a new generator with default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x530), // discriminator for ISA 530
            config: SamplingPlanGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: SamplingPlanGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x530),
            config,
        }
    }

    /// Generate sampling plans and sampled items for all CRAs at Moderate or higher.
    ///
    /// # Arguments
    /// * `cras` — All combined risk assessments for one or more entities.
    /// * `tolerable_error` — Performance materiality / tolerable error for the entity.
    ///   When `None`, a synthetic TE of 5% of the base population value is used.
    ///
    /// Returns `(plans, sampled_items)` — the plans and the flat list of all sampled items.
    pub fn generate_for_cras(
        &mut self,
        cras: &[CombinedRiskAssessment],
        tolerable_error: Option<Decimal>,
    ) -> (Vec<SamplingPlan>, Vec<SampledItem>) {
        info!("Generating sampling plans for {} CRAs", cras.len());
        let mut plans: Vec<SamplingPlan> = Vec::new();
        let mut all_items: Vec<SampledItem> = Vec::new();

        for cra in cras {
            // Only generate plans for Moderate and High CRA levels
            if cra.combined_risk < CraLevel::Moderate {
                continue;
            }

            let te =
                tolerable_error.unwrap_or_else(|| self.config.base_population_value * dec!(0.05));

            let (plan, items) = self.generate_plan(cra, te);
            all_items.extend(items);
            plans.push(plan);
        }

        info!(
            "Generated {} sampling plans with {} sampled items",
            plans.len(),
            all_items.len()
        );
        (plans, all_items)
    }

    /// Generate a single sampling plan for one CRA.
    fn generate_plan(
        &mut self,
        cra: &CombinedRiskAssessment,
        tolerable_error: Decimal,
    ) -> (SamplingPlan, Vec<SampledItem>) {
        let methodology = methodology_for_assertion(cra.assertion, cra.combined_risk);
        let rep_sample_size = sample_size_for_cra(&mut self.rng, cra.combined_risk);

        // Synthesise population size and value
        let pop_size = self
            .rng
            .random_range(self.config.min_population_size..=self.config.max_population_size);
        let pop_value = self.synthetic_population_value(pop_size);

        // Generate key items
        let key_items = self.generate_key_items(pop_size, pop_value, tolerable_error, cra);
        let key_items_value: Decimal = key_items.iter().map(|k| k.amount).sum();
        let remaining_value = (pop_value - key_items_value).max(Decimal::ZERO);

        // Compute sampling interval
        let sampling_interval = if rep_sample_size > 0 && remaining_value > Decimal::ZERO {
            remaining_value / Decimal::from(rep_sample_size as i64)
        } else {
            Decimal::ZERO
        };

        let plan_id = format!(
            "SP-{}-{}-{}",
            cra.entity_code,
            cra.account_area.replace(' ', "_").to_uppercase(),
            format!("{:?}", cra.assertion).to_uppercase(),
        );

        let plan = SamplingPlan {
            id: plan_id.clone(),
            entity_code: cra.entity_code.clone(),
            account_area: cra.account_area.clone(),
            assertion: format!("{}", cra.assertion),
            methodology,
            population_size: pop_size,
            population_value: pop_value,
            key_items: key_items.clone(),
            key_items_value,
            remaining_population_value: remaining_value,
            sample_size: rep_sample_size,
            sampling_interval,
            cra_level: cra.combined_risk.to_string(),
            tolerable_error,
        };

        // Build SampledItems: key items (always tested) + representative items
        let mut sampled_items: Vec<SampledItem> = Vec::new();
        let misstatement_p = misstatement_rate(cra.combined_risk);

        // Key items — always tested
        for ki in &key_items {
            let misstatement_found: bool = self.rng.random::<f64>() < misstatement_p;
            let misstatement_amount = if misstatement_found {
                let pct = Decimal::try_from(self.rng.random_range(0.01_f64..=0.15_f64))
                    .unwrap_or(dec!(0.05));
                Some((ki.amount * pct).round_dp(2))
            } else {
                None
            };

            sampled_items.push(SampledItem {
                item_id: ki.item_id.clone(),
                sampling_plan_id: plan_id.clone(),
                amount: ki.amount,
                selection_type: SelectionType::KeyItem,
                tested: true,
                misstatement_found,
                misstatement_amount,
            });
        }

        // Representative items
        if rep_sample_size > 0 && remaining_value > Decimal::ZERO {
            let avg_remaining_item_value =
                remaining_value / Decimal::from((pop_size - key_items.len()).max(1) as i64);

            for i in 0..rep_sample_size {
                let item_id = format!("{plan_id}-REP-{i:04}");
                // Jitter the amount around the average remaining item value
                let jitter_pct = Decimal::try_from(self.rng.random_range(0.5_f64..=2.0_f64))
                    .unwrap_or(Decimal::ONE);
                let amount = (avg_remaining_item_value * jitter_pct)
                    .round_dp(2)
                    .max(dec!(1));

                let misstatement_found: bool = self.rng.random::<f64>() < misstatement_p;
                let misstatement_amount = if misstatement_found {
                    let pct = Decimal::try_from(self.rng.random_range(0.01_f64..=0.30_f64))
                        .unwrap_or(dec!(0.05));
                    Some((amount * pct).round_dp(2))
                } else {
                    None
                };

                sampled_items.push(SampledItem {
                    item_id,
                    sampling_plan_id: plan_id.clone(),
                    amount,
                    selection_type: SelectionType::Representative,
                    tested: true,
                    misstatement_found,
                    misstatement_amount,
                });
            }
        }

        (plan, sampled_items)
    }

    /// Synthesise a realistic population value from the population size.
    fn synthetic_population_value(&mut self, pop_size: usize) -> Decimal {
        // Average item value varies from $500 (routine small transactions) to $50,000 (large balances)
        let avg_item = self.rng.random_range(500_i64..=50_000);
        let raw = Decimal::from(pop_size as i64) * Decimal::from(avg_item);
        // Round to nearest 1000
        ((raw / dec!(1000)).round() * dec!(1000)).max(dec!(10_000))
    }

    /// Generate key items for the population.
    ///
    /// Key items are synthesised as items with amounts above the tolerable error.
    /// The number of key items is driven by the key_item_fraction config and
    /// whether the CRA is High (more key items for high-risk areas).
    fn generate_key_items(
        &mut self,
        pop_size: usize,
        pop_value: Decimal,
        tolerable_error: Decimal,
        cra: &CombinedRiskAssessment,
    ) -> Vec<KeyItem> {
        let fraction = match cra.combined_risk {
            CraLevel::High => self.config.key_item_fraction * 2.0,
            _ => self.config.key_item_fraction,
        };
        let n_key_items = ((pop_size as f64 * fraction) as usize).clamp(1, 20);

        // Distribute the key item value: each key item is > TE
        let avg_key_value = pop_value
            * Decimal::try_from(self.config.key_item_fraction * 3.0).unwrap_or(dec!(0.15))
            / Decimal::from(n_key_items as i64);
        let key_item_min = tolerable_error * dec!(1.01); // just above TE
        let key_item_max = (avg_key_value * dec!(2)).max(key_item_min * dec!(2)); // ensure max > min

        let mut items = Vec::with_capacity(n_key_items);
        for i in 0..n_key_items {
            let amount_f = self.rng.random_range(
                key_item_min.to_string().parse::<f64>().unwrap_or(10_000.0)
                    ..=key_item_max.to_string().parse::<f64>().unwrap_or(500_000.0),
            );
            let amount = Decimal::try_from(amount_f)
                .unwrap_or(key_item_min)
                .round_dp(2)
                .max(key_item_min);

            let reason = self.pick_key_item_reason(cra, i);

            items.push(KeyItem {
                item_id: format!(
                    "{}-{}-KEY-{i:03}",
                    cra.entity_code,
                    cra.account_area.replace(' ', "_").to_uppercase()
                ),
                amount,
                reason,
            });
        }

        // Guard: key items must not exceed the population value (they are a subset of it).
        // If they do, scale all amounts down proportionally so their total is 80% of the
        // population value, leaving room for representative items.
        let key_total: Decimal = items.iter().map(|k| k.amount).sum();
        if key_total > pop_value {
            let scale = (pop_value * dec!(0.8)) / key_total;
            for item in &mut items {
                item.amount = (item.amount * scale).round_dp(2);
            }
        }

        items
    }

    /// Choose a key item reason based on the CRA characteristics.
    fn pick_key_item_reason(
        &mut self,
        cra: &CombinedRiskAssessment,
        index: usize,
    ) -> KeyItemReason {
        // First item is always AboveTolerableError (primary reason)
        if index == 0 {
            return KeyItemReason::AboveTolerableError;
        }
        // Significant risks generate management override / high risk flags
        if cra.significant_risk {
            let roll: f64 = self.rng.random();
            if roll < 0.40 {
                return KeyItemReason::ManagementOverride;
            }
            if roll < 0.70 {
                return KeyItemReason::HighRisk;
            }
        }
        let roll: f64 = self.rng.random();
        if roll < 0.60 {
            KeyItemReason::AboveTolerableError
        } else if roll < 0.80 {
            KeyItemReason::UnusualNature
        } else {
            KeyItemReason::HighRisk
        }
    }

    // -----------------------------------------------------------------------
    // JE-aware sampling (population-based)
    // -----------------------------------------------------------------------

    /// Generate sampling plans using real journal entry population data.
    ///
    /// Key items are actual JE lines with amount > tolerable_error.
    /// Representative items are sampled from the remaining JE population.
    /// Falls back to synthetic generation for CRAs with no matching JE lines.
    pub fn generate_for_cras_with_population(
        &mut self,
        cras: &[CombinedRiskAssessment],
        tolerable_error: Option<Decimal>,
        journal_entries: &[JournalEntry],
    ) -> (Vec<SamplingPlan>, Vec<SampledItem>) {
        info!(
            "Generating JE-aware sampling plans for {} CRAs against {} journal entries",
            cras.len(),
            journal_entries.len()
        );
        let mut plans: Vec<SamplingPlan> = Vec::new();
        let mut all_items: Vec<SampledItem> = Vec::new();

        for cra in cras {
            // Only generate plans for Moderate and High CRA levels
            if cra.combined_risk < CraLevel::Moderate {
                continue;
            }

            let te =
                tolerable_error.unwrap_or_else(|| self.config.base_population_value * dec!(0.05));

            let matching_lines = filter_je_lines_for_area(journal_entries, &cra.account_area);

            let (plan, items) = if matching_lines.is_empty() {
                // Fallback to synthetic generation when no JE lines match
                self.generate_plan(cra, te)
            } else {
                self.generate_plan_from_population(cra, te, &matching_lines)
            };

            all_items.extend(items);
            plans.push(plan);
        }

        info!(
            "Generated {} JE-aware sampling plans with {} sampled items",
            plans.len(),
            all_items.len()
        );
        (plans, all_items)
    }

    /// Generate a sampling plan from a real JE population for one CRA.
    fn generate_plan_from_population(
        &mut self,
        cra: &CombinedRiskAssessment,
        tolerable_error: Decimal,
        matching_lines: &[(&JournalEntry, &JournalEntryLine, Decimal)],
    ) -> (SamplingPlan, Vec<SampledItem>) {
        let methodology = methodology_for_assertion(cra.assertion, cra.combined_risk);
        let rep_sample_size = sample_size_for_cra(&mut self.rng, cra.combined_risk);

        // Compute real population metrics
        let population_size = matching_lines.len();
        let population_value: Decimal = matching_lines.iter().map(|(_, _, amt)| *amt).sum();

        // Sort lines descending by amount for key item selection
        let mut sorted_lines: Vec<_> = matching_lines.to_vec();
        sorted_lines.sort_by(|a, b| b.2.cmp(&a.2));

        // Select key items: lines where amount > tolerable_error, capped at 20
        let mut key_items: Vec<KeyItem> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        for (idx, (je, _line, amount)) in sorted_lines.iter().enumerate() {
            if *amount <= tolerable_error {
                break;
            }
            if key_items.len() >= 20 {
                break;
            }
            let je_id = je.header.document_id.to_string();
            // Skip duplicate JE IDs (same JE may have multiple matching lines)
            if seen_ids.contains(&je_id) {
                continue;
            }
            seen_ids.insert(je_id.clone());
            let reason = self.pick_key_item_reason(cra, idx);
            key_items.push(KeyItem {
                item_id: je_id,
                amount: *amount,
                reason,
            });
        }

        let key_items_value: Decimal = key_items.iter().map(|k| k.amount).sum();
        let remaining_value = (population_value - key_items_value).max(Decimal::ZERO);

        // Select representative items from remaining lines using systematic selection
        let remaining: Vec<_> = sorted_lines
            .iter()
            .filter(|(je, _, _)| !seen_ids.contains(&je.header.document_id.to_string()))
            .collect();
        let actual_rep_size = rep_sample_size.min(remaining.len());
        let step = if actual_rep_size > 0 {
            remaining.len() / actual_rep_size
        } else {
            0
        };
        let start = if step > 0 {
            self.rng.random_range(0..step)
        } else {
            0
        };

        // Compute sampling interval
        let sampling_interval = if actual_rep_size > 0 && remaining_value > Decimal::ZERO {
            remaining_value / Decimal::from(actual_rep_size as i64)
        } else {
            Decimal::ZERO
        };

        let plan_id = format!(
            "SP-{}-{}-{}",
            cra.entity_code,
            cra.account_area.replace(' ', "_").to_uppercase(),
            format!("{:?}", cra.assertion).to_uppercase(),
        );

        let plan = SamplingPlan {
            id: plan_id.clone(),
            entity_code: cra.entity_code.clone(),
            account_area: cra.account_area.clone(),
            assertion: format!("{}", cra.assertion),
            methodology,
            population_size,
            population_value,
            key_items: key_items.clone(),
            key_items_value,
            remaining_population_value: remaining_value,
            sample_size: actual_rep_size,
            sampling_interval,
            cra_level: cra.combined_risk.to_string(),
            tolerable_error,
        };

        // Build SampledItems
        let mut sampled_items: Vec<SampledItem> = Vec::new();
        let misstatement_p = misstatement_rate(cra.combined_risk);

        // Key items — always tested
        for ki in &key_items {
            let misstatement_found: bool = self.rng.random::<f64>() < misstatement_p;
            let misstatement_amount = if misstatement_found {
                let pct = Decimal::try_from(self.rng.random_range(0.01_f64..=0.15_f64))
                    .unwrap_or(dec!(0.05));
                Some((ki.amount * pct).round_dp(2))
            } else {
                None
            };

            sampled_items.push(SampledItem {
                item_id: ki.item_id.clone(),
                sampling_plan_id: plan_id.clone(),
                amount: ki.amount,
                selection_type: SelectionType::KeyItem,
                tested: true,
                misstatement_found,
                misstatement_amount,
            });
        }

        // Representative items via systematic selection
        if actual_rep_size > 0 && step > 0 {
            let mut rep_seen: HashSet<String> = HashSet::new();
            for i in 0..actual_rep_size {
                let idx = (start + i * step) % remaining.len();
                let (je, _line, amount) = remaining[idx];
                let je_id = je.header.document_id.to_string();

                // Avoid duplicate representative items
                if rep_seen.contains(&je_id) {
                    continue;
                }
                rep_seen.insert(je_id.clone());

                let misstatement_found: bool = self.rng.random::<f64>() < misstatement_p;
                let misstatement_amount = if misstatement_found {
                    let pct = Decimal::try_from(self.rng.random_range(0.01_f64..=0.30_f64))
                        .unwrap_or(dec!(0.05));
                    Some((amount * pct).round_dp(2))
                } else {
                    None
                };

                sampled_items.push(SampledItem {
                    item_id: je_id,
                    sampling_plan_id: plan_id.clone(),
                    amount: *amount,
                    selection_type: SelectionType::Representative,
                    tested: true,
                    misstatement_found,
                    misstatement_amount,
                });
            }
        }

        (plan, sampled_items)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::audit::risk_assessment_cra::RiskRating;
    use rust_decimal_macros::dec;

    fn make_cra(
        account_area: &str,
        assertion: AuditAssertion,
        ir: RiskRating,
        cr: RiskRating,
    ) -> CombinedRiskAssessment {
        CombinedRiskAssessment::new("C001", account_area, assertion, ir, cr, false, vec![])
    }

    #[test]
    fn moderate_cra_generates_plan() {
        let cra = make_cra(
            "Trade Receivables",
            AuditAssertion::Existence,
            RiskRating::Medium,
            RiskRating::Medium,
        );
        assert_eq!(cra.combined_risk, CraLevel::Moderate);

        let mut gen = SamplingPlanGenerator::new(42);
        let (plans, items) = gen.generate_for_cras(&[cra], Some(dec!(32_500)));

        assert_eq!(
            plans.len(),
            1,
            "Should generate exactly one plan for Moderate CRA"
        );
        let plan = &plans[0];
        assert!(!items.is_empty(), "Should generate sampled items");
        assert!(
            plan.sample_size >= 20 && plan.sample_size <= 30,
            "Moderate CRA sample size 20–30"
        );
    }

    #[test]
    fn low_cra_skipped() {
        let cra = make_cra(
            "Cash",
            AuditAssertion::Existence,
            RiskRating::Low,
            RiskRating::Low,
        );
        assert_eq!(cra.combined_risk, CraLevel::Minimal);

        let mut gen = SamplingPlanGenerator::new(42);
        let (plans, _items) = gen.generate_for_cras(&[cra], Some(dec!(32_500)));

        assert!(
            plans.is_empty(),
            "Minimal CRA should produce no sampling plan"
        );
    }

    #[test]
    fn high_cra_large_sample() {
        let cra = make_cra(
            "Revenue",
            AuditAssertion::Occurrence,
            RiskRating::High,
            RiskRating::High,
        );
        assert_eq!(cra.combined_risk, CraLevel::High);

        let mut gen = SamplingPlanGenerator::new(99);
        let (plans, _) = gen.generate_for_cras(&[cra], Some(dec!(32_500)));

        assert_eq!(plans.len(), 1);
        let plan = &plans[0];
        assert!(
            plan.sample_size >= 40,
            "High CRA sample size should be 40–60"
        );
    }

    #[test]
    fn key_items_all_above_tolerable_error() {
        let cra = make_cra(
            "Provisions",
            AuditAssertion::ValuationAndAllocation,
            RiskRating::High,
            RiskRating::Medium,
        );

        let mut gen = SamplingPlanGenerator::new(7);
        let te = dec!(32_500);
        let (plans, _) = gen.generate_for_cras(&[cra], Some(te));

        assert!(!plans.is_empty());
        let plan = &plans[0];
        for ki in &plan.key_items {
            assert!(
                ki.amount >= te,
                "Key item amount {} must be >= tolerable error {}",
                ki.amount,
                te
            );
        }
    }

    #[test]
    fn sampling_interval_formula() {
        let cra = make_cra(
            "Inventory",
            AuditAssertion::Existence,
            RiskRating::High,
            RiskRating::Medium,
        );

        let mut gen = SamplingPlanGenerator::new(13);
        let te = dec!(32_500);
        let (plans, _) = gen.generate_for_cras(&[cra], Some(te));

        assert!(!plans.is_empty());
        let plan = &plans[0];
        if plan.sample_size > 0 && plan.remaining_population_value > Decimal::ZERO {
            let expected_interval =
                plan.remaining_population_value / Decimal::from(plan.sample_size as i64);
            // Allow 1 cent rounding tolerance
            let diff = (plan.sampling_interval - expected_interval).abs();
            assert!(
                diff < dec!(0.01),
                "Interval {} ≠ remaining/sample_size {}",
                plan.sampling_interval,
                expected_interval
            );
        }
    }

    #[test]
    fn balance_assertion_uses_mus() {
        let cra = make_cra(
            "Trade Receivables",
            AuditAssertion::Existence,
            RiskRating::Medium,
            RiskRating::Medium,
        );
        let methodology = methodology_for_assertion(cra.assertion, CraLevel::Moderate);
        assert_eq!(methodology, SamplingMethodology::MonetaryUnitSampling);
    }

    #[test]
    fn transaction_assertion_uses_systematic() {
        let methodology = methodology_for_assertion(AuditAssertion::Occurrence, CraLevel::Moderate);
        assert_eq!(methodology, SamplingMethodology::SystematicSelection);
    }

    #[test]
    fn all_sampled_items_have_plan_id() {
        let cras = vec![
            make_cra(
                "Revenue",
                AuditAssertion::Occurrence,
                RiskRating::High,
                RiskRating::Medium,
            ),
            make_cra(
                "Inventory",
                AuditAssertion::Existence,
                RiskRating::High,
                RiskRating::Low,
            ),
        ];

        let mut gen = SamplingPlanGenerator::new(55);
        let te = dec!(32_500);
        let (plans, items) = gen.generate_for_cras(&cras, Some(te));

        assert!(!plans.is_empty());
        assert!(!items.is_empty());
        // Verify at least some items have tested=true
        assert!(
            items.iter().all(|i| i.tested),
            "All items should be marked tested"
        );
    }
}
