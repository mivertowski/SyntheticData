//! Configuration for the evaluation framework.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Main configuration for running an evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvaluationConfig {
    /// Statistical evaluation settings.
    pub statistical: StatisticalConfig,
    /// Coherence evaluation settings.
    pub coherence: CoherenceConfig,
    /// Data quality evaluation settings.
    pub quality: QualityConfig,
    /// ML-readiness evaluation settings.
    pub ml: MlConfig,
    /// Privacy evaluation settings.
    #[serde(default)]
    pub privacy: PrivacyEvaluationConfig,
    /// Report generation settings.
    pub report: ReportConfig,
    /// Pass/fail thresholds.
    pub thresholds: EvaluationThresholds,
    /// Quality gate configuration.
    #[serde(default)]
    pub quality_gates: QualityGateConfig,
}

/// Configuration for quality gates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateConfig {
    /// Whether quality gate evaluation is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Profile name: "strict", "default", "lenient", or "custom".
    #[serde(default = "default_gate_profile")]
    pub profile: String,
    /// Custom gate definitions (used when profile = "custom").
    #[serde(default)]
    pub custom_gates: Vec<CustomGateConfig>,
    /// Whether to fail the generation run on gate violations.
    #[serde(default)]
    pub fail_on_violation: bool,
}

fn default_gate_profile() -> String {
    "default".to_string()
}

impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            profile: default_gate_profile(),
            custom_gates: Vec::new(),
            fail_on_violation: false,
        }
    }
}

/// Configuration for a custom quality gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGateConfig {
    /// Gate name.
    pub name: String,
    /// Metric to check (e.g., "benford_mad", "completion_rate", "duplicate_rate").
    pub metric: String,
    /// Threshold value.
    pub threshold: f64,
    /// Upper threshold for "between" comparison.
    #[serde(default)]
    pub upper_threshold: Option<f64>,
    /// Comparison: "gte", "lte", "eq", "between".
    #[serde(default = "default_comparison")]
    pub comparison: String,
}

fn default_comparison() -> String {
    "gte".to_string()
}

/// Privacy evaluation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyEvaluationConfig {
    /// Enable membership inference attack testing.
    pub mia_enabled: bool,
    /// Enable linkage attack assessment.
    pub linkage_enabled: bool,
    /// Enable NIST SP 800-226 alignment report.
    pub nist_alignment_enabled: bool,
    /// Enable SynQP quality-privacy matrix.
    pub synqp_enabled: bool,
    /// Maximum AUC-ROC threshold for MIA (default: 0.6).
    pub mia_auc_threshold: f64,
    /// Maximum re-identification rate for linkage (default: 0.05).
    pub max_reidentification_rate: f64,
    /// Minimum k-anonymity for linkage (default: 5).
    pub min_k_anonymity: usize,
}

impl Default for PrivacyEvaluationConfig {
    fn default() -> Self {
        Self {
            mia_enabled: false,
            linkage_enabled: false,
            nist_alignment_enabled: false,
            synqp_enabled: false,
            mia_auc_threshold: 0.6,
            max_reidentification_rate: 0.05,
            min_k_anonymity: 5,
        }
    }
}

/// Statistical evaluation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalConfig {
    /// Enable Benford's Law analysis.
    pub benford_enabled: bool,
    /// Enable amount distribution analysis.
    pub amount_distribution_enabled: bool,
    /// Enable line item distribution analysis.
    pub line_item_enabled: bool,
    /// Enable temporal pattern analysis.
    pub temporal_enabled: bool,
    /// Enable drift detection analysis.
    pub drift_detection_enabled: bool,
    /// Significance level for statistical tests (default: 0.05).
    pub significance_level: f64,
    /// Minimum sample size for statistical tests.
    pub min_sample_size: usize,
    /// Window size for drift detection rolling statistics.
    pub drift_window_size: usize,
}

impl Default for StatisticalConfig {
    fn default() -> Self {
        Self {
            benford_enabled: true,
            amount_distribution_enabled: true,
            line_item_enabled: true,
            temporal_enabled: true,
            drift_detection_enabled: true,
            significance_level: 0.05,
            min_sample_size: 100,
            drift_window_size: 10,
        }
    }
}

/// Coherence evaluation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceConfig {
    /// Enable balance sheet validation.
    pub balance_enabled: bool,
    /// Enable subledger reconciliation.
    pub subledger_enabled: bool,
    /// Enable document chain validation.
    pub document_chain_enabled: bool,
    /// Enable intercompany matching validation.
    pub intercompany_enabled: bool,
    /// Enable referential integrity validation.
    pub referential_enabled: bool,
    /// Tolerance for balance differences.
    pub balance_tolerance: Decimal,
}

impl Default for CoherenceConfig {
    fn default() -> Self {
        Self {
            balance_enabled: true,
            subledger_enabled: true,
            document_chain_enabled: true,
            intercompany_enabled: true,
            referential_enabled: true,
            balance_tolerance: Decimal::new(1, 2), // 0.01
        }
    }
}

/// Data quality evaluation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Enable uniqueness validation.
    pub uniqueness_enabled: bool,
    /// Enable completeness validation.
    pub completeness_enabled: bool,
    /// Enable format consistency validation.
    pub format_enabled: bool,
    /// Enable cross-field consistency validation.
    pub consistency_enabled: bool,
    /// Similarity threshold for near-duplicate detection (0.0-1.0).
    pub near_duplicate_threshold: f64,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            uniqueness_enabled: true,
            completeness_enabled: true,
            format_enabled: true,
            consistency_enabled: true,
            near_duplicate_threshold: 0.95,
        }
    }
}

/// ML-readiness evaluation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlConfig {
    /// Enable feature distribution analysis.
    pub features_enabled: bool,
    /// Enable label quality analysis.
    pub labels_enabled: bool,
    /// Enable train/test split validation.
    pub splits_enabled: bool,
    /// Enable graph structure analysis.
    pub graph_enabled: bool,
}

impl Default for MlConfig {
    fn default() -> Self {
        Self {
            features_enabled: true,
            labels_enabled: true,
            splits_enabled: true,
            graph_enabled: true,
        }
    }
}

/// Report generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Generate HTML report.
    pub html_enabled: bool,
    /// Generate JSON report.
    pub json_enabled: bool,
    /// Include charts in HTML report.
    pub charts_enabled: bool,
    /// Path to baseline report for comparison.
    pub baseline_path: Option<String>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            html_enabled: true,
            json_enabled: true,
            charts_enabled: true,
            baseline_path: None,
        }
    }
}

/// Pass/fail thresholds for evaluation metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationThresholds {
    // Statistical thresholds
    /// Minimum p-value for Benford's Law chi-squared test.
    pub benford_p_value_min: f64,
    /// Maximum Mean Absolute Deviation for Benford's Law.
    pub benford_mad_max: f64,
    /// Minimum p-value for amount distribution KS test.
    pub amount_ks_p_value_min: f64,
    /// Minimum correlation for temporal patterns.
    pub temporal_correlation_min: f64,

    // Drift detection thresholds
    /// Minimum drift magnitude to consider significant.
    pub drift_magnitude_min: f64,
    /// Maximum Hellinger distance threshold.
    pub drift_hellinger_max: f64,
    /// Maximum Population Stability Index (PSI) threshold.
    pub drift_psi_max: f64,
    /// Minimum F1 score for drift detection quality.
    pub drift_f1_score_min: f64,

    // Coherence thresholds
    /// Maximum balance sheet imbalance.
    pub balance_tolerance: Decimal,
    /// Minimum subledger reconciliation rate.
    pub subledger_reconciliation_rate_min: f64,
    /// Minimum document chain completion rate.
    pub document_chain_completion_min: f64,
    /// Minimum intercompany match rate.
    pub ic_match_rate_min: f64,
    /// Minimum referential integrity rate.
    pub referential_integrity_min: f64,

    // Quality thresholds
    /// Maximum duplicate rate.
    pub duplicate_rate_max: f64,
    /// Minimum completeness rate.
    pub completeness_rate_min: f64,
    /// Minimum format consistency rate.
    pub format_consistency_min: f64,

    // ML thresholds
    /// Minimum anomaly rate.
    pub anomaly_rate_min: f64,
    /// Maximum anomaly rate.
    pub anomaly_rate_max: f64,
    /// Minimum label coverage.
    pub label_coverage_min: f64,
    /// Minimum train ratio.
    pub train_ratio_min: f64,
    /// Minimum graph connectivity.
    pub graph_connectivity_min: f64,
}

impl Default for EvaluationThresholds {
    fn default() -> Self {
        Self {
            // Statistical
            benford_p_value_min: 0.05,
            benford_mad_max: 0.015,
            amount_ks_p_value_min: 0.05,
            temporal_correlation_min: 0.80,

            // Drift detection
            drift_magnitude_min: 0.05,
            drift_hellinger_max: 0.30,
            drift_psi_max: 0.25,
            drift_f1_score_min: 0.50,

            // Coherence
            balance_tolerance: Decimal::new(1, 2), // 0.01
            subledger_reconciliation_rate_min: 0.99,
            document_chain_completion_min: 0.90,
            ic_match_rate_min: 0.95,
            referential_integrity_min: 0.99,

            // Quality
            duplicate_rate_max: 0.01,
            completeness_rate_min: 0.95,
            format_consistency_min: 0.99,

            // ML
            anomaly_rate_min: 0.01,
            anomaly_rate_max: 0.20,
            label_coverage_min: 0.99,
            train_ratio_min: 0.60,
            graph_connectivity_min: 0.95,
        }
    }
}

impl EvaluationThresholds {
    /// Create strict thresholds for rigorous validation.
    pub fn strict() -> Self {
        Self {
            benford_p_value_min: 0.10,
            benford_mad_max: 0.010,
            amount_ks_p_value_min: 0.10,
            temporal_correlation_min: 0.90,
            drift_magnitude_min: 0.03,
            drift_hellinger_max: 0.20,
            drift_psi_max: 0.15,
            drift_f1_score_min: 0.70,
            balance_tolerance: Decimal::new(1, 4), // 0.0001
            subledger_reconciliation_rate_min: 0.999,
            document_chain_completion_min: 0.95,
            ic_match_rate_min: 0.99,
            referential_integrity_min: 0.999,
            duplicate_rate_max: 0.001,
            completeness_rate_min: 0.99,
            format_consistency_min: 0.999,
            anomaly_rate_min: 0.01,
            anomaly_rate_max: 0.10,
            label_coverage_min: 0.999,
            train_ratio_min: 0.70,
            graph_connectivity_min: 0.99,
        }
    }

    /// Create lenient thresholds for exploratory validation.
    pub fn lenient() -> Self {
        Self {
            benford_p_value_min: 0.01,
            benford_mad_max: 0.025,
            amount_ks_p_value_min: 0.01,
            temporal_correlation_min: 0.60,
            drift_magnitude_min: 0.10,
            drift_hellinger_max: 0.50,
            drift_psi_max: 0.40,
            drift_f1_score_min: 0.30,
            balance_tolerance: Decimal::new(1, 1), // 0.1
            subledger_reconciliation_rate_min: 0.90,
            document_chain_completion_min: 0.80,
            ic_match_rate_min: 0.85,
            referential_integrity_min: 0.95,
            duplicate_rate_max: 0.05,
            completeness_rate_min: 0.90,
            format_consistency_min: 0.95,
            anomaly_rate_min: 0.005,
            anomaly_rate_max: 0.30,
            label_coverage_min: 0.95,
            train_ratio_min: 0.50,
            graph_connectivity_min: 0.90,
        }
    }
}
