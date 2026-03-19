//! Integration tests for the Expected Credit Loss (IFRS 9 / ASC 326) generator.

use datasynth_config::schema::EclConfig;
use datasynth_core::models::expected_credit_loss::{EclApproach, EclStage};
use datasynth_core::models::subledger::ar::AgingBucket;
use datasynth_generators::EclGenerator;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ============================================================================
// Helper constructors
// ============================================================================

fn default_config() -> EclConfig {
    EclConfig::default()
}

fn sample_bucket_exposures() -> Vec<(AgingBucket, Decimal)> {
    vec![
        (AgingBucket::Current, dec!(500_000)),
        (AgingBucket::Days1To30, dec!(120_000)),
        (AgingBucket::Days31To60, dec!(45_000)),
        (AgingBucket::Days61To90, dec!(15_000)),
        (AgingBucket::Over90Days, dec!(8_000)),
    ]
}

fn measurement_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid date")
}

// ============================================================================
// Test: ECL per bucket = exposure × applied_loss_rate
// ============================================================================

#[test]
fn test_provision_per_bucket_equals_exposure_times_rate() {
    let config = default_config();
    let mut gen = EclGenerator::new(42);
    let snap = gen.generate(
        "C001",
        measurement_date(),
        &sample_bucket_exposures(),
        &config,
        "2024-12",
        "IFRS_9",
    );

    assert_eq!(snap.ecl_models.len(), 1);
    let model = &snap.ecl_models[0];
    let matrix = model
        .provision_matrix
        .as_ref()
        .expect("provision matrix present");

    for row in &matrix.aging_buckets {
        let expected_provision = (row.exposure * row.applied_loss_rate).round_dp(2);
        assert_eq!(
            row.provision, expected_provision,
            "Bucket {:?}: provision {} != exposure {} × applied_rate {}",
            row.bucket, row.provision, row.exposure, row.applied_loss_rate
        );
    }
}

// ============================================================================
// Test: Sum of provisions = total ECL
// ============================================================================

#[test]
fn test_sum_of_provisions_equals_total_ecl() {
    let config = default_config();
    let mut gen = EclGenerator::new(99);
    let snap = gen.generate(
        "C001",
        measurement_date(),
        &sample_bucket_exposures(),
        &config,
        "2024-12",
        "IFRS_9",
    );

    let model = &snap.ecl_models[0];
    let matrix = model
        .provision_matrix
        .as_ref()
        .expect("provision matrix present");

    let sum_provisions: Decimal = matrix.aging_buckets.iter().map(|r| r.provision).sum();
    assert_eq!(
        sum_provisions, matrix.total_provision,
        "Sum of bucket provisions {} != total_provision {}",
        sum_provisions, matrix.total_provision
    );

    // Also verify total_ecl on the model matches
    assert_eq!(
        model.total_ecl, sum_provisions,
        "Model total_ecl {} != sum of provisions {}",
        model.total_ecl, sum_provisions
    );
}

// ============================================================================
// Test: Forward-looking adjustment changes the provision
// ============================================================================

#[test]
fn test_forward_looking_adjustment_changes_provision() {
    let buckets = sample_bucket_exposures();

    // Base config (blended multiplier = 0.5*1.0 + 0.3*0.8 + 0.2*1.4 = 1.02)
    let base_config = EclConfig::default();

    // Pessimistic config: put 100% weight on pessimistic multiplier 1.4
    let pessimistic_config = EclConfig {
        base_scenario_weight: 0.0,
        base_scenario_multiplier: 1.0,
        optimistic_scenario_weight: 0.0,
        optimistic_scenario_multiplier: 0.8,
        pessimistic_scenario_weight: 1.0,
        pessimistic_scenario_multiplier: 1.4,
        ..EclConfig::default()
    };

    // Optimistic config: put 100% weight on optimistic multiplier 0.8
    let optimistic_config = EclConfig {
        base_scenario_weight: 0.0,
        base_scenario_multiplier: 1.0,
        optimistic_scenario_weight: 1.0,
        optimistic_scenario_multiplier: 0.8,
        pessimistic_scenario_weight: 0.0,
        pessimistic_scenario_multiplier: 1.4,
        ..EclConfig::default()
    };

    let mut gen = EclGenerator::new(1);
    let date = measurement_date();

    let base_snap = gen.generate("C001", date, &buckets, &base_config, "2024-12", "IFRS_9");
    let pess_snap = gen.generate(
        "C001",
        date,
        &buckets,
        &pessimistic_config,
        "2024-12",
        "IFRS_9",
    );
    let opt_snap = gen.generate(
        "C001",
        date,
        &buckets,
        &optimistic_config,
        "2024-12",
        "IFRS_9",
    );

    let base_ecl = base_snap.ecl_models[0].total_ecl;
    let pess_ecl = pess_snap.ecl_models[0].total_ecl;
    let opt_ecl = opt_snap.ecl_models[0].total_ecl;

    assert!(
        pess_ecl > base_ecl,
        "Pessimistic ECL {} should exceed base ECL {}",
        pess_ecl,
        base_ecl
    );
    assert!(
        opt_ecl < pess_ecl,
        "Optimistic ECL {} should be less than pessimistic ECL {}",
        opt_ecl,
        pess_ecl
    );
}

// ============================================================================
// Test: Provision movement: opening + new_originations - write_offs = closing
// ============================================================================

#[test]
fn test_provision_movement_closing_balance() {
    let mut gen = EclGenerator::new(7);
    let config = default_config();
    let snap = gen.generate(
        "C001",
        measurement_date(),
        &sample_bucket_exposures(),
        &config,
        "2024-12",
        "IFRS_9",
    );

    assert_eq!(snap.provision_movements.len(), 1);
    let mov = &snap.provision_movements[0];

    let expected_closing = (mov.opening + mov.new_originations + mov.stage_transfers
        - mov.write_offs
        + mov.recoveries)
        .round_dp(2);

    assert_eq!(
        mov.closing, expected_closing,
        "closing {} != opening {} + new_originations {} + stage_transfers {} - write_offs {} + recoveries {}",
        mov.closing, mov.opening, mov.new_originations, mov.stage_transfers, mov.write_offs, mov.recoveries
    );
}

// ============================================================================
// Test: Generated JE is balanced
// ============================================================================

#[test]
fn test_ecl_journal_entry_is_balanced() {
    let mut gen = EclGenerator::new(5);
    let config = default_config();
    let snap = gen.generate(
        "C001",
        measurement_date(),
        &sample_bucket_exposures(),
        &config,
        "2024-12",
        "IFRS_9",
    );

    assert_eq!(snap.journal_entries.len(), 1);
    let je = &snap.journal_entries[0];
    assert!(
        je.is_balanced(),
        "JE should be balanced; debits = {:?}, credits = {:?}",
        je.total_debit(),
        je.total_credit()
    );
}

// ============================================================================
// Test: ECL model metadata is correct
// ============================================================================

#[test]
fn test_ecl_model_metadata() {
    let mut gen = EclGenerator::new(3);
    let config = default_config();
    let snap = gen.generate(
        "ACME",
        measurement_date(),
        &sample_bucket_exposures(),
        &config,
        "2024-Q4",
        "ASC_326",
    );

    let model = &snap.ecl_models[0];
    assert_eq!(model.entity_code, "ACME");
    assert_eq!(model.framework, "ASC_326");
    assert_eq!(model.approach, EclApproach::Simplified);
    assert_eq!(model.measurement_date, measurement_date());
    assert!(!model.id.is_empty());

    // Should have one portfolio segment
    assert_eq!(model.portfolio_segments.len(), 1);
    assert_eq!(
        model.portfolio_segments[0].segment_name,
        "Trade Receivables"
    );

    // Should have three stage allocations
    assert_eq!(model.portfolio_segments[0].staging.len(), 3);
    let stages: Vec<EclStage> = model.portfolio_segments[0]
        .staging
        .iter()
        .map(|s| s.stage)
        .collect();
    assert!(stages.contains(&EclStage::Stage1Month12));
    assert!(stages.contains(&EclStage::Stage2Lifetime));
    assert!(stages.contains(&EclStage::Stage3CreditImpaired));
}

// ============================================================================
// Test: Zero exposure produces zero ECL
// ============================================================================

#[test]
fn test_zero_exposure_produces_zero_ecl() {
    let zero_buckets: Vec<(AgingBucket, Decimal)> = AgingBucket::all()
        .into_iter()
        .map(|b| (b, Decimal::ZERO))
        .collect();

    let mut gen = EclGenerator::new(11);
    let config = default_config();
    let snap = gen.generate(
        "C001",
        measurement_date(),
        &zero_buckets,
        &config,
        "2024-12",
        "IFRS_9",
    );

    let model = &snap.ecl_models[0];
    assert_eq!(model.total_ecl, Decimal::ZERO);
    assert_eq!(model.total_exposure, Decimal::ZERO);

    // JE should have no lines (zero amount)
    let je = &snap.journal_entries[0];
    assert!(
        je.lines.is_empty(),
        "Zero ECL should produce empty JE lines"
    );
}

// ============================================================================
// Test: Provision movement P&L charge is consistent
// ============================================================================

#[test]
fn test_provision_movement_pl_charge() {
    let mut gen = EclGenerator::new(13);
    let config = default_config();
    let snap = gen.generate(
        "C001",
        measurement_date(),
        &sample_bucket_exposures(),
        &config,
        "2024-12",
        "IFRS_9",
    );

    let mov = &snap.provision_movements[0];
    let expected_pl =
        (mov.new_originations + mov.stage_transfers + mov.recoveries - mov.write_offs).round_dp(2);
    assert_eq!(
        mov.pl_charge, expected_pl,
        "P&L charge {} should equal new_originations {} + stage_transfers {} + recoveries {} - write_offs {}",
        mov.pl_charge, mov.new_originations, mov.stage_transfers, mov.recoveries, mov.write_offs
    );
}

// ============================================================================
// Test: Applied rate = historical rate × forward looking adjustment
// ============================================================================

#[test]
fn test_applied_rate_is_historical_times_adjustment() {
    let config = default_config();
    let mut gen = EclGenerator::new(17);
    let snap = gen.generate(
        "C001",
        measurement_date(),
        &sample_bucket_exposures(),
        &config,
        "2024-12",
        "IFRS_9",
    );

    let matrix = snap.ecl_models[0]
        .provision_matrix
        .as_ref()
        .expect("provision matrix present");

    for row in &matrix.aging_buckets {
        let expected_rate = (row.historical_loss_rate * row.forward_looking_adjustment).round_dp(6);
        assert_eq!(
            row.applied_loss_rate,
            expected_rate,
            "Bucket {:?}: applied_rate {} != historical {} × fla {}",
            row.bucket,
            row.applied_loss_rate,
            row.historical_loss_rate,
            row.forward_looking_adjustment
        );
    }
}
