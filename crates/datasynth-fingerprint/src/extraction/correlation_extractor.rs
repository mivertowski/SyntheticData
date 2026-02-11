//! Correlation extractor.

use crate::error::FingerprintResult;
use crate::models::{CorrelationFingerprint, CorrelationMatrix, CorrelationType};
use crate::privacy::PrivacyEngine;

use super::{DataSource, ExtractedComponent, ExtractionConfig, Extractor};

/// Extractor for correlation information.
pub struct CorrelationExtractor;

impl Extractor for CorrelationExtractor {
    fn name(&self) -> &'static str {
        "correlations"
    }

    fn extract(
        &self,
        data: &DataSource,
        config: &ExtractionConfig,
        privacy: &mut PrivacyEngine,
    ) -> FingerprintResult<ExtractedComponent> {
        let correlations = match data {
            DataSource::Csv(csv) => extract_from_csv(csv, config, privacy)?,
            DataSource::Parquet(_) | DataSource::Json(_) => {
                // For Parquet and JSON, reuse the same logic via memory conversion
                // For now, return empty correlations (can be extended later)
                CorrelationFingerprint::new()
            }
            DataSource::Memory(mem) => extract_from_memory(mem, config, privacy)?,
            DataSource::Directory(_) => {
                // Directory sources are handled by FingerprintExtractor::extract_from_directory_impl
                return Err(crate::error::FingerprintError::extraction(
                    "correlations",
                    "Directory sources should be handled at the FingerprintExtractor level",
                ));
            }
        };

        Ok(ExtractedComponent::Correlations(correlations))
    }
}

/// Extract correlations from CSV.
fn extract_from_csv(
    csv: &super::CsvDataSource,
    _config: &ExtractionConfig,
    _privacy: &mut PrivacyEngine,
) -> FingerprintResult<CorrelationFingerprint> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(csv.has_headers)
        .delimiter(csv.delimiter)
        .from_path(&csv.path)?;

    let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();

    // Collect numeric columns
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); headers.len()];
    let mut is_numeric: Vec<bool> = vec![true; headers.len()];

    for result in reader.records() {
        let record = result?;
        for (i, field) in record.iter().enumerate() {
            if i < columns.len() {
                if let Ok(v) = field.parse::<f64>() {
                    columns[i].push(v);
                } else {
                    is_numeric[i] = false;
                }
            }
        }
    }

    // Filter to numeric columns
    let numeric_cols: Vec<(String, Vec<f64>)> = headers
        .into_iter()
        .zip(columns)
        .zip(is_numeric)
        .filter_map(|((name, values), is_num)| {
            if is_num && !values.is_empty() {
                Some((name, values))
            } else {
                None
            }
        })
        .collect();

    let table_name = csv
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data");

    let matrix = compute_correlation_matrix(&numeric_cols);

    let mut correlations = CorrelationFingerprint::new();
    correlations.add_matrix(table_name, matrix);

    Ok(correlations)
}

/// Extract correlations from memory.
fn extract_from_memory(
    mem: &super::MemoryDataSource,
    _config: &ExtractionConfig,
    _privacy: &mut PrivacyEngine,
) -> FingerprintResult<CorrelationFingerprint> {
    // Transpose rows to columns and filter numeric
    let mut numeric_cols: Vec<(String, Vec<f64>)> = Vec::new();

    for (i, col_name) in mem.columns.iter().enumerate() {
        let values: Vec<f64> = mem
            .rows
            .iter()
            .filter_map(|row| row.get(i).and_then(|v| v.parse().ok()))
            .collect();

        if values.len() > mem.rows.len() / 2 {
            numeric_cols.push((col_name.clone(), values));
        }
    }

    let matrix = compute_correlation_matrix(&numeric_cols);

    let mut correlations = CorrelationFingerprint::new();
    correlations.add_matrix("memory", matrix);

    Ok(correlations)
}

/// Compute Pearson correlation matrix.
fn compute_correlation_matrix(columns: &[(String, Vec<f64>)]) -> CorrelationMatrix {
    let names: Vec<String> = columns.iter().map(|(n, _)| n.clone()).collect();
    let n_cols = names.len();

    if n_cols == 0 {
        return CorrelationMatrix::new(Vec::new(), CorrelationType::Pearson);
    }

    // Find minimum length for alignment
    let min_len = columns.iter().map(|(_, v)| v.len()).min().unwrap_or(0);

    // Build full correlation matrix
    let mut full_matrix = vec![vec![0.0; n_cols]; n_cols];

    for i in 0..n_cols {
        full_matrix[i][i] = 1.0;
        for j in (i + 1)..n_cols {
            let corr = pearson_correlation(&columns[i].1, &columns[j].1, min_len);
            full_matrix[i][j] = corr;
            full_matrix[j][i] = corr;
        }
    }

    let mut matrix =
        CorrelationMatrix::from_full_matrix(names, &full_matrix, CorrelationType::Pearson);
    matrix.sample_size = min_len as u64;
    matrix
}

/// Compute Pearson correlation coefficient.
fn pearson_correlation(x: &[f64], y: &[f64], n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }

    let n = n.min(x.len()).min(y.len());
    let x = &x[..n];
    let y = &y[..n];

    let mean_x: f64 = x.iter().sum::<f64>() / n as f64;
    let mean_y: f64 = y.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return 0.0;
    }

    cov / (var_x.sqrt() * var_y.sqrt())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_pearson_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = pearson_correlation(&x, &y, 5);
        assert!((corr - 1.0).abs() < 0.001); // Perfect positive correlation

        let z = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let corr_neg = pearson_correlation(&x, &z, 5);
        assert!((corr_neg + 1.0).abs() < 0.001); // Perfect negative correlation
    }
}
