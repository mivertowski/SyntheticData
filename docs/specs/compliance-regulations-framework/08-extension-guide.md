# Part 8: Extension Guide & API Surface

> **Parent:** [Compliance & Regulations Framework](00-index.md)
> **Status:** Implemented | **Date:** 2026-03-09

---

## 8.1 Overview

The Compliance & Regulations Framework is designed for extensibility at every layer. This guide covers how to:

1. **Add a new compliance standard** to the registry
2. **Add a new country/jurisdiction profile**
3. **Create custom audit procedure templates**
4. **Add new graph node and edge types**
5. **Extend the temporal versioning system**
6. **Build custom compliance generators**

Each extension point has a **no-code path** (YAML/JSON) and a **code path** (Rust traits).

---

## 8.2 Adding a New Standard

### 8.2.1 No-Code: YAML Registration

Create a YAML file in the custom standards directory:

```yaml
# custom-standards/esg-tcfd.yaml
standard:
  id: "TCFD-2017"
  title: "Task Force on Climate-related Financial Disclosures"
  issuing_body: custom  # Or a built-in: iasb, iaasb, fasb, pcaob, sec, etc.
  category: sustainability_standard
  domain: sustainability

  versions:
    - version_id: "2017"
      issued_date: "2017-06-29"
      effective_from: "2017-06-29"
      change_summary:
        - "Initial TCFD recommendations published"
      impact: high

    - version_id: "2021-guidance"
      issued_date: "2021-10-14"
      effective_from: "2021-10-14"
      change_summary:
        - "Updated implementing guidance"
        - "Cross-industry metrics refined"
      impact: medium

  supersedes: []
  superseded_by: "ISSB-S2"  # ISSB subsumes TCFD

  cross_references:
    - standard_id: "EU-CSRD"
      relationship: complementary
    - standard_id: "ISSB-S1"
      relationship: incorporated_into
    - standard_id: "ISSB-S2"
      relationship: incorporated_into

  requirements:
    - id: "TCFD-GOV"
      title: "Governance"
      description: "Board oversight of climate-related risks and opportunities"
      assertions: [existence, completeness]
    - id: "TCFD-STRAT"
      title: "Strategy"
      description: "Impact of climate risks on organization's strategy"
      assertions: [completeness, accuracy]
    - id: "TCFD-RISK"
      title: "Risk Management"
      description: "Processes for identifying, assessing, and managing climate risks"
      assertions: [existence, completeness]
    - id: "TCFD-MET"
      title: "Metrics and Targets"
      description: "Metrics and targets used to assess climate risks"
      assertions: [accuracy, completeness]

  mandatory_jurisdictions: []  # Not legally mandatory anywhere as TCFD
  permitted_jurisdictions: ["*"]  # Available everywhere
```

Then reference it in configuration:

```yaml
compliance_regulations:
  registry:
    custom_standards_dir: "./custom-standards"
  standards:
    sustainability:
      enabled: true
      frameworks: ["TCFD-2017"]
```

### 8.2.2 Code Path: Rust Trait

For standards that require custom generation logic:

```rust
use datasynth_core::compliance::{
    ComplianceStandard, StandardId, StandardCategory, ComplianceDomain,
    TemporalVersion, StandardRequirement,
};

/// Trait for standards with custom generation behavior.
pub trait ComplianceStandardGenerator: Send + Sync {
    /// Returns the standard metadata.
    fn standard(&self) -> &ComplianceStandard;

    /// Generates standard-specific records.
    fn generate(
        &self,
        rng: &mut ChaCha8Rng,
        context: &GenerationContext,
    ) -> Vec<ComplianceRecord>;

    /// Returns any additional graph nodes this standard contributes.
    fn graph_nodes(&self, context: &GenerationContext) -> Vec<GraphNode>;

    /// Returns any additional graph edges this standard contributes.
    fn graph_edges(&self, context: &GenerationContext) -> Vec<GraphEdge>;
}

// Registration in the generator factory:
impl ComplianceGeneratorFactory {
    pub fn register(
        &mut self,
        id: StandardId,
        generator: Box<dyn ComplianceStandardGenerator>,
    ) {
        self.generators.insert(id, generator);
    }
}
```

---

## 8.3 Adding a New Country Profile

### 8.3.1 No-Code: YAML Profile

```yaml
# jurisdiction-profiles/SA.yaml
country_code: SA
country_name: "Kingdom of Saudi Arabia"
memberships: [GCC]

accounting:
  primary_framework: ifrs  # IFRS mandatory since 2017
  standards_body: SOCPA
  key_standards:
    - { id: "IFRS-15", local_name: "IFRS 15 (SOCPA endorsed)", effective: "2018-01-01" }
    - { id: "IFRS-16", local_name: "IFRS 16 (SOCPA endorsed)", effective: "2019-01-01" }
    - { id: "IFRS-9", local_name: "IFRS 9 (SOCPA endorsed)", effective: "2018-01-01" }
  local_rules:
    zakat_accounting: true  # Zakat instead of/alongside income tax

auditing:
  framework: isa
  standards_body: SOCPA
  key_standards:
    - { id: "ISA-315", local_name: "ISA (SA) 315" }
    - { id: "ISA-700", local_name: "ISA (SA) 700" }
  local_additions:
    - "SOCPA specific reporting requirements"
    - "Sharia compliance review (for Islamic finance)"

regulatory:
  securities_regulator: CMA
  stock_exchange: [TADAWUL]
  key_regulations:
    - { id: "SA-CL", scope: "all_entities", description: "Companies Law" }
    - { id: "SA-CMA", scope: "listed_entities", description: "CMA requirements" }

tax:
  tax_authority: ZATCA
  corporate_tax_rate: 0.20  # For non-Saudi/non-GCC shareholders
  zakat_rate: 0.025  # 2.5% on adjusted net worth for Saudi shareholders
  key_regulations:
    - { id: "ZATCA-VAT", scope: "above_threshold", description: "15% VAT" }
    - { id: "ZATCA-EINV", scope: "all_entities", description: "E-invoicing (Fatoora)" }
  e_invoicing: mandatory

reporting:
  formats:
    - { type: "Annual Financial Statements", frequency: annual, regulator: CMA }
    - { type: "Interim Financial Statements", frequency: quarterly, regulator: CMA }
  electronic_filing: TADAWUL_platform
  language: [arabic, english]

sustainability:
  frameworks:
    - { id: "SA-ESG", effective: "2024-01-01", scope: "listed_entities" }
```

Configuration:

```yaml
compliance_regulations:
  jurisdictions:
    custom_profiles_dir: "./jurisdiction-profiles"
```

### 8.3.2 Code Path: JurisdictionPlugin Trait

```rust
/// Trait for jurisdictions with custom compliance logic.
pub trait JurisdictionPlugin: Send + Sync {
    /// Returns the country code.
    fn country_code(&self) -> &str;

    /// Returns the full jurisdiction profile.
    fn profile(&self) -> &CountryComplianceProfile;

    /// Applies jurisdiction-specific transformations to generated data.
    fn apply_local_rules(
        &self,
        data: &mut GeneratedData,
        rng: &mut ChaCha8Rng,
    );

    /// Returns jurisdiction-specific output files.
    fn export_files(&self, data: &GeneratedData) -> Vec<ExportFile>;

    /// Returns additional graph nodes for this jurisdiction.
    fn graph_contributions(&self, data: &GeneratedData) -> JurisdictionGraphContribution;
}

// Example: Saudi Arabia plugin with Zakat-specific logic
pub struct SaudiArabiaPlugin;

impl JurisdictionPlugin for SaudiArabiaPlugin {
    fn country_code(&self) -> &str { "SA" }

    fn apply_local_rules(&self, data: &mut GeneratedData, rng: &mut ChaCha8Rng) {
        // Apply Zakat calculation rules
        // Generate Zakat-specific journal entries
        // Apply Islamic finance compliance rules
    }

    fn export_files(&self, data: &GeneratedData) -> Vec<ExportFile> {
        vec![
            ExportFile::new("zakat_calculation.csv", self.generate_zakat_report(data)),
            ExportFile::new("e_invoice_fatoora.xml", self.generate_fatoora(data)),
        ]
    }
    // ...
}
```

---

## 8.4 Creating Custom Audit Procedure Templates

### 8.4.1 Template Authoring

Custom templates follow the schema defined in Part 5. Key steps:

1. **Create the YAML file** in your custom directory
2. **Define metadata** (id, name, standards, assertions)
3. **Define steps** using built-in step types
4. **Define findings** profile
5. **Optionally inherit** from a built-in template

```yaml
# audit-procedures/custom/insurance-contract-testing.yaml
template:
  id: "CUSTOM-INS-001"
  name: "Insurance Contract Liability Testing (IFRS 17)"
  version: "1.0"
  category: substantive_test

  standards:
    - id: "IFRS-17"
      requirements: ["IFRS-17.R40", "IFRS-17.R44", "IFRS-17.R80"]

  assertions:
    - valuation_allocation
    - completeness
    - accuracy

  risk_levels: [significant]

  applicable_accounts:
    - insurance_contract_liabilities
    - reinsurance_assets

  steps:
    - step_id: "S1"
      name: "Contract Grouping Verification"
      type: recalculation
      params:
        calculation_type: contract_grouping
        precision: exact
        population: insurance_contracts
        sample_size: 25

    - step_id: "S2"
      name: "Fulfilment Cash Flow Recalculation"
      type: recalculation
      params:
        calculation_type: present_value
        discount_rates: yield_curve
        tolerance: 0.005  # 0.5% tolerance

    - step_id: "S3"
      name: "Risk Adjustment Reasonableness"
      type: analytical_procedure
      params:
        procedure_type: reasonableness_test
        comparisons:
          - { metric: "risk_adjustment_percent", expectation: "confidence_level_range", min: 0.55, max: 0.90 }
        investigation_threshold: 0.05

    - step_id: "S4"
      name: "CSM Amortization Testing"
      type: recalculation
      params:
        calculation_type: amortization_schedule
        method: coverage_units
        tolerance: 0.01

  findings:
    overall_exception_rate: 0.10
    severity_distribution:
      - { severity: high, weight: 0.10 }
      - { severity: moderate, weight: 0.40 }
      - { severity: low, weight: 0.50 }
    finding_types:
      - { type: "grouping_error", probability: 0.03, severity: high }
      - { type: "discount_rate_deviation", probability: 0.04, severity: moderate }
      - { type: "csm_calculation_error", probability: 0.02, severity: moderate }
      - { type: "rounding_difference", probability: 0.05, severity: low }
```

### 8.4.2 Template Validation

Templates are validated against the JSON schema on load:

```bash
# Validate a custom template
datasynth-data validate --template ./audit-procedures/custom/insurance-contract-testing.yaml
```

Validation checks:
- Required fields present
- Step types are recognized
- Standard IDs exist in registry
- Assertion types are valid
- Probabilities sum correctly
- Severity distribution sums to 1.0

---

## 8.5 Adding Graph Node and Edge Types

### 8.5.1 New Node Type

```rust
// In datasynth-graph/src/models/nodes.rs
pub enum NodeType {
    // ... existing ...

    // Add your custom node type to the enum
    // OR use NodeType::Custom("my_type".to_string())
}

// Create a specialized node struct (optional, for feature computation)
pub struct RegulatoryFilingNode {
    pub node: GraphNode,
    pub filing_type: String,
    pub jurisdiction: String,
    pub filing_date: NaiveDate,
    pub deadline: NaiveDate,
    pub status: FilingStatus,
}

impl RegulatoryFilingNode {
    pub fn compute_features(&mut self) {
        // Days until/past deadline
        let days_to_deadline = (self.deadline - self.filing_date).num_days() as f64;
        self.node.features.push(days_to_deadline / 365.0);

        // Filing status
        let status_code = match self.status {
            FilingStatus::Filed => 1.0,
            FilingStatus::Pending => 0.5,
            FilingStatus::Overdue => 0.0,
        };
        self.node.features.push(status_code);

        // Categorical
        self.node.categorical_features.insert(
            "filing_type".to_string(), self.filing_type.clone()
        );
    }
}
```

### 8.5.2 New Edge Type

```rust
// In datasynth-graph/src/models/edges.rs
pub enum EdgeType {
    // ... existing ...

    // Add your custom edge type
    // OR use EdgeType::Custom("my_edge".to_string())
}

// Create a specialized edge struct (optional)
pub struct RegulatoryRequiresEdge {
    pub edge: GraphEdge,
    pub regulation_id: String,
    pub standard_id: String,
    pub requirement_type: RequirementType,
    pub criticality: f64,
}

impl RegulatoryRequiresEdge {
    pub fn compute_features(&mut self) {
        self.edge.features.push(self.criticality);
        let req_type_code = match self.requirement_type {
            RequirementType::Mandatory => 1.0,
            RequirementType::ConditionallyMandatory => 0.75,
            RequirementType::Recommended => 0.5,
            RequirementType::Optional => 0.25,
        };
        self.edge.features.push(req_type_code);
    }
}
```

### 8.5.3 Registering with the Graph Builder

```rust
// In your custom builder
impl ComplianceGraphBuilder {
    fn build_filing_nodes(&self, graph: &mut ComplianceGraph, filings: &[RegulatoryFiling]) {
        for filing in filings {
            let node_id = self.next_node_id();
            let mut filing_node = RegulatoryFilingNode {
                node: GraphNode::new(
                    node_id,
                    NodeType::Custom("RegulatoryFiling".to_string()),
                    filing.id.clone(),
                    format!("{} - {}", filing.filing_type, filing.jurisdiction),
                ),
                filing_type: filing.filing_type.clone(),
                jurisdiction: filing.jurisdiction.clone(),
                filing_date: filing.filing_date,
                deadline: filing.deadline,
                status: filing.status.clone(),
            };
            filing_node.compute_features();
            graph.add_node(filing_node.node);

            // Connect to jurisdiction
            if let Some(jurisd_id) = self.node_index.get(
                &ComplianceNodeKey::Jurisdiction(filing.jurisdiction.clone())
            ) {
                let edge = GraphEdge::new(
                    self.next_edge_id(),
                    node_id,
                    *jurisd_id,
                    EdgeType::Custom("FilingRequiredBy".to_string()),
                );
                graph.add_edge(edge);
            }
        }
    }
}
```

---

## 8.6 Extending Temporal Versioning

### 8.6.1 Adding Amendment Events

```yaml
# custom-standards/ifrs16-amendments.yaml
amendments:
  - id: "IFRS-16-COVID"
    title: "COVID-19-Related Rent Concessions beyond 30 June 2022"
    amends_standard: "IFRS-16"
    amends_version: "2019"
    effective_from: "2022-04-01"
    sunset_date: "2023-06-30"
    changes:
      - id: "COVID-RENT-EXT"
        description: "Extended practical expedient for rent concessions"
        data_impact:
          type: parameter_change
          parameter: "lease_modification_simplified"
          old_value: "false"
          new_value: "true"
          affected_entities: "lessees_with_concessions"
```

### 8.6.2 Custom Regime Changes

```yaml
compliance_regulations:
  temporal:
    regime_changes:
      enabled: true
      custom:
        - id: "CUSTOM-IFRS18-TRANSITION"
          standard: "IFRS-18"
          date: "2027-01-01"
          description: "IFRS 18 replaces IAS 1 — P&L restructured"
          effects:
            - field: "income_statement_categories"
              change: "restructured"
              description: "Operating, Investing, Financing categories replace functional"
            - field: "management_defined_measures"
              change: "new_disclosure"
              description: "Required disclosure of non-GAAP measures"
```

---

## 8.7 Building Custom Compliance Generators

### 8.7.1 Generator Trait

```rust
/// Trait for compliance-specific data generators.
pub trait ComplianceGenerator: Send + Sync {
    /// Generator name for logging and diagnostics.
    fn name(&self) -> &str;

    /// What standards does this generator address?
    fn applicable_standards(&self) -> Vec<StandardId>;

    /// Generate compliance records.
    fn generate(
        &self,
        rng: &mut ChaCha8Rng,
        config: &ComplianceRegulationsConfig,
        context: &ComplianceContext,
    ) -> Result<ComplianceOutput, GenerationError>;
}

/// Context provided to compliance generators.
pub struct ComplianceContext {
    /// Resolved regulatory state for each entity
    pub regulatory_states: HashMap<String, ResolvedRegulatoryState>,
    /// Standard registry reference
    pub registry: Arc<StandardRegistry>,
    /// Generated data from upstream generators
    pub upstream_data: Arc<GeneratedData>,
    /// Audit procedure template engine
    pub template_engine: Arc<TemplateEngine>,
}

/// Output from a compliance generator.
pub struct ComplianceOutput {
    /// Records to be written to output files
    pub records: Vec<(String, Vec<Box<dyn Serialize>>)>,  // (filename, records)
    /// Graph nodes contributed
    pub graph_nodes: Vec<GraphNode>,
    /// Graph edges contributed
    pub graph_edges: Vec<GraphEdge>,
}
```

### 8.7.2 Example: Custom Basel Generator

```rust
pub struct BaselCapitalGenerator;

impl ComplianceGenerator for BaselCapitalGenerator {
    fn name(&self) -> &str { "Basel Capital Requirements" }

    fn applicable_standards(&self) -> Vec<StandardId> {
        vec![
            StandardId::new("BASEL", "III-CAP"),
            StandardId::new("BASEL", "III-LCR"),
            StandardId::new("BASEL", "III-NSFR"),
        ]
    }

    fn generate(
        &self,
        rng: &mut ChaCha8Rng,
        config: &ComplianceRegulationsConfig,
        context: &ComplianceContext,
    ) -> Result<ComplianceOutput, GenerationError> {
        let mut output = ComplianceOutput::new();

        // Generate capital adequacy ratios
        let cet1_ratio = self.generate_cet1(rng, context);
        let lcr = self.generate_lcr(rng, context);
        let nsfr = self.generate_nsfr(rng, context);

        output.add_records("capital_adequacy.csv", vec![cet1_ratio, lcr, nsfr]);

        // Generate graph nodes for prudential requirements
        for std in self.applicable_standards() {
            output.add_node(GraphNode::new(
                0, // ID assigned by builder
                NodeType::Standard,
                std.0.clone(),
                format!("Basel: {}", std.0),
            ));
        }

        Ok(output)
    }
}

// Register the generator
compliance_factory.register(Box::new(BaselCapitalGenerator));
```

---

## 8.8 API Surface Summary

### 8.8.1 Public Types (datasynth-core)

| Type | Location | Purpose |
|------|----------|---------|
| `StandardId` | `models/compliance/standard_id.rs` | Canonical standard identifier |
| `ComplianceStandard` | `models/compliance/standard.rs` | Standard metadata |
| `TemporalVersion` | `models/compliance/temporal.rs` | Version with temporal bounds |
| `JurisdictionProfile` | `models/compliance/jurisdiction.rs` | Country compliance profile |
| `ComplianceAssertion` | `models/compliance/assertion.rs` | Audit assertion |
| `ComplianceFinding` | `models/compliance/finding.rs` | Audit finding |
| `RegulatoryFiling` | `models/compliance/filing.rs` | Filing requirement |
| `StandardCategory` | `models/compliance/enums.rs` | Standard categorization |
| `ComplianceDomain` | `models/compliance/enums.rs` | Compliance domain |
| `AuditProcedureTemplate` | `models/compliance/template.rs` | Procedure template |

### 8.8.2 Public Traits (datasynth-standards)

| Trait | Location | Purpose |
|-------|----------|---------|
| `ComplianceStandardGenerator` | `traits.rs` | Custom standard generation |
| `JurisdictionPlugin` | `jurisdiction.rs` | Custom jurisdiction logic |
| `ComplianceGenerator` | `generator.rs` | Custom compliance generation |
| `TemplateStep` | `templates/step.rs` | Custom audit step type |

### 8.8.3 Public APIs (datasynth-standards)

| API | Method | Description |
|-----|--------|-------------|
| `StandardRegistry::active_version` | `fn(&StandardId, NaiveDate) -> Option<&TemporalVersion>` | Get active version at date |
| `StandardRegistry::standards_for_jurisdiction` | `fn(&str, NaiveDate) -> Vec<&ComplianceStandard>` | Standards for a country |
| `StandardRegistry::cross_references` | `fn(&StandardId) -> Vec<CrossReference>` | Get cross-references |
| `StandardRegistry::register_custom` | `fn(&Path) -> Result<StandardId>` | Register custom standard |
| `TemporalResolver::resolve` | `fn(&str, NaiveDate, &EntityConfig) -> ResolvedRegulatoryState` | Resolve regulatory state |
| `TemplateEngine::compile` | `fn(&AuditProcedureTemplate) -> Result<CompiledProcedure>` | Compile a template |
| `TemplateEngine::execute` | `fn(&CompiledProcedure, &DataContext) -> ProcedureResult` | Execute a procedure |

### 8.8.4 Configuration (datasynth-config)

| Section | Type | Validated By |
|---------|------|-------------|
| `compliance_regulations` | `ComplianceRegulationsConfig` | `ComplianceValidator` |
| `compliance_regulations.registry` | `RegistryConfig` | Schema validation |
| `compliance_regulations.jurisdictions` | `JurisdictionConfig` | ISO code validation |
| `compliance_regulations.temporal` | `TemporalConfig` | Date range validation |
| `compliance_regulations.standards` | `StandardsSelectionConfig` | Registry cross-check |
| `compliance_regulations.audit_templates` | `AuditTemplateConfig` | Template schema validation |
| `compliance_regulations.graph` | `ComplianceGraphConfig` | Node/edge type validation |

---

## 8.9 Versioning & Backward Compatibility

### 8.9.1 Registry Versioning

The built-in registry is versioned alongside DataSynth releases:

| DataSynth Version | Registry Version | Changes |
|-------------------|-----------------|---------|
| v1.1.0 | 1.0 | Initial registry with IFRS, ISA, US GAAP, SOX, Basel, EU regs |
| v1.2.0 | 1.1 | Add ISSB S1/S2, IFRS 18, updated CSRD phases |
| v1.3.0 | 1.2 | Add XBRL taxonomies, expanded country profiles |

### 8.9.2 Configuration Backward Compatibility

The `compliance_regulations` section is entirely new in v1.1.0. Existing configurations without this section continue to work unchanged. The existing `accounting_standards` and `audit_standards` sections remain functional and are treated as the "Layer 0" configuration that the compliance framework enhances.

**Migration path:**
- v1.0 configs with `accounting_standards` + `audit_standards` → Works as-is
- v1.1 configs can add `compliance_regulations` alongside or instead of the existing sections
- When both are present, `compliance_regulations` takes precedence for overlapping settings

### 8.9.3 Custom Standard/Template Compatibility

Custom YAML files include a `schema_version` field:

```yaml
schema_version: "1.0"  # Validated against supported schema versions
standard:
  id: "CUSTOM-001"
  # ...
```

The framework validates custom files against the declared schema version and provides migration guidance when breaking changes occur.

---

## 8.10 Testing Custom Extensions

### 8.10.1 Template Testing

```bash
# Validate template syntax and schema
datasynth-data validate --template ./my-template.yaml

# Dry-run template execution (no full generation)
datasynth-data generate --config config.yaml --dry-run --template-only

# Generate with verbose template logging
RUST_LOG=datasynth_standards::templates=debug datasynth-data generate --config config.yaml
```

### 8.10.2 Standard Registry Testing

```bash
# Dump resolved standards for a date/jurisdiction
datasynth-data info --standards --country US --date 2025-06-30

# Validate custom standards
datasynth-data validate --standards-dir ./custom-standards

# Show supersession chains
datasynth-data info --supersession-chain IFRS-16
```

### 8.10.3 Jurisdiction Profile Testing

```bash
# Dump jurisdiction profile
datasynth-data info --jurisdiction DE --date 2025-06-30

# Validate custom profiles
datasynth-data validate --jurisdiction-profiles ./jurisdiction-profiles

# Show applicable standards for a company
datasynth-data info --company-compliance --config config.yaml
```

---

## 8.11 Future Extension Points (v1.2+)

The following extension points are reserved for future versions:

| Extension Point | Version | Description |
|----------------|---------|-------------|
| `LlmAugmentedRegulationText` | v1.2 | Generate natural-language regulation text using LLM integration |
| `ComplianceGapAnalyzer` | v1.2 | ML-powered gap analysis from generated graph |
| `XbrlTaxonomyGenerator` | v1.3 | Generate XBRL/iXBRL tagged financial statements |
| `RealTimeRegulatoryFeed` | v1.3 | Ingest live standard updates from regulatory APIs |
| `CrossBorderTransferPricing` | v1.2 | OECD BEPS transfer pricing scenarios |
| `ComplianceSimulator` | v1.3 | What-if analysis for regulatory changes |
