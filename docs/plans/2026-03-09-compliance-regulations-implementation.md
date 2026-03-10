# Compliance & Regulations Framework — Implementation Plan

> **Date:** 2026-03-09
> **Branch:** `claude/compliance-regulations-framework-V6sQ0`
> **Spec:** `docs/specs/compliance-regulations-framework/`

## Implementation Phases

### Phase 1: Core Models (datasynth-core)
New module: `models/compliance/`
- `mod.rs` — module exports
- `standard_id.rs` — `StandardId` canonical identifier
- `standard.rs` — `ComplianceStandard`, `StandardCategory`, `ComplianceDomain`, `IssuingBody`
- `temporal.rs` — `TemporalVersion`, `ChangeImpact`, `TransitionalProvision`
- `jurisdiction.rs` — `JurisdictionProfile`, `JurisdictionStandard`, `ApplicabilityCriteria`
- `assertion.rs` — `ComplianceAssertion` enum (extends SoxAssertion)
- `finding.rs` — `ComplianceFinding`, `DeficiencyLevel`
- `filing.rs` — `RegulatoryFiling`, `FilingRequirement`
- `cross_reference.rs` — `CrossReference`, `CrossReferenceType`

### Phase 2: Standard Registry (datasynth-standards)
New module: `registry/`
- `mod.rs` — `StandardRegistry` struct with temporal resolution
- `built_in.rs` — Built-in standards catalog (IFRS, ISA, SOX, ASC, Basel, EU)
- `loader.rs` — YAML custom standard loader

Extend: `regulatory/`
- Add `eu_regulations.rs`, `basel.rs`

### Phase 3: Configuration (datasynth-config)
Extend `schema.rs`:
- Add `ComplianceRegulationsConfig` struct
- Sub-configs: registry, jurisdictions, temporal, standards selection, audit_templates, graph, findings, filings, output
- Preset resolution logic
- Validation rules

### Phase 4: Compliance Generators (datasynth-generators)
New module: `compliance/`
- `mod.rs`
- `regulation_generator.rs` — Generate regulation records
- `procedure_generator.rs` — Template-driven audit procedures
- `finding_generator.rs` — Compliance findings with deficiency classification
- `assertion_generator.rs` — Assertion-to-account mapping
- `filing_generator.rs` — Regulatory filing metadata

### Phase 5: Graph Integration (datasynth-graph)
Extend `models/nodes.rs` — Add Standard, Regulation, AuditProcedure, Finding, Jurisdiction, Assertion node types
Extend `models/edges.rs` — Add MapsToStandard, TestsControl, FindingOnControl, SubjectToJurisdiction, etc.
New builder: `builders/compliance_graph.rs`

### Phase 6: Runtime Wiring (datasynth-runtime)
Extend `enhanced_orchestrator.rs`:
- Add `phase_compliance_regulations()` method
- Wire into generation pipeline after phase_accounting_standards
- Add `ComplianceSnapshot` to `EnhancedGenerationResult`

Extend output_writer:
- Export compliance files (registry, findings, procedures, etc.)

### Phase 7: Tests & Validation
- Unit tests per module
- Integration test: full generation with compliance enabled
- cargo check, fmt, clippy

## Execution Order
Phases 1-3 first (foundation), then 4-5 (generation + graph), then 6-7 (wiring + tests).
