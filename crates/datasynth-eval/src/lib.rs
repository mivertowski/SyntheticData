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

pub mod banking;
pub mod causal;
pub mod diff_engine;
pub mod enrichment;
pub mod process_mining;
pub mod scenario_diff;

// Re-exports
pub use config::{EvaluationConfig, EvaluationThresholds, PrivacyEvaluationConfig};
pub use error::{EvalError, EvalResult};

pub use statistical::{
    AmountDistributionAnalysis, AmountDistributionAnalyzer, AnomalyRealismEvaluation,
    AnomalyRealismEvaluator, BenfordAnalysis, BenfordAnalyzer, BenfordConformity,
    DetectionDifficulty, DriftDetectionAnalysis, DriftDetectionAnalyzer, DriftDetectionEntry,
    DriftDetectionMetrics, DriftEventCategory, LabeledDriftEvent, LabeledEventAnalysis,
    LineItemAnalysis, LineItemAnalyzer, LineItemEntry, StatisticalEvaluation, TemporalAnalysis,
    TemporalAnalyzer, TemporalEntry,
};

pub use coherence::{
    AccountType,
    ApprovalLevelData,
    AuditEvaluation,
    AuditEvaluator,
    AuditFindingData,
    AuditRiskData,
    AuditTrailEvaluation,
    AuditTrailGap,
    BalanceSheetEvaluation,
    BalanceSheetEvaluator,
    BalanceSnapshot,
    BankReconciliationEvaluation,
    BankReconciliationEvaluator,
    BidEvaluationData,
    BudgetVarianceData,
    CashPositionData,
    CoherenceEvaluation,
    ConcentrationMetrics,
    CountryPackData,
    CountryPackEvaluation,
    CountryPackEvaluator,
    CountryPackThresholds,
    CovenantData,
    CrossProcessEvaluation,
    CrossProcessEvaluator,
    CycleCountData,
    DocumentChainEvaluation,
    DocumentChainEvaluator,
    DocumentReferenceData,
    EarnedValueData,
    EntityReferenceData,
    EsgEvaluation,
    EsgEvaluator,
    EsgThresholds,
    ExpenseReportData,
    FairValueEvaluation,
    // Task 4.1: Financial Ratio Evaluator
    FinancialRatios,
    FinancialReportingEvaluation,
    FinancialReportingEvaluator,
    FinancialStatementData,
    FrameworkViolation,
    GovernanceData,
    HedgeEffectivenessData,
    HolidayData,
    HrPayrollEvaluation,
    HrPayrollEvaluator,
    ICMatchingData,
    ICMatchingEvaluation,
    ICMatchingEvaluator,
    ImpairmentEvaluation,
    IsaComplianceEvaluation,
    // Task 4.2: JE Risk Scoring Evaluator
    JeRiskScoringResult,
    KpiData,
    LeaseAccountingEvaluation,
    LeaseAccountingEvaluator,
    LeaseEvaluation,
    ManufacturingEvaluation,
    ManufacturingEvaluator,
    MaterialityData,
    NettingData,
    NetworkEdge,
    NetworkEvaluation,
    NetworkEvaluator,
    NetworkNode,
    NetworkThresholds,
    O2CChainData,
    P2PChainData,
    PayrollHoursData,
    PayrollLineItemData,
    PayrollRunData,
    PcaobComplianceEvaluation,
    PerformanceObligation,
    ProductionOrderData,
    ProjectAccountingEvaluation,
    ProjectAccountingEvaluator,
    ProjectAccountingThresholds,
    ProjectRevenueData,
    QualityInspectionData,
    QuoteLineData,
    RatioAnalysisResult,
    RatioCheck,
    ReconciliationData,
    ReferentialData,
    ReferentialIntegrityEvaluation,
    ReferentialIntegrityEvaluator,
    RetainageData,
    RevenueContract,
    RevenueRecognitionEvaluation,
    RevenueRecognitionEvaluator,
    RiskAttributeStats,
    RiskDistribution,
    RoutingOperationData,
    SafetyMetricData,
    SalesQuoteData,
    SalesQuoteEvaluation,
    SalesQuoteEvaluator,
    SalesQuoteThresholds,
    ScorecardCoverageData,
    SourcingEvaluation,
    SourcingEvaluator,
    SourcingProjectData,
    SoxComplianceEvaluation,
    SpendAnalysisData,
    StandardsComplianceEvaluation,
    StandardsThresholds,
    StrengthStats,
    SubledgerEvaluator,
    SubledgerReconciliationEvaluation,
    SupplierEsgData,
    TaxEvaluation,
    TaxEvaluator,
    TaxLineData,
    TaxRateData,
    TaxReturnData,
    TaxThresholds,
    TimeEntryData,
    TreasuryEvaluation,
    TreasuryEvaluator,
    TreasuryThresholds,
    UnmatchedICItem,
    VariableConsideration,
    ViolationSeverity,
    WaterUsageData,
    WithholdingData,
    WorkpaperData,
};

pub use quality::{
    CompletenessAnalysis, CompletenessAnalyzer, ConsistencyAnalysis, ConsistencyAnalyzer,
    ConsistencyRule, DuplicateInfo, FieldCompleteness, FieldDefinition, FieldValue, FormatAnalysis,
    FormatAnalyzer, FormatVariation, QualityEvaluation, UniqueRecord, UniquenessAnalysis,
    UniquenessAnalyzer,
};

pub use ml::{
    AnomalyScoringAnalysis, AnomalyScoringAnalyzer, CrossModalAnalysis, CrossModalAnalyzer,
    DomainGapAnalysis, DomainGapAnalyzer, EmbeddingReadinessAnalysis, EmbeddingReadinessAnalyzer,
    FeatureAnalysis, FeatureAnalyzer, FeatureQualityAnalysis, FeatureQualityAnalyzer, FeatureStats,
    GnnReadinessAnalysis, GnnReadinessAnalyzer, GraphAnalysis, GraphAnalyzer, GraphMetrics,
    LabelAnalysis, LabelAnalyzer, LabelDistribution, MLReadinessEvaluation,
    SchemeDetectabilityAnalysis, SchemeDetectabilityAnalyzer, SplitAnalysis, SplitAnalyzer,
    SplitMetrics, TemporalFidelityAnalysis, TemporalFidelityAnalyzer,
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

pub use banking::{
    AmlDetectabilityAnalysis, AmlDetectabilityAnalyzer, AmlTransactionData, BankingEvaluation,
    KycCompletenessAnalysis, KycCompletenessAnalyzer, KycProfileData, TypologyData,
};

pub use process_mining::{
    EventSequenceAnalysis, EventSequenceAnalyzer, ProcessEventData, ProcessMiningEvaluation,
    VariantAnalysis, VariantAnalyzer, VariantData,
};

pub use causal::{CausalModelEvaluation, CausalModelEvaluator};

pub use enrichment::{EnrichmentQualityEvaluation, EnrichmentQualityEvaluator};

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
    /// Banking/KYC/AML evaluation (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banking: Option<BankingEvaluation>,
    /// OCEL 2.0 process mining evaluation (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_mining: Option<ProcessMiningEvaluation>,
    /// Causal model evaluation (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causal: Option<CausalModelEvaluation>,
    /// LLM enrichment quality evaluation (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enrichment_quality: Option<EnrichmentQualityEvaluation>,
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
            banking: None,
            process_mining: None,
            causal: None,
            enrichment_quality: None,
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

        // Check banking evaluation
        if let Some(ref mut banking) = self.banking {
            banking.check_thresholds();
            self.failures.extend(banking.issues.clone());
        }

        // Check process mining evaluation
        if let Some(ref mut pm) = self.process_mining {
            pm.check_thresholds();
            self.failures.extend(pm.issues.clone());
        }

        // Check causal model evaluation
        if let Some(ref causal) = self.causal {
            if !causal.passes {
                self.failures.extend(causal.issues.clone());
            }
        }

        // Check enrichment quality evaluation
        if let Some(ref enrichment) = self.enrichment_quality {
            if !enrichment.passes {
                self.failures.extend(enrichment.issues.clone());
            }
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
    /// # Architectural note
    ///
    /// This zero-argument variant returns a default (passing) evaluation because the
    /// `Evaluator` struct holds only configuration — it has no access to the generated
    /// journal entry or balance data that the sub-module evaluators require.
    ///
    /// To evaluate actual generation output, use [`run_evaluation_with_amounts`] which
    /// accepts raw JE amounts and runs the Benford analysis.  Full wiring of all
    /// sub-modules (BalanceSheetEvaluator, DocumentChainEvaluator, etc.) requires
    /// passing the complete `EnhancedGenerationResult` from the runtime crate, which
    /// would create a circular dependency.  The recommended integration point is the
    /// orchestrator layer (datasynth-runtime) which already calls the gate engine with
    /// a populated `ComprehensiveEvaluation`.
    pub fn run_evaluation(&self) -> ComprehensiveEvaluation {
        let mut evaluation = ComprehensiveEvaluation::new();
        evaluation.check_all_thresholds(&self.config.thresholds);
        evaluation
    }

    /// Run a Benford-augmented evaluation given raw JE amounts.
    ///
    /// This method calls the [`BenfordAnalyzer`] sub-module and populates the
    /// `statistical.benford` field of the returned [`ComprehensiveEvaluation`].
    /// All other sub-module fields remain at their default (passing) values.
    pub fn run_evaluation_with_amounts(
        &self,
        je_amounts: &[rust_decimal::Decimal],
    ) -> ComprehensiveEvaluation {
        let mut evaluation = ComprehensiveEvaluation::new();

        if !je_amounts.is_empty() {
            let analyzer = BenfordAnalyzer::new(self.config.thresholds.benford_p_value_min);
            match analyzer.analyze(je_amounts) {
                Ok(benford) => {
                    evaluation.statistical.benford = Some(benford);
                }
                Err(e) => {
                    evaluation
                        .failures
                        .push(format!("Benford analysis failed: {e}"));
                    evaluation.passes = false;
                }
            }
        }

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
