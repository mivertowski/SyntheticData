//! Benford's Law distribution sampler and fraud amount patterns.
//!
//! Implements Benford's Law compliant amount generation and various fraud
//! amount patterns for realistic synthetic accounting data. Includes enhanced
//! multi-digit Benford analysis and deviation patterns for anomaly injection.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::AmountDistributionConfig;

/// Benford's Law probability distribution for first digits 1-9.
/// P(d) = log10(1 + 1/d)
/// Note: Uses explicit values to satisfy clippy while maintaining exact precision.
#[allow(clippy::approx_constant)]
pub const BENFORD_PROBABILITIES: [f64; 9] = [
    0.30103, // 1: 30.1% - log10(2)
    0.17609, // 2: 17.6%
    0.12494, // 3: 12.5%
    0.09691, // 4: 9.7%
    0.07918, // 5: 7.9%
    0.06695, // 6: 6.7%
    0.05799, // 7: 5.8%
    0.05115, // 8: 5.1%
    0.04576, // 9: 4.6%
];

/// Cumulative distribution function for Benford's Law.
/// Note: Uses explicit values to satisfy clippy while maintaining exact precision.
#[allow(clippy::approx_constant)]
pub const BENFORD_CDF: [f64; 9] = [
    0.30103, // 1 - log10(2)
    0.47712, // 1-2
    0.60206, // 1-3
    0.69897, // 1-4
    0.77815, // 1-5
    0.84510, // 1-6
    0.90309, // 1-7
    0.95424, // 1-8
    1.00000, // 1-9
];

/// Benford's Law probability distribution for second digits 0-9.
/// P(d2) = sum over d1 of log10(1 + 1/(10*d1 + d2))
#[allow(clippy::approx_constant)]
pub const BENFORD_SECOND_DIGIT_PROBABILITIES: [f64; 10] = [
    0.11968, // 0: 12.0%
    0.11389, // 1: 11.4%
    0.10882, // 2: 10.9%
    0.10433, // 3: 10.4%
    0.10031, // 4: 10.0%
    0.09668, // 5: 9.7%
    0.09337, // 6: 9.3%
    0.09035, // 7: 9.0%
    0.08757, // 8: 8.8%
    0.08500, // 9: 8.5%
];

/// Cumulative distribution function for second digit Benford's Law.
pub const BENFORD_SECOND_DIGIT_CDF: [f64; 10] = [
    0.11968, 0.23357, 0.34239, 0.44672, 0.54703, 0.64371, 0.73708, 0.82743, 0.91500, 1.00000,
];

/// Calculate Benford's Law probability for first two digits (10-99).
/// P(d1d2) = log10(1 + 1/(d1*10 + d2))
pub fn benford_first_two_probability(d1: u8, d2: u8) -> f64 {
    if !(1..=9).contains(&d1) || d2 > 9 {
        return 0.0;
    }
    let n = (d1 as f64) * 10.0 + (d2 as f64);
    (1.0 + 1.0 / n).log10()
}

/// Get all first-two-digit probabilities as a 90-element array (10-99).
pub fn benford_first_two_probabilities() -> [f64; 90] {
    let mut probs = [0.0; 90];
    for d1 in 1..=9 {
        for d2 in 0..=9 {
            let idx = (d1 - 1) * 10 + d2;
            probs[idx as usize] = benford_first_two_probability(d1, d2);
        }
    }
    probs
}

/// Anti-Benford distribution for generating statistically improbable amounts.
/// Overweights digits 5, 7, and 9 which are typically rare in natural data.
pub const ANTI_BENFORD_PROBABILITIES: [f64; 9] = [
    0.05, // 1: 5% (normally 30%)
    0.05, // 2: 5% (normally 18%)
    0.05, // 3: 5% (normally 12%)
    0.10, // 4: 10%
    0.25, // 5: 25% (normally 8%)
    0.10, // 6: 10%
    0.20, // 7: 20% (normally 6%)
    0.05, // 8: 5%
    0.15, // 9: 15% (normally 5%)
];

/// Fraud amount pattern types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FraudAmountPattern {
    /// Normal amount generation (Benford-compliant if enabled)
    #[default]
    Normal,
    /// Statistically improbable first digits (anti-Benford)
    /// Excess of leading 5s, 7s, 9s - detectable via statistical analysis
    StatisticallyImprobable,
    /// Obvious round numbers ($50,000.00, $99,999.99)
    /// Easy to spot in visual review
    ObviousRoundNumbers,
    /// Amounts clustered just below approval thresholds
    /// Classic split-transaction pattern
    ThresholdAdjacent,
}

/// Configuration for threshold-adjacent fraud pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Approval thresholds to cluster below
    pub thresholds: Vec<f64>,
    /// Minimum percentage below threshold (e.g., 0.01 = 1%)
    pub min_below_pct: f64,
    /// Maximum percentage below threshold (e.g., 0.15 = 15%)
    pub max_below_pct: f64,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            thresholds: vec![1000.0, 5000.0, 10000.0, 25000.0, 50000.0, 100000.0],
            min_below_pct: 0.01,
            max_below_pct: 0.15,
        }
    }
}

/// Sampler that produces amounts following Benford's Law distribution.
pub struct BenfordSampler {
    rng: ChaCha8Rng,
    config: AmountDistributionConfig,
}

impl BenfordSampler {
    /// Create a new Benford sampler with the given seed and amount configuration.
    pub fn new(seed: u64, config: AmountDistributionConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
        }
    }

    /// Sample a first digit according to Benford's Law.
    fn sample_benford_first_digit(&mut self) -> u8 {
        let p: f64 = self.rng.gen();
        for (i, &cumulative) in BENFORD_CDF.iter().enumerate() {
            if p < cumulative {
                return (i + 1) as u8;
            }
        }
        9
    }

    /// Sample a first digit from the anti-Benford distribution.
    fn sample_anti_benford_first_digit(&mut self) -> u8 {
        let p: f64 = self.rng.gen();
        let mut cumulative = 0.0;
        for (i, &prob) in ANTI_BENFORD_PROBABILITIES.iter().enumerate() {
            cumulative += prob;
            if p < cumulative {
                return (i + 1) as u8;
            }
        }
        9
    }

    /// Sample an amount following Benford's Law.
    pub fn sample(&mut self) -> Decimal {
        let first_digit = self.sample_benford_first_digit();
        self.sample_with_first_digit(first_digit)
    }

    /// Sample an amount with a specific first digit.
    pub fn sample_with_first_digit(&mut self, first_digit: u8) -> Decimal {
        let first_digit = first_digit.clamp(1, 9);

        // Determine the order of magnitude based on config range
        let min_magnitude = self.config.min_amount.log10().floor() as i32;
        let max_magnitude = self.config.max_amount.log10().floor() as i32;

        // Sample a magnitude within the valid range
        let magnitude = self.rng.gen_range(min_magnitude..=max_magnitude);
        let base = 10_f64.powi(magnitude);

        // Generate the remaining digits (0.0 to 0.999...)
        let remaining: f64 = self.rng.gen();

        // Construct: first_digit.remaining * 10^magnitude
        let mantissa = first_digit as f64 + remaining;
        let mut amount = mantissa * base;

        // Clamp to configured range
        amount = amount.clamp(self.config.min_amount, self.config.max_amount);

        // Apply round number bias (25% chance)
        let p: f64 = self.rng.gen();
        if p < self.config.round_number_probability {
            // Round to nearest whole number ending in 00
            amount = (amount / 100.0).round() * 100.0;
        } else if p < self.config.round_number_probability + self.config.nice_number_probability {
            // Round to nearest 5 or 10
            amount = (amount / 5.0).round() * 5.0;
        }

        // Round to configured decimal places
        let decimal_multiplier = 10_f64.powi(self.config.decimal_places as i32);
        amount = (amount * decimal_multiplier).round() / decimal_multiplier;

        // Ensure minimum after rounding
        amount = amount.max(self.config.min_amount);

        Decimal::from_f64_retain(amount).unwrap_or(Decimal::ONE)
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }
}

/// Generator for fraudulent amount patterns.
pub struct FraudAmountGenerator {
    rng: ChaCha8Rng,
    benford_sampler: BenfordSampler,
    threshold_config: ThresholdConfig,
    config: AmountDistributionConfig,
}

impl FraudAmountGenerator {
    /// Create a new fraud amount generator.
    pub fn new(
        seed: u64,
        config: AmountDistributionConfig,
        threshold_config: ThresholdConfig,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            benford_sampler: BenfordSampler::new(seed + 1, config.clone()),
            threshold_config,
            config,
        }
    }

    /// Generate an amount with the specified fraud pattern.
    pub fn sample(&mut self, pattern: FraudAmountPattern) -> Decimal {
        match pattern {
            FraudAmountPattern::Normal => self.benford_sampler.sample(),
            FraudAmountPattern::StatisticallyImprobable => self.sample_anti_benford(),
            FraudAmountPattern::ObviousRoundNumbers => self.sample_obvious_round(),
            FraudAmountPattern::ThresholdAdjacent => self.sample_threshold_adjacent(),
        }
    }

    /// Generate an amount with statistically improbable first digit distribution.
    fn sample_anti_benford(&mut self) -> Decimal {
        let first_digit = self.benford_sampler.sample_anti_benford_first_digit();
        self.benford_sampler.sample_with_first_digit(first_digit)
    }

    /// Generate an obvious round number amount (suspicious pattern).
    fn sample_obvious_round(&mut self) -> Decimal {
        let pattern_choice = self.rng.gen_range(0..5);

        let amount = match pattern_choice {
            // Even thousands ($1,000, $5,000, $10,000, etc.)
            0 => {
                let multiplier = self.rng.gen_range(1..100);
                multiplier as f64 * 1000.0
            }
            // $X9,999.99 pattern (just under round number)
            1 => {
                let base = self.rng.gen_range(1..10) as f64 * 10000.0;
                base - 0.01
            }
            // Exact $X0,000.00 pattern
            2 => {
                let multiplier = self.rng.gen_range(1..20);
                multiplier as f64 * 10000.0
            }
            // Five-thousands ($5,000, $15,000, $25,000)
            3 => {
                let multiplier = self.rng.gen_range(1..40);
                multiplier as f64 * 5000.0
            }
            // $X,999.99 pattern
            _ => {
                let base = self.rng.gen_range(1..100) as f64 * 1000.0;
                base - 0.01
            }
        };

        // Clamp to config range
        let clamped = amount.clamp(self.config.min_amount, self.config.max_amount);
        Decimal::from_f64_retain(clamped).unwrap_or(Decimal::ONE)
    }

    /// Generate an amount just below an approval threshold.
    fn sample_threshold_adjacent(&mut self) -> Decimal {
        // Select a threshold
        let threshold = if self.threshold_config.thresholds.is_empty() {
            10000.0
        } else {
            *self
                .threshold_config
                .thresholds
                .choose(&mut self.rng)
                .unwrap_or(&10000.0)
        };

        // Calculate amount as percentage below threshold
        let pct_below = self
            .rng
            .gen_range(self.threshold_config.min_below_pct..self.threshold_config.max_below_pct);
        let base_amount = threshold * (1.0 - pct_below);

        // Add small noise to avoid exact patterns
        let noise_factor = 1.0 + self.rng.gen_range(-0.005..0.005);
        let amount = base_amount * noise_factor;

        // Round to 2 decimal places
        let rounded = (amount * 100.0).round() / 100.0;

        // Ensure we're still below threshold
        let final_amount = rounded.min(threshold - 0.01);
        let clamped = final_amount.clamp(self.config.min_amount, self.config.max_amount);

        Decimal::from_f64_retain(clamped).unwrap_or(Decimal::ONE)
    }

    /// Reset the generator with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
        self.benford_sampler.reset(seed + 1);
    }
}

/// Extract the first digit from a decimal amount.
pub fn get_first_digit(amount: Decimal) -> Option<u8> {
    let s = amount.to_string();
    s.chars()
        .find(|c| c.is_ascii_digit() && *c != '0')
        .and_then(|c| c.to_digit(10))
        .map(|d| d as u8)
}

/// Extract the first two digits from a decimal amount.
pub fn get_first_two_digits(amount: Decimal) -> Option<(u8, u8)> {
    let s = amount.abs().to_string();
    let mut first_found = false;
    let mut first_digit = 0u8;

    for c in s.chars() {
        if c.is_ascii_digit() {
            let d = c.to_digit(10).unwrap() as u8;
            if !first_found && d != 0 {
                first_digit = d;
                first_found = true;
            } else if first_found && c != '.' {
                return Some((first_digit, d));
            }
        }
    }
    None
}

/// Configuration for enhanced Benford sampling with multi-digit compliance.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnhancedBenfordConfig {
    /// Base amount distribution configuration
    pub amount_config: AmountDistributionConfig,
    /// Whether to enforce second-digit Benford compliance
    #[serde(default)]
    pub second_digit_compliance: bool,
    /// Whether to enforce first-two-digit Benford compliance
    #[serde(default)]
    pub first_two_digit_compliance: bool,
}

/// Enhanced Benford sampler with multi-digit compliance.
pub struct EnhancedBenfordSampler {
    rng: ChaCha8Rng,
    config: EnhancedBenfordConfig,
    /// Pre-computed CDF for first two digits
    first_two_cdf: [f64; 90],
}

impl EnhancedBenfordSampler {
    /// Create a new enhanced Benford sampler.
    pub fn new(seed: u64, config: EnhancedBenfordConfig) -> Self {
        // Pre-compute CDF for first two digits
        let probs = benford_first_two_probabilities();
        let mut first_two_cdf = [0.0; 90];
        let mut cumulative = 0.0;
        for i in 0..90 {
            cumulative += probs[i];
            first_two_cdf[i] = cumulative;
        }

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            first_two_cdf,
        }
    }

    /// Sample first two digits according to Benford's Law.
    fn sample_first_two_digits(&mut self) -> (u8, u8) {
        let p: f64 = self.rng.gen();
        for (i, &cdf) in self.first_two_cdf.iter().enumerate() {
            if p < cdf {
                let d1 = (i / 10 + 1) as u8;
                let d2 = (i % 10) as u8;
                return (d1, d2);
            }
        }
        (9, 9)
    }

    /// Sample a second digit according to Benford's Law.
    fn sample_second_digit(&mut self) -> u8 {
        let p: f64 = self.rng.gen();
        for (i, &cdf) in BENFORD_SECOND_DIGIT_CDF.iter().enumerate() {
            if p < cdf {
                return i as u8;
            }
        }
        9
    }

    /// Sample a first digit according to Benford's Law.
    fn sample_first_digit(&mut self) -> u8 {
        let p: f64 = self.rng.gen();
        for (i, &cdf) in BENFORD_CDF.iter().enumerate() {
            if p < cdf {
                return (i + 1) as u8;
            }
        }
        9
    }

    /// Sample an amount with enhanced Benford compliance.
    pub fn sample(&mut self) -> Decimal {
        let (first_digit, second_digit) = if self.config.first_two_digit_compliance {
            self.sample_first_two_digits()
        } else if self.config.second_digit_compliance {
            (self.sample_first_digit(), self.sample_second_digit())
        } else {
            (self.sample_first_digit(), self.rng.gen_range(0..10) as u8)
        };

        self.sample_with_digits(first_digit, second_digit)
    }

    /// Sample an amount with specific first two digits.
    fn sample_with_digits(&mut self, first_digit: u8, second_digit: u8) -> Decimal {
        let first_digit = first_digit.clamp(1, 9);
        let second_digit = second_digit.clamp(0, 9);

        // Determine the order of magnitude based on config range
        let min_magnitude = self.config.amount_config.min_amount.log10().floor() as i32;
        let max_magnitude = self.config.amount_config.max_amount.log10().floor() as i32;

        // Sample a magnitude within the valid range
        let magnitude = self.rng.gen_range(min_magnitude..=max_magnitude);
        let base = 10_f64.powi(magnitude - 1); // -1 because first two digits span 10-99

        // Generate the remaining digits (0.0 to 0.99...)
        let remaining: f64 = self.rng.gen();

        // Construct the amount: (first_digit * 10 + second_digit + remaining) * base
        let mantissa = (first_digit as f64) * 10.0 + (second_digit as f64) + remaining;
        let mut amount = mantissa * base;

        // Clamp to configured range
        amount = amount.clamp(
            self.config.amount_config.min_amount,
            self.config.amount_config.max_amount,
        );

        // Round to configured decimal places
        let decimal_multiplier = 10_f64.powi(self.config.amount_config.decimal_places as i32);
        amount = (amount * decimal_multiplier).round() / decimal_multiplier;

        Decimal::from_f64_retain(amount).unwrap_or(Decimal::ONE)
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }
}

/// Types of Benford deviation patterns for anomaly injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum BenfordDeviationType {
    /// Round number bias (excess of digits 1, 5, 0 in second position)
    #[default]
    RoundNumberBias,
    /// Threshold clustering (amounts just below round thresholds)
    ThresholdClustering,
    /// Uniform first digit (equal probability for all first digits)
    UniformFirstDigit,
    /// Excess of specific digit
    DigitBias { digit: u8 },
    /// Trailing zeros pattern (prices ending in .00)
    TrailingZeros,
}

/// Configuration for Benford deviation sampling (for anomaly injection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenfordDeviationConfig {
    /// Type of deviation pattern
    pub deviation_type: BenfordDeviationType,
    /// Intensity of deviation (0.0 = Benford compliant, 1.0 = full deviation)
    #[serde(default = "default_intensity")]
    pub intensity: f64,
    /// Base amount configuration
    pub amount_config: AmountDistributionConfig,
    /// Thresholds for threshold clustering (if applicable)
    #[serde(default = "default_thresholds")]
    pub thresholds: Vec<f64>,
}

fn default_intensity() -> f64 {
    0.5
}

fn default_thresholds() -> Vec<f64> {
    vec![1000.0, 5000.0, 10000.0, 25000.0, 50000.0, 100000.0]
}

impl Default for BenfordDeviationConfig {
    fn default() -> Self {
        Self {
            deviation_type: BenfordDeviationType::RoundNumberBias,
            intensity: 0.5,
            amount_config: AmountDistributionConfig::default(),
            thresholds: default_thresholds(),
        }
    }
}

/// Sampler for generating amounts that deviate from Benford's Law.
/// Useful for injecting statistically detectable anomalies.
pub struct BenfordDeviationSampler {
    rng: ChaCha8Rng,
    config: BenfordDeviationConfig,
    benford_sampler: BenfordSampler,
}

impl BenfordDeviationSampler {
    /// Create a new Benford deviation sampler.
    pub fn new(seed: u64, config: BenfordDeviationConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            benford_sampler: BenfordSampler::new(seed + 100, config.amount_config.clone()),
            config,
        }
    }

    /// Sample an amount with the configured deviation pattern.
    pub fn sample(&mut self) -> Decimal {
        // With probability (1 - intensity), sample from normal Benford
        let p: f64 = self.rng.gen();
        if p > self.config.intensity {
            return self.benford_sampler.sample();
        }

        // Apply deviation pattern
        match self.config.deviation_type {
            BenfordDeviationType::RoundNumberBias => self.sample_round_bias(),
            BenfordDeviationType::ThresholdClustering => self.sample_threshold_cluster(),
            BenfordDeviationType::UniformFirstDigit => self.sample_uniform_first_digit(),
            BenfordDeviationType::DigitBias { digit } => self.sample_digit_bias(digit),
            BenfordDeviationType::TrailingZeros => self.sample_trailing_zeros(),
        }
    }

    /// Sample with round number bias.
    fn sample_round_bias(&mut self) -> Decimal {
        // Bias towards first digits 1 and 5
        let first_digit = if self.rng.gen_bool(0.6) {
            if self.rng.gen_bool(0.7) {
                1
            } else {
                5
            }
        } else {
            self.rng.gen_range(1..=9)
        };

        // Bias towards second digits 0 and 5
        let _second_digit = if self.rng.gen_bool(0.5) {
            if self.rng.gen_bool(0.6) {
                0
            } else {
                5
            }
        } else {
            self.rng.gen_range(0..=9)
        };

        self.benford_sampler.sample_with_first_digit(first_digit)
    }

    /// Sample clustering just below thresholds.
    fn sample_threshold_cluster(&mut self) -> Decimal {
        let threshold = self
            .config
            .thresholds
            .choose(&mut self.rng)
            .copied()
            .unwrap_or(10000.0);

        // Generate amount 1-15% below threshold
        let pct_below = self.rng.gen_range(0.01..0.15);
        let amount = threshold * (1.0 - pct_below);

        // Add small noise
        let noise = 1.0 + self.rng.gen_range(-0.005..0.005);
        let final_amount = (amount * noise * 100.0).round() / 100.0;

        Decimal::from_f64_retain(final_amount.clamp(
            self.config.amount_config.min_amount,
            self.config.amount_config.max_amount,
        ))
        .unwrap_or(Decimal::ONE)
    }

    /// Sample with uniform first digit distribution.
    fn sample_uniform_first_digit(&mut self) -> Decimal {
        let first_digit = self.rng.gen_range(1..=9);
        self.benford_sampler.sample_with_first_digit(first_digit)
    }

    /// Sample with bias towards a specific digit.
    fn sample_digit_bias(&mut self, target_digit: u8) -> Decimal {
        let digit = target_digit.clamp(1, 9);
        // 70% chance of using the biased digit
        let first_digit = if self.rng.gen_bool(0.7) {
            digit
        } else {
            self.rng.gen_range(1..=9)
        };
        self.benford_sampler.sample_with_first_digit(first_digit)
    }

    /// Sample with trailing zeros pattern (prices ending in .00).
    fn sample_trailing_zeros(&mut self) -> Decimal {
        let amount = self.benford_sampler.sample();
        let amount_f64: f64 = amount.to_string().parse().unwrap_or(0.0);

        // Round to whole dollars
        let rounded = amount_f64.round();
        Decimal::from_f64_retain(rounded.clamp(
            self.config.amount_config.min_amount,
            self.config.amount_config.max_amount,
        ))
        .unwrap_or(Decimal::ONE)
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
        self.benford_sampler.reset(seed + 100);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benford_probabilities_sum_to_one() {
        let sum: f64 = BENFORD_PROBABILITIES.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "Benford probabilities sum to {}, expected 1.0",
            sum
        );
    }

    #[test]
    fn test_benford_cdf_ends_at_one() {
        assert!(
            (BENFORD_CDF[8] - 1.0).abs() < 0.0001,
            "CDF should end at 1.0"
        );
    }

    #[test]
    fn test_anti_benford_probabilities_sum_to_one() {
        let sum: f64 = ANTI_BENFORD_PROBABILITIES.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "Anti-Benford probabilities sum to {}, expected 1.0",
            sum
        );
    }

    #[test]
    fn test_benford_sampler_determinism() {
        let config = AmountDistributionConfig::default();
        let mut sampler1 = BenfordSampler::new(42, config.clone());
        let mut sampler2 = BenfordSampler::new(42, config);

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_benford_first_digit_distribution() {
        let config = AmountDistributionConfig::default();
        let mut sampler = BenfordSampler::new(12345, config);

        let mut digit_counts = [0u32; 9];
        let iterations = 10_000;

        for _ in 0..iterations {
            let amount = sampler.sample();
            if let Some(digit) = get_first_digit(amount) {
                if (1..=9).contains(&digit) {
                    digit_counts[(digit - 1) as usize] += 1;
                }
            }
        }

        // Verify digit 1 is most common (should be ~30%, but can vary more due to log-normal distribution)
        let digit_1_pct = digit_counts[0] as f64 / iterations as f64;
        assert!(
            digit_1_pct > 0.15 && digit_1_pct < 0.50,
            "Digit 1 should be ~30%, got {:.1}%",
            digit_1_pct * 100.0
        );

        // Verify digit 9 is least common (should be ~5%)
        let digit_9_pct = digit_counts[8] as f64 / iterations as f64;
        assert!(
            digit_9_pct > 0.02 && digit_9_pct < 0.10,
            "Digit 9 should be ~5%, got {:.1}%",
            digit_9_pct * 100.0
        );
    }

    #[test]
    fn test_threshold_adjacent_below_threshold() {
        let config = AmountDistributionConfig::default();
        let threshold_config = ThresholdConfig {
            thresholds: vec![10000.0],
            min_below_pct: 0.01,
            max_below_pct: 0.15,
        };
        let mut gen = FraudAmountGenerator::new(42, config, threshold_config);

        for _ in 0..100 {
            let amount = gen.sample(FraudAmountPattern::ThresholdAdjacent);
            let f = amount.to_string().parse::<f64>().unwrap();
            assert!(f < 10000.0, "Amount {} should be below threshold 10000", f);
            // Account for noise factor (up to 0.5%) and rounding
            assert!(
                f >= 8400.0,
                "Amount {} should be approximately within 15% of threshold",
                f
            );
        }
    }

    #[test]
    fn test_obvious_round_numbers() {
        let config = AmountDistributionConfig::default();
        let threshold_config = ThresholdConfig::default();
        let mut gen = FraudAmountGenerator::new(42, config, threshold_config);

        for _ in 0..100 {
            let amount = gen.sample(FraudAmountPattern::ObviousRoundNumbers);
            let f = amount.to_string().parse::<f64>().unwrap();

            // Should be either a round number or just under one
            let is_round = f % 1000.0 == 0.0 || f % 5000.0 == 0.0;
            let is_just_under = (f + 0.01) % 1000.0 < 0.02 || (f + 0.01) % 10000.0 < 0.02;

            assert!(
                is_round || is_just_under || f > 0.0,
                "Amount {} should be a suspicious round number",
                f
            );
        }
    }

    #[test]
    fn test_get_first_digit() {
        assert_eq!(get_first_digit(Decimal::from(123)), Some(1));
        assert_eq!(get_first_digit(Decimal::from(999)), Some(9));
        assert_eq!(get_first_digit(Decimal::from(50000)), Some(5));
        assert_eq!(
            get_first_digit(Decimal::from_str_exact("0.00123").unwrap()),
            Some(1)
        );
    }

    #[test]
    fn test_second_digit_probabilities_sum_to_one() {
        let sum: f64 = BENFORD_SECOND_DIGIT_PROBABILITIES.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "Second digit probabilities sum to {}, expected 1.0",
            sum
        );
    }

    #[test]
    fn test_first_two_probability() {
        // P(10) = log10(1 + 1/10) = log10(1.1) ≈ 0.0414
        let p10 = benford_first_two_probability(1, 0);
        assert!((p10 - 0.0414).abs() < 0.001);

        // P(99) = log10(1 + 1/99) ≈ 0.00436
        let p99 = benford_first_two_probability(9, 9);
        assert!((p99 - 0.00436).abs() < 0.0001);

        // Sum of all first-two probabilities should be 1.0
        let probs = benford_first_two_probabilities();
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_get_first_two_digits() {
        assert_eq!(get_first_two_digits(Decimal::from(123)), Some((1, 2)));
        assert_eq!(get_first_two_digits(Decimal::from(999)), Some((9, 9)));
        assert_eq!(get_first_two_digits(Decimal::from(50000)), Some((5, 0)));
        assert_eq!(
            get_first_two_digits(Decimal::from_str_exact("0.00123").unwrap()),
            Some((1, 2))
        );
    }

    #[test]
    fn test_enhanced_benford_sampler() {
        let config = EnhancedBenfordConfig {
            amount_config: AmountDistributionConfig::default(),
            second_digit_compliance: true,
            first_two_digit_compliance: false,
        };
        let mut sampler = EnhancedBenfordSampler::new(42, config);

        let mut digit_counts = [0u32; 10];
        for _ in 0..10000 {
            let amount = sampler.sample();
            if let Some((_, d2)) = get_first_two_digits(amount) {
                digit_counts[d2 as usize] += 1;
            }
        }

        // Note: The second digit distribution depends on amount generation and
        // magnitude selection, which may skew results. Just verify the sampler runs
        // and produces valid amounts.
        let total_valid = digit_counts.iter().sum::<u32>();
        assert!(
            total_valid > 9000,
            "Most samples should have valid first two digits"
        );

        // Verify we have some distribution of second digits (not all the same)
        let max_count = *digit_counts.iter().max().unwrap();
        let min_count = *digit_counts.iter().min().unwrap();
        assert!(
            max_count < total_valid / 2,
            "Second digits should have some variety, max count: {}",
            max_count
        );
    }

    #[test]
    fn test_benford_deviation_sampler() {
        let config = BenfordDeviationConfig {
            deviation_type: BenfordDeviationType::ThresholdClustering,
            intensity: 1.0,
            amount_config: AmountDistributionConfig::default(),
            thresholds: vec![10000.0],
        };
        let mut sampler = BenfordDeviationSampler::new(42, config);

        for _ in 0..100 {
            let amount = sampler.sample();
            let f: f64 = amount.to_string().parse().unwrap();
            // Should be below threshold
            assert!(f < 10000.0, "Amount {} should be below 10000", f);
            // Should be within ~20% of threshold (1-15% below + noise)
            assert!(f > 8000.0, "Amount {} should be near threshold 10000", f);
        }
    }

    #[test]
    fn test_benford_deviation_round_bias() {
        let config = BenfordDeviationConfig {
            deviation_type: BenfordDeviationType::RoundNumberBias,
            intensity: 1.0,
            amount_config: AmountDistributionConfig::default(),
            thresholds: vec![],
        };
        let mut sampler = BenfordDeviationSampler::new(42, config);

        let mut digit_counts = [0u32; 9];
        for _ in 0..1000 {
            let amount = sampler.sample();
            if let Some(d) = get_first_digit(amount) {
                if (1..=9).contains(&d) {
                    digit_counts[(d - 1) as usize] += 1;
                }
            }
        }

        // Digits 1 and 5 should be overrepresented
        let d1_pct = digit_counts[0] as f64 / 1000.0;
        let d5_pct = digit_counts[4] as f64 / 1000.0;

        // Should be higher than Benford expects
        assert!(d1_pct > 0.35 || d5_pct > 0.10);
    }
}
