#![deny(clippy::unwrap_used)]
// Allow some clippy lints that are common in test/evaluation code
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::upper_case_acronyms)] // MCAR, MAR, MNAR, ISO are standard abbreviations

//! Synthetic Data Evaluation Framework
//!
//! This crate provides comprehensive evaluation capabilities for validating
//! the quality and correctness of generated synthetic financial data.
//!
//! # Features
//!
//! - **Statistical Quality**: Benford's Law, amount distributions, line item patterns
//! - **Semantic Coherence**: Balance sheet validation, subledger reconciliation
//! - **Data Quality**: Uniqueness, completeness, format consistency
//! - **ML-Readiness**: Feature distributions, label quality, graph structure
//! - **Reporting**: HTML and JSON reports with pass/fail thresholds
//!
//! # Example
//!
//! ```ignore
//! use datasynth_eval::{Evaluator, EvaluationConfig};
//!
//! let config = EvaluationConfig::default();
//! let evaluator = Evaluator::new(config);
//!
//! // Evaluate generated data
//! let result = evaluator.evaluate(&generation_result)?;
//!
//! // Generate report
//! result.generate_html_report("evaluation_report.html")?;
//! ```

pub mod benchmarks;
pub mod config;
pub mod enhancement;
pub mod error;
pub mod gates;
pub mod privacy;

pub mod coherence;
pub mod ml;
pub mod quality;
pub mod report;
pub mod statistical;
pub mod tuning;

// Re-exports
pub use config::{EvaluationConfig, EvaluationThresholds, PrivacyEvaluationConfig};
pub use error::{EvalError, EvalResult};

pub use statistical::{
    AmountDistributionAnalysis, AmountDistributionAnalyzer, BenfordAnalysis, BenfordAnalyzer,
    BenfordConformity, DetectionDifficulty, DriftDetectionAnalysis, DriftDetectionAnalyzer,
    DriftDetectionEntry, DriftDetectionMetrics, DriftEventCategory, LabeledDriftEvent,
    LabeledEventAnalysis, LineItemAnalysis, LineItemAnalyzer, LineItemEntry, StatisticalEvaluation,
    TemporalAnalysis, TemporalAnalyzer, TemporalEntry,
};

pub use coherence::{
    AuditTrailEvaluation, AuditTrailGap, BalanceSheetEvaluation, BalanceSheetEvaluator,
    CoherenceEvaluation, ConcentrationMetrics, DocumentChainEvaluation, DocumentChainEvaluator,
    FairValueEvaluation, FrameworkViolation, ICMatchingEvaluation, ICMatchingEvaluator,
    ImpairmentEvaluation, IsaComplianceEvaluation, LeaseAccountingEvaluation,
    LeaseAccountingEvaluator, LeaseEvaluation, NetworkEdge, NetworkEvaluation, NetworkEvaluator,
    NetworkNode, NetworkThresholds, PcaobComplianceEvaluation, PerformanceObligation,
    ReferentialIntegrityEvaluation, ReferentialIntegrityEvaluator, RevenueContract,
    RevenueRecognitionEvaluation, RevenueRecognitionEvaluator, SoxComplianceEvaluation,
    StandardsComplianceEvaluation, StandardsThresholds, StrengthStats, SubledgerEvaluator,
    SubledgerReconciliationEvaluation, VariableConsideration, ViolationSeverity,
};

pub use quality::{
    CompletenessAnalysis, CompletenessAnalyzer, ConsistencyAnalysis, ConsistencyAnalyzer,
    ConsistencyRule, DuplicateInfo, FieldCompleteness, FormatAnalysis, FormatAnalyzer,
    FormatVariation, QualityEvaluation, UniquenessAnalysis, UniquenessAnalyzer,
};

pub use ml::{
    FeatureAnalysis, FeatureAnalyzer, FeatureStats, GraphAnalysis, GraphAnalyzer, GraphMetrics,
    LabelAnalysis, LabelAnalyzer, LabelDistribution, MLReadinessEvaluation, SplitAnalysis,
    SplitAnalyzer, SplitMetrics,
};

pub use report::{
    BaselineComparison, ComparisonResult, EvaluationReport, HtmlReportGenerator,
    JsonReportGenerator, MetricChange, ReportMetadata, ThresholdChecker, ThresholdResult,
};

pub use tuning::{
    ConfigSuggestion, ConfigSuggestionGenerator, TuningAnalyzer, TuningCategory, TuningOpportunity,
};

pub use enhancement::{
    AutoTuneResult, AutoTuner, ConfigPatch, EnhancementReport, Recommendation,
    RecommendationCategory, RecommendationEngine, RecommendationPriority, RootCause,
    SuggestedAction,
};

pub use privacy::{
    LinkageAttack, LinkageConfig, LinkageResults, MembershipInferenceAttack, MiaConfig, MiaResults,
    NistAlignmentReport, NistCriterion, PrivacyEvaluation, SynQPMatrix, SynQPQuadrant,
};

pub use benchmarks::{
    // ACFE-calibrated benchmarks
    acfe_calibrated_1k,
    acfe_collusion_5k,
    acfe_management_override_2k,
    all_acfe_benchmarks,
    all_benchmarks,
    // Industry-specific benchmarks
    all_industry_benchmarks,
    anomaly_bench_1k,
    data_quality_100k,
    entity_match_5k,
    financial_services_fraud_5k,
    fraud_detect_10k,
    get_benchmark,
    get_industry_benchmark,
    graph_fraud_10k,
    healthcare_fraud_5k,
    manufacturing_fraud_5k,
    retail_fraud_10k,
    technology_fraud_3k,
    AcfeAlignment,
    AcfeCalibration,
    AcfeCategoryDistribution,
    BaselineModelType,
    BaselineResult,
    BenchmarkBuilder,
    BenchmarkSuite,
    BenchmarkTaskType,
    CostMatrix,
    DatasetSpec,
    EvaluationSpec,
    FeatureSet,
    IndustryBenchmarkAnalysis,
    LeaderboardEntry,
    MetricType,
    SplitRatios,
};

use serde::{Deserialize, Serialize};

/// Comprehensive evaluation result combining all evaluation modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveEvaluation {
    /// Statistical quality evaluation.
    pub statistical: StatisticalEvaluation,
    /// Semantic coherence evaluation.
    pub coherence: CoherenceEvaluation,
    /// Data quality evaluation.
    pub quality: QualityEvaluation,
    /// ML-readiness evaluation.
    pub ml_readiness: MLReadinessEvaluation,
    /// Privacy evaluation (optional — only populated when privacy testing is enabled).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy: Option<PrivacyEvaluation>,
    /// Overall pass/fail status.
    pub passes: bool,
    /// Summary of all failures.
    pub failures: Vec<String>,
    /// Tuning opportunities identified.
    pub tuning_opportunities: Vec<TuningOpportunity>,
    /// Configuration suggestions.
    pub config_suggestions: Vec<ConfigSuggestion>,
}

impl ComprehensiveEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            statistical: StatisticalEvaluation::default(),
            coherence: CoherenceEvaluation::default(),
            quality: QualityEvaluation::default(),
            ml_readiness: MLReadinessEvaluation::default(),
            privacy: None,
            passes: true,
            failures: Vec::new(),
            tuning_opportunities: Vec::new(),
            config_suggestions: Vec::new(),
        }
    }

    /// Check all evaluations against thresholds and update overall status.
    pub fn check_all_thresholds(&mut self, thresholds: &EvaluationThresholds) {
        self.failures.clear();

        // Check statistical thresholds
        self.statistical.check_thresholds(thresholds);
        self.failures.extend(self.statistical.failures.clone());

        // Check coherence thresholds
        self.coherence.check_thresholds(thresholds);
        self.failures.extend(self.coherence.failures.clone());

        // Check quality thresholds
        self.quality.check_thresholds(thresholds);
        self.failures.extend(self.quality.failures.clone());

        // Check ML thresholds
        self.ml_readiness.check_thresholds(thresholds);
        self.failures.extend(self.ml_readiness.failures.clone());

        // Check privacy evaluation (if present)
        if let Some(ref mut privacy) = self.privacy {
            privacy.update_status();
            self.failures.extend(privacy.failures.clone());
        }

        self.passes = self.failures.is_empty();
    }
}

impl Default for ComprehensiveEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

/// Main evaluator that coordinates all evaluation modules.
pub struct Evaluator {
    /// Evaluation configuration.
    config: EvaluationConfig,
}

impl Evaluator {
    /// Create a new evaluator with the given configuration.
    pub fn new(config: EvaluationConfig) -> Self {
        Self { config }
    }

    /// Create an evaluator with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(EvaluationConfig::default())
    }

    /// Get the configuration.
    pub fn config(&self) -> &EvaluationConfig {
        &self.config
    }

    /// Run a comprehensive evaluation and return results.
    ///
    /// This is a placeholder - actual implementation would take
    /// generation results as input.
    pub fn run_evaluation(&self) -> ComprehensiveEvaluation {
        let mut evaluation = ComprehensiveEvaluation::new();
        evaluation.check_all_thresholds(&self.config.thresholds);
        evaluation
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_evaluation_new() {
        let eval = ComprehensiveEvaluation::new();
        assert!(eval.passes);
        assert!(eval.failures.is_empty());
    }

    #[test]
    fn test_evaluator_creation() {
        let evaluator = Evaluator::with_defaults();
        assert_eq!(evaluator.config().thresholds.benford_p_value_min, 0.05);
    }
}
