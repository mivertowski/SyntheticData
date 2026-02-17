//! ESG (Environmental, Social, Governance) coherence evaluator.
//!
//! Validates water consumption formulas, safety incident rates (TRIR/LTIR),
//! board governance ratios, and supplier ESG scoring consistency.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for ESG evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgThresholds {
    /// Minimum accuracy for metric calculations.
    pub min_metric_accuracy: f64,
    /// Minimum accuracy for safety rate calculations.
    pub min_safety_rate_accuracy: f64,
    /// Tolerance for metric comparisons.
    pub metric_tolerance: f64,
}

impl Default for EsgThresholds {
    fn default() -> Self {
        Self {
            min_metric_accuracy: 0.99,
            min_safety_rate_accuracy: 0.999,
            metric_tolerance: 0.01,
        }
    }
}

/// Water usage data for consumption validation.
#[derive(Debug, Clone)]
pub struct WaterUsageData {
    /// Record identifier.
    pub record_id: String,
    /// Water withdrawal in cubic meters.
    pub withdrawal_m3: f64,
    /// Water discharge in cubic meters.
    pub discharge_m3: f64,
    /// Water consumption (withdrawal - discharge).
    pub consumption_m3: f64,
}

/// Safety metric data for incident rate validation.
#[derive(Debug, Clone)]
pub struct SafetyMetricData {
    /// Metric identifier.
    pub metric_id: String,
    /// Total hours worked.
    pub total_hours_worked: f64,
    /// Number of recordable incidents.
    pub recordable_incidents: u32,
    /// Total Recordable Incident Rate.
    pub trir: f64,
    /// Number of lost time incidents.
    pub lost_time_incidents: u32,
    /// Lost Time Incident Rate.
    pub ltir: f64,
}

/// Governance data for board ratio validation.
#[derive(Debug, Clone)]
pub struct GovernanceData {
    /// Metric identifier.
    pub metric_id: String,
    /// Total board size.
    pub board_size: u32,
    /// Number of independent directors.
    pub independent_directors: u32,
    /// Reported independence ratio.
    pub independence_ratio: f64,
}

/// Supplier ESG scoring data.
#[derive(Debug, Clone)]
pub struct SupplierEsgData {
    /// Assessment identifier.
    pub assessment_id: String,
    /// Environmental score (0-100).
    pub environmental_score: f64,
    /// Social score (0-100).
    pub social_score: f64,
    /// Governance score (0-100).
    pub governance_score: f64,
    /// Overall score (should be average of E, S, G).
    pub overall_score: f64,
}

/// Results of ESG coherence evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgEvaluation {
    /// Fraction of water records with correct consumption.
    pub water_accuracy: f64,
    /// Fraction of safety records with correct TRIR.
    pub trir_accuracy: f64,
    /// Fraction of safety records with correct LTIR.
    pub ltir_accuracy: f64,
    /// Fraction of governance records with correct independence ratio.
    pub governance_accuracy: f64,
    /// Fraction of supplier assessments with correct overall score.
    pub supplier_scoring_accuracy: f64,
    /// Total water records evaluated.
    pub total_water_records: usize,
    /// Total safety records evaluated.
    pub total_safety_records: usize,
    /// Total governance records evaluated.
    pub total_governance_records: usize,
    /// Total supplier assessments evaluated.
    pub total_supplier_assessments: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for ESG coherence.
pub struct EsgEvaluator {
    thresholds: EsgThresholds,
}

impl EsgEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: EsgThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: EsgThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate ESG data coherence.
    pub fn evaluate(
        &self,
        water: &[WaterUsageData],
        safety: &[SafetyMetricData],
        governance: &[GovernanceData],
        suppliers: &[SupplierEsgData],
    ) -> EvalResult<EsgEvaluation> {
        let mut issues = Vec::new();
        let tolerance = self.thresholds.metric_tolerance;

        // 1. Water: consumption ≈ withdrawal - discharge
        let water_ok = water
            .iter()
            .filter(|w| {
                let expected = w.withdrawal_m3 - w.discharge_m3;
                (w.consumption_m3 - expected).abs() <= tolerance * w.withdrawal_m3.abs().max(1.0)
            })
            .count();
        let water_accuracy = if water.is_empty() {
            1.0
        } else {
            water_ok as f64 / water.len() as f64
        };

        // 2. TRIR: recordable_incidents * 200_000 / total_hours_worked
        let trir_ok = safety
            .iter()
            .filter(|s| {
                if s.total_hours_worked <= 0.0 {
                    return true;
                }
                let expected = s.recordable_incidents as f64 * 200_000.0 / s.total_hours_worked;
                (s.trir - expected).abs() <= tolerance * expected.abs().max(0.001)
            })
            .count();
        let trir_accuracy = if safety.is_empty() {
            1.0
        } else {
            trir_ok as f64 / safety.len() as f64
        };

        // 3. LTIR: lost_time_incidents * 200_000 / total_hours_worked
        let ltir_ok = safety
            .iter()
            .filter(|s| {
                if s.total_hours_worked <= 0.0 {
                    return true;
                }
                let expected = s.lost_time_incidents as f64 * 200_000.0 / s.total_hours_worked;
                (s.ltir - expected).abs() <= tolerance * expected.abs().max(0.001)
            })
            .count();
        let ltir_accuracy = if safety.is_empty() {
            1.0
        } else {
            ltir_ok as f64 / safety.len() as f64
        };

        // 4. Governance: independence_ratio ≈ independent_directors / board_size
        let gov_ok = governance
            .iter()
            .filter(|g| {
                if g.board_size == 0 {
                    return true;
                }
                let expected = g.independent_directors as f64 / g.board_size as f64;
                (g.independence_ratio - expected).abs() <= tolerance
            })
            .count();
        let governance_accuracy = if governance.is_empty() {
            1.0
        } else {
            gov_ok as f64 / governance.len() as f64
        };

        // 5. Supplier scoring: overall ≈ (E + S + G) / 3
        let supplier_ok = suppliers
            .iter()
            .filter(|s| {
                let expected = (s.environmental_score + s.social_score + s.governance_score) / 3.0;
                (s.overall_score - expected).abs() <= tolerance * expected.abs().max(1.0)
            })
            .count();
        let supplier_scoring_accuracy = if suppliers.is_empty() {
            1.0
        } else {
            supplier_ok as f64 / suppliers.len() as f64
        };

        // Check thresholds
        if water_accuracy < self.thresholds.min_metric_accuracy {
            issues.push(format!(
                "Water consumption accuracy {:.4} < {:.4}",
                water_accuracy, self.thresholds.min_metric_accuracy
            ));
        }
        if trir_accuracy < self.thresholds.min_safety_rate_accuracy {
            issues.push(format!(
                "TRIR accuracy {:.4} < {:.4}",
                trir_accuracy, self.thresholds.min_safety_rate_accuracy
            ));
        }
        if ltir_accuracy < self.thresholds.min_safety_rate_accuracy {
            issues.push(format!(
                "LTIR accuracy {:.4} < {:.4}",
                ltir_accuracy, self.thresholds.min_safety_rate_accuracy
            ));
        }
        if governance_accuracy < self.thresholds.min_metric_accuracy {
            issues.push(format!(
                "Governance ratio accuracy {:.4} < {:.4}",
                governance_accuracy, self.thresholds.min_metric_accuracy
            ));
        }
        if supplier_scoring_accuracy < self.thresholds.min_metric_accuracy {
            issues.push(format!(
                "Supplier ESG scoring accuracy {:.4} < {:.4}",
                supplier_scoring_accuracy, self.thresholds.min_metric_accuracy
            ));
        }

        let passes = issues.is_empty();

        Ok(EsgEvaluation {
            water_accuracy,
            trir_accuracy,
            ltir_accuracy,
            governance_accuracy,
            supplier_scoring_accuracy,
            total_water_records: water.len(),
            total_safety_records: safety.len(),
            total_governance_records: governance.len(),
            total_supplier_assessments: suppliers.len(),
            passes,
            issues,
        })
    }
}

impl Default for EsgEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_esg_data() {
        let evaluator = EsgEvaluator::new();
        let water = vec![WaterUsageData {
            record_id: "W001".to_string(),
            withdrawal_m3: 1000.0,
            discharge_m3: 700.0,
            consumption_m3: 300.0,
        }];
        let safety = vec![SafetyMetricData {
            metric_id: "S001".to_string(),
            total_hours_worked: 1_000_000.0,
            recordable_incidents: 5,
            trir: 1.0, // 5 * 200_000 / 1_000_000
            lost_time_incidents: 2,
            ltir: 0.4, // 2 * 200_000 / 1_000_000
        }];
        let governance = vec![GovernanceData {
            metric_id: "G001".to_string(),
            board_size: 10,
            independent_directors: 7,
            independence_ratio: 0.7,
        }];
        let suppliers = vec![SupplierEsgData {
            assessment_id: "ESG001".to_string(),
            environmental_score: 80.0,
            social_score: 70.0,
            governance_score: 90.0,
            overall_score: 80.0,
        }];

        let result = evaluator
            .evaluate(&water, &safety, &governance, &suppliers)
            .unwrap();
        assert!(result.passes);
        assert_eq!(result.total_water_records, 1);
        assert_eq!(result.total_safety_records, 1);
    }

    #[test]
    fn test_wrong_water_consumption() {
        let evaluator = EsgEvaluator::new();
        let water = vec![WaterUsageData {
            record_id: "W001".to_string(),
            withdrawal_m3: 1000.0,
            discharge_m3: 700.0,
            consumption_m3: 500.0, // Wrong: should be 300
        }];

        let result = evaluator.evaluate(&water, &[], &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues[0].contains("Water consumption"));
    }

    #[test]
    fn test_wrong_trir() {
        let evaluator = EsgEvaluator::new();
        let safety = vec![SafetyMetricData {
            metric_id: "S001".to_string(),
            total_hours_worked: 1_000_000.0,
            recordable_incidents: 5,
            trir: 5.0, // Wrong: should be 1.0
            lost_time_incidents: 2,
            ltir: 0.4,
        }];

        let result = evaluator.evaluate(&[], &safety, &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("TRIR")));
    }

    #[test]
    fn test_wrong_supplier_scoring() {
        let evaluator = EsgEvaluator::new();
        let suppliers = vec![SupplierEsgData {
            assessment_id: "ESG001".to_string(),
            environmental_score: 80.0,
            social_score: 70.0,
            governance_score: 90.0,
            overall_score: 90.0, // Wrong: should be 80.0
        }];

        let result = evaluator.evaluate(&[], &[], &[], &suppliers).unwrap();
        assert!(!result.passes);
        assert!(result.issues[0].contains("Supplier ESG"));
    }

    #[test]
    fn test_wrong_ltir() {
        let evaluator = EsgEvaluator::new();
        let safety = vec![SafetyMetricData {
            metric_id: "S001".to_string(),
            total_hours_worked: 1_000_000.0,
            recordable_incidents: 5,
            trir: 1.0, // Correct
            lost_time_incidents: 2,
            ltir: 2.0, // Wrong: should be 0.4
        }];

        let result = evaluator.evaluate(&[], &safety, &[], &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("LTIR")));
    }

    #[test]
    fn test_wrong_governance_ratio() {
        let evaluator = EsgEvaluator::new();
        let governance = vec![GovernanceData {
            metric_id: "G001".to_string(),
            board_size: 10,
            independent_directors: 7,
            independence_ratio: 0.5, // Wrong: should be 0.7
        }];

        let result = evaluator.evaluate(&[], &[], &governance, &[]).unwrap();
        assert!(!result.passes);
        assert!(result.issues.iter().any(|i| i.contains("Governance ratio")));
    }

    #[test]
    fn test_empty_data() {
        let evaluator = EsgEvaluator::new();
        let result = evaluator.evaluate(&[], &[], &[], &[]).unwrap();
        assert!(result.passes);
    }
}
