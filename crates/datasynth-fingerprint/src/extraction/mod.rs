//! Extraction engine for fingerprinting.
//!
//! This module provides extractors that analyze data and produce
//! fingerprint components while applying privacy mechanisms.
//!
//! # Overview
//!
//! The extraction process analyzes real data and produces a [`Fingerprint`]
//! that captures statistical properties without storing individual records.
//!
//! # Basic Usage
//!
//! ```ignore
//! use datasynth_fingerprint::extraction::FingerprintExtractor;
//! use datasynth_fingerprint::models::PrivacyLevel;
//! use std::path::Path;
//!
//! // Create extractor with standard privacy
//! let extractor = FingerprintExtractor::new(PrivacyLevel::Standard);
//!
//! // Extract from CSV file
//! let fingerprint = extractor.extract_from_csv(Path::new("data.csv"))?;
//! ```
//!
//! # Data Sources
//!
//! Multiple data source types are supported:
//!
//! ```ignore
//! use datasynth_fingerprint::extraction::{DataSource, CsvDataSource, ParquetDataSource, JsonDataSource};
//!
//! // CSV files
//! let csv_source = DataSource::Csv(CsvDataSource::new("data.csv"));
//!
//! // Parquet files
//! let parquet_source = DataSource::Parquet(ParquetDataSource::new("data.parquet"));
//!
//! // JSON files (array or newline-delimited)
//! let json_source = DataSource::Json(JsonDataSource::json_array("data.json"));
//! let jsonl_source = DataSource::Json(JsonDataSource::jsonl("data.jsonl"));
//!
//! // Multi-table from directory
//! let fingerprint = extractor.extract_from_directory(Path::new("./data_folder/"))?;
//! ```
//!
//! # Streaming Extraction
//!
//! For large files that don't fit in memory, use streaming extraction:
//!
//! ```ignore
//! use datasynth_fingerprint::extraction::{FingerprintExtractor, ExtractionConfig};
//!
//! let config = ExtractionConfig {
//!     streaming: true,
//!     stream_batch_size: 100_000,  // Process 100k rows at a time
//!     ..ExtractionConfig::default()
//! };
//!
//! let extractor = FingerprintExtractor::with_config(config);
//! let fingerprint = extractor.extract_streaming_csv(Path::new("large_file.csv"))?;
//! ```
//!
//! # Component Extractors
//!
//! Individual extractors handle different fingerprint components:
//!
//! | Extractor | Output | Description |
//! |-----------|--------|-------------|
//! | [`SchemaExtractor`] | [`SchemaFingerprint`] | Column types, constraints |
//! | [`StatsExtractor`] | [`StatisticsFingerprint`] | Distributions, percentiles |
//! | [`CorrelationExtractor`] | [`CorrelationFingerprint`] | Correlation matrices |
//! | [`IntegrityExtractor`] | [`IntegrityFingerprint`] | FK relationships |
//! | [`RulesExtractor`] | [`RulesFingerprint`] | Business rules |
//! | [`AnomalyExtractor`] | [`AnomalyFingerprint`] | Anomaly patterns |
//!
//! # Streaming Statistics
//!
//! The [`StreamingNumericStats`] and [`StreamingCategoricalStats`] types
//! provide memory-efficient online algorithms for computing statistics:
//!
//! ```ignore
//! use datasynth_fingerprint::extraction::StreamingNumericStats;
//!
//! let mut stats = StreamingNumericStats::new();
//! for value in data_iterator {
//!     stats.update(value);
//! }
//! let final_stats = stats.finalize();
//! ```
//!
//! [`Fingerprint`]: crate::models::Fingerprint
//! [`SchemaFingerprint`]: crate::models::SchemaFingerprint
//! [`StatisticsFingerprint`]: crate::models::StatisticsFingerprint
//! [`CorrelationFingerprint`]: crate::models::CorrelationFingerprint
//! [`IntegrityFingerprint`]: crate::models::IntegrityFingerprint
//! [`RulesFingerprint`]: crate::models::RulesFingerprint
//! [`AnomalyFingerprint`]: crate::models::AnomalyFingerprint

mod anomaly_extractor;
mod correlation_extractor;
mod integrity_extractor;
mod rules_extractor;
mod schema_extractor;
mod stats_extractor;
pub mod streaming;

pub use anomaly_extractor::*;
pub use correlation_extractor::*;
pub use integrity_extractor::*;
pub use rules_extractor::*;
pub use schema_extractor::*;
pub use stats_extractor::*;
pub use streaming::{StreamingCategoricalStats, StreamingNumericStats};

use std::path::Path;

use crate::error::{FingerprintError, FingerprintResult};
use crate::models::{
    Fingerprint, Manifest, PrivacyLevel, PrivacyMetadata, SchemaFingerprint, SourceMetadata,
    StatisticsFingerprint,
};
use crate::privacy::{PrivacyConfig, PrivacyEngine};

/// Configuration for fingerprint extraction.
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Privacy configuration.
    pub privacy: PrivacyConfig,
    /// Whether to extract correlations.
    pub extract_correlations: bool,
    /// Whether to extract integrity constraints.
    pub extract_integrity: bool,
    /// Whether to extract business rules.
    pub extract_rules: bool,
    /// Whether to extract anomaly patterns.
    pub extract_anomalies: bool,
    /// Maximum sample size for large datasets.
    pub max_sample_size: Option<usize>,
    /// Minimum rows required for extraction.
    pub min_rows: usize,
    /// Enable streaming extraction for large files.
    ///
    /// When enabled, uses online algorithms for statistics computation
    /// to reduce memory usage. Set `stream_batch_size` to control memory.
    pub streaming: bool,
    /// Batch size for streaming extraction (number of rows per batch).
    ///
    /// Smaller values reduce memory but may increase computation time.
    /// Default is 10,000 rows.
    pub stream_batch_size: usize,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            privacy: PrivacyConfig::from_level(PrivacyLevel::Standard),
            extract_correlations: true,
            extract_integrity: true,
            extract_rules: true,
            extract_anomalies: true,
            max_sample_size: None,
            min_rows: 10,
            streaming: false,
            stream_batch_size: 10_000,
        }
    }
}

impl ExtractionConfig {
    /// Create with a specific privacy level.
    pub fn with_privacy_level(level: PrivacyLevel) -> Self {
        Self {
            privacy: PrivacyConfig::from_level(level),
            ..Default::default()
        }
    }

    /// Enable streaming mode for large datasets.
    ///
    /// Streaming mode uses online algorithms to compute statistics
    /// without loading all data into memory.
    pub fn with_streaming(mut self, batch_size: usize) -> Self {
        self.streaming = true;
        self.stream_batch_size = batch_size;
        self
    }
}

/// Trait for data extractors.
pub trait Extractor: Send + Sync {
    /// Name of this extractor.
    fn name(&self) -> &'static str;

    /// Extract component from data.
    fn extract(
        &self,
        data: &DataSource,
        config: &ExtractionConfig,
        privacy: &mut PrivacyEngine,
    ) -> FingerprintResult<ExtractedComponent>;
}

/// Source of data for extraction.
#[derive(Debug)]
pub enum DataSource {
    /// CSV file.
    Csv(CsvDataSource),
    /// Parquet file.
    Parquet(ParquetDataSource),
    /// JSON/JSONL file.
    Json(JsonDataSource),
    /// Directory with multiple files.
    Directory(DirectoryDataSource),
    /// In-memory data.
    Memory(MemoryDataSource),
}

/// CSV data source.
#[derive(Debug)]
pub struct CsvDataSource {
    /// Path to the CSV file.
    pub path: std::path::PathBuf,
    /// Whether the CSV has headers.
    pub has_headers: bool,
    /// Delimiter character.
    pub delimiter: u8,
}

impl CsvDataSource {
    /// Create from a path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            has_headers: true,
            delimiter: b',',
        }
    }
}

/// Parquet data source.
#[derive(Debug)]
pub struct ParquetDataSource {
    /// Path to the Parquet file.
    pub path: std::path::PathBuf,
    /// Row group indices to read (None = all).
    pub row_groups: Option<Vec<usize>>,
    /// Columns to read (None = all).
    pub columns: Option<Vec<String>>,
}

impl ParquetDataSource {
    /// Create from a path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            row_groups: None,
            columns: None,
        }
    }

    /// Specify row groups to read.
    pub fn with_row_groups(mut self, groups: Vec<usize>) -> Self {
        self.row_groups = Some(groups);
        self
    }

    /// Specify columns to read.
    pub fn with_columns(mut self, columns: Vec<String>) -> Self {
        self.columns = Some(columns);
        self
    }
}

/// JSON/JSONL data source.
#[derive(Debug)]
pub struct JsonDataSource {
    /// Path to the JSON or JSONL file.
    pub path: std::path::PathBuf,
    /// Format: true for JSON array, false for JSONL (one object per line).
    pub is_array: bool,
}

impl JsonDataSource {
    /// Create from a path, auto-detecting format from extension.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let is_array = path
            .extension()
            .map(|ext| ext != "jsonl" && ext != "ndjson")
            .unwrap_or(true);
        Self { path, is_array }
    }

    /// Create a JSON array source.
    pub fn json_array(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            is_array: true,
        }
    }

    /// Create a JSONL (newline-delimited) source.
    pub fn jsonl(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            is_array: false,
        }
    }
}

/// Directory data source for multi-table extraction.
#[derive(Debug)]
pub struct DirectoryDataSource {
    /// Path to the directory.
    pub path: std::path::PathBuf,
    /// File extensions to include (empty = all supported).
    pub extensions: Vec<String>,
    /// Whether to recurse into subdirectories.
    pub recursive: bool,
}

impl DirectoryDataSource {
    /// Create from a directory path.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            extensions: vec![
                "csv".to_string(),
                "parquet".to_string(),
                "json".to_string(),
                "jsonl".to_string(),
            ],
            recursive: false,
        }
    }

    /// Set file extensions to include.
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Enable recursive directory traversal.
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    /// Get all matching files in the directory.
    pub fn files(&self) -> std::io::Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();
        self.collect_files(&self.path, &mut files)?;
        Ok(files)
    }

    fn collect_files(
        &self,
        dir: &Path,
        files: &mut Vec<std::path::PathBuf>,
    ) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if self.recursive {
                    self.collect_files(&path, files)?;
                }
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if self.extensions.is_empty()
                    || self
                        .extensions
                        .iter()
                        .any(|e| e.to_lowercase() == ext_lower)
                {
                    files.push(path);
                }
            }
        }
        Ok(())
    }
}

/// In-memory data source.
#[derive(Debug)]
pub struct MemoryDataSource {
    /// Column names.
    pub columns: Vec<String>,
    /// Row data (each inner Vec is a row).
    pub rows: Vec<Vec<String>>,
}

impl MemoryDataSource {
    /// Create from columns and rows.
    pub fn new(columns: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self { columns, rows }
    }

    /// Get row count.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get column count.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}

/// Result of extraction from a single extractor.
#[derive(Debug)]
pub enum ExtractedComponent {
    Schema(SchemaFingerprint),
    Statistics(StatisticsFingerprint),
    Correlations(crate::models::CorrelationFingerprint),
    Integrity(crate::models::IntegrityFingerprint),
    Rules(crate::models::RulesFingerprint),
    Anomalies(crate::models::AnomalyFingerprint),
}

/// Main fingerprint extractor that coordinates all extraction.
pub struct FingerprintExtractor {
    config: ExtractionConfig,
}

impl FingerprintExtractor {
    /// Create a new extractor with default configuration.
    pub fn new() -> Self {
        Self {
            config: ExtractionConfig::default(),
        }
    }

    /// Create with a specific privacy level.
    pub fn with_privacy_level(level: PrivacyLevel) -> Self {
        Self {
            config: ExtractionConfig::with_privacy_level(level),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: ExtractionConfig) -> Self {
        Self { config }
    }

    /// Extract fingerprint from a CSV file.
    pub fn extract_from_csv(&self, path: impl AsRef<Path>) -> FingerprintResult<Fingerprint> {
        let source = DataSource::Csv(CsvDataSource::new(path));
        self.extract(&source)
    }

    /// Extract fingerprint from a large CSV file using streaming.
    ///
    /// This method processes the CSV in batches to reduce memory usage,
    /// using online algorithms for statistics computation.
    ///
    /// # Arguments
    /// * `path` - Path to the CSV file
    ///
    /// # Example
    /// ```no_run
    /// use datasynth_fingerprint::extraction::FingerprintExtractor;
    ///
    /// let extractor = FingerprintExtractor::new();
    /// let fingerprint = extractor.extract_streaming_csv("large_data.csv").unwrap();
    /// ```
    pub fn extract_streaming_csv(&self, path: impl AsRef<Path>) -> FingerprintResult<Fingerprint> {
        use std::collections::HashMap;
        use streaming::{StreamingCategoricalStats, StreamingNumericStats};

        let path = path.as_ref();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();

        // Initialize streaming accumulators for each column
        let mut numeric_accumulators: HashMap<usize, StreamingNumericStats> = HashMap::new();
        let mut categorical_accumulators: HashMap<usize, StreamingCategoricalStats> =
            HashMap::new();
        let mut column_is_numeric: HashMap<usize, bool> = HashMap::new();
        let mut row_count: u64 = 0;

        // Process rows in streaming fashion
        for result in reader.records() {
            let record = result?;
            row_count += 1;

            for (i, field) in record.iter().enumerate() {
                if i >= headers.len() {
                    continue;
                }

                // Determine if column is numeric (on first batch)
                let is_numeric = column_is_numeric
                    .entry(i)
                    .or_insert_with(|| field.parse::<f64>().is_ok() || field.is_empty());

                if *is_numeric {
                    if let Ok(value) = field.parse::<f64>() {
                        let acc = numeric_accumulators
                            .entry(i)
                            .or_insert_with(|| StreamingNumericStats::new(10000));
                        acc.add(value);
                    }
                } else {
                    let acc = categorical_accumulators
                        .entry(i)
                        .or_insert_with(|| StreamingCategoricalStats::new(1000));
                    acc.add(field.to_string());
                }
            }

            // Optional: limit rows if max_sample_size is set
            if let Some(max) = self.config.max_sample_size {
                if row_count >= max as u64 {
                    break;
                }
            }
        }

        // Check minimum rows
        if row_count < self.config.min_rows as u64 {
            return Err(FingerprintError::InsufficientData {
                required: self.config.min_rows,
                actual: row_count as usize,
            });
        }

        // Build schema
        let mut schema = SchemaFingerprint::new();
        let table_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("data");

        let mut table = crate::models::TableSchema::new(table_name, row_count);
        for (i, header) in headers.iter().enumerate() {
            let is_numeric = column_is_numeric.get(&i).copied().unwrap_or(false);
            let data_type = if is_numeric {
                crate::models::DataType::Float64
            } else {
                crate::models::DataType::String
            };
            let field = crate::models::FieldSchema::new(header.clone(), data_type);
            table.add_column(field);
        }
        schema.add_table(table_name.to_string(), table);

        // Build statistics from accumulators
        let mut statistics = StatisticsFingerprint::new();

        for (i, acc) in numeric_accumulators {
            let header = &headers[i];
            let numeric_stats = crate::models::NumericStats {
                count: acc.count(),
                min: acc.min(),
                max: acc.max(),
                mean: acc.mean(),
                std_dev: acc.std_dev(),
                percentiles: acc.percentiles(),
                distribution: crate::models::DistributionType::Unknown,
                distribution_params: crate::models::DistributionParams::empty(),
                zero_rate: acc.zero_rate(),
                negative_rate: acc.negative_rate(),
                benford_first_digit: Some(acc.benford_distribution()),
            };
            statistics.add_numeric(table_name, header, numeric_stats);
        }

        for (i, acc) in categorical_accumulators {
            let header = &headers[i];
            let top_values: Vec<crate::models::CategoryFrequency> = acc
                .top_values(100)
                .into_iter()
                .map(|(value, count)| {
                    let frequency = count as f64 / acc.count() as f64;
                    crate::models::CategoryFrequency::new(value, frequency)
                })
                .collect();

            let categorical_stats = crate::models::CategoricalStats {
                count: acc.count(),
                cardinality: acc.cardinality(),
                top_values,
                rare_values_suppressed: true,
                suppressed_count: 0,
                entropy: acc.entropy(),
            };
            statistics.add_categorical(table_name, header, categorical_stats);
        }

        // Build manifest
        let source_meta = SourceMetadata::new(
            format!("CSV file: {} (streaming extraction)", path.display()),
            vec![table_name.to_string()],
            row_count,
        );
        let privacy_meta = PrivacyMetadata::from_level(self.config.privacy.level);
        let manifest = Manifest::new(source_meta, privacy_meta);

        // Build fingerprint (minimal privacy audit for streaming mode)
        let privacy_audit = crate::models::PrivacyAudit::new(
            self.config.privacy.epsilon,
            self.config.privacy.k_anonymity,
        );

        let fingerprint = Fingerprint::new(manifest, schema, statistics, privacy_audit);

        Ok(fingerprint)
    }

    /// Extract fingerprint from in-memory data.
    pub fn extract_from_memory(
        &self,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    ) -> FingerprintResult<Fingerprint> {
        let source = DataSource::Memory(MemoryDataSource::new(columns, rows));
        self.extract(&source)
    }

    /// Extract fingerprint from a directory.
    pub fn extract_from_directory(&self, path: impl AsRef<Path>) -> FingerprintResult<Fingerprint> {
        let source = DataSource::Directory(DirectoryDataSource::new(path));
        self.extract(&source)
    }

    /// Extract fingerprint from a data source.
    pub fn extract(&self, source: &DataSource) -> FingerprintResult<Fingerprint> {
        // Handle directory sources specially by extracting from each file and merging
        if let DataSource::Directory(dir) = source {
            return self.extract_from_directory_impl(dir);
        }

        let mut privacy = PrivacyEngine::new(self.config.privacy.clone());

        // Extract schema
        let schema_extractor = SchemaExtractor;
        let schema = match schema_extractor.extract(source, &self.config, &mut privacy)? {
            ExtractedComponent::Schema(s) => s,
            _ => {
                return Err(FingerprintError::extraction(
                    "schema",
                    "Unexpected component type",
                ))
            }
        };

        // Extract statistics
        let stats_extractor = StatsExtractor;
        let statistics = match stats_extractor.extract(source, &self.config, &mut privacy)? {
            ExtractedComponent::Statistics(s) => s,
            _ => {
                return Err(FingerprintError::extraction(
                    "statistics",
                    "Unexpected component type",
                ))
            }
        };

        // Extract optional components
        let correlations = if self.config.extract_correlations {
            let extractor = CorrelationExtractor;
            match extractor.extract(source, &self.config, &mut privacy) {
                Ok(ExtractedComponent::Correlations(c)) => Some(c),
                Ok(_) => None,
                Err(_) => None, // Optional, ignore errors
            }
        } else {
            None
        };

        let integrity = if self.config.extract_integrity {
            let extractor = IntegrityExtractor;
            match extractor.extract(source, &self.config, &mut privacy) {
                Ok(ExtractedComponent::Integrity(i)) => Some(i),
                Ok(_) => None,
                Err(_) => None,
            }
        } else {
            None
        };

        let rules = if self.config.extract_rules {
            let extractor = RulesExtractor;
            match extractor.extract(source, &self.config, &mut privacy) {
                Ok(ExtractedComponent::Rules(r)) => Some(r),
                Ok(_) => None,
                Err(_) => None,
            }
        } else {
            None
        };

        let anomalies = if self.config.extract_anomalies {
            let extractor = AnomalyExtractor;
            match extractor.extract(source, &self.config, &mut privacy) {
                Ok(ExtractedComponent::Anomalies(a)) => Some(a),
                Ok(_) => None,
                Err(_) => None,
            }
        } else {
            None
        };

        // Build manifest with composition metadata from the engine
        let source_meta = build_source_metadata(source, &schema);
        let privacy_meta = privacy.build_privacy_metadata();
        let manifest = Manifest::new(source_meta, privacy_meta);

        // Get privacy audit (includes composition method and RDP alpha)
        let privacy_audit = privacy.into_audit();

        // Build fingerprint
        let mut fingerprint = Fingerprint::new(manifest, schema, statistics, privacy_audit);

        if let Some(c) = correlations {
            fingerprint = fingerprint.with_correlations(c);
        }
        if let Some(i) = integrity {
            fingerprint = fingerprint.with_integrity(i);
        }
        if let Some(r) = rules {
            fingerprint = fingerprint.with_rules(r);
        }
        if let Some(a) = anomalies {
            fingerprint = fingerprint.with_anomalies(a);
        }

        Ok(fingerprint)
    }

    /// Extract fingerprint from a directory by processing each file.
    fn extract_from_directory_impl(
        &self,
        dir: &DirectoryDataSource,
    ) -> FingerprintResult<Fingerprint> {
        let files = dir.files()?;

        if files.is_empty() {
            return Err(FingerprintError::InvalidFormat(format!(
                "No supported files found in directory: {}",
                dir.path.display()
            )));
        }

        // Extract from each file and merge
        let mut merged_schema = SchemaFingerprint::new();
        let mut merged_stats = StatisticsFingerprint::new();
        let mut total_rows: u64 = 0;
        let mut table_names: Vec<String> = Vec::new();

        // Track total epsilon spent across all files
        let mut total_epsilon_spent = 0.0;
        let mut all_actions = Vec::new();

        // Divide epsilon budget among files to ensure each file gets some budget
        let num_files = files.len();
        let per_file_epsilon = self.config.privacy.epsilon / num_files as f64;

        for file_path in &files {
            // Determine file type
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            let source = match ext.as_str() {
                "csv" => DataSource::Csv(CsvDataSource::new(file_path)),
                "parquet" => DataSource::Parquet(ParquetDataSource::new(file_path)),
                "json" => DataSource::Json(JsonDataSource::json_array(file_path)),
                "jsonl" | "ndjson" => DataSource::Json(JsonDataSource::jsonl(file_path)),
                _ => continue, // Skip unknown file types
            };

            // Create a fresh privacy engine for each file with proportional budget
            let mut per_file_config = self.config.privacy.clone();
            per_file_config.epsilon = per_file_epsilon;
            let mut file_privacy = PrivacyEngine::new(per_file_config);

            // Extract schema
            let schema_extractor = SchemaExtractor;
            if let Ok(ExtractedComponent::Schema(schema)) =
                schema_extractor.extract(&source, &self.config, &mut file_privacy)
            {
                for (name, table) in schema.tables {
                    total_rows += table.row_count;
                    table_names.push(name.clone());
                    merged_schema.add_table(name, table);
                }
            }

            // Extract statistics
            let stats_extractor = StatsExtractor;
            if let Ok(ExtractedComponent::Statistics(stats)) =
                stats_extractor.extract(&source, &self.config, &mut file_privacy)
            {
                // Merge statistics
                for (key, numeric) in stats.numeric_columns {
                    merged_stats.numeric_columns.insert(key, numeric);
                }
                for (key, categorical) in stats.categorical_columns {
                    merged_stats.categorical_columns.insert(key, categorical);
                }
            }

            // Collect privacy audit from this file
            let file_audit = file_privacy.into_audit();
            total_epsilon_spent += file_audit.total_epsilon_spent;
            all_actions.extend(file_audit.actions);
        }

        // Build source metadata
        let description = format!("Directory: {} ({} files)", dir.path.display(), files.len());
        let source_meta = SourceMetadata::new(description, table_names, total_rows);
        let privacy_meta = PrivacyMetadata::from_level(self.config.privacy.level);
        let manifest = Manifest::new(source_meta, privacy_meta);

        // Build combined privacy audit
        let mut privacy_audit = crate::models::PrivacyAudit::new(
            self.config.privacy.epsilon,
            self.config.privacy.k_anonymity,
        );
        privacy_audit.total_epsilon_spent = total_epsilon_spent;
        privacy_audit.actions = all_actions;

        // Build fingerprint
        let fingerprint = Fingerprint::new(manifest, merged_schema, merged_stats, privacy_audit);

        Ok(fingerprint)
    }
}

impl Default for FingerprintExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Build source metadata from data source and schema.
fn build_source_metadata(source: &DataSource, schema: &SchemaFingerprint) -> SourceMetadata {
    let (description, tables, total_rows) = match source {
        DataSource::Csv(csv) => {
            let name = csv
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let rows = schema.tables.values().map(|t| t.row_count).sum();
            (format!("CSV file: {}", name), vec![name], rows)
        }
        DataSource::Parquet(pq) => {
            let name = pq
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let rows = schema.tables.values().map(|t| t.row_count).sum();
            (format!("Parquet file: {}", name), vec![name], rows)
        }
        DataSource::Json(json) => {
            let name = json
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let rows = schema.tables.values().map(|t| t.row_count).sum();
            let format_type = if json.is_array { "JSON" } else { "JSONL" };
            (format!("{} file: {}", format_type, name), vec![name], rows)
        }
        DataSource::Memory(mem) => {
            let rows = mem.row_count() as u64;
            (
                "In-memory data".to_string(),
                vec!["memory".to_string()],
                rows,
            )
        }
        DataSource::Directory(dir) => {
            // This shouldn't be called for directories - they're handled separately
            let name = dir.path.display().to_string();
            (format!("Directory: {}", name), vec![], 0)
        }
    };

    SourceMetadata::new(description, tables, total_rows)
}
