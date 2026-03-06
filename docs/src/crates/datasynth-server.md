# datasynth-server

REST, gRPC, and WebSocket server for synthetic data generation.

## Overview

`datasynth-server` provides server-based access to DataSynth:

- **REST API**: Configuration management and stream control
- **gRPC API**: High-performance streaming generation
- **WebSocket**: Real-time event streaming
- **Production Features**: Authentication, rate limiting, timeouts

## Starting the Server

```bash
cargo run -p datasynth-server -- --port 3000 --worker-threads 4
```

### Command-Line Options

| Option | Default | Description |
|--------|---------|-------------|
| `--port` | 3000 | HTTP/WebSocket port |
| `--grpc-port` | 50051 | gRPC port |
| `--worker-threads` | CPU cores | Worker thread count |
| `--api-key` | None | Required API key |
| `--rate-limit` | 100 | Max requests per minute |
| `--memory-limit` | None | Memory limit in bytes |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      datasynth-server                       │
├─────────────────────────────────────────────────────────────┤
│  REST API (Axum)  │  gRPC (Tonic)  │  WebSocket (Axum)      │
├─────────────────────────────────────────────────────────────┤
│                   Middleware Layer                          │
│  Auth │ Rate Limit │ Timeout │ CORS │ Logging               │
├─────────────────────────────────────────────────────────────┤
│                 Generation Service                          │
│        (wraps datasynth-runtime orchestrator)               │
└─────────────────────────────────────────────────────────────┘
```

## REST API Endpoints

### Configuration

```bash
# Get current configuration
curl http://localhost:3000/api/config

# Update configuration
curl -X POST http://localhost:3000/api/config \
  -H "Content-Type: application/json" \
  -d '{"transactions": {"target_count": 50000}}'

# Validate configuration
curl -X POST http://localhost:3000/api/config/validate \
  -H "Content-Type: application/json" \
  -d @config.json
```

### Stream Control

```bash
# Start generation
curl -X POST http://localhost:3000/api/stream/start

# Pause
curl -X POST http://localhost:3000/api/stream/pause

# Resume
curl -X POST http://localhost:3000/api/stream/resume

# Stop
curl -X POST http://localhost:3000/api/stream/stop

# Trigger pattern (month_end, quarter_end, year_end)
curl -X POST http://localhost:3000/api/stream/trigger/month_end
```

### Health Check

```bash
curl http://localhost:3000/health
```

## WebSocket API

Connect to `ws://localhost:3000/ws/events` for real-time events.

### Event Types

```json
// Progress
{"type": "progress", "current": 50000, "total": 100000, "percent": 50.0}

// Entry (streamed data)
{"type": "entry", "data": {"document_id": "abc-123", ...}}

// Error
{"type": "error", "message": "Memory limit exceeded"}

// Complete
{"type": "complete", "total_entries": 100000, "duration_ms": 1200}
```

## gRPC API

### Proto Definition

```protobuf
syntax = "proto3";
package synth;

service SynthService {
  rpc GetConfig(Empty) returns (Config);
  rpc SetConfig(Config) returns (Status);
  rpc StartGeneration(GenerationRequest) returns (stream Entry);
  rpc StopGeneration(Empty) returns (Status);
}
```

### Client Example

```rust
use synth::synth_client::SynthClient;

let mut client = SynthClient::connect("http://localhost:50051").await?;

let request = tonic::Request::new(GenerationRequest { count: Some(1000) });
let mut stream = client.start_generation(request).await?.into_inner();

while let Some(entry) = stream.message().await? {
    println!("Entry: {:?}", entry.document_id);
}
```

## Middleware

### Authentication

```bash
# With API key
curl -H "X-API-Key: your-key" http://localhost:3000/api/config
```

### Rate Limiting

Sliding window rate limiter with per-client tracking.

```json
// 429 response when exceeded
{
  "error": "rate_limit_exceeded",
  "retry_after": 30
}
```

### Request Timeout

Default timeout is 30 seconds. Long-running operations use streaming.

## Key Types

### Server Configuration

```rust
pub struct ServerConfig {
    pub port: u16,
    pub grpc_port: u16,
    pub worker_threads: usize,
    pub api_key: Option<String>,
    pub rate_limit: RateLimitConfig,
    pub memory_limit: Option<u64>,
    pub cors_origins: Vec<String>,
}
```

### Rate Limit Configuration

```rust
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub exempt_paths: Vec<String>,
}
```

## Production Deployment

### Docker

```dockerfile
FROM rust:1.88 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p datasynth-server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/datasynth-server /usr/local/bin/
EXPOSE 3000 50051
CMD ["datasynth-server", "--port", "3000"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: datasynth-server
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: datasynth-server
        image: datasynth-server:latest
        ports:
        - containerPort: 3000
        - containerPort: 50051
        env:
        - name: SYNTH_API_KEY
          valueFrom:
            secretKeyRef:
              name: synth-secrets
              key: api-key
        resources:
          limits:
            memory: "2Gi"
```

## Monitoring

### Health Endpoint

```bash
curl http://localhost:3000/health
```

```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "memory_usage_mb": 512,
  "active_streams": 2
}
```

### Logging

Enable structured logging:

```bash
RUST_LOG=synth_server=info cargo run -p datasynth-server
```

## See Also

- [Server API Reference](../user-guide/server-api.md)
- [datasynth-runtime](datasynth-runtime.md)
- [Performance Tuning](../advanced/performance.md)
