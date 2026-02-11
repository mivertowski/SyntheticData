//! Pareto privacy-utility frontier analysis.
//!
//! This module provides tools for exploring and navigating the tradeoff between
//! privacy (measured as differential privacy epsilon) and data utility. The
//! [`ParetoFrontier`] identifies the set of non-dominated operating points
//! where no improvement in privacy can be achieved without sacrificing utility,
//! and vice versa.
//!
//! # Privacy-Utility Tradeoff
//!
//! In differential privacy, lower epsilon means stronger privacy but typically
//! lower data utility. The Pareto frontier identifies the optimal tradeoff
//! curve, filtering out dominated points (where another point offers both
//! better privacy AND better utility).
//!
//! # Example
//!
//! ```
//! use datasynth_fingerprint::privacy::pareto::{ParetoFrontier, ParetoPoint};
//!
//! let frontier = ParetoFrontier;
//!
//! // Explore utility at various epsilon values
//! let points = ParetoFrontier::explore(
//!     &[0.1, 0.5, 1.0, 2.0, 5.0],
//!     |eps| ParetoPoint {
//!         epsilon: eps,
//!         delta: None,
//!         utility_score: 1.0 - (-eps).exp(), // utility increases with epsilon
//!         benford_mad: 0.05 / eps,           // lower MAD with higher epsilon
//!         correlation_score: (1.0 - (-eps).exp()).min(1.0),
//!     },
//! );
//!
//! // Find minimum epsilon for a target utility
//! let recommended = ParetoFrontier::recommend(&points, 0.8);
//! assert!(recommended.is_some());
//! ```

use serde::{Deserialize, Serialize};

/// A single point on the privacy-utility tradeoff curve.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParetoPoint {
    /// Differential privacy epsilon (lower = more private).
    pub epsilon: f64,
    /// Optional delta parameter for approximate DP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,
    /// Overall utility score (higher = better utility, 0.0 to 1.0).
    pub utility_score: f64,
    /// Benford's Law Mean Absolute Deviation (lower = better).
    pub benford_mad: f64,
    /// Correlation preservation score (higher = better, 0.0 to 1.0).
    pub correlation_score: f64,
}

/// Pareto frontier analysis for privacy-utility tradeoffs.
///
/// Identifies non-dominated points on the privacy-utility curve and
/// provides recommendations for selecting privacy parameters.
pub struct ParetoFrontier;

impl ParetoFrontier {
    /// Explore the privacy-utility frontier by evaluating a utility function
    /// at each epsilon value.
    ///
    /// Returns only the Pareto-optimal (non-dominated) points, sorted by
    /// increasing epsilon. A point is dominated if another point has both
    /// lower (or equal) epsilon AND higher (or equal) utility score, with
    /// at least one strict inequality.
    pub fn explore(epsilons: &[f64], utility_fn: impl Fn(f64) -> ParetoPoint) -> Vec<ParetoPoint> {
        if epsilons.is_empty() {
            return Vec::new();
        }

        // Evaluate utility at each epsilon
        let mut points: Vec<ParetoPoint> = epsilons.iter().map(|&eps| utility_fn(eps)).collect();

        // Sort by epsilon ascending
        points.sort_by(|a, b| a.epsilon.total_cmp(&b.epsilon));

        // Filter to Pareto-optimal points
        Self::filter_dominated(&points)
    }

    /// Recommend the minimum epsilon that achieves at least the target utility.
    ///
    /// Searches through the provided points (which should be Pareto-optimal)
    /// and returns the smallest epsilon whose utility score meets or exceeds
    /// the target.
    ///
    /// Returns `None` if no point achieves the target utility.
    pub fn recommend(points: &[ParetoPoint], target_utility: f64) -> Option<f64> {
        let mut candidates: Vec<&ParetoPoint> = points
            .iter()
            .filter(|p| p.utility_score >= target_utility)
            .collect();

        candidates.sort_by(|a, b| a.epsilon.total_cmp(&b.epsilon));
        candidates.first().map(|p| p.epsilon)
    }

    /// Check if point `a` is dominated by point `b`.
    ///
    /// Point `b` dominates `a` if `b` has lower or equal epsilon AND higher
    /// or equal utility, with at least one strict inequality. In other words,
    /// `b` is at least as good as `a` in both dimensions and strictly better
    /// in at least one.
    pub fn is_dominated(a: &ParetoPoint, b: &ParetoPoint) -> bool {
        let b_leq_eps = b.epsilon <= a.epsilon;
        let b_geq_util = b.utility_score >= a.utility_score;
        let strictly_better = b.epsilon < a.epsilon || b.utility_score > a.utility_score;

        b_leq_eps && b_geq_util && strictly_better
    }

    /// Filter out dominated points, returning only Pareto-optimal ones.
    fn filter_dominated(points: &[ParetoPoint]) -> Vec<ParetoPoint> {
        let mut result = Vec::new();

        for (i, point) in points.iter().enumerate() {
            let mut dominated = false;
            for (j, other) in points.iter().enumerate() {
                if i != j && Self::is_dominated(point, other) {
                    dominated = true;
                    break;
                }
            }
            if !dominated {
                result.push(point.clone());
            }
        }

        // Sort by epsilon ascending
        result.sort_by(|a, b| a.epsilon.total_cmp(&b.epsilon));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotonic_utility_all_pareto_optimal() {
        // When utility strictly increases with epsilon, all points are Pareto-optimal
        let points = ParetoFrontier::explore(&[0.1, 0.5, 1.0, 2.0, 5.0], |eps| ParetoPoint {
            epsilon: eps,
            delta: None,
            utility_score: 1.0 - (-eps).exp(),
            benford_mad: 0.05 / eps,
            correlation_score: (1.0 - (-0.5 * eps).exp()).min(1.0),
        });

        // All 5 points should be on the frontier (monotonically increasing utility)
        assert_eq!(points.len(), 5);

        // Should be sorted by epsilon
        for i in 1..points.len() {
            assert!(
                points[i].epsilon > points[i - 1].epsilon,
                "Points should be sorted by epsilon"
            );
            assert!(
                points[i].utility_score > points[i - 1].utility_score,
                "Utility should increase with epsilon for this function"
            );
        }
    }

    #[test]
    fn test_recommend_returns_correct_epsilon() {
        let points = ParetoFrontier::explore(&[0.1, 0.5, 1.0, 2.0, 5.0], |eps| ParetoPoint {
            epsilon: eps,
            delta: None,
            utility_score: 1.0 - (-eps).exp(),
            benford_mad: 0.05 / eps,
            correlation_score: 0.9,
        });

        // Utility at eps=1.0 is ~0.632, at eps=2.0 is ~0.865
        let rec = ParetoFrontier::recommend(&points, 0.8);
        assert!(rec.is_some());
        let eps = rec.expect("recommendation exists");
        assert!(
            (eps - 2.0).abs() < 1e-10,
            "Should recommend eps=2.0 for utility target 0.8, got {}",
            eps
        );

        // Very high target that nothing achieves
        let rec_impossible = ParetoFrontier::recommend(&points, 0.9999);
        // utility at eps=5.0 is ~0.9933, which is < 0.9999
        assert!(
            rec_impossible.is_none(),
            "Should return None for unachievable target"
        );
    }

    #[test]
    fn test_dominated_points_filtered() {
        // Create points where some are dominated
        let points = ParetoFrontier::explore(&[0.5, 1.0, 1.5, 2.0], |eps| {
            // At eps=1.5, utility is LOWER than at eps=1.0 (anomalous)
            let utility = if (eps - 1.5).abs() < 1e-10 {
                0.3 // worse utility than eps=1.0
            } else {
                1.0 - (-eps).exp()
            };
            ParetoPoint {
                epsilon: eps,
                delta: None,
                utility_score: utility,
                benford_mad: 0.05,
                correlation_score: 0.9,
            }
        });

        // eps=1.5 with utility=0.3 is dominated by eps=1.0 with utility~=0.632
        // So it should be filtered out
        let epsilons: Vec<f64> = points.iter().map(|p| p.epsilon).collect();
        assert!(
            !epsilons.contains(&1.5),
            "eps=1.5 should be filtered as dominated, got frontier: {:?}",
            epsilons
        );
        // Other points should remain
        assert!(epsilons.contains(&0.5));
        assert!(epsilons.contains(&1.0));
        assert!(epsilons.contains(&2.0));
    }

    #[test]
    fn test_is_dominated() {
        let a = ParetoPoint {
            epsilon: 2.0,
            delta: None,
            utility_score: 0.5,
            benford_mad: 0.02,
            correlation_score: 0.9,
        };
        let b = ParetoPoint {
            epsilon: 1.0,
            delta: None,
            utility_score: 0.6,
            benford_mad: 0.03,
            correlation_score: 0.8,
        };

        // b dominates a: lower epsilon (1.0 < 2.0) AND higher utility (0.6 > 0.5)
        assert!(ParetoFrontier::is_dominated(&a, &b));
        // a does NOT dominate b
        assert!(!ParetoFrontier::is_dominated(&b, &a));
    }

    #[test]
    fn test_same_point_not_dominated() {
        let a = ParetoPoint {
            epsilon: 1.0,
            delta: None,
            utility_score: 0.5,
            benford_mad: 0.02,
            correlation_score: 0.9,
        };

        // A point does not dominate itself (no strict inequality)
        assert!(!ParetoFrontier::is_dominated(&a, &a));
    }

    #[test]
    fn test_empty_input_handled() {
        let points = ParetoFrontier::explore(&[], |eps| ParetoPoint {
            epsilon: eps,
            delta: None,
            utility_score: 0.0,
            benford_mad: 0.0,
            correlation_score: 0.0,
        });
        assert!(points.is_empty());

        let rec = ParetoFrontier::recommend(&[], 0.5);
        assert!(rec.is_none());
    }

    #[test]
    fn test_single_point_frontier() {
        let points = ParetoFrontier::explore(&[1.0], |eps| ParetoPoint {
            epsilon: eps,
            delta: Some(1e-5),
            utility_score: 0.7,
            benford_mad: 0.02,
            correlation_score: 0.85,
        });

        assert_eq!(points.len(), 1);
        assert!((points[0].epsilon - 1.0).abs() < 1e-10);
        assert!((points[0].utility_score - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_recommend_with_exact_match() {
        let points = vec![
            ParetoPoint {
                epsilon: 0.5,
                delta: None,
                utility_score: 0.3,
                benford_mad: 0.05,
                correlation_score: 0.5,
            },
            ParetoPoint {
                epsilon: 1.0,
                delta: None,
                utility_score: 0.6,
                benford_mad: 0.03,
                correlation_score: 0.7,
            },
            ParetoPoint {
                epsilon: 2.0,
                delta: None,
                utility_score: 0.9,
                benford_mad: 0.01,
                correlation_score: 0.95,
            },
        ];

        // Target exactly matches a point's utility
        let rec = ParetoFrontier::recommend(&points, 0.6);
        assert_eq!(rec, Some(1.0));
    }
}
