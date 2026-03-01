#![deny(clippy::unwrap_used)]
//! # synth-runtime
//!
//! Runtime orchestration, parallel execution, and memory management.
//!
//! This crate provides orchestrators:
//! - `GenerationOrchestrator`: Basic orchestrator for CoA and journal entries
//! - `EnhancedOrchestrator`: Full-featured orchestrator with all phases
//! - `StreamingOrchestrator`: Streaming orchestrator for real-time generation
//!
//! And support modules for:
//! - `run_manifest`: Run metadata and reproducibility tracking
//! - `label_export`: Anomaly label export to CSV/JSON formats

pub mod causal_engine;
pub mod enhanced_orchestrator;
pub mod label_export;
pub mod lineage;
pub mod orchestrator;
pub mod prov;
pub mod run_manifest;
#[cfg(feature = "streaming")]
pub mod stream_client;
pub mod streaming_orchestrator;
pub mod webhooks;

pub use enhanced_orchestrator::*;
pub use label_export::*;
pub use orchestrator::*;
pub use run_manifest::*;
pub use streaming_orchestrator::*;
