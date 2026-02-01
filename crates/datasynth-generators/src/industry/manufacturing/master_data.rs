//! Manufacturing master data structures.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Manufacturing industry settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingSettings {
    /// Bill of Materials depth (typical: 3-7).
    pub bom_depth: u32,
    /// Whether just-in-time inventory is used.
    pub just_in_time: bool,
    /// Production order types to generate.
    pub production_order_types: Vec<String>,
    /// Quality framework (ISO 9001, Six Sigma, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_framework: Option<String>,
    /// Number of supplier tiers to model.
    pub supplier_tiers: u32,
    /// Standard cost update frequency (monthly, quarterly, annual).
    pub standard_cost_frequency: String,
    /// Target yield rate (0.95-0.99 typical).
    pub target_yield_rate: f64,
    /// Scrap percentage threshold for alerts.
    pub scrap_alert_threshold: f64,
}

impl Default for ManufacturingSettings {
    fn default() -> Self {
        Self {
            bom_depth: 4,
            just_in_time: false,
            production_order_types: vec![
                "standard".to_string(),
                "rework".to_string(),
                "prototype".to_string(),
            ],
            quality_framework: Some("ISO_9001".to_string()),
            supplier_tiers: 2,
            standard_cost_frequency: "quarterly".to_string(),
            target_yield_rate: 0.97,
            scrap_alert_threshold: 0.03,
        }
    }
}

/// Bill of Materials for a product.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillOfMaterials {
    /// Product/finished goods ID.
    pub product_id: String,
    /// Product name.
    pub product_name: String,
    /// BOM components.
    pub components: Vec<BomComponent>,
    /// Number of levels in the BOM.
    pub levels: u32,
    /// Expected yield rate (0.95-0.99).
    pub yield_rate: f64,
    /// Scrap factor (0.01-0.05).
    pub scrap_factor: f64,
    /// Effective date.
    pub effective_date: String,
    /// Version number.
    pub version: u32,
    /// Whether this is the active BOM.
    pub is_active: bool,
}

impl BillOfMaterials {
    /// Creates a new BOM.
    pub fn new(product_id: impl Into<String>, product_name: impl Into<String>) -> Self {
        Self {
            product_id: product_id.into(),
            product_name: product_name.into(),
            components: Vec::new(),
            levels: 1,
            yield_rate: 0.97,
            scrap_factor: 0.02,
            effective_date: String::new(),
            version: 1,
            is_active: true,
        }
    }

    /// Adds a component.
    pub fn add_component(&mut self, component: BomComponent) {
        // Update levels if this component has a deeper BOM
        if component.bom_level >= self.levels {
            self.levels = component.bom_level + 1;
        }
        self.components.push(component);
    }

    /// Calculates total material cost at standard.
    pub fn total_material_cost(&self) -> Decimal {
        self.components
            .iter()
            .map(|c| c.standard_cost * Decimal::from_f64_retain(c.quantity).unwrap_or(Decimal::ONE))
            .sum()
    }

    /// Returns component count.
    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}

/// A component in a Bill of Materials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomComponent {
    /// Component material ID.
    pub material_id: String,
    /// Component name.
    pub material_name: String,
    /// Quantity required per unit of parent.
    pub quantity: f64,
    /// Unit of measure.
    pub unit_of_measure: String,
    /// BOM level (0 = direct component).
    pub bom_level: u32,
    /// Standard cost per unit.
    pub standard_cost: Decimal,
    /// Whether this is a phantom item (not stocked).
    pub is_phantom: bool,
    /// Scrap percentage for this component.
    pub scrap_percentage: f64,
    /// Lead time in days.
    pub lead_time_days: u32,
    /// Operation at which this is consumed (if routing-linked).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_number: Option<u32>,
}

impl BomComponent {
    /// Creates a new BOM component.
    pub fn new(
        material_id: impl Into<String>,
        material_name: impl Into<String>,
        quantity: f64,
        unit_of_measure: impl Into<String>,
    ) -> Self {
        Self {
            material_id: material_id.into(),
            material_name: material_name.into(),
            quantity,
            unit_of_measure: unit_of_measure.into(),
            bom_level: 0,
            standard_cost: Decimal::ZERO,
            is_phantom: false,
            scrap_percentage: 0.02,
            lead_time_days: 5,
            operation_number: None,
        }
    }

    /// Sets the standard cost.
    pub fn with_standard_cost(mut self, cost: Decimal) -> Self {
        self.standard_cost = cost;
        self
    }

    /// Sets the BOM level.
    pub fn at_level(mut self, level: u32) -> Self {
        self.bom_level = level;
        self
    }

    /// Marks as phantom item.
    pub fn as_phantom(mut self) -> Self {
        self.is_phantom = true;
        self
    }

    /// Sets the operation number.
    pub fn at_operation(mut self, op: u32) -> Self {
        self.operation_number = Some(op);
        self
    }
}

/// Manufacturing routing for a product.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Routing {
    /// Product ID this routing is for.
    pub product_id: String,
    /// Routing name/description.
    pub name: String,
    /// Routing operations.
    pub operations: Vec<RoutingOperation>,
    /// Effective date.
    pub effective_date: String,
    /// Version number.
    pub version: u32,
    /// Whether this is the active routing.
    pub is_active: bool,
}

impl Routing {
    /// Creates a new routing.
    pub fn new(product_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            product_id: product_id.into(),
            name: name.into(),
            operations: Vec::new(),
            effective_date: String::new(),
            version: 1,
            is_active: true,
        }
    }

    /// Adds an operation.
    pub fn add_operation(&mut self, operation: RoutingOperation) {
        self.operations.push(operation);
    }

    /// Returns total standard labor time.
    pub fn total_labor_time(&self) -> Decimal {
        self.operations
            .iter()
            .map(|o| o.setup_time_minutes + o.run_time_per_unit)
            .sum()
    }

    /// Returns total standard cost.
    pub fn total_standard_cost(&self) -> Decimal {
        self.operations
            .iter()
            .map(|o| {
                let setup_cost = o.setup_time_minutes / Decimal::new(60, 0) * o.labor_rate;
                let run_cost = o.run_time_per_unit / Decimal::new(60, 0) * o.labor_rate;
                let machine_cost = o.run_time_per_unit / Decimal::new(60, 0) * o.machine_rate;
                setup_cost + run_cost + machine_cost
            })
            .sum()
    }
}

/// A routing operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingOperation {
    /// Operation number (10, 20, 30, etc.).
    pub operation_number: u32,
    /// Operation description.
    pub description: String,
    /// Work center ID.
    pub work_center: String,
    /// Setup time in minutes.
    pub setup_time_minutes: Decimal,
    /// Run time per unit in minutes.
    pub run_time_per_unit: Decimal,
    /// Labor rate per hour.
    pub labor_rate: Decimal,
    /// Machine rate per hour.
    pub machine_rate: Decimal,
    /// Overlap percentage with previous operation (0-100).
    pub overlap_percent: f64,
    /// Move time to next operation in minutes.
    pub move_time_minutes: Decimal,
    /// Queue time before operation in minutes.
    pub queue_time_minutes: Decimal,
}

impl RoutingOperation {
    /// Creates a new routing operation.
    pub fn new(
        operation_number: u32,
        description: impl Into<String>,
        work_center: impl Into<String>,
    ) -> Self {
        Self {
            operation_number,
            description: description.into(),
            work_center: work_center.into(),
            setup_time_minutes: Decimal::new(30, 0),
            run_time_per_unit: Decimal::new(10, 0),
            labor_rate: Decimal::new(25, 0),
            machine_rate: Decimal::new(15, 0),
            overlap_percent: 0.0,
            move_time_minutes: Decimal::new(5, 0),
            queue_time_minutes: Decimal::new(60, 0),
        }
    }

    /// Sets run time per unit.
    pub fn with_run_time(mut self, minutes: Decimal) -> Self {
        self.run_time_per_unit = minutes;
        self
    }

    /// Sets labor rate.
    pub fn with_labor_rate(mut self, rate: Decimal) -> Self {
        self.labor_rate = rate;
        self
    }
}

/// Work center definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkCenter {
    /// Work center ID.
    pub work_center_id: String,
    /// Work center name.
    pub name: String,
    /// Department.
    pub department: String,
    /// Capacity in hours per day.
    pub capacity_hours: Decimal,
    /// Number of machines/resources.
    pub resource_count: u32,
    /// Efficiency percentage (0-100).
    pub efficiency: f64,
    /// Standard labor rate per hour.
    pub labor_rate: Decimal,
    /// Standard machine rate per hour.
    pub machine_rate: Decimal,
    /// Overhead rate per hour.
    pub overhead_rate: Decimal,
    /// Cost center for allocation.
    pub cost_center: String,
}

impl WorkCenter {
    /// Creates a new work center.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        department: impl Into<String>,
    ) -> Self {
        Self {
            work_center_id: id.into(),
            name: name.into(),
            department: department.into(),
            capacity_hours: Decimal::new(8, 0),
            resource_count: 1,
            efficiency: 85.0,
            labor_rate: Decimal::new(25, 0),
            machine_rate: Decimal::new(15, 0),
            overhead_rate: Decimal::new(10, 0),
            cost_center: String::new(),
        }
    }

    /// Sets the cost center.
    pub fn with_cost_center(mut self, cc: impl Into<String>) -> Self {
        self.cost_center = cc.into();
        self
    }

    /// Calculates total rate per hour.
    pub fn total_rate(&self) -> Decimal {
        self.labor_rate + self.machine_rate + self.overhead_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bom() {
        let mut bom = BillOfMaterials::new("FG001", "Finished Good 1");

        bom.add_component(
            BomComponent::new("RM001", "Raw Material 1", 2.0, "EA")
                .with_standard_cost(Decimal::new(10, 0))
                .at_level(0),
        );
        bom.add_component(
            BomComponent::new("RM002", "Raw Material 2", 1.5, "KG")
                .with_standard_cost(Decimal::new(5, 0))
                .at_level(0),
        );

        assert_eq!(bom.component_count(), 2);
        assert_eq!(bom.total_material_cost(), Decimal::new(275, 1)); // 2*10 + 1.5*5 = 27.5
    }

    #[test]
    fn test_routing() {
        let mut routing = Routing::new("FG001", "Standard Routing");

        routing.add_operation(
            RoutingOperation::new(10, "Cutting", "WC-CUT")
                .with_run_time(Decimal::new(5, 0))
                .with_labor_rate(Decimal::new(30, 0)),
        );
        routing.add_operation(RoutingOperation::new(20, "Assembly", "WC-ASM"));

        assert_eq!(routing.operations.len(), 2);
        assert!(routing.total_standard_cost() > Decimal::ZERO);
    }

    #[test]
    fn test_work_center() {
        let wc =
            WorkCenter::new("WC-001", "Assembly Line 1", "Production").with_cost_center("CC-PROD");

        assert_eq!(wc.total_rate(), Decimal::new(50, 0)); // 25 + 15 + 10
    }

    #[test]
    fn test_manufacturing_settings() {
        let settings = ManufacturingSettings::default();

        assert_eq!(settings.bom_depth, 4);
        assert!(!settings.just_in_time);
        assert!(settings.target_yield_rate > 0.9);
    }
}
