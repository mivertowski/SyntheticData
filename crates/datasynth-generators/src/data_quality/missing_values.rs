//! Missing value injection for data quality simulation.
//!
//! Simulates realistic missing data patterns including:
//! - Random missing values (MCAR - Missing Completely At Random)
//! - Conditional missing values (MAR - Missing At Random)
//! - Systematic missing values (MNAR - Missing Not At Random)

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Strategy for missing value injection.
#[derive(Debug, Clone)]
pub enum MissingValueStrategy {
    /// Missing Completely At Random - each value has equal probability of being missing.
    MCAR {
        /// Probability of a value being missing (0.0 - 1.0).
        probability: f64,
    },
    /// Missing At Random - probability depends on other observed values.
    MAR {
        /// Base probability.
        base_probability: f64,
        /// Conditions that increase probability.
        conditions: Vec<MissingCondition>,
    },
    /// Missing Not At Random - probability depends on the value itself.
    MNAR {
        /// Missing probability for specific value ranges/patterns.
        value_patterns: Vec<MissingPattern>,
    },
    /// Systematic missing - entire fields missing for certain records.
    Systematic {
        /// Fields that are systematically missing together.
        field_groups: Vec<Vec<String>>,
        /// Probability of group being missing.
        probability: f64,
    },
}

impl Default for MissingValueStrategy {
    fn default() -> Self {
        MissingValueStrategy::MCAR { probability: 0.01 }
    }
}

/// Condition for MAR missing values.
#[derive(Debug, Clone)]
pub struct MissingCondition {
    /// Field to check.
    pub field: String,
    /// Condition type.
    pub condition_type: ConditionType,
    /// Probability multiplier when condition is met.
    pub multiplier: f64,
}

/// Type of condition for missing values.
#[derive(Debug, Clone)]
pub enum ConditionType {
    /// Field equals specific value.
    Equals(String),
    /// Field contains substring.
    Contains(String),
    /// Field is empty.
    IsEmpty,
    /// Field matches pattern.
    Matches(String),
    /// Numeric field greater than threshold.
    GreaterThan(f64),
    /// Numeric field less than threshold.
    LessThan(f64),
}

/// Pattern for MNAR missing values.
#[derive(Debug, Clone)]
pub struct MissingPattern {
    /// Description of the pattern.
    pub description: String,
    /// Field to check.
    pub field: String,
    /// Pattern type.
    pub pattern_type: PatternType,
    /// Probability when pattern matches.
    pub probability: f64,
}

/// Type of pattern for MNAR.
#[derive(Debug, Clone)]
pub enum PatternType {
    /// High values tend to be missing.
    HighValues { threshold: f64 },
    /// Low values tend to be missing.
    LowValues { threshold: f64 },
    /// Extreme values (outliers) tend to be missing.
    ExtremeValues { low: f64, high: f64 },
    /// Sensitive values tend to be missing.
    SensitivePatterns { patterns: Vec<String> },
}

/// Configuration for missing values by field.
#[derive(Debug, Clone)]
pub struct MissingValueConfig {
    /// Global missing rate (default for fields not specified).
    pub global_rate: f64,
    /// Field-specific missing rates.
    pub field_rates: HashMap<String, f64>,
    /// Fields that should never be missing (required fields).
    pub required_fields: HashSet<String>,
    /// Strategy for missing value injection.
    pub strategy: MissingValueStrategy,
    /// Whether to track missing value statistics.
    pub track_statistics: bool,
}

impl Default for MissingValueConfig {
    fn default() -> Self {
        let mut required_fields = HashSet::new();
        // Common required fields in accounting data
        required_fields.insert("document_number".to_string());
        required_fields.insert("company_code".to_string());
        required_fields.insert("posting_date".to_string());
        required_fields.insert("account_code".to_string());

        Self {
            global_rate: 0.01,
            field_rates: HashMap::new(),
            required_fields,
            strategy: MissingValueStrategy::default(),
            track_statistics: true,
        }
    }
}

impl MissingValueConfig {
    /// Creates a configuration with specific field rates.
    pub fn with_field_rates(mut self, rates: HashMap<String, f64>) -> Self {
        self.field_rates = rates;
        self
    }

    /// Adds a required field.
    pub fn with_required_field(mut self, field: &str) -> Self {
        self.required_fields.insert(field.to_string());
        self
    }

    /// Sets the strategy.
    pub fn with_strategy(mut self, strategy: MissingValueStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Gets the missing rate for a specific field.
    pub fn get_rate(&self, field: &str) -> f64 {
        if self.required_fields.contains(field) {
            return 0.0;
        }
        *self.field_rates.get(field).unwrap_or(&self.global_rate)
    }
}

/// Statistics about missing values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MissingValueStats {
    /// Total fields processed.
    pub total_fields: usize,
    /// Total missing values injected.
    pub total_missing: usize,
    /// Missing count by field.
    pub by_field: HashMap<String, usize>,
    /// Records with any missing value.
    pub records_with_missing: usize,
    /// Total records processed.
    pub total_records: usize,
}

impl MissingValueStats {
    /// Returns the overall missing rate.
    pub fn overall_rate(&self) -> f64 {
        if self.total_fields == 0 {
            0.0
        } else {
            self.total_missing as f64 / self.total_fields as f64
        }
    }

    /// Returns the rate for a specific field.
    pub fn field_rate(&self, field: &str, total_records: usize) -> f64 {
        if total_records == 0 {
            return 0.0;
        }
        *self.by_field.get(field).unwrap_or(&0) as f64 / total_records as f64
    }
}

/// Missing value injector.
pub struct MissingValueInjector {
    config: MissingValueConfig,
    stats: MissingValueStats,
}

impl MissingValueInjector {
    /// Creates a new missing value injector.
    pub fn new(config: MissingValueConfig) -> Self {
        Self {
            config,
            stats: MissingValueStats::default(),
        }
    }

    /// Determines if a value should be made missing.
    pub fn should_be_missing<R: Rng>(
        &mut self,
        field: &str,
        value: Option<&str>,
        context: &HashMap<String, String>,
        rng: &mut R,
    ) -> bool {
        // Never make required fields missing
        if self.config.required_fields.contains(field) {
            return false;
        }

        let probability = self.calculate_probability(field, value, context);

        if self.config.track_statistics {
            self.stats.total_fields += 1;
        }

        let is_missing = rng.gen::<f64>() < probability;

        if is_missing && self.config.track_statistics {
            self.stats.total_missing += 1;
            *self.stats.by_field.entry(field.to_string()).or_insert(0) += 1;
        }

        is_missing
    }

    /// Calculates the missing probability based on strategy.
    fn calculate_probability(
        &self,
        field: &str,
        value: Option<&str>,
        context: &HashMap<String, String>,
    ) -> f64 {
        match &self.config.strategy {
            MissingValueStrategy::MCAR { probability } => {
                // Use field-specific rate if available
                let base = self.config.get_rate(field);
                if base > 0.0 {
                    base
                } else {
                    *probability
                }
            }
            MissingValueStrategy::MAR {
                base_probability,
                conditions,
            } => {
                let mut prob = *base_probability;

                for condition in conditions {
                    if let Some(field_value) = context.get(&condition.field) {
                        if self.check_condition(&condition.condition_type, field_value) {
                            prob *= condition.multiplier;
                        }
                    }
                }

                prob.min(1.0)
            }
            MissingValueStrategy::MNAR { value_patterns } => {
                if let Some(val) = value {
                    for pattern in value_patterns {
                        if pattern.field == field
                            && self.check_value_pattern(&pattern.pattern_type, val)
                        {
                            return pattern.probability;
                        }
                    }
                }
                self.config.get_rate(field)
            }
            MissingValueStrategy::Systematic {
                field_groups,
                probability,
            } => {
                // Check if field is in any group
                for group in field_groups {
                    if group.contains(&field.to_string()) {
                        return *probability;
                    }
                }
                self.config.get_rate(field)
            }
        }
    }

    /// Checks if a condition is met.
    fn check_condition(&self, condition: &ConditionType, value: &str) -> bool {
        match condition {
            ConditionType::Equals(expected) => value == expected,
            ConditionType::Contains(substring) => value.contains(substring),
            ConditionType::IsEmpty => value.is_empty(),
            ConditionType::Matches(pattern) => {
                // Simple pattern matching (could use regex)
                value.contains(pattern)
            }
            ConditionType::GreaterThan(threshold) => value
                .parse::<f64>()
                .map(|v| v > *threshold)
                .unwrap_or(false),
            ConditionType::LessThan(threshold) => value
                .parse::<f64>()
                .map(|v| v < *threshold)
                .unwrap_or(false),
        }
    }

    /// Checks if a value matches an MNAR pattern.
    fn check_value_pattern(&self, pattern: &PatternType, value: &str) -> bool {
        match pattern {
            PatternType::HighValues { threshold } => value
                .parse::<f64>()
                .map(|v| v > *threshold)
                .unwrap_or(false),
            PatternType::LowValues { threshold } => value
                .parse::<f64>()
                .map(|v| v < *threshold)
                .unwrap_or(false),
            PatternType::ExtremeValues { low, high } => value
                .parse::<f64>()
                .map(|v| v < *low || v > *high)
                .unwrap_or(false),
            PatternType::SensitivePatterns { patterns } => {
                patterns.iter().any(|p| value.contains(p))
            }
        }
    }

    /// Records that a record was processed.
    pub fn record_processed(&mut self, had_missing: bool) {
        if self.config.track_statistics {
            self.stats.total_records += 1;
            if had_missing {
                self.stats.records_with_missing += 1;
            }
        }
    }

    /// Returns the statistics.
    pub fn stats(&self) -> &MissingValueStats {
        &self.stats
    }

    /// Resets statistics.
    pub fn reset_stats(&mut self) {
        self.stats = MissingValueStats::default();
    }
}

/// Represents a missing value placeholder.
#[derive(Debug, Clone, PartialEq)]
pub enum MissingValue {
    /// Standard null/None.
    Null,
    /// Empty string.
    Empty,
    /// Special marker string.
    Marker(String),
    /// NA string.
    NA,
    /// Dash placeholder.
    Dash,
    /// Question mark.
    Unknown,
}

impl MissingValue {
    /// Converts to string representation.
    pub fn to_string_value(&self) -> String {
        match self {
            MissingValue::Null => String::new(),
            MissingValue::Empty => String::new(),
            MissingValue::Marker(s) => s.clone(),
            MissingValue::NA => "N/A".to_string(),
            MissingValue::Dash => "-".to_string(),
            MissingValue::Unknown => "?".to_string(),
        }
    }

    /// Returns common missing value representations.
    pub fn common_representations() -> Vec<Self> {
        vec![
            MissingValue::Null,
            MissingValue::Empty,
            MissingValue::NA,
            MissingValue::Marker("NULL".to_string()),
            MissingValue::Marker("NONE".to_string()),
            MissingValue::Marker("#N/A".to_string()),
            MissingValue::Dash,
            MissingValue::Unknown,
        ]
    }
}

/// Selects a random missing value representation.
pub fn random_missing_representation<R: Rng>(rng: &mut R) -> MissingValue {
    let representations = MissingValue::common_representations();
    representations[rng.gen_range(0..representations.len())].clone()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_mcar_strategy() {
        let config = MissingValueConfig {
            global_rate: 0.5, // High rate for testing
            strategy: MissingValueStrategy::MCAR { probability: 0.5 },
            ..Default::default()
        };

        let mut injector = MissingValueInjector::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let context = HashMap::new();

        let mut missing_count = 0;
        for _ in 0..1000 {
            if injector.should_be_missing("description", Some("test"), &context, &mut rng) {
                missing_count += 1;
            }
        }

        // Should be roughly 50%
        assert!(missing_count > 400 && missing_count < 600);
    }

    #[test]
    fn test_required_fields() {
        let config = MissingValueConfig {
            global_rate: 1.0, // 100% rate
            ..Default::default()
        };

        let mut injector = MissingValueInjector::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let context = HashMap::new();

        // Required field should never be missing
        assert!(!injector.should_be_missing("document_number", Some("JE001"), &context, &mut rng));

        // Non-required field should be missing at 100% rate
        assert!(injector.should_be_missing("description", Some("test"), &context, &mut rng));
    }

    #[test]
    fn test_field_specific_rates() {
        let mut field_rates = HashMap::new();
        field_rates.insert("description".to_string(), 0.0);
        field_rates.insert("cost_center".to_string(), 1.0);

        let config = MissingValueConfig::default().with_field_rates(field_rates);

        let mut injector = MissingValueInjector::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let context = HashMap::new();

        // Description should never be missing (0% rate)
        assert!(!injector.should_be_missing("description", Some("test"), &context, &mut rng));

        // Cost center should always be missing (100% rate)
        assert!(injector.should_be_missing("cost_center", Some("CC001"), &context, &mut rng));
    }

    #[test]
    fn test_statistics() {
        let config = MissingValueConfig {
            global_rate: 0.5,
            track_statistics: true,
            ..Default::default()
        };

        let mut injector = MissingValueInjector::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let context = HashMap::new();

        for _ in 0..100 {
            injector.should_be_missing("description", Some("test"), &context, &mut rng);
        }

        assert_eq!(injector.stats().total_fields, 100);
        assert!(injector.stats().total_missing > 0);
    }
}
