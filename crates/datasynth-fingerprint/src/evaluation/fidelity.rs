//! Fidelity evaluation comparing synthetic data to fingerprints.

use std::collections::HashMap;

use serde::Serialize;

use crate::error::FingerprintResult;
use crate::extraction::{DataSource, FingerprintExtractor};
use crate::models::{Fingerprint, NumericStats, StatisticsFingerprint};

/// Configuration for fidelity evaluation.
#[derive(Debug, Clone)]
pub struct FidelityConfig {
    /// Minimum acceptable overall fidelity score (0.0-1.0).
    pub threshold: f64,
    /// Weight for statistical fidelity.
    pub statistical_weight: f64,
    /// Weight for correlation fidelity.
    pub correlation_weight: f64,
    /// Weight for schema fidelity.
    pub schema_weight: f64,
    /// Weight for rule compliance.
    pub rule_weight: f64,
    /// Weight for anomaly fidelity.
    pub anomaly_weight: f64,
}

impl Default for FidelityConfig {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            statistical_weight: 0.30,
            correlation_weight: 0.20,
            schema_weight: 0.20,
            rule_weight: 0.20,
            anomaly_weight: 0.10,
        }
    }
}

/// Report from fidelity evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct FidelityReport {
    /// Overall fidelity score (0.0-1.0).
    pub overall_score: f64,
    /// Statistical fidelity score.
    pub statistical_fidelity: f64,
    /// Correlation fidelity score.
    pub correlation_fidelity: f64,
    /// Schema fidelity score.
    pub schema_fidelity: f64,
    /// Rule compliance score.
    pub rule_compliance: f64,
    /// Anomaly fidelity score.
    pub anomaly_fidelity: f64,
    /// Whether overall score passes threshold.
    pub passes: bool,
    /// Detailed metrics.
    pub details: FidelityDetails,
}

/// Detailed fidelity metrics.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FidelityDetails {
    /// Per-column statistical metrics.
    pub column_metrics: HashMap<String, ColumnFidelityMetrics>,
    /// KS statistics by column.
    pub ks_statistics: HashMap<String, f64>,
    /// Wasserstein distances by column.
    pub wasserstein_distances: HashMap<String, f64>,
    /// Jensen-Shannon divergences by column.
    pub js_divergences: HashMap<String, f64>,
    /// Benford's Law MAD.
    pub benford_mad: Option<f64>,
    /// Correlation matrix RMSE.
    pub correlation_rmse: Option<f64>,
    /// Row count ratio (synthetic / fingerprint).
    pub row_count_ratio: f64,
    /// Warnings.
    pub warnings: Vec<String>,
}

/// Fidelity metrics for a single column.
#[derive(Debug, Clone, Serialize)]
pub struct ColumnFidelityMetrics {
    /// Column name.
    pub name: String,
    /// KS statistic.
    pub ks_statistic: f64,
    /// Wasserstein distance.
    pub wasserstein_distance: f64,
    /// Jensen-Shannon divergence.
    pub js_divergence: f64,
    /// Mean difference (normalized).
    pub mean_diff: f64,
    /// Std dev difference (normalized).
    pub std_dev_diff: f64,
}

/// Evaluator for fidelity between synthetic data and fingerprints.
pub struct FidelityEvaluator {
    config: FidelityConfig,
}

impl FidelityEvaluator {
    /// Create a new evaluator with default configuration.
    pub fn new() -> Self {
        Self {
            config: FidelityConfig::default(),
        }
    }

    /// Create with a specific threshold.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            config: FidelityConfig {
                threshold,
                ..Default::default()
            },
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: FidelityConfig) -> Self {
        Self { config }
    }

    /// Evaluate fidelity of synthetic data against a fingerprint.
    pub fn evaluate(
        &self,
        fingerprint: &Fingerprint,
        synthetic_data: &DataSource,
    ) -> FingerprintResult<FidelityReport> {
        // Extract fingerprint from synthetic data for comparison
        let extractor = FingerprintExtractor::new();
        let synthetic_fp = extractor.extract(synthetic_data)?;

        self.evaluate_fingerprints(fingerprint, &synthetic_fp)
    }

    /// Evaluate fidelity between two fingerprints.
    pub fn evaluate_fingerprints(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
    ) -> FingerprintResult<FidelityReport> {
        let mut details = FidelityDetails::default();

        // Schema fidelity (computed first as it affects other scores)
        let schema_fidelity = self.evaluate_schema(original, synthetic, &mut details);

        // Statistical fidelity
        let statistical_fidelity =
            self.evaluate_statistical(&original.statistics, &synthetic.statistics, &mut details);

        // For correlation, rules, and anomalies:
        // If schema fidelity is very low (completely different schemas),
        // don't let "nothing to compare" result in a high score.
        // Instead, use schema fidelity as the baseline.

        // Correlation fidelity
        let raw_correlation_fidelity =
            self.evaluate_correlations(original, synthetic, &mut details);
        let correlation_fidelity = if raw_correlation_fidelity >= 0.99 && schema_fidelity < 0.5 {
            // "Nothing to compare" case with schema mismatch
            schema_fidelity
        } else {
            raw_correlation_fidelity
        };

        // Rule compliance
        let raw_rule_compliance = self.evaluate_rules(original, synthetic, &mut details);
        let rule_compliance = if raw_rule_compliance >= 0.99 && schema_fidelity < 0.5 {
            schema_fidelity
        } else {
            raw_rule_compliance
        };

        // Anomaly fidelity
        let raw_anomaly_fidelity = self.evaluate_anomalies(original, synthetic, &mut details);
        let anomaly_fidelity = if raw_anomaly_fidelity >= 0.99 && schema_fidelity < 0.5 {
            schema_fidelity
        } else {
            raw_anomaly_fidelity
        };

        // Calculate overall score
        let overall_score = self.config.statistical_weight * statistical_fidelity
            + self.config.correlation_weight * correlation_fidelity
            + self.config.schema_weight * schema_fidelity
            + self.config.rule_weight * rule_compliance
            + self.config.anomaly_weight * anomaly_fidelity;

        let passes = overall_score >= self.config.threshold;

        Ok(FidelityReport {
            overall_score,
            statistical_fidelity,
            correlation_fidelity,
            schema_fidelity,
            rule_compliance,
            anomaly_fidelity,
            passes,
            details,
        })
    }

    /// Evaluate statistical fidelity.
    fn evaluate_statistical(
        &self,
        original: &StatisticsFingerprint,
        synthetic: &StatisticsFingerprint,
        details: &mut FidelityDetails,
    ) -> f64 {
        let mut scores = Vec::new();

        // Count total and matching columns
        let orig_numeric_count = original.numeric_columns.len();
        let orig_categorical_count = original.categorical_columns.len();
        let total_orig_columns = orig_numeric_count + orig_categorical_count;

        let mut matched_numeric = 0;
        let mut matched_categorical = 0;

        // Build a lookup by stripped column name for synthetic columns
        // (strip table prefix to match columns with different table names)
        let syn_numeric_by_stripped: HashMap<String, (&str, &NumericStats)> = synthetic
            .numeric_columns
            .iter()
            .map(|(k, v)| {
                let stripped = k.split('.').next_back().unwrap_or(k).to_string();
                (stripped, (k.as_str(), v))
            })
            .collect();

        let syn_categorical_keys: std::collections::HashSet<String> = synthetic
            .categorical_columns
            .keys()
            .map(|k| k.split('.').next_back().unwrap_or(k).to_string())
            .collect();

        // Compare numeric columns
        for (col_name, orig_stats) in &original.numeric_columns {
            let stripped = col_name.split('.').next_back().unwrap_or(col_name);

            // First try exact match
            if let Some(syn_stats) = synthetic.numeric_columns.get(col_name) {
                matched_numeric += 1;
                let metrics = self.compare_numeric_stats(col_name, orig_stats, syn_stats);
                let col_score = 1.0
                    - (metrics.ks_statistic
                        + metrics.mean_diff.abs().min(1.0)
                        + metrics.std_dev_diff.abs().min(1.0))
                        / 3.0;
                scores.push(col_score.max(0.0));
                details
                    .ks_statistics
                    .insert(col_name.clone(), metrics.ks_statistic);
                details
                    .wasserstein_distances
                    .insert(col_name.clone(), metrics.wasserstein_distance);
                details
                    .js_divergences
                    .insert(col_name.clone(), metrics.js_divergence);
                details.column_metrics.insert(col_name.clone(), metrics);
            } else if let Some((_syn_key, syn_stats)) = syn_numeric_by_stripped.get(stripped) {
                // Match by stripped column name (different table prefix)
                matched_numeric += 1;
                let metrics = self.compare_numeric_stats(stripped, orig_stats, syn_stats);
                let col_score = 1.0
                    - (metrics.ks_statistic
                        + metrics.mean_diff.abs().min(1.0)
                        + metrics.std_dev_diff.abs().min(1.0))
                        / 3.0;
                scores.push(col_score.max(0.0));
                details
                    .ks_statistics
                    .insert(col_name.clone(), metrics.ks_statistic);
                details
                    .wasserstein_distances
                    .insert(col_name.clone(), metrics.wasserstein_distance);
                details
                    .js_divergences
                    .insert(col_name.clone(), metrics.js_divergence);
                details.column_metrics.insert(col_name.clone(), metrics);
            }
        }

        // Compare categorical columns
        for col_name in original.categorical_columns.keys() {
            let stripped = col_name.split('.').next_back().unwrap_or(col_name);
            if synthetic.categorical_columns.contains_key(col_name)
                || syn_categorical_keys.contains(stripped)
            {
                matched_categorical += 1;
            }
        }

        // Compare Benford's Law if available
        if let (Some(orig_benford), Some(syn_benford)) =
            (&original.benford_analysis, &synthetic.benford_analysis)
        {
            let benford_mad = compute_benford_mad(
                &orig_benford.observed_frequencies,
                &syn_benford.observed_frequencies,
            );
            details.benford_mad = Some(benford_mad);
            scores.push(1.0 - benford_mad.min(0.1) * 10.0); // Scale MAD to score
        }

        // If no columns matched but both fingerprints have columns,
        // this indicates completely different schemas - return low score
        if total_orig_columns > 0 {
            let total_matched = matched_numeric + matched_categorical;
            let match_ratio = total_matched as f64 / total_orig_columns as f64;

            if total_matched == 0 {
                // No columns match - completely different schemas
                details.warnings.push(
                    "No columns matched between original and synthetic fingerprints".to_string(),
                );
                return 0.0;
            }

            // Weight the scores by column match ratio
            if scores.is_empty() {
                // Only categorical columns matched (or numeric columns didn't match)
                return match_ratio;
            }

            // Combine column scores with match ratio penalty
            let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
            return avg_score * match_ratio;
        }

        if scores.is_empty() {
            return 1.0; // No columns to compare in either fingerprint
        }

        scores.iter().sum::<f64>() / scores.len() as f64
    }

    /// Compare numeric statistics.
    fn compare_numeric_stats(
        &self,
        name: &str,
        original: &NumericStats,
        synthetic: &NumericStats,
    ) -> ColumnFidelityMetrics {
        // KS-like statistic from percentile comparison
        let ks_statistic = self.compute_percentile_ks(original, synthetic);

        // Normalized differences
        let mean_range = (original.max - original.min).max(1.0);
        let mean_diff = (original.mean - synthetic.mean) / mean_range;
        let std_dev_diff = if original.std_dev > 0.0 {
            (original.std_dev - synthetic.std_dev) / original.std_dev
        } else {
            0.0
        };

        // Wasserstein-1 distance from percentile-based inverse CDFs
        let wasserstein_distance =
            wasserstein_distance_from_percentiles(&original.percentiles, &synthetic.percentiles);

        // Jensen-Shannon divergence from percentile bins
        let js_divergence =
            js_divergence_from_percentiles(&original.percentiles, &synthetic.percentiles);

        ColumnFidelityMetrics {
            name: name.to_string(),
            ks_statistic,
            wasserstein_distance,
            js_divergence,
            mean_diff,
            std_dev_diff,
        }
    }

    /// Compute KS-like statistic from percentiles.
    fn compute_percentile_ks(&self, original: &NumericStats, synthetic: &NumericStats) -> f64 {
        let orig_pcts = original.percentiles.to_array();
        let syn_pcts = synthetic.percentiles.to_array();

        let range = (original.max - original.min).max(1.0);

        orig_pcts
            .iter()
            .zip(syn_pcts.iter())
            .map(|(&o, &s)| ((o - s) / range).abs())
            .fold(0.0, f64::max)
    }

    /// Evaluate correlation fidelity.
    fn evaluate_correlations(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
        details: &mut FidelityDetails,
    ) -> f64 {
        let (orig_corr, syn_corr) = match (&original.correlations, &synthetic.correlations) {
            (Some(o), Some(s)) => (o, s),
            _ => return 1.0, // No correlations to compare
        };

        let mut rmse_sum = 0.0;
        let mut count = 0;

        for (table_name, orig_matrix) in &orig_corr.matrices {
            if let Some(syn_matrix) = syn_corr.matrices.get(table_name) {
                // Compare correlation values
                for (i, &orig_val) in orig_matrix.correlations.iter().enumerate() {
                    if let Some(&syn_val) = syn_matrix.correlations.get(i) {
                        rmse_sum += (orig_val - syn_val).powi(2);
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            return 1.0;
        }

        let rmse = (rmse_sum / count as f64).sqrt();
        details.correlation_rmse = Some(rmse);

        // Convert RMSE to score (RMSE of 0 = 1.0, RMSE of 1 = 0.0)
        1.0 - rmse.min(1.0)
    }

    /// Evaluate schema fidelity.
    fn evaluate_schema(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
        details: &mut FidelityDetails,
    ) -> f64 {
        // Check table presence
        let orig_tables: std::collections::HashSet<_> = original.schema.tables.keys().collect();
        let syn_tables: std::collections::HashSet<_> = synthetic.schema.tables.keys().collect();

        // Calculate table overlap
        let common_tables = orig_tables.intersection(&syn_tables).count();
        let total_tables = orig_tables.len().max(syn_tables.len());

        if total_tables == 0 {
            return 1.0; // No tables in either
        }

        let table_overlap_ratio = common_tables as f64 / total_tables as f64;

        // Check row count ratio
        let orig_rows: u64 = original.schema.tables.values().map(|t| t.row_count).sum();
        let syn_rows: u64 = synthetic.schema.tables.values().map(|t| t.row_count).sum();

        let ratio = if orig_rows > 0 {
            syn_rows as f64 / orig_rows as f64
        } else {
            1.0
        };
        details.row_count_ratio = ratio;

        // Penalize if ratio is too far from 1.0 (unless intentionally scaled)
        let ratio_penalty = (ratio - 1.0).abs().min(1.0) * 0.2;

        // Check column match
        let mut column_match_scores = Vec::new();

        if common_tables > 0 {
            // Tables with matching names - compare columns directly
            for (table_name, orig_table) in &original.schema.tables {
                if let Some(syn_table) = synthetic.schema.tables.get(table_name) {
                    let orig_cols: std::collections::HashSet<_> =
                        orig_table.columns.iter().map(|c| &c.name).collect();
                    let syn_cols: std::collections::HashSet<_> =
                        syn_table.columns.iter().map(|c| &c.name).collect();

                    let common_cols = orig_cols.intersection(&syn_cols).count();
                    let total_cols = orig_cols.len().max(syn_cols.len());

                    if total_cols > 0 {
                        column_match_scores.push(common_cols as f64 / total_cols as f64);
                    }
                }
            }
        } else if orig_tables.len() == syn_tables.len() {
            // No matching table names, but same number of tables
            // Try to match by column structure (common use case: same schema, different file names)
            let orig_table_list: Vec<_> = original.schema.tables.values().collect();
            let syn_table_list: Vec<_> = synthetic.schema.tables.values().collect();

            // For each original table, find best matching synthetic table by columns
            for orig_table in &orig_table_list {
                let orig_cols: std::collections::HashSet<_> =
                    orig_table.columns.iter().map(|c| &c.name).collect();

                let mut best_match_score: f64 = 0.0;
                for syn_table in &syn_table_list {
                    let syn_cols: std::collections::HashSet<_> =
                        syn_table.columns.iter().map(|c| &c.name).collect();

                    let common_cols = orig_cols.intersection(&syn_cols).count();
                    let total_cols = orig_cols.len().max(syn_cols.len());

                    if total_cols > 0 {
                        let score = common_cols as f64 / total_cols as f64;
                        best_match_score = best_match_score.max(score);
                    }
                }
                column_match_scores.push(best_match_score);
            }
        } else {
            // Different number of tables and no name overlap - truly different schemas
            let missing = orig_tables.difference(&syn_tables).count();
            details.warnings.push(format!(
                "{} tables missing in synthetic data (no overlap)",
                missing
            ));
            return 0.0;
        }

        let missing = orig_tables.difference(&syn_tables).count();
        if missing > 0 && common_tables > 0 {
            details
                .warnings
                .push(format!("{} tables missing in synthetic data", missing));
        }

        // Calculate column match score
        let column_score = if column_match_scores.is_empty() {
            if common_tables == 0 {
                0.0 // No tables matched by name and couldn't match by structure
            } else {
                1.0 // Tables matched but had no columns (edge case)
            }
        } else {
            column_match_scores.iter().sum::<f64>() / column_match_scores.len() as f64
        };

        // If table names didn't match but columns did, still consider it a good match
        let effective_table_ratio = if common_tables == 0 && column_score > 0.8 {
            // Tables matched by column structure rather than name
            1.0
        } else {
            table_overlap_ratio
        };

        // Combine: table overlap (40%), column match (40%), row ratio (20%)
        let score = 0.4 * effective_table_ratio + 0.4 * column_score + 0.2 * (1.0 - ratio_penalty);

        score.clamp(0.0, 1.0)
    }

    /// Evaluate rule compliance.
    fn evaluate_rules(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
        _details: &mut FidelityDetails,
    ) -> f64 {
        // Compare rule compliance rates if available
        let (orig_rules, syn_rules) = match (&original.rules, &synthetic.rules) {
            (Some(o), Some(s)) => (o, s),
            _ => return 1.0,
        };

        let mut score = 1.0;

        // Compare balance rule compliance
        for orig_rule in &orig_rules.balance_rules {
            if let Some(syn_rule) = syn_rules
                .balance_rules
                .iter()
                .find(|r| r.name == orig_rule.name)
            {
                let diff = (orig_rule.compliance_rate - syn_rule.compliance_rate).abs();
                score -= diff * 0.1;
            }
        }

        score.max(0.0)
    }

    /// Evaluate anomaly fidelity.
    fn evaluate_anomalies(
        &self,
        original: &Fingerprint,
        synthetic: &Fingerprint,
        _details: &mut FidelityDetails,
    ) -> f64 {
        let (orig_anomalies, syn_anomalies) = match (&original.anomalies, &synthetic.anomalies) {
            (Some(o), Some(s)) => (o, s),
            _ => return 1.0,
        };

        // Compare overall anomaly rates
        let rate_diff =
            (orig_anomalies.overall.anomaly_rate - syn_anomalies.overall.anomaly_rate).abs();

        // Convert to score (0.1 rate difference = 0.0 score)
        1.0 - (rate_diff * 10.0).min(1.0)
    }
}

impl Default for FidelityEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Percentile probability levels corresponding to p1, p5, p10, p25, p50, p75, p90, p95, p99.
const PERCENTILE_PROBS: [f64; 9] = [0.01, 0.05, 0.10, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99];

/// Compute Wasserstein-1 (Earth Mover's) distance between two distributions
/// using piecewise-linear inverse CDFs built from percentile data.
///
/// The W1 distance is defined as:
///   W1 = integral_0^1 |F_orig^{-1}(p) - F_synth^{-1}(p)| dp
///
/// We approximate each inverse CDF as a piecewise-linear function through the
/// known percentile points and use the trapezoidal rule for numeric integration.
fn wasserstein_distance_from_percentiles(
    original: &crate::models::Percentiles,
    synthetic: &crate::models::Percentiles,
) -> f64 {
    let orig_vals = original.to_array();
    let syn_vals = synthetic.to_array();

    // Build evaluation points: use the 9 percentile probabilities plus intermediate
    // points for better trapezoidal approximation. We add the endpoints 0 and 1
    // by extrapolating linearly from the first/last segments.
    //
    // The inverse CDF at probability p is interpolated from the known
    // (PERCENTILE_PROBS[i], value[i]) pairs.

    // Number of sub-intervals for trapezoidal integration
    let n_steps = 200;
    let dp = 1.0 / n_steps as f64;

    let mut integral = 0.0;
    let prev_diff = (interp_inv_cdf(0.0, &orig_vals) - interp_inv_cdf(0.0, &syn_vals)).abs();
    let mut prev = prev_diff;

    for i in 1..=n_steps {
        let p = i as f64 * dp;
        let orig_q = interp_inv_cdf(p, &orig_vals);
        let syn_q = interp_inv_cdf(p, &syn_vals);
        let curr = (orig_q - syn_q).abs();
        integral += (prev + curr) * dp / 2.0;
        prev = curr;
    }

    integral
}

/// Interpolate the inverse CDF (quantile function) at probability `p`
/// using piecewise-linear interpolation through the 9 percentile knots.
///
/// For p outside [0.01, 0.99], we linearly extrapolate from the nearest segment.
fn interp_inv_cdf(p: f64, values: &[f64; 9]) -> f64 {
    // Clamp to avoid wild extrapolation
    let p = p.clamp(0.0, 1.0);

    // Find the bracketing interval
    if p <= PERCENTILE_PROBS[0] {
        // Extrapolate below p1 using the p1-p5 segment slope
        let slope = if (PERCENTILE_PROBS[1] - PERCENTILE_PROBS[0]).abs() > f64::EPSILON {
            (values[1] - values[0]) / (PERCENTILE_PROBS[1] - PERCENTILE_PROBS[0])
        } else {
            0.0
        };
        values[0] + slope * (p - PERCENTILE_PROBS[0])
    } else if p >= PERCENTILE_PROBS[8] {
        // Extrapolate above p99 using the p95-p99 segment slope
        let slope = if (PERCENTILE_PROBS[8] - PERCENTILE_PROBS[7]).abs() > f64::EPSILON {
            (values[8] - values[7]) / (PERCENTILE_PROBS[8] - PERCENTILE_PROBS[7])
        } else {
            0.0
        };
        values[8] + slope * (p - PERCENTILE_PROBS[8])
    } else {
        // Find the bracketing interval and linearly interpolate
        for i in 0..8 {
            if p >= PERCENTILE_PROBS[i] && p <= PERCENTILE_PROBS[i + 1] {
                let frac =
                    (p - PERCENTILE_PROBS[i]) / (PERCENTILE_PROBS[i + 1] - PERCENTILE_PROBS[i]);
                return values[i] + frac * (values[i + 1] - values[i]);
            }
        }
        // Fallback (should not reach here)
        values[4] // median
    }
}

/// Compute Jensen-Shannon divergence between two distributions using
/// PMFs constructed from percentile bins.
///
/// The percentiles define 10 bins:
///   [min..p1], (p1..p5], (p5..p10], (p10..p25], (p25..p50],
///   (p50..p75], (p75..p90], (p90..p95], (p95..p99], (p99..max]
///
/// Each bin has a known probability mass (e.g., bin (p1..p5] has mass 0.04).
/// We construct PMFs P and Q from original and synthetic percentiles respectively,
/// then compute:
///   JS(P||Q) = 0.5 * KL(P||M) + 0.5 * KL(Q||M), where M = 0.5*(P+Q)
fn js_divergence_from_percentiles(
    original: &crate::models::Percentiles,
    synthetic: &crate::models::Percentiles,
) -> f64 {
    // Bin probability masses (the probability of falling in each bin)
    // Bins: [0, p1], (p1, p5], (p5, p10], (p10, p25], (p25, p50],
    //       (p50, p75], (p75, p90], (p90, p95], (p95, p99], (p99, 1.0]
    let bin_probs: [f64; 10] = [
        0.01, // [0, p1]
        0.04, // (p1, p5]
        0.05, // (p5, p10]
        0.15, // (p10, p25]
        0.25, // (p25, p50]
        0.25, // (p50, p75]
        0.15, // (p75, p90]
        0.05, // (p90, p95]
        0.04, // (p95, p99]
        0.01, // (p99, 1.0]
    ];

    let orig_vals = original.to_array();
    let syn_vals = synthetic.to_array();

    // Compute bin widths for each distribution. The PMF is approximated
    // by density * bin_width, but since we want to compare shapes we use
    // density = probability_mass / bin_width. For JS divergence on PMFs,
    // we actually need probability masses assigned to *common* bins.
    //
    // Strategy: create a unified set of bin edges from both distributions,
    // then assign probability mass from each distribution to these common bins.

    // Collect all unique bin edges from both distributions, sorted
    let orig_edges = percentile_bin_edges(&orig_vals);
    let syn_edges = percentile_bin_edges(&syn_vals);

    let mut all_edges: Vec<f64> = orig_edges.iter().chain(syn_edges.iter()).copied().collect();
    all_edges.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_edges.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON * 100.0);

    if all_edges.len() < 2 {
        return 0.0;
    }

    // For each interval in the unified partition, compute the probability mass
    // from each distribution
    let n_bins = all_edges.len() - 1;
    let mut p_masses = vec![0.0; n_bins];
    let mut q_masses = vec![0.0; n_bins];

    for i in 0..n_bins {
        let lo = all_edges[i];
        let hi = all_edges[i + 1];
        p_masses[i] = probability_mass_in_interval(lo, hi, &orig_vals, &bin_probs);
        q_masses[i] = probability_mass_in_interval(lo, hi, &syn_vals, &bin_probs);
    }

    // Normalize to ensure they sum to 1 (they should, but floating point)
    let p_sum: f64 = p_masses.iter().sum();
    let q_sum: f64 = q_masses.iter().sum();
    if p_sum > 0.0 {
        for m in &mut p_masses {
            *m /= p_sum;
        }
    }
    if q_sum > 0.0 {
        for m in &mut q_masses {
            *m /= q_sum;
        }
    }

    // Compute JS divergence
    let mut js = 0.0;
    for i in 0..n_bins {
        let p = p_masses[i];
        let q = q_masses[i];
        let m = 0.5 * (p + q);
        if m > 0.0 {
            if p > 0.0 {
                js += 0.5 * p * (p / m).ln();
            }
            if q > 0.0 {
                js += 0.5 * q * (q / m).ln();
            }
        }
    }

    // JS divergence is bounded in [0, ln(2)] for natural log.
    // Clamp to handle floating point noise.
    js.max(0.0)
}

/// Build bin edges from percentile values. We use the percentile values
/// themselves as edges, extending slightly beyond the extremes.
fn percentile_bin_edges(values: &[f64; 9]) -> Vec<f64> {
    // Edges: slightly below p1, then p1..p99, then slightly above p99
    let margin = if (values[8] - values[0]).abs() > f64::EPSILON {
        (values[8] - values[0]) * 0.01
    } else {
        1.0
    };

    let mut edges = Vec::with_capacity(11);
    edges.push(values[0] - margin); // lower bound
    for &v in values.iter() {
        edges.push(v);
    }
    edges.push(values[8] + margin); // upper bound
    edges
}

/// Compute the probability mass a distribution assigns to the interval [lo, hi],
/// given its percentile values and the bin probability masses.
///
/// The distribution is modeled as piecewise-uniform within each percentile bin.
/// For example, between p25 and p50 there is 25% probability mass uniformly spread.
fn probability_mass_in_interval(
    lo: f64,
    hi: f64,
    percentile_vals: &[f64; 9],
    bin_probs: &[f64; 10],
) -> f64 {
    if hi <= lo {
        return 0.0;
    }

    // Build the full set of edges for this distribution
    let margin = if (percentile_vals[8] - percentile_vals[0]).abs() > f64::EPSILON {
        (percentile_vals[8] - percentile_vals[0]) * 0.01
    } else {
        1.0
    };

    let edges: [f64; 11] = [
        percentile_vals[0] - margin, // bin 0 lower
        percentile_vals[0],          // p1
        percentile_vals[1],          // p5
        percentile_vals[2],          // p10
        percentile_vals[3],          // p25
        percentile_vals[4],          // p50
        percentile_vals[5],          // p75
        percentile_vals[6],          // p90
        percentile_vals[7],          // p95
        percentile_vals[8],          // p99
        percentile_vals[8] + margin, // bin 9 upper
    ];

    let mut total_mass = 0.0;

    for i in 0..10 {
        let bin_lo = edges[i];
        let bin_hi = edges[i + 1];
        let bin_width = bin_hi - bin_lo;

        if bin_width <= 0.0 {
            // Zero-width bin: assign mass as a point mass at that location
            if lo <= bin_lo && bin_lo < hi {
                total_mass += bin_probs[i];
            }
            continue;
        }

        // Overlap of [lo, hi] with [bin_lo, bin_hi]
        let overlap_lo = lo.max(bin_lo);
        let overlap_hi = hi.min(bin_hi);
        if overlap_hi > overlap_lo {
            let overlap_fraction = (overlap_hi - overlap_lo) / bin_width;
            total_mass += bin_probs[i] * overlap_fraction;
        }
    }

    total_mass
}

/// Compute Benford's Law MAD between two distributions.
fn compute_benford_mad(original: &[f64; 9], synthetic: &[f64; 9]) -> f64 {
    let sum: f64 = original
        .iter()
        .zip(synthetic.iter())
        .map(|(&o, &s)| (o - s).abs())
        .sum();
    sum / 9.0
}

/// Generate an HTML report from fidelity results.
pub fn generate_html_report(report: &FidelityReport) -> String {
    let status_class = if report.passes { "pass" } else { "fail" };
    let status_text = if report.passes { "PASS" } else { "FAIL" };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Fidelity Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .pass {{ color: green; }}
        .fail {{ color: red; }}
        .metric {{ margin: 10px 0; }}
        .score {{ font-weight: bold; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
    </style>
</head>
<body>
    <h1>Fidelity Evaluation Report</h1>

    <div class="metric">
        <h2>Overall Score: <span class="score {}">{:.1}%</span></h2>
        <p>Status: <span class="{}">{}</span></p>
    </div>

    <h2>Component Scores</h2>
    <table>
        <tr><th>Component</th><th>Score</th></tr>
        <tr><td>Statistical Fidelity</td><td>{:.1}%</td></tr>
        <tr><td>Correlation Fidelity</td><td>{:.1}%</td></tr>
        <tr><td>Schema Fidelity</td><td>{:.1}%</td></tr>
        <tr><td>Rule Compliance</td><td>{:.1}%</td></tr>
        <tr><td>Anomaly Fidelity</td><td>{:.1}%</td></tr>
    </table>

    <h2>Details</h2>
    <p>Row count ratio: {:.2}</p>
    {}
    {}
</body>
</html>"#,
        status_class,
        report.overall_score * 100.0,
        status_class,
        status_text,
        report.statistical_fidelity * 100.0,
        report.correlation_fidelity * 100.0,
        report.schema_fidelity * 100.0,
        report.rule_compliance * 100.0,
        report.anomaly_fidelity * 100.0,
        report.details.row_count_ratio,
        report
            .details
            .benford_mad
            .map(|m| format!("<p>Benford MAD: {:.4}</p>", m))
            .unwrap_or_default(),
        report
            .details
            .correlation_rmse
            .map(|r| format!("<p>Correlation RMSE: {:.4}</p>", r))
            .unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Percentiles;

    #[test]
    fn test_benford_mad() {
        let original = [
            0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
        ];
        let synthetic = [
            0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
        ];

        let mad = compute_benford_mad(&original, &synthetic);
        assert!(mad < 0.001); // Identical distributions
    }

    fn make_percentiles(vals: [f64; 9]) -> Percentiles {
        Percentiles::from_array(vals)
    }

    // --- Wasserstein distance tests ---

    #[test]
    fn test_wasserstein_identical_distributions() {
        let pcts = make_percentiles([1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0]);
        let w = wasserstein_distance_from_percentiles(&pcts, &pcts);
        assert!(
            w.abs() < 1e-10,
            "Wasserstein distance between identical distributions should be ~0, got {}",
            w
        );
    }

    #[test]
    fn test_wasserstein_shifted_distributions() {
        let orig = make_percentiles([1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0]);
        // Shift by +10 everywhere
        let shifted = make_percentiles([11.0, 15.0, 20.0, 35.0, 60.0, 85.0, 100.0, 105.0, 109.0]);
        let w = wasserstein_distance_from_percentiles(&orig, &shifted);
        // The distance should be positive and roughly equal to the average shift (~10)
        assert!(
            w > 5.0,
            "Expected W1 > 5 for shifted distributions, got {}",
            w
        );
    }

    #[test]
    fn test_wasserstein_symmetry() {
        let a = make_percentiles([1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0]);
        let b = make_percentiles([2.0, 8.0, 15.0, 30.0, 55.0, 78.0, 92.0, 97.0, 100.0]);
        let w_ab = wasserstein_distance_from_percentiles(&a, &b);
        let w_ba = wasserstein_distance_from_percentiles(&b, &a);
        assert!(
            (w_ab - w_ba).abs() < 1e-10,
            "Wasserstein distance should be symmetric: {} vs {}",
            w_ab,
            w_ba
        );
    }

    #[test]
    fn test_wasserstein_constant_shift() {
        // If one distribution is just shifted by a constant c,
        // W1 should be approximately c.
        let orig = make_percentiles([10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]);
        let shifted = make_percentiles([15.0, 25.0, 35.0, 45.0, 55.0, 65.0, 75.0, 85.0, 95.0]);
        let w = wasserstein_distance_from_percentiles(&orig, &shifted);
        assert!(
            (w - 5.0).abs() < 0.5,
            "For a constant shift of 5, expected W1 ~ 5.0, got {}",
            w
        );
    }

    #[test]
    fn test_wasserstein_non_negative() {
        let a = make_percentiles([0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 80.0, 100.0]);
        let b = make_percentiles([1.0, 3.0, 5.0, 10.0, 20.0, 40.0, 60.0, 85.0, 99.0]);
        let w = wasserstein_distance_from_percentiles(&a, &b);
        assert!(
            w >= 0.0,
            "Wasserstein distance should be non-negative, got {}",
            w
        );
    }

    // --- Jensen-Shannon divergence tests ---

    #[test]
    fn test_js_identical_distributions() {
        let pcts = make_percentiles([1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0]);
        let js = js_divergence_from_percentiles(&pcts, &pcts);
        assert!(
            js.abs() < 1e-10,
            "JS divergence between identical distributions should be ~0, got {}",
            js
        );
    }

    #[test]
    fn test_js_symmetry() {
        let a = make_percentiles([1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0]);
        let b = make_percentiles([2.0, 8.0, 15.0, 30.0, 55.0, 78.0, 92.0, 97.0, 100.0]);
        let js_ab = js_divergence_from_percentiles(&a, &b);
        let js_ba = js_divergence_from_percentiles(&b, &a);
        assert!(
            (js_ab - js_ba).abs() < 1e-10,
            "JS divergence should be symmetric: {} vs {}",
            js_ab,
            js_ba
        );
    }

    #[test]
    fn test_js_non_negative() {
        let a = make_percentiles([0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 80.0, 100.0]);
        let b = make_percentiles([1.0, 3.0, 5.0, 10.0, 20.0, 40.0, 60.0, 85.0, 99.0]);
        let js = js_divergence_from_percentiles(&a, &b);
        assert!(
            js >= 0.0,
            "JS divergence should be non-negative, got {}",
            js
        );
    }

    #[test]
    fn test_js_bounded_by_ln2() {
        // JS divergence (with natural log) is bounded by ln(2) ~ 0.693
        let a = make_percentiles([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let b = make_percentiles([
            100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0,
        ]);
        let js = js_divergence_from_percentiles(&a, &b);
        assert!(
            js <= std::f64::consts::LN_2 + 0.01,
            "JS divergence should be <= ln(2) ~ 0.693, got {}",
            js
        );
    }

    #[test]
    fn test_js_different_distributions_positive() {
        let a = make_percentiles([1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0]);
        let b = make_percentiles([10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0]);
        let js = js_divergence_from_percentiles(&a, &b);
        assert!(
            js > 0.0,
            "JS divergence between different distributions should be > 0, got {}",
            js
        );
    }

    // --- Inverse CDF interpolation tests ---

    #[test]
    fn test_interp_inv_cdf_at_knots() {
        let values = [1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0];
        // At each percentile probability, we should get back the percentile value
        for (i, &prob) in PERCENTILE_PROBS.iter().enumerate() {
            let result = interp_inv_cdf(prob, &values);
            assert!(
                (result - values[i]).abs() < 1e-10,
                "At p={}, expected {}, got {}",
                prob,
                values[i],
                result
            );
        }
    }

    #[test]
    fn test_interp_inv_cdf_interpolation() {
        let values = [0.0, 10.0, 20.0, 30.0, 50.0, 70.0, 80.0, 90.0, 100.0];
        // At p=0.03 (midpoint between p1=0.01 and p5=0.05), should interpolate
        let result = interp_inv_cdf(0.03, &values);
        // Linear interpolation: 0.0 + (10.0-0.0) * (0.03-0.01)/(0.05-0.01) = 5.0
        assert!((result - 5.0).abs() < 1e-10, "Expected 5.0, got {}", result);
    }

    // --- Integration: compare_numeric_stats populates W1 and JS ---

    #[test]
    fn test_compare_numeric_stats_populates_metrics() {
        let orig = NumericStats {
            count: 1000,
            min: 0.0,
            max: 100.0,
            mean: 50.0,
            std_dev: 15.0,
            percentiles: make_percentiles([1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0]),
            distribution: crate::models::DistributionType::Normal,
            distribution_params: crate::models::DistributionParams::normal(50.0, 15.0),
            zero_rate: 0.0,
            negative_rate: 0.0,
            benford_first_digit: None,
        };

        let syn = NumericStats {
            count: 1000,
            min: 2.0,
            max: 98.0,
            mean: 52.0,
            std_dev: 14.0,
            percentiles: make_percentiles([2.0, 6.0, 12.0, 27.0, 52.0, 77.0, 91.0, 96.0, 98.0]),
            distribution: crate::models::DistributionType::Normal,
            distribution_params: crate::models::DistributionParams::normal(52.0, 14.0),
            zero_rate: 0.0,
            negative_rate: 0.0,
            benford_first_digit: None,
        };

        let evaluator = FidelityEvaluator::new();
        let metrics = evaluator.compare_numeric_stats("test_col", &orig, &syn);

        // W1 should be small but positive (distributions are similar but shifted)
        assert!(
            metrics.wasserstein_distance > 0.0,
            "W1 should be positive for different distributions"
        );
        assert!(
            metrics.wasserstein_distance < 10.0,
            "W1 should be modest for similar distributions, got {}",
            metrics.wasserstein_distance
        );

        // JS should be small but positive
        assert!(metrics.js_divergence >= 0.0, "JS should be non-negative");
        assert!(
            metrics.js_divergence < 0.5,
            "JS should be modest for similar distributions, got {}",
            metrics.js_divergence
        );
    }
}
