//! Run manifest and metadata tracking for reproducibility.
//!
//! This module provides structures for capturing complete generation run metadata,
//! enabling reproducibility and traceability of generated data.

use chrono::{DateTime, Utc};
use datasynth_config::schema::GeneratorConfig;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

use super::EnhancedGenerationStatistics;

/// Complete manifest of a generation run for reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    /// Unique identifier for this run.
    pub run_id: String,
    /// Timestamp when generation started.
    pub started_at: DateTime<Utc>,
    /// Timestamp when generation completed.
    pub completed_at: Option<DateTime<Utc>>,
    /// SHA-256 hash of the configuration (for quick comparison).
    pub config_hash: String,
    /// Complete configuration snapshot.
    pub config_snapshot: GeneratorConfig,
    /// Seed used for random number generation.
    pub seed: u64,
    /// Scenario tags for categorization.
    #[serde(default)]
    pub scenario_tags: Vec<String>,
    /// Generation statistics.
    #[serde(default)]
    pub statistics: Option<EnhancedGenerationStatistics>,
    /// Duration in seconds.
    pub duration_seconds: Option<f64>,
    /// Version of the generator.
    pub generator_version: String,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Output directory path.
    pub output_directory: Option<String>,
    /// List of output files generated.
    #[serde(default)]
    pub output_files: Vec<OutputFileInfo>,
    /// Any warnings or notes from the generation.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Information about an output file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFileInfo {
    /// Relative path from output directory.
    pub path: String,
    /// File format (csv, json, parquet).
    pub format: String,
    /// Record count.
    pub record_count: Option<usize>,
    /// File size in bytes.
    pub size_bytes: Option<u64>,
}

impl RunManifest {
    /// Creates a new run manifest.
    pub fn new(config: &GeneratorConfig, seed: u64) -> Self {
        let run_id = Uuid::new_v4().to_string();
        let config_hash = Self::hash_config(config);

        Self {
            run_id,
            started_at: Utc::now(),
            completed_at: None,
            config_hash,
            config_snapshot: config.clone(),
            seed,
            scenario_tags: Vec::new(),
            statistics: None,
            duration_seconds: None,
            generator_version: env!("CARGO_PKG_VERSION").to_string(),
            metadata: HashMap::new(),
            output_directory: None,
            output_files: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Computes SHA-256 hash of the configuration.
    fn hash_config(config: &GeneratorConfig) -> String {
        let json = serde_json::to_string(config).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Marks the run as complete.
    pub fn complete(&mut self, statistics: EnhancedGenerationStatistics) {
        self.completed_at = Some(Utc::now());
        self.duration_seconds =
            Some((self.completed_at.unwrap() - self.started_at).num_milliseconds() as f64 / 1000.0);
        self.statistics = Some(statistics);
    }

    /// Adds a scenario tag.
    pub fn add_tag(&mut self, tag: &str) {
        if !self.scenario_tags.contains(&tag.to_string()) {
            self.scenario_tags.push(tag.to_string());
        }
    }

    /// Adds multiple scenario tags.
    pub fn add_tags(&mut self, tags: &[String]) {
        for tag in tags {
            self.add_tag(tag);
        }
    }

    /// Sets the output directory.
    pub fn set_output_directory(&mut self, path: &Path) {
        self.output_directory = Some(path.display().to_string());
    }

    /// Adds an output file record.
    pub fn add_output_file(&mut self, info: OutputFileInfo) {
        self.output_files.push(info);
    }

    /// Adds a warning message.
    pub fn add_warning(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }

    /// Adds metadata.
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// Writes the manifest to a JSON file.
    pub fn write_to_file(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Returns the run ID.
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
}

// Note: ScenarioConfig is now defined in datasynth-config/src/schema.rs
// and exported via datasynth_config::schema::ScenarioConfig

#[cfg(test)]
mod tests {
    use super::*;
    use datasynth_config::schema::*;

    fn create_test_config() -> GeneratorConfig {
        GeneratorConfig {
            global: GlobalConfig {
                industry: datasynth_core::models::IndustrySector::Manufacturing,
                start_date: "2024-01-01".to_string(),
                period_months: 1,
                seed: Some(42),
                parallel: false,
                group_currency: "USD".to_string(),
                worker_threads: 1,
                memory_limit_mb: 512,
            },
            companies: vec![CompanyConfig {
                code: "TEST".to_string(),
                name: "Test Company".to_string(),
                currency: "USD".to_string(),
                country: "US".to_string(),
                annual_transaction_volume: TransactionVolume::TenK,
                volume_weight: 1.0,
                fiscal_year_variant: "K4".to_string(),
            }],
            chart_of_accounts: ChartOfAccountsConfig::default(),
            transactions: TransactionConfig::default(),
            output: OutputConfig::default(),
            fraud: FraudConfig::default(),
            internal_controls: InternalControlsConfig::default(),
            business_processes: BusinessProcessConfig::default(),
            user_personas: UserPersonaConfig::default(),
            templates: TemplateConfig::default(),
            approval: ApprovalConfig::default(),
            departments: DepartmentConfig::default(),
            master_data: MasterDataConfig::default(),
            document_flows: DocumentFlowConfig::default(),
            intercompany: IntercompanyConfig::default(),
            balance: BalanceConfig::default(),
            ocpm: OcpmConfig::default(),
            audit: AuditGenerationConfig::default(),
            banking: datasynth_banking::BankingConfig::default(),
            data_quality: DataQualitySchemaConfig::default(),
            scenario: ScenarioConfig::default(),
            temporal: TemporalDriftConfig::default(),
            graph_export: GraphExportConfig::default(),
            streaming: StreamingSchemaConfig::default(),
            rate_limit: RateLimitSchemaConfig::default(),
            temporal_attributes: TemporalAttributeSchemaConfig::default(),
            relationships: RelationshipSchemaConfig::default(),
            accounting_standards: AccountingStandardsConfig::default(),
            audit_standards: AuditStandardsConfig::default(),
            distributions: Default::default(),
            temporal_patterns: Default::default(),
            vendor_network: VendorNetworkSchemaConfig::default(),
            customer_segmentation: CustomerSegmentationSchemaConfig::default(),
            relationship_strength: RelationshipStrengthSchemaConfig::default(),
            cross_process_links: CrossProcessLinksSchemaConfig::default(),
            organizational_events: OrganizationalEventsSchemaConfig::default(),
            behavioral_drift: BehavioralDriftSchemaConfig::default(),
            market_drift: MarketDriftSchemaConfig::default(),
            drift_labeling: DriftLabelingSchemaConfig::default(),
        }
    }

    #[test]
    fn test_run_manifest_creation() {
        let config = create_test_config();
        let manifest = RunManifest::new(&config, 42);

        assert!(!manifest.run_id.is_empty());
        assert_eq!(manifest.seed, 42);
        assert!(!manifest.config_hash.is_empty());
        assert!(manifest.completed_at.is_none());
    }

    #[test]
    fn test_run_manifest_completion() {
        let config = create_test_config();
        let mut manifest = RunManifest::new(&config, 42);

        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));

        let stats = EnhancedGenerationStatistics {
            total_entries: 100,
            total_line_items: 500,
            ..Default::default()
        };
        manifest.complete(stats);

        assert!(manifest.completed_at.is_some());
        assert!(manifest.duration_seconds.unwrap() >= 0.01);
        assert_eq!(manifest.statistics.as_ref().unwrap().total_entries, 100);
    }

    #[test]
    fn test_config_hash_consistency() {
        let config = create_test_config();
        let hash1 = RunManifest::hash_config(&config);
        let hash2 = RunManifest::hash_config(&config);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_scenario_tags() {
        let config = create_test_config();
        let mut manifest = RunManifest::new(&config, 42);

        manifest.add_tag("fraud_detection");
        manifest.add_tag("retail");
        manifest.add_tag("fraud_detection"); // Duplicate

        assert_eq!(manifest.scenario_tags.len(), 2);
        assert!(manifest
            .scenario_tags
            .contains(&"fraud_detection".to_string()));
        assert!(manifest.scenario_tags.contains(&"retail".to_string()));
    }

    #[test]
    fn test_output_file_tracking() {
        let config = create_test_config();
        let mut manifest = RunManifest::new(&config, 42);

        manifest.add_output_file(OutputFileInfo {
            path: "journal_entries.csv".to_string(),
            format: "csv".to_string(),
            record_count: Some(1000),
            size_bytes: Some(102400),
        });

        assert_eq!(manifest.output_files.len(), 1);
        assert_eq!(manifest.output_files[0].record_count, Some(1000));
    }
}
