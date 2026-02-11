//! Linkage Attack Assessment.
//!
//! Evaluates the risk of re-identification by checking if synthetic records
//! can be uniquely linked back to original records using quasi-identifiers (QIs).
//!
//! A linkage attack selects a subset of fields as quasi-identifiers and checks
//! how many synthetic records uniquely match a single original record on those QIs.
//! Low re-identification rates and high k-anonymity indicate good privacy.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for linkage attack evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkageConfig {
    /// Maximum acceptable re-identification rate (0.0 - 1.0).
    /// Default: 0.05 (5%).
    pub max_reidentification_rate: f64,
    /// Minimum k-anonymity level to achieve.
    /// Default: 5 (each record matches at least 5 others).
    pub min_k_anonymity: usize,
}

impl Default for LinkageConfig {
    fn default() -> Self {
        Self {
            max_reidentification_rate: 0.05,
            min_k_anonymity: 5,
        }
    }
}

/// Results from a linkage attack evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkageResults {
    /// Fraction of synthetic records that uniquely match a single original record.
    pub re_identification_rate: f64,
    /// The effective k-anonymity achieved (minimum group size across all QI combinations).
    pub k_anonymity_achieved: usize,
    /// Number of unique QI combinations in the original data.
    pub unique_qi_combos_original: usize,
    /// Number of unique QI combinations in the synthetic data.
    pub unique_qi_combos_synthetic: usize,
    /// Number of QI combinations that appear in both datasets.
    pub overlapping_combos: usize,
    /// Number of synthetic records that could be uniquely linked.
    pub uniquely_linked: usize,
    /// Total synthetic records evaluated.
    pub total_synthetic: usize,
    /// Whether privacy passes the configured thresholds.
    pub passes: bool,
}

/// Linkage attack evaluator.
///
/// Checks if synthetic records can be uniquely re-identified in the original
/// dataset based on quasi-identifier fields.
pub struct LinkageAttack {
    config: LinkageConfig,
}

impl LinkageAttack {
    /// Create a new linkage attack evaluator.
    pub fn new(config: LinkageConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(LinkageConfig::default())
    }

    /// Run the linkage attack evaluation.
    ///
    /// # Arguments
    /// * `original_qis` - Quasi-identifier tuples from the original dataset.
    ///   Each entry is a record represented as a Vec of string-valued QI fields.
    /// * `synthetic_qis` - Quasi-identifier tuples from the synthetic dataset.
    ///
    /// # Example
    /// ```ignore
    /// // Each record has QI fields: [age_bucket, zip_prefix, gender]
    /// let original = vec![
    ///     vec!["30-39".into(), "100".into(), "M".into()],
    ///     vec!["40-49".into(), "200".into(), "F".into()],
    /// ];
    /// let synthetic = vec![
    ///     vec!["30-39".into(), "100".into(), "M".into()],
    /// ];
    /// let results = attack.evaluate(&original, &synthetic);
    /// ```
    pub fn evaluate(
        &self,
        original_qis: &[Vec<String>],
        synthetic_qis: &[Vec<String>],
    ) -> LinkageResults {
        if original_qis.is_empty() || synthetic_qis.is_empty() {
            return LinkageResults {
                re_identification_rate: 0.0,
                k_anonymity_achieved: usize::MAX,
                unique_qi_combos_original: 0,
                unique_qi_combos_synthetic: 0,
                overlapping_combos: 0,
                uniquely_linked: 0,
                total_synthetic: synthetic_qis.len(),
                passes: true,
            };
        }

        // Build frequency map for original data QI combinations
        let mut original_freq: HashMap<Vec<String>, usize> = HashMap::new();
        for qi in original_qis {
            *original_freq.entry(qi.clone()).or_insert(0) += 1;
        }

        // Build frequency map for synthetic data QI combinations
        let mut synthetic_freq: HashMap<Vec<String>, usize> = HashMap::new();
        for qi in synthetic_qis {
            *synthetic_freq.entry(qi.clone()).or_insert(0) += 1;
        }

        // Count overlapping QI combinations
        let overlapping_combos = synthetic_freq
            .keys()
            .filter(|qi| original_freq.contains_key(*qi))
            .count();

        // Count uniquely linked records:
        // A synthetic record is "uniquely linked" if its QI combination maps to
        // exactly 1 record in the original dataset
        let mut uniquely_linked = 0usize;
        for qi in synthetic_qis {
            if let Some(&orig_count) = original_freq.get(qi) {
                if orig_count == 1 {
                    uniquely_linked += 1;
                }
            }
        }

        let re_identification_rate = if synthetic_qis.is_empty() {
            0.0
        } else {
            uniquely_linked as f64 / synthetic_qis.len() as f64
        };

        // k-anonymity: minimum group size across all QI combos present in original data
        let k_anonymity_achieved = original_freq.values().copied().min().unwrap_or(0);

        let passes = re_identification_rate <= self.config.max_reidentification_rate
            && k_anonymity_achieved >= self.config.min_k_anonymity;

        LinkageResults {
            re_identification_rate,
            k_anonymity_achieved,
            unique_qi_combos_original: original_freq.len(),
            unique_qi_combos_synthetic: synthetic_freq.len(),
            overlapping_combos,
            uniquely_linked,
            total_synthetic: synthetic_qis.len(),
            passes,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_qi(fields: &[&str]) -> Vec<String> {
        fields.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_k_anonymized_data_low_reidentification() {
        // Each QI combo appears at least 5 times in original
        let mut original = Vec::new();
        for _ in 0..5 {
            original.push(make_qi(&["30-39", "100", "M"]));
            original.push(make_qi(&["40-49", "200", "F"]));
            original.push(make_qi(&["50-59", "300", "M"]));
        }

        let synthetic = vec![
            make_qi(&["30-39", "100", "M"]),
            make_qi(&["40-49", "200", "F"]),
            make_qi(&["50-59", "300", "M"]),
        ];

        let attack = LinkageAttack::with_defaults();
        let results = attack.evaluate(&original, &synthetic);

        assert_eq!(results.re_identification_rate, 0.0);
        assert_eq!(results.k_anonymity_achieved, 5);
        assert!(results.passes);
    }

    #[test]
    fn test_unique_records_high_reidentification() {
        // Every record in original has a unique QI combination
        let original = vec![
            make_qi(&["25", "10001", "M"]),
            make_qi(&["32", "10002", "F"]),
            make_qi(&["45", "10003", "M"]),
            make_qi(&["58", "10004", "F"]),
        ];

        // Synthetic has matching QI combos
        let synthetic = vec![
            make_qi(&["25", "10001", "M"]),
            make_qi(&["32", "10002", "F"]),
        ];

        let attack = LinkageAttack::with_defaults();
        let results = attack.evaluate(&original, &synthetic);

        // All synthetic records uniquely match
        assert!((results.re_identification_rate - 1.0).abs() < 1e-10);
        assert_eq!(results.k_anonymity_achieved, 1);
        assert!(!results.passes);
    }

    #[test]
    fn test_no_overlap() {
        let original = vec![make_qi(&["A", "1"]), make_qi(&["B", "2"])];
        let synthetic = vec![make_qi(&["C", "3"]), make_qi(&["D", "4"])];

        let attack = LinkageAttack::with_defaults();
        let results = attack.evaluate(&original, &synthetic);

        assert_eq!(results.re_identification_rate, 0.0);
        assert_eq!(results.overlapping_combos, 0);
        assert_eq!(results.uniquely_linked, 0);
    }

    #[test]
    fn test_empty_datasets() {
        let attack = LinkageAttack::with_defaults();
        let results = attack.evaluate(&[], &[]);
        assert!(results.passes);
        assert_eq!(results.re_identification_rate, 0.0);
    }

    #[test]
    fn test_linkage_config_serde() {
        let config = LinkageConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: LinkageConfig = serde_json::from_str(&json).unwrap();
        assert!((parsed.max_reidentification_rate - 0.05).abs() < 1e-10);
        assert_eq!(parsed.min_k_anonymity, 5);
    }

    #[test]
    fn test_linkage_results_serde() {
        let results = LinkageResults {
            re_identification_rate: 0.02,
            k_anonymity_achieved: 10,
            unique_qi_combos_original: 50,
            unique_qi_combos_synthetic: 45,
            overlapping_combos: 30,
            uniquely_linked: 1,
            total_synthetic: 100,
            passes: true,
        };
        let json = serde_json::to_string(&results).unwrap();
        let parsed: LinkageResults = serde_json::from_str(&json).unwrap();
        assert!((parsed.re_identification_rate - 0.02).abs() < 1e-10);
        assert_eq!(parsed.k_anonymity_achieved, 10);
    }

    #[test]
    fn test_partial_overlap() {
        // Some records unique, some with k>=2
        let original = vec![
            make_qi(&["A", "1"]), // unique
            make_qi(&["B", "2"]), // appears twice
            make_qi(&["B", "2"]),
            make_qi(&["C", "3"]), // appears 3 times
            make_qi(&["C", "3"]),
            make_qi(&["C", "3"]),
        ];

        // Synthetic has all three combos
        let synthetic = vec![
            make_qi(&["A", "1"]), // uniquely linked (orig count=1)
            make_qi(&["B", "2"]), // not uniquely linked (orig count=2)
            make_qi(&["C", "3"]), // not uniquely linked (orig count=3)
        ];

        let attack = LinkageAttack::with_defaults();
        let results = attack.evaluate(&original, &synthetic);

        assert_eq!(results.uniquely_linked, 1);
        assert!((results.re_identification_rate - 1.0 / 3.0).abs() < 1e-10);
        assert_eq!(results.k_anonymity_achieved, 1); // min group size = 1
    }
}
