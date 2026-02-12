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
source_to_pay:             # Source-to-Pay (v0.6.0)
financial_reporting:       # Financial statements & KPIs (v0.6.0)
hr:                        # HR / payroll / expenses (v0.6.0)
manufacturing:             # Production orders & costing (v0.6.0)
sales_quotes:              # Quote-to-order pipeline (v0.6.0)
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

## Source-to-Pay Configuration (v0.6.0)

```yaml
source_to_pay:
  enabled: false                       # Enable source-to-pay generation

  spend_analysis:
    hhi_threshold: 2500.0              # f64 - HHI threshold for sourcing trigger
    contract_coverage_target: 0.80     # f64, 0-1 - target spend under contracts

  sourcing:
    projects_per_year: 10              # u32 - sourcing projects per year
    renewal_horizon_months: 3          # u32 - months before expiry to trigger renewal
    project_duration_months: 4         # u32 - average project duration

  qualification:
    pass_rate: 0.75                    # f64, 0-1 - qualification pass rate
    validity_days: 365                 # u32 - qualification validity in days
    financial_weight: 0.25             # f64 - financial stability weight
    quality_weight: 0.30               # f64 - quality management weight
    delivery_weight: 0.25              # f64 - delivery performance weight
    compliance_weight: 0.20            # f64 - compliance weight

  rfx:
    rfi_threshold: 100000.0            # f64 - spend above which RFI required
    min_invited_vendors: 3             # u32 - minimum vendors per RFx
    max_invited_vendors: 8             # u32 - maximum vendors per RFx
    response_rate: 0.70                # f64, 0-1 - vendor response rate
    default_price_weight: 0.40         # f64 - price weight in evaluation
    default_quality_weight: 0.35       # f64 - quality weight in evaluation
    default_delivery_weight: 0.25      # f64 - delivery weight in evaluation

  contracts:
    min_duration_months: 12            # u32 - minimum contract duration
    max_duration_months: 36            # u32 - maximum contract duration
    auto_renewal_rate: 0.40            # f64, 0-1 - auto-renewal rate
    amendment_rate: 0.20               # f64, 0-1 - contracts with amendments
    type_distribution:
      fixed_price: 0.40               # f64 - fixed price contracts
      blanket: 0.30                    # f64 - blanket/framework agreements
      time_and_materials: 0.15         # f64 - T&M contracts
      service_agreement: 0.15          # f64 - service agreements

  catalog:
    preferred_vendor_flag_rate: 0.70   # f64, 0-1 - items marked as preferred
    multi_source_rate: 0.25            # f64, 0-1 - items with multiple sources

  scorecards:
    frequency: "quarterly"             # string - review frequency
    on_time_delivery_weight: 0.30      # f64 - OTD weight in score
    quality_weight: 0.30               # f64 - quality weight in score
    price_weight: 0.25                 # f64 - price competitiveness weight
    responsiveness_weight: 0.15        # f64 - responsiveness weight
    grade_a_threshold: 90.0            # f64 - grade A threshold
    grade_b_threshold: 75.0            # f64 - grade B threshold
    grade_c_threshold: 60.0            # f64 - grade C threshold

  p2p_integration:
    off_contract_rate: 0.15            # f64, 0-1 - maverick purchase rate
    price_tolerance: 0.02              # f64 - contract price variance allowed
    catalog_enforcement: false          # bool - enforce catalog ordering
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable source-to-pay generation |
| `sourcing.projects_per_year` | u32 | `10` | Sourcing projects per year |
| `qualification.pass_rate` | f64 | `0.75` | Supplier qualification pass rate |
| `rfx.response_rate` | f64 | `0.70` | Fraction of invited vendors that respond |
| `contracts.auto_renewal_rate` | f64 | `0.40` | Auto-renewal rate |
| `scorecards.frequency` | string | `"quarterly"` | Scorecard review frequency |
| `p2p_integration.off_contract_rate` | f64 | `0.15` | Rate of off-contract (maverick) purchases |

## Financial Reporting Configuration (v0.6.0)

```yaml
financial_reporting:
  enabled: false                       # Enable financial reporting generation
  generate_balance_sheet: true         # bool - generate balance sheet
  generate_income_statement: true      # bool - generate income statement
  generate_cash_flow: true             # bool - generate cash flow statement
  generate_changes_in_equity: true     # bool - generate changes in equity
  comparative_periods: 1               # u32 - number of comparative periods

  management_kpis:
    enabled: false                     # bool - enable KPI generation
    frequency: "monthly"               # string - monthly, quarterly

  budgets:
    enabled: false                     # bool - enable budget generation
    revenue_growth_rate: 0.05          # f64 - expected revenue growth rate
    expense_inflation_rate: 0.03       # f64 - expected expense inflation rate
    variance_noise: 0.10               # f64 - noise for budget vs actual
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable financial reporting generation |
| `generate_balance_sheet` | bool | `true` | Generate balance sheet output |
| `generate_income_statement` | bool | `true` | Generate income statement output |
| `generate_cash_flow` | bool | `true` | Generate cash flow statement output |
| `generate_changes_in_equity` | bool | `true` | Generate changes in equity statement |
| `comparative_periods` | u32 | `1` | Number of comparative periods to include |
| `management_kpis.enabled` | bool | `false` | Enable management KPI calculation |
| `management_kpis.frequency` | string | `"monthly"` | KPI calculation frequency |
| `budgets.enabled` | bool | `false` | Enable budget generation |
| `budgets.revenue_growth_rate` | f64 | `0.05` | Expected revenue growth rate for budgeting |
| `budgets.expense_inflation_rate` | f64 | `0.03` | Expected expense inflation rate |
| `budgets.variance_noise` | f64 | `0.10` | Random noise added to budget vs actual |

## HR Configuration (v0.6.0)

```yaml
hr:
  enabled: false                       # Enable HR generation

  payroll:
    enabled: true                      # bool - enable payroll generation
    pay_frequency: "monthly"           # string - monthly, biweekly, weekly
    salary_ranges:
      staff_min: 50000.0               # f64 - staff level minimum salary
      staff_max: 70000.0               # f64 - staff level maximum salary
      manager_min: 80000.0             # f64 - manager level minimum salary
      manager_max: 120000.0            # f64 - manager level maximum salary
      director_min: 120000.0           # f64 - director level minimum salary
      director_max: 180000.0           # f64 - director level maximum salary
      executive_min: 180000.0          # f64 - executive level minimum salary
      executive_max: 350000.0          # f64 - executive level maximum salary
    tax_rates:
      federal_effective: 0.22          # f64 - federal effective tax rate
      state_effective: 0.05            # f64 - state effective tax rate
      fica: 0.0765                     # f64 - FICA/social security rate
    benefits_enrollment_rate: 0.60     # f64, 0-1 - benefits enrollment rate
    retirement_participation_rate: 0.45 # f64, 0-1 - retirement plan participation

  time_attendance:
    enabled: true                      # bool - enable time tracking
    overtime_rate: 0.10                # f64, 0-1 - employees with overtime

  expenses:
    enabled: true                      # bool - enable expense report generation
    submission_rate: 0.30              # f64, 0-1 - employees submitting per month
    policy_violation_rate: 0.08        # f64, 0-1 - rate of policy violations
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable HR generation |
| `payroll.enabled` | bool | `true` | Enable payroll generation |
| `payroll.pay_frequency` | string | `"monthly"` | Pay frequency: `monthly`, `biweekly`, `weekly` |
| `payroll.benefits_enrollment_rate` | f64 | `0.60` | Benefits enrollment rate |
| `payroll.retirement_participation_rate` | f64 | `0.45` | Retirement plan participation rate |
| `time_attendance.enabled` | bool | `true` | Enable time tracking |
| `time_attendance.overtime_rate` | f64 | `0.10` | Rate of employees with overtime |
| `expenses.enabled` | bool | `true` | Enable expense report generation |
| `expenses.submission_rate` | f64 | `0.30` | Rate of employees submitting expenses per month |
| `expenses.policy_violation_rate` | f64 | `0.08` | Rate of policy violations |

## Manufacturing Configuration (v0.6.0)

```yaml
manufacturing:
  enabled: false                       # Enable manufacturing generation

  production_orders:
    orders_per_month: 50               # u32 - production orders per month
    avg_batch_size: 100                # u32 - average batch size
    yield_rate: 0.97                   # f64, 0-1 - production yield rate
    make_to_order_rate: 0.20           # f64, 0-1 - MTO vs MTS ratio
    rework_rate: 0.03                  # f64, 0-1 - rework rate

  costing:
    labor_rate_per_hour: 35.0          # f64 - labor rate per hour
    overhead_rate: 1.50                # f64 - overhead multiplier on direct labor
    standard_cost_update_frequency: "quarterly"  # string - cost update cycle

  routing:
    avg_operations: 4                  # u32 - average operations per routing
    setup_time_hours: 1.5              # f64 - average setup time in hours
    run_time_variation: 0.15           # f64 - run time variation coefficient
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable manufacturing generation |
| `production_orders.orders_per_month` | u32 | `50` | Number of production orders per month |
| `production_orders.avg_batch_size` | u32 | `100` | Average batch size |
| `production_orders.yield_rate` | f64 | `0.97` | Production yield rate |
| `production_orders.rework_rate` | f64 | `0.03` | Rework rate |
| `costing.labor_rate_per_hour` | f64 | `35.0` | Direct labor cost per hour |
| `costing.overhead_rate` | f64 | `1.50` | Overhead application multiplier |
| `routing.avg_operations` | u32 | `4` | Average operations per routing step |
| `routing.setup_time_hours` | f64 | `1.5` | Average machine setup time in hours |

## Sales Quotes Configuration (v0.6.0)

```yaml
sales_quotes:
  enabled: false                       # Enable sales quote generation
  quotes_per_month: 30                 # u32 - quotes generated per month
  win_rate: 0.35                       # f64, 0-1 - quote-to-order conversion
  validity_days: 30                    # u32 - default quote validity period
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable sales quote generation |
| `quotes_per_month` | u32 | `30` | Number of quotes generated per month |
| `win_rate` | f64 | `0.35` | Fraction of quotes that convert to sales orders |
| `validity_days` | u32 | `30` | Default quote validity period in days |

## See Also

- [Configuration Overview](README.md)
- [Industry Presets](industry-presets.md)
- [datasynth-config Crate](../crates/datasynth-config.md)
