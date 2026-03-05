//! Integration tests for the Banking pipeline.
//!
//! Verifies end-to-end coherence of the `BankingOrchestrator`: customer-account
//! linkage, transaction-account linkage, label coverage, and AML typology
//! injection with a small deterministic configuration.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use uuid::Uuid;

use datasynth_banking::{BankingConfig, BankingOrchestrator};

// =============================================================================
// Full pipeline coherence
// =============================================================================

/// Generate banking data with a small config and verify cross-entity coherence.
#[test]
fn test_banking_pipeline_coherence() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 42);
    let data = orchestrator.generate();

    // ── Basic non-emptiness ──────────────────────────────────────────────
    assert!(!data.customers.is_empty(), "Should generate customers");
    assert!(!data.accounts.is_empty(), "Should generate accounts");
    assert!(
        !data.transactions.is_empty(),
        "Should generate transactions"
    );
    assert!(
        !data.transaction_labels.is_empty(),
        "Should generate transaction labels"
    );
    assert!(
        !data.customer_labels.is_empty(),
        "Should generate customer labels"
    );

    // ── Customer ID set ──────────────────────────────────────────────────
    let customer_ids: HashSet<Uuid> = data.customers.iter().map(|c| c.customer_id).collect();

    // ── Account ID set ───────────────────────────────────────────────────
    let account_ids: HashSet<Uuid> = data.accounts.iter().map(|a| a.account_id).collect();

    // ── All accounts belong to known customers ───────────────────────────
    for account in &data.accounts {
        assert!(
            customer_ids.contains(&account.primary_owner_id),
            "Account {:?} references unknown customer {:?}",
            account.account_id,
            account.primary_owner_id
        );
    }

    // ── Every customer has at least one account ──────────────────────────
    let customers_with_accounts: HashSet<Uuid> =
        data.accounts.iter().map(|a| a.primary_owner_id).collect();

    for cust in &data.customers {
        assert!(
            customers_with_accounts.contains(&cust.customer_id),
            "Customer {:?} has no accounts",
            cust.customer_id
        );
    }

    // ── All transactions reference valid accounts ────────────────────────
    for txn in &data.transactions {
        assert!(
            account_ids.contains(&txn.account_id),
            "Transaction {:?} references unknown account {:?}",
            txn.transaction_id,
            txn.account_id
        );
    }

    // ── Transaction labels reference valid transaction IDs ───────────────
    let transaction_ids: HashSet<Uuid> = data.transactions.iter().map(|t| t.transaction_id).collect();

    for label in &data.transaction_labels {
        assert!(
            transaction_ids.contains(&label.transaction_id),
            "Transaction label references unknown transaction {:?}",
            label.transaction_id
        );
        assert!(
            label.confidence >= 0.0 && label.confidence <= 1.0,
            "Label confidence {} out of range [0, 1]",
            label.confidence
        );
    }

    // ── Customer labels reference valid customer IDs ─────────────────────
    for label in &data.customer_labels {
        assert!(
            customer_ids.contains(&label.customer_id),
            "Customer label references unknown customer {:?}",
            label.customer_id
        );
    }

    // ── Account labels reference valid account IDs ───────────────────────
    for label in &data.account_labels {
        assert!(
            account_ids.contains(&label.account_id),
            "Account label references unknown account {:?}",
            label.account_id
        );
    }

    // ── Statistics match actual counts ───────────────────────────────────
    assert_eq!(data.stats.customer_count, data.customers.len());
    assert_eq!(data.stats.account_count, data.accounts.len());
    assert_eq!(data.stats.transaction_count, data.transactions.len());

    let actual_suspicious = data.transactions.iter().filter(|t| t.is_suspicious).count();
    assert_eq!(
        data.stats.suspicious_count, actual_suspicious,
        "Suspicious count stat should match actual"
    );

    println!(
        "Banking pipeline coherence OK: {} customers, {} accounts, {} transactions, {} suspicious",
        data.stats.customer_count,
        data.stats.account_count,
        data.stats.transaction_count,
        data.stats.suspicious_count
    );
}

// =============================================================================
// AML typology coherence
// =============================================================================

/// Verify AML scenarios reference valid customer and transaction IDs.
#[test]
fn test_banking_aml_typology_coherence() {
    let mut config = BankingConfig::small();
    config.typologies.suspicious_rate = 0.10;
    config.typologies.structuring_rate = 0.03;
    config.typologies.funnel_rate = 0.02;
    config.typologies.layering_rate = 0.02;
    config.typologies.mule_rate = 0.02;

    let orchestrator = BankingOrchestrator::new(config, 12345);
    let data = orchestrator.generate();

    let customer_ids: HashSet<Uuid> = data.customers.iter().map(|c| c.customer_id).collect();
    let transaction_ids: HashSet<Uuid> = data.transactions.iter().map(|t| t.transaction_id).collect();

    // Verify scenario references
    for scenario in &data.scenarios {
        // Involved customers should be known
        for cust_id in &scenario.involved_customers {
            assert!(
                customer_ids.contains(cust_id),
                "AML scenario references unknown customer {:?}",
                cust_id
            );
        }
        // Involved transactions should be known
        for txn_id in &scenario.involved_transactions {
            assert!(
                transaction_ids.contains(txn_id),
                "AML scenario references unknown transaction {:?}",
                txn_id
            );
        }
    }

    // Suspicious transactions should be labeled
    let suspicious_ids: HashSet<Uuid> = data
        .transactions
        .iter()
        .filter(|t| t.is_suspicious)
        .map(|t| t.transaction_id)
        .collect();
    let labeled_suspicious: HashSet<Uuid> = data
        .transaction_labels
        .iter()
        .filter(|l| l.is_suspicious)
        .map(|l| l.transaction_id)
        .collect();

    for txn_id in &suspicious_ids {
        assert!(
            labeled_suspicious.contains(txn_id),
            "Suspicious transaction {:?} should have a suspicious label",
            txn_id
        );
    }

    // AML scenario counts should be reasonable
    assert!(
        data.stats.scenario_count == data.scenarios.len(),
        "Scenario count stat should match actual"
    );

    println!(
        "AML coherence OK: {} scenarios, {} suspicious out of {} transactions",
        data.scenarios.len(),
        suspicious_ids.len(),
        data.transactions.len()
    );
}

// =============================================================================
// Determinism
// =============================================================================

/// Running the orchestrator twice with the same config and seed should produce
/// identical output counts and IDs.
#[test]
fn test_banking_pipeline_deterministic() {
    let config = BankingConfig::small();
    let seed = 42u64;

    let data1 = BankingOrchestrator::new(config.clone(), seed).generate();
    let data2 = BankingOrchestrator::new(config, seed).generate();

    assert_eq!(
        data1.customers.len(),
        data2.customers.len(),
        "Customer count should be deterministic"
    );
    assert_eq!(
        data1.accounts.len(),
        data2.accounts.len(),
        "Account count should be deterministic"
    );
    assert_eq!(
        data1.transactions.len(),
        data2.transactions.len(),
        "Transaction count should be deterministic"
    );
    assert_eq!(
        data1.stats.suspicious_count, data2.stats.suspicious_count,
        "Suspicious count should be deterministic"
    );
    assert_eq!(
        data1.scenarios.len(),
        data2.scenarios.len(),
        "Scenario count should be deterministic"
    );

    // Verify first few customer IDs match
    for (c1, c2) in data1.customers.iter().zip(data2.customers.iter()) {
        assert_eq!(
            c1.customer_id, c2.customer_id,
            "Customer IDs should be deterministic"
        );
    }

    // Verify first few transaction IDs match
    for (t1, t2) in data1
        .transactions
        .iter()
        .take(20)
        .zip(data2.transactions.iter().take(20))
    {
        assert_eq!(
            t1.transaction_id, t2.transaction_id,
            "Transaction IDs should be deterministic"
        );
    }
}

// =============================================================================
// Transaction amount validation
// =============================================================================

/// All generated transactions should have positive amounts.
#[test]
fn test_banking_transaction_amounts_positive() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 77777);
    let data = orchestrator.generate();

    for txn in &data.transactions {
        assert!(
            txn.amount > rust_decimal::Decimal::ZERO,
            "Transaction {:?} has non-positive amount: {}",
            txn.transaction_id,
            txn.amount
        );
    }
}
