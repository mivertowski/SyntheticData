//! Quality inspection models for manufacturing processes.
//!
//! These models represent quality inspections performed on materials
//! during incoming receipt, in-process production, and final output.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// Result of a quality inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InspectionResult {
    /// Material accepted without conditions
    #[default]
    Accepted,
    /// Material rejected
    Rejected,
    /// Material accepted with conditions or concessions
    Conditionally,
    /// Inspection not yet completed
    Pending,
}

/// Type of quality inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InspectionType {
    /// Inspection of incoming materials from vendors
    #[default]
    Incoming,
    /// Inspection during production process
    InProcess,
    /// Final inspection before delivery or storage
    Final,
    /// Random sampling inspection
    Random,
    /// Scheduled periodic inspection
    Periodic,
}

/// A quality inspection record for a material lot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspection {
    /// Unique inspection identifier
    pub inspection_id: String,
    /// Company code this inspection belongs to
    pub company_code: String,
    /// Type of reference document (e.g., "production_order", "goods_receipt")
    pub reference_type: String,
    /// Identifier of the reference document
    pub reference_id: String,
    /// Material being inspected
    pub material_id: String,
    /// Description of the material being inspected
    pub material_description: String,
    /// Type of inspection performed
    pub inspection_type: InspectionType,
    /// Date the inspection was performed
    pub inspection_date: NaiveDate,
    /// Inspector who performed the inspection
    pub inspector_id: Option<String>,
    /// Total lot size under inspection
    #[serde(with = "rust_decimal::serde::str")]
    pub lot_size: Decimal,
    /// Sample size drawn for inspection
    #[serde(with = "rust_decimal::serde::str")]
    pub sample_size: Decimal,
    /// Number of defects found
    pub defect_count: u32,
    /// Defect rate (defect_count / sample_size)
    pub defect_rate: f64,
    /// Overall inspection result
    pub result: InspectionResult,
    /// Individual inspection characteristics measured
    pub characteristics: Vec<InspectionCharacteristic>,
    /// Disposition action (e.g., "use_as_is", "return_to_vendor", "scrap")
    pub disposition: Option<String>,
    /// Additional notes or observations
    pub notes: Option<String>,
}

impl ToNodeProperties for QualityInspection {
    fn node_type_name(&self) -> &'static str {
        "quality_inspection"
    }
    fn node_type_code(&self) -> u16 {
        341
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "inspectionId".into(),
            GraphPropertyValue::String(self.inspection_id.clone()),
        );
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.company_code.clone()),
        );
        p.insert(
            "referenceType".into(),
            GraphPropertyValue::String(self.reference_type.clone()),
        );
        p.insert(
            "referenceId".into(),
            GraphPropertyValue::String(self.reference_id.clone()),
        );
        p.insert(
            "materialId".into(),
            GraphPropertyValue::String(self.material_id.clone()),
        );
        p.insert(
            "materialDescription".into(),
            GraphPropertyValue::String(self.material_description.clone()),
        );
        p.insert(
            "inspectionType".into(),
            GraphPropertyValue::String(format!("{:?}", self.inspection_type)),
        );
        p.insert(
            "inspectionDate".into(),
            GraphPropertyValue::Date(self.inspection_date),
        );
        p.insert("lotSize".into(), GraphPropertyValue::Decimal(self.lot_size));
        p.insert(
            "inspectedQuantity".into(),
            GraphPropertyValue::Decimal(self.sample_size),
        );
        p.insert(
            "defectQuantity".into(),
            GraphPropertyValue::Int(self.defect_count as i64),
        );
        p.insert(
            "defectRate".into(),
            GraphPropertyValue::Float(self.defect_rate),
        );
        p.insert(
            "result".into(),
            GraphPropertyValue::String(format!("{:?}", self.result)),
        );
        p.insert(
            "isPassed".into(),
            GraphPropertyValue::Bool(matches!(
                self.result,
                InspectionResult::Accepted | InspectionResult::Conditionally
            )),
        );
        p
    }
}

/// A single measured characteristic within a quality inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionCharacteristic {
    /// Name of the characteristic (e.g., "diameter", "weight", "tensile_strength")
    pub name: String,
    /// Target specification value
    pub target_value: f64,
    /// Actual measured value
    pub actual_value: f64,
    /// Lower specification limit
    pub lower_limit: f64,
    /// Upper specification limit
    pub upper_limit: f64,
    /// Whether this characteristic passed inspection
    pub passed: bool,
}
