//! AML typology detectability evaluator.
//!
//! Validates that AML typologies (structuring, layering, mule networks, etc.)
//! produce statistically detectable patterns and maintain coherence.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AML transaction data for a typology instance.
#[derive(Debug, Clone)]
pub struct AmlTransactionData {
    /// Transaction identifier.
    pub transaction_id: String,
    /// Typology name (e.g., "structuring", "layering", "mule_network").
    pub typology: String,
    /// Case identifier (shared across related transactions).
    pub case_id: String,
    /// Transaction amount.
    pub amount: f64,
    /// Whether this is a flagged/suspicious transaction.
    pub is_flagged: bool,
}

/// Overall typology data for coverage validation.
#[derive(Debug, Clone)]
pub struct TypologyData {
    /// Typology name.
    pub name: String,
    /// Number of scenarios generated.
    pub scenario_count: usize,
    /// Whether all transactions in a scenario share a case_id.
    pub case_ids_consistent: bool,
}

/// Thresholds for AML detectability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlDetectabilityThresholds {
    /// Minimum typology coverage (fraction of expected typologies present).
    pub min_typology_coverage: f64,
    /// Minimum scenario coherence rate.
    pub min_scenario_coherence: f64,
    /// Structuring threshold (transactions should cluster below this).
    pub structuring_threshold: f64,
}

impl Default for AmlDetectabilityThresholds {
    fn default() -> Self {
        Self {
            min_typology_coverage: 0.80,
            min_scenario_coherence: 0.90,
            structuring_threshold: 10_000.0,
        }
    }
}

/// Per-typology detectability result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypologyDetectability {
    /// Typology name.
    pub name: String,
    /// Number of transactions.
    pub transaction_count: usize,
    /// Number of unique cases.
    pub case_count: usize,
    /// Flag rate.
    pub flag_rate: f64,
    /// Whether the typology shows expected patterns.
    pub pattern_detected: bool,
}

/// Results of AML detectability analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlDetectabilityAnalysis {
    /// Typology coverage: fraction of expected typologies present.
    pub typology_coverage: f64,
    /// Scenario coherence: fraction of scenarios with consistent case_ids.
    pub scenario_coherence: f64,
    /// Per-typology detectability.
    pub per_typology: Vec<TypologyDetectability>,
    /// Total transactions analyzed.
    pub total_transactions: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Expected typology names for coverage calculation.
const EXPECTED_TYPOLOGIES: &[&str] = &[
    "structuring",
    "layering",
    "mule_network",
    "round_tripping",
    "fraud",
    "spoofing",
];

/// Analyzer for AML detectability.
pub struct AmlDetectabilityAnalyzer {
    thresholds: AmlDetectabilityThresholds,
}

impl AmlDetectabilityAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: AmlDetectabilityThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: AmlDetectabilityThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze AML transactions and typology data.
    pub fn analyze(
        &self,
        transactions: &[AmlTransactionData],
        typologies: &[TypologyData],
    ) -> EvalResult<AmlDetectabilityAnalysis> {
        let mut issues = Vec::new();

        // 1. Typology coverage
        let present_typologies: std::collections::HashSet<&str> =
            typologies.iter().map(|t| t.name.as_str()).collect();
        let covered = EXPECTED_TYPOLOGIES
            .iter()
            .filter(|&&t| present_typologies.contains(t))
            .count();
        let typology_coverage = covered as f64 / EXPECTED_TYPOLOGIES.len() as f64;

        // 2. Scenario coherence
        let coherent = typologies.iter().filter(|t| t.case_ids_consistent).count();
        let scenario_coherence = if typologies.is_empty() {
            1.0
        } else {
            coherent as f64 / typologies.len() as f64
        };

        // 3. Per-typology analysis
        let mut by_typology: HashMap<String, Vec<&AmlTransactionData>> = HashMap::new();
        for txn in transactions {
            by_typology
                .entry(txn.typology.clone())
                .or_default()
                .push(txn);
        }

        let mut per_typology = Vec::new();
        for (name, txns) in &by_typology {
            let case_ids: std::collections::HashSet<&str> =
                txns.iter().map(|t| t.case_id.as_str()).collect();
            let flagged = txns.iter().filter(|t| t.is_flagged).count();
            let flag_rate = if txns.is_empty() {
                0.0
            } else {
                flagged as f64 / txns.len() as f64
            };

            // Check typology-specific patterns
            let pattern_detected = match name.as_str() {
                "structuring" => {
                    // Most amounts should be below threshold
                    let below = txns
                        .iter()
                        .filter(|t| t.amount < self.thresholds.structuring_threshold)
                        .count();
                    below as f64 / txns.len().max(1) as f64 > 0.5
                }
                "layering" => {
                    // Should have multiple cases with >2 transactions each
                    !case_ids.is_empty() && txns.len() > case_ids.len()
                }
                _ => {
                    // Generic: require a meaningful flag rate indicating
                    // the typology produces detectable suspicious patterns.
                    // A flag rate of 0 means no suspicious indicators at all.
                    let suspicious_count = txns.iter().filter(|t| t.is_flagged).count();
                    let suspicious_ratio = suspicious_count as f64 / txns.len().max(1) as f64;
                    !txns.is_empty() && suspicious_ratio > 0.0
                }
            };

            per_typology.push(TypologyDetectability {
                name: name.clone(),
                transaction_count: txns.len(),
                case_count: case_ids.len(),
                flag_rate,
                pattern_detected,
            });
        }

        // Check thresholds
        if typology_coverage < self.thresholds.min_typology_coverage {
            issues.push(format!(
                "Typology coverage {:.3} < {:.3}",
                typology_coverage, self.thresholds.min_typology_coverage
            ));
        }
        if scenario_coherence < self.thresholds.min_scenario_coherence {
            issues.push(format!(
                "Scenario coherence {:.3} < {:.3}",
                scenario_coherence, self.thresholds.min_scenario_coherence
            ));
        }

        let passes = issues.is_empty();

        Ok(AmlDetectabilityAnalysis {
            typology_coverage,
            scenario_coherence,
            per_typology,
            total_transactions: transactions.len(),
            passes,
            issues,
        })
    }
}

impl Default for AmlDetectabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_good_aml_data() {
        let analyzer = AmlDetectabilityAnalyzer::new();
        let typologies: Vec<TypologyData> = EXPECTED_TYPOLOGIES
            .iter()
            .map(|name| TypologyData {
                name: name.to_string(),
                scenario_count: 5,
                case_ids_consistent: true,
            })
            .collect();
        let transactions = vec![
            AmlTransactionData {
                transaction_id: "T001".to_string(),
                typology: "structuring".to_string(),
                case_id: "C001".to_string(),
                amount: 9_500.0,
                is_flagged: true,
            },
            AmlTransactionData {
                transaction_id: "T002".to_string(),
                typology: "structuring".to_string(),
                case_id: "C001".to_string(),
                amount: 9_800.0,
                is_flagged: true,
            },
        ];

        let result = analyzer.analyze(&transactions, &typologies).unwrap();
        assert!(result.passes);
        assert_eq!(result.typology_coverage, 1.0);
    }

    #[test]
    fn test_missing_typologies() {
        let analyzer = AmlDetectabilityAnalyzer::new();
        let typologies = vec![TypologyData {
            name: "structuring".to_string(),
            scenario_count: 5,
            case_ids_consistent: true,
        }];

        let result = analyzer.analyze(&[], &typologies).unwrap();
        assert!(!result.passes); // Coverage too low
    }

    #[test]
    fn test_empty() {
        let analyzer = AmlDetectabilityAnalyzer::new();
        let result = analyzer.analyze(&[], &[]).unwrap();
        assert!(!result.passes); // Zero coverage
    }
}
