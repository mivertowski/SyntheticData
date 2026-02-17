//! Debt Instrument and Covenant Generator.
//!
//! Creates term loans, revolving credit facilities, and bonds with amortization
//! schedules and financial covenant monitoring. Generates [`AmortizationPayment`]
//! vectors that sum to principal and computes covenant compliance with headroom.

use chrono::{Datelike, NaiveDate};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_config::schema::{CovenantDef, DebtInstrumentDef, DebtSchemaConfig};
use datasynth_core::models::{
    AmortizationPayment, CovenantType, DebtCovenant, DebtInstrument, DebtType, Frequency,
    InterestRateType,
};

// ---------------------------------------------------------------------------
// Lender pool
// ---------------------------------------------------------------------------

const LENDERS: &[&str] = &[
    "First National Bank",
    "Wells Fargo",
    "JPMorgan Chase",
    "Bank of America",
    "Citibank",
    "HSBC",
    "Deutsche Bank",
    "Barclays",
    "BNP Paribas",
    "Goldman Sachs",
];

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

/// Generates debt instruments with amortization schedules and covenants.
pub struct DebtGenerator {
    rng: ChaCha8Rng,
    config: DebtSchemaConfig,
    instrument_counter: u64,
    covenant_counter: u64,
}

impl DebtGenerator {
    /// Creates a new debt generator.
    pub fn new(seed: u64, config: DebtSchemaConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            instrument_counter: 0,
            covenant_counter: 0,
        }
    }

    /// Generates all debt instruments from config definitions.
    pub fn generate(
        &mut self,
        entity_id: &str,
        currency: &str,
        origination_date: NaiveDate,
    ) -> Vec<DebtInstrument> {
        let defs: Vec<DebtInstrumentDef> = self.config.instruments.clone();
        let covenant_defs: Vec<CovenantDef> = self.config.covenants.clone();

        let mut instruments = Vec::new();
        for def in &defs {
            let instrument =
                self.generate_from_def(entity_id, currency, origination_date, def, &covenant_defs);
            instruments.push(instrument);
        }
        instruments
    }

    /// Generates a single debt instrument from a definition.
    fn generate_from_def(
        &mut self,
        entity_id: &str,
        currency: &str,
        origination_date: NaiveDate,
        def: &DebtInstrumentDef,
        covenant_defs: &[CovenantDef],
    ) -> DebtInstrument {
        self.instrument_counter += 1;
        let id = format!("DEBT-{:06}", self.instrument_counter);
        let lender = self.random_lender();
        let debt_type = self.parse_debt_type(&def.instrument_type);
        let principal =
            Decimal::try_from(def.principal.unwrap_or(5_000_000.0)).unwrap_or(dec!(5000000));
        let rate = Decimal::try_from(def.rate.unwrap_or(0.055))
            .unwrap_or(dec!(0.055))
            .round_dp(4);
        let maturity_months = def.maturity_months.unwrap_or(60);
        let maturity_date = add_months(origination_date, maturity_months);

        let rate_type = if matches!(debt_type, DebtType::RevolvingCredit) {
            InterestRateType::Variable
        } else {
            InterestRateType::Fixed
        };

        let facility_limit =
            Decimal::try_from(def.facility.unwrap_or(0.0)).unwrap_or(Decimal::ZERO);

        let mut instrument = DebtInstrument::new(
            id,
            entity_id,
            debt_type,
            lender,
            principal,
            currency,
            rate,
            rate_type,
            origination_date,
            maturity_date,
        );

        // Generate amortization schedule for term loans
        if matches!(debt_type, DebtType::TermLoan | DebtType::Bond) {
            let schedule =
                self.generate_amortization(principal, rate, origination_date, maturity_months);
            instrument = instrument.with_amortization_schedule(schedule);
        }

        // Set revolving credit specifics
        if matches!(debt_type, DebtType::RevolvingCredit) && facility_limit > Decimal::ZERO {
            let drawn = (facility_limit * dec!(0.40)).round_dp(2);
            instrument = instrument
                .with_facility_limit(facility_limit)
                .with_drawn_amount(drawn);
        }

        // Attach covenants
        let measurement_date = origination_date;
        for cdef in covenant_defs {
            let covenant = self.generate_covenant(cdef, measurement_date);
            instrument = instrument.with_covenant(covenant);
        }

        instrument
    }

    /// Generates a level-payment amortization schedule.
    ///
    /// Uses quarterly payments. Total principal payments sum to the original principal.
    fn generate_amortization(
        &mut self,
        principal: Decimal,
        annual_rate: Decimal,
        start_date: NaiveDate,
        maturity_months: u32,
    ) -> Vec<AmortizationPayment> {
        let num_payments = maturity_months / 3; // quarterly
        if num_payments == 0 {
            return Vec::new();
        }

        let quarterly_rate = (annual_rate / dec!(4)).round_dp(6);
        let principal_per_period = (principal / Decimal::from(num_payments)).round_dp(2);

        let mut schedule = Vec::new();
        let mut remaining = principal;

        for i in 0..num_payments {
            let payment_date = add_months(start_date, (i + 1) * 3);
            let interest = (remaining * quarterly_rate).round_dp(2);

            // Last payment gets the remainder to ensure exact sum
            let principal_payment = if i == num_payments - 1 {
                remaining
            } else {
                principal_per_period
            };

            remaining = (remaining - principal_payment).round_dp(2);

            schedule.push(AmortizationPayment {
                date: payment_date,
                principal_payment,
                interest_payment: interest,
                balance_after: remaining.max(Decimal::ZERO),
            });
        }

        schedule
    }

    /// Generates a covenant from a definition with simulated actual values.
    fn generate_covenant(
        &mut self,
        def: &CovenantDef,
        measurement_date: NaiveDate,
    ) -> DebtCovenant {
        self.covenant_counter += 1;
        let id = format!("COV-{:06}", self.covenant_counter);
        let covenant_type = self.parse_covenant_type(&def.covenant_type);
        let threshold = Decimal::try_from(def.threshold).unwrap_or(dec!(3.0));

        // Generate actual value: usually compliant (90%), occasionally breached (10%)
        let actual = if self.rng.gen_bool(0.90) {
            self.generate_compliant_value(covenant_type, threshold)
        } else {
            self.generate_breached_value(covenant_type, threshold)
        };

        DebtCovenant::new(
            id,
            covenant_type,
            threshold,
            Frequency::Quarterly,
            actual,
            measurement_date,
        )
    }

    /// Generates an actual value that is compliant with the covenant.
    fn generate_compliant_value(
        &mut self,
        covenant_type: CovenantType,
        threshold: Decimal,
    ) -> Decimal {
        match covenant_type {
            // Maximum covenants: actual < threshold
            CovenantType::DebtToEquity | CovenantType::DebtToEbitda => {
                let factor = self.rng.gen_range(0.50f64..0.90f64);
                (threshold * Decimal::try_from(factor).unwrap_or(dec!(0.70))).round_dp(2)
            }
            // Minimum covenants: actual > threshold
            _ => {
                let factor = self.rng.gen_range(1.10f64..2.00f64);
                (threshold * Decimal::try_from(factor).unwrap_or(dec!(1.50))).round_dp(2)
            }
        }
    }

    /// Generates an actual value that breaches the covenant.
    fn generate_breached_value(
        &mut self,
        covenant_type: CovenantType,
        threshold: Decimal,
    ) -> Decimal {
        match covenant_type {
            CovenantType::DebtToEquity | CovenantType::DebtToEbitda => {
                let factor = self.rng.gen_range(1.05f64..1.30f64);
                (threshold * Decimal::try_from(factor).unwrap_or(dec!(1.10))).round_dp(2)
            }
            _ => {
                let factor = self.rng.gen_range(0.70f64..0.95f64);
                (threshold * Decimal::try_from(factor).unwrap_or(dec!(0.85))).round_dp(2)
            }
        }
    }

    fn random_lender(&mut self) -> &'static str {
        let idx = self.rng.gen_range(0..LENDERS.len());
        LENDERS[idx]
    }

    fn parse_debt_type(&self, s: &str) -> DebtType {
        match s {
            "revolving_credit" => DebtType::RevolvingCredit,
            "bond" => DebtType::Bond,
            "commercial_paper" => DebtType::CommercialPaper,
            "bridge_loan" => DebtType::BridgeLoan,
            _ => DebtType::TermLoan,
        }
    }

    fn parse_covenant_type(&self, s: &str) -> CovenantType {
        match s {
            "debt_to_equity" => CovenantType::DebtToEquity,
            "interest_coverage" => CovenantType::InterestCoverage,
            "current_ratio" => CovenantType::CurrentRatio,
            "net_worth" => CovenantType::NetWorth,
            "fixed_charge_coverage" => CovenantType::FixedChargeCoverage,
            _ => CovenantType::DebtToEbitda,
        }
    }
}

/// Adds months to a date (clamping to end of month if needed).
fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    let total_months = date.month0() + months;
    let year = date.year() + (total_months / 12) as i32;
    let month = (total_months % 12) + 1;
    // Clamp day to last day of month
    let day = date.day().min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap_or(date)
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

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn test_amortization_sums_to_principal() {
        let config = DebtSchemaConfig {
            enabled: true,
            instruments: vec![DebtInstrumentDef {
                instrument_type: "term_loan".to_string(),
                principal: Some(5_000_000.0),
                rate: Some(0.055),
                maturity_months: Some(60),
                facility: None,
            }],
            covenants: Vec::new(),
        };
        let mut gen = DebtGenerator::new(42, config);
        let instruments = gen.generate("C001", "USD", d("2025-01-01"));

        assert_eq!(instruments.len(), 1);
        let debt = &instruments[0];
        assert_eq!(debt.instrument_type, DebtType::TermLoan);
        assert!(!debt.amortization_schedule.is_empty());

        // Total principal payments should equal original principal
        assert_eq!(debt.total_principal_payments(), dec!(5000000));

        // Last payment should have zero balance
        let last = debt.amortization_schedule.last().unwrap();
        assert_eq!(last.balance_after, Decimal::ZERO);
    }

    #[test]
    fn test_revolving_credit_facility() {
        let config = DebtSchemaConfig {
            enabled: true,
            instruments: vec![DebtInstrumentDef {
                instrument_type: "revolving_credit".to_string(),
                principal: None,
                rate: Some(0.045),
                maturity_months: Some(36),
                facility: Some(2_000_000.0),
            }],
            covenants: Vec::new(),
        };
        let mut gen = DebtGenerator::new(42, config);
        let instruments = gen.generate("C001", "USD", d("2025-01-01"));

        assert_eq!(instruments.len(), 1);
        let revolver = &instruments[0];
        assert_eq!(revolver.instrument_type, DebtType::RevolvingCredit);
        assert_eq!(revolver.rate_type, InterestRateType::Variable);
        assert_eq!(revolver.facility_limit, dec!(2000000));
        assert!(revolver.drawn_amount < revolver.facility_limit);
        assert!(revolver.available_capacity() > Decimal::ZERO);
        // No amortization on revolving credit
        assert!(revolver.amortization_schedule.is_empty());
    }

    #[test]
    fn test_covenant_generation() {
        let config = DebtSchemaConfig {
            enabled: true,
            instruments: vec![DebtInstrumentDef {
                instrument_type: "term_loan".to_string(),
                principal: Some(3_000_000.0),
                rate: Some(0.05),
                maturity_months: Some(48),
                facility: None,
            }],
            covenants: vec![
                CovenantDef {
                    covenant_type: "debt_to_ebitda".to_string(),
                    threshold: 3.5,
                },
                CovenantDef {
                    covenant_type: "interest_coverage".to_string(),
                    threshold: 3.0,
                },
            ],
        };
        let mut gen = DebtGenerator::new(42, config);
        let instruments = gen.generate("C001", "USD", d("2025-01-01"));

        let debt = &instruments[0];
        assert_eq!(debt.covenants.len(), 2);

        // Each covenant should have a threshold and headroom
        for cov in &debt.covenants {
            assert!(cov.threshold > Decimal::ZERO);
            // headroom is positive if compliant, negative if breached
            if cov.is_compliant {
                assert!(cov.headroom > Decimal::ZERO);
            } else {
                assert!(cov.headroom < Decimal::ZERO);
            }
        }
    }

    #[test]
    fn test_multiple_instruments() {
        let config = DebtSchemaConfig {
            enabled: true,
            instruments: vec![
                DebtInstrumentDef {
                    instrument_type: "term_loan".to_string(),
                    principal: Some(5_000_000.0),
                    rate: Some(0.055),
                    maturity_months: Some(60),
                    facility: None,
                },
                DebtInstrumentDef {
                    instrument_type: "revolving_credit".to_string(),
                    principal: None,
                    rate: Some(0.045),
                    maturity_months: Some(36),
                    facility: Some(2_000_000.0),
                },
            ],
            covenants: Vec::new(),
        };
        let mut gen = DebtGenerator::new(42, config);
        let instruments = gen.generate("C001", "USD", d("2025-01-01"));

        assert_eq!(instruments.len(), 2);
        assert_eq!(instruments[0].instrument_type, DebtType::TermLoan);
        assert_eq!(instruments[1].instrument_type, DebtType::RevolvingCredit);
    }

    #[test]
    fn test_add_months() {
        assert_eq!(add_months(d("2025-01-31"), 1), d("2025-02-28"));
        assert_eq!(add_months(d("2025-01-15"), 3), d("2025-04-15"));
        assert_eq!(add_months(d("2025-01-15"), 12), d("2026-01-15"));
        assert_eq!(add_months(d("2024-01-31"), 1), d("2024-02-29")); // leap year
    }

    #[test]
    fn test_empty_config_no_instruments() {
        let config = DebtSchemaConfig::default();
        let mut gen = DebtGenerator::new(42, config);
        let instruments = gen.generate("C001", "USD", d("2025-01-01"));
        assert!(instruments.is_empty());
    }
}
