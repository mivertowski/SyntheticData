//! Temporal clustering for anomaly injection.
//!
//! Defines time-based patterns where anomaly rates increase during
//! specific periods (month-end, quarter-end, year-end, post-holiday).

use chrono::{Datelike, NaiveDate, Weekday};
use rand::Rng;
use serde::{Deserialize, Serialize};

use datasynth_core::models::AnomalyType;

/// Temporal window for anomaly clustering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalWindow {
    /// Last N business days of the month.
    LastBusinessDays(u32),
    /// First N business days after a holiday.
    FirstBusinessDaysAfterHoliday(u32),
    /// Last week of quarter.
    LastWeekOfQuarter,
    /// December (year-end).
    December,
    /// Last N days of year.
    LastDaysOfYear(u32),
    /// First N days of month.
    FirstDaysOfMonth(u32),
    /// Custom date range.
    Custom {
        /// Start day of month (1-31).
        start_day: u32,
        /// End day of month (1-31).
        end_day: u32,
        /// Months this applies to (1-12).
        months: Vec<u32>,
    },
}

impl TemporalWindow {
    /// Checks if a date falls within this window.
    pub fn contains(&self, date: NaiveDate) -> bool {
        match self {
            TemporalWindow::LastBusinessDays(n) => {
                // Check if in last N business days of month
                let last_day = Self::last_day_of_month(date);
                let days_until_end = (last_day - date.day()) as i32;

                // Count business days until end
                let business_days_until_end = (0..=days_until_end)
                    .filter(|d| {
                        let check_date = date + chrono::Duration::days(*d as i64);
                        !Self::is_weekend(check_date)
                    })
                    .count() as u32;

                business_days_until_end <= *n
            }
            TemporalWindow::FirstBusinessDaysAfterHoliday(n) => {
                // Simplified: check if in first N days of month after major holidays
                let is_post_holiday_month = matches!(date.month(), 1 | 7 | 12); // Jan, Jul, Dec
                date.day() <= *n && is_post_holiday_month
            }
            TemporalWindow::LastWeekOfQuarter => {
                let is_quarter_end_month = matches!(date.month(), 3 | 6 | 9 | 12);
                let last_day = Self::last_day_of_month(date);
                is_quarter_end_month && date.day() > last_day - 7
            }
            TemporalWindow::December => date.month() == 12,
            TemporalWindow::LastDaysOfYear(n) => date.month() == 12 && date.day() > 31 - *n,
            TemporalWindow::FirstDaysOfMonth(n) => date.day() <= *n,
            TemporalWindow::Custom {
                start_day,
                end_day,
                months,
            } => {
                months.contains(&date.month()) && date.day() >= *start_day && date.day() <= *end_day
            }
        }
    }

    /// Helper to get last day of month.
    fn last_day_of_month(date: NaiveDate) -> u32 {
        let (year, month) = (date.year(), date.month());
        let next_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        };
        next_month
            .map(|d| (d - chrono::Duration::days(1)).day())
            .unwrap_or(31)
    }

    /// Helper to check if date is weekend.
    fn is_weekend(date: NaiveDate) -> bool {
        matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
    }

    /// Returns a descriptive name for this window.
    pub fn name(&self) -> String {
        match self {
            TemporalWindow::LastBusinessDays(n) => format!("last_{}_business_days", n),
            TemporalWindow::FirstBusinessDaysAfterHoliday(n) => {
                format!("first_{}_days_after_holiday", n)
            }
            TemporalWindow::LastWeekOfQuarter => "last_week_of_quarter".to_string(),
            TemporalWindow::December => "december".to_string(),
            TemporalWindow::LastDaysOfYear(n) => format!("last_{}_days_of_year", n),
            TemporalWindow::FirstDaysOfMonth(n) => format!("first_{}_days_of_month", n),
            TemporalWindow::Custom { .. } => "custom".to_string(),
        }
    }
}

/// A temporal anomaly cluster definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnomalyCluster {
    /// Name of the cluster.
    pub name: String,
    /// Description.
    pub description: String,
    /// Temporal window for this cluster.
    pub window: TemporalWindow,
    /// Anomaly types that spike during this window.
    pub anomaly_types: Vec<AnomalyType>,
    /// Rate multiplier during this window.
    pub rate_multiplier: f64,
    /// Whether this cluster is enabled.
    pub enabled: bool,
}

impl TemporalAnomalyCluster {
    /// Creates a new temporal anomaly cluster.
    pub fn new(name: impl Into<String>, window: TemporalWindow, rate_multiplier: f64) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            window,
            anomaly_types: Vec::new(),
            rate_multiplier: rate_multiplier.max(1.0),
            enabled: true,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Adds an anomaly type to this cluster.
    pub fn with_anomaly_type(mut self, anomaly_type: AnomalyType) -> Self {
        self.anomaly_types.push(anomaly_type);
        self
    }

    /// Sets multiple anomaly types.
    pub fn with_anomaly_types(mut self, types: Vec<AnomalyType>) -> Self {
        self.anomaly_types = types;
        self
    }

    /// Sets whether enabled.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Checks if this cluster applies to a date.
    pub fn applies_to(&self, date: NaiveDate) -> bool {
        self.enabled && self.window.contains(date)
    }

    /// Gets the rate multiplier for a given anomaly type and date.
    pub fn get_multiplier(&self, date: NaiveDate, anomaly_type: &AnomalyType) -> f64 {
        if self.applies_to(date)
            && (self.anomaly_types.is_empty() || self.anomaly_types.contains(anomaly_type))
        {
            self.rate_multiplier
        } else {
            1.0
        }
    }
}

/// Generator for temporal clustering.
pub struct TemporalClusterGenerator {
    /// Registered clusters.
    clusters: Vec<TemporalAnomalyCluster>,
}

impl Default for TemporalClusterGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalClusterGenerator {
    /// Creates a new generator with default clusters.
    pub fn new() -> Self {
        Self {
            clusters: Self::default_clusters(),
        }
    }

    /// Creates default temporal clusters.
    fn default_clusters() -> Vec<TemporalAnomalyCluster> {
        use datasynth_core::models::{ErrorType, FraudType, ProcessIssueType};

        vec![
            // Period-end error spike
            TemporalAnomalyCluster::new(
                "period_end_errors",
                TemporalWindow::LastBusinessDays(5),
                2.5,
            )
            .with_description("Errors increase during period close")
            .with_anomaly_types(vec![
                AnomalyType::Error(ErrorType::WrongPeriod),
                AnomalyType::Error(ErrorType::DuplicateEntry),
                AnomalyType::ProcessIssue(ProcessIssueType::LatePosting),
                AnomalyType::ProcessIssue(ProcessIssueType::RushedPeriodEnd),
            ]),
            // Quarter-end pressure
            TemporalAnomalyCluster::new(
                "quarter_end_pressure",
                TemporalWindow::LastWeekOfQuarter,
                1.5,
            )
            .with_description("Fraud and process issues spike at quarter end")
            .with_anomaly_types(vec![
                AnomalyType::Fraud(FraudType::RevenueManipulation),
                AnomalyType::Fraud(FraudType::ChannelStuffing),
                AnomalyType::ProcessIssue(ProcessIssueType::SkippedApproval),
                AnomalyType::ProcessIssue(ProcessIssueType::ManualOverride),
            ]),
            // Year-end spike
            TemporalAnomalyCluster::new("year_end_spike", TemporalWindow::LastDaysOfYear(10), 3.0)
                .with_description("All anomalies spike at year end")
                .with_anomaly_types(vec![
                    // Revenue manipulation
                    AnomalyType::Fraud(FraudType::RevenueManipulation),
                    AnomalyType::Fraud(FraudType::PrematureRevenue),
                    AnomalyType::Fraud(FraudType::ChannelStuffing),
                    // Expense manipulation
                    AnomalyType::Fraud(FraudType::ImproperCapitalization),
                    AnomalyType::Fraud(FraudType::ReserveManipulation),
                    // Timing errors
                    AnomalyType::Error(ErrorType::WrongPeriod),
                    AnomalyType::Error(ErrorType::CutoffError),
                ]),
            // Post-holiday errors
            TemporalAnomalyCluster::new(
                "post_holiday_errors",
                TemporalWindow::FirstBusinessDaysAfterHoliday(3),
                1.8,
            )
            .with_description("Errors increase after holidays")
            .with_anomaly_types(vec![
                AnomalyType::Error(ErrorType::BackdatedEntry),
                AnomalyType::Error(ErrorType::MissingField),
                AnomalyType::ProcessIssue(ProcessIssueType::LatePosting),
            ]),
            // Month-start reconciliation
            TemporalAnomalyCluster::new(
                "month_start_reconciliation",
                TemporalWindow::FirstDaysOfMonth(5),
                1.3,
            )
            .with_description("Reconciliation-related issues at month start")
            .with_anomaly_types(vec![
                AnomalyType::Error(ErrorType::DuplicateEntry),
                AnomalyType::Error(ErrorType::ReversedAmount),
            ]),
        ]
    }

    /// Adds a custom cluster.
    pub fn add_cluster(&mut self, cluster: TemporalAnomalyCluster) {
        self.clusters.push(cluster);
    }

    /// Gets the combined rate multiplier for a date and anomaly type.
    pub fn get_multiplier(&self, date: NaiveDate, anomaly_type: &AnomalyType) -> f64 {
        // Use the maximum multiplier from all applicable clusters
        self.clusters
            .iter()
            .map(|c| c.get_multiplier(date, anomaly_type))
            .fold(1.0, f64::max)
    }

    /// Gets active clusters for a date.
    pub fn get_active_clusters(&self, date: NaiveDate) -> Vec<&TemporalAnomalyCluster> {
        self.clusters
            .iter()
            .filter(|c| c.applies_to(date))
            .collect()
    }

    /// Returns all clusters.
    pub fn clusters(&self) -> &[TemporalAnomalyCluster] {
        &self.clusters
    }

    /// Enables or disables a cluster by name.
    pub fn set_cluster_enabled(&mut self, name: &str, enabled: bool) {
        for cluster in &mut self.clusters {
            if cluster.name == name {
                cluster.enabled = enabled;
                break;
            }
        }
    }

    /// Determines if an anomaly should be injected based on temporal patterns.
    pub fn should_inject<R: Rng>(
        &self,
        base_rate: f64,
        date: NaiveDate,
        anomaly_type: &AnomalyType,
        rng: &mut R,
    ) -> bool {
        let multiplier = self.get_multiplier(date, anomaly_type);
        let adjusted_rate = (base_rate * multiplier).min(1.0);
        rng.gen::<f64>() < adjusted_rate
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::ErrorType;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_temporal_window_last_business_days() {
        let window = TemporalWindow::LastBusinessDays(5);

        // June 30, 2024 is a Sunday - last business day is June 28
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 28).unwrap();
        assert!(window.contains(month_end));

        // June 15 should not be in last 5 business days
        let mid_month = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert!(!window.contains(mid_month));
    }

    #[test]
    fn test_temporal_window_last_week_of_quarter() {
        let window = TemporalWindow::LastWeekOfQuarter;

        // March 28 should be in last week of Q1
        let q1_end = NaiveDate::from_ymd_opt(2024, 3, 28).unwrap();
        assert!(window.contains(q1_end));

        // March 15 should not be
        let mid_march = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        assert!(!window.contains(mid_march));

        // April 28 should not be (not quarter end month)
        let april = NaiveDate::from_ymd_opt(2024, 4, 28).unwrap();
        assert!(!window.contains(april));
    }

    #[test]
    fn test_temporal_window_december() {
        let window = TemporalWindow::December;

        let dec = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        assert!(window.contains(dec));

        let nov = NaiveDate::from_ymd_opt(2024, 11, 15).unwrap();
        assert!(!window.contains(nov));
    }

    #[test]
    fn test_temporal_anomaly_cluster() {
        let cluster =
            TemporalAnomalyCluster::new("test_cluster", TemporalWindow::LastBusinessDays(5), 2.5)
                .with_anomaly_type(AnomalyType::Error(ErrorType::WrongPeriod));

        assert_eq!(cluster.name, "test_cluster");
        assert!((cluster.rate_multiplier - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_temporal_cluster_generator() {
        let generator = TemporalClusterGenerator::new();
        assert!(!generator.clusters().is_empty());

        // Year-end date should have high multiplier
        let year_end = NaiveDate::from_ymd_opt(2024, 12, 28).unwrap();
        let multiplier = generator.get_multiplier(
            year_end,
            &AnomalyType::Fraud(datasynth_core::models::FraudType::RevenueManipulation),
        );
        assert!(multiplier > 1.0);

        // Mid-year date should have normal multiplier
        let mid_year = NaiveDate::from_ymd_opt(2024, 7, 15).unwrap();
        let multiplier = generator.get_multiplier(
            mid_year,
            &AnomalyType::Fraud(datasynth_core::models::FraudType::DuplicatePayment),
        );
        assert!((multiplier - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_should_inject() {
        let generator = TemporalClusterGenerator::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Count injections at year end vs mid year
        let year_end = NaiveDate::from_ymd_opt(2024, 12, 28).unwrap();
        let mid_year = NaiveDate::from_ymd_opt(2024, 7, 15).unwrap();
        let anomaly_type = AnomalyType::Error(ErrorType::WrongPeriod);
        let base_rate = 0.1;

        let mut year_end_count = 0;
        let mut mid_year_count = 0;

        for _ in 0..1000 {
            if generator.should_inject(base_rate, year_end, &anomaly_type, &mut rng) {
                year_end_count += 1;
            }
            if generator.should_inject(base_rate, mid_year, &anomaly_type, &mut rng) {
                mid_year_count += 1;
            }
        }

        // Year-end should have more injections due to multiplier
        assert!(year_end_count > mid_year_count);
    }
}
