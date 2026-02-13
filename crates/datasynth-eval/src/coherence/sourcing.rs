//! Source-to-Contract (S2C) evaluator.
//!
//! Validates sourcing chain completeness including project-to-contract flow,
//! bid scoring consistency, evaluation-recommendation matching,
//! spend concentration, and scorecard coverage.

use crate::error::EvalResult;
use serde::{Deserialize, Serialize};

/// Thresholds for S2C evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcingThresholds {
    /// Minimum RFx completion rate (projects that reach RFx stage).
    pub min_rfx_completion: f64,
    /// Minimum bid receipt rate (RFx events that receive bids).
    pub min_bid_receipt: f64,
    /// Minimum ranking consistency (rankings match scores).
    pub min_ranking_consistency: f64,
    /// Minimum evaluation completion rate.
    pub min_evaluation_completion: f64,
}

impl Default for SourcingThresholds {
    fn default() -> Self {
        Self {
            min_rfx_completion: 0.90,
            min_bid_receipt: 0.80,
            min_ranking_consistency: 0.95,
            min_evaluation_completion: 0.85,
        }
    }
}

/// Sourcing project data for chain validation.
#[derive(Debug, Clone)]
pub struct SourcingProjectData {
    /// Project identifier.
    pub project_id: String,
    /// Whether an RFx event was created.
    pub has_rfx: bool,
    /// Whether bids were received.
    pub has_bids: bool,
    /// Whether evaluation was completed.
    pub has_evaluation: bool,
    /// Whether a contract was awarded.
    pub has_contract: bool,
}

/// Bid evaluation data for scoring validation.
#[derive(Debug, Clone)]
pub struct BidEvaluationData {
    /// Evaluation identifier.
    pub evaluation_id: String,
    /// Criteria weights (should sum to 1.0).
    pub criteria_weights: Vec<f64>,
    /// Bid scores (one per bid, computed from weighted criteria).
    pub bid_scores: Vec<f64>,
    /// Bid rankings (1 = best).
    pub bid_rankings: Vec<u32>,
    /// Recommended vendor index (into bid arrays).
    pub recommended_vendor_idx: Option<usize>,
}

/// Spend analysis data.
#[derive(Debug, Clone)]
pub struct SpendAnalysisData {
    /// Vendor spend amounts.
    pub vendor_spends: Vec<f64>,
}

/// Scorecard coverage data.
#[derive(Debug, Clone)]
pub struct ScorecardCoverageData {
    /// Total active vendors.
    pub total_active_vendors: usize,
    /// Vendors with scorecards.
    pub vendors_with_scorecards: usize,
}

/// Results of S2C evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcingEvaluation {
    /// RFx completion rate: projects that reach RFx stage.
    pub rfx_completion_rate: f64,
    /// Bid receipt rate: RFx events that receive bids.
    pub bid_receipt_rate: f64,
    /// Evaluation completion rate.
    pub evaluation_completion_rate: f64,
    /// Contract award rate.
    pub contract_award_rate: f64,
    /// Criteria weight compliance: fraction of evaluations with weights summing to 1.0.
    pub criteria_weight_compliance: f64,
    /// Ranking consistency: fraction where rankings match score ordering.
    pub ranking_consistency: f64,
    /// Recommendation match rate: recommended vendor = top-ranked bid.
    pub recommendation_match_rate: f64,
    /// HHI (Herfindahl-Hirschman Index) for spend concentration.
    pub spend_hhi: f64,
    /// Scorecard coverage: fraction of active vendors with scorecards.
    pub scorecard_coverage: f64,
    /// Total projects evaluated.
    pub total_projects: usize,
    /// Overall pass/fail.
    pub passes: bool,
    /// Issues found.
    pub issues: Vec<String>,
}

/// Evaluator for S2C chain coherence.
pub struct SourcingEvaluator {
    thresholds: SourcingThresholds,
}

impl SourcingEvaluator {
    /// Create a new evaluator with default thresholds.
    pub fn new() -> Self {
        Self {
            thresholds: SourcingThresholds::default(),
        }
    }

    /// Create with custom thresholds.
    pub fn with_thresholds(thresholds: SourcingThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate sourcing data.
    pub fn evaluate(
        &self,
        projects: &[SourcingProjectData],
        evaluations: &[BidEvaluationData],
        spend: &Option<SpendAnalysisData>,
        scorecard: &Option<ScorecardCoverageData>,
    ) -> EvalResult<SourcingEvaluation> {
        let mut issues = Vec::new();
        let total_projects = projects.len();

        // 1. Chain completion rates
        let rfx_count = projects.iter().filter(|p| p.has_rfx).count();
        let bid_count = projects.iter().filter(|p| p.has_bids).count();
        let eval_count = projects.iter().filter(|p| p.has_evaluation).count();
        let contract_count = projects.iter().filter(|p| p.has_contract).count();

        let rfx_completion_rate = if total_projects > 0 {
            rfx_count as f64 / total_projects as f64
        } else {
            1.0
        };
        let bid_receipt_rate = if rfx_count > 0 {
            bid_count as f64 / rfx_count as f64
        } else {
            1.0
        };
        let evaluation_completion_rate = if bid_count > 0 {
            eval_count as f64 / bid_count as f64
        } else {
            1.0
        };
        let contract_award_rate = if eval_count > 0 {
            contract_count as f64 / eval_count as f64
        } else {
            1.0
        };

        // 2. Criteria weight compliance: weights sum to 1.0 (±0.01)
        let weight_ok = evaluations
            .iter()
            .filter(|e| {
                if e.criteria_weights.is_empty() {
                    return true;
                }
                let sum: f64 = e.criteria_weights.iter().sum();
                (sum - 1.0).abs() <= 0.01
            })
            .count();
        let criteria_weight_compliance = if evaluations.is_empty() {
            1.0
        } else {
            weight_ok as f64 / evaluations.len() as f64
        };

        // 3. Ranking consistency: rankings should match score ordering
        let ranking_ok = evaluations
            .iter()
            .filter(|e| {
                if e.bid_scores.len() != e.bid_rankings.len() || e.bid_scores.is_empty() {
                    return true;
                }
                // Create pairs (score, ranking) and sort by score descending
                let mut pairs: Vec<(f64, u32)> = e
                    .bid_scores
                    .iter()
                    .zip(e.bid_rankings.iter())
                    .map(|(&s, &r)| (s, r))
                    .collect();
                pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                // Rankings should be ascending
                pairs.windows(2).all(|w| w[0].1 <= w[1].1)
            })
            .count();
        let ranking_consistency = if evaluations.is_empty() {
            1.0
        } else {
            ranking_ok as f64 / evaluations.len() as f64
        };

        // 4. Recommendation match: recommended = top-ranked (rank 1)
        let rec_ok = evaluations
            .iter()
            .filter(|e| {
                if let Some(rec_idx) = e.recommended_vendor_idx {
                    if rec_idx < e.bid_rankings.len() {
                        return e.bid_rankings[rec_idx] == 1;
                    }
                }
                true // No recommendation = not checked
            })
            .count();
        let recommendation_match_rate = if evaluations.is_empty() {
            1.0
        } else {
            rec_ok as f64 / evaluations.len() as f64
        };

        // 5. Spend HHI
        let spend_hhi = if let Some(ref sp) = spend {
            let total_spend: f64 = sp.vendor_spends.iter().sum();
            if total_spend > 0.0 {
                sp.vendor_spends
                    .iter()
                    .map(|s| (s / total_spend).powi(2))
                    .sum::<f64>()
            } else {
                0.0
            }
        } else {
            0.0
        };

        // 6. Scorecard coverage
        let scorecard_coverage = if let Some(ref sc) = scorecard {
            if sc.total_active_vendors > 0 {
                sc.vendors_with_scorecards as f64 / sc.total_active_vendors as f64
            } else {
                1.0
            }
        } else {
            1.0
        };

        // Check thresholds
        if rfx_completion_rate < self.thresholds.min_rfx_completion {
            issues.push(format!(
                "RFx completion rate {:.3} < {:.3}",
                rfx_completion_rate, self.thresholds.min_rfx_completion
            ));
        }
        if bid_receipt_rate < self.thresholds.min_bid_receipt {
            issues.push(format!(
                "Bid receipt rate {:.3} < {:.3}",
                bid_receipt_rate, self.thresholds.min_bid_receipt
            ));
        }
        if ranking_consistency < self.thresholds.min_ranking_consistency {
            issues.push(format!(
                "Ranking consistency {:.3} < {:.3}",
                ranking_consistency, self.thresholds.min_ranking_consistency
            ));
        }

        let passes = issues.is_empty();

        Ok(SourcingEvaluation {
            rfx_completion_rate,
            bid_receipt_rate,
            evaluation_completion_rate,
            contract_award_rate,
            criteria_weight_compliance,
            ranking_consistency,
            recommendation_match_rate,
            spend_hhi,
            scorecard_coverage,
            total_projects,
            passes,
            issues,
        })
    }
}

impl Default for SourcingEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sourcing_chain() {
        let evaluator = SourcingEvaluator::new();
        let projects = vec![SourcingProjectData {
            project_id: "SP001".to_string(),
            has_rfx: true,
            has_bids: true,
            has_evaluation: true,
            has_contract: true,
        }];
        let evals = vec![BidEvaluationData {
            evaluation_id: "EV001".to_string(),
            criteria_weights: vec![0.4, 0.3, 0.3],
            bid_scores: vec![90.0, 80.0, 70.0],
            bid_rankings: vec![1, 2, 3],
            recommended_vendor_idx: Some(0),
        }];

        let result = evaluator.evaluate(&projects, &evals, &None, &None).unwrap();
        assert!(result.passes);
        assert_eq!(result.rfx_completion_rate, 1.0);
        assert_eq!(result.ranking_consistency, 1.0);
    }

    #[test]
    fn test_inconsistent_rankings() {
        let evaluator = SourcingEvaluator::new();
        let evals = vec![BidEvaluationData {
            evaluation_id: "EV001".to_string(),
            criteria_weights: vec![0.5, 0.5],
            bid_scores: vec![90.0, 80.0],
            bid_rankings: vec![2, 1], // Wrong: highest score should be rank 1
            recommended_vendor_idx: None,
        }];

        let result = evaluator.evaluate(&[], &evals, &None, &None).unwrap();
        assert_eq!(result.ranking_consistency, 0.0);
    }

    #[test]
    fn test_empty_data() {
        let evaluator = SourcingEvaluator::new();
        let result = evaluator.evaluate(&[], &[], &None, &None).unwrap();
        assert!(result.passes);
    }

    #[test]
    fn test_spend_hhi() {
        let evaluator = SourcingEvaluator::new();
        let spend = Some(SpendAnalysisData {
            vendor_spends: vec![50.0, 50.0],
        });
        let result = evaluator.evaluate(&[], &[], &spend, &None).unwrap();
        assert!((result.spend_hhi - 0.5).abs() < 0.001); // 0.25 + 0.25 = 0.5
    }
}
