//! Intercompany Netting Run Generator.
//!
//! Creates periodic netting runs from intercompany transaction amounts,
//! computing per-entity gross receivable/payable positions and net settlements.

use std::collections::HashMap;

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

use datasynth_config::schema::NettingSchemaConfig;
use datasynth_core::models::{NettingCycle, NettingPosition, NettingRun, PayOrReceive};

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates intercompany netting runs from IC transaction amounts.
pub struct NettingRunGenerator {
    #[allow(dead_code)]
    rng: ChaCha8Rng,
    config: NettingSchemaConfig,
    counter: u64,
}

impl NettingRunGenerator {
    /// Creates a new netting run generator.
    pub fn new(config: NettingSchemaConfig, seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            counter: 0,
        }
    }

    /// Generates netting runs from intercompany matched-pair amounts.
    ///
    /// `ic_amounts` contains `(seller_entity, buyer_entity, amount)` tuples.
    /// Transactions are grouped into monthly (or configured cycle) netting runs.
    /// Each run contains per-entity positions with gross receivables, gross
    /// payables, and the resulting net position / settlement direction.
    pub fn generate(
        &mut self,
        entity_ids: &[String],
        currency: &str,
        start_date: NaiveDate,
        period_months: u32,
        ic_amounts: &[(String, String, Decimal)],
    ) -> Vec<NettingRun> {
        if entity_ids.len() < 2 || ic_amounts.is_empty() {
            return Vec::new();
        }

        let cycle = self.parse_cycle();
        let mut runs = Vec::new();

        // Generate one netting run per period according to cycle
        let period_count = match cycle {
            NettingCycle::Daily => period_months * 30, // approximate
            NettingCycle::Weekly => period_months * 4,
            NettingCycle::Monthly => period_months,
        };

        // Spread IC amounts across periods roughly evenly
        let amounts_per_period = if period_count > 0 {
            ic_amounts.len() / period_count as usize
        } else {
            ic_amounts.len()
        }
        .max(1);

        let mut amount_idx = 0;

        for period in 0..period_count {
            if amount_idx >= ic_amounts.len() {
                break;
            }

            // Compute netting date
            let netting_date = match cycle {
                NettingCycle::Daily => start_date + chrono::Duration::days(period as i64),
                NettingCycle::Weekly => start_date + chrono::Duration::weeks(period as i64),
                NettingCycle::Monthly => {
                    // Last day of the month
                    add_months_end(start_date, period)
                }
            };

            // Collect the subset of IC amounts for this period
            let end_idx = (amount_idx + amounts_per_period).min(ic_amounts.len());
            let period_amounts = &ic_amounts[amount_idx..end_idx];
            amount_idx = end_idx;

            if period_amounts.is_empty() {
                continue;
            }

            // Build positions: accumulate per-entity receivable/payable
            let mut receivables: HashMap<&str, Decimal> = HashMap::new();
            let mut payables: HashMap<&str, Decimal> = HashMap::new();

            for (seller, buyer, amount) in period_amounts {
                // Seller is owed money (receivable), buyer owes money (payable)
                *receivables.entry(seller.as_str()).or_insert(Decimal::ZERO) += amount;
                *payables.entry(buyer.as_str()).or_insert(Decimal::ZERO) += amount;
            }

            // Build NettingPosition for each participating entity
            let mut all_entities: Vec<&str> = receivables
                .keys()
                .chain(payables.keys())
                .copied()
                .collect();
            all_entities.sort();
            all_entities.dedup();

            let positions: Vec<NettingPosition> = all_entities
                .into_iter()
                .map(|eid| {
                    let gross_receivable =
                        receivables.get(eid).copied().unwrap_or(Decimal::ZERO).round_dp(2);
                    let gross_payable =
                        payables.get(eid).copied().unwrap_or(Decimal::ZERO).round_dp(2);
                    let net_position = (gross_receivable - gross_payable).round_dp(2);
                    let settlement_direction = if net_position > Decimal::ZERO {
                        PayOrReceive::Receive
                    } else if net_position < Decimal::ZERO {
                        PayOrReceive::Pay
                    } else {
                        PayOrReceive::Flat
                    };
                    NettingPosition {
                        entity_id: eid.to_string(),
                        gross_receivable,
                        gross_payable,
                        net_position,
                        settlement_direction,
                    }
                })
                .collect();

            if positions.is_empty() {
                continue;
            }

            self.counter += 1;
            let id = format!("NR-{:06}", self.counter);

            let run = NettingRun::new(id, netting_date, cycle, currency, positions);
            runs.push(run);
        }

        runs
    }

    fn parse_cycle(&self) -> NettingCycle {
        match self.config.cycle.as_str() {
            "daily" => NettingCycle::Daily,
            "weekly" => NettingCycle::Weekly,
            _ => NettingCycle::Monthly,
        }
    }
}

/// Adds months to a date, returning the last day of the target month.
fn add_months_end(date: NaiveDate, months: u32) -> NaiveDate {
    use chrono::Datelike;
    let total_months = date.month0() + months;
    let year = date.year() + (total_months / 12) as i32;
    let month = (total_months % 12) + 1;
    let last_day = days_in_month(year, month);
    NaiveDate::from_ymd_opt(year, month, last_day).unwrap_or(date)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_basic_generation() {
        let config = NettingSchemaConfig {
            enabled: true,
            cycle: "monthly".to_string(),
        };
        let mut gen = NettingRunGenerator::new(config, 42);

        let entity_ids = vec!["C001".to_string(), "C002".to_string(), "C003".to_string()];
        let ic_amounts = vec![
            ("C001".to_string(), "C002".to_string(), dec!(100000)),
            ("C002".to_string(), "C003".to_string(), dec!(50000)),
            ("C003".to_string(), "C001".to_string(), dec!(30000)),
        ];

        let runs = gen.generate(&entity_ids, "USD", d("2025-01-01"), 3, &ic_amounts);

        assert!(!runs.is_empty());
        for run in &runs {
            assert!(run.id.starts_with("NR-"));
            assert_eq!(run.settlement_currency, "USD");
            assert_eq!(run.cycle, NettingCycle::Monthly);
            assert!(!run.positions.is_empty());
            // Gross receivables should equal gross payables (closed system)
            assert_eq!(run.gross_receivables, run.gross_payables);
        }
    }

    #[test]
    fn test_deterministic() {
        let config = NettingSchemaConfig {
            enabled: true,
            cycle: "monthly".to_string(),
        };
        let entity_ids = vec!["C001".to_string(), "C002".to_string()];
        let ic_amounts = vec![
            ("C001".to_string(), "C002".to_string(), dec!(100000)),
            ("C002".to_string(), "C001".to_string(), dec!(60000)),
        ];

        let mut gen1 = NettingRunGenerator::new(config.clone(), 42);
        let r1 = gen1.generate(&entity_ids, "USD", d("2025-01-01"), 2, &ic_amounts);

        let mut gen2 = NettingRunGenerator::new(config, 42);
        let r2 = gen2.generate(&entity_ids, "USD", d("2025-01-01"), 2, &ic_amounts);

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.gross_receivables, b.gross_receivables);
            assert_eq!(a.net_settlement, b.net_settlement);
        }
    }

    #[test]
    fn test_empty_input() {
        let config = NettingSchemaConfig {
            enabled: true,
            cycle: "monthly".to_string(),
        };
        let mut gen = NettingRunGenerator::new(config, 42);

        // Empty IC amounts
        let entity_ids = vec!["C001".to_string(), "C002".to_string()];
        let runs = gen.generate(&entity_ids, "USD", d("2025-01-01"), 3, &[]);
        assert!(runs.is_empty());

        // Single entity (needs 2+)
        let single = vec!["C001".to_string()];
        let ic_amounts = vec![("C001".to_string(), "C001".to_string(), dec!(100000))];
        let runs = gen.generate(&single, "USD", d("2025-01-01"), 3, &ic_amounts);
        assert!(runs.is_empty());
    }

    #[test]
    fn test_net_positions_balance() {
        let config = NettingSchemaConfig {
            enabled: true,
            cycle: "monthly".to_string(),
        };
        let mut gen = NettingRunGenerator::new(config, 42);

        let entity_ids = vec!["C001".to_string(), "C002".to_string()];
        let ic_amounts = vec![
            ("C001".to_string(), "C002".to_string(), dec!(100000)),
            ("C002".to_string(), "C001".to_string(), dec!(40000)),
        ];

        let runs = gen.generate(&entity_ids, "USD", d("2025-01-01"), 1, &ic_amounts);
        assert_eq!(runs.len(), 1);

        let run = &runs[0];
        // Net positions should sum to zero (closed system)
        let net_sum: Decimal = run.positions.iter().map(|p| p.net_position).sum();
        assert_eq!(net_sum, Decimal::ZERO);

        // Savings should be positive (netting reduces payment flows)
        assert!(run.savings() >= Decimal::ZERO);
    }
}
