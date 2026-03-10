//! Manufacturing model structs for inventory movements.
//!
//! Complements the existing `production_order.rs`, `quality_inspection.rs`,
//! and `cycle_count.rs` models with stock movement tracking.
//! BOM components live in `material.rs` alongside the existing `BomComponent`.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// Type of inventory movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MovementType {
    /// Receipt from purchase order
    #[default]
    GoodsReceipt,
    /// Issue to production order or cost center
    GoodsIssue,
    /// Transfer between storage locations
    Transfer,
    /// Return to vendor
    Return,
    /// Scrap / write-off
    Scrap,
    /// Inventory adjustment (cycle count, revaluation)
    Adjustment,
}

/// A stock movement record (goods receipt, issue, transfer, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryMovement {
    /// Unique movement document ID
    pub id: String,
    /// Company / entity code
    pub entity_code: String,
    /// Material ID
    pub material_code: String,
    /// Material description
    pub material_description: String,
    /// Date of the movement
    pub movement_date: NaiveDate,
    /// Fiscal period (e.g. "2024-06")
    pub period: String,
    /// Movement type
    pub movement_type: MovementType,
    /// Quantity moved
    #[serde(with = "rust_decimal::serde::str")]
    pub quantity: Decimal,
    /// Unit of measure
    pub unit: String,
    /// Total value of the movement
    #[serde(with = "rust_decimal::serde::str")]
    pub value: Decimal,
    /// Currency code
    pub currency: String,
    /// Storage location
    pub storage_location: String,
    /// Reference document (PO, production order, etc.)
    pub reference_doc: String,
}

impl InventoryMovement {
    /// Create a new inventory movement.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        entity_code: impl Into<String>,
        material_code: impl Into<String>,
        material_description: impl Into<String>,
        movement_date: NaiveDate,
        period: impl Into<String>,
        movement_type: MovementType,
        quantity: Decimal,
        unit: impl Into<String>,
        value: Decimal,
        currency: impl Into<String>,
        storage_location: impl Into<String>,
        reference_doc: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            entity_code: entity_code.into(),
            material_code: material_code.into(),
            material_description: material_description.into(),
            movement_date,
            period: period.into(),
            movement_type,
            quantity,
            unit: unit.into(),
            value,
            currency: currency.into(),
            storage_location: storage_location.into(),
            reference_doc: reference_doc.into(),
        }
    }
}

impl ToNodeProperties for InventoryMovement {
    fn node_type_name(&self) -> &'static str {
        "inventory_movement"
    }
    fn node_type_code(&self) -> u16 {
        105
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.entity_code.clone()),
        );
        p.insert(
            "materialCode".into(),
            GraphPropertyValue::String(self.material_code.clone()),
        );
        p.insert(
            "materialDescription".into(),
            GraphPropertyValue::String(self.material_description.clone()),
        );
        p.insert(
            "movementDate".into(),
            GraphPropertyValue::Date(self.movement_date),
        );
        p.insert(
            "period".into(),
            GraphPropertyValue::String(self.period.clone()),
        );
        p.insert(
            "movementType".into(),
            GraphPropertyValue::String(format!("{:?}", self.movement_type)),
        );
        p.insert(
            "quantity".into(),
            GraphPropertyValue::Decimal(self.quantity),
        );
        p.insert("unit".into(), GraphPropertyValue::String(self.unit.clone()));
        p.insert("value".into(), GraphPropertyValue::Decimal(self.value));
        p.insert(
            "currency".into(),
            GraphPropertyValue::String(self.currency.clone()),
        );
        p.insert(
            "storageLocation".into(),
            GraphPropertyValue::String(self.storage_location.clone()),
        );
        p.insert(
            "referenceDoc".into(),
            GraphPropertyValue::String(self.reference_doc.clone()),
        );
        p
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_movement_properties() {
        let mv = InventoryMovement::new(
            "MV-001",
            "C001",
            "MAT-100",
            "Widget A",
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            "2024-06",
            MovementType::GoodsReceipt,
            Decimal::new(100, 0),
            "EA",
            Decimal::new(5000, 0),
            "USD",
            "WH01",
            "PO-12345",
        );
        let props = mv.to_node_properties();
        assert_eq!(mv.node_type_name(), "inventory_movement");
        assert_eq!(mv.node_type_code(), 105);
        assert!(props.contains_key("movementType"));
        assert!(props.contains_key("storageLocation"));
        assert_eq!(
            props["quantity"],
            GraphPropertyValue::Decimal(Decimal::new(100, 0))
        );
    }
}
