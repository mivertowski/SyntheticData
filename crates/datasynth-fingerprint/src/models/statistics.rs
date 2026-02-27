//! Statistical fingerprint models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Statistics fingerprint containing distribution information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsFingerprint {
    /// Statistics for numeric columns, keyed by "table.column".
    pub numeric_columns: HashMap<String, NumericStats>,

    /// Statistics for categorical columns, keyed by "table.column".
    pub categorical_columns: HashMap<String, CategoricalStats>,

    /// Statistics for temporal columns, keyed by "table.column".
    pub temporal_columns: HashMap<String, TemporalStats>,

    /// Global Benford's Law analysis for amount fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benford_analysis: Option<BenfordStats>,

    /// Per-account-class amount statistics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub account_class_stats: Vec<AccountClassStats>,
}

impl StatisticsFingerprint {
    /// Create a new empty statistics fingerprint.
    pub fn new() -> Self {
        Self {
            numeric_columns: HashMap::new(),
            categorical_columns: HashMap::new(),
            temporal_columns: HashMap::new(),
            benford_analysis: None,
            account_class_stats: Vec::new(),
        }
    }

    /// Add numeric statistics for a column.
    pub fn add_numeric(&mut self, table: &str, column: &str, stats: NumericStats) {
        let key = format!("{}.{}", table, column);
        self.numeric_columns.insert(key, stats);
    }

    /// Add categorical statistics for a column.
    pub fn add_categorical(&mut self, table: &str, column: &str, stats: CategoricalStats) {
        let key = format!("{}.{}", table, column);
        self.categorical_columns.insert(key, stats);
    }

    /// Add temporal statistics for a column.
    pub fn add_temporal(&mut self, table: &str, column: &str, stats: TemporalStats) {
        let key = format!("{}.{}", table, column);
        self.temporal_columns.insert(key, stats);
    }

    /// Add account class statistics.
    pub fn add_account_class_stats(&mut self, stats: AccountClassStats) {
        self.account_class_stats.push(stats);
    }
}

impl Default for StatisticsFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a numeric column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericStats {
    /// Number of non-null values.
    pub count: u64,

    /// Minimum value (after privacy processing).
    pub min: f64,

    /// Maximum value (after privacy processing).
    pub max: f64,

    /// Mean value (with DP noise if enabled).
    pub mean: f64,

    /// Standard deviation (with DP noise if enabled).
    pub std_dev: f64,

    /// Percentiles [1, 5, 10, 25, 50, 75, 90, 95, 99] (with DP noise).
    pub percentiles: Percentiles,

    /// Fitted distribution type.
    pub distribution: DistributionType,

    /// Distribution parameters.
    pub distribution_params: DistributionParams,

    /// Proportion of zero values.
    pub zero_rate: f64,

    /// Proportion of negative values.
    pub negative_rate: f64,

    /// First-digit Benford's Law distribution (digits 1-9).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benford_first_digit: Option<[f64; 9]>,
}

impl NumericStats {
    /// Create basic numeric stats.
    pub fn new(count: u64, min: f64, max: f64, mean: f64, std_dev: f64) -> Self {
        Self {
            count,
            min,
            max,
            mean,
            std_dev,
            percentiles: Percentiles::default(),
            distribution: DistributionType::Unknown,
            distribution_params: DistributionParams::empty(),
            zero_rate: 0.0,
            negative_rate: 0.0,
            benford_first_digit: None,
        }
    }

    /// Check if distribution appears to follow Benford's Law.
    pub fn follows_benford(&self) -> bool {
        self.benford_first_digit
            .map(|digits| {
                // Check if digit 1 frequency is close to expected 0.301
                (digits[0] - 0.301).abs() < 0.05
            })
            .unwrap_or(false)
    }
}

/// Percentile values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Percentiles {
    pub p1: f64,
    pub p5: f64,
    pub p10: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

impl Default for Percentiles {
    fn default() -> Self {
        Self {
            p1: 0.0,
            p5: 0.0,
            p10: 0.0,
            p25: 0.0,
            p50: 0.0,
            p75: 0.0,
            p90: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

impl Percentiles {
    /// Create from an array of percentile values.
    pub fn from_array(values: [f64; 9]) -> Self {
        Self {
            p1: values[0],
            p5: values[1],
            p10: values[2],
            p25: values[3],
            p50: values[4],
            p75: values[5],
            p90: values[6],
            p95: values[7],
            p99: values[8],
        }
    }

    /// Convert to array.
    pub fn to_array(&self) -> [f64; 9] {
        [
            self.p1, self.p5, self.p10, self.p25, self.p50, self.p75, self.p90, self.p95, self.p99,
        ]
    }

    /// Get interquartile range.
    pub fn iqr(&self) -> f64 {
        self.p75 - self.p25
    }
}

/// Supported distribution types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionType {
    /// Normal (Gaussian) distribution.
    Normal,
    /// Log-normal distribution.
    LogNormal,
    /// Gamma distribution.
    Gamma,
    /// Exponential distribution.
    Exponential,
    /// Pareto distribution.
    Pareto,
    /// Uniform distribution.
    Uniform,
    /// Point mass (constant value).
    PointMass,
    /// Mixture of distributions.
    Mixture,
    /// Empirical (histogram) distribution.
    Empirical,
    /// Unknown or could not be fitted.
    Unknown,
}

/// Distribution parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionParams {
    /// Primary parameter (e.g., mean for normal, mu for log-normal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param1: Option<f64>,

    /// Secondary parameter (e.g., std_dev for normal, sigma for log-normal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param2: Option<f64>,

    /// Shift parameter for shifted distributions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shift: Option<f64>,

    /// Scale parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,

    /// Histogram bins for empirical distributions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub histogram: Option<Histogram>,

    /// Mixture components for mixture distributions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mixture_components: Option<Vec<MixtureComponent>>,
}

impl DistributionParams {
    /// Create empty parameters.
    pub fn empty() -> Self {
        Self {
            param1: None,
            param2: None,
            shift: None,
            scale: None,
            histogram: None,
            mixture_components: None,
        }
    }

    /// Create normal distribution parameters.
    pub fn normal(mean: f64, std_dev: f64) -> Self {
        Self {
            param1: Some(mean),
            param2: Some(std_dev),
            ..Self::empty()
        }
    }

    /// Create log-normal distribution parameters.
    pub fn log_normal(mu: f64, sigma: f64) -> Self {
        Self {
            param1: Some(mu),
            param2: Some(sigma),
            ..Self::empty()
        }
    }

    /// Create uniform distribution parameters.
    pub fn uniform(min: f64, max: f64) -> Self {
        Self {
            param1: Some(min),
            param2: Some(max),
            ..Self::empty()
        }
    }

    /// Create exponential distribution parameters.
    pub fn exponential(rate: f64) -> Self {
        Self {
            param1: Some(rate),
            ..Self::empty()
        }
    }
}

/// Histogram for empirical distributions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    /// Bin edges (n+1 values for n bins).
    pub bin_edges: Vec<f64>,
    /// Bin counts (normalized to proportions).
    pub bin_weights: Vec<f64>,
}

/// Component of a mixture distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixtureComponent {
    /// Weight of this component (0.0 to 1.0).
    pub weight: f64,
    /// Distribution type of the component.
    pub distribution: DistributionType,
    /// Parameters of the component distribution.
    pub params: DistributionParams,
}

/// Statistics for a categorical column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoricalStats {
    /// Number of non-null values.
    pub count: u64,

    /// Number of unique values (cardinality).
    pub cardinality: u64,

    /// Top-k most frequent values with their frequencies.
    /// Values may be generalized or suppressed for privacy.
    pub top_values: Vec<CategoryFrequency>,

    /// Whether rare values were suppressed for privacy.
    pub rare_values_suppressed: bool,

    /// Number of suppressed rare values.
    pub suppressed_count: u64,

    /// Entropy of the distribution.
    pub entropy: f64,
}

impl CategoricalStats {
    /// Create basic categorical stats.
    pub fn new(count: u64, cardinality: u64) -> Self {
        Self {
            count,
            cardinality,
            top_values: Vec::new(),
            rare_values_suppressed: false,
            suppressed_count: 0,
            entropy: 0.0,
        }
    }
}

/// Frequency entry for a categorical value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryFrequency {
    /// The categorical value (may be hashed for privacy).
    pub value: String,
    /// Frequency as proportion (0.0 to 1.0).
    pub frequency: f64,
    /// Whether this value was generalized (e.g., "USA" -> "North America").
    #[serde(default)]
    pub generalized: bool,
}

impl CategoryFrequency {
    /// Create a new category frequency.
    pub fn new(value: impl Into<String>, frequency: f64) -> Self {
        Self {
            value: value.into(),
            frequency,
            generalized: false,
        }
    }
}

/// Statistics for a temporal column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalStats {
    /// Number of non-null values.
    pub count: u64,

    /// Minimum date/time as string (ISO 8601).
    pub min: String,

    /// Maximum date/time as string (ISO 8601).
    pub max: String,

    /// Day-of-week distribution (Monday=0 to Sunday=6).
    pub day_of_week_distribution: [f64; 7],

    /// Month-of-year distribution (Jan=0 to Dec=11).
    pub month_distribution: [f64; 12],

    /// Hour-of-day distribution (0-23), if timestamp has time component.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hour_distribution: Option<[f64; 24]>,

    /// Whether weekends have significantly different patterns.
    pub weekend_effect: bool,

    /// Whether end-of-month has different patterns.
    pub month_end_effect: bool,

    /// Whether year-end has different patterns.
    pub year_end_effect: bool,

    /// Seasonality strength (0.0 = none, 1.0 = strong).
    pub seasonality_strength: f64,
}

impl TemporalStats {
    /// Create basic temporal stats.
    pub fn new(count: u64, min: String, max: String) -> Self {
        Self {
            count,
            min,
            max,
            day_of_week_distribution: [1.0 / 7.0; 7],
            month_distribution: [1.0 / 12.0; 12],
            hour_distribution: None,
            weekend_effect: false,
            month_end_effect: false,
            year_end_effect: false,
            seasonality_strength: 0.0,
        }
    }
}

/// Global Benford's Law statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenfordStats {
    /// Number of amounts analyzed.
    pub sample_size: u64,

    /// Observed first-digit frequencies.
    pub observed_frequencies: [f64; 9],

    /// Expected Benford frequencies.
    pub expected_frequencies: [f64; 9],

    /// Mean Absolute Deviation from expected.
    pub mad: f64,

    /// Chi-squared statistic.
    pub chi_squared: f64,

    /// P-value from chi-squared test.
    pub p_value: f64,

    /// Whether data conforms to Benford's Law.
    pub conforms: bool,
}

/// Per-account-class statistics for amount fingerprinting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountClassStats {
    /// Account class pattern (e.g., "1XXX" for assets, "4XXX" for revenue).
    pub class_pattern: String,

    /// Human-readable label (e.g., "Assets", "Revenue").
    pub class_label: String,

    /// Numeric statistics for amounts in this account class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeric: Option<NumericStats>,

    /// Row count for this account class.
    pub row_count: u64,

    /// Benford's law analysis for this class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benford_first_digit: Option<[f64; 9]>,
}

impl AccountClassStats {
    /// Create new empty account class stats.
    pub fn new(class_pattern: String, class_label: String) -> Self {
        Self {
            class_pattern,
            class_label,
            numeric: None,
            row_count: 0,
            benford_first_digit: None,
        }
    }
}

/// Round number bias statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundNumberStats {
    /// Proportion ending in .00.
    pub round_hundred_rate: f64,
    /// Proportion ending in 0.
    pub round_ten_rate: f64,
    /// Proportion ending in 5.
    pub round_five_rate: f64,
    /// Proportion with exactly zero decimal places.
    pub whole_number_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_class_stats_creation_and_add() {
        let mut fp = StatisticsFingerprint::new();
        assert!(fp.account_class_stats.is_empty());

        let mut stats = AccountClassStats::new("1XXX".to_string(), "Assets".to_string());
        assert_eq!(stats.class_pattern, "1XXX");
        assert_eq!(stats.class_label, "Assets");
        assert!(stats.numeric.is_none());
        assert_eq!(stats.row_count, 0);
        assert!(stats.benford_first_digit.is_none());

        stats.row_count = 42;
        stats.numeric = Some(NumericStats::new(42, 10.0, 5000.0, 1234.5, 800.0));
        stats.benford_first_digit = Some([
            0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
        ]);

        fp.add_account_class_stats(stats);
        assert_eq!(fp.account_class_stats.len(), 1);
        assert_eq!(fp.account_class_stats[0].class_pattern, "1XXX");
        assert_eq!(fp.account_class_stats[0].row_count, 42);
        assert!(fp.account_class_stats[0].numeric.is_some());
        assert!(fp.account_class_stats[0].benford_first_digit.is_some());
    }

    #[test]
    fn test_account_class_stats_json_roundtrip() {
        let mut stats = AccountClassStats::new("4XXX".to_string(), "Revenue".to_string());
        stats.row_count = 100;
        stats.numeric = Some(NumericStats::new(100, 500.0, 99000.0, 12345.67, 5000.0));

        let json = serde_json::to_string(&stats).expect("serialize");
        let deserialized: AccountClassStats = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.class_pattern, "4XXX");
        assert_eq!(deserialized.class_label, "Revenue");
        assert_eq!(deserialized.row_count, 100);
        assert!(deserialized.numeric.is_some());
        assert!(deserialized.benford_first_digit.is_none());
    }

    #[test]
    fn test_account_class_stats_skip_serializing_when_empty() {
        let fp = StatisticsFingerprint::new();
        let json = serde_json::to_string(&fp).expect("serialize");

        // account_class_stats should NOT appear in JSON when the Vec is empty
        assert!(
            !json.contains("account_class_stats"),
            "empty account_class_stats should be skipped in serialization"
        );

        // Now add one entry and confirm it appears
        let mut fp2 = StatisticsFingerprint::new();
        fp2.add_account_class_stats(AccountClassStats::new(
            "5XXX".to_string(),
            "Expenses".to_string(),
        ));
        let json2 = serde_json::to_string(&fp2).expect("serialize");
        assert!(
            json2.contains("account_class_stats"),
            "non-empty account_class_stats should appear in serialization"
        );
    }

    #[test]
    fn test_statistics_fingerprint_json_roundtrip_with_account_class_stats() {
        let mut fp = StatisticsFingerprint::new();
        fp.add_account_class_stats(AccountClassStats::new(
            "1XXX".to_string(),
            "Assets".to_string(),
        ));
        fp.add_account_class_stats(AccountClassStats::new(
            "4XXX".to_string(),
            "Revenue".to_string(),
        ));

        let json = serde_json::to_string(&fp).expect("serialize");
        let deserialized: StatisticsFingerprint = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.account_class_stats.len(), 2);
        assert_eq!(deserialized.account_class_stats[0].class_pattern, "1XXX");
        assert_eq!(deserialized.account_class_stats[1].class_label, "Revenue");
    }

    #[test]
    fn test_deserialize_without_account_class_stats_field() {
        // Simulate JSON from an older version that lacks the field entirely.
        // The #[serde(default)] on the field should populate an empty Vec.
        let json = r#"{
            "numeric_columns": {},
            "categorical_columns": {},
            "temporal_columns": {}
        }"#;
        let fp: StatisticsFingerprint = serde_json::from_str(json).expect("deserialize");
        assert!(fp.account_class_stats.is_empty());
    }
}
