//! Production order models for manufacturing processes.
//!
//! These models represent production orders and their routing operations,
//! supporting the full manufacturing lifecycle from planning through completion.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of a production order through the manufacturing lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProductionOrderStatus {
    /// Order has been planned but not yet released
    Planned,
    /// Order has been released for execution
    Released,
    /// Order is currently being processed
    #[default]
    InProcess,
    /// Order production is completed
    Completed,
    /// Order has been closed and settled
    Closed,
    /// Order has been cancelled
    Cancelled,
}

/// Type of production order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProductionOrderType {
    /// Standard production run
    #[default]
    Standard,
    /// Rework of defective materials
    Rework,
    /// Prototype or trial production
    Prototype,
    /// Production triggered by a specific customer order
    MakeToOrder,
    /// Production for inventory replenishment
    MakeToStock,
}

/// A production order representing a manufacturing run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrder {
    /// Unique production order identifier
    pub order_id: String,
    /// Company code this order belongs to
    pub company_code: String,
    /// Material being produced
    pub material_id: String,
    /// Description of the material being produced
    pub material_description: String,
    /// Type of production order
    pub order_type: ProductionOrderType,
    /// Current status of the production order
    pub status: ProductionOrderStatus,
    /// Planned production quantity
    #[serde(with = "rust_decimal::serde::str")]
    pub planned_quantity: Decimal,
    /// Actual quantity produced
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_quantity: Decimal,
    /// Quantity scrapped during production
    #[serde(with = "rust_decimal::serde::str")]
    pub scrap_quantity: Decimal,
    /// Planned start date
    pub planned_start: NaiveDate,
    /// Planned end date
    pub planned_end: NaiveDate,
    /// Actual start date (set when production begins)
    pub actual_start: Option<NaiveDate>,
    /// Actual end date (set when production completes)
    pub actual_end: Option<NaiveDate>,
    /// Work center responsible for production
    pub work_center: String,
    /// Optional routing identifier
    pub routing_id: Option<String>,
    /// Planned cost of production
    #[serde(with = "rust_decimal::serde::str")]
    pub planned_cost: Decimal,
    /// Actual cost incurred
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_cost: Decimal,
    /// Total labor hours consumed
    pub labor_hours: f64,
    /// Total machine hours consumed
    pub machine_hours: f64,
    /// Production yield rate (0.0 to 1.0)
    pub yield_rate: f64,
    /// Optional batch number for traceability
    pub batch_number: Option<String>,
    /// Routing operations for this production order
    pub operations: Vec<RoutingOperation>,
}

/// A single operation within a production routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingOperation {
    /// Sequential operation number
    pub operation_number: u32,
    /// Description of the operation
    pub operation_description: String,
    /// Work center where the operation is performed
    pub work_center: String,
    /// Setup time in hours
    pub setup_time_hours: f64,
    /// Run time in hours
    pub run_time_hours: f64,
    /// Planned quantity for this operation
    #[serde(with = "rust_decimal::serde::str")]
    pub planned_quantity: Decimal,
    /// Actual quantity processed in this operation
    #[serde(with = "rust_decimal::serde::str")]
    pub actual_quantity: Decimal,
    /// Current status of the operation
    pub status: OperationStatus,
    /// Date the operation started
    pub started_at: Option<NaiveDate>,
    /// Date the operation completed
    pub completed_at: Option<NaiveDate>,
}

/// Status of a routing operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    /// Operation has not yet started
    #[default]
    Pending,
    /// Operation is currently being executed
    InProcess,
    /// Operation has been completed
    Completed,
    /// Operation has been cancelled
    Cancelled,
}
