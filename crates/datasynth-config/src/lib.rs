#![deny(clippy::unwrap_used)]
//! # synth-config
//!
//! Configuration schema, validation, and presets for synthetic data generation.

pub mod env_interpolation;
pub mod fraud_packs;
pub mod presets;
pub mod schema;
pub mod validation;

pub use schema::*;
pub use validation::*;
