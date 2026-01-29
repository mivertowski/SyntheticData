# Research: Interconnectivity and Relationship Modeling

## Current State Analysis

### Existing Relationship Infrastructure

| Relationship Type | Implementation | Depth |
|------------------|----------------|-------|
| Document Chains | `DocumentChainManager` | Strong |
| Three-Way Match | `ThreeWayMatcher` | Strong |
| Intercompany | `ICMatchingEngine` | Strong |
| GL Balance Links | Account hierarchies | Medium |
| Vendor-Customer | Basic master data | Weak |
| Employee-Approval | Approval chains | Medium |
| Entity Registry | `EntityRegistry` | Medium |

### Current Strengths

1. **Document flow integrity**: PO → GR → Invoice → Payment chains maintained
2. **Intercompany matching**: Automatic generation of offsetting entries
3. **Balance coherence**: Trial balance validation, A=L+E enforcement
4. **Graph export**: PyTorch Geometric, Neo4j, DGL formats supported
5. **COSO control mapping**: Controls linked to processes and risks

### Current Gaps

1. **Shallow vendor networks**: No supplier-of-supplier modeling
2. **Limited customer relationships**: No customer segmentation
3. **No organizational hierarchy depth**: Flat cost center structures
4. **Missing behavioral clustering**: Entities don't cluster by behavior
5. **No network effects**: Relationships don't influence behavior
6. **Static relationships**: No relationship lifecycle modeling

---

## Improvement Recommendations

### 1. Deep Vendor Network Modeling

#### 1.1 Multi-Tier Supply Chain

```yaml
vendor_network:
  enabled: true
  depth: 3  # Tier-1, Tier-2, Tier-3 suppliers

  tiers:
    tier_1:
      count: 50-100
      relationship: direct_supplier
      visibility: full
      transaction_volume: high

    tier_2:
      count: 200-500
      relationship: supplier_of_supplier
      visibility: partial
      transaction_volume: medium
      # Only visible through Tier-1 transactions

    tier_3:
      count: 500-2000
      relationship: indirect
      visibility: minimal
      transaction_volume: low

  # Dependency modeling
  dependencies:
    concentration:
      max_single_vendor: 0.15  # No vendor > 15% of spend
      top_5_vendors: 0.45      # Top 5 < 45% of spend

    critical_materials:
      single_source: 0.05      # 5% of materials are single-source
      dual_source: 0.15
      multi_source: 0.80

    substitutability:
      easy: 0.60
      moderate: 0.30
      difficult: 0.10
```

#### 1.2 Vendor Relationship Attributes

```rust
pub struct VendorRelationship {
    vendor_id: VendorId,
    relationship_type: VendorRelationshipType,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,

    // Relationship strength
    strategic_importance: StrategicLevel,  // Critical, Important, Standard, Transactional
    spend_tier: SpendTier,                 // Platinum, Gold, Silver, Bronze

    // Behavioral attributes
    payment_history: PaymentBehavior,
    dispute_frequency: DisputeLevel,
    quality_score: f64,

    // Contract terms
    contracted_rates: Vec<ContractedRate>,
    rebate_agreements: Vec<RebateAgreement>,
    payment_terms: PaymentTerms,

    // Network position
    tier: SupplyChainTier,
    parent_vendor: Option<VendorId>,
    child_vendors: Vec<VendorId>,
}

pub enum VendorRelationshipType {
    DirectSupplier,
    ServiceProvider,
    Contractor,
    Distributor,
    Manufacturer,
    RawMaterialSupplier,
    OEMPartner,
    Affiliate,
}
```

#### 1.3 Vendor Behavior Clustering

```yaml
vendor_clusters:
  enabled: true

  clusters:
    reliable_strategic:
      size: 0.20
      characteristics:
        payment_terms: [30, 45, 60]
        on_time_delivery: 0.95-1.0
        quality_issues: rare
        price_stability: high
        transaction_frequency: weekly+

    standard_operational:
      size: 0.50
      characteristics:
        payment_terms: [30]
        on_time_delivery: 0.85-0.95
        quality_issues: occasional
        price_stability: medium
        transaction_frequency: monthly

    transactional:
      size: 0.25
      characteristics:
        payment_terms: [0, 15]
        on_time_delivery: 0.75-0.90
        quality_issues: moderate
        price_stability: low
        transaction_frequency: quarterly

    problematic:
      size: 0.05
      characteristics:
        payment_terms: [0]  # COD only
        on_time_delivery: 0.50-0.80
        quality_issues: frequent
        price_stability: volatile
        transaction_frequency: declining
```

---

### 2. Customer Relationship Depth

#### 2.1 Customer Segmentation

```yaml
customer_segmentation:
  enabled: true

  dimensions:
    value:
      - segment: enterprise
        revenue_share: 0.40
        customer_share: 0.05
        characteristics:
          avg_order_value: 50000+
          order_frequency: weekly
          payment_behavior: terms
          churn_risk: low

      - segment: mid_market
        revenue_share: 0.35
        customer_share: 0.20
        characteristics:
          avg_order_value: 5000-50000
          order_frequency: monthly
          payment_behavior: mixed
          churn_risk: medium

      - segment: smb
        revenue_share: 0.20
        customer_share: 0.50
        characteristics:
          avg_order_value: 500-5000
          order_frequency: quarterly
          payment_behavior: prepay
          churn_risk: high

      - segment: consumer
        revenue_share: 0.05
        customer_share: 0.25
        characteristics:
          avg_order_value: 50-500
          order_frequency: occasional
          payment_behavior: immediate
          churn_risk: very_high

    lifecycle:
      - stage: prospect
        conversion_rate: 0.15
        avg_duration_days: 30

      - stage: new
        definition: "<90 days"
        behavior: exploring
        support_intensity: high

      - stage: growth
        definition: "90-365 days"
        behavior: expanding
        upsell_opportunity: high

      - stage: mature
        definition: ">365 days"
        behavior: stable
        retention_focus: true

      - stage: at_risk
        triggers: [declining_orders, late_payments, complaints]
        intervention: required

      - stage: churned
        definition: "no activity >180 days"
        win_back_probability: 0.10
```

#### 2.2 Customer Network Effects

```yaml
customer_networks:
  enabled: true

  # Referral relationships
  referrals:
    enabled: true
    referral_rate: 0.15
    referred_customer_value_multiplier: 1.2
    max_referral_chain: 3

  # Parent-child relationships (corporate structures)
  corporate_hierarchies:
    enabled: true
    probability: 0.30
    hierarchy_depth: 3
    billing_consolidation: true

  # Industry clustering
  industry_affinity:
    enabled: true
    same_industry_cluster_probability: 0.40
    industry_trend_correlation: 0.70
```

---

### 3. Organizational Hierarchy Modeling

#### 3.1 Deep Cost Center Structure

```yaml
organizational_structure:
  depth: 5

  levels:
    - level: 1
      name: division
      count: 3-5
      examples: ["North America", "EMEA", "APAC"]

    - level: 2
      name: business_unit
      count_per_parent: 2-4
      examples: ["Commercial", "Consumer", "Industrial"]

    - level: 3
      name: department
      count_per_parent: 3-6
      examples: ["Sales", "Marketing", "Operations", "Finance"]

    - level: 4
      name: function
      count_per_parent: 2-5
      examples: ["Inside Sales", "Field Sales", "Sales Ops"]

    - level: 5
      name: team
      count_per_parent: 2-4
      examples: ["Team Alpha", "Team Beta"]

  # Cross-cutting structures
  matrix_relationships:
    enabled: true
    types:
      - primary: division
        secondary: function
        # e.g., "EMEA Sales" reports to both EMEA Head and Global Sales VP

  # Shared services
  shared_services:
    enabled: true
    centers:
      - name: "Corporate Finance"
        serves: all_divisions
        allocation_method: headcount

      - name: "IT Infrastructure"
        serves: all_divisions
        allocation_method: usage

      - name: "HR Services"
        serves: all_divisions
        allocation_method: headcount
```

#### 3.2 Approval Hierarchy

```yaml
approval_hierarchy:
  enabled: true

  # Spending authority matrix
  authority_matrix:
    manager:
      limit: 5000
      exception_rate: 0.02

    senior_manager:
      limit: 25000
      exception_rate: 0.01

    director:
      limit: 100000
      exception_rate: 0.005

    vp:
      limit: 500000
      exception_rate: 0.002

    c_level:
      limit: unlimited
      exception_rate: 0.001

  # Approval chains
  chain_rules:
    sequential:
      enabled: true
      for: [capital_expenditure, contracts]

    parallel:
      enabled: true
      for: [operational_expenses]
      minimum_approvals: 2

    skip_level:
      enabled: true
      probability: 0.05
      audit_flag: true
```

---

### 4. Entity Relationship Graph

#### 4.1 Comprehensive Relationship Model

```rust
/// Unified entity relationship graph
pub struct EntityGraph {
    nodes: HashMap<EntityId, EntityNode>,
    edges: Vec<RelationshipEdge>,
    indexes: GraphIndexes,
}

pub struct EntityNode {
    id: EntityId,
    entity_type: EntityType,
    attributes: EntityAttributes,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
}

pub enum EntityType {
    Company,
    Vendor,
    Customer,
    Employee,
    Department,
    CostCenter,
    Project,
    Contract,
    Asset,
    BankAccount,
}

pub struct RelationshipEdge {
    from_id: EntityId,
    to_id: EntityId,
    relationship_type: RelationshipType,
    strength: f64,           // 0.0 - 1.0
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    attributes: RelationshipAttributes,
}

pub enum RelationshipType {
    // Transactional
    BuysFrom,
    SellsTo,
    PaysTo,
    ReceivesFrom,

    // Organizational
    ReportsTo,
    Manages,
    BelongsTo,
    OwnedBy,

    // Contractual
    ContractedWith,
    GuaranteedBy,
    InsuredBy,

    // Financial
    LendsTo,
    BorrowsFrom,
    InvestsIn,

    // Network
    ReferredBy,
    PartnersWith,
    CompetesWith,
}
```

#### 4.2 Relationship Strength Modeling

```yaml
relationship_strength:
  calculation:
    type: composite
    factors:
      transaction_volume:
        weight: 0.30
        normalization: log_scale

      transaction_count:
        weight: 0.25
        normalization: sqrt_scale

      relationship_duration:
        weight: 0.20
        decay: none

      recency:
        weight: 0.15
        decay: exponential
        half_life_days: 90

      mutual_connections:
        weight: 0.10
        normalization: jaccard_similarity

  thresholds:
    strong: 0.7
    moderate: 0.4
    weak: 0.1
    dormant: 0.0
```

---

### 5. Transaction Chain Integrity

#### 5.1 Extended Document Chains

```yaml
document_chains:
  # P2P extended chain
  procure_to_pay:
    stages:
      - type: purchase_requisition
        optional: true
        approval_required: conditional  # >$1000

      - type: purchase_order
        required: true
        generates: commitment

      - type: goods_receipt
        required: conditional  # For goods, not services
        updates: inventory
        tolerance: 0.05  # 5% over-receipt allowed

      - type: vendor_invoice
        required: true
        matching: three_way  # PO, GR, Invoice
        tolerance: 0.02

      - type: payment
        required: true
        methods: [ach, wire, check, virtual_card]
        generates: bank_transaction

    # Chain integrity rules
    integrity:
      sequence_enforcement: strict
      backdating_allowed: false
      amount_cascade: true  # Amounts must flow through

  # O2C extended chain
  order_to_cash:
    stages:
      - type: quote
        optional: true
        validity_days: 30

      - type: sales_order
        required: true
        credit_check: conditional

      - type: pick_list
        required: conditional
        triggers: inventory_reservation

      - type: delivery
        required: conditional
        updates: inventory
        generates: shipping_document

      - type: customer_invoice
        required: true
        triggers: revenue_recognition

      - type: customer_receipt
        required: true
        applies_to: invoices
        generates: bank_transaction

    integrity:
      partial_shipment: allowed
      partial_payment: allowed
      credit_memo: allowed
```

#### 5.2 Cross-Process Linkages

```yaml
cross_process_links:
  enabled: true

  links:
    # Inventory connects P2P and O2C
    - source_process: procure_to_pay
      source_stage: goods_receipt
      target_process: order_to_cash
      target_stage: pick_list
      through: inventory

    # Returns create reverse flows
    - source_process: order_to_cash
      source_stage: delivery
      target_process: returns
      target_stage: return_receipt
      condition: quality_issue

    # Payments connect to bank reconciliation
    - source_process: procure_to_pay
      source_stage: payment
      target_process: bank_reconciliation
      target_stage: bank_statement_line
      matching: automatic

    # Intercompany bilateral links
    - source_process: intercompany_sale
      source_stage: ic_invoice
      target_process: intercompany_purchase
      target_stage: ic_invoice
      matching: elimination_required
```

---

### 6. Network Effect Modeling

#### 6.1 Behavioral Influence

```yaml
network_effects:
  enabled: true

  influence_types:
    # Transaction patterns spread through network
    transaction_contagion:
      enabled: true
      effect: "similar vendors show similar payment patterns"
      correlation: 0.40
      lag_days: 30

    # Risk propagation
    risk_propagation:
      enabled: true
      effect: "vendor issues affect connected vendors"
      propagation_depth: 2
      decay_per_hop: 0.50

    # Seasonal correlation
    seasonal_sync:
      enabled: true
      effect: "connected entities show correlated seasonality"
      correlation: 0.60

    # Price correlation
    price_linkage:
      enabled: true
      effect: "commodity price changes propagate"
      propagation_speed: immediate
      pass_through_rate: 0.80
```

#### 6.2 Community Detection

```yaml
community_detection:
  enabled: true
  algorithms:
    - type: louvain
      resolution: 1.0
      output: vendor_communities

    - type: label_propagation
      output: customer_segments

    - type: girvan_newman
      output: department_clusters

  use_cases:
    # Fraud detection
    fraud_rings:
      algorithm: connected_components
      edge_filter: suspicious_transactions
      min_size: 3

    # Vendor consolidation
    vendor_overlap:
      algorithm: jaccard_similarity
      threshold: 0.70
      output: consolidation_candidates

    # Customer segmentation
    behavioral_clusters:
      algorithm: spectral
      features: [purchase_pattern, payment_behavior, product_mix]
```

---

### 7. Relationship Lifecycle

#### 7.1 Lifecycle Stages

```yaml
relationship_lifecycle:
  enabled: true

  vendor_lifecycle:
    stages:
      onboarding:
        duration_days: 30-90
        activities: [due_diligence, contract_negotiation, system_setup]
        transaction_volume: limited

      ramp_up:
        duration_days: 90-180
        activities: [volume_increase, performance_monitoring]
        transaction_volume: growing

      steady_state:
        duration_days: ongoing
        activities: [regular_transactions, periodic_review]
        transaction_volume: stable

      decline:
        triggers: [quality_issues, price_competitiveness, strategic_shift]
        activities: [reduced_orders, alternative_sourcing]
        transaction_volume: decreasing

      termination:
        triggers: [contract_end, performance_failure, strategic_decision]
        activities: [final_settlement, transition]
        transaction_volume: zero

    transitions:
      probability_matrix:
        onboarding:
          ramp_up: 0.80
          termination: 0.20
        ramp_up:
          steady_state: 0.85
          decline: 0.10
          termination: 0.05
        steady_state:
          steady_state: 0.90
          decline: 0.08
          termination: 0.02
        decline:
          steady_state: 0.20
          decline: 0.50
          termination: 0.30

  customer_lifecycle:
    # Similar structure for customer relationships
    stages:
      prospect: { conversion_rate: 0.15 }
      new: { retention_rate: 0.70 }
      active: { retention_rate: 0.90 }
      at_risk: { save_rate: 0.50 }
      churned: { win_back_rate: 0.10 }
```

---

### 8. Graph Export Enhancements

#### 8.1 Enhanced PyTorch Geometric Export

```yaml
graph_export:
  pytorch_geometric:
    enabled: true

    node_features:
      # Node type encoding
      type_encoding: one_hot

      # Numeric features
      numeric:
        - field: transaction_volume
          normalization: log_scale
        - field: relationship_duration_days
          normalization: min_max
        - field: average_amount
          normalization: z_score

      # Categorical features
      categorical:
        - field: industry
          encoding: label
        - field: region
          encoding: one_hot
        - field: segment
          encoding: embedding

    edge_features:
      - field: relationship_strength
        normalization: none
      - field: transaction_count
        normalization: log_scale
      - field: last_transaction_days_ago
        normalization: min_max

    # Temporal graphs
    temporal:
      enabled: true
      snapshot_frequency: monthly
      edge_weight_decay: exponential
      half_life_days: 90

    # Heterogeneous graph support
    heterogeneous:
      enabled: true
      node_types: [company, vendor, customer, employee, account]
      edge_types: [buys_from, sells_to, reports_to, pays_to]
```

#### 8.2 Enhanced Neo4j Export

```yaml
neo4j_export:
  enabled: true

  # Node labels
  node_labels:
    - label: Company
      properties: [code, name, currency, country]
    - label: Vendor
      properties: [id, name, category, rating]
    - label: Customer
      properties: [id, name, segment, region]
    - label: Transaction
      properties: [id, amount, date, type]

  # Relationship types
  relationships:
    - type: TRANSACTS_WITH
      properties: [volume, count, first_date, last_date]
    - type: BELONGS_TO
      properties: [start_date, role]
    - type: SUPPLIES
      properties: [material_type, contract_id]

  # Indexes for query optimization
  indexes:
    - label: Transaction
      property: date
      type: range
    - label: Vendor
      property: id
      type: unique
    - label: Customer
      property: segment
      type: lookup

  # Full-text search
  fulltext:
    - name: entity_search
      labels: [Vendor, Customer]
      properties: [name, description]
```

---

### 9. Implementation Priority

| Enhancement | Complexity | Impact | Priority |
|-------------|------------|--------|----------|
| Vendor network depth | High | High | P1 |
| Customer segmentation | Medium | High | P1 |
| Organizational hierarchy | Medium | Medium | P2 |
| Relationship strength modeling | Medium | High | P1 |
| Cross-process linkages | Medium | High | P1 |
| Network effect modeling | High | Medium | P2 |
| Relationship lifecycle | Medium | Medium | P2 |
| Community detection | High | Medium | P3 |
| Enhanced graph export | Low | High | P1 |

---

### 10. Validation Framework

```yaml
relationship_validation:
  integrity_checks:
    # All transactions have valid entity references
    referential_integrity:
      enabled: true
      strict: true

    # Document chains are complete
    chain_completeness:
      enabled: true
      allow_partial: false
      exception_rate: 0.02

    # Intercompany entries balance
    intercompany_balance:
      enabled: true
      tolerance: 0.01

  network_metrics:
    # Graph connectivity
    connectivity:
      check_strongly_connected: false
      check_weakly_connected: true
      max_isolated_nodes: 0.05

    # Degree distribution
    degree_distribution:
      check_power_law: true
      min_alpha: 1.5
      max_alpha: 3.0

    # Clustering coefficient
    clustering:
      min_coefficient: 0.1
      max_coefficient: 0.5
```

---

*See also*: [05-pattern-drift.md](05-pattern-drift.md) for temporal evolution of patterns
