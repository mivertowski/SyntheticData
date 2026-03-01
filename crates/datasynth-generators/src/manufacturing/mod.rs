//! Manufacturing process generators.
//!
//! This module provides generators for manufacturing-specific data including
//! production orders, quality inspections, cycle counts, BOM components,
//! and inventory movements.

mod bom_generator;
mod cycle_count_generator;
mod inventory_movement_generator;
mod production_order_generator;
mod quality_inspection_generator;

pub use bom_generator::*;
pub use cycle_count_generator::*;
pub use inventory_movement_generator::*;
pub use production_order_generator::*;
pub use quality_inspection_generator::*;
