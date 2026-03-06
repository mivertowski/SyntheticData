//! Feature distribution analysis.
//!
//! Analyzes feature distributions for ML model training suitability.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Results of feature analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAnalysis {
    /// Per-feature statistics.
    pub feature_stats: Vec<FeatureStats>,
    /// Correlation matrix (feature pairs with high correlation).
    pub high_correlations: Vec<CorrelationPair>,
    /// Zero-variance features.
    pub zero_variance_features: Vec<String>,
    /// Features with high missing rate.
    pub high_missing_features: Vec<String>,
    /// Overall feature quality score (0.0-1.0).
    pub quality_score: f64,
    /// Number of usable features.
    pub usable_features: usize,
    /// Total features analyzed.
    pub total_features: usize,
}

/// Statistics for a single feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStats {
    /// Feature name.
    pub name: String,
    /// Feature type.
    pub feature_type: FeatureType,
    /// Count of non-null values.
    pub count: usize,
    /// Missing rate.
    pub missing_rate: f64,
    /// Mean (for numeric).
    pub mean: Option<f64>,
    /// Standard deviation (for numeric).
    pub std_dev: Option<f64>,
    /// Minimum (for numeric).
    pub min: Option<f64>,
    /// Maximum (for numeric).
    pub max: Option<f64>,
    /// Skewness (for numeric).
    pub skewness: Option<f64>,
    /// Kurtosis (for numeric).
    pub kurtosis: Option<f64>,
    /// Number of unique values (for categorical).
    pub unique_values: Option<usize>,
    /// Whether feature is usable for ML.
    pub is_usable: bool,
    /// Issues with this feature.
    pub issues: Vec<String>,
}

/// Type of feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureType {
    /// Numeric (continuous).
    Numeric,
    /// Categorical (discrete).
    Categorical,
    /// Boolean.
    Boolean,
    /// Date/Time.
    DateTime,
    /// Text (requires encoding).
    Text,
}

/// A pair of highly correlated features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationPair {
    /// First feature name.
    pub feature1: String,
    /// Second feature name.
    pub feature2: String,
    /// Correlation coefficient.
    pub correlation: f64,
}

/// Input for feature analysis.
#[derive(Debug, Clone, Default)]
pub struct FeatureData {
    /// Numeric features: feature_name -> values.
    pub numeric_features: HashMap<String, Vec<Option<f64>>>,
    /// Categorical features: feature_name -> values.
    pub categorical_features: HashMap<String, Vec<Option<String>>>,
    /// Boolean features: feature_name -> values.
    pub boolean_features: HashMap<String, Vec<Option<bool>>>,
}

/// Analyzer for feature distributions.
pub struct FeatureAnalyzer {
    /// Threshold for high correlation warning.
    correlation_threshold: f64,
    /// Threshold for high missing rate warning.
    missing_threshold: f64,
    /// Maximum unique values for categorical features.
    max_categorical_cardinality: usize,
}

impl FeatureAnalyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self {
            correlation_threshold: 0.95,
            missing_threshold: 0.20,
            max_categorical_cardinality: 1000,
        }
    }

    /// Analyze feature distributions.
    pub fn analyze(&self, data: &FeatureData) -> EvalResult<FeatureAnalysis> {
        let mut feature_stats = Vec::new();
        let mut zero_variance_features = Vec::new();
        let mut high_missing_features = Vec::new();
        let mut usable_features = 0;

        // Analyze numeric features
        for (name, values) in &data.numeric_features {
            let stats = self.analyze_numeric_feature(name, values);
            if stats.std_dev == Some(0.0) {
                zero_variance_features.push(name.clone());
            }
            if stats.missing_rate > self.missing_threshold {
                high_missing_features.push(name.clone());
            }
            if stats.is_usable {
                usable_features += 1;
            }
            feature_stats.push(stats);
        }

        // Analyze categorical features
        for (name, values) in &data.categorical_features {
            let stats = self.analyze_categorical_feature(name, values);
            if stats.missing_rate > self.missing_threshold {
                high_missing_features.push(name.clone());
            }
            if stats.is_usable {
                usable_features += 1;
            }
            feature_stats.push(stats);
        }

        // Analyze boolean features
        for (name, values) in &data.boolean_features {
            let stats = self.analyze_boolean_feature(name, values);
            if stats.missing_rate > self.missing_threshold {
                high_missing_features.push(name.clone());
            }
            if stats.is_usable {
                usable_features += 1;
            }
            feature_stats.push(stats);
        }

        // Calculate correlations for numeric features
        let high_correlations = self.find_high_correlations(&data.numeric_features);

        let total_features = feature_stats.len();
        let quality_score = if total_features > 0 {
            usable_features as f64 / total_features as f64
        } else {
            1.0
        };

        Ok(FeatureAnalysis {
            feature_stats,
            high_correlations,
            zero_variance_features,
            high_missing_features,
            quality_score,
            usable_features,
            total_features,
        })
    }

    /// Analyze a numeric feature.
    fn analyze_numeric_feature(&self, name: &str, values: &[Option<f64>]) -> FeatureStats {
        let total = values.len();
        let present: Vec<f64> = values.iter().filter_map(|v| *v).collect();
        let count = present.len();
        let missing_rate = if total > 0 {
            (total - count) as f64 / total as f64
        } else {
            0.0
        };

        let mut issues = Vec::new();

        if count == 0 {
            issues.push("No non-null values".to_string());
            return FeatureStats {
                name: name.to_string(),
                feature_type: FeatureType::Numeric,
                count: 0,
                missing_rate,
                mean: None,
                std_dev: None,
                min: None,
                max: None,
                skewness: None,
                kurtosis: None,
                unique_values: None,
                is_usable: false,
                issues,
            };
        }

        let mean = present.iter().sum::<f64>() / count as f64;
        let variance: f64 = present.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        let min = present.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = present.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Skewness and kurtosis
        let (skewness, kurtosis) = if std_dev > 0.0 {
            let m3: f64 = present
                .iter()
                .map(|x| ((x - mean) / std_dev).powi(3))
                .sum::<f64>()
                / count as f64;
            let m4: f64 = present
                .iter()
                .map(|x| ((x - mean) / std_dev).powi(4))
                .sum::<f64>()
                / count as f64;
            (Some(m3), Some(m4 - 3.0)) // Excess kurtosis
        } else {
            (None, None)
        };

        // Check for issues
        if std_dev == 0.0 {
            issues.push("Zero variance".to_string());
        }
        if missing_rate > self.missing_threshold {
            issues.push(format!("High missing rate: {:.2}%", missing_rate * 100.0));
        }
        if let Some(s) = skewness {
            if s.abs() > 2.0 {
                issues.push(format!("High skewness: {s:.2}"));
            }
        }

        let is_usable = std_dev > 0.0 && missing_rate < 0.5;

        FeatureStats {
            name: name.to_string(),
            feature_type: FeatureType::Numeric,
            count,
            missing_rate,
            mean: Some(mean),
            std_dev: Some(std_dev),
            min: Some(min),
            max: Some(max),
            skewness,
            kurtosis,
            unique_values: None,
            is_usable,
            issues,
        }
    }

    /// Analyze a categorical feature.
    fn analyze_categorical_feature(&self, name: &str, values: &[Option<String>]) -> FeatureStats {
        let total = values.len();
        let present: Vec<&String> = values.iter().filter_map(|v| v.as_ref()).collect();
        let count = present.len();
        let missing_rate = if total > 0 {
            (total - count) as f64 / total as f64
        } else {
            0.0
        };

        let unique: HashSet<_> = present.iter().collect();
        let unique_count = unique.len();

        let mut issues = Vec::new();

        if unique_count == 0 {
            issues.push("No non-null values".to_string());
        } else if unique_count == 1 {
            issues.push("Only one unique value".to_string());
        } else if unique_count > self.max_categorical_cardinality {
            issues.push(format!("High cardinality: {unique_count} unique values"));
        }

        if missing_rate > self.missing_threshold {
            issues.push(format!("High missing rate: {:.2}%", missing_rate * 100.0));
        }

        let is_usable = unique_count > 1
            && unique_count <= self.max_categorical_cardinality
            && missing_rate < 0.5;

        FeatureStats {
            name: name.to_string(),
            feature_type: FeatureType::Categorical,
            count,
            missing_rate,
            mean: None,
            std_dev: None,
            min: None,
            max: None,
            skewness: None,
            kurtosis: None,
            unique_values: Some(unique_count),
            is_usable,
            issues,
        }
    }

    /// Analyze a boolean feature.
    fn analyze_boolean_feature(&self, name: &str, values: &[Option<bool>]) -> FeatureStats {
        let total = values.len();
        let present: Vec<bool> = values.iter().filter_map(|v| *v).collect();
        let count = present.len();
        let missing_rate = if total > 0 {
            (total - count) as f64 / total as f64
        } else {
            0.0
        };

        let true_count = present.iter().filter(|v| **v).count();
        let true_rate = if count > 0 {
            true_count as f64 / count as f64
        } else {
            0.0
        };

        let mut issues = Vec::new();

        if count == 0 {
            issues.push("No non-null values".to_string());
        } else if true_rate == 0.0 || true_rate == 1.0 {
            issues.push("No variance (all same value)".to_string());
        }

        if missing_rate > self.missing_threshold {
            issues.push(format!("High missing rate: {:.2}%", missing_rate * 100.0));
        }

        let is_usable = count > 0 && true_rate > 0.0 && true_rate < 1.0 && missing_rate < 0.5;

        FeatureStats {
            name: name.to_string(),
            feature_type: FeatureType::Boolean,
            count,
            missing_rate,
            mean: Some(true_rate),
            std_dev: None,
            min: Some(0.0),
            max: Some(1.0),
            skewness: None,
            kurtosis: None,
            unique_values: Some(2),
            is_usable,
            issues,
        }
    }

    /// Find highly correlated feature pairs.
    fn find_high_correlations(
        &self,
        numeric_features: &HashMap<String, Vec<Option<f64>>>,
    ) -> Vec<CorrelationPair> {
        let mut correlations = Vec::new();

        let feature_names: Vec<_> = numeric_features.keys().collect();

        for i in 0..feature_names.len() {
            for j in (i + 1)..feature_names.len() {
                let name1 = feature_names[i];
                let name2 = feature_names[j];

                if let (Some(vals1), Some(vals2)) =
                    (numeric_features.get(name1), numeric_features.get(name2))
                {
                    if let Some(corr) = self.calculate_correlation(vals1, vals2) {
                        if corr.abs() >= self.correlation_threshold {
                            correlations.push(CorrelationPair {
                                feature1: name1.clone(),
                                feature2: name2.clone(),
                                correlation: corr,
                            });
                        }
                    }
                }
            }
        }

        correlations
    }

    /// Calculate Pearson correlation between two feature vectors.
    fn calculate_correlation(&self, vals1: &[Option<f64>], vals2: &[Option<f64>]) -> Option<f64> {
        let pairs: Vec<(f64, f64)> = vals1
            .iter()
            .zip(vals2.iter())
            .filter_map(|(a, b)| match (a, b) {
                (Some(a), Some(b)) => Some((*a, *b)),
                _ => None,
            })
            .collect();

        if pairs.len() < 3 {
            return None;
        }

        let n = pairs.len() as f64;
        let mean1: f64 = pairs.iter().map(|(a, _)| a).sum::<f64>() / n;
        let mean2: f64 = pairs.iter().map(|(_, b)| b).sum::<f64>() / n;

        let cov: f64 = pairs
            .iter()
            .map(|(a, b)| (a - mean1) * (b - mean2))
            .sum::<f64>()
            / n;

        let std1 = (pairs.iter().map(|(a, _)| (a - mean1).powi(2)).sum::<f64>() / n).sqrt();
        let std2 = (pairs.iter().map(|(_, b)| (b - mean2).powi(2)).sum::<f64>() / n).sqrt();

        if std1 == 0.0 || std2 == 0.0 {
            return None;
        }

        Some(cov / (std1 * std2))
    }
}

impl Default for FeatureAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_feature() {
        let mut data = FeatureData::default();
        data.numeric_features.insert(
            "amount".to_string(),
            vec![Some(100.0), Some(200.0), Some(150.0), Some(175.0)],
        );

        let analyzer = FeatureAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.total_features, 1);
        assert_eq!(result.usable_features, 1);

        let stats = &result.feature_stats[0];
        assert!(stats.mean.is_some());
        assert!(stats.std_dev.is_some());
        assert!(stats.is_usable);
    }

    #[test]
    fn test_zero_variance_feature() {
        let mut data = FeatureData::default();
        data.numeric_features.insert(
            "constant".to_string(),
            vec![Some(100.0), Some(100.0), Some(100.0)],
        );

        let analyzer = FeatureAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.zero_variance_features.len(), 1);
        assert!(!result.feature_stats[0].is_usable);
    }

    #[test]
    fn test_categorical_feature() {
        let mut data = FeatureData::default();
        data.categorical_features.insert(
            "category".to_string(),
            vec![
                Some("A".to_string()),
                Some("B".to_string()),
                Some("A".to_string()),
            ],
        );

        let analyzer = FeatureAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        let stats = &result.feature_stats[0];
        assert_eq!(stats.unique_values, Some(2));
        assert!(stats.is_usable);
    }
}
