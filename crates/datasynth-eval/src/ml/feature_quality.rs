//! Feature quality evaluation.
//!
//! Analyzes feature importance via label correlation, multicollinearity
//! via VIF (Variance Inflation Factor), and feature stability.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// A single feature vector with optional label values for importance estimation.
#[derive(Debug, Clone)]
pub struct FeatureVector {
    /// Name of this feature.
    pub feature_name: String,
    /// Observed values for this feature.
    pub values: Vec<f64>,
    /// Optional label values for correlation-based importance.
    pub label_values: Option<Vec<f64>>,
}

/// Thresholds for feature quality analysis.
#[derive(Debug, Clone)]
pub struct FeatureQualityThresholds {
    /// Minimum overall feature quality score.
    pub min_feature_quality: f64,
    /// Maximum VIF before a feature is flagged as multicollinear.
    pub max_vif: f64,
}

impl Default for FeatureQualityThresholds {
    fn default() -> Self {
        Self {
            min_feature_quality: 0.60,
            max_vif: 10.0,
        }
    }
}

/// Results of feature quality analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureQualityAnalysis {
    /// Overall feature quality score (0.0-1.0).
    pub feature_quality_score: f64,
    /// Per-feature VIF values.
    pub per_feature_vif: Vec<(String, f64)>,
    /// Features with VIF exceeding the threshold.
    pub multicollinear_features: Vec<String>,
    /// Feature importance rankings (descending by absolute correlation with label).
    pub importance_rankings: Vec<(String, f64)>,
    /// Total number of features analyzed.
    pub total_features: usize,
    /// Whether the analysis passes all thresholds.
    pub passes: bool,
    /// Issues found during analysis.
    pub issues: Vec<String>,
}

/// Analyzer for feature quality metrics.
pub struct FeatureQualityAnalyzer {
    thresholds: FeatureQualityThresholds,
}

impl FeatureQualityAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: FeatureQualityThresholds::default(),
        }
    }

    /// Create an analyzer with custom thresholds.
    pub fn with_thresholds(thresholds: FeatureQualityThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze feature quality.
    pub fn analyze(&self, features: &[FeatureVector]) -> EvalResult<FeatureQualityAnalysis> {
        let mut issues = Vec::new();
        let total_features = features.len();

        if features.is_empty() {
            return Ok(FeatureQualityAnalysis {
                feature_quality_score: 0.0,
                per_feature_vif: Vec::new(),
                multicollinear_features: Vec::new(),
                importance_rankings: Vec::new(),
                total_features: 0,
                passes: true,
                issues: vec!["No features provided".to_string()],
            });
        }

        // Compute importance rankings via Pearson correlation with label
        let mut importance_rankings: Vec<(String, f64)> = Vec::new();
        for feature in features {
            if let Some(ref label_vals) = feature.label_values {
                if let Some(corr) = pearson_correlation(&feature.values, label_vals) {
                    importance_rankings.push((feature.feature_name.clone(), corr.abs()));
                }
            }
        }
        importance_rankings
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Compute pairwise correlations for simplified VIF
        let per_feature_vif = self.compute_vif(features);
        let multicollinear_features: Vec<String> = per_feature_vif
            .iter()
            .filter(|(_, vif)| *vif > self.thresholds.max_vif)
            .map(|(name, _)| name.clone())
            .collect();

        if !multicollinear_features.is_empty() {
            issues.push(format!(
                "{} feature(s) have VIF > {:.1}: {}",
                multicollinear_features.len(),
                self.thresholds.max_vif,
                multicollinear_features.join(", ")
            ));
        }

        // Overall quality score: fraction of non-multicollinear features,
        // penalized if no importance can be computed
        let non_mc_fraction = if total_features > 0 {
            (total_features - multicollinear_features.len()) as f64 / total_features as f64
        } else {
            1.0
        };
        let importance_available = if importance_rankings.is_empty() {
            0.5 // partial penalty when label values are absent
        } else {
            1.0
        };
        let feature_quality_score = (non_mc_fraction * importance_available).clamp(0.0, 1.0);

        if feature_quality_score < self.thresholds.min_feature_quality {
            issues.push(format!(
                "Feature quality score {:.4} < {:.4} (threshold)",
                feature_quality_score, self.thresholds.min_feature_quality
            ));
        }

        let passes = issues.is_empty();

        Ok(FeatureQualityAnalysis {
            feature_quality_score,
            per_feature_vif,
            multicollinear_features,
            importance_rankings,
            total_features,
            passes,
            issues,
        })
    }

    /// Compute simplified VIF for each feature.
    ///
    /// VIF_i = 1 / (1 - R^2_i), where R^2_i is the maximum R^2 from
    /// regressing feature i on any other feature (simplified as max pairwise r^2).
    fn compute_vif(&self, features: &[FeatureVector]) -> Vec<(String, f64)> {
        let mut vifs = Vec::new();

        for (i, fi) in features.iter().enumerate() {
            let mut max_r2 = 0.0_f64;

            for (j, fj) in features.iter().enumerate() {
                if i == j {
                    continue;
                }
                if let Some(corr) = pearson_correlation(&fi.values, &fj.values) {
                    let r2 = corr * corr;
                    if r2 > max_r2 {
                        max_r2 = r2;
                    }
                }
            }

            let vif = if (1.0 - max_r2).abs() < 1e-12 {
                f64::MAX
            } else {
                1.0 / (1.0 - max_r2)
            };

            vifs.push((fi.feature_name.clone(), vif));
        }

        vifs
    }
}

/// Compute Pearson correlation between two vectors.
fn pearson_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len().min(y.len());
    if n < 3 {
        return None;
    }

    let mean_x = x[..n].iter().sum::<f64>() / n as f64;
    let mean_y = y[..n].iter().sum::<f64>() / n as f64;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    if denom < 1e-12 {
        return None;
    }

    Some(cov / denom)
}

impl Default for FeatureQualityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_features() {
        let features = vec![
            FeatureVector {
                feature_name: "amount".to_string(),
                values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
                label_values: Some(vec![0.0, 0.0, 1.0, 1.0, 1.0]),
            },
            FeatureVector {
                feature_name: "count".to_string(),
                values: vec![10.0, 20.0, 15.0, 25.0, 30.0],
                label_values: Some(vec![0.0, 0.0, 1.0, 1.0, 1.0]),
            },
        ];

        let analyzer = FeatureQualityAnalyzer::new();
        let result = analyzer.analyze(&features).unwrap();

        assert_eq!(result.total_features, 2);
        assert!(result.feature_quality_score > 0.0);
        assert!(result.passes);
    }

    #[test]
    fn test_multicollinear_features() {
        // Two features that are perfectly correlated
        let features = vec![
            FeatureVector {
                feature_name: "f1".to_string(),
                values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
                label_values: None,
            },
            FeatureVector {
                feature_name: "f2".to_string(),
                values: vec![2.0, 4.0, 6.0, 8.0, 10.0],
                label_values: None,
            },
        ];

        let analyzer = FeatureQualityAnalyzer::new();
        let result = analyzer.analyze(&features).unwrap();

        assert!(!result.multicollinear_features.is_empty());
    }

    #[test]
    fn test_empty_features() {
        let analyzer = FeatureQualityAnalyzer::new();
        let result = analyzer.analyze(&[]).unwrap();

        assert_eq!(result.total_features, 0);
        assert_eq!(result.feature_quality_score, 0.0);
    }
}
