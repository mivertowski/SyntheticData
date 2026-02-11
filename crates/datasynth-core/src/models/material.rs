//! Material master data model.
//!
//! Provides material/product master data for realistic inventory
//! and procurement transaction generation.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of material in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MaterialType {
    /// Raw materials used in production
    #[default]
    RawMaterial,
    /// Semi-finished goods
    SemiFinished,
    /// Finished goods for sale
    FinishedGood,
    /// Trading goods (resale without transformation)
    TradingGood,
    /// Operating supplies (consumables)
    OperatingSupplies,
    /// Spare parts
    SparePart,
    /// Packaging material
    Packaging,
    /// Service (non-physical)
    Service,
}

impl MaterialType {
    /// Get the typical account category for this material type.
    pub fn inventory_account_category(&self) -> &'static str {
        match self {
            Self::RawMaterial => "Raw Materials Inventory",
            Self::SemiFinished => "Work in Progress",
            Self::FinishedGood => "Finished Goods Inventory",
            Self::TradingGood => "Trading Goods Inventory",
            Self::OperatingSupplies => "Supplies Inventory",
            Self::SparePart => "Spare Parts Inventory",
            Self::Packaging => "Packaging Materials",
            Self::Service => "N/A",
        }
    }

    /// Check if this material type has physical inventory.
    pub fn has_inventory(&self) -> bool {
        !matches!(self, Self::Service)
    }
}

/// Material group for categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MaterialGroup {
    /// Electronics and components
    #[default]
    Electronics,
    /// Mechanical parts
    Mechanical,
    /// Chemicals and raw materials
    Chemicals,
    /// Chemical (alias for Chemicals)
    Chemical,
    /// Office supplies
    OfficeSupplies,
    /// IT equipment
    ItEquipment,
    /// Furniture
    Furniture,
    /// Packaging materials
    PackagingMaterials,
    /// Safety equipment
    SafetyEquipment,
    /// Tools
    Tools,
    /// Services
    Services,
    /// Consumables
    Consumables,
    /// Finished goods
    FinishedGoods,
}

impl MaterialGroup {
    /// Get typical unit of measure for this material group.
    pub fn typical_uom(&self) -> &'static str {
        match self {
            Self::Electronics | Self::Mechanical | Self::ItEquipment => "EA",
            Self::Chemicals | Self::Chemical => "KG",
            Self::OfficeSupplies | Self::PackagingMaterials | Self::Consumables => "EA",
            Self::Furniture | Self::FinishedGoods => "EA",
            Self::SafetyEquipment | Self::Tools => "EA",
            Self::Services => "HR",
        }
    }
}

/// Inventory valuation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ValuationMethod {
    /// Standard cost valuation
    #[default]
    StandardCost,
    /// Moving average price
    MovingAverage,
    /// First-in, first-out
    Fifo,
    /// Last-in, first-out (where permitted)
    Lifo,
    /// Specific identification
    SpecificIdentification,
}

/// Unit of measure for materials.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitOfMeasure {
    /// Unit code (e.g., "EA", "KG", "L")
    pub code: String,
    /// Full name
    pub name: String,
    /// Conversion factor to base unit (1.0 for base unit)
    pub conversion_factor: Decimal,
}

impl UnitOfMeasure {
    /// Create each (piece) unit.
    pub fn each() -> Self {
        Self {
            code: "EA".to_string(),
            name: "Each".to_string(),
            conversion_factor: Decimal::ONE,
        }
    }

    /// Create kilogram unit.
    pub fn kilogram() -> Self {
        Self {
            code: "KG".to_string(),
            name: "Kilogram".to_string(),
            conversion_factor: Decimal::ONE,
        }
    }

    /// Create liter unit.
    pub fn liter() -> Self {
        Self {
            code: "L".to_string(),
            name: "Liter".to_string(),
            conversion_factor: Decimal::ONE,
        }
    }

    /// Create hour unit (for services).
    pub fn hour() -> Self {
        Self {
            code: "HR".to_string(),
            name: "Hour".to_string(),
            conversion_factor: Decimal::ONE,
        }
    }
}

impl Default for UnitOfMeasure {
    fn default() -> Self {
        Self::each()
    }
}

/// Component in a bill of materials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomComponent {
    /// Component material ID
    pub component_material_id: String,
    /// Quantity required per parent unit
    pub quantity: Decimal,
    /// Unit of measure
    pub uom: String,
    /// Scrap percentage (waste factor)
    pub scrap_percentage: Decimal,
    /// Is this component optional?
    pub is_optional: bool,
    /// Position/sequence in BOM
    pub position: u16,
}

impl BomComponent {
    /// Create a new BOM component.
    pub fn new(
        component_material_id: impl Into<String>,
        quantity: Decimal,
        uom: impl Into<String>,
    ) -> Self {
        Self {
            component_material_id: component_material_id.into(),
            quantity,
            uom: uom.into(),
            scrap_percentage: Decimal::ZERO,
            is_optional: false,
            position: 0,
        }
    }

    /// Set scrap percentage.
    pub fn with_scrap(mut self, scrap_percentage: Decimal) -> Self {
        self.scrap_percentage = scrap_percentage;
        self
    }

    /// Calculate effective quantity including scrap.
    pub fn effective_quantity(&self) -> Decimal {
        self.quantity * (Decimal::ONE + self.scrap_percentage / Decimal::from(100))
    }
}

/// Account determination rules for material transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialAccountDetermination {
    /// Inventory account
    pub inventory_account: String,
    /// COGS account (for sales)
    pub cogs_account: String,
    /// Revenue account (for sales)
    pub revenue_account: String,
    /// Purchase expense account (for non-inventory items)
    pub purchase_expense_account: String,
    /// Price difference account
    pub price_difference_account: String,
    /// GR/IR clearing account
    pub gr_ir_account: String,
}

impl Default for MaterialAccountDetermination {
    fn default() -> Self {
        Self {
            inventory_account: "140000".to_string(),
            cogs_account: "500000".to_string(),
            revenue_account: "400000".to_string(),
            purchase_expense_account: "600000".to_string(),
            price_difference_account: "580000".to_string(),
            gr_ir_account: "290000".to_string(),
        }
    }
}

/// Material master data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Material ID (e.g., "MAT-001234")
    pub material_id: String,

    /// Material description
    pub description: String,

    /// Type of material
    pub material_type: MaterialType,

    /// Material group
    pub material_group: MaterialGroup,

    /// Base unit of measure
    pub base_uom: UnitOfMeasure,

    /// Valuation method
    pub valuation_method: ValuationMethod,

    /// Standard cost per base unit
    pub standard_cost: Decimal,

    /// List price (selling price) per base unit
    pub list_price: Decimal,

    /// Purchase price per base unit
    pub purchase_price: Decimal,

    /// Bill of materials components (if this is a produced item)
    pub bom_components: Option<Vec<BomComponent>>,

    /// Account determination rules
    pub account_determination: MaterialAccountDetermination,

    /// Weight per base unit (kg)
    pub weight_kg: Option<Decimal>,

    /// Volume per base unit (m3)
    pub volume_m3: Option<Decimal>,

    /// Shelf life in days (for perishables)
    pub shelf_life_days: Option<u32>,

    /// Is this material active?
    pub is_active: bool,

    /// Company code (if material is company-specific)
    pub company_code: Option<String>,

    /// Plant/location codes where material is available
    pub plants: Vec<String>,

    /// Minimum order quantity
    pub min_order_quantity: Decimal,

    /// Lead time in days for procurement
    pub lead_time_days: u16,

    /// Safety stock quantity
    pub safety_stock: Decimal,

    /// Reorder point
    pub reorder_point: Decimal,

    /// Preferred vendor ID
    pub preferred_vendor_id: Option<String>,

    /// ABC classification (A=high value, C=low value)
    pub abc_classification: char,
}

impl Material {
    /// Create a new material with minimal required fields.
    pub fn new(
        material_id: impl Into<String>,
        description: impl Into<String>,
        material_type: MaterialType,
    ) -> Self {
        Self {
            material_id: material_id.into(),
            description: description.into(),
            material_type,
            material_group: MaterialGroup::default(),
            base_uom: UnitOfMeasure::default(),
            valuation_method: ValuationMethod::default(),
            standard_cost: Decimal::ZERO,
            list_price: Decimal::ZERO,
            purchase_price: Decimal::ZERO,
            bom_components: None,
            account_determination: MaterialAccountDetermination::default(),
            weight_kg: None,
            volume_m3: None,
            shelf_life_days: None,
            is_active: true,
            company_code: None,
            plants: vec!["1000".to_string()],
            min_order_quantity: Decimal::ONE,
            lead_time_days: 7,
            safety_stock: Decimal::ZERO,
            reorder_point: Decimal::ZERO,
            preferred_vendor_id: None,
            abc_classification: 'B',
        }
    }

    /// Set material group.
    pub fn with_group(mut self, group: MaterialGroup) -> Self {
        self.material_group = group;
        self
    }

    /// Set standard cost.
    pub fn with_standard_cost(mut self, cost: Decimal) -> Self {
        self.standard_cost = cost;
        self
    }

    /// Set list price.
    pub fn with_list_price(mut self, price: Decimal) -> Self {
        self.list_price = price;
        self
    }

    /// Set purchase price.
    pub fn with_purchase_price(mut self, price: Decimal) -> Self {
        self.purchase_price = price;
        self
    }

    /// Set BOM components.
    pub fn with_bom(mut self, components: Vec<BomComponent>) -> Self {
        self.bom_components = Some(components);
        self
    }

    /// Set company code.
    pub fn with_company_code(mut self, code: impl Into<String>) -> Self {
        self.company_code = Some(code.into());
        self
    }

    /// Set preferred vendor.
    pub fn with_preferred_vendor(mut self, vendor_id: impl Into<String>) -> Self {
        self.preferred_vendor_id = Some(vendor_id.into());
        self
    }

    /// Set ABC classification.
    pub fn with_abc_classification(mut self, classification: char) -> Self {
        self.abc_classification = classification;
        self
    }

    /// Calculate the theoretical cost from BOM.
    pub fn calculate_bom_cost(
        &self,
        component_costs: &std::collections::HashMap<String, Decimal>,
    ) -> Option<Decimal> {
        self.bom_components.as_ref().map(|components| {
            components
                .iter()
                .map(|c| {
                    let unit_cost = component_costs
                        .get(&c.component_material_id)
                        .copied()
                        .unwrap_or(Decimal::ZERO);
                    unit_cost * c.effective_quantity()
                })
                .sum()
        })
    }

    /// Calculate gross margin percentage.
    pub fn gross_margin_percent(&self) -> Decimal {
        if self.list_price > Decimal::ZERO {
            (self.list_price - self.standard_cost) / self.list_price * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    }

    /// Check if reorder is needed based on current stock.
    pub fn needs_reorder(&self, current_stock: Decimal) -> bool {
        current_stock <= self.reorder_point
    }

    /// Calculate reorder quantity based on EOQ principles.
    pub fn suggested_reorder_quantity(&self) -> Decimal {
        // Simplified: order enough to cover lead time plus safety stock
        self.reorder_point + self.safety_stock + self.min_order_quantity
    }
}

/// Pool of materials for transaction generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialPool {
    /// All materials
    pub materials: Vec<Material>,
    /// Index by material type
    #[serde(skip)]
    type_index: std::collections::HashMap<MaterialType, Vec<usize>>,
    /// Index by material group
    #[serde(skip)]
    group_index: std::collections::HashMap<MaterialGroup, Vec<usize>>,
    /// Index by ABC classification
    #[serde(skip)]
    abc_index: std::collections::HashMap<char, Vec<usize>>,
}

impl MaterialPool {
    /// Create a new empty material pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a material pool from a vector of materials.
    ///
    /// This is the preferred way to create a pool from generated master data,
    /// ensuring JEs reference real entities.
    pub fn from_materials(materials: Vec<Material>) -> Self {
        let mut pool = Self::new();
        for material in materials {
            pool.add_material(material);
        }
        pool
    }

    /// Add a material to the pool.
    pub fn add_material(&mut self, material: Material) {
        let idx = self.materials.len();
        let material_type = material.material_type;
        let material_group = material.material_group;
        let abc = material.abc_classification;

        self.materials.push(material);

        self.type_index.entry(material_type).or_default().push(idx);
        self.group_index
            .entry(material_group)
            .or_default()
            .push(idx);
        self.abc_index.entry(abc).or_default().push(idx);
    }

    /// Get a random material.
    pub fn random_material(&self, rng: &mut impl rand::Rng) -> Option<&Material> {
        use rand::seq::SliceRandom;
        self.materials.choose(rng)
    }

    /// Get a random material of a specific type.
    pub fn random_material_of_type(
        &self,
        material_type: MaterialType,
        rng: &mut impl rand::Rng,
    ) -> Option<&Material> {
        use rand::seq::SliceRandom;
        self.type_index
            .get(&material_type)
            .and_then(|indices| indices.choose(rng))
            .map(|&idx| &self.materials[idx])
    }

    /// Get materials by ABC classification.
    pub fn get_by_abc(&self, classification: char) -> Vec<&Material> {
        self.abc_index
            .get(&classification)
            .map(|indices| indices.iter().map(|&i| &self.materials[i]).collect())
            .unwrap_or_default()
    }

    /// Rebuild indices after deserialization.
    pub fn rebuild_indices(&mut self) {
        self.type_index.clear();
        self.group_index.clear();
        self.abc_index.clear();

        for (idx, material) in self.materials.iter().enumerate() {
            self.type_index
                .entry(material.material_type)
                .or_default()
                .push(idx);
            self.group_index
                .entry(material.material_group)
                .or_default()
                .push(idx);
            self.abc_index
                .entry(material.abc_classification)
                .or_default()
                .push(idx);
        }
    }

    /// Get material by ID.
    pub fn get_by_id(&self, material_id: &str) -> Option<&Material> {
        self.materials.iter().find(|m| m.material_id == material_id)
    }

    /// Get count of materials.
    pub fn len(&self) -> usize {
        self.materials.len()
    }

    /// Check if pool is empty.
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let material = Material::new("MAT-001", "Test Material", MaterialType::RawMaterial)
            .with_standard_cost(Decimal::from(100))
            .with_list_price(Decimal::from(150))
            .with_abc_classification('A');

        assert_eq!(material.material_id, "MAT-001");
        assert_eq!(material.standard_cost, Decimal::from(100));
        assert_eq!(material.abc_classification, 'A');
    }

    #[test]
    fn test_gross_margin() {
        let material = Material::new("MAT-001", "Test", MaterialType::FinishedGood)
            .with_standard_cost(Decimal::from(60))
            .with_list_price(Decimal::from(100));

        let margin = material.gross_margin_percent();
        assert_eq!(margin, Decimal::from(40));
    }

    #[test]
    fn test_bom_cost_calculation() {
        let mut component_costs = std::collections::HashMap::new();
        component_costs.insert("COMP-001".to_string(), Decimal::from(10));
        component_costs.insert("COMP-002".to_string(), Decimal::from(20));

        let material = Material::new("FG-001", "Finished Good", MaterialType::FinishedGood)
            .with_bom(vec![
                BomComponent::new("COMP-001", Decimal::from(2), "EA"),
                BomComponent::new("COMP-002", Decimal::from(3), "EA"),
            ]);

        let bom_cost = material.calculate_bom_cost(&component_costs).unwrap();
        assert_eq!(bom_cost, Decimal::from(80)); // 2*10 + 3*20
    }

    #[test]
    fn test_material_pool() {
        let mut pool = MaterialPool::new();

        pool.add_material(Material::new("MAT-001", "Raw 1", MaterialType::RawMaterial));
        pool.add_material(Material::new(
            "MAT-002",
            "Finished 1",
            MaterialType::FinishedGood,
        ));
        pool.add_material(Material::new("MAT-003", "Raw 2", MaterialType::RawMaterial));

        assert_eq!(pool.len(), 3);
        assert!(pool.get_by_id("MAT-001").is_some());
        assert!(pool.get_by_id("MAT-999").is_none());
    }

    #[test]
    fn test_bom_component_scrap() {
        let component =
            BomComponent::new("COMP-001", Decimal::from(100), "EA").with_scrap(Decimal::from(5)); // 5% scrap

        let effective = component.effective_quantity();
        assert_eq!(effective, Decimal::from(105)); // 100 * 1.05
    }
}
