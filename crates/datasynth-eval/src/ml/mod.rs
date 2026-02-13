//! ML-readiness evaluation module.
//!
//! Validates that generated data is suitable for machine learning tasks
//! including feature distributions, label quality, and graph structure.
//!
//! Also provides baseline task definitions for benchmarking synthetic data.

mod anomaly_scoring;
mod baselines;
mod cross_modal;
mod domain_gap;
mod embedding_readiness;
mod feature_quality;
mod features;
mod gnn_readiness;
mod graph;
mod labels;
mod scheme_detectability;
mod splits;
mod temporal_fidelity;

pub use anomaly_scoring::{
    AnomalyScoringAnalysis, AnomalyScoringAnalyzer, AnomalyScoringThresholds, ScoredRecord,
};
pub use baselines::{
    get_accounting_baseline_tasks, BaselineAlgorithm, BaselineConfig, BaselineEvaluation,
    BaselineResult, BaselineSummary, BaselineTask, ClassificationMetrics, ExpectedMetrics,
    MLTaskType, PerformanceGrade, RankingMetrics, RegressionMetrics,
};
pub use cross_modal::{
    CrossModalAnalysis, CrossModalAnalyzer, CrossModalThresholds, EntityModalData,
};
pub use domain_gap::{
    DistributionSample, DomainGapAnalysis, DomainGapAnalyzer, DomainGapDetail, DomainGapThresholds,
};
pub use embedding_readiness::{
    EmbeddingInput, EmbeddingReadinessAnalysis, EmbeddingReadinessAnalyzer,
    EmbeddingReadinessThresholds,
};
pub use feature_quality::{
    FeatureQualityAnalysis, FeatureQualityAnalyzer, FeatureQualityThresholds, FeatureVector,
};
pub use features::{FeatureAnalysis, FeatureAnalyzer, FeatureStats};
pub use gnn_readiness::GraphData as GnnGraphData;
pub use gnn_readiness::{GnnReadinessAnalysis, GnnReadinessAnalyzer, GnnReadinessThresholds};
pub use graph::{GraphAnalysis, GraphAnalyzer, GraphMetrics};
pub use labels::{LabelAnalysis, LabelAnalyzer, LabelDistribution};
pub use scheme_detectability::{
    SchemeDetectabilityAnalysis, SchemeDetectabilityAnalyzer, SchemeDetectabilityThresholds,
    SchemeRecord,
};
pub use splits::{SplitAnalysis, SplitAnalyzer, SplitMetrics};
pub use temporal_fidelity::{
    TemporalFidelityAnalysis, TemporalFidelityAnalyzer, TemporalFidelityThresholds, TemporalRecord,
};

use serde::{Deserialize, Serialize};

/// Combined ML-readiness evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLReadinessEvaluation {
    /// Feature distribution analysis.
    pub features: Option<FeatureAnalysis>,
    /// Label quality analysis.
    pub labels: Option<LabelAnalysis>,
    /// Train/test split analysis.
    pub splits: Option<SplitAnalysis>,
    /// Graph structure analysis.
    pub graph: Option<GraphAnalysis>,
    /// Anomaly scoring analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anomaly_scoring: Option<AnomalyScoringAnalysis>,
    /// Feature quality analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_quality: Option<FeatureQualityAnalysis>,
    /// GNN readiness analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gnn_readiness: Option<GnnReadinessAnalysis>,
    /// Domain gap analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_gap: Option<DomainGapAnalysis>,
    /// Temporal fidelity analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_fidelity: Option<TemporalFidelityAnalysis>,
    /// Scheme detectability analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme_detectability: Option<SchemeDetectabilityAnalysis>,
    /// Cross-modal consistency analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cross_modal: Option<CrossModalAnalysis>,
    /// Embedding readiness analysis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding_readiness: Option<EmbeddingReadinessAnalysis>,
    /// Overall ML-readiness score (0.0-1.0).
    pub overall_score: f64,
    /// Whether data meets ML-readiness criteria.
    pub passes: bool,
    /// ML-readiness issues found.
    pub issues: Vec<String>,
    /// ML-readiness failures (alias for issues).
    pub failures: Vec<String>,
}

impl MLReadinessEvaluation {
    /// Create a new empty evaluation.
    pub fn new() -> Self {
        Self {
            features: None,
            labels: None,
            splits: None,
            graph: None,
            anomaly_scoring: None,
            feature_quality: None,
            gnn_readiness: None,
            domain_gap: None,
            temporal_fidelity: None,
            scheme_detectability: None,
            cross_modal: None,
            embedding_readiness: None,
            overall_score: 1.0,
            passes: true,
            issues: Vec::new(),
            failures: Vec::new(),
        }
    }

    /// Check all results against thresholds.
    pub fn check_thresholds(&mut self, thresholds: &crate::config::EvaluationThresholds) {
        self.issues.clear();
        self.failures.clear();
        let mut scores = Vec::new();

        if let Some(ref labels) = self.labels {
            // Check anomaly rate is within expected range
            if labels.anomaly_rate < thresholds.anomaly_rate_min {
                self.issues.push(format!(
                    "Anomaly rate {} < {} (min threshold)",
                    labels.anomaly_rate, thresholds.anomaly_rate_min
                ));
            }
            if labels.anomaly_rate > thresholds.anomaly_rate_max {
                self.issues.push(format!(
                    "Anomaly rate {} > {} (max threshold)",
                    labels.anomaly_rate, thresholds.anomaly_rate_max
                ));
            }

            // Check label coverage
            if labels.label_coverage < thresholds.label_coverage_min {
                self.issues.push(format!(
                    "Label coverage {} < {} (threshold)",
                    labels.label_coverage, thresholds.label_coverage_min
                ));
            }

            scores.push(labels.quality_score);
        }

        if let Some(ref splits) = self.splits {
            if !splits.is_valid {
                self.issues
                    .push("Train/test split validation failed".to_string());
            }
            scores.push(if splits.is_valid { 1.0 } else { 0.0 });
        }

        if let Some(ref graph) = self.graph {
            if graph.connectivity_score < thresholds.graph_connectivity_min {
                self.issues.push(format!(
                    "Graph connectivity {} < {} (threshold)",
                    graph.connectivity_score, thresholds.graph_connectivity_min
                ));
            }
            scores.push(graph.connectivity_score);
        }

        if let Some(ref features) = self.features {
            scores.push(features.quality_score);
        }

        // New ML enrichment evaluators
        if let Some(ref as_eval) = self.anomaly_scoring {
            if !as_eval.passes {
                self.issues.extend(as_eval.issues.clone());
            }
            scores.push(as_eval.anomaly_separability);
        }
        if let Some(ref fq_eval) = self.feature_quality {
            if !fq_eval.passes {
                self.issues.extend(fq_eval.issues.clone());
            }
            scores.push(fq_eval.feature_quality_score);
        }
        if let Some(ref gnn_eval) = self.gnn_readiness {
            if !gnn_eval.passes {
                self.issues.extend(gnn_eval.issues.clone());
            }
            scores.push(gnn_eval.gnn_readiness_score);
        }
        if let Some(ref dg_eval) = self.domain_gap {
            if !dg_eval.passes {
                self.issues.extend(dg_eval.issues.clone());
            }
            // Domain gap is inverted: lower = better, so score = 1 - gap
            scores.push(1.0 - dg_eval.domain_gap_score);
        }
        if let Some(ref tf_eval) = self.temporal_fidelity {
            if !tf_eval.passes {
                self.issues.extend(tf_eval.issues.clone());
            }
            scores.push(tf_eval.temporal_fidelity_score);
        }
        if let Some(ref sd_eval) = self.scheme_detectability {
            if !sd_eval.passes {
                self.issues.extend(sd_eval.issues.clone());
            }
            scores.push(sd_eval.detectability_score);
        }
        if let Some(ref cm_eval) = self.cross_modal {
            if !cm_eval.passes {
                self.issues.extend(cm_eval.issues.clone());
            }
            scores.push(cm_eval.consistency_score);
        }
        if let Some(ref er_eval) = self.embedding_readiness {
            if !er_eval.passes {
                self.issues.extend(er_eval.issues.clone());
            }
            scores.push(er_eval.embedding_readiness_score);
        }

        self.overall_score = if scores.is_empty() {
            1.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        // Sync failures with issues
        self.failures = self.issues.clone();
        self.passes = self.issues.is_empty();
    }
}

impl Default for MLReadinessEvaluation {
    fn default() -> Self {
        Self::new()
    }
}
