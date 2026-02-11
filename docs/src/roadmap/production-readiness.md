# Production Readiness Roadmap

> **Version**: 1.0 | **Date**: February 2026 | **Status**: Living Document

This roadmap addresses the infrastructure, operations, security, compliance, and ecosystem maturity required to transition DataSynth from a feature-complete beta to a production-grade enterprise platform. It complements the existing [feature roadmap](README.md) which covers domain-specific enhancements.

---

## Table of Contents

- [Current State Assessment](#current-state-assessment)
- [Phase 1: Foundation (0-3 months)](#phase-1-foundation-0-3-months)
- [Phase 2: Hardening (3-6 months)](#phase-2-hardening-3-6-months)
- [Phase 3: Enterprise Grade (6-12 months)](#phase-3-enterprise-grade-6-12-months)
- [Phase 4: Market Leadership (12-18 months)](#phase-4-market-leadership-12-18-months)
- [Industry & Research Context](#industry--research-context)
- [Competitive Positioning](#competitive-positioning)
- [Regulatory Landscape](#regulatory-landscape)
- [Risk Register](#risk-register)

---

## Current State Assessment

### Production Readiness Scorecard (v0.5.0 — Phase 2 Complete)

| Category | Score | Status | Key Findings |
|----------|-------|--------|--------------|
| **Workspace Structure** | 9/10 | Excellent | 15 well-organized crates, clear separation of concerns |
| **Testing** | 10/10 | Excellent | 2,500+ tests, property testing via proptest, fuzzing harnesses (cargo-fuzz), k6 load tests, coverage via cargo-llvm-cov + Codecov |
| **CI/CD** | 9/10 | Excellent | 7-job pipeline: fmt, clippy, cross-platform test (Linux/macOS/Windows), MSRV 1.88, security scanning (cargo-deny + cargo-audit), coverage, benchmark regression |
| **Error Handling** | 10/10 | Excellent | Idiomatic `thiserror`/`anyhow`; `#![deny(clippy::unwrap_used)]` enforced across all library crates; zero unwrap calls in non-test code |
| **Observability** | 9/10 | Excellent | Structured JSON logging, feature-gated OpenTelemetry (OTLP traces + Prometheus metrics), request ID propagation, request logging middleware, data lineage graph |
| **Deployment** | 10/10 | Excellent | Multi-stage Dockerfile (distroless), Docker Compose, Kubernetes Helm chart (HPA, PDB, Redis subchart), SystemD service, comprehensive deployment guides (Docker, K8s, bare-metal) |
| **Security** | 9/10 | Excellent | Argon2id key hashing with timing-safe comparison, security headers, request validation, TLS support (rustls), env var interpolation for secrets, cargo-deny + cargo-audit in CI, security hardening guide |
| **Performance** | 9/10 | Excellent | 5 Criterion benchmark suites, 100K+ entries/sec; CI benchmark regression tracking on PRs; k6 load testing framework |
| **Python Bindings** | 8/10 | Strong | Strict mypy, PEP 561 compliant, blueprints; classified as "Beta", no async support |
| **Server** | 10/10 | Excellent | REST/gRPC/WebSocket complete; async job queue; distributed rate limiting (Redis); stateless config loading; enhanced probes; full middleware stack |
| **Documentation** | 10/10 | Excellent | mdBook + rustdoc + CHANGELOG + CONTRIBUTING; deployment guides (Docker, K8s, bare-metal), operational runbook, capacity planning, DR procedures, API reference, security hardening |
| **Code Quality** | 10/10 | Excellent | Zero TODO/FIXME comments, warnings-as-errors enforced, panic-free library crates, 6 unsafe blocks (all justified) |
| **Privacy** | 9/10 | Excellent | Formal DP composition (RDP, zCDP), privacy budget management, MIA/linkage evaluation, NIST SP 800-226 alignment, SynQP matrix, custom privacy levels |
| **Data Lineage** | 9/10 | Excellent | Per-file checksums, lineage graph, W3C PROV-JSON export, CLI verify command for manifest integrity |

**Overall: 9.4/10** — Enterprise-grade with Kubernetes deployment, formal privacy guarantees, panic-free library code, comprehensive operations documentation, and data lineage tracking. Remaining gaps: RBAC/OAuth2, plugin SDK, Python async support.

---

## Phase 1: Foundation (0-3 months)

*Goal: Establish the minimum viable production infrastructure.*

### 1.1 Containerization & Packaging

**Priority: Critical** | **Effort: Medium**

| Deliverable | Description |
|-------------|-------------|
| Multi-stage Dockerfile | Rust builder stage + distroless/alpine runtime (~20MB image) |
| Docker Compose | Local dev stack: server + Prometheus + Grafana + Redis |
| OCI image publishing | GitHub Actions workflow to push to GHCR/ECR on tagged releases |
| Binary distribution | Pre-built binaries for Linux (x86_64, aarch64), macOS (Apple Silicon), Windows |
| SystemD service file | Production daemon configuration with resource limits |

**Implementation Notes:**
```
# Target image structure
FROM rust:1.88-bookworm AS builder
# ... build with --release
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/datasynth-server /
EXPOSE 3000
ENTRYPOINT ["/datasynth-server"]
```

### 1.2 Security Hardening

**Priority: Critical** | **Effort: Medium**

| Deliverable | Description |
|-------------|-------------|
| API key hashing | Argon2id for stored keys; timing-safe comparison via `subtle` crate |
| Request validation middleware | Content-Type enforcement, configurable max body size (default 10MB) |
| TLS support | Native `rustls` integration or documented reverse proxy (nginx/Caddy) setup |
| Secrets management | Environment variable interpolation in config (`${ENV_VAR}` syntax) |
| Security headers | `X-Content-Type-Options`, `X-Frame-Options`, `Strict-Transport-Security` |
| Input sanitization | Validate all user-supplied config values before processing |
| Dependency auditing | `cargo-audit` and `cargo-deny` in CI pipeline |

### 1.3 Observability Stack

**Priority: Critical** | **Effort: Medium**

| Deliverable | Description |
|-------------|-------------|
| OpenTelemetry integration | Replace custom metrics with `opentelemetry` + `opentelemetry-otlp` crates |
| Structured logging | JSON-formatted logs with request IDs, span context, correlation traces |
| Prometheus metrics | Generation throughput, latency histograms, error rates, resource utilization |
| Distributed tracing | Trace generation pipeline phases end-to-end with span hierarchy |
| Health check enhancement | Add dependency checks (disk space, memory) to `/ready` endpoint |
| Alert rules | Example Prometheus alerting rules for SLO violations |

**Key Metrics to Instrument:**
- `datasynth_generation_entries_total` (Counter) — Total entries generated
- `datasynth_generation_duration_seconds` (Histogram) — Per-phase latency
- `datasynth_generation_errors_total` (Counter) — Errors by type
- `datasynth_memory_usage_bytes` (Gauge) — Current memory consumption
- `datasynth_active_sessions` (Gauge) — Concurrent generation sessions
- `datasynth_api_request_duration_seconds` (Histogram) — API latency by endpoint

### 1.4 CI/CD Hardening

**Priority: High** | **Effort: Low**

| Deliverable | Description |
|-------------|-------------|
| Code coverage | `cargo-tarpaulin` or `cargo-llvm-cov` with Codecov integration |
| Security scanning | `cargo-audit` for CVEs, `cargo-deny` for license compliance |
| MSRV validation | CI job testing against minimum supported Rust version (1.88) |
| Cross-platform matrix | Test on Linux, macOS, Windows in CI |
| Benchmark tracking | Criterion results uploaded to GitHub Pages; regression alerts on PRs |
| Release automation | Semantic versioning with auto-changelog via `git-cliff` |
| Container scanning | Trivy or Grype scanning of published Docker images |

---

## Phase 2: Hardening (3-6 months)

*Goal: Enterprise-grade reliability, scalability, and compliance foundations.*

### 2.1 Scalability & High Availability

**Priority: High** | **Effort: High**

| Deliverable | Description |
|-------------|-------------|
| Redis-backed rate limiting | Distributed rate limiting via `redis-rs` for multi-instance deployments |
| Horizontal scaling | Stateless server design; shared config via Redis/S3 |
| Kubernetes Helm chart | Production-ready chart with HPA, PDB, resource limits, readiness probes |
| Load testing framework | k6 or Locust scripts for API stress testing |
| Graceful rolling updates | Zero-downtime deployments with connection draining |
| Job queue | Async generation jobs with status tracking (Redis Streams or similar) |

### 2.2 Data Lineage & Provenance

**Priority: High** | **Effort: Medium**

| Deliverable | Description |
|-------------|-------------|
| Generation manifest | JSON/YAML file recording: config hash, seed, version, timestamp, checksums for all outputs |
| Data lineage graph | Track which config section produced which output file and row ranges |
| Reproducibility verification | CLI command: `datasynth-data verify --manifest manifest.json --output ./output/` |
| W3C PROV compatibility | Export lineage in W3C PROV-JSON format for interoperability |
| Audit trail | Append-only log of all generation runs with user, config, and output metadata |

**Rationale:** Data lineage is becoming a regulatory requirement under the EU AI Act (Article 10 — data governance for training data) and is a key differentiator in the enterprise synthetic data market. NIST AI RMF 1.0 also emphasizes provenance tracking under its MAP and MEASURE functions.

### 2.3 Enhanced Privacy Guarantees

**Priority: High** | **Effort: High**

| Deliverable | Description |
|-------------|-------------|
| Formal DP accounting | Implement Renyi DP and zero-concentrated DP (zCDP) composition tracking |
| Privacy budget management | Global budget tracking across multiple generation runs |
| Membership inference testing | Automated MIA evaluation as post-generation quality gate |
| NIST SP 800-226 alignment | Validate DP implementation against NIST Guidelines for Evaluating DP Guarantees |
| SynQP framework integration | Implement the IEEE SynQP evaluation matrix for joint quality-privacy assessment |
| Configurable privacy levels | Presets: `relaxed` (ε=10), `standard` (ε=1), `strict` (ε=0.1) with utility tradeoff documentation |

**Research Context:** The NIST SP 800-226 (Guidelines for Evaluating Differential Privacy Guarantees) provides the authoritative framework for DP evaluation. The SynQP framework (IEEE, 2025) introduces standardized privacy-quality evaluation matrices. Benchmarking DP tabular synthesis algorithms was a key topic at TPDP 2025, and federated DP approaches (FedDPSyn) are emerging for distributed generation.

### 2.4 Unwrap Audit & Robustness

**Priority: Medium** | **Effort: Medium**

| Deliverable | Description |
|-------------|-------------|
| Unwrap elimination | Audit and replace ~2,300 `unwrap()` calls in non-test code with proper error handling |
| Panic-free guarantee | Add `#![deny(clippy::unwrap_used)]` lint for library crates (not test/bench) |
| Fuzzing harnesses | `cargo-fuzz` targets for config parsing, fingerprint loading, and API endpoints |
| Property test expansion | Increase `proptest` coverage for statistical invariants and balance coherence |

### 2.5 Documentation: Operations

**Priority: Medium** | **Effort: Low**

| Deliverable | Description |
|-------------|-------------|
| Deployment guide | Docker, K8s, bare-metal deployment with step-by-step instructions |
| Operational runbook | Monitoring dashboards, common alerts, troubleshooting procedures |
| Capacity planning guide | Memory/CPU/disk sizing for different generation scales |
| Disaster recovery | Backup/restore procedures for server state and configurations |
| API rate limits documentation | Document auth, rate limiting, and CORS behavior for integrators |
| Security hardening guide | Checklist for production security configuration |

---

## Phase 3: Enterprise Grade (6-12 months)

*Goal: Enterprise features, compliance certifications, and ecosystem maturity.*

### 3.1 Multi-Tenancy & Access Control

**Priority: High** | **Effort: High**

| Deliverable | Description |
|-------------|-------------|
| RBAC | Role-based access control (admin, operator, viewer) with JWT/OAuth2 |
| Tenant isolation | Namespace-based isolation for multi-tenant SaaS deployment |
| Audit logging | Structured audit events for all API actions (who/what/when) |
| SSO integration | SAML 2.0 and OIDC support for enterprise identity providers |
| API versioning | URL-based API versioning (v1, v2) with deprecation lifecycle |

### 3.2 Advanced Evaluation & Quality Gates

**Priority: High** | **Effort: Medium**

| Deliverable | Description |
|-------------|-------------|
| Automated quality gates | Pre-configured pass/fail criteria for generation runs |
| Benchmark suite expansion | Domain-specific benchmarks: financial realism, fraud detection efficacy, audit trail coherence |
| Regression testing | Golden dataset comparison with tolerance thresholds |
| Quality dashboard | Web-based visualization of quality metrics over time |
| Third-party validation | Integration with SDMetrics and SDV evaluation utilities |

**Quality Metrics to Implement:**
- **Statistical fidelity**: Column distribution similarity (KL divergence, Wasserstein distance)
- **Structural fidelity**: Correlation matrix preservation, inter-table referential integrity
- **Privacy**: Nearest-neighbor distance ratio, attribute disclosure risk, identity disclosure risk (SynQP)
- **Utility**: Train-on-synthetic-test-on-real (TSTR) ML performance parity
- **Temporal fidelity**: Autocorrelation preservation, seasonal pattern retention
- **Domain-specific**: Benford compliance MAD, balance equation coherence, document chain integrity

### 3.3 Plugin & Extension SDK

**Priority: Medium** | **Effort: High**

| Deliverable | Description |
|-------------|-------------|
| Generator trait API | Stable, documented trait interface for custom generators |
| Plugin loading | Dynamic plugin loading via `libloading` or WASM runtime |
| Template marketplace | Repository of community-contributed industry templates |
| Custom output sinks | Plugin API for custom export formats (database write, S3, GCS) |
| Webhook system | Event-driven notifications (generation start/complete/error) |

### 3.4 Python Ecosystem Maturity

**Priority: Medium** | **Effort: Medium**

| Deliverable | Description |
|-------------|-------------|
| Async support | `asyncio`-compatible API using `websockets` for streaming |
| Conda package | Publish to conda-forge for data science workflows |
| Jupyter integration | Example notebooks for common use cases (fraud ML, audit analytics) |
| pandas/polars integration | Direct DataFrame output without intermediate CSV |
| PyPI 1.0.0 release | Promote from Beta to Production/Stable classifier |
| Type stubs | Complete `.pyi` stubs for IDE support |

### 3.5 Regulatory Compliance Framework

**Priority: Medium** | **Effort: Medium**

| Deliverable | Description |
|-------------|-------------|
| EU AI Act readiness | Synthetic content marking (Article 50), training data documentation (Article 10) |
| NIST AI RMF alignment | Self-assessment against MAP, MEASURE, MANAGE, GOVERN functions |
| SOC 2 Type II preparation | Document controls for security, availability, processing integrity |
| GDPR compliance documentation | Data processing documentation, privacy impact assessment template |
| ISO 27001 alignment | Information security management system controls mapping |

**Regulatory Context:** The EU AI Act's Article 50 transparency obligations (enforceable August 2026) require AI systems generating synthetic content to mark outputs as artificially generated in a machine-readable format. Article 10 mandates training data governance including documentation of data sources. Organizations face penalties up to €35M or 7% of global turnover for non-compliance. The NIST AI RMF 1.0 (expanded significantly through 2024-2025) provides the voluntary framework becoming the "operational layer" beneath regulatory compliance globally.

---

## Phase 4: Market Leadership (12-18 months)

*Goal: Cutting-edge capabilities informed by latest research, establishing DataSynth as the reference platform for financial synthetic data.*

### 4.1 LLM-Augmented Generation

**Priority: Medium** | **Effort: High**

| Deliverable | Description |
|-------------|-------------|
| LLM-guided metadata enrichment | Use LLMs to generate realistic vendor names, descriptions, memo fields |
| Natural language config | Generate YAML configs from natural language descriptions ("Generate 1 year of manufacturing data for a mid-size German company") |
| Semantic constraint validation | LLM-based validation of inter-column logical relationships |
| Explanation generation | Natural language explanations for anomaly labels and findings |

**Research Context:** Multiple 2025 papers demonstrate LLM-augmented tabular data generation. LLM-TabFlow (March 2025) addresses preserving inter-column logical relationships. StructSynth (August 2025) focuses on structure-aware synthesis in low-data regimes. LLM-TabLogic (August 2025) uses prompt-guided latent diffusion to maintain logical constraints. The CFA Institute's July 2025 report on "Synthetic Data in Investment Management" validates the growing importance of synthetic data in financial applications.

### 4.2 Diffusion Model Integration

**Priority: Medium** | **Effort: Very High**

| Deliverable | Description |
|-------------|-------------|
| TabDDPM backend | Optional diffusion-model-based generation for learned distribution capture |
| FinDiff integration | Financial-domain diffusion model for learned financial patterns |
| Hybrid generation | Combine rule-based generators with learned models for maximum fidelity |
| Model fine-tuning pipeline | Train custom diffusion models on fingerprint data |
| Imb-FinDiff for rare events | Diffusion-based class imbalance handling for fraud patterns |

**Research Context:** The diffusion model landscape for tabular data has matured rapidly. TabDiff (ICLR 2025) introduced joint continuous-time diffusion with feature-wise learnable schedules, achieving 22.5% improvement over prior SOTA. FinDiff and its extensions (Imb-FinDiff for class imbalance, DP-Fed-FinDiff for federated privacy-preserving generation) are specifically designed for financial tabular data. A comprehensive survey (February 2025) catalogs 15+ diffusion models for tabular data. TabGraphSyn (December 2025) combines GNNs with diffusion for graph-guided tabular synthesis.

### 4.3 Advanced Privacy Techniques

**Priority: Medium** | **Effort: High**

| Deliverable | Description |
|-------------|-------------|
| Federated fingerprinting | Extract fingerprints from distributed data sources without centralization |
| Synthetic data certificates | Cryptographic proof that output satisfies DP guarantees |
| Privacy-utility Pareto frontier | Automated exploration of optimal ε values for given utility targets |
| Surrogate public data | Support for surrogate public data approaches to improve DP utility |

**Research Context:** TPDP 2025 featured FedDPSyn for federated DP tabular synthesis and research on surrogate public data for DP (Hod et al.). The AI-generated synthetic tabular data market reached $1.36B in 2024 and is projected to reach $6.73B by 2029 (37.9% CAGR), driven by privacy regulation and AI training demand.

### 4.4 Ecosystem & Integration

**Priority: Medium** | **Effort: Medium**

| Deliverable | Description |
|-------------|-------------|
| Terraform provider | Infrastructure-as-code for DataSynth server deployment |
| Airflow/Dagster operators | Pipeline integration for automated generation in data workflows |
| dbt integration | Generate synthetic data as dbt sources for analytics testing |
| Spark connector | Read DataSynth output directly as Spark DataFrames |
| MLflow integration | Track generation runs as MLflow experiments with metrics |

### 4.5 Causal & Counterfactual Generation

**Priority: Low** | **Effort: Very High**

| Deliverable | Description |
|-------------|-------------|
| Causal graph specification | Define causal relationships between entities in config |
| Interventional generation | "What-if" scenarios: generate data under hypothetical interventions |
| Counterfactual samples | Generate counterfactual versions of existing records |
| Causal discovery validation | Validate that generated data preserves specified causal structure |

---

## Industry & Research Context

### Synthetic Data Market (2025-2026)

The synthetic data market is experiencing explosive growth:

- **Gartner predicts** 75% of businesses will use GenAI to create synthetic customer data by 2026, up from <5% in 2023.
- The **AI-generated synthetic tabular data** market reached $1.36B in 2024, projected to $6.73B by 2029 (37.9% CAGR).
- Synthetic data is predicted to account for >60% of all training data for GenAI models by 2030 (CFA Institute, July 2025).

### Key Research Papers & Developments

#### Tabular Data Generation
- **TabDiff** (ICLR 2025) — Mixed-type diffusion with learnable feature-wise schedules; 22.5% improvement on correlation preservation
- **LLM-TabFlow** (March 2025) — Preserving inter-column logical relationships via LLM guidance
- **StructSynth** (August 2025) — Structure-aware LLM synthesis for low-data regimes
- **LLM-TabLogic** (August 2025) — Prompt-guided latent diffusion maintaining logical constraints
- **TabGraphSyn** (December 2025) — Graph-guided latent diffusion combining VAE+GNN with diffusion

#### Financial Domain
- **FinDiff** (ICAIF 2023) — Diffusion models for financial tabular data
- **Imb-FinDiff** (ICAIF 2024) — Conditional diffusion for class-imbalanced financial data
- **DP-Fed-FinDiff** — Federated DP diffusion for privacy-preserving financial synthesis
- **CFA Institute Report** (July 2025) — "Synthetic Data in Investment Management" validating FinDiff as SOTA

#### Privacy & Evaluation
- **SynQP** (IEEE, 2025) — Standardized quality-privacy evaluation framework for synthetic data
- **NIST SP 800-226** — Guidelines for Evaluating Differential Privacy Guarantees
- **TPDP 2025** — Benchmarking DP tabular synthesis; federated approaches; membership inference attacks
- **Consensus Privacy Metrics** (Pilgram et al., 2025) — Framework for standardized privacy evaluation

#### Surveys
- **"Diffusion Models for Tabular Data"** (February 2025) — Comprehensive survey cataloging 15+ models
- **"Comprehensive Survey of Synthetic Tabular Data Generation"** (Shi et al., 2025) — Broad overview of methods

### Technology Trends Impacting DataSynth

| Trend | Impact | Timeframe |
|-------|--------|-----------|
| LLM-augmented generation | Realistic metadata, natural language config | 2026 |
| Diffusion models for tabular data | Learned distribution capture as alternative/complement to rule-based | 2026-2027 |
| Federated DP synthesis | Generate from distributed sources without centralization | 2027 |
| Causal modeling | "What-if" scenarios and interventional generation | 2027-2028 |
| OTEL standardization | Unified observability across Rust ecosystem | 2026 |
| WASM plugins | Safe, sandboxed extensibility for custom generators | 2026-2027 |
| EU AI Act enforcement | Mandatory synthetic content marking and data governance | August 2026 |

---

## Competitive Positioning

### Market Landscape (2025-2026)

| Platform | Focus | Key Differentiator | Pricing | Status |
|----------|-------|--------------------|---------|--------|
| **Gretel.ai** | Developer APIs | Navigator (NL-to-data); acquired by NVIDIA (March 2025) | Usage-based | Integrated into NVIDIA NeMo |
| **MOSTLY AI** | Enterprise compliance | TabularARGN with built-in DP; fairness controls | Enterprise license | Independent |
| **Tonic.ai** | Test data management | Database-aware synthesis; acquired Fabricate (April 2025) | Per-database | Growing |
| **Hazy** | Financial services | Regulated-sector focus; sequential data | Enterprise license | Independent |
| **SDV/DataCebo** | Open source ecosystem | CTGAN, TVAEs, Gaussian copulas; Python-native | Freemium | Open source core |
| **K2view** | Entity-based testing | All-in-one enterprise data management | Enterprise license | Established |

### DataSynth Competitive Advantages

| Advantage | Detail |
|-----------|--------|
| **Domain depth** | Deepest financial/accounting domain model (IFRS, US GAAP, ISA, SOX, COSO, KYC/AML) |
| **Rule-based coherence** | Guaranteed balance equations, document chain integrity, three-way matching |
| **Deterministic reproducibility** | ChaCha8 RNG with seed control; bit-exact reproducibility across runs |
| **Performance** | 100K+ entries/sec (Rust native); 10-100x faster than Python-based competitors |
| **Privacy-preserving fingerprinting** | Unique extract-synthesize workflow with DP guarantees |
| **Process mining** | Native OCEL 2.0 event log generation (unique in market) |
| **Graph-native** | Direct PyTorch Geometric, Neo4j, DGL export for GNN workflows |
| **Full-stack** | CLI + REST/gRPC/WebSocket server + Desktop UI + Python bindings |

### Competitive Gaps to Address

| Gap | Competitors with Feature | Priority |
|-----|--------------------------|----------|
| Cloud-hosted SaaS offering | Gretel, MOSTLY AI, Tonic | Phase 3 |
| No-code UI for non-technical users | MOSTLY AI, K2view | Phase 3 |
| Database-aware synthesis from production data | Tonic.ai | Phase 4 |
| LLM-powered natural language interface | Gretel Navigator | Phase 4 |
| Pre-built ML model training pipelines | Gretel | Phase 3 |
| Marketplace for community templates | SDV ecosystem | Phase 3 |

---

## Regulatory Landscape

### EU AI Act Timeline

| Date | Milestone | DataSynth Impact |
|------|-----------|------------------|
| Feb 2025 | Prohibited AI systems discontinued; AI literacy obligations | Low — DataSynth is a tool, not a prohibited system |
| Aug 2025 | GPAI transparency requirements; training data documentation | Medium — Users training AI with DataSynth output need provenance |
| **Aug 2026** | **Full high-risk AI compliance; Article 50 transparency** | **High — Synthetic content marking required; data governance mandated** |
| Aug 2027 | High-risk AI in harmonized products | Low — Indirect impact |

### Required Compliance Features

1. **Synthetic content marking** (Article 50): All generated data must include machine-readable markers indicating artificial generation
2. **Training data documentation** (Article 10): Generation manifests must document configs, sources, and processing steps
3. **Quality management** (Annex IV): Documented quality assurance processes for generation and evaluation
4. **Risk assessment**: Template for users to assess risks of using synthetic data in AI systems

### Other Regulatory Frameworks

| Framework | Relevance | Status |
|-----------|-----------|--------|
| **NIST AI RMF 1.0** | Voluntary; becoming the operational governance layer globally | Self-assessment planned (Phase 3) |
| **NIST SP 800-226** | DP evaluation guidelines | Alignment planned (Phase 2) |
| **GDPR** | Synthetic data reduces but doesn't eliminate privacy obligations | Documentation in Phase 3 |
| **SOX** | DataSynth already generates SOX-compliant test data | Feature complete |
| **ISO 27001** | Information security controls for server deployment | Alignment in Phase 3 |
| **SOC 2 Type II** | Trust service criteria for SaaS offering | Phase 3 preparation |

---

## Risk Register

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Performance regression with OTEL instrumentation | Medium | Medium | Benchmark-gated CI; sampling in production |
| Breaking API changes during versioning | Low | High | Semantic versioning; deprecation policy; compatibility tests |
| Memory safety issues in unsafe blocks | Low | Critical | Miri testing; minimize unsafe; regular audits |
| Dependency CVEs | Medium | High | `cargo-audit` in CI; Dependabot alerts |
| Plugin system security (WASM/dynamic loading) | Medium | High | WASM sandboxing; capability-based permissions |

### Business Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| EU AI Act scope broader than anticipated | Medium | High | Proactive Article 50 compliance; legal review |
| Competitor acqui-hires (Gretel→NVIDIA pattern) | Medium | Medium | Build unique domain depth as defensible moat |
| Open-source competitors (SDV) closing feature gap | Medium | Medium | Focus on financial domain depth and performance |
| Enterprise customers requiring SOC 2 certification | High | Medium | Begin SOC 2 preparation in Phase 3 |
| Python ecosystem expects native (PyO3) bindings | Medium | Medium | Evaluate PyO3 migration for v2.0 |

### Operational Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Production incidents without runbooks | High | Medium | Prioritize ops documentation in Phase 2 |
| Scaling issues under concurrent load | Medium | High | Load testing in Phase 2; HPA configuration |
| Secret exposure in logs or configs | Low | Critical | Structured logging with PII filtering; secret scanning |

---

## Success Criteria

### Phase 1 Exit Criteria
- [x] Docker image published and scannable (multi-stage distroless build)
- [x] `cargo-audit` and `cargo-deny` passing in CI
- [x] OTEL traces available via feature-gated `otel` flag with OTLP export
- [x] Prometheus metrics scraped and graphed (Docker Compose stack)
- [x] Code coverage measured and reported via cargo-llvm-cov + Codecov
- [x] Cross-platform CI (Linux + macOS + Windows)

### Phase 2 Exit Criteria
- [x] Helm chart deployed to staging K8s cluster
- [x] Generation manifest produced for every run (with per-file checksums, lineage graph, W3C PROV-JSON)
- [x] Load test: k6 scripts for health, bulk generation, WebSocket, job queue, and soak testing
- [x] Zero `unwrap()` calls in library crate non-test code (`#![deny(clippy::unwrap_used)]` enforced)
- [x] Formal DP composition tracking with budget management (RDP, zCDP, privacy budget manager)
- [x] Operations runbook reviewed and validated (deployment guides, runbook, capacity planning, DR, API reference, security hardening)

### Phase 3 Exit Criteria
- [ ] JWT/OAuth2 authentication with RBAC
- [ ] Automated quality gates blocking below-threshold runs
- [ ] Plugin SDK documented with 2+ community plugins
- [ ] Python 1.0.0 on PyPI with async support
- [ ] EU AI Act Article 50 compliance verified
- [ ] SOC 2 Type II readiness assessment completed

### Phase 4 Exit Criteria
- [x] LLM-augmented generation available as opt-in feature
- [x] Diffusion model backend demonstrated on financial dataset
- [x] 3+ ecosystem integrations (Airflow, dbt, MLflow)
- [x] Causal generation prototype validated

---

## Appendix A: OpenTelemetry Integration Architecture

```
┌─────────────────────────────────────────────────────┐
│                   DataSynth Server                  │
│  ┌───────────┐  ┌──────────┐  ┌─────────────────┐  │
│  │  REST API  │  │   gRPC   │  │   WebSocket     │  │
│  └─────┬─────┘  └────┬─────┘  └───────┬─────────┘  │
│        │              │                │             │
│  ┌─────┴──────────────┴────────────────┴──────────┐ │
│  │          Tower Middleware Stack                 │ │
│  │  [Auth] [RateLimit] [Tracing] [Metrics]        │ │
│  └────────────────────┬───────────────────────────┘ │
│                       │                              │
│  ┌────────────────────┴───────────────────────────┐ │
│  │           OpenTelemetry SDK                    │ │
│  │  ┌─────────┐ ┌──────────┐ ┌─────────────────┐ │ │
│  │  │ Traces  │ │ Metrics  │ │     Logs        │ │ │
│  │  └────┬────┘ └────┬─────┘ └───────┬─────────┘ │ │
│  └───────┼───────────┼───────────────┼────────────┘ │
│          │           │               │               │
│  ┌───────┴───────────┴───────────────┴────────────┐ │
│  │           OTLP Exporter (gRPC/HTTP)            │ │
│  └────────────────────┬───────────────────────────┘ │
└───────────────────────┼─────────────────────────────┘
                        │
              ┌─────────┴──────────┐
              │   OTel Collector   │
              │  (Agent sidecar)   │
              └──┬──────┬──────┬───┘
                 │      │      │
           ┌─────┘  ┌───┘  ┌──┘
           ▼        ▼      ▼
       ┌──────┐ ┌──────┐ ┌─────┐
       │Jaeger│ │Prom. │ │Loki │
       │/Tempo│ │      │ │     │
       └──────┘ └──────┘ └─────┘
```

## Appendix B: Recommended Rust Crate Additions

| Category | Crate | Purpose | Phase |
|----------|-------|---------|-------|
| Observability | `opentelemetry` (0.27+) | Unified telemetry API | 1 |
| Observability | `opentelemetry-otlp` | OTLP exporter | 1 |
| Observability | `tracing-opentelemetry` | Bridge tracing → OTEL | 1 |
| Security | `argon2` | Password/key hashing | 1 |
| Security | `subtle` | Constant-time comparison | 1 |
| Security | `rustls` | Native TLS | 1 |
| Scalability | `redis` | Distributed state/rate-limiting | 2 |
| Scalability | `deadpool-redis` | Redis connection pooling | 2 |
| Testing | `cargo-tarpaulin` | Code coverage | 1 |
| Testing | `cargo-fuzz` | Fuzz testing | 2 |
| Auth | `jsonwebtoken` | JWT tokens | 3 |
| Auth | `oauth2` | OAuth2 client | 3 |
| Plugins | `wasmtime` | WASM plugin runtime | 3 |
| Build | `git-cliff` | Changelog generation | 1 |

## Appendix C: Key References

### Standards & Guidelines
- NIST AI RMF 1.0 — AI Risk Management Framework
- NIST SP 800-226 — Guidelines for Evaluating Differential Privacy Guarantees
- EU AI Act (Regulation 2024/1689) — Articles 10, 50
- ISO/IEC 25020:2019 — Systems and software Quality Requirements and Evaluation (SQuaRE)

### Research Papers
- Chen et al. (2025) — "Benchmarking Differentially Private Tabular Data Synthesis Algorithms" (TPDP 2025)
- SynQP (IEEE, 2025) — "A Framework and Metrics for Evaluating the Quality and Privacy Risk of Synthetic Data"
- Xu et al. (2025) — "TabDiff: a Mixed-type Diffusion Model for Tabular Data Generation" (ICLR 2025)
- Sattarov & Schreyer (2023) — "FinDiff: Diffusion Models for Financial Tabular Data Generation" (ICAIF 2023)
- Shi et al. (2025) — "Comprehensive Survey of Synthetic Tabular Data Generation"
- CFA Institute (July 2025) — "Synthetic Data in Investment Management"
- Pilgram et al. (2025) — "A Consensus Privacy Metrics Framework for Synthetic Data"

### Industry Reports
- Gartner (2024) — "By 2026, 75% of businesses will use GenAI for synthetic customer data"
- GlobeNewsWire (January 2026) — AI-Generated Synthetic Tabular Dataset Market: $6.73B by 2029
