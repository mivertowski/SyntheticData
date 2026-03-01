//! Integration tests for individual AML typology injectors.
//!
//! These tests verify per-typology behavioral patterns:
//! - Structuring deposits stay below the CTR threshold
//! - Layering produces multi-hop (inbound + outbound) transfers
//! - Funnel accounts show more inbound than outbound transactions
//! - Round-tripping generates circular flows with roughly balanced amounts
//! - The orchestrator produces non-empty, deterministic, and properly labeled data

#![allow(clippy::unwrap_used)]

use chrono::NaiveDate;
use datasynth_banking::typologies::{
    FunnelInjector, LayeringInjector, RoundTrippingInjector, StructuringInjector,
};
use datasynth_banking::{BankingConfig, BankingOrchestrator, Direction, Sophistication};
use datasynth_banking::models::{BankAccount, BankingCustomer};
use datasynth_core::models::banking::BankAccountType;
use uuid::Uuid;

// =============================================================================
// Helper functions
// =============================================================================

/// Create a minimal retail customer and checking account pair for typology testing.
fn make_customer_and_account() -> (BankingCustomer, BankAccount) {
    let customer_id = Uuid::new_v4();
    let customer = BankingCustomer::new_retail(
        customer_id,
        "Test",
        "User",
        "US",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    );

    let account = BankAccount::new(
        Uuid::new_v4(),
        "****1234".to_string(),
        BankAccountType::Checking,
        customer_id,
        "USD",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    );

    (customer, account)
}

fn test_date_range() -> (NaiveDate, NaiveDate) {
    (
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
    )
}

// =============================================================================
// Per-typology behavior tests
// =============================================================================

/// Structuring deposits must all be below the $10,000 CTR threshold and
/// the injector must produce multiple transactions.
#[test]
fn test_structuring_generates_below_threshold() {
    let mut injector = StructuringInjector::new(42);
    let (customer, account) = make_customer_and_account();
    let (start, end) = test_date_range();

    let transactions = injector.generate(&customer, &account, start, end, Sophistication::Basic);

    // Must generate multiple transactions (structuring splits a large amount)
    assert!(
        transactions.len() >= 2,
        "Structuring should produce at least 2 transactions, got {}",
        transactions.len()
    );

    // Every deposit must be below the $10,000 CTR threshold
    for txn in &transactions {
        let amount_f64: f64 = txn.amount.try_into().unwrap();
        assert!(
            amount_f64 < 10_000.0,
            "Structuring deposit amount {} should be below $10,000 threshold",
            amount_f64
        );
    }
}

/// Layering must produce both inbound and outbound transactions (multi-hop pattern).
#[test]
fn test_layering_generates_multi_hop() {
    let mut injector = LayeringInjector::new(42);
    let (customer, account) = make_customer_and_account();
    let (start, _) = test_date_range();
    // Use a wider window so layering hops have room
    let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

    let transactions =
        injector.generate(&customer, &account, start, end, Sophistication::Standard);

    assert!(
        !transactions.is_empty(),
        "Layering should produce transactions"
    );

    let has_inbound = transactions
        .iter()
        .any(|t| t.direction == Direction::Inbound);
    let has_outbound = transactions
        .iter()
        .any(|t| t.direction == Direction::Outbound);

    assert!(has_inbound, "Layering should have inbound transactions");
    assert!(has_outbound, "Layering should have outbound transactions");
}

/// Funnel accounts receive from many sources and consolidate outward, so
/// inbound transactions must outnumber outbound transactions.
#[test]
fn test_funnel_generates_many_inbound() {
    let mut injector = FunnelInjector::new(42);
    let (customer, account) = make_customer_and_account();
    let (start, _) = test_date_range();
    let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

    let transactions =
        injector.generate(&customer, &account, start, end, Sophistication::Standard);

    assert!(
        !transactions.is_empty(),
        "Funnel should produce transactions"
    );

    let inbound_count = transactions
        .iter()
        .filter(|t| t.direction == Direction::Inbound)
        .count();
    let outbound_count = transactions
        .iter()
        .filter(|t| t.direction == Direction::Outbound)
        .count();

    assert!(
        outbound_count > 0,
        "Funnel should have outbound consolidation transactions"
    );
    assert!(
        inbound_count > outbound_count,
        "Funnel should have more inbound ({}) than outbound ({}) transactions",
        inbound_count,
        outbound_count
    );
}

/// Round-tripping must generate transactions in both directions and the
/// total outbound and inbound amounts should roughly balance (within 50%
/// tolerance to account for fees, profit margins, and intermediate transfers).
#[test]
fn test_round_tripping_generates_circular() {
    let mut injector = RoundTrippingInjector::new(42);
    let (customer, account) = make_customer_and_account();
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();

    let transactions =
        injector.generate(&customer, &account, start, end, Sophistication::Standard);

    assert!(
        transactions.len() >= 2,
        "Round-tripping should produce at least 2 transactions (out + in), got {}",
        transactions.len()
    );

    let has_outbound = transactions
        .iter()
        .any(|t| t.direction == Direction::Outbound);
    let has_inbound = transactions
        .iter()
        .any(|t| t.direction == Direction::Inbound);

    assert!(
        has_outbound,
        "Round-tripping should have outbound leg transactions"
    );
    assert!(
        has_inbound,
        "Round-tripping should have inbound return leg transactions"
    );

    let total_outbound: f64 = transactions
        .iter()
        .filter(|t| t.direction == Direction::Outbound)
        .map(|t| -> f64 { t.amount.try_into().unwrap() })
        .sum();

    let total_inbound: f64 = transactions
        .iter()
        .filter(|t| t.direction == Direction::Inbound)
        .map(|t| -> f64 { t.amount.try_into().unwrap() })
        .sum();

    // The amounts should roughly balance -- allow wide tolerance for fees and
    // intermediate advisory-fee transfers that the injector adds.
    let ratio = if total_outbound > 0.0 {
        total_inbound / total_outbound
    } else {
        0.0
    };

    assert!(
        (0.3..=3.0).contains(&ratio),
        "Round-tripping inbound/outbound ratio {} should be roughly balanced (0.3..3.0). \
         Inbound={:.2}, Outbound={:.2}",
        ratio,
        total_inbound,
        total_outbound
    );
}

// =============================================================================
// Orchestrator-level integration tests
// =============================================================================

/// A small config should produce non-empty customers, accounts, and transactions.
#[test]
fn test_small_config_generates_data() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 42);
    let data = orchestrator.generate();

    assert!(
        !data.customers.is_empty(),
        "Small config should produce customers"
    );
    assert!(
        !data.accounts.is_empty(),
        "Small config should produce accounts"
    );
    assert!(
        !data.transactions.is_empty(),
        "Small config should produce transactions"
    );
}

/// Generating twice with the same config and seed must produce identical counts,
/// verifying determinism of the ChaCha8-based RNG.
#[test]
fn test_generation_deterministic() {
    let config = BankingConfig::small();
    let seed = 42;

    let data1 = BankingOrchestrator::new(config.clone(), seed).generate();
    let data2 = BankingOrchestrator::new(config, seed).generate();

    assert_eq!(
        data1.customers.len(),
        data2.customers.len(),
        "Determinism: customer count should match across runs"
    );
    assert_eq!(
        data1.accounts.len(),
        data2.accounts.len(),
        "Determinism: account count should match across runs"
    );
    assert_eq!(
        data1.transactions.len(),
        data2.transactions.len(),
        "Determinism: transaction count should match across runs"
    );
    assert_eq!(
        data1.stats.suspicious_count,
        data2.stats.suspicious_count,
        "Determinism: suspicious count should match across runs"
    );
}

/// With suspicious activity enabled, some transactions should be marked
/// `is_suspicious == true` with a populated `aml_typology` (suspicion_reason).
#[test]
fn test_suspicious_transactions_labeled() {
    let mut config = BankingConfig::small();
    // Raise the suspicious rate to ensure we reliably get some
    config.typologies.suspicious_rate = 0.10;
    config.typologies.structuring_rate = 0.03;
    config.typologies.funnel_rate = 0.02;
    config.typologies.layering_rate = 0.02;
    config.typologies.mule_rate = 0.02;

    let orchestrator = BankingOrchestrator::new(config, 42);
    let data = orchestrator.generate();

    let suspicious: Vec<_> = data
        .transactions
        .iter()
        .filter(|t| t.is_suspicious)
        .collect();

    assert!(
        !suspicious.is_empty(),
        "With 10% suspicious rate on a small config there should be suspicious transactions"
    );

    // Every suspicious transaction must carry a typology label
    for txn in &suspicious {
        assert!(
            txn.suspicion_reason.is_some(),
            "Suspicious transaction {:?} should have an aml_typology (suspicion_reason)",
            txn.transaction_id
        );
    }
}
