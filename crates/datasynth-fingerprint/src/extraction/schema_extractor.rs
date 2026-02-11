//! Schema extractor.

use arrow::array::Array;

use crate::error::{FingerprintError, FingerprintResult};
use crate::models::{DataType, FieldSchema, SchemaFingerprint, TableSchema};
use crate::privacy::PrivacyEngine;

use super::{DataSource, ExtractedComponent, ExtractionConfig, Extractor};

/// Extractor for schema information.
pub struct SchemaExtractor;

impl Extractor for SchemaExtractor {
    fn name(&self) -> &'static str {
        "schema"
    }

    fn extract(
        &self,
        data: &DataSource,
        config: &ExtractionConfig,
        privacy: &mut PrivacyEngine,
    ) -> FingerprintResult<ExtractedComponent> {
        let schema = match data {
            DataSource::Csv(csv) => extract_from_csv(csv, config, privacy)?,
            DataSource::Parquet(pq) => extract_from_parquet(pq, config, privacy)?,
            DataSource::Json(json) => extract_from_json(json, config, privacy)?,
            DataSource::Memory(mem) => extract_from_memory(mem, config, privacy)?,
            DataSource::Directory(_) => {
                // Directory sources are handled by FingerprintExtractor::extract_from_directory_impl
                return Err(crate::error::FingerprintError::extraction(
                    "schema",
                    "Directory sources should be handled at the FingerprintExtractor level",
                ));
            }
        };

        Ok(ExtractedComponent::Schema(schema))
    }
}

/// Extract schema from CSV.
fn extract_from_csv(
    csv: &super::CsvDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<SchemaFingerprint> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(csv.has_headers)
        .delimiter(csv.delimiter)
        .from_path(&csv.path)?;

    let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();

    // Sample rows to infer types
    let mut sample_rows: Vec<Vec<String>> = Vec::new();
    let mut row_count = 0u64;

    for result in reader.records() {
        let record = result?;
        row_count += 1;

        if sample_rows.len() < 1000 {
            sample_rows.push(record.iter().map(|s| s.to_string()).collect());
        }
    }

    // Check minimum rows
    if row_count < config.min_rows as u64 {
        return Err(FingerprintError::InsufficientData {
            required: config.min_rows,
            actual: row_count as usize,
        });
    }

    // Infer column types
    let columns: Vec<FieldSchema> = headers
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let values: Vec<&str> = sample_rows
                .iter()
                .filter_map(|row| row.get(i).map(|s| s.as_str()))
                .collect();

            let data_type = infer_data_type(&values);
            let null_rate =
                values.iter().filter(|v| v.is_empty()).count() as f64 / values.len() as f64;
            let cardinality = estimate_cardinality(&values);

            FieldSchema::new(name.clone(), data_type)
                .with_nullable(null_rate)
                .with_cardinality(cardinality)
        })
        .collect();

    // Add noise to row count
    let noised_row_count = privacy.add_noise_to_count(row_count, "schema.row_count")?;

    let table_name = csv
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data")
        .to_string();

    let mut table = TableSchema::new(&table_name, noised_row_count);
    for col in columns {
        table.add_column(col);
    }

    let mut schema = SchemaFingerprint::new();
    schema.add_table(table_name, table);

    Ok(schema)
}

/// Extract schema from Parquet file.
fn extract_from_parquet(
    pq: &super::ParquetDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<SchemaFingerprint> {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use std::fs::File;

    let file = File::open(&pq.path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let parquet_schema = builder.schema().clone();
    let row_count: u64 = builder.metadata().file_metadata().num_rows() as u64;

    // Check minimum rows
    if row_count < config.min_rows as u64 {
        return Err(FingerprintError::InsufficientData {
            required: config.min_rows,
            actual: row_count as usize,
        });
    }

    // Build reader for sampling
    let reader = builder.with_batch_size(1000).build()?;

    // Convert Arrow schema to our schema
    let mut columns: Vec<FieldSchema> = Vec::new();
    let mut sample_data: Vec<Vec<String>> = Vec::new();

    // Read sample batches for cardinality estimation
    let mut batches_read = 0;
    for batch_result in reader {
        if batches_read >= 10 {
            break; // Limit sampling
        }
        let batch = batch_result?;

        for (i, _field) in parquet_schema.fields().iter().enumerate() {
            let column = batch.column(i);
            let values = arrow_column_to_strings(column);

            if sample_data.len() <= i {
                sample_data.push(Vec::new());
            }
            sample_data[i].extend(values);
        }

        batches_read += 1;
    }

    // Build field schemas
    for (i, field) in parquet_schema.fields().iter().enumerate() {
        let data_type = arrow_type_to_data_type(field.data_type());
        let null_rate = if sample_data.len() > i {
            let values = &sample_data[i];
            values.iter().filter(|v| v.is_empty()).count() as f64 / values.len().max(1) as f64
        } else {
            0.0
        };
        let cardinality = if sample_data.len() > i {
            let values: Vec<&str> = sample_data[i].iter().map(|s| s.as_str()).collect();
            estimate_cardinality(&values)
        } else {
            0
        };

        columns.push(
            FieldSchema::new(field.name().to_string(), data_type)
                .with_nullable(null_rate)
                .with_cardinality(cardinality),
        );
    }

    // Add noise to row count
    let noised_row_count = privacy.add_noise_to_count(row_count, "schema.row_count")?;

    let table_name = pq
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data")
        .to_string();

    let mut table = TableSchema::new(&table_name, noised_row_count);
    for col in columns {
        table.add_column(col);
    }

    let mut schema = SchemaFingerprint::new();
    schema.add_table(table_name, table);

    Ok(schema)
}

/// Convert Arrow data type to our DataType.
fn arrow_type_to_data_type(arrow_type: &arrow::datatypes::DataType) -> DataType {
    use arrow::datatypes::DataType as ArrowType;

    match arrow_type {
        ArrowType::Boolean => DataType::Boolean,
        ArrowType::Int8 | ArrowType::Int16 | ArrowType::Int32 | ArrowType::Int64 => DataType::Int64,
        ArrowType::UInt8 | ArrowType::UInt16 | ArrowType::UInt32 | ArrowType::UInt64 => {
            DataType::Int64
        }
        ArrowType::Float16 | ArrowType::Float32 | ArrowType::Float64 => DataType::Float64,
        ArrowType::Decimal128(_, _) | ArrowType::Decimal256(_, _) => DataType::Decimal,
        ArrowType::Date32 | ArrowType::Date64 => DataType::Date,
        ArrowType::Timestamp(_, _) | ArrowType::Time32(_) | ArrowType::Time64(_) => {
            DataType::Timestamp
        }
        ArrowType::Utf8 | ArrowType::LargeUtf8 => DataType::String,
        ArrowType::Binary | ArrowType::LargeBinary => DataType::String,
        _ => DataType::String,
    }
}

/// Convert Arrow column to string values for sampling.
pub fn arrow_column_to_strings(column: &dyn Array) -> Vec<String> {
    use arrow::array::*;

    let mut values = Vec::with_capacity(column.len());

    for i in 0..column.len() {
        if column.is_null(i) {
            values.push(String::new());
            continue;
        }

        // Try to downcast to common types
        if let Some(arr) = column.as_any().downcast_ref::<StringArray>() {
            values.push(arr.value(i).to_string());
        } else if let Some(arr) = column.as_any().downcast_ref::<Int64Array>() {
            values.push(arr.value(i).to_string());
        } else if let Some(arr) = column.as_any().downcast_ref::<Float64Array>() {
            values.push(arr.value(i).to_string());
        } else if let Some(arr) = column.as_any().downcast_ref::<BooleanArray>() {
            values.push(arr.value(i).to_string());
        } else {
            values.push(format!("{:?}", column.slice(i, 1)));
        }
    }

    values
}

/// Extract schema from JSON/JSONL file.
fn extract_from_json(
    json: &super::JsonDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<SchemaFingerprint> {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(&json.path)?;
    let reader = BufReader::new(file);

    let mut sample_rows: Vec<HashMap<String, serde_json::Value>> = Vec::new();

    if json.is_array {
        // JSON array format
        let content = std::fs::read_to_string(&json.path)?;
        let array: Vec<serde_json::Value> = serde_json::from_str(&content)?;

        for value in array.into_iter().take(10000) {
            if let serde_json::Value::Object(obj) = value {
                sample_rows.push(obj.into_iter().collect());
            }
        }
    } else {
        // JSONL format
        for line in reader.lines().take(10000) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(serde_json::Value::Object(obj)) = serde_json::from_str(&line) {
                sample_rows.push(obj.into_iter().collect());
            }
        }
    }

    let row_count = sample_rows.len() as u64;

    // Check minimum rows
    if row_count < config.min_rows as u64 {
        return Err(FingerprintError::InsufficientData {
            required: config.min_rows,
            actual: row_count as usize,
        });
    }

    // Collect all column names
    let mut all_columns: std::collections::HashSet<String> = std::collections::HashSet::new();
    for row in &sample_rows {
        for key in row.keys() {
            all_columns.insert(key.clone());
        }
    }

    // Build column schemas
    let mut columns: Vec<FieldSchema> = Vec::new();
    for column_name in &all_columns {
        let values: Vec<&serde_json::Value> = sample_rows
            .iter()
            .filter_map(|row| row.get(column_name))
            .collect();

        let data_type = infer_json_type(&values);
        let null_count = sample_rows.len() - values.len();
        let null_rate = null_count as f64 / sample_rows.len().max(1) as f64;

        let string_values: Vec<String> = values.iter().map(|v| json_value_to_string(v)).collect();
        let str_values: Vec<&str> = string_values.iter().map(|s| s.as_str()).collect();
        let cardinality = estimate_cardinality(&str_values);

        columns.push(
            FieldSchema::new(column_name.clone(), data_type)
                .with_nullable(null_rate)
                .with_cardinality(cardinality),
        );
    }

    // Sort columns alphabetically for consistency
    columns.sort_by(|a, b| a.name.cmp(&b.name));

    // Add noise to row count
    let noised_row_count = privacy.add_noise_to_count(row_count, "schema.row_count")?;

    let table_name = json
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("data")
        .to_string();

    let mut table = TableSchema::new(&table_name, noised_row_count);
    for col in columns {
        table.add_column(col);
    }

    let mut schema = SchemaFingerprint::new();
    schema.add_table(table_name, table);

    Ok(schema)
}

/// Infer data type from JSON values.
fn infer_json_type(values: &[&serde_json::Value]) -> DataType {
    if values.is_empty() {
        return DataType::String;
    }

    // Check types
    let mut has_bool = false;
    let mut has_int = false;
    let mut has_float = false;
    let mut has_string = false;

    for value in values {
        match value {
            serde_json::Value::Bool(_) => has_bool = true,
            serde_json::Value::Number(n) => {
                if n.is_f64() && n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) {
                    has_float = true;
                } else {
                    has_int = true;
                }
            }
            serde_json::Value::String(s) => {
                // Check if string looks like a date
                if s.len() == 10 && s.chars().nth(4) == Some('-') {
                    // Could be date, keep as string for now
                }
                has_string = true;
            }
            _ => has_string = true,
        }
    }

    if has_string {
        DataType::String
    } else if has_float {
        DataType::Float64
    } else if has_int && !has_bool {
        DataType::Int64
    } else if has_bool && !has_int {
        DataType::Boolean
    } else {
        DataType::String
    }
}

/// Convert JSON value to string for cardinality estimation.
pub fn json_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => value.to_string(),
    }
}

/// Extract schema from memory.
fn extract_from_memory(
    mem: &super::MemoryDataSource,
    config: &ExtractionConfig,
    privacy: &mut PrivacyEngine,
) -> FingerprintResult<SchemaFingerprint> {
    let row_count = mem.row_count() as u64;

    if row_count < config.min_rows as u64 {
        return Err(FingerprintError::InsufficientData {
            required: config.min_rows,
            actual: row_count as usize,
        });
    }

    let columns: Vec<FieldSchema> = mem
        .columns
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let values: Vec<&str> = mem
                .rows
                .iter()
                .filter_map(|row| row.get(i).map(|s| s.as_str()))
                .collect();

            let data_type = infer_data_type(&values);
            let null_rate =
                values.iter().filter(|v| v.is_empty()).count() as f64 / values.len().max(1) as f64;
            let cardinality = estimate_cardinality(&values);

            FieldSchema::new(name.clone(), data_type)
                .with_nullable(null_rate)
                .with_cardinality(cardinality)
        })
        .collect();

    let noised_row_count = privacy.add_noise_to_count(row_count, "schema.row_count")?;

    let mut table = TableSchema::new("memory", noised_row_count);
    for col in columns {
        table.add_column(col);
    }

    let mut schema = SchemaFingerprint::new();
    schema.add_table("memory", table);

    Ok(schema)
}

/// Infer data type from sample values.
fn infer_data_type(values: &[&str]) -> DataType {
    let non_empty: Vec<_> = values.iter().filter(|v| !v.is_empty()).collect();
    if non_empty.is_empty() {
        return DataType::String;
    }

    // Check for boolean
    let all_bool = non_empty.iter().all(|v| {
        let lower = v.to_lowercase();
        lower == "true" || lower == "false" || lower == "1" || lower == "0"
    });
    if all_bool {
        return DataType::Boolean;
    }

    // Check for integer
    let all_int = non_empty.iter().all(|v| v.parse::<i64>().is_ok());
    if all_int {
        return DataType::Int64;
    }

    // Check for decimal/float
    let all_float = non_empty.iter().all(|v| v.parse::<f64>().is_ok());
    if all_float {
        // Check if it looks like a decimal (has decimal point)
        let has_decimal = non_empty.iter().any(|v| v.contains('.'));
        return if has_decimal {
            DataType::Decimal
        } else {
            DataType::Float64
        };
    }

    // Check for date patterns
    let date_patterns = [
        r"^\d{4}-\d{2}-\d{2}$",   // YYYY-MM-DD
        r"^\d{2}/\d{2}/\d{4}$",   // MM/DD/YYYY
        r"^\d{2}\.\d{2}\.\d{4}$", // DD.MM.YYYY
    ];
    let all_date = non_empty.iter().all(|v| {
        date_patterns.iter().any(|p| {
            regex_lite::Regex::new(p)
                .map(|r| r.is_match(v))
                .unwrap_or(false)
        })
    });
    if all_date {
        return DataType::Date;
    }

    // Check for UUID
    let all_uuid = non_empty.iter().all(|v| uuid::Uuid::parse_str(v).is_ok());
    if all_uuid {
        return DataType::Uuid;
    }

    DataType::String
}

/// Estimate cardinality from sample.
fn estimate_cardinality(values: &[&str]) -> u64 {
    let unique: std::collections::HashSet<_> = values.iter().collect();
    unique.len() as u64
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_data_type() {
        assert_eq!(infer_data_type(&["1", "2", "3"]), DataType::Int64);
        assert_eq!(infer_data_type(&["1.5", "2.5", "3.5"]), DataType::Decimal);
        assert_eq!(infer_data_type(&["true", "false"]), DataType::Boolean);
        assert_eq!(
            infer_data_type(&["2024-01-15", "2024-02-20"]),
            DataType::Date
        );
        assert_eq!(infer_data_type(&["hello", "world"]), DataType::String);
    }
}
