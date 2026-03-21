//! Master data generators for enterprise simulation.
//!
//! This module provides generators for various master data entities:
//! - Vendors (enhanced with payment behavior)
//! - Customers (enhanced with credit management)
//! - Materials (inventory and BOM)
//! - Fixed Assets (with depreciation)
//! - Employees (with org hierarchy)
//! - Cost Centers (two-level hierarchy)
//! - Entity Registry (central entity management)

mod asset_generator;
mod cost_center_generator;
mod customer_generator;
mod employee_generator;
mod entity_registry_manager;
mod material_generator;
mod vendor_generator;

pub use asset_generator::*;
pub use cost_center_generator::*;
pub use customer_generator::*;
pub use employee_generator::*;
pub use entity_registry_manager::*;
pub use material_generator::*;
pub use vendor_generator::*;
