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

pub mod confirmation_generator;
mod engagement_generator;
mod evidence_generator;
mod finding_generator;
mod judgment_generator;
mod risk_generator;
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
