#![deny(clippy::unwrap_used)]
//! # synth-output
//!
//! Output sinks for CSV, Parquet, JSON, and streaming formats.
//! Also provides ERP-specific export formats for SAP, Oracle EBS, and NetSuite.

pub mod compressed;
pub mod control_export;
pub mod csv_sink;
pub mod esg_export;
pub mod fast_csv;
pub mod formats;
pub mod json_sink;
pub mod parquet_sink;
pub mod project_accounting_export;
pub mod streaming;
pub mod tax_export;
pub mod treasury_export;

pub use compressed::{CompressedWriter, CompressionConfig};
pub use control_export::*;
pub use csv_sink::*;
pub use esg_export::*;
pub use formats::{
    write_fec_csv, write_gobd_accounts_csv, write_gobd_index_xml, write_gobd_journal_csv,
    NetSuiteExporter, NetSuiteJournalEntry, NetSuiteJournalLine, OracleExporter, OracleJeHeader,
    OracleJeLine, SapExportConfig, SapExporter, SapTableType,
};
pub use json_sink::*;
pub use parquet_sink::*;
pub use project_accounting_export::*;
pub use streaming::{
    CsvStreamingSink, JsonStreamingSink, NdjsonStreamingSink, ParquetStreamingSink,
};
pub use tax_export::*;
pub use treasury_export::*;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test_helpers;
