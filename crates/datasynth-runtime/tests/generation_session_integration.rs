//! Integration tests for GenerationSession multi-period generation.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use datasynth_config::GeneratorConfig;
use datasynth_runtime::generation_session::GenerationSession;

fn minimal_config() -> GeneratorConfig {
    serde_yaml::from_str(
        r#"
global:
  seed: 42
  industry: retail
  start_date: "2024-01-01"
  period_months: 3
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
fn test_session_single_period_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let config = minimal_config();
    let session = GenerationSession::new(config.clone(), tmp.path().to_path_buf()).unwrap();
    let dss_path = tmp.path().join("session.dss");
    session.save(&dss_path).unwrap();
    assert!(dss_path.exists());
    let resumed = GenerationSession::resume(&dss_path, config).unwrap();
    assert_eq!(resumed.remaining_periods(), 1);
}

#[test]
fn test_session_multi_period_planning() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = minimal_config();
    config.global.period_months = 6;
    config.global.fiscal_year_months = Some(3);
    let session = GenerationSession::new(config, tmp.path().to_path_buf()).unwrap();
    assert_eq!(session.periods().len(), 2);
    assert_eq!(session.remaining_periods(), 2);
}

#[test]
fn test_session_config_hash_determinism() {
    let config = minimal_config();
    let s1 = GenerationSession::new(config.clone(), PathBuf::from("/tmp/a")).unwrap();
    let s2 = GenerationSession::new(config, PathBuf::from("/tmp/b")).unwrap();
    assert_eq!(s1.state().config_hash, s2.state().config_hash);
}

#[test]
fn test_session_resume_rejects_different_config() {
    let tmp = tempfile::tempdir().unwrap();
    let config = minimal_config();
    let session = GenerationSession::new(config.clone(), tmp.path().to_path_buf()).unwrap();
    let dss_path = tmp.path().join("session.dss");
    session.save(&dss_path).unwrap();
    let mut different = config;
    different.global.seed = Some(999);
    let result = GenerationSession::resume(&dss_path, different);
    assert!(result.is_err());
}
