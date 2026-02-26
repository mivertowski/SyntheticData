//! ERP-specific output format modules.
//!
//! This module provides export functionality for common ERP systems:
//! - SAP S/4HANA (BKPF, BSEG, ACDOCA tables)
//! - Oracle EBS (GL_JE_HEADERS, GL_JE_LINES)
//! - NetSuite (Journal entries with NetSuite-specific fields)
//! - FEC (Fichier des Écritures Comptables) for French GAAP
//! - GoBD (Grundsätze zur ordnungsmäßigen Führung und Aufbewahrung) for German GAAP

pub mod fec;
pub mod gobd;
pub mod netsuite;
pub mod oracle;
pub mod sap;

pub use fec::write_fec_csv;
pub use gobd::{write_gobd_accounts_csv, write_gobd_index_xml, write_gobd_journal_csv};
pub use netsuite::{NetSuiteExporter, NetSuiteJournalEntry, NetSuiteJournalLine};
pub use oracle::{OracleExporter, OracleJeHeader, OracleJeLine};
pub use sap::{SapExportConfig, SapExporter, SapTableType};
