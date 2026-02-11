# Fraud Patterns & ACFE Taxonomy

SyntheticData includes comprehensive fraud pattern modeling aligned with the Association of Certified Fraud Examiners (ACFE) Report to the Nations. This enables generation of realistic fraud scenarios for training machine learning models and testing audit analytics.

## ACFE Fraud Taxonomy

The ACFE occupational fraud classification divides fraud into three main categories, each with distinct characteristics:

### Asset Misappropriation (86% of cases)

The most common type of fraud, involving theft of organizational assets:

```yaml
fraud:
  enabled: true
  acfe_category: asset_misappropriation
  schemes:
    cash_fraud:
      - skimming           # Sales not recorded
      - larceny            # Cash stolen after recording
      - shell_company      # Fictitious vendors
      - ghost_employee     # Non-existent employees
      - expense_schemes    # Personal expenses as business
    non_cash_fraud:
      - inventory_theft
      - fixed_asset_misuse
```

### Corruption (33% of cases)

Schemes involving conflicts of interest and bribery:

```yaml
fraud:
  enabled: true
  acfe_category: corruption
  schemes:
    - purchasing_conflict  # Undisclosed vendor ownership
    - sales_conflict       # Kickbacks from customers
    - invoice_kickback     # Vendor payment schemes
    - bid_rigging          # Collusion with vendors
    - economic_extortion   # Demands for payment
```

### Financial Statement Fraud (10% of cases)

The least common but most costly fraud type:

```yaml
fraud:
  enabled: true
  acfe_category: financial_statement
  schemes:
    overstatement:
      - premature_revenue      # Revenue before earned
      - fictitious_revenues    # Fake sales
      - concealed_liabilities  # Hidden obligations
      - improper_asset_values  # Overstated assets
    understatement:
      - understated_revenues   # Hidden sales
      - overstated_expenses    # Inflated costs
```

## ACFE Calibration

Generated fraud data is calibrated to match ACFE statistics:

| Metric | ACFE Value | Configuration |
|--------|------------|---------------|
| Median Loss | $117,000 | `acfe.median_loss` |
| Median Duration | 12 months | `acfe.median_duration_months` |
| Tip Detection | 42% | `detection_method.tip` |
| Internal Audit Detection | 16% | `detection_method.internal_audit` |
| Management Review Detection | 12% | `detection_method.management_review` |

```yaml
fraud:
  acfe_calibration:
    enabled: true
    median_loss: 117000
    median_duration_months: 12
    detection_methods:
      tip: 0.42
      internal_audit: 0.16
      management_review: 0.12
      external_audit: 0.04
      accident: 0.06
```

## Collusion & Conspiracy Modeling

SyntheticData models multi-party fraud networks with coordinated schemes:

### Collusion Ring Types

```rust
pub enum CollusionRingType {
    // Internal collusion
    EmployeePair,           // approver + processor
    DepartmentRing,         // 3-5 employees
    ManagementSubordinate,  // manager + subordinate

    // Internal-external
    EmployeeVendor,         // purchasing + vendor contact
    EmployeeCustomer,       // sales rep + customer
    EmployeeContractor,     // project manager + contractor

    // External rings
    VendorRing,             // bid rigging (2-4 vendors)
    CustomerRing,           // return fraud
}
```

### Conspirator Roles

Each conspirator in a ring has a specific role:

- **Initiator**: Conceives scheme, recruits others
- **Executor**: Performs fraudulent transactions
- **Approver**: Provides approvals/overrides
- **Concealer**: Hides evidence, manipulates records
- **Lookout**: Monitors for detection
- **Beneficiary**: External recipient of proceeds

### Configuration

```yaml
fraud:
  collusion:
    enabled: true
    ring_types:
      - type: employee_vendor
        probability: 0.15
        min_members: 2
        max_members: 4
      - type: department_ring
        probability: 0.08
        min_members: 3
        max_members: 5
    defection_probability: 0.05
    escalation_rate: 0.10
```

## Management Override

Senior-level fraud with override patterns:

```yaml
fraud:
  management_override:
    enabled: true
    perpetrator_levels:
      - senior_manager
      - cfo
      - ceo
    override_types:
      revenue:
        - journal_entry_override
        - revenue_recognition_acceleration
        - reserve_manipulation
      expense:
        - capitalization_abuse
        - expense_deferral
    pressure_sources:
      - financial_targets
      - market_expectations
      - covenant_compliance
```

### Fraud Triangle

The fraud triangle (Pressure, Opportunity, Rationalization) is modeled:

```yaml
fraud:
  fraud_triangle:
    pressure:
      source: financial_targets
      intensity: high
    opportunity:
      factors:
        - weak_internal_controls
        - management_override_capability
        - lack_of_oversight
    rationalization:
      type: temporary_adjustment  # "We'll fix it next quarter"
```

## Red Flag Generation

Probabilistic fraud indicators with calibrated Bayesian probabilities:

### Red Flag Strengths

| Strength | P(fraud\|flag) | Examples |
|----------|---------------|----------|
| Strong | > 0.5 | Matched home address vendor/employee |
| Moderate | 0.2 - 0.5 | Vendor with no physical address |
| Weak | < 0.2 | Round number invoices |

### Configuration

```yaml
fraud:
  red_flags:
    enabled: true
    inject_rate: 0.15  # 15% of transactions get flags
    patterns:
      strong:
        - name: matched_address_vendor_employee
          p_flag_given_fraud: 0.90
          p_flag_given_no_fraud: 0.001
        - name: sequential_check_numbers
          p_flag_given_fraud: 0.80
          p_flag_given_no_fraud: 0.01
      moderate:
        - name: approval_just_under_threshold
          p_flag_given_fraud: 0.70
          p_flag_given_no_fraud: 0.10
      weak:
        - name: round_number_invoice
          p_flag_given_fraud: 0.40
          p_flag_given_no_fraud: 0.20
```

## Evaluation Benchmarks

### ACFE-Calibrated Benchmarks

```rust
// General fraud detection
let bench = acfe_calibrated_1k();

// Collusion-focused benchmark
let bench = acfe_collusion_5k();

// Management override detection
let bench = acfe_management_override_2k();
```

### Benchmark Metrics

```rust
pub struct AcfeAlignment {
    /// Category distribution MAD vs ACFE
    pub category_distribution_mad: f64,
    /// Median loss ratio (actual / expected)
    pub median_loss_ratio: f64,
    /// Duration distribution KS statistic
    pub duration_distribution_ks: f64,
    /// Detection method chi-squared
    pub detection_method_chi_sq: f64,
}
```

## Output Files

| File | Description |
|------|-------------|
| `collusion_rings.json` | Collusion network details with members, roles |
| `red_flags.csv` | Red flag indicators with probabilities |
| `management_overrides.json` | Management override schemes |
| `fraud_labels.csv` | Enhanced fraud labels with ACFE category |

## Best Practices

1. **Start with ACFE calibration**: Use default ACFE statistics for realistic distribution
2. **Enable collusion gradually**: Start with simple rings before complex networks
3. **Use red flags for training**: Red flags provide weak supervision signals
4. **Validate against benchmarks**: Use ACFE benchmarks to verify model performance
5. **Consider detection difficulty**: Use `detection_difficulty` labels for curriculum learning
