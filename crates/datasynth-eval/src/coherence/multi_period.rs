use serde::{Deserialize, Serialize};

/// Data for a single fiscal period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodData {
    pub period_index: usize,
    pub opening_balance: f64,
    pub closing_balance: f64,
    pub total_debits: f64,
    pub total_credits: f64,
    pub transaction_count: usize,
    pub anomaly_count: usize,
}

/// Configurable thresholds for multi-period coherence checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPeriodThresholds {
    /// Minimum acceptable balance continuity rate (opening[i] == closing[i-1]).
    /// Default: 1.0 (exact match).
    pub min_balance_continuity: f64,
    /// Maximum coefficient of variation for transaction volumes across periods.
    /// Default: 0.50.
    pub max_volume_variance_cv: f64,
    /// Minimum fraction of periods that must have activity (transaction_count > 0).
    /// Default: 0.90.
    pub min_periods_with_activity: f64,
}

impl Default for MultiPeriodThresholds {
    fn default() -> Self {
        Self {
            min_balance_continuity: 1.0,
            max_volume_variance_cv: 0.50,
            min_periods_with_activity: 0.90,
        }
    }
}

/// Result of multi-period coherence analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPeriodAnalysis {
    /// Fraction of consecutive period pairs where opening_balance[i] == closing_balance[i-1].
    pub balance_continuity_rate: f64,
    /// Coefficient of variation of transaction counts across periods.
    pub volume_variance_cv: f64,
    /// Fraction of periods with at least one transaction.
    pub periods_with_activity_rate: f64,
    /// Total number of periods analyzed.
    pub total_periods: usize,
    /// Whether all thresholds were met.
    pub passes: bool,
    /// Human-readable issues found.
    pub issues: Vec<String>,
}

/// Analyzes multi-period coherence.
pub struct MultiPeriodAnalyzer {
    thresholds: MultiPeriodThresholds,
}

impl MultiPeriodAnalyzer {
    pub fn new(thresholds: MultiPeriodThresholds) -> Self {
        Self { thresholds }
    }

    pub fn with_defaults() -> Self {
        Self::new(MultiPeriodThresholds::default())
    }

    pub fn analyze(&self, periods: &[PeriodData]) -> MultiPeriodAnalysis {
        let total_periods = periods.len();
        let mut issues = Vec::new();

        if total_periods == 0 {
            return MultiPeriodAnalysis {
                balance_continuity_rate: 0.0,
                volume_variance_cv: 0.0,
                periods_with_activity_rate: 0.0,
                total_periods: 0,
                passes: false,
                issues: vec!["No periods provided".into()],
            };
        }

        // Balance continuity: check opening[i] == closing[i-1] for consecutive pairs
        let continuity_pairs = total_periods.saturating_sub(1);
        let mut continuity_matches = 0usize;
        for i in 1..total_periods {
            let prev_closing = periods[i - 1].closing_balance;
            let curr_opening = periods[i].opening_balance;
            if (prev_closing - curr_opening).abs() < 1e-6 {
                continuity_matches += 1;
            } else {
                issues.push(format!(
                    "Period {} opening ({:.2}) != period {} closing ({:.2})",
                    i,
                    curr_opening,
                    i - 1,
                    prev_closing
                ));
            }
        }
        let balance_continuity_rate = if continuity_pairs > 0 {
            continuity_matches as f64 / continuity_pairs as f64
        } else {
            1.0 // Single period is trivially continuous
        };

        // Volume variance: CV of transaction counts
        let counts: Vec<f64> = periods.iter().map(|p| p.transaction_count as f64).collect();
        let mean = counts.iter().sum::<f64>() / counts.len() as f64;
        let volume_variance_cv = if mean > 0.0 {
            let variance =
                counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / counts.len() as f64;
            variance.sqrt() / mean
        } else {
            0.0
        };

        // Activity rate: fraction of periods with transactions
        let active_periods = periods.iter().filter(|p| p.transaction_count > 0).count();
        let periods_with_activity_rate = active_periods as f64 / total_periods as f64;

        // Check thresholds
        if balance_continuity_rate < self.thresholds.min_balance_continuity {
            issues.push(format!(
                "Balance continuity {:.2} < threshold {:.2}",
                balance_continuity_rate, self.thresholds.min_balance_continuity
            ));
        }
        if volume_variance_cv > self.thresholds.max_volume_variance_cv {
            issues.push(format!(
                "Volume CV {:.3} > threshold {:.3}",
                volume_variance_cv, self.thresholds.max_volume_variance_cv
            ));
        }
        if periods_with_activity_rate < self.thresholds.min_periods_with_activity {
            issues.push(format!(
                "Activity rate {:.2} < threshold {:.2}",
                periods_with_activity_rate, self.thresholds.min_periods_with_activity
            ));
        }

        let passes = balance_continuity_rate >= self.thresholds.min_balance_continuity
            && volume_variance_cv <= self.thresholds.max_volume_variance_cv
            && periods_with_activity_rate >= self.thresholds.min_periods_with_activity;

        MultiPeriodAnalysis {
            balance_continuity_rate,
            volume_variance_cv,
            periods_with_activity_rate,
            total_periods,
            passes,
            issues,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_periods(count: usize) -> Vec<PeriodData> {
        let mut periods = Vec::new();
        let mut balance = 1000.0;
        for i in 0..count {
            let debits = 500.0 + (i as f64) * 10.0;
            let credits = 480.0 + (i as f64) * 10.0;
            let closing = balance + debits - credits;
            periods.push(PeriodData {
                period_index: i,
                opening_balance: balance,
                closing_balance: closing,
                total_debits: debits,
                total_credits: credits,
                transaction_count: 100 + i * 5,
                anomaly_count: 2,
            });
            balance = closing;
        }
        periods
    }

    #[test]
    fn test_multi_period_coherent_data_passes() {
        let analyzer = MultiPeriodAnalyzer::with_defaults();
        let periods = make_periods(12);
        let result = analyzer.analyze(&periods);
        assert!(result.passes, "issues: {:?}", result.issues);
        assert_eq!(result.balance_continuity_rate, 1.0);
        assert_eq!(result.total_periods, 12);
        assert_eq!(result.periods_with_activity_rate, 1.0);
    }

    #[test]
    fn test_balance_discontinuity_detected() {
        let analyzer = MultiPeriodAnalyzer::with_defaults();
        let mut periods = make_periods(4);
        // Break continuity at period 2
        periods[2].opening_balance = 9999.0;
        let result = analyzer.analyze(&periods);
        assert!(!result.passes);
        assert!(result.balance_continuity_rate < 1.0);
        assert!(result.issues.iter().any(|i| i.contains("opening")));
    }

    #[test]
    fn test_inactive_periods_detected() {
        let analyzer = MultiPeriodAnalyzer::with_defaults();
        let mut periods = make_periods(10);
        // Make 3 periods inactive
        periods[3].transaction_count = 0;
        periods[5].transaction_count = 0;
        periods[7].transaction_count = 0;
        let result = analyzer.analyze(&periods);
        assert_eq!(result.periods_with_activity_rate, 0.7);
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("Activity rate")));
    }

    #[test]
    fn test_high_volume_variance_detected() {
        let analyzer = MultiPeriodAnalyzer::with_defaults();
        let mut periods = make_periods(6);
        // Make highly variable volumes
        periods[0].transaction_count = 10;
        periods[1].transaction_count = 1000;
        periods[2].transaction_count = 5;
        periods[3].transaction_count = 500;
        periods[4].transaction_count = 20;
        periods[5].transaction_count = 800;
        let result = analyzer.analyze(&periods);
        assert!(result.volume_variance_cv > 0.5);
        assert!(!result.passes);
    }

    #[test]
    fn test_single_period_trivially_passes() {
        let analyzer = MultiPeriodAnalyzer::with_defaults();
        let periods = make_periods(1);
        let result = analyzer.analyze(&periods);
        assert!(result.passes);
        assert_eq!(result.balance_continuity_rate, 1.0);
        assert_eq!(result.total_periods, 1);
    }

    #[test]
    fn test_empty_periods_fails() {
        let analyzer = MultiPeriodAnalyzer::with_defaults();
        let result = analyzer.analyze(&[]);
        assert!(!result.passes);
        assert_eq!(result.total_periods, 0);
    }

    #[test]
    fn test_custom_thresholds() {
        let thresholds = MultiPeriodThresholds {
            min_balance_continuity: 0.5,
            max_volume_variance_cv: 2.0,
            min_periods_with_activity: 0.5,
        };
        let analyzer = MultiPeriodAnalyzer::new(thresholds);
        let mut periods = make_periods(4);
        // Break one continuity (out of 3 pairs = 66% continuity)
        periods[2].opening_balance = 9999.0;
        // Make 2 of 4 periods inactive (50% activity)
        periods[0].transaction_count = 0;
        periods[3].transaction_count = 0;
        let result = analyzer.analyze(&periods);
        // With relaxed thresholds, this should pass
        assert!(result.passes, "issues: {:?}", result.issues);
    }
}
