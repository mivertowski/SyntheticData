# Part 5: Custom Audit Procedure Templates

> **Parent:** [Compliance & Regulations Framework](00-index.md)
> **Status:** Implemented | **Date:** 2026-03-09

---

## 5.1 Overview

The Audit Procedure Template system provides a declarative YAML/JSON DSL for defining audit procedures, control tests, and compliance checks. Users can:

1. Use **built-in templates** covering common audit procedures (substantive tests, tests of controls, analytical procedures)
2. **Customize** built-in templates with parameter overrides
3. **Define new templates** for industry-specific or firm-specific procedures
4. **Link templates** to standards, assertions, and risk levels

Templates are compiled at runtime into executable generation logic that produces realistic audit workpaper data, test results, and findings.

---

## 5.2 Template Structure

### 5.2.1 Top-Level Schema

```yaml
# audit-procedures/revenue-substantive.yaml
template:
  id: "AP-REV-SUBST-001"
  name: "Revenue Substantive Testing — Contract Review"
  version: "2.0"
  category: substantive_test
  description: >
    Substantive testing of revenue transactions through contract review,
    cutoff testing, and completeness procedures.

  # Standards this procedure addresses
  standards:
    - id: "ISA-500"
      requirements: ["ISA-500.R1", "ISA-500.R2"]
    - id: "ISA-330"
      requirements: ["ISA-330.R18", "ISA-330.R20"]
    - id: "PCAOB-AS-2301"
      requirements: ["AS-2301.09", "AS-2301.10"]

  # Financial statement assertions tested
  assertions:
    - occurrence       # Transactions actually occurred
    - completeness     # All transactions recorded
    - accuracy         # Amounts correctly recorded
    - cutoff           # Recorded in correct period
    - classification   # Correctly classified in financial statements

  # Risk level this procedure addresses
  risk_levels: [significant, high]

  # Applicable account categories
  applicable_accounts:
    - revenue
    - accounts_receivable
    - contract_assets
    - deferred_revenue

  # Procedure steps (executed in order)
  steps:
    - step_id: "S1"
      name: "Sample Selection"
      type: sampling
      params:
        method: monetary_unit  # monetary_unit, random, stratified, haphazard
        confidence_level: 0.95
        tolerable_misstatement: 0.05  # 5% of population
        expected_misstatement: 0.01   # 1% expected error
        population: revenue_transactions
        stratification:
          - { stratum: "high_value", threshold: 100000, selection: "all" }
          - { stratum: "medium", min: 10000, max: 100000, sample_size: 40 }
          - { stratum: "low", max: 10000, sample_size: 20 }

    - step_id: "S2"
      name: "Contract Review"
      type: inspection
      params:
        document_type: customer_contract
        attributes_tested:
          - { attribute: "contract_signed", expected: true }
          - { attribute: "performance_obligations_identified", expected: true }
          - { attribute: "transaction_price_allocated", expected: true }
          - { attribute: "recognition_criteria_met", expected: true }
        exception_probability: 0.08  # 8% chance of exception per item

    - step_id: "S3"
      name: "Cutoff Testing"
      type: cutoff_test
      params:
        period_boundary: period_end
        window_days_before: 5
        window_days_after: 5
        sample_size: 25
        test_direction: both  # before_boundary, after_boundary, both
        misstatement_probability: 0.03

    - step_id: "S4"
      name: "Analytical Review"
      type: analytical_procedure
      params:
        procedure_type: trend_analysis
        comparisons:
          - { metric: "revenue_by_month", expectation: "prior_year_plus_growth" }
          - { metric: "gross_margin", expectation: "industry_benchmark" }
          - { metric: "revenue_per_customer", expectation: "prior_year" }
        investigation_threshold: 0.10  # Investigate if > 10% variance
        significant_variance_probability: 0.15

    - step_id: "S5"
      name: "Completeness Testing"
      type: completeness_test
      params:
        source_documents: [shipping_records, delivery_confirmations]
        target_records: revenue_transactions
        unrecorded_probability: 0.02
        sample_size: 30

  # Expected findings profile
  findings:
    overall_exception_rate: 0.05
    severity_distribution:
      - { severity: high, weight: 0.05 }
      - { severity: moderate, weight: 0.25 }
      - { severity: low, weight: 0.70 }
    finding_types:
      - { type: "missing_contract", probability: 0.02, severity: high }
      - { type: "unsigned_contract", probability: 0.03, severity: moderate }
      - { type: "cutoff_error", probability: 0.02, severity: moderate }
      - { type: "classification_error", probability: 0.01, severity: low }
      - { type: "rounding_difference", probability: 0.05, severity: low }

  # Workpaper output specification
  output:
    workpaper_ref: "WP-REV-100"
    sections:
      - "Objective and Scope"
      - "Population and Sampling"
      - "Test Results"
      - "Exceptions Identified"
      - "Conclusion"
    conclusion_template: >
      Based on our testing of {sample_size} revenue transactions totaling
      {sample_amount}, we identified {exception_count} exceptions.
      {conclusion_qualification}
```

### 5.2.2 Step Types

| Step Type | Description | Key Parameters |
|-----------|-------------|---------------|
| `sampling` | Statistical or judgmental sample selection | method, confidence, tolerable/expected misstatement, stratification |
| `inspection` | Document/record inspection | document_type, attributes_tested, exception_probability |
| `observation` | Process observation (physical) | process, observation_points, deviation_probability |
| `inquiry` | Interview/inquiry procedures | respondent_roles, topics, corroboration_required |
| `recalculation` | Independent recalculation | calculation_type, precision, rounding_tolerance |
| `reperformance` | Reperformance of a control | control_id, test_count, deviation_rate |
| `confirmation` | Third-party confirmation | confirmation_type, positive/negative, response_rate |
| `cutoff_test` | Period boundary testing | window_days, direction, misstatement_probability |
| `analytical_procedure` | Ratio/trend/reasonableness | procedure_type, comparisons, investigation_threshold |
| `completeness_test` | Source-to-record tracing | source_documents, unrecorded_probability |
| `journal_entry_test` | Journal entry review | selection_criteria, risk_indicators, test_count |

---

## 5.3 Built-In Template Library

### 5.3.1 Revenue Cycle

| Template ID | Name | Category | Standards |
|-------------|------|----------|-----------|
| `AP-REV-SUBST-001` | Revenue Substantive — Contract Review | Substantive | ISA-500, ISA-330, ASC-606 |
| `AP-REV-SUBST-002` | Revenue Cutoff Testing | Substantive | ISA-500, ISA-330 |
| `AP-REV-CTRL-001` | Revenue Controls — Authorization Testing | Test of Controls | ISA-330, SOX-404 |
| `AP-REV-CTRL-002` | Revenue Controls — Segregation of Duties | Test of Controls | ISA-315, SOX-404 |
| `AP-REV-ANAL-001` | Revenue Analytical Procedures | Analytical | ISA-520 |

### 5.3.2 Expenditure Cycle

| Template ID | Name | Category | Standards |
|-------------|------|----------|-----------|
| `AP-EXP-SUBST-001` | AP Substantive — Vendor Invoice Testing | Substantive | ISA-500 |
| `AP-EXP-SUBST-002` | AP Three-Way Match Testing | Substantive | ISA-500, SOX-404 |
| `AP-EXP-CTRL-001` | AP Controls — Approval Authorization | Test of Controls | ISA-330, SOX-404 |
| `AP-EXP-CTRL-002` | AP Controls — Vendor Master Changes | Test of Controls | ISA-315 |
| `AP-EXP-ANAL-001` | AP Analytical Procedures | Analytical | ISA-520 |

### 5.3.3 Treasury & Cash

| Template ID | Name | Category | Standards |
|-------------|------|----------|-----------|
| `AP-CSH-SUBST-001` | Bank Reconciliation Testing | Substantive | ISA-500 |
| `AP-CSH-CONF-001` | Bank Confirmation Procedures | Confirmation | ISA-505 |
| `AP-CSH-CTRL-001` | Cash Disbursement Controls | Test of Controls | ISA-330 |

### 5.3.4 Estimates & Fair Value

| Template ID | Name | Category | Standards |
|-------------|------|----------|-----------|
| `AP-EST-SUBST-001` | Accounting Estimate Evaluation | Substantive | ISA-540, ASC-820 |
| `AP-EST-SUBST-002` | Fair Value Measurement Testing | Substantive | ISA-540, IFRS-13 |
| `AP-EST-SUBST-003` | Impairment Testing Procedures | Substantive | IAS-36, ASC-360 |
| `AP-EST-CTRL-001` | Estimate Controls — Management Review | Test of Controls | ISA-540 |

### 5.3.5 Financial Close & Reporting

| Template ID | Name | Category | Standards |
|-------------|------|----------|-----------|
| `AP-CLS-SUBST-001` | Journal Entry Testing | Substantive | ISA-240, SOX-404 |
| `AP-CLS-SUBST-002` | Consolidation Elimination Testing | Substantive | ISA-600, IFRS-10 |
| `AP-CLS-CTRL-001` | Financial Close Controls — Reconciliation | Test of Controls | SOX-404 |
| `AP-CLS-ANAL-001` | Financial Statement Analytics | Analytical | ISA-520 |

### 5.3.6 Compliance-Specific

| Template ID | Name | Category | Standards |
|-------------|------|----------|-----------|
| `AP-SOX-CTRL-001` | ICFR Testing — Entity-Level Controls | Test of Controls | SOX-404, PCAOB-AS-2201 |
| `AP-SOX-CTRL-002` | ICFR Testing — Transaction-Level Controls | Test of Controls | SOX-404, PCAOB-AS-2201 |
| `AP-SOX-DEFM-001` | Deficiency Evaluation Matrix | Evaluation | SOX-404, ISA-265 |
| `AP-AML-CTRL-001` | AML Controls Testing | Test of Controls | EU-AMLD-6 |
| `AP-CSRD-SUBST-001` | Sustainability Assurance Procedures | Substantive | EU-CSRD, ISSB-S1 |

### 5.3.7 Going Concern & Special

| Template ID | Name | Category | Standards |
|-------------|------|----------|-----------|
| `AP-GC-EVAL-001` | Going Concern Evaluation | Evaluation | ISA-570 |
| `AP-RP-SUBST-001` | Related Party Transactions Testing | Substantive | ISA-550 |
| `AP-SE-SUBST-001` | Subsequent Events Review | Substantive | ISA-560 |
| `AP-FRD-SUBST-001` | Fraud Risk Procedures | Substantive | ISA-240, PCAOB-AS-2401 |

---

## 5.4 Template Customization

### 5.4.1 Parameter Overrides

Users can customize built-in templates without redefining them:

```yaml
compliance_regulations:
  audit_templates:
    customizations:
      - template_id: "AP-REV-SUBST-001"
        overrides:
          steps.S1.params.confidence_level: 0.99
          steps.S1.params.stratification[0].threshold: 200000
          steps.S2.params.exception_probability: 0.12
          findings.overall_exception_rate: 0.08
```

### 5.4.2 Template Inheritance

Custom templates can inherit from built-in templates:

```yaml
template:
  id: "CUSTOM-REV-001"
  name: "Revenue Testing — Insurance Industry"
  inherits_from: "AP-REV-SUBST-001"

  # Override specific standards
  standards:
    append:
      - id: "IFRS-17"
        requirements: ["IFRS-17.R40", "IFRS-17.R80"]

  # Override specific steps
  steps:
    override:
      - step_id: "S2"
        params:
          attributes_tested:
            append:
              - { attribute: "insurance_contract_boundary", expected: true }
              - { attribute: "risk_adjustment_calculated", expected: true }

    append:
      - step_id: "S6"
        name: "Insurance Contract Grouping Validation"
        type: recalculation
        params:
          calculation_type: contract_grouping
          precision: exact
          rounding_tolerance: 0.01
```

### 5.4.3 Conditional Steps

Steps can be conditional on the jurisdiction or entity configuration:

```yaml
steps:
  - step_id: "S7"
    name: "SOX ICFR Walkthrough"
    type: observation
    condition:
      jurisdiction_has: "SOX-404"
      entity_type: [accelerated_filer, large_accelerated_filer]
    params:
      process: revenue_recognition
      observation_points: 3
      deviation_probability: 0.04
```

---

## 5.5 Template Execution Engine

### 5.5.1 Compilation

Templates are compiled into a `CompiledProcedure` at initialization:

```rust
pub struct CompiledProcedure {
    /// Template metadata
    pub template: AuditProcedureTemplate,
    /// Resolved standards (with active versions)
    pub resolved_standards: Vec<ResolvedStandard>,
    /// Compiled steps (with resolved parameters)
    pub compiled_steps: Vec<CompiledStep>,
    /// RNG state for deterministic generation
    pub rng_seed: u64,
}

pub struct CompiledStep {
    pub step_id: String,
    pub step_type: StepType,
    /// Resolved parameters (after inheritance and overrides)
    pub params: StepParams,
    /// Whether this step is active (condition evaluated)
    pub active: bool,
}
```

### 5.5.2 Execution

Each compiled step generates output records:

```rust
impl CompiledProcedure {
    pub fn execute(
        &self,
        rng: &mut ChaCha8Rng,
        data_context: &DataContext,  // Access to generated transactions, accounts, etc.
    ) -> ProcedureResult {
        let mut result = ProcedureResult::new(&self.template);

        for step in &self.compiled_steps {
            if !step.active {
                continue;
            }

            let step_result = match step.step_type {
                StepType::Sampling => self.execute_sampling(rng, step, data_context),
                StepType::Inspection => self.execute_inspection(rng, step, data_context),
                StepType::CutoffTest => self.execute_cutoff(rng, step, data_context),
                StepType::AnalyticalProcedure => self.execute_analytical(rng, step, data_context),
                StepType::Confirmation => self.execute_confirmation(rng, step, data_context),
                StepType::Reperformance => self.execute_reperformance(rng, step, data_context),
                StepType::JournalEntryTest => self.execute_jet(rng, step, data_context),
                // ... other step types
            };

            result.add_step_result(step_result);
        }

        result.generate_findings(rng, &self.template.findings);
        result.generate_conclusion(&self.template.output);
        result
    }
}
```

### 5.5.3 Output Records

Each procedure execution produces:

| Record Type | Description | Output File |
|-------------|-------------|-------------|
| `AuditProcedureRecord` | Procedure metadata, scope, conclusion | `audit_procedures.csv` |
| `ProcedureStepResult` | Per-step test results | `procedure_step_results.csv` |
| `SampleItem` | Individual items tested | `audit_samples.csv` |
| `TestException` | Exceptions identified | `audit_exceptions.csv` |
| `AuditFinding` | Aggregated findings | `audit_findings.csv` (existing) |
| `ProcedureWorkpaper` | Workpaper documentation | `audit_workpapers.csv` (existing) |

---

## 5.6 Sampling Methods

The template engine implements ISA 530 sampling methodologies:

### 5.6.1 Monetary Unit Sampling (MUS)

```rust
pub struct MonetaryUnitSampling {
    pub confidence_level: f64,       // e.g., 0.95
    pub tolerable_misstatement: f64, // e.g., 0.05 (5% of population)
    pub expected_misstatement: f64,  // e.g., 0.01 (1%)
}

impl MonetaryUnitSampling {
    /// Calculate sample size using Poisson approximation.
    pub fn sample_size(&self, population_value: f64) -> usize {
        let reliability = self.reliability_factor();
        let interval = population_value * self.tolerable_misstatement / reliability;
        (population_value / interval).ceil() as usize
    }

    fn reliability_factor(&self) -> f64 {
        // Poisson reliability factors
        match (self.confidence_level * 100.0) as u32 {
            95 => 3.0,
            90 => 2.31,
            80 => 1.61,
            _ => 3.0,
        }
    }
}
```

### 5.6.2 Stratified Sampling

```rust
pub struct StratifiedSampling {
    pub strata: Vec<Stratum>,
}

pub struct Stratum {
    pub name: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub selection: StratumSelection,
}

pub enum StratumSelection {
    /// Select all items in this stratum
    All,
    /// Fixed sample size
    Fixed(usize),
    /// Proportional to stratum size
    Proportional(f64),
    /// Based on risk assessment
    RiskBased { risk_level: RiskLevel, multiplier: f64 },
}
```

---

## 5.7 Assertion Framework

Templates reference audit assertions per ISA 315:

### 5.7.1 Assertion Types

```rust
pub enum AuditAssertion {
    // Transaction-level assertions
    Occurrence,        // Transactions occurred and pertain to the entity
    Completeness,      // All transactions recorded
    Accuracy,          // Amounts correctly recorded
    Cutoff,            // Recorded in correct period
    Classification,    // Correctly classified

    // Balance-level assertions
    Existence,         // Assets/liabilities exist
    RightsObligations, // Entity holds rights / owes obligations
    CompletenessBalance, // All balances recorded
    ValuationAllocation, // Appropriate valuation and allocation

    // Disclosure assertions
    OccurrenceRightsObligations, // Disclosed events occurred
    CompletenessDisclosure,       // All required disclosures made
    ClassificationUnderstandability, // Properly classified and clear
    AccuracyValuation,             // Disclosed amounts are accurate
}
```

### 5.7.2 Assertion-to-Account Mapping

The framework maintains a mapping of which assertions are relevant for each account category:

| Account Category | Key Assertions | Risk Focus |
|-----------------|----------------|-----------|
| Revenue | Occurrence, Cutoff, Accuracy | Overstatement |
| Accounts Receivable | Existence, Valuation, Completeness | Overstatement |
| Inventory | Existence, Valuation | Overstatement, obsolescence |
| Fixed Assets | Existence, Valuation, Completeness | Overstatement, impairment |
| Accounts Payable | Completeness, Accuracy | Understatement |
| Provisions | Completeness, Valuation | Understatement |
| Revenue (Deferred) | Completeness, Cutoff | Understatement |

---

## 5.8 Integration with Existing Audit Module

The template system extends — not replaces — the existing `datasynth-generators/src/audit/` module:

| Existing Module | Integration Point |
|----------------|------------------|
| `engagement.rs` | Templates are linked to engagements via `engagement_id` |
| `workpaper.rs` | Template output feeds into workpaper generation |
| `evidence.rs` | Step results become audit evidence records |
| `risk.rs` | Template risk levels map to engagement risk assessments |
| `finding.rs` | Template findings feed into existing finding aggregation |
| `judgment.rs` | Template conclusions reference professional judgments |

```
Existing Pipeline:                    Template Pipeline:
engagement_generator ──┐              ┌── template_engine
workpaper_generator ───┤              ├── step_executor
evidence_generator ────┤  ◄──merge──► ├── finding_generator
risk_generator ────────┤              ├── sampling_engine
finding_generator ─────┤              └── assertion_mapper
judgment_generator ────┘
```

---

## 5.9 Template Directory Structure

```
audit-procedures/
├── built-in/
│   ├── revenue/
│   │   ├── AP-REV-SUBST-001.yaml
│   │   ├── AP-REV-SUBST-002.yaml
│   │   ├── AP-REV-CTRL-001.yaml
│   │   ├── AP-REV-CTRL-002.yaml
│   │   └── AP-REV-ANAL-001.yaml
│   ├── expenditure/
│   │   └── ...
│   ├── treasury/
│   │   └── ...
│   ├── estimates/
│   │   └── ...
│   ├── close/
│   │   └── ...
│   └── compliance/
│       ├── AP-SOX-CTRL-001.yaml
│       ├── AP-SOX-CTRL-002.yaml
│       ├── AP-AML-CTRL-001.yaml
│       └── AP-CSRD-SUBST-001.yaml
├── custom/              # User-defined templates
│   └── ...
└── schema.json          # JSON Schema for template validation
```

Configuration to load custom templates:

```yaml
compliance_regulations:
  audit_templates:
    built_in: true  # Include built-in templates
    custom_dir: "./audit-procedures/custom"
    # Selectively enable/disable template categories
    categories:
      substantive_test: true
      test_of_controls: true
      analytical: true
      confirmation: true
      compliance: true
```
