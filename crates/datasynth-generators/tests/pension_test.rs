//! Integration tests for the IAS 19 / ASC 715 defined benefit pension generator.

use chrono::NaiveDate;
use datasynth_generators::PensionGenerator;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Helper — run the generator with a fixed seed and return the snapshot.
///
/// Uses `avg_salary = None` (falls back to $50k) and `period_months = 12`
/// so pension expense is not prorated and existing identity assertions hold.
fn make_snapshot() -> datasynth_generators::pension_generator::PensionSnapshot {
    let mut gen = PensionGenerator::new(42);
    gen.generate(
        "1000",
        "Acme Corp",
        "FY2024",
        NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid date"),
        200, // employees
        "USD",
        None, // avg_salary — use default $50 000
        12,   // period_months — full year, no proration
    )
}

/// Helper — run with a specific avg_salary and period_months for proration tests.
fn make_snapshot_with(
    avg_salary: Option<Decimal>,
    period_months: u32,
) -> datasynth_generators::pension_generator::PensionSnapshot {
    let mut gen = PensionGenerator::new(42);
    gen.generate(
        "1000",
        "Acme Corp",
        "FY2024",
        NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid date"),
        200,
        "USD",
        avg_salary,
        period_months,
    )
}

// ============================================================================
// Model invariants
// ============================================================================

#[test]
fn participant_count_is_positive() {
    let snap = make_snapshot();
    assert!(!snap.plans.is_empty(), "should generate at least one plan");
    for plan in &snap.plans {
        assert!(
            plan.participant_count > 0,
            "participant_count must be > 0, got {}",
            plan.participant_count
        );
    }
}

#[test]
fn actuarial_rates_are_in_expected_ranges() {
    let snap = make_snapshot();
    for plan in &snap.plans {
        let a = &plan.assumptions;
        assert!(
            a.discount_rate >= dec!(0.02) && a.discount_rate <= dec!(0.06),
            "discount_rate out of range: {}",
            a.discount_rate
        );
        assert!(
            a.salary_growth_rate >= dec!(0.01) && a.salary_growth_rate <= dec!(0.05),
            "salary_growth_rate out of range: {}",
            a.salary_growth_rate
        );
        assert!(
            a.expected_return_on_plan_assets >= dec!(0.03)
                && a.expected_return_on_plan_assets <= dec!(0.10),
            "expected_return_on_plan_assets out of range: {}",
            a.expected_return_on_plan_assets
        );
    }
}

// ============================================================================
// DBO roll-forward identity
// ============================================================================

#[test]
fn dbo_closing_equals_roll_forward_identity() {
    let snap = make_snapshot();
    assert!(
        !snap.obligations.is_empty(),
        "should generate at least one obligation"
    );
    for ob in &snap.obligations {
        let computed =
            (ob.dbo_opening + ob.service_cost + ob.interest_cost + ob.actuarial_gains_losses
                - ob.benefits_paid)
                .round_dp(2);
        assert_eq!(
            ob.dbo_closing, computed,
            "DBO closing identity failed: {} ≠ {} (opening={}, service={}, interest={}, actuarial={}, benefits={})",
            ob.dbo_closing, computed,
            ob.dbo_opening, ob.service_cost, ob.interest_cost,
            ob.actuarial_gains_losses, ob.benefits_paid
        );
    }
}

#[test]
fn dbo_opening_is_positive() {
    let snap = make_snapshot();
    for ob in &snap.obligations {
        assert!(
            ob.dbo_opening > Decimal::ZERO,
            "dbo_opening should be positive, got {}",
            ob.dbo_opening
        );
    }
}

#[test]
fn service_cost_is_positive() {
    let snap = make_snapshot();
    for ob in &snap.obligations {
        assert!(
            ob.service_cost > Decimal::ZERO,
            "service_cost should be positive, got {}",
            ob.service_cost
        );
    }
}

#[test]
fn interest_cost_is_positive() {
    let snap = make_snapshot();
    for ob in &snap.obligations {
        assert!(
            ob.interest_cost > Decimal::ZERO,
            "interest_cost should be positive, got {}",
            ob.interest_cost
        );
    }
}

// ============================================================================
// Plan assets roll-forward identity
// ============================================================================

#[test]
fn plan_assets_closing_equals_roll_forward_identity() {
    let snap = make_snapshot();
    assert!(
        !snap.plan_assets.is_empty(),
        "should generate at least one plan assets record"
    );
    for pa in &snap.plan_assets {
        let computed = (pa.fair_value_opening
            + pa.expected_return
            + pa.actuarial_gain_loss
            + pa.employer_contributions
            - pa.benefits_paid)
            .round_dp(2);
        assert_eq!(
            pa.fair_value_closing, computed,
            "Plan assets closing identity failed: {} ≠ {} (opening={}, return={}, actuarial={}, contributions={}, benefits={})",
            pa.fair_value_closing, computed,
            pa.fair_value_opening, pa.expected_return,
            pa.actuarial_gain_loss, pa.employer_contributions, pa.benefits_paid
        );
    }
}

#[test]
fn plan_assets_opening_is_positive() {
    let snap = make_snapshot();
    for pa in &snap.plan_assets {
        assert!(
            pa.fair_value_opening > Decimal::ZERO,
            "fair_value_opening should be positive, got {}",
            pa.fair_value_opening
        );
    }
}

// ============================================================================
// Disclosure identities
// ============================================================================

#[test]
fn net_liability_equals_dbo_minus_assets() {
    let snap = make_snapshot();
    assert!(
        !snap.disclosures.is_empty(),
        "should generate at least one disclosure"
    );
    for disc in &snap.disclosures {
        // Find matching obligation and plan assets
        let ob = snap
            .obligations
            .iter()
            .find(|o| o.plan_id == disc.plan_id)
            .expect("matching obligation");
        let pa = snap
            .plan_assets
            .iter()
            .find(|a| a.plan_id == disc.plan_id)
            .expect("matching plan assets");

        let expected = (ob.dbo_closing - pa.fair_value_closing).round_dp(2);
        assert_eq!(
            disc.net_pension_liability, expected,
            "net_pension_liability ({}) ≠ DBO ({}) − plan_assets ({})",
            disc.net_pension_liability, ob.dbo_closing, pa.fair_value_closing
        );
    }
}

#[test]
fn pension_expense_equals_service_plus_interest_minus_expected_return() {
    let snap = make_snapshot();
    for disc in &snap.disclosures {
        let ob = snap
            .obligations
            .iter()
            .find(|o| o.plan_id == disc.plan_id)
            .expect("matching obligation");
        let pa = snap
            .plan_assets
            .iter()
            .find(|a| a.plan_id == disc.plan_id)
            .expect("matching plan assets");

        let expected = (ob.service_cost + ob.interest_cost - pa.expected_return).round_dp(2);
        assert_eq!(
            disc.pension_expense, expected,
            "pension_expense ({}) ≠ service_cost + interest_cost − expected_return ({} + {} − {} = {})",
            disc.pension_expense, ob.service_cost, ob.interest_cost, pa.expected_return, expected
        );
    }
}

#[test]
fn funding_ratio_is_in_plausible_range() {
    let snap = make_snapshot();
    for disc in &snap.disclosures {
        // Funding ratio should be between 0.50 and 2.0 for normal pension plans
        assert!(
            disc.funding_ratio > dec!(0.30) && disc.funding_ratio < dec!(2.5),
            "funding_ratio out of plausible range: {}",
            disc.funding_ratio
        );
    }
}

// ============================================================================
// Journal entry balance check
// ============================================================================

#[test]
fn pension_expense_je_is_balanced() {
    let snap = make_snapshot();
    assert!(
        !snap.journal_entries.is_empty(),
        "should generate at least one journal entry"
    );
    for je in &snap.journal_entries {
        let total_debit: Decimal = je.lines.iter().map(|l| l.debit_amount).sum();
        let total_credit: Decimal = je.lines.iter().map(|l| l.credit_amount).sum();
        assert_eq!(
            total_debit, total_credit,
            "Journal entry '{}' is not balanced: DR={} CR={}",
            je.header.document_id, total_debit, total_credit
        );
    }
}

#[test]
fn pension_je_line_count_is_two() {
    let snap = make_snapshot();
    for je in &snap.journal_entries {
        assert_eq!(
            je.lines.len(),
            2,
            "Each pension JE should have exactly 2 lines, got {} for '{}'",
            je.lines.len(),
            je.header.document_id
        );
    }
}

// ============================================================================
// Determinism
// ============================================================================

#[test]
fn generator_is_deterministic() {
    let snap1 = make_snapshot();
    let snap2 = make_snapshot();

    assert_eq!(
        snap1.plans.len(),
        snap2.plans.len(),
        "plan count differs between runs"
    );
    if let (Some(p1), Some(p2)) = (snap1.plans.first(), snap2.plans.first()) {
        assert_eq!(
            p1.assumptions.discount_rate, p2.assumptions.discount_rate,
            "discount_rate differs between runs"
        );
    }
    if let (Some(o1), Some(o2)) = (snap1.obligations.first(), snap2.obligations.first()) {
        assert_eq!(
            o1.dbo_closing, o2.dbo_closing,
            "dbo_closing differs between runs"
        );
    }
}

// ============================================================================
// OCI remeasurements sign convention
// ============================================================================

#[test]
fn oci_remeasurements_formula() {
    // OCI = obligation actuarial G/L − asset actuarial G/L
    let snap = make_snapshot();
    for disc in &snap.disclosures {
        let ob = snap
            .obligations
            .iter()
            .find(|o| o.plan_id == disc.plan_id)
            .expect("matching obligation");
        let pa = snap
            .plan_assets
            .iter()
            .find(|a| a.plan_id == disc.plan_id)
            .expect("matching plan assets");

        let expected = (ob.actuarial_gains_losses - pa.actuarial_gain_loss).round_dp(2);
        assert_eq!(
            disc.oci_remeasurements, expected,
            "oci_remeasurements ({}) ≠ obligation_actuarial ({}) − asset_actuarial ({})",
            disc.oci_remeasurements, ob.actuarial_gains_losses, pa.actuarial_gain_loss
        );
    }
}

// ============================================================================
// New features: avg_salary, period proration, small participant counts
// ============================================================================

#[test]
fn small_company_gets_realistic_participant_count() {
    // A company with 5 employees should get 5 participants (not clamped to 50).
    let mut gen = PensionGenerator::new(99);
    let snap = gen.generate(
        "SMALL",
        "Tiny Co",
        "FY2024",
        NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid date"),
        5,
        "USD",
        None,
        12,
    );
    assert!(!snap.plans.is_empty());
    assert_eq!(
        snap.plans[0].participant_count, 5,
        "Small company should have participant_count = employee_count (5), got {}",
        snap.plans[0].participant_count
    );
}

#[test]
fn period_proration_reduces_pension_expense() {
    // 6-month period should produce half the pension expense of a full year.
    let snap_12 = make_snapshot_with(None, 12);
    let snap_6 = make_snapshot_with(None, 6);

    let expense_12 = snap_12.disclosures[0].pension_expense;
    let expense_6 = snap_6.disclosures[0].pension_expense;

    // 6 months should be approximately half (allow 1 cent rounding)
    let half = (expense_12 / Decimal::from(2u32)).round_dp(2);
    let diff = (expense_6 - half).abs();
    assert!(
        diff <= dec!(0.01),
        "6-month expense ({}) should be half of 12-month expense ({}) → expected ~{}, diff={}",
        expense_6,
        expense_12,
        half,
        diff
    );
}

#[test]
fn avg_salary_affects_dbo_and_service_cost() {
    // Higher avg salary → higher DBO and service cost.
    let snap_low = make_snapshot_with(Some(dec!(30000)), 12);
    let snap_high = make_snapshot_with(Some(dec!(100000)), 12);

    let dbo_low = snap_low.obligations[0].dbo_opening;
    let dbo_high = snap_high.obligations[0].dbo_opening;
    assert!(
        dbo_high > dbo_low,
        "Higher avg_salary should produce a larger DBO: {} > {} failed",
        dbo_high,
        dbo_low
    );

    let sc_low = snap_low.obligations[0].service_cost;
    let sc_high = snap_high.obligations[0].service_cost;
    assert!(
        sc_high > sc_low,
        "Higher avg_salary should produce a larger service_cost: {} > {} failed",
        sc_high,
        sc_low
    );
}
