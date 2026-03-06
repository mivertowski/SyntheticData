//! Quality issue labels for ML training and data quality tracking.
//!
//! This module provides labeling structures for tracking data quality issues
//! injected into synthetic data. Labels can be exported for use in training
//! ML models for data quality detection.

use serde::{Deserialize, Serialize};

use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};

/// Type of data quality issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabeledIssueType {
    /// Missing value (null, empty, or placeholder)
    MissingValue,
    /// Typo or character error
    Typo,
    /// Format variation from standard
    FormatVariation,
    /// Duplicate record (exact or near)
    Duplicate,
    /// Encoding issue (mojibake, corruption)
    EncodingIssue,
    /// Inconsistent data (e.g., mismatched formats)
    Inconsistency,
    /// Out of range value
    OutOfRange,
    /// Invalid reference (foreign key violation)
    InvalidReference,
}

impl LabeledIssueType {
    /// Get the display name for this issue type.
    pub fn display_name(&self) -> &'static str {
        match self {
            LabeledIssueType::MissingValue => "Missing Value",
            LabeledIssueType::Typo => "Typo",
            LabeledIssueType::FormatVariation => "Format Variation",
            LabeledIssueType::Duplicate => "Duplicate",
            LabeledIssueType::EncodingIssue => "Encoding Issue",
            LabeledIssueType::Inconsistency => "Inconsistency",
            LabeledIssueType::OutOfRange => "Out of Range",
            LabeledIssueType::InvalidReference => "Invalid Reference",
        }
    }

    /// Get the severity level (1-5, with 5 being most severe).
    pub fn default_severity(&self) -> u8 {
        match self {
            LabeledIssueType::MissingValue => 3,
            LabeledIssueType::Typo => 2,
            LabeledIssueType::FormatVariation => 1,
            LabeledIssueType::Duplicate => 4,
            LabeledIssueType::EncodingIssue => 3,
            LabeledIssueType::Inconsistency => 2,
            LabeledIssueType::OutOfRange => 4,
            LabeledIssueType::InvalidReference => 5,
        }
    }
}

/// Subtype providing more detail about the issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityIssueSubtype {
    // Missing value subtypes
    NullValue,
    EmptyString,
    Placeholder,
    SystematicMissing,

    // Typo subtypes
    Substitution,
    Transposition,
    Insertion,
    Deletion,
    OcrError,
    Homophone,

    // Format variation subtypes
    DateFormatVariation,
    AmountFormatVariation,
    IdentifierFormatVariation,
    CaseVariation,

    // Duplicate subtypes
    ExactDuplicate,
    NearDuplicate,
    FuzzyDuplicate,

    // Encoding subtypes
    Mojibake,
    HtmlEntityCorruption,
    BomIssue,
    CharacterCorruption,

    // Generic
    Other(String),
}

/// A label describing a data quality issue for ML training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssueLabel {
    /// Unique identifier for this issue
    pub issue_id: String,
    /// Type of quality issue
    pub issue_type: LabeledIssueType,
    /// More specific subtype
    pub subtype: Option<QualityIssueSubtype>,
    /// ID of the affected document/record
    pub document_id: String,
    /// Name of the affected field
    pub field_name: String,
    /// Original value before modification (if available)
    pub original_value: Option<String>,
    /// Modified/corrupted value (if applicable)
    pub modified_value: Option<String>,
    /// Severity level (1-5)
    pub severity: u8,
    /// Name of the processor that created this issue
    pub processor: String,
    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl QualityIssueLabel {
    /// Create a new quality issue label.
    pub fn new(
        issue_type: LabeledIssueType,
        document_id: impl Into<String>,
        field_name: impl Into<String>,
        processor: impl Into<String>,
    ) -> Self {
        let uuid_factory = DeterministicUuidFactory::new(0, GeneratorType::Anomaly);
        Self {
            issue_id: uuid_factory.next().to_string(),
            issue_type,
            subtype: None,
            document_id: document_id.into(),
            field_name: field_name.into(),
            original_value: None,
            modified_value: None,
            severity: issue_type.default_severity(),
            processor: processor.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the subtype.
    pub fn with_subtype(mut self, subtype: QualityIssueSubtype) -> Self {
        self.subtype = Some(subtype);
        self
    }

    /// Set the original value.
    pub fn with_original(mut self, value: impl Into<String>) -> Self {
        self.original_value = Some(value.into());
        self
    }

    /// Set the modified value.
    pub fn with_modified(mut self, value: impl Into<String>) -> Self {
        self.modified_value = Some(value.into());
        self
    }

    /// Set both original and modified values.
    pub fn with_values(mut self, original: impl Into<String>, modified: impl Into<String>) -> Self {
        self.original_value = Some(original.into());
        self.modified_value = Some(modified.into());
        self
    }

    /// Set the severity level.
    pub fn with_severity(mut self, severity: u8) -> Self {
        self.severity = severity.clamp(1, 5);
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Create a missing value label.
    pub fn missing_value(
        document_id: impl Into<String>,
        field_name: impl Into<String>,
        processor: impl Into<String>,
    ) -> Self {
        Self::new(
            LabeledIssueType::MissingValue,
            document_id,
            field_name,
            processor,
        )
    }

    /// Create a typo label.
    pub fn typo(
        document_id: impl Into<String>,
        field_name: impl Into<String>,
        original: impl Into<String>,
        modified: impl Into<String>,
        processor: impl Into<String>,
    ) -> Self {
        Self::new(LabeledIssueType::Typo, document_id, field_name, processor)
            .with_values(original, modified)
    }

    /// Create a format variation label.
    pub fn format_variation(
        document_id: impl Into<String>,
        field_name: impl Into<String>,
        original: impl Into<String>,
        modified: impl Into<String>,
        processor: impl Into<String>,
    ) -> Self {
        Self::new(
            LabeledIssueType::FormatVariation,
            document_id,
            field_name,
            processor,
        )
        .with_values(original, modified)
    }

    /// Create a duplicate label.
    pub fn duplicate(
        document_id: impl Into<String>,
        original_doc_id: impl Into<String>,
        processor: impl Into<String>,
    ) -> Self {
        Self::new(
            LabeledIssueType::Duplicate,
            document_id,
            "_record",
            processor,
        )
        .with_metadata("original_document_id", original_doc_id)
    }
}

/// Collection of quality issue labels with aggregation methods.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualityLabels {
    /// All labels in this collection
    pub labels: Vec<QualityIssueLabel>,
}

impl QualityLabels {
    /// Create a new empty label collection.
    pub fn new() -> Self {
        Self { labels: Vec::new() }
    }

    /// Create with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            labels: Vec::with_capacity(capacity),
        }
    }

    /// Add a label.
    pub fn add(&mut self, label: QualityIssueLabel) {
        self.labels.push(label);
    }

    /// Extend with more labels.
    pub fn extend(&mut self, labels: impl IntoIterator<Item = QualityIssueLabel>) {
        self.labels.extend(labels);
    }

    /// Get total number of labels.
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Count labels by type.
    pub fn count_by_type(&self) -> std::collections::HashMap<LabeledIssueType, usize> {
        let mut counts = std::collections::HashMap::new();
        for label in &self.labels {
            *counts.entry(label.issue_type).or_insert(0) += 1;
        }
        counts
    }

    /// Count labels by processor.
    pub fn count_by_processor(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for label in &self.labels {
            *counts.entry(label.processor.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Get labels for a specific document.
    pub fn for_document(&self, document_id: &str) -> Vec<&QualityIssueLabel> {
        self.labels
            .iter()
            .filter(|l| l.document_id == document_id)
            .collect()
    }

    /// Get labels for a specific field.
    pub fn for_field(&self, field_name: &str) -> Vec<&QualityIssueLabel> {
        self.labels
            .iter()
            .filter(|l| l.field_name == field_name)
            .collect()
    }

    /// Get labels of a specific type.
    pub fn of_type(&self, issue_type: LabeledIssueType) -> Vec<&QualityIssueLabel> {
        self.labels
            .iter()
            .filter(|l| l.issue_type == issue_type)
            .collect()
    }

    /// Get summary statistics.
    pub fn summary(&self) -> QualityLabelSummary {
        let counts = self.count_by_type();
        QualityLabelSummary {
            total_labels: self.labels.len(),
            missing_values: *counts.get(&LabeledIssueType::MissingValue).unwrap_or(&0),
            typos: *counts.get(&LabeledIssueType::Typo).unwrap_or(&0),
            format_variations: *counts.get(&LabeledIssueType::FormatVariation).unwrap_or(&0),
            duplicates: *counts.get(&LabeledIssueType::Duplicate).unwrap_or(&0),
            encoding_issues: *counts.get(&LabeledIssueType::EncodingIssue).unwrap_or(&0),
            unique_documents: self
                .labels
                .iter()
                .map(|l| &l.document_id)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            unique_fields: self
                .labels
                .iter()
                .map(|l| &l.field_name)
                .collect::<std::collections::HashSet<_>>()
                .len(),
        }
    }

    /// Convert to CSV rows.
    pub fn to_csv_rows(&self) -> Vec<Vec<String>> {
        self.labels
            .iter()
            .map(|l| {
                vec![
                    l.issue_id.clone(),
                    format!("{:?}", l.issue_type),
                    l.subtype
                        .as_ref()
                        .map(|s| format!("{s:?}"))
                        .unwrap_or_default(),
                    l.document_id.clone(),
                    l.field_name.clone(),
                    l.original_value.clone().unwrap_or_default(),
                    l.modified_value.clone().unwrap_or_default(),
                    l.severity.to_string(),
                    l.processor.clone(),
                ]
            })
            .collect()
    }

    /// Get CSV header.
    pub fn csv_header() -> Vec<&'static str> {
        vec![
            "issue_id",
            "issue_type",
            "subtype",
            "document_id",
            "field_name",
            "original_value",
            "modified_value",
            "severity",
            "processor",
        ]
    }
}

/// Summary statistics for quality labels.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualityLabelSummary {
    /// Total number of labels
    pub total_labels: usize,
    /// Number of missing value issues
    pub missing_values: usize,
    /// Number of typo issues
    pub typos: usize,
    /// Number of format variation issues
    pub format_variations: usize,
    /// Number of duplicate issues
    pub duplicates: usize,
    /// Number of encoding issues
    pub encoding_issues: usize,
    /// Number of unique documents affected
    pub unique_documents: usize,
    /// Number of unique fields affected
    pub unique_fields: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_label_creation() {
        let label = QualityIssueLabel::new(
            LabeledIssueType::Typo,
            "doc-123",
            "vendor_name",
            "typo_processor",
        )
        .with_values("Acme Corp", "Acne Corp")
        .with_subtype(QualityIssueSubtype::Substitution);

        assert_eq!(label.issue_type, LabeledIssueType::Typo);
        assert_eq!(label.document_id, "doc-123");
        assert_eq!(label.field_name, "vendor_name");
        assert_eq!(label.original_value, Some("Acme Corp".to_string()));
        assert_eq!(label.modified_value, Some("Acne Corp".to_string()));
    }

    #[test]
    fn test_label_helpers() {
        let missing = QualityIssueLabel::missing_value("doc-1", "amount", "missing_processor");
        assert_eq!(missing.issue_type, LabeledIssueType::MissingValue);

        let typo = QualityIssueLabel::typo("doc-2", "name", "John", "Jphn", "typo_processor");
        assert_eq!(typo.issue_type, LabeledIssueType::Typo);
        assert_eq!(typo.original_value, Some("John".to_string()));

        let duplicate = QualityIssueLabel::duplicate("doc-3", "doc-1", "dup_processor");
        assert_eq!(duplicate.issue_type, LabeledIssueType::Duplicate);
    }

    #[test]
    fn test_quality_labels_collection() {
        let mut labels = QualityLabels::new();
        labels.add(QualityIssueLabel::missing_value("doc-1", "field1", "proc1"));
        labels.add(QualityIssueLabel::typo(
            "doc-1", "field2", "a", "b", "proc2",
        ));
        labels.add(QualityIssueLabel::typo(
            "doc-2", "field1", "x", "y", "proc2",
        ));

        assert_eq!(labels.len(), 3);

        let counts = labels.count_by_type();
        assert_eq!(*counts.get(&LabeledIssueType::MissingValue).unwrap(), 1);
        assert_eq!(*counts.get(&LabeledIssueType::Typo).unwrap(), 2);

        let doc1_labels = labels.for_document("doc-1");
        assert_eq!(doc1_labels.len(), 2);
    }

    #[test]
    fn test_summary() {
        let mut labels = QualityLabels::new();
        labels.add(QualityIssueLabel::missing_value("doc-1", "field1", "proc1"));
        labels.add(QualityIssueLabel::typo(
            "doc-1", "field2", "a", "b", "proc2",
        ));
        labels.add(QualityIssueLabel::format_variation(
            "doc-2",
            "date",
            "2024-01-01",
            "01/01/2024",
            "proc3",
        ));

        let summary = labels.summary();
        assert_eq!(summary.total_labels, 3);
        assert_eq!(summary.missing_values, 1);
        assert_eq!(summary.typos, 1);
        assert_eq!(summary.format_variations, 1);
        assert_eq!(summary.unique_documents, 2);
        assert_eq!(summary.unique_fields, 3);
    }

    #[test]
    fn test_csv_export() {
        let mut labels = QualityLabels::new();
        labels.add(QualityIssueLabel::typo(
            "doc-1",
            "name",
            "Test",
            "Tset",
            "typo_proc",
        ));

        let header = QualityLabels::csv_header();
        assert_eq!(header.len(), 9);

        let rows = labels.to_csv_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), 9);
    }
}
