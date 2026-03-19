//! Journal entry risk scoring evaluator.
//!
//! Scores each journal entry for fraud/error risk attributes and computes
//! aggregate statistics including anomaly separability.

use datasynth_core::models::JournalEntry;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Result types ─────────────────────────────────────────────────────────────

/// Aggregate result of JE risk scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JeRiskScoringResult {
    /// Total number of journal entries supplied.
    pub total_entries: usize,
    /// Number of entries that were actually scored.
    pub scored_entries: usize,
    /// Distribution of entries across risk bands.
    pub risk_distribution: RiskDistribution,
    /// Per-attribute statistics.
    pub risk_attributes: Vec<RiskAttributeStats>,
    /// Average anomaly score minus average clean score.
    /// Pass threshold: > 0.10.
    pub anomaly_separability: f64,
    /// True when anomaly_separability > 0.10 (or no anomaly labels present).
    pub passes: bool,
}

/// Count of entries in each risk band.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskDistribution {
    /// Score < 0.30.
    pub low_risk: usize,
    /// 0.30 ≤ score < 0.60.
    pub medium_risk: usize,
    /// Score ≥ 0.60.
    pub high_risk: usize,
}

/// Statistics for one risk attribute across all scored entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAttributeStats {
    /// Attribute name (e.g. "RoundNumber").
    pub attribute: String,
    /// Number of entries where this attribute was triggered.
    pub count: usize,
    /// Percentage of total scored entries (0–100).
    pub percentage: f64,
}

// ─── Per-entry score ──────────────────────────────────────────────────────────

/// All risk attributes that can be detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RiskAttribute {
    RoundNumber,
    UnusualHour,
    WeekendHoliday,
    NonStandardUser,
    BelowApprovalThreshold,
    ManualToAutomatedAccount,
    LargeRoundTrip,
}

impl RiskAttribute {
    fn name(self) -> &'static str {
        match self {
            Self::RoundNumber => "RoundNumber",
            Self::UnusualHour => "UnusualHour",
            Self::WeekendHoliday => "WeekendHoliday",
            Self::NonStandardUser => "NonStandardUser",
            Self::BelowApprovalThreshold => "BelowApprovalThreshold",
            Self::ManualToAutomatedAccount => "ManualToAutomatedAccount",
            Self::LargeRoundTrip => "LargeRoundTrip",
        }
    }

    fn weight(self) -> f64 {
        match self {
            Self::RoundNumber => 0.10,
            Self::UnusualHour => 0.15,
            Self::WeekendHoliday => 0.15,
            Self::NonStandardUser => 0.15,
            Self::BelowApprovalThreshold => 0.15,
            Self::ManualToAutomatedAccount => 0.15,
            Self::LargeRoundTrip => 0.15,
        }
    }

    fn all() -> &'static [RiskAttribute] {
        &[
            Self::RoundNumber,
            Self::UnusualHour,
            Self::WeekendHoliday,
            Self::NonStandardUser,
            Self::BelowApprovalThreshold,
            Self::ManualToAutomatedAccount,
            Self::LargeRoundTrip,
        ]
    }
}

// ─── Detection helpers ────────────────────────────────────────────────────────

/// Common thresholds for "split-payment" detection (amounts just below these).
const APPROVAL_THRESHOLDS: &[u64] = &[1000, 2500, 5000, 10000, 25000, 50000, 100000];

/// GL accounts that are normally auto-posted (bank, AP clearing, AR clearing).
/// Prefixes: "10" (bank/cash), "20" (AP clearing), "11" (AR clearing).
const AUTOMATED_ACCOUNT_PREFIXES: &[&str] = &["100", "101", "102", "200", "201", "110", "111"];

fn is_round_number(amount: Decimal) -> bool {
    let thousand = Decimal::from(1000u32);
    amount > Decimal::ZERO && (amount % thousand).is_zero()
}

fn is_unusual_hour(hour: u32) -> bool {
    !(7..=21).contains(&hour)
}

fn is_weekend(weekday: chrono::Weekday) -> bool {
    weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun
}

fn is_below_approval_threshold(amount: Decimal) -> bool {
    for &threshold in APPROVAL_THRESHOLDS {
        let low = Decimal::from(threshold - 100);
        let high = Decimal::from(threshold - 1);
        if amount >= low && amount <= high {
            return true;
        }
    }
    false
}

fn is_manual_to_automated_account(entry: &JournalEntry) -> bool {
    use datasynth_core::models::TransactionSource;
    if entry.header.source != TransactionSource::Manual {
        return false;
    }
    entry.lines.iter().any(|line| {
        AUTOMATED_ACCOUNT_PREFIXES
            .iter()
            .any(|prefix| line.gl_account.starts_with(prefix))
    })
}

fn has_round_trip(entry: &JournalEntry) -> bool {
    // Same account appears on both debit and credit sides within one entry.
    let debited: std::collections::HashSet<_> = entry
        .lines
        .iter()
        .filter(|l| l.debit_amount > Decimal::ZERO)
        .map(|l| l.gl_account.as_str())
        .collect();
    let credited: std::collections::HashSet<_> = entry
        .lines
        .iter()
        .filter(|l| l.credit_amount > Decimal::ZERO)
        .map(|l| l.gl_account.as_str())
        .collect();
    debited.intersection(&credited).next().is_some()
}

// ─── Pre-computation pass ─────────────────────────────────────────────────────

/// Count postings per user across all entries.
fn build_user_posting_counts(entries: &[JournalEntry]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        *counts.entry(entry.header.created_by.clone()).or_default() += 1;
    }
    counts
}

// ─── Scoring ──────────────────────────────────────────────────────────────────

/// Score a single journal entry; returns (score, triggered_attributes).
fn score_entry(
    entry: &JournalEntry,
    user_counts: &HashMap<String, usize>,
) -> (f64, Vec<RiskAttribute>) {
    use chrono::Datelike as _;
    use chrono::Timelike as _;

    let mut triggered = Vec::new();

    // Derive a representative "amount" from the entry (sum of debit amounts).
    let total_debit: Decimal = entry.lines.iter().map(|l| l.debit_amount).sum();

    // RoundNumber
    if is_round_number(total_debit) {
        triggered.push(RiskAttribute::RoundNumber);
    }

    // UnusualHour
    let hour = entry.header.created_at.hour();
    if is_unusual_hour(hour) {
        triggered.push(RiskAttribute::UnusualHour);
    }

    // WeekendHoliday
    if is_weekend(entry.header.posting_date.weekday()) {
        triggered.push(RiskAttribute::WeekendHoliday);
    }

    // NonStandardUser (fewer than 5 postings)
    let user_count = user_counts
        .get(&entry.header.created_by)
        .copied()
        .unwrap_or(0);
    if user_count < 5 {
        triggered.push(RiskAttribute::NonStandardUser);
    }

    // BelowApprovalThreshold
    if is_below_approval_threshold(total_debit) {
        triggered.push(RiskAttribute::BelowApprovalThreshold);
    }

    // ManualToAutomatedAccount
    if is_manual_to_automated_account(entry) {
        triggered.push(RiskAttribute::ManualToAutomatedAccount);
    }

    // LargeRoundTrip
    if has_round_trip(entry) {
        triggered.push(RiskAttribute::LargeRoundTrip);
    }

    let raw_score: f64 = triggered.iter().map(|a| a.weight()).sum();
    let score = raw_score.min(1.0_f64);

    (score, triggered)
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Score all journal entries and return aggregate statistics.
pub fn score_entries(entries: &[JournalEntry]) -> JeRiskScoringResult {
    let user_counts = build_user_posting_counts(entries);

    let mut distribution = RiskDistribution::default();
    let mut attribute_counts: HashMap<RiskAttribute, usize> = HashMap::new();
    let mut anomaly_scores: Vec<f64> = Vec::new();
    let mut clean_scores: Vec<f64> = Vec::new();

    for entry in entries {
        let (score, triggered) = score_entry(entry, &user_counts);

        // Risk band
        if score < 0.30 {
            distribution.low_risk += 1;
        } else if score < 0.60 {
            distribution.medium_risk += 1;
        } else {
            distribution.high_risk += 1;
        }

        // Attribute counts
        for attr in &triggered {
            *attribute_counts.entry(*attr).or_default() += 1;
        }

        // Separability tracking
        if entry.header.is_anomaly || entry.header.is_fraud {
            anomaly_scores.push(score);
        } else {
            clean_scores.push(score);
        }
    }

    let total = entries.len();
    let risk_attributes: Vec<RiskAttributeStats> = RiskAttribute::all()
        .iter()
        .map(|&attr| {
            let count = attribute_counts.get(&attr).copied().unwrap_or(0);
            let percentage = if total > 0 {
                count as f64 / total as f64 * 100.0
            } else {
                0.0
            };
            RiskAttributeStats {
                attribute: attr.name().to_string(),
                count,
                percentage,
            }
        })
        .collect();

    let avg = |v: &[f64]| -> f64 {
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };

    let anomaly_separability = if anomaly_scores.is_empty() {
        // No anomaly labels → vacuously pass
        1.0
    } else {
        avg(&anomaly_scores) - avg(&clean_scores)
    };

    let passes = anomaly_separability > 0.10;

    JeRiskScoringResult {
        total_entries: total,
        scored_entries: total,
        risk_distribution: distribution,
        risk_attributes,
        anomaly_separability,
        passes,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::{
        JournalEntry, JournalEntryHeader, JournalEntryLine, TransactionSource,
    };
    use rust_decimal_macros::dec;

    fn make_date(year: i32, month: u32, day: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn weekday_date() -> chrono::NaiveDate {
        // 2024-01-03 is a Wednesday
        make_date(2024, 1, 3)
    }

    fn weekend_date() -> chrono::NaiveDate {
        // 2024-01-06 is a Saturday
        make_date(2024, 1, 6)
    }

    fn make_je(
        company: &str,
        posting_date: chrono::NaiveDate,
        debit_account: &str,
        credit_account: &str,
        amount: Decimal,
        user: &str,
        source: TransactionSource,
    ) -> JournalEntry {
        let mut header = JournalEntryHeader::new(company.to_string(), posting_date);
        header.created_by = user.to_string();
        header.source = source;
        let doc_id = header.document_id;
        let mut entry = JournalEntry::new(header);
        entry.add_line(JournalEntryLine::debit(
            doc_id,
            1,
            debit_account.to_string(),
            amount,
        ));
        entry.add_line(JournalEntryLine::credit(
            doc_id,
            2,
            credit_account.to_string(),
            amount,
        ));
        entry
    }

    fn simple_je(amount: Decimal) -> JournalEntry {
        make_je(
            "C001",
            weekday_date(),
            "6000",
            "2000",
            amount,
            "alice",
            TransactionSource::Automated,
        )
    }

    // ── Round-number detection ────────────────────────────────────────────────

    #[test]
    fn test_round_number_detected() {
        assert!(is_round_number(dec!(1000)));
        assert!(is_round_number(dec!(5000)));
        assert!(is_round_number(dec!(100000)));
    }

    #[test]
    fn test_non_round_number() {
        assert!(!is_round_number(dec!(1234.56)));
        assert!(!is_round_number(dec!(999)));
        assert!(!is_round_number(dec!(0)));
    }

    // ── Weekend detection ─────────────────────────────────────────────────────

    #[test]
    fn test_weekend_detected() {
        let entry = make_je(
            "C001",
            weekend_date(),
            "6000",
            "2000",
            dec!(500),
            "alice",
            TransactionSource::Automated,
        );
        let counts = build_user_posting_counts(&[entry.clone()]);
        let (_score, triggered) = score_entry(&entry, &counts);
        assert!(
            triggered.contains(&RiskAttribute::WeekendHoliday),
            "Saturday should trigger WeekendHoliday"
        );
    }

    #[test]
    fn test_weekday_not_flagged() {
        let entry = make_je(
            "C001",
            weekday_date(),
            "6000",
            "2000",
            dec!(500),
            "alice",
            TransactionSource::Automated,
        );
        // post alice 10 times so she's not a NonStandardUser
        let mut entries: Vec<JournalEntry> = (0..10)
            .map(|_| {
                make_je(
                    "C001",
                    weekday_date(),
                    "6000",
                    "2000",
                    dec!(500),
                    "alice",
                    TransactionSource::Automated,
                )
            })
            .collect();
        entries.push(entry.clone());
        let counts = build_user_posting_counts(&entries);
        let (_score, triggered) = score_entry(&entry, &counts);
        assert!(
            !triggered.contains(&RiskAttribute::WeekendHoliday),
            "Wednesday should not trigger WeekendHoliday"
        );
    }

    // ── Score range ───────────────────────────────────────────────────────────

    #[test]
    fn test_score_within_range() {
        let entries: Vec<JournalEntry> = vec![simple_je(dec!(500)), simple_je(dec!(1000))];
        let counts = build_user_posting_counts(&entries);
        for entry in &entries {
            let (score, _) = score_entry(entry, &counts);
            assert!(score >= 0.0 && score <= 1.0, "Score {score} out of [0,1]");
        }
    }

    #[test]
    fn test_multi_attribute_higher_score() {
        // Entry with round number + weekend
        let risky = make_je(
            "C001",
            weekend_date(),
            "6000",
            "2000",
            dec!(5000), // round
            "alice",
            TransactionSource::Automated,
        );
        let clean = make_je(
            "C001",
            weekday_date(),
            "6000",
            "2000",
            dec!(1234),
            "alice",
            TransactionSource::Automated,
        );
        let mut entries = vec![risky.clone()];
        // 10 alice postings so she's not NonStandardUser in clean entry
        for _ in 0..10 {
            entries.push(make_je(
                "C001",
                weekday_date(),
                "6000",
                "2000",
                dec!(100),
                "alice",
                TransactionSource::Automated,
            ));
        }
        entries.push(clean.clone());
        let counts = build_user_posting_counts(&entries);
        let (risky_score, _) = score_entry(&risky, &counts);
        let (clean_score, _) = score_entry(&clean, &counts);
        assert!(
            risky_score >= clean_score,
            "Risky entry ({risky_score}) should score >= clean ({clean_score})"
        );
    }

    // ── Below-threshold detection ─────────────────────────────────────────────

    #[test]
    fn test_below_approval_threshold() {
        assert!(is_below_approval_threshold(dec!(4999)));
        assert!(is_below_approval_threshold(dec!(4950)));
        assert!(!is_below_approval_threshold(dec!(5000)));
        assert!(!is_below_approval_threshold(dec!(6000)));
    }

    // ── Round-trip detection ──────────────────────────────────────────────────

    #[test]
    fn test_round_trip_detected() {
        let header = JournalEntryHeader::new("C001".to_string(), weekday_date());
        let doc_id = header.document_id;
        let mut entry = JournalEntry::new(header);
        // Same account on both sides
        entry.add_line(JournalEntryLine::debit(
            doc_id,
            1,
            "1000".to_string(),
            dec!(100),
        ));
        entry.add_line(JournalEntryLine::credit(
            doc_id,
            2,
            "1000".to_string(),
            dec!(100),
        ));
        assert!(
            has_round_trip(&entry),
            "Same account debit+credit should be detected"
        );
    }

    #[test]
    fn test_no_round_trip() {
        let entry = simple_je(dec!(100));
        assert!(
            !has_round_trip(&entry),
            "Different accounts should not trigger round-trip"
        );
    }

    // ── Aggregate scoring ─────────────────────────────────────────────────────

    #[test]
    fn test_score_entries_basic() {
        let entries: Vec<JournalEntry> = (0..20)
            .map(|i| {
                make_je(
                    "C001",
                    weekday_date(),
                    "6000",
                    "2000",
                    Decimal::from(i * 100 + 50),
                    "alice",
                    TransactionSource::Automated,
                )
            })
            .collect();
        let result = score_entries(&entries);
        assert_eq!(result.total_entries, 20);
        assert_eq!(result.scored_entries, 20);
        assert_eq!(
            result.risk_distribution.low_risk
                + result.risk_distribution.medium_risk
                + result.risk_distribution.high_risk,
            20
        );
        assert_eq!(result.risk_attributes.len(), RiskAttribute::all().len());
    }

    #[test]
    fn test_anomaly_separability_passes_with_no_labels() {
        let entries: Vec<JournalEntry> = (0..5).map(|_| simple_je(dec!(100))).collect();
        let result = score_entries(&entries);
        // No anomaly labels → vacuously passes
        assert!(result.passes, "No anomaly labels → should pass");
    }

    #[test]
    fn test_anomaly_separability_with_flagged_entries() {
        let mut entries: Vec<JournalEntry> = Vec::new();

        // 5 clean entries (low-risk amounts, no round numbers, weekday)
        for _ in 0..5 {
            let mut e = make_je(
                "C001",
                weekday_date(),
                "6000",
                "2000",
                dec!(123),
                "bob",
                TransactionSource::Automated,
            );
            // post bob many times
            e.header.is_anomaly = false;
            entries.push(e);
        }
        // Force bob to have many postings
        for _ in 0..10 {
            entries.push(make_je(
                "C001",
                weekday_date(),
                "6000",
                "2000",
                dec!(50),
                "bob",
                TransactionSource::Automated,
            ));
        }

        // 5 anomaly entries: weekend + round number
        for _ in 0..5 {
            let mut e = make_je(
                "C001",
                weekend_date(),
                "6000",
                "2000",
                dec!(5000),
                "zz_rare_user",
                TransactionSource::Automated,
            );
            e.header.is_anomaly = true;
            entries.push(e);
        }

        let result = score_entries(&entries);
        assert!(
            result.anomaly_separability > 0.0,
            "Anomaly entries should have higher average score than clean entries"
        );
    }
}
