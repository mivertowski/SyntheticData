//! Validation utilities for fingerprint files.

use std::collections::HashSet;
use std::io::{Read, Seek};
use std::path::Path;

use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::error::FingerprintResult;
use crate::models::{Fingerprint, Manifest, FINGERPRINT_VERSION};

use super::file_names;

/// Convenience function to validate a DSF file.
pub fn validate_dsf(path: &Path) -> FingerprintResult<DsfValidationReport> {
    let result = FingerprintValidator::validate_file(path)?;

    let components: Vec<String> = if let Some(ref summary) = result.summary {
        let mut c = vec![
            "schema".to_string(),
            "statistics".to_string(),
            "privacy_audit".to_string(),
        ];
        if summary.has_correlations {
            c.push("correlations".to_string());
        }
        if summary.has_integrity {
            c.push("integrity".to_string());
        }
        if summary.has_rules {
            c.push("rules".to_string());
        }
        if summary.has_anomalies {
            c.push("anomalies".to_string());
        }
        c
    } else {
        Vec::new()
    };

    Ok(DsfValidationReport {
        is_valid: result.is_valid,
        version: result.version.unwrap_or_else(|| "unknown".to_string()),
        components,
        errors: result.errors.iter().map(|e| e.message.clone()).collect(),
        warnings: result.warnings.iter().map(|w| w.message.clone()).collect(),
    })
}

/// Simple validation report for CLI use.
#[derive(Debug, Clone)]
pub struct DsfValidationReport {
    /// Whether validation passed.
    pub is_valid: bool,
    /// Fingerprint version.
    pub version: String,
    /// Components present.
    pub components: Vec<String>,
    /// Validation errors.
    pub errors: Vec<String>,
    /// Validation warnings.
    pub warnings: Vec<String>,
}

/// Result of fingerprint validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed.
    pub is_valid: bool,
    /// List of validation errors.
    pub errors: Vec<ValidationError>,
    /// List of validation warnings.
    pub warnings: Vec<ValidationWarning>,
    /// Fingerprint version.
    pub version: Option<String>,
    /// Summary of contents.
    pub summary: Option<ContentSummary>,
}

impl ValidationResult {
    /// Create a successful validation result.
    pub fn success(version: String, summary: ContentSummary) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            version: Some(version),
            summary: Some(summary),
        }
    }

    /// Create a failed validation result.
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
            version: None,
            summary: None,
        }
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

/// A validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error code.
    pub code: String,
    /// Error message.
    pub message: String,
    /// Affected component.
    pub component: Option<String>,
}

impl ValidationError {
    /// Create a new validation error.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            component: None,
        }
    }

    /// Add component information.
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }
}

/// A validation warning.
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning code.
    pub code: String,
    /// Warning message.
    pub message: String,
}

impl ValidationWarning {
    /// Create a new validation warning.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Summary of fingerprint contents.
#[derive(Debug, Clone)]
pub struct ContentSummary {
    /// Number of tables.
    pub table_count: usize,
    /// Number of columns.
    pub column_count: usize,
    /// Whether correlations are present.
    pub has_correlations: bool,
    /// Whether integrity constraints are present.
    pub has_integrity: bool,
    /// Whether rules are present.
    pub has_rules: bool,
    /// Whether anomaly patterns are present.
    pub has_anomalies: bool,
    /// Epsilon spent on privacy.
    pub epsilon_spent: f64,
    /// Files present in archive.
    pub files_present: Vec<String>,
}

/// Validator for fingerprint files.
pub struct FingerprintValidator;

impl FingerprintValidator {
    /// Validate a fingerprint file.
    pub fn validate_file(path: &Path) -> FingerprintResult<ValidationResult> {
        let file = std::fs::File::open(path)?;
        Self::validate(file)
    }

    /// Validate a fingerprint from any reader.
    pub fn validate<R: Read + Seek>(reader: R) -> FingerprintResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Try to open as ZIP
        let mut archive = match ZipArchive::new(reader) {
            Ok(a) => a,
            Err(e) => {
                return Ok(ValidationResult::failure(vec![ValidationError::new(
                    "INVALID_FORMAT",
                    format!("Not a valid ZIP archive: {e}"),
                )]));
            }
        };

        // List all files
        let files_present: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
            .collect();

        // Check for required files
        let required_files = [
            file_names::MANIFEST,
            file_names::SCHEMA,
            file_names::STATISTICS,
            file_names::PRIVACY_AUDIT,
        ];

        for required in required_files {
            if !files_present.contains(&required.to_string()) {
                errors.push(
                    ValidationError::new(
                        "MISSING_REQUIRED_FILE",
                        format!("Required file '{required}' is missing"),
                    )
                    .with_component(required),
                );
            }
        }

        if !errors.is_empty() {
            return Ok(ValidationResult::failure(errors));
        }

        // Read and validate manifest
        let manifest: Manifest = {
            let mut file = archive.by_name(file_names::MANIFEST)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            match serde_json::from_str(&contents) {
                Ok(m) => m,
                Err(e) => {
                    return Ok(ValidationResult::failure(vec![ValidationError::new(
                        "INVALID_MANIFEST",
                        format!("Failed to parse manifest: {e}"),
                    )
                    .with_component(file_names::MANIFEST)]));
                }
            }
        };

        // Check version
        if manifest.version != FINGERPRINT_VERSION {
            warnings.push(ValidationWarning::new(
                "VERSION_MISMATCH",
                format!(
                    "File version {} differs from current version {}",
                    manifest.version, FINGERPRINT_VERSION
                ),
            ));
        }

        // Validate checksums
        for (file_name, expected_checksum) in &manifest.checksums {
            match archive.by_name(file_name) {
                Ok(mut file) => {
                    let mut contents = Vec::new();
                    file.read_to_end(&mut contents)?;
                    let actual = compute_checksum(&contents);
                    if &actual != expected_checksum {
                        errors.push(
                            ValidationError::new(
                                "CHECKSUM_MISMATCH",
                                format!(
                                    "Checksum mismatch for '{file_name}': expected {expected_checksum}, got {actual}"
                                ),
                            )
                            .with_component(file_name),
                        );
                    }
                }
                Err(_) => {
                    errors.push(
                        ValidationError::new(
                            "MISSING_CHECKSUMMED_FILE",
                            format!("File '{file_name}' listed in checksums is missing"),
                        )
                        .with_component(file_name),
                    );
                }
            }
        }

        if !errors.is_empty() {
            let mut result = ValidationResult::failure(errors);
            result.warnings = warnings;
            result.version = Some(manifest.version);
            return Ok(result);
        }

        // Parse all components to check validity
        let schema: crate::models::SchemaFingerprint = {
            let mut file = archive.by_name(file_names::SCHEMA)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            match serde_yaml::from_str(&contents) {
                Ok(s) => s,
                Err(e) => {
                    return Ok(ValidationResult::failure(vec![ValidationError::new(
                        "INVALID_SCHEMA",
                        format!("Failed to parse schema: {e}"),
                    )
                    .with_component(file_names::SCHEMA)]));
                }
            }
        };

        let privacy_audit: crate::models::PrivacyAudit = {
            let mut file = archive.by_name(file_names::PRIVACY_AUDIT)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            match serde_json::from_str(&contents) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(ValidationResult::failure(vec![ValidationError::new(
                        "INVALID_PRIVACY_AUDIT",
                        format!("Failed to parse privacy audit: {e}"),
                    )
                    .with_component(file_names::PRIVACY_AUDIT)]));
                }
            }
        };

        // Check for optional components
        let has_correlations = files_present.contains(&file_names::CORRELATIONS.to_string());
        let has_integrity = files_present.contains(&file_names::INTEGRITY.to_string());
        let has_rules = files_present.contains(&file_names::RULES.to_string());
        let has_anomalies = files_present.contains(&file_names::ANOMALIES.to_string());

        let summary = ContentSummary {
            table_count: schema.tables.len(),
            column_count: schema.total_columns(),
            has_correlations,
            has_integrity,
            has_rules,
            has_anomalies,
            epsilon_spent: privacy_audit.total_epsilon_spent,
            files_present,
        };

        let mut result = ValidationResult::success(manifest.version, summary);
        result.warnings = warnings;
        Ok(result)
    }
}

/// Compute SHA-256 checksum of data.
fn compute_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compare two fingerprints and return differences.
pub fn diff_fingerprints(a: &Fingerprint, b: &Fingerprint) -> FingerprintDiff {
    let mut diff = FingerprintDiff::default();

    // Compare tables
    let tables_a: HashSet<_> = a.schema.tables.keys().collect();
    let tables_b: HashSet<_> = b.schema.tables.keys().collect();

    diff.tables_added = tables_b
        .difference(&tables_a)
        .map(|s| (*s).clone())
        .collect();
    diff.tables_removed = tables_a
        .difference(&tables_b)
        .map(|s| (*s).clone())
        .collect();

    // Compare privacy settings
    if (a.privacy_audit.epsilon_budget - b.privacy_audit.epsilon_budget).abs() > 0.001 {
        diff.privacy_changes.push(format!(
            "Epsilon budget changed: {} -> {}",
            a.privacy_audit.epsilon_budget, b.privacy_audit.epsilon_budget
        ));
    }

    if a.privacy_audit.k_anonymity != b.privacy_audit.k_anonymity {
        diff.privacy_changes.push(format!(
            "K-anonymity changed: {} -> {}",
            a.privacy_audit.k_anonymity, b.privacy_audit.k_anonymity
        ));
    }

    // Compare optional components
    if a.correlations.is_some() != b.correlations.is_some() {
        if b.correlations.is_some() {
            diff.components_added.push("correlations".to_string());
        } else {
            diff.components_removed.push("correlations".to_string());
        }
    }

    if a.integrity.is_some() != b.integrity.is_some() {
        if b.integrity.is_some() {
            diff.components_added.push("integrity".to_string());
        } else {
            diff.components_removed.push("integrity".to_string());
        }
    }

    if a.rules.is_some() != b.rules.is_some() {
        if b.rules.is_some() {
            diff.components_added.push("rules".to_string());
        } else {
            diff.components_removed.push("rules".to_string());
        }
    }

    if a.anomalies.is_some() != b.anomalies.is_some() {
        if b.anomalies.is_some() {
            diff.components_added.push("anomalies".to_string());
        } else {
            diff.components_removed.push("anomalies".to_string());
        }
    }

    diff
}

/// Differences between two fingerprints.
#[derive(Debug, Clone, Default)]
pub struct FingerprintDiff {
    /// Tables added in the second fingerprint.
    pub tables_added: Vec<String>,
    /// Tables removed in the second fingerprint.
    pub tables_removed: Vec<String>,
    /// Components added.
    pub components_added: Vec<String>,
    /// Components removed.
    pub components_removed: Vec<String>,
    /// Privacy changes.
    pub privacy_changes: Vec<String>,
    /// Statistical changes summary.
    pub statistical_changes: Vec<String>,
}

impl FingerprintDiff {
    /// Check if there are any differences.
    pub fn has_changes(&self) -> bool {
        !self.tables_added.is_empty()
            || !self.tables_removed.is_empty()
            || !self.components_added.is_empty()
            || !self.components_removed.is_empty()
            || !self.privacy_changes.is_empty()
            || !self.statistical_changes.is_empty()
    }
}
