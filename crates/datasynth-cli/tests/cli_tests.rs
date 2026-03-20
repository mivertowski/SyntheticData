//! CLI integration tests for synth-data.
//!
//! These tests verify the CLI commands work correctly.
//!
//! IMPORTANT: All tests use strict resource limits to prevent system hangs.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

/// Safe resource limits for tests
const TEST_MEMORY_LIMIT: &str = "512";
const TEST_MAX_THREADS: &str = "1";
const TEST_TIMEOUT_SECS: u64 = 300; // 5 minutes — CI coverage instrumentation is slow

/// Get a Command for our binary with timeout.
#[allow(deprecated)] // cargo_bin is still functional, just has a new alternative
fn synth_data() -> Command {
    let mut cmd = Command::cargo_bin("datasynth-data").unwrap();
    cmd.timeout(Duration::from_secs(TEST_TIMEOUT_SECS));
    cmd
}

/// Get a Command for generate with safe resource limits.
fn synth_data_generate() -> Command {
    let mut cmd = synth_data();
    cmd.arg("generate")
        .arg("--memory-limit")
        .arg(TEST_MEMORY_LIMIT)
        .arg("--max-threads")
        .arg(TEST_MAX_THREADS);
    cmd
}

// ==========================================================================
// Help and Version Tests
// ==========================================================================

#[test]
fn test_help_flag() {
    synth_data()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Synthetic Enterprise Accounting Data Generator",
        ));
}

#[test]
fn test_version_flag() {
    synth_data()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("datasynth-data"));
}

#[test]
fn test_no_subcommand_shows_help() {
    synth_data()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

// ==========================================================================
// Info Command Tests
// ==========================================================================

#[test]
fn test_info_command() {
    synth_data()
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Industry Presets"))
        .stdout(predicate::str::contains("manufacturing"))
        .stdout(predicate::str::contains("retail"))
        .stdout(predicate::str::contains("healthcare"))
        .stdout(predicate::str::contains("technology"));
}

#[test]
fn test_info_shows_complexity_levels() {
    synth_data()
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("small"))
        .stdout(predicate::str::contains("medium"))
        .stdout(predicate::str::contains("large"));
}

#[test]
fn test_info_shows_transaction_volumes() {
    synth_data()
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("ten_k"))
        .stdout(predicate::str::contains("hundred_k"))
        .stdout(predicate::str::contains("one_m"));
}

// ==========================================================================
// Init Command Tests
// ==========================================================================

#[test]
fn test_init_creates_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.yaml");

    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();

    assert!(config_path.exists(), "Config file should be created");

    // Verify it's valid YAML
    let content = fs::read_to_string(&config_path).unwrap();
    let _config: datasynth_config::GeneratorConfig =
        serde_yaml::from_str(&content).expect("Should be valid config");
}

#[test]
fn test_init_with_industry_manufacturing() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("manufacturing.yaml");

    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .arg("-i")
        .arg("manufacturing")
        .assert()
        .success();

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("manufacturing"));
}

#[test]
fn test_init_with_industry_retail() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("retail.yaml");

    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .arg("-i")
        .arg("retail")
        .assert()
        .success();

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("retail"));
}

#[test]
fn test_init_with_complexity_small() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("small.yaml");

    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .arg("-c")
        .arg("small")
        .assert()
        .success();

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("small"));
}

#[test]
fn test_init_with_complexity_large() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("large.yaml");

    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .arg("-c")
        .arg("large")
        .assert()
        .success();

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("large"));
}

// ==========================================================================
// Validate Command Tests
// ==========================================================================

#[test]
fn test_validate_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("valid_config.yaml");

    // First create a valid config
    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();

    // Then validate it
    synth_data()
        .arg("validate")
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_validate_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.yaml");

    // Write invalid YAML
    fs::write(&config_path, "this is not: valid: yaml: {{{{").unwrap();

    synth_data()
        .arg("validate")
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .assert()
        .failure();
}

#[test]
fn test_validate_missing_file() {
    synth_data()
        .arg("validate")
        .arg("-c")
        .arg("/nonexistent/path/config.yaml")
        .assert()
        .failure();
}

#[test]
fn test_validate_requires_config_arg() {
    synth_data()
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--config"));
}

// ==========================================================================
// Generate Command Tests
// ==========================================================================

#[test]
fn test_generate_demo_creates_output() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");

    synth_data_generate()
        .arg("--demo")
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .assert()
        .success();

    assert!(output_dir.exists(), "Output directory should be created");

    // Check for journal entries output files
    let je_csv_path = output_dir.join("journal_entries.csv");
    let je_json_path = output_dir.join("journal_entries.json");
    assert!(
        je_csv_path.exists() || je_json_path.exists(),
        "Journal entries output file should be created"
    );

    // Verify JSON is valid if it exists
    if je_json_path.exists() {
        let content = fs::read_to_string(&je_json_path).unwrap();
        let _: serde_json::Value = serde_json::from_str(&content).expect("Should be valid JSON");
    }
}

#[test]
fn test_generate_with_seed() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");

    synth_data_generate()
        .arg("--demo")
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .arg("-s")
        .arg("12345")
        .assert()
        .success();

    assert!(output_dir.exists());
}

#[test]
fn test_generate_from_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    let output_dir = temp_dir.path().join("output");

    // Create config
    synth_data()
        .arg("init")
        .arg("-o")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success();

    // Generate from config (with safe resource limits)
    synth_data_generate()
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .assert()
        .success();

    assert!(output_dir.exists());
}

#[test]
fn test_generate_defaults_to_demo_preset() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");

    // Running without --demo or --config should default to demo preset
    // (with safe resource limits)
    synth_data_generate()
        .arg("-o")
        .arg(output_dir.to_str().unwrap())
        .assert()
        .success();

    assert!(output_dir.exists());
}

// ==========================================================================
// Verbose Flag Tests
// ==========================================================================

#[test]
fn test_verbose_flag_accepted() {
    synth_data().arg("-v").arg("info").assert().success();
}

#[test]
fn test_verbose_long_flag_accepted() {
    synth_data().arg("--verbose").arg("info").assert().success();
}
