//! Example `SinkPlugin` that writes `GeneratedRecord`s to an in-memory buffer
//! as CSV-like lines.
//!
//! This plugin demonstrates the full sink lifecycle: initialize, write, finalize.

use crate::error::SynthError;
use crate::traits::plugin::{GeneratedRecord, SinkPlugin, SinkSummary};

/// A sink plugin that formats records as CSV-like lines into a `String` buffer.
///
/// # Configuration
///
/// Accepts a JSON object with an optional `delimiter` field (single character,
/// defaults to `,`).
///
/// ```json
/// { "delimiter": ";" }
/// ```
pub struct CsvEchoSink {
    /// Column delimiter character.
    delimiter: char,
    /// Accumulated output lines.
    buffer: Vec<String>,
    /// Total records written so far.
    record_count: usize,
}

impl Default for CsvEchoSink {
    fn default() -> Self {
        Self {
            delimiter: ',',
            buffer: Vec::new(),
            record_count: 0,
        }
    }
}

impl CsvEchoSink {
    /// Create a new `CsvEchoSink` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a reference to the accumulated output lines.
    pub fn lines(&self) -> &[String] {
        &self.buffer
    }

    /// Return the total number of records written.
    pub fn record_count(&self) -> usize {
        self.record_count
    }
}

impl SinkPlugin for CsvEchoSink {
    fn name(&self) -> &str {
        "csv_echo"
    }

    fn initialize(&mut self, config: &serde_json::Value) -> Result<(), SynthError> {
        if let Some(delim) = config.get("delimiter").and_then(|v| v.as_str()) {
            let mut chars = delim.chars();
            self.delimiter = chars.next().ok_or_else(|| {
                SynthError::generation("delimiter must be a non-empty string")
            })?;
        }
        self.buffer.clear();
        self.record_count = 0;
        Ok(())
    }

    fn write_records(&mut self, records: &[GeneratedRecord]) -> Result<usize, SynthError> {
        for record in records {
            // Collect field keys in sorted order for deterministic output.
            let mut keys: Vec<&String> = record.fields.keys().collect();
            keys.sort();

            let values: Vec<String> = keys
                .iter()
                .map(|k| {
                    record
                        .fields
                        .get(*k)
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default()
                })
                .collect();

            let line = values.join(&self.delimiter.to_string());
            self.buffer.push(line);
        }
        self.record_count += records.len();
        Ok(records.len())
    }

    fn finalize(&mut self) -> Result<SinkSummary, SynthError> {
        let total_bytes: usize = self.buffer.iter().map(|l| l.len()).sum();
        let mut summary = SinkSummary::new(self.record_count);
        summary.bytes_written = Some(total_bytes as u64);
        summary
            .metadata
            .insert("delimiter".to_string(), self.delimiter.to_string());
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_records() -> Vec<GeneratedRecord> {
        vec![
            GeneratedRecord::new("vendor")
                .with_field("id", serde_json::json!("V001"))
                .with_field("name", serde_json::json!("Acme Corp")),
            GeneratedRecord::new("vendor")
                .with_field("id", serde_json::json!("V002"))
                .with_field("name", serde_json::json!("Globex Inc")),
        ]
    }

    #[test]
    fn test_csv_echo_default_delimiter() {
        let mut sink = CsvEchoSink::new();
        sink.initialize(&serde_json::json!({}))
            .expect("init should succeed");

        let written = sink
            .write_records(&sample_records())
            .expect("write should succeed");
        assert_eq!(written, 2);

        let lines = sink.lines();
        assert_eq!(lines.len(), 2);
        // Keys sorted: "id", "name"
        assert_eq!(lines[0], "V001,Acme Corp");
        assert_eq!(lines[1], "V002,Globex Inc");

        let summary = sink.finalize().expect("finalize should succeed");
        assert_eq!(summary.records_written, 2);
        assert!(summary.bytes_written.is_some());
    }

    #[test]
    fn test_csv_echo_custom_delimiter() {
        let mut sink = CsvEchoSink::new();
        sink.initialize(&serde_json::json!({ "delimiter": ";" }))
            .expect("init should succeed");

        sink.write_records(&sample_records())
            .expect("write should succeed");

        let lines = sink.lines();
        assert_eq!(lines[0], "V001;Acme Corp");
    }

    #[test]
    fn test_csv_echo_empty_records() {
        let mut sink = CsvEchoSink::new();
        sink.initialize(&serde_json::json!({}))
            .expect("init should succeed");

        let written = sink.write_records(&[]).expect("write should succeed");
        assert_eq!(written, 0);
        assert_eq!(sink.record_count(), 0);
    }

    #[test]
    fn test_csv_echo_multiple_batches() {
        let mut sink = CsvEchoSink::new();
        sink.initialize(&serde_json::json!({}))
            .expect("init should succeed");

        sink.write_records(&sample_records())
            .expect("first batch");
        sink.write_records(&sample_records())
            .expect("second batch");

        assert_eq!(sink.record_count(), 4);
        assert_eq!(sink.lines().len(), 4);
    }
}
