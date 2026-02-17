# Enterprise Process Chains

SyntheticData models enterprise operations as interconnected process chains — end-to-end business flows that share master data, generate journal entries, and link through common documents. This page maps the current implementation status and shows how the chains integrate.

## Coverage Matrix

| Chain | Full Name | Coverage | Status | Key Modules |
|-------|-----------|----------|--------|-------------|
| **S2P** | Source-to-Pay | 95% | Implemented (P2P + S2C + OCPM) | `document_flow/p2p_generator`, `sourcing/`, `ocpm/s2c_generator` |
| **O2C** | Order-to-Cash | 95% | Implemented (+ OCPM) | `document_flow/o2c_generator`, `master_data/customer`, `subledger/ar` |
| **R2R** | Record-to-Report | 85% | Implemented (+ Bank Recon OCPM) | `je_generator`, `balance/`, `period_close/`, `ocpm/bank_recon_generator` |
| **A2R** | Acquire-to-Retire | 70% | Partially implemented | `master_data/asset`, `subledger/fa`, `period_close/depreciation` |
| **INV** | Inventory Management | 55% | Partially implemented | `subledger/inventory`, `document_flow/` (GR/delivery links) |
| **BANK** | Banking & Treasury | 85% | Implemented (+ OCPM) | `datasynth-banking`, `ocpm/bank_generator` |
| **H2R** | Hire-to-Retire | 85% | Implemented (+ OCPM) | `hr/`, `master_data/employee`, `ocpm/h2r_generator` |
| **MFG** | Plan-to-Produce | 85% | Implemented (+ OCPM) | `manufacturing/`, `ocpm/mfg_generator` |
| **AUDIT** | Audit Lifecycle | 90% | Implemented (+ OCPM) | `audit/`, `ocpm/audit_generator` |
| **TAX** | Tax Accounting | 90% | Implemented (v0.7.0) | `tax/`, `ocpm/tax` |
| **TREASURY** | Treasury & Cash Mgmt | 90% | Implemented (v0.7.0) | `treasury/`, `ocpm/treasury` |
| **PROJECT** | Project Accounting | 90% | Implemented (v0.7.0) | `project_accounting/`, `ocpm/project_accounting` |
| **ESG** | ESG / Sustainability | 90% | Implemented (v0.7.0) | `esg/`, `ocpm/esg` |

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

### Banking & Treasury (BANK) — 85%

**Implemented:** Bank customer profiles, KYC/AML, bank accounts, transactions with fraud typologies (structuring, funnel, layering, mule, round-tripping). OCPM events for customer onboarding, KYC review, account management, and transaction lifecycle.

**Gaps:** Cash forecasting, liquidity management.

### Hire-to-Retire (H2R) — 85%

**Implemented:** Employee master data, payroll runs with tax/deduction calculations, time entries with overtime, expense reports with policy violations. OCPM events for payroll lifecycle, time entry approval, and expense approval chains.

**Gaps:** Benefits administration, workforce planning.

### Plan-to-Produce (MFG) — 85%

**Implemented:** Production orders with BOM explosion, routing operations, WIP costing, quality inspections, cycle counting. OCPM events for production order lifecycle, quality inspection, and cycle count reconciliation.

**Gaps:** Material requirements planning (MRP), advanced shop floor control.

### Audit Lifecycle (AUDIT) — 90%

**Implemented:** Engagement planning, risk assessment (ISA 315/330), workpaper creation and review (ISA 230), evidence collection (ISA 500), findings (ISA 265), professional judgment documentation (ISA 200). OCPM events for the full engagement lifecycle.

**Gaps:** Multi-engagement portfolio management.

## Partially Implemented Chains

### Acquire-to-Retire (A2R) — 70%

**Implemented:** Fixed asset master data, depreciation (6 methods), acquisition from PO, disposal with gain/loss accounting, impairment testing (ASC 360/IAS 36).

**Gaps:** Capital project/WBS integration, asset transfers between companies, construction-in-progress (CIP) tracking.

### Inventory Management (INV) — 55%

**Implemented:** Inventory positions, 22 movement types, 4 valuation methods, stock status tracking, P2P goods receipts, O2C goods issues.

**Gaps:** Quality inspection integration, obsolescence management, ABC analysis.

---

## Cross-Process Integration

Process chains share data through several integration points, now with full OCPM event coverage:

```
    S2C ──→ S2P                    O2C                    R2R
    │        │                      │                      │
    Contract GR ──── Inventory ─────┼── Delivery           │
             │         │            │                      │
       Payment ────────┼────────────┼── Receipt ──── Bank Recon
             │         │            │                  │   │
       AP Open Item    │       AR Open Item         BANK  │
             │     MFG─┘            │                 │   │
             └──H2R──┴──────────────┴──── Journal Entries ┘
                  │                                   │
              AUDIT ─────────────────────────── Trial Balance
                                                      │
                                               Consolidation

    ──── All chains feed OCEL 2.0 Event Log (88 activities) ────
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
