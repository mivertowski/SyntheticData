# Streaming Output

SyntheticData provides streaming output sinks for real-time data generation, enabling memory-efficient export of large datasets without loading everything into memory at once.

## Overview

The streaming module in `datasynth-output` implements the `StreamingSink` trait for four output formats:

| Sink | Description | File Extension |
|------|-------------|----------------|
| `CsvStreamingSink` | CSV with automatic headers | `.csv` |
| `JsonStreamingSink` | Pretty-printed JSON arrays | `.json` |
| `NdjsonStreamingSink` | Newline-delimited JSON | `.jsonl` / `.ndjson` |
| `ParquetStreamingSink` | Apache Parquet columnar | `.parquet` |

All streaming sinks accept `StreamEvent` values through the `process()` method:

```rust
pub enum StreamEvent<T> {
    Data(T),       // A data record to write
    Flush,         // Force flush to disk
    Close,         // Close the stream
}
```

## StreamingSink Trait

All streaming sinks implement:

```rust
pub trait StreamingSink<T: Serialize + Send> {
    /// Process a single stream event (data, flush, or close).
    fn process(&mut self, event: StreamEvent<T>) -> SynthResult<()>;

    /// Close the stream and flush remaining data.
    fn close(&mut self) -> SynthResult<()>;

    /// Return the number of items written so far.
    fn items_written(&self) -> u64;

    /// Return the number of bytes written so far.
    fn bytes_written(&self) -> u64;
}
```

## When to Use Streaming vs Batch

| Scenario | Recommendation |
|----------|----------------|
| < 100K records | Batch (`CsvSink` / `JsonSink`) — simpler API |
| 100K–10M records | Streaming — lower memory footprint |
| > 10M records | Streaming with Parquet — columnar compression |
| Real-time consumers | Streaming NDJSON — line-by-line parsing |
| REST/WebSocket API | Streaming — integrated with server endpoints |

## CSV Streaming

```rust
use datasynth_output::streaming::CsvStreamingSink;
use datasynth_core::traits::StreamEvent;

let mut sink = CsvStreamingSink::<JournalEntry>::new("output.csv".into())?;

// Write records one at a time (memory efficient)
for entry in generate_entries() {
    sink.process(StreamEvent::Data(entry))?;
}

// Periodic flush (optional — ensures data is on disk)
sink.process(StreamEvent::Flush)?;

// Close when done
sink.close()?;

println!("Wrote {} items ({} bytes)", sink.items_written(), sink.bytes_written());
```

Headers are written automatically on the first `Data` event.

## JSON Streaming

### Pretty-printed JSON Array

```rust
use datasynth_output::streaming::JsonStreamingSink;

let mut sink = JsonStreamingSink::<JournalEntry>::new("output.json".into())?;
for entry in entries {
    sink.process(StreamEvent::Data(entry))?;
}
sink.close()?;  // Writes closing bracket
```

Output:
```json
[
  { "document_id": "abc-001", ... },
  { "document_id": "abc-002", ... }
]
```

### Newline-Delimited JSON (NDJSON)

```rust
use datasynth_output::streaming::NdjsonStreamingSink;

let mut sink = NdjsonStreamingSink::<JournalEntry>::new("output.jsonl".into())?;
for entry in entries {
    sink.process(StreamEvent::Data(entry))?;
}
sink.close()?;
```

Output:
```
{"document_id":"abc-001",...}
{"document_id":"abc-002",...}
```

NDJSON is ideal for streaming consumers that process records line by line (e.g., `jq`, Kafka, log aggregators).

## Parquet Streaming

Apache Parquet provides columnar compression, making it ideal for large analytical datasets:

```rust
use datasynth_output::streaming::ParquetStreamingSink;

let mut sink = ParquetStreamingSink::<JournalEntry>::new("output.parquet".into())?;
for entry in entries {
    sink.process(StreamEvent::Data(entry))?;
}
sink.close()?;
```

Parquet benefits:
- **Columnar storage**: Efficient for analytical queries that touch few columns
- **Built-in compression**: Snappy, Gzip, or Zstd per column group
- **Schema embedding**: Self-describing files with full type information
- **Predicate pushdown**: Query engines can skip irrelevant row groups

## Configuration

Streaming output is enabled when using the server API or when the runtime detects memory pressure:

```yaml
output:
  format: csv           # csv, json, jsonl, parquet
  streaming: true       # Enable streaming mode
  compression: none     # none, gzip, zstd (CSV/JSON) or snappy/gzip/zstd (Parquet)
```

### Server Streaming

The server API uses streaming sinks for the `/api/stream/` endpoints:

```bash
# Start streaming generation
curl -X POST http://localhost:3000/api/stream/start \
  -H "Content-Type: application/json" \
  -d '{"config": {...}, "format": "ndjson"}'

# WebSocket streaming
wscat -c ws://localhost:3000/ws/events
```

## CLI Streaming

The `--stream-file` flag enables phase-aware streaming output during generation, writing JSONL envelopes to a file as data is produced:

```bash
datasynth-data generate --config config.yaml --output ./output --stream-file events.jsonl
```

Each line is a JSON envelope with generation phase metadata:

```json
{"phase":"journal_entries","item_type":"JournalEntry","data":{...}}
{"phase":"vendors","item_type":"Vendor","data":{...}}
```

This is useful for feeding generated data into downstream pipelines (Kafka, log aggregators) as it is produced, rather than waiting for batch completion.

## Backpressure

Streaming sinks monitor write throughput and provide backpressure signals:

- **`items_written()`** / **`bytes_written()`**: Track progress for rate limiting
- **`Flush` events**: Force periodic disk writes to bound memory usage
- **Disk space monitoring**: The runtime's `DiskGuard` can pause generation when disk space runs low

## Performance

| Format | Throughput | File Size | Use Case |
|--------|-----------|-----------|----------|
| CSV | ~150K rows/sec | Largest | Universal compatibility |
| NDJSON | ~120K rows/sec | Large | Streaming consumers |
| JSON | ~100K rows/sec | Large | Human-readable |
| Parquet | ~80K rows/sec | Smallest | Analytics, data lakes |

Throughput varies with record complexity and disk speed.

## See Also

- [Output Formats](output-formats.md) — Batch output format details
- [ERP Output Formats](erp-output-formats.md) — SAP/Oracle/NetSuite formats
- [Output Settings](../configuration/output-settings.md) — Configuration reference
- [Server API](server-api.md) — Streaming via REST/WebSocket
- [datasynth-output](../crates/datasynth-output.md) — Crate reference
