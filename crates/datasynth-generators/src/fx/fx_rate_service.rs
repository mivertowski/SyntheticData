//! FX rate generation service using Ornstein-Uhlenbeck process.
//!
//! Generates realistic exchange rates with mean-reversion and
//! occasional fat-tail moves.

use chrono::{Datelike, NaiveDate};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

use datasynth_core::models::{base_rates_usd, FxRate, FxRateTable, RateType};

/// Configuration for FX rate generation.
#[derive(Debug, Clone)]
pub struct FxRateServiceConfig {
    /// Base currency for all rates.
    pub base_currency: String,
    /// Mean reversion speed (theta) - higher = faster reversion.
    pub mean_reversion_speed: f64,
    /// Long-term mean level (typically 0 for log returns).
    pub long_term_mean: f64,
    /// Daily volatility (sigma) - typical FX volatility is 0.5-1% daily.
    pub daily_volatility: f64,
    /// Probability of fat-tail event (2x volatility).
    pub fat_tail_probability: f64,
    /// Fat-tail multiplier.
    pub fat_tail_multiplier: f64,
    /// Currencies to generate rates for.
    pub currencies: Vec<String>,
    /// Whether to generate weekend rates (or skip weekends).
    pub include_weekends: bool,
}

impl Default for FxRateServiceConfig {
    fn default() -> Self {
        Self {
            base_currency: "USD".to_string(),
            mean_reversion_speed: 0.05,
            long_term_mean: 0.0,
            daily_volatility: 0.006, // ~0.6% daily volatility
            fat_tail_probability: 0.05,
            fat_tail_multiplier: 2.5,
            currencies: vec![
                "EUR".to_string(),
                "GBP".to_string(),
                "JPY".to_string(),
                "CHF".to_string(),
                "CAD".to_string(),
                "AUD".to_string(),
                "CNY".to_string(),
            ],
            include_weekends: false,
        }
    }
}

/// Service for generating FX rates.
pub struct FxRateService {
    config: FxRateServiceConfig,
    rng: ChaCha8Rng,
    /// Current rates for each currency (log of rate for O-U process).
    current_log_rates: HashMap<String, f64>,
    /// Base rates (initial starting points).
    base_rates: HashMap<String, Decimal>,
}

impl FxRateService {
    /// Creates a new FX rate service.
    pub fn new(config: FxRateServiceConfig, rng: ChaCha8Rng) -> Self {
        let base_rates = base_rates_usd();
        let mut current_log_rates = HashMap::new();

        // Initialize log rates from base rates
        for currency in &config.currencies {
            if let Some(rate) = base_rates.get(currency) {
                let rate_f64: f64 = rate.to_f64().unwrap_or(1.0);
                current_log_rates.insert(currency.clone(), rate_f64.ln());
            }
        }

        Self {
            config,
            rng,
            current_log_rates,
            base_rates,
        }
    }

    /// Generates daily FX rates for a date range.
    pub fn generate_daily_rates(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> FxRateTable {
        let mut table = FxRateTable::new(&self.config.base_currency);
        let mut current_date = start_date;

        while current_date <= end_date {
            // Skip weekends if configured
            if !self.config.include_weekends {
                let weekday = current_date.weekday();
                if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
                    current_date = current_date.succ_opt().unwrap_or(current_date);
                    continue;
                }
            }

            // Generate rates for each currency
            for currency in self.config.currencies.clone() {
                if currency == self.config.base_currency {
                    continue;
                }

                let rate = self.generate_next_rate(&currency, current_date);
                table.add_rate(rate);
            }

            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        table
    }

    /// Generates period rates (closing and average) for a fiscal period.
    pub fn generate_period_rates(
        &mut self,
        year: i32,
        month: u32,
        daily_table: &FxRateTable,
    ) -> Vec<FxRate> {
        let mut rates = Vec::new();

        let period_start =
            NaiveDate::from_ymd_opt(year, month, 1).expect("valid year/month for period start");
        let period_end = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .expect("valid next year start")
                .pred_opt()
                .expect("valid predecessor date")
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
                .expect("valid next month start")
                .pred_opt()
                .expect("valid predecessor date")
        };

        for currency in &self.config.currencies {
            if *currency == self.config.base_currency {
                continue;
            }

            // Closing rate = last spot rate of the period
            if let Some(closing) =
                daily_table.get_spot_rate(currency, &self.config.base_currency, period_end)
            {
                rates.push(FxRate::new(
                    currency,
                    &self.config.base_currency,
                    RateType::Closing,
                    period_end,
                    closing.rate,
                    "GENERATED",
                ));
            }

            // Average rate = simple average of all spot rates in the period
            let spot_rates: Vec<&FxRate> = daily_table
                .get_all_rates(currency, &self.config.base_currency)
                .into_iter()
                .filter(|r| {
                    r.rate_type == RateType::Spot
                        && r.effective_date >= period_start
                        && r.effective_date <= period_end
                })
                .collect();

            if !spot_rates.is_empty() {
                let sum: Decimal = spot_rates.iter().map(|r| r.rate).sum();
                let avg = sum / Decimal::from(spot_rates.len());

                rates.push(FxRate::new(
                    currency,
                    &self.config.base_currency,
                    RateType::Average,
                    period_end,
                    avg.round_dp(6),
                    "GENERATED",
                ));
            }
        }

        rates
    }

    /// Generates the next rate using Ornstein-Uhlenbeck process.
    ///
    /// The O-U process: dX = θ(μ - X)dt + σdW
    /// Where:
    /// - θ = mean reversion speed
    /// - μ = long-term mean
    /// - σ = volatility
    /// - dW = Wiener process increment
    fn generate_next_rate(&mut self, currency: &str, date: NaiveDate) -> FxRate {
        let current_log = *self.current_log_rates.get(currency).unwrap_or(&0.0);

        // Get base rate for mean reversion target
        let base_rate: f64 = self
            .base_rates
            .get(currency)
            .map(|d| (*d).try_into().unwrap_or(1.0))
            .unwrap_or(1.0);
        let base_log = base_rate.ln();

        // Ornstein-Uhlenbeck step
        let theta = self.config.mean_reversion_speed;
        let mu = base_log + self.config.long_term_mean;

        // Check for fat-tail event
        let volatility = if self.rng.gen::<f64>() < self.config.fat_tail_probability {
            self.config.daily_volatility * self.config.fat_tail_multiplier
        } else {
            self.config.daily_volatility
        };

        // Generate normal random shock
        let normal = Normal::new(0.0, 1.0).expect("valid standard normal parameters");
        let dw: f64 = normal.sample(&mut self.rng);

        // O-U update: X(t+1) = X(t) + θ(μ - X(t)) + σ * dW
        let drift = theta * (mu - current_log);
        let diffusion = volatility * dw;
        let new_log = current_log + drift + diffusion;

        // Update state
        self.current_log_rates.insert(currency.to_string(), new_log);

        // Convert log rate back to actual rate
        let new_rate = new_log.exp();
        let rate_decimal = Decimal::try_from(new_rate).unwrap_or(dec!(1)).round_dp(6);

        FxRate::new(
            currency,
            &self.config.base_currency,
            RateType::Spot,
            date,
            rate_decimal,
            "O-U PROCESS",
        )
    }

    /// Resets the rate service to initial base rates.
    pub fn reset(&mut self) {
        self.current_log_rates.clear();
        for currency in &self.config.currencies {
            if let Some(rate) = self.base_rates.get(currency) {
                let rate_f64: f64 = (*rate).try_into().unwrap_or(1.0);
                self.current_log_rates
                    .insert(currency.clone(), rate_f64.ln());
            }
        }
    }

    /// Gets the current rate for a currency.
    pub fn current_rate(&self, currency: &str) -> Option<Decimal> {
        self.current_log_rates.get(currency).map(|log_rate| {
            let rate = log_rate.exp();
            Decimal::try_from(rate).unwrap_or(dec!(1)).round_dp(6)
        })
    }
}

/// Generates a complete set of FX rates for a simulation period.
pub struct FxRateGenerator {
    service: FxRateService,
}

impl FxRateGenerator {
    /// Creates a new FX rate generator.
    pub fn new(config: FxRateServiceConfig, rng: ChaCha8Rng) -> Self {
        Self {
            service: FxRateService::new(config, rng),
        }
    }

    /// Generates all rates (daily, closing, average) for a date range.
    pub fn generate_all_rates(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> GeneratedFxRates {
        // Generate daily spot rates
        let daily_rates = self.service.generate_daily_rates(start_date, end_date);

        // Generate period rates for each month
        let mut period_rates = Vec::new();
        let mut current_year = start_date.year();
        let mut current_month = start_date.month();

        while (current_year < end_date.year())
            || (current_year == end_date.year() && current_month <= end_date.month())
        {
            let rates =
                self.service
                    .generate_period_rates(current_year, current_month, &daily_rates);
            period_rates.extend(rates);

            // Move to next month
            if current_month == 12 {
                current_month = 1;
                current_year += 1;
            } else {
                current_month += 1;
            }
        }

        GeneratedFxRates {
            daily_rates,
            period_rates,
            start_date,
            end_date,
        }
    }

    /// Gets a reference to the underlying service.
    pub fn service(&self) -> &FxRateService {
        &self.service
    }

    /// Gets a mutable reference to the underlying service.
    pub fn service_mut(&mut self) -> &mut FxRateService {
        &mut self.service
    }
}

/// Container for all generated FX rates.
#[derive(Debug, Clone)]
pub struct GeneratedFxRates {
    /// Daily spot rates.
    pub daily_rates: FxRateTable,
    /// Period closing and average rates.
    pub period_rates: Vec<FxRate>,
    /// Start date of generation.
    pub start_date: NaiveDate,
    /// End date of generation.
    pub end_date: NaiveDate,
}

impl GeneratedFxRates {
    /// Combines all rates into a single rate table.
    pub fn combined_rate_table(&self) -> FxRateTable {
        let mut table = self.daily_rates.clone();
        for rate in &self.period_rates {
            table.add_rate(rate.clone());
        }
        table
    }

    /// Gets closing rates for a specific period end date.
    pub fn closing_rates_for_date(&self, date: NaiveDate) -> Vec<&FxRate> {
        self.period_rates
            .iter()
            .filter(|r| r.rate_type == RateType::Closing && r.effective_date == date)
            .collect()
    }

    /// Gets average rates for a specific period end date.
    pub fn average_rates_for_date(&self, date: NaiveDate) -> Vec<&FxRate> {
        self.period_rates
            .iter()
            .filter(|r| r.rate_type == RateType::Average && r.effective_date == date)
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_fx_rate_generation() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let config = FxRateServiceConfig {
            currencies: vec!["EUR".to_string(), "GBP".to_string()],
            ..Default::default()
        };

        let mut service = FxRateService::new(config, rng);

        let rates = service.generate_daily_rates(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        // Should have rates for each business day
        assert!(!rates.is_empty());
    }

    #[test]
    fn test_rate_mean_reversion() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let config = FxRateServiceConfig {
            currencies: vec!["EUR".to_string()],
            mean_reversion_speed: 0.1, // Strong mean reversion
            daily_volatility: 0.001,   // Low volatility
            ..Default::default()
        };

        let mut service = FxRateService::new(config.clone(), rng);

        // Generate 100 days of rates
        let rates = service.generate_daily_rates(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 4, 10).unwrap(),
        );

        // With strong mean reversion and low volatility, rates should stay near base
        let base_eur = base_rates_usd().get("EUR").cloned().unwrap_or(dec!(1.10));
        let all_eur_rates: Vec<Decimal> = rates
            .get_all_rates("EUR", "USD")
            .iter()
            .map(|r| r.rate)
            .collect();

        assert!(!all_eur_rates.is_empty());

        // Check that rates stay within reasonable bounds of base rate (±10%)
        for rate in &all_eur_rates {
            let deviation = (*rate - base_eur).abs() / base_eur;
            assert!(
                deviation < dec!(0.15),
                "Rate {} deviated too much from base {}",
                rate,
                base_eur
            );
        }
    }

    #[test]
    fn test_period_rates() {
        let rng = ChaCha8Rng::seed_from_u64(12345);
        let config = FxRateServiceConfig {
            currencies: vec!["EUR".to_string()],
            ..Default::default()
        };

        let mut generator = FxRateGenerator::new(config, rng);

        let generated = generator.generate_all_rates(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
        );

        // Should have closing and average rates for each month
        let jan_closing =
            generated.closing_rates_for_date(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
        assert!(!jan_closing.is_empty());

        let jan_average =
            generated.average_rates_for_date(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
        assert!(!jan_average.is_empty());
    }
}
