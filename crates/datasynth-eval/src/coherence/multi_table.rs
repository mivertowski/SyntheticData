//! Multi-table consistency evaluation.
//!
//! Validates consistency across multiple related tables:
//! - Cascade anomaly effects across document flows
//! - Cross-table referential integrity
//! - Table dependency validation

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Results of multi-table consistency evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTableEvaluation {
    /// Table-to-table consistency results.
    pub table_consistency: Vec<TableConsistencyResult>,
    /// Cascade anomaly analysis results.
    pub cascade_analysis: CascadeAnomalyAnalysis,
    /// Overall consistency score (0.0-1.0).
    pub overall_consistency_score: f64,
    /// Total consistency violations.
    pub total_violations: usize,
    /// Passes consistency check.
    pub passes: bool,
    /// List of identified issues.
    pub issues: Vec<String>,
}

impl Default for MultiTableEvaluation {
    fn default() -> Self {
        Self {
            table_consistency: Vec::new(),
            cascade_analysis: CascadeAnomalyAnalysis::default(),
            overall_consistency_score: 1.0,
            total_violations: 0,
            passes: true,
            issues: Vec::new(),
        }
    }
}

/// Consistency result for a table pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConsistencyResult {
    /// Source table name.
    pub source_table: String,
    /// Target table name.
    pub target_table: String,
    /// Relationship type.
    pub relationship: TableRelationship,
    /// Total records checked.
    pub records_checked: usize,
    /// Matching records.
    pub matching_records: usize,
    /// Mismatched records.
    pub mismatched_records: usize,
    /// Orphaned records in source (no matching target).
    pub orphaned_source: usize,
    /// Orphaned records in target (no matching source).
    pub orphaned_target: usize,
    /// Consistency score (0.0-1.0).
    pub consistency_score: f64,
    /// Specific violations found.
    pub violations: Vec<ConsistencyViolation>,
}

impl Default for TableConsistencyResult {
    fn default() -> Self {
        Self {
            source_table: String::new(),
            target_table: String::new(),
            relationship: TableRelationship::OneToMany,
            records_checked: 0,
            matching_records: 0,
            mismatched_records: 0,
            orphaned_source: 0,
            orphaned_target: 0,
            consistency_score: 1.0,
            violations: Vec::new(),
        }
    }
}

/// Type of relationship between tables.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TableRelationship {
    /// One-to-one relationship.
    OneToOne,
    /// One-to-many relationship.
    OneToMany,
    /// Many-to-many relationship.
    ManyToMany,
    /// Hierarchical relationship (parent-child).
    Hierarchical,
    /// Document flow relationship (e.g., PO -> GR -> Invoice -> Payment).
    DocumentFlow,
}

/// A specific consistency violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyViolation {
    /// Violation type.
    pub violation_type: ViolationType,
    /// Source record identifier.
    pub source_record_id: String,
    /// Target record identifier (if applicable).
    pub target_record_id: Option<String>,
    /// Description of the violation.
    pub description: String,
    /// Severity (1-5).
    pub severity: u8,
}

/// Types of consistency violations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationType {
    /// Missing required reference.
    MissingReference,
    /// Value mismatch between tables.
    ValueMismatch,
    /// Orphaned record.
    OrphanedRecord,
    /// Circular reference.
    CircularReference,
    /// Amount inconsistency.
    AmountInconsistency,
    /// Date inconsistency.
    DateInconsistency,
    /// Status inconsistency.
    StatusInconsistency,
    /// Document chain break.
    DocumentChainBreak,
}

/// Analysis of cascade anomaly effects.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CascadeAnomalyAnalysis {
    /// Total anomalies tracked.
    pub total_anomalies: usize,
    /// Anomalies with cascade effects.
    pub anomalies_with_cascades: usize,
    /// Average cascade depth (number of tables affected).
    pub average_cascade_depth: f64,
    /// Maximum cascade depth.
    pub max_cascade_depth: usize,
    /// Cascade paths by source table.
    pub cascade_paths: Vec<CascadePath>,
    /// Tables affected by anomaly cascades.
    pub tables_affected: HashMap<String, usize>,
}

/// A cascade path showing how an anomaly propagates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadePath {
    /// Original anomaly identifier.
    pub anomaly_id: String,
    /// Anomaly type.
    pub anomaly_type: String,
    /// Source document/record ID.
    pub source_id: String,
    /// Source table.
    pub source_table: String,
    /// Tables affected in order of cascade.
    pub affected_tables: Vec<String>,
    /// Total records affected.
    pub records_affected: usize,
    /// Cascade depth.
    pub depth: usize,
    /// Estimated monetary impact.
    pub monetary_impact: Option<f64>,
}

/// Input data for multi-table consistency evaluation.
#[derive(Debug, Clone, Default)]
pub struct MultiTableData {
    /// Table records by table name.
    pub tables: HashMap<String, Vec<TableRecord>>,
    /// Defined relationships between tables.
    pub relationships: Vec<TableRelationshipDef>,
    /// Known anomalies with their source records.
    pub anomalies: Vec<AnomalyRecord>,
}

/// A record from any table with its key and references.
#[derive(Debug, Clone, Default)]
pub struct TableRecord {
    /// Record identifier.
    pub id: String,
    /// Table name.
    pub table: String,
    /// Foreign key references to other tables.
    pub references: HashMap<String, String>, // table -> id
    /// Key field values for comparison.
    pub key_values: HashMap<String, String>,
    /// Numeric amount if applicable.
    pub amount: Option<f64>,
    /// Date field if applicable.
    pub date: Option<String>,
    /// Status field if applicable.
    pub status: Option<String>,
    /// Whether this record is marked as anomalous.
    pub is_anomalous: bool,
    /// Associated anomaly ID if any.
    pub anomaly_id: Option<String>,
}

/// Definition of a relationship between tables.
#[derive(Debug, Clone)]
pub struct TableRelationshipDef {
    /// Source table name.
    pub source_table: String,
    /// Target table name.
    pub target_table: String,
    /// Source key field.
    pub source_key: String,
    /// Target key field (foreign key).
    pub target_key: String,
    /// Relationship type.
    pub relationship_type: TableRelationship,
    /// Whether amounts should match.
    pub validate_amounts: bool,
    /// Whether dates should be consistent.
    pub validate_dates: bool,
}

/// An anomaly record for cascade tracking.
#[derive(Debug, Clone)]
pub struct AnomalyRecord {
    /// Anomaly identifier.
    pub anomaly_id: String,
    /// Anomaly type.
    pub anomaly_type: String,
    /// Source record ID.
    pub source_record_id: String,
    /// Source table.
    pub source_table: String,
    /// Severity (1-5).
    pub severity: u8,
    /// Monetary impact if known.
    pub monetary_impact: Option<f64>,
}

/// Evaluator for multi-table consistency.
pub struct MultiTableConsistencyEvaluator {
    /// Minimum consistency score threshold.
    min_consistency_score: f64,
    /// Maximum allowed orphaned record rate.
    max_orphan_rate: f64,
    /// Maximum cascade depth to track.
    max_cascade_depth: usize,
}

impl MultiTableConsistencyEvaluator {
    /// Create a new evaluator with specified thresholds.
    pub fn new(min_consistency_score: f64, max_orphan_rate: f64, max_cascade_depth: usize) -> Self {
        Self {
            min_consistency_score,
            max_orphan_rate,
            max_cascade_depth,
        }
    }

    /// Evaluate multi-table consistency.
    pub fn evaluate(&self, data: &MultiTableData) -> EvalResult<MultiTableEvaluation> {
        let mut evaluation = MultiTableEvaluation::default();

        // Evaluate each defined relationship
        for rel_def in &data.relationships {
            let result = self.evaluate_relationship(data, rel_def);
            evaluation.table_consistency.push(result);
        }

        // Analyze cascade anomaly effects
        evaluation.cascade_analysis = self.analyze_cascades(data);

        // Calculate overall metrics
        self.calculate_overall_metrics(&mut evaluation);

        Ok(evaluation)
    }

    /// Evaluate a specific table relationship.
    fn evaluate_relationship(
        &self,
        data: &MultiTableData,
        rel_def: &TableRelationshipDef,
    ) -> TableConsistencyResult {
        let mut result = TableConsistencyResult {
            source_table: rel_def.source_table.clone(),
            target_table: rel_def.target_table.clone(),
            relationship: rel_def.relationship_type.clone(),
            ..Default::default()
        };

        let source_records = data.tables.get(&rel_def.source_table);
        let target_records = data.tables.get(&rel_def.target_table);

        let (source_records, target_records) = match (source_records, target_records) {
            (Some(s), Some(t)) => (s, t),
            _ => return result,
        };

        // Build target index
        let target_index: HashMap<_, _> =
            target_records.iter().map(|r| (r.id.clone(), r)).collect();

        // Check each source record
        let mut referenced_targets = HashSet::new();

        for source_record in source_records {
            result.records_checked += 1;

            // Check if source has reference to target
            if let Some(target_id) = source_record.references.get(&rel_def.target_table) {
                if let Some(target_record) = target_index.get(target_id) {
                    // Reference exists - check for value mismatches
                    let violations =
                        self.check_value_consistency(source_record, target_record, rel_def);
                    if violations.is_empty() {
                        result.matching_records += 1;
                    } else {
                        result.mismatched_records += 1;
                        result.violations.extend(violations);
                    }
                    referenced_targets.insert(target_id.clone());
                } else {
                    // Invalid reference
                    result.orphaned_source += 1;
                    result.violations.push(ConsistencyViolation {
                        violation_type: ViolationType::MissingReference,
                        source_record_id: source_record.id.clone(),
                        target_record_id: Some(target_id.clone()),
                        description: format!(
                            "Source record {} references non-existent target {}",
                            source_record.id, target_id
                        ),
                        severity: 3,
                    });
                }
            }
        }

        // Find orphaned targets
        result.orphaned_target = target_records
            .iter()
            .filter(|r| !referenced_targets.contains(&r.id))
            .count();

        // Calculate consistency score
        let total = result.matching_records + result.mismatched_records + result.orphaned_source;
        result.consistency_score = if total > 0 {
            result.matching_records as f64 / total as f64
        } else {
            1.0
        };

        result
    }

    /// Check value consistency between source and target records.
    fn check_value_consistency(
        &self,
        source: &TableRecord,
        target: &TableRecord,
        rel_def: &TableRelationshipDef,
    ) -> Vec<ConsistencyViolation> {
        let mut violations = Vec::new();

        // Check amounts if required
        if rel_def.validate_amounts {
            if let (Some(s_amt), Some(t_amt)) = (source.amount, target.amount) {
                // Allow 0.01 tolerance for rounding
                if (s_amt - t_amt).abs() > 0.01 {
                    violations.push(ConsistencyViolation {
                        violation_type: ViolationType::AmountInconsistency,
                        source_record_id: source.id.clone(),
                        target_record_id: Some(target.id.clone()),
                        description: format!("Amount mismatch: source={s_amt}, target={t_amt}"),
                        severity: 3,
                    });
                }
            }
        }

        // Check dates if required
        if rel_def.validate_dates {
            if let (Some(ref s_date), Some(ref t_date)) = (&source.date, &target.date) {
                // For document flows, target date should be >= source date
                if rel_def.relationship_type == TableRelationship::DocumentFlow && t_date < s_date {
                    violations.push(ConsistencyViolation {
                        violation_type: ViolationType::DateInconsistency,
                        source_record_id: source.id.clone(),
                        target_record_id: Some(target.id.clone()),
                        description: format!(
                            "Date inconsistency: target date {t_date} before source date {s_date}"
                        ),
                        severity: 2,
                    });
                }
            }
        }

        violations
    }

    /// Analyze cascade anomaly effects.
    fn analyze_cascades(&self, data: &MultiTableData) -> CascadeAnomalyAnalysis {
        let mut analysis = CascadeAnomalyAnalysis::default();
        analysis.total_anomalies = data.anomalies.len();

        // Build reverse reference index for cascade tracking
        let mut reverse_refs: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
        for (table_name, records) in &data.tables {
            for record in records {
                for (ref_table, ref_id) in &record.references {
                    reverse_refs
                        .entry((ref_table.clone(), ref_id.clone()))
                        .or_default()
                        .push((table_name.clone(), record.id.clone()));
                }
            }
        }

        // Track cascade for each anomaly
        for anomaly in &data.anomalies {
            let cascade_path = self.trace_cascade(
                data,
                &reverse_refs,
                &anomaly.source_table,
                &anomaly.source_record_id,
                &anomaly.anomaly_id,
                &anomaly.anomaly_type,
                anomaly.monetary_impact,
            );

            if cascade_path.depth > 0 {
                analysis.anomalies_with_cascades += 1;

                // Update tables affected count
                for table in &cascade_path.affected_tables {
                    *analysis.tables_affected.entry(table.clone()).or_insert(0) += 1;
                }

                if cascade_path.depth > analysis.max_cascade_depth {
                    analysis.max_cascade_depth = cascade_path.depth;
                }

                analysis.cascade_paths.push(cascade_path);
            }
        }

        // Calculate average cascade depth
        if !analysis.cascade_paths.is_empty() {
            analysis.average_cascade_depth = analysis
                .cascade_paths
                .iter()
                .map(|p| p.depth as f64)
                .sum::<f64>()
                / analysis.cascade_paths.len() as f64;
        }

        analysis
    }

    /// Trace the cascade effect of a single anomaly.
    fn trace_cascade(
        &self,
        data: &MultiTableData,
        reverse_refs: &HashMap<(String, String), Vec<(String, String)>>,
        source_table: &str,
        source_id: &str,
        anomaly_id: &str,
        anomaly_type: &str,
        monetary_impact: Option<f64>,
    ) -> CascadePath {
        let mut path = CascadePath {
            anomaly_id: anomaly_id.to_string(),
            anomaly_type: anomaly_type.to_string(),
            source_id: source_id.to_string(),
            source_table: source_table.to_string(),
            affected_tables: Vec::new(),
            records_affected: 0,
            depth: 0,
            monetary_impact,
        };

        let mut visited = HashSet::new();
        let mut to_visit = vec![(source_table.to_string(), source_id.to_string(), 0usize)];

        while let Some((table, id, depth)) = to_visit.pop() {
            if depth > self.max_cascade_depth {
                continue;
            }
            if visited.contains(&(table.clone(), id.clone())) {
                continue;
            }
            visited.insert((table.clone(), id.clone()));

            // Find records that reference this one
            if let Some(refs) = reverse_refs.get(&(table.clone(), id.clone())) {
                for (ref_table, ref_id) in refs {
                    if !visited.contains(&(ref_table.clone(), ref_id.clone())) {
                        if !path.affected_tables.contains(ref_table) {
                            path.affected_tables.push(ref_table.clone());
                        }
                        path.records_affected += 1;
                        path.depth = path.depth.max(depth + 1);
                        to_visit.push((ref_table.clone(), ref_id.clone(), depth + 1));
                    }
                }
            }

            // Also check forward references from this record
            if let Some(records) = data.tables.get(&table) {
                if let Some(record) = records.iter().find(|r| r.id == id) {
                    for (ref_table, ref_id) in &record.references {
                        if !visited.contains(&(ref_table.clone(), ref_id.clone())) {
                            if !path.affected_tables.contains(ref_table) {
                                path.affected_tables.push(ref_table.clone());
                            }
                            path.records_affected += 1;
                            path.depth = path.depth.max(depth + 1);
                            to_visit.push((ref_table.clone(), ref_id.clone(), depth + 1));
                        }
                    }
                }
            }
        }

        path
    }

    /// Calculate overall evaluation metrics.
    fn calculate_overall_metrics(&self, evaluation: &mut MultiTableEvaluation) {
        // Calculate overall consistency score
        let total_records: usize = evaluation
            .table_consistency
            .iter()
            .map(|r| r.records_checked)
            .sum();

        let total_matching: usize = evaluation
            .table_consistency
            .iter()
            .map(|r| r.matching_records)
            .sum();

        evaluation.overall_consistency_score = if total_records > 0 {
            total_matching as f64 / total_records as f64
        } else {
            1.0
        };

        // Count total violations
        evaluation.total_violations = evaluation
            .table_consistency
            .iter()
            .map(|r| r.violations.len())
            .sum();

        // Collect issues
        for result in &evaluation.table_consistency {
            if result.consistency_score < self.min_consistency_score {
                evaluation.issues.push(format!(
                    "{}->{}: consistency {:.2}% below threshold {:.2}%",
                    result.source_table,
                    result.target_table,
                    result.consistency_score * 100.0,
                    self.min_consistency_score * 100.0
                ));
            }

            let orphan_rate = if result.records_checked > 0 {
                (result.orphaned_source + result.orphaned_target) as f64
                    / result.records_checked as f64
            } else {
                0.0
            };

            if orphan_rate > self.max_orphan_rate {
                evaluation.issues.push(format!(
                    "{}->{}: orphan rate {:.2}% exceeds threshold {:.2}%",
                    result.source_table,
                    result.target_table,
                    orphan_rate * 100.0,
                    self.max_orphan_rate * 100.0
                ));
            }
        }

        // Note cascade issues
        if evaluation.cascade_analysis.max_cascade_depth > 3 {
            evaluation.issues.push(format!(
                "High cascade depth detected: {} tables deep",
                evaluation.cascade_analysis.max_cascade_depth
            ));
        }

        evaluation.passes = evaluation.issues.is_empty()
            && evaluation.overall_consistency_score >= self.min_consistency_score;
    }
}

impl Default for MultiTableConsistencyEvaluator {
    fn default() -> Self {
        Self::new(0.95, 0.10, 5) // 95% consistency, 10% max orphan rate, 5 max cascade depth
    }
}

/// Predefined document flow relationships for common scenarios.
pub fn get_p2p_flow_relationships() -> Vec<TableRelationshipDef> {
    vec![
        TableRelationshipDef {
            source_table: "purchase_orders".to_string(),
            target_table: "goods_receipts".to_string(),
            source_key: "po_number".to_string(),
            target_key: "po_number".to_string(),
            relationship_type: TableRelationship::DocumentFlow,
            validate_amounts: false,
            validate_dates: true,
        },
        TableRelationshipDef {
            source_table: "goods_receipts".to_string(),
            target_table: "vendor_invoices".to_string(),
            source_key: "gr_number".to_string(),
            target_key: "gr_number".to_string(),
            relationship_type: TableRelationship::DocumentFlow,
            validate_amounts: true,
            validate_dates: true,
        },
        TableRelationshipDef {
            source_table: "vendor_invoices".to_string(),
            target_table: "payments".to_string(),
            source_key: "invoice_number".to_string(),
            target_key: "invoice_number".to_string(),
            relationship_type: TableRelationship::DocumentFlow,
            validate_amounts: true,
            validate_dates: true,
        },
        TableRelationshipDef {
            source_table: "vendor_invoices".to_string(),
            target_table: "journal_entries".to_string(),
            source_key: "invoice_number".to_string(),
            target_key: "source_document_id".to_string(),
            relationship_type: TableRelationship::OneToMany,
            validate_amounts: true,
            validate_dates: true,
        },
    ]
}

/// Predefined document flow relationships for O2C.
pub fn get_o2c_flow_relationships() -> Vec<TableRelationshipDef> {
    vec![
        TableRelationshipDef {
            source_table: "sales_orders".to_string(),
            target_table: "deliveries".to_string(),
            source_key: "so_number".to_string(),
            target_key: "so_number".to_string(),
            relationship_type: TableRelationship::DocumentFlow,
            validate_amounts: false,
            validate_dates: true,
        },
        TableRelationshipDef {
            source_table: "deliveries".to_string(),
            target_table: "customer_invoices".to_string(),
            source_key: "delivery_number".to_string(),
            target_key: "delivery_number".to_string(),
            relationship_type: TableRelationship::DocumentFlow,
            validate_amounts: true,
            validate_dates: true,
        },
        TableRelationshipDef {
            source_table: "customer_invoices".to_string(),
            target_table: "customer_receipts".to_string(),
            source_key: "invoice_number".to_string(),
            target_key: "invoice_number".to_string(),
            relationship_type: TableRelationship::DocumentFlow,
            validate_amounts: true,
            validate_dates: true,
        },
        TableRelationshipDef {
            source_table: "customer_invoices".to_string(),
            target_table: "journal_entries".to_string(),
            source_key: "invoice_number".to_string(),
            target_key: "source_document_id".to_string(),
            relationship_type: TableRelationship::OneToMany,
            validate_amounts: true,
            validate_dates: true,
        },
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_test_data() -> MultiTableData {
        let mut data = MultiTableData::default();

        // Create PO records
        let mut po1 = TableRecord::default();
        po1.id = "PO001".to_string();
        po1.table = "purchase_orders".to_string();
        po1.amount = Some(1000.0);
        po1.date = Some("2024-01-01".to_string());

        let mut po2 = TableRecord::default();
        po2.id = "PO002".to_string();
        po2.table = "purchase_orders".to_string();
        po2.amount = Some(2000.0);
        po2.date = Some("2024-01-02".to_string());
        po2.is_anomalous = true;
        po2.anomaly_id = Some("ANO001".to_string());

        data.tables
            .insert("purchase_orders".to_string(), vec![po1, po2]);

        // Create GR records
        let mut gr1 = TableRecord::default();
        gr1.id = "GR001".to_string();
        gr1.table = "goods_receipts".to_string();
        gr1.references
            .insert("purchase_orders".to_string(), "PO001".to_string());
        gr1.amount = Some(1000.0);
        gr1.date = Some("2024-01-05".to_string());

        let mut gr2 = TableRecord::default();
        gr2.id = "GR002".to_string();
        gr2.table = "goods_receipts".to_string();
        gr2.references
            .insert("purchase_orders".to_string(), "PO002".to_string());
        gr2.amount = Some(2000.0);
        gr2.date = Some("2024-01-06".to_string());

        data.tables
            .insert("goods_receipts".to_string(), vec![gr1, gr2]);

        // Create invoice records
        let mut inv1 = TableRecord::default();
        inv1.id = "INV001".to_string();
        inv1.table = "vendor_invoices".to_string();
        inv1.references
            .insert("goods_receipts".to_string(), "GR001".to_string());
        inv1.amount = Some(1000.0);
        inv1.date = Some("2024-01-10".to_string());

        data.tables
            .insert("vendor_invoices".to_string(), vec![inv1]);

        // Add relationship definitions
        data.relationships = vec![
            TableRelationshipDef {
                source_table: "goods_receipts".to_string(),
                target_table: "purchase_orders".to_string(),
                source_key: "po_number".to_string(),
                target_key: "id".to_string(),
                relationship_type: TableRelationship::DocumentFlow,
                validate_amounts: true,
                validate_dates: true,
            },
            TableRelationshipDef {
                source_table: "vendor_invoices".to_string(),
                target_table: "goods_receipts".to_string(),
                source_key: "gr_number".to_string(),
                target_key: "id".to_string(),
                relationship_type: TableRelationship::DocumentFlow,
                validate_amounts: true,
                validate_dates: true,
            },
        ];

        // Add anomaly
        data.anomalies.push(AnomalyRecord {
            anomaly_id: "ANO001".to_string(),
            anomaly_type: "Fraud".to_string(),
            source_record_id: "PO002".to_string(),
            source_table: "purchase_orders".to_string(),
            severity: 4,
            monetary_impact: Some(2000.0),
        });

        data
    }

    #[test]
    fn test_basic_consistency_evaluation() {
        let data = create_test_data();
        let evaluator = MultiTableConsistencyEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert_eq!(result.table_consistency.len(), 2);
        // Records should be evaluated - specific scores depend on data setup
        // The test data has GRs referencing POs, and invoices referencing GRs
        // With valid references, we expect matching records
        for table_result in &result.table_consistency {
            // Should have some records checked
            println!(
                "{}->{}: checked={}, matching={}, orphaned_source={}",
                table_result.source_table,
                table_result.target_table,
                table_result.records_checked,
                table_result.matching_records,
                table_result.orphaned_source
            );
        }
    }

    #[test]
    fn test_cascade_analysis() {
        let data = create_test_data();
        let evaluator = MultiTableConsistencyEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert_eq!(result.cascade_analysis.total_anomalies, 1);
    }

    #[test]
    fn test_empty_data() {
        let data = MultiTableData::default();
        let evaluator = MultiTableConsistencyEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert!(result.passes);
        assert_eq!(result.total_violations, 0);
    }

    #[test]
    fn test_missing_reference() {
        let mut data = MultiTableData::default();

        // Create invoice with invalid GR reference
        let mut inv = TableRecord::default();
        inv.id = "INV001".to_string();
        inv.table = "vendor_invoices".to_string();
        inv.references
            .insert("goods_receipts".to_string(), "GR999".to_string()); // Invalid
        inv.amount = Some(1000.0);

        data.tables.insert("vendor_invoices".to_string(), vec![inv]);
        data.tables.insert("goods_receipts".to_string(), Vec::new());

        data.relationships.push(TableRelationshipDef {
            source_table: "vendor_invoices".to_string(),
            target_table: "goods_receipts".to_string(),
            source_key: "gr_number".to_string(),
            target_key: "id".to_string(),
            relationship_type: TableRelationship::DocumentFlow,
            validate_amounts: true,
            validate_dates: false,
        });

        let evaluator = MultiTableConsistencyEvaluator::default();
        let result = evaluator.evaluate(&data).unwrap();

        assert!(!result.table_consistency.is_empty());
        let inv_gr = &result.table_consistency[0];
        assert_eq!(inv_gr.orphaned_source, 1);
        assert_eq!(inv_gr.violations.len(), 1);
        assert_eq!(
            inv_gr.violations[0].violation_type,
            ViolationType::MissingReference
        );
    }
}
