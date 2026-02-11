#![deny(clippy::unwrap_used)]
//! # synth-generators
//!
//! Data generators for journal entries, chart of accounts, ACDOCA event logs,
//! master data entities, document flows, intercompany transactions, balance coherence,
//! subledger transactions, FX rates, period close processes, anomaly injection,
//! and data quality variations.

// Allow dead code for methods that are part of the public API but not yet used internally
#![allow(dead_code)]
// Allow complex types for return types that model business domain complexity
#![allow(clippy::type_complexity)]
// Allow functions with many arguments for domain-specific operations
#![allow(clippy::too_many_arguments)]
// Allow large error types as they contain useful diagnostic information
#![allow(clippy::result_large_err)]

pub mod anomaly;
pub mod audit;
pub mod balance;
pub mod coa_generator;
pub mod company_selector;
pub mod control_generator;
pub mod counterfactual;
pub mod data_quality;
pub mod disruption;
pub mod document_flow;
pub mod fraud;
pub mod fx;
pub mod industry;
pub mod intercompany;
pub mod je_generator;
pub mod llm_enrichment;
pub mod master_data;
pub mod period_close;
pub mod relationships;
pub mod subledger;
pub mod temporal;
pub mod user_generator;

pub use anomaly::*;
pub use audit::*;
pub use balance::*;
pub use coa_generator::*;
pub use company_selector::*;
pub use control_generator::*;
pub use counterfactual::*;
pub use data_quality::*;
pub use disruption::*;
pub use document_flow::*;
pub use fraud::*;
pub use fx::*;
pub use industry::*;
pub use intercompany::*;
pub use je_generator::*;
pub use master_data::*;
pub use period_close::*;
pub use relationships::*;
pub use subledger::*;
pub use temporal::*;
pub use user_generator::*;
