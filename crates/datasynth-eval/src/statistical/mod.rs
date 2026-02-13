//! Statistical quality evaluation module.
//!
//! Provides statistical tests and analyses for validating that generated
//! synthetic data follows expected distributions.
//!
//! # Modules
//!
//! - **amount_distribution**: Log-normal amount distribution analysis
//! - **benford**: Benford's Law compliance testing
//! - **line_item**: Line item distribution analysis
//! - **temporal**: Temporal pattern analysis
//! - **correlation**: Cross-field correlation analysis
//! - **anderson_darling**: Anderson-Darling goodness-of-fit test
//! - **chi_squared**: Chi-squared goodness-of-fit test
//! - **drift_detection**: Drift detection evaluation and ground truth validation

mod amount_distribution;
mod anderson_darling;
mod anomaly_realism;
mod benford;
mod chi_squared;
mod correlation;
mod drift_detection;
mod line_item;
mod temporal;

pub use amount_distribution::{AmountDistributionAnalysis, AmountDistributionAnalyzer};
pub use anderson_darling::{
    AndersonDarlingAnalysis, AndersonDarlingAnalyzer, CriticalValues, FittedParameters,
    TargetDistribution,
};
pub use benford::{BenfordAnalysis, BenfordAnalyzer, BenfordConformity};
pub use chi_squared::{
    BinFrequency, BinningStrategy, ChiSquaredAnalysis, ChiSquaredAnalyzer, ExpectedDistribution,
};
pub use correlation::{
    pearson_correlation, spearman_correlation, CorrelationAnalysis, CorrelationAnalyzer,
    CorrelationCheckResult, ExpectedCorrelation,
};
pub use drift_detection::{
    DetectionDifficulty, DriftDetectionAnalysis, DriftDetectionAnalyzer, DriftDetectionEntry,
    DriftDetectionMetrics, DriftEventCategory, LabeledDriftEvent, LabeledEventAnalysis,
};
pub use line_item::{LineItemAnalysis, LineItemAnalyzer, LineItemEntry};
pub use temporal::{TemporalAnalysis, TemporalAnalyzer, TemporalEntry};

pub use anomaly_realism::{
    AnomalyData, AnomalyRealismEvaluation, AnomalyRealismEvaluator, AnomalyRealismThresholds,
};

use serde::{Deserialize, Serialize};

/// Combined statistical evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalEvaluation {
    /// Benford's Law analysis results.
    pub benford: Option<BenfordAnalysis>,
    /// Amount distribution analysis results.
    pub amount_distribution: Option<AmountDistributionAnalysis>,
    /// Line item distribution analysis results.
    pub line_item: Option<LineItemAnalysis>,
    /// Temporal pattern analysis results.
    pub temporal: Option<TemporalAnalysis>,
    /// Correlation analysis results.
    pub correlation: Option<CorrelationAnalysis>,
    /// Anderson-Darling goodness-of-fit test results.
    pub anderson_darling: Option<AndersonDarlingAnalysis>,
    /// Chi-squared goodness-of-fit test results.
    pub chi_squared: Option<ChiSquaredAnalysis>,
    /// Drift detection analysis results.
    pub drift_detection: Option<DriftDetectionAnalysis>,
    /// Labeled drift event analysis results.
    pub drift_events: Option<LabeledEventAnalysis>,
    /// Anomaly injection realism analysis results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anomaly_realism: Option<AnomalyRealismEvaluation>,
    /// Overall pass/fail status.
    pub passes: bool,
    /// Summary of failed checks.
    pub failures: Vec<String>,
    /// Summary of issues (alias for failures).
    pub issues: Vec<String>,
    /// Overall statistical quality score (0.0-1.0).
    pub overall_score: f64,
}

impl StatisticalEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            benford: None,
            amount_distribution: None,
            line_item: None,
            temporal: None,
            correlation: None,
            anderson_darling: None,
            chi_squared: None,
            drift_detection: None,
            drift_events: None,
            anomaly_realism: None,
            passes: true,
            failures: Vec::new(),
            issues: Vec::new(),
            overall_score: 1.0,
        }
    }

    /// Check all results against thresholds and update pass status.
    pub fn check_thresholds(&mut self, thresholds: &crate::config::EvaluationThresholds) {
        self.failures.clear();
        self.issues.clear();
        let mut scores = Vec::new();

        if let Some(ref benford) = self.benford {
            if benford.p_value < thresholds.benford_p_value_min {
                self.failures.push(format!(
                    "Benford p-value {} < {} (threshold)",
                    benford.p_value, thresholds.benford_p_value_min
                ));
            }
            if benford.mad > thresholds.benford_mad_max {
                self.failures.push(format!(
                    "Benford MAD {} > {} (threshold)",
                    benford.mad, thresholds.benford_mad_max
                ));
            }
            // Benford score: higher p-value and lower MAD are better
            let p_score = (benford.p_value / 0.5).min(1.0);
            let mad_score = 1.0 - (benford.mad / 0.05).min(1.0);
            scores.push((p_score + mad_score) / 2.0);
        }

        if let Some(ref amount) = self.amount_distribution {
            if let Some(p_value) = amount.lognormal_ks_pvalue {
                if p_value < thresholds.amount_ks_p_value_min {
                    self.failures.push(format!(
                        "Amount KS p-value {} < {} (threshold)",
                        p_value, thresholds.amount_ks_p_value_min
                    ));
                }
                scores.push((p_value / 0.5).min(1.0));
            }
        }

        if let Some(ref temporal) = self.temporal {
            if temporal.pattern_correlation < thresholds.temporal_correlation_min {
                self.failures.push(format!(
                    "Temporal correlation {} < {} (threshold)",
                    temporal.pattern_correlation, thresholds.temporal_correlation_min
                ));
            }
            scores.push(temporal.pattern_correlation);
        }

        // Check correlation analysis
        if let Some(ref correlation) = self.correlation {
            if !correlation.passes {
                for issue in &correlation.issues {
                    self.failures.push(format!("Correlation: {}", issue));
                }
            }
            // Score based on pass rate
            let total_checks = correlation.checks_passed + correlation.checks_failed;
            if total_checks > 0 {
                scores.push(correlation.checks_passed as f64 / total_checks as f64);
            }
        }

        // Check Anderson-Darling test
        if let Some(ref ad) = self.anderson_darling {
            if !ad.passes {
                for issue in &ad.issues {
                    self.failures.push(format!("Anderson-Darling: {}", issue));
                }
            }
            // Score based on p-value (higher is better for goodness-of-fit)
            scores.push((ad.p_value / 0.5).min(1.0));
        }

        // Check Chi-squared test
        if let Some(ref chi_sq) = self.chi_squared {
            if !chi_sq.passes {
                for issue in &chi_sq.issues {
                    self.failures.push(format!("Chi-squared: {}", issue));
                }
            }
            // Score based on p-value (higher is better for goodness-of-fit)
            scores.push((chi_sq.p_value / 0.5).min(1.0));
        }

        // Check drift detection
        if let Some(ref drift) = self.drift_detection {
            if !drift.passes {
                for issue in &drift.issues {
                    self.failures.push(format!("Drift detection: {}", issue));
                }
            }
            // Score based on F1 score if drift was significant
            if drift.drift_magnitude >= thresholds.drift_magnitude_min {
                scores.push(drift.detection_metrics.f1_score);
            }
            // Check Hellinger distance threshold
            if let Some(hellinger) = drift.hellinger_distance {
                if hellinger > thresholds.drift_hellinger_max {
                    self.failures.push(format!(
                        "Drift Hellinger distance {} > {} (threshold)",
                        hellinger, thresholds.drift_hellinger_max
                    ));
                }
            }
            // Check PSI threshold
            if let Some(psi) = drift.psi {
                if psi > thresholds.drift_psi_max {
                    self.failures.push(format!(
                        "Drift PSI {} > {} (threshold)",
                        psi, thresholds.drift_psi_max
                    ));
                }
            }
        }

        // Check labeled drift events
        if let Some(ref events) = self.drift_events {
            if !events.passes {
                for issue in &events.issues {
                    self.failures.push(format!("Drift events: {}", issue));
                }
            }
            // Score based on event coverage
            if events.total_events > 0 {
                let difficulty_score = 1.0 - events.avg_difficulty;
                scores.push(difficulty_score);
            }
        }

        // Check anomaly realism
        if let Some(ref anomaly_realism) = self.anomaly_realism {
            if !anomaly_realism.passes {
                for issue in &anomaly_realism.issues {
                    self.failures.push(format!("Anomaly realism: {}", issue));
                }
            }
            // Score based on detectability
            scores.push(anomaly_realism.statistical_detectability);
        }

        // Sync issues with failures
        self.issues = self.failures.clone();
        self.passes = self.failures.is_empty();

        // Calculate overall score
        self.overall_score = if scores.is_empty() {
            1.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };
    }
}

impl Default for StatisticalEvaluation {
    fn default() -> Self {
        Self::new()
    }
}
