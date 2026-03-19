//! Integration tests for the JE risk scoring evaluator.

#![allow(clippy::unwrap_used)]

use datasynth_core::models::{JournalEntry, JournalEntryHeader, JournalEntryLine, TransactionSource};
use datasynth_eval::coherence::je_risk_scoring::{score_entries, JeRiskScoringResult};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn weekday() -> chrono::NaiveDate {
    // Wednesday 2024-01-03
    chrono::NaiveDate::from_ymd_opt(2024, 1, 3).unwrap()
}

fn saturday() -> chrono::NaiveDate {
    // Saturday 2024-01-06
    chrono::NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()
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
    entry.add_line(JournalEntryLine::debit(doc_id, 1, debit_account.to_string(), amount));
    entry.add_line(JournalEntryLine::credit(doc_id, 2, credit_account.to_string(), amount));
    entry
}

/// Build a list of N identical clean entries for a given user (enough to exceed
/// the 5-posting threshold so they are not flagged as NonStandardUser).
fn many_clean(n: usize, user: &str) -> Vec<JournalEntry> {
    (0..n)
        .map(|_| make_je("C001", weekday(), "6000", "2000", dec!(123), user, TransactionSource::Automated))
        .collect()
}

// ─── Basic structure ──────────────────────────────────────────────────────────

#[test]
fn test_empty_input() {
    let result = score_entries(&[]);
    assert_eq!(result.total_entries, 0);
    assert_eq!(result.scored_entries, 0);
    assert_eq!(result.risk_distribution.low_risk, 0);
    assert_eq!(result.risk_distribution.medium_risk, 0);
    assert_eq!(result.risk_distribution.high_risk, 0);
}

#[test]
fn test_total_counts_match() {
    let entries: Vec<JournalEntry> = many_clean(20, "alice");
    let result = score_entries(&entries);
    assert_eq!(result.total_entries, 20);
    assert_eq!(result.scored_entries, 20);
    let band_total = result.risk_distribution.low_risk
        + result.risk_distribution.medium_risk
        + result.risk_distribution.high_risk;
    assert_eq!(band_total, 20, "Risk bands must sum to total entries");
}

#[test]
fn test_attribute_stats_present_for_all_attributes() {
    let entries = many_clean(5, "alice");
    let result = score_entries(&entries);
    assert_eq!(result.risk_attributes.len(), 7, "Should report all 7 risk attributes");
    let names: Vec<&str> = result.risk_attributes.iter().map(|a| a.attribute.as_str()).collect();
    assert!(names.contains(&"RoundNumber"));
    assert!(names.contains(&"UnusualHour"));
    assert!(names.contains(&"WeekendHoliday"));
    assert!(names.contains(&"NonStandardUser"));
    assert!(names.contains(&"BelowApprovalThreshold"));
    assert!(names.contains(&"ManualToAutomatedAccount"));
    assert!(names.contains(&"LargeRoundTrip"));
}

// ─── Round number detection ───────────────────────────────────────────────────

#[test]
fn test_round_number_exact_thousand() {
    // Many clean entries for alice so NonStandardUser is not triggered
    let mut entries = many_clean(10, "alice");
    entries.push(make_je("C001", weekday(), "6000", "2000", dec!(1000), "alice", TransactionSource::Automated));
    let result = score_entries(&entries);
    let rn = result.risk_attributes.iter().find(|a| a.attribute == "RoundNumber").unwrap();
    assert!(rn.count > 0, "1000.00 should trigger RoundNumber");
}

#[test]
fn test_non_round_amount_not_flagged() {
    let entries = many_clean(10, "alice");
    // All entries have amount 123 (non-round)
    let result = score_entries(&entries);
    let rn = result.risk_attributes.iter().find(|a| a.attribute == "RoundNumber").unwrap();
    assert_eq!(rn.count, 0, "123 should not trigger RoundNumber");
}

// ─── Weekend detection ────────────────────────────────────────────────────────

#[test]
fn test_weekend_entries_flagged() {
    let mut entries = many_clean(10, "alice"); // establish alice as non-rare
    entries.push(make_je("C001", saturday(), "6000", "2000", dec!(123), "alice", TransactionSource::Automated));
    let result = score_entries(&entries);
    let wh = result.risk_attributes.iter().find(|a| a.attribute == "WeekendHoliday").unwrap();
    assert!(wh.count >= 1, "Saturday entry should trigger WeekendHoliday");
}

#[test]
fn test_weekday_entries_not_flagged_as_weekend() {
    let entries = many_clean(10, "alice");
    let result = score_entries(&entries);
    let wh = result.risk_attributes.iter().find(|a| a.attribute == "WeekendHoliday").unwrap();
    assert_eq!(wh.count, 0, "Wednesday entries should not trigger WeekendHoliday");
}

// ─── Non-standard user ────────────────────────────────────────────────────────

#[test]
fn test_rare_user_flagged() {
    // "alice" has 10 postings, "zara_rare" has only 1
    let mut entries = many_clean(10, "alice");
    entries.push(make_je("C001", weekday(), "6000", "2000", dec!(123), "zara_rare", TransactionSource::Automated));
    let result = score_entries(&entries);
    let nsu = result.risk_attributes.iter().find(|a| a.attribute == "NonStandardUser").unwrap();
    assert!(nsu.count >= 1, "User with 1 posting should trigger NonStandardUser");
}

#[test]
fn test_frequent_user_not_flagged() {
    let entries = many_clean(10, "alice");
    let result = score_entries(&entries);
    let nsu = result.risk_attributes.iter().find(|a| a.attribute == "NonStandardUser").unwrap();
    assert_eq!(nsu.count, 0, "User with 10 postings should not trigger NonStandardUser");
}

// ─── Below-approval-threshold ─────────────────────────────────────────────────

#[test]
fn test_amount_just_below_threshold_flagged() {
    let mut entries = many_clean(10, "alice");
    entries.push(make_je("C001", weekday(), "6000", "2000", dec!(4999), "alice", TransactionSource::Automated));
    let result = score_entries(&entries);
    let bat = result.risk_attributes.iter().find(|a| a.attribute == "BelowApprovalThreshold").unwrap();
    assert!(bat.count >= 1, "4999 is just below 5000 threshold");
}

#[test]
fn test_normal_amount_not_flagged_as_below_threshold() {
    let entries = many_clean(10, "alice"); // all 123
    let result = score_entries(&entries);
    let bat = result.risk_attributes.iter().find(|a| a.attribute == "BelowApprovalThreshold").unwrap();
    assert_eq!(bat.count, 0, "123 is not near any approval threshold");
}

// ─── Manual to automated account ──────────────────────────────────────────────

#[test]
fn test_manual_posting_to_bank_account_flagged() {
    let mut entries = many_clean(10, "alice");
    let manual_bank = make_je("C001", weekday(), "1001", "3000", dec!(500), "alice", TransactionSource::Manual);
    entries.push(manual_bank);
    let result = score_entries(&entries);
    let mta = result.risk_attributes.iter().find(|a| a.attribute == "ManualToAutomatedAccount").unwrap();
    assert!(mta.count >= 1, "Manual posting to 1001 (bank) should be flagged");
}

#[test]
fn test_automated_posting_to_bank_not_flagged() {
    let entries: Vec<JournalEntry> = (0..10)
        .map(|_| make_je("C001", weekday(), "1001", "3000", dec!(500), "alice", TransactionSource::Automated))
        .collect();
    let result = score_entries(&entries);
    let mta = result.risk_attributes.iter().find(|a| a.attribute == "ManualToAutomatedAccount").unwrap();
    assert_eq!(mta.count, 0, "Automated posting should not be flagged");
}

// ─── Round-trip detection ─────────────────────────────────────────────────────

#[test]
fn test_same_account_debit_and_credit_flagged() {
    let mut entries = many_clean(10, "alice");

    // Build a JE where account 1000 appears on both sides
    let header = JournalEntryHeader::new("C001".to_string(), weekday());
    let doc_id = header.document_id;
    let mut rt_entry = JournalEntry::new(header);
    rt_entry.header.created_by = "alice".to_string();
    rt_entry.add_line(JournalEntryLine::debit(doc_id, 1, "1000".to_string(), dec!(500)));
    rt_entry.add_line(JournalEntryLine::credit(doc_id, 2, "1000".to_string(), dec!(500)));
    entries.push(rt_entry);

    let result = score_entries(&entries);
    let lrt = result.risk_attributes.iter().find(|a| a.attribute == "LargeRoundTrip").unwrap();
    assert!(lrt.count >= 1, "Same debit/credit account should trigger LargeRoundTrip");
}

// ─── Risk distribution ────────────────────────────────────────────────────────

#[test]
fn test_clean_entries_fall_in_low_risk_band() {
    // 10 clean automated entries, alice is frequent — should mostly be low risk
    let entries = many_clean(10, "alice");
    let result = score_entries(&entries);
    assert!(
        result.risk_distribution.low_risk > 0,
        "Clean entries should produce some low-risk scores"
    );
}

#[test]
fn test_risky_entries_have_higher_scores() {
    // Risky: round number (0.10) + weekend (0.15) + below-threshold (0.15) = 0.40 → medium risk
    // Amount 4999 is below the 5000 threshold AND is not a round number.
    // Use dec!(4000) which is round + weekend = 0.25 (low) — not enough.
    // Instead use a rare user + weekend + round to get 0.10+0.15+0.15 = 0.40 (medium).
    // zz_once posts exactly 3 entries → < 5 → NonStandardUser triggered
    // Each risky entry triggers: NonStandardUser (0.15) + WeekendHoliday (0.15) + RoundNumber (0.10) = 0.40 → medium
    let mut risky_entries: Vec<JournalEntry> = (0..3)
        .map(|_| make_je("C001", saturday(), "6000", "2000", dec!(5000), "zz_once", TransactionSource::Automated))
        .collect();
    // Add alice entries (she's a frequent user, won't be flagged as rare)
    for _ in 0..10 {
        risky_entries.push(make_je("C001", weekday(), "6000", "2000", dec!(100), "alice", TransactionSource::Automated));
    }
    let result = score_entries(&risky_entries);
    // zz_once: NonStandardUser (0.15) + WeekendHoliday (0.15) + RoundNumber (0.10) = 0.40 → medium
    assert!(
        result.risk_distribution.medium_risk + result.risk_distribution.high_risk > 0,
        "Weekend + round-number + rare-user entries should reach medium/high risk band"
    );
}

// ─── Anomaly separability ─────────────────────────────────────────────────────

#[test]
fn test_no_anomaly_labels_vacuously_passes() {
    let entries = many_clean(10, "alice");
    let result = score_entries(&entries);
    // All is_anomaly = false → separability = 1.0 (vacuous)
    assert!(result.passes, "No anomaly labels → should pass vacuously");
}

#[test]
fn test_anomaly_entries_have_higher_average_score() {
    let mut entries = Vec::new();

    // 10 clean entries
    for e in many_clean(10, "alice") {
        entries.push(e);
    }

    // 5 anomaly entries: weekend + round number + rare user
    for _ in 0..5 {
        let mut e = make_je(
            "C001",
            saturday(),
            "6000",
            "2000",
            dec!(5000),
            "zz_rare",
            TransactionSource::Automated,
        );
        e.header.is_anomaly = true;
        entries.push(e);
    }

    let result = score_entries(&entries);
    assert!(
        result.anomaly_separability > 0.0,
        "Anomaly entries should have higher average score; separability = {}",
        result.anomaly_separability
    );
}

#[test]
fn test_anomaly_separability_threshold_at_point_one() {
    // Design entries so anomaly avg >> clean avg
    let mut entries = Vec::new();

    // 20 clean entries (alice, weekday, non-round)
    for e in many_clean(20, "alice") {
        entries.push(e);
    }

    // 10 anomaly entries hitting many risk attributes
    for _ in 0..10 {
        // weekend (0.15) + round number (0.10) + rare user (0.15) = 0.40 per entry
        let mut e = make_je(
            "C001",
            saturday(),
            "6000",
            "2000",
            dec!(10000),
            "zz_once",
            TransactionSource::Automated,
        );
        e.header.is_anomaly = true;
        entries.push(e);
    }

    let result = score_entries(&entries);
    // Anomaly score ~0.40, clean score ~0 → separability > 0.10
    assert!(
        result.passes,
        "High-risk anomaly entries vs clean should produce separability > 0.10; got {}",
        result.anomaly_separability
    );
}

// ─── Percentage computation ───────────────────────────────────────────────────

#[test]
fn test_attribute_percentage_sums_bounded() {
    let mut entries = many_clean(10, "alice");
    entries.push(make_je("C001", saturday(), "6000", "2000", dec!(5000), "alice", TransactionSource::Automated));
    let result = score_entries(&entries);
    for attr in &result.risk_attributes {
        assert!(
            attr.percentage >= 0.0 && attr.percentage <= 100.0,
            "Percentage for {} out of range: {}",
            attr.attribute, attr.percentage
        );
    }
}

// ─── Score always in [0,1] ────────────────────────────────────────────────────

#[test]
fn test_result_is_serializable() {
    let entries = many_clean(5, "alice");
    let result = score_entries(&entries);
    let json = serde_json::to_string(&result).unwrap();
    let back: JeRiskScoringResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_entries, result.total_entries);
}
