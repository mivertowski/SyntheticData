//! Threshold checking for pass/fail determination.
//!
//! Validates metrics against configured thresholds and generates
//! pass/fail results with detailed feedback.

use crate::config::EvaluationThresholds;
use serde::{Deserialize, Serialize};

/// Result of threshold checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdResult {
    /// Metric name.
    pub metric_name: String,
    /// Actual value.
    pub actual_value: f64,
    /// Threshold value.
    pub threshold_value: f64,
    /// Comparison operator.
    pub operator: ThresholdOperator,
    /// Whether threshold was met.
    pub passed: bool,
    /// Human-readable explanation.
    pub explanation: String,
}

/// Threshold comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdOperator {
    /// Greater than or equal.
    GreaterOrEqual,
    /// Less than or equal.
    LessOrEqual,
    /// Greater than.
    GreaterThan,
    /// Less than.
    LessThan,
    /// Equal (with tolerance).
    Equal,
    /// Within range.
    InRange,
}

/// Checker for threshold validation.
pub struct ThresholdChecker {
    /// Thresholds to check against.
    thresholds: EvaluationThresholds,
}

impl ThresholdChecker {
    /// Create a new checker with the specified thresholds.
    pub fn new(thresholds: EvaluationThresholds) -> Self {
        Self { thresholds }
    }

    /// Check a single metric against a minimum threshold.
    pub fn check_min(&self, name: &str, actual: f64, threshold: f64) -> ThresholdResult {
        let passed = actual >= threshold;
        ThresholdResult {
            metric_name: name.to_string(),
            actual_value: actual,
            threshold_value: threshold,
            operator: ThresholdOperator::GreaterOrEqual,
            passed,
            explanation: if passed {
                format!("{name} ({actual:.4}) >= {threshold} (threshold)")
            } else {
                format!("{name} ({actual:.4}) < {threshold} (threshold) - FAILED")
            },
        }
    }

    /// Check a single metric against a maximum threshold.
    pub fn check_max(&self, name: &str, actual: f64, threshold: f64) -> ThresholdResult {
        let passed = actual <= threshold;
        ThresholdResult {
            metric_name: name.to_string(),
            actual_value: actual,
            threshold_value: threshold,
            operator: ThresholdOperator::LessOrEqual,
            passed,
            explanation: if passed {
                format!("{name} ({actual:.4}) <= {threshold} (threshold)")
            } else {
                format!("{name} ({actual:.4}) > {threshold} (threshold) - FAILED")
            },
        }
    }

    /// Check a metric is within a range.
    pub fn check_range(&self, name: &str, actual: f64, min: f64, max: f64) -> ThresholdResult {
        let passed = actual >= min && actual <= max;
        ThresholdResult {
            metric_name: name.to_string(),
            actual_value: actual,
            threshold_value: (min + max) / 2.0,
            operator: ThresholdOperator::InRange,
            passed,
            explanation: if passed {
                format!("{name} ({actual:.4}) in range [{min}, {max}]")
            } else {
                format!("{name} ({actual:.4}) outside range [{min}, {max}] - FAILED")
            },
        }
    }

    /// Check all statistical thresholds.
    pub fn check_statistical(
        &self,
        benford_p: Option<f64>,
        benford_mad: Option<f64>,
        temporal_corr: Option<f64>,
    ) -> Vec<ThresholdResult> {
        let mut results = Vec::new();

        if let Some(p) = benford_p {
            results.push(self.check_min("benford_p_value", p, self.thresholds.benford_p_value_min));
        }

        if let Some(mad) = benford_mad {
            results.push(self.check_max("benford_mad", mad, self.thresholds.benford_mad_max));
        }

        if let Some(corr) = temporal_corr {
            results.push(self.check_min(
                "temporal_correlation",
                corr,
                self.thresholds.temporal_correlation_min,
            ));
        }

        results
    }

    /// Check all coherence thresholds.
    pub fn check_coherence(
        &self,
        balance_imbalance: Option<f64>,
        subledger_rate: Option<f64>,
        doc_chain_rate: Option<f64>,
        ic_match_rate: Option<f64>,
    ) -> Vec<ThresholdResult> {
        let mut results = Vec::new();

        if let Some(imb) = balance_imbalance {
            let tolerance = self
                .thresholds
                .balance_tolerance
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.01);
            results.push(self.check_max("balance_imbalance", imb, tolerance));
        }

        if let Some(rate) = subledger_rate {
            results.push(self.check_min(
                "subledger_reconciliation",
                rate,
                self.thresholds.subledger_reconciliation_rate_min,
            ));
        }

        if let Some(rate) = doc_chain_rate {
            results.push(self.check_min(
                "document_chain_completion",
                rate,
                self.thresholds.document_chain_completion_min,
            ));
        }

        if let Some(rate) = ic_match_rate {
            results.push(self.check_min("ic_match_rate", rate, self.thresholds.ic_match_rate_min));
        }

        results
    }

    /// Check all quality thresholds.
    pub fn check_quality(
        &self,
        duplicate_rate: Option<f64>,
        completeness: Option<f64>,
        format_consistency: Option<f64>,
    ) -> Vec<ThresholdResult> {
        let mut results = Vec::new();

        if let Some(rate) = duplicate_rate {
            results.push(self.check_max(
                "duplicate_rate",
                rate,
                self.thresholds.duplicate_rate_max,
            ));
        }

        if let Some(comp) = completeness {
            results.push(self.check_min(
                "completeness",
                comp,
                self.thresholds.completeness_rate_min,
            ));
        }

        if let Some(fmt) = format_consistency {
            results.push(self.check_min(
                "format_consistency",
                fmt,
                self.thresholds.format_consistency_min,
            ));
        }

        results
    }

    /// Check all ML thresholds.
    pub fn check_ml(
        &self,
        anomaly_rate: Option<f64>,
        label_coverage: Option<f64>,
        graph_connectivity: Option<f64>,
    ) -> Vec<ThresholdResult> {
        let mut results = Vec::new();

        if let Some(rate) = anomaly_rate {
            results.push(self.check_range(
                "anomaly_rate",
                rate,
                self.thresholds.anomaly_rate_min,
                self.thresholds.anomaly_rate_max,
            ));
        }

        if let Some(cov) = label_coverage {
            results.push(self.check_min("label_coverage", cov, self.thresholds.label_coverage_min));
        }

        if let Some(conn) = graph_connectivity {
            results.push(self.check_min(
                "graph_connectivity",
                conn,
                self.thresholds.graph_connectivity_min,
            ));
        }

        results
    }

    /// Get all threshold results.
    pub fn check_all(
        &self,
        benford_p: Option<f64>,
        benford_mad: Option<f64>,
        temporal_corr: Option<f64>,
        balance_imbalance: Option<f64>,
        subledger_rate: Option<f64>,
        doc_chain_rate: Option<f64>,
        ic_match_rate: Option<f64>,
        duplicate_rate: Option<f64>,
        completeness: Option<f64>,
        format_consistency: Option<f64>,
        anomaly_rate: Option<f64>,
        label_coverage: Option<f64>,
        graph_connectivity: Option<f64>,
    ) -> Vec<ThresholdResult> {
        let mut all = Vec::new();
        all.extend(self.check_statistical(benford_p, benford_mad, temporal_corr));
        all.extend(self.check_coherence(
            balance_imbalance,
            subledger_rate,
            doc_chain_rate,
            ic_match_rate,
        ));
        all.extend(self.check_quality(duplicate_rate, completeness, format_consistency));
        all.extend(self.check_ml(anomaly_rate, label_coverage, graph_connectivity));
        all
    }

    /// Check if all results pass.
    pub fn all_pass(results: &[ThresholdResult]) -> bool {
        results.iter().all(|r| r.passed)
    }
}

impl Default for ThresholdChecker {
    fn default() -> Self {
        Self::new(EvaluationThresholds::default())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_check_min() {
        let checker = ThresholdChecker::default();
        let result = checker.check_min("test_metric", 0.95, 0.90);
        assert!(result.passed);
    }

    #[test]
    fn test_check_min_fail() {
        let checker = ThresholdChecker::default();
        let result = checker.check_min("test_metric", 0.85, 0.90);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_max() {
        let checker = ThresholdChecker::default();
        let result = checker.check_max("test_metric", 0.05, 0.10);
        assert!(result.passed);
    }

    #[test]
    fn test_check_range() {
        let checker = ThresholdChecker::default();
        let result = checker.check_range("test_metric", 0.10, 0.05, 0.15);
        assert!(result.passed);

        let result2 = checker.check_range("test_metric", 0.20, 0.05, 0.15);
        assert!(!result2.passed);
    }
}
