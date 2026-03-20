//! Analytical relationship generator — ISA 520.
//!
//! Generates 8–12 standard analytical relationships per entity from the actual
//! journal entry data.  Each relationship captures the formula, the current
//! period value derived from JE totals, 2–3 simulated prior-period data points,
//! an expected range based on industry norms, and any variance explanation when
//! the current value falls outside the expected range.
//!
//! # Relationships generated
//!
//! | # | Name                       | Formula                              | Expected range |
//! |---|----------------------------|--------------------------------------|----------------|
//! | 1 | DSO                        | AR / Revenue × 365                   | 30–60 days     |
//! | 2 | DPO                        | AP / COGS × 365                      | 30–45 days     |
//! | 3 | Inventory Turnover         | COGS / Inventory                     | 4–12×          |
//! | 4 | Gross Margin               | (Revenue − COGS) / Revenue           | 30–60%         |
//! | 5 | Payroll to Revenue         | Payroll / Revenue                    | 15–40%         |
//! | 6 | Depreciation to Gross FA   | Depreciation / Gross FA              | 5–15%          |
//! | 7 | Revenue Growth             | Current Revenue / Prior Revenue − 1  | −10% to +20%   |
//! | 8 | Operating Expense Ratio    | OpEx / Revenue                       | 20–40%         |

use datasynth_core::models::audit::analytical_relationships::{
    AnalyticalRelationship, DataReliability, PeriodDataPoint, RelationshipType, SupportingMetric,
};
use datasynth_core::models::journal_entry::JournalEntry;
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ---------------------------------------------------------------------------
// Account ranges (using account prefix conventions from datasynth_core::accounts)
// ---------------------------------------------------------------------------

// Revenue: 4xxx
const REVENUE_PREFIX: char = '4';
// COGS: 5xxx
const COGS_PREFIX: char = '5';
// Operating expenses: 6xxx
const OPEX_PREFIX: char = '6';
// AR control: 1100
const AR_ACCOUNT: &str = "1100";
// AP control: 2000
const AP_ACCOUNT: &str = "2000";
// Inventory: 1200
const INVENTORY_ACCOUNT: &str = "1200";
// Fixed assets: 1500
const FA_ACCOUNT: &str = "1500";
// Depreciation expense: 6000
const DEPRECIATION_ACCOUNT: &str = "6000";
// Salaries + benefits: 6100, 6200
const PAYROLL_ACCOUNTS: &[&str] = &["6100", "6200"];

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the analytical relationship generator.
#[derive(Debug, Clone)]
pub struct AnalyticalRelationshipGeneratorConfig {
    /// Number of historical comparison periods to generate (excluding current).
    pub historical_periods: usize,
    /// Maximum ±% variation between historical periods (0.0–1.0).
    pub historical_variation: f64,
    /// When `true`, include supporting non-financial metrics for each relationship.
    pub include_supporting_metrics: bool,
}

impl Default for AnalyticalRelationshipGeneratorConfig {
    fn default() -> Self {
        Self {
            historical_periods: 3,
            historical_variation: 0.12,
            include_supporting_metrics: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generator for ISA 520 analytical relationships.
pub struct AnalyticalRelationshipGenerator {
    rng: ChaCha8Rng,
    config: AnalyticalRelationshipGeneratorConfig,
}

impl AnalyticalRelationshipGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0x5201),
            config: AnalyticalRelationshipGeneratorConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: AnalyticalRelationshipGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0x5201),
            config,
        }
    }

    /// Generate 8–12 analytical relationships for a single entity.
    ///
    /// # Arguments
    /// * `entity_code` — Company / entity code.
    /// * `entries` — All journal entries for the full dataset (filtered internally).
    /// * `current_period_label` — Human-readable label for the current period (e.g. "FY2024").
    /// * `prior_period_label` — Label for the immediately preceding period (e.g. "FY2023").
    ///   Used to simulate prior-period data points.
    pub fn generate_for_entity(
        &mut self,
        entity_code: &str,
        entries: &[JournalEntry],
        current_period_label: &str,
        prior_period_label: &str,
    ) -> Vec<AnalyticalRelationship> {
        // Filter entries to this entity
        let entity_entries: Vec<&JournalEntry> = entries
            .iter()
            .filter(|e| e.header.company_code == entity_code)
            .collect();

        // ---- Extract raw financial amounts from JEs -------------------------
        let revenue = sum_account_prefix(
            &entity_entries,
            REVENUE_PREFIX,
            true, /* credit normal */
        );
        let cogs = sum_account_prefix(&entity_entries, COGS_PREFIX, false /* debit normal */);
        let opex = sum_account_prefix(&entity_entries, OPEX_PREFIX, false);
        let ar = sum_account_exact(&entity_entries, AR_ACCOUNT, false); // debit balance
        let ap = sum_account_exact(&entity_entries, AP_ACCOUNT, true); // credit balance
        let inventory = sum_account_exact(&entity_entries, INVENTORY_ACCOUNT, false);
        let gross_fa = sum_account_exact(&entity_entries, FA_ACCOUNT, false);
        let depreciation = sum_account_exact(&entity_entries, DEPRECIATION_ACCOUNT, false);
        let payroll = PAYROLL_ACCOUNTS
            .iter()
            .map(|acct| sum_account_exact(&entity_entries, acct, false))
            .fold(Decimal::ZERO, |acc, v| acc + v);

        // Simulate a prior-period revenue (used for revenue growth calculation)
        // ~5-15% lower than current to give a realistic growth trend
        let prior_revenue_factor = dec!(1)
            - Decimal::try_from(self.rng.random_range(5i64..=15) as f64 / 100.0)
                .unwrap_or(dec!(0.10));
        let prior_revenue = if revenue > Decimal::ZERO {
            revenue * prior_revenue_factor
        } else {
            Decimal::ZERO
        };

        let mut relationships: Vec<AnalyticalRelationship> = Vec::new();
        let mut counter: u32 = 0;

        // Helper closure to generate the next ID (no RNG needed)
        let mut next_id = {
            let ec = entity_code.to_string();
            move || {
                counter += 1;
                format!("AR-{}-{:03}", ec, counter)
            }
        };

        // Pre-generate all random supporting metric values so we don't
        // need to borrow `self.rng` simultaneously with `self.build_relationship`.
        let rng = &mut self.rng;
        let include_metrics = self.config.include_supporting_metrics;

        let sm_dso: Vec<SupportingMetric> = if include_metrics {
            vec![SupportingMetric {
                metric_name: "Number of active customers".to_string(),
                value: Decimal::from(rng.random_range(50u64..=500)),
                source: "CRM system".to_string(),
            }]
        } else {
            vec![]
        };

        let sm_dpo: Vec<SupportingMetric> = if include_metrics {
            vec![SupportingMetric {
                metric_name: "Number of active vendors".to_string(),
                value: Decimal::from(rng.random_range(20u64..=200)),
                source: "Procurement system".to_string(),
            }]
        } else {
            vec![]
        };

        let sm_inv: Vec<SupportingMetric> = if include_metrics {
            vec![SupportingMetric {
                metric_name: "SKUs / product lines".to_string(),
                value: Decimal::from(rng.random_range(50u64..=2000)),
                source: "Warehouse management system".to_string(),
            }]
        } else {
            vec![]
        };

        let sm_payroll: Vec<SupportingMetric> = if include_metrics {
            vec![SupportingMetric {
                metric_name: "Full-time equivalent headcount".to_string(),
                value: Decimal::from(rng.random_range(25u64..=2000)),
                source: "HR system".to_string(),
            }]
        } else {
            vec![]
        };

        let sm_dep: Vec<SupportingMetric> = if include_metrics {
            vec![SupportingMetric {
                metric_name: "Average useful life (years)".to_string(),
                value: Decimal::from(rng.random_range(5u64..=20)),
                source: "Fixed asset register".to_string(),
            }]
        } else {
            vec![]
        };

        let sm_rev_growth: Vec<SupportingMetric> = if include_metrics {
            vec![SupportingMetric {
                metric_name: "Units sold / services delivered".to_string(),
                value: Decimal::from(rng.random_range(1000u64..=100_000)),
                source: "Sales / order management system".to_string(),
            }]
        } else {
            vec![]
        };

        // ---- 1. DSO ---------------------------------------------------------
        let dso = compute_dso(ar, revenue);
        relationships.push(self.build_relationship(
            next_id(),
            entity_code,
            "Days Sales Outstanding (DSO)",
            "Receivables",
            RelationshipType::Ratio,
            "AR / Revenue × 365 = DSO",
            dso,
            (dec!(30), dec!(60)),
            "days",
            current_period_label,
            prior_period_label,
            DataReliability::High,
            sm_dso,
        ));

        // ---- 2. DPO ---------------------------------------------------------
        let dpo = compute_dpo(ap, cogs);
        relationships.push(self.build_relationship(
            next_id(),
            entity_code,
            "Days Payable Outstanding (DPO)",
            "Payables",
            RelationshipType::Ratio,
            "AP / COGS × 365 = DPO",
            dpo,
            (dec!(30), dec!(45)),
            "days",
            current_period_label,
            prior_period_label,
            DataReliability::High,
            sm_dpo,
        ));

        // ---- 3. Inventory Turnover ------------------------------------------
        let inv_turnover = compute_inventory_turnover(cogs, inventory);
        relationships.push(self.build_relationship(
            next_id(),
            entity_code,
            "Inventory Turnover",
            "Inventory",
            RelationshipType::Ratio,
            "COGS / Average Inventory = Inventory Turnover",
            inv_turnover,
            (dec!(4), dec!(12)),
            "times",
            current_period_label,
            prior_period_label,
            DataReliability::High,
            sm_inv,
        ));

        // ---- 4. Gross Margin ------------------------------------------------
        let gross_margin = compute_gross_margin(revenue, cogs);
        relationships.push(self.build_relationship(
            next_id(),
            entity_code,
            "Gross Margin",
            "Revenue / Cost of Sales",
            RelationshipType::Ratio,
            "(Revenue − COGS) / Revenue × 100 = Gross Margin %",
            gross_margin,
            (dec!(30), dec!(60)),
            "%",
            current_period_label,
            prior_period_label,
            DataReliability::High,
            vec![],
        ));

        // ---- 5. Payroll to Revenue ------------------------------------------
        let payroll_ratio = compute_ratio(payroll, revenue, dec!(100));
        relationships.push(self.build_relationship(
            next_id(),
            entity_code,
            "Payroll to Revenue",
            "Payroll / Human Resources",
            RelationshipType::Correlation,
            "Payroll Expense / Revenue × 100 = Payroll Ratio %",
            payroll_ratio,
            (dec!(15), dec!(40)),
            "%",
            current_period_label,
            prior_period_label,
            DataReliability::High,
            sm_payroll,
        ));

        // ---- 6. Depreciation to Gross FA ------------------------------------
        let dep_ratio = compute_ratio(depreciation, gross_fa, dec!(100));
        relationships.push(self.build_relationship(
            next_id(),
            entity_code,
            "Depreciation to Gross Fixed Assets",
            "Fixed Assets",
            RelationshipType::Reasonableness,
            "Depreciation Expense / Gross Fixed Assets × 100 = Depreciation Rate %",
            dep_ratio,
            (dec!(5), dec!(15)),
            "%",
            current_period_label,
            prior_period_label,
            DataReliability::High,
            sm_dep,
        ));

        // ---- 7. Revenue Growth ----------------------------------------------
        let revenue_growth = compute_growth(revenue, prior_revenue);
        relationships.push(self.build_relationship(
            next_id(),
            entity_code,
            "Revenue Growth",
            "Revenue",
            RelationshipType::Trend,
            "( Current Revenue / Prior Revenue − 1 ) × 100 = Revenue Growth %",
            revenue_growth,
            (dec!(-10), dec!(20)),
            "%",
            current_period_label,
            prior_period_label,
            DataReliability::High,
            sm_rev_growth,
        ));

        // ---- 8. Operating Expense Ratio -------------------------------------
        let opex_ratio = compute_ratio(opex, revenue, dec!(100));
        relationships.push(self.build_relationship(
            next_id(),
            entity_code,
            "Operating Expense Ratio",
            "Operating Expenses",
            RelationshipType::Ratio,
            "Operating Expenses / Revenue × 100 = OpEx Ratio %",
            opex_ratio,
            (dec!(20), dec!(40)),
            "%",
            current_period_label,
            prior_period_label,
            DataReliability::Medium,
            vec![],
        ));

        relationships
    }

    /// Generate relationships for multiple entities.
    pub fn generate_for_entities(
        &mut self,
        entity_codes: &[String],
        entries: &[JournalEntry],
        current_period_label: &str,
        prior_period_label: &str,
    ) -> Vec<AnalyticalRelationship> {
        entity_codes
            .iter()
            .flat_map(|code| {
                self.generate_for_entity(code, entries, current_period_label, prior_period_label)
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Private builder
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn build_relationship(
        &mut self,
        id: String,
        entity_code: &str,
        name: &str,
        account_area: &str,
        rel_type: RelationshipType,
        formula: &str,
        current_value: Decimal,
        expected_range: (Decimal, Decimal),
        unit: &str,
        current_period_label: &str,
        prior_period_label: &str,
        reliability: DataReliability,
        supporting_metrics: Vec<SupportingMetric>,
    ) -> AnalyticalRelationship {
        let (lo, hi) = expected_range;
        let within_expected_range = current_value >= lo && current_value <= hi;

        // ---- Generate historical periods ------------------------------------
        let mut periods: Vec<PeriodDataPoint> = Vec::new();

        // Add historical periods in chronological order (oldest first)
        for i in (1..=self.config.historical_periods).rev() {
            let variation: f64 = self
                .rng
                .random_range(-self.config.historical_variation..=self.config.historical_variation);
            let factor = Decimal::try_from(1.0 + variation).unwrap_or(dec!(1));
            let historical_value = clamp_positive(current_value * factor);
            let label = if i == 1 {
                prior_period_label.to_string()
            } else {
                format!("{}-{}", prior_period_label, i)
            };
            periods.push(PeriodDataPoint {
                period: label,
                value: historical_value,
                is_current: false,
            });
        }

        // Add current period last
        periods.push(PeriodDataPoint {
            period: current_period_label.to_string(),
            value: current_value,
            is_current: true,
        });

        // ---- Variance explanation ------------------------------------------
        let variance_explanation = if !within_expected_range {
            Some(build_variance_explanation(
                name,
                current_value,
                lo,
                hi,
                unit,
                entity_code,
            ))
        } else {
            None
        };

        AnalyticalRelationship {
            id,
            entity_code: entity_code.to_string(),
            relationship_name: name.to_string(),
            account_area: account_area.to_string(),
            relationship_type: rel_type,
            formula: formula.to_string(),
            periods,
            expected_range: (format!("{:.2} {}", lo, unit), format!("{:.2} {}", hi, unit)),
            variance_explanation,
            supporting_metrics,
            reliability,
            within_expected_range,
        }
    }
}

// ---------------------------------------------------------------------------
// Financial computation helpers
// ---------------------------------------------------------------------------

/// Sum all credit-normal (or debit-normal) flows for accounts starting with
/// a given character prefix (first digit of account code).
fn sum_account_prefix(entries: &[&JournalEntry], prefix: char, credit_normal: bool) -> Decimal {
    let mut total = Decimal::ZERO;
    for je in entries {
        for line in &je.lines {
            if line.gl_account.starts_with(prefix) {
                if credit_normal {
                    total += line.credit_amount;
                } else {
                    total += line.debit_amount;
                }
            }
        }
    }
    total
}

/// Sum the flows for a single exact GL account.
fn sum_account_exact(entries: &[&JournalEntry], account: &str, credit_normal: bool) -> Decimal {
    let mut total = Decimal::ZERO;
    for je in entries {
        for line in &je.lines {
            if line.gl_account == account {
                if credit_normal {
                    total += line.credit_amount;
                } else {
                    total += line.debit_amount;
                }
            }
        }
    }
    total
}

/// Compute Days Sales Outstanding: AR / Revenue × 365.
/// Returns 0 if revenue is zero.
fn compute_dso(ar: Decimal, revenue: Decimal) -> Decimal {
    if revenue.is_zero() {
        return Decimal::ZERO;
    }
    clamp_positive((ar / revenue) * dec!(365))
}

/// Compute Days Payable Outstanding: AP / COGS × 365.
fn compute_dpo(ap: Decimal, cogs: Decimal) -> Decimal {
    if cogs.is_zero() {
        return Decimal::ZERO;
    }
    clamp_positive((ap / cogs) * dec!(365))
}

/// Compute Inventory Turnover: COGS / Inventory.
fn compute_inventory_turnover(cogs: Decimal, inventory: Decimal) -> Decimal {
    if inventory.is_zero() {
        return Decimal::ZERO;
    }
    clamp_positive(cogs / inventory)
}

/// Compute Gross Margin: (Revenue − COGS) / Revenue × 100.
fn compute_gross_margin(revenue: Decimal, cogs: Decimal) -> Decimal {
    if revenue.is_zero() {
        return Decimal::ZERO;
    }
    ((revenue - cogs) / revenue) * dec!(100)
}

/// Compute a simple ratio: numerator / denominator × multiplier.
fn compute_ratio(numerator: Decimal, denominator: Decimal, multiplier: Decimal) -> Decimal {
    if denominator.is_zero() {
        return Decimal::ZERO;
    }
    (numerator / denominator) * multiplier
}

/// Compute period-on-period growth: (current / prior − 1) × 100.
fn compute_growth(current: Decimal, prior: Decimal) -> Decimal {
    if prior.is_zero() {
        return Decimal::ZERO;
    }
    ((current / prior) - dec!(1)) * dec!(100)
}

/// Clamp a decimal to a non-negative value.
fn clamp_positive(value: Decimal) -> Decimal {
    if value < Decimal::ZERO {
        Decimal::ZERO
    } else {
        value
    }
}

/// Build an explanatory narrative for a value outside the expected range.
fn build_variance_explanation(
    name: &str,
    value: Decimal,
    lo: Decimal,
    hi: Decimal,
    unit: &str,
    entity_code: &str,
) -> String {
    if value < lo {
        format!(
            "{}: current value of {:.2} {} for entity {} is below the expected minimum of {:.2} {}. \
             This may indicate changes in business mix, pricing policy, or classification differences \
             that require further investigation.",
            name, value, unit, entity_code, lo, unit
        )
    } else {
        format!(
            "{}: current value of {:.2} {} for entity {} exceeds the expected maximum of {:.2} {}. \
             This may reflect changes in payment terms, collection activity, inventory policy, \
             or a concentration of period-end transactions requiring analytical review.",
            name, value, unit, entity_code, hi, unit
        )
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::journal_entry::{
        JournalEntry, JournalEntryHeader, JournalEntryLine,
    };
    use rust_decimal_macros::dec;

    /// Create a minimal journal entry for a given account.
    fn make_entry(
        company_code: &str,
        gl_account_debit: &str,
        gl_account_credit: &str,
        amount: Decimal,
    ) -> JournalEntry {
        let posting_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let header = JournalEntryHeader::new(company_code.to_string(), posting_date);
        let doc_id = header.document_id;
        let lines = vec![
            JournalEntryLine::debit(doc_id, 1, gl_account_debit.to_string(), amount),
            JournalEntryLine::credit(doc_id, 2, gl_account_credit.to_string(), amount),
        ];
        JournalEntry {
            header,
            lines: lines.into(),
        }
    }

    fn build_test_entries() -> Vec<JournalEntry> {
        vec![
            // Revenue entries (4000 credit)
            make_entry("C001", "1100", "4000", dec!(100_000)),
            make_entry("C001", "1100", "4000", dec!(80_000)),
            // COGS entries (5000 debit)
            make_entry("C001", "5000", "1200", dec!(60_000)),
            make_entry("C001", "5000", "1200", dec!(40_000)),
            // OpEx (6100 salaries debit)
            make_entry("C001", "6100", "2210", dec!(25_000)),
            // AR balance (1100 debit from another source)
            make_entry("C001", "1100", "4100", dec!(15_000)),
            // AP balance (2000 credit)
            make_entry("C001", "5000", "2000", dec!(20_000)),
            // Inventory (1200 debit)
            make_entry("C001", "1200", "2000", dec!(30_000)),
            // Fixed assets (1500 debit)
            make_entry("C001", "1500", "2600", dec!(50_000)),
            // Depreciation (6000 debit)
            make_entry("C001", "6000", "1510", dec!(5_000)),
        ]
    }

    #[test]
    fn test_generates_at_least_8_relationships_per_entity() {
        let entries = build_test_entries();
        let mut gen = AnalyticalRelationshipGenerator::new(42);
        let rels = gen.generate_for_entity("C001", &entries, "FY2024", "FY2023");
        assert!(
            rels.len() >= 8,
            "Expected ≥8 relationships, got {}",
            rels.len()
        );
    }

    #[test]
    fn test_current_period_is_marked() {
        let entries = build_test_entries();
        let mut gen = AnalyticalRelationshipGenerator::new(42);
        let rels = gen.generate_for_entity("C001", &entries, "FY2024", "FY2023");
        for rel in &rels {
            let current = rel.current_period();
            assert!(
                current.is_some(),
                "Relationship '{}' has no current period",
                rel.relationship_name
            );
            assert!(current.unwrap().is_current);
        }
    }

    #[test]
    fn test_historical_periods_not_current() {
        let entries = build_test_entries();
        let mut gen = AnalyticalRelationshipGenerator::new(42);
        let rels = gen.generate_for_entity("C001", &entries, "FY2024", "FY2023");
        for rel in &rels {
            let historical: Vec<_> = rel.periods.iter().filter(|p| !p.is_current).collect();
            assert!(
                !historical.is_empty(),
                "Expected historical periods for '{}'",
                rel.relationship_name
            );
        }
    }

    #[test]
    fn test_dso_in_expected_range_with_normal_data() {
        // AR = 20_000, Revenue = 180_000 → DSO = 40.6 days (within 30–60)
        let entries = vec![
            make_entry("C001", "1100", "4000", dec!(180_000)),
            make_entry("C001", "5000", "1100", dec!(160_000)), // reduces AR
        ];
        let dso = compute_dso(dec!(20_000), dec!(180_000));
        // ~40.6 days — within 30–60
        assert!(dso >= dec!(30) && dso <= dec!(60), "DSO = {}", dso);
        drop(entries);
    }

    #[test]
    fn test_variance_explanation_populated_when_out_of_range() {
        // Force inventory turnover way out of range (> 12)
        // COGS = 200_000, Inventory = 5_000 → turnover = 40×
        let entries = vec![
            make_entry("C001", "5000", "1200", dec!(200_000)),
            make_entry("C001", "1200", "2000", dec!(5_000)),
            make_entry("C001", "1100", "4000", dec!(300_000)), // revenue so other ratios work
        ];
        let mut gen = AnalyticalRelationshipGenerator::new(99);
        let rels = gen.generate_for_entity("C001", &entries, "FY2024", "FY2023");
        let inv_rel = rels
            .iter()
            .find(|r| r.relationship_name.contains("Inventory Turnover"))
            .unwrap();
        // If out of range, variance_explanation should be set
        if !inv_rel.within_expected_range {
            assert!(
                inv_rel.variance_explanation.is_some(),
                "variance_explanation should be set when out of range"
            );
        }
    }

    #[test]
    fn test_unique_ids_across_entities() {
        let entries = build_test_entries();
        // Build entries for C002 too
        let mut entries2 = build_test_entries();
        for e in &mut entries2 {
            e.header.company_code = "C002".to_string();
        }
        let all: Vec<JournalEntry> = entries.into_iter().chain(entries2).collect();

        let mut gen = AnalyticalRelationshipGenerator::new(42);
        let rels = gen.generate_for_entities(
            &["C001".to_string(), "C002".to_string()],
            &all,
            "FY2024",
            "FY2023",
        );

        let ids: std::collections::HashSet<&str> = rels.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids.len(), rels.len(), "Relationship IDs must be unique");
    }

    #[test]
    fn test_serialisation_roundtrip() {
        let entries = build_test_entries();
        let mut gen = AnalyticalRelationshipGenerator::new(42);
        let rels = gen.generate_for_entity("C001", &entries, "FY2024", "FY2023");
        let json = serde_json::to_string(&rels).unwrap();
        let decoded: Vec<AnalyticalRelationship> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.len(), rels.len());
    }
}
