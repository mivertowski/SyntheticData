# datasynth-ocpm

Object-Centric Process Mining (OCPM) models and generators.

## Overview

`datasynth-ocpm` provides OCEL 2.0 compliant event log generation across 8 enterprise process families:

- **OCEL 2.0 Models**: Events, objects, relationships per IEEE standard
- **8 Process Generators**: P2P, O2C, S2C, H2R, MFG, BANK, AUDIT, Bank Recon
- **88 Activity Types**: Covering the full enterprise lifecycle
- **52 Object Types**: With lifecycle states and inter-object relationships
- **Export Formats**: OCEL 2.0 JSON, XML, and SQLite

## OCEL 2.0 Standard

Implements the Object-Centric Event Log standard:

| Element | Description |
|---------|-------------|
| Events | Activities with timestamps and attributes |
| Objects | Business objects (PO, Invoice, Payment, etc.) |
| Object Types | Type definitions with attribute schemas |
| Relationships | Object-to-object relationships |
| Event-Object Links | Many-to-many event-object associations |

## Key Types

### OCEL Models

```rust
pub struct OcelEventLog {
    pub object_types: Vec<ObjectType>,
    pub event_types: Vec<EventType>,
    pub objects: Vec<Object>,
    pub events: Vec<Event>,
    pub relationships: Vec<ObjectRelationship>,
}

pub struct Event {
    pub id: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, Value>,
    pub objects: Vec<ObjectRef>,
}

pub struct Object {
    pub id: String,
    pub object_type: String,
    pub attributes: HashMap<String, Value>,
}
```

### Process Flow Documents

```rust
pub struct P2pDocuments {
    pub po_number: String,
    pub vendor_id: String,
    pub company_code: String,
    pub amount: Decimal,
    pub currency: String,
}

pub struct O2cDocuments {
    pub so_number: String,
    pub customer_id: String,
    pub company_code: String,
    pub amount: Decimal,
    pub currency: String,
}
```

## Process Flows

### Procure-to-Pay (P2P)

```
Create PO → Approve PO → Release PO → Create GR → Post GR →
Receive Invoice → Verify Invoice → Post Invoice → Execute Payment
```

Events generated:
- `Create Purchase Order`
- `Approve Purchase Order`
- `Release Purchase Order`
- `Create Goods Receipt`
- `Post Goods Receipt`
- `Receive Vendor Invoice`
- `Verify Three-Way Match`
- `Post Vendor Invoice`
- `Execute Payment`

### Order-to-Cash (O2C)

```
Create SO → Check Credit → Release SO → Create Delivery →
Pick → Pack → Ship → Create Invoice → Post Invoice → Receive Payment
```

Events generated:
- `Create Sales Order`
- `Check Credit`
- `Release Sales Order`
- `Create Delivery`
- `Pick Materials`
- `Pack Shipment`
- `Ship Goods`
- `Create Customer Invoice`
- `Post Customer Invoice`
- `Receive Customer Payment`

## Usage Examples

### Generate P2P Case

```rust
use synth_ocpm::{OcpmGenerator, P2pDocuments};

let mut generator = OcpmGenerator::new(seed);

let documents = P2pDocuments::new(
    "PO-001",
    "V-001",
    "1000",
    dec!(5000.00),
    "USD",
);

let users = vec!["user1", "user2", "user3"];
let start_time = Utc::now();

let result = generator.generate_p2p_case(&documents, start_time, &users);
```

### Generate O2C Case

```rust
use synth_ocpm::{OcpmGenerator, O2cDocuments};

let documents = O2cDocuments::new(
    "SO-001",
    "C-001",
    "1000",
    dec!(10000.00),
    "USD",
);

let result = generator.generate_o2c_case(&documents, start_time, &users);
```

### Generate Complete Event Log

```rust
use synth_ocpm::OcpmGenerator;

let mut generator = OcpmGenerator::new(seed);
let event_log = generator.generate_event_log(
    p2p_count: 1000,
    o2c_count: 500,
    start_date,
    end_date,
)?;
```

## Export Formats

### OCEL 2.0 JSON

```rust
use synth_ocpm::export::{Ocel2Exporter, ExportFormat};

let exporter = Ocel2Exporter::new(ExportFormat::Json);
exporter.export(&event_log, "output/ocel2.json")?;
```

Output structure:
```json
{
  "objectTypes": [...],
  "eventTypes": [...],
  "objects": [...],
  "events": [...],
  "relations": [...]
}
```

### OCEL 2.0 XML

```rust
let exporter = Ocel2Exporter::new(ExportFormat::Xml);
exporter.export(&event_log, "output/ocel2.xml")?;
```

### SQLite Database

```rust
let exporter = Ocel2Exporter::new(ExportFormat::Sqlite);
exporter.export(&event_log, "output/ocel2.sqlite")?;
```

Tables created:
- `object_types`
- `event_types`
- `objects`
- `events`
- `event_objects`
- `object_relationships`

## Process Families (v0.6.2)

| Family | Generator | Activities | Object Types | Variants |
|--------|-----------|-----------|-------------|----------|
| **P2P** | `generate_p2p_case()` | 9 | PurchaseOrder, GoodsReceipt, VendorInvoice, Payment, Material, Vendor | Happy, Exception, Error |
| **O2C** | `generate_o2c_case()` | 10 | SalesOrder, Delivery, CustomerInvoice, CustomerPayment, Material, Customer | Happy, Exception, Error |
| **S2C** | `generate_s2c_case()` | 8 | SourcingProject, SupplierQualification, RfxEvent, SupplierBid, BidEvaluation, ProcurementContract | Happy, Exception, Error |
| **H2R** | `generate_h2r_case()` | 8 | PayrollRun, PayrollLineItem, TimeEntry, ExpenseReport | Happy, Exception, Error |
| **MFG** | `generate_mfg_case()` | 10 | ProductionOrder, RoutingOperation, QualityInspection, CycleCount | Happy, Exception, Error |
| **BANK** | `generate_bank_case()` | 8 | BankingCustomer, BankAccount, BankTransaction | Happy, Exception, Error |
| **AUDIT** | `generate_audit_case()` | 10 | AuditEngagement, Workpaper, AuditFinding, AuditEvidence, RiskAssessment, ProfessionalJudgment | Happy, Exception, Error |
| **Bank Recon** | `generate_bank_recon_case()` | 8 | BankReconciliation, BankStatementLine, ReconcilingItem | Happy, Exception, Error |

Variant distribution: HappyPath (75%), ExceptionPath (20%), ErrorPath (5%).

## Object Types (P2P/O2C)

| Type | Description |
|------|-------------|
| PurchaseOrder | P2P ordering document |
| GoodsReceipt | Inventory receipt |
| VendorInvoice | AP invoice |
| Payment | Payment document |
| SalesOrder | O2C ordering document |
| Delivery | Shipment document |
| CustomerInvoice | AR invoice |
| CustomerPayment | Customer receipt |
| Material | Product/item |
| Vendor | Supplier |
| Customer | Customer/buyer |

## Integration with Process Mining Tools

OCEL 2.0 exports are compatible with:
- **PM4Py**: Python process mining library
- **Celonis**: Enterprise process mining platform
- **PROM**: Academic process mining toolkit
- **OCPA**: Object-centric process analysis tool

### Loading in PM4Py

```python
import pm4py
from pm4py.objects.ocel.importer import jsonocel

ocel = jsonocel.apply("ocel2.json")
print(f"Events: {len(ocel.events)}")
print(f"Objects: {len(ocel.objects)}")
```

## See Also

- [Process Mining Use Case](../use-cases/process-mining.md)
- [Document Flows](../configuration/document-flows.md)
- [datasynth-generators](datasynth-generators.md)
