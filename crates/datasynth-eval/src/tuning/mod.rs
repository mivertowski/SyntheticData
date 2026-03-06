//! Configuration tuning and optimization suggestions.
//!
//! Analyzes evaluation results to identify tuning opportunities
//! and generate actionable configuration suggestions.

use crate::ComprehensiveEvaluation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Category of tuning opportunity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TuningCategory {
    /// Statistical distribution tuning (Benford's, amount distributions).
    Statistical,
    /// Balance and coherence tuning.
    Coherence,
    /// Data quality tuning (completeness, uniqueness).
    Quality,
    /// ML-readiness tuning (labels, splits, features).
    MLReadiness,
    /// Performance optimization.
    Performance,
    /// Anomaly injection tuning.
    Anomaly,
}

/// Priority level for tuning recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TuningPriority {
    /// Critical issue that needs immediate attention.
    Critical,
    /// High priority improvement.
    High,
    /// Medium priority enhancement.
    Medium,
    /// Low priority fine-tuning.
    Low,
    /// Informational suggestion.
    Info,
}

/// A tuning opportunity identified from evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningOpportunity {
    /// Category of the tuning opportunity.
    pub category: TuningCategory,
    /// Priority level.
    pub priority: TuningPriority,
    /// Short title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Current value or state.
    pub current_value: Option<String>,
    /// Recommended target value or state.
    pub target_value: Option<String>,
    /// Expected improvement description.
    pub expected_improvement: String,
    /// Related configuration path(s).
    pub config_paths: Vec<String>,
}

impl TuningOpportunity {
    /// Create a new tuning opportunity.
    pub fn new(
        category: TuningCategory,
        priority: TuningPriority,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            category,
            priority,
            title: title.into(),
            description: description.into(),
            current_value: None,
            target_value: None,
            expected_improvement: String::new(),
            config_paths: Vec::new(),
        }
    }

    /// Set current value.
    pub fn with_current_value(mut self, value: impl Into<String>) -> Self {
        self.current_value = Some(value.into());
        self
    }

    /// Set target value.
    pub fn with_target_value(mut self, value: impl Into<String>) -> Self {
        self.target_value = Some(value.into());
        self
    }

    /// Set expected improvement.
    pub fn with_expected_improvement(mut self, improvement: impl Into<String>) -> Self {
        self.expected_improvement = improvement.into();
        self
    }

    /// Add related config path.
    pub fn with_config_path(mut self, path: impl Into<String>) -> Self {
        self.config_paths.push(path.into());
        self
    }
}

/// A specific configuration change suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSuggestion {
    /// Configuration path (e.g., "transactions.amount.round_number_bias").
    pub path: String,
    /// Current value (as string representation).
    pub current_value: String,
    /// Suggested new value.
    pub suggested_value: String,
    /// Reason for the suggestion.
    pub reason: String,
    /// Confidence level (0.0-1.0).
    pub confidence: f64,
    /// Whether this is an automatic fix.
    pub auto_fixable: bool,
}

impl ConfigSuggestion {
    /// Create a new config suggestion.
    pub fn new(
        path: impl Into<String>,
        current_value: impl Into<String>,
        suggested_value: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            current_value: current_value.into(),
            suggested_value: suggested_value.into(),
            reason: reason.into(),
            confidence: 0.5,
            auto_fixable: false,
        }
    }

    /// Set confidence level.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Mark as auto-fixable.
    pub fn auto_fixable(mut self) -> Self {
        self.auto_fixable = true;
        self
    }
}

/// Analyzes evaluation results to identify tuning opportunities.
pub struct TuningAnalyzer {
    /// Minimum threshold gap to trigger a suggestion (as fraction).
    min_gap_fraction: f64,
    /// Whether to include low-priority suggestions.
    include_low_priority: bool,
}

impl TuningAnalyzer {
    /// Create a new tuning analyzer.
    pub fn new() -> Self {
        Self {
            min_gap_fraction: 0.05,
            include_low_priority: true,
        }
    }

    /// Set minimum gap fraction to trigger suggestions.
    pub fn with_min_gap(mut self, gap: f64) -> Self {
        self.min_gap_fraction = gap;
        self
    }

    /// Set whether to include low-priority suggestions.
    pub fn with_low_priority(mut self, include: bool) -> Self {
        self.include_low_priority = include;
        self
    }

    /// Analyze evaluation results and return tuning opportunities.
    pub fn analyze(&self, evaluation: &ComprehensiveEvaluation) -> Vec<TuningOpportunity> {
        let mut opportunities = Vec::new();

        // Analyze statistical issues
        self.analyze_statistical(&evaluation.statistical, &mut opportunities);

        // Analyze coherence issues
        self.analyze_coherence(&evaluation.coherence, &mut opportunities);

        // Analyze quality issues
        self.analyze_quality(&evaluation.quality, &mut opportunities);

        // Analyze ML-readiness issues
        self.analyze_ml_readiness(&evaluation.ml_readiness, &mut opportunities);

        // Filter by priority if needed
        if !self.include_low_priority {
            opportunities.retain(|o| {
                o.priority != TuningPriority::Low && o.priority != TuningPriority::Info
            });
        }

        // Sort by priority
        opportunities.sort_by(|a, b| a.priority.cmp(&b.priority));

        opportunities
    }

    fn analyze_statistical(
        &self,
        stat: &crate::statistical::StatisticalEvaluation,
        opportunities: &mut Vec<TuningOpportunity>,
    ) {
        // Check Benford's Law conformity
        if let Some(ref benford) = stat.benford {
            if benford.p_value < 0.05 {
                let priority = if benford.p_value < 0.01 {
                    TuningPriority::High
                } else {
                    TuningPriority::Medium
                };

                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Statistical,
                        priority,
                        "Benford's Law Non-Conformance",
                        "Generated amounts do not follow Benford's Law distribution",
                    )
                    .with_current_value(format!("p-value: {:.4}", benford.p_value))
                    .with_target_value("p-value > 0.05")
                    .with_expected_improvement("Better statistical realism")
                    .with_config_path("transactions.amount.benford_compliance"),
                );
            }
        }

        // Check amount distribution
        if let Some(ref amount) = stat.amount_distribution {
            if let Some(p_value) = amount.lognormal_ks_pvalue {
                if p_value < 0.05 {
                    opportunities.push(
                        TuningOpportunity::new(
                            TuningCategory::Statistical,
                            TuningPriority::Medium,
                            "Amount Distribution Mismatch",
                            "Amount distribution does not match expected log-normal pattern",
                        )
                        .with_current_value(format!("KS p-value: {p_value:.4}"))
                        .with_target_value("KS p-value > 0.05")
                        .with_expected_improvement("More realistic amount patterns")
                        .with_config_path("transactions.amount.distribution"),
                    );
                }
            }

            // Check round number bias
            if amount.round_number_ratio < 0.05 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Statistical,
                        TuningPriority::Low,
                        "Low Round Number Bias",
                        "Round number occurrence is lower than typically seen in real data",
                    )
                    .with_current_value(format!("{:.1}%", amount.round_number_ratio * 100.0))
                    .with_target_value("5-15%")
                    .with_expected_improvement("More natural-looking amounts")
                    .with_config_path("transactions.amount.round_number_bias"),
                );
            }
        }

        // Check temporal patterns
        if let Some(ref temporal) = stat.temporal {
            if temporal.pattern_correlation < 0.6 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Statistical,
                        TuningPriority::Medium,
                        "Weak Temporal Patterns",
                        "Generated data lacks strong temporal patterns",
                    )
                    .with_current_value(format!("correlation: {:.3}", temporal.pattern_correlation))
                    .with_target_value("correlation > 0.8")
                    .with_expected_improvement("Better temporal realism")
                    .with_config_path("transactions.temporal"),
                );
            }
        }
    }

    fn analyze_coherence(
        &self,
        coherence: &crate::coherence::CoherenceEvaluation,
        opportunities: &mut Vec<TuningOpportunity>,
    ) {
        // Check balance sheet
        if let Some(ref balance) = coherence.balance {
            if !balance.equation_balanced {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Coherence,
                        TuningPriority::Critical,
                        "Balance Sheet Imbalance",
                        "Assets do not equal Liabilities + Equity",
                    )
                    .with_current_value(format!("max imbalance: {}", balance.max_imbalance))
                    .with_target_value("imbalance = 0")
                    .with_expected_improvement("Valid trial balance")
                    .with_config_path("balance.coherence_enabled"),
                );
            }
        }

        // Check subledger reconciliation
        if let Some(ref subledger) = coherence.subledger {
            if subledger.completeness_score < 0.99 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Coherence,
                        TuningPriority::High,
                        "Subledger Reconciliation Issues",
                        "Subledger balances do not fully reconcile to GL control accounts",
                    )
                    .with_current_value(format!("{:.1}%", subledger.completeness_score * 100.0))
                    .with_target_value("> 99%")
                    .with_expected_improvement("Full GL-subledger reconciliation")
                    .with_config_path("subledger"),
                );
            }
        }

        // Check document chains
        if let Some(ref doc_chain) = coherence.document_chain {
            let avg_completion =
                (doc_chain.p2p_completion_rate + doc_chain.o2c_completion_rate) / 2.0;
            if avg_completion < 0.90 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Coherence,
                        TuningPriority::Medium,
                        "Incomplete Document Chains",
                        "Many document flows do not complete to payment/receipt",
                    )
                    .with_current_value(format!(
                        "P2P: {:.1}%, O2C: {:.1}%",
                        doc_chain.p2p_completion_rate * 100.0,
                        doc_chain.o2c_completion_rate * 100.0
                    ))
                    .with_target_value("> 90%")
                    .with_expected_improvement("More complete P2P/O2C flows")
                    .with_config_path("document_flows"),
                );
            }
        }

        // Check IC matching
        if let Some(ref ic) = coherence.intercompany {
            if ic.match_rate < 0.95 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Coherence,
                        TuningPriority::High,
                        "Intercompany Matching Issues",
                        "Intercompany transactions are not fully matched",
                    )
                    .with_current_value(format!("{:.1}%", ic.match_rate * 100.0))
                    .with_target_value("> 95%")
                    .with_expected_improvement("Clean IC reconciliation")
                    .with_config_path("intercompany"),
                );
            }
        }
    }

    fn analyze_quality(
        &self,
        quality: &crate::quality::QualityEvaluation,
        opportunities: &mut Vec<TuningOpportunity>,
    ) {
        // Check uniqueness
        if let Some(ref uniqueness) = quality.uniqueness {
            if uniqueness.duplicate_rate > 0.01 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Quality,
                        TuningPriority::High,
                        "High Duplicate Rate",
                        "Excessive duplicate records detected",
                    )
                    .with_current_value(format!("{:.2}%", uniqueness.duplicate_rate * 100.0))
                    .with_target_value("< 1%")
                    .with_expected_improvement("Cleaner unique data")
                    .with_config_path("data_quality.duplicate_rate"),
                );
            }
        }

        // Check completeness
        if let Some(ref completeness) = quality.completeness {
            if completeness.overall_completeness < 0.95 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Quality,
                        TuningPriority::Medium,
                        "Low Data Completeness",
                        "Many fields have missing values",
                    )
                    .with_current_value(format!(
                        "{:.1}%",
                        completeness.overall_completeness * 100.0
                    ))
                    .with_target_value("> 95%")
                    .with_expected_improvement("More complete records")
                    .with_config_path("data_quality.missing_rate"),
                );
            }
        }

        // Check format consistency
        if let Some(ref format) = quality.format {
            if format.consistency_score < 0.99 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::Quality,
                        TuningPriority::Low,
                        "Format Inconsistencies",
                        "Some fields have inconsistent formats",
                    )
                    .with_current_value(format!("{:.1}%", format.consistency_score * 100.0))
                    .with_target_value("> 99%")
                    .with_expected_improvement("Consistent field formats")
                    .with_config_path("data_quality.format_variations"),
                );
            }
        }
    }

    fn analyze_ml_readiness(
        &self,
        ml: &crate::ml::MLReadinessEvaluation,
        opportunities: &mut Vec<TuningOpportunity>,
    ) {
        // Check labels
        if let Some(ref labels) = ml.labels {
            // Check anomaly rate bounds
            if labels.anomaly_rate < 0.01 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::MLReadiness,
                        TuningPriority::High,
                        "Low Anomaly Rate",
                        "Too few anomalies for effective ML training",
                    )
                    .with_current_value(format!("{:.2}%", labels.anomaly_rate * 100.0))
                    .with_target_value("1-20%")
                    .with_expected_improvement("Better ML model training")
                    .with_config_path("anomaly_injection.base_rate"),
                );
            } else if labels.anomaly_rate > 0.20 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::MLReadiness,
                        TuningPriority::Medium,
                        "High Anomaly Rate",
                        "Too many anomalies may reduce model effectiveness",
                    )
                    .with_current_value(format!("{:.1}%", labels.anomaly_rate * 100.0))
                    .with_target_value("1-20%")
                    .with_expected_improvement("Realistic anomaly distribution")
                    .with_config_path("anomaly_injection.base_rate"),
                );
            }

            // Check label coverage
            if labels.label_coverage < 0.99 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::MLReadiness,
                        TuningPriority::High,
                        "Low Label Coverage",
                        "Not all records have proper labels",
                    )
                    .with_current_value(format!("{:.1}%", labels.label_coverage * 100.0))
                    .with_target_value("> 99%")
                    .with_expected_improvement("Complete supervised labels")
                    .with_config_path("anomaly_injection"),
                );
            }
        }

        // Check splits
        if let Some(ref splits) = ml.splits {
            if !splits.is_valid {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::MLReadiness,
                        TuningPriority::High,
                        "Invalid Train/Test Splits",
                        "Train/validation/test splits have issues",
                    )
                    .with_expected_improvement("Valid ML evaluation setup")
                    .with_config_path("graph_export.train_ratio")
                    .with_config_path("graph_export.validation_ratio"),
                );
            }
        }

        // Check graph structure
        if let Some(ref graph) = ml.graph {
            if graph.connectivity_score < 0.95 {
                opportunities.push(
                    TuningOpportunity::new(
                        TuningCategory::MLReadiness,
                        TuningPriority::Medium,
                        "Low Graph Connectivity",
                        "Transaction graph has isolated components",
                    )
                    .with_current_value(format!("{:.1}%", graph.connectivity_score * 100.0))
                    .with_target_value("> 95%")
                    .with_expected_improvement("Better GNN training")
                    .with_config_path("graph_export"),
                );
            }
        }
    }
}

impl Default for TuningAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates configuration suggestions from tuning opportunities.
pub struct ConfigSuggestionGenerator {
    /// Template suggestions by config path.
    templates: HashMap<String, SuggestionTemplate>,
}

#[derive(Clone)]
struct SuggestionTemplate {
    default_value: String,
    description: String,
    auto_fixable: bool,
}

impl ConfigSuggestionGenerator {
    /// Create a new suggestion generator.
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Add common templates
        templates.insert(
            "transactions.amount.benford_compliance".to_string(),
            SuggestionTemplate {
                default_value: "true".to_string(),
                description: "Enable Benford's Law compliance for amount generation".to_string(),
                auto_fixable: true,
            },
        );

        templates.insert(
            "transactions.amount.round_number_bias".to_string(),
            SuggestionTemplate {
                default_value: "0.10".to_string(),
                description: "Increase round number occurrence rate".to_string(),
                auto_fixable: true,
            },
        );

        templates.insert(
            "anomaly_injection.base_rate".to_string(),
            SuggestionTemplate {
                default_value: "0.05".to_string(),
                description: "Adjust anomaly injection rate".to_string(),
                auto_fixable: true,
            },
        );

        Self { templates }
    }

    /// Generate config suggestions from tuning opportunities.
    pub fn generate(&self, opportunities: &[TuningOpportunity]) -> Vec<ConfigSuggestion> {
        let mut suggestions = Vec::new();

        for opportunity in opportunities {
            for path in &opportunity.config_paths {
                if let Some(template) = self.templates.get(path) {
                    let current = opportunity.current_value.clone().unwrap_or_default();
                    let suggested = opportunity
                        .target_value
                        .clone()
                        .unwrap_or_else(|| template.default_value.clone());

                    let mut suggestion = ConfigSuggestion::new(
                        path.clone(),
                        current,
                        suggested,
                        template.description.clone(),
                    );

                    // Set confidence based on priority
                    let confidence = match opportunity.priority {
                        TuningPriority::Critical => 0.95,
                        TuningPriority::High => 0.85,
                        TuningPriority::Medium => 0.70,
                        TuningPriority::Low => 0.50,
                        TuningPriority::Info => 0.30,
                    };

                    suggestion = suggestion.with_confidence(confidence);

                    if template.auto_fixable {
                        suggestion = suggestion.auto_fixable();
                    }

                    suggestions.push(suggestion);
                }
            }
        }

        suggestions
    }

    /// Add a custom template.
    pub fn add_template(
        &mut self,
        path: impl Into<String>,
        default_value: impl Into<String>,
        description: impl Into<String>,
        auto_fixable: bool,
    ) {
        self.templates.insert(
            path.into(),
            SuggestionTemplate {
                default_value: default_value.into(),
                description: description.into(),
                auto_fixable,
            },
        );
    }
}

impl Default for ConfigSuggestionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_tuning_opportunity_creation() {
        let opportunity = TuningOpportunity::new(
            TuningCategory::Statistical,
            TuningPriority::High,
            "Test Opportunity",
            "Test description",
        )
        .with_current_value("0.01")
        .with_target_value("0.05")
        .with_expected_improvement("Better results")
        .with_config_path("test.path");

        assert_eq!(opportunity.category, TuningCategory::Statistical);
        assert_eq!(opportunity.priority, TuningPriority::High);
        assert_eq!(opportunity.current_value, Some("0.01".to_string()));
        assert_eq!(opportunity.config_paths.len(), 1);
    }

    #[test]
    fn test_config_suggestion_creation() {
        let suggestion =
            ConfigSuggestion::new("test.path", "old_value", "new_value", "Test reason")
                .with_confidence(0.8)
                .auto_fixable();

        assert_eq!(suggestion.path, "test.path");
        assert_eq!(suggestion.confidence, 0.8);
        assert!(suggestion.auto_fixable);
    }

    #[test]
    fn test_tuning_analyzer_default() {
        let analyzer = TuningAnalyzer::default();
        assert!(analyzer.include_low_priority);
    }

    #[test]
    fn test_suggestion_generator() {
        let generator = ConfigSuggestionGenerator::new();
        assert!(generator
            .templates
            .contains_key("anomaly_injection.base_rate"));
    }
}
