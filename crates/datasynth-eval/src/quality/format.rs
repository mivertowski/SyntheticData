//! Format consistency evaluation.
//!
//! Analyzes format variations in dates, amounts, identifiers, and currency codes.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Results of format consistency analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatAnalysis {
    /// Date format variations.
    pub date_formats: Vec<FormatVariation>,
    /// Amount format variations.
    pub amount_formats: Vec<FormatVariation>,
    /// Identifier format variations.
    pub identifier_formats: Vec<FormatVariation>,
    /// Currency code compliance.
    pub currency_compliance: f64,
    /// Overall format consistency score (0.0-1.0).
    pub consistency_score: f64,
    /// Format issues detected.
    pub issues: Vec<FormatIssue>,
}

/// Variation in a specific format type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatVariation {
    /// Field name.
    pub field_name: String,
    /// Format type (e.g., "ISO", "US", "EU").
    pub format_type: String,
    /// Count of values in this format.
    pub count: usize,
    /// Percentage of total.
    pub percentage: f64,
    /// Example values.
    pub examples: Vec<String>,
}

/// A format issue detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatIssue {
    /// Field name.
    pub field_name: String,
    /// Issue type.
    pub issue_type: FormatIssueType,
    /// Description.
    pub description: String,
    /// Example problematic values.
    pub examples: Vec<String>,
}

/// Type of format issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatIssueType {
    /// Multiple date formats in same field.
    InconsistentDateFormat,
    /// Multiple amount formats in same field.
    InconsistentAmountFormat,
    /// Case inconsistency in identifiers.
    InconsistentCase,
    /// Invalid currency code.
    InvalidCurrencyCode,
    /// Invalid decimal places.
    InvalidDecimalPlaces,
    /// Invalid separator usage.
    InvalidSeparator,
}

/// Detected date format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateFormat {
    /// ISO 8601 (2024-01-15).
    ISO,
    /// US format (01/15/2024).
    US,
    /// European format (15.01.2024).
    EU,
    /// Long format (January 15, 2024).
    Long,
    /// Unknown format.
    Unknown,
}

/// Detected amount format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmountFormat {
    /// Plain (1234.56).
    Plain,
    /// US with comma thousands (1,234.56).
    USComma,
    /// European (1.234,56).
    European,
    /// With currency prefix ($1,234.56).
    CurrencyPrefix,
    /// With currency suffix (1.234,56 EUR).
    CurrencySuffix,
    /// Unknown format.
    Unknown,
}

/// Input data for format analysis.
#[derive(Debug, Clone, Default)]
pub struct FormatData {
    /// Date field values: field_name -> values.
    pub date_fields: HashMap<String, Vec<String>>,
    /// Amount field values: field_name -> values.
    pub amount_fields: HashMap<String, Vec<String>>,
    /// Identifier field values: field_name -> values.
    pub identifier_fields: HashMap<String, Vec<String>>,
    /// Currency codes used.
    pub currency_codes: Vec<String>,
}

/// Analyzer for format consistency.
pub struct FormatAnalyzer {
    /// Valid ISO 4217 currency codes.
    valid_currencies: std::collections::HashSet<String>,
    /// Minimum consistency threshold for a single field.
    min_field_consistency: f64,
}

impl FormatAnalyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        let valid_currencies: std::collections::HashSet<String> = [
            "USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "CNY", "HKD", "SGD", "INR", "BRL",
            "MXN", "KRW", "RUB", "ZAR", "SEK", "NOK", "DKK", "NZD", "THB", "MYR", "IDR", "PHP",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            valid_currencies,
            min_field_consistency: 0.95,
        }
    }

    /// Analyze format consistency.
    pub fn analyze(&self, data: &FormatData) -> EvalResult<FormatAnalysis> {
        let mut date_formats = Vec::new();
        let mut amount_formats = Vec::new();
        let mut identifier_formats = Vec::new();
        let mut issues = Vec::new();
        let mut consistency_scores = Vec::new();

        // Analyze date formats
        for (field_name, values) in &data.date_fields {
            let (formats, field_issues, consistency) = self.analyze_date_field(field_name, values);
            date_formats.extend(formats);
            issues.extend(field_issues);
            consistency_scores.push(consistency);
        }

        // Analyze amount formats
        for (field_name, values) in &data.amount_fields {
            let (formats, field_issues, consistency) =
                self.analyze_amount_field(field_name, values);
            amount_formats.extend(formats);
            issues.extend(field_issues);
            consistency_scores.push(consistency);
        }

        // Analyze identifier formats
        for (field_name, values) in &data.identifier_fields {
            let (formats, field_issues, consistency) =
                self.analyze_identifier_field(field_name, values);
            identifier_formats.extend(formats);
            issues.extend(field_issues);
            consistency_scores.push(consistency);
        }

        // Check currency code compliance
        let valid_count = data
            .currency_codes
            .iter()
            .filter(|c| self.valid_currencies.contains(c.to_uppercase().as_str()))
            .count();
        let currency_compliance = if data.currency_codes.is_empty() {
            1.0
        } else {
            valid_count as f64 / data.currency_codes.len() as f64
        };

        if currency_compliance < 1.0 {
            let invalid: Vec<_> = data
                .currency_codes
                .iter()
                .filter(|c| !self.valid_currencies.contains(c.to_uppercase().as_str()))
                .take(5)
                .cloned()
                .collect();
            issues.push(FormatIssue {
                field_name: "currency_code".to_string(),
                issue_type: FormatIssueType::InvalidCurrencyCode,
                description: format!(
                    "Found {} invalid currency codes",
                    data.currency_codes.len() - valid_count
                ),
                examples: invalid,
            });
        }

        consistency_scores.push(currency_compliance);

        let consistency_score = if consistency_scores.is_empty() {
            1.0
        } else {
            consistency_scores.iter().sum::<f64>() / consistency_scores.len() as f64
        };

        Ok(FormatAnalysis {
            date_formats,
            amount_formats,
            identifier_formats,
            currency_compliance,
            consistency_score,
            issues,
        })
    }

    /// Analyze date field formats.
    fn analyze_date_field(
        &self,
        field_name: &str,
        values: &[String],
    ) -> (Vec<FormatVariation>, Vec<FormatIssue>, f64) {
        let mut format_counts: HashMap<DateFormat, Vec<String>> = HashMap::new();

        for value in values {
            let format = self.detect_date_format(value);
            format_counts.entry(format).or_default().push(value.clone());
        }

        let total = values.len();
        let variations: Vec<FormatVariation> = format_counts
            .iter()
            .map(|(format, examples)| FormatVariation {
                field_name: field_name.to_string(),
                format_type: format!("{:?}", format),
                count: examples.len(),
                percentage: if total > 0 {
                    examples.len() as f64 / total as f64
                } else {
                    0.0
                },
                examples: examples.iter().take(3).cloned().collect(),
            })
            .collect();

        let mut issues = Vec::new();
        let dominant_count = format_counts.values().map(|v| v.len()).max().unwrap_or(0);
        let consistency = if total > 0 {
            dominant_count as f64 / total as f64
        } else {
            1.0
        };

        if consistency < self.min_field_consistency && format_counts.len() > 1 {
            issues.push(FormatIssue {
                field_name: field_name.to_string(),
                issue_type: FormatIssueType::InconsistentDateFormat,
                description: format!(
                    "Multiple date formats detected ({} variants)",
                    format_counts.len()
                ),
                examples: values.iter().take(5).cloned().collect(),
            });
        }

        (variations, issues, consistency)
    }

    /// Detect date format from a string.
    fn detect_date_format(&self, value: &str) -> DateFormat {
        let value = value.trim();

        // ISO format: 2024-01-15
        if value.len() == 10
            && value.chars().nth(4) == Some('-')
            && value.chars().nth(7) == Some('-')
        {
            return DateFormat::ISO;
        }

        // US format: 01/15/2024
        if value.len() == 10
            && value.chars().nth(2) == Some('/')
            && value.chars().nth(5) == Some('/')
        {
            return DateFormat::US;
        }

        // EU format: 15.01.2024
        if value.len() == 10
            && value.chars().nth(2) == Some('.')
            && value.chars().nth(5) == Some('.')
        {
            return DateFormat::EU;
        }

        // Long format contains month name
        if value.contains("January")
            || value.contains("February")
            || value.contains("March")
            || value.contains("April")
            || value.contains("May")
            || value.contains("June")
            || value.contains("July")
            || value.contains("August")
            || value.contains("September")
            || value.contains("October")
            || value.contains("November")
            || value.contains("December")
        {
            return DateFormat::Long;
        }

        DateFormat::Unknown
    }

    /// Analyze amount field formats.
    fn analyze_amount_field(
        &self,
        field_name: &str,
        values: &[String],
    ) -> (Vec<FormatVariation>, Vec<FormatIssue>, f64) {
        let mut format_counts: HashMap<AmountFormat, Vec<String>> = HashMap::new();

        for value in values {
            let format = self.detect_amount_format(value);
            format_counts.entry(format).or_default().push(value.clone());
        }

        let total = values.len();
        let variations: Vec<FormatVariation> = format_counts
            .iter()
            .map(|(format, examples)| FormatVariation {
                field_name: field_name.to_string(),
                format_type: format!("{:?}", format),
                count: examples.len(),
                percentage: if total > 0 {
                    examples.len() as f64 / total as f64
                } else {
                    0.0
                },
                examples: examples.iter().take(3).cloned().collect(),
            })
            .collect();

        let mut issues = Vec::new();
        let dominant_count = format_counts.values().map(|v| v.len()).max().unwrap_or(0);
        let consistency = if total > 0 {
            dominant_count as f64 / total as f64
        } else {
            1.0
        };

        if consistency < self.min_field_consistency && format_counts.len() > 1 {
            issues.push(FormatIssue {
                field_name: field_name.to_string(),
                issue_type: FormatIssueType::InconsistentAmountFormat,
                description: format!(
                    "Multiple amount formats detected ({} variants)",
                    format_counts.len()
                ),
                examples: values.iter().take(5).cloned().collect(),
            });
        }

        (variations, issues, consistency)
    }

    /// Detect amount format from a string.
    fn detect_amount_format(&self, value: &str) -> AmountFormat {
        let value = value.trim();

        // Currency prefix ($, €, £)
        if value.starts_with('$') || value.starts_with('€') || value.starts_with('£') {
            return AmountFormat::CurrencyPrefix;
        }

        // Currency suffix (EUR, USD at end)
        if value.ends_with("EUR")
            || value.ends_with("USD")
            || value.ends_with("GBP")
            || value.ends_with("JPY")
        {
            return AmountFormat::CurrencySuffix;
        }

        // European format (1.234,56)
        if value.contains('.') && value.contains(',') {
            let dot_pos = value.rfind('.').unwrap_or(0);
            let comma_pos = value.rfind(',').unwrap_or(0);
            if comma_pos > dot_pos {
                return AmountFormat::European;
            }
        }

        // US comma format (1,234.56)
        if value.contains(',') && value.contains('.') {
            return AmountFormat::USComma;
        }

        // Plain format (1234.56)
        if value.contains('.') || value.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return AmountFormat::Plain;
        }

        AmountFormat::Unknown
    }

    /// Analyze identifier field formats.
    fn analyze_identifier_field(
        &self,
        field_name: &str,
        values: &[String],
    ) -> (Vec<FormatVariation>, Vec<FormatIssue>, f64) {
        let mut upper_count = 0;
        let mut lower_count = 0;
        let mut mixed_count = 0;

        for value in values {
            if value
                .chars()
                .filter(|c| c.is_alphabetic())
                .all(|c| c.is_uppercase())
            {
                upper_count += 1;
            } else if value
                .chars()
                .filter(|c| c.is_alphabetic())
                .all(|c| c.is_lowercase())
            {
                lower_count += 1;
            } else {
                mixed_count += 1;
            }
        }

        let total = values.len();
        let mut variations = Vec::new();

        if upper_count > 0 {
            variations.push(FormatVariation {
                field_name: field_name.to_string(),
                format_type: "UPPERCASE".to_string(),
                count: upper_count,
                percentage: upper_count as f64 / total.max(1) as f64,
                examples: values
                    .iter()
                    .filter(|v| {
                        v.chars()
                            .filter(|c| c.is_alphabetic())
                            .all(|c| c.is_uppercase())
                    })
                    .take(3)
                    .cloned()
                    .collect(),
            });
        }

        if lower_count > 0 {
            variations.push(FormatVariation {
                field_name: field_name.to_string(),
                format_type: "lowercase".to_string(),
                count: lower_count,
                percentage: lower_count as f64 / total.max(1) as f64,
                examples: values
                    .iter()
                    .filter(|v| {
                        v.chars()
                            .filter(|c| c.is_alphabetic())
                            .all(|c| c.is_lowercase())
                    })
                    .take(3)
                    .cloned()
                    .collect(),
            });
        }

        if mixed_count > 0 {
            variations.push(FormatVariation {
                field_name: field_name.to_string(),
                format_type: "MixedCase".to_string(),
                count: mixed_count,
                percentage: mixed_count as f64 / total.max(1) as f64,
                examples: values.iter().take(3).cloned().collect(),
            });
        }

        let dominant_count = upper_count.max(lower_count).max(mixed_count);
        let consistency = if total > 0 {
            dominant_count as f64 / total as f64
        } else {
            1.0
        };

        let mut issues = Vec::new();
        if consistency < self.min_field_consistency && variations.len() > 1 {
            issues.push(FormatIssue {
                field_name: field_name.to_string(),
                issue_type: FormatIssueType::InconsistentCase,
                description: format!(
                    "Mixed case formats detected ({} variants)",
                    variations.len()
                ),
                examples: values.iter().take(5).cloned().collect(),
            });
        }

        (variations, issues, consistency)
    }
}

impl Default for FormatAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_formats() {
        let mut data = FormatData::default();
        data.date_fields.insert(
            "posting_date".to_string(),
            vec![
                "2024-01-15".to_string(),
                "2024-01-16".to_string(),
                "2024-01-17".to_string(),
            ],
        );

        let analyzer = FormatAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert_eq!(result.date_formats.len(), 1);
        assert!(result.consistency_score > 0.95);
    }

    #[test]
    fn test_date_format_detection() {
        let analyzer = FormatAnalyzer::new();

        assert_eq!(analyzer.detect_date_format("2024-01-15"), DateFormat::ISO);
        assert_eq!(analyzer.detect_date_format("01/15/2024"), DateFormat::US);
        assert_eq!(analyzer.detect_date_format("15.01.2024"), DateFormat::EU);
        assert_eq!(
            analyzer.detect_date_format("January 15, 2024"),
            DateFormat::Long
        );
    }

    #[test]
    fn test_currency_compliance() {
        let mut data = FormatData::default();
        data.currency_codes = vec!["USD".to_string(), "EUR".to_string(), "INVALID".to_string()];

        let analyzer = FormatAnalyzer::new();
        let result = analyzer.analyze(&data).unwrap();

        assert!(result.currency_compliance < 1.0);
        assert!(result.currency_compliance > 0.5);
    }
}
