//! Error types for country pack loading and validation.

use thiserror::Error;

/// Errors that can occur when working with country packs.
#[derive(Error, Debug)]
pub enum CountryPackError {
    /// Invalid or unrecognized country code.
    #[error("Invalid country code: {0}")]
    InvalidCountryCode(String),

    /// Failed to parse a country pack JSON file.
    #[error("Failed to parse country pack: {0}")]
    ParseError(String),

    /// Error during deep-merge of pack overrides.
    #[error("Merge error: {0}")]
    MergeError(String),

    /// Error accessing external pack directory.
    #[error("Directory error: {0}")]
    DirectoryError(String),

    /// Schema version mismatch between packs.
    #[error("Schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: String, found: String },
}

impl CountryPackError {
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    pub fn merge(msg: impl Into<String>) -> Self {
        Self::MergeError(msg.into())
    }

    pub fn directory(msg: impl Into<String>) -> Self {
        Self::DirectoryError(msg.into())
    }
}
