#![deny(clippy::unwrap_used)]
//! Accounting and Audit Standards Framework for Synthetic Data Generation.
//!
//! This crate provides comprehensive support for major accounting and auditing
//! standards frameworks used in financial reporting and audit procedures:
//!
//! ## Accounting Standards
//!
//! - **US GAAP**: United States Generally Accepted Accounting Principles
//!   - ASC 606: Revenue from Contracts with Customers
//!   - ASC 842: Leases
//!   - ASC 820: Fair Value Measurement
//!   - ASC 360: Impairment of Long-Lived Assets
//!
//! - **IFRS**: International Financial Reporting Standards
//!   - IFRS 15: Revenue from Contracts with Customers
//!   - IFRS 16: Leases
//!   - IFRS 13: Fair Value Measurement
//!   - IAS 36: Impairment of Assets
//!
//! ## Audit Standards
//!
//! - **ISA**: International Standards on Auditing
//!   - ISA 200-720: Complete coverage of 34 ISA standards
//!   - ISA 520: Analytical Procedures
//!   - ISA 505: External Confirmations
//!   - ISA 700/705/706/701: Audit Reports and Opinions
//!
//! - **PCAOB**: Public Company Accounting Oversight Board Standards
//!   - AS 2201: Auditing Internal Control Over Financial Reporting
//!   - AS 2110: Identifying and Assessing Risks
//!   - AS 3101: The Auditor's Report
//!
//! ## Regulatory Frameworks
//!
//! - **SOX**: Sarbanes-Oxley Act
//!   - Section 302: CEO/CFO Certifications
//!   - Section 404: Internal Control Assessment
//!
//! ## Usage
//!
//! ```rust
//! use datasynth_standards::framework::AccountingFramework;
//! use datasynth_standards::accounting::revenue::{CustomerContract, PerformanceObligation};
//! use datasynth_standards::audit::isa_reference::IsaStandard;
//!
//! // Select accounting framework
//! let framework = AccountingFramework::UsGaap;
//!
//! // Revenue recognition under ASC 606
//! // let contract = CustomerContract::new(...);
//!
//! // ISA compliance tracking
//! let standard = IsaStandard::Isa315;
//! ```

pub mod framework;

pub mod accounting;
pub mod audit;
pub mod registry;
pub mod regulatory;

// Re-export key types at crate root for convenience
pub use framework::{AccountingFramework, FrameworkSettings};
