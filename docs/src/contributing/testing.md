# Testing

Testing guidelines and practices for DataSynth.

## Running Tests

### All Tests

```bash
# Run all tests
cargo test

# Run with output displayed
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test

# Run tests sequentially
cargo test -- --test-threads=1
```

### Specific Tests

```bash
# Run tests for a specific crate
cargo test -p datasynth-core
cargo test -p datasynth-generators

# Run a single test by name
cargo test test_balanced_entry

# Run tests matching a pattern
cargo test benford
cargo test journal_entry
```

### Test Output

```bash
# Show stdout/stderr from tests
cargo test -- --nocapture

# Show test timing
cargo test -- --show-output

# Run ignored tests
cargo test -- --ignored

# Run all tests including ignored
cargo test -- --include-ignored
```

## Test Organization

### Unit Tests

Place unit tests in the same file as the code:

```rust
// src/generators/je_generator.rs

pub struct JournalEntryGenerator { ... }

impl JournalEntryGenerator {
    pub fn generate(&self) -> Result<JournalEntry> { ... }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_balanced_entry() {
        let generator = JournalEntryGenerator::new(test_config(), 42);
        let entry = generator.generate().unwrap();
        assert!(entry.is_balanced());
    }
}
```

### Integration Tests

Place integration tests in the `tests/` directory:

```
crates/datasynth-generators/
├── src/
│   └── ...
└── tests/
    ├── generation_flow.rs
    └── document_chains.rs
```

### Test Modules

Group related tests in submodules:

```rust
#[cfg(test)]
mod tests {
    mod generation {
        use super::super::*;

        #[test]
        fn batch_generation() { ... }

        #[test]
        fn streaming_generation() { ... }
    }

    mod validation {
        use super::super::*;

        #[test]
        fn rejects_invalid_config() { ... }
    }
}
```

## Test Patterns

### Arrange-Act-Assert

Use the AAA pattern for test structure:

```rust
#[test]
fn calculates_correct_total() {
    // Arrange
    let entries = vec![
        create_entry(dec!(100.00)),
        create_entry(dec!(200.00)),
        create_entry(dec!(300.00)),
    ];

    // Act
    let total = calculate_total(&entries);

    // Assert
    assert_eq!(total, dec!(600.00));
}
```

### Test Fixtures

Create helper functions for common test data:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> GeneratorConfig {
        GeneratorConfig {
            seed: 42,
            batch_size: 100,
            ..Default::default()
        }
    }

    fn create_test_entry() -> JournalEntry {
        JournalEntryBuilder::new()
            .with_company("1000")
            .with_date(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
            .add_line(Account::CASH, dec!(1000.00), Decimal::ZERO)
            .add_line(Account::REVENUE, Decimal::ZERO, dec!(1000.00))
            .build()
            .unwrap()
    }
}
```

### Deterministic Testing

Use fixed seeds for reproducibility:

```rust
#[test]
fn deterministic_generation() {
    let seed = 42;

    let gen1 = Generator::new(config.clone(), seed);
    let gen2 = Generator::new(config.clone(), seed);

    let result1 = gen1.generate_batch(100).unwrap();
    let result2 = gen2.generate_batch(100).unwrap();

    assert_eq!(result1, result2);
}
```

### Property-Based Testing

Use `proptest` for property-based tests:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn entries_are_always_balanced(
        debit in 1u64..1_000_000,
        line_count in 2usize..10,
    ) {
        let entry = generate_entry(debit, line_count);
        prop_assert!(entry.is_balanced());
    }
}
```

## Domain-Specific Tests

### Balance Verification

Test that journal entries are balanced:

```rust
#[test]
fn entry_debits_equal_credits() {
    let entry = generate_test_entry();

    let total_debits: Decimal = entry.lines
        .iter()
        .map(|l| l.debit_amount)
        .sum();

    let total_credits: Decimal = entry.lines
        .iter()
        .map(|l| l.credit_amount)
        .sum();

    assert_eq!(total_debits, total_credits);
}
```

### Benford's Law

Test amount distribution compliance:

```rust
#[test]
fn amounts_follow_benford() {
    let entries = generate_entries(10_000);
    let first_digits = extract_first_digits(&entries);

    let observed = calculate_distribution(&first_digits);
    let expected = benford_distribution();

    let chi_square = calculate_chi_square(&observed, &expected);
    assert!(chi_square < 15.51, "Distribution deviates from Benford's Law");
}
```

### Document Chain Integrity

Test document reference chains:

```rust
#[test]
fn p2p_chain_is_complete() {
    let documents = generate_p2p_flow();

    // Verify chain: PO -> GR -> Invoice -> Payment
    let po = &documents.purchase_order;
    let gr = &documents.goods_receipt;
    let invoice = &documents.vendor_invoice;
    let payment = &documents.payment;

    assert_eq!(gr.po_reference, Some(po.po_number.clone()));
    assert_eq!(invoice.po_reference, Some(po.po_number.clone()));
    assert_eq!(payment.invoice_reference, Some(invoice.invoice_number.clone()));
}
```

### Decimal Precision

Test that decimal values maintain precision:

```rust
#[test]
fn decimal_precision_preserved() {
    let original = dec!(1234.5678);

    // Serialize and deserialize
    let json = serde_json::to_string(&original).unwrap();
    let restored: Decimal = serde_json::from_str(&json).unwrap();

    assert_eq!(original, restored);
}
```

## Benchmarks

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench generation_throughput

# Run benchmark with specific filter
cargo bench -- batch_generation
```

### Writing Benchmarks

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn generation_benchmark(c: &mut Criterion) {
    let config = test_config();

    c.bench_function("generate_1000_entries", |b| {
        b.iter(|| {
            let generator = Generator::new(config.clone(), 42);
            generator.generate_batch(1000).unwrap()
        })
    });
}

fn scaling_benchmark(c: &mut Criterion) {
    let config = test_config();
    let mut group = c.benchmark_group("scaling");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter(|| {
                    let generator = Generator::new(config.clone(), 42);
                    generator.generate_batch(size).unwrap()
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, generation_benchmark, scaling_benchmark);
criterion_main!(benches);
```

## Test Coverage

### Measuring Coverage

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

### Coverage Guidelines

- Aim for 80%+ coverage on core logic
- 100% coverage on public API
- Focus on behavior, not lines
- Don't test trivial getters/setters

## Continuous Integration

Tests run automatically on:

- Pull request creation
- Push to main branch
- Nightly scheduled runs

### CI Test Matrix

| Test Type | Trigger | Platform |
|-----------|---------|----------|
| Unit tests | All PRs | Linux, macOS, Windows |
| Integration tests | All PRs | Linux |
| Benchmarks | Main branch | Linux |
| Coverage | Weekly | Linux |

## See Also

- [Code Style](code-style.md) - Coding standards
- [Pull Requests](pull-requests.md) - Submission process
