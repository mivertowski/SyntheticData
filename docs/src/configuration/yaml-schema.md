# YAML Schema Reference

Complete reference for all configuration options.

## Schema Overview

```yaml
global:                    # Global settings
companies:                 # Company definitions
chart_of_accounts:         # COA structure
transactions:              # Transaction settings
master_data:               # Master data settings
document_flows:            # P2P, O2C flows
intercompany:              # IC settings
balance:                   # Balance settings
subledger:                 # Subledger settings
fx:                        # FX settings
period_close:              # Period close settings
fraud:                     # Fraud injection
internal_controls:         # SOX controls
anomaly_injection:         # Anomaly injection
data_quality:              # Data quality variations
graph_export:              # Graph export settings
output:                    # Output settings
business_processes:        # Process distribution
templates:                 # External templates
approval:                  # Approval thresholds
departments:               # Department distribution
```

## global

```yaml
global:
  seed: 42                           # u64, optional - RNG seed
  industry: manufacturing            # string - industry preset
  start_date: 2024-01-01             # date - generation start
  period_months: 12                  # u32, 1-120 - duration
  group_currency: USD                # string - base currency
  worker_threads: 4                  # usize, optional - parallelism
  memory_limit: 2147483648           # u64, optional - bytes
```

**Industries:** `manufacturing`, `retail`, `financial_services`, `healthcare`, `technology`, `energy`, `telecom`, `transportation`, `hospitality`

## companies

```yaml
companies:
  - code: "1000"                     # string - unique code
    name: "Headquarters"             # string - display name
    currency: USD                    # string - local currency
    country: US                      # string - ISO country code
    volume_weight: 0.6               # f64, 0-1 - transaction weight
    is_parent: true                  # bool - consolidation parent
    parent_code: null                # string, optional - parent ref
```

**Constraints:**
- `volume_weight` across all companies must sum to 1.0
- `code` must be unique

## chart_of_accounts

```yaml
chart_of_accounts:
  complexity: medium                 # small, medium, large
  industry_specific: true            # bool - use industry COA
  custom_accounts: []                # list - additional accounts
```

**Complexity levels:**
- `small`: ~100 accounts
- `medium`: ~400 accounts
- `large`: ~2500 accounts

## transactions

```yaml
transactions:
  target_count: 100000               # u64 - total JEs to generate

  line_items:
    distribution: empirical          # empirical, uniform, custom
    min_lines: 2                     # u32 - minimum line items
    max_lines: 20                    # u32 - maximum line items
    custom_distribution:             # only if distribution: custom
      2: 0.6068
      3: 0.0524
      4: 0.1732

  amounts:
    min: 100                         # f64 - minimum amount
    max: 1000000                     # f64 - maximum amount
    distribution: log_normal         # log_normal, uniform, custom
    round_number_bias: 0.15          # f64, 0-1 - round number preference

  sources:                           # transaction source weights
    manual: 0.3
    automated: 0.5
    recurring: 0.15
    adjustment: 0.05

  benford:
    enabled: true                    # bool - Benford's Law compliance

  temporal:
    month_end_spike: 2.5             # f64 - month-end volume multiplier
    quarter_end_spike: 3.0           # f64 - quarter-end multiplier
    year_end_spike: 4.0              # f64 - year-end multiplier
    working_hours_only: true         # bool - restrict to business hours
```

## master_data

```yaml
master_data:
  vendors:
    count: 200                       # u32 - number of vendors
    intercompany_ratio: 0.05         # f64, 0-1 - IC vendor ratio

  customers:
    count: 500                       # u32 - number of customers
    intercompany_ratio: 0.05         # f64, 0-1 - IC customer ratio

  materials:
    count: 1000                      # u32 - number of materials

  fixed_assets:
    count: 100                       # u32 - number of assets

  employees:
    count: 50                        # u32 - number of employees
    hierarchy_depth: 4               # u32 - org chart depth
```

## document_flows

```yaml
document_flows:
  p2p:                               # Procure-to-Pay
    enabled: true
    flow_rate: 0.3                   # f64, 0-1 - JE percentage
    completion_rate: 0.95            # f64, 0-1 - full flow rate
    three_way_match:
      quantity_tolerance: 0.02       # f64, 0-1 - qty variance allowed
      price_tolerance: 0.01          # f64, 0-1 - price variance allowed

  o2c:                               # Order-to-Cash
    enabled: true
    flow_rate: 0.3                   # f64, 0-1 - JE percentage
    completion_rate: 0.95            # f64, 0-1 - full flow rate
```

## intercompany

```yaml
intercompany:
  enabled: true
  transaction_types:                 # weights must sum to 1.0
    goods_sale: 0.4
    service_provided: 0.2
    loan: 0.15
    dividend: 0.1
    management_fee: 0.1
    royalty: 0.05

  transfer_pricing:
    method: cost_plus                # cost_plus, resale_minus, comparable
    markup_range:
      min: 0.03
      max: 0.10
```

## balance

```yaml
balance:
  opening_balance:
    enabled: true
    total_assets: 10000000           # f64 - opening balance sheet size

  coherence_check:
    enabled: true                    # bool - verify A = L + E
    tolerance: 0.01                  # f64 - allowed imbalance
```

## subledger

```yaml
subledger:
  ar:
    enabled: true
    aging_buckets: [30, 60, 90]      # list of days

  ap:
    enabled: true
    aging_buckets: [30, 60, 90]

  fixed_assets:
    enabled: true
    depreciation_methods:
      - straight_line
      - declining_balance

  inventory:
    enabled: true
    valuation_methods:
      - fifo
      - weighted_average
```

## fx

```yaml
fx:
  enabled: true
  base_currency: USD

  currency_pairs:                    # currencies to generate
    - EUR
    - GBP
    - CHF
    - JPY

  volatility: 0.01                   # f64 - daily volatility

  translation:
    method: current_rate             # current_rate, temporal
```

## period_close

```yaml
period_close:
  enabled: true

  monthly:
    accruals: true
    depreciation: true

  quarterly:
    intercompany_elimination: true

  annual:
    closing_entries: true
    retained_earnings: true
```

## fraud

```yaml
fraud:
  enabled: true
  fraud_rate: 0.005                  # f64, 0-1 - fraud percentage

  types:                             # weights must sum to 1.0
    fictitious_transaction: 0.15
    revenue_manipulation: 0.10
    expense_capitalization: 0.10
    split_transaction: 0.15
    round_tripping: 0.05
    kickback_scheme: 0.10
    ghost_employee: 0.05
    duplicate_payment: 0.15
    unauthorized_discount: 0.10
    suspense_abuse: 0.05
```

## internal_controls

```yaml
internal_controls:
  enabled: true

  controls:
    - id: "CTL-001"
      name: "Payment Approval"
      type: preventive
      frequency: continuous

  sod_rules:
    - conflict_type: create_approve
      processes: [ap_invoice, ap_payment]
```

## anomaly_injection

```yaml
anomaly_injection:
  enabled: true
  total_rate: 0.02                   # f64, 0-1 - total anomaly rate
  generate_labels: true              # bool - output ML labels

  categories:                        # weights must sum to 1.0
    fraud: 0.25
    error: 0.40
    process_issue: 0.20
    statistical: 0.10
    relational: 0.05

  temporal_pattern:
    year_end_spike: 1.5              # f64 - year-end multiplier

  clustering:
    enabled: true
    cluster_probability: 0.2
```

## data_quality

```yaml
data_quality:
  enabled: true

  missing_values:
    rate: 0.01                       # f64, 0-1
    pattern: mcar                    # mcar, mar, mnar, systematic

  format_variations:
    date_formats: true
    amount_formats: true

  duplicates:
    rate: 0.001                      # f64, 0-1
    types: [exact, near, fuzzy]

  typos:
    rate: 0.005                      # f64, 0-1
    keyboard_aware: true
```

## graph_export

```yaml
graph_export:
  enabled: true

  formats:
    - pytorch_geometric
    - neo4j
    - dgl

  graphs:
    - transaction_network
    - approval_network
    - entity_relationship

  split:
    train: 0.7
    val: 0.15
    test: 0.15
    stratify: is_anomaly

  features:
    temporal: true
    amount: true
    structural: true
    categorical: true
```

## output

```yaml
output:
  format: csv                        # csv, json
  compression: none                  # none, gzip, zstd
  compression_level: 6               # u32, 1-9 (if compression enabled)

  files:
    journal_entries: true
    acdoca: true
    master_data: true
    documents: true
    subledgers: true
    trial_balances: true
    labels: true
    controls: true
```

## Validation Summary

| Field | Constraint |
|-------|------------|
| `period_months` | 1-120 |
| `compression_level` | 1-9 |
| All rates/percentages | 0.0-1.0 |
| Distributions | Sum to 1.0 (±0.01) |
| Company codes | Unique |
| Dates | Valid and consistent |

## Diffusion Configuration (v0.5.0)

```yaml
diffusion:
  enabled: false                    # Enable diffusion model backend
  n_steps: 1000                     # Number of diffusion steps (default: 1000)
  schedule: "linear"                # Noise schedule: "linear", "cosine", "sigmoid"
  sample_size: 1000                 # Number of samples to generate (default: 1000)
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable diffusion model generation |
| `n_steps` | integer | `1000` | Number of forward/reverse diffusion steps |
| `schedule` | string | `"linear"` | Noise schedule type: `linear`, `cosine`, `sigmoid` |
| `sample_size` | integer | `1000` | Number of samples to generate |

## Causal Configuration (v0.5.0)

```yaml
causal:
  enabled: false                    # Enable causal generation
  template: "fraud_detection"       # Built-in template or custom graph path
  sample_size: 1000                 # Number of samples to generate
  validate: true                    # Validate causal structure in output
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable causal/counterfactual generation |
| `template` | string | `"fraud_detection"` | Template name (`fraud_detection`, `revenue_cycle`) or path to custom YAML |
| `sample_size` | integer | `1000` | Number of causal samples to generate |
| `validate` | bool | `true` | Run causal structure validation on output |

### Built-in Causal Templates

| Template | Variables | Description |
|----------|-----------|-------------|
| `fraud_detection` | transaction_amount, approval_level, vendor_risk, fraud_flag | Fraud detection causal graph |
| `revenue_cycle` | order_size, credit_score, payment_delay, revenue | Revenue cycle causal graph |

## Certificate Configuration (v0.5.0)

```yaml
certificates:
  enabled: false                    # Enable synthetic data certificates
  issuer: "DataSynth"              # Certificate issuer name
  include_quality_metrics: true     # Include quality metrics in certificate
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Attach certificate to generated output |
| `issuer` | string | `"DataSynth"` | Issuer identity for the certificate |
| `include_quality_metrics` | bool | `true` | Include Benford MAD, correlation, fidelity metrics |

## See Also

- [Configuration Overview](README.md)
- [Industry Presets](industry-presets.md)
- [datasynth-config Crate](../crates/datasynth-config.md)
