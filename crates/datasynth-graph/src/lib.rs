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

#![allow(ambiguous_glob_reexports)]

pub mod builders;
pub mod exporters;
pub mod ml;
pub mod models;

#[cfg(test)]
pub(crate) mod test_helpers;

pub use builders::*;
pub use exporters::*;
pub use ml::*;
pub use models::*;
