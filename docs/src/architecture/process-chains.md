# Enterprise Process Chains

SyntheticData models enterprise operations as interconnected process chains — end-to-end business flows that share master data, generate journal entries, and link through common documents. This page maps the current implementation status and shows how the chains integrate.

## Coverage Matrix

| Chain | Full Name | Coverage | Status | Key Modules |
|-------|-----------|----------|--------|-------------|
| **S2P** | Source-to-Pay | 85% | Implemented (P2P complete, S2C planned) | `document_flow/p2p_generator`, `master_data/vendor`, `subledger/ap` |
| **O2C** | Order-to-Cash | 93% | Implemented | `document_flow/o2c_generator`, `master_data/customer`, `subledger/ar` |
| **R2R** | Record-to-Report | 78% | Mostly implemented | `je_generator`, `balance/`, `period_close/`, `intercompany/` |
| **A2R** | Acquire-to-Retire | 70% | Partially implemented | `master_data/asset`, `subledger/fa`, `period_close/depreciation` |
| **INV** | Inventory Management | 55% | Partially implemented | `subledger/inventory`, `document_flow/` (GR/delivery links) |
| **BANK** | Banking & Treasury | 65% | Partially implemented | `datasynth-banking` (KYC/AML), payment clearing |
| **H2R** | Hire-to-Retire | 30% | Minimal | `master_data/employee`, `user_generator` |
| **MFG** | Plan-to-Produce | 20% | Config only | Industry-specific manufacturing config |

---

## Implemented Chains

### Source-to-Pay (S2P)

The S2P chain covers procurement from purchase requisition through payment:

```
                    Source-to-Contract (S2C) — Planned
                    ┌──────────────────────────────────────────────┐
                    │ Spend Analysis → RFx → Bid Eval → Contract  │
                    └──────────────────────────┬───────────────────┘
                                               │
    ┌──────────────────────────────────────────┼──────────────────────────┐
    │              Procure-to-Pay (P2P) — Implemented                    │
    │                                          │                         │
    │  Purchase    Purchase    Goods     Vendor    Three-Way              │
    │  Requisition → Order  → Receipt → Invoice → Match    → Payment    │
    │                  │                   │         │           │        │
    │                  │              ┌────┘         │           │        │
    │                  ▼              ▼              ▼           ▼        │
    │              AP Open Item ← Match Result   AP Aging    Bank        │
    └────────────────────────────────────────────────────────────────────┘
                                               │
                    ┌──────────────────────────┘
                    ▼
    Vendor Network (quality scores, clusters, supply chain tiers)
```

**P2P implementation details:**

| Component | Types/Variants | Key Config |
|-----------|---------------|------------|
| Purchase Orders | 6 types: Standard, Service, Framework, Consignment, StockTransfer, Subcontracting | `flow_rate`, `completion_rate` |
| Goods Receipts | 7 movement types: GrForPo, ReturnToVendor, GrForProduction, TransferPosting, InitialEntry, Scrapping, Consumption | `gr_rate` |
| Vendor Invoices | Three-way match with tolerance | `price_tolerance`, `quantity_tolerance` |
| Payments | Configurable terms and scheduling | `payment_rate`, timing ranges |
| Three-Way Match | PO ↔ GR ↔ Invoice validation with 6 variance types | `allow_over_delivery`, `max_over_delivery_pct` |

### Order-to-Cash (O2C)

The O2C chain covers the revenue cycle from sales order through cash collection:

```
    ┌─────────────────────────────────────────────────────────────────────┐
    │                    Order-to-Cash (O2C)                              │
    │                                                                     │
    │  Sales    Credit   Delivery   Customer   Customer                   │
    │  Order  → Check  → (Pick/  → Invoice  → Receipt                    │
    │    │               Pack/        │          │                        │
    │    │               Ship)        │          │                        │
    │    │                │           ▼          ▼                        │
    │    │                │      AR Open Item  AR Aging                   │
    │    │                │           │                                   │
    │    │                │           └→ Dunning Notices                  │
    │    │                ▼                                               │
    │    │          Inventory Issue                                       │
    │    │          (COGS posting)                                        │
    └────┼────────────────────────────────────────────────────────────────┘
         │
    Revenue Recognition (ASC 606 / IFRS 15)
    Customer Contracts → Performance Obligations
```

**O2C implementation details:**

| Component | Types/Variants | Key Config |
|-----------|---------------|------------|
| Sales Orders | 9 types: Standard, Rush, CashSale, Return, FreeOfCharge, Consignment, Service, CreditMemoRequest, DebitMemoRequest | `flow_rate`, `credit_check` |
| Deliveries | 6 types: Outbound, Return, StockTransfer, Replenishment, ConsignmentIssue, ConsignmentReturn | `delivery_rate` |
| Customer Invoices | 7 types: Standard, CreditMemo, DebitMemo, ProForma, DownPaymentRequest, FinalInvoice, Intercompany | `invoice_rate` |
| Customer Receipts | Full, partial, on-account, corrections, NSF | `collection_rate` |

### Record-to-Report (R2R)

The R2R chain covers financial close and reporting:

```
    Journal Entries (from all chains)
         │
         ▼
    Balance Tracker → Trial Balance → Adjustments → Close
         │                                │            │
         ├→ Intercompany Matching         ├→ Accruals   ├→ Year-End Close
         │     └→ IC Elimination          ├→ Reclasses  └→ Retained Earnings
         │                                └→ FX Reval
         ▼
    Consolidation
         ├→ Currency Translation
         ├→ CTA Adjustments
         └→ Consolidated Trial Balance
```

**R2R coverage:**
- Journal entry generation from all process chains
- Opening balance, running balance tracking, trial balance per period
- Intercompany matching and elimination entries
- Period close engine: accruals, depreciation, year-end closing
- Audit simulation (ISA-compliant workpapers, findings, opinions)

**Gaps:** Financial statement generation (balance sheet, income statement, cash flow), budget vs actual reporting.

---

## Partially Implemented Chains

### Acquire-to-Retire (A2R) — 70%

**Implemented:** Fixed asset master data, depreciation (6 methods), acquisition from PO, disposal with gain/loss accounting, impairment testing (ASC 360/IAS 36).

**Gaps:** Capital project/WBS integration, asset transfers between companies, construction-in-progress (CIP) tracking.

### Inventory Management (INV) — 55%

**Implemented:** Inventory positions, 22 movement types, 4 valuation methods, stock status tracking, P2P goods receipts, O2C goods issues.

**Gaps:** Cycle counting with count programs, quality inspection integration, obsolescence management, ABC analysis.

### Banking & Treasury (BANK) — 65%

**Implemented:** Bank customer profiles, KYC/AML, bank accounts, transactions with fraud typologies (structuring, funnel, layering, mule, round-tripping).

**Gaps:** Bank statement reconciliation, cash forecasting, liquidity management.

### Hire-to-Retire (H2R) — 30%

**Implemented:** Employee master data, user/authorization generation.

**Gaps:** Payroll runs, time management, expense management, benefits, workforce planning.

### Plan-to-Produce (MFG) — 20%

**Implemented:** Industry-specific manufacturing configuration (BOM depth, yield rates, work centers).

**Gaps:** Production orders, WIP costing, material requirements planning (MRP), shop floor control.

---

## Cross-Process Integration

Process chains share data through several integration points:

```
         S2P                    O2C                    R2R
          │                      │                      │
    GR ───┼──── Inventory ───────┼── Delivery           │
          │         │            │                      │
    Payment ────────┼────────────┼── Receipt ──── Bank Recon
          │         │            │                      │
    AP Open Item    │       AR Open Item                │
          │         │            │                      │
          └─────────┴────────────┴───── Journal Entries ┘
                                                │
                                          Trial Balance
                                                │
                                         Consolidation
```

### Integration Map

| Integration Point | From Chain | To Chain | Mechanism |
|-------------------|-----------|----------|-----------|
| Inventory bridge | S2P (Goods Receipt) | O2C (Delivery) | GR increases stock, delivery decreases |
| Payment clearing | S2P / O2C | BANK | Payment status → bank reconciliation |
| Journal entries | All chains | R2R | Every document posts GL entries |
| Asset acquisition | S2P (Capital PO) | A2R | PO → GR → Fixed Asset Record |
| Revenue recognition | O2C (Invoice) | R2R | Contract → Revenue JE |
| Depreciation | A2R | R2R | Monthly depreciation → Trial Balance |
| Intercompany | S2P / O2C | R2R | IC invoices → IC matching → elimination |

### Document Reference Types

Documents maintain referential integrity across chains through 9 reference types:

| Reference Type | Description | Example |
|----------------|-------------|---------|
| `FollowOn` | Normal flow succession | PO → GR |
| `Payment` | Payment for invoice | PAY → VI |
| `Reversal` | Correction/reversal | Credit Memo → Invoice |
| `Partial` | Partial fulfillment | Partial GR → PO |
| `CreditMemo` | Credit against invoice | CM → Invoice |
| `DebitMemo` | Debit against invoice | DM → Invoice |
| `Return` | Return against delivery | Return → Delivery |
| `IntercompanyMatch` | IC matching pair | IC-INV → IC-INV |
| `Manual` | User-defined reference | Any → Any |

---

## Roadmap

The process chain expansion follows a wave-based plan:

| Wave | Focus | Chains Affected |
|------|-------|----------------|
| **Wave 1** | S2C completion, bank reconciliation, financial statements | S2P, BANK, R2R |
| **Wave 2** | Payroll/time, revenue recognition generator, impairment generator | H2R, O2C, A2R |
| **Wave 3** | Production orders/WIP, cycle counting/QA, expense management | MFG, INV, H2R |
| **Wave 4** | Sales quotes, cash forecasting, KPIs/budgets, obsolescence | O2C, BANK, R2R, INV |

For detailed coverage targets and implementation plans, see:
- [S2P Process Chain Spec](../../specs/s2p-process-chain-spec.md) — Source-to-Contract extension
- [Enterprise Process Chain Gaps](../../specs/enterprise-process-chain-gaps.md) — Full gap analysis across all chains

## See Also

- [Document Flows](../configuration/document-flows.md) — P2P and O2C configuration
- [Subledgers](../configuration/subledgers.md) — AR, AP, FA, Inventory detail
- [FX & Currency](../configuration/fx-currency.md) — Multi-currency and translation
- [Generation Pipeline](generation-pipeline.md) — How the orchestrator sequences generators
- [Roadmap](../roadmap/README.md) — Future development plans
