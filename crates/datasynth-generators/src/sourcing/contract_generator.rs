//! Procurement contract generator.

use chrono::NaiveDate;
use datasynth_config::schema::ContractConfig;
use datasynth_core::models::sourcing::{
    ContractLineItem, ContractSla, ContractStatus, ContractTerms, ContractType,
    ProcurementContract, SupplierBid,
};
use datasynth_core::utils::seeded_rng;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;

/// Generates procurement contracts from awarded bids.
pub struct ContractGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: ContractConfig,
}

impl ContractGenerator {
    /// Create a new contract generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ProcurementContract),
            config: ContractConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: ContractConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::ProcurementContract),
            config,
        }
    }

    /// Generate a contract from a winning bid.
    pub fn generate_from_bid(
        &mut self,
        winning_bid: &SupplierBid,
        sourcing_project_id: Option<&str>,
        category_id: &str,
        owner_id: &str,
        start_date: NaiveDate,
    ) -> ProcurementContract {
        let duration_months = self
            .rng
            .random_range(self.config.min_duration_months..=self.config.max_duration_months);
        let end_date = start_date + chrono::Duration::days((duration_months * 30) as i64);

        let contract_type = self.select_contract_type();

        let auto_renewal = self.rng.random_bool(self.config.auto_renewal_rate);

        let terms = ContractTerms {
            payment_terms: winning_bid.payment_terms.clone(),
            delivery_terms: winning_bid.delivery_terms.clone(),
            warranty_months: if self.rng.random_bool(0.5) {
                Some(self.rng.random_range(6..=24))
            } else {
                None
            },
            early_termination_penalty_pct: Some(self.rng.random_range(0.02..=0.10)),
            auto_renewal,
            termination_notice_days: self.rng.random_range(30..=120),
            price_adjustment_clause: self.rng.random_bool(0.3),
            max_annual_price_increase_pct: if self.rng.random_bool(0.4) {
                Some(self.rng.random_range(0.02..=0.05))
            } else {
                None
            },
        };

        let slas = vec![
            ContractSla {
                metric_name: "on_time_delivery".to_string(),
                target_value: 0.95,
                minimum_value: 0.90,
                breach_penalty_pct: 0.02,
                measurement_frequency: "monthly".to_string(),
            },
            ContractSla {
                metric_name: "defect_rate".to_string(),
                target_value: 0.02,
                minimum_value: 0.05,
                breach_penalty_pct: 0.03,
                measurement_frequency: "quarterly".to_string(),
            },
        ];

        let line_items: Vec<ContractLineItem> = winning_bid
            .line_items
            .iter()
            .map(|bl| {
                let annual_qty = bl.quantity * Decimal::from(12);
                ContractLineItem {
                    line_number: bl.item_number,
                    material_id: None,
                    description: format!("Contract item {}", bl.item_number),
                    unit_price: bl.unit_price,
                    uom: "EA".to_string(),
                    min_quantity: Some(bl.quantity),
                    max_quantity: Some(annual_qty),
                    quantity_released: Decimal::ZERO,
                    value_released: Decimal::ZERO,
                }
            })
            .collect();

        // Total value = sum of max quantities * prices
        let total_value: Decimal = line_items
            .iter()
            .map(|li| li.max_quantity.unwrap_or(Decimal::ZERO) * li.unit_price)
            .sum();

        ProcurementContract {
            contract_id: self.uuid_factory.next().to_string(),
            company_code: winning_bid.company_code.clone(),
            contract_type,
            status: ContractStatus::Active,
            vendor_id: winning_bid.vendor_id.clone(),
            title: format!(
                "Contract with {} for {}",
                winning_bid.vendor_id, category_id
            ),
            sourcing_project_id: sourcing_project_id.map(std::string::ToString::to_string),
            bid_id: Some(winning_bid.bid_id.clone()),
            start_date,
            end_date,
            total_value,
            consumed_value: Decimal::ZERO,
            terms,
            slas,
            line_items,
            category_id: category_id.to_string(),
            owner_id: owner_id.to_string(),
            amendment_count: if self.rng.random_bool(self.config.amendment_rate) {
                self.rng.random_range(1..=3)
            } else {
                0
            },
            previous_contract_id: None,
            purchase_order_ids: Vec::new(),
        }
    }

    fn select_contract_type(&mut self) -> ContractType {
        let dist = &self.config.type_distribution;
        let r: f64 = self.rng.random();
        if r < dist.fixed_price {
            ContractType::FixedPrice
        } else if r < dist.fixed_price + dist.blanket {
            ContractType::Blanket
        } else if r < dist.fixed_price + dist.blanket + dist.time_and_materials {
            ContractType::TimeAndMaterials
        } else {
            ContractType::ServiceAgreement
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use datasynth_core::models::sourcing::{BidLineItem, BidStatus};

    fn test_winning_bid() -> SupplierBid {
        SupplierBid {
            bid_id: "BID-001".to_string(),
            rfx_id: "RFX-001".to_string(),
            vendor_id: "V001".to_string(),
            company_code: "C001".to_string(),
            status: BidStatus::Submitted,
            submission_date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            line_items: vec![
                BidLineItem {
                    item_number: 1,
                    unit_price: Decimal::from(50),
                    quantity: Decimal::from(100),
                    total_amount: Decimal::from(5000),
                    lead_time_days: 10,
                    notes: None,
                },
                BidLineItem {
                    item_number: 2,
                    unit_price: Decimal::from(25),
                    quantity: Decimal::from(200),
                    total_amount: Decimal::from(5000),
                    lead_time_days: 15,
                    notes: None,
                },
            ],
            total_amount: Decimal::from(10000),
            validity_days: 60,
            payment_terms: "NET30".to_string(),
            delivery_terms: Some("FCA".to_string()),
            technical_summary: None,
            is_on_time: true,
            is_compliant: true,
        }
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = ContractGenerator::new(42);
        let bid = test_winning_bid();
        let start = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let contract = gen.generate_from_bid(&bid, Some("SP-001"), "CAT-001", "BUYER-01", start);

        assert!(!contract.contract_id.is_empty());
        assert_eq!(contract.company_code, "C001");
        assert_eq!(contract.vendor_id, "V001");
        assert_eq!(contract.status, ContractStatus::Active);
        assert_eq!(contract.bid_id.as_deref(), Some("BID-001"));
        assert_eq!(contract.sourcing_project_id.as_deref(), Some("SP-001"));
        assert_eq!(contract.category_id, "CAT-001");
        assert_eq!(contract.owner_id, "BUYER-01");
        assert_eq!(contract.start_date, start);
        assert!(contract.end_date > start);
        assert!(contract.total_value > Decimal::ZERO);
        assert_eq!(contract.consumed_value, Decimal::ZERO);
        assert_eq!(contract.line_items.len(), 2);
        assert_eq!(contract.slas.len(), 2);
    }

    #[test]
    fn test_deterministic() {
        let bid = test_winning_bid();
        let start = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();

        let mut gen1 = ContractGenerator::new(42);
        let mut gen2 = ContractGenerator::new(42);

        let r1 = gen1.generate_from_bid(&bid, Some("SP-001"), "CAT-001", "BUYER-01", start);
        let r2 = gen2.generate_from_bid(&bid, Some("SP-001"), "CAT-001", "BUYER-01", start);

        assert_eq!(r1.contract_id, r2.contract_id);
        assert_eq!(r1.contract_type, r2.contract_type);
        assert_eq!(r1.end_date, r2.end_date);
        assert_eq!(r1.total_value, r2.total_value);
        assert_eq!(r1.terms.payment_terms, r2.terms.payment_terms);
        assert_eq!(r1.terms.auto_renewal, r2.terms.auto_renewal);
        assert_eq!(r1.amendment_count, r2.amendment_count);
    }

    #[test]
    fn test_field_constraints() {
        let mut gen = ContractGenerator::new(99);
        let bid = test_winning_bid();
        let start = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let contract = gen.generate_from_bid(&bid, None, "CAT-001", "BUYER-01", start);

        // Duration should be within configured range (12-36 months default)
        let duration_days = (contract.end_date - contract.start_date).num_days();
        assert!((12 * 30..=36 * 30).contains(&duration_days));

        // Terms constraints
        assert!(contract.terms.termination_notice_days >= 30);
        assert!(contract.terms.termination_notice_days <= 120);
        assert_eq!(contract.terms.payment_terms, "NET30"); // From winning bid

        // SLA metrics
        for sla in &contract.slas {
            assert!(!sla.metric_name.is_empty());
            assert!(!sla.measurement_frequency.is_empty());
        }

        // Contract line items should match bid line items
        assert_eq!(contract.line_items.len(), 2);
        for line in &contract.line_items {
            assert!(line.unit_price > Decimal::ZERO);
            assert_eq!(line.quantity_released, Decimal::ZERO);
            assert_eq!(line.value_released, Decimal::ZERO);
        }

        // No sourcing project when None passed
        assert!(contract.sourcing_project_id.is_none());
    }
}
