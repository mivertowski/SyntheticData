# Counterfactual & What-If Simulation — Roadmap

**Date**: 2026-02-19
**Status**: Draft
**Scope**: Multi-phase enhancement to transform DataSynth from a synthetic data generator into a counterfactual simulation platform

---

## Executive Summary

DataSynth already possesses the deep domain modeling (accounting rules, document chains, control frameworks, process flows) that makes **constrained counterfactual simulation** possible — a capability no competing synthetic data platform offers. This roadmap extends those foundations into a full what-if simulation engine that generates paired baseline/counterfactual datasets, propagates interventions through a causal model, and provides interactive exploration.

**Why this matters for adoption:**

| Audience | Value Proposition |
|----------|------------------|
| **Auditors & Risk teams** | "What if control C003 failed for 3 months?" — model the undetected-error population without touching production |
| **ML/AI engineers** | Matched factual/counterfactual pairs with ground-truth labels — the holy grail for fraud detection training |
| **CFOs & Controllers** | Slider-based assumption exploration: drag interest-rate to +200bp, see real-time financial statement impact |
| **Regulators & Compliance** | Stress testing (CCAR/DFAST-style) at the transaction level, not just portfolio aggregates |
| **Academics & Students** | Hands-on scenario exploration for accounting, auditing, and forensic analytics courses |
| **Community contributors** | Scenario library — share, rate, and compose named scenarios ("Stagflation 2.0", "Vendor Collusion Ring") |

---

## Current State Assessment

### What already exists

DataSynth has substantial building blocks scattered across crates:

| Capability | Location | Maturity |
|-----------|----------|----------|
| `CounterfactualPair` + `CounterfactualSpec` (11 modification types) | `datasynth-generators/src/counterfactual/mod.rs` | Alpha — single-JE scope |
| Drift controller (gradual, sudden, regime, economic cycle) | `datasynth-core/src/distributions/drift.rs` | Production-ready |
| Behavioral drift (vendor, customer, employee, collective) | `datasynth-core/src/distributions/behavioral_drift.rs` | Production-ready |
| Market drift (industry cycles, commodity shocks) | `datasynth-core/src/distributions/market_drift.rs` | Production-ready |
| Disruption modeling (outages, migrations, process changes) | `datasynth-generators/src/disruption/mod.rs` | Beta |
| Anomaly injection (100+ patterns) | `datasynth-generators/src/anomaly/` | Production-ready |
| Causal model evaluation (DAG, interventions) | `datasynth-eval/src/causal/` | Beta |
| Auto-tuner & recommendation engine | `datasynth-eval/src/enhancement/` | Beta |
| Fingerprint extraction → synthesis | `datasynth-fingerprint/` | Production-ready |
| Scenario tags + data quality profiles | `datasynth-config/src/schema.rs` | Production-ready |

### The gap

These pieces operate independently. There is no unified **Scenario Engine** that:

1. Accepts a named scenario definition with typed interventions
2. Validates scenario consistency against a causal model
3. Propagates interventions through dependent parameters
4. Generates paired baseline + counterfactual datasets sharing the same seed for unchanged portions
5. Produces structured comparison output (diffs, impact summaries, sensitivity metrics)
6. Exposes interactive exploration via the UI

---

## Roadmap Phases

```
Phase 1        Phase 2          Phase 3            Phase 4           Phase 5
Scenario       Causal           Interactive         ML/AI             Community
Engine         Propagation      Exploration         Integration       & Marketplace
────────────── ──────────────── ─────────────────── ────────────────── ──────────────
Q2 2026        Q3 2026          Q4 2026             Q1 2027           Q2 2027
```

---

## Phase 1 — Scenario Engine (Foundation)

**Goal**: Unified scenario definition format + paired generation + diff output.

### 1.1 Scenario Definition Schema

Extend YAML configuration with a top-level `scenarios` section:

```yaml
scenarios:
  - name: "supply_chain_disruption_q3"
    description: "Three strategic vendors default in Q3, triggering spot purchasing"
    tags: [supply_chain, risk, stress_test]
    base: default                        # or reference another scenario by name
    probability_weight: 0.15             # for IFRS 9-style probability-weighted outcomes
    interventions:
      - type: entity_event
        subtype: vendor_default
        target:
          vendor_cluster: reliable_strategic
          count: 3
        timing:
          start_month: 7
          duration_months: 4
          onset: sudden                  # sudden | gradual | oscillating
      - type: parameter_shift
        target: distributions.amounts.components[0].mu
        from: 6.0
        to: 5.5
        timing:
          start_month: 7
          ramp_periods: 2               # linear interpolation over 2 months
      - type: control_failure
        target: control_id: "C003"      # three-way match
        severity: 0.60                  # effectiveness drops to 60%
        timing:
          start_month: 8
          duration_months: 3
    constraints:
      preserve_accounting_identity: true
      preserve_document_chains: true
      preserve_period_close: true
    output:
      paired: true                       # generate baseline alongside
      diff_format: [summary, record_level, aggregate]
```

### 1.2 Intervention Type Library

| Category | Intervention Types | Maps To |
|----------|-------------------|---------|
| **Entity Events** | vendor_default, customer_churn, employee_departure, new_vendor_onboarding, merger_acquisition | Behavioral drift + master data changes |
| **Parameter Shifts** | amount_distribution, volume_multiplier, error_rate, approval_threshold | Drift config parameters |
| **Control Failures** | control_effectiveness, sod_bypass, approval_override, it_control_failure | Control generator + anomaly injection |
| **Process Changes** | new_approval_level, policy_change, system_migration, process_automation | Disruption module |
| **Macro Shocks** | recession, inflation_spike, currency_crisis, interest_rate_shock, commodity_shock | Economic cycle + market drift |
| **Regulatory Changes** | new_standard_adoption, materiality_change, reporting_requirement, threshold_change | Standards module |
| **Custom** | user_defined_function | Lambda-style transformation |

### 1.3 Paired Generation Engine

Core algorithm:

```
fn generate_scenario(base_config, scenario_def):
    # 1. Snapshot the base RNG state
    base_seed = config.global.seed

    # 2. Generate baseline dataset
    baseline = generate(base_config, seed=base_seed)

    # 3. Apply interventions to create counterfactual config
    cf_config = apply_interventions(base_config, scenario_def.interventions)

    # 4. Generate counterfactual with SAME seed for structural consistency
    counterfactual = generate(cf_config, seed=base_seed)

    # 5. Compute diff
    diff = compute_diff(baseline, counterfactual, scenario_def.output.diff_format)

    return ScenarioResult { baseline, counterfactual, diff, metadata }
```

Sharing the seed ensures that entities generated before any intervention timing are identical, and divergence occurs naturally at the intervention point.

### 1.4 Scenario Diff Output

```
output/
├── baseline/                    # Full baseline dataset
│   └── (all standard output files)
├── scenarios/
│   └── supply_chain_disruption_q3/
│       ├── data/                # Full counterfactual dataset
│       │   └── (all standard output files)
│       ├── diff/
│       │   ├── impact_summary.json      # High-level KPI changes
│       │   ├── record_level_diff.csv    # Record-by-record changes
│       │   ├── aggregate_comparison.json # Aggregated metrics comparison
│       │   └── intervention_trace.json  # Which interventions caused which changes
│       └── scenario_manifest.yaml       # The scenario definition + execution metadata
```

### 1.5 CLI Integration

```bash
# Generate with scenarios
datasynth-data generate --config config.yaml --output ./output --scenarios all

# Generate specific scenario only
datasynth-data generate --config config.yaml --output ./output --scenario supply_chain_disruption_q3

# List available scenarios in config
datasynth-data scenario list --config config.yaml

# Validate scenario consistency
datasynth-data scenario validate --config config.yaml --scenario supply_chain_disruption_q3

# Compare two scenario outputs
datasynth-data scenario diff --baseline ./output/baseline --counterfactual ./output/scenarios/supply_chain_disruption_q3/data
```

### 1.6 Implementation Plan

| Task | Crate | Effort |
|------|-------|--------|
| `ScenarioConfig` schema + YAML deserialization | `datasynth-config` | M |
| `InterventionType` enum + intervention application logic | `datasynth-config` | L |
| `ScenarioEngine` orchestration (paired generation) | `datasynth-runtime` | L |
| `ScenarioDiff` computation (summary, record-level, aggregate) | `datasynth-eval` | M |
| CLI `scenario` subcommand | `datasynth-cli` | S |
| Refactor existing `CounterfactualGenerator` to use new schema | `datasynth-generators` | M |
| Integration tests (baseline ↔ counterfactual consistency) | `datasynth-test-utils` | M |

**Estimated scope**: ~4-6 weeks for a single developer

---

## Phase 2 — Causal Propagation Engine

**Goal**: Interventions automatically ripple through a causal DAG. Users define "what" changes; the engine determines "what else" changes as a consequence.

### 2.1 Financial Process Causal DAG

Define a domain-specific causal model connecting operational parameters to outcomes:

```
EconomicConditions ──→ TransactionVolume
                   ──→ DefaultRate
                   ──→ InflationRate

TransactionVolume ──→ StaffingPressure
                  ──→ ProcessingBacklog

StaffingPressure ──→ ControlEffectiveness
                 ──→ ErrorRate
                 ──→ ProcessingLagDays

ControlEffectiveness ──→ FraudDetectionRate
                     ──→ UndetectedErrors

DefaultRate ──→ BadDebtExpense ──→ FinancialStatements
ErrorRate ──→ MisstatementRisk ──→ AuditFindings

InflationRate ──→ PurchasePrices ──→ COGS ──→ GrossMargin
              ──→ PayrollCosts

VendorDefault ──→ SpotPurchasing ──→ UnitCostIncrease
              ──→ SupplyDelay ──→ ProductionDelay
              ──→ AlternateVendorOnboarding

CustomerChurn ──→ RevenueDecline ──→ CashFlowPressure
              ──→ ARWriteOff
```

### 2.2 Propagation Mechanics

Each edge in the DAG carries a **transfer function** specifying how upstream changes translate to downstream effects:

```yaml
causal_model:
  edges:
    - from: economic_conditions.gdp_growth
      to: transaction_volume
      transfer: linear
      coefficient: 0.8           # 1% GDP change → 0.8% volume change
      lag_months: 1

    - from: transaction_volume
      to: staffing_pressure
      transfer: threshold
      threshold: 1.2             # pressure activates above 120% of normal volume
      saturation: 2.0            # caps at 200%

    - from: staffing_pressure
      to: control_effectiveness
      transfer: inverse_logistic
      midpoint: 1.5              # 50% effectiveness loss at 150% pressure
      steepness: 3.0

    - from: vendor_default
      to: spot_purchasing
      transfer: step
      magnitude: 0.30            # 30% of defaulted vendor's volume goes to spot
      recovery_months: 6         # spot purchasing decays as new vendors qualify
```

**Transfer function types**: `linear`, `exponential`, `logistic`, `inverse_logistic`, `step`, `threshold`, `decay`, `custom_fn`

### 2.3 Consistency Validation

The causal engine validates scenario coherence:

- **No contradictions**: Interventions on the same node must be compatible
- **Temporal ordering**: Causes precede effects (respecting lag parameters)
- **Accounting identity preservation**: Propagated changes maintain debits = credits
- **Constraint satisfaction**: User-specified constraints override propagation where they conflict (with warnings)

### 2.4 Reverse Stress Testing

Given a failure condition, find scenarios that produce it:

```bash
datasynth-data scenario reverse-stress \
  --target "audit_findings.material_weakness >= 1" \
  --free-variables economic_conditions,vendor_defaults,control_effectiveness \
  --max-scenarios 10 \
  --config config.yaml
```

The engine searches the intervention space using the causal DAG to find minimal interventions that reach the target condition.

### 2.5 Implementation Plan

| Task | Crate | Effort |
|------|-------|--------|
| `CausalDAG` data structure + YAML schema | `datasynth-core` | M |
| Transfer function library (7 types) | `datasynth-core` | M |
| `PropagationEngine` (topological-sort forward pass) | `datasynth-runtime` | L |
| Consistency validator | `datasynth-config` | M |
| Default DAG for financial processes | `datasynth-config/templates` | M |
| Reverse stress test solver (constraint optimization) | `datasynth-eval` | XL |
| Extend existing `datasynth-eval/causal` module | `datasynth-eval` | M |

**Estimated scope**: ~6-8 weeks

---

## Phase 3 — Interactive Exploration

**Goal**: Visual scenario builder in the Tauri/SvelteKit desktop UI with real-time feedback and comparison dashboards.

### 3.1 Scenario Builder UI

**Layout**: Three-panel design

```
┌─────────────────────────────────────────────────────────────────┐
│ [Scenario: supply_chain_disruption_q3 ▼]  [Save] [Compare] [▶] │
├──────────────┬─────────────────────────┬────────────────────────┤
│  ASSUMPTIONS │      TIMELINE           │     IMPACT PREVIEW     │
│              │                         │                        │
│  Macro       │  ──┬──┬──┬──┬──┬──→    │  Revenue: -12.3%       │
│  ┊ GDP  ──●──│    │  │  │  ◆  │       │  COGS:    +8.1%        │
│  ┊ Rate ──●──│    │  │  │  ║  │       │  Margin:  -20.4%       │
│  ┊ FX   ──●──│    │  │  │  ║  │       │                        │
│              │    │  │  ◆━━╝  │       │  Anomalies: +340%      │
│  Operational │    │  │  ║     │       │  Control gaps: 3       │
│  ┊ Vendors ──│    │  │  ║     │       │                        │
│  ┊ Controls ─│    │  │  ║     │       │  ┌──────────────────┐  │
│  ┊ Staff   ──│    │  │  ║     │       │  │  [Tornado Chart]  │  │
│              │ M1 M2 M3 M4  M5 M6    │  │  GDP ████████░░   │  │
│  Regulatory  │                         │  │  Rate ██████░░░   │  │
│  ┊ Material ─│  ◆ = Event dropped     │  │  FX   ████░░░░░   │  │
│  ┊ Threshold │  ║ = Duration          │  └──────────────────┘  │
│              │  ● = Slider value      │                        │
└──────────────┴─────────────────────────┴────────────────────────┘
```

**Left panel — Assumption sliders**:
- Grouped by category (Macro, Operational, Regulatory, Custom)
- Each slider shows current value, baseline reference, min/max bounds
- Linked sliders (from causal DAG) show propagated effects in real time
- "Lock" toggle to prevent a parameter from being affected by propagation

**Center panel — Timeline**:
- Horizontal axis = generation period (months)
- Users drag-and-drop intervention events onto the timeline
- Events show their type, duration, and magnitude as visual indicators
- Overlapping events are stacked with interaction warnings

**Right panel — Impact preview**:
- Key KPIs with baseline vs. counterfactual comparison
- Tornado diagram showing parameter sensitivity
- Mini financial statement comparison (top 10 changed line items)
- Updates in sub-second time using pre-computed approximations

### 3.2 Comparison Dashboard

After full generation, a dedicated comparison view:

| Visualization | Purpose |
|--------------|---------|
| **Side-by-side financial statements** | Balance sheet, P&L, cash flow — baseline vs. counterfactual with highlighted deltas |
| **Tornado diagram** | Rank assumptions by impact on selected KPI |
| **Spider/radar chart** | Multi-KPI comparison across 2-5 named scenarios |
| **Fan chart** | Time series with uncertainty bands across probability-weighted scenarios |
| **Heat map** | 2D sensitivity: pick two parameters, see how output varies across both |
| **Parallel coordinates** | High-dimensional scenario comparison (each axis = one KPI or parameter) |
| **Sankey diagram** | Flow of financial impacts from intervention → intermediate effects → outcomes |
| **Record-level diff table** | Filterable, sortable table of individual record changes |

### 3.3 Sensitivity Analysis Automation

```bash
# Automated sensitivity analysis
datasynth-data scenario sensitivity \
  --config config.yaml \
  --parameters gdp_growth,interest_rate,vendor_default_rate,control_effectiveness \
  --target net_income,anomaly_count,material_weakness_risk \
  --steps 10 \
  --output ./sensitivity_results/
```

Generates a grid of scenarios varying each parameter, producing:
- Tornado diagram data (one-at-a-time sensitivity)
- Heat map data (pairwise sensitivity)
- Spider chart data (named scenario comparison)
- Interaction effects (which parameter combinations produce non-linear outcomes)

### 3.4 Implementation Plan

| Task | Crate | Effort |
|------|-------|--------|
| Scenario Builder Svelte component (3-panel layout) | `datasynth-ui` | XL |
| Assumption slider system with causal linking | `datasynth-ui` | L |
| Timeline drag-and-drop event editor | `datasynth-ui` | L |
| Impact preview with approximate fast-path computation | `datasynth-ui` + `datasynth-runtime` | L |
| Comparison dashboard (6 chart types) | `datasynth-ui` | XL |
| Sensitivity analysis CLI + computation engine | `datasynth-eval` + `datasynth-cli` | L |
| Chart library integration (ECharts or LayerCake) | `datasynth-ui` | M |
| WebSocket streaming for real-time preview updates | `datasynth-server` | M |

**Estimated scope**: ~8-12 weeks

---

## Phase 4 — ML/AI Integration

**Goal**: First-class counterfactual training data for fraud detection, fairness testing, and robustness evaluation.

### 4.1 Counterfactual Training Data API

Structured output formats for ML consumption:

```python
from datasynth_py import DataSynth, ScenarioConfig

ds = DataSynth()
result = ds.generate_counterfactual(
    config="config.yaml",
    scenario="vendor_fraud_ring",
    output_format="ml_pairs",      # matched factual/counterfactual pairs
)

# Returns:
# {
#   "pairs": [
#     {
#       "factual": { "journal_entry": {...}, "features": [...] },
#       "counterfactual": { "journal_entry": {...}, "features": [...] },
#       "treatment": "fraud_injection",
#       "treatment_effect": { "amount_delta": 15000, "anomaly_label": "FictitiousTransaction" },
#       "ground_truth_causal_effect": 1.0,
#     },
#     ...
#   ],
#   "metadata": {
#     "scenario": "vendor_fraud_ring",
#     "n_pairs": 5000,
#     "treatment_fraction": 0.10,
#   }
# }
```

**Output formats**:

| Format | Description | Use Case |
|--------|-------------|----------|
| `ml_pairs` | Matched factual/counterfactual record pairs with labels | Causal ML training (DoWhy, EconML, CausalML) |
| `treatment_control` | Standard treatment/control split with assignment indicator | Uplift modeling, A/B test simulation |
| `time_series_intervention` | Pre/post intervention time series with known intervention point | Interrupted time series analysis |
| `graph_counterfactual` | Paired graphs (PyG/DGL format) with node-level treatment labels | Graph neural network training |
| `fairness_audit` | Entities with counterfactual protected-attribute variants | Counterfactual fairness testing |

### 4.2 Fraud Detection Scenario Library

Pre-built scenarios that generate training data for specific fraud typologies:

```yaml
# Built-in scenario: vendor_collusion_ring
scenarios:
  - name: vendor_collusion_ring
    description: |
      Three vendors operated by the same beneficial owner submit
      coordinated bids to procurement. Winning vendor rotates.
      Prices are 15-25% above market. Kickbacks flow through
      ghost invoices on losing vendors.
    interventions:
      - type: entity_event
        subtype: vendor_collusion
        parameters:
          ring_size: 3
          price_inflation: [0.15, 0.25]
          rotation_pattern: round_robin
          kickback_rate: 0.08
          ghost_invoice_frequency: monthly
    expected_signals:
      - bid_price_clustering           # Bids are suspiciously close
      - vendor_address_similarity      # Shared registered addresses
      - round_robin_winning            # Win pattern is non-random
      - payment_timing_correlation     # Payments cluster on same dates
      - ghost_invoice_markers          # Invoices without matching GR/PO
```

### 4.3 Counterfactual Fairness Module

```bash
datasynth-data scenario fairness \
  --config config.yaml \
  --protected-attributes vendor_country,customer_segment,employee_gender \
  --model-under-test ./risk_model.onnx \
  --output ./fairness_report/
```

Generates counterfactual entities that differ only in protected attributes and tests whether model predictions change. Produces:
- Per-attribute fairness scores
- Worst-case counterfactual pairs (maximum prediction change)
- Intersectional fairness analysis (combinations of attributes)
- Remediation suggestions (which features are proxies for protected attributes)

### 4.4 Robustness Testing Suite

```yaml
robustness_test:
  distribution_shifts:
    - name: "recession_shift"
      scenario: recession_2008_replay
      expected_degradation_threshold: 0.15    # max 15% AUC drop
    - name: "regulatory_shift"
      scenario: new_revenue_standard
      expected_degradation_threshold: 0.10
  adversarial:
    perturbation_budget: 0.05                 # max 5% change per feature
    target_metric: fraud_detection_auc
    n_adversarial_samples: 1000
  temporal:
    train_periods: [1, 12]                    # train on months 1-12
    test_scenarios:                            # test on counterfactual months 13-24
      - baseline
      - recession
      - control_failure
```

### 4.5 Implementation Plan

| Task | Crate | Effort |
|------|-------|--------|
| `ml_pairs` output format + Python API | `datasynth-output` + `python/` | L |
| `treatment_control` and `time_series_intervention` formats | `datasynth-output` | M |
| `graph_counterfactual` format (PyG + DGL) | `datasynth-graph` | M |
| Fraud detection scenario library (10 pre-built) | `datasynth-config/templates` | L |
| Fairness testing module | `datasynth-eval` | L |
| Robustness testing suite | `datasynth-eval` | L |
| ONNX model loading for fairness/robustness tests | `datasynth-eval` | M |
| Python API extensions for ML workflows | `python/` | M |

**Estimated scope**: ~6-8 weeks

---

## Phase 5 — Community & Marketplace

**Goal**: Scenario library, sharing format, challenges, and educational resources that build an active user community.

### 5.1 Scenario Library

Curated collection of pre-built, validated scenarios:

```
scenarios/
├── macro/
│   ├── recession_2008_replay.yaml
│   ├── pandemic_disruption_2020.yaml
│   ├── interest_rate_shock_plus_300bp.yaml
│   ├── stagflation.yaml
│   ├── currency_crisis_em.yaml
│   └── supply_chain_global_disruption.yaml
├── fraud/
│   ├── vendor_collusion_ring.yaml
│   ├── management_override_revenue.yaml
│   ├── procurement_kickback_scheme.yaml
│   ├── ghost_employee_payroll.yaml
│   ├── channel_stuffing.yaml
│   ├── round_tripping_intercompany.yaml
│   └── expense_reimbursement_fraud.yaml
├── control_failures/
│   ├── sox_material_weakness.yaml
│   ├── it_general_control_breakdown.yaml
│   ├── sod_bypass_systematic.yaml
│   ├── three_way_match_failure.yaml
│   └── period_close_breakdown.yaml
├── regulatory/
│   ├── ifrs_17_transition.yaml
│   ├── new_lease_standard_adoption.yaml
│   ├── revenue_recognition_restatement.yaml
│   ├── materiality_threshold_reduction.yaml
│   └── sox_first_year_implementation.yaml
├── operational/
│   ├── erp_migration_cutover.yaml
│   ├── key_person_departure.yaml
│   ├── rapid_growth_stress.yaml
│   ├── acquisition_integration.yaml
│   └── outsourcing_transition.yaml
└── industry/
    ├── banking/
    │   ├── credit_crisis.yaml
    │   ├── aml_sanctions_failure.yaml
    │   └── interest_rate_mismatch.yaml
    ├── manufacturing/
    │   ├── quality_recall.yaml
    │   ├── raw_material_shortage.yaml
    │   └── production_line_failure.yaml
    ├── retail/
    │   ├── seasonal_demand_shock.yaml
    │   ├── ecommerce_migration.yaml
    │   └── inventory_shrinkage_spike.yaml
    └── healthcare/
        ├── regulatory_audit.yaml
        ├── reimbursement_rate_cut.yaml
        └── pandemic_surge.yaml
```

Each scenario file contains:
- **Narrative description**: The real-world situation being modeled
- **Interventions**: Parameter modifications
- **Expected signals**: What patterns should appear in generated data
- **Validation criteria**: Automated checks that the scenario applied correctly
- **Tags + metadata**: For discoverability and filtering
- **Difficulty rating**: How subtle the signals are (for ML training calibration)
- **References**: Academic papers, case studies, or regulatory guidance that inspired the scenario

### 5.2 Portable Scenario Format (`.dss` — DataSynth Scenario)

A self-contained, shareable scenario definition:

```yaml
# header
format_version: "1.0"
name: "vendor_collusion_ring"
author: "DataSynth Community"
license: "Apache-2.0"
created: "2026-03-15"
tags: [fraud, procurement, collusion, bid_rigging]
difficulty: hard
description: |
  Simulates a vendor collusion ring...

# compatibility
requires:
  datasynth_version: ">=0.9.0"
  modules: [document_flows, master_data, anomaly]
  config_sections: [vendor_network, source_to_pay]

# the scenario itself
interventions:
  - type: entity_event
    ...

# validation
expected_effects:
  - metric: bid_price_variance
    direction: decrease
    magnitude: [0.3, 0.5]
  - metric: vendor_win_rate_entropy
    direction: decrease

# documentation
walkthrough: |
  Step-by-step explanation of how this scenario
  manifests in the generated data, useful for
  training and education...

references:
  - title: "Bid Rigging: Red Flags and Detection Strategies"
    url: "https://example.com/paper"
    type: academic
```

### 5.3 Challenge Platform

Community competitions using counterfactual scenarios:

**Challenge types**:

| Challenge | Format | Audience |
|-----------|--------|----------|
| **Fraud Detection Challenge** | Generate scenario with embedded fraud; participants build detection models; score by AUC on held-out counterfactuals | ML engineers |
| **Audit Efficiency Challenge** | Generate scenario with embedded errors; participants design audit sampling strategies; score by detection rate at minimum sample size | Auditors |
| **Root Cause Analysis Challenge** | Generate scenario with known interventions; participants identify which interventions occurred from the data alone | Data analysts |
| **Forecasting Challenge** | Generate baseline + intervention; participants predict KPI impacts before seeing counterfactual data | Finance professionals |
| **Scenario Design Challenge** | Given a real-world news event, participants design the most realistic scenario definition | Domain experts |

### 5.4 Educational Resources

- **Interactive tutorials**: Step-by-step scenario building in the UI with guided exploration
- **Course modules**: Pre-built datasets + exercises for university courses (accounting, auditing, forensic analytics, data science)
- **Certification scenarios**: Practice datasets aligned with CPA, CIA, CISA, CFE exam content areas
- **Case studies**: Real-world-inspired scenarios with full discussion guides

### 5.5 Implementation Plan

| Task | Crate/Area | Effort |
|------|-----------|--------|
| Scenario library (30+ pre-built scenarios) | `scenarios/` | XL |
| `.dss` format specification + parser | `datasynth-config` | M |
| Scenario import/export CLI commands | `datasynth-cli` | S |
| Scenario validation runner (expected_effects checks) | `datasynth-eval` | M |
| Community contribution guide + templates | `docs/` | M |
| Challenge framework (submission, scoring, leaderboard) | Separate service | XL |
| Documentation + tutorials | `docs/` | L |

**Estimated scope**: ~8-12 weeks

---

## Cross-Cutting Concerns

### Performance

Paired generation doubles compute time. Mitigations:
- **Lazy diff**: Only compute differences for requested output files, not all 100+ export types
- **Shared prefix optimization**: When intervention timing is month 7, reuse months 1-6 output verbatim
- **Approximate preview mode**: For UI sliders, use pre-computed linear approximations instead of full generation
- **Parallel scenario generation**: Independent scenarios can run on separate threads/cores

### Backward Compatibility

- Scenarios are purely additive — existing configs without `scenarios:` continue to work unchanged
- The existing `CounterfactualGenerator` in `datasynth-generators` is refactored into the new Scenario Engine but the old API remains as a convenience wrapper
- CLI behavior is unchanged unless `--scenario` flags are used

### Testing Strategy

| Level | Coverage |
|-------|----------|
| Unit | Intervention application, causal propagation, transfer functions |
| Integration | Paired generation consistency (shared records are byte-identical) |
| Property | Accounting identity holds in all counterfactuals |
| Snapshot | Named scenarios produce deterministic output |
| Evaluation | Generated scenarios pass their own expected_effects validation |

---

## Success Metrics

| Phase | Metric | Target |
|-------|--------|--------|
| 1 | Scenarios definable in YAML | 10+ intervention types |
| 1 | Paired generation overhead | < 2.1x baseline time |
| 2 | Causal propagation coverage | 80%+ of parameter changes auto-propagate |
| 2 | Reverse stress test | Finds valid scenarios in < 60s |
| 3 | UI scenario exploration | Sub-500ms slider response time |
| 3 | Comparison visualizations | 6+ chart types |
| 4 | ML output formats | 5 formats (pairs, T/C, time series, graph, fairness) |
| 4 | Pre-built fraud scenarios | 10+ typologies |
| 5 | Community scenarios | 30+ in library |
| 5 | Challenge completions | Track adoption |

---

## Dependencies & Risks

| Risk | Mitigation |
|------|------------|
| Causal DAG complexity → maintenance burden | Start with a curated default DAG; allow but don't require user customization |
| Performance regression from paired generation | Shared-prefix optimization; lazy diff computation |
| Scenario combinatorial explosion | Constrain UI to max 5 simultaneous interventions; provide composition guardrails |
| Community adoption requires critical mass | Seed the library with 30+ high-quality scenarios; partner with academic institutions |
| UI complexity alienates non-technical users | Progressive disclosure: simple mode (named scenarios) vs. advanced mode (custom interventions) |

---

## Appendix: Competitive Landscape

| Platform | Counterfactual Capability | DataSynth Differentiator |
|----------|--------------------------|-------------------------|
| Mostly AI | Conditional generation (fix columns, regenerate) | No accounting constraint preservation |
| Gretel Navigator | LLM-prompted scenario description | No causal model, no document chain integrity |
| SAS Risk Mgmt | Macro scenarios → portfolio losses | Top-down only; no transaction-level generation |
| Palantir Foundry | Ontology-grounded simulation | Proprietary; no synthetic data generation |
| SAP Analytics Cloud | Slider-based planning scenarios | No synthetic transaction data; only aggregates |
| DataSynth (proposed) | **Causal DAG + accounting constraints + transaction-level paired generation + interactive exploration** | **Full stack: definition → propagation → generation → comparison → ML output** |
