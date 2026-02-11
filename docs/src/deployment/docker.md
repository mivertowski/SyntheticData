# Docker Deployment

This guide walks through building, configuring, and running DataSynth as Docker containers.

## Prerequisites

- Docker Engine 24+ (or Docker Desktop 4.25+)
- Docker Compose v2
- 2 GB RAM minimum (4 GB recommended)
- 10 GB disk for images and generated data

## Images

DataSynth provides two container images:

| Image | Dockerfile | Purpose |
|-------|-----------|---------|
| `datasynth/datasynth-server` | `Dockerfile` | Server (REST + gRPC + WebSocket) |
| `datasynth/datasynth-cli` | `Dockerfile.cli` | CLI for batch generation jobs |

## Multi-Stage Build Walkthrough

The server Dockerfile uses a four-stage build with `cargo-chef` for dependency caching:

```
Stage 1: chef       -- installs cargo-chef on rust:1.82-bookworm
Stage 2: planner    -- computes recipe.json from Cargo.lock
Stage 3: builder    -- cooks dependencies (cached), then builds datasynth-server + datasynth-data
Stage 4: runtime    -- copies binaries into gcr.io/distroless/cc-debian12
```

Build the server image:

```bash
docker build -t datasynth/datasynth-server:0.5.0 .
```

Build the CLI-only image:

```bash
docker build -t datasynth/datasynth-cli:0.5.0 -f Dockerfile.cli .
```

### Build Arguments and Features

To enable optional features (TLS, Redis rate limiting, OpenTelemetry), modify the build command in the builder stage. For example, to enable Redis:

```dockerfile
# In the builder stage, replace the cargo build line:
RUN cargo build --release -p datasynth-server -p datasynth-cli --features redis
```

### Image Size

The distroless runtime image is approximately 40-60 MB. The build cache layer with cooked dependencies significantly speeds up rebuilds when only application code changes.

## Docker Compose Stack

The repository includes a production-ready `docker-compose.yml` with the full observability stack:

```yaml
services:
  datasynth-server:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "50051:50051"  # gRPC
      - "3000:3000"    # REST
    environment:
      - RUST_LOG=info
      - DATASYNTH_API_KEYS=${DATASYNTH_API_KEYS:-}
    healthcheck:
      test: ["CMD", "/usr/local/bin/datasynth-data", "--help"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: "2.0"
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    profiles:
      - redis
    ports:
      - "6379:6379"
    command: >
      redis-server --maxmemory 256mb --maxmemory-policy allkeys-lru
    deploy:
      resources:
        limits:
          memory: 256M
          cpus: "0.5"
    volumes:
      - redis-data:/data
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:v2.51.0
    ports:
      - "9090:9090"
    volumes:
      - ./deploy/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - ./deploy/prometheus-alerts.yml:/etc/prometheus/alerts.yml:ro
      - prometheus-data:/prometheus
    command:
      - "--config.file=/etc/prometheus/prometheus.yml"
      - "--storage.tsdb.retention.time=30d"
    restart: unless-stopped

  grafana:
    image: grafana/grafana:10.4.0
    ports:
      - "3001:3000"
    volumes:
      - ./deploy/grafana/provisioning:/etc/grafana/provisioning:ro
      - grafana-data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD:-admin}
      - GF_USERS_ALLOW_SIGN_UP=false
    restart: unless-stopped

volumes:
  prometheus-data:
  grafana-data:
  redis-data:
```

### Starting the Stack

Basic server only:

```bash
docker compose up -d datasynth-server
```

Full observability stack (server + Prometheus + Grafana):

```bash
docker compose up -d
```

With Redis for distributed rate limiting:

```bash
docker compose --profile redis up -d
```

### Verifying the Deployment

```bash
# Health check
curl http://localhost:3000/health

# Readiness probe
curl http://localhost:3000/ready

# Prometheus metrics
curl http://localhost:3000/metrics

# Grafana UI
open http://localhost:3001  # admin / admin (or GRAFANA_PASSWORD)
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `DATASYNTH_API_KEYS` | (none) | Comma-separated API keys for authentication |
| `DATASYNTH_WORKER_THREADS` | `0` (auto) | Tokio worker threads; 0 = CPU count |
| `DATASYNTH_REDIS_URL` | (none) | Redis URL for distributed rate limiting |
| `DATASYNTH_TLS_CERT` | (none) | Path to TLS certificate (PEM) |
| `DATASYNTH_TLS_KEY` | (none) | Path to TLS private key (PEM) |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | (none) | OpenTelemetry collector endpoint |
| `OTEL_SERVICE_NAME` | (none) | OpenTelemetry service name |

## Resource Limits

Recommended container resource limits by workload:

| Workload | CPU | Memory | Notes |
|----------|-----|--------|-------|
| Light (dev/test) | 1 core | 1 GB | Small configs, < 10K entries |
| Medium (staging) | 2 cores | 2 GB | Medium configs, up to 100K entries |
| Heavy (production) | 4 cores | 4 GB | Large configs, streaming, multiple clients |
| Batch CLI job | 2-8 cores | 2-8 GB | Scales linearly with core count |

## Running CLI Jobs in Docker

Generate data with the CLI image:

```bash
docker run --rm \
  -v $(pwd)/output:/output \
  datasynth/datasynth-cli:0.5.0 \
  generate --demo --output /output
```

Generate from a custom config:

```bash
docker run --rm \
  -v $(pwd)/config.yaml:/config.yaml:ro \
  -v $(pwd)/output:/output \
  datasynth/datasynth-cli:0.5.0 \
  generate --config /config.yaml --output /output
```

## Networking

The server binds to `0.0.0.0` by default inside the container. Port mapping:

| Container Port | Protocol | Service |
|---------------|----------|---------|
| 3000 | TCP | REST API + WebSocket + Prometheus metrics |
| 50051 | TCP | gRPC API |

For WebSocket connections through a reverse proxy, ensure the proxy supports HTTP Upgrade headers. See [TLS & Reverse Proxy](tls-reverse-proxy.md) for Nginx and Envoy configurations.

## Logging

DataSynth server outputs structured JSON logs to stdout, which integrates with Docker's logging drivers:

```bash
# View logs
docker compose logs -f datasynth-server

# Filter by level
docker compose logs datasynth-server | jq 'select(.level == "ERROR")'
```

To change the log format or level, set the `RUST_LOG` environment variable:

```bash
# Debug logging for the server crate only
RUST_LOG=datasynth_server=debug docker compose up -d datasynth-server
```
