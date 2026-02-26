# Performance Tuning

Optimize SyntheticData for your hardware and requirements.

## Performance Characteristics

| Metric | Typical Performance |
|--------|---------------------|
| Single-threaded | ~200,000 entries/second |
| Parallel (8 cores) | ~1,200,000 entries/second |
| Memory per 1M entries | ~500 MB |

## What's New in 0.9.0

Version 0.9.0 delivers ~2x single-threaded throughput through three optimization phases:

**Phase 1 — Hot Path Optimization:**
- Cached temporal CDF lookup (eliminates per-entry recomputation)
- Fast `Decimal` construction bypassing string parsing
- `SmallVec` for journal entry line items (avoids heap allocation for ≤4 lines)
- Binary search company selector replacing linear scan
- `#[inline]` annotations on hot paths
- 256KB `BufWriter` buffers (up from 8KB default) across 86+ output sinks

**Phase 2 — Parallel Generation:**
- `ParallelGenerator` trait with deterministic seed splitting
- Multi-core master data and journal entry generation
- Per-partition UUID factories for collision-free parallel output

**Phase 3 — I/O Optimization:**
- `itoa`/`ryu` formatting for integer and float fields
- `fast_csv` module for zero-allocation CSV row writing
- zstd `CompressedWriter` for streaming compressed output
- Optimized `CsvSink` and `JsonLinesSink` write paths

## Configuration Tuning

### Worker Threads

```yaml
global:
  worker_threads: 8                  # Match CPU cores
```

**Guidelines:**
- Default: Uses all available cores
- I/O bound: May benefit from > cores
- Memory constrained: Reduce threads

### Memory Limits

```yaml
global:
  memory_limit: 2147483648           # 2 GB
```

**Guidelines:**
- Set to ~75% of available RAM
- Leave room for OS and other processes
- Lower limit = more streaming, less memory

### Batch Sizes

The orchestrator automatically tunes batch sizes, but you can influence behavior:

```yaml
transactions:
  target_count: 100000

# Implicit batch sizing based on:
# - Available memory
# - Number of threads
# - Target count
```

## Hardware Recommendations

### Minimum

| Resource | Specification |
|----------|---------------|
| CPU | 2 cores |
| RAM | 4 GB |
| Storage | 10 GB |

**Suitable for:** <100K entries, development

### Recommended

| Resource | Specification |
|----------|---------------|
| CPU | 8 cores |
| RAM | 16 GB |
| Storage | 50 GB SSD |

**Suitable for:** 1M entries, production

### High Performance

| Resource | Specification |
|----------|---------------|
| CPU | 32+ cores |
| RAM | 64+ GB |
| Storage | NVMe SSD |

**Suitable for:** 10M+ entries, benchmarking

## Optimizing Generation

### Reduce Memory Pressure

**Enable streaming output:**
```yaml
output:
  format: csv
  # Writing as generated reduces memory
```

**Disable unnecessary features:**
```yaml
graph_export:
  enabled: false                     # Skip if not needed

anomaly_injection:
  enabled: false                     # Add in post-processing
```

### Optimize for Speed

**Maximize parallelism:**
```yaml
global:
  worker_threads: 16                 # More threads
```

**Simplify output:**
```yaml
output:
  format: csv                        # Faster than JSON
  compression: none                  # Skip compression time
```

**Reduce complexity:**
```yaml
chart_of_accounts:
  complexity: small                  # Fewer accounts

document_flows:
  p2p:
    enabled: false                   # Skip if not needed
```

### Optimize for Size

**Enable compression:**
```yaml
output:
  compression: zstd
  compression_level: 9               # Maximum compression
```

**Minimize output files:**
```yaml
output:
  files:
    journal_entries: true
    acdoca: false
    master_data: false               # Only what you need
```

## Benchmarking

### Built-in Benchmarks

```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench --bench generation_throughput

# With baseline comparison
cargo bench -- --baseline main
```

### Benchmark Categories

| Benchmark | Measures |
|-----------|----------|
| `generation_throughput` | Entries/second |
| `distribution_sampling` | Distribution speed |
| `output_sink` | Write performance |
| `scalability` | Parallel scaling |
| `correctness` | Validation overhead |

### Manual Benchmarking

```bash
# Time generation
time datasynth-data generate --config config.yaml --output ./output

# Profile memory
/usr/bin/time -v datasynth-data generate --config config.yaml --output ./output
```

## Profiling

### CPU Profiling

```bash
# With perf (Linux)
perf record datasynth-data generate --config config.yaml --output ./output
perf report

# With Instruments (macOS)
xcrun xctrace record --template "Time Profiler" \
    --launch datasynth-data generate --config config.yaml --output ./output
```

### Memory Profiling

```bash
# With heaptrack (Linux)
heaptrack datasynth-data generate --config config.yaml --output ./output
heaptrack_print heaptrack.*.gz

# With Instruments (macOS)
xcrun xctrace record --template "Allocations" \
    --launch datasynth-data generate --config config.yaml --output ./output
```

## Common Bottlenecks

### I/O Bound

**Symptoms:**
- CPU utilization < 100%
- Disk utilization high

**Solutions:**
- Use faster storage (SSD/NVMe)
- Enable compression (reduces write volume)
- Increase buffer sizes

### Memory Bound

**Symptoms:**
- OOM errors
- Excessive swapping

**Solutions:**
- Reduce `target_count`
- Lower `memory_limit`
- Enable streaming
- Reduce parallel threads

### CPU Bound

**Symptoms:**
- CPU at 100%
- Generation time scales linearly

**Solutions:**
- Add more cores
- Simplify configuration
- Disable unnecessary features

## Scaling Guidelines

### Entries vs Time

| Entries | ~Time (8 cores) |
|---------|-----------------|
| 10,000 | <1 second |
| 100,000 | ~2 seconds |
| 1,000,000 | ~20 seconds |
| 10,000,000 | ~3 minutes |

### Entries vs Memory

| Entries | Peak Memory |
|---------|-------------|
| 10,000 | ~50 MB |
| 100,000 | ~200 MB |
| 1,000,000 | ~1.5 GB |
| 10,000,000 | ~12 GB |

*Memory estimates include full in-memory processing. Streaming reduces by ~80%.*

## Server Performance

### Rate Limiting

```bash
cargo run -p datasynth-server -- \
    --port 3000 \
    --rate-limit 1000              # Requests per minute
```

### Connection Pooling

For high-concurrency scenarios, configure worker threads:

```bash
cargo run -p datasynth-server -- \
    --worker-threads 16            # Handle more connections
```

### WebSocket Optimization

```yaml
# Client-side: batch messages
const BATCH_SIZE = 100;  // Request 100 entries at a time
```

## See Also

- [Memory Management](../architecture/memory-management.md)
- [CLI Reference](../user-guide/cli-reference.md)
- [Server API](../user-guide/server-api.md)
