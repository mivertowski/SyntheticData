//! Domain models for synthetic accounting data generation.
//!
//! This module provides all the core data models for the enterprise
//! simulation, including:
//!
//! - Master data (vendors, customers, materials, fixed assets, employees)
//! - Transaction data (journal entries, ACDOCA event logs)
//! - Organizational data (companies, departments, cost centers)
//! - Control data (internal controls, SoD, approvals)
//! - Document data (purchase orders, invoices, payments, deliveries)
//! - Intercompany data (relationships, transfer pricing, eliminations)
//! - Temporal data (bi-temporal support for audit trails)
//! - Audit data (engagements, workpapers, evidence, findings)

mod acdoca;
mod anomaly;

// Graph property mapping trait (DS-001)
mod approval;
mod chart_of_accounts;
mod company;
mod control_mapping;
mod coso;
mod customer_segment;
pub mod deferred_tax;
mod department;
mod entity_registry;
mod fixed_asset;
mod fx;
pub mod graph_properties;
pub mod internal_control;
pub mod journal_entry;
mod master_data;
mod material;
mod period_close;
mod project;
mod project_accounting;
mod relationship;
mod sod;
mod tax;
mod temporal;
mod treasury;
mod user;

// ESG / Sustainability models
mod esg;
mod vendor_network;

// Source-to-Contract models (S2C pipeline)
pub mod sourcing;

// Bank reconciliation models
mod bank_reconciliation;

// Financial statement models
mod financial_statements;

// Hire-to-Retire (H2R) models
mod expense_report;
mod payroll;
mod time_entry;

// Manufacturing models
mod cycle_count;
mod manufacturing_models;
mod production_order;
mod quality_inspection;

// Wave 4: Sales Quotes, KPIs, Budgets
mod budget;
mod management_kpi;
mod sales_quote;

// Pattern drift models (Phase: Pattern and Process Drift Over Time)
pub mod drift_events;
pub mod organizational_event;
pub mod process_evolution;
pub mod regulatory_events;
pub mod technology_transition;

// Document models (Phase 2)
pub mod documents;

// Intercompany models (Phase 3)
pub mod intercompany;

// Balance coherence models (Phase 4)
pub mod balance;

// Subledger models (Phase 5)
pub mod subledger;

// Business combination models (IFRS 3 / ASC 805)
pub mod business_combination;

// Audit models (Phase 13-14: RustAssureTwin integration)
pub mod audit;

// Banking models (KYC/AML transaction generation)
pub mod banking;

// Counterfactual simulation models
pub mod causal_dag;
mod intervention;
mod scenario;

// Unified generation pipeline session models
pub mod generation_session;

// Compliance & Regulations Framework models
pub mod compliance;

pub use acdoca::*;
pub use anomaly::*;
pub use approval::*;
pub use chart_of_accounts::*;
pub use company::*;
pub use control_mapping::*;
pub use coso::*;
pub use customer_segment::*;
pub use deferred_tax::*;
pub use department::*;
pub use entity_registry::*;
pub use fixed_asset::*;
pub use fx::*;
pub use graph_properties::*;
pub use internal_control::*;
pub use journal_entry::*;
pub use master_data::*;
pub use material::*;
pub use period_close::*;
pub use project::*;
pub use project_accounting::*;
pub use relationship::*;
pub use sod::*;
pub use tax::*;
pub use temporal::*;
pub use treasury::*;
pub use user::*;
pub use vendor_network::*;

// ESG / Sustainability exports
pub use esg::*;

// Sourcing exports
pub use sourcing::*;

// Bank reconciliation exports
pub use bank_reconciliation::*;

// Financial statement exports
pub use financial_statements::*;

// Hire-to-Retire (H2R) exports
pub use expense_report::*;
pub use payroll::*;
pub use time_entry::*;

// Manufacturing exports
pub use cycle_count::*;
pub use manufacturing_models::*;
pub use production_order::*;
pub use quality_inspection::*;

// Wave 4 exports
pub use budget::*;
pub use management_kpi::*;
pub use sales_quote::*;

// Pattern drift exports
pub use drift_events::*;
pub use organizational_event::*;
pub use process_evolution::*;
pub use regulatory_events::*;
pub use technology_transition::*;

// Counterfactual simulation exports
pub use causal_dag::*;
pub use intervention::*;
pub use scenario::*;

// Business combination exports
pub use business_combination::*;

// Unified generation pipeline session exports
pub use generation_session::*;
