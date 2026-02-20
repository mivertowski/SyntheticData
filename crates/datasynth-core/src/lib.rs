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
pub mod compliance;
pub mod country;
pub mod cpu_monitor;
pub mod degradation;
pub mod diffusion;
pub mod disk_guard;
pub mod distributions;
pub mod error;
pub mod llm;
pub mod memory_guard;
pub mod models;
pub mod pcg;
pub mod pcg_loader;
pub mod plugins;
pub mod rate_limit;
pub mod resource_guard;
pub mod streaming;
pub mod templates;
pub mod traits;
pub mod utils;
pub mod uuid_factory;

// -- Explicit re-exports for commonly used infrastructure types --

pub use country::{CountryCode, CountryPack, CountryPackError, CountryPackRegistry};

pub use cpu_monitor::{CpuMonitor, CpuMonitorConfig, CpuOverloaded, CpuStats};

pub use degradation::{
    DegradationActions, DegradationConfig, DegradationController, DegradationLevel, ResourceStatus,
};

pub use disk_guard::{
    check_sufficient_disk_space, estimate_output_size_mb, get_available_space_mb, get_disk_space,
    DiskSpaceExhausted, DiskSpaceGuard, DiskSpaceGuardConfig, DiskStats, OutputFormat,
};

pub use error::{SynthError, SynthResult};

pub use memory_guard::{
    check_sufficient_memory, estimate_memory_mb, get_memory_usage_mb, MemoryGuard,
    MemoryGuardConfig, MemoryLimitExceeded, MemoryStats,
};

pub use resource_guard::{
    PreCheckResult, ResourceGuard, ResourceGuardBuilder, ResourceGuardConfig, ResourceStats,
};

pub use uuid_factory::{DeterministicUuidFactory, GeneratorType, UuidFactoryRegistry};

// -- Glob re-exports for large, widely-consumed modules --
// These modules expose many types that are used broadly across the workspace.
// Converting them to explicit re-exports would be high-risk with limited benefit.

pub use distributions::*;
pub use models::*;
pub use rate_limit::*;
pub use streaming::*;
pub use templates::*;
pub use traits::*;
