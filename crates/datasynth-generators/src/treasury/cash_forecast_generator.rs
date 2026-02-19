//! Cash Forecast Generator.
//!
//! Produces forward-looking [`CashForecast`] records from AR aging, AP aging,
//! payroll schedules, and tax deadlines. Each forecast item is probability-weighted
//! based on its source: scheduled AP payments are near-certain while overdue AR
//! collections receive lower probability.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_config::schema::CashForecastingConfig;
use datasynth_core::models::{CashForecast, CashForecastItem, TreasuryCashFlowCategory};

// ---------------------------------------------------------------------------
// Input abstractions
// ---------------------------------------------------------------------------

/// A receivable item from AR aging for forecast input.
#[derive(Debug, Clone)]
pub struct ArAgingItem {
    /// Expected collection date
    pub expected_date: NaiveDate,
    /// Invoice amount
    pub amount: Decimal,
    /// Days past due (0 = current)
    pub days_past_due: u32,
    /// Source document ID
    pub document_id: String,
}

/// A payable item from AP aging for forecast input.
#[derive(Debug, Clone)]
pub struct ApAgingItem {
    /// Scheduled payment date
    pub payment_date: NaiveDate,
    /// Payment amount
    pub amount: Decimal,
    /// Source document ID
    pub document_id: String,
}

/// A scheduled disbursement (payroll, tax, debt service).
#[derive(Debug, Clone)]
pub struct ScheduledDisbursement {
    /// Scheduled date
    pub date: NaiveDate,
    /// Disbursement amount
    pub amount: Decimal,
    /// Category of the disbursement
    pub category: TreasuryCashFlowCategory,
    /// Description or source reference
    pub description: String,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates cash forecasts with probability-weighted items.
pub struct CashForecastGenerator {
    rng: ChaCha8Rng,
    config: CashForecastingConfig,
    id_counter: u64,
    item_counter: u64,
}

impl CashForecastGenerator {
    /// Creates a new cash forecast generator.
    pub fn new(config: CashForecastingConfig, seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            id_counter: 0,
            item_counter: 0,
        }
    }

    /// Generates a cash forecast from various input sources.
    pub fn generate(
        &mut self,
        entity_id: &str,
        currency: &str,
        forecast_date: NaiveDate,
        ar_items: &[ArAgingItem],
        ap_items: &[ApAgingItem],
        disbursements: &[ScheduledDisbursement],
    ) -> CashForecast {
        let horizon_end = forecast_date + chrono::Duration::days(self.config.horizon_days as i64);
        let mut items = Vec::new();

        // AR collections (inflows)
        for ar in ar_items {
            if ar.expected_date > forecast_date && ar.expected_date <= horizon_end {
                let prob = self.ar_collection_probability(ar.days_past_due);
                self.item_counter += 1;
                items.push(CashForecastItem {
                    id: format!("CFI-{:06}", self.item_counter),
                    date: ar.expected_date,
                    category: TreasuryCashFlowCategory::ArCollection,
                    amount: ar.amount,
                    probability: prob,
                    source_document_type: Some("CustomerInvoice".to_string()),
                    source_document_id: Some(ar.document_id.clone()),
                });
            }
        }

        // AP payments (outflows, negative amounts)
        for ap in ap_items {
            if ap.payment_date > forecast_date && ap.payment_date <= horizon_end {
                self.item_counter += 1;
                items.push(CashForecastItem {
                    id: format!("CFI-{:06}", self.item_counter),
                    date: ap.payment_date,
                    category: TreasuryCashFlowCategory::ApPayment,
                    amount: -ap.amount,
                    probability: dec!(0.95), // scheduled payments are near-certain
                    source_document_type: Some("VendorInvoice".to_string()),
                    source_document_id: Some(ap.document_id.clone()),
                });
            }
        }

        // Scheduled disbursements (outflows)
        for disb in disbursements {
            if disb.date > forecast_date && disb.date <= horizon_end {
                self.item_counter += 1;
                items.push(CashForecastItem {
                    id: format!("CFI-{:06}", self.item_counter),
                    date: disb.date,
                    category: disb.category,
                    amount: -disb.amount,
                    probability: dec!(1.00), // scheduled = certain
                    source_document_type: None,
                    source_document_id: None,
                });
            }
        }

        self.id_counter += 1;
        let confidence = Decimal::try_from(self.config.confidence_interval).unwrap_or(dec!(0.90));

        CashForecast::new(
            format!("CF-{:06}", self.id_counter),
            entity_id,
            currency,
            forecast_date,
            self.config.horizon_days,
            items,
            confidence,
        )
    }

    /// Computes AR collection probability based on aging.
    ///
    /// Overdue invoices get progressively lower probability:
    /// - Current (0 days): 95%
    /// - 1-30 days past due: 85%
    /// - 31-60 days: 65%
    /// - 61-90 days: 40%
    /// - 90+ days: 15%
    fn ar_collection_probability(&mut self, days_past_due: u32) -> Decimal {
        let base = match days_past_due {
            0 => dec!(0.95),
            1..=30 => dec!(0.85),
            31..=60 => dec!(0.65),
            61..=90 => dec!(0.40),
            _ => dec!(0.15),
        };
        // Add small random jitter (±5%)
        let jitter =
            Decimal::try_from(self.rng.gen_range(-0.05f64..0.05f64)).unwrap_or(Decimal::ZERO);
        (base + jitter).max(dec!(0.05)).min(dec!(1.00)).round_dp(2)
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
    fn test_forecast_from_ar_ap() {
        let mut gen = CashForecastGenerator::new(CashForecastingConfig::default(), 42);
        let ar = vec![ArAgingItem {
            expected_date: d("2025-02-15"),
            amount: dec!(50000),
            days_past_due: 0,
            document_id: "INV-001".to_string(),
        }];
        let ap = vec![ApAgingItem {
            payment_date: d("2025-02-10"),
            amount: dec!(30000),
            document_id: "VI-001".to_string(),
        }];
        let forecast = gen.generate("C001", "USD", d("2025-01-31"), &ar, &ap, &[]);

        assert_eq!(forecast.items.len(), 2);
        // AR item is positive
        let ar_item = forecast
            .items
            .iter()
            .find(|i| i.category == TreasuryCashFlowCategory::ArCollection)
            .unwrap();
        assert!(ar_item.amount > Decimal::ZERO);
        // AP item is negative
        let ap_item = forecast
            .items
            .iter()
            .find(|i| i.category == TreasuryCashFlowCategory::ApPayment)
            .unwrap();
        assert!(ap_item.amount < Decimal::ZERO);
    }

    #[test]
    fn test_overdue_ar_lower_probability() {
        let mut gen = CashForecastGenerator::new(CashForecastingConfig::default(), 42);
        let ar = vec![
            ArAgingItem {
                expected_date: d("2025-02-15"),
                amount: dec!(10000),
                days_past_due: 0,
                document_id: "INV-CURRENT".to_string(),
            },
            ArAgingItem {
                expected_date: d("2025-02-20"),
                amount: dec!(10000),
                days_past_due: 90,
                document_id: "INV-OVERDUE".to_string(),
            },
        ];
        let forecast = gen.generate("C001", "USD", d("2025-01-31"), &ar, &[], &[]);

        let current = forecast
            .items
            .iter()
            .find(|i| i.source_document_id.as_deref() == Some("INV-CURRENT"))
            .unwrap();
        let overdue = forecast
            .items
            .iter()
            .find(|i| i.source_document_id.as_deref() == Some("INV-OVERDUE"))
            .unwrap();
        assert!(
            current.probability > overdue.probability,
            "current prob {} should exceed overdue prob {}",
            current.probability,
            overdue.probability
        );
    }

    #[test]
    fn test_disbursements_included() {
        let mut gen = CashForecastGenerator::new(CashForecastingConfig::default(), 42);
        let disbursements = vec![
            ScheduledDisbursement {
                date: d("2025-02-28"),
                amount: dec!(100000),
                category: TreasuryCashFlowCategory::PayrollDisbursement,
                description: "February payroll".to_string(),
            },
            ScheduledDisbursement {
                date: d("2025-03-15"),
                amount: dec!(50000),
                category: TreasuryCashFlowCategory::TaxPayment,
                description: "Q4 VAT payment".to_string(),
            },
        ];
        let forecast = gen.generate("C001", "USD", d("2025-01-31"), &[], &[], &disbursements);

        assert_eq!(forecast.items.len(), 2);
        for item in &forecast.items {
            assert!(item.amount < Decimal::ZERO); // outflows are negative
            assert_eq!(item.probability, dec!(1.00)); // scheduled = certain
        }
    }

    #[test]
    fn test_items_outside_horizon_excluded() {
        let config = CashForecastingConfig {
            horizon_days: 30,
            ..CashForecastingConfig::default()
        };
        let mut gen = CashForecastGenerator::new(config, 42);
        let ar = vec![ArAgingItem {
            expected_date: d("2025-06-15"), // way beyond 30-day horizon
            amount: dec!(10000),
            days_past_due: 0,
            document_id: "INV-FAR".to_string(),
        }];
        let forecast = gen.generate("C001", "USD", d("2025-01-31"), &ar, &[], &[]);
        assert_eq!(forecast.items.len(), 0);
    }

    #[test]
    fn test_net_position_computed() {
        let mut gen = CashForecastGenerator::new(CashForecastingConfig::default(), 42);
        let ar = vec![ArAgingItem {
            expected_date: d("2025-02-15"),
            amount: dec!(100000),
            days_past_due: 0,
            document_id: "INV-001".to_string(),
        }];
        let ap = vec![ApAgingItem {
            payment_date: d("2025-02-10"),
            amount: dec!(60000),
            document_id: "VI-001".to_string(),
        }];
        let forecast = gen.generate("C001", "USD", d("2025-01-31"), &ar, &ap, &[]);

        // Net should be computed from probability-weighted amounts
        assert_eq!(forecast.net_position, forecast.computed_net_position());
        // Net should be positive (AR inflow > AP outflow after weighting)
        assert!(forecast.net_position > Decimal::ZERO);
    }
}
