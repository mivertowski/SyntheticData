# Process Mining

Generate OCEL 2.0 event logs for process mining analysis across 8 enterprise process families.

## Overview

DataSynth generates comprehensive process mining data:

- OCEL 2.0 compliant event logs with **88 activity types** and **52 object types**
- **8 process families**: P2P, O2C, S2C, H2R, MFG, BANK, AUDIT, Bank Recon
- Object-centric relationships with lifecycle states
- Three variant types per generator: HappyPath (75%), ExceptionPath (20%), ErrorPath (5%)
- Cross-process object linking via shared document IDs

## Configuration

```yaml
global:
  seed: 42
  industry: manufacturing
  start_date: 2024-01-01
  period_months: 6

transactions:
  target_count: 50000

document_flows:
  p2p:
    enabled: true
    flow_rate: 0.4
    completion_rate: 0.95

    stages:
      po_approval_rate: 0.9
      gr_rate: 0.98
      invoice_rate: 0.95
      payment_rate: 0.92

  o2c:
    enabled: true
    flow_rate: 0.4
    completion_rate: 0.90

    stages:
      so_approval_rate: 0.95
      credit_check_pass_rate: 0.9
      delivery_rate: 0.98
      invoice_rate: 0.95
      collection_rate: 0.85

master_data:
  vendors:
    count: 100
  customers:
    count: 200
  materials:
    count: 500
  employees:
    count: 30

output:
  format: csv
```

## OCEL 2.0 Export

Use the `datasynth-ocpm` crate for OCEL 2.0 export:

```rust
use synth_ocpm::{OcpmGenerator, Ocel2Exporter, ExportFormat};

let mut generator = OcpmGenerator::new(seed);
let event_log = generator.generate_event_log(
    p2p_count: 5000,
    o2c_count: 5000,
    start_date,
    end_date,
)?;

let exporter = Ocel2Exporter::new(ExportFormat::Json);
exporter.export(&event_log, "output/ocel2.json")?;
```

## P2P Process

### Event Sequence

```
Create PO → Approve PO → Release PO → Create GR → Post GR →
Receive Invoice → Verify Invoice → Post Invoice → Execute Payment
```

### Objects

| Object Type | Attributes |
|-------------|------------|
| PurchaseOrder | po_number, vendor_id, total_amount |
| GoodsReceipt | gr_number, po_reference, quantity |
| VendorInvoice | invoice_number, amount, due_date |
| Payment | payment_number, amount, bank_ref |
| Material | material_id, description |
| Vendor | vendor_id, name |

### Object Relationships

```
PurchaseOrder ─┬── contains ──→ Material
               └── from ──────→ Vendor

GoodsReceipt ──── for ──────→ PurchaseOrder

VendorInvoice ─── for ──────→ PurchaseOrder
               └── matches ──→ GoodsReceipt

Payment ───────── pays ──────→ VendorInvoice
```

## O2C Process

### Event Sequence

```
Create SO → Check Credit → Release SO → Create Delivery →
Pick → Pack → Ship → Create Invoice → Post Invoice → Receive Payment
```

### Objects

| Object Type | Attributes |
|-------------|------------|
| SalesOrder | so_number, customer_id, total_amount |
| Delivery | delivery_number, so_reference |
| CustomerInvoice | invoice_number, amount, due_date |
| CustomerPayment | receipt_number, amount |
| Material | material_id, description |
| Customer | customer_id, name |

## Analysis with PM4Py

### Load Event Log

```python
from pm4py.objects.ocel.importer import jsonocel

# Load OCEL 2.0
ocel = jsonocel.apply("output/ocel2.json")

print(f"Events: {len(ocel.events)}")
print(f"Objects: {len(ocel.objects)}")
print(f"Object types: {ocel.object_types}")
```

### Process Discovery

```python
from pm4py.algo.discovery.ocel import algorithm as ocel_discovery

# Discover object-centric Petri net
ocpn = ocel_discovery.apply(ocel)

# Visualize
from pm4py.visualization.ocel.ocpn import visualizer
gviz = visualizer.apply(ocpn)
visualizer.save(gviz, "ocpn.png")
```

### Object Lifecycle Analysis

```python
from pm4py.statistics.ocel import object_lifecycle

# Analyze PurchaseOrder lifecycle
po_lifecycle = object_lifecycle.get_lifecycle_summary(
    ocel,
    object_type="PurchaseOrder"
)

print("Purchase Order Lifecycle:")
print(f"  Average duration: {po_lifecycle['avg_duration']}")
print(f"  Completion rate: {po_lifecycle['completion_rate']:.2%}")
```

### Conformance Checking

```python
from pm4py.algo.conformance.ocel import algorithm as ocel_conformance

# Check conformance against expected model
results = ocel_conformance.apply(ocel, ocpn)

print(f"Conformant cases: {results['conformant']}")
print(f"Non-conformant: {results['non_conformant']}")
```

## Process Metrics

### Throughput Time

```python
import pandas as pd
from datetime import timedelta

# Load events
events = pd.DataFrame(ocel.events)

# Calculate case durations
case_durations = events.groupby('case_id').agg({
    'timestamp': ['min', 'max']
})
case_durations['duration'] = (
    case_durations[('timestamp', 'max')] -
    case_durations[('timestamp', 'min')]
)

print(f"Mean throughput time: {case_durations['duration'].mean()}")
print(f"Median throughput time: {case_durations['duration'].median()}")
```

### Activity Frequency

```python
# Count activity occurrences
activity_counts = events['activity'].value_counts()
print("Activity frequency:")
print(activity_counts)
```

### Bottleneck Analysis

```python
# Calculate waiting times between activities
events = events.sort_values(['case_id', 'timestamp'])
events['wait_time'] = events.groupby('case_id')['timestamp'].diff()

# Find bottlenecks
bottlenecks = events.groupby('activity')['wait_time'].mean().sort_values(ascending=False)
print("Bottleneck activities:")
print(bottlenecks.head(5))
```

## Variant Analysis

```python
from pm4py.algo.discovery.ocel import variants

# Get process variants
variant_stats = variants.get_variants_statistics(ocel)

print(f"Unique variants: {len(variant_stats)}")
print("\nTop variants:")
for variant, stats in sorted(variant_stats.items(), key=lambda x: -x[1]['count'])[:5]:
    print(f"  {variant}: {stats['count']} cases")
```

## Integration with Tools

### Celonis

```python
# Export to Celonis format
from pm4py.objects.ocel.exporter import csv as ocel_csv_exporter

ocel_csv_exporter.apply(ocel, "output/celonis/")
# Upload CSV files to Celonis
```

### OCPA

```python
# Export to OCPA format
from pm4py.objects.ocel.exporter import sqlite

sqlite.apply(ocel, "output/ocel.sqlite")
# Open in OCPA tool
```

## New Process Families (v0.6.2)

### S2C — Source-to-Contract

```
Create Sourcing Project → Qualify Supplier → Publish RFx →
Submit Bid → Evaluate Bids → Award Contract →
Activate Contract → Complete Sourcing
```

### H2R — Hire-to-Retire

```
Submit Time Entry → Approve Time Entry →
Create Payroll Run → Calculate Payroll → Approve Payroll → Post Payroll
Submit Expense → Approve Expense
```

### MFG — Manufacturing

```
Create Production Order → Release → Start Operation →
Complete Operation → Quality Inspection → Confirm Production →
Close Production Order
```

### BANK — Banking Operations

```
Onboard Customer → KYC Review → Open Account →
Execute Transaction → Authorize → Complete Transaction
```

### AUDIT — Audit Engagement Lifecycle

```
Create Engagement → Plan → Assess Risk → Create Workpaper →
Collect Evidence → Review Workpaper → Raise Finding →
Remediate Finding → Record Judgment → Complete Engagement
```

### Bank Recon — Bank Reconciliation

```
Import Bank Statement → Auto Match Items → Manual Match Item →
Create Reconciling Item → Resolve Exception →
Approve Reconciliation → Post Entries → Complete Reconciliation
```

## S2P Process Mining

The full Source-to-Pay chain provides rich process mining opportunities beyond basic P2P:

### Extended Event Sequence

```
Spend Analysis → Supplier Qualification → RFx Published →
Bid Received → Bid Evaluation → Contract Award →
Create PO → Approve PO → Release PO →
Create GR → Post GR →
Receive Invoice → Verify Invoice (Three-Way Match) → Post Invoice →
Schedule Payment → Execute Payment
```

### Extended Object Types

| Object Type | Attributes |
|-------------|------------|
| SpendCategory | category_code, total_spend, vendor_count |
| SourcingProject | project_type, target_savings, status |
| SupplierBid | vendor_id, bid_amount, technical_score |
| ProcurementContract | contract_value, validity_period, terms |
| PurchaseRequisition | requester, catalog_item, urgency |
| PurchaseOrder | po_type, vendor_id, total_amount |
| GoodsReceipt | gr_number, received_qty, movement_type |
| VendorInvoice | invoice_amount, match_status, due_date |
| Payment | payment_method, cleared_amount, bank_ref |

### Cycle Time Analysis

```python
# Analyze end-to-end procurement cycle times
po_events = events[events['object_type'] == 'PurchaseOrder']

# PO creation to payment completion
cycle_times = po_events.groupby('case_id').agg({
    'timestamp': ['min', 'max']
})
cycle_times['cycle_time'] = (
    cycle_times[('timestamp', 'max')] -
    cycle_times[('timestamp', 'min')]
)

# Segment by PO type
cycle_by_type = po_events.merge(
    objects[['po_type']], on='object_id'
).groupby('po_type')['cycle_time'].describe()
```

### Three-Way Match Conformance

```python
# Identify invoices that failed three-way match
match_events = events[events['activity'] == 'Verify Invoice']
blocked = match_events[match_events['match_status'] == 'blocked']

print(f"Three-way match block rate: {len(blocked)/len(match_events):.1%}")
print(f"Most common variance: {blocked['variance_type'].mode()[0]}")
```

## See Also

- [Document Flows](../configuration/document-flows.md) — P2P and O2C configuration
- [Process Chains](../architecture/process-chains.md) — Enterprise process chain architecture
- [datasynth-ocpm Crate](../crates/datasynth-ocpm.md) — OCEL 2.0 implementation
- [Audit Analytics](audit-analytics.md)
