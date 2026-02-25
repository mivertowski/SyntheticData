//! RFx event generator.

use chrono::NaiveDate;
use datasynth_config::schema::RfxConfig;
use datasynth_core::models::sourcing::{
    RfxEvaluationCriterion, RfxEvent, RfxLineItem, RfxStatus, RfxType, ScoringMethod,
};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Generates RFx events (RFI/RFP/RFQ).
pub struct RfxGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: RfxConfig,
}

impl RfxGenerator {
    /// Create a new RFx generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::RfxEvent),
            config: RfxConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: RfxConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::RfxEvent),
            config,
        }
    }

    /// Generate an RFx event for a sourcing project.
    pub fn generate(
        &mut self,
        company_code: &str,
        sourcing_project_id: &str,
        category_id: &str,
        qualified_vendor_ids: &[String],
        owner_id: &str,
        publish_date: NaiveDate,
        estimated_spend: f64,
    ) -> RfxEvent {
        let rfx_type = if estimated_spend > self.config.rfi_threshold {
            if self.rng.random_bool(0.3) {
                RfxType::Rfi
            } else {
                RfxType::Rfp
            }
        } else {
            RfxType::Rfq
        };

        let invited_count = self
            .rng
            .random_range(self.config.min_invited_vendors..=self.config.max_invited_vendors)
            .min(qualified_vendor_ids.len() as u32) as usize;

        let invited_vendors: Vec<String> = qualified_vendor_ids
            .choose_multiple(&mut self.rng, invited_count)
            .cloned()
            .collect();

        let response_deadline =
            publish_date + chrono::Duration::days(self.rng.random_range(14..=45));

        let bid_count = (invited_vendors.len() as f64 * self.config.response_rate).round() as u32;

        let criteria = vec![
            RfxEvaluationCriterion {
                name: "Price".to_string(),
                weight: self.config.default_price_weight,
                description: "Total cost of ownership".to_string(),
            },
            RfxEvaluationCriterion {
                name: "Quality".to_string(),
                weight: self.config.default_quality_weight,
                description: "Quality management and track record".to_string(),
            },
            RfxEvaluationCriterion {
                name: "Delivery".to_string(),
                weight: self.config.default_delivery_weight,
                description: "Lead time and reliability".to_string(),
            },
        ];

        let line_count = self.rng.random_range(1u16..=5);
        let line_items: Vec<RfxLineItem> = (1..=line_count)
            .map(|i| RfxLineItem {
                item_number: i,
                description: format!("Item {}", i),
                material_id: None,
                quantity: Decimal::from(self.rng.random_range(10..=1000)),
                uom: "EA".to_string(),
                target_price: Some(Decimal::from(self.rng.random_range(10..=5000))),
            })
            .collect();

        let scoring_method = match rfx_type {
            RfxType::Rfq => ScoringMethod::LowestPrice,
            RfxType::Rfp => ScoringMethod::BestValue,
            RfxType::Rfi => ScoringMethod::QualityBased,
        };

        RfxEvent {
            rfx_id: self.uuid_factory.next().to_string(),
            rfx_type,
            company_code: company_code.to_string(),
            title: format!("RFx for {}", category_id),
            description: format!(
                "Sourcing event for category {} under project {}",
                category_id, sourcing_project_id
            ),
            status: RfxStatus::Awarded,
            sourcing_project_id: sourcing_project_id.to_string(),
            category_id: category_id.to_string(),
            scoring_method,
            criteria,
            line_items,
            invited_vendors,
            publish_date,
            response_deadline,
            bid_count,
            owner_id: owner_id.to_string(),
            awarded_vendor_id: None,
            awarded_bid_id: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_vendor_ids() -> Vec<String> {
        (1..=6).map(|i| format!("V{:04}", i)).collect()
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = RfxGenerator::new(42);
        let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let rfx = gen.generate(
            "C001",
            "SP-001",
            "CAT-001",
            &test_vendor_ids(),
            "BUYER-01",
            date,
            200_000.0,
        );

        assert!(!rfx.rfx_id.is_empty());
        assert_eq!(rfx.company_code, "C001");
        assert_eq!(rfx.sourcing_project_id, "SP-001");
        assert_eq!(rfx.category_id, "CAT-001");
        assert_eq!(rfx.owner_id, "BUYER-01");
        assert_eq!(rfx.status, RfxStatus::Awarded);
        assert!(!rfx.invited_vendors.is_empty());
        assert!(!rfx.criteria.is_empty());
        assert!(!rfx.line_items.is_empty());
        assert!(rfx.response_deadline > date);
    }

    #[test]
    fn test_deterministic() {
        let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let vendors = test_vendor_ids();

        let mut gen1 = RfxGenerator::new(42);
        let mut gen2 = RfxGenerator::new(42);

        let r1 = gen1.generate(
            "C001", "SP-001", "CAT-001", &vendors, "BUYER-01", date, 200_000.0,
        );
        let r2 = gen2.generate(
            "C001", "SP-001", "CAT-001", &vendors, "BUYER-01", date, 200_000.0,
        );

        assert_eq!(r1.rfx_id, r2.rfx_id);
        assert_eq!(r1.rfx_type, r2.rfx_type);
        assert_eq!(r1.invited_vendors, r2.invited_vendors);
        assert_eq!(r1.line_items.len(), r2.line_items.len());
        assert_eq!(r1.bid_count, r2.bid_count);
    }

    #[test]
    fn test_field_constraints() {
        let mut gen = RfxGenerator::new(99);
        let date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();

        // High spend should produce RFI or RFP
        let rfx_high = gen.generate(
            "C001",
            "SP-001",
            "CAT-001",
            &test_vendor_ids(),
            "BUYER-01",
            date,
            500_000.0,
        );
        assert!(matches!(rfx_high.rfx_type, RfxType::Rfi | RfxType::Rfp));

        // Low spend should produce RFQ
        let rfx_low = gen.generate(
            "C001",
            "SP-002",
            "CAT-002",
            &test_vendor_ids(),
            "BUYER-01",
            date,
            50_000.0,
        );
        assert_eq!(rfx_low.rfx_type, RfxType::Rfq);

        // Criteria weights should exist
        assert_eq!(rfx_high.criteria.len(), 3);
        let weight_sum: f64 = rfx_high.criteria.iter().map(|c| c.weight).sum();
        assert!((weight_sum - 1.0).abs() < 0.01);

        // Line items should have valid item numbers
        for (i, item) in rfx_high.line_items.iter().enumerate() {
            assert_eq!(item.item_number, (i + 1) as u16);
            assert!(item.quantity > Decimal::ZERO);
        }
    }
}
