# Data Flow

How data flows through the DataSynth system.

## High-Level Flow

```
┌─────────────┐
│   Config    │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────┐
│                     Orchestrator                             │
│                                                              │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐ │
│  │  Master  │ → │  Opening │ → │ Transact │ → │  Period  │ │
│  │   Data   │   │ Balances │   │   ions   │   │  Close   │ │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘ │
│                                                              │
└───────────────────────────┬─────────────────────────────────┘
                            │
       ┌────────────────────┼────────────────────┐
       │                    │                    │
       ▼                    ▼                    ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  CSV Sink   │      │ Graph Export│      │  Labels     │
└─────────────┘      └─────────────┘      └─────────────┘
```

## Phase 1: Configuration Loading

```
YAML File → Parser → Validator → Config Object
```

1. **Load**: Read YAML/JSON file
2. **Parse**: Convert to strongly-typed structures
3. **Validate**: Check constraints and ranges
4. **Resolve**: Apply defaults and presets

```rust
let config = Config::from_yaml_file("config.yaml")?;
ConfigValidator::new().validate(&config)?;
```

## Phase 2: Master Data Generation

```
Config → Master Data Generators → Entity Registry
```

Order of generation (to satisfy dependencies):

1. **Chart of Accounts**: GL account structure
2. **Employees**: Users with approval limits
3. **Vendors**: Suppliers (reference employees as approvers)
4. **Customers**: Buyers (reference employees)
5. **Materials**: Products (reference accounts)
6. **Fixed Assets**: Capital assets (reference accounts)

```rust
// Entity registry maintains references
let registry = EntityRegistry::new();
registry.register_vendors(&vendors);
registry.register_customers(&customers);
```

## Phase 3: Opening Balance Generation

```
Config + CoA → Balance Generator → Opening JEs
```

Generates coherent opening balance sheet:

1. Calculate target balances per account type
2. Distribute across accounts
3. Generate opening entries
4. Verify A = L + E

```rust
let opening = OpeningBalanceGenerator::new(&config);
let entries = opening.generate()?;

// Verify balance coherence
assert!(entries.iter().all(|e| e.is_balanced()));
```

## Phase 4: Transaction Generation

### Document Flow Path

```
Config → P2P/O2C Generators → Documents → JE Generator → Entries
```

P2P Flow:
```
PO Generator → Purchase Order
                    │
                    ▼
GR Generator → Goods Receipt → JE (Inventory/GR-IR)
                    │
                    ▼
Invoice Gen. → Vendor Invoice → JE (GR-IR/AP)
                    │
                    ▼
Payment Gen. → Payment → JE (AP/Cash)
```

### Direct JE Path

```
Config → JE Generator → Entries
```

For transactions not from document flows:
- Manual entries
- Recurring entries
- Adjustments

## Phase 5: Balance Tracking

```
Entries → Balance Tracker → Running Balances → Trial Balance
```

Continuous tracking during generation:

```rust
let mut tracker = BalanceTracker::new(&coa);

for entry in &entries {
    tracker.post(&entry)?;

    // Verify coherence after each entry
    assert!(tracker.is_balanced());
}

let trial_balance = tracker.to_trial_balance(period);
```

## Phase 6: Anomaly Injection

```
Entries → Anomaly Injector → Modified Entries + Labels
```

Anomalies injected post-generation:

1. Select entries based on targeting strategy
2. Apply anomaly transformation
3. Generate label record

```rust
let injector = AnomalyInjector::new(&config.anomaly_injection);
let (modified, labels) = injector.inject(&entries)?;
```

## Phase 7: Period Close

```
Entries + Balances → Close Engine → Closing Entries
```

Monthly:
- Accruals
- Depreciation
- Subledger reconciliation

Quarterly:
- IC eliminations
- Currency translation

Annual:
- Closing entries
- Retained earnings

## Phase 8: Output Generation

### CSV/JSON Output

```
Entries + Master Data → Sinks → Files
```

```rust
let mut sink = CsvSink::new("output/journal_entries.csv")?;
sink.write_batch(&entries)?;
sink.flush()?;
```

### Graph Output

```
Entries → Graph Builder → Graph → Exporter → PyG/Neo4j
```

```rust
let builder = TransactionGraphBuilder::new();
let graph = builder.build(&entries)?;

let exporter = PyTorchGeometricExporter::new("output/graphs");
exporter.export(&graph, split_config)?;
```

## Phase 9: Enterprise Process Chains (v0.6.0)

### Source-to-Contract (S2C) Flow

```
Spend Analysis → Sourcing Project → Supplier Qualification → RFx Event → Bids →
Bid Evaluation → Contract Award → Catalog Items → [feeds into P2P] → Supplier Scorecard
```

S2C data feeds into the existing P2P procurement flow. Procurement contracts and catalog items provide the upstream sourcing context for purchase orders.

### HR / Payroll Flow

```
Employees (Master Data) → Time Entries → Payroll Run → JE (Salary Expense/Cash)
                        → Expense Reports → JE (Expense/AP)
```

HR data depends on the employee master data from Phase 2. Payroll runs generate journal entries that post to salary expense and cash accounts.

### Financial Reporting Flow

```
Trial Balance → Balance Sheet + Income Statement
             → Cash Flow Statement (indirect method)
             → Changes in Equity
             → Management KPIs
             → Budget Variance Analysis

Payments (P2P/O2C) → Bank Reconciliation → Matched/Unmatched Items
```

Financial statements are derived from the adjusted trial balance. Bank reconciliations match payments from document flows against bank statement lines.

### Manufacturing Flow

```
Materials (Master Data) → Production Orders → Quality Inspections
                                            → Cycle Counts
```

Manufacturing data depends on materials from the master data. Production orders consume raw materials and produce finished goods.

### Sales Quote Flow

```
Customers (Master Data) → Sales Quotes → [feeds into O2C when won]
```

The quote-to-order pipeline generates sales quotes that, when won, link to sales orders in the O2C flow.

### Accounting Standards Flow

```
Customers → Customer Contracts → Performance Obligations (ASC 606/IFRS 15)
Fixed Assets → Impairment Tests → Recoverable Amount Calculations
```

Revenue recognition generates contracts with performance obligations. Impairment testing evaluates fixed asset carrying amounts against recoverable values.

## Data Dependencies

```
         ┌─────────────┐
         │    Config   │
         └──────┬──────┘
                │
    ┌───────────┼───────────┐
    │           │           │
    ▼           ▼           ▼
┌───────┐  ┌───────┐  ┌───────┐
│  CoA  │  │Vendors│  │Customs│
└───┬───┘  └───┬───┘  └───┬───┘
    │          │          │
    │    ┌─────┴─────┐    │
    │    │           │    │
    ▼    ▼           ▼    ▼
┌─────────────┐  ┌─────────────┐
│   P2P Docs  │  │   O2C Docs  │
└──────┬──────┘  └──────┬──────┘
       │                │
       └───────┬────────┘
               │
               ▼
        ┌─────────────┐
        │   Entries   │
        └──────┬──────┘
               │
    ┌──────────┼──────────┐──────────┐──────────┐
    │          │          │          │          │
    ▼          ▼          ▼          ▼          ▼
┌───────┐ ┌───────┐ ┌───────┐ ┌─────────┐ ┌───────┐
│  TB   │ │ Graph │ │Labels │ │Fin.Stmt │ │BankRec│
└───────┘ └───────┘ └───────┘ └─────────┘ └───────┘
```

## Streaming vs Batch

### Batch Mode

All data in memory:

```rust
let entries = generator.generate_batch(100000)?;
sink.write_batch(&entries)?;
```

**Pro:** Fast parallel processing
**Con:** Memory intensive

### Streaming Mode

Process one at a time:

```rust
for entry in generator.generate_stream() {
    sink.write(&entry?)?;
}
```

**Pro:** Memory efficient
**Con:** No parallelism

### Hybrid Mode

Batch with periodic flush:

```rust
for batch in generator.generate_batches(1000) {
    let entries = batch?;
    sink.write_batch(&entries)?;

    if memory_guard.check().exceeds_soft_limit {
        sink.flush()?;
    }
}
```

## See Also

- [Generation Pipeline](generation-pipeline.md)
- [Memory Management](memory-management.md)
- [datasynth-runtime](../crates/datasynth-runtime.md)
