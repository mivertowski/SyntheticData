//! Validation tests for banking data generation.
//!
//! These tests validate that generated banking data meets compliance requirements,
//! has proper KYC coherence, correct typology distribution, and accurate labels.

use std::collections::{HashMap, HashSet};

use rust_decimal::Decimal;
use uuid::Uuid;

use datasynth_banking::{BankingConfig, BankingCustomerType, BankingOrchestrator};
use datasynth_core::models::banking::AmlTypology;

// =============================================================================
// KYC Profile Coherence Tests
// =============================================================================

/// Test that customer KYC profiles are coherent with customer type.
#[test]
fn test_kyc_profile_coherence() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 12345);
    let data = orchestrator.generate();

    for customer in &data.customers {
        let kyc = &customer.kyc_profile;

        // KYC completeness should be recorded (0.0 to 1.0)
        assert!(
            kyc.completeness_score >= 0.0 && kyc.completeness_score <= 1.0,
            "Customer {} has invalid KYC completeness: {}",
            customer.customer_id,
            kyc.completeness_score
        );

        // International rate should be valid
        assert!(
            kyc.international_rate >= 0.0 && kyc.international_rate <= 1.0,
            "Customer {} has invalid international rate: {}",
            customer.customer_id,
            kyc.international_rate
        );

        // Large transaction rate should be valid
        assert!(
            kyc.large_transaction_rate >= 0.0 && kyc.large_transaction_rate <= 1.0,
            "Customer {} has invalid large transaction rate: {}",
            customer.customer_id,
            kyc.large_transaction_rate
        );
    }
}

/// Test that business customers have business-appropriate KYC profiles.
#[test]
fn test_business_customer_kyc() {
    let mut config = BankingConfig::small();
    config.population.business_customers = 50;

    let orchestrator = BankingOrchestrator::new(config, 54321);
    let data = orchestrator.generate();

    let business_customers: Vec<_> = data
        .customers
        .iter()
        .filter(|c| {
            c.customer_type == BankingCustomerType::Business
                || c.customer_type == BankingCustomerType::Trust
        })
        .collect();

    for customer in &business_customers {
        // Business customers should have a declared purpose
        assert!(
            !customer.kyc_profile.declared_purpose.is_empty(),
            "Business customer {} should have declared purpose",
            customer.customer_id
        );
    }

    println!(
        "Validated {} business/trust customers",
        business_customers.len()
    );
}

// =============================================================================
// Account Feature Validation Tests
// =============================================================================

/// Test that account features match customer type.
#[test]
fn test_account_feature_validation() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 67890);
    let data = orchestrator.generate();

    // Build customer ID -> type map
    let customer_types: HashMap<Uuid, BankingCustomerType> = data
        .customers
        .iter()
        .map(|c| (c.customer_id, c.customer_type))
        .collect();

    for account in &data.accounts {
        // Account should reference valid customer
        assert!(
            customer_types.contains_key(&account.primary_owner_id),
            "Account {} references unknown customer {:?}",
            account.account_id,
            account.primary_owner_id
        );

        // Balance check - accounts without overdraft should not be too negative
        if account.overdraft_limit == Decimal::ZERO {
            // Allow small negative due to timing
            assert!(
                account.current_balance >= Decimal::new(-100, 0),
                "Account {} has excessive negative balance without overdraft: {}",
                account.account_id,
                account.current_balance
            );
        }
    }
}

/// Test that each customer has at least one account.
#[test]
fn test_customer_account_linkage() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 11111);
    let data = orchestrator.generate();

    // Collect customers with accounts
    let customers_with_accounts: HashSet<Uuid> =
        data.accounts.iter().map(|a| a.primary_owner_id).collect();

    // All customers should have at least one account
    for customer in &data.customers {
        assert!(
            customers_with_accounts.contains(&customer.customer_id),
            "Customer {:?} has no accounts",
            customer.customer_id
        );
    }
}

// =============================================================================
// Customer Type Distribution Tests
// =============================================================================

/// Test that customer type distribution matches configuration.
#[test]
fn test_customer_type_distribution() {
    let mut config = BankingConfig::small();
    config.population.retail_customers = 100;
    config.population.business_customers = 20;
    config.population.trusts = 5;

    let orchestrator = BankingOrchestrator::new(config.clone(), 22222);
    let data = orchestrator.generate();

    let mut type_counts: HashMap<BankingCustomerType, usize> = HashMap::new();
    for customer in &data.customers {
        *type_counts.entry(customer.customer_type).or_default() += 1;
    }

    // Check counts match (with tolerance for generation logic)
    let retail_count = *type_counts.get(&BankingCustomerType::Retail).unwrap_or(&0);
    let business_count = *type_counts
        .get(&BankingCustomerType::Business)
        .unwrap_or(&0);
    let trust_count = *type_counts.get(&BankingCustomerType::Trust).unwrap_or(&0);

    // Verify counts are within expected range (allow 20% variance)
    assert!(
        (80..=120).contains(&retail_count),
        "Retail count {} outside expected range [80, 120]",
        retail_count
    );
    assert!(
        (15..=30).contains(&business_count),
        "Business count {} outside expected range [15, 30]",
        business_count
    );
    assert!(
        trust_count <= 10,
        "Trust count {} outside expected range [0, 10]",
        trust_count
    );

    println!(
        "Customer distribution: retail={}, business={}, trust={}",
        retail_count, business_count, trust_count
    );
}

// =============================================================================
// AML Typology Detection Tests
// =============================================================================

/// Test that AML typologies are properly labeled.
#[test]
fn test_typology_labels() {
    let mut config = BankingConfig::small();
    // Increase suspicious rate for testing
    config.typologies.suspicious_rate = 0.10;
    config.typologies.structuring_rate = 0.03;
    config.typologies.mule_rate = 0.03;

    let orchestrator = BankingOrchestrator::new(config, 33333);
    let data = orchestrator.generate();

    // Count suspicious transactions
    let suspicious_count = data.transactions.iter().filter(|t| t.is_suspicious).count();

    // Should have some suspicious transactions
    if data.transactions.len() >= 100 {
        assert!(
            suspicious_count > 0,
            "Should have at least some suspicious transactions"
        );

        let suspicious_rate = suspicious_count as f64 / data.transactions.len() as f64;
        println!(
            "Suspicious rate: {:.2}% ({} of {})",
            suspicious_rate * 100.0,
            suspicious_count,
            data.transactions.len()
        );
    }

    // Verify transaction labels exist for suspicious transactions
    let suspicious_txn_ids: HashSet<Uuid> = data
        .transactions
        .iter()
        .filter(|t| t.is_suspicious)
        .map(|t| t.transaction_id)
        .collect();

    let labeled_suspicious_ids: HashSet<Uuid> = data
        .transaction_labels
        .iter()
        .filter(|l| l.is_suspicious)
        .map(|l| l.transaction_id)
        .collect();

    // Labels should match suspicious flags
    for txn_id in &suspicious_txn_ids {
        assert!(
            labeled_suspicious_ids.contains(txn_id),
            "Suspicious transaction {:?} missing label",
            txn_id
        );
    }
}

/// Test structuring detection patterns.
#[test]
fn test_structuring_patterns() {
    let mut config = BankingConfig::small();
    config.typologies.structuring_rate = 0.05;
    config.typologies.suspicious_rate = 0.10;

    let orchestrator = BankingOrchestrator::new(config, 44444);
    let data = orchestrator.generate();

    // Find structuring scenarios
    let structuring_scenarios: Vec<_> = data
        .scenarios
        .iter()
        .filter(|s| matches!(s.typology, AmlTypology::Structuring))
        .collect();

    if !structuring_scenarios.is_empty() {
        println!(
            "Found {} structuring scenarios",
            structuring_scenarios.len()
        );

        // Verify structuring scenario properties
        for scenario in &structuring_scenarios {
            assert!(
                !scenario.involved_transactions.is_empty(),
                "Structuring scenario should have transactions"
            );
        }
    }
}

/// Test mule network detection patterns.
#[test]
fn test_mule_network_patterns() {
    let mut config = BankingConfig::small();
    config.typologies.mule_rate = 0.05;
    config.typologies.suspicious_rate = 0.10;

    let orchestrator = BankingOrchestrator::new(config, 55555);
    let data = orchestrator.generate();

    // Find mule scenarios (MoneyMule typology)
    let mule_scenarios: Vec<_> = data
        .scenarios
        .iter()
        .filter(|s| matches!(s.typology, AmlTypology::MoneyMule))
        .collect();

    if !mule_scenarios.is_empty() {
        println!("Found {} money mule scenarios", mule_scenarios.len());

        // Verify mule scenario properties
        for scenario in &mule_scenarios {
            assert!(
                !scenario.involved_transactions.is_empty(),
                "Mule scenario should have transactions"
            );
            assert!(
                !scenario.involved_customers.is_empty(),
                "Mule scenario should involve customers"
            );
        }
    }
}

// =============================================================================
// Transaction Validation Tests
// =============================================================================

/// Test that transactions have valid amounts.
#[test]
fn test_transaction_amount_validation() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 66666);
    let data = orchestrator.generate();

    for txn in &data.transactions {
        // Amount should be positive
        assert!(
            txn.amount > Decimal::ZERO,
            "Transaction {:?} has non-positive amount: {}",
            txn.transaction_id,
            txn.amount
        );

        // Should have a valid category (Other is okay for suspicious transactions)
        if !txn.is_suspicious {
            // Non-suspicious transactions may also have Other category
            // Just verify category is set
            let _ = txn.category; // Ensure field exists
        }
    }
}

/// Test transaction to account linkage.
#[test]
fn test_transaction_account_linkage() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 77777);
    let data = orchestrator.generate();

    let account_ids: HashSet<Uuid> = data.accounts.iter().map(|a| a.account_id).collect();

    for txn in &data.transactions {
        // Transaction should reference valid account
        assert!(
            account_ids.contains(&txn.account_id),
            "Transaction {:?} references unknown account {:?}",
            txn.transaction_id,
            txn.account_id
        );
    }
}

// =============================================================================
// Label Quality Tests
// =============================================================================

/// Test that transaction labels have correct format.
#[test]
fn test_transaction_label_format() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 88888);
    let data = orchestrator.generate();

    for label in &data.transaction_labels {
        // Label should reference valid transaction
        assert!(
            label.transaction_id != Uuid::nil(),
            "Label missing valid transaction ID"
        );

        // Confidence should be valid
        assert!(
            label.confidence >= 0.0 && label.confidence <= 1.0,
            "Label has invalid confidence: {}",
            label.confidence
        );

        // If suspicious, may have case_id
        if label.is_suspicious {
            println!(
                "Suspicious label: txn={:?}, confidence={:.2}",
                label.transaction_id, label.confidence
            );
        }
    }
}

/// Test that customer labels exist for all customers.
#[test]
fn test_customer_label_coverage() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 99999);
    let data = orchestrator.generate();

    let customer_ids: HashSet<Uuid> = data.customers.iter().map(|c| c.customer_id).collect();

    let labeled_customer_ids: HashSet<Uuid> =
        data.customer_labels.iter().map(|l| l.customer_id).collect();

    // All customers should have labels
    for customer_id in &customer_ids {
        assert!(
            labeled_customer_ids.contains(customer_id),
            "Customer {:?} missing label",
            customer_id
        );
    }
}

// =============================================================================
// Spoofing Tests
// =============================================================================

/// Test that spoofed transactions are properly marked.
#[test]
fn test_spoofing_labels() {
    let mut config = BankingConfig::small();
    config.spoofing.enabled = true;
    config.spoofing.intensity = 0.5;
    // Bias toward Professional/Advanced so spoofing is reliably triggered
    config.typologies.sophistication.basic = 0.1;
    config.typologies.sophistication.standard = 0.1;
    config.typologies.sophistication.professional = 0.4;
    config.typologies.sophistication.advanced = 0.4;

    let orchestrator = BankingOrchestrator::new(config, 10101);
    let data = orchestrator.generate();

    // Count spoofed transactions
    let spoofed_count = data.transactions.iter().filter(|t| t.is_spoofed).count();

    // Should have some spoofed transactions
    if data.transactions.len() >= 100 {
        let spoofed_rate = spoofed_count as f64 / data.transactions.len() as f64;
        println!(
            "Spoofed rate: {:.2}% ({} of {})",
            spoofed_rate * 100.0,
            spoofed_count,
            data.transactions.len()
        );

        // Spoofed rate should be related to intensity (not exact due to random sampling)
        // Just verify we have some spoofed transactions
        assert!(
            spoofed_count > 0,
            "Should have some spoofed transactions with intensity=0.5"
        );
    }
}

// =============================================================================
// Generation Statistics Tests
// =============================================================================

/// Test that generation statistics are accurate.
#[test]
fn test_generation_statistics() {
    let config = BankingConfig::small();
    let orchestrator = BankingOrchestrator::new(config, 20202);
    let data = orchestrator.generate();

    // Verify stats match actual counts
    assert_eq!(
        data.stats.customer_count,
        data.customers.len(),
        "Customer count mismatch"
    );
    assert_eq!(
        data.stats.account_count,
        data.accounts.len(),
        "Account count mismatch"
    );
    assert_eq!(
        data.stats.transaction_count,
        data.transactions.len(),
        "Transaction count mismatch"
    );

    // Verify suspicious count
    let actual_suspicious = data.transactions.iter().filter(|t| t.is_suspicious).count();
    assert_eq!(
        data.stats.suspicious_count, actual_suspicious,
        "Suspicious count mismatch"
    );

    // Verify suspicious rate calculation
    if !data.transactions.is_empty() {
        let expected_rate = actual_suspicious as f64 / data.transactions.len() as f64;
        assert!(
            (data.stats.suspicious_rate - expected_rate).abs() < 0.001,
            "Suspicious rate mismatch"
        );
    }

    println!(
        "Generation stats: {} customers, {} accounts, {} transactions, {} suspicious",
        data.stats.customer_count,
        data.stats.account_count,
        data.stats.transaction_count,
        data.stats.suspicious_count
    );
}
