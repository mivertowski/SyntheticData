# Capacity Planning

This guide provides sizing models, reference benchmarks, and recommendations for provisioning DataSynth deployments.

## Performance Characteristics

DataSynth is CPU-bound during generation and I/O-bound during output. Key characteristics:

- **Throughput**: 100K+ journal entries per second on a single core
- **Scaling**: Near-linear scaling with CPU cores for batch generation
- **Memory**: Proportional to active dataset size (companies, accounts, master data)
- **Disk**: Output size depends on format, compression, and enabled modules
- **Network**: REST/gRPC overhead is minimal; bulk generation is the bottleneck

## Sizing Model

### CPU

DataSynth uses Rayon for parallel generation and Tokio for async I/O. The relationship between CPU cores and throughput:

| Cores | Approx. Entries/sec | Use Case |
|-------|---------------------|----------|
| 1 | 100K | Development, small datasets |
| 2 | 180K | Staging, medium datasets |
| 4 | 350K | Production, large datasets |
| 8 | 650K | High-throughput batch jobs |
| 16 | 1.1M | Maximum single-node throughput |

These numbers are for journal entry generation with balanced debit/credit lines. Enabling additional modules (document flows, subledgers, master data, anomaly injection) reduces throughput by 30-60% due to cross-referencing overhead.

### Memory

Memory usage depends on the active generation context:

| Component | Approximate Memory |
|-----------|--------------------|
| Base server process | 50-100 MB |
| Chart of accounts (small) | 5-10 MB |
| Chart of accounts (large) | 30-50 MB |
| Master data per company (small) | 20-40 MB |
| Master data per company (medium) | 80-150 MB |
| Master data per company (large) | 200-400 MB |
| Active journal entries buffer | 2-5 MB per 10K entries |
| Document flow chains | 50-100 MB per company |
| Anomaly injection engine | 20-50 MB |

**Sizing formula (approximate)**:

```
Memory (MB) = 100 + (companies * master_data_per_company) + (buffer_entries * 0.5)
```

### Recommended Memory by Config Complexity

| Complexity | Companies | Memory Minimum | Memory Recommended |
|------------|-----------|---------------|-------------------|
| Small | 1-2 | 512 MB | 1 GB |
| Medium | 3-5 | 1 GB | 2 GB |
| Large | 5-10 | 2 GB | 4 GB |
| Enterprise | 10-20 | 4 GB | 8 GB |

DataSynth includes built-in memory guards that trigger graceful degradation before OOM. See [Runbook - Memory Issues](runbook.md#memory-issues) for degradation levels.

## Disk Sizing

### Output Size by Format

The output size depends on the number of entries, enabled modules, and output format:

| Entries | CSV (uncompressed) | JSON (uncompressed) | Parquet (compressed) |
|---------|--------------------|--------------------|---------------------|
| 10K | 15-25 MB | 30-50 MB | 3-5 MB |
| 100K | 150-250 MB | 300-500 MB | 30-50 MB |
| 1M | 1.5-2.5 GB | 3-5 GB | 300-500 MB |
| 10M | 15-25 GB | 30-50 GB | 3-5 GB |

These estimates cover journal entries only. Enabling all modules (master data, document flows, subledgers, audit trails, etc.) can multiply total output by 5-10x.

### Output Files by Module

When all modules are enabled, a typical generation produces 60+ output files:

| Category | Typical File Count | Size Relative to JE |
|----------|-------------------|---------------------|
| Journal entries + ACDOCA | 2-3 | 1.0x (baseline) |
| Master data | 6-8 | 0.3-0.5x |
| Document flows | 8-10 | 1.5-2.0x |
| Subledgers | 8-12 | 1.0-1.5x |
| Period close + consolidation | 5-8 | 0.5-1.0x |
| Labels + controls | 6-10 | 0.1-0.3x |
| Audit trails | 6-8 | 0.3-0.5x |

### Disk Provisioning Formula

```
Disk (GB) = entries_millions * format_multiplier * module_multiplier * safety_margin

Where:
  format_multiplier:  CSV=0.25, JSON=0.50, Parquet=0.05  (per million entries)
  module_multiplier:  JE only=1.0, all modules=5.0
  safety_margin:      1.5 (for temp files, logs, etc.)
```

**Example**: 1M entries, all modules, CSV format:

```
Disk = 1 * 0.25 * 5.0 * 1.5 = 1.875 GB (round up to 2 GB)
```

## Reference Benchmarks

Benchmarks run on c5.2xlarge (8 vCPU, 16 GB RAM):

| Scenario | Config | Entries | Time | Throughput | Peak Memory |
|----------|--------|---------|------|------------|-------------|
| Batch (small) | 1 company, small CoA, JE only | 100K | 0.8s | 125K/s | 280 MB |
| Batch (medium) | 3 companies, medium CoA, all modules | 100K | 3.2s | 31K/s | 850 MB |
| Batch (large) | 5 companies, large CoA, all modules | 1M | 45s | 22K/s | 2.1 GB |
| Streaming | 1 company, JE only | continuous | -- | 10 events/s | 350 MB |
| Concurrent API | 10 parallel bulk requests | 10K each | 4.5s | 22K/s total | 1.2 GB |

## Container Resource Recommendations

### Docker / Single Host

| Profile | CPU | Memory | Disk | Use Case |
|---------|-----|--------|------|----------|
| Dev | 1 core | 1 GB | 10 GB | Local testing |
| Staging | 2 cores | 2 GB | 50 GB | Integration testing |
| Production | 4 cores | 4 GB | 100 GB | Regular generation |
| Batch worker | 8 cores | 8 GB | 200 GB | Large dataset generation |

### Kubernetes

| Profile | requests.cpu | requests.memory | limits.cpu | limits.memory | Replicas |
|---------|-------------|----------------|-----------|--------------|----------|
| Light | 250m | 256Mi | 1 | 1Gi | 2 |
| Standard | 500m | 512Mi | 2 | 2Gi | 2-5 |
| Heavy | 1000m | 1Gi | 4 | 4Gi | 3-10 |
| Burst | 2000m | 2Gi | 8 | 8Gi | 5-20 |

## Scaling Guidelines

### Vertical Scaling (Single Node)

Vertical scaling is effective up to 16 cores. Beyond that, returns diminish due to lock contention in the shared ServerState. Recommendations:

1. Start with the "Standard" Kubernetes profile.
2. Monitor `synth_entries_per_second` in Grafana.
3. If throughput plateaus at high CPU, add replicas instead.

### Horizontal Scaling (Multi-Replica)

DataSynth is stateless -- each pod generates data independently. Horizontal scaling considerations:

1. Enable Redis for shared rate limiting across replicas.
2. Use deterministic seeds per replica to avoid duplicate data (seed = base_seed + replica_index).
3. Route bulk generation requests to specific replicas if output deduplication matters.
4. WebSocket streams are per-connection and do not share state across replicas.

### Scaling Decision Tree

```
Is throughput below target?
  |
  +-- Yes: Is CPU utilization > 70%?
  |    |
  |    +-- Yes: Add more replicas (horizontal)
  |    +-- No:  Is memory > 80%?
  |         |
  |         +-- Yes: Increase memory limit
  |         +-- No:  Check I/O (disk throughput, network)
  |
  +-- No: Current sizing is adequate
```

## Network Bandwidth

DataSynth's network requirements are modest:

| Operation | Bandwidth | Notes |
|-----------|-----------|-------|
| Health checks | < 1 KB/s | Negligible |
| Prometheus scrape | 5-10 KB per scrape | Every 10-30s |
| Bulk API response (10K entries) | 5-15 MB burst | Short-lived |
| WebSocket stream | 1-5 KB/s per connection | 10 events/s default |
| gRPC streaming | 2-10 KB/s per stream | Depends on message size |

Network is rarely the bottleneck. A 1 Gbps link supports hundreds of concurrent clients.
