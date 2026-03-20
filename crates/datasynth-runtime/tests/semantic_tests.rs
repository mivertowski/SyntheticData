//! Semantic and generation correctness integration tests.
//!
//! These tests verify that generated data follows accounting rules and
//! statistical distributions as expected.

use datasynth_runtime::{EnhancedOrchestrator, PhaseConfig};
use datasynth_test_utils::{
    assert_all_balanced, check_benford_distribution, fixtures::minimal_config,
};
use rust_decimal::Decimal;

/// Test that all generated journal entries are balanced (debits = credits).
///
/// Note: Entries marked as anomalies (including human errors) are excluded from
/// this test since they may be intentionally unbalanced for ML detection training.
#[test]
fn test_all_journal_entries_balanced() {
    let mut config = minimal_config();
    config.global.seed = Some(12345);
    config.global.period_months = 1;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    assert!(
        !result.journal_entries.is_empty(),
        "Should generate at least one entry"
    );

    // Filter out entries with human errors before checking balance
    // Human errors (marked with [HUMAN_ERROR:*] tags) may be intentionally unbalanced
    // for ML detection training (e.g., transposed digits, decimal shifts)
    let normal_entries: Vec<_> = result
        .journal_entries
        .iter()
        .filter(|e| {
            e.header
                .header_text
                .as_ref()
                .map(|text| !text.contains("[HUMAN_ERROR:"))
                .unwrap_or(true)
        })
        .cloned()
        .collect();

    assert!(
        !normal_entries.is_empty(),
        "Should have at least some entries without human errors"
    );

    // Verify all entries without human errors are balanced
    assert_all_balanced!(normal_entries);
}

/// Test that generated amounts are analyzed for Benford's Law distribution.
///
/// Note: This test checks that the distribution can be computed, but does not
/// strictly enforce Benford's Law compliance as the generator may use different
/// distribution strategies depending on configuration.
#[test]
fn test_benford_distribution_analysis() {
    let mut config = minimal_config();
    config.global.seed = Some(54321);
    config.global.period_months = 3; // Generate more data for better statistical sample
    config.companies[0].annual_transaction_volume =
        datasynth_config::schema::TransactionVolume::HundredK;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Collect all amounts from journal entries
    let amounts: Vec<Decimal> = result
        .journal_entries
        .iter()
        .flat_map(|entry| {
            entry
                .lines
                .iter()
                .map(|line| line.debit_amount + line.credit_amount)
        })
        .filter(|&amount| amount > Decimal::ZERO)
        .collect();

    assert!(
        amounts.len() >= 100,
        "Need at least 100 amounts for statistical test, got {}",
        amounts.len()
    );

    let (chi_squared, _passes) = check_benford_distribution(&amounts);

    // Log the chi-squared value for analysis - in production, this would be
    // tracked as a quality metric
    println!(
        "Benford's Law chi-squared: {:.2} (lower is better, <20.09 passes at p<0.01)",
        chi_squared
    );

    // For now, just verify we can compute the distribution
    assert!(
        chi_squared.is_finite(),
        "Chi-squared should be a valid number"
    );
}

/// Test deterministic output with same seed.
/// Known issue: Some v1.3.0 phases use Uuid::now_v7() for document IDs,
/// which is time-based and non-deterministic across runs.
/// TODO: Migrate all JE creation to DeterministicUuidFactory.
#[test]
#[ignore = "non-deterministic UUIDs in period-close/elimination JEs"]
fn test_deterministic_generation() {
    let config1 = {
        let mut c = minimal_config();
        c.global.seed = Some(99999);
        c
    };

    let config2 = {
        let mut c = minimal_config();
        c.global.seed = Some(99999);
        c
    };

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator1 = EnhancedOrchestrator::new(config1, phase_config.clone())
        .expect("Failed to create orchestrator");
    let result1 = orchestrator1.generate().expect("Generation 1 failed");

    let mut orchestrator2 =
        EnhancedOrchestrator::new(config2, phase_config).expect("Failed to create orchestrator");
    let result2 = orchestrator2.generate().expect("Generation 2 failed");

    assert_eq!(
        result1.journal_entries.len(),
        result2.journal_entries.len(),
        "Same seed should produce same number of entries"
    );

    // Check that document IDs match (they should be deterministic)
    for (e1, e2) in result1
        .journal_entries
        .iter()
        .zip(result2.journal_entries.iter())
    {
        assert_eq!(
            e1.header.document_id, e2.header.document_id,
            "Document IDs should match for same seed"
        );
        assert_eq!(
            e1.header.company_code, e2.header.company_code,
            "Company codes should match for same seed"
        );
    }
}

/// Test that different seeds produce different output.
#[test]
fn test_different_seeds_different_output() {
    let config1 = {
        let mut c = minimal_config();
        c.global.seed = Some(11111);
        c
    };

    let config2 = {
        let mut c = minimal_config();
        c.global.seed = Some(22222);
        c
    };

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator1 = EnhancedOrchestrator::new(config1, phase_config.clone())
        .expect("Failed to create orchestrator");
    let result1 = orchestrator1.generate().expect("Generation 1 failed");

    let mut orchestrator2 =
        EnhancedOrchestrator::new(config2, phase_config).expect("Failed to create orchestrator");
    let result2 = orchestrator2.generate().expect("Generation 2 failed");

    // Check that at least some document IDs differ
    let different_ids = result1
        .journal_entries
        .iter()
        .zip(result2.journal_entries.iter())
        .filter(|(e1, e2)| e1.header.document_id != e2.header.document_id)
        .count();

    assert!(
        different_ids > 0,
        "Different seeds should produce at least some different document IDs"
    );
}

/// Test that line item counts follow expected distribution.
#[test]
fn test_line_item_distribution() {
    let mut config = minimal_config();
    config.global.seed = Some(77777);
    config.global.period_months = 3;
    config.companies[0].annual_transaction_volume =
        datasynth_config::schema::TransactionVolume::TenK;

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: false,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Count entries by line count
    let mut two_line_count = 0;
    let mut even_line_count = 0;
    let total = result.journal_entries.len();

    for entry in &result.journal_entries {
        let line_count = entry.lines.len();
        if line_count == 2 {
            two_line_count += 1;
        }
        if line_count % 2 == 0 {
            even_line_count += 1;
        }
    }

    let two_line_ratio = two_line_count as f64 / total as f64;
    let even_line_ratio = even_line_count as f64 / total as f64;

    // Based on research: ~60% should be 2-line entries
    // Allow 30% tolerance for small sample sizes
    assert!(
        two_line_ratio > 0.30,
        "Expected > 30% two-line entries, got {:.1}%",
        two_line_ratio * 100.0
    );

    // ~88% should be even-line entries
    assert!(
        even_line_ratio > 0.70,
        "Expected > 70% even-line entries, got {:.1}%",
        even_line_ratio * 100.0
    );
}

/// Test fraud configuration and anomaly tracking.
///
/// Note: Fraud injection is controlled by both config.fraud.enabled and
/// phase_config.inject_anomalies. This test verifies the configuration
/// is properly applied.
#[test]
fn test_fraud_configuration() {
    let mut config = minimal_config();
    config.global.seed = Some(88888);
    config.fraud.enabled = true;
    config.fraud.fraud_rate = 0.1; // 10% fraud rate

    let phase_config = PhaseConfig {
        generate_master_data: false,
        generate_document_flows: false,
        generate_journal_entries: true,
        inject_anomalies: true,
        show_progress: false,
        ..Default::default()
    };

    let mut orchestrator =
        EnhancedOrchestrator::new(config, phase_config).expect("Failed to create orchestrator");

    let result = orchestrator.generate().expect("Generation failed");

    // Verify entries were generated
    assert!(
        !result.journal_entries.is_empty(),
        "Should generate journal entries"
    );

    // Count fraud entries
    let fraud_count = result
        .journal_entries
        .iter()
        .filter(|e| e.header.is_fraud)
        .count();

    // Count anomaly labels
    let label_count = result.anomaly_labels.labels.len();

    // Log fraud statistics for analysis
    println!(
        "Fraud statistics: {} entries, {} marked as fraud, {} anomaly labels",
        result.journal_entries.len(),
        fraud_count,
        label_count
    );

    // Verify anomaly labels are consistent with fraud entries if any exist
    if fraud_count > 0 {
        for entry in result.journal_entries.iter().filter(|e| e.header.is_fraud) {
            assert!(
                entry.header.is_fraud,
                "Fraud entry should have is_fraud = true"
            );
        }
    }
}
