# Code Style

Coding standards and conventions for DataSynth.

## Rust Style

### Formatting

All code must be formatted with `rustfmt`:

```bash
# Format all code
cargo fmt

# Check formatting without changes
cargo fmt --check
```

### Linting

Code must pass Clippy without warnings:

```bash
# Run clippy
cargo clippy

# Run clippy with all features
cargo clippy --all-features

# Run clippy on all targets
cargo clippy --all-targets
```

### Configuration

The project uses these Clippy settings in `Cargo.toml`:

```toml
[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

## Naming Conventions

### General Rules

| Item | Convention | Example |
|------|------------|---------|
| Types | PascalCase | `JournalEntry`, `VendorGenerator` |
| Functions | snake_case | `generate_batch`, `parse_config` |
| Variables | snake_case | `entry_count`, `total_amount` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_LINE_ITEMS`, `DEFAULT_SEED` |
| Modules | snake_case | `je_generator`, `document_flow` |

### Domain-Specific Names

Use accounting domain terminology consistently:

```rust
// Good - uses domain terms
struct JournalEntry { ... }
struct ChartOfAccounts { ... }
fn post_to_gl() { ... }

// Avoid - generic terms
struct Entry { ... }
struct AccountList { ... }
fn save_data() { ... }
```

## Code Organization

### Module Structure

```rust
// 1. Module documentation
//! Brief description of the module.
//!
//! Extended description with examples.

// 2. Imports (grouped and sorted)
use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::JournalEntry;

// 3. Constants
const DEFAULT_BATCH_SIZE: usize = 1000;

// 4. Type definitions
pub struct Generator { ... }

// 5. Trait implementations
impl Generator { ... }

// 6. Unit tests
#[cfg(test)]
mod tests { ... }
```

### Import Organization

Group imports in this order:

1. Standard library (`std::`)
2. External crates (alphabetically)
3. Workspace crates (`synth_*`)
4. Current crate (`crate::`)

```rust
use std::collections::HashMap;
use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use synth_core::models::JournalEntry;
use synth_core::traits::Generator;

use crate::config::GeneratorConfig;
```

## Documentation

### Public API Documentation

All public items must have documentation:

```rust
/// Generates journal entries with realistic financial patterns.
///
/// This generator produces balanced journal entries following
/// configurable statistical distributions for amounts, line counts,
/// and temporal patterns.
///
/// # Examples
///
/// ```
/// use synth_generators::JournalEntryGenerator;
///
/// let generator = JournalEntryGenerator::new(config, seed);
/// let entries = generator.generate_batch(1000)?;
/// ```
///
/// # Errors
///
/// Returns `GeneratorError` if:
/// - Configuration is invalid
/// - Memory limits are exceeded
pub struct JournalEntryGenerator { ... }
```

### Module Documentation

Each module should have a module-level doc comment:

```rust
//! Journal Entry generation module.
//!
//! This module provides generators for creating realistic
//! journal entries with proper accounting rules enforcement.
//!
//! # Overview
//!
//! The main entry point is [`JournalEntryGenerator`], which
//! coordinates line item generation and balance verification.
```

## Error Handling

### Error Types

Use `thiserror` for error definitions:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Memory limit exceeded: used {used} bytes, limit {limit} bytes")]
    MemoryExceeded { used: usize, limit: usize },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Result Types

Define type aliases for common result types:

```rust
pub type Result<T> = std::result::Result<T, GeneratorError>;
```

### Error Propagation

Use `?` for error propagation:

```rust
// Good
fn process() -> Result<Data> {
    let config = load_config()?;
    let data = generate_data(&config)?;
    Ok(data)
}

// Avoid
fn process() -> Result<Data> {
    let config = match load_config() {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    // ...
}
```

## Financial Data

### Decimal Precision

Always use `rust_decimal::Decimal` for financial amounts:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Good
let amount: Decimal = dec!(1234.56);

// Avoid - floating point
let amount: f64 = 1234.56;
```

### Serialization

Serialize decimals as strings to avoid precision loss:

```rust
#[derive(Serialize, Deserialize)]
pub struct LineItem {
    #[serde(serialize_with = "serialize_decimal_as_string")]
    pub amount: Decimal,
}
```

## Testing

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Group related tests
    mod generation {
        use super::*;

        #[test]
        fn generates_balanced_entries() {
            // Arrange
            let config = test_config();
            let generator = Generator::new(config, 42);

            // Act
            let entries = generator.generate_batch(100).unwrap();

            // Assert
            for entry in entries {
                assert!(entry.is_balanced());
            }
        }
    }

    mod validation {
        // ...
    }
}
```

### Test Naming

Use descriptive test names:

```rust
// Good - describes behavior
#[test]
fn rejects_unbalanced_entry() { ... }

#[test]
fn generates_benford_compliant_amounts() { ... }

// Avoid - vague names
#[test]
fn test_1() { ... }

#[test]
fn it_works() { ... }
```

## Performance

### Allocation

Minimize allocations in hot paths:

```rust
// Good - reuse buffer
let mut buffer = Vec::with_capacity(batch_size);
for _ in 0..batch_size {
    buffer.push(generate_entry()?);
}

// Avoid - reallocations
let mut buffer = Vec::new();
for _ in 0..batch_size {
    buffer.push(generate_entry()?);
}
```

### Iterator Usage

Prefer iterators over explicit loops:

```rust
// Good
let total: Decimal = entries
    .iter()
    .map(|e| e.amount)
    .sum();

// Avoid
let mut total = Decimal::ZERO;
for entry in &entries {
    total += entry.amount;
}
```

## See Also

- [Testing](testing.md) - Testing guidelines
- [Pull Requests](pull-requests.md) - Submission process
