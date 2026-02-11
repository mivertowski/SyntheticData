//! Anomaly label export functionality.
//!
//! This module provides functions for exporting anomaly labels to various formats
//! (CSV, JSON, JSON Lines) for ML training and audit purposes.

use datasynth_core::models::LabeledAnomaly;
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Error type for label export operations.
#[derive(Debug, thiserror::Error)]
pub enum LabelExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for label export operations.
pub type LabelExportResult<T> = Result<T, LabelExportError>;

/// Flattened anomaly label for CSV export.
/// This structure converts nested fields to flat columns for easier CSV handling.
#[derive(Debug, Serialize)]
pub struct FlatAnomalyLabel {
    // Core fields
    pub anomaly_id: String,
    pub anomaly_category: String,
    pub anomaly_type: String,
    pub document_id: String,
    pub document_type: String,
    pub company_code: String,
    pub anomaly_date: String,
    pub detection_timestamp: String,
    pub confidence: f64,
    pub severity: u8,
    pub description: String,
    pub is_injected: bool,
    pub monetary_impact: Option<String>,
    pub related_entities: String, // JSON array as string
    pub cluster_id: Option<String>,

    // Provenance fields
    pub original_document_hash: Option<String>,
    pub injection_strategy: Option<String>,
    pub structured_strategy_type: Option<String>,
    pub structured_strategy_json: Option<String>,
    pub causal_reason_type: Option<String>,
    pub causal_reason_json: Option<String>,
    pub parent_anomaly_id: Option<String>,
    pub child_anomaly_ids: String, // JSON array as string
    pub scenario_id: Option<String>,
    pub run_id: Option<String>,
    pub generation_seed: Option<u64>,

    // Metadata as JSON
    pub metadata_json: String,
}

impl From<&LabeledAnomaly> for FlatAnomalyLabel {
    fn from(label: &LabeledAnomaly) -> Self {
        Self {
            anomaly_id: label.anomaly_id.clone(),
            anomaly_category: label.anomaly_type.category().to_string(),
            anomaly_type: label.anomaly_type.type_name(),
            document_id: label.document_id.clone(),
            document_type: label.document_type.clone(),
            company_code: label.company_code.clone(),
            anomaly_date: label.anomaly_date.to_string(),
            detection_timestamp: label.detection_timestamp.to_string(),
            confidence: label.confidence,
            severity: label.severity,
            description: label.description.clone(),
            is_injected: label.is_injected,
            monetary_impact: label.monetary_impact.map(|d| d.to_string()),
            related_entities: serde_json::to_string(&label.related_entities).unwrap_or_default(),
            cluster_id: label.cluster_id.clone(),

            // Provenance fields
            original_document_hash: label.original_document_hash.clone(),
            injection_strategy: label.injection_strategy.clone(),
            structured_strategy_type: label
                .structured_strategy
                .as_ref()
                .map(|s| s.strategy_type().to_string()),
            structured_strategy_json: label
                .structured_strategy
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap_or_default()),
            causal_reason_type: label.causal_reason.as_ref().map(|r| match r {
                datasynth_core::models::AnomalyCausalReason::RandomRate { .. } => {
                    "RandomRate".to_string()
                }
                datasynth_core::models::AnomalyCausalReason::TemporalPattern { .. } => {
                    "TemporalPattern".to_string()
                }
                datasynth_core::models::AnomalyCausalReason::EntityTargeting { .. } => {
                    "EntityTargeting".to_string()
                }
                datasynth_core::models::AnomalyCausalReason::ClusterMembership { .. } => {
                    "ClusterMembership".to_string()
                }
                datasynth_core::models::AnomalyCausalReason::ScenarioStep { .. } => {
                    "ScenarioStep".to_string()
                }
                datasynth_core::models::AnomalyCausalReason::DataQualityProfile { .. } => {
                    "DataQualityProfile".to_string()
                }
                datasynth_core::models::AnomalyCausalReason::MLTrainingBalance { .. } => {
                    "MLTrainingBalance".to_string()
                }
            }),
            causal_reason_json: label
                .causal_reason
                .as_ref()
                .map(|r| serde_json::to_string(r).unwrap_or_default()),
            parent_anomaly_id: label.parent_anomaly_id.clone(),
            child_anomaly_ids: serde_json::to_string(&label.child_anomaly_ids).unwrap_or_default(),
            scenario_id: label.scenario_id.clone(),
            run_id: label.run_id.clone(),
            generation_seed: label.generation_seed,

            metadata_json: serde_json::to_string(&label.metadata).unwrap_or_default(),
        }
    }
}

/// Configuration for label export.
#[derive(Debug, Clone)]
pub struct LabelExportConfig {
    /// Whether to include all provenance fields.
    pub include_provenance: bool,
    /// Whether to include metadata JSON.
    pub include_metadata: bool,
    /// Whether to pretty-print JSON output.
    pub pretty_json: bool,
}

impl Default for LabelExportConfig {
    fn default() -> Self {
        Self {
            include_provenance: true,
            include_metadata: true,
            pretty_json: true,
        }
    }
}

/// Exports anomaly labels to a CSV file.
pub fn export_labels_csv(
    labels: &[LabeledAnomaly],
    path: &Path,
    _config: &LabelExportConfig,
) -> LabelExportResult<usize> {
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));

    for label in labels {
        let flat: FlatAnomalyLabel = label.into();
        writer.serialize(&flat)?;
    }

    writer.flush()?;
    Ok(labels.len())
}

/// Exports anomaly labels to a JSON file (array format).
pub fn export_labels_json(
    labels: &[LabeledAnomaly],
    path: &Path,
    config: &LabelExportConfig,
) -> LabelExportResult<usize> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    if config.pretty_json {
        serde_json::to_writer_pretty(writer, labels)?;
    } else {
        serde_json::to_writer(writer, labels)?;
    }

    Ok(labels.len())
}

/// Exports anomaly labels to a JSON Lines file (one JSON object per line).
pub fn export_labels_jsonl(
    labels: &[LabeledAnomaly],
    path: &Path,
    _config: &LabelExportConfig,
) -> LabelExportResult<usize> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for label in labels {
        let json = serde_json::to_string(label)?;
        writeln!(writer, "{}", json)?;
    }

    writer.flush()?;
    Ok(labels.len())
}

/// Exports anomaly labels to multiple formats at once.
pub fn export_labels_all_formats(
    labels: &[LabeledAnomaly],
    output_dir: &Path,
    base_name: &str,
    config: &LabelExportConfig,
) -> LabelExportResult<Vec<(String, usize)>> {
    std::fs::create_dir_all(output_dir)?;

    let mut results = Vec::new();

    // CSV
    let csv_path = output_dir.join(format!("{}.csv", base_name));
    let count = export_labels_csv(labels, &csv_path, config)?;
    results.push((csv_path.display().to_string(), count));

    // JSON
    let json_path = output_dir.join(format!("{}.json", base_name));
    let count = export_labels_json(labels, &json_path, config)?;
    results.push((json_path.display().to_string(), count));

    // JSONL
    let jsonl_path = output_dir.join(format!("{}.jsonl", base_name));
    let count = export_labels_jsonl(labels, &jsonl_path, config)?;
    results.push((jsonl_path.display().to_string(), count));

    Ok(results)
}

/// Summary statistics for exported labels.
#[derive(Debug, Clone, Serialize)]
pub struct LabelExportSummary {
    /// Total labels exported.
    pub total_labels: usize,
    /// Labels by category.
    pub by_category: std::collections::HashMap<String, usize>,
    /// Labels by company.
    pub by_company: std::collections::HashMap<String, usize>,
    /// Labels with provenance.
    pub with_provenance: usize,
    /// Labels in scenarios.
    pub in_scenarios: usize,
    /// Labels in clusters.
    pub in_clusters: usize,
}

impl LabelExportSummary {
    /// Creates a summary from a list of labels.
    pub fn from_labels(labels: &[LabeledAnomaly]) -> Self {
        let mut by_category = std::collections::HashMap::new();
        let mut by_company = std::collections::HashMap::new();
        let mut with_provenance = 0;
        let mut in_scenarios = 0;
        let mut in_clusters = 0;

        for label in labels {
            *by_category
                .entry(label.anomaly_type.category().to_string())
                .or_insert(0) += 1;
            *by_company.entry(label.company_code.clone()).or_insert(0) += 1;

            if label.causal_reason.is_some() || label.structured_strategy.is_some() {
                with_provenance += 1;
            }
            if label.scenario_id.is_some() {
                in_scenarios += 1;
            }
            if label.cluster_id.is_some() {
                in_clusters += 1;
            }
        }

        Self {
            total_labels: labels.len(),
            by_category,
            by_company,
            with_provenance,
            in_scenarios,
            in_clusters,
        }
    }

    /// Writes the summary to a JSON file.
    pub fn write_to_file(&self, path: &Path) -> LabelExportResult<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::{AnomalyCausalReason, AnomalyType, FraudType};
    use tempfile::TempDir;

    fn create_test_labels() -> Vec<LabeledAnomaly> {
        vec![
            LabeledAnomaly::new(
                "ANO001".to_string(),
                AnomalyType::Fraud(FraudType::SelfApproval),
                "JE001".to_string(),
                "JE".to_string(),
                "1000".to_string(),
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            )
            .with_run_id("run-123")
            .with_generation_seed(42)
            .with_causal_reason(AnomalyCausalReason::RandomRate { base_rate: 0.02 }),
            LabeledAnomaly::new(
                "ANO002".to_string(),
                AnomalyType::Fraud(FraudType::DuplicatePayment),
                "JE002".to_string(),
                "JE".to_string(),
                "1000".to_string(),
                NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            )
            .with_cluster("cluster-1"),
        ]
    }

    #[test]
    fn test_export_csv() {
        let temp_dir = TempDir::new().unwrap();
        let labels = create_test_labels();
        let config = LabelExportConfig::default();

        let path = temp_dir.path().join("labels.csv");
        let count = export_labels_csv(&labels, &path, &config).unwrap();

        assert_eq!(count, 2);
        assert!(path.exists());

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("ANO001"));
        assert!(contents.contains("SelfApproval"));
    }

    #[test]
    fn test_export_json() {
        let temp_dir = TempDir::new().unwrap();
        let labels = create_test_labels();
        let config = LabelExportConfig::default();

        let path = temp_dir.path().join("labels.json");
        let count = export_labels_json(&labels, &path, &config).unwrap();

        assert_eq!(count, 2);
        assert!(path.exists());

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("ANO001"));
        assert!(contents.contains("run-123"));
    }

    #[test]
    fn test_export_jsonl() {
        let temp_dir = TempDir::new().unwrap();
        let labels = create_test_labels();
        let config = LabelExportConfig::default();

        let path = temp_dir.path().join("labels.jsonl");
        let count = export_labels_jsonl(&labels, &path, &config).unwrap();

        assert_eq!(count, 2);
        assert!(path.exists());

        let contents = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_export_all_formats() {
        let temp_dir = TempDir::new().unwrap();
        let labels = create_test_labels();
        let config = LabelExportConfig::default();

        let results =
            export_labels_all_formats(&labels, temp_dir.path(), "anomaly_labels", &config).unwrap();

        assert_eq!(results.len(), 3);
        assert!(temp_dir.path().join("anomaly_labels.csv").exists());
        assert!(temp_dir.path().join("anomaly_labels.json").exists());
        assert!(temp_dir.path().join("anomaly_labels.jsonl").exists());
    }

    #[test]
    fn test_label_export_summary() {
        let labels = create_test_labels();
        let summary = LabelExportSummary::from_labels(&labels);

        assert_eq!(summary.total_labels, 2);
        assert_eq!(summary.by_category.get("Fraud"), Some(&2));
        assert_eq!(summary.with_provenance, 1);
        assert_eq!(summary.in_clusters, 1);
    }

    #[test]
    fn test_flat_label_conversion() {
        let label = LabeledAnomaly::new(
            "ANO001".to_string(),
            AnomalyType::Fraud(FraudType::SelfApproval),
            "JE001".to_string(),
            "JE".to_string(),
            "1000".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        )
        .with_run_id("run-123")
        .with_causal_reason(AnomalyCausalReason::RandomRate { base_rate: 0.02 });

        let flat: FlatAnomalyLabel = (&label).into();

        assert_eq!(flat.anomaly_id, "ANO001");
        assert_eq!(flat.anomaly_category, "Fraud");
        assert_eq!(flat.run_id, Some("run-123".to_string()));
        assert_eq!(flat.causal_reason_type, Some("RandomRate".to_string()));
    }
}
