# Research: Pattern and Process Drift Over Time

> **Implementation Status: COMPLETE** (v0.3.0)
>
> This research document has been fully implemented. See the following modules:
> - `datasynth-core/src/models/organizational_event.rs` - Organizational events
> - `datasynth-core/src/models/process_evolution.rs` - Process evolution types
> - `datasynth-core/src/models/technology_transition.rs` - Technology transitions
> - `datasynth-core/src/models/regulatory_events.rs` - Regulatory changes
> - `datasynth-core/src/models/drift_events.rs` - Ground truth labels
> - `datasynth-core/src/distributions/behavioral_drift.rs` - Behavioral drift
> - `datasynth-core/src/distributions/market_drift.rs` - Market/economic drift
> - `datasynth-core/src/distributions/event_timeline.rs` - Event orchestration
> - `datasynth-core/src/distributions/drift_recorder.rs` - Ground truth recording
> - `datasynth-eval/src/statistical/drift_detection.rs` - Drift detection evaluation
> - `datasynth-config/src/schema.rs` - Configuration types

## Current State Analysis

### Existing Drift Implementation

The current `DriftController` (373 lines) supports:

| Drift Type | Implementation | Realism |
|------------|----------------|---------|
| Gradual | Linear parameter drift | Medium |
| Sudden | Point-in-time shifts | Medium |
| Recurring | Seasonal patterns | Good |
| Mixed | Combination modes | Medium |

### Current Capabilities

1. **Amount drift**: Mean and variance adjustments over time
2. **Anomaly rate drift**: Changing fraud/error rates
3. **Concept drift factor**: Generic drift indicator
4. **Seasonal adjustment**: Periodic recurring patterns
5. **Sudden drift probability**: Random regime changes

### Current Gaps

1. **No organizational events**: Mergers, restructuring not modeled
2. **No process evolution**: Static business processes
3. **No regulatory changes**: Compliance requirements don't evolve
4. **No technology transitions**: System changes not simulated
5. **No behavioral drift**: Entity behaviors remain static
6. **No market-driven drift**: External factors not modeled
7. **Limited drift detection signals**: Hard to validate drift presence

---

## Improvement Recommendations

### 1. Organizational Event Modeling

#### 1.1 Corporate Event Timeline

```yaml
organizational_events:
  enabled: true

  events:
    # Mergers and Acquisitions
    - type: acquisition
      date: "2024-06-15"
      acquired_entity: "TargetCorp"
      effects:
        - entity_count_increase: 1.35
        - vendor_count_increase: 1.25
        - customer_overlap: 0.15
        - integration_period_months: 12
        - synergy_realization:
            start_month: 6
            full_realization_month: 18
            cost_reduction: 0.08

    # Divestiture
    - type: divestiture
      date: "2024-09-01"
      divested_entity: "NonCoreBusiness"
      effects:
        - revenue_reduction: 0.12
        - entity_count_reduction: 0.10
        - vendor_transition_period: 6

    # Reorganization
    - type: reorganization
      date: "2024-04-01"
      type: functional_to_regional
      effects:
        - cost_center_restructure: true
        - approval_chain_changes: true
        - reporting_line_changes: true
        - transition_period_months: 3
        - temporary_confusion_factor: 1.15

    # Leadership Change
    - type: leadership_change
      date: "2024-07-01"
      position: CFO
      effects:
        - policy_changes_probability: 0.40
        - approval_threshold_review: true
        - vendor_review_trigger: true
        - audit_focus_shift: possible

    # Layoffs
    - type: workforce_reduction
      date: "2024-11-01"
      reduction_percent: 0.10
      effects:
        - employee_count_reduction: 0.10
        - workload_redistribution: true
        - approval_delays: 1.20
        - error_rate_increase: 1.15
        - duration_months: 6
```

#### 1.2 Integration Pattern Modeling

```rust
pub struct IntegrationSimulator {
    phases: Vec<IntegrationPhase>,
    current_phase: usize,
}

pub struct IntegrationPhase {
    name: String,
    start_month: u32,
    end_month: u32,
    effects: IntegrationEffects,
}

pub struct IntegrationEffects {
    // Duplicate transactions during transition
    duplicate_probability: f64,
    // Coding errors during chart migration
    miscoding_rate: f64,
    // Legacy system parallel run
    parallel_posting: bool,
    // Vendor/customer migration errors
    master_data_errors: f64,
    // Timing differences
    posting_delay_multiplier: f64,
}
```

#### 1.3 Merger Accounting Patterns

```yaml
merger_accounting:
  enabled: true

  day_1_entries:
    - type: fair_value_adjustment
      accounts: [inventory, fixed_assets, intangibles]
      adjustment_range: [-0.20, 0.30]

    - type: goodwill_recognition
      calculation: "purchase_price - fair_value_net_assets"

    - type: liability_assumption
      includes: [accounts_payable, debt, contingencies]

  post_merger:
    # Integration costs
    integration_expenses:
      monthly_range: [100000, 500000]
      duration_months: 12-18
      categories: [consulting, severance, systems, legal]

    # Synergy realization
    synergies:
      start_month: 6
      ramp_up_months: 12
      categories:
        - type: headcount_reduction
          target: 0.05
        - type: vendor_consolidation
          target: 0.10
        - type: facility_optimization
          target: 0.03

    # Restructuring reserves
    restructuring:
      initial_reserve: 5000000
      utilization_pattern: front_loaded
      true_up_probability: 0.30
```

---

### 2. Process Evolution Modeling

#### 2.1 Business Process Changes

```yaml
process_evolution:
  enabled: true

  changes:
    # New approval workflow
    - type: approval_workflow_change
      date: "2024-03-01"
      from: sequential
      to: parallel
      effects:
        - approval_time_reduction: 0.40
        - same_day_approval_increase: 0.25
        - skip_approval_detection: improved

    # Automation introduction
    - type: process_automation
      date: "2024-05-01"
      process: invoice_matching
      effects:
        - manual_matching_reduction: 0.70
        - matching_accuracy_improvement: 0.15
        - exception_visibility_increase: true
        - posting_timing: more_consistent

    # Policy change
    - type: policy_change
      date: "2024-08-01"
      policy: expense_approval_limits
      changes:
        - manager_limit: 5000 -> 7500
        - director_limit: 25000 -> 35000
      effects:
        - approval_escalation_reduction: 0.20
        - processing_time_reduction: 0.15

    # Control enhancement
    - type: control_enhancement
      date: "2024-10-01"
      control: three_way_match
      changes:
        - tolerance: 0.05 -> 0.02
        - mandatory_for: all_po_invoices
      effects:
        - exception_rate_increase: 0.15
        - fraud_detection_improvement: 0.25
```

#### 2.2 Technology Transition Patterns

```yaml
technology_transitions:
  enabled: true

  transitions:
    # ERP migration
    - type: erp_migration
      phases:
        - name: parallel_run
          start: "2024-06-01"
          duration_months: 3
          effects:
            - duplicate_entries: true
            - reconciliation_required: true
            - posting_delays: 1.30

        - name: cutover
          date: "2024-09-01"
          effects:
            - legacy_system: read_only
            - new_system: live
            - catch_up_period: 5_days

        - name: stabilization
          start: "2024-09-01"
          duration_months: 3
          effects:
            - error_rate_multiplier: 1.25
            - support_ticket_increase: 3.0
            - workaround_transactions: 0.10

    # Module implementation
    - type: module_implementation
      module: advanced_analytics
      go_live: "2024-04-15"
      effects:
        - new_transaction_types: [analytical_adjustment]
        - automated_entries_increase: 0.20

    # Integration change
    - type: integration_upgrade
      system: bank_interface
      date: "2024-07-01"
      effects:
        - real_time_enabled: true
        - batch_processing: deprecated
        - posting_frequency: continuous
```

---

### 3. Regulatory and Compliance Drift

#### 3.1 Regulatory Changes

```yaml
regulatory_changes:
  enabled: true

  changes:
    # New accounting standard
    - type: accounting_standard_adoption
      standard: ASC_842  # Leases
      effective_date: "2024-01-01"
      effects:
        - new_account_codes: [rou_asset, lease_liability]
        - reclassification_entries: true
        - disclosure_changes: true
        - audit_focus: high

    # Tax law change
    - type: tax_law_change
      date: "2024-07-01"
      jurisdiction: federal
      change: corporate_tax_rate
      from: 0.21
      to: 0.25
      effects:
        - deferred_tax_revaluation: true
        - provision_adjustment: true
        - quarterly_estimate_revision: true

    # Compliance requirement
    - type: new_compliance_requirement
      regulation: SOX_AI_controls
      effective_date: "2024-10-01"
      requirements:
        - ai_model_documentation: required
        - automated_control_testing: required
        - data_lineage_tracking: required
      effects:
        - new_control_activities: 15
        - testing_frequency: quarterly
        - documentation_overhead: 0.10

    # Industry regulation
    - type: industry_regulation
      industry: financial_services
      regulation: enhanced_kyc
      date: "2024-06-01"
      effects:
        - customer_onboarding_time: 1.50
        - documentation_requirements: increased
        - rejection_rate_increase: 0.08
```

#### 3.2 Audit Focus Shifts

```yaml
audit_focus_evolution:
  enabled: true

  shifts:
    # Risk-based changes
    - trigger: fraud_detection
      date: "2024-03-15"
      new_focus_areas:
        - vendor_payments: high
        - manual_journal_entries: high
        - related_party_transactions: medium
      effects:
        - sampling_rate_increase: 0.30
        - documentation_requests: increased

    # Industry trend response
    - trigger: industry_trend
      date: "2024-06-01"
      trend: cybersecurity_risks
      new_focus_areas:
        - it_general_controls: high
        - access_management: high
        - change_management: medium
      effects:
        - itgc_testing_expansion: true
        - soc2_requirements: enhanced

    # Prior year findings
    - trigger: prior_year_finding
      finding: revenue_recognition_timing
      date: "2024-01-01"
      effects:
        - cutoff_testing: enhanced
        - sample_sizes: increased
        - management_inquiry: extensive
```

---

### 4. Behavioral Drift

#### 4.1 Entity Behavior Evolution

```yaml
behavioral_drift:
  enabled: true

  vendor_behavior:
    # Payment term negotiation
    payment_terms_drift:
      direction: extending
      rate_per_year: 2.5  # Days per year
      variance_increase: true
      trigger: economic_conditions

    # Quality drift
    quality_drift:
      new_vendors:
        initial_period_months: 6
        quality_improvement_rate: 0.02
      established_vendors:
        complacency_risk: 0.05
        quality_decline_rate: 0.01

    # Price drift
    pricing_behavior:
      inflation_pass_through: 0.80
      contract_renegotiation_frequency: annual
      opportunistic_increase_probability: 0.10

  customer_behavior:
    # Payment behavior evolution
    payment_drift:
      economic_downturn:
        days_extension: 5-15
        bad_debt_rate_increase: 0.02
      economic_upturn:
        days_reduction: 2-5
        early_payment_discount_uptake: 0.15

    # Order pattern drift
    order_drift:
      digital_shift:
        online_order_increase_per_year: 0.05
        average_order_value_decrease: 0.03
        order_frequency_increase: 0.10

  employee_behavior:
    # Approval pattern drift
    approval_drift:
      end_of_month_rush:
        intensity_increase_per_year: 0.05
      rubber_stamping_risk:
        increase_with_volume: true
        threshold: 50  # Approvals per day

    # Error pattern drift
    error_drift:
      new_employee:
        error_rate: 0.08
        learning_curve_months: 6
        target_error_rate: 0.02
      experienced_employee:
        fatigue_increase: 0.01_per_year
```

#### 4.2 Collective Behavior Patterns

```yaml
collective_drift:
  enabled: true

  patterns:
    # Year-end behavior
    year_end_intensity:
      drift: increasing
      rate_per_year: 0.05
      explanation: "tighter close deadlines, more scrutiny"

    # Automation adoption
    automation_adoption:
      s_curve_adoption: true
      early_adopters: 0.15
      mainstream: 0.60
      laggards: 0.25
      effects_by_phase:
        early:
          manual_reduction: 0.10
          error_types_shift: true
        mainstream:
          manual_reduction: 0.50
          new_error_types: automation_failures
        late:
          manual_reduction: 0.80
          exception_handling_focus: true

    # Remote work impact
    remote_work_patterns:
      transition_date: "2024-01-01"
      remote_percentage: 0.60
      effects:
        - posting_time_distribution: flattened
        - batch_processing_increase: true
        - approval_response_time: longer
        - documentation_quality: variable
```

---

### 5. Market-Driven Drift

#### 5.1 Economic Cycle Effects

```yaml
economic_cycles:
  enabled: true

  cycles:
    # Business cycle
    business_cycle:
      type: sinusoidal
      period_months: 48
      amplitude: 0.15
      effects:
        expansion:
          revenue_growth: positive
          hiring: active
          capital_investment: high
          credit_terms: generous
        contraction:
          revenue_growth: negative
          layoffs: possible
          capital_investment: low
          credit_terms: tight

    # Industry cycle
    industry_specific:
      technology:
        period_months: 36
        amplitude: 0.25
      manufacturing:
        period_months: 60
        amplitude: 0.20
      retail:
        period_months: 12  # Annual
        amplitude: 0.35

  # Recession simulation
  recession:
    enabled: false  # Trigger explicitly
    onset: gradual  # or sudden
    duration_months: 12-24
    severity: moderate  # mild, moderate, severe
    effects:
      revenue_decline: 0.15-0.30
      ar_aging_increase: 15_days
      bad_debt_increase: 0.03
      vendor_consolidation: 0.10
      workforce_reduction: 0.08
      capex_freeze: true
```

#### 5.2 Commodity and Input Cost Drift

```yaml
input_cost_drift:
  enabled: true

  commodities:
    - name: steel
      base_price: 800  # per ton
      volatility: 0.20
      correlation_with_economy: 0.60
      pass_through_to_cogs: 0.15

    - name: energy
      base_price: 75   # per barrel equivalent
      volatility: 0.35
      seasonal_pattern: true
      pass_through_to_overhead: 0.08

    - name: labor
      base_cost: 35    # per hour
      annual_increase: 0.03
      regional_variation: true
      pass_through_to_all: true

  price_shock_events:
    - type: supply_disruption
      probability_per_year: 0.10
      duration_months: 3-9
      price_increase: 0.30-1.00
      affected_commodities: [specific]

    - type: demand_surge
      probability_per_year: 0.15
      duration_months: 2-6
      price_increase: 0.15-0.40
      affected_commodities: [broad]
```

---

### 6. Concept Drift Detection Signals

#### 6.1 Drift Indicators in Generated Data

```yaml
drift_signals:
  enabled: true

  embedded_signals:
    # Statistical shift markers
    statistical:
      - type: mean_shift
        field: transaction_amount
        visibility: detectable_by_cusum
        magnitude: configurable

      - type: variance_change
        field: processing_time
        visibility: detectable_by_levene
        direction: both

      - type: distribution_change
        field: payment_terms
        visibility: detectable_by_ks_test
        gradual: true

    # Categorical drift markers
    categorical:
      - type: category_proportion_shift
        field: transaction_type
        new_category_emergence: true
        old_category_decline: true

      - type: label_drift
        field: account_code
        new_codes: added_over_time
        deprecated_codes: declining_usage

    # Temporal drift markers
    temporal:
      - type: seasonality_change
        field: transaction_count
        pattern_evolution: true
        detectability: acf_analysis

      - type: trend_change
        field: revenue
        change_points: marked
        detectability: pelt_algorithm

  # Ground truth labels for drift
  drift_labels:
    enabled: true
    output_file: drift_events.csv
    columns:
      - event_type
      - start_date
      - end_date
      - affected_fields
      - magnitude
      - detection_difficulty
```

#### 6.2 Drift Validation Metrics

```yaml
drift_validation:
  metrics:
    # Drift presence verification
    drift_detection:
      methods:
        - adwin   # Adaptive Windowing
        - ddm     # Drift Detection Method
        - eddm    # Early Drift Detection Method
        - ph      # Page-Hinkley Test
      threshold_calibration: true

    # Drift magnitude
    magnitude_metrics:
      - hellinger_distance
      - kl_divergence
      - wasserstein_distance
      - psi  # Population Stability Index

    # Drift timing accuracy
    timing_metrics:
      - detection_delay_days
      - false_positive_rate
      - detection_precision
```

---

### 7. Implementation Framework

#### 7.1 Drift Controller Enhancement

```rust
pub struct EnhancedDriftController {
    // Existing drift
    parameter_drift: ParameterDrift,

    // New: Organizational events
    event_timeline: EventTimeline,

    // New: Process changes
    process_evolution: ProcessEvolution,

    // New: Regulatory changes
    regulatory_calendar: RegulatoryCalendar,

    // New: Behavioral models
    behavioral_drift: BehavioralDriftModel,

    // New: Market factors
    market_model: MarketModel,

    // Drift detection ground truth
    drift_labels: DriftLabelRecorder,
}

impl EnhancedDriftController {
    /// Get all active effects for a given date
    pub fn get_effects(&self, date: NaiveDate) -> DriftEffects {
        let mut effects = DriftEffects::default();

        // Apply organizational events
        effects.merge(self.event_timeline.effects_at(date));

        // Apply process evolution
        effects.merge(self.process_evolution.effects_at(date));

        // Apply regulatory changes
        effects.merge(self.regulatory_calendar.effects_at(date));

        // Apply behavioral drift
        effects.merge(self.behavioral_drift.effects_at(date));

        // Apply market conditions
        effects.merge(self.market_model.effects_at(date));

        // Record for ground truth
        self.drift_labels.record(date, &effects);

        effects
    }
}
```

#### 7.2 Configuration Integration

```yaml
# Master drift configuration
drift:
  enabled: true

  # Parameter drift (existing)
  parameters:
    amount_mean_drift: 0.02
    amount_variance_drift: 0.01

  # Organizational events (new)
  organizational:
    events_file: "organizational_events.yaml"
    random_events:
      reorganization_probability: 0.10
      leadership_change_probability: 0.15

  # Process evolution (new)
  process:
    automation_curve: s_curve
    policy_review_frequency: quarterly

  # Regulatory changes (new)
  regulatory:
    calendar_file: "regulatory_calendar.yaml"
    jurisdictions: [us, eu]

  # Behavioral drift (new)
  behavioral:
    vendor_learning: true
    customer_churn: true
    employee_turnover: 0.15

  # Market factors (new)
  market:
    economic_cycle: true
    commodity_volatility: true
    inflation_rate: 0.03

  # Drift labeling (new)
  labels:
    enabled: true
    output_format: csv
    include_magnitude: true
```

---

### 8. Implementation Priority

| Enhancement | Complexity | Impact | Priority |
|-------------|------------|--------|----------|
| Organizational events | Medium | High | P1 |
| Process evolution | Medium | High | P1 |
| Regulatory changes | Low | Medium | P2 |
| Behavioral drift | High | High | P1 |
| Market-driven drift | Medium | Medium | P2 |
| Drift detection signals | Low | High | P1 |
| Technology transitions | High | Medium | P3 |
| Collective behavior | Medium | Medium | P2 |

---

### 9. Use Cases

1. **ML Model Robustness Testing**: Train models on stable data, test on drifted data
2. **Drift Detection Benchmarking**: Evaluate drift detection algorithms on known drift
3. **Change Management Simulation**: Test system responses to organizational changes
4. **Regulatory Impact Analysis**: Model effects of compliance requirement changes
5. **Economic Scenario Planning**: Generate data under different economic conditions

---

*See also*: [06-anomaly-patterns.md](06-anomaly-patterns.md) for anomaly injection patterns
