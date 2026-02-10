# DataSynth Deployment Guide

## Docker

Build and run the server:

```bash
docker build -t datasynth:latest .
docker run -p 50051:50051 -p 3000:3000 datasynth:latest
```

Build the CLI-only image:

```bash
docker build -f Dockerfile.cli -t datasynth-cli:latest .
docker run --rm -v $(pwd)/output:/output datasynth-cli:latest generate --demo --output /output
```

## Docker Compose

Start the full stack (server + Prometheus + Grafana):

```bash
docker compose up -d
```

Access points:
- REST API: http://localhost:3000
- gRPC: localhost:50051
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3001 (admin/admin)

## SystemD

1. Install the binary:

```bash
sudo cp target/release/datasynth-server /usr/local/bin/
```

2. Create the service user:

```bash
sudo useradd --system --no-create-home --shell /usr/sbin/nologin datasynth
sudo mkdir -p /var/lib/datasynth /etc/datasynth
sudo chown datasynth:datasynth /var/lib/datasynth
```

3. Configure the environment:

```bash
sudo cp deploy/datasynth-server.env.example /etc/datasynth/server.env
sudo chmod 600 /etc/datasynth/server.env
# Edit /etc/datasynth/server.env with your settings
```

4. Install and start the service:

```bash
sudo cp deploy/datasynth-server.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now datasynth-server
sudo systemctl status datasynth-server
```

## Health Checks

| Endpoint | Purpose |
|----------|---------|
| `GET /health` | Basic health check |
| `GET /ready` | Readiness probe (config + memory + disk) |
| `GET /live` | Liveness probe |
| `GET /metrics` | Prometheus metrics |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level filter | `info` |
| `DATASYNTH_API_KEYS` | Comma-separated API keys | (disabled) |
| `DATASYNTH_TLS_CERT` | TLS certificate path | (disabled) |
| `DATASYNTH_TLS_KEY` | TLS private key path | (disabled) |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP endpoint | `http://localhost:4317` |
