#![deny(clippy::unwrap_used)]
//! # datasynth-graph-export
//!
//! Graph export pipeline for converting `EnhancedGenerationResult` into
//! RustGraph-ready bulk import format.
//!
//! ## Architecture
//!
//! The pipeline is trait-based with four pluggable stage types:
//!
//! 1. **PropertySerializer** — Converts domain model fields to `HashMap<String, Value>`.
//! 2. **NodeSynthesizer** — Creates [`ExportNode`]s from source data.
//! 3. **EdgeSynthesizer** — Creates [`ExportEdge`]s from source data.
//! 4. **PostProcessor** — Transforms the final [`GraphExportResult`].
//!
//! ## Usage
//!
//! ```ignore
//! use datasynth_graph_export::{GraphExportPipeline, ExportConfig};
//!
//! let pipeline = GraphExportPipeline::standard(ExportConfig::default());
//! let result = pipeline.export(&ds_result)?;
//!
//! // With rustgraph feature:
//! let (nodes, edges) = result.into_bulk();
//! ```

pub mod budget;
pub mod config;
pub mod edges;
pub mod error;
pub mod helpers;
pub mod id_map;
pub mod pipeline;
pub mod properties;
pub mod traits;
pub mod types;

// Re-export primary types for convenience.
pub use budget::BudgetManager;
pub use config::{
    BudgetConfig, EdgeSamplingStrategy, EdgeSynthesisConfig, ExportConfig, GroundTruthConfig,
    OcelExportConfig, PropertyCase, PropertyInclusionPolicy, RiskControlStrategy,
};
pub use error::{ExportError, ExportWarning, ExportWarnings, WarningSeverity};
pub use id_map::IdMap;
pub use pipeline::GraphExportPipeline;
pub use traits::{
    EdgeSynthesisContext, EdgeSynthesizer, NodeSynthesisContext, NodeSynthesizer, PostProcessor,
    PropertySerializer, SerializationContext,
};
pub use types::{
    ExportEdge, ExportMetadata, ExportNode, GraphExportResult, GroundTruthRecord,
    HyperedgeExport, NodeFeatureVector, OcelExport,
};
