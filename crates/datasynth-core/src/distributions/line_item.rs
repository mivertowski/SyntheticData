//! Line item count distribution sampler.
//!
//! Implements the empirical distribution of journal entry line items
//! as observed in the accounting network generation research.
//!
//! Key findings from the paper:
//! - 60.68% of journal entries have exactly 2 line items
//! - 16.63% have 4 line items
//! - 88% have an even number of line items
//! - 82% have equal debit and credit line counts

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Configuration for line item count distribution.
///
/// Based on empirical findings from Table III of the accounting network paper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemDistributionConfig {
    /// Probability of 2 line items (60.68%)
    pub two_items: f64,
    /// Probability of 3 line items (5.77%)
    pub three_items: f64,
    /// Probability of 4 line items (16.63%)
    pub four_items: f64,
    /// Probability of 5 line items (3.06%)
    pub five_items: f64,
    /// Probability of 6 line items (3.32%)
    pub six_items: f64,
    /// Probability of 7 line items (1.13%)
    pub seven_items: f64,
    /// Probability of 8 line items (1.88%)
    pub eight_items: f64,
    /// Probability of 9 line items (0.42%)
    pub nine_items: f64,
    /// Probability of 10-99 line items (6.33%)
    pub ten_to_ninety_nine: f64,
    /// Probability of 100-999 line items (0.76%)
    pub hundred_to_nine_ninety_nine: f64,
    /// Probability of 1000+ line items (0.02%)
    pub thousand_plus: f64,
}

impl Default for LineItemDistributionConfig {
    fn default() -> Self {
        // Values from Table III of the paper
        Self {
            two_items: 0.6068,
            three_items: 0.0577,
            four_items: 0.1663,
            five_items: 0.0306,
            six_items: 0.0332,
            seven_items: 0.0113,
            eight_items: 0.0188,
            nine_items: 0.0042,
            ten_to_ninety_nine: 0.0633,
            hundred_to_nine_ninety_nine: 0.0076,
            thousand_plus: 0.0002,
        }
    }
}

impl LineItemDistributionConfig {
    /// Validate that probabilities sum to approximately 1.0.
    pub fn validate(&self) -> Result<(), String> {
        let sum = self.two_items
            + self.three_items
            + self.four_items
            + self.five_items
            + self.six_items
            + self.seven_items
            + self.eight_items
            + self.nine_items
            + self.ten_to_ninety_nine
            + self.hundred_to_nine_ninety_nine
            + self.thousand_plus;

        if (sum - 1.0).abs() > 0.01 {
            return Err(format!(
                "Line item distribution probabilities sum to {}, expected ~1.0",
                sum
            ));
        }
        Ok(())
    }

    /// Get cumulative distribution values.
    fn cumulative(&self) -> [f64; 11] {
        let mut cum = [0.0; 11];
        cum[0] = self.two_items;
        cum[1] = cum[0] + self.three_items;
        cum[2] = cum[1] + self.four_items;
        cum[3] = cum[2] + self.five_items;
        cum[4] = cum[3] + self.six_items;
        cum[5] = cum[4] + self.seven_items;
        cum[6] = cum[5] + self.eight_items;
        cum[7] = cum[6] + self.nine_items;
        cum[8] = cum[7] + self.ten_to_ninety_nine;
        cum[9] = cum[8] + self.hundred_to_nine_ninety_nine;
        cum[10] = cum[9] + self.thousand_plus;
        cum
    }
}

/// Configuration for even/odd line count distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvenOddDistributionConfig {
    /// Probability of even line count (88%)
    pub even: f64,
    /// Probability of odd line count (12%)
    pub odd: f64,
}

impl Default for EvenOddDistributionConfig {
    fn default() -> Self {
        // From the paper: 88% even, 12% odd
        Self {
            even: 0.88,
            odd: 0.12,
        }
    }
}

/// Configuration for debit/credit balance distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebitCreditDistributionConfig {
    /// Probability of equal debit and credit counts (82%)
    pub equal: f64,
    /// Probability of more debit lines than credit (7%)
    pub more_debit: f64,
    /// Probability of more credit lines than debit (11%)
    pub more_credit: f64,
}

impl Default for DebitCreditDistributionConfig {
    fn default() -> Self {
        // From the paper: 82% equal, 11% more credit, 7% more debit
        Self {
            equal: 0.82,
            more_debit: 0.07,
            more_credit: 0.11,
        }
    }
}

/// Sampler for journal entry line item counts.
///
/// Produces realistic line item counts based on empirical distributions
/// from real-world general ledger data.
pub struct LineItemSampler {
    /// RNG for sampling
    rng: ChaCha8Rng,
    /// Even/odd distribution config
    even_odd_config: EvenOddDistributionConfig,
    /// Debit/credit distribution config
    debit_credit_config: DebitCreditDistributionConfig,
    /// Cumulative distribution for line counts
    cumulative: [f64; 11],
}

impl LineItemSampler {
    /// Create a new sampler with default configuration.
    pub fn new(seed: u64) -> Self {
        let line_config = LineItemDistributionConfig::default();
        let cumulative = line_config.cumulative();

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            even_odd_config: EvenOddDistributionConfig::default(),
            debit_credit_config: DebitCreditDistributionConfig::default(),
            cumulative,
        }
    }

    /// Create a sampler with custom configuration.
    pub fn with_config(
        seed: u64,
        line_config: LineItemDistributionConfig,
        even_odd_config: EvenOddDistributionConfig,
        debit_credit_config: DebitCreditDistributionConfig,
    ) -> Self {
        let cumulative = line_config.cumulative();

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            even_odd_config,
            debit_credit_config,
            cumulative,
        }
    }

    /// Sample a line item count.
    pub fn sample_count(&mut self) -> usize {
        let p: f64 = self.rng.random();

        // Find the bin using cumulative distribution
        if p < self.cumulative[0] {
            2
        } else if p < self.cumulative[1] {
            3
        } else if p < self.cumulative[2] {
            4
        } else if p < self.cumulative[3] {
            5
        } else if p < self.cumulative[4] {
            6
        } else if p < self.cumulative[5] {
            7
        } else if p < self.cumulative[6] {
            8
        } else if p < self.cumulative[7] {
            9
        } else if p < self.cumulative[8] {
            // 10-99 range - use uniform distribution within range
            self.rng.random_range(10..100)
        } else if p < self.cumulative[9] {
            // 100-999 range
            self.rng.random_range(100..1000)
        } else {
            // 1000+ range (cap at 10000 for practicality)
            self.rng.random_range(1000..10000)
        }
    }

    /// Sample whether the count should be even.
    pub fn sample_even(&mut self) -> bool {
        self.rng.random::<f64>() < self.even_odd_config.even
    }

    /// Sample a line item count with even/odd constraint.
    ///
    /// When adjustment is needed, randomly chooses to increment or decrement
    /// to avoid biasing toward lower counts.
    pub fn sample_count_with_parity(&mut self) -> usize {
        let base_count = self.sample_count();
        let should_be_even = self.sample_even();

        // Adjust to match parity requirement
        let is_even = base_count.is_multiple_of(2);
        if should_be_even != is_even {
            // Use symmetric adjustment: randomly increment or decrement
            if base_count <= 2 {
                // Can only increment for small counts
                base_count + 1
            } else if self.rng.random::<bool>() {
                // Randomly choose to increment
                base_count + 1
            } else {
                // Randomly choose to decrement
                base_count - 1
            }
        } else {
            base_count
        }
    }

    /// Sample the debit/credit split type.
    pub fn sample_debit_credit_type(&mut self) -> DebitCreditSplit {
        let p: f64 = self.rng.random();

        if p < self.debit_credit_config.equal {
            DebitCreditSplit::Equal
        } else if p < self.debit_credit_config.equal + self.debit_credit_config.more_debit {
            DebitCreditSplit::MoreDebit
        } else {
            DebitCreditSplit::MoreCredit
        }
    }

    /// Sample a complete line item specification.
    pub fn sample(&mut self) -> LineItemSpec {
        let total_count = self.sample_count_with_parity();
        let split_type = self.sample_debit_credit_type();

        let (debit_count, credit_count) = match split_type {
            DebitCreditSplit::Equal => {
                let half = total_count / 2;
                (half, total_count - half)
            }
            DebitCreditSplit::MoreDebit => {
                // More debit lines - 60% debit, 40% credit
                let debit = (total_count as f64 * 0.6).round() as usize;
                let debit = debit.max(1).min(total_count - 1);
                (debit, total_count - debit)
            }
            DebitCreditSplit::MoreCredit => {
                // More credit lines - 40% debit, 60% credit
                let credit = (total_count as f64 * 0.6).round() as usize;
                let credit = credit.max(1).min(total_count - 1);
                (total_count - credit, credit)
            }
        };

        LineItemSpec {
            total_count,
            debit_count,
            credit_count,
            split_type,
        }
    }

    /// Reset the sampler with the same seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }
}

/// Type of debit/credit split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebitCreditSplit {
    /// Equal number of debit and credit lines
    Equal,
    /// More debit lines than credit
    MoreDebit,
    /// More credit lines than debit
    MoreCredit,
}

/// Specification for line items in a journal entry.
#[derive(Debug, Clone)]
pub struct LineItemSpec {
    /// Total number of line items
    pub total_count: usize,
    /// Number of debit lines
    pub debit_count: usize,
    /// Number of credit lines
    pub credit_count: usize,
    /// Type of debit/credit split
    pub split_type: DebitCreditSplit,
}

impl LineItemSpec {
    /// Check if the spec is valid.
    pub fn is_valid(&self) -> bool {
        self.total_count >= 2
            && self.debit_count >= 1
            && self.credit_count >= 1
            && self.debit_count + self.credit_count == self.total_count
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_valid() {
        let config = LineItemDistributionConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_sampler_determinism() {
        let mut sampler1 = LineItemSampler::new(42);
        let mut sampler2 = LineItemSampler::new(42);

        for _ in 0..100 {
            assert_eq!(sampler1.sample_count(), sampler2.sample_count());
        }
    }

    #[test]
    fn test_sampler_distribution() {
        let mut sampler = LineItemSampler::new(42);
        let sample_size = 100_000;

        let mut counts = std::collections::HashMap::new();
        for _ in 0..sample_size {
            let count = sampler.sample_count();
            *counts.entry(count).or_insert(0) += 1;
        }

        // Check that 2-line items are most common
        let two_count = *counts.get(&2).unwrap_or(&0) as f64 / sample_size as f64;
        assert!(
            two_count > 0.55 && two_count < 0.65,
            "Expected ~60% 2-item entries, got {}%",
            two_count * 100.0
        );

        // Check that 4-line items are second most common
        let four_count = *counts.get(&4).unwrap_or(&0) as f64 / sample_size as f64;
        assert!(
            four_count > 0.13 && four_count < 0.20,
            "Expected ~16% 4-item entries, got {}%",
            four_count * 100.0
        );
    }

    #[test]
    fn test_line_item_spec_valid() {
        let mut sampler = LineItemSampler::new(42);

        for _ in 0..1000 {
            let spec = sampler.sample();
            assert!(spec.is_valid(), "Invalid spec: {:?}", spec);
        }
    }
}
