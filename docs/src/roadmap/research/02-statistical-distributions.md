# Research: Statistical and Numerical Distributions

## Current State Analysis

### Existing Distribution Implementations

The system currently supports several distribution types:

| Distribution | Implementation | Usage |
|--------------|----------------|-------|
| **Log-Normal** | `AmountSampler` | Transaction amounts |
| **Benford's Law** | `BenfordSampler` | First-digit distribution |
| **Uniform** | Standard | ID generation, selection |
| **Weighted** | `LineItemSampler` | Line item counts |
| **Poisson** | `TemporalSampler` | Event counts |
| **Normal/Gaussian** | Standard | Some variations |

### Current Strengths

1. **Benford's Law compliance**: First-digit distribution follows expected 30.1%, 17.6%, 12.5%... pattern
2. **Log-normal amounts**: Realistic transaction size distributions
3. **Temporal weighting**: Period-end spikes, day-of-week patterns
4. **Industry seasonality**: 10 industry profiles with event-based multipliers

### Current Gaps

1. **Single-mode distributions**: No mixture models for multi-modal data
2. **Limited correlation**: Cross-field dependencies not modeled
3. **Static parameters**: No regime changes or parameter drift
4. **Missing distributions**: Pareto, Weibull, Beta not available
5. **No copulas**: Joint distributions not correlated realistically

---

## Improvement Recommendations

### 1. Multi-Modal Distribution Support

#### 1.1 Gaussian Mixture Models

Real-world transaction amounts often exhibit multiple modes:

```rust
/// Gaussian Mixture Model for multi-modal distributions
pub struct GaussianMixture {
    components: Vec<GaussianComponent>,
}

pub struct GaussianComponent {
    weight: f64,      // Component weight (sum to 1.0)
    mean: f64,        // Component mean
    std_dev: f64,     // Component standard deviation
}

impl GaussianMixture {
    /// Sample from the mixture distribution
    pub fn sample(&self, rng: &mut impl Rng) -> f64 {
        // Select component based on weights
        let component = self.select_component(rng);
        // Sample from selected Gaussian
        component.sample(rng)
    }
}
```

**Configuration**:
```yaml
amount_distribution:
  type: gaussian_mixture
  components:
    - weight: 0.60
      mean: 500
      std_dev: 200
      label: "small_transactions"
    - weight: 0.30
      mean: 5000
      std_dev: 1500
      label: "medium_transactions"
    - weight: 0.10
      mean: 50000
      std_dev: 15000
      label: "large_transactions"
```

#### 1.2 Log-Normal Mixture

For strictly positive amounts with multiple modes:

```yaml
amount_distribution:
  type: lognormal_mixture
  components:
    - weight: 0.70
      mu: 5.5       # log-scale mean
      sigma: 1.2    # log-scale std dev
      label: "routine_expenses"
    - weight: 0.25
      mu: 8.5
      sigma: 0.8
      label: "capital_expenses"
    - weight: 0.05
      mu: 11.0
      sigma: 0.5
      label: "major_projects"
```

#### 1.3 Realistic Transaction Amount Profiles

**By Transaction Type**:

| Type | Distribution | Parameters | Notes |
|------|--------------|------------|-------|
| Petty Cash | Log-normal | μ=3.5, σ=0.8 | $10-$500 range |
| AP Invoices | Mixture(3) | See below | Multi-modal |
| Payroll | Normal | μ=4500, σ=1200 | Per employee |
| Utilities | Log-normal | μ=7.0, σ=0.4 | Monthly, stable |
| Capital | Pareto | α=1.5, xₘ=10000 | Heavy tail |

**AP Invoice Mixture**:
```yaml
ap_invoices:
  type: lognormal_mixture
  components:
    # Operating expenses
    - weight: 0.50
      mu: 6.0        # ~$400 median
      sigma: 1.5
    # Inventory/materials
    - weight: 0.35
      mu: 8.0        # ~$3000 median
      sigma: 1.0
    # Capital/projects
    - weight: 0.15
      mu: 10.5       # ~$36000 median
      sigma: 0.8
```

---

### 2. Cross-Field Correlation Modeling

#### 2.1 Correlation Matrix Support

Define correlations between numeric fields:

```yaml
correlations:
  enabled: true
  fields:
    - name: transaction_amount
    - name: line_item_count
    - name: approval_level
    - name: processing_time_hours
    - name: discount_percentage

  matrix:
    # Correlation coefficients (Pearson's r)
    # Higher amounts → more line items
    - [1.00, 0.65, 0.72, 0.45, -0.20]
    # More items → higher amount
    - [0.65, 1.00, 0.55, 0.60, -0.15]
    # Higher amount → higher approval
    - [0.72, 0.55, 1.00, 0.50, -0.30]
    # More complex → longer processing
    - [0.45, 0.60, 0.50, 1.00, -0.10]
    # Higher amount → lower discount %
    - [-0.20, -0.15, -0.30, -0.10, 1.00]
```

#### 2.2 Copula-Based Generation

For more sophisticated dependency modeling:

```rust
/// Copula types for dependency modeling
pub enum CopulaType {
    /// Gaussian copula - symmetric dependencies
    Gaussian { correlation: f64 },
    /// Clayton copula - lower tail dependence
    Clayton { theta: f64 },
    /// Gumbel copula - upper tail dependence
    Gumbel { theta: f64 },
    /// Frank copula - symmetric, no tail dependence
    Frank { theta: f64 },
    /// Student-t copula - both tail dependencies
    StudentT { correlation: f64, df: f64 },
}

pub struct CopulaGenerator {
    copula: CopulaType,
    marginals: Vec<Box<dyn Distribution>>,
}
```

**Use Cases**:
- **Amount & Days-to-Pay**: Larger invoices may have longer payment terms (Clayton copula)
- **Revenue & COGS**: Strong positive correlation (Gaussian copula)
- **Fraud Amount & Detection Delay**: Upper tail dependence (Gumbel copula)

#### 2.3 Conditional Distributions

Generate values conditional on other fields:

```yaml
conditional_distributions:
  # Discount percentage depends on order amount
  discount:
    type: conditional
    given: order_amount
    breakpoints:
      - threshold: 1000
        distribution: { type: constant, value: 0 }
      - threshold: 5000
        distribution: { type: uniform, min: 0, max: 0.05 }
      - threshold: 25000
        distribution: { type: uniform, min: 0.05, max: 0.10 }
      - threshold: 100000
        distribution: { type: uniform, min: 0.10, max: 0.15 }
      - threshold: infinity
        distribution: { type: normal, mean: 0.15, std: 0.03 }

  # Payment terms depend on vendor relationship
  payment_terms:
    type: conditional
    given: vendor_relationship_months
    breakpoints:
      - threshold: 6
        distribution: { type: choice, values: [0, 15], weights: [0.8, 0.2] }
      - threshold: 24
        distribution: { type: choice, values: [15, 30], weights: [0.6, 0.4] }
      - threshold: infinity
        distribution: { type: choice, values: [30, 45, 60], weights: [0.5, 0.35, 0.15] }
```

---

### 3. Industry-Specific Amount Distributions

#### 3.1 Retail

```yaml
retail:
  transaction_amounts:
    pos_sales:
      type: lognormal_mixture
      components:
        - weight: 0.65
          mu: 3.0      # ~$20 median
          sigma: 1.0
          label: "small_basket"
        - weight: 0.30
          mu: 4.5      # ~$90 median
          sigma: 0.8
          label: "medium_basket"
        - weight: 0.05
          mu: 6.0      # ~$400 median
          sigma: 0.6
          label: "large_basket"

    inventory_orders:
      type: lognormal
      mu: 9.0          # ~$8000 median
      sigma: 1.5

    seasonal_multipliers:
      black_friday: 3.5
      christmas_week: 2.8
      back_to_school: 1.6
```

#### 3.2 Manufacturing

```yaml
manufacturing:
  transaction_amounts:
    raw_materials:
      type: lognormal_mixture
      components:
        - weight: 0.40
          mu: 8.0      # ~$3000 median
          sigma: 1.0
          label: "consumables"
        - weight: 0.45
          mu: 10.0     # ~$22000 median
          sigma: 0.8
          label: "production_materials"
        - weight: 0.15
          mu: 12.0     # ~$163000 median
          sigma: 0.6
          label: "bulk_orders"

    maintenance:
      type: pareto
      alpha: 2.0
      x_min: 500
      label: "repair_costs"

    capital_equipment:
      type: lognormal
      mu: 12.5         # ~$268000 median
      sigma: 1.0
```

#### 3.3 Financial Services

```yaml
financial_services:
  transaction_amounts:
    wire_transfers:
      type: lognormal_mixture
      components:
        - weight: 0.30
          mu: 8.0      # ~$3000
          sigma: 1.2
          label: "retail_wire"
        - weight: 0.40
          mu: 11.0     # ~$60000
          sigma: 1.0
          label: "commercial_wire"
        - weight: 0.20
          mu: 14.0     # ~$1.2M
          sigma: 0.8
          label: "institutional_wire"
        - weight: 0.10
          mu: 17.0     # ~$24M
          sigma: 1.0
          label: "large_value"

    ach_transactions:
      type: lognormal
      mu: 7.5          # ~$1800
      sigma: 2.0

    fee_income:
      type: weibull
      scale: 500
      shape: 1.5
```

---

### 4. Regime Change Modeling

#### 4.1 Structural Breaks

Model sudden changes in distribution parameters:

```yaml
regime_changes:
  enabled: true
  changes:
    - date: "2024-03-15"
      type: acquisition
      effects:
        - field: transaction_volume
          multiplier: 1.35
        - field: average_amount
          shift: 5000
        - field: vendor_count
          multiplier: 1.25

    - date: "2024-07-01"
      type: price_increase
      effects:
        - field: cogs_ratio
          shift: 0.03
        - field: avg_invoice_amount
          multiplier: 1.08

    - date: "2024-10-01"
      type: new_product_line
      effects:
        - field: revenue
          multiplier: 1.20
        - field: inventory_turns
          multiplier: 0.85
```

#### 4.2 Gradual Parameter Drift

Model slow changes over time:

```yaml
parameter_drift:
  enabled: true
  parameters:
    - field: transaction_amount
      type: linear
      annual_drift: 0.03    # 3% annual increase (inflation)

    - field: digital_payment_ratio
      type: logistic
      start_value: 0.40
      end_value: 0.85
      midpoint_months: 18
      steepness: 0.15

    - field: approval_threshold
      type: step
      steps:
        - month: 6
          value: 5000
        - month: 18
          value: 7500
        - month: 30
          value: 10000
```

#### 4.3 Economic Cycle Modeling

```yaml
economic_cycles:
  enabled: true
  base_cycle:
    type: sinusoidal
    period_months: 48      # 4-year cycle
    amplitude: 0.15        # ±15% variation

  recession_events:
    - start: "2024-09-01"
      duration_months: 8
      severity: moderate    # 10-20% decline
      effects:
        - revenue: -0.15
        - discretionary_spend: -0.35
        - capital_investment: -0.50
        - headcount: -0.10
      recovery:
        type: gradual
        months: 12
```

---

### 5. Enhanced Benford's Law Compliance

#### 5.1 Second and Third Digit Distributions

Extend beyond first-digit to full Benford compliance:

```rust
pub struct BenfordDistribution {
    digits: BenfordDigitConfig,
}

pub struct BenfordDigitConfig {
    first_digit: bool,     // Standard Benford
    second_digit: bool,    // Second digit distribution
    first_two: bool,       // Joint first-two digits
    summation: bool,       // Summation test
}

impl BenfordDistribution {
    /// Generate amount following full Benford's Law
    pub fn sample_benford_compliant(&self, rng: &mut impl Rng) -> Decimal {
        // Use log-uniform distribution to ensure Benford compliance
        // across multiple digit positions
    }
}
```

#### 5.2 Benford Deviation Injection

For anomaly scenarios, intentionally violate Benford:

```yaml
benford_deviations:
  enabled: false  # Enable for fraud scenarios

  deviation_types:
    # Round number preference (fraud indicator)
    round_number_bias:
      probability: 0.15
      targets: [1000, 5000, 10000, 25000]
      tolerance: 0.01

    # Threshold avoidance (approval bypass)
    threshold_clustering:
      thresholds: [5000, 10000, 25000]
      cluster_below: true
      distance: 50-200

    # Uniform distribution (fabricated data)
    uniform_injection:
      probability: 0.05
      range: [1000, 9999]
```

---

### 6. Statistical Validation Framework

#### 6.1 Distribution Fitness Tests

```rust
pub struct DistributionValidator {
    tests: Vec<StatisticalTest>,
}

pub enum StatisticalTest {
    /// Kolmogorov-Smirnov test
    KolmogorovSmirnov { significance: f64 },
    /// Chi-squared goodness of fit
    ChiSquared { bins: usize, significance: f64 },
    /// Anderson-Darling test
    AndersonDarling { significance: f64 },
    /// Benford's Law chi-squared
    BenfordChiSquared { digits: u8, significance: f64 },
    /// Mean Absolute Deviation from Benford
    BenfordMAD { threshold: f64 },
}
```

#### 6.2 Validation Configuration

```yaml
validation:
  statistical_tests:
    enabled: true
    tests:
      - type: benford_first_digit
        threshold_mad: 0.015
        warning_mad: 0.010

      - type: distribution_fit
        target: lognormal
        ks_significance: 0.05

      - type: correlation_check
        expected_correlations:
          - fields: [amount, line_items]
            expected_r: 0.65
            tolerance: 0.10

  reporting:
    generate_plots: true
    output_format: html
    include_raw_data: false
```

---

### 7. New Distribution Types

#### 7.1 Pareto Distribution

For heavy-tailed phenomena (80/20 rule):

```yaml
# Top 20% of customers generate 80% of revenue
customer_revenue:
  type: pareto
  alpha: 1.16      # Shape parameter for 80/20
  x_min: 1000      # Minimum value
  truncate_max: 10000000  # Optional cap
```

#### 7.2 Weibull Distribution

For time-to-event data:

```yaml
# Days until payment
days_to_payment:
  type: weibull
  shape: 2.0       # k > 1: increasing hazard (more likely to pay over time)
  scale: 30.0      # λ: characteristic life
  shift: 0         # Minimum days
```

#### 7.3 Beta Distribution

For proportions and percentages:

```yaml
# Discount percentage
discount_rate:
  type: beta
  alpha: 2.0       # Shape parameter 1
  beta: 8.0        # Shape parameter 2
  # This gives mode around 11%, right-skewed
  scale:
    min: 0.0
    max: 0.25      # Max 25% discount
```

#### 7.4 Zero-Inflated Distributions

For data with excess zeros:

```yaml
# Credits/returns (many transactions have zero)
credit_amount:
  type: zero_inflated
  zero_probability: 0.85
  positive_distribution:
    type: lognormal
    mu: 5.0
    sigma: 1.5
```

---

### 8. Implementation Priority

| Enhancement | Complexity | Impact | Priority |
|-------------|------------|--------|----------|
| Mixture models | Medium | High | P1 |
| Correlation matrices | High | Critical | P1 |
| Industry-specific profiles | Medium | High | P1 |
| Regime changes | Medium | High | P2 |
| Copula support | High | Medium | P2 |
| Additional distributions | Low | Medium | P2 |
| Validation framework | Medium | High | P1 |
| Conditional distributions | Medium | Medium | P3 |

---

### 9. Configuration Example

```yaml
# Complete statistical distribution configuration
distributions:
  # Global amount settings
  amounts:
    default:
      type: lognormal_mixture
      components:
        - { weight: 0.6, mu: 6.0, sigma: 1.5 }
        - { weight: 0.3, mu: 8.5, sigma: 1.0 }
        - { weight: 0.1, mu: 11.0, sigma: 0.8 }

    by_transaction_type:
      payroll:
        type: normal
        mean: 4500
        std_dev: 1500
        truncate_min: 1000

      utilities:
        type: lognormal
        mu: 7.0
        sigma: 0.5

  # Correlation settings
  correlations:
    enabled: true
    model: gaussian_copula
    pairs:
      - fields: [amount, processing_days]
        correlation: 0.45
      - fields: [amount, approval_level]
        correlation: 0.72

  # Drift settings
  drift:
    enabled: true
    inflation_rate: 0.03
    regime_changes:
      - date: "2024-06-01"
        field: avg_transaction
        multiplier: 1.15

  # Validation
  validation:
    benford_compliance: true
    distribution_tests: true
    correlation_verification: true
```

---

## Technical Implementation Notes

### Performance Considerations

1. **Pre-computation**: Calculate CDF tables for frequently-used distributions
2. **Vectorization**: Use SIMD for batch sampling where possible
3. **Caching**: Cache correlation matrix decompositions (Cholesky)
4. **Lazy evaluation**: Defer complex distribution calculations until needed

### Memory Efficiency

1. **Streaming**: Generate correlated samples in batches
2. **Reference tables**: Use compact lookup tables for standard distributions
3. **On-demand**: Compute regime-adjusted parameters at sample time

---

*See also*: [03-temporal-patterns.md](03-temporal-patterns.md) for time-based distributions
