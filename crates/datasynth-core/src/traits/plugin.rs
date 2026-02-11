//! Plugin trait definitions for extensible generation and output.
//!
//! Provides stable trait interfaces for custom generators, output sinks,
//! and transform plugins. Plugins are in-process Rust trait objects.

use crate::error::SynthError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context provided to generator plugins during data generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationContext {
    /// RNG seed for reproducibility.
    pub seed: u64,
    /// Fiscal year being generated.
    pub fiscal_year: u32,
    /// Company code being generated for.
    pub company_code: String,
    /// Industry sector.
    pub industry: String,
    /// Additional context key-value pairs.
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

impl GenerationContext {
    /// Create a new generation context.
    pub fn new(seed: u64, fiscal_year: u32, company_code: impl Into<String>) -> Self {
        Self {
            seed,
            fiscal_year,
            company_code: company_code.into(),
            industry: String::new(),
            extra: HashMap::new(),
        }
    }

    /// Set the industry.
    pub fn with_industry(mut self, industry: impl Into<String>) -> Self {
        self.industry = industry.into();
        self
    }

    /// Add an extra context value.
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// A single generated record from a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedRecord {
    /// Record type identifier (e.g., "journal_entry", "vendor", "custom_report").
    pub record_type: String,
    /// Record fields as key-value pairs.
    pub fields: HashMap<String, serde_json::Value>,
}

impl GeneratedRecord {
    /// Create a new generated record.
    pub fn new(record_type: impl Into<String>) -> Self {
        Self {
            record_type: record_type.into(),
            fields: HashMap::new(),
        }
    }

    /// Add a field to the record.
    pub fn with_field(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Get a field value.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.fields.get(key)
    }

    /// Get a field as a string.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.fields.get(key).and_then(|v| v.as_str())
    }
}

/// Summary returned by a sink plugin after finalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkSummary {
    /// Total records written.
    pub records_written: usize,
    /// Total bytes written (if tracked).
    pub bytes_written: Option<u64>,
    /// Paths of files written (if applicable).
    pub file_paths: Vec<String>,
    /// Additional summary metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl SinkSummary {
    /// Create a new sink summary.
    pub fn new(records_written: usize) -> Self {
        Self {
            records_written,
            bytes_written: None,
            file_paths: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Information about a registered plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin description.
    pub description: String,
    /// Plugin type (generator, sink, transform).
    pub plugin_type: PluginType,
}

/// Type of plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    /// Data generator plugin.
    Generator,
    /// Output sink plugin.
    Sink,
    /// Data transform plugin.
    Transform,
}

/// Trait for custom data generator plugins.
///
/// Generator plugins produce records based on configuration and context.
///
/// # Example
///
/// ```rust
/// use datasynth_core::traits::plugin::*;
/// use datasynth_core::error::SynthError;
///
/// struct MyGenerator;
///
/// impl GeneratorPlugin for MyGenerator {
///     fn name(&self) -> &str { "my_generator" }
///     fn version(&self) -> &str { "1.0.0" }
///     fn description(&self) -> &str { "Generates custom records" }
///     fn config_schema(&self) -> Option<serde_json::Value> { None }
///     fn generate(
///         &self,
///         _config: &serde_json::Value,
///         _context: &GenerationContext,
///     ) -> Result<Vec<GeneratedRecord>, SynthError> {
///         Ok(vec![GeneratedRecord::new("custom").with_field("key", "value")])
///     }
/// }
/// ```
pub trait GeneratorPlugin: Send + Sync {
    /// Unique name identifying this generator.
    fn name(&self) -> &str;
    /// Semantic version of this plugin.
    fn version(&self) -> &str;
    /// Human-readable description.
    fn description(&self) -> &str;
    /// Optional JSON Schema for plugin configuration.
    fn config_schema(&self) -> Option<serde_json::Value>;
    /// Generate records given configuration and context.
    fn generate(
        &self,
        config: &serde_json::Value,
        context: &GenerationContext,
    ) -> Result<Vec<GeneratedRecord>, SynthError>;
}

/// Trait for custom output sink plugins.
///
/// Sink plugins write generated records to external destinations.
///
/// # Lifecycle
///
/// 1. `initialize()` — set up the sink (open files, connections)
/// 2. `write_records()` — write batches of records (called multiple times)
/// 3. `finalize()` — flush and close the sink, return summary
pub trait SinkPlugin: Send + Sync {
    /// Unique name identifying this sink.
    fn name(&self) -> &str;
    /// Initialize the sink with configuration.
    fn initialize(&mut self, config: &serde_json::Value) -> Result<(), SynthError>;
    /// Write a batch of records. Returns number of records written.
    fn write_records(&mut self, records: &[GeneratedRecord]) -> Result<usize, SynthError>;
    /// Finalize the sink and return a summary.
    fn finalize(&mut self) -> Result<SinkSummary, SynthError>;
}

/// Trait for data transform plugins.
///
/// Transform plugins modify or enrich records in-place.
pub trait TransformPlugin: Send + Sync {
    /// Unique name identifying this transform.
    fn name(&self) -> &str;
    /// Transform a batch of records.
    fn transform(&self, records: Vec<GeneratedRecord>) -> Result<Vec<GeneratedRecord>, SynthError>;
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_context_creation() {
        let ctx = GenerationContext::new(42, 2024, "C001")
            .with_industry("manufacturing")
            .with_extra("region", "EU");
        assert_eq!(ctx.seed, 42);
        assert_eq!(ctx.fiscal_year, 2024);
        assert_eq!(ctx.company_code, "C001");
        assert_eq!(ctx.industry, "manufacturing");
        assert_eq!(ctx.extra.get("region").map(|s| s.as_str()), Some("EU"));
    }

    #[test]
    fn test_generated_record_creation() {
        let record = GeneratedRecord::new("test_record")
            .with_field("name", serde_json::Value::String("Test".to_string()))
            .with_field("amount", serde_json::json!(100.0));
        assert_eq!(record.record_type, "test_record");
        assert_eq!(record.get_str("name"), Some("Test"));
        assert!(record.get("amount").is_some());
    }

    #[test]
    fn test_generated_record_serialization() {
        let record = GeneratedRecord::new("vendor")
            .with_field("id", serde_json::json!("V001"))
            .with_field("name", serde_json::json!("Acme Corp"));
        let json = serde_json::to_string(&record).expect("should serialize");
        let deser: GeneratedRecord = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deser.record_type, "vendor");
        assert_eq!(deser.get_str("id"), Some("V001"));
    }

    #[test]
    fn test_sink_summary_creation() {
        let summary = SinkSummary::new(100);
        assert_eq!(summary.records_written, 100);
        assert!(summary.bytes_written.is_none());
        assert!(summary.file_paths.is_empty());
    }

    #[test]
    fn test_plugin_info_serialization() {
        let info = PluginInfo {
            name: "test_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            plugin_type: PluginType::Generator,
        };
        let json = serde_json::to_string(&info).expect("should serialize");
        assert!(json.contains("test_plugin"));
        assert!(json.contains("generator"));
    }
}
