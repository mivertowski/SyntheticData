//! Audit data generators.
//!
//! This module provides generators for audit-related data:
//! - Audit engagements per ISA 210/220
//! - Workpapers per ISA 230
//! - Audit evidence per ISA 500
//! - Risk assessments per ISA 315/330
//! - Audit findings per ISA 265
//! - Professional judgments per ISA 200
//! - External confirmations per ISA 505 (`audit::confirmation_generator`)
//!
//! Note: `ConfirmationGenerator` / `ConfirmationGeneratorConfig` are NOT
//! wildcard-re-exported from this module to avoid a name collision with the
//! identically-named types in `standards::confirmation_generator`.  Import
//! them via the full path:
//! ```ignore
//! use datasynth_generators::audit::confirmation_generator::{
//!     ConfirmationGenerator, ConfirmationGeneratorConfig,
//! };
//! ```
//!
//! Similarly, the generators below are NOT wildcard-re-exported to avoid
//! potential name collisions.  Import them via their full module paths:
//! ```ignore
//! use datasynth_generators::audit::procedure_step_generator::{
//!     ProcedureStepGenerator, ProcedureStepGeneratorConfig,
//! };
//! use datasynth_generators::audit::sample_generator::{
//!     SampleGenerator, SampleGeneratorConfig,
//! };
//! use datasynth_generators::audit::analytical_procedure_generator::{
//!     AnalyticalProcedureGenerator, AnalyticalProcedureGeneratorConfig,
//! };
//! use datasynth_generators::audit::internal_audit_generator::{
//!     InternalAuditGenerator, InternalAuditGeneratorConfig,
//! };
//! use datasynth_generators::audit::related_party_generator::{
//!     RelatedPartyGenerator, RelatedPartyGeneratorConfig,
//! };
//! ```

pub mod accounting_estimate_generator;
pub mod analytical_procedure_generator;
pub mod audit_opinion_generator;
pub mod component_audit_generator;
pub mod confirmation_generator;
mod engagement_generator;
pub mod engagement_letter_generator;
mod evidence_generator;
mod finding_generator;
pub mod going_concern_generator;
pub mod internal_audit_generator;
mod judgment_generator;
pub mod procedure_step_generator;
pub mod related_party_generator;
mod risk_generator;
pub mod sample_generator;
pub mod service_org_generator;
pub mod sox_generator;
pub mod subsequent_event_generator;
mod workpaper_generator;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod test_helpers;

pub use engagement_generator::*;
pub use evidence_generator::*;
pub use finding_generator::*;
pub use judgment_generator::*;
pub use risk_generator::*;
pub use workpaper_generator::*;
