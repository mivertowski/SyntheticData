# Graph Export

Export transaction data as ML-ready graphs.

## Overview

Graph export transforms financial data into network representations:

- **Accounting Network** (GL accounts as nodes, transactions as edges) - *New in v0.2.1*
- Transaction networks (accounts and entities)
- Approval networks (users and approvals)
- Entity relationship graphs (ownership)

## Accounting Network Graph Export

The accounting network represents money flows between GL accounts, designed for **network reconstruction** and **anomaly detection** algorithms.

### Quick Start

```bash
# Generate with graph export enabled
datasynth-data generate --config config.yaml --output ./output --graph-export
```

### Graph Structure

| Element | Description |
|---------|-------------|
| **Nodes** | GL Accounts from Chart of Accounts |
| **Edges** | Money flows FROM credit accounts TO debit accounts |
| **Direction** | Directed graph (source→target) |

```
     ┌──────────────┐
     │ Credit Acct  │
     │   (2000)     │
     └──────┬───────┘
            │ $1,000
            ▼
     ┌──────────────┐
     │ Debit Acct   │
     │   (1100)     │
     └──────────────┘
```

### Edge Features (8 dimensions)

| Feature | Index | Description |
|---------|-------|-------------|
| `log_amount` | F0 | log10(transaction amount) |
| `benford_prob` | F1 | Expected first-digit probability |
| `weekday` | F2 | Day of week (normalized 0-1) |
| `period` | F3 | Fiscal period (normalized 0-1) |
| `is_month_end` | F4 | Last 3 days of month |
| `is_year_end` | F5 | Last month of year |
| `is_anomaly` | F6 | Anomaly flag (0 or 1) |
| `business_process` | F7 | Encoded business process |

### Output Files

```
output/graphs/accounting_network/pytorch_geometric/
├── edge_index.npy      # [2, E] source→target node indices
├── node_features.npy   # [N, 4] node feature vectors
├── edge_features.npy   # [E, 8] edge feature vectors
├── edge_labels.npy     # [E] anomaly labels (0=normal, 1=anomaly)
├── node_labels.npy     # [N] node labels
├── train_mask.npy      # [N] boolean training mask
├── val_mask.npy        # [N] boolean validation mask
├── test_mask.npy       # [N] boolean test mask
├── metadata.json       # Graph statistics and configuration
└── load_graph.py       # Auto-generated Python loader script
```

### Loading in Python

```python
import numpy as np
import json

# Load metadata
with open('metadata.json') as f:
    meta = json.load(f)
print(f"Nodes: {meta['num_nodes']}, Edges: {meta['num_edges']}")

# Load arrays
edge_index = np.load('edge_index.npy')      # [2, E]
node_features = np.load('node_features.npy') # [N, F]
edge_features = np.load('edge_features.npy') # [E, 8]
edge_labels = np.load('edge_labels.npy')     # [E]

# For PyTorch Geometric
import torch
from torch_geometric.data import Data

data = Data(
    x=torch.from_numpy(node_features).float(),
    edge_index=torch.from_numpy(edge_index).long(),
    edge_attr=torch.from_numpy(edge_features).float(),
    y=torch.from_numpy(edge_labels).long(),
)
```

### Configuration

```yaml
graph_export:
  enabled: true
  formats:
    - pytorch_geometric
  train_ratio: 0.7
  validation_ratio: 0.15
  # test_ratio is automatically 1 - train - val = 0.15
```

### Use Cases

1. **Anomaly Detection**: Train GNNs to detect anomalous transaction patterns
2. **Network Reconstruction**: Validate accounting network recovery algorithms
3. **Fraud Detection**: Identify suspicious money flow patterns
4. **Link Prediction**: Predict likely transaction relationships

## Configuration

```yaml
graph_export:
  enabled: true

  formats:
    - pytorch_geometric
    - neo4j
    - dgl

  graphs:
    - transaction_network
    - approval_network
    - entity_relationship

  split:
    train: 0.7
    val: 0.15
    test: 0.15
    stratify: is_anomaly

  features:
    temporal: true
    amount: true
    structural: true
    categorical: true
```

## Graph Types

### Transaction Network

Accounts and entities as nodes, transactions as edges.

```
     ┌──────────┐
     │ Account  │
     │  1100    │
     └────┬─────┘
          │ $1000
          ▼
     ┌──────────┐
     │ Customer │
     │  C-001   │
     └──────────┘
```

**Nodes:**
- GL accounts
- Vendors
- Customers
- Cost centers

**Edges:**
- Journal entry lines
- Payments
- Invoices

### Approval Network

Users as nodes, approval relationships as edges.

```
     ┌──────────┐
     │  Clerk   │
     │  U-001   │
     └────┬─────┘
          │ approved
          ▼
     ┌──────────┐
     │ Manager  │
     │  U-002   │
     └──────────┘
```

**Nodes:** Employees/users
**Edges:** Approval actions

### Entity Relationship Network

Legal entities with ownership relationships.

```
     ┌──────────┐
     │  Parent  │
     │  1000    │
     └────┬─────┘
          │ 100%
          ▼
     ┌──────────┐
     │   Sub    │
     │  2000    │
     └──────────┘
```

**Nodes:** Companies
**Edges:** Ownership, IC transactions

## Export Formats

### PyTorch Geometric

```
output/graphs/transaction_network/pytorch_geometric/
├── node_features.pt    # [num_nodes, num_features]
├── edge_index.pt       # [2, num_edges]
├── edge_attr.pt        # [num_edges, num_edge_features]
├── labels.pt           # Labels
├── train_mask.pt       # Boolean training mask
├── val_mask.pt         # Boolean validation mask
└── test_mask.pt        # Boolean test mask
```

**Loading in Python:**

```python
import torch
from torch_geometric.data import Data

# Load tensors
node_features = torch.load('node_features.pt')
edge_index = torch.load('edge_index.pt')
edge_attr = torch.load('edge_attr.pt')
labels = torch.load('labels.pt')
train_mask = torch.load('train_mask.pt')

# Create PyG Data object
data = Data(
    x=node_features,
    edge_index=edge_index,
    edge_attr=edge_attr,
    y=labels,
    train_mask=train_mask,
)

print(f"Nodes: {data.num_nodes}")
print(f"Edges: {data.num_edges}")
```

### Neo4j

```
output/graphs/transaction_network/neo4j/
├── nodes_account.csv
├── nodes_vendor.csv
├── nodes_customer.csv
├── edges_transaction.csv
├── edges_payment.csv
└── import.cypher
```

**Import script (import.cypher):**

```cypher
// Load accounts
LOAD CSV WITH HEADERS FROM 'file:///nodes_account.csv' AS row
CREATE (:Account {
    id: row.id,
    name: row.name,
    type: row.type
});

// Load transactions
LOAD CSV WITH HEADERS FROM 'file:///edges_transaction.csv' AS row
MATCH (from:Account {id: row.from_id})
MATCH (to:Account {id: row.to_id})
CREATE (from)-[:TRANSACTION {
    amount: toFloat(row.amount),
    date: date(row.posting_date),
    is_anomaly: toBoolean(row.is_anomaly)
}]->(to);
```

### DGL (Deep Graph Library)

```
output/graphs/transaction_network/dgl/
├── graph.bin           # Serialized DGL graph
├── node_feats.npy      # Node features
├── edge_feats.npy      # Edge features
└── labels.npy          # Labels
```

**Loading in Python:**

```python
import dgl
import numpy as np

# Load graph
graph = dgl.load_graphs('graph.bin')[0][0]

# Load features
graph.ndata['feat'] = torch.tensor(np.load('node_feats.npy'))
graph.edata['feat'] = torch.tensor(np.load('edge_feats.npy'))
graph.ndata['label'] = torch.tensor(np.load('labels.npy'))
```

## Features

### Temporal Features

```yaml
features:
  temporal: true
```

| Feature | Description |
|---------|-------------|
| `weekday` | Day of week (0-6) |
| `period` | Fiscal period (1-12) |
| `is_month_end` | Last 3 days of month |
| `is_quarter_end` | Last week of quarter |
| `is_year_end` | Last month of year |
| `hour` | Hour of posting |

### Amount Features

```yaml
features:
  amount: true
```

| Feature | Description |
|---------|-------------|
| `log_amount` | log10(amount) |
| `benford_prob` | Expected first-digit probability |
| `is_round_number` | Ends in 00, 000, etc. |
| `amount_zscore` | Standard deviations from mean |

### Structural Features

```yaml
features:
  structural: true
```

| Feature | Description |
|---------|-------------|
| `line_count` | Number of JE lines |
| `unique_accounts` | Distinct accounts used |
| `has_intercompany` | IC transaction flag |
| `debit_credit_ratio` | Total debits / credits |

### Categorical Features

```yaml
features:
  categorical: true
```

One-hot encoded:
- `business_process`: Manual, P2P, O2C, etc.
- `source_type`: System, User, Recurring
- `account_type`: Asset, Liability, etc.

## Train/Val/Test Splits

```yaml
split:
  train: 0.7                         # 70% training
  val: 0.15                          # 15% validation
  test: 0.15                         # 15% test
  stratify: is_anomaly               # Maintain anomaly ratio
  random_seed: 42                    # Reproducible splits
```

**Stratification options:**
- `is_anomaly`: Balanced anomaly detection
- `is_fraud`: Balanced fraud detection
- `account_type`: Balanced by account type
- `null`: Random (no stratification)

## GNN Training Example

```python
import torch
from torch_geometric.nn import GCNConv

class AnomalyGNN(torch.nn.Module):
    def __init__(self, num_features, hidden_dim):
        super().__init__()
        self.conv1 = GCNConv(num_features, hidden_dim)
        self.conv2 = GCNConv(hidden_dim, 2)  # Binary classification

    def forward(self, data):
        x, edge_index = data.x, data.edge_index
        x = self.conv1(x, edge_index).relu()
        x = self.conv2(x, edge_index)
        return x

# Train
model = AnomalyGNN(data.num_features, 64)
optimizer = torch.optim.Adam(model.parameters(), lr=0.01)

for epoch in range(100):
    model.train()
    optimizer.zero_grad()
    out = model(data)
    loss = F.cross_entropy(out[data.train_mask], data.y[data.train_mask])
    loss.backward()
    optimizer.step()
```

## Graph Property Mapping (v0.9.4)

The `ToNodeProperties` trait provides a standardized way to convert typed Rust model structs into graph node property maps with camelCase keys, suitable for Neo4j, AssureTwin, and other graph consumers.

### ToNodeProperties Trait

```rust
pub trait ToNodeProperties {
    fn node_type_name(&self) -> &'static str;  // e.g. "uncertain_tax_position"
    fn node_type_code(&self) -> u16;           // e.g. 416
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue>;
}
```

### GraphPropertyValue Enum

```rust
pub enum GraphPropertyValue {
    String(String),
    Int(i64),
    Float(f64),
    Decimal(Decimal),
    Bool(bool),
    Date(NaiveDate),
    StringList(Vec<String>),
}
```

### GraphNode::from_entity()

Bridge method for converting any `ToNodeProperties` implementor into a graph node:

```rust
let node = GraphNode::from_entity(node_id, &tax_return);
// node.properties contains all camelCase property keys
```

### Implemented Entity Types (51 types across 10 process families)

All model structs implement `ToNodeProperties`, mapping their fields to camelCase property keys. Boolean flags (`isApproved`, `isPassed`, `isActive`, `treatyApplied`, `billable`, etc.) are derived from status fields or probability-based generation for graph query convenience.

## Multi-Layer Hypergraph (v0.6.2+)

The RustGraph Hypergraph exporter supports all enterprise process families with 51 entity type codes:

### Entity Type Codes

| Range | Family | Types |
|-------|--------|-------|
| 100-106 | Core | Company, Vendor, Material, Customer, Employee, GlAccount |
| 300-303 | P2P | PurchaseOrder, GoodsReceipt, VendorInvoice, Payment |
| 310-312 | O2C | SalesOrder, Delivery, CustomerInvoice |
| 320-325 | S2C | SourcingProject, RfxEvent, SupplierBid, BidEvaluation, ProcurementContract, SupplierQualification |
| 330-333 | H2R | PayrollRun, TimeEntry, ExpenseReport, BenefitEnrollment |
| 340-345 | MFG | ProductionOrder, RoutingOperation, QualityInspection, CycleCount, BomComponent, InventoryMovement |
| 350-352 | BANK | BankingCustomer, BankAccount, BankTransaction |
| 360-365 | AUDIT | AuditEngagement, Workpaper, AuditFinding, AuditEvidence, RiskAssessment, ProfessionalJudgment |
| 370-372 | Bank Recon | BankReconciliation, BankStatementLine, ReconcilingItem |
| 400 | OCPM | OcpmEvent (events as hyperedges) |
| 410-416 | TAX | TaxJurisdiction, TaxCode, TaxLine, TaxReturn, TaxProvision, WithholdingTaxRecord, UncertainTaxPosition |
| 420-427 | Treasury | CashPosition, CashForecast, CashPool, CashPoolSweep, HedgingInstrument, HedgeRelationship, DebtInstrument, DebtCovenant |
| 430-442 | ESG | EmissionRecord, EnergyConsumption, WaterUsage, WasteRecord, WorkforceDiversityMetric, PayEquityMetric, SafetyIncident, SafetyMetric, GovernanceMetric, SupplierEsgAssessment, MaterialityAssessment, EsgDisclosure, ClimateScenario |
| 450-455 | Project | Project, ProjectCostLine, ProjectRevenue, EarnedValueMetric, ChangeOrder, ProjectMilestone |
| 500-504 | GOV | CosoComponent, CosoPrinciple, SoxAssertion, AuditEngagement, ProfessionalJudgment |
| 505-508 | Compliance | ComplianceStandard, Jurisdiction, RegulatoryFiling, ComplianceFinding |
| 510-513 | Compliance (ToNodeProperties) | ComplianceStandard, ComplianceFinding, RegulatoryFiling, JurisdictionProfile |

### Edge Type Registry (v0.9.4)

28 typed relationship variants with source→target entity constraints:

| Family | Edge Type | Source → Target |
|--------|-----------|-----------------|
| P2P | PlacedWith | PurchaseOrder → Vendor |
| P2P | MatchesOrder | VendorInvoice → PurchaseOrder |
| P2P | PaysInvoice | Payment → VendorInvoice |
| O2C | PlacedBy | SalesOrder → Customer |
| O2C | BillsOrder | CustomerInvoice → SalesOrder |
| S2C | RfxBelongsToProject | RfxEvent → SourcingProject |
| S2C | RespondsTo | SupplierBid → RfxEvent |
| S2C | AwardedFrom | ProcurementContract → BidEvaluation |
| H2R | RecordedBy | TimeEntry → Employee |
| H2R | PayrollIncludes | PayrollRun → Employee |
| H2R | SubmittedBy | ExpenseReport → Employee |
| H2R | EnrolledBy | BenefitEnrollment → Employee |
| MFG | Produces | ProductionOrder → Material |
| MFG | Inspects | QualityInspection → ProductionOrder |
| MFG | PartOf | BomComponent → Material |
| TAX | TaxLineBelongsTo | TaxLine → TaxReturn |
| TAX | ProvisionAppliesTo | TaxProvision → TaxJurisdiction |
| TAX | WithheldFrom | WithholdingTaxRecord → Vendor |
| Treasury | SweepsTo | CashPoolSweep → CashPool |
| Treasury | HedgesInstrument | HedgeRelationship → HedgingInstrument |
| Treasury | GovernsInstrument | DebtCovenant → DebtInstrument |
| ESG | EmissionReportedBy | EmissionRecord → Company |
| ESG | AssessesSupplier | SupplierEsgAssessment → Vendor |
| Project | CostChargedTo | ProjectCostLine → Project |
| Project | MilestoneOf | ProjectMilestone → Project |
| Project | ModifiesProject | ChangeOrder → Project |
| GOV | PrincipleUnder | CosoPrinciple → CosoComponent |
| GOV | AssertionCovers | SoxAssertion → GlAccount |
| GOV | JudgmentWithin | ProfessionalJudgment → AuditEngagement |
| Compliance | StandardToControl | ComplianceStandard → InternalControl |
| Compliance | FindingOnControl | ComplianceFinding → InternalControl |
| Compliance | StandardToAccount | ComplianceStandard → GlAccount |
| Compliance | FiledByCompany | RegulatoryFiling → Company |
| Compliance | GovernedByStandard | GlAccount → ComplianceStandard |
| Compliance | ImplementsStandard | InternalControl → ComplianceStandard |
| Compliance | FindingAffectsControl | ComplianceFinding → InternalControl |
| Compliance | FindingAffectsAccount | ComplianceFinding → GlAccount |

Each edge has a typed `EdgeConstraint` with `Cardinality` (OneToOne, OneToMany, ManyToMany) and optional edge properties.

### OCPM Events as Hyperedges

When `events_as_hyperedges: true`, each OCPM event becomes a hyperedge connecting all its participating objects. This enables cross-process analysis via the hypergraph structure.

### Per-Family Toggles

```yaml
graph_export:
  hypergraph:
    enabled: true
    process_layer:
      include_p2p: true
      include_o2c: true
      include_s2c: true
      include_h2r: true
      include_mfg: true
      include_bank: true
      include_audit: true
      include_r2r: true
      events_as_hyperedges: true
```

## Compliance Graph Integration (v1.1.0)

The compliance regulations framework integrates with both the standalone `ComplianceGraphBuilder` and the multi-layer hypergraph, enabling full enterprise graph traversal between regulatory standards, accounting data, and process documents.

### Cross-Domain Edges

When compliance regulations are enabled, the graph includes cross-domain edges:

```
Company ──FiledByCompany──▶ RegulatoryFiling
                                  │
                           Jurisdiction
                                  │
                        ComplianceStandard
                         ╱              ╲
          GovernedByStandard      ImplementsStandard
               ╱                          ╲
         GlAccount                 InternalControl
             │                           │
       JournalEntry              ComplianceFinding
```

### Hypergraph Placement

| Compliance Type | Hypergraph Layer | Type Code |
|-----------------|-----------------|-----------|
| ComplianceStandard | Layer 1 (GovernanceControls) | 505 |
| Jurisdiction | Layer 1 (GovernanceControls) | 506 |
| RegulatoryFiling | Layer 2 (ProcessEvents) | 507 |
| ComplianceFinding | Layer 2 (ProcessEvents) | 508 |

### Configuration

```yaml
compliance_regulations:
  graph:
    enabled: true
    include_account_links: true     # Standard → Account edges
    include_control_links: true     # Standard → Control edges
    include_company_links: true     # Filing → Company edges
```

### ToNodeProperties

All four compliance models implement `ToNodeProperties` for typed graph node conversion:

| Model | Type Code | Key Properties |
|-------|-----------|---------------|
| `ComplianceStandard` | 510 | standardId, title, issuingBody, category, domain, applicableAccountTypes, applicableProcesses |
| `ComplianceFinding` | 511 | findingId, severity, deficiencyLevel, controlId, affectedAccounts, remediationStatus |
| `RegulatoryFiling` | 512 | filingType, companyCode, jurisdiction, status, deadline |
| `JurisdictionProfile` | 513 | countryCode, accountingFramework, auditFramework, corporateTaxRate |

## See Also

- [Anomaly Injection](anomaly-injection.md)
- [Fraud Detection Use Case](../use-cases/fraud-detection.md)
- [datasynth-graph Crate](../crates/datasynth-graph.md)
- [Process Mining](../use-cases/process-mining.md)
- [Compliance Configuration](../configuration/compliance.md)
