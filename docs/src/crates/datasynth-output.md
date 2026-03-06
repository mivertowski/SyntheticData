# datasynth-output

Output sinks for CSV, JSON, and streaming formats.

## Overview

`datasynth-output` provides the output layer for DataSynth:

- **CSV Sink**: High-performance CSV writing with optional compression
- **JSON Sink**: JSON and JSONL (newline-delimited) output
- **Streaming**: Async streaming output for real-time generation
- **Control Export**: Internal control and SoD rule export

## Supported Formats

### Standard Formats

| Format | Description | Extension |
|--------|-------------|-----------|
| CSV | Standard comma-separated values | `.csv` |
| JSON | Pretty-printed JSON arrays | `.json` |
| JSONL | Newline-delimited JSON | `.jsonl` |
| Parquet | Apache Parquet columnar format | `.parquet` |

### ERP Formats

| Format | Target ERP | Tables |
|--------|-----------|--------|
| SAP S/4HANA | `SapExporter` | BKPF, BSEG, ACDOCA, LFA1, KNA1, MARA, CSKS, CEPC |
| Oracle EBS | `OracleExporter` | GL_JE_HEADERS, GL_JE_LINES, GL_JE_BATCHES |
| NetSuite | `NetSuiteExporter` | Journal entries with subsidiary/multi-book support |

### Audit Export Formats

| Format | Standard | Description |
|--------|----------|-------------|
| FEC | French Art. A47 A-1 | 18-column semicolon-separated CSV for French fiscal audit |
| GoBD | German GoBD | 13-column journal CSV + account CSV + XML index for German fiscal audit |

### Streaming Sinks

| Sink | Description |
|------|-------------|
| `CsvStreamingSink` | Streaming CSV with automatic headers |
| `JsonStreamingSink` | Streaming JSON arrays |
| `NdjsonStreamingSink` | Streaming newline-delimited JSON |
| `ParquetStreamingSink` | Streaming Apache Parquet |

## Features

- Configurable compression (gzip, zstd, snappy for Parquet)
- Streaming writes for memory efficiency with backpressure support
- ERP-native table schemas (SAP, Oracle, NetSuite)
- Decimal values serialized as strings (IEEE 754 safe)
- Configurable field ordering and headers
- Automatic directory creation

## Key Types

### OutputConfig

```rust
pub struct OutputConfig {
    pub format: OutputFormat,
    pub compression: CompressionType,
    pub compression_level: u32,
    pub include_headers: bool,
    pub decimal_precision: u32,
}

pub enum OutputFormat {
    Csv,
    Json,
    Jsonl,
}

pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}
```

### CsvSink

```rust
pub struct CsvSink<T> {
    writer: BufWriter<Box<dyn Write>>,
    config: OutputConfig,
    headers_written: bool,
    _phantom: PhantomData<T>,
}
```

### JsonSink

```rust
pub struct JsonSink<T> {
    writer: BufWriter<Box<dyn Write>>,
    format: JsonFormat,
    first_written: bool,
    _phantom: PhantomData<T>,
}
```

## Usage Examples

### CSV Output

```rust
use synth_output::{CsvSink, OutputConfig, OutputFormat};

// Create sink
let config = OutputConfig {
    format: OutputFormat::Csv,
    compression: CompressionType::None,
    include_headers: true,
    ..Default::default()
};

let mut sink = CsvSink::new("output/journal_entries.csv", config)?;

// Write data
sink.write_batch(&entries)?;
sink.flush()?;
```

### Compressed Output

```rust
use synth_output::{CsvSink, OutputConfig, CompressionType};

let config = OutputConfig {
    compression: CompressionType::Gzip,
    compression_level: 6,
    ..Default::default()
};

let mut sink = CsvSink::new("output/entries.csv.gz", config)?;
sink.write_batch(&entries)?;
```

### JSON Streaming

```rust
use synth_output::{JsonSink, OutputConfig, OutputFormat};

let config = OutputConfig {
    format: OutputFormat::Jsonl,
    ..Default::default()
};

let mut sink = JsonSink::new("output/entries.jsonl", config)?;

// Stream writes (memory efficient)
for entry in entries {
    sink.write(&entry)?;
}
sink.flush()?;
```

### Control Export

```rust
use synth_output::ControlExporter;

let exporter = ControlExporter::new("output/controls/");

// Export all control-related data
exporter.export_controls(&internal_controls)?;
exporter.export_sod_rules(&sod_rules)?;
exporter.export_control_mappings(&mappings)?;
```

## Sink Trait Implementation

All sinks implement the `Sink` trait:

```rust
impl<T: Serialize> Sink<T> for CsvSink<T> {
    type Error = OutputError;

    fn write(&mut self, item: &T) -> Result<(), Self::Error> {
        // Single item write
    }

    fn write_batch(&mut self, items: &[T]) -> Result<(), Self::Error> {
        // Batch write for efficiency
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Ensure all data written to disk
    }
}
```

## Decimal Serialization

Financial amounts are serialized as strings to prevent IEEE 754 floating-point issues:

```rust
// Internal: Decimal
let amount = dec!(1234.56);

// CSV output: "1234.56" (string)
// JSON output: "1234.56" (string, not number)
```

This ensures exact decimal representation across all systems.

## Performance Tips

### Batch Writes

Prefer batch writes over individual writes:

```rust
// Good: Single batch write
sink.write_batch(&entries)?;

// Less efficient: Multiple writes
for entry in &entries {
    sink.write(entry)?;
}
```

### Buffer Size

The default buffer size is 8KB. For very large outputs, consider adjusting:

```rust
let sink = CsvSink::with_buffer_size(
    "output/large.csv",
    config,
    64 * 1024, // 64KB buffer
)?;
```

### Compression Trade-offs

| Compression | Speed | Size | Use Case |
|-------------|-------|------|----------|
| None | Fastest | Largest | Development, streaming |
| Gzip | Medium | Small | General purpose |
| Zstd | Fast | Smallest | Production, archival |

## Output Structure

The output module creates organized directory structure:

```
output/
├── master_data/
│   ├── vendors.csv
│   └── customers.csv
├── transactions/
│   ├── journal_entries.csv
│   └── acdoca.csv
├── controls/
│   ├── internal_controls.csv
│   └── sod_rules.csv
└── labels/
    └── anomaly_labels.csv
```

## Error Handling

```rust
pub enum OutputError {
    IoError(std::io::Error),
    SerializationError(String),
    CompressionError(String),
    DirectoryCreationError(PathBuf),
}
```

## See Also

- [Output Formats](../user-guide/output-formats.md) — Standard format details
- [ERP Output Formats](../user-guide/erp-output-formats.md) — SAP, Oracle, NetSuite exports
- [Streaming Output](../user-guide/streaming-output.md) — StreamingSink API
- [Configuration - Output Settings](../configuration/output-settings.md)
- [datasynth-core](datasynth-core.md)
