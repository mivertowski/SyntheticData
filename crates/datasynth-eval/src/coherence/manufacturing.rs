//! Manufacturing evaluator.
//!
//! Validates manufacturing data coherence including yield rate consistency,
//! cost variance, operation sequencing, quality inspection accuracy,
//! and cycle count variance calculations.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for manufacturing evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingThresholds {
    /// Minimum yield rate consistency (fraction of orders with valid yield).
    pub min_yield_consistency: f64,
    /// Minimum fraction of orders with valid operation sequences.
    pub min_sequence_valid: f64,
    /// Minimum defect rate calculation accuracy.
    pub min_defect_rate_accuracy: f64,
    /// Minimum variance calculation accuracy for cycle counts.
    pub min_variance_accuracy: f64,
}

impl Default for ManufacturingThresholds {
    fn default() -> Self {
        Self {
            min_yield_consistency: 0.95,
            min_sequence_valid: 0.99,
            min_defect_rate_accuracy: 0.99,
            min_variance_accuracy: 0.99,
        }
    }
}

/// Production order data for validation.
#[derive(Debug, Clone)]
pub struct ProductionOrderData {
    /// Order identifier.
    pub order_id: String,
    /// Actual output quantity.
    pub actual_quantity: f64,
    /// Scrap quantity.
    pub scrap_quantity: f64,
    /// Reported yield rate.
    pub reported_yield: f64,
    /// Planned cost.
    pub planned_cost: f64,
    /// Actual cost.
    pub actual_cost: f64,
}

/// Routing operation data for sequence validation.
#[derive(Debug, Clone)]
pub struct RoutingOperationData {
    /// Parent order identifier.
    pub order_id: String,
    /// Operation sequence number.
    pub sequence_number: u32,
    /// Operation start timestamp (epoch seconds).
    pub start_timestamp: i64,
}

/// Quality inspection data.
#[derive(Debug, Clone)]
pub struct QualityInspectionData {
    /// Inspection lot identifier.
    pub lot_id: String,
    /// Sample size.
    pub sample_size: u32,
    /// Defect count.
    pub defect_count: u32,
    /// Reported defect rate.
    pub reported_defect_rate: f64,
    /// Characteristics within specification limits.
    pub characteristics_within_limits: u32,
    /// Total characteristics inspected.
    pub total_characteristics: u32,
}

/// Cycle count data.
#[derive(Debug, Clone)]
pub struct CycleCountData {
    /// Count record identifier.
    pub record_id: String,
    /// Book quantity.
    pub book_quantity: f64,
    /// Counted quantity.
    pub counted_quantity: f64,
    /// Reported variance.
    pub reported_variance: f64,
}

/// Results of manufacturing evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingEvaluation {
    /// Yield rate consistency: fraction of orders with correct yield calculation.
    pub yield_rate_consistency: f64,
    /// Average cost variance ratio |actual - planned| / planned.
    pub avg_cost_variance_ratio: f64,
    /// Operation sequence validity: fraction with ascending sequence numbers and dates.
    pub operation_sequence_valid: f64,
    /// Defect rate accuracy: fraction of inspections with correct defect rate.
    pub defect_rate_accuracy: f64,
    /// Characteristics compliance: fraction of characteristics within limits.
    pub characteristics_compliance: f64,
    /// Variance calculation accuracy: fraction of cycle counts with correct variance.
    pub variance_calculation_accuracy: f64,
    /// Total production orders evaluated.
    pub total_orders: usize,
    /// Total inspections evaluated.
    pub total_inspections: usize,
    /// Total cycle counts evaluated.
    pub total_cycle_counts: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for manufacturing coherence.
pub struct ManufacturingEvaluator {
    thresholds: ManufacturingThresholds,
}

impl ManufacturingEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: ManufacturingThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: ManufacturingThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate manufacturing data.
    pub fn evaluate(
        &self,
        orders: &[ProductionOrderData],
        operations: &[RoutingOperationData],
        inspections: &[QualityInspectionData],
        cycle_counts: &[CycleCountData],
    ) -> EvalResult<ManufacturingEvaluation> {
        let mut issues = Vec::new();

        // 1. Yield rate consistency: yield = actual / (actual + scrap)
        let yield_ok = orders
            .iter()
            .filter(|o| {
                let total = o.actual_quantity + o.scrap_quantity;
                if total <= 0.0 {
                    return true; // Skip zero-output orders
                }
                let expected_yield = o.actual_quantity / total;
                (o.reported_yield - expected_yield).abs() <= 0.001
            })
            .count();
        let yield_rate_consistency = if orders.is_empty() {
            1.0
        } else {
            yield_ok as f64 / orders.len() as f64
        };

        // 2. Cost variance
        let cost_variances: Vec<f64> = orders
            .iter()
            .filter(|o| o.planned_cost > 0.0)
            .map(|o| (o.actual_cost - o.planned_cost).abs() / o.planned_cost)
            .collect();
        let avg_cost_variance_ratio = if cost_variances.is_empty() {
            0.0
        } else {
            cost_variances.iter().sum::<f64>() / cost_variances.len() as f64
        };

        // 3. Operation sequencing: group by order, check ascending
        let mut order_ops: std::collections::HashMap<&str, Vec<&RoutingOperationData>> =
            std::collections::HashMap::new();
        for op in operations {
            order_ops.entry(op.order_id.as_str()).or_default().push(op);
        }
        let total_order_groups = order_ops.len();
        let seq_valid = order_ops
            .values()
            .filter(|ops| {
                let mut sorted = ops.to_vec();
                sorted.sort_by_key(|o| o.sequence_number);
                // Check sequence numbers are ascending and timestamps non-decreasing
                sorted.windows(2).all(|w| {
                    w[0].sequence_number < w[1].sequence_number
                        && w[0].start_timestamp <= w[1].start_timestamp
                })
            })
            .count();
        let operation_sequence_valid = if total_order_groups == 0 {
            1.0
        } else {
            seq_valid as f64 / total_order_groups as f64
        };

        // 4. Quality inspection: defect_rate = defect_count / sample_size
        let defect_ok = inspections
            .iter()
            .filter(|insp| {
                if insp.sample_size == 0 {
                    return true;
                }
                let expected_rate = insp.defect_count as f64 / insp.sample_size as f64;
                (insp.reported_defect_rate - expected_rate).abs() <= 0.001
            })
            .count();
        let defect_rate_accuracy = if inspections.is_empty() {
            1.0
        } else {
            defect_ok as f64 / inspections.len() as f64
        };

        // Characteristics compliance
        let total_chars: u32 = inspections.iter().map(|i| i.total_characteristics).sum();
        let within_chars: u32 = inspections
            .iter()
            .map(|i| i.characteristics_within_limits)
            .sum();
        let characteristics_compliance = if total_chars == 0 {
            1.0
        } else {
            within_chars as f64 / total_chars as f64
        };

        // 5. Cycle count variance: variance = counted - book
        let variance_ok = cycle_counts
            .iter()
            .filter(|cc| {
                let expected_variance = cc.counted_quantity - cc.book_quantity;
                (cc.reported_variance - expected_variance).abs() <= 0.01
            })
            .count();
        let variance_calculation_accuracy = if cycle_counts.is_empty() {
            1.0
        } else {
            variance_ok as f64 / cycle_counts.len() as f64
        };

        // Check thresholds
        if yield_rate_consistency < self.thresholds.min_yield_consistency {
            issues.push(format!(
                "Yield consistency {:.3} < {:.3}",
                yield_rate_consistency, self.thresholds.min_yield_consistency
            ));
        }
        if operation_sequence_valid < self.thresholds.min_sequence_valid {
            issues.push(format!(
                "Operation sequence validity {:.3} < {:.3}",
                operation_sequence_valid, self.thresholds.min_sequence_valid
            ));
        }
        if defect_rate_accuracy < self.thresholds.min_defect_rate_accuracy {
            issues.push(format!(
                "Defect rate accuracy {:.3} < {:.3}",
                defect_rate_accuracy, self.thresholds.min_defect_rate_accuracy
            ));
        }
        if variance_calculation_accuracy < self.thresholds.min_variance_accuracy {
            issues.push(format!(
                "Variance calculation accuracy {:.3} < {:.3}",
                variance_calculation_accuracy, self.thresholds.min_variance_accuracy
            ));
        }

        let passes = issues.is_empty();

        Ok(ManufacturingEvaluation {
            yield_rate_consistency,
            avg_cost_variance_ratio,
            operation_sequence_valid,
            defect_rate_accuracy,
            characteristics_compliance,
            variance_calculation_accuracy,
            total_orders: orders.len(),
            total_inspections: inspections.len(),
            total_cycle_counts: cycle_counts.len(),
            passes,
            issues,
        })
    }
}

impl Default for ManufacturingEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_manufacturing_data() {
        let evaluator = ManufacturingEvaluator::new();
        let orders = vec![ProductionOrderData {
            order_id: "PO001".to_string(),
            actual_quantity: 90.0,
            scrap_quantity: 10.0,
            reported_yield: 0.9, // 90 / (90+10) = 0.9
            planned_cost: 10_000.0,
            actual_cost: 10_500.0,
        }];

        let operations = vec![
            RoutingOperationData {
                order_id: "PO001".to_string(),
                sequence_number: 10,
                start_timestamp: 1000,
            },
            RoutingOperationData {
                order_id: "PO001".to_string(),
                sequence_number: 20,
                start_timestamp: 2000,
            },
        ];

        let inspections = vec![QualityInspectionData {
            lot_id: "LOT001".to_string(),
            sample_size: 100,
            defect_count: 5,
            reported_defect_rate: 0.05,
            characteristics_within_limits: 95,
            total_characteristics: 100,
        }];

        let cycle_counts = vec![CycleCountData {
            record_id: "CC001".to_string(),
            book_quantity: 100.0,
            counted_quantity: 98.0,
            reported_variance: -2.0,
        }];

        let result = evaluator
            .evaluate(&orders, &operations, &inspections, &cycle_counts)
            .unwrap();
        assert!(result.passes);
        assert_eq!(result.yield_rate_consistency, 1.0);
        assert_eq!(result.defect_rate_accuracy, 1.0);
    }

    #[test]
    fn test_wrong_yield() {
        let evaluator = ManufacturingEvaluator::new();
        let orders = vec![ProductionOrderData {
            order_id: "PO001".to_string(),
            actual_quantity: 90.0,
            scrap_quantity: 10.0,
            reported_yield: 0.5, // Wrong, should be 0.9
            planned_cost: 10_000.0,
            actual_cost: 10_000.0,
        }];

        let result = evaluator.evaluate(&orders, &[], &[], &[]).unwrap();
        assert!(!result.passes);
    }

    #[test]
    fn test_out_of_order_operations() {
        let evaluator = ManufacturingEvaluator::new();
        let operations = vec![
            RoutingOperationData {
                order_id: "PO001".to_string(),
                sequence_number: 10,
                start_timestamp: 2000, // Later timestamp but earlier sequence
            },
            RoutingOperationData {
                order_id: "PO001".to_string(),
                sequence_number: 20,
                start_timestamp: 1000, // Earlier timestamp but later sequence
            },
        ];

        let result = evaluator.evaluate(&[], &operations, &[], &[]).unwrap();
        assert_eq!(result.operation_sequence_valid, 0.0);
    }

    #[test]
    fn test_empty_data() {
        let evaluator = ManufacturingEvaluator::new();
        let result = evaluator.evaluate(&[], &[], &[], &[]).unwrap();
        assert!(result.passes);
    }
}
