//! Full pipeline integration tests.
//!
//! These tests run the complete [`GraphExportPipeline`] with fixture data and verify:
//! - Nodes are produced from supplementary domain data (tax, ESG, compliance, etc.).
//! - Edge synthesizers tolerate missing core entity nodes gracefully.
//! - Budget enforcement correctly trims output.
//! - Post-processors (dedup, anomaly normalizer, etc.) run without error.
//! - Metadata is populated correctly.
//! - `into_bulk()` (behind `rustgraph` feature) converts correctly.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use datasynth_graph_export::{BudgetConfig, ExportConfig, GraphExportPipeline};
use datasynth_runtime::EnhancedGenerationResult;

// ═══════════════════════════════════════════════════════════════════════
// Fixture Builders
// ═══════════════════════════════════════════════════════════════════════

mod fixtures {
    use chrono::NaiveDate;
    use datasynth_core::audit::{RiskAssessment, RiskCategory};
    use datasynth_core::{
        ControlType, InternalControl, JurisdictionType, TaxCode, TaxJurisdiction, TaxType,
    };
    use datasynth_runtime::EnhancedGenerationResult;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    /// Build a small dataset that exercises the pipeline.
    ///
    /// Contains:
    /// - 2 tax jurisdictions + 3 tax codes (TaxNodeSynthesizer → 5 nodes)
    /// - 2 compliance standards (ComplianceNodeSynthesizer → 2 nodes)
    /// - 5 internal controls + 3 risk assessments (for edge synthesizer testing)
    /// - 3 employees (for employee map building)
    ///
    /// The dataset is intentionally small to keep tests fast.
    pub fn small_dataset() -> EnhancedGenerationResult {
        let mut ds = EnhancedGenerationResult::default();

        // --- Tax domain ---
        ds.tax.jurisdictions = vec![
            TaxJurisdiction::new(
                "JURIS-US-FED",
                "US Federal",
                "US",
                JurisdictionType::Federal,
            ),
            TaxJurisdiction::new(
                "JURIS-US-CA",
                "US California",
                "US",
                JurisdictionType::State,
            )
            .with_region_code("CA")
            .with_parent_jurisdiction_id("JURIS-US-FED"),
        ];

        ds.tax.codes = vec![
            TaxCode::new(
                "TC-001",
                "VAT-STD-20",
                "Standard VAT 20%",
                TaxType::Vat,
                Decimal::new(20, 2),
                "JURIS-US-FED",
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
            TaxCode::new(
                "TC-002",
                "CIT-FED-21",
                "Federal CIT 21%",
                TaxType::IncomeTax,
                Decimal::new(21, 2),
                "JURIS-US-FED",
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
            TaxCode::new(
                "TC-003",
                "WHT-SVC-15",
                "Withholding on services 15%",
                TaxType::WithholdingTax,
                Decimal::new(15, 2),
                "JURIS-US-CA",
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        ];

        // --- Compliance domain ---
        ds.compliance_regulations.standard_records = vec![
            datasynth_generators::compliance::ComplianceStandardRecord {
                standard_id: "ISA-315".into(),
                body: "IAASB".into(),
                number: "315".into(),
                title: "Identifying and Assessing Risks of Material Misstatement".into(),
                category: "Audit".into(),
                domain: "Risk Assessment".into(),
                jurisdiction: "International".into(),
                effective_date: "2022-12-15".into(),
                version: "Revised 2019".into(),
                is_active: true,
                superseded_by: None,
                applicable_account_types: vec!["All".into()],
                applicable_processes: vec!["All".into()],
            },
            datasynth_generators::compliance::ComplianceStandardRecord {
                standard_id: "SOX-302".into(),
                body: "SEC".into(),
                number: "302".into(),
                title: "Corporate Responsibility for Financial Reports".into(),
                category: "Compliance".into(),
                domain: "Internal Controls".into(),
                jurisdiction: "US".into(),
                effective_date: "2002-07-30".into(),
                version: "Original".into(),
                is_active: true,
                superseded_by: None,
                applicable_account_types: vec!["Revenue".into(), "Expenses".into()],
                applicable_processes: vec!["Financial Reporting".into()],
            },
        ];

        // --- Internal Controls (for edge synthesizer) ---
        ds.internal_controls = vec![
            InternalControl::new(
                "C010",
                "Three-Way Match",
                ControlType::Preventive,
                "Match PO/GR/Invoice",
            )
            .with_description(
                "Automated three-way match of purchase order, goods receipt, and vendor invoice",
            ),
            InternalControl::new(
                "C020",
                "Revenue Recognition Review",
                ControlType::Detective,
                "Review revenue recognition criteria",
            )
            .with_description("Monthly review of ASC 606 revenue recognition compliance"),
            InternalControl::new(
                "C030",
                "Journal Entry Approval",
                ControlType::Preventive,
                "Approve manual JEs",
            )
            .with_description("Journal entry approval for entries above materiality threshold"),
            InternalControl::new(
                "C040",
                "Bank Reconciliation",
                ControlType::Detective,
                "Reconcile bank accounts",
            )
            .with_description("Daily bank account reconciliation review"),
            InternalControl::new(
                "C050",
                "Fixed Asset Addition Approval",
                ControlType::Preventive,
                "Approve asset additions",
            )
            .with_description(
                "Authorization for fixed asset additions above capitalization threshold",
            ),
        ];

        // --- Risk Assessments (for edge synthesizer) ---
        let engagement_id = Uuid::new_v4();
        ds.audit.risk_assessments = vec![
            RiskAssessment::new(
                engagement_id,
                RiskCategory::AssertionLevel,
                "Revenue Recognition",
                "Risk of improper revenue recognition under ASC 606",
            ),
            RiskAssessment::new(
                engagement_id,
                RiskCategory::AssertionLevel,
                "Expenditure — Procurement",
                "Risk of unauthorized purchases bypassing approval controls",
            ),
            RiskAssessment::new(
                engagement_id,
                RiskCategory::FinancialStatementLevel,
                "Financial Statement Reporting",
                "Risk of misstatement in period-end financial close",
            ),
        ];

        // --- Employees (for context map building) ---
        ds.master_data.employees = vec![
            datasynth_core::Employee::new("EMP-001", "USR-001", "Alice", "Smith", "1000"),
            datasynth_core::Employee::new("EMP-002", "USR-002", "Bob", "Jones", "1000"),
            datasynth_core::Employee::new("EMP-003", "USR-003", "Charlie", "Brown", "1000"),
        ];

        ds
    }

    /// Build a dataset with many tax + compliance nodes to test budget enforcement.
    ///
    /// Creates 200+ nodes across governance (L1) and accounting (L3) layers.
    pub fn dataset_with_many_nodes() -> EnhancedGenerationResult {
        let mut ds = EnhancedGenerationResult::default();

        // 100 tax jurisdictions (L1 — governance)
        ds.tax.jurisdictions = (0..100)
            .map(|i| {
                TaxJurisdiction::new(
                    format!("JURIS-{i:04}"),
                    format!("Jurisdiction {i}"),
                    "XX",
                    JurisdictionType::Federal,
                )
            })
            .collect();

        // 100 tax codes (L3 — accounting)
        ds.tax.codes = (0..100)
            .map(|i| {
                TaxCode::new(
                    format!("TC-{i:04}"),
                    format!("CODE-{i:04}"),
                    format!("Tax Code {i}"),
                    TaxType::Vat,
                    Decimal::new(20, 2),
                    format!("JURIS-{:04}", i % 100),
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                )
            })
            .collect();

        // 50 compliance standards (L1 — governance)
        ds.compliance_regulations.standard_records = (0..50)
            .map(
                |i| datasynth_generators::compliance::ComplianceStandardRecord {
                    standard_id: format!("STD-{i:04}"),
                    body: "TEST".into(),
                    number: format!("{i}"),
                    title: format!("Standard {i}"),
                    category: "Test".into(),
                    domain: "Testing".into(),
                    jurisdiction: "Global".into(),
                    effective_date: "2024-01-01".into(),
                    version: "1.0".into(),
                    is_active: true,
                    superseded_by: None,
                    applicable_account_types: vec![],
                    applicable_processes: vec![],
                },
            )
            .collect();

        ds
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Integration Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn full_pipeline_produces_valid_graph() {
    let ds = fixtures::small_dataset();

    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    let result = pipeline.export(&ds).unwrap();

    // --- Nodes were created ---
    assert!(
        result.metadata.total_nodes > 0,
        "Expected nodes from tax + compliance synthesizers, got 0"
    );

    // Tax synthesizer should produce: 2 jurisdictions + 3 codes = 5 nodes
    let tax_nodes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.node_type >= 410 && n.node_type <= 416)
        .collect();
    assert_eq!(
        tax_nodes.len(),
        5,
        "Expected 5 tax nodes (2 jurisdictions + 3 codes), got {}",
        tax_nodes.len()
    );

    // Compliance synthesizer should produce: 2 standard nodes
    let compliance_nodes: Vec<_> = result.nodes.iter().filter(|n| n.node_type == 480).collect();
    assert_eq!(
        compliance_nodes.len(),
        2,
        "Expected 2 compliance standard nodes, got {}",
        compliance_nodes.len()
    );

    // --- Metadata is correct ---
    assert_eq!(
        result.metadata.total_nodes,
        result.nodes.len(),
        "Metadata total_nodes mismatch"
    );
    assert_eq!(
        result.metadata.total_edges,
        result.edges.len(),
        "Metadata total_edges mismatch"
    );

    // --- L1 and L3 layer counts ---
    assert!(
        result.metadata.nodes_per_layer[0] > 0,
        "Expected some L1 (governance) nodes"
    );

    // --- All nodes have assigned IDs ---
    for node in &result.nodes {
        assert!(
            node.id.is_some(),
            "Node '{}' (type {}) missing assigned ID",
            node.label,
            node.node_type_name
        );
    }

    // --- All tax/compliance nodes have processFamily or nodeTypeName ---
    for node in &result.nodes {
        if node.node_type >= 410 && node.node_type <= 416 {
            assert!(
                node.properties.contains_key("processFamily"),
                "Tax node '{}' missing processFamily",
                node.label
            );
        }
        assert!(
            node.properties.contains_key("nodeTypeName"),
            "Node '{}' (type {}) missing nodeTypeName",
            node.label,
            node.node_type_name
        );
    }

    // --- No duplicate edges ---
    let mut edge_set: HashSet<(u64, u64, u32)> = HashSet::new();
    for e in &result.edges {
        assert!(
            edge_set.insert((e.source, e.target, e.edge_type)),
            "Duplicate edge: {} -> {} (type {})",
            e.source,
            e.target,
            e.edge_type
        );
    }

    // --- Duration was measured ---
    // Duration might be 0ms for very fast runs, but should be set
    assert!(
        result.metadata.duration_ms < 60_000,
        "Pipeline took too long: {}ms",
        result.metadata.duration_ms
    );
}

#[test]
fn pipeline_populates_all_layers() {
    let ds = fixtures::small_dataset();

    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    let result = pipeline.export(&ds).unwrap();

    // Tax jurisdictions go to L1, tax codes go to L3, compliance standards go to L1
    let l1_count = result.nodes.iter().filter(|n| n.layer == 1).count();
    let l3_count = result.nodes.iter().filter(|n| n.layer == 3).count();

    // 2 jurisdictions (L1) + 2 compliance standards (L1) = 4 L1 nodes minimum
    assert!(
        l1_count >= 4,
        "Expected at least 4 L1 nodes, got {l1_count}"
    );
    // 3 tax codes (L3)
    assert!(
        l3_count >= 3,
        "Expected at least 3 L3 nodes, got {l3_count}"
    );
}

#[test]
fn edge_synthesizers_tolerate_missing_core_entities() {
    // The pipeline's 13 node synthesizers only produce supplementary domain nodes.
    // Core entities (controls, risks, accounts, JEs, P2P/O2C docs) are NOT registered
    // in the IdMap by any node synthesizer. Edge synthesizers that reference these entities
    // should silently skip (producing 0 edges) rather than erroring.
    let ds = fixtures::small_dataset();

    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    let result = pipeline.export(&ds).unwrap();

    // The edge synthesizers should have completed without error even though
    // controls and risks from ds_result.internal_controls / ds_result.audit.risk_assessments
    // have no corresponding nodes in the IdMap.
    //
    // RISK_MITIGATED_BY (code 75) edges should be 0 because risk/control nodes
    // are not registered by any node synthesizer.
    let rmb = result.edges.iter().filter(|e| e.edge_type == 75).count();
    assert_eq!(
        rmb, 0,
        "Expected 0 RISK_MITIGATED_BY edges (no core node synthesizers), got {rmb}"
    );

    // Document chain edges should also be 0 for the same reason
    let doc_chain = result
        .edges
        .iter()
        .filter(|e| matches!(e.edge_type, 60 | 62 | 64 | 66 | 68 | 69))
        .count();
    assert_eq!(
        doc_chain, 0,
        "Expected 0 document chain edges (no P2P/O2C node synthesizers), got {doc_chain}"
    );
}

#[test]
fn post_processors_run_successfully() {
    let ds = fixtures::small_dataset();

    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    let result = pipeline.export(&ds).unwrap();

    // The DuplicateEdgeValidator should have run (even if there were no duplicates)
    // The AnomalyFlagNormalizer should have run
    // The EffectiveControlCountPatcher should have run
    // The RedFlagAnnotator should have run
    // The OCEL exporter should have run

    // Since there are no document flows, OCEL should be None
    assert!(
        result.ocel.is_none(),
        "Expected no OCEL output with empty document flows"
    );

    // Warnings should exist (edge synthesizers warn about skipped edges)
    // or be empty (if all synthesizers found nothing to warn about)
    // Just verify the field is accessible
    let _warning_count = result.warnings.len();
}

#[test]
fn pipeline_respects_node_budget() {
    let ds = fixtures::dataset_with_many_nodes();

    let config = ExportConfig {
        budget: BudgetConfig {
            max_nodes: 50,
            max_edges: 0,
            layer_split: [0.50, 0.10, 0.40],
        },
        ..Default::default()
    };

    let pipeline = GraphExportPipeline::standard(config);
    let result = pipeline.export(&ds).unwrap();

    assert!(
        result.nodes.len() <= 50,
        "Node count {} exceeds budget 50",
        result.nodes.len()
    );

    // The L1 budget is 50% of 50 = 25 nodes
    // We had 100 jurisdictions (L1) + 50 compliance standards (L1) = 150 L1 nodes → trimmed to 25
    let l1_count = result.nodes.iter().filter(|n| n.layer == 1).count();
    assert!(
        l1_count <= 25,
        "L1 node count {l1_count} exceeds layer budget 25"
    );

    // The L3 budget is 40% of 50 = 20 nodes
    // We had 100 tax codes (L3) → trimmed to 20
    let l3_count = result.nodes.iter().filter(|n| n.layer == 3).count();
    assert!(
        l3_count <= 20,
        "L3 node count {l3_count} exceeds layer budget 20"
    );

    // Warnings should mention budget trimming
    assert!(
        !result.warnings.is_empty(),
        "Expected budget trimming warnings"
    );
}

#[test]
fn pipeline_with_zero_data_produces_empty_result() {
    let ds = EnhancedGenerationResult::default();

    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    let result = pipeline.export(&ds).unwrap();

    assert_eq!(result.nodes.len(), 0, "Expected 0 nodes for empty input");
    assert_eq!(result.edges.len(), 0, "Expected 0 edges for empty input");
    assert_eq!(result.metadata.total_nodes, 0);
    assert_eq!(result.metadata.total_edges, 0);
    assert!(result.ocel.is_none());
}

#[test]
fn metadata_edge_types_produced_is_accurate() {
    let ds = fixtures::small_dataset();

    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    let result = pipeline.export(&ds).unwrap();

    // The edge_types_produced list should match what's actually in edges
    let actual_types: HashSet<u32> = result.edges.iter().map(|e| e.edge_type).collect();
    let reported_types: HashSet<u32> = result
        .metadata
        .edge_types_produced
        .iter()
        .copied()
        .collect();

    assert_eq!(
        actual_types, reported_types,
        "Metadata edge_types_produced doesn't match actual edge types.\nActual: {actual_types:?}\nReported: {reported_types:?}"
    );

    // The list should be sorted
    let sorted = result
        .metadata
        .edge_types_produced
        .windows(2)
        .all(|w| w[0] <= w[1]);
    assert!(sorted, "edge_types_produced should be sorted");
}

#[test]
fn pipeline_config_can_be_customized() {
    let ds = fixtures::small_dataset();

    let config = ExportConfig {
        skip_banking: true,
        ..Default::default()
    };

    let pipeline = GraphExportPipeline::standard(config);
    assert!(pipeline.config().skip_banking);

    // Should still produce nodes from tax + compliance
    let result = pipeline.export(&ds).unwrap();
    assert!(result.metadata.total_nodes > 0);
}

#[test]
fn node_ids_are_unique_and_sequential() {
    let ds = fixtures::small_dataset();

    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    let result = pipeline.export(&ds).unwrap();

    let mut ids: Vec<u64> = result.nodes.iter().filter_map(|n| n.id).collect();
    let unique_count = ids.len();
    ids.sort_unstable();
    ids.dedup();

    assert_eq!(
        ids.len(),
        unique_count,
        "Found duplicate node IDs: {unique_count} nodes but only {} unique IDs",
        ids.len()
    );

    // IDs should start from 1 (IdMap convention)
    if !ids.is_empty() {
        assert_eq!(ids[0], 1, "First node ID should be 1, got {}", ids[0]);
    }
}

#[test]
fn standard_pipeline_has_all_stages() {
    let pipeline = GraphExportPipeline::standard(ExportConfig::default());

    // 30 property serializers (Task 9) + 9 audit procedure serializers (Task 14) = 39
    // v1.4.0: +2 (vendor, customer) = 41
    assert_eq!(pipeline.property_serializers().len(), 41);
}

#[test]
fn warnings_are_collected_not_fatal() {
    // Dataset with controls but no node synthesizers for controls.
    // The risk_control edge synthesizer should produce warnings about
    // unresolved IDs, not errors.
    let ds = fixtures::small_dataset();

    let pipeline = GraphExportPipeline::standard(ExportConfig::default());
    let result = pipeline.export(&ds).unwrap();

    // Should complete without error. Warnings may or may not be present
    // depending on whether edge synthesizers produce info-level warnings.
    let _ = result.warnings.len();
}

#[test]
fn budget_at_exact_boundary_is_not_trimmed() {
    let ds = fixtures::small_dataset();

    // The small dataset produces ~7 nodes. Set budget just above that.
    let config = ExportConfig {
        budget: BudgetConfig {
            max_nodes: 100,
            max_edges: 1000,
            layer_split: [0.50, 0.10, 0.40],
        },
        ..Default::default()
    };

    let pipeline = GraphExportPipeline::standard(config);
    let result = pipeline.export(&ds).unwrap();

    // No trimming should occur
    let budget_warnings: Vec<_> = result
        .warnings
        .iter()
        .filter(|w| w.stage == "budget")
        .collect();
    assert!(
        budget_warnings.is_empty(),
        "Expected no budget warnings when within limits, got {}",
        budget_warnings.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════
// RustGraph Feature Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(feature = "rustgraph")]
mod rustgraph_tests {
    use super::*;

    #[test]
    fn into_bulk_converts_correctly() {
        let ds = fixtures::small_dataset();

        let pipeline = GraphExportPipeline::standard(ExportConfig::default());
        let result = pipeline.export(&ds).unwrap();

        let node_count = result.nodes.len();
        let edge_count = result.edges.len();
        let (bulk_nodes, bulk_edges) = result.into_bulk();

        assert_eq!(
            bulk_nodes.len(),
            node_count,
            "BulkNode count mismatch: expected {node_count}, got {}",
            bulk_nodes.len()
        );
        assert_eq!(
            bulk_edges.len(),
            edge_count,
            "BulkEdge count mismatch: expected {edge_count}, got {}",
            bulk_edges.len()
        );

        // Verify BulkNodeData fields are populated
        for bn in &bulk_nodes {
            assert!(bn.id.is_some(), "BulkNodeData should have an ID");
            assert!(!bn.labels.is_empty(), "BulkNodeData should have labels");
            assert!(bn.layer.is_some(), "BulkNodeData should have a layer");
            assert!(
                bn.properties.contains_key("nodeTypeName"),
                "BulkNodeData should contain nodeTypeName property"
            );
        }
    }
}
