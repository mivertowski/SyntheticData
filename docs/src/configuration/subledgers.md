# Subledgers

SyntheticData generates subsidiary ledger records for Accounts Receivable (AR), Accounts Payable (AP), Fixed Assets (FA), and Inventory, with automatic GL reconciliation and document flow linking.

## Overview

Subledger generators produce detailed records that reconcile back to GL control accounts:

| Subledger | Control Account | Record Types | Output Files |
|-----------|----------------|--------------|-------------|
| **AR** | 1100 (AR Control) | Open items, aging, receipts, credit memos, dunning | `ar_open_items.csv`, `ar_aging.csv` |
| **AP** | 2000 (AP Control) | Open items, aging, payment scheduling, debit memos | `ap_open_items.csv`, `ap_aging.csv` |
| **FA** | 1600+ (Asset accounts) | Register, depreciation, acquisitions, disposals | `fa_register.csv`, `fa_depreciation.csv` |
| **Inventory** | 1300 (Inventory) | Positions, movements (22 types), valuation | `inventory_positions.csv`, `inventory_movements.csv` |

## Configuration

```yaml
subledger:
  enabled: true
  ar:
    enabled: true
    aging_buckets: [30, 60, 90, 120]    # Days
    dunning_levels: 3
    credit_memo_rate: 0.05               # 5% of invoices get credit memos
  ap:
    enabled: true
    aging_buckets: [30, 60, 90, 120]
    early_payment_discount_rate: 0.02
    payment_scheduling: true
  fa:
    enabled: true
    depreciation_methods:
      - straight_line
      - declining_balance
      - sum_of_years_digits
    disposal_rate: 0.03                  # 3% of assets disposed per year
  inventory:
    enabled: true
    valuation_method: standard_cost      # standard_cost, moving_average, fifo, lifo
    cycle_count_frequency: monthly
```

---

## Accounts Receivable (AR)

### Record Types

The AR subledger generates:

- **Open Items**: Outstanding customer invoices with aging classification
- **Receipts**: Customer payments applied to invoices (full, partial, on-account)
- **Credit Memos**: Credits issued for returns, disputes, or pricing adjustments
- **Aging Reports**: Aged balances by customer and aging bucket
- **Dunning Notices**: Automated collection notices at configurable levels

### Open Item Fields

| Field | Description |
|-------|-------------|
| `customer_id` | Customer reference |
| `invoice_number` | Document number |
| `invoice_date` | Issue date |
| `due_date` | Payment due date |
| `original_amount` | Invoice total |
| `open_amount` | Remaining balance |
| `currency` | Invoice currency |
| `payment_terms` | Net 30, Net 60, etc. |
| `aging_bucket` | 0-30, 31-60, 61-90, 91-120, 120+ |
| `dunning_level` | Current dunning level (0-3) |
| `last_dunning_date` | Date of last dunning notice |
| `dispute_flag` | Whether item is disputed |

### Aging Buckets

Default aging buckets classify receivables by days past due:

| Bucket | Range | Typical % |
|--------|-------|-----------|
| Current | 0-30 days | 65-75% |
| 31-60 | 31-60 days | 12-18% |
| 61-90 | 61-90 days | 5-8% |
| 91-120 | 91-120 days | 2-4% |
| 120+ | Over 120 days | 1-3% |

### Dunning Process

Dunning generates progressively urgent collection notices:

| Level | Days Overdue | Action |
|-------|-------------|--------|
| 0 | 0-30 | No action (within terms) |
| 1 | 31-60 | Friendly reminder |
| 2 | 61-90 | Formal notice |
| 3 | 90+ | Final demand / collections |

### Document Flow Integration

AR open items are created from O2C customer invoices:

```
Sales Order → Delivery → Customer Invoice → AR Open Item → Customer Receipt
                                                 │
                                                 └→ Dunning Notice (if overdue)
```

---

## Accounts Payable (AP)

### Record Types

The AP subledger generates:

- **Open Items**: Outstanding vendor invoices with aging and payment scheduling
- **Payments**: Vendor payment runs (check, wire, ACH)
- **Debit Memos**: Deductions for quality issues, returns, pricing errors
- **Aging Reports**: Aged payables by vendor
- **Payment Scheduling**: Planned payments considering cash flow and discounts

### Open Item Fields

| Field | Description |
|-------|-------------|
| `vendor_id` | Vendor reference |
| `invoice_number` | Vendor invoice number |
| `invoice_date` | Invoice receipt date |
| `due_date` | Payment due date |
| `baseline_date` | Date for terms calculation |
| `original_amount` | Invoice total |
| `open_amount` | Remaining balance |
| `currency` | Invoice currency |
| `payment_terms` | 2/10 Net 30, etc. |
| `discount_date` | Discount deadline |
| `discount_amount` | Available discount |
| `payment_block` | Block code (if blocked) |
| `three_way_match_status` | Matched / Variance / Blocked |

### Early Payment Discounts

The AP generator models cash discount optimization:

```
Payment Terms: 2/10 Net 30
  → Pay within 10 days: 2% discount
  → Pay by day 30: full amount
  → Past day 30: overdue

early_payment_discount_rate: 0.02   # Take 2% discount when offered
```

### Payment Scheduling

When enabled, the AP generator creates a payment schedule that optimizes:

- **Discount capture**: Prioritize invoices with expiring discounts
- **Cash flow**: Spread payments across the period
- **Vendor priority**: Pay critical vendors first

### Document Flow Integration

AP open items are created from P2P vendor invoices:

```
Purchase Order → Goods Receipt → Vendor Invoice → Three-Way Match → AP Open Item → Payment
                                                                          │
                                                                          └→ Debit Memo (if variance)
```

---

## Fixed Assets (FA)

### Record Types

The FA subledger generates:

- **Asset Register**: Master record for each fixed asset
- **Depreciation Schedule**: Monthly depreciation entries per asset
- **Acquisitions**: New asset additions (from PO or direct capitalization)
- **Disposals**: Asset retirements, sales, scrapping
- **Transfers**: Inter-company or inter-department transfers
- **Impairment**: Write-downs when fair value drops below book value

### Asset Register Fields

| Field | Description |
|-------|-------------|
| `asset_id` | Unique identifier |
| `description` | Asset name/description |
| `asset_class` | Buildings, Equipment, Vehicles, IT, Furniture |
| `acquisition_date` | Purchase/capitalization date |
| `acquisition_cost` | Original cost |
| `useful_life_years` | Depreciable life |
| `salvage_value` | Residual value |
| `depreciation_method` | Method used |
| `accumulated_depreciation` | Total depreciation to date |
| `net_book_value` | Current carrying value |
| `disposal_date` | Date retired (if applicable) |
| `disposal_proceeds` | Sale price (if sold) |
| `disposal_gain_loss` | Gain or loss on disposal |

### Depreciation Methods

| Method | Description | Use Case |
|--------|-------------|----------|
| `StraightLine` | Equal amounts each period | Default, most common |
| `DecliningBalance { rate }` | Fixed percentage of remaining balance | Accelerated (tax) |
| `SumOfYearsDigits` | Decreasing fractions of depreciable base | Accelerated |
| `UnitsOfProduction { total_units }` | Based on usage/output | Manufacturing equipment |
| `None` | No depreciation | Land, construction in progress |

### Depreciation Journal Entries

Each period, the FA generator creates depreciation entries:

| Debit | Credit | Amount |
|-------|--------|--------|
| Depreciation Expense (6xxx) | Accumulated Depreciation (1650) | Period depreciation |

### Disposal Accounting

When an asset is disposed:

| Scenario | Debit | Credit |
|----------|-------|--------|
| Sale at gain | Cash, Accum Depr | Asset Cost, Gain on Disposal |
| Sale at loss | Cash, Accum Depr, Loss on Disposal | Asset Cost |
| Scrapping | Accum Depr, Loss on Disposal | Asset Cost |

---

## Inventory

### Record Types

The Inventory subledger generates:

- **Positions**: Current stock levels by material, plant, and storage location
- **Movements**: 22 movement types covering receipts, issues, transfers, and adjustments
- **Valuation**: Inventory value calculated using configurable valuation methods

### Position Fields

| Field | Description |
|-------|-------------|
| `material_id` | Material reference |
| `plant` | Plant/warehouse code |
| `storage_location` | Storage location within plant |
| `quantity` | Units on hand |
| `unit_of_measure` | UOM |
| `unit_cost` | Per-unit cost |
| `total_value` | Extended value |
| `valuation_method` | StandardCost, MovingAverage, FIFO, LIFO |
| `stock_status` | Unrestricted, QualityInspection, Blocked |
| `last_movement_date` | Date of last stock change |

### Movement Types (22 types)

| Category | Movement Type | Description |
|----------|--------------|-------------|
| **Goods Receipt** | `GoodsReceiptPO` | Receipt against purchase order |
| | `GoodsReceiptProduction` | Receipt from production order |
| | `GoodsReceiptOther` | Receipt without reference |
| | `GoodsReceipt` | Generic goods receipt |
| **Returns** | `ReturnToVendor` | Return materials to vendor |
| **Goods Issue** | `GoodsIssueSales` | Issue for sales order / delivery |
| | `GoodsIssueProduction` | Issue to production order |
| | `GoodsIssueCostCenter` | Issue to cost center (consumption) |
| | `GoodsIssueScrapping` | Scrap disposal |
| | `GoodsIssue` | Generic goods issue |
| | `Scrap` | Alias for scrapping |
| **Transfers** | `TransferPlant` | Between plants |
| | `TransferStorageLocation` | Between storage locations |
| | `TransferIn` | Inbound transfer |
| | `TransferOut` | Outbound transfer |
| | `TransferToInspection` | Move to quality inspection |
| | `TransferFromInspection` | Release from quality inspection |
| **Adjustments** | `PhysicalInventory` | Physical count difference |
| | `InventoryAdjustmentIn` | Positive adjustment |
| | `InventoryAdjustmentOut` | Negative adjustment |
| | `InitialStock` | Initial stock entry |
| **Reversals** | `ReversalGoodsReceipt` | Reverse a goods receipt |
| | `ReversalGoodsIssue` | Reverse a goods issue |

### Valuation Methods

| Method | Description | Use Case |
|--------|-------------|----------|
| `StandardCost` | Fixed cost per unit, variances posted separately | Manufacturing |
| `MovingAverage` | Weighted average of all receipts | General purpose |
| `FIFO` | First-in, first-out costing | Perishable goods |
| `LIFO` | Last-in, first-out costing | Tax optimization (where permitted) |

### Cycle Counting (v0.6.0)

The `cycle_count_frequency` setting controls how often physical inventory counts are performed. Cycle counting generates `PhysicalInventory` movement records that reconcile book quantities against counted quantities:

```yaml
subledger:
  inventory:
    enabled: true
    cycle_count_frequency: monthly     # monthly, quarterly, annual
```

| Frequency | Behavior |
|-----------|----------|
| `monthly` | Each storage location counted once per month on a rolling basis |
| `quarterly` | Full count once per quarter, with high-value items counted monthly |
| `annual` | Single year-end wall-to-wall count |

Cycle count differences generate adjustment entries (`InventoryAdjustmentIn` or `InventoryAdjustmentOut`) and are flagged in the quality labels output for audit trail analysis.

### Quality Inspection (v0.6.0)

Inventory positions can be placed in quality inspection status via `TransferToInspection` movements. This models the inspection hold process common in manufacturing and pharmaceutical industries:

```
Goods Receipt → Transfer to Inspection → QC Hold → Transfer from Inspection → Unrestricted Use
                                                 └→ Scrap (if rejected)
```

The rate of items routed through inspection depends on the material type and vendor scorecard grades (when `source_to_pay` is enabled). Materials from vendors with grade C or lower are routed through inspection at a higher rate.

### Inventory Journal Entries

| Movement | Debit | Credit |
|----------|-------|--------|
| Goods Receipt (PO) | Inventory | GR/IR Clearing |
| Goods Issue (Sales) | COGS | Inventory |
| Goods Issue (Production) | WIP | Inventory |
| Scrap | Scrap Expense | Inventory |
| Physical Count (surplus) | Inventory | Inventory Adjustment |
| Physical Count (shortage) | Inventory Adjustment | Inventory |

---

## GL Reconciliation

The subledger generators ensure that subledger balances reconcile to GL control accounts:

```
GL Control Account Balance = Σ Subledger Open Items

AR Control (1100) = Σ AR Open Items
AP Control (2000) = Σ AP Open Items
Inventory  (1300) = Σ Inventory Position Values
FA Gross   (1600) = Σ FA Acquisition Costs
Accum Depr (1650) = Σ FA Accumulated Depreciation
```

Reconciliation is validated by the `datasynth-eval` coherence module and any differences are flagged as potential data quality issues.

## Output Files

| File | Content |
|------|---------|
| `subledgers/ar_open_items.csv` | AR outstanding invoices |
| `subledgers/ar_aging.csv` | AR aging analysis |
| `subledgers/ap_open_items.csv` | AP outstanding invoices |
| `subledgers/ap_aging.csv` | AP aging analysis |
| `subledgers/fa_register.csv` | Fixed asset master records |
| `subledgers/fa_depreciation.csv` | Depreciation schedule entries |
| `subledgers/inventory_positions.csv` | Current stock positions |
| `subledgers/inventory_movements.csv` | Stock movement history |

## See Also

- [Document Flows](document-flows.md) — P2P and O2C document chains
- [Financial Settings](financial-settings.md) — Balance and period close config
- [FX & Currency](fx-currency.md) — Multi-currency subledger support
- [datasynth-generators](../crates/datasynth-generators.md) — Generator crate reference
