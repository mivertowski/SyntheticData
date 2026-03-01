//! Scenario diff types for baseline vs counterfactual comparison.

use serde::{Deserialize, Serialize};

/// Complete diff result for a scenario comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDiff {
    pub summary: Option<ImpactSummary>,
    pub record_level: Option<Vec<RecordLevelDiff>>,
    pub aggregate: Option<AggregateComparison>,
    pub intervention_trace: Option<InterventionTrace>,
}

/// High-level impact summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactSummary {
    pub scenario_name: String,
    pub generation_timestamp: String,
    pub interventions_applied: usize,
    pub kpi_impacts: Vec<KpiImpact>,
    pub financial_statement_impacts: Option<FinancialStatementImpact>,
    pub anomaly_impact: Option<AnomalyImpact>,
    pub control_impact: Option<ControlImpact>,
}

/// Impact on a single KPI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiImpact {
    pub kpi_name: String,
    pub baseline_value: f64,
    pub counterfactual_value: f64,
    pub absolute_change: f64,
    pub percent_change: f64,
    pub direction: ChangeDirection,
}

/// Direction of change.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeDirection {
    Increase,
    Decrease,
    Unchanged,
}

/// Financial statement level impacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialStatementImpact {
    pub revenue_change_pct: f64,
    pub cogs_change_pct: f64,
    pub margin_change_pct: f64,
    pub net_income_change_pct: f64,
    pub total_assets_change_pct: f64,
    pub total_liabilities_change_pct: f64,
    pub cash_flow_change_pct: f64,
    pub top_changed_line_items: Vec<LineItemImpact>,
}

/// Impact on a single line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemImpact {
    pub line_item: String,
    pub baseline: f64,
    pub counterfactual: f64,
    pub change_pct: f64,
}

/// Impact on anomaly counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyImpact {
    pub baseline_count: usize,
    pub counterfactual_count: usize,
    pub new_types: Vec<String>,
    pub removed_types: Vec<String>,
    pub rate_change_pct: f64,
}

/// Impact on internal controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlImpact {
    pub controls_affected: usize,
    pub new_deficiencies: Vec<ControlDeficiency>,
    pub material_weakness_risk: bool,
}

/// A control deficiency resulting from intervention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlDeficiency {
    pub control_id: String,
    pub name: String,
    pub baseline_effectiveness: f64,
    pub counterfactual_effectiveness: f64,
    pub classification: DeficiencyClassification,
}

/// Classification of a control deficiency.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeficiencyClassification {
    Deficiency,
    SignificantDeficiency,
    MaterialWeakness,
}

/// Record-level diff for a single output file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordLevelDiff {
    pub file_name: String,
    pub records_added: usize,
    pub records_removed: usize,
    pub records_modified: usize,
    pub records_unchanged: usize,
    pub sample_changes: Vec<RecordChange>,
}

/// A single record-level change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordChange {
    pub record_id: String,
    pub change_type: RecordChangeType,
    pub field_changes: Vec<FieldChange>,
}

/// Type of record-level change.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecordChangeType {
    Added,
    Removed,
    Modified,
}

/// A field-level change within a record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub field_name: String,
    pub baseline_value: String,
    pub counterfactual_value: String,
}

/// Aggregate comparison across files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateComparison {
    pub metrics: Vec<MetricComparison>,
    pub period_comparisons: Vec<PeriodComparison>,
}

/// Comparison of a single metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparison {
    pub metric_name: String,
    pub baseline: f64,
    pub counterfactual: f64,
    pub change_pct: f64,
}

/// Period-level comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodComparison {
    pub period: String,
    pub metrics: Vec<MetricComparison>,
}

/// Trace of how interventions propagated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionTrace {
    pub traces: Vec<InterventionEffect>,
}

/// Effect of a single intervention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionEffect {
    pub intervention_label: String,
    pub intervention_type: String,
    pub causal_path: Vec<CausalPathStep>,
    pub ultimate_impacts: Vec<KpiImpact>,
}

/// A step in the causal propagation path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalPathStep {
    pub node_id: String,
    pub node_label: String,
    pub input_delta: f64,
    pub output_delta: f64,
    pub transfer_function: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_diff_serde_roundtrip() {
        let diff = ScenarioDiff {
            summary: Some(ImpactSummary {
                scenario_name: "test".to_string(),
                generation_timestamp: "2024-01-01T00:00:00Z".to_string(),
                interventions_applied: 1,
                kpi_impacts: vec![KpiImpact {
                    kpi_name: "total_transactions".to_string(),
                    baseline_value: 1000.0,
                    counterfactual_value: 800.0,
                    absolute_change: -200.0,
                    percent_change: -20.0,
                    direction: ChangeDirection::Decrease,
                }],
                financial_statement_impacts: None,
                anomaly_impact: None,
                control_impact: None,
            }),
            record_level: None,
            aggregate: None,
            intervention_trace: None,
        };

        let json = serde_json::to_string(&diff).expect("serialize");
        let deserialized: ScenarioDiff = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(
            deserialized
                .summary
                .as_ref()
                .expect("has summary")
                .scenario_name,
            "test"
        );
    }

    #[test]
    fn test_change_direction_variants() {
        assert_eq!(ChangeDirection::Increase, ChangeDirection::Increase);
        assert_eq!(ChangeDirection::Decrease, ChangeDirection::Decrease);
        assert_eq!(ChangeDirection::Unchanged, ChangeDirection::Unchanged);
    }

    #[test]
    fn test_record_level_diff_serde() {
        let diff = RecordLevelDiff {
            file_name: "journal_entries.csv".to_string(),
            records_added: 10,
            records_removed: 0,
            records_modified: 50,
            records_unchanged: 940,
            sample_changes: vec![RecordChange {
                record_id: "JE-001".to_string(),
                change_type: RecordChangeType::Modified,
                field_changes: vec![FieldChange {
                    field_name: "amount".to_string(),
                    baseline_value: "1000.00".to_string(),
                    counterfactual_value: "800.00".to_string(),
                }],
            }],
        };

        let json = serde_json::to_string(&diff).expect("serialize");
        let deserialized: RecordLevelDiff = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.file_name, "journal_entries.csv");
        assert_eq!(deserialized.records_modified, 50);
    }

    #[test]
    fn test_aggregate_comparison_serde() {
        let agg = AggregateComparison {
            metrics: vec![MetricComparison {
                metric_name: "total_amount".to_string(),
                baseline: 1_000_000.0,
                counterfactual: 850_000.0,
                change_pct: -15.0,
            }],
            period_comparisons: vec![PeriodComparison {
                period: "2024-01".to_string(),
                metrics: vec![MetricComparison {
                    metric_name: "transaction_count".to_string(),
                    baseline: 100.0,
                    counterfactual: 80.0,
                    change_pct: -20.0,
                }],
            }],
        };

        let json = serde_json::to_string(&agg).expect("serialize");
        let deserialized: AggregateComparison = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.metrics.len(), 1);
        assert_eq!(deserialized.period_comparisons.len(), 1);
    }
}
