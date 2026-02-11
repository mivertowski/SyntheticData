# SOC 2 Type II Readiness

This document describes how DataSynth's architecture and controls align with the AICPA Trust Services Criteria (TSC) used in SOC 2 Type II engagements. DataSynth is a synthetic data generation tool, not a cloud-hosted SaaS product, so this assessment focuses on the controls embedded in the software itself rather than organizational policies. Organizations deploying DataSynth should layer their own operational controls (change management, personnel security, vendor management) on top of the technical controls described here.

## Assessment Scope

- **System**: DataSynth synthetic financial data generator
- **Version**: 0.5.x
- **Deployment Models**: CLI binary, REST/gRPC/WebSocket server, Python library, desktop application
- **Assessment Type**: Architecture readiness (pre-audit self-assessment)

---

## CC1: Security

The Security criterion (Common Criteria) requires that the system is protected against unauthorized access, both logical and physical.

### Authentication

DataSynth's server component (`datasynth-server`) implements two authentication mechanisms:

**API Key Authentication**: API keys are hashed with Argon2id (memory-hard, side-channel resistant) at server startup. Verification iterates all stored hashes without short-circuiting to prevent timing-based enumeration. A short-lived (5-second TTL) FNV-1a hash cache avoids repeated Argon2id computation for successive requests from the same client. Keys are accepted via `Authorization: Bearer <key>` or `X-API-Key` headers.

**JWT/OIDC** (optional `jwt` feature): External identity providers (Keycloak, Auth0, Entra ID) issue RS256-signed tokens. The `JwtValidator` verifies issuer, audience, expiration, and signature. Claims include subject, email, roles, and tenant ID for multi-tenancy.

### Authorization

Role-Based Access Control (RBAC) enforces least-privilege access:

| Role | GenerateData | ManageJobs | ViewJobs | ManageConfig | ViewConfig | ViewMetrics | ManageApiKeys |
|------|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Admin** | Y | Y | Y | Y | Y | Y | Y |
| **Operator** | Y | Y | Y | N | Y | Y | N |
| **Viewer** | N | N | Y | N | Y | Y | N |

RBAC can be disabled for development environments; when disabled, all authenticated requests are treated as Admin.

### Network Security

The security headers middleware injects the following headers on all server responses:

| Header | Value | Purpose |
|--------|-------|---------|
| `X-Content-Type-Options` | `nosniff` | Prevent MIME-type sniffing |
| `X-Frame-Options` | `DENY` | Prevent clickjacking |
| `Content-Security-Policy` | `default-src 'none'; frame-ancestors 'none'` | Restrict resource loading |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Limit referrer leakage |
| `Cache-Control` | `no-store` | Prevent caching of API responses |
| `X-XSS-Protection` | `0` | Defer to CSP (modern best practice) |

TLS termination is supported via reverse proxy (nginx, Caddy, Envoy) or Kubernetes ingress. CORS is configurable with allowlisted origins.

### Rate Limiting

Per-client rate limiting uses a sliding-window counter with configurable thresholds (requests per second, burst size). A Redis-backed rate limiter is available for multi-instance deployments (`redis` feature flag).

---

## CC2: Availability

The Availability criterion requires that the system is available for operation and use as committed.

### Graceful Degradation

The `DegradationController` in `datasynth-core` monitors memory, disk, and CPU utilization and applies progressive feature reduction:

| Level | Memory | Disk | CPU | Response |
|-------|--------|------|-----|----------|
| **Normal** | < 70% | > 1000 MB | < 80% | All features enabled, full batch sizes |
| **Reduced** | 70--85% | 500--1000 MB | 80--90% | Half batch sizes, skip data quality injection |
| **Minimal** | 85--95% | 100--500 MB | > 90% | Essential data only, no anomaly injection |
| **Emergency** | > 95% | < 100 MB | -- | Flush pending writes, terminate gracefully |

Auto-recovery with hysteresis (5% improvement required) allows the system to step back up one level at a time when resource pressure subsides.

### Resource Monitoring

- **Memory guard**: Reads `/proc/self/statm` (Linux) or `ps` (macOS) to track resident set size against configurable limits.
- **Disk guard**: Uses `statvfs` (Unix) or `GetDiskFreeSpaceExW` (Windows) to monitor available disk space in the output directory.
- **CPU monitor**: Tracks CPU utilization with auto-throttle at 0.95 threshold.
- **Resource guard**: Unified orchestration that combines all three monitors and drives the `DegradationController`.

### Graceful Shutdown

The server handles `SIGTERM` by stopping acceptance of new requests, waiting for in-flight requests to complete (with configurable timeout), and flushing pending output. The CLI supports `SIGUSR1` for pause/resume of generation runs.

### Health Endpoints

The following endpoints are exempt from authentication for infrastructure integration:

| Endpoint | Purpose |
|----------|---------|
| `/health` | General health check |
| `/ready` | Readiness probe (Kubernetes) |
| `/live` | Liveness probe (Kubernetes) |
| `/metrics` | Prometheus-compatible metrics |

---

## CC3: Processing Integrity

The Processing Integrity criterion requires that system processing is complete, valid, accurate, timely, and authorized.

### Deterministic Generation

DataSynth uses the ChaCha8 cryptographically secure pseudo-random number generator with a configurable seed. Given the same configuration YAML and seed value, output is byte-identical across runs and platforms. This provides auditability (reproduce any dataset from its configuration) and regression detection (compare output hashes after code changes).

### Quality Gates

The evaluation framework (`datasynth-eval`) applies configurable pass/fail criteria to every generation run. Built-in quality gate profiles provide three levels of strictness:

| Metric | Strict | Default | Lenient |
|--------|--------|---------|---------|
| Benford MAD | <= 0.01 | <= 0.015 | <= 0.03 |
| Balance Coherence | >= 0.999 | >= 0.99 | >= 0.95 |
| Document Chain Integrity | >= 0.95 | >= 0.90 | >= 0.80 |
| Completion Rate | >= 0.99 | >= 0.95 | >= 0.90 |
| Duplicate Rate | <= 0.001 | <= 0.01 | <= 0.05 |
| Referential Integrity | >= 0.999 | >= 0.99 | >= 0.95 |
| IC Match Rate | >= 0.99 | >= 0.95 | >= 0.85 |
| Privacy MIA AUC | <= 0.55 | <= 0.60 | <= 0.70 |

Gate evaluation supports fail-fast (stop on first failure) and collect-all (report all failures) strategies.

### Balance Validation

The `JournalEntry` model enforces debits = credits at construction time. An entry that does not balance cannot be created, eliminating an entire class of data integrity errors.

### Content Marking

EU AI Act Article 50 synthetic content credentials are embedded in all output files (CSV headers, JSON metadata, Parquet file metadata). This prevents synthetic data from being mistaken for real financial records. Content marking is enabled by default.

---

## CC4: Confidentiality

The Confidentiality criterion requires that information designated as confidential is protected as committed.

### No Real Data Storage

In the default operating mode (pure synthetic generation), DataSynth does not process, store, or transmit real data. All names, identifiers, transactions, and addresses are algorithmically generated from configuration parameters and RNG output.

### Fingerprint Privacy

When the fingerprint extraction workflow processes real data, the following privacy controls apply:

| Mechanism | Default (Standard Level) |
|-----------|--------------------------|
| Differential privacy (Laplace) | Epsilon = 1.0, Delta = 1e-5 |
| K-anonymity suppression | K >= 5 |
| Composition accounting | Naive (Renyi DP, zCDP available) |

The output `.dsf` fingerprint file contains only aggregate statistics (means, variances, correlations), not individual records.

### API Key Security

API keys are never stored in plaintext. At server startup, raw keys are hashed with Argon2id (random salt, PHC format) and discarded. Verification uses Argon2id comparison that iterates all stored hashes to prevent timing-based key enumeration.

### Audit Logging

The `JsonAuditLogger` emits structured JSON audit events via the `tracing` crate. Each event records timestamp, request ID, actor identity (user ID or API key hash prefix), action, resource, outcome (success/denied/error), tenant ID, source IP, and user agent. Events are suitable for SIEM ingestion.

---

## CC5: Privacy

The Privacy criterion requires that personal information is collected, used, retained, disclosed, and disposed of in conformity with commitments.

### Synthetic Data by Design

DataSynth's default mode generates purely synthetic data. No personal information is collected or processed. Generated entities (vendors, customers, employees) have no real-world counterparts. This eliminates most privacy obligations for pure synthetic workflows.

### Privacy Evaluation

The evaluation framework includes empirical privacy testing:

- **Membership Inference Attack (MIA)**: Distance-based classifier measures AUC-ROC. A score near 0.50 indicates the synthetic data does not memorize real data patterns.
- **Linkage Attack Assessment**: Evaluates re-identification risk using quasi-identifier combinations. Measures achieved k-anonymity and unique QI overlap.

### NIST SP 800-226 Alignment

The evaluation framework generates NIST SP 800-226 alignment reports assessing data transformation adequacy, re-identification risk, documentation completeness, and privacy control effectiveness. An overall alignment score of >= 71% is required for a passing grade.

### Fingerprint Extraction Privacy Levels

| Level | Epsilon | Delta | K-Anonymity | Use Case |
|-------|---------|-------|-------------|----------|
| `minimal` | 10.0 | 1e-3 | 2 | Non-sensitive aggregates |
| `standard` | 1.0 | 1e-5 | 5 | General business data |
| `high` | 0.5 | 1e-6 | 10 | Sensitive financial data |
| `maximum` | 0.1 | 1e-8 | 20 | Regulated personal data |

---

## Controls Mapping

The following table maps DataSynth features to SOC 2 Trust Services Criteria identifiers.

| TSC ID | Criterion | DataSynth Control | Implementation |
|--------|-----------|-------------------|----------------|
| CC6.1 | Logical access security | API key authentication | `auth.rs`: Argon2id hashing, timing-safe comparison |
| CC6.1 | Logical access security | JWT/OIDC support | `auth.rs`: RS256 token validation (optional `jwt` feature) |
| CC6.3 | Role-based access | RBAC enforcement | `rbac.rs`: Admin/Operator/Viewer roles with permission matrix |
| CC6.6 | System boundaries | Security headers | `security_headers.rs`: CSP, X-Frame-Options, HSTS support |
| CC6.6 | System boundaries | Rate limiting | `rate_limit.rs`: Per-client sliding window, Redis backend |
| CC6.8 | Transmission security | TLS support | Reverse proxy TLS termination, Kubernetes ingress |
| CC7.2 | Monitoring | Resource guards | `resource_guard.rs`: CPU, memory, disk monitoring |
| CC7.2 | Monitoring | Audit logging | `audit.rs`: Structured JSON events for SIEM |
| CC7.3 | Change detection | Config hashing | SHA-256 hash of configuration embedded in output |
| CC7.4 | Incident response | Content marking | Content credentials identify synthetic origin |
| CC8.1 | Processing integrity | Deterministic RNG | ChaCha8 with configurable seed |
| CC8.1 | Processing integrity | Quality gates | `gates/engine.rs`: Configurable pass/fail thresholds |
| CC8.1 | Processing integrity | Balance validation | `JournalEntry` enforces debits = credits at construction |
| CC9.1 | Availability management | Graceful degradation | `degradation.rs`: Normal/Reduced/Minimal/Emergency levels |
| CC9.1 | Availability management | Health endpoints | `/health`, `/ready`, `/live` (auth-exempt) |
| P3.1 | Privacy notice | Synthetic content marking | EU AI Act Article 50 credentials in all output |
| P4.1 | Collection limitation | No real data by default | Pure synthetic generation requires no data collection |
| P6.1 | Data quality | Quality gates | Statistical, coherence, and privacy quality metrics |
| P8.1 | Disposal | Deterministic generation | No persistent state; regenerate from config + seed |

---

## Gap Analysis

The following areas require organizational controls that are outside DataSynth's software scope:

| Area | Recommendation |
|------|----------------|
| Physical security | Deploy on infrastructure with appropriate physical access controls |
| Change management | Implement CI/CD pipelines with code review and approval workflows |
| Vendor management | Assess third-party dependencies via `cargo audit` and SBOM generation |
| Personnel security | Apply organizational onboarding/offboarding procedures for API key management |
| Backup and recovery | Configure backup for generation configurations and output data per retention policies |
| Incident response plan | Document procedures for scenarios where synthetic data is mistakenly treated as real |

## See Also

- [ISO 27001 Alignment](iso27001.md)
- [Security Hardening](../deployment/security-hardening.md)
- [NIST AI RMF Self-Assessment](nist-ai-rmf.md)
- [GDPR Compliance](gdpr.md)
- [EU AI Act Compliance](eu-ai-act.md)
