//! Trait definitions for pluggable pipeline stages.
//!
//! Each stage of the export pipeline is defined as a trait so that:
//! - Stages can be replaced or extended without modifying the orchestrator.
//! - Domain-specific logic is encapsulated in trait implementations.
//! - Testing can use mock implementations.
//!
//! ## Pipeline Stages
//!
//! 1. **PropertySerializer** — Converts domain model fields into `HashMap<String, Value>`.
//! 2. **NodeSynthesizer** — Creates ExportNodes from source data (one per domain).
//! 3. **EdgeSynthesizer** — Creates ExportEdges from source data (one per edge category).
//! 4. **PostProcessor** — Transforms the final result (e.g., OCEL export, ground truth).

use std::collections::HashMap;

use crate::config::ExportConfig;
use crate::error::{ExportError, ExportWarnings};
use crate::id_map::IdMap;
use crate::types::{ExportEdge, ExportNode, GraphExportResult};

use datasynth_runtime::EnhancedGenerationResult;

// ──────────────────────────── Contexts ────────────────────────

/// Shared context passed to PropertySerializer implementations.
pub struct SerializationContext<'a> {
    /// The full generation result (read-only reference).
    pub ds_result: &'a EnhancedGenerationResult,
    /// Pipeline configuration.
    pub config: &'a ExportConfig,
    /// Employee external_id → display name.
    pub employee_by_id: &'a HashMap<String, String>,
    /// Employee role → external_id (for role-based lookups).
    pub employee_by_role: &'a HashMap<String, String>,
    /// User ID → employee external_id mapping.
    pub user_to_employee: &'a HashMap<String, String>,
    /// Account external_id → opening balance.
    pub opening_balances: &'a HashMap<String, f64>,
    /// Risk external_id → risk display name.
    pub risk_names: &'a HashMap<String, String>,
}

/// Context passed to EdgeSynthesizer implementations.
pub struct EdgeSynthesisContext<'a> {
    /// The full generation result.
    pub ds_result: &'a EnhancedGenerationResult,
    /// Pipeline configuration.
    pub config: &'a ExportConfig,
    /// ID map for resolving external IDs → numeric IDs.
    pub id_map: &'a IdMap,
    /// Non-fatal warnings accumulator.
    pub warnings: &'a mut ExportWarnings,
}

/// Context passed to NodeSynthesizer implementations.
pub struct NodeSynthesisContext<'a> {
    /// The full generation result.
    pub ds_result: &'a EnhancedGenerationResult,
    /// Pipeline configuration.
    pub config: &'a ExportConfig,
    /// ID map for assigning IDs to new nodes.
    pub id_map: &'a mut IdMap,
    /// Non-fatal warnings accumulator.
    pub warnings: &'a mut ExportWarnings,
}

// ──────────────────────────── Traits ──────────────────────────

/// Serializes domain model fields into a property map for a specific entity type.
///
/// Each domain (controls, risks, vendors, accounts, etc.) implements this trait
/// to convert its strongly-typed fields into `HashMap<String, serde_json::Value>`.
pub trait PropertySerializer: Send + Sync {
    /// The entity type name this serializer handles (e.g., "internal_control", "vendor").
    fn entity_type(&self) -> &'static str;

    /// Serialize a domain object into a property map.
    ///
    /// The `node_external_id` identifies which specific entity to serialize.
    /// Returns `None` if the entity is not found in the generation result.
    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, serde_json::Value>>;
}

/// Synthesizes ExportEdges for a category of relationships.
///
/// Each edge category (document chain, risk-control, accounting network, etc.)
/// implements this trait.
pub trait EdgeSynthesizer: Send + Sync {
    /// Human-readable name for this synthesizer (e.g., "document_chain", "risk_control").
    fn name(&self) -> &'static str;

    /// Generate edges from the source data.
    ///
    /// The implementation should use `ctx.id_map` to resolve string IDs to numeric IDs,
    /// and push warnings to `ctx.warnings` for non-fatal issues.
    fn synthesize(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Result<Vec<ExportEdge>, ExportError>;
}

/// Synthesizes ExportNodes for a domain of entity types.
///
/// Each domain (controls, process, accounting, etc.) implements this trait
/// to produce the nodes its entity types require.
pub trait NodeSynthesizer: Send + Sync {
    /// Human-readable name for this synthesizer (e.g., "governance", "process", "accounting").
    fn name(&self) -> &'static str;

    /// Generate nodes from the source data.
    ///
    /// The implementation should:
    /// 1. Create ExportNodes with properties from PropertySerializers.
    /// 2. Register each node in `ctx.id_map` via `get_or_insert`.
    /// 3. Set `node.id = Some(numeric_id)`.
    /// 4. Push warnings for any skipped entities.
    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError>;
}

/// Post-processes the complete GraphExportResult before it is returned.
///
/// Post-processors run after all nodes and edges are generated.
/// Examples: OCEL export, ground truth extraction, feature vector computation,
/// hyperedge assembly, metadata finalization.
pub trait PostProcessor: Send + Sync {
    /// Human-readable name for this post-processor.
    fn name(&self) -> &'static str;

    /// Process the result in-place. May add/modify OCEL, ground truth, metadata, etc.
    fn process(
        &self,
        result: &mut GraphExportResult,
        ds_result: &EnhancedGenerationResult,
        config: &ExportConfig,
        id_map: &IdMap,
    ) -> Result<(), ExportError>;
}
