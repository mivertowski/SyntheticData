//! Fraud pattern generation with ACFE-aligned taxonomy.
//!
//! This module provides comprehensive fraud pattern generation including:
//! - ACFE-aligned fraud schemes
//! - Collusion and conspiracy network modeling
//! - Management override patterns
//! - Red flag generation with correlation probabilities
//! - Adaptive fraud behavior simulation
//! - Investigation scenario generation

pub mod collusion;
pub mod management_override;
pub mod red_flags;

pub use collusion::{
    CollusionRing, CollusionRingGenerator, CollusionRingType, Conspirator, ConspiratorRole,
    RingBehavior, RingStatus,
};
pub use management_override::{
    ManagementConcealment, ManagementOverrideScheme, OverrideType, RevenueOverrideTechnique,
};
pub use red_flags::{RedFlag, RedFlagGenerator, RedFlagPattern, RedFlagStrength};
