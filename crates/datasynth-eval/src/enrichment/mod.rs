//! LLM enrichment quality evaluator.
//!
//! Validates the quality of LLM-enriched text fields including
//! non-empty rates, uniqueness, suspicious patterns, and structured field consistency.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Enriched text field data for validation.
#[derive(Debug, Clone)]
pub struct EnrichedFieldData {
    /// Field name (e.g., "vendor_description", "journal_entry_memo").
    pub field_name: String,
    /// The enriched text value.
    pub text_value: String,
    /// Optional associated structured field for consistency check.
    pub structured_context: Option<String>,
}

/// Thresholds for enrichment quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentThresholds {
    /// Minimum non-empty rate.
    pub min_non_empty_rate: f64,
    /// Minimum unique text rate.
    pub min_unique_rate: f64,
    /// Maximum suspicious pattern rate.
    pub max_suspicious_rate: f64,
}

impl Default for EnrichmentThresholds {
    fn default() -> Self {
        Self {
            min_non_empty_rate: 0.95,
            min_unique_rate: 0.80,
            max_suspicious_rate: 0.05,
        }
    }
}

/// Suspicious text patterns.
const SUSPICIOUS_PATTERNS: &[&str] = &[
    "lorem ipsum",
    "placeholder",
    "todo",
    "test data",
    "sample text",
    "n/a",
    "tbd",
    "xxx",
    "abc123",
    "asdf",
];

/// Results of enrichment quality analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentQualityEvaluation {
    /// Non-empty rate: fraction of enriched fields with non-empty text.
    pub non_empty_rate: f64,
    /// Unique text rate: fraction of unique texts among non-empty.
    pub unique_text_rate: f64,
    /// Suspicious pattern rate: fraction containing suspicious text.
    pub suspicious_pattern_rate: f64,
    /// Average text length of non-empty fields.
    pub avg_text_length: f64,
    /// Total fields evaluated.
    pub total_fields: usize,
    /// Non-empty fields.
    pub non_empty_count: usize,
    /// Fields with suspicious patterns.
    pub suspicious_count: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for LLM enrichment quality.
pub struct EnrichmentQualityEvaluator {
    thresholds: EnrichmentThresholds,
}

impl EnrichmentQualityEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: EnrichmentThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: EnrichmentThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate enriched field quality.
    pub fn evaluate(
        &self,
        fields: &[EnrichedFieldData],
    ) -> EvalResult<EnrichmentQualityEvaluation> {
        let mut issues = Vec::new();
        let total = fields.len();

        if total == 0 {
            return Ok(EnrichmentQualityEvaluation {
                non_empty_rate: 1.0,
                unique_text_rate: 1.0,
                suspicious_pattern_rate: 0.0,
                avg_text_length: 0.0,
                total_fields: 0,
                non_empty_count: 0,
                suspicious_count: 0,
                passes: true,
                issues: Vec::new(),
            });
        }

        // Non-empty rate
        let non_empty: Vec<&EnrichedFieldData> = fields
            .iter()
            .filter(|f| !f.text_value.trim().is_empty())
            .collect();
        let non_empty_count = non_empty.len();
        let non_empty_rate = non_empty_count as f64 / total as f64;

        // Unique text rate
        let unique_texts: HashSet<&str> = non_empty.iter().map(|f| f.text_value.as_str()).collect();
        let unique_text_rate = if non_empty_count > 0 {
            unique_texts.len() as f64 / non_empty_count as f64
        } else {
            1.0
        };

        // Suspicious pattern rate
        let suspicious_count = non_empty
            .iter()
            .filter(|f| {
                let lower = f.text_value.to_lowercase();
                SUSPICIOUS_PATTERNS
                    .iter()
                    .any(|pattern| lower.contains(pattern))
            })
            .count();
        let suspicious_pattern_rate = if non_empty_count > 0 {
            suspicious_count as f64 / non_empty_count as f64
        } else {
            0.0
        };

        // Average text length
        let total_length: usize = non_empty.iter().map(|f| f.text_value.len()).sum();
        let avg_text_length = if non_empty_count > 0 {
            total_length as f64 / non_empty_count as f64
        } else {
            0.0
        };

        // Check thresholds
        if non_empty_rate < self.thresholds.min_non_empty_rate {
            issues.push(format!(
                "Non-empty rate {:.3} < {:.3}",
                non_empty_rate, self.thresholds.min_non_empty_rate
            ));
        }
        if unique_text_rate < self.thresholds.min_unique_rate {
            issues.push(format!(
                "Unique text rate {:.3} < {:.3}",
                unique_text_rate, self.thresholds.min_unique_rate
            ));
        }
        if suspicious_pattern_rate > self.thresholds.max_suspicious_rate {
            issues.push(format!(
                "Suspicious pattern rate {:.3} > {:.3}",
                suspicious_pattern_rate, self.thresholds.max_suspicious_rate
            ));
        }

        let passes = issues.is_empty();

        Ok(EnrichmentQualityEvaluation {
            non_empty_rate,
            unique_text_rate,
            suspicious_pattern_rate,
            avg_text_length,
            total_fields: total,
            non_empty_count,
            suspicious_count,
            passes,
            issues,
        })
    }
}

impl Default for EnrichmentQualityEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_good_enrichment() {
        let evaluator = EnrichmentQualityEvaluator::new();
        let fields = vec![
            EnrichedFieldData {
                field_name: "description".to_string(),
                text_value: "Office supplies for Q1 2024 operations".to_string(),
                structured_context: None,
            },
            EnrichedFieldData {
                field_name: "description".to_string(),
                text_value: "IT equipment maintenance contract renewal".to_string(),
                structured_context: None,
            },
        ];

        let result = evaluator.evaluate(&fields).unwrap();
        assert!(result.passes);
        assert_eq!(result.non_empty_rate, 1.0);
        assert_eq!(result.unique_text_rate, 1.0);
    }

    #[test]
    fn test_suspicious_patterns() {
        let evaluator = EnrichmentQualityEvaluator::new();
        let fields = vec![
            EnrichedFieldData {
                field_name: "desc".to_string(),
                text_value: "Lorem ipsum dolor sit amet".to_string(),
                structured_context: None,
            },
            EnrichedFieldData {
                field_name: "desc".to_string(),
                text_value: "This is placeholder text for testing".to_string(),
                structured_context: None,
            },
        ];

        let result = evaluator.evaluate(&fields).unwrap();
        assert!(!result.passes);
        assert_eq!(result.suspicious_count, 2);
    }

    #[test]
    fn test_all_duplicate_text() {
        let evaluator = EnrichmentQualityEvaluator::new();
        let fields: Vec<EnrichedFieldData> = (0..10)
            .map(|_| EnrichedFieldData {
                field_name: "desc".to_string(),
                text_value: "Same text everywhere".to_string(),
                structured_context: None,
            })
            .collect();

        let result = evaluator.evaluate(&fields).unwrap();
        assert!(!result.passes);
        assert!((result.unique_text_rate - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_empty() {
        let evaluator = EnrichmentQualityEvaluator::new();
        let result = evaluator.evaluate(&[]).unwrap();
        assert!(result.passes);
    }
}
