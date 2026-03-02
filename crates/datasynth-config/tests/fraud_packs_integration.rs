//! Integration tests for fraud scenario packs.

#![allow(clippy::unwrap_used)]

use datasynth_config::fraud_packs::{apply_fraud_packs, FRAUD_PACKS};
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
