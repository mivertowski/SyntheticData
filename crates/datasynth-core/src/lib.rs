#![deny(clippy::unwrap_used)]

//! # synth-core
//!
//! Core domain models, traits, and distributions for synthetic accounting data generation.
//!
//! This crate provides the foundational types used throughout the synthetic data factory:
//! - Journal Entry models (header and line items)
//! - Chart of Accounts structures
//! - SAP HANA ACDOCA/BSEG compatible event log formats
//! - Generator and Sink traits for extensibility
//! - Statistical distribution samplers based on empirical research
//! - Templates for realistic data generation (names, descriptions, references)
//! - Resource management (memory, disk, CPU) with graceful degradation
//! - Streaming infrastructure for real-time data generation

pub mod accounts;
pub mod causal;
pub mod pcg;
pub mod pcg_loader;
pub mod compliance;
pub mod cpu_monitor;
pub mod degradation;
pub mod diffusion;
pub mod disk_guard;
pub mod distributions;
pub mod error;
pub mod llm;
pub mod memory_guard;
pub mod models;
pub mod plugins;
pub mod rate_limit;
pub mod resource_guard;
pub mod streaming;
pub mod templates;
pub mod traits;
pub mod uuid_factory;

pub use cpu_monitor::*;
pub use degradation::*;
pub use disk_guard::*;
pub use distributions::*;
pub use error::*;
pub use memory_guard::*;
pub use models::*;
pub use rate_limit::*;
pub use resource_guard::*;
pub use streaming::*;
pub use templates::*;
pub use traits::*;
pub use uuid_factory::*;
