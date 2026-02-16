//! Tax accounting generators.
//!
//! This module provides generators for:
//! - Tax jurisdictions and tax code master data
//! - Tax lines on AP/AR/JE documents (decorator pattern)
//! - Tax return aggregation by jurisdiction and period
//! - Tax provisions (ASC 740 / IAS 12)
//! - Withholding tax on cross-border payments

mod tax_code_generator;

pub use tax_code_generator::*;
