//! Quality gate engine for pass/fail criteria on generation runs.
//!
//! Provides configurable threshold profiles (strict, default, lenient)
//! and a gate evaluation engine that checks generation quality metrics.

mod engine;
mod profiles;

pub use engine::{
    Comparison, FailStrategy, GateCheckResult, GateEngine, GateProfile, GateResult, QualityGate,
    QualityMetric,
};
pub use profiles::{default_profile, get_profile, lenient_profile, strict_profile};
