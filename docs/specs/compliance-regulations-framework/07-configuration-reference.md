# Part 7: Configuration Reference & Examples

> **Parent:** [Compliance & Regulations Framework](00-index.md)
> **Status:** Implemented | **Date:** 2026-03-09

---

## 7.1 Complete Configuration Schema

```yaml
# =============================================================================
# COMPLIANCE REGULATIONS FRAMEWORK - Full Configuration Reference
# =============================================================================
compliance_regulations:
  # Master switch
  enabled: true

  # ─── PRESET MODE ───────────────────────────────────────────────────────────
  # Use a preset for quick setup. Overrides individual settings.
  # Options: sox_us, isa_eu, isa_uk, multi_jurisdiction, banking_basel,
  #          comprehensive, minimal
  preset: null  # Set to use a preset; null = manual configuration

  # ─── STANDARD REGISTRY ─────────────────────────────────────────────────────
  registry:
    # Include built-in standards (IFRS, ISA, SOX, etc.)
    built_in: true
    # Directory for user-defined standards (YAML files)
    custom_standards_dir: null  # e.g., "./custom-standards"
    # Filter: only load standards from these bodies
    include_bodies: []  # Empty = all. e.g., ["IFRS", "ISA", "SOX"]
    # Filter: exclude standards from these bodies
    exclude_bodies: []  # e.g., ["BASEL"] to skip prudential regulations

  # ─── JURISDICTION SETTINGS ─────────────────────────────────────────────────
  jurisdictions:
    # How to determine applicable jurisdictions
    mode: auto  # auto (from company.country) | explicit
    # Explicit jurisdiction list (used when mode: explicit)
    countries: []  # e.g., ["US", "DE", "GB", "JP"]
    # Custom jurisdiction profiles directory
    custom_profiles_dir: null  # e.g., "./jurisdiction-profiles"

  # ─── TEMPORAL SETTINGS ─────────────────────────────────────────────────────
  temporal:
    # Target date for regulatory state resolution
    # Defaults to global.start_date + global.period_months
    target_date: null  # e.g., "2025-06-30"

    # Transition period modeling
    transition:
      enabled: false
      # Probability that entities have early-adopted when in transition window
      early_adoption_probability: 0.10
      # Generate parallel reporting data during transitions
      parallel_reporting: false
      # Specific transition overrides
      overrides: []
      # Example:
      # overrides:
      #   - old_standard: "IAS-39"
      #     new_standard: "IFRS-9"
      #     entities_transitioned_percent: 0.80

    # Regulatory regime changes affecting distributions
    regime_changes:
      enabled: false
      # See Part 4 for regime_changes configuration

  # ─── STANDARDS SELECTION ───────────────────────────────────────────────────
  standards:
    # Accounting standards
    accounting:
      enabled: true
      # Framework selection (inherits from accounting_standards.framework)
      # framework: us_gaap  # Already set in accounting_standards section

      # Per-standard toggles
      revenue_recognition: true   # IFRS 15 / ASC 606
      leases: true                # IFRS 16 / ASC 842
      fair_value: true            # IFRS 13 / ASC 820
      impairment: true            # IAS 36 / ASC 360
      financial_instruments: true # IFRS 9 / ASC 326
      consolidation: false        # IFRS 10 / ASC 810
      insurance: false            # IFRS 17 (if applicable)

    # Auditing standards
    auditing:
      enabled: true
      framework: dual  # isa | pcaob | dual | local
      # ISA-specific settings
      isa:
        compliance_level: comprehensive  # basic | standard | comprehensive
        include_revised_2019: true       # ISA 315 (Revised 2019)
        include_quality_mgmt: true       # ISQM 1/2
      # PCAOB-specific settings
      pcaob:
        enabled: true  # Only for US SEC registrants
        integrated_audit: true
        cam_reporting: true

    # Regulatory requirements
    regulatory:
      enabled: true
      sox:
        enabled: true
        sections: [302, 404]          # SOX sections to generate
        deficiency_generation: true   # Generate deficiency matrix
        materiality_threshold: 10000.0
      eu_audit_regulation:
        enabled: true                 # Auto-enabled for EU jurisdictions
        pie_requirements: true
      basel:
        enabled: false
        standards: [BASEL-III-CAP, BASEL-III-LCR]
      aml:
        enabled: false
        directive: EU-AMLD-6

    # Sustainability/ESG standards
    sustainability:
      enabled: false
      frameworks: []  # EU-CSRD, ISSB-S1, ISSB-S2, GRI-2021

  # ─── AUDIT PROCEDURE TEMPLATES ─────────────────────────────────────────────
  audit_templates:
    enabled: true
    # Include built-in template library
    built_in: true
    # Custom template directory
    custom_dir: null  # e.g., "./audit-procedures/custom"
    # Template categories to enable
    categories:
      substantive_test: true
      test_of_controls: true
      analytical: true
      confirmation: true
      compliance: true
      journal_entry_test: true
    # Procedure count per engagement
    procedures_per_engagement: 25
    # Average steps per procedure
    avg_steps_per_procedure: 4
    # Customizations to built-in templates
    customizations: []
    # Example:
    # customizations:
    #   - template_id: "AP-REV-SUBST-001"
    #     overrides:
    #       steps.S1.params.confidence_level: 0.99

  # ─── COMPLIANCE GRAPH ──────────────────────────────────────────────────────
  graph:
    enabled: true
    # Node types to include
    node_types:
      standard: true
      regulation: true
      requirement: true
      assertion: true
      control: true
      procedure: true
      finding: true
      jurisdiction: true
      coso_component: true
      engagement: true
    # Edge types to include
    edge_types:
      maps_to_standard: true
      covers_assertion: true
      implements_coso: true
      tests_control: true
      addresses_assertion: true
      finding_on_control: true
      identified_by_procedure: true
      subject_to_jurisdiction: true
      requires_standard: true
      supersedes: true
      cross_references: true
      contains_requirement: true
    # Cross-graph edges (linking to transaction/approval graphs)
    cross_graph_edges: true
    # Export formats
    export_formats: [pytorch_geometric, neo4j]  # pytorch_geometric, neo4j, dgl

  # ─── FINDINGS & DEFICIENCIES ───────────────────────────────────────────────
  findings:
    enabled: true
    # Overall finding rate
    finding_rate: 0.08  # 8% of procedures identify findings
    # Severity distribution
    severity_distribution:
      high: 0.05
      moderate: 0.25
      low: 0.70
    # Deficiency classification (SOX)
    deficiency_classification:
      material_weakness_rate: 0.02
      significant_deficiency_rate: 0.08
      control_deficiency_rate: 0.15
    # Repeat findings from prior periods
    repeat_finding_rate: 0.20
    # Remediation rates
    remediation:
      remediated_rate: 0.70
      in_progress_rate: 0.20
      open_rate: 0.10

  # ─── FILING GENERATION ─────────────────────────────────────────────────────
  filings:
    enabled: false
    # Generate filing metadata records
    types: []  # Auto-determined from jurisdiction
    # Example:
    # types: ["10-K", "10-Q", "8-K"]  # US-specific
    # types: ["Jahresabschluss", "E-Bilanz"]  # DE-specific

  # ─── OUTPUT ─────────────────────────────────────────────────────────────────
  output:
    # Output directory for compliance-specific files
    subdirectory: "compliance"
    # Standard registry metadata
    export_registry: true
    # Cross-reference map
    export_cross_references: true
    # Jurisdiction resolution details
    export_jurisdiction_details: true
    # Temporal context
    export_temporal_context: true
    # Coverage matrix (standards × controls)
    export_coverage_matrix: true
```

---

## 7.2 Presets

### 7.2.1 SOX US Preset

For US public companies requiring SOX compliance:

```yaml
compliance_regulations:
  preset: sox_us
  # Equivalent to:
  # jurisdictions: { mode: explicit, countries: [US] }
  # standards:
  #   accounting: { framework: us_gaap }
  #   auditing: { framework: pcaob, pcaob: { integrated_audit: true, cam_reporting: true } }
  #   regulatory: { sox: { enabled: true, sections: [302, 404] } }
  # audit_templates: { categories: { test_of_controls: true, journal_entry_test: true } }
```

### 7.2.2 ISA EU Preset

For EU entities with ISA-based audit:

```yaml
compliance_regulations:
  preset: isa_eu
  # Equivalent to:
  # jurisdictions: { mode: auto }  # Derives from company countries
  # standards:
  #   accounting: { framework: ifrs }
  #   auditing: { framework: isa, isa: { compliance_level: comprehensive } }
  #   regulatory: { eu_audit_regulation: { enabled: true } }
  #   sustainability: { enabled: true, frameworks: [EU-CSRD] }
```

### 7.2.3 Multi-Jurisdiction Preset

For multinational groups:

```yaml
compliance_regulations:
  preset: multi_jurisdiction
  # Equivalent to:
  # jurisdictions: { mode: auto }
  # standards:
  #   accounting: { enabled: true }  # Framework per company
  #   auditing: { framework: dual }  # ISA + PCAOB where applicable
  #   regulatory: { sox: { enabled: true }, eu_audit_regulation: { enabled: true } }
  # graph: { enabled: true, cross_graph_edges: true }
```

### 7.2.4 Banking Basel Preset

For banking/financial institutions:

```yaml
compliance_regulations:
  preset: banking_basel
  # Equivalent to:
  # standards:
  #   regulatory: { basel: { enabled: true, standards: [BASEL-III-CAP, BASEL-III-LCR, BASEL-III-NSFR] } }
  #   regulatory: { aml: { enabled: true } }
  # audit_templates: { categories: { compliance: true } }
```

### 7.2.5 Comprehensive Preset

Everything enabled:

```yaml
compliance_regulations:
  preset: comprehensive
  # Enables all standards, all templates, all graph edges, all output
```

### 7.2.6 Minimal Preset

Lightest touch — just standard references:

```yaml
compliance_regulations:
  preset: minimal
  # Registry only, no generation — just attaches standard IDs to existing data
```

---

## 7.3 End-to-End Configuration Examples

### 7.3.1 Example: US Public Company — SOX Compliance

```yaml
global:
  industry: financial_services
  start_date: "2024-01-01"
  period_months: 12
  seed: 42

companies:
  - code: "ACME"
    name: "Acme Financial Corp"
    currency: USD
    country: US
    compliance_overrides:
      entity_type: large_accelerated_filer

chart_of_accounts:
  complexity: medium

accounting_standards:
  enabled: true
  framework: us_gaap
  revenue_recognition:
    enabled: true
    generate_contracts: true
  leases:
    enabled: true
    lease_count: 100
  fair_value:
    enabled: true

audit_standards:
  enabled: true
  sox:
    enabled: true
    generate_302_certifications: true
    generate_404_assessments: true
    materiality_threshold: 50000.0

internal_controls:
  enabled: true
  coso_enabled: true
  include_entity_level_controls: true
  target_maturity_level: managed
  exception_rate: 0.03
  sod_violation_rate: 0.01

compliance_regulations:
  enabled: true
  jurisdictions:
    mode: auto
  temporal:
    target_date: "2024-12-31"
  standards:
    accounting:
      enabled: true
      revenue_recognition: true
      leases: true
      fair_value: true
      financial_instruments: true
    auditing:
      enabled: true
      framework: pcaob
      pcaob:
        integrated_audit: true
        cam_reporting: true
    regulatory:
      sox:
        enabled: true
        sections: [302, 404]
        deficiency_generation: true
  audit_templates:
    enabled: true
    categories:
      substantive_test: true
      test_of_controls: true
      analytical: true
      journal_entry_test: true
    procedures_per_engagement: 30
  graph:
    enabled: true
    export_formats: [pytorch_geometric, neo4j]
  findings:
    finding_rate: 0.06
    severity_distribution:
      high: 0.03
      moderate: 0.22
      low: 0.75

graph_export:
  enabled: true
  formats: [pytorch_geometric, neo4j]
```

### 7.3.2 Example: Multinational Group — Multi-Jurisdiction

```yaml
global:
  industry: manufacturing
  start_date: "2025-01-01"
  period_months: 12
  seed: 12345

companies:
  - code: "PARENT"
    name: "Global Manufacturing Inc"
    currency: USD
    country: US
    compliance_overrides:
      entity_type: accelerated_filer

  - code: "DE-SUB"
    name: "Deutsche Fertigung GmbH"
    currency: EUR
    country: DE
    compliance_overrides:
      entity_type: large_entity
      ifrs_reporting: true

  - code: "JP-SUB"
    name: "日本製造株式会社"
    currency: JPY
    country: JP
    compliance_overrides:
      entity_type: listed_entity

  - code: "GB-SUB"
    name: "British Manufacturing Ltd"
    currency: GBP
    country: GB
    compliance_overrides:
      entity_type: large_entity

accounting_standards:
  enabled: true
  framework: dual_reporting  # US GAAP + IFRS for consolidation

compliance_regulations:
  enabled: true
  jurisdictions:
    mode: auto  # US, DE, JP, GB derived from companies
  temporal:
    target_date: "2025-12-31"
    transition:
      enabled: true
  standards:
    accounting:
      enabled: true
      revenue_recognition: true
      leases: true
      fair_value: true
    auditing:
      enabled: true
      framework: dual  # ISA for non-US, PCAOB for US
    regulatory:
      sox:
        enabled: true
        sections: [302, 404]
      eu_audit_regulation:
        enabled: true
    sustainability:
      enabled: true
      frameworks: [EU-CSRD]
  audit_templates:
    enabled: true
    procedures_per_engagement: 40
  graph:
    enabled: true
    cross_graph_edges: true
    export_formats: [pytorch_geometric, neo4j]
  output:
    export_registry: true
    export_cross_references: true
    export_jurisdiction_details: true
```

### 7.3.3 Example: Banking — Basel + AML

```yaml
global:
  industry: financial_services
  start_date: "2025-01-01"
  period_months: 12

companies:
  - code: "BANK"
    name: "Global Bank AG"
    currency: EUR
    country: DE
    compliance_overrides:
      entity_type: public_interest_entity
      sector: banking

compliance_regulations:
  enabled: true
  standards:
    accounting:
      enabled: true
      financial_instruments: true  # IFRS 9 critical for banks
    regulatory:
      sox:
        enabled: false
      eu_audit_regulation:
        enabled: true
        pie_requirements: true
      basel:
        enabled: true
        standards:
          - BASEL-III-CAP
          - BASEL-III-LCR
          - BASEL-III-NSFR
          - BASEL-IV-SA
      aml:
        enabled: true
        directive: EU-AMLD-6
    sustainability:
      enabled: true
      frameworks: [EU-CSRD, EU-TAX]
  audit_templates:
    enabled: true
    categories:
      compliance: true
      substantive_test: true
  graph:
    enabled: true
```

---

## 7.4 Output File Reference

When compliance generation is enabled, the following files are produced:

```
output/
├── compliance/
│   ├── registry/
│   │   ├── standards_registry.json
│   │   ├── cross_reference_map.json
│   │   ├── supersession_chains.json
│   │   └── standard_coverage_matrix.csv
│   ├── jurisdictions/
│   │   ├── jurisdiction_profiles.json
│   │   ├── resolved_standards_by_entity.json
│   │   └── temporal_context.json
│   ├── regulations/
│   │   ├── applicable_regulations.csv
│   │   ├── regulatory_requirements.csv
│   │   └── filing_requirements.csv
│   ├── audit_procedures/
│   │   ├── procedure_catalog.csv
│   │   ├── procedure_step_results.csv
│   │   ├── audit_samples.csv
│   │   ├── audit_exceptions.csv
│   │   └── procedure_conclusions.csv
│   ├── findings/
│   │   ├── compliance_findings.csv
│   │   ├── deficiency_classifications.csv
│   │   └── remediation_tracker.csv
│   └── assertions/
│       ├── assertion_mapping.csv
│       └── assertion_coverage.csv
├── graphs/
│   └── compliance_network/
│       ├── pytorch_geometric/
│       │   └── ... (see Part 6)
│       └── neo4j/
│           └── ... (see Part 6)
└── standards/                  # Existing output, enhanced
    ├── accounting/
    │   └── ... (existing files + standard_version metadata)
    ├── audit/
    │   └── ... (existing files + template references)
    └── regulatory/
        └── ... (existing files + jurisdiction context)
```

---

## 7.5 Validation Rules

| Parameter | Validation | Default |
|-----------|-----------|---------|
| `finding_rate` | 0.0 - 1.0 | 0.08 |
| `severity_distribution` | Sum = 1.0 (±0.01) | {high: 0.05, moderate: 0.25, low: 0.70} |
| `deficiency_classification.*_rate` | 0.0 - 1.0 | See schema |
| `repeat_finding_rate` | 0.0 - 1.0 | 0.20 |
| `remediation.*_rate` | Sum = 1.0 (±0.01) | {remediated: 0.70, in_progress: 0.20, open: 0.10} |
| `procedures_per_engagement` | 1 - 500 | 25 |
| `avg_steps_per_procedure` | 1 - 20 | 4 |
| `target_date` | Valid ISO date | Derived from global settings |
| `preset` | One of named presets or null | null |
| `countries[]` | Valid ISO 3166-1 alpha-2 codes | Derived from companies |
