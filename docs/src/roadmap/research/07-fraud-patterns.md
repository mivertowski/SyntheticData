# Research: Fraud Pattern Improvements

## Current State Analysis

### Existing Fraud Typologies

| Category | Types Implemented | Realism |
|----------|------------------|---------|
| **Asset Misappropriation** | Ghost Employee, Duplicate Payment, Fictitious Vendor | Medium |
| **Financial Statement Fraud** | Revenue Manipulation, Round-tripping | Basic |
| **Corruption** | (Limited) | Weak |
| **Banking/AML** | Structuring, Layering, Mule, Funnel, Spoofing | Good |

### Current Strengths

1. **Banking module**: Sophisticated AML typologies with transaction networks
2. **Fraud labeling**: Ground truth labels for ML training
3. **Control mapping**: Fraud linked to control failures
4. **Amount patterns**: Benford violations for fraudulent amounts

### Current Gaps

1. **No collusion modeling**: Fraud actors operate independently
2. **Limited concealment**: Fraud isn't actively hidden
3. **No behavioral adaptation**: Fraudsters don't learn
4. **Static schemes**: Same patterns throughout
5. **Missing corruption types**: Bribery, kickbacks underdeveloped
6. **No management override**: All fraud at operational level
7. **Limited financial statement fraud**: Complex schemes not modeled

---

## Improvement Recommendations

### 1. Comprehensive Fraud Taxonomy

#### 1.1 ACFE-Aligned Framework

Based on the Association of Certified Fraud Examiners Occupational Fraud and Abuse Classification:

```yaml
fraud_taxonomy:
  # Asset Misappropriation (86% of cases, $100k median loss)
  asset_misappropriation:
    cash:
      theft_of_cash_on_hand:
        - larceny
        - skimming

      theft_of_cash_receipts:
        - sales_skimming
        - receivables_skimming
        - refund_schemes

      fraudulent_disbursements:
        - billing_schemes:
            - shell_company
            - non_accomplice_vendor
            - personal_purchases
        - payroll_schemes:
            - ghost_employee
            - falsified_wages
            - commission_schemes
        - expense_reimbursement:
            - mischaracterized_expenses
            - overstated_expenses
            - fictitious_expenses
        - check_tampering:
            - forged_maker
            - forged_endorsement
            - altered_payee
            - authorized_maker
        - register_disbursements:
            - false_voids
            - false_refunds

    inventory_and_assets:
      - misuse
      - larceny

  # Corruption (33% of cases, $150k median loss)
  corruption:
    conflicts_of_interest:
      - purchasing_schemes
      - sales_schemes

    bribery:
      - invoice_kickbacks
      - bid_rigging

    illegal_gratuities: true

    economic_extortion: true

  # Financial Statement Fraud (10% of cases, $954k median loss)
  financial_statement_fraud:
    overstatement:
      - timing_differences:
          - premature_revenue
          - delayed_expenses
      - fictitious_revenues
      - concealed_liabilities
      - improper_asset_valuations
      - improper_disclosures

    understatement:
      - understated_revenues
      - overstated_expenses
      - overstated_liabilities
```

#### 1.2 Industry-Specific Fraud Patterns

```yaml
industry_fraud_patterns:
  manufacturing:
    common_schemes:
      - type: inventory_theft
        frequency: high
        methods: [larceny, false_shipments, scrap_manipulation]
      - type: vendor_kickbacks
        frequency: medium
        methods: [inflated_pricing, phantom_materials]
      - type: quality_fraud
        frequency: low
        methods: [false_certifications, spec_violations]

  retail:
    common_schemes:
      - type: register_fraud
        frequency: high
        methods: [skimming, false_voids, sweethearting]
      - type: return_fraud
        frequency: high
        methods: [fictitious_returns, receipt_fraud]
      - type: inventory_shrinkage
        frequency: very_high
        methods: [employee_theft, vendor_collusion]

  financial_services:
    common_schemes:
      - type: loan_fraud
        frequency: medium
        methods: [false_documentation, appraisal_fraud]
      - type: insider_trading
        frequency: low
        methods: [front_running, tip_schemes]
      - type: account_takeover
        frequency: medium
        methods: [identity_theft, credential_theft]

  healthcare:
    common_schemes:
      - type: billing_fraud
        frequency: high
        methods: [upcoding, unbundling, phantom_billing]
      - type: kickbacks
        frequency: medium
        methods: [referral_fees, drug_company_payments]
      - type: identity_fraud
        frequency: medium
        methods: [patient_identity_theft, provider_impersonation]

  professional_services:
    common_schemes:
      - type: billing_fraud
        frequency: high
        methods: [inflated_hours, phantom_work]
      - type: expense_fraud
        frequency: medium
        methods: [personal_expenses, inflated_claims]
      - type: client_fund_misappropriation
        frequency: low
        methods: [trust_account_theft, advance_fee_theft]
```

---

### 2. Collusion and Conspiracy Modeling

#### 2.1 Collusion Network Generation

```yaml
collusion_networks:
  enabled: true

  network_types:
    # Internal collusion
    internal:
      - type: employee_pair
        roles: [approver, processor]
        scheme: approval_bypass
        probability: 0.005

      - type: department_ring
        size: 3-5
        roles: [initiator, approver, concealer]
        scheme: expense_fraud
        probability: 0.002

      - type: management_subordinate
        roles: [manager, subordinate]
        scheme: ghost_employee
        probability: 0.003

    # Internal-external collusion
    internal_external:
      - type: employee_vendor
        roles: [purchasing_agent, vendor_contact]
        scheme: kickback
        probability: 0.008

      - type: employee_customer
        roles: [sales_rep, customer]
        scheme: false_credits
        probability: 0.004

      - type: employee_contractor
        roles: [project_manager, contractor]
        scheme: overbilling
        probability: 0.006

    # External rings
    external:
      - type: vendor_ring
        size: 2-4
        scheme: bid_rigging
        probability: 0.002

      - type: customer_ring
        size: 2-3
        scheme: return_fraud
        probability: 0.003

  network_characteristics:
    trust_building:
      initial_period_months: 3
      test_transactions: 2-5
      test_amounts: small

    communication_patterns:
      frequency: coded
      channels: [personal_email, phone, in_person]
      visibility: low

    profit_sharing:
      methods: [equal_split, role_based, initiator_premium]
      payment_channels: [cash, personal_accounts, crypto]
```

#### 2.2 Collusion Behavior Modeling

```rust
pub struct CollusionRing {
    ring_id: Uuid,
    members: Vec<Conspirator>,
    scheme_type: SchemeType,
    formation_date: NaiveDate,
    status: RingStatus,
    total_stolen: Decimal,
    detection_risk: f64,
}

pub struct Conspirator {
    entity_id: EntityId,
    role: ConspiratorRole,
    join_date: NaiveDate,
    loyalty: f64,           // Probability of not defecting
    risk_tolerance: f64,    // Willingness to escalate
    share: f64,             // Percentage of proceeds
}

pub enum ConspiratorRole {
    Initiator,      // Conceives scheme
    Executor,       // Performs transactions
    Approver,       // Provides approvals
    Concealer,      // Hides evidence
    Lookout,        // Monitors for detection
    Beneficiary,    // External recipient
}

impl CollusionRing {
    /// Simulate ring behavior for a period
    pub fn simulate_period(&mut self, period: &Period) -> Vec<FraudAction> {
        // Check for defection
        if self.check_defection() {
            return self.dissolve();
        }

        // Check for escalation
        let escalation = self.check_escalation();

        // Generate fraudulent transactions
        let actions = self.generate_actions(period, escalation);

        // Update detection risk
        self.update_detection_risk(&actions);

        actions
    }

    /// Check if any member might defect
    fn check_defection(&self) -> bool {
        // Factors: loyalty, detection_risk, personal_circumstances
    }
}
```

---

### 3. Concealment Techniques

#### 3.1 Document Manipulation

```yaml
concealment_techniques:
  document_manipulation:
    # Forged documents
    forgery:
      types:
        - invoices
        - receipts
        - approvals
        - contracts
      quality_levels:
        crude: 0.20      # Easy to detect
        moderate: 0.50   # Requires scrutiny
        sophisticated: 0.25  # Difficult to detect
        professional: 0.05   # Expert required

    # Altered documents
    alteration:
      techniques:
        - amount_change
        - date_change
        - payee_change
        - description_change
      detection_indicators:
        - different_handwriting
        - correction_fluid
        - digital_artifacts

    # Destroyed documents
    destruction:
      methods:
        - physical_destruction
        - digital_deletion
        - "lost_in_transition"
      recovery_probability: 0.30

  audit_trail_manipulation:
    techniques:
      - backdating_entries
      - manipulating_timestamps
      - deleting_log_entries
      - creating_false_trails

    sophistication_levels:
      basic: "obvious_gaps"
      intermediate: "plausible_explanations"
      advanced: "complete_alternative_narrative"

  segregation_circumvention:
    methods:
      - shared_credentials
      - delegated_authority_abuse
      - emergency_access_exploitation
      - system_override_use
```

#### 3.2 Transaction Structuring

```yaml
transaction_structuring:
  # Below threshold structuring
  threshold_avoidance:
    thresholds:
      - type: approval_limit
        values: [1000, 5000, 10000, 25000]
        technique: split_below
        margin: 0.05-0.15

      - type: reporting_threshold
        values: [10000]  # CTR threshold
        technique: structure_below
        margin: 0.10-0.20

      - type: audit_sample_threshold
        values: [materiality * 0.5]
        technique: avoid_population
        margin: variable

  # Timing manipulation
  timing_techniques:
    - type: spread_over_periods
      purpose: avoid_trending
      pattern: randomized

    - type: burst_before_vacation
      purpose: delayed_discovery
      window: 1_week

    - type: holiday_timing
      purpose: reduced_oversight
      targets: [year_end, summer]

  # Entity rotation
  entity_rotation:
    - type: vendor_rotation
      purpose: avoid_concentration_alerts
      rotation_frequency: quarterly

    - type: account_rotation
      purpose: avoid_pattern_detection
      accounts: [expense_categories]

    - type: department_rotation
      purpose: spread_impact
      pattern: round_robin
```

---

### 4. Management Override

#### 4.1 Override Patterns

```yaml
management_override:
  enabled: true

  scenarios:
    # Revenue manipulation
    revenue_override:
      perpetrator_level: senior_management
      techniques:
        - journal_entry_override
        - revenue_recognition_acceleration
        - reserve_manipulation
        - side_agreement_concealment
      concealment:
        - false_documentation
        - intimidation_of_subordinates
        - auditor_deception

    # Expense manipulation
    expense_override:
      perpetrator_level: department_head+
      techniques:
        - capitalization_abuse
        - expense_deferral
        - cost_allocation_manipulation
      pressure_sources:
        - budget_targets
        - bonus_thresholds
        - analyst_expectations

    # Asset manipulation
    asset_override:
      perpetrator_level: senior_management
      techniques:
        - impairment_avoidance
        - valuation_manipulation
        - classification_abuse
      motivations:
        - covenant_compliance
        - credit_rating_maintenance
        - acquisition_valuation

  override_characteristics:
    # Authority abuse
    authority_patterns:
      - override_segregation_of_duties
      - suppress_exception_reports
      - modify_control_parameters
      - grant_inappropriate_access

    # Pressure and rationalization
    fraud_triangle:
      pressure:
        - financial_targets
        - personal_financial_issues
        - market_expectations
      opportunity:
        - weak_board_oversight
        - auditor_reliance_on_management
        - complex_transactions
      rationalization:
        - "temporary_adjustment"
        - "everyone_does_it"
        - "for_the_good_of_company"
```

#### 4.2 Tone at the Top Effects

```yaml
tone_effects:
  enabled: true

  # Positive tone (ethical leadership)
  ethical_leadership:
    effects:
      - fraud_rate_reduction: 0.50
      - whistleblower_increase: 2.0
      - control_compliance_improvement: 0.20

  # Negative tone (pressure culture)
  pressure_culture:
    effects:
      - fraud_rate_increase: 2.5
      - concealment_sophistication: increased
      - collusion_probability: 1.5x
      - management_override_probability: 3.0x

  # Mixed signals
  inconsistent_messaging:
    effects:
      - employee_confusion: true
      - selective_compliance: true
      - rationalization_easier: true
```

---

### 5. Adaptive Fraud Behavior

#### 5.1 Learning and Adaptation

```yaml
adaptive_fraud:
  enabled: true

  learning_behaviors:
    # Response to near-detection
    near_detection_response:
      behaviors:
        - temporary_pause: 0.40
        - technique_change: 0.30
        - amount_reduction: 0.20
        - scheme_abandonment: 0.10
      pause_duration_days: 30-90

    # Response to control changes
    control_adaptation:
      when: new_control_implemented
      behaviors:
        - find_workaround: 0.60
        - wait_for_relaxation: 0.25
        - abandon_scheme: 0.15
      adaptation_time_days: 30-60

    # Success reinforcement
    success_reinforcement:
      when: fraud_not_detected
      behaviors:
        - increase_frequency: 0.30
        - increase_amount: 0.40
        - recruit_accomplices: 0.15
        - maintain_current: 0.15

  sophistication_evolution:
    stages:
      novice:
        characteristics: [obvious_patterns, small_amounts, nervous_behavior]
        detection_difficulty: easy

      intermediate:
        characteristics: [some_concealment, medium_amounts, confidence]
        detection_difficulty: moderate

      experienced:
        characteristics: [sophisticated_concealment, varied_amounts, systematic]
        detection_difficulty: hard

      expert:
        characteristics: [professional_techniques, large_amounts, network]
        detection_difficulty: expert

    progression:
      trigger: months_undetected > 6
      probability: 0.30_per_trigger
```

#### 5.2 Detection Evasion

```rust
pub struct AdaptiveFraudster {
    experience_level: ExperienceLevel,
    known_controls: Vec<ControlId>,
    detection_events: Vec<DetectionEvent>,
    technique_repertoire: Vec<FraudTechnique>,
}

impl AdaptiveFraudster {
    /// Adapt technique based on environment
    pub fn adapt_technique(&mut self, context: &Context) -> FraudTechnique {
        // Avoid known controls
        let available = self.filter_by_controls(context.active_controls);

        // Avoid previously detected patterns
        let safe = self.filter_by_history(&available);

        // Select based on risk/reward
        self.select_optimal(&safe, context.current_risk_tolerance)
    }

    /// Learn from near-detection
    pub fn learn_from_event(&mut self, event: &DetectionEvent) {
        match event.outcome {
            DetectionOutcome::Detected => {
                self.avoid_technique(event.technique);
                self.reduce_risk_tolerance();
            }
            DetectionOutcome::NearMiss => {
                self.modify_technique(event.technique);
                self.record_warning_sign(event.indicator);
            }
            DetectionOutcome::Undetected => {
                self.reinforce_technique(event.technique);
                self.consider_escalation();
            }
        }
    }
}
```

---

### 6. Financial Statement Fraud Schemes

#### 6.1 Revenue Manipulation Schemes

```yaml
revenue_schemes:
  # Premature revenue recognition
  premature_recognition:
    techniques:
      - bill_and_hold:
          description: "Ship to warehouse, recognize revenue"
          indicators: [unusual_shipping, customer_complaints]
          journal_entries:
            - dr: accounts_receivable
              cr: revenue

      - channel_stuffing:
          description: "Force product on distributors"
          indicators: [quarter_end_spike, high_returns_next_period]
          side_agreements: [return_rights, extended_payment]

      - percentage_of_completion_abuse:
          description: "Overstate project completion"
          indicators: [optimistic_estimates, margin_improvements]
          documentation: [false_progress_reports]

      - round_tripping:
          description: "Simultaneous buy/sell with related party"
          indicators: [offsetting_transactions, unusual_counterparties]
          complexity: high

  # Fictitious revenue
  fictitious_revenue:
    techniques:
      - fake_invoices:
          description: "Bill nonexistent customers"
          concealment: [fake_customer_setup, false_confirmations]

      - side_agreements:
          description: "Hidden terms negate sale"
          concealment: [verbal_agreements, separate_documentation]

      - related_party_transactions:
          description: "Transactions with undisclosed affiliates"
          concealment: [complex_ownership, offshore_entities]
```

#### 6.2 Expense and Liability Manipulation

```yaml
expense_liability_schemes:
  # Expense deferral
  expense_deferral:
    techniques:
      - improper_capitalization:
          description: "Capitalize operating expenses"
          accounts: [fixed_assets, intangibles]
          indicators: [unusual_asset_growth, low_maintenance]

      - reserve_manipulation:
          description: "Cookie jar reserves"
          pattern: [build_in_good_years, release_in_bad]
          indicators: [volatile_provisions, earnings_smoothing]

      - period_cutoff_manipulation:
          description: "Push expenses to next period"
          timing: [quarter_end, year_end]
          techniques: [hold_invoices, delay_receipt]

  # Liability concealment
  liability_concealment:
    techniques:
      - off_balance_sheet:
          description: "Structure to avoid consolidation"
          vehicles: [SPEs, unconsolidated_subsidiaries]
          concealment: [complex_structures, offshore]

      - contingency_understatement:
          description: "Understate legal/warranty liabilities"
          rationalization: ["uncertain", "immaterial"]
          indicators: [subsequent_large_settlements]
```

---

### 7. Fraud Red Flags and Indicators

#### 7.1 Behavioral Red Flags

```yaml
behavioral_red_flags:
  # Employee behavior
  employee_indicators:
    - indicator: living_beyond_means
      fraud_correlation: 0.45
      detection_method: lifestyle_analysis

    - indicator: financial_difficulties
      fraud_correlation: 0.40
      detection_method: background_check

    - indicator: unusually_close_vendor_relationships
      fraud_correlation: 0.35
      detection_method: relationship_analysis

    - indicator: control_issues_attitude
      fraud_correlation: 0.30
      detection_method: 360_feedback

    - indicator: never_takes_vacation
      fraud_correlation: 0.50
      detection_method: hr_records

    - indicator: excessive_overtime
      fraud_correlation: 0.25
      detection_method: time_records

  # Transaction behavior
  transaction_indicators:
    - indicator: round_number_preference
      fraud_correlation: 0.20
      detection_method: benford_analysis

    - indicator: just_below_threshold
      fraud_correlation: 0.60
      detection_method: threshold_analysis

    - indicator: end_of_period_concentration
      fraud_correlation: 0.35
      detection_method: temporal_analysis

    - indicator: unusual_journal_entries
      fraud_correlation: 0.55
      detection_method: journal_entry_testing
```

#### 7.2 Red Flag Generation

```yaml
red_flag_injection:
  enabled: true

  # Inject red flags that correlate with but don't prove fraud
  correlations:
    # Strong correlation - usually indicates fraud
    strong:
      - flag: matched_home_address_vendor_employee
        fraud_probability: 0.85
        inject_with_fraud: 0.90
        inject_without_fraud: 0.001

      - flag: sequential_check_numbers_to_same_vendor
        fraud_probability: 0.70
        inject_with_fraud: 0.80
        inject_without_fraud: 0.01

    # Moderate correlation - worth investigating
    moderate:
      - flag: vendor_no_physical_address
        fraud_probability: 0.40
        inject_with_fraud: 0.60
        inject_without_fraud: 0.05

      - flag: approval_just_under_threshold
        fraud_probability: 0.35
        inject_with_fraud: 0.70
        inject_without_fraud: 0.10

    # Weak correlation - often legitimate
    weak:
      - flag: round_number_invoice
        fraud_probability: 0.15
        inject_with_fraud: 0.40
        inject_without_fraud: 0.20

      - flag: end_of_month_timing
        fraud_probability: 0.10
        inject_with_fraud: 0.50
        inject_without_fraud: 0.30
```

---

### 8. Fraud Investigation Scenarios

#### 8.1 Investigation-Ready Data

```yaml
investigation_scenarios:
  enabled: true

  scenarios:
    # Whistleblower scenario
    whistleblower_tip:
      allegation: "Vendor XYZ may be fictitious"
      evidence_trail:
        - vendor_setup_documents
        - approval_chain
        - payment_history
        - address_verification
        - phone_verification
      hidden_clues:
        - approver_is_also_requester
        - address_is_ups_store
        - phone_goes_to_employee

    # Audit finding follow-up
    audit_finding:
      initial_finding: "Unusual vendor payment pattern"
      investigation_path:
        - transaction_sample
        - vendor_analysis
        - employee_relationship_map
        - comparative_analysis
      discovery_stages:
        - stage_1: "Vendor has only one customer - us"
        - stage_2: "All invoices approved by same person"
        - stage_3: "Vendor address matches employee relative"

    # Hotline report
    anonymous_tip:
      report: "Manager taking kickbacks from contractor"
      evidence_available:
        - contract_documents
        - bid_history
        - payment_records
        - email_metadata
      additional_clues:
        - bids_always_awarded_to_same_contractor
        - contract_amendments_increase_cost_30%
        - manager_new_car_timing_correlates
```

#### 8.2 Evidence Chain Generation

```rust
pub struct FraudEvidenceChain {
    fraud_id: Uuid,
    evidence_items: Vec<EvidenceItem>,
    discovery_order: Vec<EvidenceId>,
    linking_relationships: Vec<EvidenceLink>,
}

pub struct EvidenceItem {
    id: EvidenceId,
    item_type: EvidenceType,
    content: EvidenceContent,
    source_system: String,
    timestamp: DateTime<Utc>,
    accessibility: Accessibility,  // How hard to find
    probative_value: f64,         // Strength as evidence
}

pub enum EvidenceType {
    Transaction,
    Document,
    Communication,
    SystemLog,
    ExternalRecord,
    WitnessStatement,
    PhysicalEvidence,
}

impl FraudEvidenceChain {
    /// Generate investigation-ready evidence trail
    pub fn generate_trail(&self) -> InvestigationTrail {
        // Order evidence by discoverability
        // Create logical links between items
        // Add red herrings (false leads that are eliminated)
        // Include corroborating evidence
    }
}
```

---

### 9. Implementation Priority

| Enhancement | Complexity | Impact | Priority |
|-------------|------------|--------|----------|
| ACFE-aligned taxonomy | Low | High | P1 |
| Collusion modeling | High | High | P1 |
| Concealment techniques | Medium | High | P1 |
| Management override | Medium | High | P1 |
| Adaptive behavior | High | Medium | P2 |
| Financial statement fraud | High | High | P1 |
| Red flag generation | Medium | High | P1 |
| Investigation scenarios | Medium | Medium | P2 |
| Industry-specific patterns | Medium | Medium | P2 |

---

### 10. Validation and Calibration

```yaml
fraud_validation:
  # Calibration against real-world statistics
  calibration:
    source: acfe_report_to_the_nations_2024
    metrics:
      median_loss: 117000
      median_duration_months: 12
      detection_methods:
        tip: 0.42
        internal_audit: 0.16
        management_review: 0.12
        external_audit: 0.04
        accident: 0.06
      perpetrator_departments:
        accounting: 0.21
        operations: 0.17
        executive: 0.12
        sales: 0.11
        customer_service: 0.08

  # Distribution validation
  distribution_checks:
    - metric: loss_distribution
      expected: lognormal
      parameters_from: acfe_data

    - metric: duration_distribution
      expected: exponential
      mean_months: 12

    - metric: detection_method_distribution
      expected: categorical
      match_acfe: true
```

---

*See also*: [08-domain-specific.md](08-domain-specific.md) for industry-specific enhancements
