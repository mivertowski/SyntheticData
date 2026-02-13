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
    /// Enable anomaly realism evaluation.
    #[serde(default)]
    pub anomaly_realism_enabled: bool,
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
            anomaly_realism_enabled: false,
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
    /// Enable financial reporting evaluation.
    #[serde(default)]
    pub financial_reporting_enabled: bool,
    /// Enable HR/payroll evaluation.
    #[serde(default)]
    pub hr_payroll_enabled: bool,
    /// Enable manufacturing evaluation.
    #[serde(default)]
    pub manufacturing_enabled: bool,
    /// Enable bank reconciliation evaluation.
    #[serde(default)]
    pub bank_reconciliation_enabled: bool,
    /// Enable sourcing (S2C) evaluation.
    #[serde(default)]
    pub sourcing_enabled: bool,
    /// Enable cross-process link evaluation.
    #[serde(default)]
    pub cross_process_enabled: bool,
    /// Enable audit evaluation.
    #[serde(default)]
    pub audit_enabled: bool,
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
            financial_reporting_enabled: false,
            hr_payroll_enabled: false,
            manufacturing_enabled: false,
            bank_reconciliation_enabled: false,
            sourcing_enabled: false,
            cross_process_enabled: false,
            audit_enabled: false,
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
    /// Enable anomaly scoring analysis.
    #[serde(default)]
    pub anomaly_scoring_enabled: bool,
    /// Enable feature quality analysis.
    #[serde(default)]
    pub feature_quality_enabled: bool,
    /// Enable GNN readiness analysis.
    #[serde(default)]
    pub gnn_readiness_enabled: bool,
    /// Enable domain gap analysis.
    #[serde(default)]
    pub domain_gap_enabled: bool,
    /// Enable temporal fidelity analysis.
    #[serde(default)]
    pub temporal_fidelity_enabled: bool,
    /// Enable scheme detectability analysis.
    #[serde(default)]
    pub scheme_detectability_enabled: bool,
    /// Enable cross-modal consistency analysis.
    #[serde(default)]
    pub cross_modal_enabled: bool,
    /// Enable embedding readiness analysis.
    #[serde(default)]
    pub embedding_readiness_enabled: bool,
}

impl Default for MlConfig {
    fn default() -> Self {
        Self {
            features_enabled: true,
            labels_enabled: true,
            splits_enabled: true,
            graph_enabled: true,
            anomaly_scoring_enabled: false,
            feature_quality_enabled: false,
            gnn_readiness_enabled: false,
            domain_gap_enabled: false,
            temporal_fidelity_enabled: false,
            scheme_detectability_enabled: false,
            cross_modal_enabled: false,
            embedding_readiness_enabled: false,
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

/// Banking/KYC/AML evaluation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BankingEvalConfig {
    /// Enable KYC completeness evaluation.
    #[serde(default)]
    pub kyc_enabled: bool,
    /// Enable AML detectability evaluation.
    #[serde(default)]
    pub aml_enabled: bool,
}

/// Process mining evaluation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessMiningEvalConfig {
    /// Enable event sequence validation.
    #[serde(default)]
    pub event_sequence_enabled: bool,
    /// Enable variant analysis.
    #[serde(default)]
    pub variant_analysis_enabled: bool,
}

/// Causal model evaluation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CausalEvalConfig {
    /// Enable causal model evaluation.
    #[serde(default)]
    pub enabled: bool,
}

/// LLM enrichment quality evaluation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnrichmentEvalConfig {
    /// Enable enrichment quality evaluation.
    #[serde(default)]
    pub enabled: bool,
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

    // New evaluator thresholds
    /// Minimum anomaly separability (AUC-ROC).
    #[serde(default = "default_anomaly_separability")]
    pub min_anomaly_separability: f64,
    /// Minimum feature quality score.
    #[serde(default = "default_feature_quality")]
    pub min_feature_quality: f64,
    /// Minimum GNN readiness score.
    #[serde(default = "default_gnn_readiness")]
    pub min_gnn_readiness: f64,
    /// Maximum domain gap score.
    #[serde(default = "default_domain_gap")]
    pub max_domain_gap: f64,
    /// Minimum temporal fidelity score.
    #[serde(default = "default_temporal_fidelity")]
    pub min_temporal_fidelity: f64,
    /// Minimum scheme detectability score.
    #[serde(default = "default_scheme_detectability")]
    pub min_scheme_detectability: f64,
    /// Minimum cross-modal consistency.
    #[serde(default = "default_cross_modal")]
    pub min_cross_modal_consistency: f64,
    /// Minimum embedding readiness score.
    #[serde(default = "default_embedding_readiness")]
    pub min_embedding_readiness: f64,

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

fn default_anomaly_separability() -> f64 {
    0.70
}
fn default_feature_quality() -> f64 {
    0.60
}
fn default_gnn_readiness() -> f64 {
    0.65
}
fn default_domain_gap() -> f64 {
    0.25
}
fn default_temporal_fidelity() -> f64 {
    0.70
}
fn default_scheme_detectability() -> f64 {
    0.60
}
fn default_cross_modal() -> f64 {
    0.60
}
fn default_embedding_readiness() -> f64 {
    0.50
}

impl Default for EvaluationThresholds {
    fn default() -> Self {
        Self {
            // Statistical
            benford_p_value_min: 0.05,
            benford_mad_max: 0.015,
            amount_ks_p_value_min: 0.05,
            temporal_correlation_min: 0.80,

            // New evaluator thresholds
            min_anomaly_separability: 0.70,
            min_feature_quality: 0.60,
            min_gnn_readiness: 0.65,
            max_domain_gap: 0.25,
            min_temporal_fidelity: 0.70,
            min_scheme_detectability: 0.60,
            min_cross_modal_consistency: 0.60,
            min_embedding_readiness: 0.50,

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
            min_anomaly_separability: 0.80,
            min_feature_quality: 0.70,
            min_gnn_readiness: 0.75,
            max_domain_gap: 0.15,
            min_temporal_fidelity: 0.80,
            min_scheme_detectability: 0.70,
            min_cross_modal_consistency: 0.70,
            min_embedding_readiness: 0.60,
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
            min_anomaly_separability: 0.55,
            min_feature_quality: 0.45,
            min_gnn_readiness: 0.50,
            max_domain_gap: 0.40,
            min_temporal_fidelity: 0.55,
            min_scheme_detectability: 0.45,
            min_cross_modal_consistency: 0.45,
            min_embedding_readiness: 0.35,
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
