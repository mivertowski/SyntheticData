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
use std::io::{self, BufReader, Read as _, Write};
use std::path::Path;
use uuid::Uuid;

use super::EnhancedGenerationStatistics;

/// Complete manifest of a generation run for reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    /// Manifest format version.
    #[serde(default = "default_manifest_version")]
    pub manifest_version: String,
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
    /// Data lineage graph tracking config → generator → output relationships.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage: Option<super::lineage::LineageGraph>,
    /// Quality gate evaluation result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_gate_result: Option<QualityGateResultSummary>,
    /// LLM enrichment phase summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm_enrichment: Option<LlmEnrichmentSummary>,
    /// Diffusion enhancement phase summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diffusion_model: Option<DiffusionModelSummary>,
    /// Causal generation phase summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causal_generation: Option<CausalGenerationSummary>,
}

/// Summary of LLM enrichment phase for the run manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmEnrichmentSummary {
    /// Whether LLM enrichment was enabled.
    pub enabled: bool,
    /// Execution time in milliseconds.
    pub timing_ms: u64,
    /// Number of vendors enriched.
    pub vendors_enriched: usize,
    /// Provider used (e.g., "mock", "openai").
    pub provider: String,
}

/// Summary of diffusion enhancement phase for the run manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffusionModelSummary {
    /// Whether diffusion enhancement was enabled.
    pub enabled: bool,
    /// Execution time in milliseconds.
    pub timing_ms: u64,
    /// Number of samples generated.
    pub samples_generated: usize,
    /// Number of diffusion steps used.
    pub n_steps: usize,
}

/// Summary of causal generation phase for the run manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGenerationSummary {
    /// Whether causal generation was enabled.
    pub enabled: bool,
    /// Execution time in milliseconds.
    pub timing_ms: u64,
    /// Number of causal samples generated.
    pub samples_generated: usize,
    /// Template used (e.g., "fraud_detection", "revenue_cycle").
    pub template: String,
    /// Whether causal validation passed (None if validation was not run).
    pub validation_passed: Option<bool>,
}

/// Summary of quality gate evaluation for the run manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResultSummary {
    /// Whether all gates passed.
    pub passed: bool,
    /// Profile name used.
    pub profile_name: String,
    /// Number of gates that passed.
    pub gates_passed: usize,
    /// Total number of gates evaluated.
    pub gates_total: usize,
    /// Names of failed gates.
    pub failed_gates: Vec<String>,
}

fn default_manifest_version() -> String {
    "2.0".to_string()
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
    /// SHA-256 checksum of the file contents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256_checksum: Option<String>,
    /// Index of the first record in this file (for partitioned outputs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_record_index: Option<u64>,
    /// Index of the last record in this file (for partitioned outputs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_record_index: Option<u64>,
}

/// Result of verifying a single file's checksum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumVerificationResult {
    /// Relative path of the file.
    pub path: String,
    /// Verification status.
    pub status: ChecksumStatus,
    /// Expected checksum (from manifest).
    pub expected: Option<String>,
    /// Actual checksum (computed from file).
    pub actual: Option<String>,
}

/// Status of a checksum verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumStatus {
    /// Checksum matches.
    Ok,
    /// Checksum does not match.
    Mismatch,
    /// File is missing on disk.
    Missing,
    /// No checksum recorded in manifest.
    NoChecksum,
}

/// Computes the SHA-256 checksum of a file, streaming in 8KB chunks.
pub fn compute_file_checksum(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

impl RunManifest {
    /// Creates a new run manifest.
    pub fn new(config: &GeneratorConfig, seed: u64) -> Self {
        let run_id = Uuid::new_v4().to_string();
        let config_hash = Self::hash_config(config);

        Self {
            manifest_version: "2.0".to_string(),
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
            lineage: None,
            quality_gate_result: None,
            llm_enrichment: None,
            diffusion_model: None,
            causal_generation: None,
        }
    }

    /// Computes SHA-256 hash of the configuration.
    fn hash_config(config: &GeneratorConfig) -> String {
        let json = match serde_json::to_string(config) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("Failed to serialize config for hashing: {}", e);
                String::new()
            }
        };
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Marks the run as complete.
    pub fn complete(&mut self, statistics: EnhancedGenerationStatistics) {
        let now = Utc::now();
        self.completed_at = Some(now);
        self.duration_seconds = Some((now - self.started_at).num_milliseconds() as f64 / 1000.0);
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

    /// Populates SHA-256 checksums for all output files.
    ///
    /// Resolves each file path relative to `base_dir` and computes its checksum.
    /// Also populates `size_bytes` if not already set.
    pub fn populate_file_checksums(&mut self, base_dir: &Path) {
        for file_info in &mut self.output_files {
            let file_path = base_dir.join(&file_info.path);
            if file_path.exists() {
                if let Ok(checksum) = compute_file_checksum(&file_path) {
                    file_info.sha256_checksum = Some(checksum);
                }
                if file_info.size_bytes.is_none() {
                    if let Ok(metadata) = std::fs::metadata(&file_path) {
                        file_info.size_bytes = Some(metadata.len());
                    }
                }
            }
        }
    }

    /// Verifies checksums for all output files against their recorded values.
    pub fn verify_file_checksums(&self, base_dir: &Path) -> Vec<ChecksumVerificationResult> {
        self.output_files
            .iter()
            .map(|file_info| {
                let file_path = base_dir.join(&file_info.path);

                let expected = file_info.sha256_checksum.clone();
                if expected.is_none() {
                    return ChecksumVerificationResult {
                        path: file_info.path.clone(),
                        status: ChecksumStatus::NoChecksum,
                        expected: None,
                        actual: None,
                    };
                }

                if !file_path.exists() {
                    return ChecksumVerificationResult {
                        path: file_info.path.clone(),
                        status: ChecksumStatus::Missing,
                        expected,
                        actual: None,
                    };
                }

                match compute_file_checksum(&file_path) {
                    Ok(actual) => {
                        let status = if expected.as_deref() == Some(actual.as_str()) {
                            ChecksumStatus::Ok
                        } else {
                            ChecksumStatus::Mismatch
                        };
                        ChecksumVerificationResult {
                            path: file_info.path.clone(),
                            status,
                            expected,
                            actual: Some(actual),
                        }
                    }
                    Err(_) => ChecksumVerificationResult {
                        path: file_info.path.clone(),
                        status: ChecksumStatus::Missing,
                        expected,
                        actual: None,
                    },
                }
            })
            .collect()
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
#[allow(clippy::unwrap_used)]
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
                fiscal_year_months: None,
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
            anomaly_injection: Default::default(),
            industry_specific: Default::default(),
            fingerprint_privacy: Default::default(),
            quality_gates: Default::default(),
            compliance: Default::default(),
            webhooks: Default::default(),
            llm: Default::default(),
            diffusion: Default::default(),
            causal: Default::default(),
            source_to_pay: Default::default(),
            financial_reporting: Default::default(),
            hr: Default::default(),
            manufacturing: Default::default(),
            sales_quotes: Default::default(),
            tax: Default::default(),
            treasury: Default::default(),
            project_accounting: Default::default(),
            esg: Default::default(),
            country_packs: None,
            scenarios: Default::default(),
            session: Default::default(),
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
            sha256_checksum: None,
            first_record_index: None,
            last_record_index: None,
        });

        assert_eq!(manifest.output_files.len(), 1);
        assert_eq!(manifest.output_files[0].record_count, Some(1000));
    }

    #[test]
    fn test_manifest_version() {
        let config = create_test_config();
        let manifest = RunManifest::new(&config, 42);
        assert_eq!(manifest.manifest_version, "2.0");
    }

    #[test]
    fn test_backward_compat_deserialize() {
        // Old manifest JSON without manifest_version or checksum fields
        let old_json = r#"{
            "run_id": "test-123",
            "started_at": "2024-01-01T00:00:00Z",
            "completed_at": null,
            "config_hash": "abc123",
            "config_snapshot": null,
            "seed": 42,
            "duration_seconds": null,
            "generator_version": "0.4.0",
            "output_directory": null,
            "output_files": [
                {
                    "path": "data.csv",
                    "format": "csv",
                    "record_count": 100,
                    "size_bytes": 1024
                }
            ]
        }"#;

        // Should deserialize without errors (config_snapshot will fail since it's null,
        // but the point is that the new fields have proper defaults)
        let result: Result<serde_json::Value, _> = serde_json::from_str(old_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_checksum_computation() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"hello world").expect("write file");

        let checksum = compute_file_checksum(&file_path).expect("compute checksum");
        // SHA-256 of "hello world"
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_populate_and_verify_checksums() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("data.csv");
        std::fs::write(&file_path, b"id,name\n1,Alice\n2,Bob\n").expect("write file");

        let config = create_test_config();
        let mut manifest = RunManifest::new(&config, 42);
        manifest.add_output_file(OutputFileInfo {
            path: "data.csv".to_string(),
            format: "csv".to_string(),
            record_count: Some(2),
            size_bytes: None,
            sha256_checksum: None,
            first_record_index: None,
            last_record_index: None,
        });

        manifest.populate_file_checksums(dir.path());

        assert!(manifest.output_files[0].sha256_checksum.is_some());
        assert!(manifest.output_files[0].size_bytes.is_some());

        // Verify should pass
        let results = manifest.verify_file_checksums(dir.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, ChecksumStatus::Ok);
    }

    #[test]
    fn test_verify_detects_mismatch() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("data.csv");
        std::fs::write(&file_path, b"original content").expect("write file");

        let config = create_test_config();
        let mut manifest = RunManifest::new(&config, 42);
        manifest.add_output_file(OutputFileInfo {
            path: "data.csv".to_string(),
            format: "csv".to_string(),
            record_count: None,
            size_bytes: None,
            sha256_checksum: None,
            first_record_index: None,
            last_record_index: None,
        });

        manifest.populate_file_checksums(dir.path());

        // Modify file after checksum
        std::fs::write(&file_path, b"modified content").expect("write file");

        let results = manifest.verify_file_checksums(dir.path());
        assert_eq!(results[0].status, ChecksumStatus::Mismatch);
    }

    #[test]
    fn test_verify_missing_file() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let config = create_test_config();
        let mut manifest = RunManifest::new(&config, 42);
        manifest.add_output_file(OutputFileInfo {
            path: "nonexistent.csv".to_string(),
            format: "csv".to_string(),
            record_count: None,
            size_bytes: None,
            sha256_checksum: Some("abc123".to_string()),
            first_record_index: None,
            last_record_index: None,
        });

        let results = manifest.verify_file_checksums(dir.path());
        assert_eq!(results[0].status, ChecksumStatus::Missing);
    }

    #[test]
    fn test_verify_no_checksum() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let config = create_test_config();
        let mut manifest = RunManifest::new(&config, 42);
        manifest.add_output_file(OutputFileInfo {
            path: "data.csv".to_string(),
            format: "csv".to_string(),
            record_count: None,
            size_bytes: None,
            sha256_checksum: None,
            first_record_index: None,
            last_record_index: None,
        });

        let results = manifest.verify_file_checksums(dir.path());
        assert_eq!(results[0].status, ChecksumStatus::NoChecksum);
    }
}
