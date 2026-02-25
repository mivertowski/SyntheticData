//! Material generator for inventory and product master data.

use chrono::NaiveDate;
use datasynth_core::models::{
    BomComponent, Material, MaterialAccountDetermination, MaterialGroup, MaterialPool,
    MaterialType, UnitOfMeasure, ValuationMethod,
};
use datasynth_core::utils::seeded_rng;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use tracing::debug;

/// Configuration for material generation.
#[derive(Debug, Clone)]
pub struct MaterialGeneratorConfig {
    /// Distribution of material types (type, probability)
    pub material_type_distribution: Vec<(MaterialType, f64)>,
    /// Distribution of valuation methods (method, probability)
    pub valuation_method_distribution: Vec<(ValuationMethod, f64)>,
    /// Probability of material having BOM
    pub bom_rate: f64,
    /// Default base unit of measure
    pub default_uom: String,
    /// Gross margin range (min, max) as percentages
    pub gross_margin_range: (f64, f64),
    /// Standard cost range (min, max)
    pub standard_cost_range: (Decimal, Decimal),
}

impl Default for MaterialGeneratorConfig {
    fn default() -> Self {
        Self {
            material_type_distribution: vec![
                (MaterialType::FinishedGood, 0.30),
                (MaterialType::RawMaterial, 0.35),
                (MaterialType::SemiFinished, 0.15),
                (MaterialType::TradingGood, 0.10),
                (MaterialType::OperatingSupplies, 0.05),
                (MaterialType::Packaging, 0.05),
            ],
            valuation_method_distribution: vec![
                (ValuationMethod::StandardCost, 0.60),
                (ValuationMethod::MovingAverage, 0.30),
                (ValuationMethod::Fifo, 0.08),
                (ValuationMethod::Lifo, 0.02),
            ],
            bom_rate: 0.25,
            default_uom: "EA".to_string(),
            gross_margin_range: (0.20, 0.50),
            standard_cost_range: (Decimal::from(10), Decimal::from(10_000)),
        }
    }
}

/// Material description templates by type.
const MATERIAL_DESCRIPTIONS: &[(MaterialType, &[&str])] = &[
    (
        MaterialType::FinishedGood,
        &[
            "Assembled Unit A",
            "Complete Product B",
            "Final Assembly C",
            "Packaged Item D",
            "Ready Product E",
            "Finished Component F",
            "Complete Module G",
            "Final Product H",
        ],
    ),
    (
        MaterialType::RawMaterial,
        &[
            "Steel Plate Grade A",
            "Aluminum Sheet 6061",
            "Copper Wire AWG 12",
            "Plastic Resin ABS",
            "Raw Polymer Mix",
            "Chemical Compound X",
            "Base Material Y",
            "Raw Stock Z",
        ],
    ),
    (
        MaterialType::SemiFinished,
        &[
            "Sub-Assembly Part A",
            "Machined Component B",
            "Intermediate Product C",
            "Partial Assembly D",
            "Semi-Complete Unit E",
            "Work in Progress F",
            "Partially Processed G",
            "Intermediate Module H",
        ],
    ),
    (
        MaterialType::TradingGood,
        &[
            "Resale Item A",
            "Trading Good B",
            "Merchandise C",
            "Distribution Item D",
            "Wholesale Product E",
            "Retail Item F",
            "Trade Good G",
            "Commercial Product H",
        ],
    ),
    (
        MaterialType::OperatingSupplies,
        &[
            "Cleaning Supplies",
            "Office Supplies",
            "Maintenance Supplies",
            "Workshop Consumables",
            "Safety Supplies",
            "Facility Supplies",
            "General Supplies",
            "Operating Materials",
        ],
    ),
    (
        MaterialType::Packaging,
        &[
            "Cardboard Box Large",
            "Plastic Container",
            "Shipping Carton",
            "Protective Wrap",
            "Pallet Unit",
            "Foam Insert",
            "Label Roll",
            "Tape Industrial",
        ],
    ),
];

/// Generator for material master data.
pub struct MaterialGenerator {
    rng: ChaCha8Rng,
    seed: u64,
    config: MaterialGeneratorConfig,
    material_counter: usize,
    created_materials: Vec<String>, // Track for BOM references
    /// Optional country pack for locale-aware generation
    country_pack: Option<datasynth_core::CountryPack>,
}

impl MaterialGenerator {
    /// Create a new material generator.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, MaterialGeneratorConfig::default())
    }

    /// Create a new material generator with custom configuration.
    pub fn with_config(seed: u64, config: MaterialGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            seed,
            config,
            material_counter: 0,
            created_materials: Vec::new(),
            country_pack: None,
        }
    }

    /// Set the country pack for locale-aware generation.
    pub fn set_country_pack(&mut self, pack: datasynth_core::CountryPack) {
        self.country_pack = Some(pack);
    }

    /// Generate a single material.
    pub fn generate_material(
        &mut self,
        _company_code: &str,
        _effective_date: NaiveDate,
    ) -> Material {
        self.material_counter += 1;

        let material_id = format!("MAT-{:06}", self.material_counter);
        let material_type = self.select_material_type();
        let description = self.select_description(&material_type);

        let mut material =
            Material::new(material_id.clone(), description.to_string(), material_type);

        // Set material group
        material.material_group = self.select_material_group(&material_type);

        // Set valuation method
        material.valuation_method = self.select_valuation_method();

        // Set costs and prices
        let standard_cost = self.generate_standard_cost();
        material.standard_cost = standard_cost;
        material.purchase_price = standard_cost;
        material.list_price = self.generate_list_price(standard_cost);

        // Set unit of measure
        material.base_uom = if material_type == MaterialType::OperatingSupplies {
            UnitOfMeasure::hour()
        } else {
            UnitOfMeasure::each()
        };

        // Set account determination
        material.account_determination = self.generate_account_determination(&material_type);

        // Set stock and reorder info
        if material_type != MaterialType::OperatingSupplies {
            material.safety_stock = self.generate_safety_stock();
            material.reorder_point = material.safety_stock * Decimal::from(2);
        }

        // Add to created materials for BOM references
        self.created_materials.push(material_id);

        material
    }

    /// Generate a material with specific type.
    pub fn generate_material_of_type(
        &mut self,
        material_type: MaterialType,
        _company_code: &str,
        _effective_date: NaiveDate,
    ) -> Material {
        self.material_counter += 1;

        let material_id = format!("MAT-{:06}", self.material_counter);
        let description = self.select_description(&material_type);

        let mut material =
            Material::new(material_id.clone(), description.to_string(), material_type);

        material.material_group = self.select_material_group(&material_type);
        material.valuation_method = self.select_valuation_method();

        let standard_cost = self.generate_standard_cost();
        material.standard_cost = standard_cost;
        material.purchase_price = standard_cost;
        material.list_price = self.generate_list_price(standard_cost);

        material.base_uom = if material_type == MaterialType::OperatingSupplies {
            UnitOfMeasure::hour()
        } else {
            UnitOfMeasure::each()
        };

        material.account_determination = self.generate_account_determination(&material_type);

        if material_type != MaterialType::OperatingSupplies {
            material.safety_stock = self.generate_safety_stock();
            material.reorder_point = material.safety_stock * Decimal::from(2);
        }

        self.created_materials.push(material_id);

        material
    }

    /// Generate a material with BOM.
    pub fn generate_material_with_bom(
        &mut self,
        company_code: &str,
        effective_date: NaiveDate,
        component_count: usize,
    ) -> Material {
        // Generate component materials first
        let mut components = Vec::new();
        for i in 0..component_count {
            let component_type = if i % 2 == 0 {
                MaterialType::RawMaterial
            } else {
                MaterialType::SemiFinished
            };
            let component =
                self.generate_material_of_type(component_type, company_code, effective_date);

            let quantity = Decimal::from(self.rng.random_range(1..10));
            components.push(BomComponent {
                component_material_id: component.material_id.clone(),
                quantity,
                uom: component.base_uom.code.clone(),
                position: (i + 1) as u16 * 10,
                scrap_percentage: Decimal::ZERO,
                is_optional: false,
            });
        }

        // Generate the finished good with BOM
        let mut material = self.generate_material_of_type(
            MaterialType::FinishedGood,
            company_code,
            effective_date,
        );

        material.bom_components = Some(components);

        material
    }

    /// Generate a material pool with specified count.
    pub fn generate_material_pool(
        &mut self,
        count: usize,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> MaterialPool {
        debug!(count, company_code, %effective_date, "Generating material pool");
        let mut pool = MaterialPool::new();

        for _ in 0..count {
            let material = self.generate_material(company_code, effective_date);
            pool.add_material(material);
        }

        pool
    }

    /// Generate a material pool with BOMs.
    pub fn generate_material_pool_with_bom(
        &mut self,
        count: usize,
        bom_rate: f64,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> MaterialPool {
        let mut pool = MaterialPool::new();

        // First generate raw materials and semi-finished goods
        let raw_count = (count as f64 * 0.4) as usize;
        for _ in 0..raw_count {
            let material = self.generate_material_of_type(
                MaterialType::RawMaterial,
                company_code,
                effective_date,
            );
            pool.add_material(material);
        }

        let semi_count = (count as f64 * 0.2) as usize;
        for _ in 0..semi_count {
            let material = self.generate_material_of_type(
                MaterialType::SemiFinished,
                company_code,
                effective_date,
            );
            pool.add_material(material);
        }

        // Generate finished goods, some with BOM
        let finished_count = count - raw_count - semi_count;
        for _ in 0..finished_count {
            let material =
                if self.rng.random::<f64>() < bom_rate && !self.created_materials.is_empty() {
                    self.generate_material_with_bom_from_existing(company_code, effective_date)
                } else {
                    self.generate_material_of_type(
                        MaterialType::FinishedGood,
                        company_code,
                        effective_date,
                    )
                };
            pool.add_material(material);
        }

        pool
    }

    /// Generate a material with BOM using existing materials.
    fn generate_material_with_bom_from_existing(
        &mut self,
        company_code: &str,
        effective_date: NaiveDate,
    ) -> Material {
        let mut material = self.generate_material_of_type(
            MaterialType::FinishedGood,
            company_code,
            effective_date,
        );

        // Select some existing materials as components
        let component_count = self
            .rng
            .random_range(2..=5)
            .min(self.created_materials.len());
        let mut components = Vec::new();

        for i in 0..component_count {
            if let Some(component_material_id) = self.created_materials.get(i) {
                components.push(BomComponent {
                    component_material_id: component_material_id.clone(),
                    quantity: Decimal::from(self.rng.random_range(1..5)),
                    uom: "EA".to_string(),
                    position: (i + 1) as u16 * 10,
                    scrap_percentage: Decimal::ZERO,
                    is_optional: false,
                });
            }
        }

        if !components.is_empty() {
            material.bom_components = Some(components);
        }

        material
    }

    /// Select material type based on distribution.
    fn select_material_type(&mut self) -> MaterialType {
        let roll: f64 = self.rng.random();
        let mut cumulative = 0.0;

        for (mat_type, prob) in &self.config.material_type_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *mat_type;
            }
        }

        MaterialType::FinishedGood
    }

    /// Select valuation method based on distribution.
    fn select_valuation_method(&mut self) -> ValuationMethod {
        let roll: f64 = self.rng.random();
        let mut cumulative = 0.0;

        for (method, prob) in &self.config.valuation_method_distribution {
            cumulative += prob;
            if roll < cumulative {
                return *method;
            }
        }

        ValuationMethod::StandardCost
    }

    /// Select description for material type.
    fn select_description(&mut self, material_type: &MaterialType) -> &'static str {
        for (mat_type, descriptions) in MATERIAL_DESCRIPTIONS {
            if mat_type == material_type {
                let idx = self.rng.random_range(0..descriptions.len());
                return descriptions[idx];
            }
        }
        "Generic Material"
    }

    /// Select material group for type.
    fn select_material_group(&mut self, material_type: &MaterialType) -> MaterialGroup {
        match material_type {
            MaterialType::FinishedGood => {
                let options = [
                    MaterialGroup::Electronics,
                    MaterialGroup::Mechanical,
                    MaterialGroup::FinishedGoods,
                ];
                options[self.rng.random_range(0..options.len())]
            }
            MaterialType::RawMaterial => {
                let options = [
                    MaterialGroup::Chemicals,
                    MaterialGroup::Chemical,
                    MaterialGroup::Mechanical,
                ];
                options[self.rng.random_range(0..options.len())]
            }
            MaterialType::SemiFinished => {
                let options = [MaterialGroup::Electronics, MaterialGroup::Mechanical];
                options[self.rng.random_range(0..options.len())]
            }
            MaterialType::TradingGood => MaterialGroup::FinishedGoods,
            MaterialType::OperatingSupplies => MaterialGroup::Services,
            MaterialType::Packaging | MaterialType::SparePart => MaterialGroup::Consumables,
            _ => MaterialGroup::Consumables,
        }
    }

    /// Generate standard cost.
    fn generate_standard_cost(&mut self) -> Decimal {
        let min = self.config.standard_cost_range.0;
        let max = self.config.standard_cost_range.1;
        let range = (max - min).to_string().parse::<f64>().unwrap_or(0.0);
        let offset =
            Decimal::from_f64_retain(self.rng.random::<f64>() * range).unwrap_or(Decimal::ZERO);
        (min + offset).round_dp(2)
    }

    /// Generate list price from standard cost.
    fn generate_list_price(&mut self, standard_cost: Decimal) -> Decimal {
        let (min_margin, max_margin) = self.config.gross_margin_range;
        let margin = min_margin + self.rng.random::<f64>() * (max_margin - min_margin);
        let markup = Decimal::from_f64_retain(1.0 / (1.0 - margin)).unwrap_or(Decimal::from(2));
        (standard_cost * markup).round_dp(2)
    }

    /// Generate safety stock.
    fn generate_safety_stock(&mut self) -> Decimal {
        Decimal::from(self.rng.random_range(10..500))
    }

    /// Generate account determination.
    fn generate_account_determination(
        &mut self,
        material_type: &MaterialType,
    ) -> MaterialAccountDetermination {
        match material_type {
            MaterialType::FinishedGood | MaterialType::TradingGood => {
                MaterialAccountDetermination {
                    inventory_account: "140000".to_string(),
                    cogs_account: "500000".to_string(),
                    revenue_account: "400000".to_string(),
                    purchase_expense_account: "500000".to_string(),
                    price_difference_account: "590000".to_string(),
                    gr_ir_account: "290000".to_string(),
                }
            }
            MaterialType::RawMaterial | MaterialType::SemiFinished => {
                MaterialAccountDetermination {
                    inventory_account: "141000".to_string(),
                    cogs_account: "510000".to_string(),
                    revenue_account: "400000".to_string(),
                    purchase_expense_account: "510000".to_string(),
                    price_difference_account: "591000".to_string(),
                    gr_ir_account: "290000".to_string(),
                }
            }
            MaterialType::OperatingSupplies => MaterialAccountDetermination {
                inventory_account: "".to_string(),
                cogs_account: "520000".to_string(),
                revenue_account: "410000".to_string(),
                purchase_expense_account: "520000".to_string(),
                price_difference_account: "".to_string(),
                gr_ir_account: "290000".to_string(),
            },
            _ => MaterialAccountDetermination {
                inventory_account: "145000".to_string(),
                cogs_account: "530000".to_string(),
                revenue_account: "400000".to_string(),
                purchase_expense_account: "530000".to_string(),
                price_difference_account: "595000".to_string(),
                gr_ir_account: "290000".to_string(),
            },
        }
    }

    /// Reset the generator.
    pub fn reset(&mut self) {
        self.rng = seeded_rng(self.seed, 0);
        self.material_counter = 0;
        self.created_materials.clear();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_material_generation() {
        let mut gen = MaterialGenerator::new(42);
        let material = gen.generate_material("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert!(!material.material_id.is_empty());
        assert!(!material.description.is_empty());
        assert!(material.standard_cost > Decimal::ZERO);
        assert!(material.list_price >= material.standard_cost);
    }

    #[test]
    fn test_material_pool_generation() {
        let mut gen = MaterialGenerator::new(42);
        let pool =
            gen.generate_material_pool(50, "1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(pool.materials.len(), 50);

        // Should have various material types
        let raw_count = pool
            .materials
            .iter()
            .filter(|m| m.material_type == MaterialType::RawMaterial)
            .count();
        let finished_count = pool
            .materials
            .iter()
            .filter(|m| m.material_type == MaterialType::FinishedGood)
            .count();

        assert!(raw_count > 0);
        assert!(finished_count > 0);
    }

    #[test]
    fn test_material_with_bom() {
        let mut gen = MaterialGenerator::new(42);
        let material =
            gen.generate_material_with_bom("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 3);

        assert_eq!(material.material_type, MaterialType::FinishedGood);
        assert!(material.bom_components.is_some());
        assert_eq!(material.bom_components.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_material_pool_with_bom() {
        let mut gen = MaterialGenerator::new(42);
        let pool = gen.generate_material_pool_with_bom(
            100,
            0.5,
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        assert_eq!(pool.materials.len(), 100);

        // Should have some materials with BOMs
        let bom_count = pool
            .materials
            .iter()
            .filter(|m| m.bom_components.is_some())
            .count();

        assert!(bom_count > 0);
    }

    #[test]
    fn test_deterministic_generation() {
        let mut gen1 = MaterialGenerator::new(42);
        let mut gen2 = MaterialGenerator::new(42);

        let material1 =
            gen1.generate_material("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let material2 =
            gen2.generate_material("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        assert_eq!(material1.material_id, material2.material_id);
        assert_eq!(material1.description, material2.description);
        assert_eq!(material1.standard_cost, material2.standard_cost);
    }

    #[test]
    fn test_material_margin() {
        let mut gen = MaterialGenerator::new(42);

        for _ in 0..10 {
            let material =
                gen.generate_material("1000", NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

            // List price should be higher than standard cost (gross margin)
            assert!(
                material.list_price >= material.standard_cost,
                "List price {} should be >= standard cost {}",
                material.list_price,
                material.standard_cost
            );

            // Check margin is within configured range
            let margin = material.gross_margin_percent();
            assert!(
                margin >= Decimal::from(15) && margin <= Decimal::from(55),
                "Margin {} should be within expected range",
                margin
            );
        }
    }
}
