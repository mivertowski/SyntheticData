# datasynth-core

Core domain models, traits, and distributions for synthetic accounting data generation.

## Overview

`datasynth-core` provides the foundational building blocks for the SyntheticData workspace:

- **Domain Models**: Journal entries, chart of accounts, master data, documents, anomalies
- **Statistical Distributions**: Line item sampling, amount generation, temporal patterns
- **Core Traits**: Generator and Sink interfaces for extensibility
- **Template System**: File-based templates for regional/sector customization
- **Infrastructure**: UUID factory, memory guard, GL account constants

## Module Structure

### Domain Models (`models/`)

| Module | Description |
|--------|-------------|
| `journal_entry.rs` | Journal entry header and balanced line items |
| `chart_of_accounts.rs` | Hierarchical GL accounts with account types |
| `master_data.rs` | Enhanced vendors, customers with payment behavior |
| `documents.rs` | Purchase orders, invoices, goods receipts, payments |
| `temporal.rs` | Bi-temporal data model for audit trails |
| `anomaly.rs` | Anomaly types and labels for ML training |
| `internal_control.rs` | SOX 404 control definitions |

### Statistical Distributions (`distributions/`)

| Distribution | Description |
|--------------|-------------|
| `LineItemSampler` | Empirical distribution (60.68% two-line, 88% even counts) |
| `AmountSampler` | Log-normal with round-number bias, Benford compliance |
| `TemporalSampler` | Seasonality patterns with industry integration |
| `BenfordSampler` | First-digit distribution following P(d) = log10(1 + 1/d) |
| `FraudAmountGenerator` | Suspicious amount patterns |
| `IndustrySeasonality` | Industry-specific volume patterns |
| `HolidayCalendar` | Regional holidays for US, DE, GB, CN, JP, IN |

### Infrastructure

| Component | Description |
|-----------|-------------|
| `uuid_factory.rs` | Deterministic FNV-1a hash-based UUID generation |
| `accounts.rs` | Centralized GL control account numbers |
| `templates/` | YAML/JSON template loading and merging |

### Resource Guards

| Component | Description |
|-----------|-------------|
| `memory_guard.rs` | Cross-platform memory tracking with soft/hard limits |
| `disk_guard.rs` | Disk space monitoring and pre-write capacity checks |
| `cpu_monitor.rs` | CPU load tracking with auto-throttling |
| `resource_guard.rs` | Unified orchestration of all resource guards |
| `degradation.rs` | Graceful degradation system (Normal→Reduced→Minimal→Emergency) |

### AI & ML Modules (v0.5.0)

| Module | Description |
|--------|-------------|
| `llm/provider.rs` | `LlmProvider` trait with `complete()` and `complete_batch()` methods |
| `llm/mock_provider.rs` | Deterministic `MockLlmProvider` for testing (no network required) |
| `llm/http_provider.rs` | `HttpLlmProvider` for OpenAI, Anthropic, and custom API endpoints |
| `llm/nl_config.rs` | `NlConfigGenerator` — natural language to YAML configuration |
| `llm/cache.rs` | `LlmCache` with FNV-1a hashing for prompt deduplication |
| `diffusion/backend.rs` | `DiffusionBackend` trait with `forward()`, `reverse()`, `generate()` methods |
| `diffusion/schedule.rs` | `NoiseSchedule` with linear, cosine, and sigmoid schedules |
| `diffusion/statistical.rs` | `StatisticalDiffusionBackend` — fingerprint-guided denoising |
| `diffusion/hybrid.rs` | `HybridGenerator` with Interpolate, Select, Ensemble blend strategies |
| `diffusion/training.rs` | `DiffusionTrainer` and `TrainedDiffusionModel` with save/load |
| `causal/graph.rs` | `CausalGraph` with variables, edges, and built-in templates |
| `causal/scm.rs` | `StructuralCausalModel` with topological-order generation |
| `causal/intervention.rs` | `InterventionEngine` with do-calculus and effect estimation |
| `causal/counterfactual.rs` | `CounterfactualGenerator` with abduction-action-prediction |
| `causal/validation.rs` | `CausalValidator` for causal structure validation |

## Key Types

### JournalEntry

```rust
pub struct JournalEntry {
    pub header: JournalEntryHeader,
    pub lines: Vec<JournalEntryLine>,
}

pub struct JournalEntryHeader {
    pub document_id: Uuid,
    pub company_code: String,
    pub fiscal_year: u16,
    pub fiscal_period: u8,
    pub posting_date: NaiveDate,
    pub document_date: NaiveDate,
    pub source: TransactionSource,
    pub business_process: Option<BusinessProcess>,
    pub is_fraud: bool,
    pub fraud_type: Option<FraudType>,
    pub is_anomaly: bool,
    pub anomaly_type: Option<AnomalyType>,
    // ... additional fields
}
```

### AccountType Hierarchy

```rust
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

pub enum AccountSubType {
    // Assets
    Cash,
    AccountsReceivable,
    Inventory,
    FixedAsset,
    // Liabilities
    AccountsPayable,
    AccruedLiabilities,
    LongTermDebt,
    // Equity
    CommonStock,
    RetainedEarnings,
    // Revenue
    SalesRevenue,
    ServiceRevenue,
    // Expense
    CostOfGoodsSold,
    OperatingExpense,
    // ...
}
```

### Anomaly Types

```rust
pub enum AnomalyType {
    Fraud,
    Error,
    ProcessIssue,
    Statistical,
    Relational,
}

pub struct LabeledAnomaly {
    pub document_id: Uuid,
    pub anomaly_id: String,
    pub anomaly_type: AnomalyType,
    pub category: AnomalyCategory,
    pub severity: Severity,
    pub description: String,
}
```

## Usage Examples

### Creating a Balanced Journal Entry

```rust
use synth_core::models::{JournalEntry, JournalEntryLine, JournalEntryHeader};
use rust_decimal_macros::dec;

let header = JournalEntryHeader::new(/* ... */);
let mut entry = JournalEntry::new(header);

// Add balanced lines
entry.add_line(JournalEntryLine::debit("1100", dec!(1000.00), "AR Invoice"));
entry.add_line(JournalEntryLine::credit("4000", dec!(1000.00), "Revenue"));

// Entry enforces debits = credits
assert!(entry.is_balanced());
```

### Sampling Amounts

```rust
use synth_core::distributions::AmountSampler;

let sampler = AmountSampler::new(42); // seed

// Benford-compliant amount
let amount = sampler.sample_benford_compliant(1000.0, 100000.0);

// Round-number biased
let round_amount = sampler.sample_with_round_bias(1000.0, 10000.0);
```

### Using the UUID Factory

```rust
use synth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};

let factory = DeterministicUuidFactory::new(42);

// Generate collision-free UUIDs across generators
let je_id = factory.generate(GeneratorType::JournalEntry);
let doc_id = factory.generate(GeneratorType::DocumentFlow);
```

### Memory Guard

```rust
use synth_core::memory_guard::{MemoryGuard, MemoryGuardConfig};

let config = MemoryGuardConfig {
    soft_limit: 1024 * 1024 * 1024,  // 1GB soft
    hard_limit: 2 * 1024 * 1024 * 1024, // 2GB hard
    check_interval_ms: 1000,
    ..Default::default()
};

let guard = MemoryGuard::new(config);
if guard.check().exceeds_soft_limit {
    // Slow down or pause generation
}
```

### Disk Space Guard

```rust
use synth_core::disk_guard::{DiskSpaceGuard, DiskSpaceGuardConfig};

let config = DiskSpaceGuardConfig {
    hard_limit_mb: 100,        // Require at least 100 MB free
    soft_limit_mb: 500,        // Warn when below 500 MB
    check_interval: 500,       // Check every 500 operations
    reserve_buffer_mb: 50,     // Keep 50 MB buffer
    monitor_path: Some("./output".into()),
};

let guard = DiskSpaceGuard::new(config);
guard.check()?;  // Returns error if disk full
guard.check_before_write(1024 * 1024)?;  // Pre-write check
```

### CPU Monitor

```rust
use synth_core::cpu_monitor::{CpuMonitor, CpuMonitorConfig};

let config = CpuMonitorConfig::with_thresholds(0.85, 0.95)
    .with_auto_throttle(50);  // 50ms delay when critical

let monitor = CpuMonitor::new(config);

// Sample and check in generation loop
if let Some(load) = monitor.sample() {
    if monitor.is_throttling() {
        monitor.maybe_throttle();  // Apply delay
    }
}
```

### Graceful Degradation

```rust
use synth_core::degradation::{
    DegradationController, DegradationConfig, ResourceStatus, DegradationActions
};

let controller = DegradationController::new(DegradationConfig::default());

let status = ResourceStatus::new(
    Some(0.80),   // 80% memory usage
    Some(800),    // 800 MB disk free
    Some(0.70),   // 70% CPU load
);

let (level, changed) = controller.update(&status);
let actions = DegradationActions::for_level(level);

if actions.skip_data_quality {
    // Skip data quality injection
}
if actions.terminate {
    // Flush and exit gracefully
}
```

### LLM Provider

```rust
use synth_core::llm::{LlmProvider, LlmRequest, MockLlmProvider};

let provider = MockLlmProvider::new(42);
let request = LlmRequest::new("Generate a realistic vendor name for a manufacturing company")
    .with_seed(42)
    .with_max_tokens(50);
let response = provider.complete(&request)?;
println!("Generated: {}", response.content);
```

### Causal Generation

```rust
use synth_core::causal::{CausalGraph, StructuralCausalModel};

// Use built-in fraud detection template
let graph = CausalGraph::fraud_detection_template();
let scm = StructuralCausalModel::new(graph)?;

// Generate observational samples
let samples = scm.generate(1000, 42)?;

// Run intervention: what if transaction_amount is set to 50000?
let intervened = scm.intervene("transaction_amount", 50000.0)?;
let intervention_samples = intervened.generate(1000, 42)?;
```

### Diffusion Model

```rust
use synth_core::diffusion::{
    StatisticalDiffusionBackend, DiffusionConfig, NoiseScheduleType,
    HybridGenerator, BlendStrategy,
};

let config = DiffusionConfig {
    n_steps: 1000,
    schedule: NoiseScheduleType::Cosine,
    seed: 42,
};

let backend = StatisticalDiffusionBackend::new(
    vec![100.0, 5.0],  // means
    vec![50.0, 2.0],   // stds
    config,
);

let samples = backend.generate(1000, 2, 42);

// Hybrid: blend rule-based + diffusion
let hybrid = HybridGenerator::new(0.3); // 30% diffusion weight
let blended = hybrid.blend(&rule_based, &samples, BlendStrategy::Ensemble, 42);
```

## Traits

### Generator Trait

```rust
pub trait Generator {
    type Output;
    type Error;

    fn generate_batch(&mut self, count: usize) -> Result<Vec<Self::Output>, Self::Error>;

    fn generate_stream(&mut self) -> impl Iterator<Item = Result<Self::Output, Self::Error>>;
}
```

### Sink Trait

```rust
pub trait Sink<T> {
    type Error;

    fn write(&mut self, item: &T) -> Result<(), Self::Error>;
    fn write_batch(&mut self, items: &[T]) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
}
```

### PostProcessor Trait

Interface for post-generation data transformations (e.g., data quality variations):

```rust
pub struct ProcessContext {
    pub record_index: usize,
    pub batch_size: usize,
    pub output_format: String,
    pub metadata: HashMap<String, String>,
}

pub struct ProcessorStats {
    pub records_processed: usize,
    pub records_modified: usize,
    pub labels_generated: usize,
}
```

## Template System

Load external templates for customization:

```rust
use synth_core::templates::{TemplateLoader, MergeStrategy};

let loader = TemplateLoader::new("templates/");
let names = loader.load_category("vendor_names", MergeStrategy::Extend)?;
```

**Template categories:**
- `person_names`
- `vendor_names`
- `customer_names`
- `material_descriptions`
- `line_item_descriptions`

## Decimal Handling

All financial amounts use `rust_decimal::Decimal`:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

let amount = dec!(1234.56);
let tax = amount * dec!(0.077);
```

Decimals are serialized as strings to avoid IEEE 754 floating-point issues.

## See Also

- [Domain Models](../architecture/domain-models.md)
- [datasynth-generators](datasynth-generators.md)
- [datasynth-config](datasynth-config.md)
