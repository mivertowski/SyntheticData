//! Cash Position Generator.
//!
//! Aggregates cash inflows and outflows into daily [`CashPosition`] records
//! per bank account. Runs after AP/AR/payroll/banking generators have produced
//! payment records.

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_config::schema::CashPositioningConfig;
use datasynth_core::models::CashPosition;

// ---------------------------------------------------------------------------
// Cash flow input abstraction
// ---------------------------------------------------------------------------

/// Direction of a cash flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CashFlowDirection {
    /// Money coming in (AR collection, refund, etc.)
    Inflow,
    /// Money going out (AP payment, payroll, tax, etc.)
    Outflow,
}

/// An individual cash flow event to be aggregated into positions.
///
/// This is a simplified representation that the cash position generator uses
/// to aggregate from various sources (AP payments, AR receipts, payroll, tax).
#[derive(Debug, Clone)]
pub struct CashFlow {
    /// Date the cash flow occurs
    pub date: NaiveDate,
    /// Bank account the flow affects
    pub account_id: String,
    /// Absolute amount of the flow
    pub amount: Decimal,
    /// Whether this is an inflow or outflow
    pub direction: CashFlowDirection,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates daily cash positions by aggregating cash flows per bank account.
pub struct CashPositionGenerator {
    rng: ChaCha8Rng,
    config: CashPositioningConfig,
    id_counter: u64,
}

impl CashPositionGenerator {
    /// Creates a new cash position generator.
    pub fn new(seed: u64, config: CashPositioningConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            id_counter: 0,
        }
    }

    /// Generates daily cash positions for a single bank account over a date range.
    ///
    /// # Arguments
    /// * `entity_id` — Legal entity identifier
    /// * `account_id` — Bank account identifier
    /// * `currency` — Account currency
    /// * `flows` — Cash flows for this account (filtered by caller)
    /// * `start_date` — First day of the position series
    /// * `end_date` — Last day (inclusive) of the position series
    /// * `opening_balance` — Opening balance on the start date
    pub fn generate(
        &mut self,
        entity_id: &str,
        account_id: &str,
        currency: &str,
        flows: &[CashFlow],
        start_date: NaiveDate,
        end_date: NaiveDate,
        opening_balance: Decimal,
    ) -> Vec<CashPosition> {
        let mut positions = Vec::new();
        let mut current_date = start_date;
        let mut running_balance = opening_balance;

        while current_date <= end_date {
            // Aggregate flows for this date
            let mut inflows = Decimal::ZERO;
            let mut outflows = Decimal::ZERO;

            for flow in flows {
                if flow.date == current_date {
                    match flow.direction {
                        CashFlowDirection::Inflow => inflows += flow.amount,
                        CashFlowDirection::Outflow => outflows += flow.amount,
                    }
                }
            }

            self.id_counter += 1;
            let id = format!("CP-{:06}", self.id_counter);

            let mut pos = CashPosition::new(
                id,
                entity_id,
                account_id,
                currency,
                current_date,
                running_balance,
                inflows,
                outflows,
            );

            // Simulate available balance as slightly less than closing
            // (pending transactions reduce available balance)
            let closing = pos.closing_balance;
            let pending_hold = self.random_hold_amount(closing);
            pos = pos.with_available_balance(
                (closing - pending_hold).max(Decimal::ZERO),
            );

            running_balance = pos.closing_balance;
            positions.push(pos);

            current_date = current_date
                .succ_opt()
                .unwrap_or(current_date);
        }

        positions
    }

    /// Generates positions for multiple bank accounts in a single entity.
    pub fn generate_multi_account(
        &mut self,
        entity_id: &str,
        accounts: &[(String, String, Decimal)], // (account_id, currency, opening_balance)
        flows: &[CashFlow],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<CashPosition> {
        let mut all_positions = Vec::new();

        for (account_id, currency, opening_balance) in accounts {
            let account_flows: Vec<CashFlow> = flows
                .iter()
                .filter(|f| f.account_id == *account_id)
                .cloned()
                .collect();

            let positions = self.generate(
                entity_id,
                account_id,
                currency,
                &account_flows,
                start_date,
                end_date,
                *opening_balance,
            );

            all_positions.extend(positions);
        }

        all_positions
    }

    /// Returns the minimum balance policy from config.
    pub fn minimum_balance_policy(&self) -> Decimal {
        Decimal::try_from(self.config.minimum_balance_policy).unwrap_or(dec!(100000))
    }

    /// Generates a small random hold amount (0-2% of balance) to differentiate
    /// available from closing balance.
    fn random_hold_amount(&mut self, closing_balance: Decimal) -> Decimal {
        if closing_balance <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        let pct = self.rng.gen_range(0.0f64..0.02);
        let hold = closing_balance * Decimal::try_from(pct).unwrap_or(Decimal::ZERO);
        hold.round_dp(2)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_cash_positions_from_payment_flows() {
        let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
        let flows = vec![
            CashFlow {
                date: d("2025-01-15"),
                account_id: "BA-001".into(),
                amount: dec!(5000),
                direction: CashFlowDirection::Inflow,
            },
            CashFlow {
                date: d("2025-01-15"),
                account_id: "BA-001".into(),
                amount: dec!(2000),
                direction: CashFlowDirection::Outflow,
            },
            CashFlow {
                date: d("2025-01-16"),
                account_id: "BA-001".into(),
                amount: dec!(1000),
                direction: CashFlowDirection::Outflow,
            },
        ];
        let positions = gen.generate(
            "C001",
            "BA-001",
            "USD",
            &flows,
            d("2025-01-15"),
            d("2025-01-16"),
            dec!(10000),
        );

        assert_eq!(positions.len(), 2);
        // Day 1: opening=10000, in=5000, out=2000, closing=13000
        assert_eq!(positions[0].opening_balance, dec!(10000));
        assert_eq!(positions[0].inflows, dec!(5000));
        assert_eq!(positions[0].outflows, dec!(2000));
        assert_eq!(positions[0].closing_balance, dec!(13000));
        // Day 2: opening=13000, in=0, out=1000, closing=12000
        assert_eq!(positions[1].opening_balance, dec!(13000));
        assert_eq!(positions[1].inflows, dec!(0));
        assert_eq!(positions[1].outflows, dec!(1000));
        assert_eq!(positions[1].closing_balance, dec!(12000));
    }

    #[test]
    fn test_no_flows_produces_flat_positions() {
        let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
        let positions = gen.generate(
            "C001",
            "BA-001",
            "EUR",
            &[],
            d("2025-01-01"),
            d("2025-01-03"),
            dec!(50000),
        );

        assert_eq!(positions.len(), 3);
        for pos in &positions {
            assert_eq!(pos.opening_balance, dec!(50000));
            assert_eq!(pos.inflows, dec!(0));
            assert_eq!(pos.outflows, dec!(0));
            assert_eq!(pos.closing_balance, dec!(50000));
        }
    }

    #[test]
    fn test_balance_carries_forward() {
        let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
        let flows = vec![
            CashFlow {
                date: d("2025-01-01"),
                account_id: "BA-001".into(),
                amount: dec!(10000),
                direction: CashFlowDirection::Inflow,
            },
            CashFlow {
                date: d("2025-01-02"),
                account_id: "BA-001".into(),
                amount: dec!(3000),
                direction: CashFlowDirection::Outflow,
            },
            CashFlow {
                date: d("2025-01-03"),
                account_id: "BA-001".into(),
                amount: dec!(5000),
                direction: CashFlowDirection::Inflow,
            },
        ];

        let positions = gen.generate(
            "C001",
            "BA-001",
            "USD",
            &flows,
            d("2025-01-01"),
            d("2025-01-03"),
            dec!(20000),
        );

        assert_eq!(positions.len(), 3);
        // Day 1: 20000 + 10000 = 30000
        assert_eq!(positions[0].closing_balance, dec!(30000));
        // Day 2: 30000 - 3000 = 27000
        assert_eq!(positions[1].opening_balance, dec!(30000));
        assert_eq!(positions[1].closing_balance, dec!(27000));
        // Day 3: 27000 + 5000 = 32000
        assert_eq!(positions[2].opening_balance, dec!(27000));
        assert_eq!(positions[2].closing_balance, dec!(32000));
    }

    #[test]
    fn test_available_balance_less_than_or_equal_to_closing() {
        let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
        let positions = gen.generate(
            "C001",
            "BA-001",
            "USD",
            &[],
            d("2025-01-01"),
            d("2025-01-05"),
            dec!(100000),
        );

        for pos in &positions {
            assert!(
                pos.available_balance <= pos.closing_balance,
                "available {} should be <= closing {}",
                pos.available_balance,
                pos.closing_balance
            );
        }
    }

    #[test]
    fn test_multi_account_generation() {
        let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
        let accounts = vec![
            ("BA-001".to_string(), "USD".to_string(), dec!(10000)),
            ("BA-002".to_string(), "EUR".to_string(), dec!(20000)),
        ];
        let flows = vec![
            CashFlow {
                date: d("2025-01-01"),
                account_id: "BA-001".into(),
                amount: dec!(5000),
                direction: CashFlowDirection::Inflow,
            },
            CashFlow {
                date: d("2025-01-01"),
                account_id: "BA-002".into(),
                amount: dec!(3000),
                direction: CashFlowDirection::Outflow,
            },
        ];

        let positions = gen.generate_multi_account(
            "C001",
            &accounts,
            &flows,
            d("2025-01-01"),
            d("2025-01-02"),
        );

        // 2 accounts * 2 days = 4 positions
        assert_eq!(positions.len(), 4);

        // BA-001 day 1: 10000 + 5000 = 15000
        let ba001_day1 = positions.iter().find(|p| {
            p.bank_account_id == "BA-001" && p.date == d("2025-01-01")
        }).unwrap();
        assert_eq!(ba001_day1.closing_balance, dec!(15000));

        // BA-002 day 1: 20000 - 3000 = 17000
        let ba002_day1 = positions.iter().find(|p| {
            p.bank_account_id == "BA-002" && p.date == d("2025-01-01")
        }).unwrap();
        assert_eq!(ba002_day1.closing_balance, dec!(17000));
    }

    #[test]
    fn test_minimum_balance_policy() {
        let config = CashPositioningConfig {
            minimum_balance_policy: 250_000.0,
            ..CashPositioningConfig::default()
        };
        let gen = CashPositionGenerator::new(42, config);
        assert_eq!(gen.minimum_balance_policy(), dec!(250000));
    }

    #[test]
    fn test_deterministic_generation() {
        let flows = vec![CashFlow {
            date: d("2025-01-01"),
            account_id: "BA-001".into(),
            amount: dec!(5000),
            direction: CashFlowDirection::Inflow,
        }];

        let mut gen1 = CashPositionGenerator::new(42, CashPositioningConfig::default());
        let pos1 = gen1.generate("C001", "BA-001", "USD", &flows, d("2025-01-01"), d("2025-01-01"), dec!(10000));

        let mut gen2 = CashPositionGenerator::new(42, CashPositioningConfig::default());
        let pos2 = gen2.generate("C001", "BA-001", "USD", &flows, d("2025-01-01"), d("2025-01-01"), dec!(10000));

        assert_eq!(pos1[0].closing_balance, pos2[0].closing_balance);
        assert_eq!(pos1[0].available_balance, pos2[0].available_balance);
    }
}
