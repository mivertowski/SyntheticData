//! Fingerprint manifest containing metadata.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Current fingerprint format version.
pub const FINGERPRINT_VERSION: &str = "1.0.0";

/// The file format identifier.
pub const FINGERPRINT_FORMAT: &str = "dsf";

/// Manifest containing fingerprint metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Fingerprint format version (semver).
    pub version: String,

    /// Format identifier ("dsf").
    pub format: String,

    /// Timestamp when fingerprint was created.
    pub created_at: DateTime<Utc>,

    /// Information about the source data.
    pub source: SourceMetadata,

    /// Privacy configuration used during extraction.
    pub privacy: PrivacyMetadata,

    /// SHA-256 checksums for each component file.
    pub checksums: HashMap<String, String>,

    /// Optional digital signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<SignatureMetadata>,
}

impl Manifest {
    /// Create a new manifest.
    pub fn new(source: SourceMetadata, privacy: PrivacyMetadata) -> Self {
        Self {
            version: FINGERPRINT_VERSION.to_string(),
            format: FINGERPRINT_FORMAT.to_string(),
            created_at: Utc::now(),
            source,
            privacy,
            checksums: HashMap::new(),
            signature: None,
        }
    }

    /// Add a checksum for a component file.
    pub fn add_checksum(&mut self, file: impl Into<String>, checksum: impl Into<String>) {
        self.checksums.insert(file.into(), checksum.into());
    }

    /// Verify that all required checksums are present.
    pub fn verify_checksums(&self) -> bool {
        // Required components
        let required = ["schema.yaml", "statistics.yaml"];
        required.iter().all(|f| self.checksums.contains_key(*f))
    }
}

/// Metadata about the source data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetadata {
    /// Description of the data source.
    pub description: String,

    /// Total number of tables/files processed.
    pub table_count: usize,

    /// Total number of rows across all tables.
    pub total_rows: u64,

    /// List of table names.
    pub tables: Vec<String>,

    /// Date range of the data (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_range: Option<DateRange>,

    /// Industry/domain of the source data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,

    /// Additional metadata key-value pairs.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl SourceMetadata {
    /// Create new source metadata.
    pub fn new(description: impl Into<String>, tables: Vec<String>, total_rows: u64) -> Self {
        Self {
            description: description.into(),
            table_count: tables.len(),
            total_rows,
            tables,
            date_range: None,
            industry: None,
            metadata: HashMap::new(),
        }
    }
}

/// Date range of source data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date (ISO 8601).
    pub start: String,
    /// End date (ISO 8601).
    pub end: String,
}

/// Privacy configuration metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyMetadata {
    /// Privacy level preset used.
    pub level: PrivacyLevel,

    /// Differential privacy epsilon (total budget).
    pub epsilon: f64,

    /// K-anonymity threshold.
    pub k_anonymity: u32,

    /// Outlier percentile for winsorization.
    pub outlier_percentile: f64,

    /// Fields that were always suppressed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressed_fields: Vec<String>,

    /// Minimum occurrence threshold for categorical values.
    pub min_occurrence: u32,

    /// Delta parameter for approximate DP (used with RDP and zCDP composition).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,

    /// The composition method used for budget accounting (e.g., "naive", "renyi_dp", "zcdp").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_method: Option<String>,
}

impl PrivacyMetadata {
    /// Create privacy metadata from a privacy level.
    pub fn from_level(level: PrivacyLevel) -> Self {
        let (epsilon, k, outlier_percentile, min_occurrence) = match level {
            PrivacyLevel::Minimal => (5.0, 3, 99.0, 3),
            PrivacyLevel::Standard | PrivacyLevel::Custom => (1.0, 5, 95.0, 5),
            PrivacyLevel::High => (0.5, 10, 90.0, 10),
            PrivacyLevel::Maximum => (0.1, 20, 85.0, 20),
        };

        Self {
            level,
            epsilon,
            k_anonymity: k,
            outlier_percentile,
            suppressed_fields: Vec::new(),
            min_occurrence,
            delta: None,
            composition_method: None,
        }
    }

    /// Create custom privacy metadata.
    pub fn custom(epsilon: f64, k_anonymity: u32) -> Self {
        Self {
            level: PrivacyLevel::Standard,
            epsilon,
            k_anonymity,
            outlier_percentile: 95.0,
            suppressed_fields: Vec::new(),
            min_occurrence: k_anonymity,
            delta: None,
            composition_method: None,
        }
    }
}

/// Predefined privacy levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    /// Minimal privacy protection (epsilon=5.0, k=3).
    /// Use for low-sensitivity data where utility is priority.
    Minimal,

    /// Standard privacy protection (epsilon=1.0, k=5).
    /// Balanced approach suitable for most use cases.
    #[default]
    Standard,

    /// High privacy protection (epsilon=0.5, k=10).
    /// Use for sensitive data requiring stronger guarantees.
    High,

    /// Maximum privacy protection (epsilon=0.1, k=20).
    /// Use for highly sensitive data where privacy is paramount.
    Maximum,

    /// Custom privacy parameters specified by the user.
    /// Use when predefined levels don't fit your requirements.
    Custom,
}

impl PrivacyLevel {
    /// Get the epsilon value for this privacy level.
    /// For `Custom`, returns 1.0 as a placeholder (actual value comes from config).
    pub fn epsilon(&self) -> f64 {
        match self {
            Self::Minimal => 5.0,
            Self::Standard => 1.0,
            Self::High => 0.5,
            Self::Maximum => 0.1,
            Self::Custom => 1.0,
        }
    }

    /// Get the k-anonymity threshold for this privacy level.
    /// For `Custom`, returns 5 as a placeholder (actual value comes from config).
    pub fn k_anonymity(&self) -> u32 {
        match self {
            Self::Minimal => 3,
            Self::Standard => 5,
            Self::High => 10,
            Self::Maximum => 20,
            Self::Custom => 5,
        }
    }
}

/// Digital signature metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMetadata {
    /// Signature algorithm used.
    pub algorithm: String,

    /// Key identifier.
    pub key_id: String,

    /// Base64-encoded signature.
    pub signature: String,

    /// Timestamp of signing.
    pub signed_at: DateTime<Utc>,
}
