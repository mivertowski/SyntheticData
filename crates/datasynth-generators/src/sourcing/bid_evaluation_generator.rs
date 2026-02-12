//! Bid evaluation and award recommendation generator.

use datasynth_core::models::sourcing::{
    AwardRecommendation, BidEvaluation, BidEvaluationEntry, RankedBid, RfxEvent, SupplierBid,
};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Generates bid evaluations and award recommendations.
pub struct BidEvaluationGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl BidEvaluationGenerator {
    /// Create a new bid evaluation generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::RfxEvent),
        }
    }

    /// Evaluate bids for an RFx event and produce rankings.
    pub fn evaluate(
        &mut self,
        rfx: &RfxEvent,
        bids: &[SupplierBid],
        evaluator_id: &str,
    ) -> BidEvaluation {
        // Only consider compliant, on-time bids
        let eligible_bids: Vec<&SupplierBid> = bids
            .iter()
            .filter(|b| b.is_compliant && b.is_on_time)
            .collect();

        // Find min/max total amounts for price normalization
        let amounts: Vec<f64> = eligible_bids
            .iter()
            .map(|b| b.total_amount.to_string().parse::<f64>().unwrap_or(0.0))
            .collect();
        let min_amount = amounts.iter().cloned().fold(f64::MAX, f64::min);
        let max_amount = amounts.iter().cloned().fold(0.0f64, f64::max);
        let amount_range = (max_amount - min_amount).max(1.0);

        let mut ranked_bids: Vec<RankedBid> = eligible_bids
            .iter()
            .map(|bid| {
                let bid_amount: f64 = bid.total_amount.to_string().parse().unwrap_or(0.0);

                let mut criterion_scores = Vec::new();
                let mut total_score = 0.0;
                let mut price_score_val = 0.0;
                let mut quality_score_val = 0.0;

                for criterion in &rfx.criteria {
                    let (raw_score, is_price) = if criterion.name == "Price" {
                        // Price: lower is better
                        let score = 100.0 * (1.0 - (bid_amount - min_amount) / amount_range);
                        (score, true)
                    } else {
                        // Other criteria: random score
                        (self.rng.gen_range(50.0..=100.0), false)
                    };

                    let weighted = raw_score * criterion.weight;
                    total_score += weighted;

                    if is_price {
                        price_score_val = raw_score;
                    } else {
                        quality_score_val += weighted;
                    }

                    criterion_scores.push(BidEvaluationEntry {
                        criterion_name: criterion.name.clone(),
                        raw_score,
                        weight: criterion.weight,
                        weighted_score: weighted,
                    });
                }

                RankedBid {
                    bid_id: bid.bid_id.clone(),
                    vendor_id: bid.vendor_id.clone(),
                    rank: 0, // Will be set after sorting
                    total_score,
                    price_score: price_score_val,
                    quality_score: quality_score_val,
                    total_amount: bid.total_amount,
                    criterion_scores,
                }
            })
            .collect();

        // Sort by total score (descending)
        ranked_bids.sort_by(|a, b| {
            b.total_score
                .partial_cmp(&a.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (i, bid) in ranked_bids.iter_mut().enumerate() {
            bid.rank = (i + 1) as u32;
        }

        let (recommendation, rec_vendor, rec_bid) = if ranked_bids.is_empty() {
            (AwardRecommendation::Reject, None, None)
        } else {
            (
                AwardRecommendation::Award,
                Some(ranked_bids[0].vendor_id.clone()),
                Some(ranked_bids[0].bid_id.clone()),
            )
        };

        BidEvaluation {
            evaluation_id: self.uuid_factory.next().to_string(),
            rfx_id: rfx.rfx_id.clone(),
            company_code: rfx.company_code.clone(),
            evaluator_id: evaluator_id.to_string(),
            ranked_bids,
            recommendation,
            recommended_vendor_id: rec_vendor,
            recommended_bid_id: rec_bid,
            notes: None,
            is_finalized: true,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use datasynth_core::models::sourcing::{
        BidLineItem, BidStatus, RfxEvaluationCriterion, RfxEvent, RfxLineItem, RfxStatus, RfxType,
        ScoringMethod,
    };
    use rust_decimal::Decimal;

    fn test_rfx() -> RfxEvent {
        RfxEvent {
            rfx_id: "RFX-001".to_string(),
            rfx_type: RfxType::Rfp,
            company_code: "C001".to_string(),
            title: "Test RFx".to_string(),
            description: "Test".to_string(),
            status: RfxStatus::Awarded,
            sourcing_project_id: "SP-001".to_string(),
            category_id: "CAT-001".to_string(),
            scoring_method: ScoringMethod::BestValue,
            criteria: vec![
                RfxEvaluationCriterion {
                    name: "Price".to_string(),
                    weight: 0.40,
                    description: "Cost".to_string(),
                },
                RfxEvaluationCriterion {
                    name: "Quality".to_string(),
                    weight: 0.35,
                    description: "Quality".to_string(),
                },
                RfxEvaluationCriterion {
                    name: "Delivery".to_string(),
                    weight: 0.25,
                    description: "Delivery".to_string(),
                },
            ],
            line_items: vec![RfxLineItem {
                item_number: 1,
                description: "Item A".to_string(),
                material_id: None,
                quantity: Decimal::from(100),
                uom: "EA".to_string(),
                target_price: Some(Decimal::from(50)),
            }],
            invited_vendors: vec!["V001".to_string(), "V002".to_string()],
            publish_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            response_deadline: NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            bid_count: 2,
            owner_id: "BUYER-01".to_string(),
            awarded_vendor_id: None,
            awarded_bid_id: None,
        }
    }

    fn test_bids() -> Vec<SupplierBid> {
        vec![
            SupplierBid {
                bid_id: "BID-001".to_string(),
                rfx_id: "RFX-001".to_string(),
                vendor_id: "V001".to_string(),
                company_code: "C001".to_string(),
                status: BidStatus::Submitted,
                submission_date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                line_items: vec![BidLineItem {
                    item_number: 1,
                    unit_price: Decimal::from(45),
                    quantity: Decimal::from(100),
                    total_amount: Decimal::from(4500),
                    lead_time_days: 10,
                    notes: None,
                }],
                total_amount: Decimal::from(4500),
                validity_days: 60,
                payment_terms: "NET30".to_string(),
                delivery_terms: Some("FCA".to_string()),
                technical_summary: None,
                is_on_time: true,
                is_compliant: true,
            },
            SupplierBid {
                bid_id: "BID-002".to_string(),
                rfx_id: "RFX-001".to_string(),
                vendor_id: "V002".to_string(),
                company_code: "C001".to_string(),
                status: BidStatus::Submitted,
                submission_date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                line_items: vec![BidLineItem {
                    item_number: 1,
                    unit_price: Decimal::from(55),
                    quantity: Decimal::from(100),
                    total_amount: Decimal::from(5500),
                    lead_time_days: 7,
                    notes: None,
                }],
                total_amount: Decimal::from(5500),
                validity_days: 60,
                payment_terms: "NET45".to_string(),
                delivery_terms: Some("FCA".to_string()),
                technical_summary: None,
                is_on_time: true,
                is_compliant: true,
            },
        ]
    }

    #[test]
    fn test_basic_evaluation() {
        let mut gen = BidEvaluationGenerator::new(42);
        let rfx = test_rfx();
        let bids = test_bids();
        let eval = gen.evaluate(&rfx, &bids, "EVAL-01");

        assert!(!eval.evaluation_id.is_empty());
        assert_eq!(eval.rfx_id, "RFX-001");
        assert_eq!(eval.company_code, "C001");
        assert_eq!(eval.evaluator_id, "EVAL-01");
        assert!(eval.is_finalized);
        assert_eq!(eval.ranked_bids.len(), 2);
        assert!(eval.recommended_vendor_id.is_some());
        assert!(eval.recommended_bid_id.is_some());
        assert!(matches!(eval.recommendation, AwardRecommendation::Award));
    }

    #[test]
    fn test_deterministic() {
        let rfx = test_rfx();
        let bids = test_bids();

        let mut gen1 = BidEvaluationGenerator::new(42);
        let mut gen2 = BidEvaluationGenerator::new(42);

        let r1 = gen1.evaluate(&rfx, &bids, "EVAL-01");
        let r2 = gen2.evaluate(&rfx, &bids, "EVAL-01");

        assert_eq!(r1.evaluation_id, r2.evaluation_id);
        assert_eq!(r1.ranked_bids.len(), r2.ranked_bids.len());
        for (a, b) in r1.ranked_bids.iter().zip(r2.ranked_bids.iter()) {
            assert_eq!(a.bid_id, b.bid_id);
            assert_eq!(a.rank, b.rank);
            assert_eq!(a.total_score, b.total_score);
        }
        assert_eq!(r1.recommended_vendor_id, r2.recommended_vendor_id);
    }

    #[test]
    fn test_ranking_order() {
        let mut gen = BidEvaluationGenerator::new(42);
        let rfx = test_rfx();
        let bids = test_bids();
        let eval = gen.evaluate(&rfx, &bids, "EVAL-01");

        // Ranks should be sequential starting at 1
        for (i, ranked) in eval.ranked_bids.iter().enumerate() {
            assert_eq!(ranked.rank, (i + 1) as u32);
        }

        // Scores should be in descending order
        for window in eval.ranked_bids.windows(2) {
            assert!(window[0].total_score >= window[1].total_score);
        }

        // Recommended vendor should be rank 1
        assert_eq!(
            eval.recommended_vendor_id.as_ref().unwrap(),
            &eval.ranked_bids[0].vendor_id
        );
    }

    #[test]
    fn test_non_compliant_bids_excluded() {
        let mut gen = BidEvaluationGenerator::new(42);
        let rfx = test_rfx();
        let mut bids = test_bids();
        // Make second bid non-compliant
        bids[1].is_compliant = false;

        let eval = gen.evaluate(&rfx, &bids, "EVAL-01");

        // Only 1 eligible bid
        assert_eq!(eval.ranked_bids.len(), 1);
        assert_eq!(eval.ranked_bids[0].vendor_id, "V001");
    }
}
