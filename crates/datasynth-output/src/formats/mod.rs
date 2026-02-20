//! ERP-specific output format modules.
//!
//! This module provides export functionality for common ERP systems:
//! - SAP S/4HANA (BKPF, BSEG, ACDOCA tables)
//! - Oracle EBS (GL_JE_HEADERS, GL_JE_LINES)
//! - NetSuite (Journal entries with NetSuite-specific fields)
//! - FEC (Fichier des Écritures Comptables) for French GAAP

pub mod fec;
pub mod netsuite;
pub mod oracle;
pub mod sap;

pub use fec::write_fec_csv;
pub use netsuite::{NetSuiteExporter, NetSuiteJournalEntry, NetSuiteJournalLine};
pub use oracle::{OracleExporter, OracleJeHeader, OracleJeLine};
pub use sap::{SapExportConfig, SapExporter, SapTableType};
