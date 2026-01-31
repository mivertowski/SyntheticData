//! Context-aware anomaly injection.
//!
//! This module provides entity-aware injection patterns and behavioral
//! baseline tracking for context-sensitive anomaly generation.

mod behavioral_baseline;
mod entity_aware;

pub use behavioral_baseline::{
    BehavioralBaseline, BehavioralBaselineConfig, BehavioralDeviation, DeviationType,
    EntityBaseline, Observation,
};
pub use entity_aware::{
    AccountAnomalyRules, AccountContext, EmployeeAnomalyRules, EmployeeContext,
    EntityAwareConfig, EntityAwareInjector, VendorAnomalyRules, VendorContext,
};
