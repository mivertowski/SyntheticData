//! EU AI Act Article 10 — Data Governance Report.
//!
//! Generates documentation about data sources, processing steps,
//! quality measures, and bias assessment for synthetic data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data governance report per EU AI Act Article 10.
///
/// Documents the provenance, processing, and quality of generated data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGovernanceReport {
    /// Report version.
    pub report_version: String,
    /// Generation timestamp.
    pub generated_at: DateTime<Utc>,
    /// Generator name and version.
    pub generator: String,
    /// Data sources used for generation.
    pub data_sources: Vec<DataSourceEntry>,
    /// Processing steps applied during generation.
    pub processing_steps: Vec<ProcessingStep>,
    /// Quality measures applied and their results.
    pub quality_measures: Vec<QualityMeasure>,
    /// Bias assessment summary.
    pub bias_assessment: BiasAssessment,
    /// Configuration hash for traceability.
    pub config_hash: String,
    /// Seed used for reproducibility.
    pub seed: u64,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl DataGovernanceReport {
    /// Create a new data governance report.
    pub fn new(config_hash: String, seed: u64) -> Self {
        Self {
            report_version: "1.0".to_string(),
            generated_at: Utc::now(),
            generator: format!("DataSynth v{}", env!("CARGO_PKG_VERSION")),
            data_sources: vec![DataSourceEntry {
                name: "Synthetic generation (no real data used)".to_string(),
                description: "All data is algorithmically generated using statistical distributions, domain models, and configurable parameters. No real personal or corporate data is used as input.".to_string(),
                source_type: "synthetic".to_string(),
                contains_personal_data: false,
            }],
            processing_steps: Vec::new(),
            quality_measures: Vec::new(),
            bias_assessment: BiasAssessment::default(),
            config_hash,
            seed,
            metadata: HashMap::new(),
        }
    }

    /// Add processing steps based on the phases that were executed.
    pub fn add_standard_processing_steps(&mut self) {
        self.processing_steps = vec![
            ProcessingStep {
                name: "Chart of Accounts Generation".to_string(),
                description: "Generate GL account structure based on industry and complexity"
                    .to_string(),
                order: 1,
            },
            ProcessingStep {
                name: "Master Data Generation".to_string(),
                description: "Generate vendors, customers, materials, fixed assets, employees"
                    .to_string(),
                order: 2,
            },
            ProcessingStep {
                name: "Document Flow Generation".to_string(),
                description: "Generate P2P and O2C document chains with three-way matching"
                    .to_string(),
                order: 3,
            },
            ProcessingStep {
                name: "Journal Entry Generation".to_string(),
                description: "Generate balanced journal entries following Benford's Law"
                    .to_string(),
                order: 4,
            },
            ProcessingStep {
                name: "Anomaly Injection".to_string(),
                description:
                    "Inject configurable fraud and error patterns with ground truth labels"
                        .to_string(),
                order: 5,
            },
            ProcessingStep {
                name: "Quality Validation".to_string(),
                description:
                    "Validate balance coherence, referential integrity, and statistical properties"
                        .to_string(),
                order: 6,
            },
        ];
    }

    /// Add standard quality measures.
    pub fn add_standard_quality_measures(&mut self) {
        self.quality_measures = vec![
            QualityMeasure {
                name: "Benford's Law Compliance".to_string(),
                description: "First-digit distribution follows Benford's Law (MAD < 0.015)"
                    .to_string(),
                result: "Applied".to_string(),
            },
            QualityMeasure {
                name: "Balance Coherence".to_string(),
                description: "All journal entries are balanced (debits = credits)".to_string(),
                result: "Enforced at construction".to_string(),
            },
            QualityMeasure {
                name: "Deterministic Reproducibility".to_string(),
                description: "Same config + seed produces identical output".to_string(),
                result: "ChaCha8 RNG with configurable seed".to_string(),
            },
            QualityMeasure {
                name: "Referential Integrity".to_string(),
                description: "All foreign key references are valid".to_string(),
                result: "Applied".to_string(),
            },
        ];
    }
}

/// A data source entry in the governance report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceEntry {
    /// Source name.
    pub name: String,
    /// Source description.
    pub description: String,
    /// Type of source (synthetic, real, derived).
    pub source_type: String,
    /// Whether the source contains personal data.
    pub contains_personal_data: bool,
}

/// A processing step in the generation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStep {
    /// Step name.
    pub name: String,
    /// Step description.
    pub description: String,
    /// Order in the pipeline.
    pub order: u32,
}

/// A quality measure applied during generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMeasure {
    /// Measure name.
    pub name: String,
    /// Measure description.
    pub description: String,
    /// Result or status.
    pub result: String,
}

/// Bias assessment for the generated data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasAssessment {
    /// Overall assessment.
    pub assessment: String,
    /// Known limitations.
    pub known_limitations: Vec<String>,
    /// Mitigation measures.
    pub mitigation_measures: Vec<String>,
}

impl Default for BiasAssessment {
    fn default() -> Self {
        Self {
            assessment: "Synthetic data generation uses configurable statistical distributions. \
                         Bias characteristics are determined by configuration parameters (industry profiles, \
                         distribution parameters, anomaly rates) rather than real-world data."
                .to_string(),
            known_limitations: vec![
                "Generated data reflects configured distribution parameters, not real-world distributions".to_string(),
                "Industry profiles are approximations based on published research".to_string(),
                "Temporal patterns use simplified models of business cycles".to_string(),
            ],
            mitigation_measures: vec![
                "All configuration parameters are documented and reproducible".to_string(),
                "Evaluation framework validates statistical properties of output".to_string(),
                "AutoTuner can adjust parameters based on evaluation feedback".to_string(),
                "Users should validate generated data against their specific use case requirements".to_string(),
            ],
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_report_creation() {
        let report = DataGovernanceReport::new("hash123".to_string(), 42);
        assert_eq!(report.report_version, "1.0");
        assert_eq!(report.seed, 42);
        assert_eq!(report.config_hash, "hash123");
        assert!(!report.data_sources.is_empty());
        assert!(!report.data_sources[0].contains_personal_data);
    }

    #[test]
    fn test_standard_processing_steps() {
        let mut report = DataGovernanceReport::new("hash".to_string(), 42);
        report.add_standard_processing_steps();
        assert!(report.processing_steps.len() >= 5);
        assert_eq!(report.processing_steps[0].order, 1);
    }

    #[test]
    fn test_standard_quality_measures() {
        let mut report = DataGovernanceReport::new("hash".to_string(), 42);
        report.add_standard_quality_measures();
        assert!(report.quality_measures.len() >= 3);
    }

    #[test]
    fn test_bias_assessment_default() {
        let assessment = BiasAssessment::default();
        assert!(!assessment.assessment.is_empty());
        assert!(!assessment.known_limitations.is_empty());
        assert!(!assessment.mitigation_measures.is_empty());
    }

    #[test]
    fn test_report_serialization() {
        let mut report = DataGovernanceReport::new("hash".to_string(), 42);
        report.add_standard_processing_steps();
        report.add_standard_quality_measures();
        let json = serde_json::to_string_pretty(&report).expect("should serialize");
        assert!(json.contains("DataSynth"));
        assert!(json.contains("Article 10") || json.contains("data_sources"));
        // Verify it round-trips
        let deser: DataGovernanceReport = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deser.seed, 42);
    }
}
