//! Transaction-level label generation.

use chrono::{Datelike, Timelike};
use datasynth_core::models::banking::{AmlTypology, LaunderingStage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::BankTransaction;

/// Transaction-level labels for ML training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLabel {
    /// Transaction ID
    pub transaction_id: Uuid,
    /// Binary suspicious flag
    pub is_suspicious: bool,
    /// Specific suspicion reason
    pub suspicion_reason: Option<AmlTypology>,
    /// Money laundering stage
    pub laundering_stage: Option<LaunderingStage>,
    /// Case ID for linking related transactions
    pub case_id: Option<String>,
    /// Whether transaction has been spoofed
    pub is_spoofed: bool,
    /// Spoofing intensity (0.0-1.0)
    pub spoofing_intensity: Option<f64>,
    /// Sequence within scenario
    pub scenario_sequence: Option<u32>,
    /// Confidence score for the label (for soft labels)
    pub confidence: f64,
    /// Additional feature flags
    pub features: TransactionLabelFeatures,
}

/// Additional transaction label features.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionLabelFeatures {
    /// Is this part of a structuring pattern?
    pub is_structuring: bool,
    /// Is amount below reporting threshold?
    pub below_threshold: bool,
    /// Is this a cash transaction?
    pub is_cash: bool,
    /// Is this an international transaction?
    pub is_international: bool,
    /// Rapid succession with other transactions?
    pub is_rapid_succession: bool,
    /// Is counterparty new/unknown?
    pub new_counterparty: bool,
    /// Round number amount?
    pub round_amount: bool,
    /// Transaction on weekend/holiday?
    pub unusual_timing: bool,
}

impl TransactionLabel {
    /// Create a new transaction label from a transaction.
    pub fn from_transaction(txn: &BankTransaction) -> Self {
        let amount_f64: f64 = txn.amount.try_into().unwrap_or(0.0);

        Self {
            transaction_id: txn.transaction_id,
            is_suspicious: txn.is_suspicious,
            suspicion_reason: txn.suspicion_reason,
            laundering_stage: txn.laundering_stage,
            case_id: txn.case_id.clone(),
            is_spoofed: txn.is_spoofed,
            spoofing_intensity: txn.spoofing_intensity,
            scenario_sequence: txn.scenario_sequence,
            confidence: 1.0, // Ground truth has full confidence
            features: TransactionLabelFeatures {
                is_structuring: txn.suspicion_reason == Some(AmlTypology::Structuring),
                below_threshold: amount_f64 < 10_000.0 && amount_f64 > 8_000.0,
                is_cash: matches!(
                    txn.channel,
                    datasynth_core::models::banking::TransactionChannel::Cash
                ),
                is_international: matches!(
                    txn.category,
                    datasynth_core::models::banking::TransactionCategory::InternationalTransfer
                ),
                is_rapid_succession: false, // Computed separately
                new_counterparty: false,    // Computed separately
                round_amount: Self::is_round_amount(amount_f64),
                unusual_timing: Self::is_unusual_timing(txn),
            },
        }
    }

    /// Check if amount is a round number.
    fn is_round_amount(amount: f64) -> bool {
        let cents = (amount * 100.0) % 100.0;
        cents.abs() < 0.01 && amount >= 100.0 && (amount % 100.0).abs() < 0.01
    }

    /// Check if timing is unusual (weekend/off-hours).
    fn is_unusual_timing(txn: &BankTransaction) -> bool {
        let weekday = txn.timestamp_initiated.weekday();
        let hour = txn.timestamp_initiated.hour();

        // Weekend or outside business hours
        matches!(weekday, chrono::Weekday::Sat | chrono::Weekday::Sun) || !(6..=22).contains(&hour)
    }
}

/// Transaction label extractor.
pub struct TransactionLabelExtractor;

impl TransactionLabelExtractor {
    /// Extract labels from all transactions.
    pub fn extract(transactions: &[BankTransaction]) -> Vec<TransactionLabel> {
        transactions
            .iter()
            .map(TransactionLabel::from_transaction)
            .collect()
    }

    /// Extract labels with computed features.
    pub fn extract_with_features(transactions: &[BankTransaction]) -> Vec<TransactionLabel> {
        let mut labels: Vec<_> = transactions
            .iter()
            .map(TransactionLabel::from_transaction)
            .collect();

        // Compute rapid succession features
        Self::compute_rapid_succession(&mut labels, transactions);

        // Compute new counterparty features
        Self::compute_new_counterparty(&mut labels, transactions);

        labels
    }

    /// Compute rapid succession feature.
    fn compute_rapid_succession(labels: &mut [TransactionLabel], transactions: &[BankTransaction]) {
        // Group transactions by account
        use std::collections::HashMap;
        let mut by_account: HashMap<Uuid, Vec<usize>> = HashMap::new();

        for (i, txn) in transactions.iter().enumerate() {
            by_account.entry(txn.account_id).or_default().push(i);
        }

        // Check for rapid succession within each account
        for indices in by_account.values() {
            for window in indices.windows(2) {
                let t1 = &transactions[window[0]];
                let t2 = &transactions[window[1]];

                let duration = (t2.timestamp_initiated - t1.timestamp_initiated)
                    .num_minutes()
                    .abs();
                if duration < 30 {
                    labels[window[0]].features.is_rapid_succession = true;
                    labels[window[1]].features.is_rapid_succession = true;
                }
            }
        }
    }

    /// Compute new counterparty feature.
    fn compute_new_counterparty(labels: &mut [TransactionLabel], transactions: &[BankTransaction]) {
        use std::collections::{HashMap, HashSet};

        // Track seen counterparties per account
        let mut seen: HashMap<Uuid, HashSet<String>> = HashMap::new();

        // Sort by timestamp
        let mut sorted_indices: Vec<usize> = (0..transactions.len()).collect();
        sorted_indices.sort_by_key(|&i| transactions[i].timestamp_initiated);

        for idx in sorted_indices {
            let txn = &transactions[idx];
            let counterparty_key = txn.counterparty.name.clone();

            let account_seen = seen.entry(txn.account_id).or_default();

            if !account_seen.contains(&counterparty_key) {
                labels[idx].features.new_counterparty = true;
                account_seen.insert(counterparty_key);
            }
        }
    }

    /// Get summary statistics for labels.
    pub fn summarize(labels: &[TransactionLabel]) -> LabelSummary {
        let total = labels.len();
        let suspicious = labels.iter().filter(|l| l.is_suspicious).count();
        let spoofed = labels.iter().filter(|l| l.is_spoofed).count();

        let mut by_typology = std::collections::HashMap::new();
        let mut by_stage = std::collections::HashMap::new();

        for label in labels {
            if let Some(reason) = &label.suspicion_reason {
                *by_typology.entry(*reason).or_insert(0) += 1;
            }
            if let Some(stage) = &label.laundering_stage {
                *by_stage.entry(*stage).or_insert(0) += 1;
            }
        }

        LabelSummary {
            total_transactions: total,
            suspicious_count: suspicious,
            suspicious_rate: suspicious as f64 / total as f64,
            spoofed_count: spoofed,
            spoofed_rate: spoofed as f64 / total as f64,
            by_typology,
            by_stage,
        }
    }
}

/// Summary statistics for transaction labels.
#[derive(Debug, Clone)]
pub struct LabelSummary {
    /// Total number of transactions
    pub total_transactions: usize,
    /// Number of suspicious transactions
    pub suspicious_count: usize,
    /// Rate of suspicious transactions
    pub suspicious_rate: f64,
    /// Number of spoofed transactions
    pub spoofed_count: usize,
    /// Rate of spoofed transactions
    pub spoofed_rate: f64,
    /// Counts by typology
    pub by_typology: std::collections::HashMap<AmlTypology, usize>,
    /// Counts by laundering stage
    pub by_stage: std::collections::HashMap<LaunderingStage, usize>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_label_extraction() {
        let account_id = Uuid::new_v4();

        let txn = BankTransaction::new(
            Uuid::new_v4(),
            account_id,
            rust_decimal::Decimal::from(9500),
            "USD",
            datasynth_core::models::banking::Direction::Inbound,
            datasynth_core::models::banking::TransactionChannel::Cash,
            datasynth_core::models::banking::TransactionCategory::CashDeposit,
            crate::models::CounterpartyRef::atm("ATM"),
            "Test deposit",
            chrono::Utc::now(),
        )
        .mark_suspicious(AmlTypology::Structuring, "TEST-001")
        .with_laundering_stage(LaunderingStage::Placement);

        let label = TransactionLabel::from_transaction(&txn);

        assert!(label.is_suspicious);
        assert_eq!(label.suspicion_reason, Some(AmlTypology::Structuring));
        assert_eq!(label.laundering_stage, Some(LaunderingStage::Placement));
        assert!(label.features.below_threshold);
        assert!(label.features.is_cash);
    }
}
