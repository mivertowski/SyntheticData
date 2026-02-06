# datasynth-output

Output sinks for CSV, JSON, Parquet, and streaming formats.

## Overview

`datasynth-output` provides the output layer for SyntheticData:

- **CSV Sink**: High-performance CSV writing with optional compression
- **JSON Sink**: JSON and JSONL (newline-delimited) output
- **Parquet Sink**: Apache Parquet output with Arrow schema and Zstd compression
- **Streaming**: Async streaming output for real-time generation
- **ERP Formats**: SAP, Oracle EBS, and NetSuite export formats
- **Control Export**: Internal control and SoD rule export

## Supported Formats

| Format | Description |
|--------|-------------|
| CSV | Standard comma-separated values |
| JSON | Pretty-printed JSON arrays |
| JSONL | Newline-delimited JSON (streaming-friendly) |
| Parquet | Apache Parquet with Zstd compression (15-column Arrow schema) |

## Features

- Apache Parquet output with configurable batch size (default 10K rows)
- Zstd compression for efficient storage
- Configurable compression (gzip, zstd)
- Streaming writes for memory efficiency
- Decimal values serialized as strings (IEEE 754 safe)
- Configurable field ordering and headers

## Usage

```rust
use datasynth_output::{CsvSink, JsonSink, OutputConfig};

// CSV output
let sink = CsvSink::new("output/journal_entries.csv", config)?;
sink.write_batch(&entries)?;

// JSON streaming
let sink = JsonSink::new("output/entries.jsonl", OutputConfig::jsonl())?;
for entry in entries {
    sink.write(&entry)?;
}
```

## License

Apache-2.0 - See [LICENSE](../../LICENSE) for details.
