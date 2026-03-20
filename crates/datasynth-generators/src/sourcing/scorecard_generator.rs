//! Supplier scorecard generator.

use chrono::NaiveDate;
use datasynth_config::schema::ScorecardConfig;
use datasynth_core::models::sourcing::{
    ContractComplianceMetrics, ProcurementContract, ScoreboardTrend, ScorecardRecommendation,
    SupplierScorecard,
};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Generates supplier scorecards from P2P performance data.
pub struct ScorecardGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: ScorecardConfig,
}

impl ScorecardGenerator {
    /// Create a new scorecard generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::SupplierScorecard),
            config: ScorecardConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: ScorecardConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::SupplierScorecard),
            config,
        }
    }

    /// Generate scorecards for vendors with active contracts.
    pub fn generate(
        &mut self,
        company_code: &str,
        vendor_contracts: &[(String, Vec<&ProcurementContract>)],
        period_start: NaiveDate,
        period_end: NaiveDate,
        reviewer_id: &str,
    ) -> Vec<SupplierScorecard> {
        let mut scorecards = Vec::new();

        for (vendor_id, contracts) in vendor_contracts {
            let otd_rate = self.rng.random_range(0.75..=0.99);
            let quality_rate = self.rng.random_range(0.85..=0.99);
            let price_score = self.rng.random_range(60.0..=100.0);
            let responsiveness = self.rng.random_range(50.0..=100.0);

            let overall = otd_rate * 100.0 * self.config.on_time_delivery_weight
                + quality_rate * 100.0 * self.config.quality_weight
                + price_score * self.config.price_weight
                + responsiveness * self.config.responsiveness_weight;

            let grade = if overall >= self.config.grade_a_threshold {
                "A"
            } else if overall >= self.config.grade_b_threshold {
                "B"
            } else if overall >= self.config.grade_c_threshold {
                "C"
            } else {
                "D"
            };

            let trend = if self.rng.random_bool(0.5) {
                ScoreboardTrend::Stable
            } else if self.rng.random_bool(0.6) {
                ScoreboardTrend::Improving
            } else {
                ScoreboardTrend::Declining
            };

            let recommendation = match grade {
                "A" => ScorecardRecommendation::Expand,
                "B" => ScorecardRecommendation::Maintain,
                "C" => ScorecardRecommendation::Probation,
                _ => ScorecardRecommendation::Replace,
            };

            let contract_compliance: Vec<ContractComplianceMetrics> = contracts
                .iter()
                .map(|c| {
                    let consumed_f64: f64 = c.consumed_value.to_string().parse().unwrap_or(0.0);
                    let total_f64: f64 = c.total_value.to_string().parse().unwrap_or(1.0);
                    let utilization = if total_f64 > 0.0 {
                        consumed_f64 / total_f64
                    } else {
                        0.0
                    };

                    ContractComplianceMetrics {
                        contract_id: c.contract_id.clone(),
                        utilization_pct: utilization,
                        sla_breach_count: self.rng.random_range(0..=3),
                        price_compliance_pct: self.rng.random_range(0.85..=1.0),
                        amendment_count: c.amendment_count,
                    }
                })
                .collect();

            scorecards.push(SupplierScorecard {
                scorecard_id: self.uuid_factory.next().to_string(),
                vendor_id: vendor_id.clone(),
                company_code: company_code.to_string(),
                period_start,
                period_end,
                on_time_delivery_rate: otd_rate,
                quality_rate,
                price_score,
                responsiveness_score: responsiveness,
                overall_score: overall,
                grade: grade.to_string(),
                trend,
                contract_compliance,
                recommendation,
                reviewer_id: reviewer_id.to_string(),
                comments: None,
            });
        }

        scorecards
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::sourcing::{
        ContractLineItem, ContractSla, ContractStatus, ContractTerms, ContractType,
    };
    use rust_decimal::Decimal;

    fn test_contract() -> ProcurementContract {
        ProcurementContract {
            contract_id: "CTR-001".to_string(),
            company_code: "C001".to_string(),
            contract_type: ContractType::FixedPrice,
            status: ContractStatus::Active,
            vendor_id: "V001".to_string(),
            title: "Test Contract".to_string(),
            sourcing_project_id: None,
            bid_id: None,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            total_value: Decimal::from(100_000),
            consumed_value: Decimal::from(30_000),
            terms: ContractTerms {
                payment_terms: "NET30".to_string(),
                delivery_terms: Some("FCA".to_string()),
                warranty_months: None,
                early_termination_penalty_pct: None,
                auto_renewal: false,
                termination_notice_days: 60,
                price_adjustment_clause: false,
                max_annual_price_increase_pct: None,
            },
            slas: vec![ContractSla {
                metric_name: "on_time_delivery".to_string(),
                target_value: 0.95,
                minimum_value: 0.90,
                breach_penalty_pct: 0.02,
                measurement_frequency: "monthly".to_string(),
            }],
            line_items: vec![ContractLineItem {
                line_number: 1,
                material_id: None,
                description: "Widget A".to_string(),
                unit_price: Decimal::from(50),
                uom: "EA".to_string(),
                min_quantity: Some(Decimal::from(100)),
                max_quantity: Some(Decimal::from(1200)),
                quantity_released: Decimal::ZERO,
                value_released: Decimal::ZERO,
            }],
            category_id: "CAT-001".to_string(),
            owner_id: "BUYER-01".to_string(),
            amendment_count: 1,
            previous_contract_id: None,
            purchase_order_ids: Vec::new(),
        }
    }

    fn test_vendor_contracts() -> Vec<(String, Vec<ProcurementContract>)> {
        vec![
            ("V001".to_string(), vec![test_contract()]),
            ("V002".to_string(), vec![test_contract()]),
        ]
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = ScorecardGenerator::new(42);
        let vc = test_vendor_contracts();
        let vendor_refs: Vec<(String, Vec<&ProcurementContract>)> = vc
            .iter()
            .map(|(v, cs)| (v.clone(), cs.iter().collect()))
            .collect();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let scorecards = gen.generate("C001", &vendor_refs, start, end, "REV-01");

        assert_eq!(scorecards.len(), 2);
        for sc in &scorecards {
            assert!(!sc.scorecard_id.is_empty());
            assert_eq!(sc.company_code, "C001");
            assert!(!sc.vendor_id.is_empty());
            assert_eq!(sc.period_start, start);
            assert_eq!(sc.period_end, end);
            assert_eq!(sc.reviewer_id, "REV-01");
            assert!(sc.overall_score > 0.0);
            assert!(!sc.grade.is_empty());
            assert!(!sc.contract_compliance.is_empty());
        }
    }

    #[test]
    fn test_deterministic() {
        let vc = test_vendor_contracts();
        let vendor_refs: Vec<(String, Vec<&ProcurementContract>)> = vc
            .iter()
            .map(|(v, cs)| (v.clone(), cs.iter().collect()))
            .collect();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let mut gen1 = ScorecardGenerator::new(42);
        let mut gen2 = ScorecardGenerator::new(42);

        let r1 = gen1.generate("C001", &vendor_refs, start, end, "REV-01");
        let r2 = gen2.generate("C001", &vendor_refs, start, end, "REV-01");

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.scorecard_id, b.scorecard_id);
            assert_eq!(a.vendor_id, b.vendor_id);
            assert_eq!(a.overall_score, b.overall_score);
            assert_eq!(a.grade, b.grade);
            assert_eq!(a.on_time_delivery_rate, b.on_time_delivery_rate);
        }
    }

    #[test]
    fn test_field_constraints() {
        let mut gen = ScorecardGenerator::new(99);
        let vc = test_vendor_contracts();
        let vendor_refs: Vec<(String, Vec<&ProcurementContract>)> = vc
            .iter()
            .map(|(v, cs)| (v.clone(), cs.iter().collect()))
            .collect();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

        let scorecards = gen.generate("C001", &vendor_refs, start, end, "REV-01");

        for sc in &scorecards {
            // Rates should be in valid range
            assert!(sc.on_time_delivery_rate >= 0.75 && sc.on_time_delivery_rate <= 0.99);
            assert!(sc.quality_rate >= 0.85 && sc.quality_rate <= 0.99);
            assert!(sc.price_score >= 60.0 && sc.price_score <= 100.0);
            assert!(sc.responsiveness_score >= 50.0 && sc.responsiveness_score <= 100.0);

            // Grade should be valid
            assert!(["A", "B", "C", "D"].contains(&sc.grade.as_str()));

            // Recommendation should match grade
            match sc.grade.as_str() {
                "A" => assert!(matches!(sc.recommendation, ScorecardRecommendation::Expand)),
                "B" => assert!(matches!(
                    sc.recommendation,
                    ScorecardRecommendation::Maintain
                )),
                "C" => assert!(matches!(
                    sc.recommendation,
                    ScorecardRecommendation::Probation
                )),
                "D" => assert!(matches!(
                    sc.recommendation,
                    ScorecardRecommendation::Replace
                )),
                _ => panic!("Unexpected grade"),
            }

            // Contract compliance metrics
            for cc in &sc.contract_compliance {
                assert!(!cc.contract_id.is_empty());
                assert!(cc.price_compliance_pct >= 0.85 && cc.price_compliance_pct <= 1.0);
                assert!(cc.utilization_pct >= 0.0);
            }
        }
    }
}
