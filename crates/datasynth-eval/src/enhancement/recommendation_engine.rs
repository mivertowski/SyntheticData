//! Recommendation engine for providing prioritized enhancement suggestions.
//!
//! The recommendation engine performs root cause analysis on evaluation
//! failures and provides actionable, prioritized recommendations.

use crate::{ComprehensiveEvaluation, EvaluationThresholds};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Priority level for recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RecommendationPriority {
    /// Critical issues that will cause data to fail validation.
    Critical = 0,
    /// High priority issues affecting data quality significantly.
    High = 1,
    /// Medium priority improvements.
    Medium = 2,
    /// Low priority enhancements.
    Low = 3,
    /// Informational only, no action required.
    Info = 4,
}

impl RecommendationPriority {
    /// Get display name.
    pub fn name(&self) -> &'static str {
        match self {
            RecommendationPriority::Critical => "Critical",
            RecommendationPriority::High => "High",
            RecommendationPriority::Medium => "Medium",
            RecommendationPriority::Low => "Low",
            RecommendationPriority::Info => "Info",
        }
    }
}

/// Category of the recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecommendationCategory {
    /// Statistical distribution issues.
    Statistical,
    /// Data coherence issues (balance, subledger, etc.).
    Coherence,
    /// Data quality issues (duplicates, missing, etc.).
    DataQuality,
    /// ML readiness issues.
    MLReadiness,
    /// Performance issues.
    Performance,
    /// Configuration issues.
    Configuration,
}

impl RecommendationCategory {
    /// Get display name.
    pub fn name(&self) -> &'static str {
        match self {
            RecommendationCategory::Statistical => "Statistical Quality",
            RecommendationCategory::Coherence => "Data Coherence",
            RecommendationCategory::DataQuality => "Data Quality",
            RecommendationCategory::MLReadiness => "ML Readiness",
            RecommendationCategory::Performance => "Performance",
            RecommendationCategory::Configuration => "Configuration",
        }
    }
}

/// Root cause identified for an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    /// Short description of the root cause.
    pub description: String,
    /// Detailed explanation.
    pub explanation: String,
    /// Evidence supporting this root cause.
    pub evidence: Vec<String>,
    /// Confidence level (0.0-1.0).
    pub confidence: f64,
}

impl RootCause {
    /// Create a new root cause.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            explanation: String::new(),
            evidence: Vec::new(),
            confidence: 0.5,
        }
    }

    /// Add explanation.
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }

    /// Add evidence.
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    /// Set confidence.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// A single recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Unique identifier.
    pub id: String,
    /// Priority level.
    pub priority: RecommendationPriority,
    /// Category.
    pub category: RecommendationCategory,
    /// Short title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Root causes identified.
    pub root_causes: Vec<RootCause>,
    /// Suggested actions to take.
    pub actions: Vec<SuggestedAction>,
    /// Metrics affected.
    pub affected_metrics: Vec<String>,
    /// Expected improvement if addressed.
    pub expected_improvement: String,
}

impl Recommendation {
    /// Create a new recommendation.
    pub fn new(
        id: impl Into<String>,
        priority: RecommendationPriority,
        category: RecommendationCategory,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            priority,
            category,
            title: title.into(),
            description: String::new(),
            root_causes: Vec::new(),
            actions: Vec::new(),
            affected_metrics: Vec::new(),
            expected_improvement: String::new(),
        }
    }

    /// Add description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add root cause.
    pub fn with_root_cause(mut self, root_cause: RootCause) -> Self {
        self.root_causes.push(root_cause);
        self
    }

    /// Add action.
    pub fn with_action(mut self, action: SuggestedAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Add affected metric.
    pub fn with_affected_metric(mut self, metric: impl Into<String>) -> Self {
        self.affected_metrics.push(metric.into());
        self
    }

    /// Set expected improvement.
    pub fn with_expected_improvement(mut self, improvement: impl Into<String>) -> Self {
        self.expected_improvement = improvement.into();
        self
    }
}

/// A suggested action to address an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Description of the action.
    pub description: String,
    /// Configuration path if applicable.
    pub config_path: Option<String>,
    /// Suggested value if applicable.
    pub suggested_value: Option<String>,
    /// Whether this can be automatically applied.
    pub auto_applicable: bool,
    /// Estimated effort (Low, Medium, High).
    pub effort: String,
}

impl SuggestedAction {
    /// Create a new action.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            config_path: None,
            suggested_value: None,
            auto_applicable: false,
            effort: "Medium".to_string(),
        }
    }

    /// Set config change.
    pub fn with_config_change(mut self, path: impl Into<String>, value: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self.suggested_value = Some(value.into());
        self.auto_applicable = true;
        self
    }

    /// Set effort level.
    pub fn with_effort(mut self, effort: impl Into<String>) -> Self {
        self.effort = effort.into();
        self
    }

    /// Mark as not auto-applicable.
    pub fn manual_only(mut self) -> Self {
        self.auto_applicable = false;
        self
    }
}

/// Enhancement report containing all recommendations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementReport {
    /// All recommendations.
    pub recommendations: Vec<Recommendation>,
    /// Summary by category.
    pub category_summary: HashMap<String, usize>,
    /// Summary by priority.
    pub priority_summary: HashMap<String, usize>,
    /// Overall health score (0.0-1.0).
    pub health_score: f64,
    /// Top issues to address.
    pub top_issues: Vec<String>,
    /// Quick wins (easy to fix with high impact).
    pub quick_wins: Vec<String>,
}

impl EnhancementReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self {
            recommendations: Vec::new(),
            category_summary: HashMap::new(),
            priority_summary: HashMap::new(),
            health_score: 1.0,
            top_issues: Vec::new(),
            quick_wins: Vec::new(),
        }
    }

    /// Add a recommendation.
    pub fn add(&mut self, recommendation: Recommendation) {
        // Update summaries
        *self
            .category_summary
            .entry(recommendation.category.name().to_string())
            .or_insert(0) += 1;
        *self
            .priority_summary
            .entry(recommendation.priority.name().to_string())
            .or_insert(0) += 1;

        self.recommendations.push(recommendation);
    }

    /// Finalize the report (calculate scores, sort, etc.).
    pub fn finalize(&mut self) {
        // Sort recommendations by priority
        self.recommendations
            .sort_by(|a, b| a.priority.cmp(&b.priority));

        // Calculate health score
        let critical_count = *self.priority_summary.get("Critical").unwrap_or(&0);
        let high_count = *self.priority_summary.get("High").unwrap_or(&0);
        let medium_count = *self.priority_summary.get("Medium").unwrap_or(&0);

        let penalty =
            critical_count as f64 * 0.3 + high_count as f64 * 0.1 + medium_count as f64 * 0.02;
        self.health_score = (1.0 - penalty).max(0.0);

        // Identify top issues (critical and high priority)
        self.top_issues = self
            .recommendations
            .iter()
            .filter(|r| {
                r.priority == RecommendationPriority::Critical
                    || r.priority == RecommendationPriority::High
            })
            .take(5)
            .map(|r| r.title.clone())
            .collect();

        // Identify quick wins (auto-applicable actions)
        self.quick_wins = self
            .recommendations
            .iter()
            .filter(|r| r.actions.iter().any(|a| a.auto_applicable))
            .take(5)
            .map(|r| r.title.clone())
            .collect();
    }

    /// Get recommendations by category.
    pub fn by_category(&self, category: RecommendationCategory) -> Vec<&Recommendation> {
        self.recommendations
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    /// Get recommendations by priority.
    pub fn by_priority(&self, priority: RecommendationPriority) -> Vec<&Recommendation> {
        self.recommendations
            .iter()
            .filter(|r| r.priority == priority)
            .collect()
    }

    /// Check if there are critical issues.
    pub fn has_critical_issues(&self) -> bool {
        *self.priority_summary.get("Critical").unwrap_or(&0) > 0
    }
}

impl Default for EnhancementReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Engine for generating recommendations from evaluation results.
pub struct RecommendationEngine {
    /// Thresholds for comparison.
    thresholds: EvaluationThresholds,
    /// Counter for generating unique IDs.
    id_counter: u32,
}

impl RecommendationEngine {
    /// Create a new recommendation engine.
    pub fn new() -> Self {
        Self::with_thresholds(EvaluationThresholds::default())
    }

    /// Create with specific thresholds.
    pub fn with_thresholds(thresholds: EvaluationThresholds) -> Self {
        Self {
            thresholds,
            id_counter: 0,
        }
    }

    /// Generate an enhancement report from evaluation results.
    pub fn generate_report(&mut self, evaluation: &ComprehensiveEvaluation) -> EnhancementReport {
        let mut report = EnhancementReport::new();

        // Analyze statistical issues
        self.analyze_statistical(&evaluation.statistical, &mut report);

        // Analyze coherence issues
        self.analyze_coherence(&evaluation.coherence, &mut report);

        // Analyze quality issues
        self.analyze_quality(&evaluation.quality, &mut report);

        // Analyze ML readiness issues
        self.analyze_ml_readiness(&evaluation.ml_readiness, &mut report);

        // Finalize the report
        report.finalize();

        report
    }

    /// Generate a unique ID.
    fn next_id(&mut self) -> String {
        self.id_counter += 1;
        format!("REC-{:04}", self.id_counter)
    }

    /// Analyze statistical evaluation results.
    fn analyze_statistical(
        &mut self,
        stat: &crate::statistical::StatisticalEvaluation,
        report: &mut EnhancementReport,
    ) {
        // Check Benford's Law
        if let Some(ref benford) = stat.benford {
            if benford.p_value < self.thresholds.benford_p_value_min {
                let severity = if benford.p_value < 0.01 {
                    RecommendationPriority::High
                } else {
                    RecommendationPriority::Medium
                };

                let rec = Recommendation::new(
                    self.next_id(),
                    severity,
                    RecommendationCategory::Statistical,
                    "Benford's Law Non-Conformance",
                )
                .with_description(
                    "Generated transaction amounts do not follow Benford's Law, \
                     which may indicate unrealistic data patterns.",
                )
                .with_root_cause(
                    RootCause::new("Amount generation not using Benford-compliant distribution")
                        .with_explanation(
                            "Real financial data naturally follows Benford's Law for first digits. \
                             Random or uniform distributions will fail this test.",
                        )
                        .with_evidence(format!("p-value: {:.4} (threshold: {:.4})", benford.p_value, self.thresholds.benford_p_value_min))
                        .with_confidence(0.9),
                )
                .with_action(
                    SuggestedAction::new("Enable Benford's Law compliance in amount generation")
                        .with_config_change("transactions.amount.benford_compliance", "true")
                        .with_effort("Low"),
                )
                .with_affected_metric("benford_p_value")
                .with_expected_improvement("Statistical p-value should increase to > 0.05");

                report.add(rec);
            }
        }

        // Check temporal patterns
        if let Some(ref temporal) = stat.temporal {
            if temporal.pattern_correlation < self.thresholds.temporal_correlation_min {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::Medium,
                    RecommendationCategory::Statistical,
                    "Weak Temporal Patterns",
                )
                .with_description(
                    "Generated data lacks realistic temporal patterns such as \
                     seasonality, month-end spikes, and weekday variations.",
                )
                .with_root_cause(
                    RootCause::new("Insufficient temporal variation in generation")
                        .with_explanation(
                            "Real financial data shows strong temporal patterns including \
                             month-end closing activity, seasonal variations, and weekday effects.",
                        )
                        .with_evidence(format!(
                            "Correlation: {:.3} (threshold: {:.3})",
                            temporal.pattern_correlation, self.thresholds.temporal_correlation_min
                        ))
                        .with_confidence(0.75),
                )
                .with_action(
                    SuggestedAction::new("Increase seasonality strength")
                        .with_config_change("transactions.temporal.seasonality_strength", "0.8")
                        .with_effort("Low"),
                )
                .with_action(
                    SuggestedAction::new("Enable month-end spike patterns")
                        .with_config_change("transactions.temporal.month_end_spike", "true")
                        .with_effort("Low"),
                )
                .with_affected_metric("temporal_correlation")
                .with_expected_improvement("Better temporal pattern correlation (> 0.8)");

                report.add(rec);
            }
        }
    }

    /// Analyze coherence evaluation results.
    fn analyze_coherence(
        &mut self,
        coherence: &crate::coherence::CoherenceEvaluation,
        report: &mut EnhancementReport,
    ) {
        // Check balance sheet
        if let Some(ref balance) = coherence.balance {
            if !balance.equation_balanced {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::Critical,
                    RecommendationCategory::Coherence,
                    "Balance Sheet Imbalance",
                )
                .with_description(
                    "The fundamental accounting equation (Assets = Liabilities + Equity) is violated. \
                     This is a critical data integrity issue.",
                )
                .with_root_cause(
                    RootCause::new("Unbalanced journal entries generated")
                        .with_explanation(
                            "Every journal entry must have equal debits and credits. \
                             An imbalance indicates entries were created incorrectly.",
                        )
                        .with_evidence(format!("Max imbalance: {}", balance.max_imbalance))
                        .with_confidence(0.95),
                )
                .with_action(
                    SuggestedAction::new("Enable balance coherence validation")
                        .with_config_change("balance.coherence_enabled", "true")
                        .with_effort("Low"),
                )
                .with_action(
                    SuggestedAction::new("Review JE generation logic for balance enforcement")
                        .manual_only()
                        .with_effort("High"),
                )
                .with_affected_metric("balance_equation")
                .with_expected_improvement("Zero imbalance in trial balance");

                report.add(rec);
            }
        }

        // Check intercompany matching
        if let Some(ref ic) = coherence.intercompany {
            if ic.match_rate < self.thresholds.ic_match_rate_min {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::High,
                    RecommendationCategory::Coherence,
                    "Intercompany Matching Issues",
                )
                .with_description(
                    "Intercompany transactions are not fully matched between entities. \
                     This will cause issues during consolidation.",
                )
                .with_root_cause(
                    RootCause::new("IC transaction pairs not properly linked")
                        .with_explanation(
                            "Intercompany transactions should always have matching entries \
                             in both the selling and buying entities.",
                        )
                        .with_evidence(format!(
                            "Match rate: {:.1}% (threshold: {:.1}%)",
                            ic.match_rate * 100.0,
                            self.thresholds.ic_match_rate_min * 100.0
                        ))
                        .with_confidence(0.85),
                )
                .with_action(
                    SuggestedAction::new("Increase IC matching precision")
                        .with_config_change("intercompany.match_precision", "0.99")
                        .with_effort("Low"),
                )
                .with_affected_metric("ic_match_rate")
                .with_expected_improvement("IC match rate > 95%");

                report.add(rec);
            }
        }

        // Check document chains
        if let Some(ref doc_chain) = coherence.document_chain {
            let avg_completion =
                (doc_chain.p2p_completion_rate + doc_chain.o2c_completion_rate) / 2.0;
            if avg_completion < self.thresholds.document_chain_completion_min {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::Medium,
                    RecommendationCategory::Coherence,
                    "Incomplete Document Chains",
                )
                .with_description(
                    "Many document flows (P2P, O2C) do not complete to final payment/receipt. \
                     This reduces realism for AP/AR aging analysis.",
                )
                .with_root_cause(
                    RootCause::new("Document flow completion rates set too low")
                        .with_explanation(
                            "Real business processes typically complete most document flows. \
                             Very low completion rates may not be realistic.",
                        )
                        .with_evidence(format!(
                            "P2P: {:.1}%, O2C: {:.1}% (threshold: {:.1}%)",
                            doc_chain.p2p_completion_rate * 100.0,
                            doc_chain.o2c_completion_rate * 100.0,
                            self.thresholds.document_chain_completion_min * 100.0
                        ))
                        .with_confidence(0.7),
                )
                .with_action(
                    SuggestedAction::new("Increase P2P completion rate")
                        .with_config_change("document_flows.p2p.completion_rate", "0.95")
                        .with_effort("Low"),
                )
                .with_action(
                    SuggestedAction::new("Increase O2C completion rate")
                        .with_config_change("document_flows.o2c.completion_rate", "0.95")
                        .with_effort("Low"),
                )
                .with_affected_metric("doc_chain_completion")
                .with_expected_improvement("Document chain completion > 90%");

                report.add(rec);
            }
        }
    }

    /// Analyze quality evaluation results.
    fn analyze_quality(
        &mut self,
        quality: &crate::quality::QualityEvaluation,
        report: &mut EnhancementReport,
    ) {
        // Check duplicates
        if let Some(ref uniqueness) = quality.uniqueness {
            if uniqueness.duplicate_rate > self.thresholds.duplicate_rate_max {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::High,
                    RecommendationCategory::DataQuality,
                    "High Duplicate Rate",
                )
                .with_description(
                    "Excessive duplicate records detected in the generated data. \
                     This may cause issues in downstream processing.",
                )
                .with_root_cause(
                    RootCause::new("Duplicate injection rate set too high")
                        .with_explanation(
                            "Data quality variations can inject duplicates, but \
                             high rates may be unrealistic for most use cases.",
                        )
                        .with_evidence(format!(
                            "Duplicate rate: {:.2}% (threshold: {:.2}%)",
                            uniqueness.duplicate_rate * 100.0,
                            self.thresholds.duplicate_rate_max * 100.0
                        ))
                        .with_confidence(0.9),
                )
                .with_action(
                    SuggestedAction::new("Reduce duplicate injection rate")
                        .with_config_change("data_quality.duplicates.exact_rate", "0.005")
                        .with_effort("Low"),
                )
                .with_affected_metric("duplicate_rate")
                .with_expected_improvement("Duplicate rate < 1%");

                report.add(rec);
            }
        }

        // Check completeness
        if let Some(ref completeness) = quality.completeness {
            if completeness.overall_completeness < self.thresholds.completeness_rate_min {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::Medium,
                    RecommendationCategory::DataQuality,
                    "Low Data Completeness",
                )
                .with_description(
                    "Many fields have missing values. While some missing data is realistic, \
                     excessive missing values may reduce data utility.",
                )
                .with_root_cause(
                    RootCause::new("Missing value injection rate set too high")
                        .with_explanation(
                            "Data quality variations inject missing values to simulate \
                             real-world data quality issues, but rates may be too aggressive.",
                        )
                        .with_evidence(format!(
                            "Completeness: {:.1}% (threshold: {:.1}%)",
                            completeness.overall_completeness * 100.0,
                            self.thresholds.completeness_rate_min * 100.0
                        ))
                        .with_confidence(0.8),
                )
                .with_action(
                    SuggestedAction::new("Reduce missing value injection rate")
                        .with_config_change("data_quality.missing_values.overall_rate", "0.02")
                        .with_effort("Low"),
                )
                .with_affected_metric("completeness_rate")
                .with_expected_improvement("Completeness > 95%");

                report.add(rec);
            }
        }
    }

    /// Analyze ML readiness evaluation results.
    fn analyze_ml_readiness(
        &mut self,
        ml: &crate::ml::MLReadinessEvaluation,
        report: &mut EnhancementReport,
    ) {
        // Check anomaly rate
        if let Some(ref labels) = ml.labels {
            if labels.anomaly_rate < self.thresholds.anomaly_rate_min {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::High,
                    RecommendationCategory::MLReadiness,
                    "Insufficient Anomaly Rate",
                )
                .with_description(
                    "Too few anomalies for effective ML training. Anomaly detection \
                     models need sufficient positive examples.",
                )
                .with_root_cause(
                    RootCause::new("Anomaly injection rate set too low")
                        .with_explanation(
                            "ML models for anomaly detection typically need 1-10% anomaly rate \
                             during training to learn effective patterns.",
                        )
                        .with_evidence(format!(
                            "Anomaly rate: {:.2}% (minimum: {:.2}%)",
                            labels.anomaly_rate * 100.0,
                            self.thresholds.anomaly_rate_min * 100.0
                        ))
                        .with_confidence(0.9),
                )
                .with_action(
                    SuggestedAction::new("Increase anomaly injection rate")
                        .with_config_change("anomaly_injection.base_rate", "0.05")
                        .with_effort("Low"),
                )
                .with_affected_metric("anomaly_rate")
                .with_expected_improvement("Anomaly rate 1-10% for ML training");

                report.add(rec);
            } else if labels.anomaly_rate > self.thresholds.anomaly_rate_max {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::Medium,
                    RecommendationCategory::MLReadiness,
                    "Excessive Anomaly Rate",
                )
                .with_description(
                    "Too many anomalies may reduce model effectiveness and make \
                     the data unrealistic for testing.",
                )
                .with_root_cause(
                    RootCause::new("Anomaly injection rate set too high")
                        .with_explanation(
                            "While anomalies are needed for ML training, rates above 20% \
                             are typically unrealistic and may confuse models.",
                        )
                        .with_evidence(format!(
                            "Anomaly rate: {:.1}% (maximum: {:.1}%)",
                            labels.anomaly_rate * 100.0,
                            self.thresholds.anomaly_rate_max * 100.0
                        ))
                        .with_confidence(0.75),
                )
                .with_action(
                    SuggestedAction::new("Reduce anomaly injection rate")
                        .with_config_change("anomaly_injection.base_rate", "0.05")
                        .with_effort("Low"),
                )
                .with_affected_metric("anomaly_rate")
                .with_expected_improvement("Anomaly rate within 1-20% range");

                report.add(rec);
            }

            // Check label coverage
            if labels.label_coverage < self.thresholds.label_coverage_min {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::High,
                    RecommendationCategory::MLReadiness,
                    "Incomplete Label Coverage",
                )
                .with_description(
                    "Not all records have proper labels. Supervised ML requires \
                     complete labels for training.",
                )
                .with_root_cause(
                    RootCause::new("Label generation not capturing all anomalies")
                        .with_explanation(
                            "Every injected anomaly should have a corresponding label. \
                             Missing labels indicate a labeling pipeline issue.",
                        )
                        .with_evidence(format!(
                            "Label coverage: {:.1}% (threshold: {:.1}%)",
                            labels.label_coverage * 100.0,
                            self.thresholds.label_coverage_min * 100.0
                        ))
                        .with_confidence(0.85),
                )
                .with_action(
                    SuggestedAction::new("Enable complete label generation")
                        .with_config_change("anomaly_injection.label_all", "true")
                        .with_effort("Low"),
                )
                .with_affected_metric("label_coverage")
                .with_expected_improvement("Label coverage > 99%");

                report.add(rec);
            }
        }

        // Check graph connectivity
        if let Some(ref graph) = ml.graph {
            if graph.connectivity_score < self.thresholds.graph_connectivity_min {
                let rec = Recommendation::new(
                    self.next_id(),
                    RecommendationPriority::Medium,
                    RecommendationCategory::MLReadiness,
                    "Low Graph Connectivity",
                )
                .with_description(
                    "The transaction graph has isolated components, which may \
                     reduce GNN model effectiveness.",
                )
                .with_root_cause(
                    RootCause::new("Insufficient entity relationships in generated data")
                        .with_explanation(
                            "Graph neural networks benefit from well-connected graphs. \
                             Isolated components receive no message passing.",
                        )
                        .with_evidence(format!(
                            "Connectivity: {:.1}% (threshold: {:.1}%)",
                            graph.connectivity_score * 100.0,
                            self.thresholds.graph_connectivity_min * 100.0
                        ))
                        .with_confidence(0.7),
                )
                .with_action(
                    SuggestedAction::new("Enable graph connectivity enforcement")
                        .with_config_change("graph_export.ensure_connected", "true")
                        .with_effort("Medium"),
                )
                .with_affected_metric("graph_connectivity")
                .with_expected_improvement("Graph connectivity > 95%");

                report.add(rec);
            }
        }
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_builder() {
        let rec = Recommendation::new(
            "REC-001",
            RecommendationPriority::High,
            RecommendationCategory::Statistical,
            "Test Issue",
        )
        .with_description("Test description")
        .with_root_cause(RootCause::new("Test cause").with_confidence(0.8))
        .with_action(SuggestedAction::new("Fix it").with_config_change("test.path", "value"));

        assert_eq!(rec.id, "REC-001");
        assert_eq!(rec.priority, RecommendationPriority::High);
        assert_eq!(rec.root_causes.len(), 1);
        assert_eq!(rec.actions.len(), 1);
    }

    #[test]
    fn test_enhancement_report() {
        let mut report = EnhancementReport::new();

        report.add(Recommendation::new(
            "REC-001",
            RecommendationPriority::Critical,
            RecommendationCategory::Coherence,
            "Critical Issue",
        ));

        report.add(Recommendation::new(
            "REC-002",
            RecommendationPriority::Low,
            RecommendationCategory::DataQuality,
            "Minor Issue",
        ));

        report.finalize();

        assert!(report.has_critical_issues());
        assert_eq!(report.recommendations.len(), 2);
        assert!(report.health_score < 1.0);
    }

    #[test]
    fn test_recommendation_engine() {
        let mut engine = RecommendationEngine::new();
        let evaluation = ComprehensiveEvaluation::new();

        let report = engine.generate_report(&evaluation);

        // Empty evaluation should produce no recommendations
        assert!(report.recommendations.is_empty());
        assert_eq!(report.health_score, 1.0);
    }

    #[test]
    fn test_root_cause_builder() {
        let cause = RootCause::new("Test cause")
            .with_explanation("Detailed explanation")
            .with_evidence("Evidence 1")
            .with_evidence("Evidence 2")
            .with_confidence(0.9);

        assert_eq!(cause.evidence.len(), 2);
        assert_eq!(cause.confidence, 0.9);
    }

    #[test]
    fn test_suggested_action() {
        let action = SuggestedAction::new("Do something")
            .with_config_change("path", "value")
            .with_effort("Low");

        assert!(action.auto_applicable);
        assert_eq!(action.config_path, Some("path".to_string()));
    }
}
