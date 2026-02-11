# Disaster Recovery

DataSynth is a stateless data generation engine. It does not maintain a persistent database or durable state that requires traditional backup and recovery. Instead, recovery relies on two key properties:

1. **Deterministic generation** -- Given the same configuration and seed, DataSynth produces identical output.
2. **Stateless server** -- The server process can be restarted from scratch at any time.

## What Needs to Be Backed Up

| Asset | Location | Recovery Priority |
|-------|----------|-------------------|
| Generation config (YAML) | `/etc/datasynth/`, ConfigMap, or source control | Critical |
| Environment / secrets | `/etc/datasynth/server.env`, K8s Secrets | Critical |
| API keys | Environment variable or Secret | Critical |
| Generated output files | Output directory, object storage | Depends on use case |
| Grafana dashboards | `deploy/grafana/provisioning/` or exported JSON | Low -- can re-provision |
| Prometheus data | `prometheus-data` volume | Low -- regenerate from metrics |

The generation config and seed are the most important assets. With them, you can reproduce any dataset exactly.

## Backup Procedures

### Configuration Backup

Store all DataSynth configuration in version control. This is the primary backup mechanism:

```bash
# Recommended repository structure
configs/
  production/
    manufacturing.yaml      # Generation config
    server.env.encrypted    # Encrypted environment file
  staging/
    retail.yaml
    server.env.encrypted
```

For Kubernetes, export the ConfigMap and Secret:

```bash
# Export current config
kubectl -n datasynth get configmap datasynth-config -o yaml > backup/configmap.yaml

# Export secrets (base64-encoded)
kubectl -n datasynth get secret datasynth-api-keys -o yaml > backup/secret.yaml
```

### Output Data Backup

If generated data must be preserved (not just re-generated), back up the output directory:

```bash
# Local backup
tar czf datasynth-output-$(date +%F).tar.gz /var/lib/datasynth/output/

# S3 backup
aws s3 sync /var/lib/datasynth/output/ s3://your-bucket/datasynth/$(date +%F)/
```

### Scheduled Backup Script

```bash
#!/bin/bash
# /usr/local/bin/datasynth-backup.sh
# Run via cron: 0 2 * * * /usr/local/bin/datasynth-backup.sh

BACKUP_DIR="/var/backups/datasynth"
DATE=$(date +%F)

mkdir -p "$BACKUP_DIR"

# Back up configuration
cp /etc/datasynth/server.env "$BACKUP_DIR/server.env.$DATE"

# Back up output if it exists and is non-empty
if [ -d /var/lib/datasynth/output ] && [ "$(ls -A /var/lib/datasynth/output)" ]; then
  tar czf "$BACKUP_DIR/output-$DATE.tar.gz" /var/lib/datasynth/output/
fi

# Retain 30 days of backups
find "$BACKUP_DIR" -type f -mtime +30 -delete

echo "Backup completed: $DATE"
```

## Deterministic Recovery

DataSynth uses ChaCha8 RNG with a configurable seed. When the seed is set in the configuration, every run produces byte-identical output.

### Reproducing a Dataset

To reproduce a previous generation run:

1. Retrieve the configuration file used for that run.
2. Confirm the seed value is set (not random).
3. Run the generation with the same configuration.

```yaml
# Example config with deterministic seed
global:
  industry: manufacturing
  start_date: "2024-01-01"
  period_months: 12
  seed: 42              # <-- deterministic seed
```

```bash
# Regenerate identical data
datasynth-data generate --config config.yaml --output ./recovered-output

# Verify output is identical
diff <(sha256sum original-output/*.csv | sort) <(sha256sum recovered-output/*.csv | sort)
```

### Important Caveats for Determinism

Deterministic output requires **exact version matching**:

| Factor | Must Match? | Notes |
|--------|------------|-------|
| DataSynth version | Yes | Different versions may change generation logic |
| Configuration YAML | Yes | Any parameter change alters output |
| Seed value | Yes | Different seed = different data |
| Operating system | No | Cross-platform determinism is guaranteed |
| CPU architecture | No | ChaCha8 output is platform-independent |
| Number of threads | No | Parallelism does not affect determinism |

If you need to reproduce data from a past release, pin the DataSynth version:

```bash
# Docker: use the exact version tag
docker run --rm \
  -v $(pwd)/config.yaml:/config.yaml:ro \
  -v $(pwd)/output:/output \
  datasynth/datasynth-server:0.5.0 \
  datasynth-data generate --config /config.yaml --output /output

# Source: checkout the exact tag
git checkout v0.5.0
cargo build --release -p datasynth-cli
```

## Stateless Restart

The DataSynth server maintains no persistent state. All in-memory state (counters, active streams, generation context) is ephemeral. A restart produces a fresh server.

### Restart Procedure

**Docker:**

```bash
docker compose restart datasynth-server
```

**Kubernetes:**

```bash
# Rolling restart (zero downtime with PDB)
kubectl -n datasynth rollout restart deployment/datasynth

# Verify rollout
kubectl -n datasynth rollout status deployment/datasynth
```

**SystemD:**

```bash
sudo systemctl restart datasynth-server
```

### What Is Lost on Restart

| State | Lost? | Impact |
|-------|-------|--------|
| Prometheus metrics counters | Yes | Counters reset to 0; Prometheus handles counter resets via `rate()` |
| Active WebSocket streams | Yes | Clients must reconnect |
| Uptime counter | Yes | Resets to 0 |
| In-progress bulk generation | Yes | Client receives connection error; must retry |
| Configuration (if set via API) | Yes | Reverts to default; use ConfigMap or env for persistence |
| Rate limit buckets | Yes | All clients get fresh rate limit windows |

### Mitigating Restart Impact

1. **Use config files, not the API, for persistent configuration.** The `POST /api/config` endpoint only updates in-memory state.
2. **Set up client retry logic** for bulk generation requests.
3. **Use Kubernetes PDB** to ensure at least one pod is always running during rolling restarts.
4. **Monitor with Prometheus** -- counter resets are handled automatically by `rate()` and `increase()` functions.

## Recovery Scenarios

### Scenario 1: Server Process Crash

1. SystemD or Kubernetes automatically restarts the process.
2. Verify with `curl localhost:3000/health`.
3. Check logs for crash cause: `journalctl -u datasynth-server -n 200`.
4. No data loss -- server is stateless.

### Scenario 2: Node Failure (Kubernetes)

1. Kubernetes reschedules pods to healthy nodes.
2. PDB ensures minimum availability during rescheduling.
3. Clients reconnect automatically (Service endpoint updates).
4. No manual intervention required.

### Scenario 3: Configuration Lost

1. Retrieve config from version control.
2. Redeploy: `kubectl apply -f configmap.yaml` or copy to `/etc/datasynth/`.
3. Restart server to pick up new config.

### Scenario 4: Need to Reproduce Historical Data

1. Identify the DataSynth version and config used.
2. Pin the version (Docker tag or Git tag).
3. Run generation with the same config and seed.
4. Verify with checksums.

## Recovery Time Objectives

| Component | RTO | RPO | Notes |
|-----------|-----|-----|-------|
| Server process | < 30s | N/A (stateless) | Auto-restart via SystemD/K8s |
| Full service (K8s) | < 2 min | N/A (stateless) | Pod scheduling + startup probes |
| Data regeneration | Depends on size | 0 (deterministic) | Re-run with same config+seed |
| Config recovery | < 5 min | Last commit | From version control |
