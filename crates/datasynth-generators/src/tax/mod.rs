//! Tax accounting generators.
//!
//! This module provides generators for:
//! - Tax jurisdictions and tax code master data
//! - Tax lines on AP/AR/JE documents (decorator pattern)
//! - Tax return aggregation by jurisdiction and period
//! - Tax provisions (ASC 740 / IAS 12)
//! - Withholding tax on cross-border payments

mod tax_anomaly;
mod tax_code_generator;
mod tax_line_generator;
mod tax_provision_generator;
mod tax_return_generator;
mod withholding_generator;

pub use tax_anomaly::*;
pub use tax_code_generator::*;
pub use tax_line_generator::*;
pub use tax_provision_generator::*;
pub use tax_return_generator::*;
pub use withholding_generator::*;
