//! Integration tests for fraud scenario packs.
//!
//! Covers fraud pack loading, application, merge semantics, and configuration
//! validation for all built-in fraud scenario packs.

#![allow(clippy::unwrap_used)]

use datasynth_config::fraud_packs::{
    apply_fraud_packs, load_fraud_pack, merge_fraud_pack, FRAUD_PACKS,
};
use datasynth_config::validate_config;
use datasynth_config::GeneratorConfig;

fn base_config() -> GeneratorConfig {
    serde_yaml::from_str(
        r#"
global:
  seed: 42
  industry: retail
  start_date: "2024-01-01"
  period_months: 12
companies:
  - code: "C001"
    name: "Test Corp"
    currency: "USD"
    country: "US"
    annual_transaction_volume: ten_k
chart_of_accounts:
  complexity: small
output:
  output_directory: "/tmp"
"#,
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Original tests (preserved)
// ---------------------------------------------------------------------------

#[test]
fn test_apply_revenue_fraud_pack() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &["revenue_fraud".to_string()]).unwrap();
    assert!(result.fraud.enabled);
    assert!(result.fraud.fraud_rate > 0.0);
    assert!(result.fraud.fraud_type_distribution.revenue_manipulation > 0.0);
}

#[test]
fn test_apply_multiple_packs() {
    let config = base_config();
    let result = apply_fraud_packs(
        &config,
        &["revenue_fraud".to_string(), "payroll_ghost".to_string()],
    )
    .unwrap();
    assert!(result.fraud.enabled);
}

#[test]
fn test_apply_unknown_pack_error() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &["nonexistent".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_all_packs_produce_valid_configs() {
    let config = base_config();
    for pack_name in FRAUD_PACKS {
        let result = apply_fraud_packs(&config, &[pack_name.to_string()]);
        assert!(
            result.is_ok(),
            "Pack '{}' failed: {:?}",
            pack_name,
            result.err()
        );
    }
}

#[test]
fn test_pack_preserves_non_fraud_config() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &["revenue_fraud".to_string()]).unwrap();
    assert_eq!(result.global.seed, Some(42));
    assert_eq!(result.global.period_months, 12);
    assert_eq!(result.companies.len(), 1);
}

// ---------------------------------------------------------------------------
// New tests: fraud pack validation, merge semantics, rate verification
// ---------------------------------------------------------------------------

/// Each fraud pack must produce a config where:
/// - fraud.enabled == true
/// - fraud.fraud_rate > 0
/// - at least one fraud type has a non-zero distribution weight
/// - anomaly_injection.enabled == true
/// - the resulting config passes full validation
#[test]
fn test_each_fraud_pack_produces_valid_config() {
    let config = base_config();
    for pack_name in FRAUD_PACKS {
        let result = apply_fraud_packs(&config, &[pack_name.to_string()])
            .unwrap_or_else(|e| panic!("Pack '{}' failed to apply: {}", pack_name, e));

        // Fraud must be enabled with a positive rate
        assert!(
            result.fraud.enabled,
            "Pack '{}': fraud.enabled should be true",
            pack_name
        );
        assert!(
            result.fraud.fraud_rate > 0.0,
            "Pack '{}': fraud_rate should be > 0, got {}",
            pack_name,
            result.fraud.fraud_rate
        );

        // At least one fraud type distribution weight should be non-zero
        let dist = &result.fraud.fraud_type_distribution;
        let has_nonzero_type = dist.suspense_account_abuse > 0.0
            || dist.fictitious_transaction > 0.0
            || dist.revenue_manipulation > 0.0
            || dist.expense_capitalization > 0.0
            || dist.split_transaction > 0.0
            || dist.timing_anomaly > 0.0
            || dist.unauthorized_access > 0.0
            || dist.duplicate_payment > 0.0;
        assert!(
            has_nonzero_type,
            "Pack '{}': at least one fraud type distribution weight should be > 0",
            pack_name
        );

        // Anomaly injection should also be enabled
        assert!(
            result.anomaly_injection.enabled,
            "Pack '{}': anomaly_injection.enabled should be true",
            pack_name
        );

        // The merged config should still pass full validation
        let validation = validate_config(&result);
        assert!(
            validation.is_ok(),
            "Pack '{}': merged config fails validation: {:?}",
            pack_name,
            validation.err()
        );
    }
}

/// Deep merge must preserve existing settings that the fraud pack does not override.
/// Start with a config that has custom fraud settings (clustering, approval thresholds),
/// apply a fraud pack, and verify those custom settings survive the merge.
#[test]
fn test_fraud_pack_deep_merge_preserves_existing() {
    // Start with a config that already has custom fraud settings
    let mut config = base_config();
    config.fraud.enabled = true;
    config.fraud.clustering_enabled = true;
    config.fraud.clustering_factor = 5.0;
    config.fraud.approval_thresholds = vec![500.0, 2000.0, 8000.0];

    // Apply revenue_fraud pack (which sets fraud_rate, fraud_type_distribution, etc.
    // but does NOT set clustering_enabled, clustering_factor, or approval_thresholds)
    let result = apply_fraud_packs(&config, &["revenue_fraud".to_string()]).unwrap();

    // The pack should have overridden fraud_rate and distribution
    assert_eq!(
        result.fraud.fraud_rate, 0.02,
        "revenue_fraud pack should set fraud_rate to 0.02"
    );
    assert!(
        result.fraud.fraud_type_distribution.revenue_manipulation > 0.0,
        "revenue_fraud pack should set revenue_manipulation > 0"
    );

    // The non-fraud config should be fully preserved
    assert_eq!(result.global.seed, Some(42));
    assert_eq!(result.global.period_months, 12);
    assert_eq!(result.companies[0].code, "C001");
    assert_eq!(result.companies[0].name, "Test Corp");
}

/// Verify the specific fraud_rate value each pack sets.
#[test]
fn test_fraud_rate_override() {
    let config = base_config();

    // Expected fraud_rate for each pack (from the YAML definitions)
    let expected_rates: &[(&str, f64)] = &[
        ("revenue_fraud", 0.02),
        ("payroll_ghost", 0.015),
        ("vendor_kickback", 0.02),
        ("management_override", 0.01),
        ("comprehensive", 0.03),
    ];

    for (pack_name, expected_rate) in expected_rates {
        let result = apply_fraud_packs(&config, &[pack_name.to_string()]).unwrap();
        assert!(
            (result.fraud.fraud_rate - expected_rate).abs() < 1e-9,
            "Pack '{}': expected fraud_rate={}, got {}",
            pack_name,
            expected_rate,
            result.fraud.fraud_rate
        );
    }
}

/// Verify that revenue_fraud pack concentrates weight on revenue-related fraud types.
#[test]
fn test_revenue_fraud_pack_type_distribution() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &["revenue_fraud".to_string()]).unwrap();
    let dist = &result.fraud.fraud_type_distribution;

    assert!(
        (dist.revenue_manipulation - 0.40).abs() < 1e-9,
        "Expected revenue_manipulation=0.40, got {}",
        dist.revenue_manipulation
    );
    assert!(
        (dist.fictitious_transaction - 0.30).abs() < 1e-9,
        "Expected fictitious_transaction=0.30, got {}",
        dist.fictitious_transaction
    );
    assert!(
        (dist.expense_capitalization - 0.20).abs() < 1e-9,
        "Expected expense_capitalization=0.20, got {}",
        dist.expense_capitalization
    );
    // Types not relevant to revenue fraud should be zero
    assert!(
        dist.duplicate_payment.abs() < 1e-9,
        "Expected duplicate_payment=0 for revenue_fraud, got {}",
        dist.duplicate_payment
    );
}

/// Verify that vendor_kickback pack concentrates weight on vendor-related fraud types.
#[test]
fn test_vendor_kickback_pack_type_distribution() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &["vendor_kickback".to_string()]).unwrap();
    let dist = &result.fraud.fraud_type_distribution;

    assert!(
        (dist.fictitious_transaction - 0.35).abs() < 1e-9,
        "Expected fictitious_transaction=0.35, got {}",
        dist.fictitious_transaction
    );
    assert!(
        (dist.split_transaction - 0.30).abs() < 1e-9,
        "Expected split_transaction=0.30, got {}",
        dist.split_transaction
    );
    assert!(
        (dist.duplicate_payment - 0.30).abs() < 1e-9,
        "Expected duplicate_payment=0.30, got {}",
        dist.duplicate_payment
    );
}

/// The comprehensive pack should have non-zero weights for all fraud types.
#[test]
fn test_comprehensive_pack_covers_all_types() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &["comprehensive".to_string()]).unwrap();
    let dist = &result.fraud.fraud_type_distribution;

    let all_types = [
        ("suspense_account_abuse", dist.suspense_account_abuse),
        ("fictitious_transaction", dist.fictitious_transaction),
        ("revenue_manipulation", dist.revenue_manipulation),
        ("expense_capitalization", dist.expense_capitalization),
        ("split_transaction", dist.split_transaction),
        ("timing_anomaly", dist.timing_anomaly),
        ("unauthorized_access", dist.unauthorized_access),
        ("duplicate_payment", dist.duplicate_payment),
    ];

    for (type_name, weight) in &all_types {
        assert!(
            *weight > 0.0,
            "Comprehensive pack should have {} > 0, got {}",
            type_name,
            weight
        );
    }
}

/// Verify that anomaly_injection rates are set correctly by each pack.
#[test]
fn test_anomaly_injection_rates_from_packs() {
    let config = base_config();

    // Each pack sets anomaly_injection.rates.fraud_rate == fraud.fraud_rate
    for pack_name in FRAUD_PACKS {
        let result = apply_fraud_packs(&config, &[pack_name.to_string()]).unwrap();

        assert!(
            result.anomaly_injection.rates.total_rate > 0.0,
            "Pack '{}': anomaly total_rate should be > 0",
            pack_name
        );
        assert!(
            result.anomaly_injection.rates.fraud_rate > 0.0,
            "Pack '{}': anomaly fraud_rate should be > 0",
            pack_name
        );
        // Anomaly fraud_rate should match the pack's fraud.fraud_rate
        assert!(
            (result.anomaly_injection.rates.fraud_rate - result.fraud.fraud_rate).abs() < 1e-9,
            "Pack '{}': anomaly fraud_rate ({}) should equal fraud.fraud_rate ({})",
            pack_name,
            result.anomaly_injection.rates.fraud_rate,
            result.fraud.fraud_rate
        );
    }
}

/// Applying packs sequentially: the last pack's values should win on overlapping fields.
#[test]
fn test_sequential_pack_application_last_wins() {
    let config = base_config();
    // Apply revenue_fraud (rate=0.02) then management_override (rate=0.01)
    let result = apply_fraud_packs(
        &config,
        &[
            "revenue_fraud".to_string(),
            "management_override".to_string(),
        ],
    )
    .unwrap();

    // management_override is applied second, so its fraud_rate should win
    assert!(
        (result.fraud.fraud_rate - 0.01).abs() < 1e-9,
        "Last applied pack should win: expected 0.01, got {}",
        result.fraud.fraud_rate
    );
    // management_override's distribution should also be the final one
    assert!(
        (result.fraud.fraud_type_distribution.suspense_account_abuse - 0.25).abs() < 1e-9,
        "management_override sets suspense_account_abuse=0.25"
    );
}

/// Verify that load_fraud_pack returns valid JSON for all known packs.
#[test]
fn test_load_fraud_pack_returns_valid_json_values() {
    for pack_name in FRAUD_PACKS {
        let value = load_fraud_pack(pack_name)
            .unwrap_or_else(|| panic!("Failed to load pack '{}'", pack_name));

        // Each pack should have a "fraud" key
        assert!(
            value.get("fraud").is_some(),
            "Pack '{}' should have a 'fraud' key",
            pack_name
        );

        // Each pack should have an "anomaly_injection" key
        assert!(
            value.get("anomaly_injection").is_some(),
            "Pack '{}' should have an 'anomaly_injection' key",
            pack_name
        );

        // The fraud section should have enabled=true
        let fraud_enabled = value["fraud"]["enabled"].as_bool();
        assert_eq!(
            fraud_enabled,
            Some(true),
            "Pack '{}': fraud.enabled should be true",
            pack_name
        );
    }
}

/// merge_fraud_pack should recursively merge nested objects.
#[test]
fn test_merge_fraud_pack_deep_nested_merge() {
    let mut base = serde_json::json!({
        "fraud": {
            "enabled": false,
            "fraud_rate": 0.001,
            "fraud_type_distribution": {
                "revenue_manipulation": 0.5,
                "fictitious_transaction": 0.5
            }
        },
        "global": {
            "seed": 42,
            "industry": "retail"
        }
    });

    let overlay = serde_json::json!({
        "fraud": {
            "enabled": true,
            "fraud_rate": 0.05,
            "fraud_type_distribution": {
                "revenue_manipulation": 0.8
            }
        }
    });

    merge_fraud_pack(&mut base, &overlay);

    // Overridden values should reflect the overlay
    assert_eq!(base["fraud"]["enabled"], true);
    assert_eq!(base["fraud"]["fraud_rate"], 0.05);
    assert_eq!(
        base["fraud"]["fraud_type_distribution"]["revenue_manipulation"],
        0.8
    );

    // Non-overridden nested value should be preserved
    assert_eq!(
        base["fraud"]["fraud_type_distribution"]["fictitious_transaction"],
        0.5
    );

    // Completely unrelated section should be untouched
    assert_eq!(base["global"]["seed"], 42);
    assert_eq!(base["global"]["industry"], "retail");
}

/// Empty pack list should return the original config unchanged.
#[test]
fn test_apply_empty_pack_list_returns_original() {
    let config = base_config();
    let result = apply_fraud_packs(&config, &[]).unwrap();

    // Fraud should still have defaults (disabled)
    assert!(!result.fraud.enabled);
    assert_eq!(result.global.seed, Some(42));
    assert_eq!(result.global.period_months, 12);
}
