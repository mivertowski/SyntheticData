# Deployment & Operations

This section covers everything you need to deploy, operate, and maintain DataSynth in production environments.

## Deployment Options

DataSynth supports three deployment models, each suited to different operational requirements:

| Method | Best For | Scaling | Complexity |
|--------|----------|---------|------------|
| [Docker / Compose](docker.md) | Small teams, dev/staging, single-node | Vertical | Low |
| [Kubernetes / Helm](kubernetes.md) | Production, multi-tenant, auto-scaling | Horizontal | Medium |
| [Bare Metal / SystemD](bare-metal.md) | Regulated environments, air-gapped networks | Vertical | Low |

## Architecture at a Glance

DataSynth server exposes two network interfaces:

- **REST API** on port 3000 -- configuration, bulk generation, streaming control, health probes, Prometheus metrics
- **gRPC API** on port 50051 -- high-throughput generation for programmatic clients

Both share an in-process `ServerState` with atomic counters, so a single process can serve REST, gRPC, and WebSocket clients concurrently.

## Operations Guides

| Guide | Description |
|-------|-------------|
| [Operational Runbook](runbook.md) | Grafana dashboards, alert response, troubleshooting, log analysis |
| [Capacity Planning](capacity-planning.md) | Sizing model, reference benchmarks, disk and memory estimates |
| [Disaster Recovery](disaster-recovery.md) | Backup procedures, deterministic replay, stateless restart |

## Security & API

| Guide | Description |
|-------|-------------|
| [API Reference](api-reference.md) | Endpoints, authentication, rate limiting, WebSocket protocol, error formats |
| [Security Hardening](security-hardening.md) | Pre-deployment checklist, TLS/mTLS, secrets, container security, audit logging |
| [TLS & Reverse Proxy](tls-reverse-proxy.md) | Nginx, Envoy, and native TLS configuration |

## Quick Decision Tree

1. **Need auto-scaling or HA?** -- Use [Kubernetes](kubernetes.md).
2. **Single server, want observability?** -- Use [Docker Compose](docker.md) with the full stack (Prometheus + Grafana).
3. **Air-gapped or compliance-restricted?** -- Use [Bare Metal](bare-metal.md) with SystemD.
