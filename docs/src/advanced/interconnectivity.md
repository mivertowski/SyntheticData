# Interconnectivity and Relationship Modeling

SyntheticData provides comprehensive relationship modeling capabilities for generating realistic enterprise networks with multi-tier vendor relationships, customer segmentation, relationship strength calculations, and cross-process linkages.

## Overview

Real enterprise data exhibits complex interconnections between entities:
- Vendors form multi-tier supply chains (supplier-of-supplier)
- Customers segment by value (Enterprise vs. SMB) with different behaviors
- Relationships vary in strength based on transaction history
- Business processes connect (P2P and O2C link through inventory)

SyntheticData models all of these patterns to produce realistic, interconnected data.

---

## Multi-Tier Vendor Networks

### Supply Chain Tiers

Vendors are organized into a supply chain hierarchy:

| Tier | Description | Visibility | Typical Count |
|------|-------------|------------|---------------|
| **Tier 1** | Direct suppliers | Full financial visibility | 50-100 per company |
| **Tier 2** | Supplier's suppliers | Partial visibility | 4-10 per Tier 1 |
| **Tier 3** | Deep supply chain | Minimal visibility | 2-5 per Tier 2 |

### Vendor Clusters

Vendors are classified into behavioral clusters:

| Cluster | Share | Characteristics |
|---------|-------|-----------------|
| **ReliableStrategic** | 20% | High delivery scores, low invoice errors, consistent quality |
| **StandardOperational** | 50% | Average performance, predictable patterns |
| **Transactional** | 25% | One-off or occasional purchases |
| **Problematic** | 5% | Quality issues, late deliveries, invoice discrepancies |

### Vendor Lifecycle Stages

```
Onboarding → RampUp → SteadyState → Decline → Terminated
```

Each stage has associated behaviors:
- **Onboarding**: Initial qualification, small orders
- **RampUp**: Increasing order volumes
- **SteadyState**: Stable, predictable patterns
- **Decline**: Reduced orders, performance issues
- **Terminated**: Relationship ended

### Vendor Quality Scores

| Metric | Range | Description |
|--------|-------|-------------|
| `delivery_on_time` | 0.0-1.0 | Percentage of on-time deliveries |
| `quality_pass_rate` | 0.0-1.0 | Quality inspection pass rate |
| `invoice_accuracy` | 0.0-1.0 | Invoice matching accuracy |
| `responsiveness_score` | 0.0-1.0 | Communication responsiveness |

### Vendor Concentration Analysis

SyntheticData tracks vendor concentration risks:

```yaml
dependencies:
  max_single_vendor_concentration: 0.15  # No vendor > 15% of spend
  top_5_concentration: 0.45              # Top 5 vendors < 45% of spend
  single_source_percent: 0.05            # 5% of materials single-sourced
```

---

## Customer Value Segmentation

### Value Segments

Customers follow a Pareto-like distribution:

| Segment | Revenue Share | Customer Share | Typical Order Value |
|---------|--------------|----------------|---------------------|
| **Enterprise** | 40% | 5% | $50,000+ |
| **MidMarket** | 35% | 20% | $5,000-$50,000 |
| **SMB** | 20% | 50% | $500-$5,000 |
| **Consumer** | 5% | 25% | $50-$500 |

### Customer Lifecycle

```
Prospect → New → Growth → Mature → AtRisk → Churned
                                         ↓
                                      WonBack
```

Each stage has associated behaviors:
- **Prospect**: Potential customer, conversion probability
- **New**: First purchase within 90 days
- **Growth**: Increasing order frequency/value
- **Mature**: Stable, loyal customer
- **AtRisk**: Declining activity, churn signals
- **Churned**: No activity for extended period
- **WonBack**: Previously churned, now returned

### Customer Engagement Metrics

| Metric | Description |
|--------|-------------|
| `order_frequency` | Average orders per period |
| `recency_days` | Days since last order |
| `nps_score` | Net Promoter Score (-100 to +100) |
| `engagement_score` | Composite engagement metric (0.0-1.0) |

### Customer Networks

- **Referral Networks**: Customers refer other customers (configurable rate)
- **Corporate Hierarchies**: Parent/child company relationships
- **Industry Clusters**: Customers grouped by industry vertical

---

## Relationship Strength Modeling

### Composite Strength Calculation

Relationship strength is computed from multiple factors:

| Component | Weight | Scale | Description |
|-----------|--------|-------|-------------|
| Transaction Volume | 30% | Logarithmic | Total monetary value |
| Transaction Count | 25% | Square root | Number of transactions |
| Duration | 20% | Linear | Relationship age in days |
| Recency | 15% | Exponential decay | Days since last transaction |
| Mutual Connections | 10% | Jaccard index | Shared network connections |

### Strength Categories

| Strength | Threshold | Description |
|----------|-----------|-------------|
| **Strong** | ≥ 0.7 | Core business relationship |
| **Moderate** | ≥ 0.4 | Regular, established relationship |
| **Weak** | ≥ 0.1 | Occasional relationship |
| **Dormant** | < 0.1 | Inactive relationship |

### Recency Decay

The recency component uses exponential decay:

```
recency_score = exp(-days_since_last / half_life)
```

Default half-life is 90 days.

---

## Cross-Process Linkages

### Inventory Links (P2P ↔ O2C)

Inventory naturally connects Procure-to-Pay and Order-to-Cash:

```
P2P: Purchase Order → Goods Receipt → Vendor Invoice → Payment
                           ↓
                      [Inventory]
                           ↓
O2C: Sales Order → Delivery → Customer Invoice → Receipt
```

When enabled, SyntheticData generates explicit `CrossProcessLink` records connecting:
- `GoodsReceipt` (P2P) to `Delivery` (O2C) via inventory item

### Payment-Bank Reconciliation

Links payment transactions to bank statement entries for reconciliation.

### Intercompany Bilateral Links

Ensures intercompany transactions are properly linked between sending and receiving entities.

---

## Entity Graph

### Graph Structure

The `EntityGraph` provides a unified view of all entity relationships:

| Component | Description |
|-----------|-------------|
| **Nodes** | Entities with type, ID, and metadata |
| **Edges** | Relationships with type and strength |
| **Indexes** | Fast lookups by entity type and ID |

### Entity Types (16 types)

```
Company, Vendor, Customer, Employee, Department, CostCenter,
Project, Contract, Asset, BankAccount, Material, Product,
Location, Currency, Account, Entity
```

### Relationship Types (26 types)

```
// Transactional
BuysFrom, SellsTo, PaysTo, ReceivesFrom, SuppliesTo, OrdersFrom

// Organizational
ReportsTo, Manages, BelongsTo, OwnedBy, PartOf, Contains

// Network
ReferredBy, PartnersWith, AffiliateOf, SubsidiaryOf

// Process
ApprovesFor, AuthorizesFor, ProcessesFor

// Financial
BillsTo, ShipsTo, CollectsFrom, RemitsTo

// Document
ReferencedBy, SupersededBy, AmendedBy, LinkedTo
```

---

## Configuration

### Complete Example

```yaml
vendor_network:
  enabled: true
  depth: 3
  tiers:
    tier1:
      count_min: 50
      count_max: 100
    tier2:
      count_per_parent_min: 4
      count_per_parent_max: 10
    tier3:
      count_per_parent_min: 2
      count_per_parent_max: 5
  clusters:
    reliable_strategic: 0.20
    standard_operational: 0.50
    transactional: 0.25
    problematic: 0.05
  dependencies:
    max_single_vendor_concentration: 0.15
    top_5_concentration: 0.45
    single_source_percent: 0.05

customer_segmentation:
  enabled: true
  value_segments:
    enterprise:
      revenue_share: 0.40
      customer_share: 0.05
      avg_order_min: 50000.0
    mid_market:
      revenue_share: 0.35
      customer_share: 0.20
      avg_order_min: 5000.0
      avg_order_max: 50000.0
    smb:
      revenue_share: 0.20
      customer_share: 0.50
      avg_order_min: 500.0
      avg_order_max: 5000.0
    consumer:
      revenue_share: 0.05
      customer_share: 0.25
      avg_order_min: 50.0
      avg_order_max: 500.0
  lifecycle:
    prospect_rate: 0.10
    new_rate: 0.15
    growth_rate: 0.20
    mature_rate: 0.35
    at_risk_rate: 0.10
    churned_rate: 0.08
    won_back_rate: 0.02
  networks:
    referrals:
      enabled: true
      referral_rate: 0.15
    corporate_hierarchies:
      enabled: true
      hierarchy_probability: 0.30

relationship_strength:
  enabled: true
  calculation:
    transaction_volume_weight: 0.30
    transaction_count_weight: 0.25
    relationship_duration_weight: 0.20
    recency_weight: 0.15
    mutual_connections_weight: 0.10
    recency_half_life_days: 90
  thresholds:
    strong: 0.7
    moderate: 0.4
    weak: 0.1

cross_process_links:
  enabled: true
  inventory_p2p_o2c: true
  payment_bank_reconciliation: true
  intercompany_bilateral: true
```

---

## Network Evaluation

SyntheticData includes network metrics evaluation:

| Metric | Description | Typical Range |
|--------|-------------|---------------|
| **Connectivity** | Largest connected component ratio | > 0.95 |
| **Power Law Alpha** | Degree distribution exponent | 2.0-3.0 |
| **Clustering Coefficient** | Local clustering | 0.10-0.50 |
| **Top-1 Concentration** | Largest node share | < 0.15 |
| **Top-5 Concentration** | Top 5 nodes share | < 0.45 |
| **HHI** | Herfindahl-Hirschman Index | < 0.25 |

These metrics validate that generated networks exhibit realistic properties.

---

## API Usage

### Rust API

```rust
use datasynth_core::models::{
    VendorNetwork, VendorCluster, SupplyChainTier,
    SegmentedCustomerPool, CustomerValueSegment,
    EntityGraph, RelationshipStrengthCalculator,
};
use datasynth_generators::relationships::EntityGraphGenerator;

// Generate vendor network
let vendor_generator = VendorGenerator::new(config);
let vendor_network = vendor_generator.generate_vendor_network("C001");

// Generate segmented customers
let customer_generator = CustomerGenerator::new(config);
let customer_pool = customer_generator.generate_segmented_pool("C001");

// Build entity graph with cross-process links
let graph_generator = EntityGraphGenerator::with_defaults();
let entity_graph = graph_generator.generate_entity_graph(
    &vendor_network,
    &customer_pool,
    &transactions,
    &document_flows,
);
```

### Python API

```python
from datasynth_py import DataSynth
from datasynth_py.config import VendorNetworkConfig, CustomerSegmentationConfig

config = Config(
    vendor_network=VendorNetworkConfig(
        enabled=True,
        depth=3,
        clusters={"reliable_strategic": 0.20, "standard_operational": 0.50},
    ),
    customer_segmentation=CustomerSegmentationConfig(
        enabled=True,
        value_segments={
            "enterprise": {"revenue_share": 0.40, "customer_share": 0.05},
            "mid_market": {"revenue_share": 0.35, "customer_share": 0.20},
        },
    ),
)

result = DataSynth().generate(config=config, output={"format": "csv"})
```

---

## See Also

- [Graph Export](./graph-export.md) - Exporting entity graphs to PyTorch Geometric, Neo4j, DGL
- [Intercompany Processing](./intercompany.md) - Multi-entity transaction matching
- [Master Data Configuration](../configuration/master-data.md) - Vendor and customer settings
