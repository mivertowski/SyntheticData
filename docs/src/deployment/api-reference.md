# API Reference

DataSynth exposes REST, gRPC, and WebSocket interfaces. This page documents all endpoints, authentication, rate limiting, error formats, and the WebSocket protocol.

## Base URLs

| Protocol | Default URL | Port |
|----------|-------------|------|
| REST | `http://localhost:3000` | 3000 |
| gRPC | `grpc://localhost:50051` | 50051 |
| WebSocket | `ws://localhost:3000/ws/` | 3000 |

## Authentication

Authentication is optional and disabled by default. When enabled, all endpoints except health probes require a valid API key.

### Enabling Authentication

Pass API keys at startup:

```bash
# CLI argument
datasynth-server --api-keys "key-1,key-2"

# Environment variable
DATASYNTH_API_KEYS="key-1,key-2" datasynth-server
```

### Sending API Keys

The server accepts API keys via two headers (checked in order):

| Method | Header | Example |
|--------|--------|---------|
| Bearer token | `Authorization` | `Authorization: Bearer your-api-key` |
| Custom header | `X-API-Key` | `X-API-Key: your-api-key` |

### Exempt Paths

These paths never require authentication, even when auth is enabled:

- `GET /health`
- `GET /ready`
- `GET /live`
- `GET /metrics`

### Authentication Internals

- API keys are hashed with **Argon2id** at server startup.
- Verification iterates all stored hashes (no short-circuit) to prevent timing side-channel attacks.
- A 5-second LRU cache avoids repeated Argon2 verification for rapid successive requests.

### Error Responses

```
HTTP/1.1 401 Unauthorized
WWW-Authenticate: Bearer

API key required. Provide via 'Authorization: Bearer <key>' or 'X-API-Key' header
```

```
HTTP/1.1 401 Unauthorized
WWW-Authenticate: Bearer

Invalid API key
```

## Rate Limiting

Rate limiting is configurable and disabled by default. When enabled, it tracks requests per client IP using a sliding window.

### Default Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `max_requests` | 100 | Maximum requests per window |
| `window` | 60 seconds | Time window duration |
| Exempt paths | `/health`, `/ready`, `/live` | Not rate-limited |

### Rate Limit Headers

All non-exempt responses include rate limit headers:

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Maximum requests allowed in the window |
| `X-RateLimit-Remaining` | Requests remaining in the current window |
| `Retry-After` | Seconds until the window resets (only on 429) |

### Rate Limit Exceeded Response

```
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
Retry-After: 60

Rate limit exceeded. Max 100 requests per 60 seconds.
```

### Client Identification

The rate limiter identifies clients by IP address, checked in order:

1. `X-Forwarded-For` header (first IP)
2. `X-Real-IP` header
3. Fallback: `unknown` (all unidentified clients share a bucket)

### Distributed Rate Limiting

For multi-replica deployments, enable Redis-backed rate limiting:

```bash
datasynth-server --redis-url redis://127.0.0.1:6379
```

This requires the `redis` feature to be enabled at build time.

## Security Headers

All responses include the following security headers:

| Header | Value | Purpose |
|--------|-------|---------|
| `X-Content-Type-Options` | `nosniff` | Prevent MIME type sniffing |
| `X-Frame-Options` | `DENY` | Prevent clickjacking |
| `X-XSS-Protection` | `0` | Disable legacy XSS filter (rely on CSP) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Control referrer leakage |
| `Content-Security-Policy` | `default-src 'none'; frame-ancestors 'none'` | Restrict resource loading |
| `Cache-Control` | `no-store` | Prevent caching of API responses |

## Request ID

Every response includes an `X-Request-Id` header. If the client sends an `X-Request-Id` header, its value is preserved. Otherwise, a UUID v4 is generated.

```bash
# Client-provided request ID
curl -H "X-Request-Id: my-trace-123" http://localhost:3000/health
# Response header: X-Request-Id: my-trace-123

# Auto-generated request ID
curl -v http://localhost:3000/health
# Response header: X-Request-Id: 550e8400-e29b-41d4-a716-446655440000
```

## CORS Configuration

Default allowed origins:

| Origin | Purpose |
|--------|---------|
| `http://localhost:5173` | Vite dev server |
| `http://localhost:3000` | Local development |
| `http://127.0.0.1:5173` | Localhost variant |
| `http://127.0.0.1:3000` | Localhost variant |
| `tauri://localhost` | Tauri desktop app |

Allowed methods: `GET`, `POST`, `PUT`, `DELETE`, `OPTIONS`

Allowed headers: `Content-Type`, `Authorization`, `Accept`

## REST API Endpoints

### Health & Metrics

#### GET /health

Returns overall server health status.

**Response** `200 OK`:

```json
{
  "healthy": true,
  "version": "0.5.0",
  "uptime_seconds": 3600
}
```

#### GET /ready

Kubernetes-compatible readiness probe. Performs deep checks (config, memory, disk).

**Response** `200 OK` (when ready):

```json
{
  "ready": true,
  "message": "Service is ready",
  "checks": [
    { "name": "config", "status": "ok" },
    { "name": "memory", "status": "ok" },
    { "name": "disk", "status": "ok" }
  ]
}
```

**Response** `503 Service Unavailable` (when not ready):

```json
{
  "ready": false,
  "message": "Service is not ready",
  "checks": [
    { "name": "config", "status": "ok" },
    { "name": "memory", "status": "fail" },
    { "name": "disk", "status": "ok" }
  ]
}
```

#### GET /live

Kubernetes-compatible liveness probe. Lightweight heartbeat.

**Response** `200 OK`:

```json
{
  "alive": true,
  "timestamp": "2024-01-15T10:30:00.123456789Z"
}
```

#### GET /api/metrics

Returns server metrics as JSON.

**Response** `200 OK`:

```json
{
  "total_entries_generated": 150000,
  "total_anomalies_injected": 750,
  "uptime_seconds": 3600,
  "session_entries": 150000,
  "session_entries_per_second": 41.67,
  "active_streams": 2,
  "total_stream_events": 50000
}
```

#### GET /metrics

Prometheus-compatible metrics in text exposition format.

**Response** `200 OK` (`text/plain; version=0.0.4`):

```
# HELP synth_entries_generated_total Total number of journal entries generated
# TYPE synth_entries_generated_total counter
synth_entries_generated_total 150000

# HELP synth_anomalies_injected_total Total number of anomalies injected
# TYPE synth_anomalies_injected_total counter
synth_anomalies_injected_total 750

# HELP synth_uptime_seconds Server uptime in seconds
# TYPE synth_uptime_seconds gauge
synth_uptime_seconds 3600

# HELP synth_entries_per_second Rate of entry generation
# TYPE synth_entries_per_second gauge
synth_entries_per_second 41.67

# HELP synth_active_streams Number of active streaming connections
# TYPE synth_active_streams gauge
synth_active_streams 2

# HELP synth_stream_events_total Total events sent through streams
# TYPE synth_stream_events_total counter
synth_stream_events_total 50000

# HELP synth_info Server version information
# TYPE synth_info gauge
synth_info{version="0.5.0"} 1
```

### Configuration

#### GET /api/config

Returns the current generation configuration.

**Response** `200 OK`:

```json
{
  "success": true,
  "message": "Current configuration",
  "config": {
    "industry": "Manufacturing",
    "start_date": "2024-01-01",
    "period_months": 12,
    "seed": 42,
    "coa_complexity": "Medium",
    "companies": [
      {
        "code": "1000",
        "name": "Manufacturing Corp",
        "currency": "USD",
        "country": "US",
        "annual_transaction_volume": 100000,
        "volume_weight": 1.0
      }
    ],
    "fraud_enabled": true,
    "fraud_rate": 0.02
  }
}
```

#### POST /api/config

Updates the generation configuration.

**Request body**:

```json
{
  "industry": "retail",
  "start_date": "2024-06-01",
  "period_months": 6,
  "seed": 12345,
  "coa_complexity": "large",
  "companies": [
    {
      "code": "1000",
      "name": "Retail Corp",
      "currency": "USD",
      "country": "US",
      "annual_transaction_volume": 200000,
      "volume_weight": 1.0
    }
  ],
  "fraud_enabled": true,
  "fraud_rate": 0.05
}
```

**Valid industries**: `manufacturing`, `retail`, `financial_services`, `healthcare`, `technology`, `professional_services`, `energy`, `transportation`, `real_estate`, `telecommunications`

**Valid CoA complexities**: `small`, `medium`, `large`

**Response** `200 OK`:

```json
{
  "success": true,
  "message": "Configuration updated and applied",
  "config": { ... }
}
```

**Error** `400 Bad Request`:

```json
{
  "success": false,
  "message": "Unknown industry: 'invalid'. Valid values: manufacturing, retail, ...",
  "config": null
}
```

### Generation

#### POST /api/generate/bulk

Generates journal entries in a single batch. Maximum 1,000,000 entries per request.

**Request body**:

```json
{
  "entry_count": 10000,
  "include_master_data": true,
  "inject_anomalies": true
}
```

All fields are optional. Without `entry_count`, the server uses the configured volume.

**Response** `200 OK`:

```json
{
  "success": true,
  "entries_generated": 10000,
  "duration_ms": 450,
  "anomaly_count": 50
}
```

**Error** `400 Bad Request` (entry count too large):

```
entry_count (2000000) exceeds maximum allowed value (1000000)
```

### Streaming Control

#### POST /api/stream/start

Starts the event stream. WebSocket clients begin receiving events.

**Request body**:

```json
{
  "events_per_second": 10,
  "max_events": 10000,
  "inject_anomalies": false
}
```

#### POST /api/stream/stop

Stops all active streams.

#### POST /api/stream/pause

Pauses active streams. Events stop flowing but connections remain open.

#### POST /api/stream/resume

Resumes paused streams.

#### POST /api/stream/trigger/:pattern

Triggers a named generation pattern for upcoming streamed entries.

**Valid patterns**: `year_end_spike`, `period_end_spike`, `holiday_cluster`, `fraud_cluster`, `error_cluster`, `uniform`, `custom:*`

**Response**:

```json
{
  "success": true,
  "message": "Pattern 'year_end_spike' will be applied to upcoming entries"
}
```

## WebSocket Protocol

### ws://localhost:3000/ws/metrics

Sends metrics updates every 1 second as JSON text frames:

```json
{
  "timestamp": "2024-01-15T10:30:00.123Z",
  "total_entries": 150000,
  "total_anomalies": 750,
  "entries_per_second": 41.67,
  "active_streams": 2,
  "uptime_seconds": 3600
}
```

### ws://localhost:3000/ws/events

Streams generated journal entry events as JSON text frames:

```json
{
  "sequence": 1234,
  "timestamp": "2024-01-15T10:30:00.456Z",
  "event_type": "JournalEntry",
  "document_id": "JE-2024-001234",
  "company_code": "1000",
  "amount": "15000.00",
  "is_anomaly": false
}
```

### Connection Management

- The server responds to WebSocket `Ping` frames with `Pong`.
- Clients should send periodic pings to keep the connection alive through proxies.
- Close the connection by sending a WebSocket `Close` frame.
- The server decrements `active_streams` when a client disconnects.

### Example: Connecting with wscat

```bash
# Install wscat
npm install -g wscat

# Connect to metrics stream
wscat -c ws://localhost:3000/ws/metrics

# Connect to event stream
wscat -c ws://localhost:3000/ws/events
```

### Example: Connecting with curl (WebSocket)

```bash
curl --include \
  --no-buffer \
  --header "Connection: Upgrade" \
  --header "Upgrade: websocket" \
  --header "Sec-WebSocket-Version: 13" \
  --header "Sec-WebSocket-Key: $(openssl rand -base64 16)" \
  http://localhost:3000/ws/events
```

## Request Timeout

The default request timeout is 300 seconds (5 minutes), which accommodates large bulk generation requests. Requests exceeding this timeout receive a `408 Request Timeout` response.

## Error Format

REST API errors follow a consistent format:

**Validation errors** return JSON:

```json
{
  "success": false,
  "message": "Descriptive error message",
  "config": null
}
```

**Server errors** return plain text:

```
HTTP/1.1 500 Internal Server Error

Generation failed: <error description>
```

### HTTP Status Codes

| Code | Meaning | When |
|------|---------|------|
| 200 | Success | Request completed |
| 400 | Bad Request | Invalid parameters |
| 401 | Unauthorized | Missing or invalid API key |
| 408 | Request Timeout | Request exceeded 300s timeout |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Generation or server failure |
| 503 | Service Unavailable | Readiness check failed |
