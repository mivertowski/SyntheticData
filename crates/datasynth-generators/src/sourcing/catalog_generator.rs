//! Catalog item generator.

use datasynth_config::schema::CatalogConfig;
use datasynth_core::models::sourcing::{CatalogItem, ProcurementContract};
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

/// Generates catalog items from active contracts.
pub struct CatalogGenerator {
    rng: ChaCha8Rng,
    uuid_factory: DeterministicUuidFactory,
    config: CatalogConfig,
}

impl CatalogGenerator {
    /// Create a new catalog generator.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::CatalogItem),
            config: CatalogConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: CatalogConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            uuid_factory: DeterministicUuidFactory::new(seed, GeneratorType::CatalogItem),
            config,
        }
    }

    /// Generate catalog items from a list of active contracts.
    pub fn generate(&mut self, contracts: &[ProcurementContract]) -> Vec<CatalogItem> {
        let mut items = Vec::new();

        for contract in contracts {
            for line in &contract.line_items {
                let is_preferred = self.rng.gen_bool(self.config.preferred_vendor_flag_rate);

                items.push(CatalogItem {
                    catalog_item_id: self.uuid_factory.next().to_string(),
                    contract_id: contract.contract_id.clone(),
                    contract_line_number: line.line_number,
                    vendor_id: contract.vendor_id.clone(),
                    material_id: line.material_id.clone(),
                    description: line.description.clone(),
                    catalog_price: line.unit_price,
                    uom: line.uom.clone(),
                    is_preferred,
                    category: contract.category_id.clone(),
                    min_order_quantity: line.min_quantity,
                    lead_time_days: Some(self.rng.gen_range(3..=30)),
                    is_active: true,
                });

                // Possibly add alternative source
                if self.rng.gen_bool(self.config.multi_source_rate) {
                    items.push(CatalogItem {
                        catalog_item_id: self.uuid_factory.next().to_string(),
                        contract_id: contract.contract_id.clone(),
                        contract_line_number: line.line_number,
                        vendor_id: format!("{}-ALT", contract.vendor_id),
                        material_id: line.material_id.clone(),
                        description: format!("{} (alternate)", line.description),
                        catalog_price: line.unit_price
                            * rust_decimal::Decimal::from_f64_retain(
                                self.rng.gen_range(0.95..=1.10),
                            )
                            .unwrap_or(rust_decimal::Decimal::ONE),
                        uom: line.uom.clone(),
                        is_preferred: false,
                        category: contract.category_id.clone(),
                        min_order_quantity: line.min_quantity,
                        lead_time_days: Some(self.rng.gen_range(5..=45)),
                        is_active: true,
                    });
                }
            }
        }

        items
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
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            total_value: Decimal::from(100_000),
            consumed_value: Decimal::ZERO,
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
            line_items: vec![
                ContractLineItem {
                    line_number: 1,
                    material_id: None,
                    description: "Widget A".to_string(),
                    unit_price: Decimal::from(50),
                    uom: "EA".to_string(),
                    min_quantity: Some(Decimal::from(100)),
                    max_quantity: Some(Decimal::from(1200)),
                    quantity_released: Decimal::ZERO,
                    value_released: Decimal::ZERO,
                },
                ContractLineItem {
                    line_number: 2,
                    material_id: Some("MAT-002".to_string()),
                    description: "Widget B".to_string(),
                    unit_price: Decimal::from(25),
                    uom: "EA".to_string(),
                    min_quantity: Some(Decimal::from(200)),
                    max_quantity: Some(Decimal::from(2400)),
                    quantity_released: Decimal::ZERO,
                    value_released: Decimal::ZERO,
                },
            ],
            category_id: "CAT-001".to_string(),
            owner_id: "BUYER-01".to_string(),
            amendment_count: 0,
            previous_contract_id: None,
        }
    }

    #[test]
    fn test_basic_generation() {
        let mut gen = CatalogGenerator::new(42);
        let contracts = vec![test_contract()];
        let items = gen.generate(&contracts);

        // Should have at least 2 items (one per contract line)
        assert!(items.len() >= 2);
        for item in &items {
            assert!(!item.catalog_item_id.is_empty());
            assert_eq!(item.contract_id, "CTR-001");
            assert!(!item.vendor_id.is_empty());
            assert!(item.catalog_price > Decimal::ZERO);
            assert!(item.is_active);
            assert_eq!(item.category, "CAT-001");
        }
    }

    #[test]
    fn test_deterministic() {
        let contracts = vec![test_contract()];

        let mut gen1 = CatalogGenerator::new(42);
        let mut gen2 = CatalogGenerator::new(42);

        let r1 = gen1.generate(&contracts);
        let r2 = gen2.generate(&contracts);

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.catalog_item_id, b.catalog_item_id);
            assert_eq!(a.vendor_id, b.vendor_id);
            assert_eq!(a.catalog_price, b.catalog_price);
            assert_eq!(a.is_preferred, b.is_preferred);
        }
    }

    #[test]
    fn test_field_constraints() {
        let mut gen = CatalogGenerator::new(99);
        let contracts = vec![test_contract()];
        let items = gen.generate(&contracts);

        for item in &items {
            // Lead time should be within expected range
            assert!(item.lead_time_days.is_some());
            let lt = item.lead_time_days.unwrap();
            assert!(lt >= 3 && lt <= 45);

            // UOM should be set
            assert!(!item.uom.is_empty());

            // Contract line number should be valid
            assert!(item.contract_line_number >= 1);
        }

        // Check alternate sources have "-ALT" suffix on vendor_id
        let alt_items: Vec<_> = items
            .iter()
            .filter(|i| i.vendor_id.contains("-ALT"))
            .collect();
        for alt in &alt_items {
            assert!(!alt.is_preferred); // Alternates should not be preferred
            assert!(alt.description.contains("(alternate)"));
        }
    }
}
