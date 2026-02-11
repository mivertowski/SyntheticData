# Operational Runbook

This runbook provides step-by-step procedures for monitoring, alerting, troubleshooting, and maintaining DataSynth in production.

## Monitoring Stack Overview

The recommended monitoring stack uses Prometheus for metrics collection and Grafana for dashboards and alerting. The `docker-compose.yml` in the repository root sets this up automatically.

| Component | Default URL | Purpose |
|-----------|-------------|---------|
| Prometheus | `http://localhost:9090` | Metrics storage and alerting rules |
| Grafana | `http://localhost:3001` | Dashboards and visualization |
| DataSynth `/metrics` | `http://localhost:3000/metrics` | Prometheus exposition endpoint |
| DataSynth `/api/metrics` | `http://localhost:3000/api/metrics` | JSON metrics endpoint |

## Prometheus Configuration

The repository includes a pre-configured Prometheus scrape config at `deploy/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alerts.yml"

scrape_configs:
  - job_name: "datasynth"
    static_configs:
      - targets: ["datasynth-server:3000"]
    metrics_path: "/metrics"
    scrape_interval: 10s

  - job_name: "prometheus"
    static_configs:
      - targets: ["localhost:9090"]
```

For Kubernetes, use the ServiceMonitor CRD instead (see [Kubernetes deployment](kubernetes.md#prometheus-servicemonitor)).

## Available Metrics

DataSynth exposes the following Prometheus metrics at `GET /metrics`:

| Metric | Type | Description |
|--------|------|-------------|
| `synth_entries_generated_total` | Counter | Total journal entries generated since startup |
| `synth_anomalies_injected_total` | Counter | Total anomalies injected |
| `synth_uptime_seconds` | Gauge | Server uptime in seconds |
| `synth_entries_per_second` | Gauge | Current generation throughput |
| `synth_active_streams` | Gauge | Number of active WebSocket streaming connections |
| `synth_stream_events_total` | Counter | Total events sent through WebSocket streams |
| `synth_info` | Gauge | Server version info label (always 1) |

## Grafana Dashboard Setup

### Step 1: Add Prometheus Data Source

1. Open Grafana at `http://localhost:3001`.
2. Navigate to **Configuration > Data Sources > Add data source**.
3. Select **Prometheus**.
4. Set URL to `http://prometheus:9090` (Docker) or your Prometheus endpoint.
5. Click **Save & Test**.

If using Docker Compose, the Prometheus data source is auto-provisioned via `deploy/grafana/provisioning/datasources/prometheus.yml`.

### Step 2: Create the DataSynth Dashboard

Create a new dashboard with the following panels:

#### Panel 1: Generation Throughput

```
Type: Time series
Query: rate(synth_entries_generated_total[5m])
Title: Entries Generated per Second (5m rate)
Unit: ops/sec
```

#### Panel 2: Active WebSocket Streams

```
Type: Stat
Query: synth_active_streams
Title: Active Streams
Thresholds: 0 (green), 5 (yellow), 10 (red)
```

#### Panel 3: Total Entries (Counter)

```
Type: Stat
Query: synth_entries_generated_total
Title: Total Entries Generated
Format: short
```

#### Panel 4: Anomaly Injection Rate

```
Type: Time series
Query A: rate(synth_anomalies_injected_total[5m])
Query B: rate(synth_entries_generated_total[5m])
Title: Anomaly Rate
Transform: A / B (using math expression)
Unit: percentunit
```

#### Panel 5: Server Uptime

```
Type: Stat
Query: synth_uptime_seconds
Title: Server Uptime
Unit: seconds (s)
```

#### Panel 6: Stream Events Rate

```
Type: Time series
Query: rate(synth_stream_events_total[1m])
Title: Stream Events per Second
Unit: events/sec
```

### Step 3: Save and Export

Save the dashboard and export as JSON for version control. Place the file in `deploy/grafana/provisioning/dashboards/` for automatic provisioning.

## Alert Rules

The repository includes pre-configured alert rules at `deploy/prometheus-alerts.yml`:

### Alert: ServerDown

```yaml
- alert: ServerDown
  expr: up{job="datasynth"} == 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "DataSynth server is down"
    description: "DataSynth server has been unreachable for more than 1 minute."
```

**Response procedure:**

1. Check the server process: `systemctl status datasynth-server` or `docker compose ps`.
2. Check logs: `journalctl -u datasynth-server -n 100` or `docker compose logs --tail 100 datasynth-server`.
3. Check resource exhaustion: `free -h`, `df -h`, `top`.
4. If OOM killed, increase memory limits and restart.
5. If disk full, clear output directory and restart.

### Alert: HighErrorRate

```yaml
- alert: HighErrorRate
  expr: rate(synth_errors_total[5m]) > 0.1
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High error rate on DataSynth server"
```

**Response procedure:**

1. Check application logs for error patterns: `journalctl -u datasynth-server -p err`.
2. Look for invalid configuration: `curl localhost:3000/ready`.
3. Check if clients are sending malformed requests (rate limit headers in responses).
4. If errors are generation failures, check available memory and disk.

### Alert: HighMemoryUsage

```yaml
- alert: HighMemoryUsage
  expr: synth_memory_usage_bytes / 1024 / 1024 > 3072
  for: 10m
  labels:
    severity: critical
  annotations:
    summary: "High memory usage on DataSynth server"
    description: "Memory usage is {{ $value }}MB, exceeding 3GB threshold."
```

**Response procedure:**

1. Check DataSynth's internal degradation level: `curl localhost:3000/ready` -- the `memory` check status will show `ok`, `degraded`, or `fail`.
2. If degraded, DataSynth automatically reduces batch sizes. Wait for current work to complete.
3. If in Emergency mode, stop active streams: `curl -X POST localhost:3000/api/stream/stop`.
4. Consider increasing memory limits or reducing concurrent streams.

### Alert: HighLatency

```yaml
- alert: HighLatency
  expr: histogram_quantile(0.99, rate(datasynth_api_request_duration_seconds_bucket[5m])) > 30
  for: 5m
  labels:
    severity: warning
```

**Response procedure:**

1. Check if bulk generation requests are creating large datasets. The default timeout is 300 seconds.
2. Verify CPU is not throttled: `kubectl top pod` or `docker stats`.
3. Consider splitting large generation requests into smaller batches.

### Alert: NoEntitiesGenerated

```yaml
- alert: NoEntitiesGenerated
  expr: increase(synth_entries_generated_total[1h]) == 0 and synth_active_streams > 0
  for: 15m
  labels:
    severity: warning
```

**Response procedure:**

1. Streams are connected but not producing data. Check if streams are paused.
2. Resume streams: `curl -X POST localhost:3000/api/stream/resume`.
3. Check logs for generation failures.
4. Verify the configuration is valid: `curl localhost:3000/api/config`.

## Common Troubleshooting

### Server Fails to Start

| Symptom | Cause | Resolution |
|---------|-------|------------|
| `Invalid gRPC address` | Bad `--host` or `--port` value | Check arguments and env vars |
| `Failed to bind REST listener` | Port already in use | `lsof -i :3000` to find conflict |
| `protoc not found` | Missing protobuf compiler | Install `protobuf-compiler` package |
| Immediate exit, no logs | Panic before logger init | Run with `RUST_LOG=debug` |

### Generation Errors

| Symptom | Cause | Resolution |
|---------|-------|------------|
| `Failed to create orchestrator` | Invalid config | Validate with `datasynth-data validate --config config.yaml` |
| `Rate limit exceeded` | Too many API requests | Wait for `Retry-After` header, increase rate limits |
| Empty journal entries | No companies configured | Check `curl localhost:3000/api/config` |
| Slow generation | Large period or high volume | Add worker threads, increase CPU |

### Connection Issues

| Symptom | Cause | Resolution |
|---------|-------|------------|
| `Connection refused` on 3000 | Server not running or wrong port | Check process and port bindings |
| `401 Unauthorized` | Missing or invalid API key | Add `X-API-Key` header or `Authorization: Bearer <key>` |
| `429 Too Many Requests` | Rate limit exceeded | Respect `Retry-After` header |
| WebSocket drops immediately | Proxy not forwarding Upgrade | Configure proxy for WebSocket (see [TLS doc](tls-reverse-proxy.md)) |

### Memory Issues

DataSynth monitors its own memory usage via `/proc/self/statm` (Linux) and triggers automatic degradation:

| Degradation Level | Trigger | Behavior |
|-------------------|---------|----------|
| Normal | < 70% of limit | Full throughput |
| Reduced | 70-85% | Smaller batch sizes |
| Minimal | 85-95% | Single-record generation |
| Emergency | > 95% | Rejects new work |

Check the current level:

```bash
curl -s localhost:3000/ready | jq '.checks[] | select(.name == "memory")'
```

## Log Analysis

### Structured JSON Logs

DataSynth emits structured JSON logs with the following fields:

```json
{
  "timestamp": "2024-01-15T10:30:00.123Z",
  "level": "INFO",
  "target": "datasynth_server::rest::routes",
  "message": "Configuration update requested",
  "thread_id": 42
}
```

### Common Log Queries

Filter by severity:

```bash
# SystemD
journalctl -u datasynth-server -p err --since "1 hour ago"

# Docker
docker compose logs datasynth-server | jq 'select(.level == "ERROR" or .level == "WARN")'
```

Find configuration changes:

```bash
journalctl -u datasynth-server | grep "Configuration update"
```

Track generation throughput:

```bash
journalctl -u datasynth-server | grep "entries_generated"
```

Find API authentication failures:

```bash
journalctl -u datasynth-server | grep -i "unauthorized\|invalid api key"
```

### Log Level Configuration

Set per-module log levels with `RUST_LOG`:

```bash
# Everything at info, server REST module at debug
RUST_LOG=info,datasynth_server::rest=debug

# Generation engine at trace (very verbose)
RUST_LOG=info,datasynth_runtime=trace

# Suppress noisy modules
RUST_LOG=info,hyper=warn,tower=warn
```

## Routine Maintenance

### Health Check Script

Create a monitoring script for external health checks:

```bash
#!/bin/bash
# /usr/local/bin/datasynth-healthcheck.sh

ENDPOINT="${1:-http://localhost:3000}"

# Check health
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$ENDPOINT/health")
if [ "$HTTP_CODE" != "200" ]; then
  echo "CRITICAL: Health check failed (HTTP $HTTP_CODE)"
  exit 2
fi

# Check readiness
READY=$(curl -s "$ENDPOINT/ready" | jq -r '.ready')
if [ "$READY" != "true" ]; then
  echo "WARNING: Server not ready"
  exit 1
fi

echo "OK: DataSynth healthy and ready"
exit 0
```

### Prometheus Rule Testing

Validate alert rules before deploying:

```bash
# Install promtool
go install github.com/prometheus/prometheus/cmd/promtool@latest

# Test rules
promtool check rules deploy/prometheus-alerts.yml
```

### Backup Checklist

| Item | Location | Frequency |
|------|----------|-----------|
| DataSynth config | `/etc/datasynth/server.env` | On change |
| Generation configs | YAML files | On change |
| Grafana dashboards | Export as JSON | Weekly |
| Prometheus data | `prometheus-data` volume | Per retention policy |
| API keys | Kubernetes Secret or env file | On rotation |

## Incident Response Template

When a production incident occurs:

1. **Detect**: Alert fires or user reports an issue.
2. **Triage**: Check `/health`, `/ready`, and `/metrics` endpoints.
3. **Contain**: If generating bad data, stop streams: `POST /api/stream/stop`.
4. **Diagnose**: Collect logs (`journalctl -u datasynth-server --since "1 hour ago"`).
5. **Resolve**: Apply fix (restart, config change, scale up).
6. **Verify**: Confirm `/ready` returns `ready: true` and metrics are flowing.
7. **Document**: Record root cause and remediation steps.
