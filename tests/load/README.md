# DataSynth k6 Load Tests

Load and performance tests for the DataSynth server using [k6](https://k6.io/).

## Prerequisites

### Install k6

**Linux (Debian/Ubuntu):**

```bash
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg \
  --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D68
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" \
  | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update && sudo apt-get install k6
```

**macOS:**

```bash
brew install k6
```

**Docker:**

```bash
docker run --rm -i grafana/k6 run - <k6-health.js
```

**Other platforms:** See <https://grafana.com/docs/k6/latest/set-up/install-k6/>.

### Start the DataSynth server

```bash
# From the repository root:
cargo run -p datasynth-server -- --port 3000
```

The server must be running before you execute any load test.

## Configuration

All scripts read their target URL from the `BASE_URL` environment variable
(passed to k6 via `-e`).  The default is `http://localhost:3000`.

If the server has API key authentication enabled, pass the key with `-e API_KEY=<key>`.

```bash
k6 run -e BASE_URL=http://my-server:3000 -e API_KEY=my-secret tests/load/k6-health.js
```

## Test Scripts

| Script | Purpose | VUs | Duration | Key Thresholds |
|--------|---------|-----|----------|----------------|
| `k6-health.js` | Smoke test for `/health`, `/ready`, `/live` | 10 | 30 s | p95 < 100 ms |
| `k6-bulk-generate.js` | Ramp load on `POST /api/generate/bulk` | 1 -> 50 -> 1 | 3 min | p95 < 5 s |
| `k6-streaming.js` | WebSocket `/ws/events` event stream | 5 | 1 min | messages received > 0 |
| `k6-jobs.js` | Job lifecycle: submit, poll, verify | 10 | 2 min | 80 % completion rate |
| `k6-soak.js` | Sustained load with metrics monitoring | 5 | 30 min | p95 < 10 s, error < 2 % |

## Running the Tests

### Smoke test (health endpoints)

```bash
k6 run tests/load/k6-health.js
```

### Bulk generation under load

```bash
k6 run tests/load/k6-bulk-generate.js
```

### WebSocket streaming

```bash
k6 run tests/load/k6-streaming.js
```

### Async job lifecycle

```bash
k6 run tests/load/k6-jobs.js
```

### 30-minute soak test

```bash
k6 run tests/load/k6-soak.js
```

### Run all tests sequentially

```bash
for script in k6-health.js k6-bulk-generate.js k6-streaming.js k6-jobs.js k6-soak.js; do
  echo "=== Running $script ==="
  k6 run "tests/load/$script"
  echo ""
done
```

## Shared Module

`k6-common.js` contains shared configuration used by all scripts:

- **`BASE_URL`** / **`WS_URL`** -- server address (configurable via `__ENV.BASE_URL`)
- **`API_KEY`** -- optional API key (configurable via `__ENV.API_KEY`)
- **`commonHeaders()`** -- returns standard JSON headers with optional `X-API-Key`
- **`defaultThresholds`** -- baseline thresholds (p95 < 500 ms, error rate < 1 %)
- **`demoBulkPayload`** -- JSON body for `POST /api/generate/bulk`
- **`demoJobPayload`** -- JSON body for `POST /api/jobs/submit`
- **`checkJsonResponse()`** -- helper to validate HTTP status and JSON body keys

## Interpreting Results

k6 prints a summary table at the end of each run.  Key columns:

- **http_req_duration** -- response time histogram; check p95 against thresholds
- **http_req_failed** -- fraction of non-2xx responses
- **checks** -- pass rate of all `check()` assertions
- Custom metrics (e.g., `bulk_generate_duration`, `ws_messages_received`) appear
  alongside built-in metrics

### Soak test memory monitoring

The soak test scrapes `/metrics` every 10th iteration and logs JSON checkpoints
to stdout.  Look for steadily increasing `synth_entries_generated_total` and
stable `synth_uptime_seconds` growth.  If server-side memory grows unboundedly
relative to entries generated, that indicates a leak.

## Exporting Results

k6 supports multiple output backends:

```bash
# JSON file
k6 run --out json=results.json tests/load/k6-health.js

# InfluxDB
k6 run --out influxdb=http://localhost:8086/k6 tests/load/k6-health.js

# Grafana Cloud k6
K6_CLOUD_TOKEN=<token> k6 cloud tests/load/k6-health.js
```

See <https://grafana.com/docs/k6/latest/results-output/> for all options.
