//! Main data quality injector coordinating all quality issues.
//!
//! This module provides a unified interface for introducing various
//! data quality issues into synthetic data.

use chrono::NaiveDate;
use datasynth_core::utils::seeded_rng;
use datasynth_core::CountryPack;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::duplicates::{DuplicateConfig, DuplicateGenerator, DuplicateStats};
use super::format_variations::{
    AmountFormat, DateFormat, FormatVariationConfig, FormatVariationInjector, FormatVariationStats,
};
use super::missing_values::{MissingValueConfig, MissingValueInjector, MissingValueStats};
use super::typos::{introduce_encoding_issue, EncodingIssue, TypoConfig, TypoGenerator, TypoStats};

/// Configuration for the data quality injector.
#[derive(Debug, Clone)]
pub struct DataQualityConfig {
    /// Enable missing value injection.
    pub enable_missing_values: bool,
    /// Missing value configuration.
    pub missing_values: MissingValueConfig,
    /// Enable format variations.
    pub enable_format_variations: bool,
    /// Format variation configuration.
    pub format_variations: FormatVariationConfig,
    /// Enable duplicates.
    pub enable_duplicates: bool,
    /// Duplicate configuration.
    pub duplicates: DuplicateConfig,
    /// Enable typos.
    pub enable_typos: bool,
    /// Typo configuration.
    pub typos: TypoConfig,
    /// Enable encoding issues.
    pub enable_encoding_issues: bool,
    /// Encoding issue rate.
    pub encoding_issue_rate: f64,
    /// Random seed for reproducibility.
    pub seed: u64,
    /// Track detailed statistics.
    pub track_statistics: bool,
}

impl Default for DataQualityConfig {
    fn default() -> Self {
        Self {
            enable_missing_values: true,
            missing_values: MissingValueConfig::default(),
            enable_format_variations: true,
            format_variations: FormatVariationConfig::default(),
            enable_duplicates: true,
            duplicates: DuplicateConfig::default(),
            enable_typos: true,
            typos: TypoConfig::default(),
            enable_encoding_issues: false, // Off by default (can cause issues)
            encoding_issue_rate: 0.001,
            seed: 42,
            track_statistics: true,
        }
    }
}

impl DataQualityConfig {
    /// Creates a minimal configuration (low rates).
    pub fn minimal() -> Self {
        Self {
            missing_values: MissingValueConfig {
                global_rate: 0.005,
                ..Default::default()
            },
            format_variations: FormatVariationConfig {
                date_variation_rate: 0.01,
                amount_variation_rate: 0.01,
                identifier_variation_rate: 0.005,
                text_variation_rate: 0.01,
                ..Default::default()
            },
            duplicates: DuplicateConfig {
                duplicate_rate: 0.001,
                ..Default::default()
            },
            typos: TypoConfig {
                char_error_rate: 0.001,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Creates a high-variation configuration (for stress testing).
    pub fn high_variation() -> Self {
        Self {
            missing_values: MissingValueConfig {
                global_rate: 0.05,
                ..Default::default()
            },
            format_variations: FormatVariationConfig {
                date_variation_rate: 0.2,
                amount_variation_rate: 0.1,
                identifier_variation_rate: 0.1,
                text_variation_rate: 0.2,
                ..Default::default()
            },
            duplicates: DuplicateConfig {
                duplicate_rate: 0.02,
                ..Default::default()
            },
            typos: TypoConfig {
                char_error_rate: 0.02,
                ..Default::default()
            },
            enable_encoding_issues: true,
            encoding_issue_rate: 0.01,
            ..Default::default()
        }
    }
}

/// Combined statistics for all data quality issues.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataQualityStats {
    /// Missing value statistics.
    pub missing_values: MissingValueStats,
    /// Format variation statistics.
    pub format_variations: FormatVariationStats,
    /// Duplicate statistics.
    pub duplicates: DuplicateStats,
    /// Typo statistics.
    pub typos: TypoStats,
    /// Encoding issues injected.
    pub encoding_issues: usize,
    /// Total records processed.
    pub total_records: usize,
    /// Records with any quality issue.
    pub records_with_issues: usize,
}

impl DataQualityStats {
    /// Returns the overall issue rate.
    pub fn overall_issue_rate(&self) -> f64 {
        if self.total_records == 0 {
            0.0
        } else {
            self.records_with_issues as f64 / self.total_records as f64
        }
    }

    /// Returns a summary of issues.
    pub fn summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        summary.insert(
            "missing_values".to_string(),
            self.missing_values.total_missing,
        );
        summary.insert(
            "format_variations".to_string(),
            self.format_variations.date_variations
                + self.format_variations.amount_variations
                + self.format_variations.identifier_variations
                + self.format_variations.text_variations,
        );
        summary.insert("duplicates".to_string(), self.duplicates.total_duplicates);
        summary.insert("typos".to_string(), self.typos.total_typos);
        summary.insert("encoding_issues".to_string(), self.encoding_issues);
        summary
    }
}

/// A data quality issue record.
#[derive(Debug, Clone)]
pub struct QualityIssue {
    /// Unique issue identifier.
    pub issue_id: String,
    /// Type of issue.
    pub issue_type: QualityIssueType,
    /// Record identifier.
    pub record_id: String,
    /// Field affected.
    pub field: Option<String>,
    /// Original value (if available).
    pub original_value: Option<String>,
    /// Modified value (if available).
    pub modified_value: Option<String>,
    /// Description.
    pub description: String,
}

/// Type of quality issue.
#[derive(Debug, Clone, PartialEq)]
pub enum QualityIssueType {
    /// Missing value.
    MissingValue,
    /// Date format variation.
    DateFormatVariation,
    /// Amount format variation.
    AmountFormatVariation,
    /// Identifier format variation.
    IdentifierFormatVariation,
    /// Text format variation.
    TextFormatVariation,
    /// Exact duplicate.
    ExactDuplicate,
    /// Near duplicate.
    NearDuplicate,
    /// Fuzzy duplicate.
    FuzzyDuplicate,
    /// Typo.
    Typo,
    /// Encoding issue.
    EncodingIssue,
}

/// Main data quality injector.
pub struct DataQualityInjector {
    config: DataQualityConfig,
    rng: ChaCha8Rng,
    missing_value_injector: MissingValueInjector,
    format_injector: FormatVariationInjector,
    duplicate_generator: DuplicateGenerator,
    typo_generator: TypoGenerator,
    stats: DataQualityStats,
    issues: Vec<QualityIssue>,
    next_issue_id: u64,
}

impl DataQualityInjector {
    /// Creates a new data quality injector.
    pub fn new(config: DataQualityConfig) -> Self {
        let rng = seeded_rng(config.seed, 0);
        let missing_value_injector = MissingValueInjector::new(config.missing_values.clone());
        let format_injector = FormatVariationInjector::new(config.format_variations.clone());
        let duplicate_generator = DuplicateGenerator::new(config.duplicates.clone());
        let typo_generator = TypoGenerator::new(config.typos.clone());

        Self {
            config,
            rng,
            missing_value_injector,
            format_injector,
            duplicate_generator,
            typo_generator,
            stats: DataQualityStats::default(),
            issues: Vec::new(),
            next_issue_id: 1,
        }
    }

    /// Set the country pack for locale-aware format variation baselines.
    ///
    /// Propagates to the internal `FormatVariationInjector` so that date and
    /// amount baselines reflect the country's locale conventions.
    pub fn set_country_pack(&mut self, pack: CountryPack) {
        self.format_injector.set_country_pack(pack);
    }

    /// Processes a text field, potentially introducing quality issues.
    pub fn process_text_field(
        &mut self,
        field: &str,
        value: &str,
        record_id: &str,
        context: &HashMap<String, String>,
    ) -> Option<String> {
        let mut result = value.to_string();
        let mut had_issue = false;

        // Check for missing value
        if self.config.enable_missing_values
            && self.missing_value_injector.should_be_missing(
                field,
                Some(value),
                context,
                &mut self.rng,
            )
        {
            let issue_id = self.next_issue_id();
            self.record_issue(QualityIssue {
                issue_id,
                issue_type: QualityIssueType::MissingValue,
                record_id: record_id.to_string(),
                field: Some(field.to_string()),
                original_value: Some(value.to_string()),
                modified_value: None,
                description: format!("Field '{}' set to missing", field),
            });
            return None;
        }

        // Apply typos
        if self.config.enable_typos && !self.typo_generator.is_protected(field) {
            let with_typos = self.typo_generator.introduce_typos(&result, &mut self.rng);
            if with_typos != result {
                let issue_id = self.next_issue_id();
                self.record_issue(QualityIssue {
                    issue_id,
                    issue_type: QualityIssueType::Typo,
                    record_id: record_id.to_string(),
                    field: Some(field.to_string()),
                    original_value: Some(result.clone()),
                    modified_value: Some(with_typos.clone()),
                    description: format!("Typo introduced in field '{}'", field),
                });
                result = with_typos;
                had_issue = true;
            }
        }

        // Apply format variations
        if self.config.enable_format_variations {
            let varied = self.format_injector.vary_text(&result, &mut self.rng);
            if varied != result {
                let issue_id = self.next_issue_id();
                self.record_issue(QualityIssue {
                    issue_id,
                    issue_type: QualityIssueType::TextFormatVariation,
                    record_id: record_id.to_string(),
                    field: Some(field.to_string()),
                    original_value: Some(result.clone()),
                    modified_value: Some(varied.clone()),
                    description: format!("Format variation in field '{}'", field),
                });
                result = varied;
                had_issue = true;
            }
        }

        // Apply encoding issues
        if self.config.enable_encoding_issues
            && self.rng.gen::<f64>() < self.config.encoding_issue_rate
        {
            let issues = [
                EncodingIssue::Mojibake,
                EncodingIssue::MissingChars,
                EncodingIssue::HTMLEntities,
            ];
            let issue = issues[self.rng.gen_range(0..issues.len())];
            let with_encoding = introduce_encoding_issue(&result, issue, &mut self.rng);

            if with_encoding != result {
                let issue_id = self.next_issue_id();
                self.record_issue(QualityIssue {
                    issue_id,
                    issue_type: QualityIssueType::EncodingIssue,
                    record_id: record_id.to_string(),
                    field: Some(field.to_string()),
                    original_value: Some(result.clone()),
                    modified_value: Some(with_encoding.clone()),
                    description: format!("Encoding issue ({:?}) in field '{}'", issue, field),
                });
                result = with_encoding;
                had_issue = true;
                self.stats.encoding_issues += 1;
            }
        }

        if had_issue {
            self.stats.records_with_issues += 1;
        }

        Some(result)
    }

    /// Processes a date field, potentially introducing format variations.
    pub fn process_date_field(
        &mut self,
        field: &str,
        date: NaiveDate,
        record_id: &str,
        context: &HashMap<String, String>,
    ) -> Option<String> {
        // Check for missing value
        if self.config.enable_missing_values
            && self.missing_value_injector.should_be_missing(
                field,
                Some(&date.to_string()),
                context,
                &mut self.rng,
            )
        {
            let issue_id = self.next_issue_id();
            self.record_issue(QualityIssue {
                issue_id,
                issue_type: QualityIssueType::MissingValue,
                record_id: record_id.to_string(),
                field: Some(field.to_string()),
                original_value: Some(date.to_string()),
                modified_value: None,
                description: format!("Date field '{}' set to missing", field),
            });
            return None;
        }

        // Apply format variations
        if self.config.enable_format_variations {
            let formatted = self.format_injector.vary_date(date, &mut self.rng);
            let standard = DateFormat::ISO.format(date);

            if formatted != standard {
                let issue_id = self.next_issue_id();
                self.record_issue(QualityIssue {
                    issue_id,
                    issue_type: QualityIssueType::DateFormatVariation,
                    record_id: record_id.to_string(),
                    field: Some(field.to_string()),
                    original_value: Some(standard),
                    modified_value: Some(formatted.clone()),
                    description: format!("Date format variation in field '{}'", field),
                });
            }

            return Some(formatted);
        }

        Some(DateFormat::ISO.format(date))
    }

    /// Processes an amount field, potentially introducing format variations.
    pub fn process_amount_field(
        &mut self,
        field: &str,
        amount: Decimal,
        record_id: &str,
        context: &HashMap<String, String>,
    ) -> Option<String> {
        // Check for missing value
        if self.config.enable_missing_values
            && self.missing_value_injector.should_be_missing(
                field,
                Some(&amount.to_string()),
                context,
                &mut self.rng,
            )
        {
            let issue_id = self.next_issue_id();
            self.record_issue(QualityIssue {
                issue_id,
                issue_type: QualityIssueType::MissingValue,
                record_id: record_id.to_string(),
                field: Some(field.to_string()),
                original_value: Some(amount.to_string()),
                modified_value: None,
                description: format!("Amount field '{}' set to missing", field),
            });
            return None;
        }

        // Apply format variations
        if self.config.enable_format_variations {
            let formatted = self.format_injector.vary_amount(amount, &mut self.rng);
            let standard = AmountFormat::Plain.format(amount);

            if formatted != standard {
                let issue_id = self.next_issue_id();
                self.record_issue(QualityIssue {
                    issue_id,
                    issue_type: QualityIssueType::AmountFormatVariation,
                    record_id: record_id.to_string(),
                    field: Some(field.to_string()),
                    original_value: Some(standard),
                    modified_value: Some(formatted.clone()),
                    description: format!("Amount format variation in field '{}'", field),
                });
            }

            return Some(formatted);
        }

        Some(AmountFormat::Plain.format(amount))
    }

    /// Processes an identifier field, potentially introducing variations.
    pub fn process_identifier_field(
        &mut self,
        field: &str,
        id: &str,
        record_id: &str,
        context: &HashMap<String, String>,
    ) -> Option<String> {
        // Check for missing value (rare for identifiers)
        if self.config.enable_missing_values
            && self.missing_value_injector.should_be_missing(
                field,
                Some(id),
                context,
                &mut self.rng,
            )
        {
            let issue_id = self.next_issue_id();
            self.record_issue(QualityIssue {
                issue_id,
                issue_type: QualityIssueType::MissingValue,
                record_id: record_id.to_string(),
                field: Some(field.to_string()),
                original_value: Some(id.to_string()),
                modified_value: None,
                description: format!("Identifier field '{}' set to missing", field),
            });
            return None;
        }

        // Apply format variations
        if self.config.enable_format_variations {
            let varied = self.format_injector.vary_identifier(id, &mut self.rng);

            if varied != id {
                let issue_id = self.next_issue_id();
                self.record_issue(QualityIssue {
                    issue_id,
                    issue_type: QualityIssueType::IdentifierFormatVariation,
                    record_id: record_id.to_string(),
                    field: Some(field.to_string()),
                    original_value: Some(id.to_string()),
                    modified_value: Some(varied.clone()),
                    description: format!("Identifier format variation in field '{}'", field),
                });
            }

            return Some(varied);
        }

        Some(id.to_string())
    }

    /// Determines if a record should be duplicated.
    pub fn should_duplicate(&mut self) -> bool {
        self.config.enable_duplicates && self.duplicate_generator.should_duplicate(&mut self.rng)
    }

    /// Records a quality issue.
    fn record_issue(&mut self, issue: QualityIssue) {
        if self.config.track_statistics {
            self.issues.push(issue);
        }
    }

    /// Generates the next issue ID.
    fn next_issue_id(&mut self) -> String {
        let id = format!("QI{:08}", self.next_issue_id);
        self.next_issue_id += 1;
        id
    }

    /// Returns statistics.
    pub fn stats(&self) -> &DataQualityStats {
        &self.stats
    }

    /// Returns all recorded issues.
    pub fn issues(&self) -> &[QualityIssue] {
        &self.issues
    }

    /// Returns issues for a specific record.
    pub fn issues_for_record(&self, record_id: &str) -> Vec<&QualityIssue> {
        self.issues
            .iter()
            .filter(|i| i.record_id == record_id)
            .collect()
    }

    /// Returns issues of a specific type.
    pub fn issues_by_type(&self, issue_type: QualityIssueType) -> Vec<&QualityIssue> {
        self.issues
            .iter()
            .filter(|i| i.issue_type == issue_type)
            .collect()
    }

    /// Resets all statistics and issues.
    pub fn reset(&mut self) {
        self.stats = DataQualityStats::default();
        self.issues.clear();
        self.next_issue_id = 1;
        self.missing_value_injector.reset_stats();
        self.format_injector.reset_stats();
        self.duplicate_generator.reset_stats();
        self.typo_generator.reset_stats();
    }

    /// Updates aggregate statistics.
    pub fn update_stats(&mut self) {
        self.stats.missing_values = self.missing_value_injector.stats().clone();
        self.stats.format_variations = self.format_injector.stats().clone();
        self.stats.duplicates = self.duplicate_generator.stats().clone();
        self.stats.typos = self.typo_generator.stats().clone();
    }
}

/// Builder for DataQualityConfig.
pub struct DataQualityConfigBuilder {
    config: DataQualityConfig,
}

impl DataQualityConfigBuilder {
    /// Creates a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: DataQualityConfig::default(),
        }
    }

    /// Enables or disables missing values.
    pub fn with_missing_values(mut self, enable: bool) -> Self {
        self.config.enable_missing_values = enable;
        self
    }

    /// Sets the global missing value rate.
    pub fn with_missing_rate(mut self, rate: f64) -> Self {
        self.config.missing_values.global_rate = rate;
        self
    }

    /// Enables or disables format variations.
    pub fn with_format_variations(mut self, enable: bool) -> Self {
        self.config.enable_format_variations = enable;
        self
    }

    /// Enables or disables duplicates.
    pub fn with_duplicates(mut self, enable: bool) -> Self {
        self.config.enable_duplicates = enable;
        self
    }

    /// Sets the duplicate rate.
    pub fn with_duplicate_rate(mut self, rate: f64) -> Self {
        self.config.duplicates.duplicate_rate = rate;
        self
    }

    /// Enables or disables typos.
    pub fn with_typos(mut self, enable: bool) -> Self {
        self.config.enable_typos = enable;
        self
    }

    /// Sets the typo rate.
    pub fn with_typo_rate(mut self, rate: f64) -> Self {
        self.config.typos.char_error_rate = rate;
        self
    }

    /// Enables or disables encoding issues.
    pub fn with_encoding_issues(mut self, enable: bool) -> Self {
        self.config.enable_encoding_issues = enable;
        self
    }

    /// Sets the random seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.config.seed = seed;
        self
    }

    /// Builds the configuration.
    pub fn build(self) -> DataQualityConfig {
        self.config
    }
}

impl Default for DataQualityConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_data_quality_injector_creation() {
        let config = DataQualityConfig::default();
        let injector = DataQualityInjector::new(config);

        assert_eq!(injector.stats().total_records, 0);
    }

    #[test]
    fn test_text_field_processing() {
        let config = DataQualityConfigBuilder::new()
            .with_typo_rate(0.5) // High rate for testing
            .with_seed(42)
            .build();

        let mut injector = DataQualityInjector::new(config);
        let context = HashMap::new();

        let result =
            injector.process_text_field("description", "Test Entry Description", "JE001", &context);

        assert!(result.is_some());
    }

    #[test]
    fn test_date_field_processing() {
        let config = DataQualityConfigBuilder::new()
            .with_format_variations(true)
            .with_seed(42)
            .build();

        let mut injector = DataQualityInjector::new(config);
        let context = HashMap::new();

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let result = injector.process_date_field("posting_date", date, "JE001", &context);

        assert!(result.is_some());
    }

    #[test]
    fn test_amount_field_processing() {
        let config = DataQualityConfigBuilder::new()
            .with_format_variations(true)
            .with_seed(42)
            .build();

        let mut injector = DataQualityInjector::new(config);
        let context = HashMap::new();

        let amount = dec!(1234.56);
        let result = injector.process_amount_field("debit_amount", amount, "JE001", &context);

        assert!(result.is_some());
    }

    #[test]
    fn test_minimal_config() {
        let config = DataQualityConfig::minimal();
        assert!(config.missing_values.global_rate < 0.01);
        assert!(config.typos.char_error_rate < 0.01);
    }

    #[test]
    fn test_high_variation_config() {
        let config = DataQualityConfig::high_variation();
        assert!(config.missing_values.global_rate > 0.01);
        assert!(config.enable_encoding_issues);
    }
}
