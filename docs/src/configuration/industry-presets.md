# Industry Presets

DataSynth includes pre-configured settings for common industries.

## Using Presets

```bash
# Create configuration from preset
datasynth-data init --industry manufacturing --complexity medium -o config.yaml
```

## Available Industries

| Industry | Key Characteristics |
|----------|---------------------|
| [Manufacturing](#manufacturing) | Heavy P2P, inventory, fixed assets |
| [Retail](#retail) | High O2C volume, seasonal patterns |
| [Financial Services](#financial-services) | Complex intercompany, high controls |
| [Healthcare](#healthcare) | Regulatory focus, insurance seasonality |
| [Technology](#technology) | SaaS revenue, R&D capitalization |

## Complexity Levels

| Level | Accounts | Vendors | Customers | Materials |
|-------|----------|---------|-----------|-----------|
| Small | ~100 | 50 | 100 | 200 |
| Medium | ~400 | 200 | 500 | 1000 |
| Large | ~2500 | 1000 | 5000 | 10000 |

---

## Manufacturing

**Characteristics:**
- High P2P activity (procurement, production)
- Significant inventory and WIP
- Fixed asset intensive
- Cost accounting emphasis

**Key Settings:**

```yaml
global:
  industry: manufacturing

transactions:
  sources:
    manual: 0.2
    automated: 0.6
    recurring: 0.15
    adjustment: 0.05

document_flows:
  p2p:
    enabled: true
    flow_rate: 0.4          # 40% of JEs from P2P
  o2c:
    enabled: true
    flow_rate: 0.25         # 25% of JEs from O2C

master_data:
  materials:
    count: 1000
  fixed_assets:
    count: 200

subledger:
  inventory:
    enabled: true
    valuation_methods:
      - weighted_average
      - fifo
```

**Typical Account Distribution:**
- 45% expense accounts (production costs)
- 25% asset accounts (inventory, equipment)
- 15% liability accounts
- 10% revenue accounts
- 5% equity accounts

---

## Retail

**Characteristics:**
- High transaction volume
- Strong seasonal patterns
- High O2C activity
- Inventory turnover focus

**Key Settings:**

```yaml
global:
  industry: retail

transactions:
  target_count: 500000      # High volume
  temporal:
    month_end_spike: 1.5
    quarter_end_spike: 2.0
    year_end_spike: 5.0     # Holiday season

document_flows:
  p2p:
    enabled: true
    flow_rate: 0.25
  o2c:
    enabled: true
    flow_rate: 0.45         # High sales activity

master_data:
  customers:
    count: 2000
  materials:
    count: 5000

subledger:
  ar:
    enabled: true
    aging_buckets: [30, 60, 90, 120]
```

**Seasonal Pattern:**
- Q4 volume: 200-300% of Q1-Q3 average
- Black Friday/holiday spikes
- Post-holiday returns

---

## Financial Services

**Characteristics:**
- Complex intercompany structures
- High regulatory requirements
- Sophisticated controls
- Mark-to-market adjustments

**Key Settings:**

```yaml
global:
  industry: financial_services

transactions:
  sources:
    automated: 0.7          # High automation
    adjustment: 0.15        # MTM adjustments

intercompany:
  enabled: true
  transaction_types:
    loan: 0.3
    service_provided: 0.25
    dividend: 0.2
    management_fee: 0.15
    royalty: 0.1

internal_controls:
  enabled: true
  controls:
    - id: "SOX-001"
      type: preventive
      frequency: continuous

fx:
  enabled: true
  currency_pairs:
    - EUR
    - GBP
    - CHF
    - JPY
    - CNY
  volatility: 0.015
```

**Control Requirements:**
- SOX 404 compliance mandatory
- High SoD enforcement
- Continuous monitoring

---

## Healthcare

**Characteristics:**
- Complex revenue recognition (insurance)
- Regulatory compliance (HIPAA)
- Seasonal patterns (flu season, open enrollment)
- High accounts receivable

**Key Settings:**

```yaml
global:
  industry: healthcare

transactions:
  amounts:
    min: 50
    max: 500000
    distribution: log_normal

document_flows:
  o2c:
    enabled: true
    flow_rate: 0.5          # Revenue cycle focus

master_data:
  customers:
    count: 1000             # Patient/payer mix

subledger:
  ar:
    enabled: true
    aging_buckets: [30, 60, 90, 120, 180]  # Extended aging

fraud:
  types:
    fictitious_transaction: 0.2
    revenue_manipulation: 0.3   # Upcoding focus
    duplicate_payment: 0.2
```

**Seasonal Pattern:**
- Q1 spike (insurance deductible reset)
- Flu season (Oct-Feb)
- Open enrollment (Nov-Dec)

---

## Technology

**Characteristics:**
- SaaS/subscription revenue
- R&D capitalization
- Stock compensation
- Deferred revenue management

**Key Settings:**

```yaml
global:
  industry: technology

transactions:
  sources:
    automated: 0.65
    recurring: 0.25         # Subscription billing
    manual: 0.08
    adjustment: 0.02

document_flows:
  o2c:
    enabled: true
    flow_rate: 0.35

subledger:
  ar:
    enabled: true

# Additional technology-specific
deferred_revenue:
  enabled: true
  recognition_period: monthly

capitalization:
  r_and_d:
    enabled: true
    threshold: 50000
```

**Revenue Pattern:**
- Monthly recurring revenue (MRR)
- Annual contract billing (ACV)
- Usage-based components

---

## Process Chain Defaults (v0.6.0)

Starting in v0.6.0, all five industry presets include default settings for the new enterprise process chains. When you generate a configuration with `datasynth-data init`, the preset populates sensible defaults for each new section, though they remain disabled until explicitly turned on.

| Process Chain | Manufacturing | Retail | Financial Services | Healthcare | Technology |
|---------------|:---:|:---:|:---:|:---:|:---:|
| `source_to_pay` | High | Medium | Low | Medium | Low |
| `financial_reporting` | Full | Full | Full | Full | Full |
| `hr` | Full | Full | Full | Full | Full |
| `manufacturing` | High | -- | -- | -- | -- |
| `sales_quotes` | Medium | High | Low | Medium | High |

**Manufacturing** presets emphasize production orders, routing, and costing. **Retail** presets increase sales quote volume and quote-to-order win rates. **Financial Services** presets focus on financial reporting with comprehensive KPIs and budgets. **Healthcare** and **Technology** presets provide balanced defaults.

Each preset configures the following when you set `enabled: true`:

- **source_to_pay**: Sourcing projects, RFx events, contract management, catalogs, and vendor scorecards that feed into the existing P2P document flow.
- **financial_reporting**: Balance sheets, income statements, cash flow statements, management KPIs, and budget vs. actual variance analysis.
- **hr**: Payroll runs based on employee master data, time and attendance tracking, and expense report generation with policy violation injection.
- **manufacturing**: Production orders, WIP tracking, standard costing with labor and overhead, and routing operations.
- **sales_quotes**: Quote-to-order pipeline that feeds into the existing O2C document flow.

---

## Customizing Presets

Start with a preset and customize:

```bash
# Generate preset
datasynth-data init --industry manufacturing -o config.yaml

# Edit config.yaml
# - Adjust transaction counts
# - Add companies
# - Enable additional features

# Validate and generate
datasynth-data validate --config config.yaml
datasynth-data generate --config config.yaml --output ./output
```

## Combining Industries

For conglomerates, use multiple companies with different characteristics:

```yaml
companies:
  - code: "1000"
    name: "Manufacturing Division"
    volume_weight: 0.5

  - code: "2000"
    name: "Retail Division"
    volume_weight: 0.3

  - code: "3000"
    name: "Services Division"
    volume_weight: 0.2
```

## See Also

- [Configuration Overview](README.md)
- [Global Settings](global-settings.md)
- [Companies](companies.md)
