//! Export functionality for OCPM event logs.
//!
//! This module provides export capabilities for OCPM event logs
//! in multiple standard formats:
//!
//! - **OCEL 2.0**: The Object-Centric Event Log standard for multi-object process mining
//! - **XES 2.0**: The IEEE standard for event logs, supported by ProM, Celonis, Disco
//! - **Reference Models**: Canonical process models for P2P, O2C, R2R processes

mod ocel2;
mod reference_model;
mod xes;

pub use ocel2::*;
pub use reference_model::{
    ReferenceActivity, ReferenceModelExporter, ReferenceProcessModel, ReferenceTransition,
    ReferenceVariant,
};
pub use xes::XesExporter;
