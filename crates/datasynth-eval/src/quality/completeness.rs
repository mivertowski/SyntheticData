//! Completeness evaluation.
//!
//! Analyzes missing values, required field coverage, and missing patterns.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Results of completeness analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessAnalysis {
    /// Total records analyzed.
    pub total_records: usize,
    /// Per-field completeness metrics.
    pub field_completeness: Vec<FieldCompleteness>,
    /// Overall completeness rate (0.0-1.0).
    pub overall_completeness: f64,
    /// Required field completeness rate.
    pub required_completeness: f64,
    /// Optional field completeness rate.
    pub optional_completeness: f64,
    /// Detected missing pattern.
    pub missing_pattern: MissingPattern,
    /// Fields with systematic missing values.
    pub systematic_missing: Vec<String>,
    /// Record-level completeness (% of records with all required fields).
    pub record_completeness: f64,
}

/// Completeness metrics for a single field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCompleteness {
    /// Field name.
    pub field_name: String,
    /// Whether field is required.
    pub is_required: bool,
    /// Total values.
    pub total_values: usize,
    /// Non-null, non-empty values.
    pub present_values: usize,
    /// Null values.
    pub null_values: usize,
    /// Empty string values.
    pub empty_values: usize,
    /// Completeness rate (0.0-1.0).
    pub completeness_rate: f64,
}

/// Detected missing value pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingPattern {
    /// Missing Completely At Random - equal probability across all records.
    MCAR,
    /// Missing At Random - depends on other observed values.
    MAR,
    /// Missing Not At Random - depends on the missing value itself.
    MNAR,
    /// Systematic - entire field groups missing together.
    Systematic,
    /// No significant missing pattern detected.
    None,
}

/// Field definition for completeness checking.
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Field name.
    pub name: String,
    /// Whether field is required.
    pub required: bool,
    /// Related fields (for pattern detection).
    pub related_fields: Vec<String>,
}

/// A single field value for completeness checking.
#[derive(Debug, Clone)]
pub enum FieldValue {
    /// Present value (not null, not empty).
    Present,
    /// Null value.
    Null,
    /// Empty string.
    Empty,
}

/// Analyzer for completeness.
pub struct CompletenessAnalyzer {
    /// Field definitions.
    field_definitions: Vec<FieldDefinition>,
}

impl CompletenessAnalyzer {
    /// Create a new analyzer with field definitions.
    pub fn new(field_definitions: Vec<FieldDefinition>) -> Self {
        Self { field_definitions }
    }

    /// Analyze completeness of records.
    pub fn analyze(
        &self,
        records: &[HashMap<String, FieldValue>],
    ) -> EvalResult<CompletenessAnalysis> {
        let total_records = records.len();
        if total_records == 0 {
            return Ok(CompletenessAnalysis {
                total_records: 0,
                field_completeness: vec![],
                overall_completeness: 1.0,
                required_completeness: 1.0,
                optional_completeness: 1.0,
                missing_pattern: MissingPattern::None,
                systematic_missing: vec![],
                record_completeness: 1.0,
            });
        }

        let mut field_completeness = Vec::new();
        let mut required_total = 0;
        let mut required_present = 0;
        let mut optional_total = 0;
        let mut optional_present = 0;
        let mut all_total = 0;
        let mut all_present = 0;

        // Analyze each field
        for field_def in &self.field_definitions {
            let mut present = 0;
            let mut null = 0;
            let mut empty = 0;

            for record in records {
                match record.get(&field_def.name) {
                    Some(FieldValue::Present) => present += 1,
                    Some(FieldValue::Null) => null += 1,
                    Some(FieldValue::Empty) => empty += 1,
                    None => null += 1,
                }
            }

            let total = present + null + empty;
            let rate = if total > 0 {
                present as f64 / total as f64
            } else {
                1.0
            };

            if field_def.required {
                required_total += total;
                required_present += present;
            } else {
                optional_total += total;
                optional_present += present;
            }

            all_total += total;
            all_present += present;

            field_completeness.push(FieldCompleteness {
                field_name: field_def.name.clone(),
                is_required: field_def.required,
                total_values: total,
                present_values: present,
                null_values: null,
                empty_values: empty,
                completeness_rate: rate,
            });
        }

        let overall_completeness = if all_total > 0 {
            all_present as f64 / all_total as f64
        } else {
            1.0
        };

        let required_completeness = if required_total > 0 {
            required_present as f64 / required_total as f64
        } else {
            1.0
        };

        let optional_completeness = if optional_total > 0 {
            optional_present as f64 / optional_total as f64
        } else {
            1.0
        };

        // Detect missing pattern
        let (missing_pattern, systematic_missing) =
            self.detect_missing_pattern(records, &field_completeness);

        // Calculate record-level completeness
        let required_fields: Vec<_> = self
            .field_definitions
            .iter()
            .filter(|f| f.required)
            .map(|f| &f.name)
            .collect();

        let complete_records = records
            .iter()
            .filter(|record| {
                required_fields
                    .iter()
                    .all(|field| matches!(record.get(*field), Some(FieldValue::Present)))
            })
            .count();

        let record_completeness = if total_records > 0 {
            complete_records as f64 / total_records as f64
        } else {
            1.0
        };

        Ok(CompletenessAnalysis {
            total_records,
            field_completeness,
            overall_completeness,
            required_completeness,
            optional_completeness,
            missing_pattern,
            systematic_missing,
            record_completeness,
        })
    }

    /// Detect missing value pattern.
    fn detect_missing_pattern(
        &self,
        records: &[HashMap<String, FieldValue>],
        field_completeness: &[FieldCompleteness],
    ) -> (MissingPattern, Vec<String>) {
        let mut systematic_missing = Vec::new();

        // Check for systematic patterns (fields missing together)
        for field_def in &self.field_definitions {
            if !field_def.related_fields.is_empty() {
                let field_missing: Vec<bool> = records
                    .iter()
                    .map(|r| !matches!(r.get(&field_def.name), Some(FieldValue::Present)))
                    .collect();

                for related in &field_def.related_fields {
                    let related_missing: Vec<bool> = records
                        .iter()
                        .map(|r| !matches!(r.get(related), Some(FieldValue::Present)))
                        .collect();

                    // Check correlation
                    let both_missing = field_missing
                        .iter()
                        .zip(&related_missing)
                        .filter(|(a, b)| **a && **b)
                        .count();
                    let either_missing = field_missing
                        .iter()
                        .zip(&related_missing)
                        .filter(|(a, b)| **a || **b)
                        .count();

                    if either_missing > 0 && both_missing as f64 / either_missing as f64 > 0.8 {
                        systematic_missing.push(format!("{} + {}", field_def.name, related));
                    }
                }
            }
        }

        if !systematic_missing.is_empty() {
            return (MissingPattern::Systematic, systematic_missing);
        }

        // Check for MCAR (uniform missing rate across all fields)
        let rates: Vec<f64> = field_completeness
            .iter()
            .map(|f| 1.0 - f.completeness_rate)
            .filter(|r| *r > 0.0)
            .collect();

        if rates.is_empty() {
            return (MissingPattern::None, vec![]);
        }

        let mean_rate = rates.iter().sum::<f64>() / rates.len() as f64;
        let variance: f64 =
            rates.iter().map(|r| (r - mean_rate).powi(2)).sum::<f64>() / rates.len() as f64;
        let std_dev = variance.sqrt();

        // Low variance suggests MCAR
        if std_dev < 0.05 {
            return (MissingPattern::MCAR, vec![]);
        }

        // Default to MAR if we can't determine pattern
        (MissingPattern::MAR, vec![])
    }
}

impl Default for CompletenessAnalyzer {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_data() {
        let fields = vec![
            FieldDefinition {
                name: "id".to_string(),
                required: true,
                related_fields: vec![],
            },
            FieldDefinition {
                name: "name".to_string(),
                required: true,
                related_fields: vec![],
            },
        ];

        let records: Vec<HashMap<String, FieldValue>> = vec![
            [
                ("id".to_string(), FieldValue::Present),
                ("name".to_string(), FieldValue::Present),
            ]
            .into_iter()
            .collect(),
            [
                ("id".to_string(), FieldValue::Present),
                ("name".to_string(), FieldValue::Present),
            ]
            .into_iter()
            .collect(),
        ];

        let analyzer = CompletenessAnalyzer::new(fields);
        let result = analyzer.analyze(&records).unwrap();

        assert_eq!(result.overall_completeness, 1.0);
        assert_eq!(result.record_completeness, 1.0);
    }

    #[test]
    fn test_missing_values() {
        let fields = vec![
            FieldDefinition {
                name: "id".to_string(),
                required: true,
                related_fields: vec![],
            },
            FieldDefinition {
                name: "name".to_string(),
                required: true,
                related_fields: vec![],
            },
        ];

        let records: Vec<HashMap<String, FieldValue>> = vec![
            [
                ("id".to_string(), FieldValue::Present),
                ("name".to_string(), FieldValue::Null),
            ]
            .into_iter()
            .collect(),
            [
                ("id".to_string(), FieldValue::Present),
                ("name".to_string(), FieldValue::Present),
            ]
            .into_iter()
            .collect(),
        ];

        let analyzer = CompletenessAnalyzer::new(fields);
        let result = analyzer.analyze(&records).unwrap();

        assert!(result.overall_completeness < 1.0);
        assert_eq!(result.record_completeness, 0.5);
    }

    #[test]
    fn test_empty_records() {
        let analyzer = CompletenessAnalyzer::default();
        let result = analyzer.analyze(&[]).unwrap();
        assert_eq!(result.overall_completeness, 1.0);
    }
}
