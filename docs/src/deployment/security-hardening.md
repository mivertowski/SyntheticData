# Security Hardening

This guide provides a pre-deployment security checklist and detailed guidance on TLS, secrets management, container security, and audit logging for DataSynth.

## Pre-Deployment Checklist

Complete this checklist before exposing DataSynth to any network beyond localhost:

| # | Item | Priority | Status |
|---|------|----------|--------|
| 1 | Enable API key authentication | Critical | |
| 2 | Use strong, unique API keys (32+ chars) | Critical | |
| 3 | Enable TLS (direct or via reverse proxy) | Critical | |
| 4 | Set explicit CORS allowed origins | High | |
| 5 | Enable rate limiting | High | |
| 6 | Run as non-root user | High | |
| 7 | Use read-only root filesystem (container) | High | |
| 8 | Drop all Linux capabilities | High | |
| 9 | Set resource limits (memory, CPU, file descriptors) | High | |
| 10 | Restrict network exposure (firewall, security groups) | High | |
| 11 | Enable structured logging to a central log aggregator | Medium | |
| 12 | Set up Prometheus monitoring and alerts | Medium | |
| 13 | Rotate API keys periodically | Medium | |
| 14 | Review and restrict CORS origins | Medium | |
| 15 | Enable mTLS for gRPC if used in service mesh | Low | |

## Authentication Hardening

### Strong API Keys

Generate cryptographically strong API keys:

```bash
# Generate a 48-character random key
openssl rand -base64 36

# Example output: kZ9mR3xY7pQ2wV5nL8jH4cF6gT0aD1bE3sU9iO7
```

Recommendations:

- Minimum 32 characters, ideally 48+
- Use different keys per environment (dev, staging, production)
- Use different keys per client/team when possible
- Rotate keys quarterly or after any suspected compromise

### Argon2id Hashing

DataSynth hashes API keys with Argon2id (the recommended password hashing algorithm). Keys are hashed at startup; the plaintext is never stored in memory after hashing.

For pre-hashed keys (avoiding plaintext in environment variables), hash the key externally and pass the PHC-format hash:

```python
# Python example: pre-hash an API key
from argon2 import PasswordHasher

ph = PasswordHasher()
hash = ph.hash("your-api-key")
print(hash)
# $argon2id$v=19$m=65536,t=3,p=4$...
```

Pass the pre-hashed value to the server via the `AuthConfig::with_prehashed_keys()` API (for embedded use) or store in a secrets manager.

### API Key Rotation

To rotate keys without downtime:

1. Add the new key to `DATASYNTH_API_KEYS` alongside the old key.
2. Restart the server (rolling restart in K8s).
3. Update all clients to use the new key.
4. Remove the old key from `DATASYNTH_API_KEYS`.
5. Restart again.

## TLS Configuration

### Option 1: Reverse Proxy TLS (Recommended)

Terminate TLS at a reverse proxy (Nginx, Envoy, cloud load balancer) and forward plain HTTP to DataSynth. See [TLS & Reverse Proxy](tls-reverse-proxy.md) for full configurations.

Advantages:

- Centralized certificate management
- Standard renewal workflows (cert-manager, Let's Encrypt)
- Offloads TLS from the application
- Easier to audit and rotate certificates

### Option 2: Native TLS

Build DataSynth with TLS support:

```bash
cargo build --release -p datasynth-server --features tls
```

Run with certificate and key:

```bash
datasynth-server \
  --tls-cert /etc/datasynth/tls/cert.pem \
  --tls-key /etc/datasynth/tls/key.pem
```

### Certificate Requirements

| Requirement | Detail |
|-------------|--------|
| Format | PEM-encoded X.509 |
| Key type | RSA 2048+ or ECDSA P-256/P-384 |
| Protocol | TLS 1.2 or 1.3 (1.0/1.1 disabled) |
| Cipher suites | HIGH:!aNULL:!MD5 (Nginx default) |
| Subject Alternative Name | Must match the hostname clients use |

### mTLS for gRPC

For service-to-service communication, configure mutual TLS:

```nginx
# Nginx mTLS configuration
server {
    listen 50051 ssl http2;

    ssl_certificate /etc/ssl/certs/server.pem;
    ssl_certificate_key /etc/ssl/private/server-key.pem;

    # Client certificate verification
    ssl_client_certificate /etc/ssl/certs/ca.pem;
    ssl_verify_client on;

    location / {
        grpc_pass grpc://127.0.0.1:50051;
    }
}
```

## Secret Management

### Environment Variables

For simple deployments, store secrets in environment files with restricted permissions:

```bash
# Create the environment file
sudo install -m 640 -o root -g datasynth /dev/null /etc/datasynth/server.env

# Edit the file
sudo vi /etc/datasynth/server.env
```

Never commit plaintext secrets to version control. Use `.gitignore` to exclude env files.

### Kubernetes Secrets

For Kubernetes, store API keys in a Secret resource:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: datasynth-api-keys
  namespace: datasynth
type: Opaque
stringData:
  api-keys: "key-1,key-2"
```

### External Secrets Operator

For production, integrate with a secrets manager via the [External Secrets Operator](https://external-secrets.io/):

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: datasynth-api-keys
  namespace: datasynth
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: ClusterSecretStore
  target:
    name: datasynth-api-keys
  data:
    - secretKey: api-keys
      remoteRef:
        key: datasynth/api-keys
```

### HashiCorp Vault

Inject secrets via the Vault Agent sidecar:

```yaml
# Pod annotations for Vault Agent Injector
podAnnotations:
  vault.hashicorp.com/agent-inject: "true"
  vault.hashicorp.com/role: "datasynth"
  vault.hashicorp.com/agent-inject-secret-api-keys: "secret/data/datasynth/api-keys"
  vault.hashicorp.com/agent-inject-template-api-keys: |
    {{- with secret "secret/data/datasynth/api-keys" -}}
    {{ .Data.data.keys }}
    {{- end -}}
```

## Container Security

### Distroless Base Image

The production Dockerfile uses `gcr.io/distroless/cc-debian12`, which contains:

- No shell (`/bin/sh`, `/bin/bash`)
- No package manager
- No unnecessary system utilities
- Only the C runtime library and certificates

This minimizes the attack surface and prevents shell-based exploits.

### Security Context (Kubernetes)

The Helm chart enforces the following security context:

```yaml
podSecurityContext:
  runAsNonRoot: true        # Pod must run as non-root
  runAsUser: 1000           # UID 1000
  runAsGroup: 1000          # GID 1000
  fsGroup: 1000             # Filesystem group

securityContext:
  allowPrivilegeEscalation: false    # No setuid/setgid
  readOnlyRootFilesystem: true       # Read-only root FS
  capabilities:
    drop:
      - ALL                          # Drop all Linux capabilities
```

### SystemD Sandboxing

The SystemD unit file includes comprehensive sandboxing:

```ini
NoNewPrivileges=true          # Prevent privilege escalation
ProtectSystem=strict          # Read-only filesystem
ProtectHome=true              # Hide home directories
PrivateTmp=true               # Isolated /tmp
PrivateDevices=true           # No device access
ProtectKernelTunables=true    # No sysctl modification
ProtectKernelModules=true     # No module loading
ProtectControlGroups=true     # No cgroup modification
RestrictNamespaces=true       # No namespace creation
RestrictRealtime=true         # No realtime scheduling
RestrictSUIDSGID=true         # No SUID/SGID
```

### Image Scanning

Scan the container image for vulnerabilities before deployment:

```bash
# Trivy
trivy image datasynth/datasynth-server:0.5.0

# Grype
grype datasynth/datasynth-server:0.5.0

# Docker Scout
docker scout cves datasynth/datasynth-server:0.5.0
```

The distroless base image has a minimal CVE surface. Address any findings in the Rust dependencies via `cargo audit`:

```bash
cargo install cargo-audit
cargo audit
```

## Network Security

### Principle of Least Exposure

Only expose the ports and endpoints that clients need:

| Deployment | Expose REST (3000) | Expose gRPC (50051) | Expose Metrics |
|------------|-------------------|---------------------|----------------|
| Internal API only | Via Ingress/LB | Via Ingress/LB | Prometheus only |
| Public API | Via Ingress + WAF | No | No |
| Dev/staging | Localhost only | Localhost only | Localhost only |

### Network Policies (Kubernetes)

Restrict pod-to-pod communication:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: datasynth-allow-ingress
  namespace: datasynth
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: datasynth
  policyTypes:
    - Ingress
  ingress:
    # Allow from Ingress controller
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: ingress-nginx
      ports:
        - port: 3000
          protocol: TCP
    # Allow Prometheus scraping
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: monitoring
      ports:
        - port: 3000
          protocol: TCP
```

### CORS Lockdown

In production, override the default CORS configuration to allow only your application's domain:

```rust
// Programmatic configuration
let cors = CorsConfig {
    allowed_origins: vec![
        "https://app.example.com".to_string(),
    ],
    allow_any_origin: false,
};
```

Never enable `allow_any_origin: true` in production.

## Audit Logging

### Request Tracing

Every request receives an `X-Request-Id` header (auto-generated UUID v4 or client-provided). Use this to correlate logs across services.

### Structured Log Fields

DataSynth emits JSON-structured logs with the following fields useful for security auditing:

```json
{
  "timestamp": "2024-01-15T10:30:00.123Z",
  "level": "INFO",
  "target": "datasynth_server::rest::routes",
  "message": "Configuration update requested: industry=retail, period_months=6",
  "thread_id": 42
}
```

### Log Events to Monitor

| Event | Log Pattern | Severity |
|-------|-------------|----------|
| Authentication failure | `Unauthorized` / `Invalid API key` | High |
| Rate limit exceeded | `Rate limit exceeded` | Medium |
| Configuration change | `Configuration update requested` | Medium |
| Stream start/stop | `Stream started` / `Stream stopped` | Low |
| WebSocket connection | `WebSocket connected` / `disconnected` | Low |
| Server panic | `Server panic:` | Critical |

### Centralized Logging

Forward structured logs to a central aggregator:

**Docker:**

```yaml
services:
  datasynth-server:
    logging:
      driver: "fluentd"
      options:
        fluentd-address: "localhost:24224"
        tag: "datasynth.server"
```

**SystemD to Loki:**

```bash
# Install Promtail for journal forwarding
# /etc/promtail/config.yaml
scrape_configs:
  - job_name: datasynth
    journal:
      matches:
        - _SYSTEMD_UNIT=datasynth-server.service
      labels:
        job: datasynth
```

## RBAC (Kubernetes)

The Helm chart creates a ServiceAccount by default. Bind minimal permissions:

```yaml
serviceAccount:
  create: true
  automount: true   # Only if needed by the application
  annotations: {}
```

DataSynth does not require any Kubernetes API access. If `automount` is not needed, set it to `false` to prevent the ServiceAccount token from being mounted into the pod.

## Supply Chain Security

### Reproducible Builds

The Dockerfile uses pinned versions:

- `rust:1.82-bookworm` -- pinned Rust compiler version
- `gcr.io/distroless/cc-debian12` -- pinned distroless image
- `cargo-chef --locked` -- locked dependency resolution

### Dependency Auditing

```bash
# Check for known vulnerabilities
cargo audit

# Check for unmaintained or yanked crates
cargo audit --deny warnings
```

Run `cargo audit` in CI on every pull request.

### SBOM Generation

Generate a Software Bill of Materials for compliance:

```bash
# Using cargo-cyclonedx
cargo install cargo-cyclonedx
cargo cyclonedx --all

# Using syft for container images
syft datasynth/datasynth-server:0.5.0 -o cyclonedx-json > sbom.json
```
