//! Manufacturing industry transaction generation.
//!
//! Provides manufacturing-specific:
//! - Production transactions (work orders, material requisitions, labor)
//! - Inventory costing (standard cost, variances, WIP)
//! - Master data (BOM, routings, work centers)
//! - Anomalies (yield manipulation, phantom production, cost fraud)

mod anomalies;
mod master_data;
mod transactions;

pub use anomalies::ManufacturingAnomaly;
pub use master_data::{
    BillOfMaterials, BomComponent, ManufacturingSettings, Routing, RoutingOperation, WorkCenter,
};
pub use transactions::{
    ManufacturingTransaction, ManufacturingTransactionGenerator, ProductionOrderType, ScrapReason,
    VarianceType,
};
