//! Baseline comparison for evaluation reports.
//!
//! Compares current evaluation results against a baseline to track
//! improvements or regressions over time.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Direction of metric change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeDirection {
    /// Metric improved.
    Improved,
    /// Metric regressed.
    Regressed,
    /// No significant change.
    Unchanged,
}

/// Significance of the change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeSeverity {
    /// Critical change requiring attention.
    Critical,
    /// Notable change.
    Notable,
    /// Minor change.
    Minor,
    /// Negligible change.
    Negligible,
}

/// A single metric change between baseline and current.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricChange {
    /// Metric name.
    pub metric_name: String,
    /// Metric category (e.g., "statistical", "coherence").
    pub category: String,
    /// Baseline value.
    pub baseline_value: f64,
    /// Current value.
    pub current_value: f64,
    /// Absolute change (current - baseline).
    pub absolute_change: f64,
    /// Percentage change ((current - baseline) / baseline * 100).
    pub percent_change: f64,
    /// Direction of change.
    pub direction: ChangeDirection,
    /// Severity of change.
    pub severity: ChangeSeverity,
    /// Whether higher values are better for this metric.
    pub higher_is_better: bool,
}

impl MetricChange {
    /// Create a new metric change.
    pub fn new(
        metric_name: impl Into<String>,
        category: impl Into<String>,
        baseline_value: f64,
        current_value: f64,
        higher_is_better: bool,
    ) -> Self {
        let absolute_change = current_value - baseline_value;
        let percent_change = if baseline_value.abs() > 1e-10 {
            (absolute_change / baseline_value) * 100.0
        } else if current_value.abs() > 1e-10 {
            100.0 // From zero to non-zero
        } else {
            0.0 // Both zero
        };

        // Determine direction based on whether higher is better
        let direction = if absolute_change.abs() < 1e-6 {
            ChangeDirection::Unchanged
        } else if (absolute_change > 0.0) == higher_is_better {
            ChangeDirection::Improved
        } else {
            ChangeDirection::Regressed
        };

        // Determine severity based on percent change
        let severity = match percent_change.abs() {
            x if x >= 20.0 => ChangeSeverity::Critical,
            x if x >= 10.0 => ChangeSeverity::Notable,
            x if x >= 2.0 => ChangeSeverity::Minor,
            _ => ChangeSeverity::Negligible,
        };

        Self {
            metric_name: metric_name.into(),
            category: category.into(),
            baseline_value,
            current_value,
            absolute_change,
            percent_change,
            direction,
            severity,
            higher_is_better,
        }
    }

    /// Check if this change is a regression.
    pub fn is_regression(&self) -> bool {
        self.direction == ChangeDirection::Regressed
    }

    /// Check if this change is an improvement.
    pub fn is_improvement(&self) -> bool {
        self.direction == ChangeDirection::Improved
    }

    /// Check if this change is significant (notable or critical).
    pub fn is_significant(&self) -> bool {
        matches!(
            self.severity,
            ChangeSeverity::Critical | ChangeSeverity::Notable
        )
    }
}

/// Result of comparing current evaluation against baseline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Individual metric changes.
    pub metric_changes: Vec<MetricChange>,
    /// Number of improved metrics.
    pub improvements: usize,
    /// Number of regressed metrics.
    pub regressions: usize,
    /// Number of unchanged metrics.
    pub unchanged: usize,
    /// Number of critical regressions.
    pub critical_regressions: usize,
    /// Overall comparison summary.
    pub summary: ComparisonSummary,
}

/// Summary of comparison results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonSummary {
    /// Overall improvement.
    Improved,
    /// Overall regression.
    Regressed,
    /// Mixed results.
    Mixed,
    /// No significant changes.
    Stable,
}

impl ComparisonResult {
    /// Create a new comparison result from metric changes.
    pub fn from_changes(metric_changes: Vec<MetricChange>) -> Self {
        let improvements = metric_changes.iter().filter(|c| c.is_improvement()).count();
        let regressions = metric_changes.iter().filter(|c| c.is_regression()).count();
        let unchanged = metric_changes.len() - improvements - regressions;
        let critical_regressions = metric_changes
            .iter()
            .filter(|c| c.is_regression() && c.severity == ChangeSeverity::Critical)
            .count();

        let summary = if critical_regressions > 0 {
            ComparisonSummary::Regressed
        } else if regressions == 0 && improvements > 0 {
            ComparisonSummary::Improved
        } else if regressions > 0 && improvements > 0 {
            ComparisonSummary::Mixed
        } else {
            ComparisonSummary::Stable
        };

        Self {
            metric_changes,
            improvements,
            regressions,
            unchanged,
            critical_regressions,
            summary,
        }
    }

    /// Get all regressions.
    pub fn get_regressions(&self) -> Vec<&MetricChange> {
        self.metric_changes
            .iter()
            .filter(|c| c.is_regression())
            .collect()
    }

    /// Get all improvements.
    pub fn get_improvements(&self) -> Vec<&MetricChange> {
        self.metric_changes
            .iter()
            .filter(|c| c.is_improvement())
            .collect()
    }

    /// Get significant changes only.
    pub fn get_significant_changes(&self) -> Vec<&MetricChange> {
        self.metric_changes
            .iter()
            .filter(|c| c.is_significant())
            .collect()
    }

    /// Get changes by category.
    pub fn get_by_category(&self, category: &str) -> Vec<&MetricChange> {
        self.metric_changes
            .iter()
            .filter(|c| c.category == category)
            .collect()
    }
}

/// Baseline metrics for comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    /// Baseline report metadata.
    pub baseline_source: String,
    /// When baseline was recorded.
    pub baseline_timestamp: String,
    /// Comparison results.
    pub comparison: ComparisonResult,
}

impl BaselineComparison {
    /// Create a new baseline comparison.
    pub fn new(
        baseline_source: impl Into<String>,
        baseline_timestamp: impl Into<String>,
        comparison: ComparisonResult,
    ) -> Self {
        Self {
            baseline_source: baseline_source.into(),
            baseline_timestamp: baseline_timestamp.into(),
            comparison,
        }
    }
}

/// Compares evaluation reports against baselines.
#[allow(dead_code)] // Reserved for baseline comparison feature
pub struct BaselineComparer {
    /// Metric definitions with higher_is_better flags.
    metric_definitions: HashMap<String, MetricDefinition>,
    /// Threshold for considering a change significant.
    significance_threshold: f64,
}

#[allow(dead_code)]
#[derive(Clone)]
struct MetricDefinition {
    category: String,
    higher_is_better: bool,
}

#[allow(dead_code)]
impl BaselineComparer {
    /// Create a new baseline comparer with default metric definitions.
    pub fn new() -> Self {
        let mut definitions = HashMap::new();

        // Statistical metrics (higher p-values are better)
        definitions.insert(
            "benford_p_value".to_string(),
            MetricDefinition {
                category: "statistical".to_string(),
                higher_is_better: true,
            },
        );
        definitions.insert(
            "benford_mad".to_string(),
            MetricDefinition {
                category: "statistical".to_string(),
                higher_is_better: false, // Lower MAD is better
            },
        );
        definitions.insert(
            "amount_ks_p_value".to_string(),
            MetricDefinition {
                category: "statistical".to_string(),
                higher_is_better: true,
            },
        );
        definitions.insert(
            "temporal_correlation".to_string(),
            MetricDefinition {
                category: "statistical".to_string(),
                higher_is_better: true,
            },
        );

        // Coherence metrics (higher is better)
        definitions.insert(
            "balance_sheet_balanced".to_string(),
            MetricDefinition {
                category: "coherence".to_string(),
                higher_is_better: true,
            },
        );
        definitions.insert(
            "subledger_reconciliation".to_string(),
            MetricDefinition {
                category: "coherence".to_string(),
                higher_is_better: true,
            },
        );
        definitions.insert(
            "document_chain_completion".to_string(),
            MetricDefinition {
                category: "coherence".to_string(),
                higher_is_better: true,
            },
        );
        definitions.insert(
            "ic_match_rate".to_string(),
            MetricDefinition {
                category: "coherence".to_string(),
                higher_is_better: true,
            },
        );

        // Quality metrics
        definitions.insert(
            "duplicate_rate".to_string(),
            MetricDefinition {
                category: "quality".to_string(),
                higher_is_better: false, // Lower is better
            },
        );
        definitions.insert(
            "completeness".to_string(),
            MetricDefinition {
                category: "quality".to_string(),
                higher_is_better: true,
            },
        );
        definitions.insert(
            "format_consistency".to_string(),
            MetricDefinition {
                category: "quality".to_string(),
                higher_is_better: true,
            },
        );

        // ML metrics
        definitions.insert(
            "anomaly_rate".to_string(),
            MetricDefinition {
                category: "ml".to_string(),
                higher_is_better: true, // We want anomalies for training
            },
        );
        definitions.insert(
            "label_coverage".to_string(),
            MetricDefinition {
                category: "ml".to_string(),
                higher_is_better: true,
            },
        );
        definitions.insert(
            "graph_connectivity".to_string(),
            MetricDefinition {
                category: "ml".to_string(),
                higher_is_better: true,
            },
        );

        Self {
            metric_definitions: definitions,
            significance_threshold: 2.0, // 2% change is significant
        }
    }

    /// Set significance threshold (in percent).
    pub fn with_significance_threshold(mut self, threshold: f64) -> Self {
        self.significance_threshold = threshold;
        self
    }

    /// Add a custom metric definition.
    pub fn add_metric(
        &mut self,
        name: impl Into<String>,
        category: impl Into<String>,
        higher_is_better: bool,
    ) {
        self.metric_definitions.insert(
            name.into(),
            MetricDefinition {
                category: category.into(),
                higher_is_better,
            },
        );
    }

    /// Compare baseline and current metric values.
    pub fn compare(
        &self,
        baseline: &HashMap<String, f64>,
        current: &HashMap<String, f64>,
    ) -> ComparisonResult {
        let mut changes = Vec::new();

        for (metric_name, &current_value) in current {
            if let Some(&baseline_value) = baseline.get(metric_name) {
                let (category, higher_is_better) = self
                    .metric_definitions
                    .get(metric_name)
                    .map(|d| (d.category.clone(), d.higher_is_better))
                    .unwrap_or(("unknown".to_string(), true));

                changes.push(MetricChange::new(
                    metric_name.clone(),
                    category,
                    baseline_value,
                    current_value,
                    higher_is_better,
                ));
            }
        }

        ComparisonResult::from_changes(changes)
    }

    /// Create a baseline comparison from metric maps.
    pub fn create_comparison(
        &self,
        baseline_source: impl Into<String>,
        baseline_timestamp: impl Into<String>,
        baseline_metrics: &HashMap<String, f64>,
        current_metrics: &HashMap<String, f64>,
    ) -> BaselineComparison {
        let comparison = self.compare(baseline_metrics, current_metrics);
        BaselineComparison::new(baseline_source, baseline_timestamp, comparison)
    }
}

impl Default for BaselineComparer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_change_improvement() {
        let change = MetricChange::new(
            "completeness",
            "quality",
            0.90,
            0.95,
            true, // higher is better
        );

        assert!(change.is_improvement());
        assert!(!change.is_regression());
        assert_eq!(change.direction, ChangeDirection::Improved);
    }

    #[test]
    fn test_metric_change_regression() {
        let change = MetricChange::new(
            "completeness",
            "quality",
            0.95,
            0.90,
            true, // higher is better
        );

        assert!(change.is_regression());
        assert!(!change.is_improvement());
        assert_eq!(change.direction, ChangeDirection::Regressed);
    }

    #[test]
    fn test_metric_change_lower_is_better() {
        let change = MetricChange::new(
            "duplicate_rate",
            "quality",
            0.05,
            0.02,
            false, // lower is better
        );

        assert!(change.is_improvement());
        assert_eq!(change.direction, ChangeDirection::Improved);
    }

    #[test]
    fn test_comparison_result() {
        let changes = vec![
            MetricChange::new("metric1", "cat1", 0.80, 0.90, true),
            MetricChange::new("metric2", "cat1", 0.90, 0.85, true),
            MetricChange::new("metric3", "cat2", 0.95, 0.95, true),
        ];

        let result = ComparisonResult::from_changes(changes);

        assert_eq!(result.improvements, 1);
        assert_eq!(result.regressions, 1);
        assert_eq!(result.unchanged, 1);
        assert_eq!(result.summary, ComparisonSummary::Mixed);
    }

    #[test]
    fn test_baseline_comparer() {
        let comparer = BaselineComparer::new();

        let mut baseline = HashMap::new();
        baseline.insert("completeness".to_string(), 0.90);
        baseline.insert("duplicate_rate".to_string(), 0.05);

        let mut current = HashMap::new();
        current.insert("completeness".to_string(), 0.95);
        current.insert("duplicate_rate".to_string(), 0.03);

        let result = comparer.compare(&baseline, &current);

        assert_eq!(result.improvements, 2);
        assert_eq!(result.regressions, 0);
    }

    #[test]
    fn test_critical_severity() {
        let change = MetricChange::new("metric", "category", 0.50, 0.70, true);

        assert_eq!(change.severity, ChangeSeverity::Critical);
        assert!(change.percent_change >= 20.0);
    }
}
