//! Observability infrastructure for the DataSynth server.
//!
//! Provides metrics collection and optional OpenTelemetry integration.
//!
//! ## Feature flags
//!
//! - Default: Manual Prometheus text format metrics at `/metrics`
//! - `otel`: OpenTelemetry OTLP trace export + Prometheus metric bridge

pub mod metrics;

#[cfg(feature = "otel")]
pub mod otel;
