//! Compliance & Regulations Framework models.
//!
//! Provides unified abstractions for regulatory standards, jurisdiction profiles,
//! temporal versioning, audit assertions, compliance findings, and cross-references.
//!
//! ## Key Types
//!
//! - [`StandardId`]: Canonical identifier for any compliance standard
//! - [`ComplianceStandard`]: Full metadata for a standard with versions and cross-references
//! - [`TemporalVersion`]: A specific version of a standard with effective date bounds
//! - [`JurisdictionProfile`]: Country-specific compliance composition
//! - [`ComplianceAssertion`]: Audit assertion types (ISA 315)
//! - [`ComplianceFinding`]: Audit finding with deficiency classification
//! - [`CrossReference`]: Typed link between related standards

mod assertion;
mod cross_reference;
mod filing;
mod finding;
mod jurisdiction;
mod standard;
mod standard_id;
mod temporal;

pub use assertion::*;
pub use cross_reference::*;
pub use filing::*;
pub use finding::*;
pub use jurisdiction::*;
pub use standard::*;
pub use standard_id::*;
pub use temporal::*;
