//! Near-miss pattern generator.
//!
//! Generates near-miss cases that appear suspicious but are actually
//! legitimate, useful for training models to reduce false positives.

use chrono::{Datelike, NaiveDate};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use datasynth_core::models::{
    FalsePositiveTrigger, LegitimatePatternType, NearMissLabel, NearMissPattern,
};

/// Configuration for near-miss generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearMissConfig {
    /// Proportion of "suspicious" entries that are near-misses (0.0-1.0).
    pub proportion: f64,
    /// Enable near-duplicate detection.
    pub near_duplicate_enabled: bool,
    /// Enable threshold proximity.
    pub threshold_proximity_enabled: bool,
    /// Enable unusual legitimate patterns.
    pub unusual_legitimate_enabled: bool,
    /// Enable corrected errors.
    pub corrected_errors_enabled: bool,
    /// Date difference range for near-duplicates.
    pub near_duplicate_days: (u32, u32),
    /// Proximity range for threshold proximity (0.90-0.99).
    pub proximity_range: (f64, f64),
    /// Correction lag range in days.
    pub correction_lag_days: (u32, u32),
    /// Random seed.
    pub seed: u64,
}

impl Default for NearMissConfig {
    fn default() -> Self {
        Self {
            proportion: 0.30,
            near_duplicate_enabled: true,
            threshold_proximity_enabled: true,
            unusual_legitimate_enabled: true,
            corrected_errors_enabled: true,
            near_duplicate_days: (1, 3),
            proximity_range: (0.90, 0.99),
            correction_lag_days: (1, 5),
            seed: 42,
        }
    }
}

/// Generator for near-miss patterns.
pub struct NearMissGenerator {
    config: NearMissConfig,
    rng: ChaCha8Rng,
    /// Generated near-miss labels.
    labels: Vec<NearMissLabel>,
    /// Tracking recent transactions for near-duplicate detection.
    recent_transactions: Vec<RecentTransaction>,
    /// Maximum recent transactions to track.
    max_recent: usize,
}

/// Tracked transaction for near-duplicate detection.
#[derive(Debug, Clone)]
struct RecentTransaction {
    document_id: String,
    date: NaiveDate,
    amount: Decimal,
    account: String,
    counterparty: Option<String>,
}

impl NearMissGenerator {
    /// Creates a new near-miss generator.
    pub fn new(config: NearMissConfig) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(config.seed);
        Self {
            config,
            rng,
            labels: Vec::new(),
            recent_transactions: Vec::new(),
            max_recent: 100,
        }
    }

    /// Records a transaction for near-duplicate tracking.
    pub fn record_transaction(
        &mut self,
        document_id: impl Into<String>,
        date: NaiveDate,
        amount: Decimal,
        account: impl Into<String>,
        counterparty: Option<String>,
    ) {
        let tx = RecentTransaction {
            document_id: document_id.into(),
            date,
            amount,
            account: account.into(),
            counterparty,
        };

        self.recent_transactions.push(tx);

        // Prune old transactions
        if self.recent_transactions.len() > self.max_recent {
            self.recent_transactions.remove(0);
        }
    }

    /// Checks if a transaction should be marked as a near-miss.
    pub fn check_near_miss(
        &mut self,
        document_id: impl Into<String>,
        date: NaiveDate,
        amount: Decimal,
        account: impl Into<String>,
        counterparty: Option<String>,
        thresholds: &[Decimal],
    ) -> Option<NearMissLabel> {
        // Check proportion
        if self.rng.gen::<f64>() >= self.config.proportion {
            return None;
        }

        let doc_id = document_id.into();
        let acct = account.into();

        // Try different near-miss patterns
        let patterns = self.get_applicable_patterns(date, amount, &acct, &counterparty, thresholds);

        if patterns.is_empty() {
            return None;
        }

        // Select random pattern
        let idx = self.rng.gen_range(0..patterns.len());
        let (pattern, trigger, explanation) = patterns.into_iter().nth(idx).unwrap();

        // Calculate suspicion score based on pattern
        let suspicion_score = match &pattern {
            NearMissPattern::NearDuplicate { .. } => 0.70,
            NearMissPattern::ThresholdProximity { proximity, .. } => 0.50 + proximity * 0.4,
            NearMissPattern::UnusualLegitimate { .. } => 0.55,
            NearMissPattern::CorrectedError { .. } => 0.60,
        };

        let label = NearMissLabel::new(doc_id, pattern, suspicion_score, trigger, explanation);

        self.labels.push(label.clone());
        Some(label)
    }

    /// Gets applicable near-miss patterns for a transaction.
    fn get_applicable_patterns(
        &mut self,
        date: NaiveDate,
        amount: Decimal,
        account: &str,
        counterparty: &Option<String>,
        thresholds: &[Decimal],
    ) -> Vec<(NearMissPattern, FalsePositiveTrigger, String)> {
        let mut patterns = Vec::new();

        // Check for near-duplicate
        if self.config.near_duplicate_enabled {
            if let Some(similar) = self.find_similar_transaction(date, amount, account, counterparty)
            {
                let days_diff = (date - similar.date).num_days().unsigned_abs() as u32;
                if days_diff >= self.config.near_duplicate_days.0
                    && days_diff <= self.config.near_duplicate_days.1
                {
                    patterns.push((
                        NearMissPattern::NearDuplicate {
                            date_difference_days: days_diff,
                            similar_transaction_id: similar.document_id.clone(),
                        },
                        FalsePositiveTrigger::SimilarTransaction,
                        format!(
                            "Similar transaction {} days apart - different business event",
                            days_diff
                        ),
                    ));
                }
            }
        }

        // Check for threshold proximity
        if self.config.threshold_proximity_enabled {
            for threshold in thresholds {
                let proximity = self.calculate_proximity(amount, *threshold);
                if proximity >= self.config.proximity_range.0
                    && proximity <= self.config.proximity_range.1
                {
                    patterns.push((
                        NearMissPattern::ThresholdProximity {
                            threshold: *threshold,
                            proximity,
                        },
                        FalsePositiveTrigger::AmountNearThreshold,
                        format!(
                            "Amount is {:.1}% of threshold {} - coincidental",
                            proximity * 100.0,
                            threshold
                        ),
                    ));
                }
            }
        }

        // Check for unusual legitimate patterns
        if self.config.unusual_legitimate_enabled {
            if let Some((pattern_type, justification)) =
                self.check_unusual_legitimate(date, amount, account)
            {
                patterns.push((
                    NearMissPattern::UnusualLegitimate {
                        pattern_type,
                        justification: justification.clone(),
                    },
                    FalsePositiveTrigger::UnusualTiming,
                    justification,
                ));
            }
        }

        patterns
    }

    /// Finds a similar recent transaction.
    fn find_similar_transaction(
        &self,
        date: NaiveDate,
        amount: Decimal,
        account: &str,
        counterparty: &Option<String>,
    ) -> Option<&RecentTransaction> {
        self.recent_transactions.iter().find(|tx| {
            // Check amount similarity (within 5%)
            let amount_diff = (tx.amount - amount).abs();
            let amount_similar = amount_diff <= tx.amount * dec!(0.05);

            // Check account match
            let account_match = tx.account == account;

            // Check counterparty match
            let counterparty_match = match (&tx.counterparty, counterparty) {
                (Some(a), Some(b)) => a == b,
                _ => true, // If either is missing, don't exclude
            };

            // Check date range (not same day, but within range)
            let days_diff = (date - tx.date).num_days().abs();
            let date_in_range = days_diff > 0
                && days_diff <= self.config.near_duplicate_days.1 as i64;

            amount_similar && account_match && counterparty_match && date_in_range
        })
    }

    /// Calculates proximity to a threshold.
    fn calculate_proximity(&self, amount: Decimal, threshold: Decimal) -> f64 {
        if threshold == Decimal::ZERO {
            return 0.0;
        }
        let amount_f64: f64 = amount.try_into().unwrap_or(0.0);
        let threshold_f64: f64 = threshold.try_into().unwrap_or(1.0);
        (amount_f64 / threshold_f64).min(1.0)
    }

    /// Checks for unusual but legitimate patterns.
    fn check_unusual_legitimate(
        &mut self,
        date: NaiveDate,
        amount: Decimal,
        _account: &str,
    ) -> Option<(LegitimatePatternType, String)> {
        // Year-end bonuses (December, large amounts)
        if date.month() == 12 && amount >= dec!(10000) && self.rng.gen::<f64>() < 0.3 {
            return Some((
                LegitimatePatternType::YearEndBonus,
                "Year-end bonus payment per compensation plan".to_string(),
            ));
        }

        // Contract prepayments (Q1, moderate amounts)
        if date.month() <= 3 && amount >= dec!(5000) && self.rng.gen::<f64>() < 0.2 {
            return Some((
                LegitimatePatternType::ContractPrepayment,
                "Annual contract prepayment per terms".to_string(),
            ));
        }

        // Promotional spending (Q4)
        if date.month() >= 10 && amount >= dec!(25000) && self.rng.gen::<f64>() < 0.2 {
            return Some((
                LegitimatePatternType::PromotionalSpending,
                "Holiday promotional campaign spending".to_string(),
            ));
        }

        // Seasonal inventory (Q3-Q4)
        if date.month() >= 8 && date.month() <= 11 && amount >= dec!(50000) && self.rng.gen::<f64>() < 0.15 {
            return Some((
                LegitimatePatternType::SeasonalInventory,
                "Seasonal inventory buildup for holiday sales".to_string(),
            ));
        }

        // One-time payments (any time, large amounts)
        if amount >= dec!(100000) && self.rng.gen::<f64>() < 0.1 {
            return Some((
                LegitimatePatternType::OneTimePayment,
                "One-time strategic vendor payment".to_string(),
            ));
        }

        None
    }

    /// Creates a corrected error near-miss.
    pub fn create_corrected_error(
        &mut self,
        document_id: impl Into<String>,
        original_error_id: impl Into<String>,
        correction_lag_days: u32,
    ) -> NearMissLabel {
        let pattern = NearMissPattern::CorrectedError {
            correction_lag_days,
            correction_document_id: original_error_id.into(),
        };

        let label = NearMissLabel::new(
            document_id,
            pattern,
            0.60,
            FalsePositiveTrigger::SimilarTransaction,
            format!("Error caught and corrected within {} days", correction_lag_days),
        );

        self.labels.push(label.clone());
        label
    }

    /// Returns all generated labels.
    pub fn get_labels(&self) -> &[NearMissLabel] {
        &self.labels
    }

    /// Resets the generator.
    pub fn reset(&mut self) {
        self.labels.clear();
        self.recent_transactions.clear();
        self.rng = ChaCha8Rng::seed_from_u64(self.config.seed);
    }

    /// Returns statistics about generated near-misses.
    pub fn get_statistics(&self) -> NearMissStatistics {
        let mut by_pattern = std::collections::HashMap::new();
        let mut by_trigger = std::collections::HashMap::new();

        for label in &self.labels {
            let pattern_name = match &label.pattern {
                NearMissPattern::NearDuplicate { .. } => "near_duplicate",
                NearMissPattern::ThresholdProximity { .. } => "threshold_proximity",
                NearMissPattern::UnusualLegitimate { .. } => "unusual_legitimate",
                NearMissPattern::CorrectedError { .. } => "corrected_error",
            };

            *by_pattern.entry(pattern_name.to_string()).or_insert(0) += 1;

            let trigger_name = match label.false_positive_trigger {
                FalsePositiveTrigger::AmountNearThreshold => "amount_near_threshold",
                FalsePositiveTrigger::UnusualTiming => "unusual_timing",
                FalsePositiveTrigger::SimilarTransaction => "similar_transaction",
                FalsePositiveTrigger::NewCounterparty => "new_counterparty",
                FalsePositiveTrigger::UnusualAccountCombination => "unusual_account",
                FalsePositiveTrigger::VolumeSpike => "volume_spike",
                FalsePositiveTrigger::RoundAmount => "round_amount",
            };

            *by_trigger.entry(trigger_name.to_string()).or_insert(0) += 1;
        }

        let avg_suspicion = if self.labels.is_empty() {
            0.0
        } else {
            self.labels.iter().map(|l| l.suspicion_score).sum::<f64>() / self.labels.len() as f64
        };

        NearMissStatistics {
            total_count: self.labels.len(),
            by_pattern,
            by_trigger,
            average_suspicion_score: avg_suspicion,
        }
    }
}

/// Statistics about near-miss generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearMissStatistics {
    /// Total near-miss count.
    pub total_count: usize,
    /// Count by pattern type.
    pub by_pattern: std::collections::HashMap<String, usize>,
    /// Count by trigger type.
    pub by_trigger: std::collections::HashMap<String, usize>,
    /// Average suspicion score.
    pub average_suspicion_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_near_miss_config() {
        let config = NearMissConfig::default();
        assert!((config.proportion - 0.30).abs() < 0.01);
        assert!(config.near_duplicate_enabled);
    }

    #[test]
    fn test_near_miss_generator_creation() {
        let generator = NearMissGenerator::new(NearMissConfig::default());
        assert!(generator.labels.is_empty());
    }

    #[test]
    fn test_record_transaction() {
        let mut generator = NearMissGenerator::new(NearMissConfig::default());

        generator.record_transaction(
            "JE001",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            dec!(10000),
            "5000",
            Some("VENDOR001".to_string()),
        );

        assert_eq!(generator.recent_transactions.len(), 1);
    }

    #[test]
    fn test_threshold_proximity() {
        let mut generator = NearMissGenerator::new(NearMissConfig {
            proportion: 1.0, // Always check
            threshold_proximity_enabled: true,
            ..Default::default()
        });

        let thresholds = vec![dec!(10000), dec!(50000)];

        // Amount is 95% of threshold
        let label = generator.check_near_miss(
            "JE001",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            dec!(9500),
            "5000",
            None,
            &thresholds,
        );

        // May or may not generate depending on RNG and pattern selection
        if let Some(label) = label {
            // If threshold proximity was selected
            if matches!(label.pattern, NearMissPattern::ThresholdProximity { .. }) {
                assert_eq!(label.false_positive_trigger, FalsePositiveTrigger::AmountNearThreshold);
            }
        }
    }

    #[test]
    fn test_corrected_error() {
        let mut generator = NearMissGenerator::new(NearMissConfig::default());

        let label = generator.create_corrected_error("JE002", "JE001", 3);

        assert!(matches!(label.pattern, NearMissPattern::CorrectedError { correction_lag_days: 3, .. }));
        assert_eq!(generator.labels.len(), 1);
    }

    #[test]
    fn test_statistics() {
        let mut generator = NearMissGenerator::new(NearMissConfig::default());

        generator.create_corrected_error("JE001", "JE000", 2);
        generator.create_corrected_error("JE002", "JE000", 3);

        let stats = generator.get_statistics();
        assert_eq!(stats.total_count, 2);
        assert!(stats.by_pattern.contains_key("corrected_error"));
    }
}
