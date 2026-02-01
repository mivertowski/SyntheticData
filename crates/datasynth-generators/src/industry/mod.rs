//! Industry-specific transaction and anomaly generation.
//!
//! This module provides industry-authentic transaction patterns, master data,
//! and anomalies for:
//!
//! - Manufacturing: Production orders, BOM, inventory costing, variances
//! - Retail: POS transactions, returns, inventory shrinkage, promotions
//! - Healthcare: Revenue cycle, clinical coding, payer transactions
//! - Technology: Licenses, subscriptions, R&D capitalization
//! - Financial Services: Banking, investments, insurance
//! - Professional Services: Time/billing, engagements, trust accounts

pub mod healthcare;
pub mod manufacturing;
pub mod retail;

// Common traits and types for industry modules
mod common;

pub use common::{IndustryAnomaly, IndustryTransaction, IndustryTransactionGenerator};
pub use healthcare::{
    CodingSystem, HealthcareAnomaly, HealthcareSettings, HealthcareTransaction,
    HealthcareTransactionGenerator, PayerType,
};
pub use manufacturing::{
    BillOfMaterials, ManufacturingAnomaly, ManufacturingSettings, ManufacturingTransaction,
    ManufacturingTransactionGenerator, ProductionOrderType,
};
pub use retail::{
    RetailAnomaly, RetailSettings, RetailTransaction, RetailTransactionGenerator, StoreType,
};
