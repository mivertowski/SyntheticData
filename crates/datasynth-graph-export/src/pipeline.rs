//! Graph export pipeline orchestrator.
//!
//! [`GraphExportPipeline`] coordinates the full export: node synthesis → edge synthesis →
//! post-processing → metadata finalization. Individual stages are plugged in via the
//! trait-based builder API.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use rust_decimal::prelude::ToPrimitive;
use tracing::{debug, info};

use crate::budget::BudgetManager;
use crate::config::ExportConfig;
use crate::error::{ExportError, ExportWarnings, WarningSeverity};
use crate::id_map::IdMap;
use crate::traits::{
    EdgeSynthesisContext, EdgeSynthesizer, NodeSynthesisContext, NodeSynthesizer, PostProcessor,
    PropertySerializer, SerializationContext,
};
use crate::types::{ExportMetadata, GraphExportResult};

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
        let mut pipeline = Self::new(config);

        // Stage 1: Property serializers (Task 8)
        for serializer in crate::properties::all_serializers() {
            pipeline.property_serializers.push(serializer);
        }

        // Stage 2: Node synthesizers (Task 12)
        for synthesizer in crate::nodes::all_synthesizers() {
            pipeline.node_synthesizers.push(synthesizer);
        }

        // Stage 3: Edge synthesizers (Task 10)
        for synthesizer in crate::edges::all_synthesizers() {
            pipeline.edge_synthesizers.push(synthesizer);
        }

        // Stage 4: Post-processors (Task 13)
        for processor in crate::post_process::all_post_processors() {
            pipeline.post_processors.push(processor);
        }
        // OCEL exporter runs last (after all node/edge mutations).
        pipeline
            .post_processors
            .push(Box::new(crate::ocel::OcelExporterPostProcessor));

        pipeline
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
    /// 3. Budget enforcement on nodes (trim if over budget).
    /// 4. Edge synthesis (all EdgeSynthesizers in topological order by `name()`).
    /// 5. Budget enforcement on edges (trim if over budget).
    /// 6. Post-processing (OCEL, ground truth, feature vectors, metadata).
    ///
    /// Returns the complete [`GraphExportResult`] or a fatal [`ExportError`].
    pub fn export(
        &self,
        ds_result: &EnhancedGenerationResult,
    ) -> Result<GraphExportResult, ExportError> {
        let start = Instant::now();
        let mut warnings = ExportWarnings::new();
        let mut id_map = IdMap::with_capacity(self.config.budget.max_nodes);

        info!(
            "Starting graph export pipeline: {} node synthesizers, {} edge synthesizers, {} post-processors",
            self.node_synthesizers.len(),
            self.edge_synthesizers.len(),
            self.post_processors.len()
        );

        // ── Stage 1: Build SerializationContext lookup maps ──────────
        debug!("Stage 1: Building serialization context lookup maps");
        let (employee_by_id, employee_by_role, user_to_employee) = build_employee_maps(ds_result);
        let opening_balances = build_opening_balance_map(ds_result);
        let risk_names = build_risk_name_map(ds_result);

        let _serialization_ctx = SerializationContext {
            ds_result,
            config: &self.config,
            employee_by_id: &employee_by_id,
            employee_by_role: &employee_by_role,
            user_to_employee: &user_to_employee,
            opening_balances: &opening_balances,
            risk_names: &risk_names,
        };

        debug!(
            "Context maps built: {} employees, {} opening balances, {} risk names",
            employee_by_id.len(),
            opening_balances.len(),
            risk_names.len()
        );

        // ── Stage 2: Node synthesis ─────────────────────────────────
        debug!(
            "Stage 2: Running {} node synthesizers",
            self.node_synthesizers.len()
        );
        let mut nodes = Vec::new();

        for synthesizer in &self.node_synthesizers {
            let name = synthesizer.name();
            debug!("Running node synthesizer: {name}");

            let mut ctx = NodeSynthesisContext {
                ds_result,
                config: &self.config,
                id_map: &mut id_map,
                warnings: &mut warnings,
            };

            match synthesizer.synthesize(&mut ctx) {
                Ok(mut synth_nodes) => {
                    info!(
                        "Node synthesizer '{name}' produced {} nodes",
                        synth_nodes.len()
                    );
                    nodes.append(&mut synth_nodes);
                }
                Err(e) => {
                    return Err(ExportError::StageError {
                        stage: "node_synthesis",
                        message: format!("node synthesizer '{name}' failed: {e}"),
                    });
                }
            }
        }

        info!(
            "Node synthesis complete: {} total nodes, {} IDs assigned",
            nodes.len(),
            id_map.len()
        );

        // ── Stage 3: Node budget enforcement ────────────────────────
        debug!("Stage 3: Enforcing node budget");
        let budget_manager = BudgetManager::new(&self.config.budget);
        nodes = budget_manager.enforce_node_budget(nodes, &mut id_map, &mut warnings);

        // ── Stage 4: Edge synthesis (topological order) ─────────────
        debug!(
            "Stage 4: Running {} edge synthesizers",
            self.edge_synthesizers.len()
        );
        let mut edges = Vec::new();

        // Edge synthesizers run in registration order. Since the pipeline builder
        // controls registration order, and later tasks will register them in
        // dependency order, no topological sort is needed at runtime.
        // However, we verify there are no duplicate names (which would indicate
        // a configuration error).
        let mut seen_names: HashSet<&str> = HashSet::new();
        for synth in &self.edge_synthesizers {
            if !seen_names.insert(synth.name()) {
                warnings.add(
                    "edge_synthesis",
                    WarningSeverity::Low,
                    format!("Duplicate edge synthesizer name: '{}'", synth.name()),
                );
            }
        }

        for synthesizer in &self.edge_synthesizers {
            let name = synthesizer.name();
            debug!("Running edge synthesizer: {name}");

            let mut ctx = EdgeSynthesisContext {
                ds_result,
                config: &self.config,
                id_map: &id_map,
                warnings: &mut warnings,
            };

            match synthesizer.synthesize(&mut ctx) {
                Ok(mut synth_edges) => {
                    info!(
                        "Edge synthesizer '{name}' produced {} edges",
                        synth_edges.len()
                    );
                    edges.append(&mut synth_edges);
                }
                Err(e) => {
                    return Err(ExportError::StageError {
                        stage: "edge_synthesis",
                        message: format!("edge synthesizer '{name}' failed: {e}"),
                    });
                }
            }
        }

        info!("Edge synthesis complete: {} total edges", edges.len());

        // ── Stage 5: Edge budget enforcement ────────────────────────
        debug!("Stage 5: Enforcing edge budget");
        edges = budget_manager.enforce_edge_budget(
            edges,
            &id_map,
            self.config.edge_synthesis.sampling_strategy,
            &mut warnings,
        );

        // ── Stage 6: Build result + post-processing ─────────────────
        debug!(
            "Stage 6: Building result and running {} post-processors",
            self.post_processors.len()
        );

        // Compute metadata before post-processors (they may augment it)
        let metadata = build_metadata(&nodes, &edges, start);

        let mut result = GraphExportResult {
            nodes,
            edges,
            ocel: None,
            ground_truth: Vec::new(),
            feature_vectors: Vec::new(),
            hyperedges: Vec::new(),
            metadata,
            warnings,
        };

        for processor in &self.post_processors {
            let name = processor.name();
            debug!("Running post-processor: {name}");

            if let Err(e) = processor.process(&mut result, ds_result, &self.config, &id_map) {
                return Err(ExportError::StageError {
                    stage: "post_processing",
                    message: format!("post-processor '{name}' failed: {e}"),
                });
            }
        }

        // Finalize timing
        result.metadata.duration_ms = start.elapsed().as_millis() as u64;

        info!(
            "Pipeline complete: {} nodes, {} edges, {} warnings in {}ms",
            result.metadata.total_nodes,
            result.metadata.total_edges,
            result.warnings.len(),
            result.metadata.duration_ms
        );

        Ok(result)
    }
}

impl std::fmt::Debug for GraphExportPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphExportPipeline")
            .field("config", &self.config)
            .field("property_serializers", &self.property_serializers.len())
            .field("node_synthesizers", &self.node_synthesizers.len())
            .field("edge_synthesizers", &self.edge_synthesizers.len())
            .field("post_processors", &self.post_processors.len())
            .finish()
    }
}

// ──────────────────────────── Context Builders ──────────────────────

/// Build employee lookup maps from the generation result.
///
/// Returns:
/// - `employee_by_id`: external employee_id → display_name
/// - `employee_by_role`: job_title → employee_id (last one wins for duplicates)
/// - `user_to_employee`: user_id → employee_id
fn build_employee_maps(
    ds_result: &EnhancedGenerationResult,
) -> (
    HashMap<String, String>,
    HashMap<String, String>,
    HashMap<String, String>,
) {
    let employees = &ds_result.master_data.employees;
    let capacity = employees.len();

    let mut by_id = HashMap::with_capacity(capacity);
    let mut by_role = HashMap::with_capacity(capacity);
    let mut user_to_emp = HashMap::with_capacity(capacity);

    for emp in employees {
        by_id.insert(emp.employee_id.clone(), emp.display_name.clone());
        by_role.insert(emp.job_title.clone(), emp.employee_id.clone());
        user_to_emp.insert(emp.user_id.clone(), emp.employee_id.clone());
    }

    (by_id, by_role, user_to_emp)
}

/// Build opening balance map: account external_id → balance as f64.
fn build_opening_balance_map(ds_result: &EnhancedGenerationResult) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    for ob in &ds_result.opening_balances {
        for (account_id, balance) in &ob.balances {
            if let Some(f) = balance.to_f64() {
                map.insert(account_id.clone(), f);
            }
        }
    }
    map
}

/// Build risk name map: risk_ref → description.
fn build_risk_name_map(ds_result: &EnhancedGenerationResult) -> HashMap<String, String> {
    ds_result
        .audit
        .risk_assessments
        .iter()
        .map(|r| (r.risk_ref.clone(), r.description.clone()))
        .collect()
}

/// Build export metadata from the final nodes and edges.
fn build_metadata(
    nodes: &[crate::types::ExportNode],
    edges: &[crate::types::ExportEdge],
    start: Instant,
) -> ExportMetadata {
    let mut nodes_per_layer = [0usize; 3];
    for node in nodes {
        let idx = (node.layer as usize).saturating_sub(1).min(2);
        nodes_per_layer[idx] += 1;
    }

    let mut edge_types: HashSet<u32> = HashSet::new();
    for edge in edges {
        edge_types.insert(edge.edge_type);
    }
    let mut edge_types_produced: Vec<u32> = edge_types.into_iter().collect();
    edge_types_produced.sort_unstable();

    ExportMetadata {
        total_nodes: nodes.len(),
        total_edges: edges.len(),
        nodes_per_layer,
        edge_types_produced,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn empty_pipeline_produces_empty_result() {
        // We cannot construct a real EnhancedGenerationResult in a unit test
        // without pulling in the full generation machinery. The smoke test
        // in tests/pipeline_smoke.rs covers the constructor path.
        // Here we just verify the pipeline builds correctly.
        let pipeline = GraphExportPipeline::new(ExportConfig::default());
        assert_eq!(pipeline.node_synthesizers.len(), 0);
        assert_eq!(pipeline.edge_synthesizers.len(), 0);
        assert_eq!(pipeline.post_processors.len(), 0);
    }

    #[test]
    fn standard_returns_pipeline() {
        let pipeline = GraphExportPipeline::standard(ExportConfig::default());
        // Task 9: 30 property serializers + Task 14: 9 audit procedure serializers = 39.
        // v1.4.0: +2 (vendor, customer) = 41.
        assert_eq!(pipeline.property_serializers.len(), 41);
        // Task 12: 13 node synthesizers + Task 14: 1 audit procedures synthesizer + v1.3.0 = 15.
        assert_eq!(pipeline.node_synthesizers.len(), 15);
        // Task 13: 4 post-processors + 1 OCEL exporter = 5 total.
        assert_eq!(pipeline.post_processors.len(), 5);
    }

    #[test]
    fn builder_adds_stages() {
        use crate::traits::{
            EdgeSynthesisContext, EdgeSynthesizer, NodeSynthesisContext, NodeSynthesizer,
            PostProcessor,
        };
        use crate::types::{ExportEdge, ExportNode};

        struct TestNodeSynth;
        impl NodeSynthesizer for TestNodeSynth {
            fn name(&self) -> &'static str {
                "test_node"
            }
            fn synthesize(
                &self,
                _ctx: &mut NodeSynthesisContext<'_>,
            ) -> Result<Vec<ExportNode>, ExportError> {
                Ok(vec![])
            }
        }

        struct TestEdgeSynth;
        impl EdgeSynthesizer for TestEdgeSynth {
            fn name(&self) -> &'static str {
                "test_edge"
            }
            fn synthesize(
                &self,
                _ctx: &mut EdgeSynthesisContext<'_>,
            ) -> Result<Vec<ExportEdge>, ExportError> {
                Ok(vec![])
            }
        }

        struct TestPostProc;
        impl PostProcessor for TestPostProc {
            fn name(&self) -> &'static str {
                "test_post"
            }
            fn process(
                &self,
                _result: &mut GraphExportResult,
                _ds_result: &EnhancedGenerationResult,
                _config: &ExportConfig,
                _id_map: &IdMap,
            ) -> Result<(), ExportError> {
                Ok(())
            }
        }

        let pipeline = GraphExportPipeline::new(ExportConfig::default())
            .with_node_synthesizer(Box::new(TestNodeSynth))
            .with_edge_synthesizer(Box::new(TestEdgeSynth))
            .with_post_processor(Box::new(TestPostProc));

        assert_eq!(pipeline.node_synthesizers.len(), 1);
        assert_eq!(pipeline.edge_synthesizers.len(), 1);
        assert_eq!(pipeline.post_processors.len(), 1);
    }

    #[test]
    fn build_metadata_computes_correctly() {
        use crate::types::{ExportEdge, ExportNode};

        let nodes = vec![
            ExportNode {
                id: Some(1),
                node_type: 100,
                node_type_name: "a".into(),
                label: "A".into(),
                layer: 1,
                properties: HashMap::new(),
            },
            ExportNode {
                id: Some(2),
                node_type: 200,
                node_type_name: "b".into(),
                label: "B".into(),
                layer: 2,
                properties: HashMap::new(),
            },
            ExportNode {
                id: Some(3),
                node_type: 200,
                node_type_name: "c".into(),
                label: "C".into(),
                layer: 2,
                properties: HashMap::new(),
            },
            ExportNode {
                id: Some(4),
                node_type: 300,
                node_type_name: "d".into(),
                label: "D".into(),
                layer: 3,
                properties: HashMap::new(),
            },
        ];

        let edges = vec![
            ExportEdge {
                source: 1,
                target: 2,
                edge_type: 40,
                weight: 1.0,
                properties: HashMap::new(),
            },
            ExportEdge {
                source: 2,
                target: 3,
                edge_type: 60,
                weight: 1.0,
                properties: HashMap::new(),
            },
            ExportEdge {
                source: 3,
                target: 4,
                edge_type: 40,
                weight: 1.0,
                properties: HashMap::new(),
            },
        ];

        let start = Instant::now();
        let meta = build_metadata(&nodes, &edges, start);

        assert_eq!(meta.total_nodes, 4);
        assert_eq!(meta.total_edges, 3);
        assert_eq!(meta.nodes_per_layer, [1, 2, 1]);
        assert_eq!(meta.edge_types_produced, vec![40, 60]);
    }
}
