# Streaming Pipeline

The streaming pipeline enables real-time output of generated records through the `PhaseSink` trait and `StreamPipeline` orchestrator.

## Architecture

```
Generator -> PhaseSink -> StreamTarget -> Output
               |
          Buffer (configurable)
               |
     Backpressure Strategy
```

## PhaseSink Trait

The `PhaseSink` trait provides phase-aware streaming:

```rust
pub trait PhaseSink: Send + Sync {
    fn on_phase_start(&self, phase: &str) -> Result<(), SynthError>;
    fn on_record(&self, phase: &str, record: &serde_json::Value) -> Result<(), SynthError>;
    fn on_phase_end(&self, phase: &str) -> Result<(), SynthError>;
    fn flush(&self) -> Result<(), SynthError>;
}
```

## Stream Targets

| Target | Description |
|--------|-------------|
| File | Writes JSONL to a file path |
| HTTP | Posts records to an HTTP endpoint |
| NoOp | Discards output (benchmarking) |

## JSONL Format

Each line is a JSON object:
```json
{"phase":"journal_entries","record":{"id":"JE-001","date":"2024-01-15"},"timestamp":"2024-01-15T10:30:00Z"}
```

## Backpressure Strategies

| Strategy | Behavior |
|----------|----------|
| `block` | Block producer when buffer full (default, no data loss) |
| `drop_oldest` | Drop oldest buffered items |
| `drop_newest` | Drop incoming items |
| `buffer` | Unbounded buffer growth (use with caution) |

## CLI Usage

```bash
# Stream to JSONL file
datasynth-data generate --config config.yaml --stream-file ./output/stream.jsonl --output ./output
```

## Python Usage

```python
from datasynth_py.config import blueprints

config = blueprints.retail_small()
config = blueprints.with_streaming(config, buffer_size=5000, backpressure="block")

# Or via generate()
result = synth.generate(config, stream_file="./output/stream.jsonl")
```

## Configuration

```yaml
streaming:
  enabled: true
  target: file
  file_path: ./output/stream.jsonl
  buffer_size: 1000
  backpressure: block
  phase_filters:
    master_data: true
    journal_entries: true
    document_flows: true
    anomaly_injection: true
    ocpm: true
```
