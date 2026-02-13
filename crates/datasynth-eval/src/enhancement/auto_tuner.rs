//! Auto-tuning engine for deriving optimal configuration from evaluation results.
//!
//! The AutoTuner analyzes evaluation results to identify metric gaps and
//! computes suggested configuration values that should improve those metrics.

use crate::{ComprehensiveEvaluation, EvaluationThresholds};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A configuration patch representing a change to apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPatch {
    /// Configuration path (dot-separated).
    pub path: String,
    /// The current value (if known).
    pub current_value: Option<String>,
    /// The suggested new value.
    pub suggested_value: String,
    /// Confidence level (0.0-1.0) that this change will help.
    pub confidence: f64,
    /// Expected improvement description.
    pub expected_impact: String,
}

impl ConfigPatch {
    /// Create a new config patch.
    pub fn new(path: impl Into<String>, suggested_value: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            current_value: None,
            suggested_value: suggested_value.into(),
            confidence: 0.5,
            expected_impact: String::new(),
        }
    }

    /// Set the current value.
    pub fn with_current(mut self, value: impl Into<String>) -> Self {
        self.current_value = Some(value.into());
        self
    }

    /// Set the confidence level.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set the expected impact.
    pub fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.expected_impact = impact.into();
        self
    }
}

/// Result of auto-tuning analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTuneResult {
    /// Configuration patches to apply.
    pub patches: Vec<ConfigPatch>,
    /// Overall improvement score (0.0-1.0).
    pub expected_improvement: f64,
    /// Metrics that will be addressed.
    pub addressed_metrics: Vec<String>,
    /// Metrics that cannot be automatically fixed.
    pub unaddressable_metrics: Vec<String>,
    /// Summary message.
    pub summary: String,
}

impl AutoTuneResult {
    /// Create a new empty result.
    pub fn new() -> Self {
        Self {
            patches: Vec::new(),
            expected_improvement: 0.0,
            addressed_metrics: Vec::new(),
            unaddressable_metrics: Vec::new(),
            summary: String::new(),
        }
    }

    /// Check if any patches are suggested.
    pub fn has_patches(&self) -> bool {
        !self.patches.is_empty()
    }

    /// Get patches sorted by confidence (highest first).
    pub fn patches_by_confidence(&self) -> Vec<&ConfigPatch> {
        let mut sorted: Vec<_> = self.patches.iter().collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }
}

impl Default for AutoTuneResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric gap analysis result.
#[derive(Debug, Clone)]
pub struct MetricGap {
    /// Name of the metric.
    pub metric_name: String,
    /// Current value.
    pub current_value: f64,
    /// Target threshold value.
    pub target_value: f64,
    /// Gap (target - current for min thresholds, current - target for max).
    pub gap: f64,
    /// Whether this is a minimum threshold (true) or maximum (false).
    pub is_minimum: bool,
    /// Related configuration paths.
    pub config_paths: Vec<String>,
}

impl MetricGap {
    /// Calculate the severity of the gap (0.0-1.0).
    pub fn severity(&self) -> f64 {
        if self.target_value == 0.0 {
            if self.gap.abs() > 0.0 {
                1.0
            } else {
                0.0
            }
        } else {
            (self.gap.abs() / self.target_value.abs()).min(1.0)
        }
    }
}

/// Auto-tuner that derives optimal configuration from evaluation results.
pub struct AutoTuner {
    /// Thresholds to compare against.
    thresholds: EvaluationThresholds,
    /// Known metric-to-config mappings.
    metric_mappings: HashMap<String, Vec<MetricConfigMapping>>,
}

/// Mapping from a metric to configuration paths that affect it.
#[derive(Debug, Clone)]
struct MetricConfigMapping {
    /// Configuration path.
    config_path: String,
    /// How much influence this config has on the metric (0.0-1.0).
    influence: f64,
    /// Function to compute suggested value given the gap.
    compute_value: ComputeStrategy,
}

/// Strategy for computing suggested config values.
#[allow(dead_code)] // Variants reserved for future tuning strategies
#[derive(Debug, Clone, Copy)]
enum ComputeStrategy {
    /// Enable a boolean flag.
    EnableBoolean,
    /// Set to a specific value.
    SetFixed(f64),
    /// Increase by the gap amount.
    IncreaseByGap,
    /// Decrease by the gap amount.
    DecreaseByGap,
    /// Set to target value directly.
    SetToTarget,
    /// Multiply current by factor based on gap.
    MultiplyByGapFactor,
}

impl AutoTuner {
    /// Create a new auto-tuner with default thresholds.
    pub fn new() -> Self {
        Self::with_thresholds(EvaluationThresholds::default())
    }

    /// Create an auto-tuner with specific thresholds.
    pub fn with_thresholds(thresholds: EvaluationThresholds) -> Self {
        let mut tuner = Self {
            thresholds,
            metric_mappings: HashMap::new(),
        };
        tuner.initialize_mappings();
        tuner
    }

    /// Initialize known metric-to-config mappings.
    fn initialize_mappings(&mut self) {
        // Benford's Law
        self.metric_mappings.insert(
            "benford_p_value".to_string(),
            vec![MetricConfigMapping {
                config_path: "transactions.amount.benford_compliance".to_string(),
                influence: 0.9,
                compute_value: ComputeStrategy::EnableBoolean,
            }],
        );

        // Round number bias
        self.metric_mappings.insert(
            "round_number_ratio".to_string(),
            vec![MetricConfigMapping {
                config_path: "transactions.amount.round_number_bias".to_string(),
                influence: 0.95,
                compute_value: ComputeStrategy::SetToTarget,
            }],
        );

        // Temporal correlation
        self.metric_mappings.insert(
            "temporal_correlation".to_string(),
            vec![MetricConfigMapping {
                config_path: "transactions.temporal.seasonality_strength".to_string(),
                influence: 0.7,
                compute_value: ComputeStrategy::IncreaseByGap,
            }],
        );

        // Anomaly rate
        self.metric_mappings.insert(
            "anomaly_rate".to_string(),
            vec![MetricConfigMapping {
                config_path: "anomaly_injection.base_rate".to_string(),
                influence: 0.95,
                compute_value: ComputeStrategy::SetToTarget,
            }],
        );

        // Label coverage
        self.metric_mappings.insert(
            "label_coverage".to_string(),
            vec![MetricConfigMapping {
                config_path: "anomaly_injection.label_all".to_string(),
                influence: 0.9,
                compute_value: ComputeStrategy::EnableBoolean,
            }],
        );

        // Duplicate rate
        self.metric_mappings.insert(
            "duplicate_rate".to_string(),
            vec![MetricConfigMapping {
                config_path: "data_quality.duplicates.exact_rate".to_string(),
                influence: 0.8,
                compute_value: ComputeStrategy::SetToTarget,
            }],
        );

        // Completeness
        self.metric_mappings.insert(
            "completeness_rate".to_string(),
            vec![MetricConfigMapping {
                config_path: "data_quality.missing_values.overall_rate".to_string(),
                influence: 0.9,
                compute_value: ComputeStrategy::DecreaseByGap,
            }],
        );

        // IC match rate
        self.metric_mappings.insert(
            "ic_match_rate".to_string(),
            vec![MetricConfigMapping {
                config_path: "intercompany.match_precision".to_string(),
                influence: 0.85,
                compute_value: ComputeStrategy::IncreaseByGap,
            }],
        );

        // Document chain completion
        self.metric_mappings.insert(
            "doc_chain_completion".to_string(),
            vec![
                MetricConfigMapping {
                    config_path: "document_flows.p2p.completion_rate".to_string(),
                    influence: 0.5,
                    compute_value: ComputeStrategy::SetToTarget,
                },
                MetricConfigMapping {
                    config_path: "document_flows.o2c.completion_rate".to_string(),
                    influence: 0.5,
                    compute_value: ComputeStrategy::SetToTarget,
                },
            ],
        );

        // Graph connectivity
        self.metric_mappings.insert(
            "graph_connectivity".to_string(),
            vec![MetricConfigMapping {
                config_path: "graph_export.ensure_connected".to_string(),
                influence: 0.8,
                compute_value: ComputeStrategy::EnableBoolean,
            }],
        );

        // --- New evaluator metric mappings ---

        // Payroll accuracy
        self.metric_mappings.insert(
            "payroll_accuracy".to_string(),
            vec![MetricConfigMapping {
                config_path: "hr.payroll.calculation_precision".to_string(),
                influence: 0.9,
                compute_value: ComputeStrategy::SetToTarget,
            }],
        );

        // Manufacturing yield
        self.metric_mappings.insert(
            "manufacturing_yield".to_string(),
            vec![MetricConfigMapping {
                config_path: "manufacturing.production_orders.yield_target".to_string(),
                influence: 0.8,
                compute_value: ComputeStrategy::SetToTarget,
            }],
        );

        // S2C chain completion
        self.metric_mappings.insert(
            "s2c_chain_completion".to_string(),
            vec![MetricConfigMapping {
                config_path: "source_to_pay.rfx_completion_rate".to_string(),
                influence: 0.85,
                compute_value: ComputeStrategy::SetToTarget,
            }],
        );

        // Bank reconciliation balance
        self.metric_mappings.insert(
            "bank_recon_balance".to_string(),
            vec![MetricConfigMapping {
                config_path: "enterprise.bank_reconciliation.tolerance".to_string(),
                influence: 0.9,
                compute_value: ComputeStrategy::DecreaseByGap,
            }],
        );

        // Financial reporting tie-back
        self.metric_mappings.insert(
            "financial_reporting_tie_back".to_string(),
            vec![MetricConfigMapping {
                config_path: "financial_reporting.statement_generation.enabled".to_string(),
                influence: 0.85,
                compute_value: ComputeStrategy::EnableBoolean,
            }],
        );

        // AML detectability
        self.metric_mappings.insert(
            "aml_detectability".to_string(),
            vec![MetricConfigMapping {
                config_path: "enterprise.banking.aml_typology_count".to_string(),
                influence: 0.8,
                compute_value: ComputeStrategy::IncreaseByGap,
            }],
        );

        // Process mining coverage
        self.metric_mappings.insert(
            "process_mining_coverage".to_string(),
            vec![MetricConfigMapping {
                config_path: "business_processes.ocel_enabled".to_string(),
                influence: 0.85,
                compute_value: ComputeStrategy::EnableBoolean,
            }],
        );

        // Audit evidence coverage
        self.metric_mappings.insert(
            "audit_evidence_coverage".to_string(),
            vec![MetricConfigMapping {
                config_path: "audit_standards.evidence_per_finding".to_string(),
                influence: 0.8,
                compute_value: ComputeStrategy::IncreaseByGap,
            }],
        );

        // Anomaly separability
        self.metric_mappings.insert(
            "anomaly_separability".to_string(),
            vec![MetricConfigMapping {
                config_path: "anomaly_injection.base_rate".to_string(),
                influence: 0.75,
                compute_value: ComputeStrategy::IncreaseByGap,
            }],
        );

        // Feature quality
        self.metric_mappings.insert(
            "feature_quality".to_string(),
            vec![MetricConfigMapping {
                config_path: "graph_export.feature_completeness".to_string(),
                influence: 0.7,
                compute_value: ComputeStrategy::EnableBoolean,
            }],
        );

        // GNN readiness
        self.metric_mappings.insert(
            "gnn_readiness".to_string(),
            vec![
                MetricConfigMapping {
                    config_path: "graph_export.ensure_connected".to_string(),
                    influence: 0.6,
                    compute_value: ComputeStrategy::EnableBoolean,
                },
                MetricConfigMapping {
                    config_path: "cross_process_links.enabled".to_string(),
                    influence: 0.5,
                    compute_value: ComputeStrategy::EnableBoolean,
                },
            ],
        );

        // Domain gap
        self.metric_mappings.insert(
            "domain_gap".to_string(),
            vec![MetricConfigMapping {
                config_path: "distributions.industry_profile".to_string(),
                influence: 0.7,
                compute_value: ComputeStrategy::SetFixed(1.0), // placeholder - needs manual review
            }],
        );
    }

    /// Analyze evaluation results and produce auto-tune suggestions.
    pub fn analyze(&self, evaluation: &ComprehensiveEvaluation) -> AutoTuneResult {
        let mut result = AutoTuneResult::new();

        // Identify metric gaps
        let gaps = self.identify_gaps(evaluation);

        // Generate patches for each gap
        for gap in gaps {
            if let Some(mappings) = self.metric_mappings.get(&gap.metric_name) {
                for mapping in mappings {
                    if let Some(patch) = self.generate_patch(&gap, mapping) {
                        result.patches.push(patch);
                        if !result.addressed_metrics.contains(&gap.metric_name) {
                            result.addressed_metrics.push(gap.metric_name.clone());
                        }
                    }
                }
            } else if !result.unaddressable_metrics.contains(&gap.metric_name) {
                result.unaddressable_metrics.push(gap.metric_name.clone());
            }
        }

        // Calculate expected improvement
        if !result.patches.is_empty() {
            let avg_confidence: f64 = result.patches.iter().map(|p| p.confidence).sum::<f64>()
                / result.patches.len() as f64;
            result.expected_improvement = avg_confidence;
        }

        // Generate summary
        result.summary = self.generate_summary(&result);

        result
    }

    /// Identify gaps between current metrics and thresholds.
    fn identify_gaps(&self, evaluation: &ComprehensiveEvaluation) -> Vec<MetricGap> {
        let mut gaps = Vec::new();

        // Check statistical metrics
        if let Some(ref benford) = evaluation.statistical.benford {
            if benford.p_value < self.thresholds.benford_p_value_min {
                gaps.push(MetricGap {
                    metric_name: "benford_p_value".to_string(),
                    current_value: benford.p_value,
                    target_value: self.thresholds.benford_p_value_min,
                    gap: self.thresholds.benford_p_value_min - benford.p_value,
                    is_minimum: true,
                    config_paths: vec!["transactions.amount.benford_compliance".to_string()],
                });
            }
        }

        if let Some(ref amount) = evaluation.statistical.amount_distribution {
            if amount.round_number_ratio < 0.05 {
                gaps.push(MetricGap {
                    metric_name: "round_number_ratio".to_string(),
                    current_value: amount.round_number_ratio,
                    target_value: 0.10, // Target 10%
                    gap: 0.10 - amount.round_number_ratio,
                    is_minimum: true,
                    config_paths: vec!["transactions.amount.round_number_bias".to_string()],
                });
            }
        }

        if let Some(ref temporal) = evaluation.statistical.temporal {
            if temporal.pattern_correlation < self.thresholds.temporal_correlation_min {
                gaps.push(MetricGap {
                    metric_name: "temporal_correlation".to_string(),
                    current_value: temporal.pattern_correlation,
                    target_value: self.thresholds.temporal_correlation_min,
                    gap: self.thresholds.temporal_correlation_min - temporal.pattern_correlation,
                    is_minimum: true,
                    config_paths: vec!["transactions.temporal.seasonality_strength".to_string()],
                });
            }
        }

        // Check coherence metrics
        if let Some(ref ic) = evaluation.coherence.intercompany {
            if ic.match_rate < self.thresholds.ic_match_rate_min {
                gaps.push(MetricGap {
                    metric_name: "ic_match_rate".to_string(),
                    current_value: ic.match_rate,
                    target_value: self.thresholds.ic_match_rate_min,
                    gap: self.thresholds.ic_match_rate_min - ic.match_rate,
                    is_minimum: true,
                    config_paths: vec!["intercompany.match_precision".to_string()],
                });
            }
        }

        if let Some(ref doc_chain) = evaluation.coherence.document_chain {
            let avg_completion =
                (doc_chain.p2p_completion_rate + doc_chain.o2c_completion_rate) / 2.0;
            if avg_completion < self.thresholds.document_chain_completion_min {
                gaps.push(MetricGap {
                    metric_name: "doc_chain_completion".to_string(),
                    current_value: avg_completion,
                    target_value: self.thresholds.document_chain_completion_min,
                    gap: self.thresholds.document_chain_completion_min - avg_completion,
                    is_minimum: true,
                    config_paths: vec![
                        "document_flows.p2p.completion_rate".to_string(),
                        "document_flows.o2c.completion_rate".to_string(),
                    ],
                });
            }
        }

        // Check quality metrics
        if let Some(ref uniqueness) = evaluation.quality.uniqueness {
            if uniqueness.duplicate_rate > self.thresholds.duplicate_rate_max {
                gaps.push(MetricGap {
                    metric_name: "duplicate_rate".to_string(),
                    current_value: uniqueness.duplicate_rate,
                    target_value: self.thresholds.duplicate_rate_max,
                    gap: uniqueness.duplicate_rate - self.thresholds.duplicate_rate_max,
                    is_minimum: false, // This is a maximum threshold
                    config_paths: vec!["data_quality.duplicates.exact_rate".to_string()],
                });
            }
        }

        if let Some(ref completeness) = evaluation.quality.completeness {
            if completeness.overall_completeness < self.thresholds.completeness_rate_min {
                gaps.push(MetricGap {
                    metric_name: "completeness_rate".to_string(),
                    current_value: completeness.overall_completeness,
                    target_value: self.thresholds.completeness_rate_min,
                    gap: self.thresholds.completeness_rate_min - completeness.overall_completeness,
                    is_minimum: true,
                    config_paths: vec!["data_quality.missing_values.overall_rate".to_string()],
                });
            }
        }

        // Check ML metrics
        if let Some(ref labels) = evaluation.ml_readiness.labels {
            if labels.anomaly_rate < self.thresholds.anomaly_rate_min {
                gaps.push(MetricGap {
                    metric_name: "anomaly_rate".to_string(),
                    current_value: labels.anomaly_rate,
                    target_value: self.thresholds.anomaly_rate_min,
                    gap: self.thresholds.anomaly_rate_min - labels.anomaly_rate,
                    is_minimum: true,
                    config_paths: vec!["anomaly_injection.base_rate".to_string()],
                });
            } else if labels.anomaly_rate > self.thresholds.anomaly_rate_max {
                gaps.push(MetricGap {
                    metric_name: "anomaly_rate".to_string(),
                    current_value: labels.anomaly_rate,
                    target_value: self.thresholds.anomaly_rate_max,
                    gap: labels.anomaly_rate - self.thresholds.anomaly_rate_max,
                    is_minimum: false,
                    config_paths: vec!["anomaly_injection.base_rate".to_string()],
                });
            }

            if labels.label_coverage < self.thresholds.label_coverage_min {
                gaps.push(MetricGap {
                    metric_name: "label_coverage".to_string(),
                    current_value: labels.label_coverage,
                    target_value: self.thresholds.label_coverage_min,
                    gap: self.thresholds.label_coverage_min - labels.label_coverage,
                    is_minimum: true,
                    config_paths: vec!["anomaly_injection.label_all".to_string()],
                });
            }
        }

        if let Some(ref graph) = evaluation.ml_readiness.graph {
            if graph.connectivity_score < self.thresholds.graph_connectivity_min {
                gaps.push(MetricGap {
                    metric_name: "graph_connectivity".to_string(),
                    current_value: graph.connectivity_score,
                    target_value: self.thresholds.graph_connectivity_min,
                    gap: self.thresholds.graph_connectivity_min - graph.connectivity_score,
                    is_minimum: true,
                    config_paths: vec!["graph_export.ensure_connected".to_string()],
                });
            }
        }

        // --- New evaluator metric gaps ---

        // HR/Payroll accuracy
        if let Some(ref hr) = evaluation.coherence.hr_payroll {
            if hr.gross_to_net_accuracy < 0.999 {
                gaps.push(MetricGap {
                    metric_name: "payroll_accuracy".to_string(),
                    current_value: hr.gross_to_net_accuracy,
                    target_value: 0.999,
                    gap: 0.999 - hr.gross_to_net_accuracy,
                    is_minimum: true,
                    config_paths: vec!["hr.payroll.calculation_precision".to_string()],
                });
            }
        }

        // Manufacturing yield
        if let Some(ref mfg) = evaluation.coherence.manufacturing {
            if mfg.yield_rate_consistency < 0.95 {
                gaps.push(MetricGap {
                    metric_name: "manufacturing_yield".to_string(),
                    current_value: mfg.yield_rate_consistency,
                    target_value: 0.95,
                    gap: 0.95 - mfg.yield_rate_consistency,
                    is_minimum: true,
                    config_paths: vec!["manufacturing.production_orders.yield_target".to_string()],
                });
            }
        }

        // S2C chain completion
        if let Some(ref sourcing) = evaluation.coherence.sourcing {
            if sourcing.rfx_completion_rate < 0.90 {
                gaps.push(MetricGap {
                    metric_name: "s2c_chain_completion".to_string(),
                    current_value: sourcing.rfx_completion_rate,
                    target_value: 0.90,
                    gap: 0.90 - sourcing.rfx_completion_rate,
                    is_minimum: true,
                    config_paths: vec!["source_to_pay.rfx_completion_rate".to_string()],
                });
            }
        }

        // Anomaly separability
        if let Some(ref as_eval) = evaluation.ml_readiness.anomaly_scoring {
            if as_eval.anomaly_separability < self.thresholds.min_anomaly_separability {
                gaps.push(MetricGap {
                    metric_name: "anomaly_separability".to_string(),
                    current_value: as_eval.anomaly_separability,
                    target_value: self.thresholds.min_anomaly_separability,
                    gap: self.thresholds.min_anomaly_separability - as_eval.anomaly_separability,
                    is_minimum: true,
                    config_paths: vec!["anomaly_injection.base_rate".to_string()],
                });
            }
        }

        // Feature quality
        if let Some(ref fq_eval) = evaluation.ml_readiness.feature_quality {
            if fq_eval.feature_quality_score < self.thresholds.min_feature_quality {
                gaps.push(MetricGap {
                    metric_name: "feature_quality".to_string(),
                    current_value: fq_eval.feature_quality_score,
                    target_value: self.thresholds.min_feature_quality,
                    gap: self.thresholds.min_feature_quality - fq_eval.feature_quality_score,
                    is_minimum: true,
                    config_paths: vec!["graph_export.feature_completeness".to_string()],
                });
            }
        }

        // GNN readiness
        if let Some(ref gnn_eval) = evaluation.ml_readiness.gnn_readiness {
            if gnn_eval.gnn_readiness_score < self.thresholds.min_gnn_readiness {
                gaps.push(MetricGap {
                    metric_name: "gnn_readiness".to_string(),
                    current_value: gnn_eval.gnn_readiness_score,
                    target_value: self.thresholds.min_gnn_readiness,
                    gap: self.thresholds.min_gnn_readiness - gnn_eval.gnn_readiness_score,
                    is_minimum: true,
                    config_paths: vec![
                        "graph_export.ensure_connected".to_string(),
                        "cross_process_links.enabled".to_string(),
                    ],
                });
            }
        }

        // Domain gap (max threshold - lower is better)
        if let Some(ref dg_eval) = evaluation.ml_readiness.domain_gap {
            if dg_eval.domain_gap_score > self.thresholds.max_domain_gap {
                gaps.push(MetricGap {
                    metric_name: "domain_gap".to_string(),
                    current_value: dg_eval.domain_gap_score,
                    target_value: self.thresholds.max_domain_gap,
                    gap: dg_eval.domain_gap_score - self.thresholds.max_domain_gap,
                    is_minimum: false,
                    config_paths: vec!["distributions.industry_profile".to_string()],
                });
            }
        }

        gaps
    }

    /// Generate a config patch for a metric gap.
    fn generate_patch(
        &self,
        gap: &MetricGap,
        mapping: &MetricConfigMapping,
    ) -> Option<ConfigPatch> {
        let suggested_value = match mapping.compute_value {
            ComputeStrategy::EnableBoolean => "true".to_string(),
            ComputeStrategy::SetFixed(v) => format!("{:.4}", v),
            ComputeStrategy::IncreaseByGap => format!("{:.4}", gap.current_value + gap.gap * 1.2),
            ComputeStrategy::DecreaseByGap => {
                format!("{:.4}", (gap.current_value - gap.gap * 1.2).max(0.0))
            }
            ComputeStrategy::SetToTarget => format!("{:.4}", gap.target_value),
            ComputeStrategy::MultiplyByGapFactor => {
                let factor = if gap.is_minimum {
                    1.0 + gap.severity() * 0.5
                } else {
                    1.0 / (1.0 + gap.severity() * 0.5)
                };
                format!("{:.4}", gap.current_value * factor)
            }
        };

        let confidence = mapping.influence * (1.0 - gap.severity() * 0.3);
        let impact = format!(
            "Should improve {} from {:.3} toward {:.3}",
            gap.metric_name, gap.current_value, gap.target_value
        );

        Some(
            ConfigPatch::new(&mapping.config_path, suggested_value)
                .with_current(format!("{:.4}", gap.current_value))
                .with_confidence(confidence)
                .with_impact(impact),
        )
    }

    /// Generate a summary message for the auto-tune result.
    fn generate_summary(&self, result: &AutoTuneResult) -> String {
        if result.patches.is_empty() {
            "No configuration changes suggested. All metrics meet thresholds.".to_string()
        } else {
            let high_confidence: Vec<_> = result
                .patches
                .iter()
                .filter(|p| p.confidence > 0.7)
                .collect();
            let addressable = result.addressed_metrics.len();
            let unaddressable = result.unaddressable_metrics.len();

            format!(
                "Suggested {} configuration changes ({} high-confidence). \
                 {} metrics can be improved, {} require manual investigation.",
                result.patches.len(),
                high_confidence.len(),
                addressable,
                unaddressable
            )
        }
    }

    /// Get the thresholds being used.
    pub fn thresholds(&self) -> &EvaluationThresholds {
        &self.thresholds
    }
}

impl Default for AutoTuner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::statistical::{BenfordAnalysis, BenfordConformity};

    #[test]
    fn test_auto_tuner_creation() {
        let tuner = AutoTuner::new();
        assert!(!tuner.metric_mappings.is_empty());
    }

    #[test]
    fn test_config_patch_builder() {
        let patch = ConfigPatch::new("test.path", "value")
            .with_current("old")
            .with_confidence(0.8)
            .with_impact("Should help");

        assert_eq!(patch.path, "test.path");
        assert_eq!(patch.current_value, Some("old".to_string()));
        assert_eq!(patch.confidence, 0.8);
    }

    #[test]
    fn test_auto_tune_result() {
        let mut result = AutoTuneResult::new();
        assert!(!result.has_patches());

        result
            .patches
            .push(ConfigPatch::new("test", "value").with_confidence(0.9));
        assert!(result.has_patches());

        let sorted = result.patches_by_confidence();
        assert_eq!(sorted.len(), 1);
    }

    #[test]
    fn test_metric_gap_severity() {
        let gap = MetricGap {
            metric_name: "test".to_string(),
            current_value: 0.02,
            target_value: 0.05,
            gap: 0.03,
            is_minimum: true,
            config_paths: vec![],
        };

        // Severity = gap / target = 0.03 / 0.05 = 0.6
        assert!((gap.severity() - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_analyze_empty_evaluation() {
        let tuner = AutoTuner::new();
        let evaluation = ComprehensiveEvaluation::new();

        let result = tuner.analyze(&evaluation);

        // Empty evaluation should produce no patches
        assert!(result.patches.is_empty());
    }

    #[test]
    fn test_analyze_with_benford_gap() {
        let tuner = AutoTuner::new();
        let mut evaluation = ComprehensiveEvaluation::new();

        // Set a failing Benford analysis
        evaluation.statistical.benford = Some(BenfordAnalysis {
            sample_size: 1000,
            observed_frequencies: [0.1; 9],
            observed_counts: [100; 9],
            expected_frequencies: [
                0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
            ],
            chi_squared: 25.0,
            degrees_of_freedom: 8,
            p_value: 0.01, // Below threshold of 0.05
            mad: 0.02,
            conformity: BenfordConformity::NonConforming,
            max_deviation: (1, 0.2), // Tuple of (digit_index, deviation)
            passes: false,
            anti_benford_score: 0.5,
        });

        let result = tuner.analyze(&evaluation);

        // Should suggest enabling Benford compliance
        assert!(!result.patches.is_empty());
        assert!(result
            .addressed_metrics
            .contains(&"benford_p_value".to_string()));
    }
}
