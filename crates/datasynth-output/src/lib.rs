#![deny(clippy::unwrap_used)]
//! # synth-output
//!
//! Output sinks for CSV, Parquet, JSON, and streaming formats.
//! Also provides ERP-specific export formats for SAP, Oracle EBS, and NetSuite.

pub mod control_export;
pub mod csv_sink;
pub mod formats;
pub mod json_sink;
pub mod parquet_sink;
pub mod streaming;
pub mod tax_export;
pub mod treasury_export;

pub use control_export::*;
pub use csv_sink::*;
pub use tax_export::*;
pub use treasury_export::*;
pub use formats::{
    NetSuiteExporter, NetSuiteJournalEntry, NetSuiteJournalLine, OracleExporter, OracleJeHeader,
    OracleJeLine, SapExportConfig, SapExporter, SapTableType,
};
pub use json_sink::*;
pub use parquet_sink::*;
pub use streaming::{
    CsvStreamingSink, JsonStreamingSink, NdjsonStreamingSink, ParquetStreamingSink,
};

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test_helpers;
