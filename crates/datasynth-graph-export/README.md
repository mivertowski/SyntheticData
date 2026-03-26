# datasynth-graph-export

Graph export pipeline for converting synthetic data into graph-ready bulk import formats.

## Overview

`datasynth-graph-export` provides a trait-based pipeline that transforms `EnhancedGenerationResult` data into typed graph nodes and edges:

- **Four-stage pipeline**: PropertySerializer, NodeSynthesizer, EdgeSynthesizer, PostProcessor
- **78+ entity types**: Covers accounting, audit, banking, process mining, and compliance domains
- **RustGraph integration**: Optional `rustgraph` feature for `BulkNodeData`/`BulkEdgeData` conversion
- **Ground truth records**: Labeled anomaly/fraud data for ML training
- **OCEL export**: Object-centric event log projection from graph data

## Key Types

| Type | Purpose |
|------|---------|
| `GraphExportPipeline` | Orchestrates the four-stage export |
| `ExportConfig` | Controls property inclusion, edge sampling, budget limits |
| `ExportNode` / `ExportEdge` | Typed graph elements with property maps |
| `GraphExportResult` | Final output with nodes, edges, metadata, ground truth |
| `IdMap` | Deterministic ID mapping across entity types |
| `BudgetManager` | Controls output size limits |

## Usage

```rust
use datasynth_graph_export::{GraphExportPipeline, ExportConfig};

let pipeline = GraphExportPipeline::standard(ExportConfig::default());
let result = pipeline.export(&generation_result)?;

println!("Nodes: {}, Edges: {}", result.nodes.len(), result.edges.len());
```

With the `rustgraph` feature enabled:

```rust
let (bulk_nodes, bulk_edges) = result.into_bulk();
```

## Modules

| Module | Purpose |
|--------|---------|
| `pipeline` | `GraphExportPipeline` orchestration |
| `nodes` | Node synthesizers for all entity types |
| `edges` | Edge synthesizers for cross-entity relationships |
| `properties` | Property serialization from domain models |
| `config` | `ExportConfig`, budget, sampling, and ground truth settings |
| `types` | `ExportNode`, `ExportEdge`, `GraphExportResult`, feature vectors |
| `traits` | Pluggable stage traits (serializer, synthesizer, post-processor) |
| `id_map` | Deterministic ID mapping |
| `ocel` | OCEL 2.0 projection export |
| `post_process` | Result transformation and filtering |

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
