//! Project accounting generators.
//!
//! This module provides generators for:
//! - Project creation with WBS hierarchies
//! - Cost linking (time entries, expenses, POs → project cost lines)
//! - Revenue recognition (Percentage of Completion / ASC 606)
//! - Earned Value Management (EVM) metrics
//! - Change orders with cost/schedule/revenue impacts
//! - Milestones with payment and completion tracking
//! - Retainage hold and release

mod project_generator;
mod project_cost_generator;
mod revenue_generator;
mod earned_value_generator;
mod change_order_generator;

pub use project_generator::*;
pub use project_cost_generator::*;
pub use revenue_generator::*;
pub use earned_value_generator::*;
pub use change_order_generator::*;
