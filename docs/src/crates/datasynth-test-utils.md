# datasynth-test-utils

Test utilities and helpers for the DataSynth workspace.

## Overview

`datasynth-test-utils` provides shared testing infrastructure:

- **Test Fixtures**: Pre-configured test data and scenarios
- **Assertion Helpers**: Domain-specific assertions for financial data
- **Mock Generators**: Simplified generators for unit testing
- **Snapshot Testing**: Helpers for snapshot-based testing

## Fixtures

### Journal Entry Fixtures

```rust
use synth_test_utils::fixtures;

// Balanced two-line entry
let entry = fixtures::balanced_journal_entry();
assert!(entry.is_balanced());

// Entry with specific amounts
let entry = fixtures::journal_entry_with_amount(dec!(1000.00));

// Fraudulent entry for testing detection
let entry = fixtures::fraudulent_entry(FraudType::SplitTransaction);
```

### Master Data Fixtures

```rust
// Sample vendors
let vendors = fixtures::sample_vendors(10);

// Sample customers
let customers = fixtures::sample_customers(20);

// Chart of accounts
let coa = fixtures::test_chart_of_accounts();
```

### Amount Fixtures

```rust
// Benford-compliant amounts
let amounts = fixtures::sample_amounts(1000);

// Round-number biased amounts
let amounts = fixtures::round_amounts(100);

// Fraud-pattern amounts
let amounts = fixtures::suspicious_amounts(50);
```

### Configuration Fixtures

```rust
// Minimal valid config
let config = fixtures::test_config();

// Manufacturing preset
let config = fixtures::manufacturing_config();

// Config with specific transaction count
let config = fixtures::config_with_transactions(10000);
```

## Assertions

### Balance Assertions

```rust
use synth_test_utils::assertions;

#[test]
fn test_entry_is_balanced() {
    let entry = create_entry();
    assertions::assert_balanced(&entry);
}

#[test]
fn test_trial_balance() {
    let tb = generate_trial_balance();
    assertions::assert_trial_balance_balanced(&tb);
}
```

### Benford's Law Assertions

```rust
#[test]
fn test_benford_compliance() {
    let amounts = generate_amounts(10000);
    assertions::assert_benford_compliant(&amounts, 0.05);
}
```

### Document Chain Assertions

```rust
#[test]
fn test_p2p_chain() {
    let documents = generate_p2p_flow();
    assertions::assert_valid_document_chain(&documents);
}
```

### Uniqueness Assertions

```rust
#[test]
fn test_no_duplicate_ids() {
    let entries = generate_entries(1000);
    assertions::assert_unique_document_ids(&entries);
}
```

## Mock Generators

### Simple Journal Entry Generator

```rust
use synth_test_utils::mocks::MockJeGenerator;

let mut generator = MockJeGenerator::new(42);

// Generate entries without full config
let entries = generator.generate(100);
```

### Predictable Amount Generator

```rust
use synth_test_utils::mocks::MockAmountGenerator;

let mut generator = MockAmountGenerator::new();

// Returns predictable sequence
let amount1 = generator.next(); // 100.00
let amount2 = generator.next(); // 200.00
```

### Fixed Date Generator

```rust
use synth_test_utils::mocks::MockDateGenerator;

let generator = MockDateGenerator::fixed(
    NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
);
```

## Snapshot Testing

```rust
use synth_test_utils::snapshots;

#[test]
fn test_je_serialization() {
    let entry = fixtures::balanced_journal_entry();
    snapshots::assert_json_snapshot("je_balanced", &entry);
}

#[test]
fn test_csv_output() {
    let entries = fixtures::sample_entries(10);
    snapshots::assert_csv_snapshot("entries_sample", &entries);
}
```

## Test Helpers

### Temporary Directories

```rust
use synth_test_utils::temp_dir;

#[test]
fn test_output_writing() {
    let dir = temp_dir::create();

    // Test writes to temp directory
    let path = dir.path().join("test.csv");
    write_output(&path)?;

    assert!(path.exists());
    // Directory cleaned up on drop
}
```

### Seed Management

```rust
use synth_test_utils::seeds;

#[test]
fn test_deterministic_generation() {
    let seed = seeds::fixed();

    let result1 = generate_with_seed(seed);
    let result2 = generate_with_seed(seed);

    assert_eq!(result1, result2);
}
```

### Time Helpers

```rust
use synth_test_utils::time;

#[test]
fn test_with_frozen_time() {
    let frozen = time::freeze_at(2024, 1, 15);

    let entry = generate_entry_with_current_date();

    assert_eq!(entry.posting_date, frozen.date());
}
```

## Usage in Other Crates

Add to `Cargo.toml`:

```toml
[dev-dependencies]
datasynth-test-utils = { path = "../datasynth-test-utils" }
```

Use in tests:

```rust
#[cfg(test)]
mod tests {
    use synth_test_utils::{fixtures, assertions};

    #[test]
    fn test_my_function() {
        let input = fixtures::test_config();
        let result = my_function(&input);
        assertions::assert_balanced(&result);
    }
}
```

## Fixture Data Files

Test data files in `fixtures/`:

```
datasynth-test-utils/
└── fixtures/
    ├── chart_of_accounts.yaml
    ├── sample_entries.json
    ├── vendor_master.csv
    └── test_config.yaml
```

## See Also

- [Testing Guidelines](../contributing/testing.md)
- [datasynth-eval](datasynth-eval.md)
- [Development Setup](../contributing/development-setup.md)
