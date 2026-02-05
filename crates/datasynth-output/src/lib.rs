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

pub use control_export::*;
pub use csv_sink::*;
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
mod test_helpers;
