# Compliance & Regulations Framework — v1.1.0 Specification

> **Status:** Draft
> **Date:** 2026-03-09
> **Target Release:** v1.1.0
> **Scope:** Integrated strategy for accounting standards, regulatory compliance, audit procedures, and graph-based compliance networks with country-specific profiles and temporal versioning

---

## Document Index

This specification is organized into eight parts for readability and modularity:

| Part | Title | Description |
|------|-------|-------------|
| [01](01-architecture-overview.md) | **Architecture Overview & Design Principles** | High-level architecture, core abstractions, design philosophy, and integration points with existing crates |
| [02](02-regulatory-standards-registry.md) | **Regulatory Standards Registry** | Complete registry of supported standards (SOX, ISA, IFRS, Basel, PCAOB, etc.) with metadata, versioning, and cross-reference maps |
| [03](03-country-compliance-profiles.md) | **Country-Specific Compliance Profiles** | Per-jurisdiction compliance profiles covering 15+ countries, mapping standards to local requirements |
| [04](04-temporal-versioning.md) | **Temporal Versioning & Change Management** | Time-aware standard evolution, effective dates, transition periods, and retroactive amendment handling |
| [05](05-audit-procedure-templates.md) | **Custom Audit Procedure Templates** | YAML/JSON template DSL for defining audit procedures, control tests, and compliance checks |
| [06](06-graph-integration.md) | **Graph Integration & Edge Semantics** | Compliance graph layer: new node/edge types, regulatory relationship modeling, and ML feature engineering |
| [07](07-configuration-reference.md) | **Configuration Reference & Examples** | Complete YAML configuration schema, industry presets, and end-to-end examples |
| [08](08-extension-guide.md) | **Extension Guide & API Surface** | How to add new standards, country packs, audit templates, and graph edge types |

---

## Motivation

DataSynth v1.0 already supports accounting frameworks (US GAAP, IFRS, French GAAP, German GAAP), ISA/PCAOB audit standards, and SOX compliance at a foundational level. However, the current implementation treats these as isolated generation modules — standards produce CSV outputs but lack:

1. **Unified regulatory identity** — No common `Standard` or `Regulation` abstraction that generators can reference
2. **Temporal awareness** — Standards are generated at a point-in-time; no modeling of standard evolution (e.g., IFRS 9 replacing IAS 39)
3. **Country-specific compliance orchestration** — Country packs define locale data (names, holidays) but not which regulations apply, which audit procedures are mandatory, or which reporting formats are required
4. **Graph connectivity** — Compliance relationships (control→standard, audit_procedure→assertion, entity→jurisdiction) are not represented as graph edges, limiting ML applications
5. **Extensible audit templates** — Audit procedures are hardcoded; users cannot define custom procedures, control tests, or compliance checks

This specification addresses all five gaps with a cohesive, extensible framework.

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Registry-based standard identification** | Every standard gets a canonical `StandardId` (e.g., `SOX-404`, `ISA-315`, `IFRS-16`) enabling cross-references without coupling |
| **Temporal versioning via effective-date ranges** | Standards change over time; the framework models `[effective_from, superseded_at)` intervals so generation can target any historical period |
| **Country profiles compose standards** | A country profile is a composition of applicable standards + local overrides, not a separate code path |
| **Graph-first compliance relationships** | Every compliance artifact (control, procedure, assertion, finding) is a graph node with typed edges to related entities |
| **YAML template DSL for audit procedures** | Users define custom procedures declaratively; the runtime compiles templates into executable generation logic |
| **Trait-based extensibility** | New standards, jurisdictions, and edge types implement Rust traits (`ComplianceStandard`, `JurisdictionProfile`, `ComplianceEdge`) |

---

## Integration with Existing Crates

```
┌─────────────────────────────────────────────────────────────────┐
│                        datasynth-config                         │
│  compliance_regulations:                                        │
│    registry, country_profiles, temporal, audit_templates         │
└───────────────┬─────────────────────────────────────────────────┘
                │
    ┌───────────▼───────────┐     ┌─────────────────────┐
    │  datasynth-standards  │◄────│  datasynth-core      │
    │  (extended)           │     │  models/compliance/   │
    │  - regulatory/        │     │  - StandardId         │
    │  - accounting/        │     │  - Regulation         │
    │  - audit/             │     │  - ComplianceEdge     │
    │  - templates/         │     │  - TemporalVersion    │
    └───────────┬───────────┘     └──────────┬──────────┘
                │                             │
    ┌───────────▼───────────┐     ┌──────────▼──────────┐
    │ datasynth-generators  │     │  datasynth-graph     │
    │  compliance/          │     │  builders/           │
    │  - regulation_gen     │     │    compliance_graph  │
    │  - procedure_gen      │     │  models/             │
    │  - finding_gen        │     │    compliance_edges  │
    │  - country_gen        │     │    compliance_nodes  │
    └───────────┬───────────┘     └──────────┬──────────┘
                │                             │
    ┌───────────▼─────────────────────────────▼──────────┐
    │              datasynth-runtime                       │
    │  ComplianceOrchestrator                              │
    │  - resolves country profile → applicable standards   │
    │  - generates compliance artifacts in dependency order │
    │  - wires artifacts into graph builder                 │
    └─────────────────────────────────────────────────────┘
```

---

## Glossary

| Term | Definition |
|------|-----------|
| **Standard** | A formal specification issued by a standards body (e.g., IFRS 16, ISA 315, SOX Section 404) |
| **Regulation** | A legally binding requirement imposed by a jurisdiction (e.g., EU Audit Regulation, UK Companies Act) |
| **Compliance Profile** | A per-country composition of applicable standards, regulations, and local overrides |
| **Audit Procedure Template** | A declarative YAML definition of an audit procedure including assertions, sampling, and expected findings |
| **Temporal Version** | A specific version of a standard with effective and superseded dates |
| **Compliance Edge** | A typed graph edge connecting compliance artifacts (control→standard, procedure→assertion, finding→control) |
| **Standard Registry** | The central catalog of all supported standards with metadata, versions, and cross-references |
| **Jurisdiction** | A legal territory where specific regulations apply (country, state, or supranational like EU) |
