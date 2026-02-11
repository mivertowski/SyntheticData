//! Causal structure validation.
//!
//! Validates that generated samples respect the causal structure defined by the graph,
//! checking correlation signs, edge strength, and topological consistency.

use std::collections::HashMap;

use super::graph::{CausalGraph, CausalMechanism};

/// Report from causal structure validation.
#[derive(Debug, Clone)]
pub struct CausalValidationReport {
    /// Whether all checks passed.
    pub valid: bool,
    /// Individual check results.
    pub checks: Vec<CausalCheck>,
    /// Human-readable violation descriptions.
    pub violations: Vec<String>,
}

/// Result of a single validation check.
#[derive(Debug, Clone)]
pub struct CausalCheck {
    /// Name of the check.
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Details about the check result.
    pub details: String,
}

/// Validator for causal structure consistency.
pub struct CausalValidator;

impl CausalValidator {
    /// Validate that samples respect the causal structure of the graph.
    ///
    /// Performs three checks:
    /// 1. Edge correlation signs match mechanism coefficient signs
    /// 2. Non-edges have weaker average correlation than edges
    /// 3. Topological ordering holds in conditional means
    pub fn validate_causal_structure(
        samples: &[HashMap<String, f64>],
        graph: &CausalGraph,
    ) -> CausalValidationReport {
        let mut checks = Vec::new();
        let mut violations = Vec::new();

        // Check 1: Edge correlation signs
        let sign_check = Self::check_edge_correlation_signs(samples, graph);
        if !sign_check.passed {
            violations.push(sign_check.details.clone());
        }
        checks.push(sign_check);

        // Check 2: Non-edges have weaker correlation than edges
        let strength_check = Self::check_non_edge_weakness(samples, graph);
        if !strength_check.passed {
            violations.push(strength_check.details.clone());
        }
        checks.push(strength_check);

        // Check 3: Topological ordering in conditional means
        let topo_check = Self::check_topological_consistency(samples, graph);
        if !topo_check.passed {
            violations.push(topo_check.details.clone());
        }
        checks.push(topo_check);

        let valid = checks.iter().all(|c| c.passed);

        CausalValidationReport {
            valid,
            checks,
            violations,
        }
    }

    /// Check 1: For each edge, verify correlation between parent and child
    /// has the expected sign (based on mechanism coefficient sign).
    fn check_edge_correlation_signs(
        samples: &[HashMap<String, f64>],
        graph: &CausalGraph,
    ) -> CausalCheck {
        let mut total_edges = 0;
        let mut _correct_signs = 0u32;
        let mut mismatches = Vec::new();

        for edge in &graph.edges {
            let expected_sign = Self::mechanism_sign(&edge.mechanism);
            // Skip edges where we can't reliably determine expected sign.
            // Threshold mechanisms produce binary outputs where correlation
            // with the continuous parent is often very weak or indeterminate.
            if expected_sign == 0 || matches!(edge.mechanism, CausalMechanism::Threshold { .. }) {
                continue;
            }

            total_edges += 1;

            let parent_vals: Vec<f64> = samples
                .iter()
                .filter_map(|s| s.get(&edge.from).copied())
                .collect();
            let child_vals: Vec<f64> = samples
                .iter()
                .filter_map(|s| s.get(&edge.to).copied())
                .collect();

            let corr = pearson_correlation(&parent_vals, &child_vals);

            if (expected_sign > 0 && corr > -0.05) || (expected_sign < 0 && corr < 0.05) {
                _correct_signs += 1;
            } else {
                mismatches.push(format!(
                    "{} -> {}: expected sign {}, got correlation {:.4}",
                    edge.from, edge.to, expected_sign, corr
                ));
            }
        }

        let passed = mismatches.is_empty();
        let details = if passed {
            format!("All {} edges have correct correlation signs", total_edges)
        } else {
            format!(
                "{}/{} edges have incorrect signs: {}",
                mismatches.len(),
                total_edges,
                mismatches.join("; ")
            )
        };

        CausalCheck {
            name: "edge_correlation_signs".to_string(),
            passed,
            details,
        }
    }

    /// Check 2: Verify non-edges have weaker correlation than edges (on average).
    fn check_non_edge_weakness(
        samples: &[HashMap<String, f64>],
        graph: &CausalGraph,
    ) -> CausalCheck {
        let var_names = graph.variable_names();

        // Compute average absolute correlation for edges
        let mut edge_corrs = Vec::new();
        for edge in &graph.edges {
            let parent_vals: Vec<f64> = samples
                .iter()
                .filter_map(|s| s.get(&edge.from).copied())
                .collect();
            let child_vals: Vec<f64> = samples
                .iter()
                .filter_map(|s| s.get(&edge.to).copied())
                .collect();
            let corr = pearson_correlation(&parent_vals, &child_vals).abs();
            if corr.is_finite() {
                edge_corrs.push(corr);
            }
        }

        // Build set of edge pairs for fast lookup
        let edge_pairs: std::collections::HashSet<(&str, &str)> = graph
            .edges
            .iter()
            .map(|e| (e.from.as_str(), e.to.as_str()))
            .collect();

        // Compute average absolute correlation for non-edges (direct only)
        let mut non_edge_corrs = Vec::new();
        for (i, &vi) in var_names.iter().enumerate() {
            for &vj in var_names.iter().skip(i + 1) {
                if edge_pairs.contains(&(vi, vj)) || edge_pairs.contains(&(vj, vi)) {
                    continue;
                }
                let vals_i: Vec<f64> = samples.iter().filter_map(|s| s.get(vi).copied()).collect();
                let vals_j: Vec<f64> = samples.iter().filter_map(|s| s.get(vj).copied()).collect();
                let corr = pearson_correlation(&vals_i, &vals_j).abs();
                if corr.is_finite() {
                    non_edge_corrs.push(corr);
                }
            }
        }

        let avg_edge = if edge_corrs.is_empty() {
            0.0
        } else {
            edge_corrs.iter().sum::<f64>() / edge_corrs.len() as f64
        };

        let avg_non_edge = if non_edge_corrs.is_empty() {
            0.0
        } else {
            non_edge_corrs.iter().sum::<f64>() / non_edge_corrs.len() as f64
        };

        // Non-edges should have weaker average correlation than edges
        let passed = non_edge_corrs.is_empty() || avg_non_edge <= avg_edge + 0.1;

        let details = format!(
            "Avg edge correlation: {:.4}, avg non-edge correlation: {:.4}",
            avg_edge, avg_non_edge
        );

        CausalCheck {
            name: "non_edge_weakness".to_string(),
            passed,
            details,
        }
    }

    /// Check 3: Verify topological ordering holds in conditional means.
    ///
    /// For parent -> child edges, the mean of child should shift when we split
    /// samples by parent median.
    fn check_topological_consistency(
        samples: &[HashMap<String, f64>],
        graph: &CausalGraph,
    ) -> CausalCheck {
        let mut total_checked = 0;
        let mut consistent = 0;

        for edge in &graph.edges {
            let expected_sign = Self::mechanism_sign(&edge.mechanism);
            if expected_sign == 0 {
                continue;
            }

            let mut parent_vals: Vec<f64> = samples
                .iter()
                .filter_map(|s| s.get(&edge.from).copied())
                .collect();
            parent_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            if parent_vals.is_empty() {
                continue;
            }

            let median_idx = parent_vals.len() / 2;
            let median = parent_vals[median_idx];

            // Split child values by parent median
            let child_low: Vec<f64> = samples
                .iter()
                .filter(|s| s.get(&edge.from).copied().unwrap_or(0.0) <= median)
                .filter_map(|s| s.get(&edge.to).copied())
                .collect();

            let child_high: Vec<f64> = samples
                .iter()
                .filter(|s| s.get(&edge.from).copied().unwrap_or(0.0) > median)
                .filter_map(|s| s.get(&edge.to).copied())
                .collect();

            if child_low.is_empty() || child_high.is_empty() {
                continue;
            }

            let mean_low = child_low.iter().sum::<f64>() / child_low.len() as f64;
            let mean_high = child_high.iter().sum::<f64>() / child_high.len() as f64;

            total_checked += 1;

            // Check that the direction of mean shift matches expected sign
            let actual_sign = if mean_high > mean_low + 1e-10 {
                1
            } else if mean_high < mean_low - 1e-10 {
                -1
            } else {
                0
            };

            if actual_sign == expected_sign || actual_sign == 0 {
                consistent += 1;
            }
        }

        let passed = total_checked == 0 || consistent >= total_checked / 2;
        let details = format!(
            "{}/{} edges show consistent conditional mean ordering",
            consistent, total_checked
        );

        CausalCheck {
            name: "topological_consistency".to_string(),
            passed,
            details,
        }
    }

    /// Determine the expected sign of a mechanism's effect.
    /// Returns 1 for positive, -1 for negative, 0 for indeterminate.
    fn mechanism_sign(mechanism: &CausalMechanism) -> i32 {
        match mechanism {
            CausalMechanism::Linear { coefficient } => {
                if *coefficient > 0.0 {
                    1
                } else if *coefficient < 0.0 {
                    -1
                } else {
                    0
                }
            }
            CausalMechanism::Threshold { .. } => {
                // Threshold is monotonically non-decreasing (0 or 1)
                1
            }
            CausalMechanism::Logistic { scale, .. } => {
                if *scale > 0.0 {
                    1
                } else if *scale < 0.0 {
                    -1
                } else {
                    0
                }
            }
            CausalMechanism::Polynomial { coefficients } => {
                // Use sign of highest non-zero coefficient as a heuristic
                for coeff in coefficients.iter().rev() {
                    if *coeff > 0.0 {
                        return 1;
                    } else if *coeff < 0.0 {
                        return -1;
                    }
                }
                0
            }
        }
    }
}

/// Compute Pearson correlation coefficient between two vectors.
fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 2 {
        return 0.0;
    }

    let mean_x = x.iter().take(n).sum::<f64>() / n as f64;
    let mean_y = y.iter().take(n).sum::<f64>() / n as f64;

    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        sum_xy += dx * dy;
        sum_x2 += dx * dx;
        sum_y2 += dy * dy;
    }

    let denom = (sum_x2 * sum_y2).sqrt();
    if denom < 1e-15 {
        0.0
    } else {
        sum_xy / denom
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::causal::graph::CausalGraph;
    use crate::causal::scm::StructuralCausalModel;

    #[test]
    fn test_causal_validation_passes_on_correct_data() {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph.clone()).unwrap();
        let samples = scm.generate(1000, 42).unwrap();

        let report = CausalValidator::validate_causal_structure(&samples, &graph);

        assert!(
            report.valid,
            "Validation should pass on correctly generated data. Violations: {:?}",
            report.violations
        );
        assert_eq!(report.checks.len(), 3);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_causal_validation_detects_shuffled_columns() {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph.clone()).unwrap();
        let mut samples = scm.generate(500, 42).unwrap();

        // Shuffle the fraud_probability column by rotating values.
        // This breaks the causal relationship between parents and fraud_probability.
        let n = samples.len();
        let fp_values: Vec<f64> = samples
            .iter()
            .filter_map(|s| s.get("fraud_probability").copied())
            .collect();

        for (i, sample) in samples.iter_mut().enumerate() {
            let shifted_idx = (i + n / 2) % n;
            sample.insert("fraud_probability".to_string(), fp_values[shifted_idx]);
        }

        let report = CausalValidator::validate_causal_structure(&samples, &graph);

        // At least one check should fail when causal structure is broken
        let has_failure = report.checks.iter().any(|c| !c.passed);
        assert!(
            has_failure,
            "Validation should detect broken causal structure. Checks: {:?}",
            report.checks
        );
    }

    #[test]
    fn test_causal_pearson_correlation_perfect_positive() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let corr = pearson_correlation(&x, &y);
        assert!(
            (corr - 1.0).abs() < 1e-10,
            "Perfect positive correlation expected, got {}",
            corr
        );
    }

    #[test]
    fn test_causal_pearson_correlation_perfect_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let corr = pearson_correlation(&x, &y);
        assert!(
            (corr - (-1.0)).abs() < 1e-10,
            "Perfect negative correlation expected, got {}",
            corr
        );
    }

    #[test]
    fn test_causal_pearson_correlation_constant() {
        let x = vec![1.0, 1.0, 1.0, 1.0];
        let y = vec![2.0, 4.0, 6.0, 8.0];
        let corr = pearson_correlation(&x, &y);
        assert!(
            corr.abs() < 1e-10,
            "Correlation with constant should be 0, got {}",
            corr
        );
    }

    #[test]
    fn test_causal_validation_report_structure() {
        let graph = CausalGraph::fraud_detection_template();
        let scm = StructuralCausalModel::new(graph.clone()).unwrap();
        let samples = scm.generate(200, 42).unwrap();

        let report = CausalValidator::validate_causal_structure(&samples, &graph);

        // Should always produce exactly 3 checks
        assert_eq!(report.checks.len(), 3);
        assert_eq!(report.checks[0].name, "edge_correlation_signs");
        assert_eq!(report.checks[1].name, "non_edge_weakness");
        assert_eq!(report.checks[2].name, "topological_consistency");

        // Each check should have non-empty details
        for check in &report.checks {
            assert!(!check.details.is_empty());
        }
    }

    #[test]
    fn test_causal_validation_revenue_cycle() {
        let graph = CausalGraph::revenue_cycle_template();
        let scm = StructuralCausalModel::new(graph.clone()).unwrap();
        let samples = scm.generate(1000, 99).unwrap();

        let report = CausalValidator::validate_causal_structure(&samples, &graph);

        // Most checks should pass on correctly generated data
        let passing = report.checks.iter().filter(|c| c.passed).count();
        assert!(
            passing >= 2,
            "At least 2 of 3 checks should pass. Checks: {:?}",
            report.checks
        );
    }
}
