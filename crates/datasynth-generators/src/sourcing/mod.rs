//! Source-to-Contract (S2C) generators for the procurement pipeline.
//!
//! Generation DAG:
//! spend_analysis → sourcing_project → qualification → rfx → bid →
//! bid_evaluation → contract → catalog → [P2P existing] → scorecard

mod bid_evaluation_generator;
mod bid_generator;
mod catalog_generator;
mod contract_generator;
mod qualification_generator;
mod rfx_generator;
mod scorecard_generator;
mod sourcing_project_generator;
mod spend_analysis_generator;

pub use bid_evaluation_generator::*;
pub use bid_generator::*;
pub use catalog_generator::*;
pub use contract_generator::*;
pub use qualification_generator::*;
pub use rfx_generator::*;
pub use scorecard_generator::*;
pub use sourcing_project_generator::*;
pub use spend_analysis_generator::*;
