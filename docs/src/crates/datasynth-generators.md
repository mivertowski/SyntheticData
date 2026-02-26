# datasynth-generators

Data generators for journal entries, master data, document flows, and anomalies.

## Overview

`datasynth-generators` contains all data generation logic for SyntheticData:

- **Core Generators**: Journal entries, chart of accounts, users
- **Master Data**: Vendors, customers, materials, assets, employees
- **Document Flows**: P2P (Procure-to-Pay), O2C (Order-to-Cash)
- **Financial**: Intercompany, balance tracking, subledgers, FX, period close
- **Quality**: Anomaly injection, data quality variations
- **Sourcing (S2C)**: Spend analysis, RFx, bids, contracts, catalogs, scorecards (v0.6.0)
- **HR / Payroll**: Payroll runs, time entries, expense reports (v0.6.0)
- **Financial Reporting**: Financial statements, bank reconciliation (v0.6.0)
- **Standards**: Revenue recognition, impairment testing (v0.6.0)
- **Manufacturing**: Production orders, quality inspections, cycle counts (v0.6.0)

## Module Structure

### Core Generators

| Generator | Description |
|-----------|-------------|
| `je_generator` | Journal entry generation with statistical distributions |
| `coa_generator` | Chart of accounts with industry-specific structures; `CoAFramework` enum dispatches US GAAP / French PCG / German SKR04 |
| `company_selector` | Weighted company selection for transactions |
| `user_generator` | User/persona generation with roles |
| `control_generator` | Internal controls and SoD rules |

### Master Data (`master_data/`)

| Generator | Description |
|-----------|-------------|
| `vendor_generator` | Vendors with payment terms, bank accounts, behaviors |
| `customer_generator` | Customers with credit ratings, payment patterns |
| `material_generator` | Materials/products with BOM, valuations |
| `asset_generator` | Fixed assets with depreciation schedules; German GWG expensing, Degressiv method, AfA-Tabellen useful lives |
| `employee_generator` | Employees with manager hierarchy |
| `entity_registry_manager` | Central entity registry with temporal validity |

### Document Flow (`document_flow/`)

| Generator | Description |
|-----------|-------------|
| `p2p_generator` | PO → GR → Invoice → Payment flow |
| `o2c_generator` | SO → Delivery → Invoice → Receipt flow |
| `document_chain_manager` | Reference chain management |
| `document_flow_je_generator` | Generate JEs from document flows; framework-aware auxiliary GL account lookup for FEC/GoBD |
| `three_way_match` | PO/GR/Invoice matching validation |

### Intercompany (`intercompany/`)

| Generator | Description |
|-----------|-------------|
| `ic_generator` | Matched intercompany entry pairs |
| `matching_engine` | IC matching and reconciliation |
| `elimination_generator` | Consolidation elimination entries |

### Balance (`balance/`)

| Generator | Description |
|-----------|-------------|
| `opening_balance_generator` | Coherent opening balance sheet |
| `balance_tracker` | Running balance validation |
| `trial_balance_generator` | Period-end trial balance |

### Subledger (`subledger/`)

| Generator | Description |
|-----------|-------------|
| `ar_generator` | AR invoices, receipts, credit memos, aging |
| `ap_generator` | AP invoices, payments, debit memos |
| `fa_generator` | Fixed assets, depreciation, disposals |
| `inventory_generator` | Inventory positions, movements, valuation |
| `reconciliation` | GL-to-subledger reconciliation |

### FX (`fx/`)

| Generator | Description |
|-----------|-------------|
| `fx_rate_service` | FX rate generation (Ornstein-Uhlenbeck process) |
| `currency_translator` | Trial balance translation |
| `cta_generator` | Currency Translation Adjustment entries |

### Period Close (`period_close/`)

| Generator | Description |
|-----------|-------------|
| `close_engine` | Main orchestration |
| `accruals` | Accrual entry generation |
| `depreciation` | Monthly depreciation runs |
| `year_end` | Year-end closing entries |

### Anomaly (`anomaly/`)

| Generator | Description |
|-----------|-------------|
| `injector` | Main anomaly injection engine |
| `types` | Weighted anomaly type configurations |
| `strategies` | Injection strategies (amount, date, duplication) |
| `patterns` | Temporal patterns, clustering, entity targeting |

### Data Quality (`data_quality/`)

| Generator | Description |
|-----------|-------------|
| `injector` | Main data quality injector |
| `missing_values` | MCAR, MAR, MNAR, Systematic patterns |
| `format_variations` | Date, amount, identifier formats |
| `duplicates` | Exact, near, fuzzy duplicates |
| `typos` | Keyboard-aware typos, OCR errors |
| `labels` | ML training labels for data quality issues |

### Audit (`audit/`)

ISA-compliant audit data generation.

| Generator | Description |
|-----------|-------------|
| `engagement_generator` | Audit engagement with phases (Planning, Fieldwork, Completion) |
| `workpaper_generator` | Audit workpapers per ISA 230 |
| `evidence_generator` | Audit evidence per ISA 500 |
| `risk_generator` | Risk assessment per ISA 315/330 |
| `finding_generator` | Audit findings per ISA 265 |
| `judgment_generator` | Professional judgment documentation per ISA 200 |

### LLM Enrichment (`llm_enrichment/`) — v0.5.0

| Generator | Description |
|-----------|-------------|
| `VendorLlmEnricher` | Generate realistic vendor names by industry, spend category, and country |
| `TransactionLlmEnricher` | Generate transaction descriptions and memo fields |
| `AnomalyLlmExplainer` | Generate natural language explanations for injected anomalies |

### Sourcing (`sourcing/`) -- v0.6.0

Source-to-Contract (S2C) procurement pipeline generators.

| Generator | Description |
|-----------|-------------|
| `spend_analysis_generator` | Spend analysis records and category hierarchies |
| `sourcing_project_generator` | Sourcing project lifecycle management |
| `qualification_generator` | Supplier qualification assessments |
| `rfx_generator` | RFx events (RFI/RFP/RFQ) with invited suppliers |
| `bid_generator` | Supplier bids with pricing and compliance data |
| `bid_evaluation_generator` | Bid scoring, ranking, and award recommendations |
| `contract_generator` | Procurement contracts with terms and renewal rules |
| `catalog_generator` | Catalog items linked to contracts |
| `scorecard_generator` | Supplier scorecards with performance metrics |

Generation DAG: `spend_analysis -> sourcing_project -> qualification -> rfx -> bid -> bid_evaluation -> contract -> catalog -> [P2P] -> scorecard`

### HR (`hr/`) -- v0.6.0

Hire-to-Retire (H2R) generators for the HR process chain.

| Generator | Description |
|-----------|-------------|
| `payroll_generator` | Payroll runs with employee pay line items (gross, deductions, net, employer cost) |
| `time_entry_generator` | Employee time entries with regular, overtime, PTO, and sick hours |
| `expense_report_generator` | Expense reports with categorized line items and approval workflows |

### Standards (`standards/`) -- v0.6.0

Accounting and audit standards generators.

| Generator | Description |
|-----------|-------------|
| `revenue_recognition_generator` | ASC 606/IFRS 15 customer contracts with performance obligations |
| `impairment_generator` | Asset impairment tests with recoverable amount calculations |

### Period Close Additions -- v0.6.0

| Generator | Description |
|-----------|-------------|
| `financial_statement_generator` | Balance sheet, income statement, cash flow, and changes in equity from trial balance data |

### Bank Reconciliation -- v0.6.0

| Generator | Description |
|-----------|-------------|
| `bank_reconciliation_generator` | Bank reconciliations with statement lines, auto-matching, and reconciling items |

### Relationships (`relationships/`)

| Generator | Description |
|-----------|-------------|
| `entity_graph_generator` | Cross-process entity relationship graphs |
| `relationship_strength` | Weighted relationship strength calculation |

**Audit Engagement Structure:**

```rust
pub struct AuditEngagement {
    pub engagement_id: String,
    pub client_name: String,
    pub fiscal_year: u16,
    pub phase: AuditPhase,  // Planning, Fieldwork, Completion
    pub materiality: MaterialityLevels,
    pub team_size: usize,
    pub has_fraud_risk: bool,
    pub has_significant_risk: bool,
}

pub struct MaterialityLevels {
    pub primary_materiality: Decimal,        // 0.3-1% of base
    pub performance_materiality: Decimal,    // 50-75% of primary
    pub clearly_trivial: Decimal,            // 3-5% of primary
}
```

## Usage Examples

### Journal Entry Generation

```rust
use synth_generators::je_generator::JournalEntryGenerator;

let mut generator = JournalEntryGenerator::new(config, seed);

// Generate batch
let entries = generator.generate_batch(1000)?;

// Stream generation
for entry in generator.generate_stream().take(1000) {
    process(entry?);
}
```

### Master Data Generation

```rust
use synth_generators::master_data::{VendorGenerator, CustomerGenerator};

let mut vendor_gen = VendorGenerator::new(seed);
let vendors = vendor_gen.generate(100);

let mut customer_gen = CustomerGenerator::new(seed);
let customers = customer_gen.generate(200);
```

### Document Flow Generation

```rust
use synth_generators::document_flow::{P2pGenerator, O2cGenerator};

let mut p2p = P2pGenerator::new(config, seed);
let p2p_flows = p2p.generate_batch(500)?;

let mut o2c = O2cGenerator::new(config, seed);
let o2c_flows = o2c.generate_batch(500)?;
```

### Anomaly Injection

```rust
use synth_generators::anomaly::AnomalyInjector;

let mut injector = AnomalyInjector::new(config.anomaly_injection, seed);

// Inject into existing entries
let (modified_entries, labels) = injector.inject(&entries)?;
```

### LLM Enrichment

```rust
use synth_generators::llm_enrichment::{VendorLlmEnricher, TransactionLlmEnricher};
use synth_core::llm::MockLlmProvider;
use std::sync::Arc;

let provider = Arc::new(MockLlmProvider::new(42));

// Enrich vendor names
let vendor_enricher = VendorLlmEnricher::new(provider.clone());
let name = vendor_enricher.enrich_vendor_name("manufacturing", "raw_materials", "US")?;

// Enrich transaction descriptions
let tx_enricher = TransactionLlmEnricher::new(provider);
let desc = tx_enricher.enrich_description("Office Supplies", "1000-5000", "retail", 3)?;
let memo = tx_enricher.enrich_memo("VendorInvoice", "Acme Corp", "2500.00")?;
```

## Three-Way Match

The P2P generator validates document matching:

```rust
use synth_generators::document_flow::ThreeWayMatch;

let match_result = ThreeWayMatch::validate(
    &purchase_order,
    &goods_receipt,
    &vendor_invoice,
    tolerance_config,
);

match match_result {
    MatchResult::Passed => { /* Process normally */ }
    MatchResult::QuantityVariance(var) => { /* Handle variance */ }
    MatchResult::PriceVariance(var) => { /* Handle variance */ }
}
```

## Balance Coherence

The balance tracker maintains accounting equation:

```rust
use synth_generators::balance::BalanceTracker;

let mut tracker = BalanceTracker::new();

for entry in &entries {
    tracker.post(&entry)?;
}

// Verify Assets = Liabilities + Equity
assert!(tracker.is_balanced());
```

## FX Rate Generation

Uses Ornstein-Uhlenbeck process for realistic rate movements:

```rust
use synth_generators::fx::FxRateService;

let mut fx_service = FxRateService::new(config.fx, seed);

// Get rate for date
let rate = fx_service.get_rate("EUR", "USD", date)?;

// Generate daily rates
let rates = fx_service.generate_daily_rates(start, end)?;
```

## Anomaly Types

### Fraud Types
- FictitiousTransaction, RevenueManipulation, ExpenseCapitalization
- SplitTransaction, RoundTripping, KickbackScheme
- GhostEmployee, DuplicatePayment, UnauthorizedDiscount

### Error Types
- DuplicateEntry, ReversedAmount, WrongPeriod
- WrongAccount, MissingReference, IncorrectTaxCode

### Process Issues
- LatePosting, SkippedApproval, ThresholdManipulation
- MissingDocumentation, OutOfSequence

### Statistical Anomalies
- UnusualAmount, TrendBreak, BenfordViolation, OutlierValue

### Relational Anomalies
- CircularTransaction, DormantAccountActivity, UnusualCounterparty

## See Also

- [datasynth-core](datasynth-core.md)
- [Anomaly Injection](../advanced/anomaly-injection.md)
- [Document Flows](../configuration/document-flows.md)
