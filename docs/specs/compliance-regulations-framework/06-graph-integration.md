# Part 6: Graph Integration & Edge Semantics

> **Parent:** [Compliance & Regulations Framework](00-index.md)
> **Status:** Implemented | **Date:** 2026-03-09

---

## 6.1 Overview

The compliance graph layer extends DataSynth's existing graph infrastructure (`datasynth-graph`) with new node types, edge types, and ML features specifically designed for compliance network analysis. The goal is to represent the **full compliance topology** — from high-level regulatory requirements down to individual test results — as a heterogeneous graph suitable for:

- **Compliance coverage analysis** — Identify untested controls, unmapped standards
- **Risk propagation** — Model how a control failure affects upstream standards
- **Anomaly detection** — Missing or unusual edges indicate compliance gaps
- **Link prediction** — Predict which procedures should test which controls
- **GNN-based compliance scoring** — Learn embeddings that capture compliance posture

---

## 6.2 Extended Node Types

### 6.2.1 New Node Types

```rust
pub enum NodeType {
    // ... existing types ...
    Account,
    JournalEntry,
    Vendor,
    Customer,
    User,
    Company,
    CostCenter,
    ProfitCenter,
    Material,
    FixedAsset,

    // NEW: Compliance node types
    /// A compliance standard (e.g., IFRS-16, ISA-315)
    Standard,
    /// A regulatory requirement (e.g., SOX-404, EU-AR-537)
    Regulation,
    /// A specific requirement within a standard
    Requirement,
    /// An audit assertion (occurrence, completeness, etc.)
    Assertion,
    /// An audit procedure instance
    AuditProcedure,
    /// An internal control
    Control,
    /// A compliance finding
    Finding,
    /// A legal jurisdiction
    Jurisdiction,
    /// A COSO component
    CosoComponent,
    /// A filing/regulatory report
    Filing,
    /// An audit engagement
    Engagement,
    /// A risk assessment
    RiskAssessment,

    Custom(String),
}
```

### 6.2.2 Node Feature Vectors

Each compliance node type has a defined feature vector:

**Standard Node (8 features):**

| Index | Feature | Description | Range |
|-------|---------|-------------|-------|
| F0 | `category_encoded` | One-hot: accounting(0), audit(1), regulatory(2), prudential(3) | [0, 3] |
| F1 | `domain_encoded` | One-hot encoding of ComplianceDomain | [0, 9] |
| F2 | `version_age_days` | Days since current version became effective | [0, ∞) normalized |
| F3 | `requirement_count` | Number of requirements in the standard | [0, ∞) log-scaled |
| F4 | `cross_reference_count` | Number of cross-referenced standards | [0, ∞) |
| F5 | `jurisdiction_count` | Number of mandatory jurisdictions | [0, ∞) |
| F6 | `is_superseded` | Whether this version has been superseded | {0, 1} |
| F7 | `impact_level` | Change impact (low=0.25, medium=0.5, high=0.75, replacement=1.0) | [0, 1] |

**Control Node (10 features):**

| Index | Feature | Description | Range |
|-------|---------|-------------|-------|
| F0 | `control_scope` | Entity-level(0), Transaction-level(1), IT-general(2), IT-app(3) | [0, 3] |
| F1 | `coso_component` | COSO component encoding | [0, 4] |
| F2 | `automation_level` | Manual(0), Semi-automated(0.5), Automated(1) | [0, 1] |
| F3 | `frequency` | Annual(0.1), Quarterly(0.25), Monthly(0.5), Weekly(0.75), Daily(1.0) | [0, 1] |
| F4 | `maturity_level` | COSO maturity (0-5) normalized | [0, 1] |
| F5 | `exception_rate` | Historical exception rate | [0, 1] |
| F6 | `standard_count` | Number of standards this control maps to | [0, ∞) log-scaled |
| F7 | `test_count` | Number of procedures testing this control | [0, ∞) |
| F8 | `finding_count` | Number of findings on this control | [0, ∞) |
| F9 | `risk_score` | Computed risk score | [0, 1] |

**Finding Node (7 features):**

| Index | Feature | Description | Range |
|-------|---------|-------------|-------|
| F0 | `severity` | High(1.0), Moderate(0.66), Low(0.33) | [0, 1] |
| F1 | `deficiency_type` | Material weakness(1.0), Significant(0.66), Control def.(0.33) | [0, 1] |
| F2 | `is_repeat` | Whether this finding recurred from prior period | {0, 1} |
| F3 | `days_open` | Days since finding was identified | [0, ∞) normalized |
| F4 | `remediation_status` | Open(0), In-progress(0.5), Remediated(1.0) | [0, 1] |
| F5 | `financial_impact` | Estimated financial impact (log-scaled) | [0, ∞) |
| F6 | `assertion_count` | Number of assertions affected | [1, 5] normalized |

**AuditProcedure Node (8 features):**

| Index | Feature | Description | Range |
|-------|---------|-------------|-------|
| F0 | `procedure_type` | Substantive(0), ToC(0.33), Analytical(0.66), Compliance(1.0) | [0, 1] |
| F1 | `sample_size` | Number of items tested | [0, ∞) log-scaled |
| F2 | `exception_rate` | Exceptions / sample size | [0, 1] |
| F3 | `risk_level` | Significant(1.0), High(0.75), Moderate(0.5), Low(0.25) | [0, 1] |
| F4 | `assertion_count` | Number of assertions tested | [1, 7] normalized |
| F5 | `standard_count` | Number of standards addressed | [0, ∞) |
| F6 | `conclusion` | Satisfactory(1.0), Exception(0.5), Unsatisfactory(0.0) | [0, 1] |
| F7 | `is_j_sox` | Whether this is a J-SOX/SOX/K-SOX specific procedure | {0, 1} |

---

## 6.3 Extended Edge Types

### 6.3.1 New Edge Types

```rust
pub enum EdgeType {
    // ... existing types ...
    Transaction,
    Approval,
    ReportsTo,
    Ownership,
    Intercompany,
    DocumentReference,
    CostAllocation,

    // NEW: Compliance edge types

    /// Control → Standard: This control addresses this standard
    MapsToStandard,
    /// Control → Assertion: This control covers this assertion
    CoversAssertion,
    /// Control → COSO Component: This control implements this COSO principle
    ImplementsCoso,
    /// Procedure → Control: This procedure tests this control
    TestsControl,
    /// Procedure → Assertion: This procedure addresses this assertion
    AddressesAssertion,
    /// Finding → Control: This finding was identified on this control
    FindingOnControl,
    /// Finding → Procedure: This procedure identified this finding
    IdentifiedByProcedure,
    /// Entity → Jurisdiction: This entity is subject to this jurisdiction
    SubjectToJurisdiction,
    /// Jurisdiction → Standard: This jurisdiction requires this standard
    RequiresStandard,
    /// Standard → Standard: This standard supersedes another
    Supersedes,
    /// Standard ↔ Standard: Cross-reference between standards
    CrossReferences,
    /// Standard → Requirement: This standard contains this requirement
    ContainsRequirement,
    /// Engagement → Entity: This engagement covers this entity
    CoversEntity,
    /// Engagement → Standard: This engagement addresses this standard
    EngagementStandard,
    /// RiskAssessment → Account: Risk assessed for this account
    RiskAssessedFor,
    /// Filing → Jurisdiction: This filing is required by this jurisdiction
    FilingRequiredBy,

    Custom(String),
}
```

### 6.3.2 Edge Feature Vectors

**MapsToStandard Edge (4 features):**

| Index | Feature | Description | Range |
|-------|---------|-------------|-------|
| F0 | `mapping_strength` | Direct(1.0), Indirect(0.5), Partial(0.25) | [0, 1] |
| F1 | `requirement_coverage` | % of standard requirements covered by this control | [0, 1] |
| F2 | `is_key_control` | Whether this is a key control for the standard | {0, 1} |
| F3 | `effective_date_days` | Days since the mapping became effective | [0, ∞) normalized |

**TestsControl Edge (5 features):**

| Index | Feature | Description | Range |
|-------|---------|-------------|-------|
| F0 | `test_frequency` | How often this procedure tests the control (annual=0.1 ... daily=1.0) | [0, 1] |
| F1 | `sample_coverage` | Proportion of control instances tested | [0, 1] |
| F2 | `exception_rate` | Exception rate from testing | [0, 1] |
| F3 | `test_result` | Pass(1.0), Exception(0.5), Fail(0.0) | [0, 1] |
| F4 | `design_vs_operating` | Design effectiveness(0), Operating effectiveness(1) | {0, 1} |

**FindingOnControl Edge (3 features):**

| Index | Feature | Description | Range |
|-------|---------|-------------|-------|
| F0 | `severity_score` | Finding severity | [0, 1] |
| F1 | `recurrence_count` | Number of prior periods with same finding | [0, ∞) |
| F2 | `compensating_control_exists` | Whether a compensating control mitigates | {0, 1} |

**SubjectToJurisdiction Edge (3 features):**

| Index | Feature | Description | Range |
|-------|---------|-------------|-------|
| F0 | `entity_type_encoded` | Listed(1), PIE(0.8), Large(0.6), SME(0.3), Micro(0.1) | [0, 1] |
| F1 | `standard_count` | Number of standards applicable via this jurisdiction | [0, ∞) log-scaled |
| F2 | `filing_frequency` | Annual(0.25), Semi-annual(0.5), Quarterly(0.75), Monthly(1.0) | [0, 1] |

---

## 6.4 Compliance Graph Builder

### 6.4.1 Architecture

```rust
pub struct ComplianceGraphBuilder {
    /// Reference to the standard registry
    registry: Arc<StandardRegistry>,
    /// Generated compliance artifacts
    artifacts: ComplianceArtifacts,
    /// Node ID allocator
    node_id_seq: AtomicU64,
    /// Edge ID allocator
    edge_id_seq: AtomicU64,
    /// Node index for deduplication
    node_index: HashMap<ComplianceNodeKey, NodeId>,
}

/// Key for deduplicating compliance nodes.
#[derive(Hash, Eq, PartialEq)]
pub enum ComplianceNodeKey {
    Standard(StandardId),
    Regulation(StandardId),
    Requirement(String),         // Requirement ID
    Control(String),             // Control ID
    Procedure(String),           // Procedure template ID + instance
    Finding(String),             // Finding ID
    Assertion(String, String),   // (Account, AssertionType)
    Jurisdiction(String),        // Country code
    Engagement(String),          // Engagement ID
}
```

### 6.4.2 Build Pipeline

```rust
impl ComplianceGraphBuilder {
    pub fn build(
        &mut self,
        resolved_state: &ResolvedRegulatoryState,
        controls: &[InternalControl],
        procedures: &[ProcedureResult],
        findings: &[ComplianceFinding],
        companies: &[CompanyConfig],
    ) -> ComplianceGraph {
        let mut graph = ComplianceGraph::new();

        // Phase 1: Standard & Regulation nodes
        self.build_standard_nodes(&mut graph, resolved_state);

        // Phase 2: Jurisdiction nodes
        self.build_jurisdiction_nodes(&mut graph, companies);

        // Phase 3: Control nodes with COSO mapping
        self.build_control_nodes(&mut graph, controls);

        // Phase 4: Assertion nodes
        self.build_assertion_nodes(&mut graph, resolved_state);

        // Phase 5: Procedure nodes
        self.build_procedure_nodes(&mut graph, procedures);

        // Phase 6: Finding nodes
        self.build_finding_nodes(&mut graph, findings);

        // Phase 7: Connect everything with edges
        self.build_standard_jurisdiction_edges(&mut graph, resolved_state, companies);
        self.build_control_standard_edges(&mut graph, controls);
        self.build_control_coso_edges(&mut graph, controls);
        self.build_procedure_control_edges(&mut graph, procedures);
        self.build_procedure_assertion_edges(&mut graph, procedures);
        self.build_finding_edges(&mut graph, findings);
        self.build_cross_reference_edges(&mut graph, resolved_state);
        self.build_supersession_edges(&mut graph, resolved_state);

        // Phase 8: Compute all node and edge features
        graph.compute_features();

        graph
    }
}
```

---

## 6.5 Graph Topology Examples

### 6.5.1 SOX Compliance Subgraph

```
                    ┌──────────────┐
                    │  SOX-404     │
                    │  (Standard)  │
                    └──────┬───────┘
                           │ RequiresStandard
                    ┌──────▼───────┐
                    │  US          │
                    │ (Jurisdiction)│
                    └──────┬───────┘
                           │ SubjectTo
                    ┌──────▼───────┐
                    │  C001 (US)   │
                    │  (Company)   │
                    └──────────────┘

         ┌─────────────────────────────────────────────┐
         │          COSO Control Environment            │
         │                (CosoComponent)               │
         └─────────────────┬───────────────────────────┘
                           │ ImplementsCoso
              ┌────────────┼────────────┐
              ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │ C001     │ │ C010     │ │ C070     │
        │ (Control)│ │ (Control)│ │ (Control)│
        │ 3-Way   │ │ JE Review│ │ Code of  │
        │ Match    │ │          │ │ Conduct  │
        └────┬─────┘ └────┬─────┘ └────┬─────┘
             │ MapsTo      │ MapsTo     │ MapsTo
             ▼             ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │ SOX-404  │ │ ISA-315  │ │ SOX-302  │
        │(Standard)│ │(Standard)│ │(Standard)│
        └──────────┘ └──────────┘ └──────────┘

        ┌──────────────┐
        │ AP-SOX-001   │  TestsControl
        │ (Procedure)  │──────────────────▶ C001 (Control)
        └──────┬───────┘
               │ AddressesAssertion
               ▼
        ┌──────────────┐
        │ Occurrence   │
        │ (Assertion)  │──────CoversAssertion──▶ C001 (Control)
        └──────────────┘

        ┌──────────────┐
        │ FND-001      │
        │ (Finding)    │──FindingOnControl──▶ C001
        │ Severity:Med │──IdentifiedBy─────▶ AP-SOX-001
        └──────────────┘
```

### 6.5.2 Multi-Jurisdiction Compliance Graph

```
        ┌──────────┐    ┌──────────┐    ┌──────────┐
        │  US      │    │  DE      │    │  JP      │
        │(Jurisd.) │    │(Jurisd.) │    │(Jurisd.) │
        └────┬─────┘    └────┬─────┘    └────┬─────┘
             │               │               │
    Requires │      Requires │      Requires │
             ▼               ▼               ▼
        ┌──────────┐    ┌──────────┐    ┌──────────┐
        │ SOX-404  │    │ EU-AR-537│    │ J-SOX    │
        │ PCAOB    │    │ ISA (IDW)│    │ JICPA    │
        │ ASC-606  │    │ HGB-253  │    │ J-GAAP   │
        └──────────┘    │ EU-CSRD  │    └──────────┘
                        └──────────┘

    CrossReferences:
        SOX-404  ◄────────────────►  EU-AR-537
        SOX-404  ◄────────────────►  J-SOX
        ASC-606  ◄────────────────►  IFRS-15

    Supersedes:
        IAS-17   ──Supersedes──▶   IFRS-16  (2019-01-01)
        IAS-39   ──Supersedes──▶   IFRS-9   (2018-01-01)
```

---

## 6.6 Integration with Existing Graph Builders

The compliance graph merges into the existing unified graph export:

```rust
// In datasynth-graph/src/builders/mod.rs
pub fn build_unified_graph(data: &GeneratedData) -> UnifiedGraph {
    let mut graph = UnifiedGraph::new();

    // Existing builders
    graph.merge(TransactionGraphBuilder::build(&data.journal_entries, &data.coa));
    graph.merge(ApprovalGraphBuilder::build(&data.approvals, &data.users));
    graph.merge(EntityGraphBuilder::build(&data.companies, &data.relationships));

    // NEW: Compliance graph
    if data.compliance.is_some() {
        graph.merge(ComplianceGraphBuilder::build(
            &data.compliance,
            &data.controls,
            &data.audit_procedures,
            &data.findings,
            &data.companies,
        ));
    }

    graph
}
```

### 6.6.1 Cross-Graph Edges

The compliance graph connects to existing graph layers through **cross-graph edges**:

| Edge | Source Graph | Target Graph | Semantics |
|------|-------------|--------------|-----------|
| Control → Account | Compliance | Transaction | Control covers this account |
| Procedure → JournalEntry | Compliance | Transaction | Procedure tested this transaction |
| Finding → JournalEntry | Compliance | Transaction | Finding relates to this transaction |
| Entity → Jurisdiction | Entity | Compliance | Entity subject to jurisdiction |
| User → Procedure | Approval | Compliance | Auditor performed this procedure |
| User → Finding | Approval | Compliance | Auditor identified this finding |

---

## 6.7 ML Feature Engineering

### 6.7.1 Compliance-Aware Node Embeddings

The compliance graph enables training of compliance-aware GNN models:

```python
import torch
from torch_geometric.data import HeteroData

# Load heterogeneous compliance graph
data = HeteroData()

# Node types with features
data['standard'].x = torch.load('standard_features.pt')    # [N_std, 8]
data['control'].x = torch.load('control_features.pt')      # [N_ctrl, 10]
data['procedure'].x = torch.load('procedure_features.pt')  # [N_proc, 8]
data['finding'].x = torch.load('finding_features.pt')      # [N_find, 7]
data['account'].x = torch.load('account_features.pt')      # [N_acct, 4]

# Edge types with features
data['control', 'maps_to', 'standard'].edge_index = ...
data['control', 'maps_to', 'standard'].edge_attr = ...     # [E, 4]
data['procedure', 'tests', 'control'].edge_index = ...
data['procedure', 'tests', 'control'].edge_attr = ...      # [E, 5]
data['finding', 'on', 'control'].edge_index = ...
data['finding', 'on', 'control'].edge_attr = ...           # [E, 3]

# Labels for supervised tasks
data['control'].y = torch.load('control_labels.pt')  # Deficiency label
data['finding'].y = torch.load('finding_labels.pt')  # Severity label
```

### 6.7.2 Supervised Tasks

| Task | Type | Labels | Description |
|------|------|--------|-------------|
| **Control Deficiency Prediction** | Node classification | Control → {effective, deficient} | Predict which controls will have deficiencies |
| **Finding Severity** | Node classification | Finding → {high, moderate, low} | Predict finding severity from graph structure |
| **Missing Control Mapping** | Link prediction | Control → Standard | Predict unmapped standard-control relationships |
| **Compliance Risk Score** | Node regression | Account → risk_score | Predict account-level compliance risk |
| **Audit Procedure Recommendation** | Link prediction | Procedure → Control | Recommend which procedures to apply to controls |
| **Material Weakness Detection** | Graph classification | Subgraph → {material_weakness, no_mw} | Detect entity-level material weaknesses from graph patterns |

### 6.7.3 Compliance Graph Statistics (Output)

```json
{
  "compliance_graph_stats": {
    "node_counts": {
      "standard": 45,
      "regulation": 12,
      "requirement": 230,
      "control": 78,
      "procedure": 156,
      "finding": 23,
      "assertion": 42,
      "jurisdiction": 3,
      "engagement": 1
    },
    "edge_counts": {
      "maps_to_standard": 312,
      "covers_assertion": 195,
      "implements_coso": 78,
      "tests_control": 234,
      "addresses_assertion": 390,
      "finding_on_control": 23,
      "identified_by_procedure": 23,
      "subject_to_jurisdiction": 5,
      "requires_standard": 45,
      "supersedes": 8,
      "cross_references": 67,
      "contains_requirement": 230
    },
    "coverage_metrics": {
      "standards_with_mapped_controls": 0.89,
      "controls_with_test_procedures": 0.95,
      "assertions_with_procedures": 0.92,
      "findings_with_remediation": 0.78
    }
  }
}
```

---

## 6.8 Export Formats

The compliance graph is exported alongside existing graph formats:

```
output/graphs/
├── transaction_network/        # Existing
│   └── pytorch_geometric/
├── approval_network/           # Existing
│   └── pytorch_geometric/
├── entity_network/             # Existing
│   └── pytorch_geometric/
├── compliance_network/         # NEW
│   ├── pytorch_geometric/
│   │   ├── hetero_data.pt       # HeteroData with all node/edge types
│   │   ├── standard_features.npy
│   │   ├── control_features.npy
│   │   ├── procedure_features.npy
│   │   ├── finding_features.npy
│   │   ├── edge_index_maps_to.npy
│   │   ├── edge_index_tests.npy
│   │   ├── edge_attr_maps_to.npy
│   │   ├── edge_attr_tests.npy
│   │   ├── control_labels.npy
│   │   ├── finding_labels.npy
│   │   ├── metadata.json
│   │   └── load_compliance_graph.py
│   ├── neo4j/
│   │   ├── standards.csv
│   │   ├── controls.csv
│   │   ├── procedures.csv
│   │   ├── findings.csv
│   │   ├── maps_to_standard.csv
│   │   ├── tests_control.csv
│   │   ├── finding_on_control.csv
│   │   └── import.cypher
│   └── dgl/
│       └── compliance_graph.dgl
└── unified/                    # Combined graph (all layers)
    └── ...
```

---

## 6.9 Neo4j Cypher Queries

Example queries for the compliance graph in Neo4j:

```cypher
// Find all controls that map to SOX-404 but have no test procedures
MATCH (c:Control)-[:MAPS_TO]->(s:Standard {id: 'SOX-404'})
WHERE NOT (c)<-[:TESTS]-(:AuditProcedure)
RETURN c.id, c.description

// Find the compliance coverage chain for an account
MATCH path = (a:Account)<-[:COVERS]-(ctrl:Control)-[:MAPS_TO]->(std:Standard)
WHERE a.account_code = '4000'
RETURN path

// Identify controls with findings but no remediation
MATCH (f:Finding)-[:FINDING_ON]->(c:Control)
WHERE f.remediation_status = 'Open' AND f.severity = 'High'
RETURN c.id, c.description, count(f) AS open_findings
ORDER BY open_findings DESC

// Cross-jurisdiction standard equivalence
MATCH (j1:Jurisdiction)-[:REQUIRES]->(s1:Standard)-[:CROSS_REFERENCES]->(s2:Standard)<-[:REQUIRES]-(j2:Jurisdiction)
WHERE j1.code = 'US' AND j2.code = 'DE'
RETURN s1.title AS us_standard, s2.title AS de_equivalent

// Risk propagation: high-risk accounts with weak controls
MATCH (a:Account)<-[:COVERS]-(c:Control)
WHERE c.exception_rate > 0.1
WITH a, count(c) AS weak_controls, collect(c.id) AS control_ids
MATCH (a)<-[:RISK_ASSESSED_FOR]-(r:RiskAssessment)
WHERE r.risk_level = 'High'
RETURN a.account_code, a.account_name, weak_controls, control_ids
```
