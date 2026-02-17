//! Integration tests for the full treasury data generation pipeline.
//!
//! Validates end-to-end workflows: cash flows -> cash positions -> cash forecast ->
//! hedging instruments -> debt instruments with covenants -> cash pool sweeps ->
//! anomaly injection. Checks cross-generator consistency, determinism, balance
//! chaining, forecast probability decay, and hedge effectiveness corridors.

#![allow(clippy::unwrap_used)]

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use datasynth_config::schema::{
    CashForecastingConfig, CashPoolingConfig, CashPositioningConfig, CovenantDef,
    DebtInstrumentDef, DebtSchemaConfig, HedgingSchemaConfig,
};
use datasynth_core::models::{
    CashPosition, DebtType, HedgeType, HedgedItemType, InterestRateType, PoolType,
    TreasuryCashFlowCategory,
};
use datasynth_generators::treasury::{
    AccountBalance, ApAgingItem, ArAgingItem, CashFlow, CashFlowDirection,
    CashForecastGenerator, CashPoolGenerator, CashPositionGenerator, DebtGenerator,
    FxExposure, HedgingGenerator, ScheduledDisbursement, TreasuryAnomalyInjector,
    TreasuryAnomalyType,
};

// =============================================================================
// Helpers
// =============================================================================

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

/// Generates a realistic set of cash flows simulating AP payments and AR receipts
/// over a 30-day period.
fn generate_mock_cash_flows() -> Vec<CashFlow> {
    let mut flows = Vec::new();

    // AR receipts (inflows) — simulate customer payments
    for i in 0..15 {
        flows.push(CashFlow {
            date: d("2025-01-02") + chrono::Duration::days(i * 2),
            account_id: "BA-001".to_string(),
            amount: dec!(5000) + Decimal::from(i) * dec!(500),
            direction: CashFlowDirection::Inflow,
        });
    }

    // AP payments (outflows) — simulate vendor payments
    for i in 0..10 {
        flows.push(CashFlow {
            date: d("2025-01-03") + chrono::Duration::days(i * 3),
            account_id: "BA-001".to_string(),
            amount: dec!(3000) + Decimal::from(i) * dec!(1000),
            direction: CashFlowDirection::Outflow,
        });
    }

    // EUR account flows
    for i in 0..5 {
        flows.push(CashFlow {
            date: d("2025-01-05") + chrono::Duration::days(i * 7),
            account_id: "BA-002".to_string(),
            amount: dec!(10000) + Decimal::from(i) * dec!(2000),
            direction: CashFlowDirection::Inflow,
        });
    }
    for i in 0..3 {
        flows.push(CashFlow {
            date: d("2025-01-10") + chrono::Duration::days(i * 10),
            account_id: "BA-002".to_string(),
            amount: dec!(8000) + Decimal::from(i) * dec!(3000),
            direction: CashFlowDirection::Outflow,
        });
    }

    flows
}

/// Creates a debt config with a term loan and covenants for testing.
fn test_debt_config() -> DebtSchemaConfig {
    DebtSchemaConfig {
        enabled: true,
        instruments: vec![
            DebtInstrumentDef {
                instrument_type: "term_loan".to_string(),
                principal: Some(5_000_000.0),
                rate: Some(0.055),
                maturity_months: Some(60),
                facility: None,
            },
            DebtInstrumentDef {
                instrument_type: "revolving_credit".to_string(),
                principal: None,
                rate: Some(0.045),
                maturity_months: Some(36),
                facility: Some(2_000_000.0),
            },
        ],
        covenants: vec![
            CovenantDef {
                covenant_type: "debt_to_ebitda".to_string(),
                threshold: 3.5,
            },
            CovenantDef {
                covenant_type: "interest_coverage".to_string(),
                threshold: 3.0,
            },
        ],
    }
}

// =============================================================================
// 1. Full Pipeline Test
// =============================================================================

#[test]
fn test_full_treasury_pipeline() {
    // Step 1: Generate cash positions from mock payment flows
    let flows = generate_mock_cash_flows();
    let accounts = vec![
        ("BA-001".to_string(), "USD".to_string(), dec!(100000)),
        ("BA-002".to_string(), "EUR".to_string(), dec!(50000)),
    ];
    let mut pos_gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
    let positions = pos_gen.generate_multi_account(
        "C001",
        &accounts,
        &flows,
        d("2025-01-01"),
        d("2025-01-31"),
    );
    assert!(
        !positions.is_empty(),
        "Should produce cash positions"
    );

    // Step 2: Generate cash forecasts from AR/AP aging
    let ar_items = vec![
        ArAgingItem {
            expected_date: d("2025-02-15"),
            amount: dec!(50000),
            days_past_due: 0,
            document_id: "INV-001".to_string(),
        },
        ArAgingItem {
            expected_date: d("2025-02-20"),
            amount: dec!(30000),
            days_past_due: 45,
            document_id: "INV-002".to_string(),
        },
        ArAgingItem {
            expected_date: d("2025-03-01"),
            amount: dec!(20000),
            days_past_due: 95,
            document_id: "INV-003".to_string(),
        },
    ];
    let ap_items = vec![
        ApAgingItem {
            payment_date: d("2025-02-10"),
            amount: dec!(40000),
            document_id: "VI-001".to_string(),
        },
        ApAgingItem {
            payment_date: d("2025-02-28"),
            amount: dec!(25000),
            document_id: "VI-002".to_string(),
        },
    ];
    let disbursements = vec![ScheduledDisbursement {
        date: d("2025-02-28"),
        amount: dec!(80000),
        category: TreasuryCashFlowCategory::PayrollDisbursement,
        description: "February payroll".to_string(),
    }];

    let mut forecast_gen = CashForecastGenerator::new(43, CashForecastingConfig::default());
    let forecast = forecast_gen.generate(
        "C001",
        "USD",
        d("2025-01-31"),
        &ar_items,
        &ap_items,
        &disbursements,
    );
    assert!(
        !forecast.items.is_empty(),
        "Should produce forecast items"
    );

    // Step 3: Generate hedging instruments from FX exposures
    let exposures = vec![
        FxExposure {
            currency_pair: "EUR/USD".to_string(),
            foreign_currency: "EUR".to_string(),
            net_amount: dec!(500000),
            settlement_date: d("2025-06-30"),
            description: "EUR receivables Q2".to_string(),
        },
        FxExposure {
            currency_pair: "GBP/USD".to_string(),
            foreign_currency: "GBP".to_string(),
            net_amount: dec!(-200000),
            settlement_date: d("2025-06-30"),
            description: "GBP payables Q2".to_string(),
        },
    ];
    let mut hedge_gen = HedgingGenerator::new(44, HedgingSchemaConfig::default());
    let (instruments, relationships) = hedge_gen.generate(d("2025-01-15"), &exposures);
    assert_eq!(instruments.len(), 2);
    assert_eq!(relationships.len(), 2);

    // Step 4: Generate debt instruments with covenants
    let mut debt_gen = DebtGenerator::new(45, test_debt_config());
    let debt_instruments = debt_gen.generate("C001", "USD", d("2025-01-01"));
    assert_eq!(debt_instruments.len(), 2);

    // Step 5: Generate cash pool sweeps
    let mut pool_gen = CashPoolGenerator::new(46, CashPoolingConfig::default());
    let pool = pool_gen
        .create_pool(
            "USD Master Pool",
            "USD",
            &[
                "BA-HEADER".to_string(),
                "BA-001".to_string(),
                "BA-002".to_string(),
            ],
        )
        .unwrap();
    let eod_balances = vec![
        AccountBalance {
            account_id: "BA-001".to_string(),
            balance: dec!(75000),
        },
        AccountBalance {
            account_id: "BA-002".to_string(),
            balance: dec!(-15000),
        },
    ];
    let sweeps = pool_gen.generate_sweeps(&pool, d("2025-01-31"), "USD", &eod_balances);
    assert_eq!(sweeps.len(), 2);

    // Step 6: Inject anomalies into cash positions
    let mut usd_positions: Vec<CashPosition> = positions
        .iter()
        .filter(|p| p.bank_account_id == "BA-001")
        .cloned()
        .collect();
    let mut anomaly_injector = TreasuryAnomalyInjector::new(47, 0.15);
    let pos_anomaly_labels =
        anomaly_injector.inject_into_cash_positions(&mut usd_positions, dec!(50000));

    // Step 7: Inject anomalies into hedge relationships
    let mut rels = relationships;
    let hedge_anomaly_labels = anomaly_injector.inject_into_hedge_relationships(&mut rels);

    // Step 8: Inject anomalies into debt covenants
    let mut covenants: Vec<_> = debt_instruments
        .iter()
        .flat_map(|d| d.covenants.clone())
        .collect();
    let cov_anomaly_labels = anomaly_injector.inject_into_debt_covenants(&mut covenants);

    // Verify all anomaly labels have required fields
    let all_labels: Vec<_> = pos_anomaly_labels
        .iter()
        .chain(hedge_anomaly_labels.iter())
        .chain(cov_anomaly_labels.iter())
        .collect();

    for label in &all_labels {
        assert!(!label.id.is_empty(), "Anomaly label should have an ID");
        assert!(
            !label.document_type.is_empty(),
            "Anomaly label should have a document_type"
        );
        assert!(
            !label.document_id.is_empty(),
            "Anomaly label should have a document_id"
        );
        assert!(
            !label.description.is_empty(),
            "Anomaly label should have a description"
        );
    }

    // Verify the pipeline produced all expected artifact types
    assert!(!positions.is_empty(), "Cash positions present");
    assert!(!forecast.items.is_empty(), "Forecast items present");
    assert!(!instruments.is_empty(), "Hedging instruments present");
    assert!(!debt_instruments.is_empty(), "Debt instruments present");
    assert!(!sweeps.is_empty(), "Pool sweeps present");
}

// =============================================================================
// 2. Position Balance Chaining Tests
// =============================================================================

#[test]
fn test_position_balances_chain_day_to_day() {
    let flows = generate_mock_cash_flows();
    let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
    let positions = gen.generate(
        "C001",
        "BA-001",
        "USD",
        &flows,
        d("2025-01-01"),
        d("2025-01-31"),
        dec!(100000),
    );

    assert!(positions.len() > 1, "Need multiple days to test chaining");

    // Verify day-to-day chaining: each day's opening = previous day's closing
    for window in positions.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        assert_eq!(
            curr.opening_balance, prev.closing_balance,
            "Day {} opening ({}) should equal day {} closing ({})",
            curr.date, curr.opening_balance, prev.date, prev.closing_balance
        );
    }
}

#[test]
fn test_position_closing_balance_formula() {
    let flows = generate_mock_cash_flows();
    let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
    let positions = gen.generate(
        "C001",
        "BA-001",
        "USD",
        &flows,
        d("2025-01-01"),
        d("2025-01-31"),
        dec!(100000),
    );

    for pos in &positions {
        let expected = pos.opening_balance + pos.inflows - pos.outflows;
        assert_eq!(
            pos.closing_balance, expected,
            "Day {}: closing {} != opening {} + inflows {} - outflows {}",
            pos.date, pos.closing_balance, pos.opening_balance, pos.inflows, pos.outflows
        );
    }
}

#[test]
fn test_position_available_balance_bounded() {
    let flows = generate_mock_cash_flows();
    let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
    let positions = gen.generate(
        "C001",
        "BA-001",
        "USD",
        &flows,
        d("2025-01-01"),
        d("2025-01-31"),
        dec!(100000),
    );

    for pos in &positions {
        assert!(
            pos.available_balance <= pos.closing_balance,
            "Day {}: available {} should be <= closing {}",
            pos.date, pos.available_balance, pos.closing_balance
        );
        assert!(
            pos.available_balance >= Decimal::ZERO,
            "Day {}: available {} should be >= 0",
            pos.date, pos.available_balance
        );
    }
}

// =============================================================================
// 3. Forecast Probability Decay Tests
// =============================================================================

#[test]
fn test_forecast_probability_decays_with_aging() {
    let mut gen = CashForecastGenerator::new(42, CashForecastingConfig::default());

    // Create AR items at different aging buckets
    let ar_items = vec![
        ArAgingItem {
            expected_date: d("2025-02-15"),
            amount: dec!(10000),
            days_past_due: 0,
            document_id: "INV-CURRENT".to_string(),
        },
        ArAgingItem {
            expected_date: d("2025-02-16"),
            amount: dec!(10000),
            days_past_due: 25,
            document_id: "INV-30DPD".to_string(),
        },
        ArAgingItem {
            expected_date: d("2025-02-17"),
            amount: dec!(10000),
            days_past_due: 50,
            document_id: "INV-60DPD".to_string(),
        },
        ArAgingItem {
            expected_date: d("2025-02-18"),
            amount: dec!(10000),
            days_past_due: 80,
            document_id: "INV-90DPD".to_string(),
        },
        ArAgingItem {
            expected_date: d("2025-02-19"),
            amount: dec!(10000),
            days_past_due: 120,
            document_id: "INV-120DPD".to_string(),
        },
    ];

    let forecast = gen.generate("C001", "USD", d("2025-01-31"), &ar_items, &[], &[]);

    let find_prob = |doc_id: &str| -> Decimal {
        forecast
            .items
            .iter()
            .find(|i| i.source_document_id.as_deref() == Some(doc_id))
            .map(|i| i.probability)
            .unwrap_or(Decimal::ZERO)
    };

    let prob_current = find_prob("INV-CURRENT");
    let _prob_30 = find_prob("INV-30DPD");
    let prob_60 = find_prob("INV-60DPD");
    let _prob_90 = find_prob("INV-90DPD");
    let prob_120 = find_prob("INV-120DPD");

    // Probabilities should generally decrease with aging
    // (with small jitter, so we check the overall trend rather than strict monotonicity)
    assert!(
        prob_current > prob_60,
        "Current ({}) should have higher probability than 60 DPD ({})",
        prob_current, prob_60
    );
    assert!(
        prob_60 > prob_120,
        "60 DPD ({}) should have higher probability than 120 DPD ({})",
        prob_60, prob_120
    );

    // Most overdue items should have much lower probability than current
    assert!(
        prob_120 < dec!(0.30),
        "120+ DPD items should have probability < 30%, got {}",
        prob_120
    );
    assert!(
        prob_current > dec!(0.80),
        "Current items should have probability > 80%, got {}",
        prob_current
    );

    // Check boundaries: all probabilities within valid range
    for item in &forecast.items {
        assert!(
            item.probability >= dec!(0.05) && item.probability <= dec!(1.00),
            "Probability {} out of valid range for item {}",
            item.probability, item.id
        );
    }
}

#[test]
fn test_forecast_ap_payments_near_certain() {
    let mut gen = CashForecastGenerator::new(42, CashForecastingConfig::default());
    let ap_items = vec![
        ApAgingItem {
            payment_date: d("2025-02-10"),
            amount: dec!(30000),
            document_id: "VI-001".to_string(),
        },
        ApAgingItem {
            payment_date: d("2025-02-28"),
            amount: dec!(50000),
            document_id: "VI-002".to_string(),
        },
    ];

    let forecast = gen.generate("C001", "USD", d("2025-01-31"), &[], &ap_items, &[]);

    for item in &forecast.items {
        assert_eq!(
            item.category,
            TreasuryCashFlowCategory::ApPayment,
        );
        assert_eq!(
            item.probability,
            dec!(0.95),
            "AP payments should be at 95% probability, got {}",
            item.probability
        );
        assert!(
            item.amount < Decimal::ZERO,
            "AP items should be negative (outflow), got {}",
            item.amount
        );
    }
}

#[test]
fn test_forecast_net_position_coherent() {
    let mut gen = CashForecastGenerator::new(42, CashForecastingConfig::default());
    let ar_items = vec![ArAgingItem {
        expected_date: d("2025-02-15"),
        amount: dec!(100000),
        days_past_due: 0,
        document_id: "INV-001".to_string(),
    }];
    let ap_items = vec![ApAgingItem {
        payment_date: d("2025-02-10"),
        amount: dec!(60000),
        document_id: "VI-001".to_string(),
    }];

    let forecast = gen.generate("C001", "USD", d("2025-01-31"), &ar_items, &ap_items, &[]);

    // Net position should match computed value
    assert_eq!(forecast.net_position, forecast.computed_net_position());
}

// =============================================================================
// 4. Hedge Effectiveness Tests
// =============================================================================

#[test]
fn test_hedge_effectiveness_within_corridor() {
    // Generate many hedge relationships and verify most are within 80-125% corridor
    let mut gen = HedgingGenerator::new(42, HedgingSchemaConfig::default());

    let mut all_relationships = Vec::new();
    for i in 0..20 {
        let exposures = vec![FxExposure {
            currency_pair: "EUR/USD".to_string(),
            foreign_currency: "EUR".to_string(),
            net_amount: dec!(100000) + Decimal::from(i) * dec!(10000),
            settlement_date: d("2025-06-30"),
            description: format!("Exposure {}", i),
        }];
        let (_, rels) = gen.generate(d("2025-01-15"), &exposures);
        all_relationships.extend(rels);
    }

    assert_eq!(all_relationships.len(), 20);

    let effective_count = all_relationships
        .iter()
        .filter(|r| r.is_effective)
        .count();

    // At least 70% should be effective (generator targets 90%)
    assert!(
        effective_count >= 14,
        "Expected at least 14/20 effective relationships, got {}",
        effective_count
    );

    // Verify effectiveness flag matches ratio
    for rel in &all_relationships {
        let in_corridor =
            rel.effectiveness_ratio >= dec!(0.80) && rel.effectiveness_ratio <= dec!(1.25);
        assert_eq!(
            rel.is_effective, in_corridor,
            "Relationship {}: is_effective={} but ratio={} (in_corridor={})",
            rel.id, rel.is_effective, rel.effectiveness_ratio, in_corridor
        );
    }
}

#[test]
fn test_hedge_instrument_covers_exposure() {
    let hedge_ratio = dec!(0.75);
    let mut gen = HedgingGenerator::new(42, HedgingSchemaConfig::default());
    let exposures = vec![FxExposure {
        currency_pair: "EUR/USD".to_string(),
        foreign_currency: "EUR".to_string(),
        net_amount: dec!(1000000),
        settlement_date: d("2025-06-30"),
        description: "EUR receivables".to_string(),
    }];

    let (instruments, _) = gen.generate(d("2025-01-15"), &exposures);

    assert_eq!(instruments.len(), 1);
    assert_eq!(
        instruments[0].notional_amount,
        (dec!(1000000) * hedge_ratio).round_dp(2),
        "Notional should be {} of exposure",
        hedge_ratio
    );
}

#[test]
fn test_hedge_relationship_type_is_cash_flow() {
    let mut gen = HedgingGenerator::new(42, HedgingSchemaConfig::default());
    let exposures = vec![FxExposure {
        currency_pair: "GBP/USD".to_string(),
        foreign_currency: "GBP".to_string(),
        net_amount: dec!(-300000),
        settlement_date: d("2025-09-30"),
        description: "GBP payables".to_string(),
    }];

    let (_, relationships) = gen.generate(d("2025-01-15"), &exposures);
    assert_eq!(relationships.len(), 1);
    assert_eq!(relationships[0].hedge_type, HedgeType::CashFlowHedge);
    assert_eq!(
        relationships[0].hedged_item_type,
        HedgedItemType::ForecastedTransaction
    );
}

// =============================================================================
// 5. Debt and Covenant Tests
// =============================================================================

#[test]
fn test_debt_amortization_sums_to_principal() {
    let mut gen = DebtGenerator::new(42, test_debt_config());
    let instruments = gen.generate("C001", "USD", d("2025-01-01"));

    let term_loan = instruments
        .iter()
        .find(|i| i.instrument_type == DebtType::TermLoan)
        .unwrap();

    assert_eq!(
        term_loan.total_principal_payments(),
        dec!(5000000),
        "Amortization principal payments should sum to original principal"
    );

    // Verify last payment leaves zero balance
    let last_payment = term_loan.amortization_schedule.last().unwrap();
    assert_eq!(
        last_payment.balance_after,
        Decimal::ZERO,
        "Last amortization payment should leave zero balance"
    );
}

#[test]
fn test_revolving_credit_has_available_capacity() {
    let mut gen = DebtGenerator::new(42, test_debt_config());
    let instruments = gen.generate("C001", "USD", d("2025-01-01"));

    let revolver = instruments
        .iter()
        .find(|i| i.instrument_type == DebtType::RevolvingCredit)
        .unwrap();

    assert_eq!(revolver.rate_type, InterestRateType::Variable);
    assert_eq!(revolver.facility_limit, dec!(2000000));
    assert!(
        revolver.drawn_amount < revolver.facility_limit,
        "Drawn amount {} should be less than facility limit {}",
        revolver.drawn_amount, revolver.facility_limit
    );
    assert!(
        revolver.available_capacity() > Decimal::ZERO,
        "Should have available capacity"
    );
    assert!(
        revolver.amortization_schedule.is_empty(),
        "Revolving credit should have no amortization schedule"
    );
}

#[test]
fn test_covenant_compliance_logic() {
    let mut gen = DebtGenerator::new(42, test_debt_config());
    let instruments = gen.generate("C001", "USD", d("2025-01-01"));

    for instrument in &instruments {
        for cov in &instrument.covenants {
            assert!(
                cov.threshold > Decimal::ZERO,
                "Covenant threshold should be positive"
            );
            // Headroom sign should match compliance
            if cov.is_compliant {
                assert!(
                    cov.headroom > Decimal::ZERO,
                    "Covenant {}: compliant but headroom {} is not positive",
                    cov.id, cov.headroom
                );
            } else {
                assert!(
                    cov.headroom < Decimal::ZERO,
                    "Covenant {}: non-compliant but headroom {} is not negative",
                    cov.id, cov.headroom
                );
            }
        }
    }
}

// =============================================================================
// 6. Cash Pool Sweep Tests
// =============================================================================

#[test]
fn test_zero_balancing_net_effect() {
    let mut gen = CashPoolGenerator::new(42, CashPoolingConfig::default());
    let pool = gen
        .create_pool(
            "Test Pool",
            "USD",
            &[
                "BA-HEADER".to_string(),
                "BA-001".to_string(),
                "BA-002".to_string(),
                "BA-003".to_string(),
            ],
        )
        .unwrap();

    assert_eq!(pool.pool_type, PoolType::ZeroBalancing);

    let balances = vec![
        AccountBalance {
            account_id: "BA-001".to_string(),
            balance: dec!(50000),
        },
        AccountBalance {
            account_id: "BA-002".to_string(),
            balance: dec!(-20000),
        },
        AccountBalance {
            account_id: "BA-003".to_string(),
            balance: dec!(30000),
        },
    ];

    let sweeps = gen.generate_sweeps(&pool, d("2025-01-15"), "USD", &balances);

    // All non-zero participants should have sweeps
    assert_eq!(sweeps.len(), 3);

    // Positive balances go to header, negative balances funded from header
    for sweep in &sweeps {
        let bal = balances
            .iter()
            .find(|b| b.account_id == sweep.from_account_id || b.account_id == sweep.to_account_id)
            .unwrap();
        if bal.balance > Decimal::ZERO {
            assert_eq!(sweep.from_account_id, bal.account_id);
            assert_eq!(sweep.to_account_id, "BA-HEADER");
        }
    }
}

// =============================================================================
// 7. Anomaly Injection Tests
// =============================================================================

#[test]
fn test_anomaly_injection_modifies_positions() {
    let flows = generate_mock_cash_flows();
    let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
    let mut positions = gen.generate(
        "C001",
        "BA-001",
        "USD",
        &flows,
        d("2025-01-01"),
        d("2025-01-31"),
        dec!(100000),
    );

    let original_count = positions.len();

    let mut injector = TreasuryAnomalyInjector::new(42, 0.20); // 20% rate
    let labels = injector.inject_into_cash_positions(&mut positions, dec!(50000));

    // With 31 positions and 20% rate, expect roughly 3-10 anomalies
    assert!(
        !labels.is_empty(),
        "Should inject at least one anomaly at 20% rate with 31 positions"
    );

    // Position count should remain unchanged (anomalies modify in-place)
    assert_eq!(positions.len(), original_count);

    // All labels should reference valid anomaly types
    for label in &labels {
        assert!(
            label.anomaly_type == TreasuryAnomalyType::UnusualCashMovement
                || label.anomaly_type == TreasuryAnomalyType::LiquidityCrisis,
            "Cash position anomaly should be UnusualCashMovement or LiquidityCrisis, got {:?}",
            label.anomaly_type
        );
        assert_eq!(label.document_type, "cash_position");
        assert!(label.original_value.is_some());
        assert!(label.anomalous_value.is_some());
    }
}

#[test]
fn test_no_anomalies_at_zero_rate() {
    let flows = generate_mock_cash_flows();
    let mut gen = CashPositionGenerator::new(42, CashPositioningConfig::default());
    let mut positions = gen.generate(
        "C001",
        "BA-001",
        "USD",
        &flows,
        d("2025-01-01"),
        d("2025-01-31"),
        dec!(100000),
    );

    let original_closings: Vec<Decimal> = positions.iter().map(|p| p.closing_balance).collect();

    let mut injector = TreasuryAnomalyInjector::new(42, 0.0);
    let labels = injector.inject_into_cash_positions(&mut positions, dec!(50000));

    assert!(labels.is_empty(), "No anomalies at 0% rate");
    // Positions should be unchanged
    for (i, pos) in positions.iter().enumerate() {
        assert_eq!(
            pos.closing_balance, original_closings[i],
            "Position {} should be unchanged at 0% rate",
            i
        );
    }
}

#[test]
fn test_hedge_anomaly_makes_ineffective() {
    let mut gen = HedgingGenerator::new(42, HedgingSchemaConfig::default());
    let exposures = vec![FxExposure {
        currency_pair: "EUR/USD".to_string(),
        foreign_currency: "EUR".to_string(),
        net_amount: dec!(500000),
        settlement_date: d("2025-06-30"),
        description: "EUR receivables".to_string(),
    }];
    let (_, mut relationships) = gen.generate(d("2025-01-15"), &exposures);

    // Inject at 100% rate to guarantee anomaly
    let mut injector = TreasuryAnomalyInjector::new(42, 1.0);
    let labels = injector.inject_into_hedge_relationships(&mut relationships);

    assert_eq!(labels.len(), 1);
    assert_eq!(
        labels[0].anomaly_type,
        TreasuryAnomalyType::HedgeIneffectiveness
    );
    // After injection, the hedge should be ineffective
    assert!(
        !relationships[0].is_effective,
        "Hedge should be ineffective after anomaly injection"
    );
    // Ratio should be outside 80-125% corridor
    let ratio = relationships[0].effectiveness_ratio;
    assert!(
        ratio < dec!(0.80) || ratio > dec!(1.25),
        "Effectiveness ratio {} should be outside 80-125% corridor",
        ratio
    );
}

#[test]
fn test_covenant_anomaly_causes_breach() {
    // Use only max covenants (debt_to_ebitda) where the injector's
    // breach_factor (1.05..1.25) actually pushes actual > threshold => breach.
    // Min covenants (interest_coverage) would become MORE compliant.
    let config = DebtSchemaConfig {
        enabled: true,
        instruments: vec![DebtInstrumentDef {
            instrument_type: "term_loan".to_string(),
            principal: Some(3_000_000.0),
            rate: Some(0.05),
            maturity_months: Some(48),
            facility: None,
        }],
        covenants: vec![
            CovenantDef {
                covenant_type: "debt_to_ebitda".to_string(),
                threshold: 3.5,
            },
            CovenantDef {
                covenant_type: "debt_to_equity".to_string(),
                threshold: 2.0,
            },
        ],
    };

    let mut gen = DebtGenerator::new(42, config);
    let instruments = gen.generate("C001", "USD", d("2025-01-01"));
    let mut covenants: Vec<_> = instruments
        .into_iter()
        .flat_map(|d| d.covenants)
        .collect();

    // Inject at 100% rate to guarantee anomaly
    let mut injector = TreasuryAnomalyInjector::new(42, 1.0);
    let labels = injector.inject_into_debt_covenants(&mut covenants);

    assert_eq!(labels.len(), covenants.len());
    for (cov, label) in covenants.iter().zip(labels.iter()) {
        assert_eq!(label.anomaly_type, TreasuryAnomalyType::CovenantBreachRisk);
        // For max covenants, injected actual = threshold * 1.05..1.25 > threshold => breach
        assert!(
            !cov.is_compliant,
            "Max covenant {} should be non-compliant after anomaly injection (actual={}, threshold={})",
            cov.id, cov.actual_value, cov.threshold
        );
        assert!(
            cov.headroom < Decimal::ZERO,
            "Covenant {} headroom should be negative after breach",
            cov.id
        );
    }
}

// =============================================================================
// 8. Determinism Tests
// =============================================================================

#[test]
fn test_deterministic_treasury_pipeline() {
    let results: Vec<PipelineResult> = (0..2).map(|_| run_deterministic_pipeline(42)).collect();

    let r1 = &results[0];
    let r2 = &results[1];

    // Cash positions
    assert_eq!(r1.positions.len(), r2.positions.len());
    for (p1, p2) in r1.positions.iter().zip(r2.positions.iter()) {
        assert_eq!(p1.opening_balance, p2.opening_balance);
        assert_eq!(p1.closing_balance, p2.closing_balance);
        assert_eq!(p1.available_balance, p2.available_balance);
    }

    // Forecast items
    assert_eq!(r1.forecast_items, r2.forecast_items);

    // Hedging instruments
    assert_eq!(r1.instruments.len(), r2.instruments.len());
    for (i1, i2) in r1.instruments.iter().zip(r2.instruments.iter()) {
        assert_eq!(i1.notional_amount, i2.notional_amount);
        assert_eq!(i1.fair_value, i2.fair_value);
    }

    // Hedge relationships
    assert_eq!(r1.relationships.len(), r2.relationships.len());
    for (r1_rel, r2_rel) in r1.relationships.iter().zip(r2.relationships.iter()) {
        assert_eq!(r1_rel.effectiveness_ratio, r2_rel.effectiveness_ratio);
        assert_eq!(r1_rel.is_effective, r2_rel.is_effective);
    }

    // Debt instruments
    assert_eq!(r1.debt_instruments.len(), r2.debt_instruments.len());
    for (d1, d2) in r1.debt_instruments.iter().zip(r2.debt_instruments.iter()) {
        assert_eq!(d1.principal, d2.principal);
        assert_eq!(d1.interest_rate, d2.interest_rate);
        assert_eq!(d1.amortization_schedule.len(), d2.amortization_schedule.len());
        assert_eq!(d1.covenants.len(), d2.covenants.len());
    }
}

#[test]
fn test_different_seeds_produce_different_output() {
    let r1 = run_deterministic_pipeline(1);
    let r2 = run_deterministic_pipeline(99999);

    let mut any_difference = false;

    // Check available balances (affected by random hold)
    for (p1, p2) in r1.positions.iter().zip(r2.positions.iter()) {
        if p1.available_balance != p2.available_balance {
            any_difference = true;
            break;
        }
    }

    // Check hedge effectiveness ratios
    for (r1_rel, r2_rel) in r1.relationships.iter().zip(r2.relationships.iter()) {
        if r1_rel.effectiveness_ratio != r2_rel.effectiveness_ratio {
            any_difference = true;
        }
    }

    // Check debt covenant actuals
    for (d1, d2) in r1.debt_instruments.iter().zip(r2.debt_instruments.iter()) {
        for (c1, c2) in d1.covenants.iter().zip(d2.covenants.iter()) {
            if c1.actual_value != c2.actual_value {
                any_difference = true;
            }
        }
    }

    assert!(
        any_difference,
        "Different seeds should produce at least some different outputs"
    );
}

// =============================================================================
// Pipeline result helpers
// =============================================================================

struct PipelineResult {
    positions: Vec<CashPosition>,
    forecast_items: usize,
    instruments: Vec<datasynth_core::models::HedgingInstrument>,
    relationships: Vec<datasynth_core::models::HedgeRelationship>,
    debt_instruments: Vec<datasynth_core::models::DebtInstrument>,
}

fn run_deterministic_pipeline(seed: u64) -> PipelineResult {
    let flows = generate_mock_cash_flows();
    let mut pos_gen = CashPositionGenerator::new(seed, CashPositioningConfig::default());
    let positions = pos_gen.generate(
        "C001",
        "BA-001",
        "USD",
        &flows,
        d("2025-01-01"),
        d("2025-01-15"),
        dec!(100000),
    );

    let ar_items = vec![ArAgingItem {
        expected_date: d("2025-02-15"),
        amount: dec!(50000),
        days_past_due: 0,
        document_id: "INV-001".to_string(),
    }];
    let ap_items = vec![ApAgingItem {
        payment_date: d("2025-02-10"),
        amount: dec!(30000),
        document_id: "VI-001".to_string(),
    }];
    let mut forecast_gen = CashForecastGenerator::new(seed, CashForecastingConfig::default());
    let forecast = forecast_gen.generate("C001", "USD", d("2025-01-31"), &ar_items, &ap_items, &[]);

    let exposures = vec![FxExposure {
        currency_pair: "EUR/USD".to_string(),
        foreign_currency: "EUR".to_string(),
        net_amount: dec!(500000),
        settlement_date: d("2025-06-30"),
        description: "EUR receivables".to_string(),
    }];
    let mut hedge_gen = HedgingGenerator::new(seed, HedgingSchemaConfig::default());
    let (instruments, relationships) = hedge_gen.generate(d("2025-01-15"), &exposures);

    let mut debt_gen = DebtGenerator::new(seed, test_debt_config());
    let debt_instruments = debt_gen.generate("C001", "USD", d("2025-01-01"));

    PipelineResult {
        positions,
        forecast_items: forecast.items.len(),
        instruments,
        relationships,
        debt_instruments,
    }
}
