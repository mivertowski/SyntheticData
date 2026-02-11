//! Statistics extractor.

use std::collections::HashMap;

use crate::error::FingerprintResult;
use crate::models::{
    CategoricalStats, CategoryFrequency, DistributionParams, DistributionType, NumericStats,
    Percentiles, StatisticsFingerprint,
};
use crate::privacy::PrivacyEngine;

use super::{DataSource, ExtractedComponent, ExtractionConfig, Extractor};

/// Extractor for statistical information.
pub struct StatsExtractor;

impl Extractor for StatsExtractor {
    fn name(&self) -> &'static str {
        "statistics"
    }

    fn extract(
        &self,
        data: &DataSource,
        config: &ExtractionConfig,
        privacy: &mut PrivacyEngine,
    ) -> FingerprintResult<ExtractedComponent> {
        let stats = match data {
            DataSource::Csv(csv) => extract_from_csv(csv, config, privacy)?,
            DataSource::Parquet(pq) => extract_from_parquet(pq, config, privacy)?,
            DataSource::Json(json) => extract_from_json(json, config, privacy)?,
            DataSource::Memory(mem) => extract_from_memory(mem, config, privacy)?,
            DataSource::Directory(_) => {
                // Directory sources are handled by FingerprintExtractor::extract_from_directory_impl
                return Err(crate::error::FingerprintError::extraction(
                    "statistics",
                    "Directory sources should be handled at the FingerprintExtractor level",
                ));
            }
        };

        Ok(ExtractedComponent::Statistics(stats))
    }
}

/// Extract statistics from CSV.
fn extract_from_csv(
    csv: &super::CsvDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<StatisticsFingerprint> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(csv.has_headers)
        .delimiter(csv.delimiter)
        .from_path(&csv.path)?;

    let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();

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

    extract_column_stats(&headers, &columns, table_name, config, privacy)
}

/// Extract statistics from memory.
fn extract_from_memory(
    mem: &super::MemoryDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<StatisticsFingerprint> {
    // Transpose rows to columns
    let mut columns: Vec<Vec<String>> = vec![Vec::new(); mem.columns.len()];

    for row in &mem.rows {
        for (i, value) in row.iter().enumerate() {
            if i < columns.len() {
                columns[i].push(value.clone());
            }
        }
    }

    extract_column_stats(&mem.columns, &columns, "memory", config, privacy)
}

/// Extract statistics from Parquet file.
fn extract_from_parquet(
    pq: &super::ParquetDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<StatisticsFingerprint> {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use std::fs::File;

    let file = File::open(&pq.path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let schema = builder.schema().clone();
    let reader = builder.with_batch_size(10000).build()?;

    // Collect column names
    let headers: Vec<String> = schema.fields().iter().map(|f| f.name().clone()).collect();
    let mut columns: Vec<Vec<String>> = vec![Vec::new(); headers.len()];

    // Read batches
    for batch_result in reader {
        let batch = batch_result?;
        for (i, _field) in schema.fields().iter().enumerate() {
            let column = batch.column(i);
            let values = super::schema_extractor::arrow_column_to_strings(column.as_ref());
            columns[i].extend(values);
        }
    }

    let table_name = pq
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data");

    extract_column_stats(&headers, &columns, table_name, config, privacy)
}

/// Extract statistics from JSON/JSONL file.
fn extract_from_json(
    json: &super::JsonDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<StatisticsFingerprint> {
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(&json.path)?;
    let reader = BufReader::new(file);

    let mut rows: Vec<HashMap<String, serde_json::Value>> = Vec::new();

    if json.is_array {
        // JSON array format
        let content = std::fs::read_to_string(&json.path)?;
        let array: Vec<serde_json::Value> = serde_json::from_str(&content)?;

        for value in array {
            if let serde_json::Value::Object(obj) = value {
                rows.push(obj.into_iter().collect());
            }
        }
    } else {
        // JSONL format
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(serde_json::Value::Object(obj)) = serde_json::from_str(&line) {
                rows.push(obj.into_iter().collect());
            }
        }
    }

    // Collect all column names
    let mut all_columns: HashSet<String> = HashSet::new();
    for row in &rows {
        for key in row.keys() {
            all_columns.insert(key.clone());
        }
    }

    // Sort columns for consistency
    let mut headers: Vec<String> = all_columns.into_iter().collect();
    headers.sort();

    // Build columns
    let mut columns: Vec<Vec<String>> = vec![Vec::new(); headers.len()];
    for row in &rows {
        for (i, header) in headers.iter().enumerate() {
            let value = row
                .get(header)
                .map(super::schema_extractor::json_value_to_string)
                .unwrap_or_default();
            columns[i].push(value);
        }
    }

    let table_name = json
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data");

    extract_column_stats(&headers, &columns, table_name, config, privacy)
}

/// Extract statistics for all columns.
fn extract_column_stats(
    headers: &[String],
    columns: &[Vec<String>],
    table_name: &str,
    _config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<StatisticsFingerprint> {
    let mut stats = StatisticsFingerprint::new();

    for (i, header) in headers.iter().enumerate() {
        let values = &columns[i];

        // Try to parse as numeric
        let numeric_values: Vec<f64> = values
            .iter()
            .filter_map(|v| v.parse::<f64>().ok())
            .collect();

        if numeric_values.len() > values.len() / 2 {
            // Treat as numeric
            let target = format!("{}.{}", table_name, header);
            let numeric_stats = compute_numeric_stats(&numeric_values, &target, privacy)?;
            stats.add_numeric(table_name, header, numeric_stats);
        } else {
            // Treat as categorical
            let target = format!("{}.{}", table_name, header);
            let cat_stats = compute_categorical_stats(values, &target, privacy)?;
            stats.add_categorical(table_name, header, cat_stats);
        }
    }

    // Compute global Benford analysis for numeric columns
    let all_amounts: Vec<f64> = stats
        .numeric_columns
        .values()
        .flat_map(|s| vec![s.mean]) // Simplified - would use actual values in production
        .filter(|v| *v > 0.0)
        .collect();

    if all_amounts.len() >= 100 {
        // Would compute actual Benford stats from raw values
        // For now, placeholder
    }

    Ok(stats)
}

/// Compute numeric statistics.
fn compute_numeric_stats(
    values: &[f64],
    target: &str,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<NumericStats> {
    if values.is_empty() {
        return Ok(NumericStats::new(0, 0.0, 0.0, 0.0, 0.0));
    }

    let count = values.len() as u64;
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    // Winsorize before computing stats
    privacy.winsorize(&mut sorted, target);

    let min = sorted.first().copied().unwrap_or(0.0);
    let max = sorted.last().copied().unwrap_or(0.0);
    let sum: f64 = sorted.iter().sum();
    let mean = sum / sorted.len() as f64;

    let variance: f64 =
        sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / sorted.len() as f64;
    let std_dev = variance.sqrt();

    // Add noise to statistics
    let noised_mean = privacy.add_noise(mean, max - min, &format!("{}.mean", target))?;
    let noised_std_dev =
        privacy.add_noise(std_dev, (max - min) / 2.0, &format!("{}.std_dev", target))?;

    // Compute percentiles
    let percentiles = compute_percentiles(&sorted);

    // Fit distribution
    let (distribution, params) = fit_distribution(&sorted, mean, std_dev);

    // Zero and negative rates
    let zero_rate = sorted.iter().filter(|v| **v == 0.0).count() as f64 / count as f64;
    let negative_rate = sorted.iter().filter(|v| **v < 0.0).count() as f64 / count as f64;

    // Benford first digit
    let benford = compute_benford_first_digit(&sorted);

    Ok(NumericStats {
        count,
        min,
        max,
        mean: noised_mean,
        std_dev: noised_std_dev.abs(),
        percentiles,
        distribution,
        distribution_params: params,
        zero_rate,
        negative_rate,
        benford_first_digit: Some(benford),
    })
}

/// Compute percentiles from sorted values.
fn compute_percentiles(sorted: &[f64]) -> Percentiles {
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    Percentiles {
        p1: percentile(sorted, 1.0),
        p5: percentile(sorted, 5.0),
        p10: percentile(sorted, 10.0),
        p25: percentile(sorted, 25.0),
        p50: percentile(sorted, 50.0),
        p75: percentile(sorted, 75.0),
        p90: percentile(sorted, 90.0),
        p95: percentile(sorted, 95.0),
        p99: percentile(sorted, 99.0),
    }
}

/// Fit a distribution to the data.
fn fit_distribution(
    sorted: &[f64],
    mean: f64,
    std_dev: f64,
) -> (DistributionType, DistributionParams) {
    // Simple heuristic-based fitting

    // Check for uniform
    let range = sorted.last().unwrap_or(&0.0) - sorted.first().unwrap_or(&0.0);
    let expected_std_uniform = range / (12.0_f64).sqrt();
    if (std_dev - expected_std_uniform).abs() / expected_std_uniform < 0.1 {
        return (
            DistributionType::Uniform,
            DistributionParams::uniform(
                *sorted.first().unwrap_or(&0.0),
                *sorted.last().unwrap_or(&1.0),
            ),
        );
    }

    // Check for log-normal (skewed, positive values)
    let all_positive = sorted.iter().all(|v| *v > 0.0);
    let skewness = compute_skewness(sorted, mean, std_dev);

    if all_positive && skewness > 0.5 {
        // Fit log-normal
        let log_values: Vec<f64> = sorted.iter().map(|v| v.ln()).collect();
        let log_mean: f64 = log_values.iter().sum::<f64>() / log_values.len() as f64;
        let log_var: f64 = log_values
            .iter()
            .map(|v| (v - log_mean).powi(2))
            .sum::<f64>()
            / log_values.len() as f64;
        let log_std = log_var.sqrt();

        return (
            DistributionType::LogNormal,
            DistributionParams::log_normal(log_mean, log_std),
        );
    }

    // Default to normal
    (
        DistributionType::Normal,
        DistributionParams::normal(mean, std_dev),
    )
}

/// Compute skewness.
fn compute_skewness(values: &[f64], mean: f64, std_dev: f64) -> f64 {
    if std_dev == 0.0 || values.is_empty() {
        return 0.0;
    }

    let n = values.len() as f64;
    let m3: f64 = values.iter().map(|v| (v - mean).powi(3)).sum::<f64>() / n;
    m3 / std_dev.powi(3)
}

/// Compute Benford first digit distribution.
fn compute_benford_first_digit(values: &[f64]) -> [f64; 9] {
    let mut counts = [0u64; 9];
    let mut total = 0u64;

    for v in values {
        let abs_v = v.abs();
        if abs_v > 0.0 {
            let s = format!("{:.15}", abs_v);
            for c in s.chars() {
                if c.is_ascii_digit() && c != '0' {
                    if let Some(digit) = c.to_digit(10) {
                        let digit = digit as usize;
                        if (1..=9).contains(&digit) {
                            counts[digit - 1] += 1;
                            total += 1;
                        }
                    }
                    break;
                }
            }
        }
    }

    if total == 0 {
        return [0.0; 9];
    }

    let mut freqs = [0.0; 9];
    for i in 0..9 {
        freqs[i] = counts[i] as f64 / total as f64;
    }
    freqs
}

/// Compute categorical statistics.
fn compute_categorical_stats(
    values: &[String],
    target: &str,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<CategoricalStats> {
    let non_empty: Vec<_> = values.iter().filter(|v| !v.is_empty()).collect();
    let count = non_empty.len() as u64;

    if count == 0 {
        return Ok(CategoricalStats::new(0, 0));
    }

    // Count frequencies
    let mut freq_map: HashMap<&String, u64> = HashMap::new();
    for v in &non_empty {
        *freq_map.entry(v).or_default() += 1;
    }

    let cardinality = freq_map.len() as u64;

    // Convert to list for privacy filtering
    let frequencies: Vec<(String, u64)> =
        freq_map.into_iter().map(|(k, v)| (k.clone(), v)).collect();

    // Apply k-anonymity filtering
    let filtered = privacy.filter_categories(frequencies, count, target);

    // Convert to CategoryFrequency
    let top_values: Vec<CategoryFrequency> = filtered
        .into_iter()
        .map(|(value, freq)| CategoryFrequency::new(value, freq))
        .take(100) // Limit to top 100
        .collect();

    // Compute entropy
    let entropy = compute_entropy(&top_values);

    Ok(CategoricalStats {
        count,
        cardinality,
        top_values,
        rare_values_suppressed: true, // Privacy filtering applied
        suppressed_count: 0,          // Would be computed from filtering
        entropy,
    })
}

/// Compute entropy of a distribution.
fn compute_entropy(frequencies: &[CategoryFrequency]) -> f64 {
    let mut entropy = 0.0;
    for freq in frequencies {
        if freq.frequency > 0.0 {
            entropy -= freq.frequency * freq.frequency.ln();
        }
    }
    entropy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benford_first_digit() {
        let values = vec![123.0, 456.0, 789.0, 100.0, 200.0, 300.0];
        let benford = compute_benford_first_digit(&values);

        // Should have counts for digits 1, 2, 3, 4, 7
        assert!(benford[0] > 0.0); // digit 1
        assert!(benford[1] > 0.0); // digit 2
        assert!(benford[2] > 0.0); // digit 3
    }
}
