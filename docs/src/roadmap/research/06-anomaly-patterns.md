# Research: Anomaly Pattern Enhancements

## Current State Analysis

### Existing Anomaly Categories

| Category | Types | Implementation |
|----------|-------|----------------|
| **Fraud** | Fictitious, Revenue Manipulation, Split, Round-trip, Ghost Employee, Duplicate Payment | Good |
| **Error** | Duplicate Entry, Reversed Amount, Wrong Period, Wrong Account, Missing Reference | Good |
| **Process** | Late Posting, Skipped Approval, Threshold Manipulation | Medium |
| **Statistical** | Unusual Amount, Trend Break, Benford Violation | Medium |
| **Relational** | Circular Transaction, Dormant Account | Basic |

### Current Strengths

1. **Labeled output**: `anomaly_labels.csv` with ground truth
2. **Configurable injection rate**: Per-anomaly-type rates
3. **Quality issue labeling**: Separate from fraud labels
4. **Multiple anomaly types**: 20+ distinct patterns
5. **COSO control mapping**: Anomalies linked to control failures

### Current Gaps

1. **Binary labeling only**: No severity or confidence scores
2. **Independent injection**: Anomalies don't correlate with each other
3. **No multi-stage anomalies**: Complex schemes not modeled
4. **Static patterns**: Same anomaly signature throughout
5. **No near-miss generation**: Only clear anomalies or clean data
6. **Limited context awareness**: Anomalies don't adapt to entity behavior
7. **No detection difficulty labeling**: All anomalies treated equally

---

## Improvement Recommendations

### 1. Multi-Dimensional Anomaly Labeling

#### 1.1 Enhanced Label Schema

```yaml
anomaly_labeling:
  schema:
    # Primary classification
    anomaly_id: uuid
    transaction_ids: [uuid]
    anomaly_type: string
    anomaly_category: [fraud, error, process, statistical, relational]

    # Severity scoring
    severity:
      level: [low, medium, high, critical]
      score: 0.0-1.0
      financial_impact: decimal
      materiality_threshold: exceeded | below

    # Detection characteristics
    detection:
      difficulty: [trivial, easy, moderate, hard, expert]
      recommended_methods: [rule_based, statistical, ml, graph, hybrid]
      expected_false_positive_rate: 0.0-1.0
      key_indicators: [string]

    # Confidence and certainty
    confidence:
      ground_truth_certainty: [definite, probable, possible]
      label_source: [injected, inferred, manual]

    # Temporal characteristics
    temporal:
      first_occurrence: date
      last_occurrence: date
      frequency: [one_time, recurring, continuous]
      detection_window: days

    # Relationship context
    context:
      related_anomalies: [uuid]
      affected_entities: [entity_id]
      control_failures: [control_id]
      root_cause: string
```

#### 1.2 Materiality-Based Severity

```yaml
severity_calculation:
  materiality_thresholds:
    trivial: 0.001        # 0.1% of relevant base
    immaterial: 0.01      # 1%
    material: 0.05        # 5%
    highly_material: 0.10 # 10%

  bases_by_type:
    revenue: total_revenue
    expense: total_expenses
    asset: total_assets
    liability: total_liabilities

  severity_factors:
    financial_impact:
      weight: 0.40
      calculation: amount / materiality_threshold

    detection_difficulty:
      weight: 0.25
      mapping:
        trivial: 0.1
        easy: 0.3
        moderate: 0.5
        hard: 0.7
        expert: 0.9

    persistence:
      weight: 0.20
      calculation: duration_days / 365

    entity_involvement:
      weight: 0.15
      calculation: log(affected_entity_count)
```

---

### 2. Correlated Anomaly Injection

#### 2.1 Anomaly Co-occurrence Patterns

```yaml
anomaly_correlations:
  enabled: true

  patterns:
    # Fraud often accompanied by concealment
    fraud_concealment:
      primary: fictitious_vendor
      correlated:
        - type: document_manipulation
          probability: 0.80
          lag_days: 0-30
        - type: approval_bypass
          probability: 0.60
          lag_days: 0
        - type: audit_trail_gaps
          probability: 0.40
          lag_days: 0-90

    # Error cascades
    error_cascade:
      primary: wrong_account_coding
      correlated:
        - type: reconciliation_difference
          probability: 0.90
          lag_days: 30-60
        - type: balance_discrepancy
          probability: 0.70
          lag_days: 30
        - type: correcting_entry
          probability: 0.85
          lag_days: 1-45

    # Process failures cluster
    process_breakdown:
      primary: skipped_approval
      correlated:
        - type: threshold_splitting
          probability: 0.50
          lag_days: -30 to 30
        - type: late_posting
          probability: 0.40
          lag_days: 0-15
        - type: documentation_missing
          probability: 0.60
          lag_days: 0
```

#### 2.2 Temporal Clustering

```yaml
temporal_clustering:
  enabled: true

  clusters:
    # Period-end error spikes
    period_end_errors:
      window: last_5_business_days
      error_rate_multiplier: 2.5
      types: [wrong_period, duplicate_entry, late_posting]

    # Post-holiday cleanup
    post_holiday:
      window: first_3_business_days_after_holiday
      types: [duplicate_entry, missing_reference]
      multiplier: 1.8

    # Quarter-end pressure
    quarter_end:
      window: last_week_of_quarter
      fraud_types: [revenue_manipulation, expense_deferral]
      multiplier: 1.5

    # Year-end audit prep
    year_end_audit:
      window: december
      correction_types: [reclassification, prior_period_adjustment]
      multiplier: 3.0
```

---

### 3. Multi-Stage Anomaly Patterns

#### 3.1 Complex Scheme Modeling

```yaml
multi_stage_anomalies:
  enabled: true

  schemes:
    # Gradual embezzlement
    gradual_embezzlement:
      stages:
        - stage: 1
          name: testing
          duration_months: 2
          transactions: 3-5
          amount_range: [100, 500]
          detection_difficulty: hard

        - stage: 2
          name: escalation
          duration_months: 6
          transactions: 10-20
          amount_range: [500, 2000]
          detection_difficulty: moderate

        - stage: 3
          name: acceleration
          duration_months: 3
          transactions: 20-50
          amount_range: [2000, 10000]
          detection_difficulty: easy

        - stage: 4
          name: desperation
          duration_months: 1
          transactions: 5-10
          amount_range: [10000, 50000]
          detection_difficulty: trivial

      total_scheme_probability: 0.02

    # Revenue manipulation over time
    revenue_scheme:
      stages:
        - stage: 1
          name: acceleration
          quarter: Q4
          action: early_revenue_recognition
          amount_percent: 0.02

        - stage: 2
          name: deferral
          quarter: Q1_next
          action: expense_deferral
          amount_percent: 0.03

        - stage: 3
          name: reserve_manipulation
          quarter: Q2
          action: reserve_release
          amount_percent: 0.02

        - stage: 4
          name: channel_stuffing
          quarter: Q4
          action: forced_sales
          amount_percent: 0.05

      cycle_probability: 0.01

    # Vendor kickback scheme
    kickback_scheme:
      stages:
        - stage: 1
          name: vendor_setup
          actions: [create_vendor, build_relationship]
          duration_months: 3

        - stage: 2
          name: price_inflation
          actions: [inflated_invoices]
          inflation_percent: 0.10-0.25
          duration_months: 12

        - stage: 3
          name: kickback_payments
          actions: [off_book_payments]
          kickback_percent: 0.50
          frequency: quarterly

        - stage: 4
          name: concealment
          actions: [document_destruction, false_approvals]
          ongoing: true
```

#### 3.2 Scheme Evolution

```rust
pub struct MultiStageAnomaly {
    scheme_id: Uuid,
    scheme_type: SchemeType,
    current_stage: u32,
    start_date: NaiveDate,
    perpetrators: Vec<EntityId>,
    transactions: Vec<TransactionId>,
    total_impact: Decimal,
    detection_status: DetectionStatus,
}

impl MultiStageAnomaly {
    /// Advance scheme to next stage
    pub fn advance(&mut self, date: NaiveDate) -> Vec<AnomalyAction> {
        // Check if conditions met for stage advancement
        // Return actions for current stage
    }

    /// Check if scheme should be detected based on accumulated evidence
    pub fn detection_probability(&self) -> f64 {
        // Increases with:
        // - Number of transactions
        // - Total amount
        // - Duration
        // - Carelessness factor
    }
}
```

---

### 4. Near-Miss and Edge Case Generation

#### 4.1 Near-Anomaly Patterns

```yaml
near_miss_generation:
  enabled: true
  proportion_of_anomalies: 0.30  # 30% of "anomalies" are near-misses

  patterns:
    # Almost duplicate (timing difference)
    near_duplicate:
      description: "Similar transaction, different timing"
      difference:
        amount: exact_match
        date: 1-3_days_apart
        vendor: same
      label: not_anomaly
      detection_challenge: high

    # Threshold proximity
    threshold_proximity:
      description: "Transaction just below approval threshold"
      distance_from_threshold: [0.90, 0.99]
      label: not_anomaly
      suspicion_score: high

    # Unusual but explainable
    unusual_legitimate:
      description: "Unusual pattern with valid business reason"
      types:
        - year_end_bonus
        - contract_prepayment
        - settlement_payment
        - insurance_claim
      label: not_anomaly
      false_positive_trigger: high

    # Corrected error
    corrected_error:
      description: "Error that was caught and fixed"
      original_error: any
      correction_lag_days: 1-5
      net_impact: zero
      label: error_corrected
      visibility: both_entries_visible
```

#### 4.2 Boundary Condition Testing

```yaml
boundary_conditions:
  enabled: true

  conditions:
    # Exact threshold matches
    exact_thresholds:
      types: [approval_limit, materiality, tolerance]
      probability: 0.01
      label: boundary_case

    # Round number preference (non-fraudulent)
    legitimate_round_numbers:
      amounts: [1000, 5000, 10000, 25000]
      probability: 0.05
      label: not_anomaly
      context: budget_allocations

    # Last-minute but legitimate
    period_boundary:
      timing: last_hour_before_close
      legitimate_probability: 0.80
      label: timing_anomaly_only

    # Zero and negative amounts
    edge_amounts:
      zero_amount_probability: 0.001
      negative_amount_probability: 0.002
      labels: data_quality_issue
```

---

### 5. Context-Aware Anomaly Injection

#### 5.1 Entity-Specific Patterns

```yaml
entity_aware_anomalies:
  enabled: true

  vendor_specific:
    # New vendors have higher error rates
    new_vendor_errors:
      definition: vendor_age < 90_days
      error_rate_multiplier: 2.5
      common_errors: [wrong_account, missing_po]

    # Large vendors have more complex issues
    strategic_vendor_issues:
      definition: vendor_spend > percentile_90
      anomaly_types: [contract_deviation, price_variance]
      rate_multiplier: 1.5

    # International vendors
    international_vendor_issues:
      definition: vendor_country != company_country
      anomaly_types: [fx_errors, withholding_tax_errors]
      rate_multiplier: 2.0

  employee_specific:
    # New employee learning curve
    new_employee_errors:
      definition: employee_tenure < 180_days
      error_rate: 0.05
      error_types: [coding_error, approval_violation]
      decay: exponential

    # High-volume processors
    volume_fatigue:
      definition: daily_transactions > 50
      error_rate_increase: 0.02
      peak_time: end_of_day

    # Vacation coverage
    coverage_errors:
      trigger: primary_approver_absent
      error_rate_multiplier: 1.8
      types: [delayed_approval, wrong_approver]

  account_specific:
    # High-risk accounts
    high_risk_accounts:
      accounts: [cash, revenue, inventory]
      monitoring_level: enhanced
      anomaly_injection_rate: 1.5x

    # Infrequently used accounts
    dormant_account_activity:
      definition: no_activity_90_days
      any_activity_suspicious: true
      label: statistical_anomaly
```

#### 5.2 Behavioral Baseline Deviation

```yaml
behavioral_deviation:
  enabled: true

  baselines:
    # Establish per-entity behavioral baseline
    baseline_period: 90_days
    metrics:
      - average_transaction_amount
      - transaction_frequency
      - typical_posting_time
      - common_counterparties
      - usual_account_codes

  deviations:
    # Amount deviation
    amount_anomaly:
      threshold: 3_standard_deviations
      label: statistical_anomaly
      severity: based_on_deviation

    # Frequency deviation
    frequency_anomaly:
      threshold: 2_standard_deviations
      types: [sudden_increase, sudden_decrease, irregular_pattern]

    # Counterparty deviation
    new_counterparty:
      first_time_transaction: true
      risk_score: elevated
      label: relationship_anomaly

    # Timing deviation
    timing_anomaly:
      threshold: outside_usual_hours
      consideration: legitimate_reasons_exist
      label: timing_anomaly
```

---

### 6. Detection Difficulty Classification

#### 6.1 Difficulty Taxonomy

```yaml
detection_difficulty:
  levels:
    trivial:
      description: "Obvious on cursory review"
      examples:
        - duplicate_same_day
        - obviously_wrong_amount
        - missing_required_field
      expected_detection_rate: 0.99
      detection_methods: [basic_rules]

    easy:
      description: "Detectable with standard controls"
      examples:
        - threshold_violations
        - approval_gaps
        - segregation_of_duties
      expected_detection_rate: 0.90
      detection_methods: [automated_rules, basic_analytics]

    moderate:
      description: "Requires analytical procedures"
      examples:
        - trend_deviations
        - ratio_anomalies
        - benford_violations
      expected_detection_rate: 0.70
      detection_methods: [statistical_analysis, ratio_analysis]

    hard:
      description: "Requires advanced techniques or domain expertise"
      examples:
        - complex_fraud_schemes
        - collusion_patterns
        - sophisticated_manipulation
      expected_detection_rate: 0.40
      detection_methods: [ml_models, graph_analysis, forensic_audit]

    expert:
      description: "Only detectable by specialized investigation"
      examples:
        - long_running_schemes
        - management_override
        - deep_concealment
      expected_detection_rate: 0.15
      detection_methods: [tip_or_complaint, forensic_investigation, external_audit]
```

#### 6.2 Difficulty Factors

```rust
pub struct DifficultyCalculator {
    factors: Vec<DifficultyFactor>,
}

pub enum DifficultyFactor {
    // Concealment techniques
    Concealment {
        document_manipulation: bool,
        approval_circumvention: bool,
        timing_exploitation: bool,
        splitting: bool,
    },

    // Blending with normal activity
    Blending {
        amount_within_normal_range: bool,
        timing_within_normal_hours: bool,
        counterparty_is_established: bool,
        account_coding_correct: bool,
    },

    // Collusion
    Collusion {
        number_of_participants: u32,
        includes_management: bool,
        external_parties: bool,
    },

    // Duration and frequency
    Temporal {
        duration_months: u32,
        transaction_frequency: Frequency,
        gradual_escalation: bool,
    },

    // Amount characteristics
    Amount {
        total_amount: Decimal,
        individual_amounts_small: bool,
        round_numbers_avoided: bool,
    },
}
```

---

### 7. Anomaly Generation Strategies

#### 7.1 Strategy Configuration

```yaml
anomaly_strategies:
  # Random injection (current approach)
  random:
    enabled: true
    weight: 0.40
    parameters:
      base_rate: 0.02
      per_type_rates: {...}

  # Scenario-based injection
  scenario_based:
    enabled: true
    weight: 0.30
    scenarios:
      - name: "new_employee_fraud"
        trigger: employee_tenure < 365
        probability: 0.005
        scheme: embezzlement

      - name: "vendor_collusion"
        trigger: vendor_concentration > 0.15
        probability: 0.01
        scheme: kickback

      - name: "year_end_pressure"
        trigger: month == 12
        probability: 0.03
        types: [revenue_manipulation, reserve_adjustment]

  # Adversarial injection
  adversarial:
    enabled: true
    weight: 0.20
    strategy: evade_known_detectors
    detectors_to_evade:
      - benford_analysis
      - duplicate_detection
      - threshold_monitoring
    techniques:
      - amount_variation
      - timing_spreading
      - entity_rotation

  # Benchmark-based injection
  benchmark:
    enabled: true
    weight: 0.10
    source: acfe_report_to_the_nations
    calibration:
      median_loss: 117000
      duration_months: 12
      detection_method_distribution: {...}
```

#### 7.2 Adaptive Anomaly Injection

```rust
pub struct AdaptiveAnomalyInjector {
    // Tracks what's been injected
    injection_history: Vec<InjectedAnomaly>,

    // Ensures variety
    type_distribution: TypeDistribution,

    // Ensures difficulty spread
    difficulty_distribution: DifficultyDistribution,

    // Ensures temporal spread
    temporal_distribution: TemporalDistribution,
}

impl AdaptiveAnomalyInjector {
    /// Inject anomaly with awareness of what's already been injected
    pub fn inject(&mut self, context: &GenerationContext) -> Option<Anomaly> {
        // Check if injection appropriate at this point
        if !self.should_inject(context) {
            return None;
        }

        // Select type based on current distribution gaps
        let anomaly_type = self.select_type_for_balance();

        // Select difficulty based on current distribution gaps
        let difficulty = self.select_difficulty_for_balance();

        // Generate anomaly
        let anomaly = self.generate_anomaly(anomaly_type, difficulty, context);

        // Record injection
        self.record_injection(&anomaly);

        Some(anomaly)
    }
}
```

---

### 8. Output Enhancements

#### 8.1 Enhanced Label File

```yaml
output:
  anomaly_labels:
    format: parquet  # or csv
    columns:
      # Identifiers
      - anomaly_id
      - transaction_ids  # Array
      - scheme_id        # For multi-stage

      # Classification
      - anomaly_type
      - category
      - subcategory

      # Severity
      - severity_level
      - severity_score
      - financial_impact
      - is_material

      # Detection
      - difficulty_level
      - difficulty_score
      - recommended_detection_methods  # Array
      - key_indicators  # Array

      # Temporal
      - first_date
      - last_date
      - duration_days
      - stage  # For multi-stage

      # Context
      - affected_entities  # Array
      - control_failures  # Array
      - related_anomalies  # Array

      # Metadata
      - injection_strategy
      - generation_seed
      - ground_truth_certainty

  # Separate scheme file for multi-stage
  schemes:
    format: json
    structure:
      scheme_id: uuid
      scheme_type: string
      stages: [...]
      transactions_by_stage: {...}
      total_impact: decimal
      perpetrators: [entity_ids]
```

#### 8.2 Detection Benchmark Output

```yaml
detection_benchmarks:
  enabled: true

  outputs:
    # Performance expectations by method
    expected_performance:
      format: json
      content:
        by_method:
          rule_based:
            expected_recall: 0.40
            expected_precision: 0.85
          statistical:
            expected_recall: 0.55
            expected_precision: 0.70
          ml_supervised:
            expected_recall: 0.75
            expected_precision: 0.80
          graph_based:
            expected_recall: 0.65
            expected_precision: 0.75

    # Difficulty distribution
    difficulty_summary:
      format: csv
      columns: [difficulty_level, count, percentage, avg_amount]

    # Detection challenge set
    challenge_cases:
      format: json
      description: "Curated set of hardest-to-detect anomalies"
      count: 100
      selection_criteria: difficulty_score > 0.7
```

---

### 9. Implementation Priority

| Enhancement | Complexity | Impact | Priority |
|-------------|------------|--------|----------|
| Multi-dimensional labeling | Low | High | P1 |
| Correlated anomaly injection | Medium | High | P1 |
| Multi-stage schemes | High | High | P1 |
| Near-miss generation | Medium | High | P1 |
| Context-aware injection | Medium | High | P2 |
| Difficulty classification | Low | High | P1 |
| Adaptive injection | Medium | Medium | P2 |
| Detection benchmarks | Low | Medium | P2 |

---

*See also*: [07-fraud-patterns.md](07-fraud-patterns.md) for fraud-specific patterns
