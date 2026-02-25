//! Transaction amount distribution sampler.
//!
//! Generates realistic transaction amounts using log-normal distributions
//! and round-number bias commonly observed in accounting data.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, LogNormal};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::benford::{BenfordSampler, FraudAmountGenerator, FraudAmountPattern, ThresholdConfig};

/// Configuration for amount distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountDistributionConfig {
    /// Minimum transaction amount
    pub min_amount: f64,
    /// Maximum transaction amount
    pub max_amount: f64,
    /// Log-normal mu parameter (location)
    pub lognormal_mu: f64,
    /// Log-normal sigma parameter (scale)
    pub lognormal_sigma: f64,
    /// Number of decimal places to round to
    pub decimal_places: u8,
    /// Probability of round number (ending in .00)
    pub round_number_probability: f64,
    /// Probability of nice number (ending in 0 or 5)
    pub nice_number_probability: f64,
}

impl Default for AmountDistributionConfig {
    fn default() -> Self {
        Self {
            min_amount: 0.01,
            max_amount: 100_000_000.0, // 100 million
            lognormal_mu: 7.0,         // Center around ~1000
            lognormal_sigma: 2.5,      // Wide spread
            decimal_places: 2,
            round_number_probability: 0.25, // 25% chance of .00 ending
            nice_number_probability: 0.15,  // 15% chance of nice numbers
        }
    }
}

impl AmountDistributionConfig {
    /// Configuration for small transactions (e.g., retail).
    pub fn small_transactions() -> Self {
        Self {
            min_amount: 0.01,
            max_amount: 10_000.0,
            lognormal_mu: 4.0, // Center around ~55
            lognormal_sigma: 1.5,
            decimal_places: 2,
            round_number_probability: 0.30,
            nice_number_probability: 0.20,
        }
    }

    /// Configuration for medium transactions (e.g., B2B).
    pub fn medium_transactions() -> Self {
        Self {
            min_amount: 100.0,
            max_amount: 1_000_000.0,
            lognormal_mu: 8.5, // Center around ~5000
            lognormal_sigma: 2.0,
            decimal_places: 2,
            round_number_probability: 0.20,
            nice_number_probability: 0.15,
        }
    }

    /// Configuration for large transactions (e.g., enterprise).
    pub fn large_transactions() -> Self {
        Self {
            min_amount: 1000.0,
            max_amount: 100_000_000.0,
            lognormal_mu: 10.0, // Center around ~22000
            lognormal_sigma: 2.5,
            decimal_places: 2,
            round_number_probability: 0.15,
            nice_number_probability: 0.10,
        }
    }
}

/// Sampler for realistic transaction amounts.
pub struct AmountSampler {
    /// RNG for sampling
    rng: ChaCha8Rng,
    /// Configuration
    config: AmountDistributionConfig,
    /// Log-normal distribution
    lognormal: LogNormal<f64>,
    /// Decimal multiplier for rounding
    decimal_multiplier: f64,
    /// Optional Benford sampler for compliant generation
    benford_sampler: Option<BenfordSampler>,
    /// Optional fraud amount generator
    fraud_generator: Option<FraudAmountGenerator>,
    /// Whether Benford's Law compliance is enabled
    benford_enabled: bool,
}

impl AmountSampler {
    /// Create a new sampler with default configuration.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, AmountDistributionConfig::default())
    }

    /// Create a sampler with custom configuration.
    pub fn with_config(seed: u64, config: AmountDistributionConfig) -> Self {
        let lognormal = LogNormal::new(config.lognormal_mu, config.lognormal_sigma)
            .expect("Invalid log-normal parameters");
        let decimal_multiplier = 10_f64.powi(config.decimal_places as i32);

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            lognormal,
            decimal_multiplier,
            benford_sampler: None,
            fraud_generator: None,
            benford_enabled: false,
        }
    }

    /// Create a sampler with Benford's Law compliance enabled.
    pub fn with_benford(seed: u64, config: AmountDistributionConfig) -> Self {
        let lognormal = LogNormal::new(config.lognormal_mu, config.lognormal_sigma)
            .expect("Invalid log-normal parameters");
        let decimal_multiplier = 10_f64.powi(config.decimal_places as i32);

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            benford_sampler: Some(BenfordSampler::new(seed + 100, config.clone())),
            fraud_generator: Some(FraudAmountGenerator::new(
                seed + 200,
                config.clone(),
                ThresholdConfig::default(),
            )),
            config,
            lognormal,
            decimal_multiplier,
            benford_enabled: true,
        }
    }

    /// Create a sampler with full fraud configuration.
    pub fn with_fraud_config(
        seed: u64,
        config: AmountDistributionConfig,
        threshold_config: ThresholdConfig,
        benford_enabled: bool,
    ) -> Self {
        let lognormal = LogNormal::new(config.lognormal_mu, config.lognormal_sigma)
            .expect("Invalid log-normal parameters");
        let decimal_multiplier = 10_f64.powi(config.decimal_places as i32);

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            benford_sampler: if benford_enabled {
                Some(BenfordSampler::new(seed + 100, config.clone()))
            } else {
                None
            },
            fraud_generator: Some(FraudAmountGenerator::new(
                seed + 200,
                config.clone(),
                threshold_config,
            )),
            config,
            lognormal,
            decimal_multiplier,
            benford_enabled,
        }
    }

    /// Enable or disable Benford's Law compliance.
    pub fn set_benford_enabled(&mut self, enabled: bool) {
        self.benford_enabled = enabled;
        if enabled && self.benford_sampler.is_none() {
            // Initialize Benford sampler if not already present
            let seed = self.rng.random();
            self.benford_sampler = Some(BenfordSampler::new(seed, self.config.clone()));
        }
    }

    /// Check if Benford's Law compliance is enabled.
    pub fn is_benford_enabled(&self) -> bool {
        self.benford_enabled
    }

    /// Sample a single amount.
    ///
    /// If Benford's Law compliance is enabled, uses the Benford sampler.
    /// Otherwise uses log-normal distribution with round-number bias.
    #[inline]
    pub fn sample(&mut self) -> Decimal {
        // Use Benford sampler if enabled
        if self.benford_enabled {
            if let Some(ref mut benford) = self.benford_sampler {
                return benford.sample();
            }
        }

        // Fall back to log-normal sampling
        self.sample_lognormal()
    }

    /// Sample using the log-normal distribution (original behavior).
    #[inline]
    pub fn sample_lognormal(&mut self) -> Decimal {
        let mut amount = self.lognormal.sample(&mut self.rng);

        // Clamp to configured range
        amount = amount.clamp(self.config.min_amount, self.config.max_amount);

        // Apply round number bias
        let p: f64 = self.rng.random();
        if p < self.config.round_number_probability {
            // Round to nearest whole number ending in 00
            amount = (amount / 100.0).round() * 100.0;
        } else if p < self.config.round_number_probability + self.config.nice_number_probability {
            // Round to nearest 5 or 10
            amount = (amount / 5.0).round() * 5.0;
        }

        // Round to configured decimal places
        amount = (amount * self.decimal_multiplier).round() / self.decimal_multiplier;

        // Ensure minimum after rounding
        amount = amount.max(self.config.min_amount);

        // Convert to Decimal using fast integer math instead of string formatting.
        // Multiply by 100, truncate to integer, then construct Decimal with scale 2.
        // This avoids the overhead of format!() + parse() (~15x faster).
        let cents = (amount * 100.0).round() as i64;
        Decimal::new(cents, 2)
    }

    /// Sample a fraud amount with the specified pattern.
    ///
    /// Returns a normal amount if fraud generator is not configured.
    pub fn sample_fraud(&mut self, pattern: FraudAmountPattern) -> Decimal {
        if let Some(ref mut fraud_gen) = self.fraud_generator {
            fraud_gen.sample(pattern)
        } else {
            // Fallback to normal sampling
            self.sample()
        }
    }

    /// Sample multiple amounts that sum to a target total.
    ///
    /// Useful for generating line items that must balance.
    /// The sum of returned amounts is guaranteed to equal `total` exactly.
    /// Every returned amount is guaranteed to be > 0 when `total > 0` and
    /// `count * 0.01 <= total`.
    pub fn sample_summing_to(&mut self, count: usize, total: Decimal) -> Vec<Decimal> {
        use rust_decimal::prelude::ToPrimitive;

        let min_amount = Decimal::new(1, 2); // 0.01

        if count == 0 {
            return Vec::new();
        }
        if count == 1 {
            return vec![total];
        }

        let total_f64 = total.to_f64().unwrap_or(0.0);

        // Generate random weights ensuring minimum weight
        let mut weights: Vec<f64> = (0..count)
            .map(|_| self.rng.random::<f64>().max(0.01))
            .collect();
        let sum: f64 = weights.iter().sum();
        weights.iter_mut().for_each(|w| *w /= sum);

        // Calculate amounts based on weights, using fast integer math for precision
        let mut amounts: Vec<Decimal> = weights
            .iter()
            .map(|w| {
                let amount = total_f64 * w;
                let rounded = (amount * self.decimal_multiplier).round() / self.decimal_multiplier;
                // Convert via integer cents — avoids format!()/parse() overhead
                let cents = (rounded * 100.0).round() as i64;
                Decimal::new(cents, 2)
            })
            .collect();

        // Adjust last amount to ensure exact sum
        let current_sum: Decimal = amounts.iter().copied().sum();
        let diff = total - current_sum;
        let last_idx = amounts.len() - 1;
        amounts[last_idx] += diff;

        // If last amount became negative (rare edge case), redistribute
        if amounts[last_idx] < Decimal::ZERO {
            let mut remaining = amounts[last_idx].abs();
            amounts[last_idx] = Decimal::ZERO;

            // Distribute the negative amount across all earlier amounts
            for amt in amounts.iter_mut().take(last_idx).rev() {
                if remaining <= Decimal::ZERO {
                    break;
                }
                let take = remaining.min(*amt);
                *amt -= take;
                remaining -= take;
            }

            // If still remaining, absorb into the first non-zero amount
            if remaining > Decimal::ZERO {
                for amt in amounts.iter_mut() {
                    if *amt > Decimal::ZERO {
                        *amt -= remaining;
                        break;
                    }
                }
            }
        }

        // Post-process: fix zero-amount lines by transferring min_amount from the
        // largest line. This preserves the exact sum while eliminating zeros.
        // Only attempt when total is large enough to support min_amount per line.
        if total >= min_amount * Decimal::from(count as u32) {
            loop {
                // Find a zero-amount line
                let zero_idx = amounts.iter().position(|a| *a == Decimal::ZERO);
                let Some(zi) = zero_idx else { break };

                // Find the largest amount (must be > min_amount to donate)
                let donor = amounts
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != zi)
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(j, _)| j);

                if let Some(di) = donor {
                    if amounts[di] > min_amount {
                        amounts[zi] = min_amount;
                        amounts[di] -= min_amount;
                    } else {
                        break; // No donor has enough headroom
                    }
                } else {
                    break;
                }
            }
        }

        amounts
    }

    /// Sample an amount within a specific range.
    pub fn sample_in_range(&mut self, min: Decimal, max: Decimal) -> Decimal {
        let min_f64 = min.to_string().parse::<f64>().unwrap_or(0.0);
        let max_f64 = max.to_string().parse::<f64>().unwrap_or(1000000.0);

        let range = max_f64 - min_f64;
        let amount = min_f64 + self.rng.random::<f64>() * range;

        let rounded = (amount * self.decimal_multiplier).round() / self.decimal_multiplier;
        Decimal::from_f64_retain(rounded).unwrap_or(min)
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }
}

/// Sampler for currency exchange rates.
pub struct ExchangeRateSampler {
    rng: ChaCha8Rng,
    /// Base rates for common currency pairs (vs USD)
    base_rates: std::collections::HashMap<String, f64>,
    /// Daily volatility (standard deviation)
    volatility: f64,
}

impl ExchangeRateSampler {
    /// Create a new exchange rate sampler.
    pub fn new(seed: u64) -> Self {
        let mut base_rates = std::collections::HashMap::new();
        // Approximate rates as of 2024
        base_rates.insert("EUR".to_string(), 0.92);
        base_rates.insert("GBP".to_string(), 0.79);
        base_rates.insert("CHF".to_string(), 0.88);
        base_rates.insert("JPY".to_string(), 149.0);
        base_rates.insert("CNY".to_string(), 7.24);
        base_rates.insert("CAD".to_string(), 1.36);
        base_rates.insert("AUD".to_string(), 1.53);
        base_rates.insert("INR".to_string(), 83.0);
        base_rates.insert("USD".to_string(), 1.0);

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            base_rates,
            volatility: 0.005, // 0.5% daily volatility
        }
    }

    /// Get exchange rate from one currency to another.
    pub fn get_rate(&mut self, from: &str, to: &str) -> Decimal {
        let from_usd = self.base_rates.get(from).copied().unwrap_or(1.0);
        let to_usd = self.base_rates.get(to).copied().unwrap_or(1.0);

        // Base rate
        let base_rate = to_usd / from_usd;

        // Add some random variation
        let variation = 1.0 + (self.rng.random::<f64>() - 0.5) * 2.0 * self.volatility;
        let rate = base_rate * variation;

        // Round to 6 decimal places
        let rounded = (rate * 1_000_000.0).round() / 1_000_000.0;
        Decimal::from_f64_retain(rounded).unwrap_or(Decimal::ONE)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_sampler_determinism() {
        let mut sampler1 = AmountSampler::new(42);
        let mut sampler2 = AmountSampler::new(42);

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_amount_sampler_range() {
        let config = AmountDistributionConfig {
            min_amount: 100.0,
            max_amount: 1000.0,
            ..Default::default()
        };
        let mut sampler = AmountSampler::with_config(42, config);

        for _ in 0..1000 {
            let amount = sampler.sample();
            let amount_f64: f64 = amount.to_string().parse().unwrap();
            assert!(amount_f64 >= 100.0, "Amount {} below minimum", amount);
            assert!(amount_f64 <= 1000.0, "Amount {} above maximum", amount);
        }
    }

    #[test]
    fn test_summing_amounts() {
        let mut sampler = AmountSampler::new(42);
        let total = Decimal::from(10000);
        let amounts = sampler.sample_summing_to(5, total);

        assert_eq!(amounts.len(), 5);

        let sum: Decimal = amounts.iter().sum();
        assert_eq!(sum, total, "Sum {} doesn't match total {}", sum, total);
    }

    #[test]
    fn test_exchange_rate() {
        let mut sampler = ExchangeRateSampler::new(42);

        let eur_usd = sampler.get_rate("EUR", "USD");
        let eur_f64: f64 = eur_usd.to_string().parse().unwrap();
        assert!(
            eur_f64 > 0.8 && eur_f64 < 1.2,
            "EUR/USD rate {} out of range",
            eur_f64
        );

        let usd_usd = sampler.get_rate("USD", "USD");
        let usd_f64: f64 = usd_usd.to_string().parse().unwrap();
        assert!(
            (usd_f64 - 1.0).abs() < 0.01,
            "USD/USD rate {} should be ~1.0",
            usd_f64
        );
    }
}
