#![deny(clippy::unwrap_used)]
//! # synth-generators
//!
//! Data generators for journal entries, chart of accounts, ACDOCA event logs,
//! master data entities, document flows, intercompany transactions, balance coherence,
//! subledger transactions, FX rates, period close processes, anomaly injection,
//! and data quality variations.

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

// Industry benchmark generator (WI-3)
pub mod industry_benchmark_generator;

// Enterprise process chain generators
pub mod bank_reconciliation_generator;
pub mod budget_generator;
pub mod compliance;
pub mod drift_event_generator;
pub mod esg;
pub mod hr;
pub mod kpi_generator;
pub mod manufacturing;
pub mod organizational_event_generator;
pub mod process_evolution_generator;
pub mod project_accounting;
pub mod sales_quote_generator;
pub mod sourcing;
pub mod standards;
pub mod tax;
pub mod treasury;

// ---------------------------------------------------------------------------
// Root-level re-exports
//
// Modules consumed by datasynth-runtime or internal tests from the crate root
// keep glob re-exports so that `use datasynth_generators::SomeType;` works.
// Modules that are only accessed via qualified paths (e.g.
// `datasynth_generators::fraud::RedFlagGenerator`) expose their types through
// `pub mod` above and do NOT pollute the crate root namespace.
// ---------------------------------------------------------------------------

// Core generators
pub use coa_generator::*;
pub use je_generator::*;
pub use user_generator::*;

// Master data generators
pub use master_data::*;

// Document flow generators
pub use document_flow::*;

// Anomaly injection
pub use anomaly::*;

// Data quality
pub use data_quality::*;

// Audit generators
pub use audit::*;

// Balance validation
pub use balance::*;

// Subledger generators
pub use subledger::*;

// Sourcing generators (S2C)
pub use sourcing::*;

// Period close / financial statements
pub use period_close::*;

// Bank reconciliation generator
pub use bank_reconciliation_generator::*;

// ESG generators
pub use esg::*;

// Intercompany generators
pub use intercompany::*;

// HR generators (payroll, time entry, expense report)
pub use hr::*;

// Manufacturing generators
pub use manufacturing::*;

// Accounting standards generators
pub use standards::*;

// Enterprise process chain generators
pub use budget_generator::*;
pub use drift_event_generator::*;
pub use kpi_generator::*;
pub use organizational_event_generator::*;
pub use process_evolution_generator::*;
pub use sales_quote_generator::*;
pub use tax::*;

// Control generator
pub use control_generator::{ControlGenerator, ControlGeneratorConfig};

// Industry benchmark generator (WI-3)
pub use industry_benchmark_generator::*;

// ---------------------------------------------------------------------------
// Modules below are accessible via qualified paths only:
//   datasynth_generators::company_selector::WeightedCompanySelector
//   datasynth_generators::counterfactual::CounterfactualGenerator
//   datasynth_generators::disruption::DisruptionManager
//   datasynth_generators::fraud::{RedFlagGenerator, CollusionNetwork, ...}
//   datasynth_generators::fx::{FxRateService, CurrencyTranslator, ...}
//   datasynth_generators::industry::{IndustryTransactionGenerator, ...}
//   datasynth_generators::relationships::{EntityGraphGenerator, ...}
//   datasynth_generators::temporal::TemporalAttributeGenerator
//   datasynth_generators::project_accounting::{ProjectGenerator, ...}
//   datasynth_generators::treasury::{CashPositionGenerator, ...}
// ---------------------------------------------------------------------------
