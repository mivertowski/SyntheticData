# Server API

DataSynth provides a server component with REST, gRPC, and WebSocket APIs for application integration.

## Starting the Server

```bash
cargo run -p datasynth-server -- --port 3000 --worker-threads 4
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--port` | 3000 | HTTP/WebSocket port |
| `--grpc-port` | 50051 | gRPC port |
| `--worker-threads` | CPU cores | Worker thread count |
| `--api-key` | None | Required API key |
| `--rate-limit` | 100 | Max requests per minute |

## Authentication

When `--api-key` is set, include it in requests:

```bash
curl -H "X-API-Key: your-api-key" http://localhost:3000/api/config
```

## REST API

### Configuration Endpoints

#### GET /api/config

Retrieve current configuration.

```bash
curl http://localhost:3000/api/config
```

**Response:**
```json
{
  "global": {
    "seed": 42,
    "industry": "manufacturing",
    "start_date": "2024-01-01",
    "period_months": 12
  },
  "transactions": {
    "target_count": 100000
  }
}
```

#### POST /api/config

Update configuration.

```bash
curl -X POST http://localhost:3000/api/config \
  -H "Content-Type: application/json" \
  -d '{"transactions": {"target_count": 50000}}'
```

#### POST /api/config/validate

Validate configuration without applying.

```bash
curl -X POST http://localhost:3000/api/config/validate \
  -H "Content-Type: application/json" \
  -d @config.json
```

### Stream Control Endpoints

#### POST /api/stream/start

Start data generation.

```bash
curl -X POST http://localhost:3000/api/stream/start
```

**Response:**
```json
{
  "status": "started",
  "stream_id": "abc123"
}
```

#### POST /api/stream/stop

Stop current generation.

```bash
curl -X POST http://localhost:3000/api/stream/stop
```

#### POST /api/stream/pause

Pause generation.

```bash
curl -X POST http://localhost:3000/api/stream/pause
```

#### POST /api/stream/resume

Resume paused generation.

```bash
curl -X POST http://localhost:3000/api/stream/resume
```

### Pattern Trigger Endpoints

#### POST /api/stream/trigger/{pattern}

Trigger special event patterns.

**Available patterns:**
- `month_end` - Month-end close activities
- `quarter_end` - Quarter-end activities
- `year_end` - Year-end close activities

```bash
curl -X POST http://localhost:3000/api/stream/trigger/month_end
```

### Health Check

#### GET /health

```bash
curl http://localhost:3000/health
```

**Response:**
```json
{
  "status": "healthy",
  "uptime_seconds": 3600
}
```

## WebSocket API

Connect to receive real-time events during generation.

### Connection

```javascript
const ws = new WebSocket('ws://localhost:3000/ws/events');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(data);
};
```

### Event Types

**Progress Event:**
```json
{
  "type": "progress",
  "current": 50000,
  "total": 100000,
  "percent": 50.0,
  "rate": 85000.5
}
```

**Entry Event:**
```json
{
  "type": "entry",
  "data": {
    "document_id": "abc-123",
    "posting_date": "2024-03-15",
    "account": "1100",
    "debit": "1000.00",
    "credit": "0.00"
  }
}
```

**Error Event:**
```json
{
  "type": "error",
  "message": "Memory limit exceeded"
}
```

**Complete Event:**
```json
{
  "type": "complete",
  "total_entries": 100000,
  "duration_ms": 1200
}
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

message Config {
  string yaml = 1;
}

message GenerationRequest {
  optional int64 count = 1;
}

message Entry {
  string document_id = 1;
  string posting_date = 2;
  string company_code = 3;
  repeated LineItem lines = 4;
}

message LineItem {
  string account = 1;
  string debit = 2;
  string credit = 3;
}
```

### Client Example (Rust)

```rust
use synth::synth_client::SynthClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SynthClient::connect("http://localhost:50051").await?;

    let request = tonic::Request::new(GenerationRequest { count: Some(1000) });
    let mut stream = client.start_generation(request).await?.into_inner();

    while let Some(entry) = stream.message().await? {
        println!("Entry: {:?}", entry.document_id);
    }

    Ok(())
}
```

## Rate Limiting

The server implements sliding window rate limiting:

| Metric | Default |
|--------|---------|
| Window | 1 minute |
| Max requests | 100 |

Exceeding the limit returns `429 Too Many Requests`:

```json
{
  "error": "rate_limit_exceeded",
  "retry_after": 30
}
```

## Memory Management

The server enforces memory limits:

```bash
# Set memory limit (bytes)
cargo run -p datasynth-server -- --memory-limit 1073741824  # 1GB
```

When memory is low:
- Generation pauses automatically
- WebSocket sends warning event
- New requests may be rejected

## Error Responses

| HTTP Status | Meaning |
|-------------|---------|
| 400 | Invalid request/configuration |
| 401 | Missing or invalid API key |
| 429 | Rate limit exceeded |
| 500 | Internal server error |
| 503 | Server overloaded |

**Error Response Format:**
```json
{
  "error": "error_code",
  "message": "Human readable description",
  "details": {}
}
```

## Integration Examples

### Python Client

```python
import requests
import websocket
import json

BASE_URL = "http://localhost:3000"

# Set configuration
config = {
    "transactions": {"target_count": 10000}
}
requests.post(f"{BASE_URL}/api/config", json=config)

# Start generation
requests.post(f"{BASE_URL}/api/stream/start")

# Monitor via WebSocket
ws = websocket.create_connection(f"ws://localhost:3000/ws/events")
while True:
    event = json.loads(ws.recv())
    if event["type"] == "complete":
        break
    print(f"Progress: {event.get('percent', 0)}%")
```

### Node.js Client

```javascript
const axios = require('axios');
const WebSocket = require('ws');

const BASE_URL = 'http://localhost:3000';

async function generate() {
    // Configure
    await axios.post(`${BASE_URL}/api/config`, {
        transactions: { target_count: 10000 }
    });

    // Start
    await axios.post(`${BASE_URL}/api/stream/start`);

    // Monitor
    const ws = new WebSocket('ws://localhost:3000/ws/events');
    ws.on('message', (data) => {
        const event = JSON.parse(data);
        console.log(event);
    });
}
```

## See Also

- [CLI Reference](cli-reference.md)
- [Configuration](../configuration/README.md)
- [Performance Tuning](../advanced/performance.md)
