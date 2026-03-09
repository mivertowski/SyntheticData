# datasynth-graph

Graph/network export for synthetic accounting data with ML-ready formats.

## Overview

`datasynth-graph` provides graph construction and export capabilities:

- **Graph Builders**: Transaction, approval, entity relationship, compliance, and multi-layer hypergraph builders
- **Compliance Graph**: Cross-domain linking of standards to GL accounts, internal controls, companies, and business processes with full enterprise graph traversal
- **ML Export**: PyTorch Geometric, Neo4j, DGL, RustGraph, and RustGraph Hypergraph formats
- **Unified Config**: `CommonExportConfig` shared across all ML exporters for consistent feature/label/mask settings
- **Feature Engineering**: Temporal, amount, structural, and categorical features
- **Data Splits**: Train/validation/test split generation

## Graph Types

| Graph | Nodes | Edges | Use Case |
|-------|-------|-------|----------|
| Transaction Network | Accounts, Entities | Transactions | Anomaly detection |
| Approval Network | Users | Approvals | SoD analysis |
| Entity Relationship | Legal Entities | Ownership | Consolidation analysis |
| Compliance Network | Standards, Findings, Filings, Jurisdictions | GovernedByStandard, ImplementsStandard, FiledByCompany | Compliance coverage, risk propagation |
| Multi-Layer Hypergraph | 55+ entity types across 10+ families | 36+ edge types | Enterprise-wide analysis |

## Export Formats

### PyTorch Geometric

```
graphs/transaction_network/pytorch_geometric/
├── node_features.pt    # [num_nodes, num_features]
├── edge_index.pt       # [2, num_edges]
├── edge_attr.pt        # [num_edges, num_edge_features]
├── labels.pt           # [num_nodes] or [num_edges]
├── train_mask.pt       # Boolean mask
├── val_mask.pt
└── test_mask.pt
```

### Neo4j

```
graphs/entity_relationship/neo4j/
├── nodes_account.csv
├── nodes_entity.csv
├── edges_transaction.csv
└── import.cypher
```

## Features

| Category | Features |
|----------|----------|
| Temporal | weekday, period, is_month_end, is_quarter_end, is_year_end |
| Amount | log(amount), benford_probability, is_round_number |
| Structural | line_count, unique_accounts, has_intercompany |
| Categorical | business_process (one-hot), source_type (one-hot) |

## Usage

```rust
use datasynth_graph::{TransactionGraphBuilder, PyTorchGeometricExporter};

let builder = TransactionGraphBuilder::new();
let graph = builder.build(&entries)?;

let exporter = PyTorchGeometricExporter::new("output/graphs");
exporter.export(&graph, split_config)?;
```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
