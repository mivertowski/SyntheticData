//! Correlated anomaly injection patterns.
//!
//! This module provides patterns for injecting correlated anomalies,
//! including co-occurrence patterns, temporal clustering, and error cascades.

mod cascade;
mod co_occurrence;
mod temporal_clustering;

pub use cascade::{CascadeConfig, CascadeGenerator, CascadeStep};
pub use co_occurrence::{AnomalyCoOccurrence, CorrelatedAnomaly, CoOccurrencePattern};
pub use temporal_clustering::{TemporalAnomalyCluster, TemporalClusterGenerator, TemporalWindow};
