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
mod approval;
mod chart_of_accounts;
mod company;
mod control_mapping;
mod coso;
mod customer_segment;
mod department;
mod entity_registry;
mod fixed_asset;
mod fx;
mod internal_control;
mod journal_entry;
mod master_data;
mod material;
mod period_close;
mod project;
mod relationship;
mod sod;
mod temporal;
mod user;
mod vendor_network;

// Document models (Phase 2)
pub mod documents;

// Intercompany models (Phase 3)
pub mod intercompany;

// Balance coherence models (Phase 4)
pub mod balance;

// Subledger models (Phase 5)
pub mod subledger;

// Audit models (Phase 13-14: RustAssureTwin integration)
pub mod audit;

// Banking models (KYC/AML transaction generation)
pub mod banking;

pub use acdoca::*;
pub use anomaly::*;
pub use approval::*;
pub use chart_of_accounts::*;
pub use company::*;
pub use control_mapping::*;
pub use coso::*;
pub use customer_segment::*;
pub use department::*;
pub use entity_registry::*;
pub use fixed_asset::*;
pub use fx::*;
pub use internal_control::*;
pub use journal_entry::*;
pub use master_data::*;
pub use material::*;
pub use period_close::*;
pub use project::*;
pub use relationship::*;
pub use sod::*;
pub use temporal::*;
pub use user::*;
pub use vendor_network::*;
