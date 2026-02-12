# datasynth-generators

Data generators for journal entries, master data, document flows, and anomalies.

## Overview

`datasynth-generators` contains all data generation logic for SyntheticData:

- **Core Generators**: Journal entries, chart of accounts, users
- **Master Data**: Vendors, customers, materials, assets, employees
- **Document Flows**: P2P (Procure-to-Pay), O2C (Order-to-Cash)
- **Financial**: Intercompany, balance tracking, subledgers, FX, period close
- **Quality**: Anomaly injection, data quality variations

## Generator Modules

### Core Generators

| Generator | Description |
|-----------|-------------|
| `je_generator` | Journal entry generation with statistical distributions |
| `coa_generator` | Chart of accounts with industry-specific structures |
| `company_selector` | Weighted company selection for transactions |
| `user_generator` | User/persona generation with roles |
| `control_generator` | Internal controls and SoD rules |

### Master Data (`master_data/`)

| Generator | Description |
|-----------|-------------|
| `vendor_generator` | Vendors with payment terms, bank accounts, behaviors |
| `customer_generator` | Customers with credit ratings, payment patterns |
| `material_generator` | Materials/products with BOM, valuations |
| `asset_generator` | Fixed assets with depreciation schedules |
| `employee_generator` | Employees with manager hierarchy |

### Document Flow (`document_flow/`)

| Generator | Description |
|-----------|-------------|
| `p2p_generator` | PO → GR → Invoice → Payment flow |
| `o2c_generator` | SO → Delivery → Invoice → Receipt flow |
| `document_chain_manager` | Reference chain management |
| `three_way_match` | PO/GR/Invoice matching validation |

### Financial (`intercompany/`, `balance/`, `subledger/`, `fx/`, `period_close/`)

| Generator | Description |
|-----------|-------------|
| `ic_generator` | Matched intercompany entry pairs |
| `balance_tracker` | Running balance validation |
| `ar_generator` / `ap_generator` | Subledger records |
| `fx_rate_service` | Ornstein-Uhlenbeck FX rates |
| `close_engine` | Period close orchestration |

### Sourcing (`sourcing/`) (v0.6.0)

| Generator | Description |
|-----------|-------------|
| `sourcing_project_generator` | End-to-end sourcing project lifecycle |
| `rfx_generator` | RFI/RFP/RFQ event generation |
| `supplier_bid_generator` | Supplier bid responses with scoring |
| `procurement_contract_generator` | Contract creation from awarded bids |
| `catalog_generator` | Procurement catalog items and pricing |
| 4 additional generators | Supplier qualification, spend analysis, savings tracking, compliance |

### HR (`hr/`) (v0.6.0)

| Generator | Description |
|-----------|-------------|
| `payroll_generator` | Payroll runs with deductions and tax withholdings |
| `time_generator` | Time entry tracking with approval workflows |
| `expense_generator` | Expense reports with receipt matching and policy checks |

### Manufacturing (`manufacturing/`) (v0.6.0)

| Generator | Description |
|-----------|-------------|
| `production_order_generator` | Production orders with operations and material consumption |
| `quality_inspection_generator` | Quality inspection lots and defect recording |
| `cycle_count_generator` | Inventory cycle count programs and variance analysis |

### Standards (`standards/`) (v0.6.0)

| Generator | Description |
|-----------|-------------|
| `revenue_recognition_generator` | ASC 606 / IFRS 15 revenue recognition schedules |
| `impairment_generator` | ASC 360 / IAS 36 impairment test generation |

### Additional Generators (v0.6.0)

| Generator | Description |
|-----------|-------------|
| `bank_reconciliation_generator` | Bank statement reconciliation with auto-matching |
| `financial_statement_generator` | Income statement, balance sheet, cash flow generation (in `period_close/`) |
| `sales_quote_generator` | Sales quotation generation with quote-to-order conversion |
| `kpi_generator` | Management KPI and scorecard metric generation |
| `budget_generator` | Budget planning with line items and variance analysis |

### Quality (`anomaly/`, `data_quality/`)

| Generator | Description |
|-----------|-------------|
| `injector` | Anomaly injection engine |
| `missing_values` | MCAR, MAR, MNAR patterns |
| `typos` | Keyboard-aware typo generation |
| `duplicates` | Exact, near, fuzzy duplicates |

## Usage

```rust
use datasynth_generators::je_generator::JournalEntryGenerator;
use datasynth_generators::master_data::VendorGenerator;

let mut je_gen = JournalEntryGenerator::new(config, seed);
let entries = je_gen.generate_batch(1000)?;

let mut vendor_gen = VendorGenerator::new(seed);
let vendors = vendor_gen.generate(100);
```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
