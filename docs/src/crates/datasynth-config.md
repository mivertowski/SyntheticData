# datasynth-config

Configuration schema, validation, and industry presets for synthetic data generation.

## Overview

`datasynth-config` provides the configuration layer for DataSynth:

- **Schema Definition**: Complete YAML configuration schema
- **Validation**: Bounds checking, constraint validation, distribution sum verification
- **Industry Presets**: Pre-configured settings for common industries
- **Complexity Levels**: Small, medium, and large organization profiles

## Configuration Sections

| Section | Description |
|---------|-------------|
| `global` | Industry, dates, seed, performance settings |
| `companies` | Company codes, currencies, volume weights |
| `chart_of_accounts` | COA complexity and structure |
| `transactions` | Line items, amounts, sources, temporal patterns |
| `master_data` | Vendors, customers, materials, assets, employees |
| `document_flows` | P2P, O2C configuration |
| `intercompany` | IC transaction types and transfer pricing |
| `balance` | Opening balances, trial balance generation |
| `subledger` | AR, AP, FA, inventory settings |
| `fx` | Currency and exchange rate settings |
| `period_close` | Close tasks and schedules |
| `fraud` | Fraud injection rates and types |
| `internal_controls` | SOX controls and SoD rules |
| `anomaly_injection` | Anomaly rates and labeling |
| `data_quality` | Missing values, typos, duplicates |
| `graph_export` | ML graph export formats |
| `output` | Output format and compression |

## Industry Presets

| Industry | Description |
|----------|-------------|
| `manufacturing` | Heavy P2P, inventory, fixed assets |
| `retail` | High O2C volume, seasonal patterns |
| `financial_services` | Complex intercompany, high controls |
| `healthcare` | Regulatory focus, seasonal insurance |
| `technology` | SaaS revenue patterns, R&D capitalization |

## Key Types

### Config

```rust
pub struct Config {
    pub global: GlobalConfig,
    pub companies: Vec<CompanyConfig>,
    pub chart_of_accounts: CoaConfig,
    pub transactions: TransactionConfig,
    pub master_data: MasterDataConfig,
    pub document_flows: DocumentFlowConfig,
    pub intercompany: IntercompanyConfig,
    pub balance: BalanceConfig,
    pub subledger: SubledgerConfig,
    pub fx: FxConfig,
    pub period_close: PeriodCloseConfig,
    pub fraud: FraudConfig,
    pub internal_controls: ControlConfig,
    pub anomaly_injection: AnomalyConfig,
    pub data_quality: DataQualityConfig,
    pub graph_export: GraphExportConfig,
    pub output: OutputConfig,
}
```

### GlobalConfig

```rust
pub struct GlobalConfig {
    pub seed: Option<u64>,
    pub industry: Industry,
    pub start_date: NaiveDate,
    pub period_months: u32,      // 1-120
    pub group_currency: String,
    pub worker_threads: Option<usize>,
    pub memory_limit: Option<u64>,
}
```

### CompanyConfig

```rust
pub struct CompanyConfig {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub country: String,
    pub volume_weight: f64,     // Must sum to 1.0 across companies
    pub is_parent: bool,
    pub parent_code: Option<String>,
}
```

## Validation Rules

The `ConfigValidator` enforces:

| Rule | Constraint |
|------|------------|
| `period_months` | 1-120 (max 10 years) |
| `compression_level` | 1-9 when compression enabled |
| Rate fields | 0.0-1.0 |
| Approval thresholds | Strictly ascending order |
| Distribution weights | Sum to 1.0 (±0.01 tolerance) |
| Company codes | Unique within configuration |
| Dates | `start_date` + `period_months` is valid |

## Usage Examples

### Loading Configuration

```rust
use synth_config::{Config, ConfigValidator};

// From YAML file
let config = Config::from_yaml_file("config.yaml")?;

// Validate
let validator = ConfigValidator::new();
validator.validate(&config)?;
```

### Using Presets

```rust
use synth_config::{Config, Industry, Complexity};

// Create from preset
let config = Config::from_preset(Industry::Manufacturing, Complexity::Medium);

// Modify as needed
config.transactions.target_count = 50000;
```

### Creating Configuration Programmatically

```rust
use synth_config::{Config, GlobalConfig, TransactionConfig};

let config = Config {
    global: GlobalConfig {
        seed: Some(42),
        industry: Industry::Manufacturing,
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        period_months: 12,
        group_currency: "USD".to_string(),
        ..Default::default()
    },
    transactions: TransactionConfig {
        target_count: 100000,
        ..Default::default()
    },
    ..Default::default()
};
```

### Saving Configuration

```rust
// To YAML
config.to_yaml_file("config.yaml")?;

// To JSON
config.to_json_file("config.json")?;

// To string
let yaml = config.to_yaml_string()?;
```

## Configuration Examples

### Minimal Configuration

```yaml
global:
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 12

transactions:
  target_count: 10000

output:
  format: csv
```

### Full Configuration

See the [YAML Schema Reference](../configuration/yaml-schema.md) for complete documentation.

## Complexity Levels

| Level | Accounts | Vendors | Customers | Materials |
|-------|----------|---------|-----------|-----------|
| `small` | ~100 | 50 | 100 | 200 |
| `medium` | ~400 | 200 | 500 | 1000 |
| `large` | ~2500 | 1000 | 5000 | 10000 |

## Validation Error Types

```rust
pub enum ConfigError {
    MissingRequiredField(String),
    InvalidValue { field: String, value: String, constraint: String },
    DistributionSumError { field: String, sum: f64 },
    DuplicateCode { field: String, code: String },
    DateRangeError { start: NaiveDate, end: NaiveDate },
    ParseError(String),
}
```

## See Also

- [Configuration Overview](../configuration/README.md)
- [YAML Schema Reference](../configuration/yaml-schema.md)
- [Industry Presets](../configuration/industry-presets.md)
