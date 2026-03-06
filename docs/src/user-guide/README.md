# User Guide

This section covers the different ways to use DataSynth.

## Interfaces

DataSynth offers three interfaces:

| Interface | Use Case |
|-----------|----------|
| **[CLI](cli-reference.md)** | Command-line generation, scripting, automation |
| **[Server API](server-api.md)** | REST/gRPC/WebSocket for applications |
| **[Desktop UI](desktop-ui.md)** | Visual configuration and monitoring |

## Quick Comparison

| Feature | CLI | Server | Desktop UI |
|---------|-----|--------|------------|
| Configuration editing | YAML files | API endpoints | Visual forms |
| Batch generation | Yes | Yes | Yes |
| Streaming generation | No | Yes | Yes (view) |
| Progress monitoring | Progress bar | WebSocket | Real-time |
| Scripting/automation | Yes | Yes | No |
| Visual feedback | Minimal | None | Full |

## CLI Overview

The command-line interface (`datasynth-data`) is ideal for:
- Batch generation
- CI/CD pipelines
- Scripting and automation
- Server environments

```bash
datasynth-data generate --config config.yaml --output ./output
```

## Server Overview

The server (`datasynth-server`) provides:
- REST API for configuration and control
- gRPC for high-performance integration
- WebSocket for real-time streaming

```bash
cargo run -p datasynth-server -- --port 3000
```

## Desktop UI Overview

The desktop application offers:
- Visual configuration editor
- Industry preset selector
- Real-time generation monitoring
- Cross-platform support (Windows, macOS, Linux)

```bash
cd crates/datasynth-ui && npm run tauri dev
```

## Output Formats

DataSynth produces various output formats:
- **CSV**: Standard tabular data
- **JSON**: Structured data with nested objects
- **ACDOCA**: SAP HANA Universal Journal format
- **PyTorch Geometric**: ML-ready graph tensors
- **Neo4j**: Graph database import format

See [Output Formats](output-formats.md) for details.

## Choosing an Interface

**Use the CLI if you:**
- Need to automate generation
- Work in headless/server environments
- Prefer command-line tools
- Want to integrate with shell scripts

**Use the Server if you:**
- Build applications that consume synthetic data
- Need streaming generation
- Want API-based control
- Integrate with microservices

**Use the Desktop UI if you:**
- Prefer visual configuration
- Want to explore options interactively
- Need real-time monitoring
- Are new to DataSynth

## Next Steps

- [CLI Reference](cli-reference.md) - Complete command documentation
- [Server API](server-api.md) - REST, gRPC, and WebSocket endpoints
- [Desktop UI](desktop-ui.md) - Desktop application guide
- [Output Formats](output-formats.md) - Detailed output file documentation
