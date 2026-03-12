//! Graph export pipeline orchestrator.
//!
//! [`GraphExportPipeline`] coordinates the full export: node synthesis → edge synthesis →
//! post-processing → metadata finalization. Individual stages are plugged in via the
//! trait-based builder API.

use crate::config::ExportConfig;
use crate::error::ExportError;
use crate::traits::{EdgeSynthesizer, NodeSynthesizer, PostProcessor, PropertySerializer};
use crate::types::GraphExportResult;

use datasynth_runtime::EnhancedGenerationResult;

/// The main export pipeline.
///
/// Use [`GraphExportPipeline::standard()`] for the default configuration with all
/// built-in serializers, synthesizers, and post-processors, or build a custom pipeline
/// with the builder methods.
pub struct GraphExportPipeline {
    config: ExportConfig,
    property_serializers: Vec<Box<dyn PropertySerializer>>,
    node_synthesizers: Vec<Box<dyn NodeSynthesizer>>,
    edge_synthesizers: Vec<Box<dyn EdgeSynthesizer>>,
    post_processors: Vec<Box<dyn PostProcessor>>,
}

impl GraphExportPipeline {
    /// Create a new pipeline with the given configuration and no stages.
    pub fn new(config: ExportConfig) -> Self {
        Self {
            config,
            property_serializers: Vec::new(),
            node_synthesizers: Vec::new(),
            edge_synthesizers: Vec::new(),
            post_processors: Vec::new(),
        }
    }

    /// Create the standard pipeline with all built-in stages.
    ///
    /// This registers the default property serializers, node synthesizers,
    /// edge synthesizers, and post-processors for all supported domains.
    pub fn standard(config: ExportConfig) -> Self {
        // TODO(task-7): Register all built-in stages.
        Self::new(config)
    }

    /// Add a property serializer for a specific entity type.
    pub fn with_property_serializer(mut self, serializer: Box<dyn PropertySerializer>) -> Self {
        self.property_serializers.push(serializer);
        self
    }

    /// Add a node synthesizer for a domain.
    pub fn with_node_synthesizer(mut self, synthesizer: Box<dyn NodeSynthesizer>) -> Self {
        self.node_synthesizers.push(synthesizer);
        self
    }

    /// Add an edge synthesizer for an edge category.
    pub fn with_edge_synthesizer(mut self, synthesizer: Box<dyn EdgeSynthesizer>) -> Self {
        self.edge_synthesizers.push(synthesizer);
        self
    }

    /// Add a post-processor.
    pub fn with_post_processor(mut self, processor: Box<dyn PostProcessor>) -> Self {
        self.post_processors.push(processor);
        self
    }

    /// Get a reference to the pipeline configuration.
    pub fn config(&self) -> &ExportConfig {
        &self.config
    }

    /// Get a mutable reference to the pipeline configuration.
    pub fn config_mut(&mut self) -> &mut ExportConfig {
        &mut self.config
    }

    /// Get the registered property serializers.
    pub fn property_serializers(&self) -> &[Box<dyn PropertySerializer>] {
        &self.property_serializers
    }

    /// Execute the full export pipeline.
    ///
    /// Pipeline stages run in order:
    /// 1. Build lookup maps (employee, risk, opening balances).
    /// 2. Node synthesis (all NodeSynthesizers in registration order).
    /// 3. Edge synthesis (all EdgeSynthesizers in registration order).
    /// 4. Budget enforcement (trim nodes/edges if over budget).
    /// 5. Post-processing (OCEL, ground truth, feature vectors, metadata).
    ///
    /// Returns the complete [`GraphExportResult`] or a fatal [`ExportError`].
    pub fn export(
        &self,
        _ds_result: &EnhancedGenerationResult,
    ) -> Result<GraphExportResult, ExportError> {
        // TODO(task-7): Implement the full pipeline orchestration.
        let _ = &self.config;
        let _ = &self.property_serializers;
        let _ = &self.node_synthesizers;
        let _ = &self.edge_synthesizers;
        let _ = &self.post_processors;
        todo!("GraphExportPipeline::export() — implemented in Task 7")
    }
}

impl std::fmt::Debug for GraphExportPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphExportPipeline")
            .field("config", &self.config)
            .field(
                "property_serializers",
                &self.property_serializers.len(),
            )
            .field("node_synthesizers", &self.node_synthesizers.len())
            .field("edge_synthesizers", &self.edge_synthesizers.len())
            .field("post_processors", &self.post_processors.len())
            .finish()
    }
}
