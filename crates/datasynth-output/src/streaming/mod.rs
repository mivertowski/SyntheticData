//! Streaming output sinks for real-time data generation.
//!
//! This module provides file-based streaming sinks that implement the
//! `StreamingSink` trait for CSV, JSON, and Parquet output with backpressure support.

mod csv_sink;
mod json_sink;
mod parquet_sink;

pub use csv_sink::CsvStreamingSink;
pub use json_sink::{JsonStreamingSink, NdjsonStreamingSink};
pub use parquet_sink::ParquetStreamingSink;
