//! Integrity constraint extractor.

use crate::error::FingerprintResult;
use crate::models::{IntegrityFingerprint, UniqueConstraint};
use crate::privacy::PrivacyEngine;

use super::{DataSource, ExtractedComponent, ExtractionConfig, Extractor};

/// Extractor for integrity constraints.
pub struct IntegrityExtractor;

impl Extractor for IntegrityExtractor {
    fn name(&self) -> &'static str {
        "integrity"
    }

    fn extract(
        &self,
        data: &DataSource,
        config: &ExtractionConfig,
        privacy: &mut PrivacyEngine,
    ) -> FingerprintResult<ExtractedComponent> {
        let integrity = match data {
            DataSource::Csv(csv) => extract_from_csv(csv, config, privacy)?,
            DataSource::Parquet(_) | DataSource::Json(_) => {
                // For Parquet and JSON, return empty integrity (can be extended later)
                IntegrityFingerprint::new()
            }
            DataSource::Memory(mem) => extract_from_memory(mem, config, privacy)?,
            DataSource::Directory(_) => {
                // Directory sources are handled by FingerprintExtractor::extract_from_directory_impl
                return Err(crate::error::FingerprintError::extraction(
                    "integrity",
                    "Directory sources should be handled at the FingerprintExtractor level",
                ));
            }
        };

        Ok(ExtractedComponent::Integrity(integrity))
    }
}

/// Extract integrity from CSV.
fn extract_from_csv(
    csv: &super::CsvDataSource,
    _config: &ExtractionConfig,
    _privacy: &mut PrivacyEngine,
) -> FingerprintResult<IntegrityFingerprint> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(csv.has_headers)
        .delimiter(csv.delimiter)
        .from_path(&csv.path)?;

    let headers: Vec<String> = reader
        .headers()?
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    // Collect all values by column
    let mut columns: Vec<Vec<String>> = vec![Vec::new(); headers.len()];

    for result in reader.records() {
        let record = result?;
        for (i, field) in record.iter().enumerate() {
            if i < columns.len() {
                columns[i].push(field.to_string());
            }
        }
    }

    let table_name = csv
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data");

    let constraints = detect_unique_constraints(&headers, &columns, table_name);

    let mut integrity = IntegrityFingerprint::new();
    for constraint in constraints {
        integrity.unique_constraints.push(constraint);
    }

    Ok(integrity)
}

/// Extract integrity from memory.
fn extract_from_memory(
    mem: &super::MemoryDataSource,
    _config: &ExtractionConfig,
    _privacy: &mut PrivacyEngine,
) -> FingerprintResult<IntegrityFingerprint> {
    // Transpose rows to columns
    let mut columns: Vec<Vec<String>> = vec![Vec::new(); mem.columns.len()];

    for row in &mem.rows {
        for (i, value) in row.iter().enumerate() {
            if i < columns.len() {
                columns[i].push(value.clone());
            }
        }
    }

    let constraints = detect_unique_constraints(&mem.columns, &columns, "memory");

    let mut integrity = IntegrityFingerprint::new();
    for constraint in constraints {
        integrity.unique_constraints.push(constraint);
    }

    Ok(integrity)
}

/// Detect unique constraints.
fn detect_unique_constraints(
    headers: &[String],
    columns: &[Vec<String>],
    table_name: &str,
) -> Vec<UniqueConstraint> {
    let mut constraints = Vec::new();

    for (i, header) in headers.iter().enumerate() {
        let values = &columns[i];
        let unique_count = {
            let set: std::collections::HashSet<_> = values.iter().collect();
            set.len()
        };

        // Check if column is unique (or near-unique)
        if unique_count == values.len() && !values.is_empty() {
            let mut constraint = UniqueConstraint::new(table_name, vec![header.clone()]);
            constraint.is_satisfied = true;
            constraints.push(constraint);
        }
    }

    constraints
}
