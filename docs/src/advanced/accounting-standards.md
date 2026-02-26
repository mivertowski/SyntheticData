# Accounting & Audit Standards

SyntheticData includes comprehensive support for major accounting and auditing standards frameworks, enabling the generation of standards-compliant synthetic financial data suitable for audit analytics, compliance testing, and ML model training.

## Overview

The `datasynth-standards` crate provides domain models and generation logic for:

| Category | Standards |
|----------|-----------|
| **Accounting** | US GAAP (ASC), IFRS, French GAAP (PCG), German GAAP (HGB/SKR04) |
| **Auditing** | ISA (International Standards on Auditing), PCAOB |
| **Regulatory** | SOX (Sarbanes-Oxley Act) |

## Accounting Framework Selection

### Framework Options

```yaml
accounting_standards:
  enabled: true
  framework: us_gaap  # Options: us_gaap, ifrs, french_gaap, german_gaap, dual_reporting
```

| Framework | Description |
|-----------|-------------|
| `us_gaap` | United States Generally Accepted Accounting Principles |
| `ifrs` | International Financial Reporting Standards |
| `french_gaap` | French Generally Accepted Accounting Principles (Plan Comptable Général) |
| `german_gaap` | German Generally Accepted Accounting Principles (HGB / Handelsgesetzbuch, SKR04) |
| `dual_reporting` | Generate data for both frameworks with reconciliation |

### Generalized Framework Architecture

A central `FrameworkAccounts` struct maps ~45 semantic account purposes (AR control, AP control, inventory, depreciation expense, etc.) to framework-specific GL codes. Each framework has its own classifier for mapping account numbers to `AccountCategory`.

The `AuditExportConfig` controls framework-specific audit exports:
- **FEC** (French): Article A47 A-1 compliant 18-column export
- **GoBD** (German): 13-column journal export with XML index

### Key Framework Differences

The generator automatically handles framework-specific rules:

| Area | US GAAP | IFRS | French GAAP (PCG) | German GAAP (HGB) |
|------|---------|------|-------------------|-------------------|
| Inventory costing | LIFO permitted | LIFO prohibited | LIFO prohibited | LIFO prohibited |
| Development costs | Generally expensed | Capitalized when criteria met | Capitalized when criteria met | Optional (§248(2)) |
| PPE revaluation | Cost model only | Revaluation model permitted | Revaluation permitted | Not permitted |
| Impairment reversal | Not permitted | Permitted (except goodwill) | Permitted | Mandatory (§253(5)) |
| Lease classification | Bright-line tests (75%/90%) | Principles-based | Principles-based (IFRS 16 aligned) | BMF-Leasingerlasse (40-90%), operating off-balance |
| Low-value assets | No special rule | No special rule | No special rule | GWG: ≤800 EUR immediate expense (§6(2)) |
| Depreciation methods | Straight-line, declining | Straight-line, declining | Straight-line, declining | Straight-line, Degressiv (3x SL, max 30%) |
| Goodwill | Impairment-only | Impairment-only | Impairment-only | Amortized (max 10yr, §253(3)) |
| Pending loss provisions | Not required | Not required | Not required | Required |

## Revenue Recognition (ASC 606 / IFRS 15)

Generate realistic customer contracts with performance obligations:

```yaml
accounting_standards:
  revenue_recognition:
    enabled: true
    generate_contracts: true
    avg_obligations_per_contract: 2.0
    variable_consideration_rate: 0.15
    over_time_recognition_rate: 0.30
    contract_count: 100
```

### Generated Entities

- **Customer Contracts**: Transaction price, status, framework
- **Performance Obligations**: Goods, services, licenses with satisfaction patterns
- **Variable Consideration**: Discounts, rebates, incentives with constraint application
- **Revenue Recognition Schedule**: Period-by-period recognition

### 5-Step Model Compliance

The generator follows the 5-step revenue recognition model:
1. Identify the contract
2. Identify performance obligations
3. Determine transaction price
4. Allocate transaction price to obligations
5. Recognize revenue when/as obligations are satisfied

## Lease Accounting (ASC 842 / IFRS 16)

Generate lease portfolios with ROU assets and lease liabilities:

```yaml
accounting_standards:
  leases:
    enabled: true
    lease_count: 50
    finance_lease_percent: 0.30
    avg_lease_term_months: 60
    generate_amortization: true
    real_estate_percent: 0.40
```

### Generated Entities

- **Leases**: Classification, commencement date, term, payments, discount rate
- **ROU Assets**: Initial measurement, accumulated depreciation, carrying amount
- **Lease Liabilities**: Current/non-current portions
- **Amortization Schedules**: Period-by-period interest and principal

### Classification Logic

- **US GAAP**: Bright-line tests (75% term, 90% PV)
- **IFRS**: All leases (except short-term/low-value) recognized on balance sheet
- **French GAAP**: Delegates to IFRS 16 principles-based classification (ANC règlement 2019-01)
- **German GAAP**: BMF-Leasingerlasse 40-90% economic life test; operating leases remain off-balance sheet

## Fair Value Measurement (ASC 820 / IFRS 13)

Generate fair value measurements across hierarchy levels:

```yaml
accounting_standards:
  fair_value:
    enabled: true
    measurement_count: 30
    level1_percent: 0.60    # Quoted prices
    level2_percent: 0.30    # Observable inputs
    level3_percent: 0.10    # Unobservable inputs
    include_sensitivity_analysis: true
```

### Fair Value Hierarchy

| Level | Description | Examples |
|-------|-------------|----------|
| Level 1 | Quoted prices in active markets | Listed stocks, exchange-traded funds |
| Level 2 | Observable inputs | Corporate bonds, interest rate swaps |
| Level 3 | Unobservable inputs | Private equity, complex derivatives |

## Impairment Testing (ASC 360 / IAS 36)

Generate impairment tests with framework-specific methodology:

```yaml
accounting_standards:
  impairment:
    enabled: true
    test_count: 15
    impairment_rate: 0.20
    generate_projections: true
    include_goodwill: true
```

### Framework Differences

- **US GAAP**: Two-step test (recoverability then measurement)
- **IFRS**: One-step test comparing to recoverable amount
- **French GAAP**: Follows IFRS approach (one-step test); reversal of impairment permitted
- **German GAAP**: One-step test (IFRS-style); impairment reversal mandatory (§253(5)); goodwill reversal permitted

## German GAAP-Specific Features (HGB)

### Chart of Accounts (SKR04)

When `framework: german_gaap` is selected, the generator uses the Standardkontenrahmen 04 (SKR04) with 4-digit accounts across classes 0-9:

| Class | Description | Example Accounts |
|-------|-------------|-----------------|
| 0 | Fixed assets (Anlagevermögen) | 0200 Sachanlagen, 0700 Kumulierte Abschreibungen |
| 1 | Current assets (Umlaufvermögen) | 1200 Forderungen L+L, 1570 Vorsteuer |
| 2 | Equity (Eigenkapital) | 2000 Gezeichnetes Kapital, 2970 Gewinnvortrag |
| 3 | Liabilities (Verbindlichkeiten) | 3300 Verbindlichkeiten L+L, 3800 Umsatzsteuer |
| 4 | Revenue (Erlöse) | 4000 Umsatzerlöse, 4400 Erlöse Leistungen |
| 5 | COGS (Materialaufwand) | 5000 Materialaufwand |
| 6 | Operating expenses | 6000 Löhne, 6220 Abschreibungen, 6310 Miete |
| 7 | Financial items | 7300 Zinsaufwendungen, 7600 Steuern |
| 9 | Statistical accounts | Off-balance items |

### Depreciation Methods

- **Straight-line** (linear): Standard method
- **Degressiv** (declining balance): min(3x straight-line rate, 30%) on net book value, with automatic switch to straight-line when SL exceeds Degressiv (EStG §7(2) / Wachstumschancengesetz)
- **GWG** (geringwertige Wirtschaftsgüter): Assets ≤ 800 EUR receive immediate full expense (EStG §6(2))

### GoBD Audit Export

When German GAAP is active, a GoBD-compliant export is generated:

```
output/
└── gobd_export/
    ├── gobd_journal.csv     # 13-column semicolon-separated journal
    ├── gobd_accounts.csv    # Account master data
    └── index.xml            # GoBD-compliant XML index
```

The journal CSV columns: Belegdatum, Buchungsdatum, Belegnummer, Buchungstext, Kontonummer, Gegenkontonummer, Sollbetrag, Habenbetrag, Steuerschlüssel, Steuerbetrag, Währung, Kostenstelle, Belegnummernkreis.

### Auxiliary GL Accounts

Both French and German GAAP generate auxiliary GL sub-accounts for vendors and customers:

| Framework | Vendor Format | Customer Format |
|-----------|---------------|-----------------|
| French PCG | `401XXXX` | `411XXXX` |
| German SKR04 | `3300XXXX` | `1200XXXX` |

These auxiliary accounts are used as `auxiliary_account_number` in FEC/GoBD journal exports instead of raw partner IDs.

## ISA Compliance (Audit Standards)

Generate audit procedures mapped to ISA requirements:

```yaml
audit_standards:
  isa_compliance:
    enabled: true
    compliance_level: comprehensive  # basic, standard, comprehensive
    generate_isa_mappings: true
    generate_coverage_summary: true
    include_pcaob: true
    framework: dual  # isa, pcaob, dual
```

### Supported ISA Standards

The crate includes 34 ISA standards from ISA 200 through ISA 720:

| Series | Focus Area |
|--------|------------|
| ISA 200-265 | General principles and responsibilities |
| ISA 300-450 | Risk assessment and response |
| ISA 500-580 | Audit evidence |
| ISA 600-620 | Using work of others |
| ISA 700-720 | Conclusions and reporting |

## Analytical Procedures (ISA 520)

Generate analytical procedures with variance investigation:

```yaml
audit_standards:
  analytical_procedures:
    enabled: true
    procedures_per_account: 3
    variance_probability: 0.20
    generate_investigations: true
    include_ratio_analysis: true
```

### Procedure Types

- **Trend analysis**: Year-over-year comparisons
- **Ratio analysis**: Key financial ratios
- **Reasonableness tests**: Expected vs. actual comparisons

## External Confirmations (ISA 505)

Generate confirmation procedures with response tracking:

```yaml
audit_standards:
  confirmations:
    enabled: true
    confirmation_count: 50
    positive_response_rate: 0.85
    exception_rate: 0.10
```

### Confirmation Types

- Bank confirmations
- Accounts receivable confirmations
- Accounts payable confirmations
- Legal confirmations

## Audit Opinion (ISA 700/705/706/701)

Generate audit opinions with key audit matters:

```yaml
audit_standards:
  opinion:
    enabled: true
    generate_kam: true
    average_kam_count: 3
```

### Opinion Types

- Unmodified
- Qualified
- Adverse
- Disclaimer

## SOX Compliance

Generate SOX 302/404 compliance documentation:

```yaml
audit_standards:
  sox:
    enabled: true
    generate_302_certifications: true
    generate_404_assessments: true
    materiality_threshold: 10000.0
```

### Section 302 Certifications

- CEO and CFO certifications
- Disclosure controls effectiveness
- Material weakness identification

### Section 404 Assessments

- ICFR effectiveness assessment
- Key control testing
- Deficiency classification matrix

### Deficiency Classification

The `DeficiencyMatrix` classifies deficiencies based on:

| Likelihood | Magnitude | Classification |
|------------|-----------|----------------|
| Probable | Material | Material Weakness |
| Reasonably Possible | More Than Inconsequential | Significant Deficiency |
| Remote | Inconsequential | Control Deficiency |

## PCAOB Standards

Generate PCAOB-specific audit elements:

```yaml
audit_standards:
  pcaob:
    enabled: true
    generate_cam: true
    integrated_audit: true
```

### PCAOB-Specific Requirements

- Critical Audit Matters (CAMs) vs. Key Audit Matters (KAMs)
- Integrated audit (ICFR + financial statements)
- AS 2201 ICFR testing requirements

## Evaluation and Validation

The `datasynth-eval` crate includes standards compliance evaluators:

```rust
use datasynth_eval::coherence::{
    StandardsComplianceEvaluation,
    RevenueRecognitionEvaluator,
    LeaseAccountingEvaluator,
    StandardsThresholds,
};

// Evaluate revenue recognition compliance
let eval = RevenueRecognitionEvaluator::evaluate(&contracts);
assert!(eval.po_allocation_compliance >= 0.95);

// Evaluate lease classification accuracy
let eval = LeaseAccountingEvaluator::evaluate(&leases, "us_gaap");
assert!(eval.classification_accuracy >= 0.90);
```

### Compliance Thresholds

| Metric | Default Threshold |
|--------|-------------------|
| PO allocation compliance | 95% |
| Revenue timing compliance | 95% |
| Lease classification accuracy | 90% |
| ROU asset accuracy | 95% |
| Fair value hierarchy compliance | 95% |
| ISA coverage | 90% |
| SOX control coverage | 95% |
| Audit trail completeness | 90% |

## Output Files

When standards generation is enabled, additional files are exported:

```
output/
├── standards/
│   ├── accounting/
│   │   ├── customer_contracts.csv
│   │   ├── performance_obligations.csv
│   │   ├── variable_consideration.csv
│   │   ├── revenue_recognition_schedule.csv
│   │   ├── leases.csv
│   │   ├── rou_assets.csv
│   │   ├── lease_liabilities.csv
│   │   ├── lease_amortization.csv
│   │   ├── fair_value_measurements.csv
│   │   ├── impairment_tests.csv
│   │   └── framework_differences.csv
│   ├── audit/
│   │   ├── isa_requirement_mappings.csv
│   │   ├── isa_coverage_summary.csv
│   │   ├── analytical_procedures.csv
│   │   ├── variance_investigations.csv
│   │   ├── confirmations.csv
│   │   ├── confirmation_responses.csv
│   │   ├── audit_opinions.csv
│   │   ├── key_audit_matters.csv
│   │   ├── audit_trails.json
│   │   └── pcaob_mappings.csv
│   └── regulatory/
│       ├── sox_302_certifications.csv
│       ├── sox_404_assessments.csv
│       ├── deficiency_classifications.csv
│       └── material_weaknesses.csv
├── fec.csv                          # French GAAP only (18 columns, Art. A47 A-1)
└── gobd_export/                     # German GAAP only
    ├── gobd_journal.csv             # 13-column GoBD journal
    ├── gobd_accounts.csv            # Account master data
    └── index.xml                    # GoBD XML index
```

## Use Cases

### Audit Analytics Training

Generate labeled data for training audit analytics models with known standards compliance levels.

### Compliance Testing

Test compliance monitoring systems with synthetic data covering all major accounting and auditing standards.

### IFRS to US GAAP Reconciliation

Use dual reporting mode to generate reconciliation data for multi-framework analysis.

### SOX Testing

Generate internal control data with known deficiencies for testing SOX monitoring systems.

## See Also

- [COSO Framework](../configuration/compliance.md) - Internal control framework
- [Audit Simulation](../use-cases/audit-analytics.md) - Audit analytics use cases
- [SOX Compliance](../use-cases/sox-compliance.md) - SOX testing use cases
