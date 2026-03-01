//! Procurement contract models.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::graph_properties::{GraphPropertyValue, ToNodeProperties};

/// Type of procurement contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContractType {
    /// Fixed price contract
    #[default]
    FixedPrice,
    /// Blanket/framework agreement with quantity commitments
    Blanket,
    /// Time and materials contract
    TimeAndMaterials,
    /// Cost-plus contract
    CostPlus,
    /// Service level agreement
    ServiceAgreement,
}

/// Status of a procurement contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    /// Contract drafted
    #[default]
    Draft,
    /// Pending approval
    PendingApproval,
    /// Active and in force
    Active,
    /// Suspended (temporarily inactive)
    Suspended,
    /// Expired
    Expired,
    /// Terminated early
    Terminated,
    /// Renewed (new contract created)
    Renewed,
}

/// Contract terms and conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTerms {
    /// Payment terms (e.g., "NET30", "2/10 NET30")
    pub payment_terms: String,
    /// Delivery terms (incoterms)
    pub delivery_terms: Option<String>,
    /// Warranty period in months
    pub warranty_months: Option<u32>,
    /// Early termination penalty percentage
    pub early_termination_penalty_pct: Option<f64>,
    /// Auto-renewal enabled
    pub auto_renewal: bool,
    /// Notice period for termination (days)
    pub termination_notice_days: u32,
    /// Price adjustment clause enabled
    pub price_adjustment_clause: bool,
    /// Maximum annual price increase percentage
    pub max_annual_price_increase_pct: Option<f64>,
}

impl Default for ContractTerms {
    fn default() -> Self {
        Self {
            payment_terms: "NET30".to_string(),
            delivery_terms: None,
            warranty_months: None,
            early_termination_penalty_pct: None,
            auto_renewal: false,
            termination_notice_days: 90,
            price_adjustment_clause: false,
            max_annual_price_increase_pct: None,
        }
    }
}

/// Service level agreement within a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSla {
    /// SLA metric name (e.g., "on_time_delivery", "defect_rate")
    pub metric_name: String,
    /// Target value
    pub target_value: f64,
    /// Minimum acceptable value
    pub minimum_value: f64,
    /// Penalty for breach (percentage of contract value)
    pub breach_penalty_pct: f64,
    /// Measurement frequency (monthly, quarterly, etc.)
    pub measurement_frequency: String,
}

/// Line item within a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLineItem {
    /// Line number
    pub line_number: u16,
    /// Material/service ID
    pub material_id: Option<String>,
    /// Description
    pub description: String,
    /// Contracted unit price
    #[serde(with = "rust_decimal::serde::str")]
    pub unit_price: Decimal,
    /// Unit of measure
    pub uom: String,
    /// Minimum order quantity
    pub min_quantity: Option<Decimal>,
    /// Maximum/committed quantity
    pub max_quantity: Option<Decimal>,
    /// Quantity released (ordered) so far
    #[serde(default)]
    pub quantity_released: Decimal,
    /// Value released so far
    #[serde(default, with = "rust_decimal::serde::str")]
    pub value_released: Decimal,
}

/// A procurement contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcurementContract {
    /// Unique contract identifier
    pub contract_id: String,
    /// Company code
    pub company_code: String,
    /// Contract type
    pub contract_type: ContractType,
    /// Current status
    pub status: ContractStatus,
    /// Vendor ID
    pub vendor_id: String,
    /// Contract title
    pub title: String,
    /// Sourcing project ID (origin)
    pub sourcing_project_id: Option<String>,
    /// Winning bid ID (origin)
    pub bid_id: Option<String>,
    /// Start date
    pub start_date: NaiveDate,
    /// End date
    pub end_date: NaiveDate,
    /// Total contract value
    #[serde(with = "rust_decimal::serde::str")]
    pub total_value: Decimal,
    /// Value consumed so far
    #[serde(with = "rust_decimal::serde::str")]
    pub consumed_value: Decimal,
    /// Contract terms
    pub terms: ContractTerms,
    /// SLAs
    pub slas: Vec<ContractSla>,
    /// Line items
    pub line_items: Vec<ContractLineItem>,
    /// Spend category
    pub category_id: String,
    /// Contract owner
    pub owner_id: String,
    /// Amendment count
    pub amendment_count: u32,
    /// Previous contract ID (if renewal)
    pub previous_contract_id: Option<String>,
}

impl ToNodeProperties for ProcurementContract {
    fn node_type_name(&self) -> &'static str {
        "procurement_contract"
    }
    fn node_type_code(&self) -> u16 {
        324
    }
    fn to_node_properties(&self) -> HashMap<String, GraphPropertyValue> {
        let mut p = HashMap::new();
        p.insert(
            "contractId".into(),
            GraphPropertyValue::String(self.contract_id.clone()),
        );
        p.insert(
            "entityCode".into(),
            GraphPropertyValue::String(self.company_code.clone()),
        );
        p.insert(
            "contractType".into(),
            GraphPropertyValue::String(format!("{:?}", self.contract_type)),
        );
        p.insert(
            "status".into(),
            GraphPropertyValue::String(format!("{:?}", self.status)),
        );
        p.insert(
            "vendorId".into(),
            GraphPropertyValue::String(self.vendor_id.clone()),
        );
        p.insert(
            "title".into(),
            GraphPropertyValue::String(self.title.clone()),
        );
        p.insert(
            "startDate".into(),
            GraphPropertyValue::Date(self.start_date),
        );
        p.insert("endDate".into(), GraphPropertyValue::Date(self.end_date));
        p.insert(
            "totalValue".into(),
            GraphPropertyValue::Decimal(self.total_value),
        );
        p.insert(
            "consumedValue".into(),
            GraphPropertyValue::Decimal(self.consumed_value),
        );
        p.insert(
            "lineItemCount".into(),
            GraphPropertyValue::Int(self.line_items.len() as i64),
        );
        p.insert(
            "amendmentCount".into(),
            GraphPropertyValue::Int(self.amendment_count as i64),
        );
        p.insert(
            "isActive".into(),
            GraphPropertyValue::Bool(matches!(self.status, ContractStatus::Active)),
        );
        p
    }
}
