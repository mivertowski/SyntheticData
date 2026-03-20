//! Integration tests for the provisions and contingencies generator
//! (IAS 37 / ASC 450).

use chrono::NaiveDate;
use datasynth_core::models::provision::ContingentProbability;
use datasynth_generators::ProvisionGenerator;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn reporting_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()
}

fn revenue_proxy() -> Decimal {
    dec!(10_000_000)
}

// ---------------------------------------------------------------------------
// Provision count
// ---------------------------------------------------------------------------

#[test]
fn test_provision_count_within_range() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    // At least 3 (backfill guarantee) and at most 10 (max loop count) + 3 backfill
    // Actual bound: the loop generates 3–10 candidates, then we top up to at least 3.
    assert!(
        snap.provisions.len() >= 3,
        "Expected at least 3 provisions, got {}",
        snap.provisions.len()
    );
    assert!(
        snap.provisions.len() <= 13,
        "Expected at most 13 provisions, got {}",
        snap.provisions.len()
    );
}

#[test]
fn test_provision_count_different_seeds() {
    // Different seeds should produce different (but valid) counts.
    for seed in [0u64, 1, 42, 999, 12345] {
        let mut gen = ProvisionGenerator::new(seed);
        let snap = gen.generate(
            "C001",
            "USD",
            revenue_proxy(),
            reporting_date(),
            "FY2024",
            "IFRS",
        None,
        );
        assert!(
            snap.provisions.len() >= 3,
            "seed={seed}: expected >= 3 provisions"
        );
    }
}

// ---------------------------------------------------------------------------
// Framework-aware recognition
// ---------------------------------------------------------------------------

#[test]
fn test_ifrs_provisions_for_probable_items() {
    // IFRS recognises at > 50%.  US GAAP at > 75%.
    // With the same seed, IFRS should produce >= as many provisions as US GAAP
    // (looser threshold → more items recognised).
    let mut gen_ifrs = ProvisionGenerator::new(77);
    let snap_ifrs = gen_ifrs.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    let mut gen_gaap = ProvisionGenerator::new(77);
    let snap_gaap = gen_gaap.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "US_GAAP",
        None,
    );

    // Framework recorded on each provision must match.
    for p in &snap_ifrs.provisions {
        assert_eq!(p.framework, "IFRS");
    }
    for p in &snap_gaap.provisions {
        assert_eq!(p.framework, "US_GAAP");
    }

    // IFRS threshold is lower → should recognise at least as many as US GAAP.
    // (Backfill guarantees minimum 3 for both, so this holds.)
    assert!(
        snap_ifrs.provisions.len() >= snap_gaap.provisions.len(),
        "IFRS ({}) should have >= provisions vs US GAAP ({})",
        snap_ifrs.provisions.len(),
        snap_gaap.provisions.len()
    );
}

// ---------------------------------------------------------------------------
// Movement balance check
// ---------------------------------------------------------------------------

#[test]
fn test_movement_balance_identity() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    for mv in &snap.movements {
        let expected_closing =
            mv.opening + mv.additions - mv.utilizations - mv.reversals + mv.unwinding_of_discount;
        assert_eq!(
            mv.closing,
            expected_closing.max(Decimal::ZERO),
            "Movement balance identity failed for provision_id={}",
            mv.provision_id
        );
    }
}

#[test]
fn test_movement_non_negative_closing() {
    for seed in [0u64, 1, 42, 99, 1234] {
        let mut gen = ProvisionGenerator::new(seed);
        let snap = gen.generate(
            "C001",
            "USD",
            revenue_proxy(),
            reporting_date(),
            "FY2024",
            "US_GAAP",
        None,
        );
        for mv in &snap.movements {
            assert!(
                mv.closing >= Decimal::ZERO,
                "seed={seed}: negative closing balance in provision_id={}",
                mv.provision_id
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Journal entry balance
// ---------------------------------------------------------------------------

#[test]
fn test_journal_entries_balanced() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    for je in &snap.journal_entries {
        let total_debits: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
        let total_credits: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
        assert_eq!(
            total_debits, total_credits,
            "Unbalanced JE: doc_id={} debits={} credits={}",
            je.header.document_id, total_debits, total_credits
        );
    }
}

#[test]
fn test_journal_entries_use_correct_accounts() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    // Each recognition JE should have a debit to 6850 (provision expense)
    // and a credit to 2450 (provision liability).
    for je in &snap.journal_entries {
        let debit_accounts: Vec<&str> = je
            .lines
            .iter()
            .filter(|l| l.debit_amount > Decimal::ZERO)
            .map(|l| l.gl_account.as_str())
            .collect();
        let credit_accounts: Vec<&str> = je
            .lines
            .iter()
            .filter(|l| l.credit_amount > Decimal::ZERO)
            .map(|l| l.gl_account.as_str())
            .collect();

        assert!(
            debit_accounts.contains(&"6850") || debit_accounts.contains(&"7100"),
            "Expected debit to 6850 or 7100, got {:?}",
            debit_accounts
        );
        assert!(
            credit_accounts.contains(&"2450"),
            "Expected credit to 2450, got {:?}",
            credit_accounts
        );
    }
}

// ---------------------------------------------------------------------------
// Contingent liabilities
// ---------------------------------------------------------------------------

#[test]
fn test_contingent_liabilities_count() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    assert!(
        (1..=3).contains(&snap.contingent_liabilities.len()),
        "Expected 1–3 contingent liabilities, got {}",
        snap.contingent_liabilities.len()
    );
}

#[test]
fn test_contingent_liabilities_disclosure_only() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    for cl in &snap.contingent_liabilities {
        assert!(
            cl.disclosure_only,
            "contingent_liability {} should have disclosure_only = true",
            cl.id
        );
    }
}

#[test]
fn test_contingent_liabilities_possible_probability() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "US_GAAP",
        None,
    );

    for cl in &snap.contingent_liabilities {
        assert_eq!(
            cl.probability,
            ContingentProbability::Possible,
            "contingent_liability {} should be Possible, got {:?}",
            cl.id,
            cl.probability
        );
    }
}

// ---------------------------------------------------------------------------
// Range checks
// ---------------------------------------------------------------------------

#[test]
fn test_provision_range_order() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    for p in &snap.provisions {
        assert!(
            p.range_low <= p.best_estimate,
            "range_low ({}) > best_estimate ({}) for provision {}",
            p.range_low,
            p.best_estimate,
            p.id
        );
        assert!(
            p.best_estimate <= p.range_high,
            "best_estimate ({}) > range_high ({}) for provision {}",
            p.best_estimate,
            p.range_high,
            p.id
        );
    }
}

#[test]
fn test_provision_amounts_positive() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    for p in &snap.provisions {
        assert!(
            p.best_estimate > Decimal::ZERO,
            "best_estimate must be positive, got {} for provision {}",
            p.best_estimate,
            p.id
        );
    }
}

// ---------------------------------------------------------------------------
// IDs are unique
// ---------------------------------------------------------------------------

#[test]
fn test_provision_ids_unique() {
    use std::collections::HashSet;

    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    let ids: HashSet<&str> = snap.provisions.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        snap.provisions.len(),
        "Duplicate provision IDs detected"
    );
}

// ---------------------------------------------------------------------------
// Provision type variety
// ---------------------------------------------------------------------------

#[test]
fn test_provision_types_present() {
    // With a sufficient sample across seeds, we should see more than one type.
    let mut types_seen = std::collections::HashSet::new();
    for seed in 0u64..20 {
        let mut gen = ProvisionGenerator::new(seed);
        let snap = gen.generate(
            "C001",
            "USD",
            revenue_proxy(),
            reporting_date(),
            "FY2024",
            "IFRS",
        None,
        );
        for p in &snap.provisions {
            types_seen.insert(format!("{:?}", p.provision_type));
        }
    }
    assert!(
        types_seen.len() >= 2,
        "Expected at least 2 distinct provision types across seeds, got {:?}",
        types_seen
    );
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn test_determinism() {
    let mut gen1 = ProvisionGenerator::new(42);
    let snap1 = gen1.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    let mut gen2 = ProvisionGenerator::new(42);
    let snap2 = gen2.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    assert_eq!(snap1.provisions.len(), snap2.provisions.len());
    for (p1, p2) in snap1.provisions.iter().zip(snap2.provisions.iter()) {
        assert_eq!(p1.id, p2.id);
        assert_eq!(p1.best_estimate, p2.best_estimate);
    }
}

// ---------------------------------------------------------------------------
// Movement count matches provision count
// ---------------------------------------------------------------------------

#[test]
fn test_movement_count_matches_provision_count() {
    let mut gen = ProvisionGenerator::new(42);
    let snap = gen.generate(
        "C001",
        "USD",
        revenue_proxy(),
        reporting_date(),
        "FY2024",
        "IFRS",
        None,
    );

    assert_eq!(
        snap.provisions.len(),
        snap.movements.len(),
        "Each provision should have exactly one movement roll-forward"
    );
}

// ---------------------------------------------------------------------------
// Discount rate on long-term provisions
// ---------------------------------------------------------------------------

#[test]
fn test_long_term_provisions_may_have_discount_rate() {
    // Run many seeds and check that at least some provisions have a discount_rate.
    let mut found_discounted = false;
    for seed in 0u64..50 {
        let mut gen = ProvisionGenerator::new(seed);
        let snap = gen.generate(
            "C001",
            "USD",
            revenue_proxy(),
            reporting_date(),
            "FY2024",
            "IFRS",
        None,
        );
        for p in &snap.provisions {
            if p.discount_rate.is_some() {
                found_discounted = true;
                let rate = p.discount_rate.unwrap();
                assert!(
                    rate >= dec!(0.03) && rate <= dec!(0.05),
                    "Discount rate {rate} out of expected 3–5% range"
                );
            }
        }
    }
    assert!(
        found_discounted,
        "Expected at least one long-term provision with a discount_rate across 50 seeds"
    );
}
