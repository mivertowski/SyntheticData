#![deny(clippy::unwrap_used)]
//! # synth-graph
//!
//! Graph/network export library for synthetic accounting data.
//!
//! This crate provides:
//! - Graph models for representing accounting data as networks
//! - Builders for creating transaction, approval, and entity graphs
//! - Exporters for PyTorch Geometric, Neo4j, and DGL formats
//! - ML utilities for feature extraction and dataset splitting
//!
//! ## Graph Types
//!
//! - **Transaction Network**: Accounts/entities as nodes, transactions as edges
//! - **Approval Network**: Users as nodes, approvals as edges (for SoD detection)
//! - **Entity Relationship**: Legal entities with ownership edges
//!
//! ## Export Formats
//!
//! - **PyTorch Geometric**: node_features.pt, edge_index.pt, edge_attr.pt
//! - **Neo4j**: CSV files with Cypher import scripts
//! - **DGL**: Compatible format for Deep Graph Library

pub mod builders;
pub mod exporters;
pub mod ml;
pub mod models;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod test_helpers;

// Re-export builder types
pub use builders::{
    ApprovalGraphBuilder, ApprovalGraphConfig, BankingGraphBuilder, BankingGraphConfig,
    EntityGraphBuilder, EntityGraphConfig, HypergraphBuilder, HypergraphConfig,
    OwnershipHierarchy, OwnershipHierarchyNode, SimpleApproval, TransactionGraphBuilder,
    TransactionGraphConfig,
};

// Re-export exporter types
pub use exporters::{
    CommonExportConfig, CommonGraphMetadata, CypherQueryBuilder, DGLExportConfig, DGLExporter,
    DGLMetadata, HypergraphExportConfig, HypergraphExporter, Neo4jExportConfig, Neo4jExporter,
    Neo4jMetadata, PyGExportConfig, PyGExporter, PyGMetadata, RawUnifiedEdge, RawUnifiedHyperedge,
    RawUnifiedNode, RustGraphEdgeMetadata, RustGraphEdgeOutput, RustGraphExportConfig,
    RustGraphExporter, RustGraphMetadata, RustGraphNodeMetadata, RustGraphNodeOutput,
    RustGraphOutputFormat, RustGraphUnifiedExporter, UnifiedExportConfig,
    UnifiedHypergraphMetadata,
};

// Re-export ML types
pub use ml::*;

// Re-export model types
pub use models::{
    AccountNode, AggregationStrategy, ApprovalEdge, CompanyNode, CrossLayerEdge, EdgeDirection,
    EdgeId, EdgeProperty, EdgeType, Graph, GraphEdge, GraphMetadata, GraphNode, GraphType,
    HeterogeneousGraph, Hyperedge, HyperedgeParticipant, Hypergraph, HypergraphLayer,
    HypergraphMetadata, HypergraphNode, NodeBudget, NodeBudgetReport, NodeId, NodeProperty,
    NodeType, OwnershipEdge, TransactionEdge, UserNode,
};
