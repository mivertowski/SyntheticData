//! Federated fingerprint extraction protocol.
//!
//! This module implements a protocol for extracting partial fingerprints from
//! distributed data sources and aggregating them without centralizing raw data.
//!
//! Each data source contributes a [`PartialFingerprint`] containing locally
//! computed statistics with a local differential privacy budget. The
//! [`FederatedFingerprintProtocol`] then aggregates these partials into a
//! single [`AggregatedFingerprint`] using weighted averaging by record count.
//!
//! # Privacy Model
//!
//! The total privacy budget (epsilon) of the aggregated fingerprint is the sum
//! of all local epsilons, following the sequential composition theorem. Each
//! source controls its own privacy-utility tradeoff independently.
//!
//! # Example
//!
//! ```
//! use datasynth_fingerprint::federated::{
//!     FederatedConfig, FederatedFingerprintProtocol, AggregationMethod,
//! };
//!
//! let config = FederatedConfig {
//!     min_sources: 2,
//!     max_epsilon_per_source: 5.0,
//!     aggregation_method: AggregationMethod::WeightedAverage,
//! };
//!
//! let protocol = FederatedFingerprintProtocol::new(config);
//!
//! let p1 = FederatedFingerprintProtocol::create_partial(
//!     "site-a",
//!     vec!["amount".into(), "qty".into()],
//!     1000,
//!     vec![100.0, 5.0],
//!     vec![20.0, 2.0],
//!     vec![10.0, 1.0],   // mins
//!     vec![200.0, 10.0], // maxs
//!     vec![],             // correlations
//!     1.0,
//! );
//!
//! let p2 = FederatedFingerprintProtocol::create_partial(
//!     "site-b",
//!     vec!["amount".into(), "qty".into()],
//!     3000,
//!     vec![120.0, 7.0],
//!     vec![25.0, 3.0],
//!     vec![5.0, 0.5],    // mins
//!     vec![250.0, 15.0], // maxs
//!     vec![],             // correlations
//!     1.0,
//! );
//!
//! let aggregated = protocol.aggregate(&[p1, p2]).expect("aggregation succeeds");
//! assert_eq!(aggregated.source_count, 2);
//! assert_eq!(aggregated.total_record_count, 4000);
//! ```

use serde::{Deserialize, Serialize};

/// A partial fingerprint extracted from a single data source.
///
/// Contains locally computed per-column statistics along with the
/// differential privacy budget spent for extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialFingerprint {
    /// Identifier for the data source.
    pub source_id: String,
    /// Local differential privacy epsilon spent.
    pub local_epsilon: f64,
    /// Number of records at this source.
    pub record_count: u64,
    /// Column names corresponding to the statistics vectors.
    pub column_names: Vec<String>,
    /// Per-column means.
    pub means: Vec<f64>,
    /// Per-column standard deviations.
    pub stds: Vec<f64>,
    /// Per-column minimum values.
    pub mins: Vec<f64>,
    /// Per-column maximum values.
    pub maxs: Vec<f64>,
    /// Flat correlation matrix (row-major, length = columns^2).
    pub correlations: Vec<f64>,
}

/// Method used to aggregate partial fingerprints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AggregationMethod {
    /// Weighted average by record count (default).
    #[default]
    WeightedAverage,
    /// Median of per-source statistics.
    Median,
    /// Trimmed mean (discard top/bottom 10% then average).
    TrimmedMean,
}

/// Configuration for federated fingerprint aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedConfig {
    /// Minimum number of sources required for aggregation.
    pub min_sources: usize,
    /// Maximum epsilon budget allowed per source.
    pub max_epsilon_per_source: f64,
    /// Aggregation method to use.
    pub aggregation_method: AggregationMethod,
}

impl Default for FederatedConfig {
    fn default() -> Self {
        Self {
            min_sources: 2,
            max_epsilon_per_source: 5.0,
            aggregation_method: AggregationMethod::WeightedAverage,
        }
    }
}

/// The result of aggregating multiple partial fingerprints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedFingerprint {
    /// Column names.
    pub column_names: Vec<String>,
    /// Aggregated per-column means.
    pub means: Vec<f64>,
    /// Aggregated per-column standard deviations.
    pub stds: Vec<f64>,
    /// Aggregated per-column minimums.
    pub mins: Vec<f64>,
    /// Aggregated per-column maximums.
    pub maxs: Vec<f64>,
    /// Aggregated correlation matrix (flat, row-major).
    pub correlations: Vec<f64>,
    /// Total record count across all sources.
    pub total_record_count: u64,
    /// Total epsilon (sum of local epsilons, sequential composition).
    pub total_epsilon: f64,
    /// Number of sources that contributed.
    pub source_count: usize,
}

/// Protocol for federated fingerprint extraction and aggregation.
///
/// Coordinates the creation of partial fingerprints from distributed
/// data sources and their aggregation into a single combined fingerprint.
#[derive(Debug, Clone)]
pub struct FederatedFingerprintProtocol {
    config: FederatedConfig,
}

impl FederatedFingerprintProtocol {
    /// Create a new protocol instance with the given configuration.
    pub fn new(config: FederatedConfig) -> Self {
        Self { config }
    }

    /// Create a partial fingerprint from locally computed statistics.
    ///
    /// The caller is responsible for computing the per-column statistics
    /// and selecting an appropriate local epsilon. Optional `mins`, `maxs`,
    /// and `correlations` can be provided for richer fingerprints; pass
    /// empty vectors to omit them.
    #[allow(clippy::too_many_arguments)]
    pub fn create_partial(
        source_id: &str,
        columns: Vec<String>,
        record_count: u64,
        means: Vec<f64>,
        stds: Vec<f64>,
        mins: Vec<f64>,
        maxs: Vec<f64>,
        correlations: Vec<f64>,
        epsilon: f64,
    ) -> PartialFingerprint {
        PartialFingerprint {
            source_id: source_id.to_string(),
            local_epsilon: epsilon,
            record_count,
            column_names: columns,
            means,
            stds,
            mins,
            maxs,
            correlations,
        }
    }

    /// Aggregate multiple partial fingerprints into a single combined fingerprint.
    ///
    /// Uses weighted averaging by record count for means and standard deviations.
    /// Total epsilon is the sum of all local epsilons (sequential composition).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Fewer than `min_sources` partials are provided
    /// - Any partial has an epsilon exceeding `max_epsilon_per_source`
    /// - Partials have mismatched column names
    /// - Any partial has zero record count
    pub fn aggregate(
        &self,
        partials: &[PartialFingerprint],
    ) -> Result<AggregatedFingerprint, String> {
        // Validate minimum source count
        if partials.len() < self.config.min_sources {
            return Err(format!(
                "Need at least {} sources, got {}",
                self.config.min_sources,
                partials.len()
            ));
        }

        // Validate each partial
        for p in partials {
            if p.record_count == 0 {
                return Err(format!("Source '{}' has zero records", p.source_id));
            }
            if p.local_epsilon > self.config.max_epsilon_per_source {
                return Err(format!(
                    "Source '{}' epsilon {} exceeds max {}",
                    p.source_id, p.local_epsilon, self.config.max_epsilon_per_source
                ));
            }
        }

        // Validate column consistency
        let first = &partials[0];
        let n_cols = first.column_names.len();
        for p in &partials[1..] {
            if p.column_names.len() != n_cols {
                return Err(format!(
                    "Column count mismatch: source '{}' has {} columns, expected {}",
                    p.source_id,
                    p.column_names.len(),
                    n_cols
                ));
            }
            for (i, name) in p.column_names.iter().enumerate() {
                if name != &first.column_names[i] {
                    return Err(format!(
                        "Column name mismatch at index {}: source '{}' has '{}', expected '{}'",
                        i, p.source_id, name, first.column_names[i]
                    ));
                }
            }
        }

        // Compute total record count and total epsilon
        let total_record_count: u64 = partials.iter().map(|p| p.record_count).sum();
        let total_epsilon: f64 = partials.iter().map(|p| p.local_epsilon).sum();

        // Aggregate using the configured method
        match self.config.aggregation_method {
            AggregationMethod::WeightedAverage => {
                self.aggregate_weighted(partials, n_cols, total_record_count, total_epsilon)
            }
            AggregationMethod::Median => {
                self.aggregate_median(partials, n_cols, total_record_count, total_epsilon)
            }
            AggregationMethod::TrimmedMean => {
                self.aggregate_trimmed_mean(partials, n_cols, total_record_count, total_epsilon)
            }
        }
    }

    /// Weighted average aggregation (weights proportional to record count).
    fn aggregate_weighted(
        &self,
        partials: &[PartialFingerprint],
        n_cols: usize,
        total_record_count: u64,
        total_epsilon: f64,
    ) -> Result<AggregatedFingerprint, String> {
        let total_f = total_record_count as f64;
        let mut agg_means = vec![0.0_f64; n_cols];
        let mut agg_stds = vec![0.0_f64; n_cols];
        let mut agg_mins = vec![f64::INFINITY; n_cols];
        let mut agg_maxs = vec![f64::NEG_INFINITY; n_cols];

        for p in partials {
            let w = p.record_count as f64 / total_f;
            for i in 0..n_cols {
                if i < p.means.len() {
                    agg_means[i] += w * p.means[i];
                }
                if i < p.stds.len() {
                    agg_stds[i] += w * p.stds[i];
                }
                if i < p.mins.len() && p.mins[i].is_finite() && p.mins[i] < agg_mins[i] {
                    agg_mins[i] = p.mins[i];
                }
                if i < p.maxs.len() && p.maxs[i].is_finite() && p.maxs[i] > agg_maxs[i] {
                    agg_maxs[i] = p.maxs[i];
                }
            }
        }

        // Aggregate correlations if all partials have them
        let corr_len = n_cols * n_cols;
        let all_have_corr = partials.iter().all(|p| p.correlations.len() == corr_len);
        let agg_corr = if all_have_corr && corr_len > 0 {
            let mut corr = vec![0.0_f64; corr_len];
            for p in partials {
                let w = p.record_count as f64 / total_f;
                for (j, val) in p.correlations.iter().enumerate() {
                    corr[j] += w * val;
                }
            }
            corr
        } else {
            Vec::new()
        };

        Ok(AggregatedFingerprint {
            column_names: partials[0].column_names.clone(),
            means: agg_means,
            stds: agg_stds,
            mins: agg_mins,
            maxs: agg_maxs,
            correlations: agg_corr,
            total_record_count,
            total_epsilon,
            source_count: partials.len(),
        })
    }

    /// Median aggregation.
    fn aggregate_median(
        &self,
        partials: &[PartialFingerprint],
        n_cols: usize,
        total_record_count: u64,
        total_epsilon: f64,
    ) -> Result<AggregatedFingerprint, String> {
        let mut agg_means = vec![0.0_f64; n_cols];
        let mut agg_stds = vec![0.0_f64; n_cols];
        let mut agg_mins = vec![f64::INFINITY; n_cols];
        let mut agg_maxs = vec![f64::NEG_INFINITY; n_cols];

        for i in 0..n_cols {
            let mut col_means: Vec<f64> = partials
                .iter()
                .filter(|p| i < p.means.len())
                .map(|p| p.means[i])
                .collect();
            let mut col_stds: Vec<f64> = partials
                .iter()
                .filter(|p| i < p.stds.len())
                .map(|p| p.stds[i])
                .collect();

            col_means.sort_by(|a, b| a.total_cmp(b));
            col_stds.sort_by(|a, b| a.total_cmp(b));

            agg_means[i] = compute_median(&col_means);
            agg_stds[i] = compute_median(&col_stds);

            for p in partials {
                if i < p.mins.len() && p.mins[i].is_finite() && p.mins[i] < agg_mins[i] {
                    agg_mins[i] = p.mins[i];
                }
                if i < p.maxs.len() && p.maxs[i].is_finite() && p.maxs[i] > agg_maxs[i] {
                    agg_maxs[i] = p.maxs[i];
                }
            }
        }

        Ok(AggregatedFingerprint {
            column_names: partials[0].column_names.clone(),
            means: agg_means,
            stds: agg_stds,
            mins: agg_mins,
            maxs: agg_maxs,
            correlations: Vec::new(),
            total_record_count,
            total_epsilon,
            source_count: partials.len(),
        })
    }

    /// Trimmed mean aggregation (discard top/bottom 10%).
    fn aggregate_trimmed_mean(
        &self,
        partials: &[PartialFingerprint],
        n_cols: usize,
        total_record_count: u64,
        total_epsilon: f64,
    ) -> Result<AggregatedFingerprint, String> {
        let n = partials.len();
        let trim_count = (n as f64 * 0.1).floor() as usize;

        let mut agg_means = vec![0.0_f64; n_cols];
        let mut agg_stds = vec![0.0_f64; n_cols];
        let mut agg_mins = vec![f64::INFINITY; n_cols];
        let mut agg_maxs = vec![f64::NEG_INFINITY; n_cols];

        for i in 0..n_cols {
            let mut col_means: Vec<f64> = partials
                .iter()
                .filter(|p| i < p.means.len())
                .map(|p| p.means[i])
                .collect();
            let mut col_stds: Vec<f64> = partials
                .iter()
                .filter(|p| i < p.stds.len())
                .map(|p| p.stds[i])
                .collect();

            col_means.sort_by(|a, b| a.total_cmp(b));
            col_stds.sort_by(|a, b| a.total_cmp(b));

            // Trim top/bottom
            let trimmed_means = trim_slice(&col_means, trim_count);
            let trimmed_stds = trim_slice(&col_stds, trim_count);

            agg_means[i] = if trimmed_means.is_empty() {
                0.0
            } else {
                trimmed_means.iter().sum::<f64>() / trimmed_means.len() as f64
            };
            agg_stds[i] = if trimmed_stds.is_empty() {
                0.0
            } else {
                trimmed_stds.iter().sum::<f64>() / trimmed_stds.len() as f64
            };

            for p in partials {
                if i < p.mins.len() && p.mins[i].is_finite() && p.mins[i] < agg_mins[i] {
                    agg_mins[i] = p.mins[i];
                }
                if i < p.maxs.len() && p.maxs[i].is_finite() && p.maxs[i] > agg_maxs[i] {
                    agg_maxs[i] = p.maxs[i];
                }
            }
        }

        Ok(AggregatedFingerprint {
            column_names: partials[0].column_names.clone(),
            means: agg_means,
            stds: agg_stds,
            mins: agg_mins,
            maxs: agg_maxs,
            correlations: Vec::new(),
            total_record_count,
            total_epsilon,
            source_count: partials.len(),
        })
    }
}

/// Compute median of a sorted slice.
fn compute_median(sorted: &[f64]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Trim `count` elements from each end of a slice.
fn trim_slice(sorted: &[f64], count: usize) -> &[f64] {
    if count * 2 >= sorted.len() {
        return sorted; // Can't trim more than half, return all
    }
    &sorted[count..sorted.len() - count]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_partial(
        source_id: &str,
        record_count: u64,
        means: Vec<f64>,
        stds: Vec<f64>,
        epsilon: f64,
    ) -> PartialFingerprint {
        let columns = vec!["amount".to_string(), "qty".to_string()];
        FederatedFingerprintProtocol::create_partial(
            source_id,
            columns,
            record_count,
            means,
            stds,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            epsilon,
        )
    }

    #[test]
    fn test_three_sources_aggregate_correctly() {
        let config = FederatedConfig {
            min_sources: 2,
            max_epsilon_per_source: 5.0,
            aggregation_method: AggregationMethod::WeightedAverage,
        };
        let protocol = FederatedFingerprintProtocol::new(config);

        let p1 = make_partial("site-a", 1000, vec![100.0, 5.0], vec![10.0, 1.0], 1.0);
        let p2 = make_partial("site-b", 2000, vec![200.0, 10.0], vec![20.0, 2.0], 0.5);
        let p3 = make_partial("site-c", 1000, vec![300.0, 15.0], vec![30.0, 3.0], 0.8);

        let result = protocol.aggregate(&[p1, p2, p3]).expect("should aggregate");

        assert_eq!(result.source_count, 3);
        assert_eq!(result.total_record_count, 4000);

        // Weighted average for means:
        // amount: (1000*100 + 2000*200 + 1000*300) / 4000 = (100k + 400k + 300k) / 4000 = 200.0
        assert!((result.means[0] - 200.0).abs() < 1e-10);
        // qty: (1000*5 + 2000*10 + 1000*15) / 4000 = (5k + 20k + 15k) / 4000 = 10.0
        assert!((result.means[1] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_weights_proportional_to_record_count() {
        let config = FederatedConfig::default();
        let protocol = FederatedFingerprintProtocol::new(config);

        // Site-b has 3x the records of site-a, so it should dominate
        let p1 = make_partial("site-a", 1000, vec![100.0, 1.0], vec![10.0, 1.0], 1.0);
        let p2 = make_partial("site-b", 3000, vec![200.0, 3.0], vec![20.0, 3.0], 1.0);

        let result = protocol.aggregate(&[p1, p2]).expect("should aggregate");

        // Weighted mean: (1000*100 + 3000*200) / 4000 = 700000/4000 = 175.0
        assert!((result.means[0] - 175.0).abs() < 1e-10);
        // Weighted mean: (1000*1 + 3000*3) / 4000 = 10000/4000 = 2.5
        assert!((result.means[1] - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_total_epsilon_sums_correctly() {
        let config = FederatedConfig::default();
        let protocol = FederatedFingerprintProtocol::new(config);

        let p1 = make_partial("a", 100, vec![1.0, 2.0], vec![0.1, 0.2], 0.5);
        let p2 = make_partial("b", 200, vec![3.0, 4.0], vec![0.3, 0.4], 1.5);
        let p3 = make_partial("c", 300, vec![5.0, 6.0], vec![0.5, 0.6], 2.0);

        let result = protocol.aggregate(&[p1, p2, p3]).expect("should aggregate");

        assert!((result.total_epsilon - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_sources_rejected() {
        let config = FederatedConfig {
            min_sources: 2,
            ..FederatedConfig::default()
        };
        let protocol = FederatedFingerprintProtocol::new(config);

        // Too few sources
        let p1 = make_partial("a", 100, vec![1.0, 2.0], vec![0.1, 0.2], 1.0);
        let result = protocol.aggregate(&[p1]);
        assert!(result.is_err());
        assert!(result
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("Need at least 2 sources")));

        // Empty slice
        let result = protocol.aggregate(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_record_count_rejected() {
        let config = FederatedConfig::default();
        let protocol = FederatedFingerprintProtocol::new(config);

        let p1 = make_partial("a", 100, vec![1.0, 2.0], vec![0.1, 0.2], 1.0);
        let p2 = make_partial("b", 0, vec![3.0, 4.0], vec![0.3, 0.4], 1.0);

        let result = protocol.aggregate(&[p1, p2]);
        assert!(result.is_err());
        assert!(result
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("zero records")));
    }

    #[test]
    fn test_single_source_works() {
        let config = FederatedConfig {
            min_sources: 1,
            ..FederatedConfig::default()
        };
        let protocol = FederatedFingerprintProtocol::new(config);

        let p1 = make_partial("only", 500, vec![42.0, 7.0], vec![5.0, 1.0], 1.0);
        let result = protocol
            .aggregate(&[p1])
            .expect("single source should work");

        assert_eq!(result.source_count, 1);
        assert_eq!(result.total_record_count, 500);
        assert!((result.means[0] - 42.0).abs() < 1e-10);
        assert!((result.means[1] - 7.0).abs() < 1e-10);
        assert!((result.total_epsilon - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_epsilon_per_source_limit() {
        let config = FederatedConfig {
            min_sources: 2,
            max_epsilon_per_source: 1.0,
            ..FederatedConfig::default()
        };
        let protocol = FederatedFingerprintProtocol::new(config);

        let p1 = make_partial("a", 100, vec![1.0, 2.0], vec![0.1, 0.2], 0.5);
        let p2 = make_partial("b", 200, vec![3.0, 4.0], vec![0.3, 0.4], 2.0); // exceeds max

        let result = protocol.aggregate(&[p1, p2]);
        assert!(result.is_err());
        assert!(result
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("exceeds max")));
    }

    #[test]
    fn test_column_mismatch_rejected() {
        let config = FederatedConfig {
            min_sources: 2,
            ..FederatedConfig::default()
        };
        let protocol = FederatedFingerprintProtocol::new(config);

        let p1 = make_partial("a", 100, vec![1.0, 2.0], vec![0.1, 0.2], 1.0);
        let mut p2 = make_partial("b", 200, vec![3.0, 4.0], vec![0.3, 0.4], 1.0);
        p2.column_names = vec!["amount".to_string(), "price".to_string()]; // different name

        let result = protocol.aggregate(&[p1, p2]);
        assert!(result.is_err());
        assert!(result
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("Column name mismatch")));
    }

    #[test]
    fn test_median_aggregation() {
        let config = FederatedConfig {
            min_sources: 1,
            max_epsilon_per_source: 5.0,
            aggregation_method: AggregationMethod::Median,
        };
        let protocol = FederatedFingerprintProtocol::new(config);

        let p1 = make_partial("a", 100, vec![10.0, 1.0], vec![1.0, 0.1], 1.0);
        let p2 = make_partial("b", 100, vec![20.0, 2.0], vec![2.0, 0.2], 1.0);
        let p3 = make_partial("c", 100, vec![30.0, 3.0], vec![3.0, 0.3], 1.0);

        let result = protocol.aggregate(&[p1, p2, p3]).expect("should aggregate");

        // Median of [10, 20, 30] = 20
        assert!((result.means[0] - 20.0).abs() < 1e-10);
        // Median of [1, 2, 3] = 2
        assert!((result.means[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_serde_roundtrip() {
        let partial = make_partial("test", 100, vec![1.0, 2.0], vec![0.1, 0.2], 1.0);
        let json = serde_json::to_string(&partial).expect("serialize");
        let deserialized: PartialFingerprint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.source_id, "test");
        assert_eq!(deserialized.record_count, 100);
    }
}
