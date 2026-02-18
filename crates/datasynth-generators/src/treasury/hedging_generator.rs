//! Hedging Instrument Generator.
//!
//! Creates FX forward and interest rate swap instruments to hedge FX exposures
//! from multi-currency AP/AR balances. Designates hedge accounting relationships
//! under ASC 815 / IFRS 9 and tests effectiveness (80-125% corridor).

use chrono::NaiveDate;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_config::schema::HedgingSchemaConfig;
use datasynth_core::models::{
    EffectivenessMethod, HedgeInstrumentType, HedgeRelationship, HedgeType, HedgedItemType,
    HedgingInstrument,
};

// ---------------------------------------------------------------------------
// Input abstraction
// ---------------------------------------------------------------------------

/// An FX exposure from outstanding AP/AR balances.
#[derive(Debug, Clone)]
pub struct FxExposure {
    /// Currency pair (e.g., "EUR/USD")
    pub currency_pair: String,
    /// Foreign currency code
    pub foreign_currency: String,
    /// Net exposure amount in foreign currency (positive = long, negative = short)
    pub net_amount: Decimal,
    /// Expected settlement date
    pub settlement_date: NaiveDate,
    /// Description of the exposure source
    pub description: String,
}

// ---------------------------------------------------------------------------
// Counterparty pool
// ---------------------------------------------------------------------------

const COUNTERPARTIES: &[&str] = &[
    "JPMorgan Chase",
    "Deutsche Bank",
    "Citibank",
    "HSBC",
    "Barclays",
    "BNP Paribas",
    "Goldman Sachs",
    "Morgan Stanley",
    "UBS",
    "Credit Suisse",
];

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates hedging instruments and hedge relationship designations.
pub struct HedgingGenerator {
    rng: ChaCha8Rng,
    config: HedgingSchemaConfig,
    instrument_counter: u64,
    relationship_counter: u64,
}

impl HedgingGenerator {
    /// Creates a new hedging generator.
    pub fn new(config: HedgingSchemaConfig, seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            instrument_counter: 0,
            relationship_counter: 0,
        }
    }

    /// Generates hedging instruments and designations from FX exposures.
    ///
    /// For each exposure, creates an FX forward covering `hedge_ratio` of the
    /// net exposure. If hedge accounting is enabled, also designates a
    /// [`HedgeRelationship`] with effectiveness testing.
    pub fn generate(
        &mut self,
        trade_date: NaiveDate,
        exposures: &[FxExposure],
    ) -> (Vec<HedgingInstrument>, Vec<HedgeRelationship>) {
        let mut instruments = Vec::new();
        let mut relationships = Vec::new();
        let hedge_ratio = Decimal::try_from(self.config.hedge_ratio).unwrap_or(dec!(0.75));

        for exposure in exposures {
            if exposure.net_amount.is_zero() {
                continue;
            }

            let notional = (exposure.net_amount.abs() * hedge_ratio).round_dp(2);
            let counterparty = self.random_counterparty();
            let forward_rate = self.generate_forward_rate();

            self.instrument_counter += 1;
            let instr_id = format!("HI-{:06}", self.instrument_counter);

            let instrument = HedgingInstrument::new(
                &instr_id,
                HedgeInstrumentType::FxForward,
                notional,
                &exposure.foreign_currency,
                trade_date,
                exposure.settlement_date,
                counterparty,
            )
            .with_currency_pair(&exposure.currency_pair)
            .with_fixed_rate(forward_rate)
            .with_fair_value(self.generate_fair_value(notional));

            instruments.push(instrument);

            // Designate hedge relationship if hedge accounting is enabled
            if self.config.hedge_accounting {
                let effectiveness = self.generate_effectiveness_ratio();
                self.relationship_counter += 1;
                let rel_id = format!("HR-{:06}", self.relationship_counter);

                let method = self.parse_effectiveness_method();

                let relationship = HedgeRelationship::new(
                    rel_id,
                    HedgedItemType::ForecastedTransaction,
                    &exposure.description,
                    &instr_id,
                    HedgeType::CashFlowHedge,
                    trade_date,
                    method,
                    effectiveness,
                )
                .with_ineffectiveness_amount(
                    self.generate_ineffectiveness(notional, effectiveness),
                );

                relationships.push(relationship);
            }
        }

        (instruments, relationships)
    }

    /// Generates an interest rate swap instrument.
    pub fn generate_ir_swap(
        &mut self,
        entity_currency: &str,
        notional: Decimal,
        trade_date: NaiveDate,
        maturity_date: NaiveDate,
    ) -> HedgingInstrument {
        let counterparty = self.random_counterparty();
        let fixed_rate = dec!(0.03)
            + Decimal::try_from(self.rng.gen_range(0.0f64..0.025)).unwrap_or(Decimal::ZERO);

        self.instrument_counter += 1;
        HedgingInstrument::new(
            format!("HI-{:06}", self.instrument_counter),
            HedgeInstrumentType::InterestRateSwap,
            notional,
            entity_currency,
            trade_date,
            maturity_date,
            counterparty,
        )
        .with_fixed_rate(fixed_rate.round_dp(4))
        .with_floating_index("SOFR")
        .with_fair_value(self.generate_fair_value(notional))
    }

    fn random_counterparty(&mut self) -> &'static str {
        let idx = self.rng.gen_range(0..COUNTERPARTIES.len());
        COUNTERPARTIES[idx]
    }

    fn generate_forward_rate(&mut self) -> Decimal {
        // Typical FX forward rate around 1.0-1.5 range
        let rate = self.rng.gen_range(0.85f64..1.50f64);
        Decimal::try_from(rate).unwrap_or(dec!(1.10)).round_dp(4)
    }

    fn generate_fair_value(&mut self, notional: Decimal) -> Decimal {
        // Fair value is typically a small fraction of notional (±2%)
        let pct = self.rng.gen_range(-0.02f64..0.02f64);
        (notional * Decimal::try_from(pct).unwrap_or(Decimal::ZERO)).round_dp(2)
    }

    fn generate_effectiveness_ratio(&mut self) -> Decimal {
        // Most hedges are effective (0.85-1.15), with occasional failures
        if self.rng.gen_bool(0.90) {
            // Effective: within 80-125% corridor
            let ratio = self.rng.gen_range(0.85f64..1.15f64);
            Decimal::try_from(ratio).unwrap_or(dec!(1.00)).round_dp(4)
        } else {
            // Ineffective: outside corridor
            if self.rng.gen_bool(0.5) {
                let ratio = self.rng.gen_range(0.60f64..0.79f64);
                Decimal::try_from(ratio).unwrap_or(dec!(0.75)).round_dp(4)
            } else {
                let ratio = self.rng.gen_range(1.26f64..1.50f64);
                Decimal::try_from(ratio).unwrap_or(dec!(1.30)).round_dp(4)
            }
        }
    }

    fn generate_ineffectiveness(&mut self, notional: Decimal, ratio: Decimal) -> Decimal {
        // Ineffectiveness = |1.0 - ratio| * notional * small factor
        let deviation = (dec!(1.00) - ratio).abs();
        (notional * deviation * dec!(0.1)).round_dp(2)
    }

    fn parse_effectiveness_method(&self) -> EffectivenessMethod {
        match self.config.effectiveness_method.as_str() {
            "dollar_offset" => EffectivenessMethod::DollarOffset,
            "critical_terms" => EffectivenessMethod::CriticalTerms,
            _ => EffectivenessMethod::Regression,
        }
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
    fn test_generates_fx_forwards_from_exposures() {
        let mut gen = HedgingGenerator::new(HedgingSchemaConfig::default(), 42);
        let exposures = vec![
            FxExposure {
                currency_pair: "EUR/USD".to_string(),
                foreign_currency: "EUR".to_string(),
                net_amount: dec!(1000000),
                settlement_date: d("2025-06-30"),
                description: "EUR receivables Q2".to_string(),
            },
            FxExposure {
                currency_pair: "GBP/USD".to_string(),
                foreign_currency: "GBP".to_string(),
                net_amount: dec!(-500000),
                settlement_date: d("2025-06-30"),
                description: "GBP payables Q2".to_string(),
            },
        ];

        let (instruments, relationships) = gen.generate(d("2025-01-15"), &exposures);

        assert_eq!(instruments.len(), 2);
        assert_eq!(relationships.len(), 2); // hedge accounting enabled by default

        // Notional should be hedge_ratio * exposure
        let hedge_ratio = dec!(0.75);
        assert_eq!(
            instruments[0].notional_amount,
            (dec!(1000000) * hedge_ratio).round_dp(2)
        );
        assert_eq!(
            instruments[1].notional_amount,
            (dec!(500000) * hedge_ratio).round_dp(2)
        );

        // All should be FX Forwards
        for instr in &instruments {
            assert_eq!(instr.instrument_type, HedgeInstrumentType::FxForward);
            assert!(instr.is_active());
            assert!(instr.fixed_rate.is_some());
        }
    }

    #[test]
    fn test_hedge_relationships_effectiveness() {
        let mut gen = HedgingGenerator::new(HedgingSchemaConfig::default(), 42);
        let exposures = vec![FxExposure {
            currency_pair: "EUR/USD".to_string(),
            foreign_currency: "EUR".to_string(),
            net_amount: dec!(1000000),
            settlement_date: d("2025-06-30"),
            description: "EUR receivables".to_string(),
        }];

        let (_, relationships) = gen.generate(d("2025-01-15"), &exposures);
        assert_eq!(relationships.len(), 1);
        let rel = &relationships[0];
        assert_eq!(rel.hedge_type, HedgeType::CashFlowHedge);
        assert_eq!(rel.hedged_item_type, HedgedItemType::ForecastedTransaction);
        // Effectiveness ratio should be set
        assert!(rel.effectiveness_ratio > Decimal::ZERO);
    }

    #[test]
    fn test_no_hedge_relationships_when_accounting_disabled() {
        let config = HedgingSchemaConfig {
            hedge_accounting: false,
            ..HedgingSchemaConfig::default()
        };
        let mut gen = HedgingGenerator::new(config, 42);
        let exposures = vec![FxExposure {
            currency_pair: "EUR/USD".to_string(),
            foreign_currency: "EUR".to_string(),
            net_amount: dec!(1000000),
            settlement_date: d("2025-06-30"),
            description: "EUR receivables".to_string(),
        }];

        let (instruments, relationships) = gen.generate(d("2025-01-15"), &exposures);
        assert_eq!(instruments.len(), 1);
        assert_eq!(relationships.len(), 0);
    }

    #[test]
    fn test_zero_exposure_skipped() {
        let mut gen = HedgingGenerator::new(HedgingSchemaConfig::default(), 42);
        let exposures = vec![FxExposure {
            currency_pair: "EUR/USD".to_string(),
            foreign_currency: "EUR".to_string(),
            net_amount: dec!(0),
            settlement_date: d("2025-06-30"),
            description: "Zero exposure".to_string(),
        }];

        let (instruments, _) = gen.generate(d("2025-01-15"), &exposures);
        assert_eq!(instruments.len(), 0);
    }

    #[test]
    fn test_ir_swap_generation() {
        let mut gen = HedgingGenerator::new(HedgingSchemaConfig::default(), 42);
        let swap = gen.generate_ir_swap("USD", dec!(5000000), d("2025-01-01"), d("2030-01-01"));

        assert_eq!(swap.instrument_type, HedgeInstrumentType::InterestRateSwap);
        assert_eq!(swap.notional_amount, dec!(5000000));
        assert!(swap.fixed_rate.is_some());
        assert_eq!(swap.floating_index, Some("SOFR".to_string()));
        assert!(swap.is_active());
    }

    #[test]
    fn test_deterministic_generation() {
        let exposures = vec![FxExposure {
            currency_pair: "EUR/USD".to_string(),
            foreign_currency: "EUR".to_string(),
            net_amount: dec!(1000000),
            settlement_date: d("2025-06-30"),
            description: "EUR receivables".to_string(),
        }];

        let mut gen1 = HedgingGenerator::new(HedgingSchemaConfig::default(), 42);
        let (i1, r1) = gen1.generate(d("2025-01-15"), &exposures);

        let mut gen2 = HedgingGenerator::new(HedgingSchemaConfig::default(), 42);
        let (i2, r2) = gen2.generate(d("2025-01-15"), &exposures);

        assert_eq!(i1[0].notional_amount, i2[0].notional_amount);
        assert_eq!(i1[0].fair_value, i2[0].fair_value);
        assert_eq!(r1[0].effectiveness_ratio, r2[0].effectiveness_ratio);
    }
}
