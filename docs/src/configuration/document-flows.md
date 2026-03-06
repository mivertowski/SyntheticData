# Document Flows

Document flow settings control P2P (Procure-to-Pay) and O2C (Order-to-Cash) process generation, including document types, three-way matching, credit checks, and document chain management.

## Configuration

```yaml
document_flows:
  p2p:
    enabled: true
    flow_rate: 0.3
    completion_rate: 0.95
    three_way_match:
      quantity_tolerance: 0.02
      price_tolerance: 0.01

  o2c:
    enabled: true
    flow_rate: 0.3
    completion_rate: 0.95
```

---

## Procure-to-Pay (P2P)

### Flow

```
Purchase     Purchase    Goods      Vendor     Three-Way
Requisition → Order   → Receipt  → Invoice  → Match    → Payment
                │                     │          │
                │                ┌────┘          │
                ▼                ▼               ▼
           AP Open Item ← Match Result      AP Aging
```

### Purchase Order Types

DataSynth models 6 PO types, each with different downstream behavior:

| Type | Description | Requires GR? | Use Case |
|------|-------------|-------------|----------|
| `Standard` | Standard goods purchase | Yes | Most common PO type |
| `Service` | Service procurement | No | Consulting, maintenance, etc. |
| `Framework` | Blanket/framework agreement | Yes | Long-term supply agreements |
| `Consignment` | Vendor-managed inventory | Yes | Consignment stock |
| `StockTransfer` | Inter-plant transfer | Yes | Internal stock movement |
| `Subcontracting` | External processing | Yes | Outsourced manufacturing |

### Goods Receipt Movement Types

Goods receipts use SAP-style movement type codes:

| Movement Type | Code | Description |
|---------------|------|-------------|
| `GrForPo` | 101 | Standard GR against purchase order |
| `ReturnToVendor` | 122 | Return materials to vendor |
| `GrForProduction` | 131 | GR from production order |
| `TransferPosting` | 301 | Transfer between plants/locations |
| `InitialEntry` | 561 | Initial stock entry |
| `Scrapping` | 551 | Scrap disposal |
| `Consumption` | 201 | Direct consumption posting |

### Three-Way Match

The three-way match validator compares Purchase Order, Goods Receipt, and Vendor Invoice to detect variances before payment.

#### Algorithm

```
For each invoice line item:
  1. Find matching PO line (by PO reference + line number)
  2. Sum GR quantities for that PO line (supports multiple partial GRs)
  3. Compare:
     a. PO quantity vs GR quantity → QuantityPoGr variance
     b. GR quantity vs Invoice quantity → QuantityGrInvoice variance
     c. PO unit price vs Invoice unit price → PricePoInvoice variance
     d. PO total vs Invoice total → TotalAmount variance
  4. Apply tolerances:
     - Quantity: ±quantity_tolerance (default 2%)
     - Price: ±price_tolerance (default 5%)
     - Absolute: ±absolute_amount_tolerance (default $0.01)
  5. Check over-delivery:
     - If GR qty > PO qty and allow_over_delivery=true:
       allow up to max_over_delivery_pct (default 10%)
```

#### Variance Types

| Variance Type | Description | Detection |
|---------------|-------------|-----------|
| `QuantityPoGr` | GR quantity differs from PO quantity | PO vs GR comparison |
| `QuantityGrInvoice` | Invoice quantity differs from GR quantity | GR vs Invoice comparison |
| `PricePoInvoice` | Invoice unit price differs from PO price | PO vs Invoice comparison |
| `TotalAmount` | Total invoice amount mismatch | Overall amount check |
| `MissingLine` | PO line not found in invoice or GR | Line matching |
| `ExtraLine` | Invoice has lines not on PO | Line matching |

#### Match Outcomes

| Outcome | Meaning | Action |
|---------|---------|--------|
| `passed` | All within tolerance | Proceed to payment |
| `quantity_variance` | Quantity outside tolerance | Review required |
| `price_variance` | Price outside tolerance | Review required |
| `blocked` | Multiple variances or critical mismatch | Manual resolution |

#### Configuration

```yaml
document_flows:
  p2p:
    three_way_match:
      enabled: true
      price_tolerance: 0.05              # 5% price variance allowed
      quantity_tolerance: 0.02            # 2% quantity variance allowed
      absolute_amount_tolerance: 0.01     # $0.01 rounding tolerance
      allow_over_delivery: true
      max_over_delivery_pct: 0.10         # 10% over-delivery allowed
```

### P2P Stage Configuration

```yaml
document_flows:
  p2p:
    enabled: true
    flow_rate: 0.3                       # 30% of JEs from P2P
    completion_rate: 0.95                # 95% complete full flow

    stages:
      po_approval_rate: 0.9             # 90% of POs approved
      gr_rate: 0.98                     # 98% of POs get goods receipts
      invoice_rate: 0.95                # 95% of GRs get invoices
      payment_rate: 0.92                # 92% of invoices get paid

    timing:
      po_to_gr_days:
        min: 1
        max: 30
      gr_to_invoice_days:
        min: 1
        max: 14
      invoice_to_payment_days:
        min: 10
        max: 60
```

### P2P Journal Entries

| Stage | Debit | Credit | Trigger |
|-------|-------|--------|---------|
| Goods Receipt | Inventory (1300) | GR/IR Clearing (2100) | GR posted |
| Invoice Receipt | GR/IR Clearing (2100) | Accounts Payable (2000) | Invoice verified |
| Payment | Accounts Payable (2000) | Cash (1000) | Payment executed |
| Price Variance | PPV Expense (5xxx) | GR/IR Clearing (2100) | Price mismatch |

---

## Order-to-Cash (O2C)

### Flow

```
Sales     Credit   Delivery    Customer    Customer
Order   → Check  → (Pick/   → Invoice   → Receipt
  │                Pack/         │           │
  │                Ship)         │           │
  │                  │           ▼           ▼
  │                  │      AR Open Item   AR Aging
  │                  │           │
  │                  │           └→ Dunning (if overdue)
  │                  ▼
  │            Inventory Issue
  │            (COGS posting)
  ▼
Revenue Recognition
(ASC 606 / IFRS 15)
```

### Sales Order Types

DataSynth models 9 SO types:

| Type | Description | Requires Delivery? |
|------|-------------|-------------------|
| `Standard` | Standard sales order | Yes |
| `Rush` | Priority/expedited order | Yes |
| `CashSale` | Immediate payment at sale | Yes |
| `Return` | Customer return order | No (creates return delivery) |
| `FreeOfCharge` | No-charge delivery (samples, warranty) | Yes |
| `Consignment` | Consignment fill-up/issue | Yes |
| `Service` | Service order (no physical delivery) | No |
| `CreditMemoRequest` | Request for credit memo | No |
| `DebitMemoRequest` | Request for debit memo | No |

### Delivery Types

6 delivery types model different fulfillment scenarios:

| Type | Description | Direction |
|------|-------------|-----------|
| `Outbound` | Standard outbound delivery | Ship to customer |
| `Return` | Customer return delivery | Receive from customer |
| `StockTransfer` | Inter-plant stock transfer | Internal movement |
| `Replenishment` | Replenishment delivery | Warehouse → store |
| `ConsignmentIssue` | Issue from consignment stock | Consignment → customer |
| `ConsignmentReturn` | Return to consignment stock | Customer → consignment |

### Customer Invoice Types

7 invoice types with different accounting treatment:

| Type | Description | AR Impact |
|------|-------------|-----------|
| `Standard` | Normal sales invoice | Creates receivable |
| `CreditMemo` | Credit for returns/adjustments | Reduces receivable |
| `DebitMemo` | Additional charge | Increases receivable |
| `ProForma` | Pre-delivery invoice (no posting) | None |
| `DownPaymentRequest` | Advance payment request | Creates special receivable |
| `FinalInvoice` | Settles down payment | Clears down payment |
| `Intercompany` | IC billing | Creates IC receivable |

### Credit Check

Sales orders pass through credit verification before delivery:

```yaml
document_flows:
  o2c:
    credit_check:
      enabled: true
      check_credit_limit: true          # Verify customer limit
      check_overdue: true               # Check for past-due AR
      block_threshold: 0.9              # Block if >90% of limit used
```

### O2C Stage Configuration

```yaml
document_flows:
  o2c:
    enabled: true
    flow_rate: 0.3                       # 30% of JEs from O2C
    completion_rate: 0.95                # 95% complete full flow

    stages:
      so_approval_rate: 0.95            # 95% of SOs approved
      credit_check_pass_rate: 0.9       # 90% pass credit check
      delivery_rate: 0.98               # 98% of SOs get deliveries
      invoice_rate: 0.95                # 95% of deliveries get invoices
      collection_rate: 0.85             # 85% of invoices collected

    timing:
      so_to_delivery_days:
        min: 1
        max: 14
      delivery_to_invoice_days:
        min: 0
        max: 3
      invoice_to_payment_days:
        min: 15
        max: 90
```

### O2C Journal Entries

| Stage | Debit | Credit | Trigger |
|-------|-------|--------|---------|
| Delivery | Cost of Goods Sold (5000) | Inventory (1300) | Goods issued |
| Invoice | Accounts Receivable (1100) | Revenue (4000) | Invoice posted |
| Receipt | Cash (1000) | Accounts Receivable (1100) | Payment received |
| Credit Memo | Revenue (4000) | Accounts Receivable (1100) | Credit issued |

---

## Document Chain Manager

The document chain manager maintains referential integrity across the complete document flow by tracking references between documents.

### Reference Types

| Type | Description | Example |
|------|-------------|---------|
| `FollowOn` | Next document in normal flow | PO → GR → Invoice → Payment |
| `Payment` | Payment for invoice | PAY-001 → INV-001 |
| `Reversal` | Correction or reversal document | CRED-001 → INV-001 |
| `Partial` | Partial fulfillment | GR-001 (partial) → PO-001 |
| `CreditMemo` | Credit against invoice | CM-001 → INV-001 |
| `DebitMemo` | Debit against invoice | DM-001 → INV-001 |
| `Return` | Return against delivery | RET-001 → DEL-001 |
| `IntercompanyMatch` | IC matched pair | IC-INV-001 → IC-INV-002 |
| `Manual` | User-defined reference | Any → Any |

### Document Chain Output

```
PO-001 ─→ GR-001 ─→ INV-001 ─→ PAY-001
   │          │          │          │
   └──────────┴──────────┴──────────┘
              Document Chain
```

The `document_references.csv` output file records all links:

| Field | Description |
|-------|-------------|
| `source_document_id` | Referencing document |
| `target_document_id` | Referenced document |
| `reference_type` | Type of reference |
| `created_date` | Date reference was created |

---

## Complex Scenario Examples

### Partial Deliveries with Split Invoice

```yaml
document_flows:
  p2p:
    enabled: true
    flow_rate: 0.4
    completion_rate: 0.90           # 10% incomplete (partial deliveries)
    three_way_match:
      quantity_tolerance: 0.05      # 5% tolerance for partials
      allow_over_delivery: true
      max_over_delivery_pct: 0.10
    timing:
      po_to_gr_days: { min: 3, max: 45 }    # Longer lead times
      gr_to_invoice_days: { min: 1, max: 21 }
      invoice_to_payment_days: { min: 30, max: 90 }
```

### High-Volume Retail O2C

```yaml
document_flows:
  o2c:
    enabled: true
    flow_rate: 0.5                  # 50% of JEs from O2C
    completion_rate: 0.98           # High completion rate
    stages:
      so_approval_rate: 0.99       # Auto-approved
      credit_check_pass_rate: 0.95
      delivery_rate: 0.99
      invoice_rate: 0.99
      collection_rate: 0.92
    timing:
      so_to_delivery_days: { min: 0, max: 3 }     # Fast fulfillment
      delivery_to_invoice_days: { min: 0, max: 0 } # Immediate invoice
      invoice_to_payment_days: { min: 10, max: 45 }
```

### Combined Manufacturing P2P + O2C

```yaml
document_flows:
  p2p:
    enabled: true
    flow_rate: 0.35
    completion_rate: 0.95
    three_way_match:
      quantity_tolerance: 0.02
      price_tolerance: 0.01
    timing:
      po_to_gr_days: { min: 5, max: 30 }
      gr_to_invoice_days: { min: 1, max: 10 }
      invoice_to_payment_days: { min: 20, max: 45 }

  o2c:
    enabled: true
    flow_rate: 0.35
    completion_rate: 0.90
    credit_check:
      enabled: true
      block_threshold: 0.85
    timing:
      so_to_delivery_days: { min: 3, max: 21 }
      delivery_to_invoice_days: { min: 0, max: 2 }
      invoice_to_payment_days: { min: 30, max: 60 }
```

## Validation

| Check | Rule |
|-------|------|
| `flow_rate` | 0.0 - 1.0 |
| `completion_rate` | 0.0 - 1.0 |
| `tolerance` values | 0.0 - 1.0 |
| `timing.min` | ≥ 0 |
| `timing.max` | ≥ min |
| Stage rates | 0.0 - 1.0 |

## See Also

- [Subledgers](subledgers.md) — AR/AP records generated by document flows
- [FX & Currency](fx-currency.md) — Multi-currency document flows
- [Master Data](master-data.md) — Vendor and customer master records
- [Process Chains](../architecture/process-chains.md) — Enterprise process chain architecture
- [Process Mining](../use-cases/process-mining.md) — OCEL 2.0 event logs from document flows
- [datasynth-generators](../crates/datasynth-generators.md) — Generator crate reference
