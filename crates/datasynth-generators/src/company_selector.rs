//! Weighted company selection for transaction generation.
//!
//! Implements probability-weighted company selection based on
//! volume_weight configuration to produce realistic transaction
//! distributions across company codes.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use datasynth_config::schema::CompanyConfig;

/// Weighted company selector using cumulative distribution function.
///
/// Selects companies with probability proportional to their volume_weight.
/// For example, companies with weights [1.0, 0.5, 0.5] will be selected
/// with probabilities [50%, 25%, 25%].
#[derive(Clone)]
pub struct WeightedCompanySelector {
    /// Company codes in selection order.
    company_codes: Vec<String>,
    /// Cumulative distribution function for selection.
    cumulative_weights: Vec<f64>,
    /// Total weight (for statistics).
    total_weight: f64,
}

impl WeightedCompanySelector {
    /// Create a new weighted company selector from company configurations.
    pub fn from_configs(configs: &[CompanyConfig]) -> Self {
        if configs.is_empty() {
            return Self {
                company_codes: vec!["1000".to_string()],
                cumulative_weights: vec![1.0],
                total_weight: 1.0,
            };
        }

        let company_codes: Vec<String> = configs.iter().map(|c| c.code.clone()).collect();
        let weights: Vec<f64> = configs.iter().map(|c| c.volume_weight).collect();

        Self::from_codes_and_weights(company_codes, weights)
    }

    /// Create a selector from company codes and explicit weights.
    pub fn from_codes_and_weights(codes: Vec<String>, weights: Vec<f64>) -> Self {
        if codes.is_empty() {
            return Self {
                company_codes: vec!["1000".to_string()],
                cumulative_weights: vec![1.0],
                total_weight: 1.0,
            };
        }

        // Ensure weights has same length as codes
        let weights: Vec<f64> = if weights.len() != codes.len() {
            vec![1.0; codes.len()]
        } else {
            weights
        };

        // Calculate total weight; fall back to uniform if all weights are zero
        let total_weight: f64 = weights.iter().sum();
        let effective_weights: Vec<f64> = if total_weight == 0.0 {
            vec![1.0; codes.len()]
        } else {
            weights
        };
        let effective_total: f64 = effective_weights.iter().sum();

        // Build cumulative distribution function
        let mut cumulative = Vec::with_capacity(codes.len());
        let mut running_sum = 0.0;
        for weight in &effective_weights {
            running_sum += weight / effective_total;
            cumulative.push(running_sum);
        }

        // Ensure last value is exactly 1.0 to avoid floating point issues
        if let Some(last) = cumulative.last_mut() {
            *last = 1.0;
        }

        Self {
            company_codes: codes,
            cumulative_weights: cumulative,
            total_weight: effective_total,
        }
    }

    /// Create a selector with uniform weights.
    pub fn uniform(codes: Vec<String>) -> Self {
        let weights = vec![1.0; codes.len()];
        Self::from_codes_and_weights(codes, weights)
    }

    /// Select a company code using the weighted distribution.
    ///
    /// Uses binary search over the pre-computed CDF for O(log n) selection
    /// instead of O(n) linear scan.
    #[inline]
    pub fn select(&self, rng: &mut ChaCha8Rng) -> &str {
        let p: f64 = rng.random();

        // Binary search: find the first index where cumulative_weight >= p
        let idx = self.cumulative_weights.partition_point(|&w| w < p);

        if idx < self.company_codes.len() {
            &self.company_codes[idx]
        } else {
            // Fallback to last company (should rarely happen due to floating point)
            self.company_codes
                .last()
                .map(|s| s.as_str())
                .unwrap_or("1000")
        }
    }

    /// Get the probability of selecting a specific company.
    pub fn probability(&self, company_code: &str) -> f64 {
        let idx = self.company_codes.iter().position(|c| c == company_code);

        match idx {
            Some(0) => self.cumulative_weights[0],
            Some(i) => self.cumulative_weights[i] - self.cumulative_weights[i - 1],
            None => 0.0,
        }
    }

    /// Get all company codes.
    pub fn company_codes(&self) -> &[String] {
        &self.company_codes
    }

    /// Get the total weight (for statistics).
    pub fn total_weight(&self) -> f64 {
        self.total_weight
    }

    /// Get the number of companies.
    pub fn len(&self) -> usize {
        self.company_codes.len()
    }

    /// Check if the selector is empty.
    pub fn is_empty(&self) -> bool {
        self.company_codes.is_empty()
    }
}

impl Default for WeightedCompanySelector {
    fn default() -> Self {
        Self {
            company_codes: vec!["1000".to_string()],
            cumulative_weights: vec![1.0],
            total_weight: 1.0,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_selection() {
        let selector = WeightedCompanySelector::uniform(vec![
            "1000".to_string(),
            "2000".to_string(),
            "3000".to_string(),
        ]);

        // Each should have ~33% probability
        let prob = selector.probability("1000");
        assert!((prob - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_weighted_selection() {
        let codes = vec!["1000".to_string(), "2000".to_string(), "3000".to_string()];
        let weights = vec![1.0, 0.5, 0.5];

        let selector = WeightedCompanySelector::from_codes_and_weights(codes, weights);

        // Company 1000 should have 50% probability
        assert!((selector.probability("1000") - 0.5).abs() < 0.01);

        // Companies 2000 and 3000 should each have 25% probability
        assert!((selector.probability("2000") - 0.25).abs() < 0.01);
        assert!((selector.probability("3000") - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_selection_distribution() {
        let codes = vec!["1000".to_string(), "2000".to_string()];
        let weights = vec![3.0, 1.0]; // 75% / 25%

        let selector = WeightedCompanySelector::from_codes_and_weights(codes, weights);

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut count_1000 = 0;
        let mut count_2000 = 0;

        for _ in 0..10000 {
            match selector.select(&mut rng) {
                "1000" => count_1000 += 1,
                "2000" => count_2000 += 1,
                _ => {}
            }
        }

        // Check distribution is approximately correct
        let ratio = count_1000 as f64 / (count_1000 + count_2000) as f64;
        assert!((ratio - 0.75).abs() < 0.02);
    }

    #[test]
    fn test_from_configs() {
        let configs = vec![
            CompanyConfig {
                code: "1000".to_string(),
                name: "US HQ".to_string(),
                volume_weight: 1.0,
                country: "US".to_string(),
                currency: "USD".to_string(),
                fiscal_year_variant: "K4".to_string(),
                annual_transaction_volume: datasynth_config::TransactionVolume::HundredK,
            },
            CompanyConfig {
                code: "2000".to_string(),
                name: "EU Sub".to_string(),
                volume_weight: 0.5,
                country: "DE".to_string(),
                currency: "EUR".to_string(),
                fiscal_year_variant: "K4".to_string(),
                annual_transaction_volume: datasynth_config::TransactionVolume::HundredK,
            },
        ];

        let selector = WeightedCompanySelector::from_configs(&configs);

        // 1000 should have ~66.7% probability
        assert!((selector.probability("1000") - 0.667).abs() < 0.01);

        // 2000 should have ~33.3% probability
        assert!((selector.probability("2000") - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_empty_configs() {
        let selector = WeightedCompanySelector::from_configs(&[]);

        assert_eq!(selector.len(), 1);
        assert_eq!(selector.company_codes()[0], "1000");
    }

    #[test]
    fn test_single_company() {
        let codes = vec!["5000".to_string()];
        let weights = vec![1.0];

        let selector = WeightedCompanySelector::from_codes_and_weights(codes, weights);

        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Should always return the same company
        for _ in 0..100 {
            assert_eq!(selector.select(&mut rng), "5000");
        }
    }

    #[test]
    fn test_deterministic_selection() {
        let codes = vec!["1000".to_string(), "2000".to_string(), "3000".to_string()];
        let weights = vec![1.0, 1.0, 1.0];

        let selector = WeightedCompanySelector::from_codes_and_weights(codes, weights);

        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        // Same seed should produce same selection sequence
        for _ in 0..100 {
            assert_eq!(selector.select(&mut rng1), selector.select(&mut rng2));
        }
    }
}
