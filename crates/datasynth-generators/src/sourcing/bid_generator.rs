//! Supplier bid generator.

use chrono::NaiveDate;
use datasynth_core::models::sourcing::{BidLineItem, BidStatus, RfxEvent, SupplierBid};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Generates supplier bids in response to RFx events.
pub struct BidGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
}

impl BidGenerator {
    /// Create a new bid generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::SupplierBid),
        }
    }

    /// Generate bids for an RFx event from responding vendors.
    pub fn generate(
        &mut self,
        rfx: &RfxEvent,
        responding_vendor_ids: &[String],
        submission_date: NaiveDate,
    ) -> Vec<SupplierBid> {
        let mut bids = Vec::new();

        for vendor_id in responding_vendor_ids {
            let line_items: Vec<BidLineItem> = rfx
                .line_items
                .iter()
                .map(|rfx_item| {
                    // Vendor offers price within ±30% of target
                    let target = rfx_item.target_price.unwrap_or(Decimal::from(100));
                    let target_f64: f64 = target.to_string().parse().unwrap_or(100.0);
                    let price_factor = self.rng.gen_range(0.70..=1.30);
                    let unit_price =
                        Decimal::from_f64_retain(target_f64 * price_factor).unwrap_or(target);
                    let quantity = rfx_item.quantity;
                    let total = unit_price * quantity;

                    BidLineItem {
                        item_number: rfx_item.item_number,
                        unit_price,
                        quantity,
                        total_amount: total,
                        lead_time_days: self.rng.gen_range(5..=60),
                        notes: None,
                    }
                })
                .collect();

            let total_amount: Decimal = line_items.iter().map(|i| i.total_amount).sum();

            let is_on_time = self.rng.gen_bool(0.92);
            let is_compliant = self.rng.gen_bool(0.88);

            bids.push(SupplierBid {
                bid_id: self.uuid_factory.next().to_string(),
                rfx_id: rfx.rfx_id.clone(),
                vendor_id: vendor_id.clone(),
                company_code: rfx.company_code.clone(),
                status: BidStatus::Submitted,
                submission_date,
                line_items,
                total_amount,
                validity_days: self.rng.gen_range(30..=90),
                payment_terms: ["NET30", "NET45", "NET60", "2/10 NET30"][self.rng.gen_range(0..4)]
                    .to_string(),
                delivery_terms: Some("FCA".to_string()),
                technical_summary: None,
                is_on_time,
                is_compliant,
            });
        }

        bids
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::sourcing::{
        RfxEvaluationCriterion, RfxLineItem, RfxStatus, RfxType, ScoringMethod,
    };

    fn test_rfx() -> RfxEvent {
        RfxEvent {
            rfx_id: "RFX-001".to_string(),
            rfx_type: RfxType::Rfp,
            company_code: "C001".to_string(),
            title: "Test RFx".to_string(),
            description: "Test description".to_string(),
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
            line_items: vec![
                RfxLineItem {
                    item_number: 1,
                    description: "Item A".to_string(),
                    material_id: None,
                    quantity: Decimal::from(100),
                    uom: "EA".to_string(),
                    target_price: Some(Decimal::from(50)),
                },
                RfxLineItem {
                    item_number: 2,
                    description: "Item B".to_string(),
                    material_id: None,
                    quantity: Decimal::from(200),
                    uom: "EA".to_string(),
                    target_price: Some(Decimal::from(25)),
                },
            ],
            invited_vendors: vec!["V001".to_string(), "V002".to_string(), "V003".to_string()],
            publish_date: NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            response_deadline: NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
            bid_count: 3,
            owner_id: "BUYER-01".to_string(),
            awarded_vendor_id: None,
            awarded_bid_id: None,
        }
    }

    fn test_responding_vendors() -> Vec<String> {
        vec!["V001".to_string(), "V002".to_string(), "V003".to_string()]
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = BidGenerator::new(42);
        let rfx = test_rfx();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let bids = gen.generate(&rfx, &test_responding_vendors(), date);

        assert_eq!(bids.len(), 3);
        for bid in &bids {
            assert!(!bid.bid_id.is_empty());
            assert_eq!(bid.rfx_id, "RFX-001");
            assert_eq!(bid.company_code, "C001");
            assert_eq!(bid.submission_date, date);
            assert_eq!(bid.line_items.len(), 2);
            assert!(bid.total_amount > Decimal::ZERO);
            assert_eq!(bid.status, BidStatus::Submitted);
        }
    }

    #[test]
    fn test_deterministic() {
        let rfx = test_rfx();
        let vendors = test_responding_vendors();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        let mut gen1 = BidGenerator::new(42);
        let mut gen2 = BidGenerator::new(42);

        let r1 = gen1.generate(&rfx, &vendors, date);
        let r2 = gen2.generate(&rfx, &vendors, date);

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.bid_id, b.bid_id);
            assert_eq!(a.vendor_id, b.vendor_id);
            assert_eq!(a.total_amount, b.total_amount);
            assert_eq!(a.payment_terms, b.payment_terms);
        }
    }

    #[test]
    fn test_field_constraints() {
        let mut gen = BidGenerator::new(99);
        let rfx = test_rfx();
        let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let bids = gen.generate(&rfx, &test_responding_vendors(), date);

        for bid in &bids {
            // Validity days should be in range
            assert!(bid.validity_days >= 30 && bid.validity_days <= 90);

            // Payment terms should be one of the valid options
            assert!(["NET30", "NET45", "NET60", "2/10 NET30"].contains(&bid.payment_terms.as_str()));

            // Line items should match RFx line items
            for line in &bid.line_items {
                assert!(line.unit_price > Decimal::ZERO);
                assert!(line.total_amount > Decimal::ZERO);
                assert!(line.lead_time_days >= 5 && line.lead_time_days <= 60);
            }

            // Total amount should equal sum of line totals
            let line_sum: Decimal = bid.line_items.iter().map(|l| l.total_amount).sum();
            assert_eq!(bid.total_amount, line_sum);
        }
    }
}
