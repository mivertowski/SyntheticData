//! Source-to-Contract (S2C) models for procurement pipeline simulation.
//!
//! This module provides models for the complete sourcing lifecycle:
//! - Spend analysis and category management
//! - Sourcing projects and supplier qualification
//! - RFx events (RFI/RFP/RFQ) and bid management
//! - Bid evaluation and award recommendations
//! - Procurement contracts and catalog management
//! - Supplier scorecards and performance tracking

mod bid;
mod bid_evaluation;
mod catalog;
mod contract;
mod qualification;
mod rfx;
mod scorecard;
mod sourcing_project;
mod spend_analysis;

pub use bid::*;
pub use bid_evaluation::*;
pub use catalog::*;
pub use contract::*;
pub use qualification::*;
pub use rfx::*;
pub use scorecard::*;
pub use sourcing_project::*;
pub use spend_analysis::*;
