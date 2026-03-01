# Spec 07: Cross-Factor Correlation & Coherence Engine

**Status**: Draft
**Priority**: High
**Depends on**: Spec 01 (Macro Engine), Spec 02 (Shock System)
**Extends**: `datasynth-core/src/distributions/copula.rs`, `correlation.rs`

---

## 1. Problem Statement

Even with individual factor models generating realistic paths, the **joint behavior**
of these factors can be unrealistic if they move independently. In real economies,
factors are deeply interconnected: when GDP falls, unemployment rises, credit spreads
widen, defaults increase, and consumer confidence collapses — simultaneously, not
in isolation. The current copula module (`copula.rs`) handles field-level correlations
within records, but there is no mechanism to enforce **cross-module, cross-factor**
coherence at the macroeconomic level.

A generated scenario where GDP is declining but unemployment is simultaneously falling
and credit spreads tightening would be immediately recognizable as synthetic. This spec
introduces the engine that prevents such incoherence.

## 2. Scientific Foundation

### 2.1 Vector Autoregression for Factor Coherence

A VAR(p) model (Sims, 1980) inherently enforces cross-factor coherence because each
factor depends on its own lags **and** the lags of all other factors:

```
Y_t = c + A_1·Y_{t-1} + ... + A_p·Y_{t-p} + ε_t,  ε_t ~ N(0, Σ)
```

The coefficient matrices `A_i` encode the **impulse response structure**: how a shock
to one factor propagates to all others over time. The innovation covariance matrix `Σ`
captures the **contemporaneous correlation** of shocks.

For a 5-factor system (GDP, rate, inflation, unemployment, credit spread), the system
has 25 parameters per lag. With p=2 lags, that's 50 cross-factor coefficients plus
15 unique covariance parameters = 65 parameters total.

**Key impulse response relationships** (Stock & Watson, 2001):

| Shock to | GDP Response | Inflation Response | Unemployment Response | Timing |
|----------|-------------|-------------------|---------------------|--------|
| Interest rate +100bp | -0.7% | -1.0% | +0.3pp | 2-6 quarters |
| GDP +1% | — | +0.3% | -0.4pp | 1-3 quarters |
| Inflation +1% | -0.3% | — | +0.2pp | 2-4 quarters |
| Credit spread +100bp | -0.5% | -0.2% | +0.2pp | 1-4 quarters |

**Reference**: Stock, J.H. & Watson, M.W. (2001). "Vector Autoregressions."
*Journal of Economic Perspectives*, 15(4), 101-115.

### 2.2 Copula-Based Dependency Structure

Copulas separate marginal distributions from their dependency structure via
**Sklar's theorem**:

```
F(x₁,...,x_d) = C(F₁(x₁),...,F_d(x_d))
```

For macroeconomic factors, tail dependence is critical — crisis events cause
factors to become **more correlated** than in normal times.

| Copula | Tail Dependence | Best For |
|--------|----------------|----------|
| **Gaussian** | None (symmetric, no tail) | Normal regime modeling |
| **Student-t** | Symmetric heavy tails | Joint crashes and booms (df=4-6) |
| **Clayton** | Lower tail (asymmetric) | Crisis co-movement only |
| **Gumbel** | Upper tail (asymmetric) | Boom co-movement |
| **Frank** | No tail (symmetric) | Weak/moderate dependence |

**Key finding**: Research on financial return data shows significant evidence for
non-zero tail dependence, implying that the Gaussian copula (which was partly
responsible for mispricing CDOs in 2008) **underestimates joint extreme events**.
A Student-t copula with df=4-6 is recommended for macro factor generation.

**Reference**: Embrechts, P., Lindskog, F., & McNeil, A. (2003). "Modelling
Dependence with Copulas." Department of Mathematics, ETHZ.

### 2.3 Macro-Financial Coherence Constraints

The Fed's stress test macro model enforces coherence through structural relationships:

1. **Phillips Curve**: π_t = f(u_t, E[π_{t+1}]) — inflation constrained by unemployment
2. **Taylor Rule**: i_t = r* + π_t + 0.5(π_t - π*) + 0.5(y_t - y*_t) — rates constrained
   by inflation and output gap
3. **Okun's Law**: Δu_t ≈ -0.5 · (g_t - g*) — unemployment responds to growth gap
4. **Credit channel**: spread_t = f(PD_t, VIX_t, GDP_t) — credit follows macro state
5. **IS curve**: y_t = f(i_t, expectations, fiscal) — output responds to rates

These constraints can be implemented as **soft bounds** (penalty-based correction)
or **hard bounds** (rejection sampling) in the coherence engine.

### 2.4 Dynamic Factor Models

Stock & Watson's Dynamic Factor Model (DFM) extracts a small number of common
factors from many observed series:

```
X_it = λ_i · F_t + e_it
F_t = Φ · F_{t-1} + η_t
```

Where `F_t` is a vector of 2-3 common factors that drive the co-movement of all
observable macro variables. This achieves dimensionality reduction while preserving
the correlation structure.

**For synthetic data**: Generate the common factors first (using VAR), then derive
observable macro variables via factor loadings `λ_i`. This guarantees coherence
by construction.

**Reference**: Stock, J.H. & Watson, M.W. (2016). "Dynamic Factor Models,
Factor-Augmented Vector Autoregressions, and Structural Vector Autoregressions in
Macroeconomics." *Handbook of Macroeconomics*, Vol. 2, 415-525.

## 3. Proposed Design

### 3.1 Correlation Engine

```rust
/// Cross-factor correlation and coherence engine
pub struct CrossFactorEngine {
    /// Factor correlation model
    model: CorrelationModel,
    /// Structural constraints
    constraints: Vec<CoherenceConstraint>,
    /// Historical correlation matrix (target)
    target_correlation: Array2<f64>,
    /// Current regime (affects correlations)
    current_regime: EconomicRegime,
    /// Regime-dependent correlation matrices
    regime_correlations: HashMap<EconomicRegime, Array2<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrelationModel {
    /// Use VAR for generating correlated factor paths (recommended)
    Var {
        coefficients: VarCoefficients,
        innovation_copula: CopulaSpec,
    },
    /// Use a dynamic factor model
    DynamicFactor {
        n_factors: usize,
        factor_loadings: Array2<f64>,
        factor_var: VarCoefficients,
    },
    /// Use direct copula on factor innovations
    DirectCopula {
        copula: CopulaSpec,
        marginals: Vec<MarginalSpec>,
    },
    /// No correlation enforcement (factors independent)
    Independent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopulaSpec {
    pub copula_type: CopulaType,
    pub correlation_matrix: Array2<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CopulaType {
    Gaussian,
    StudentT { degrees_of_freedom: f64 },
    Clayton { theta: f64 },
    Gumbel { theta: f64 },
    Frank { theta: f64 },
}
```

### 3.2 Structural Coherence Constraints

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoherenceConstraint {
    /// Phillips Curve: inflation responds to unemployment gap
    PhillipsCurve {
        /// Slope (κ): how much inflation changes per unit of unemployment gap
        slope: f64,
        /// Natural rate of unemployment (NAIRU)
        nairu: f64,
        /// Inflation expectations anchoring (0=unanchored, 1=fully anchored)
        anchoring: f64,
    },

    /// Taylor Rule: policy rate responds to inflation and output gap
    TaylorRule {
        /// Neutral real rate (r*)
        neutral_rate: f64,
        /// Inflation target (π*)
        inflation_target: f64,
        /// Inflation response coefficient (typically 1.5)
        inflation_response: f64,
        /// Output gap response coefficient (typically 0.5)
        output_response: f64,
        /// Interest rate smoothing parameter (0-1)
        smoothing: f64,
        /// Effective lower bound
        lower_bound: f64,
    },

    /// Okun's Law: unemployment responds to GDP growth gap
    OkunsLaw {
        /// Okun coefficient (typically -0.5)
        coefficient: f64,
        /// Potential GDP growth rate
        potential_growth: f64,
    },

    /// Credit channel: spreads respond to macro conditions
    CreditChannel {
        /// GDP sensitivity (spread widening per % GDP decline)
        gdp_sensitivity: f64,
        /// Unemployment sensitivity
        unemployment_sensitivity: f64,
        /// Base spread (bps) in normal times
        base_spread_bps: f64,
    },

    /// Sign constraint: variables must move in expected direction
    SignConstraint {
        /// If this variable decreases...
        trigger_variable: MacroVariable,
        /// ...this variable must not decrease (or must increase)
        constrained_variable: MacroVariable,
        /// Direction: Positive = must move same direction, Negative = opposite
        relationship: SignRelationship,
    },

    /// Bound constraint: variable must stay within range
    BoundConstraint {
        variable: MacroVariable,
        min: Option<f64>,
        max: Option<f64>,
    },

    /// Lag constraint: variable B responds to variable A with delay
    LagConstraint {
        leading_variable: MacroVariable,
        lagging_variable: MacroVariable,
        lag_periods: usize,
        response_coefficient: f64,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MacroVariable {
    GdpGrowth,
    PolicyRate,
    CpiInflation,
    Unemployment,
    CreditSpread,
    ConsumerConfidence,
    EquityReturn,
    HousePrices,
    CommodityPrices,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SignRelationship {
    /// Variables move in the same direction
    Positive,
    /// Variables move in opposite directions
    Negative,
}
```

### 3.3 Coherence Validation & Correction

```rust
impl CrossFactorEngine {
    /// Validate a factor vector for coherence and correct if needed
    pub fn ensure_coherence(
        &self,
        mut adjustments: ExternalAdjustments,
        period: usize,
    ) -> ExternalAdjustments {
        for constraint in &self.constraints {
            match constraint {
                CoherenceConstraint::TaylorRule {
                    neutral_rate, inflation_target,
                    inflation_response, output_response,
                    smoothing, lower_bound,
                } => {
                    // Implied policy rate from Taylor Rule
                    let implied_rate = neutral_rate
                        + adjustments.inflation()
                        + inflation_response * (adjustments.inflation() - inflation_target)
                        + output_response * adjustments.output_gap();

                    let implied_rate = implied_rate.max(*lower_bound);

                    // Blend actual and implied (soft constraint)
                    let actual_rate = adjustments.policy_rate();
                    let corrected = smoothing * actual_rate + (1.0 - smoothing) * implied_rate;

                    adjustments.set_policy_rate(corrected);
                }

                CoherenceConstraint::OkunsLaw { coefficient, potential_growth } => {
                    let growth_gap = adjustments.gdp_growth() - potential_growth;
                    let implied_unemployment_change = coefficient * growth_gap;
                    let max_deviation = 0.02; // Allow ±2pp deviation from Okun

                    let actual_change = adjustments.unemployment_change();
                    if (actual_change - implied_unemployment_change).abs() > max_deviation {
                        let corrected = implied_unemployment_change
                            + (actual_change - implied_unemployment_change)
                                .clamp(-max_deviation, max_deviation);
                        adjustments.set_unemployment_change(corrected);
                    }
                }

                CoherenceConstraint::SignConstraint {
                    trigger_variable, constrained_variable, relationship,
                } => {
                    let trigger_change = adjustments.variable_change(*trigger_variable);
                    let constrained_change = adjustments.variable_change(*constrained_variable);

                    let violation = match relationship {
                        SignRelationship::Positive => {
                            trigger_change.signum() != constrained_change.signum()
                                && trigger_change.abs() > 0.001
                        }
                        SignRelationship::Negative => {
                            trigger_change.signum() == constrained_change.signum()
                                && trigger_change.abs() > 0.001
                        }
                    };

                    if violation {
                        // Correct the constrained variable to match expected sign
                        let corrected = match relationship {
                            SignRelationship::Positive => {
                                constrained_change.abs() * trigger_change.signum()
                            }
                            SignRelationship::Negative => {
                                constrained_change.abs() * (-trigger_change.signum())
                            }
                        };
                        adjustments.set_variable_change(*constrained_variable, corrected);
                    }
                }

                CoherenceConstraint::BoundConstraint { variable, min, max } => {
                    let value = adjustments.variable_value(*variable);
                    if let Some(min_val) = min {
                        if value < *min_val {
                            adjustments.set_variable_value(*variable, *min_val);
                        }
                    }
                    if let Some(max_val) = max {
                        if value > *max_val {
                            adjustments.set_variable_value(*variable, *max_val);
                        }
                    }
                }

                _ => {} // Other constraints
            }
        }

        adjustments
    }

    /// Validate a complete factor time series for coherence
    pub fn validate_series(
        &self,
        factors: &[MacroFactors],
    ) -> CoherenceReport {
        let mut report = CoherenceReport::new();

        for (i, window) in factors.windows(2).enumerate() {
            let prev = &window[0];
            let curr = &window[1];

            // Check sign relationships
            self.check_sign_coherence(prev, curr, i, &mut report);

            // Check Phillips Curve consistency
            self.check_phillips_curve(curr, i, &mut report);

            // Check Taylor Rule consistency
            self.check_taylor_rule(curr, i, &mut report);

            // Check Okun's Law consistency
            self.check_okuns_law(prev, curr, i, &mut report);

            // Check bound constraints
            self.check_bounds(curr, i, &mut report);
        }

        // Check correlation matrix matches target
        self.check_correlation_matrix(factors, &mut report);

        report
    }

    /// Check that realized correlations match target
    fn check_correlation_matrix(
        &self,
        factors: &[MacroFactors],
        report: &mut CoherenceReport,
    ) {
        let realized = self.compute_realized_correlations(factors);
        let target = self.regime_correlations.get(&self.current_regime)
            .unwrap_or(&self.target_correlation);

        let frobenius_norm = (&realized - target).mapv(|x| x * x).sum().sqrt();
        let max_element_deviation = (&realized - target).mapv(f64::abs)
            .iter().cloned().fold(0.0_f64, f64::max);

        report.correlation_frobenius_error = frobenius_norm;
        report.max_correlation_deviation = max_element_deviation;

        if max_element_deviation > 0.15 {
            report.add_warning(format!(
                "Max correlation deviation {:.3} exceeds threshold 0.15",
                max_element_deviation,
            ));
        }
    }
}
```

### 3.4 Regime-Dependent Correlation Matrices

```rust
impl CrossFactorEngine {
    /// Pre-calibrated correlation matrices by economic regime
    fn default_regime_correlations() -> HashMap<EconomicRegime, Array2<f64>> {
        let mut map = HashMap::new();

        // Order: GDP, Rate, Inflation, Unemployment, Credit Spread
        // Expansion regime — moderate, stable correlations
        map.insert(EconomicRegime::Expansion, array![
            [ 1.00,  0.30,  0.25, -0.60, -0.35],  // GDP
            [ 0.30,  1.00,  0.55, -0.20, -0.15],  // Rate
            [ 0.25,  0.55,  1.00, -0.30,  0.10],  // Inflation
            [-0.60, -0.20, -0.30,  1.00,  0.40],  // Unemployment
            [-0.35, -0.15,  0.10,  0.40,  1.00],  // Credit Spread
        ]);

        // Contraction regime — stronger correlations, crisis behavior
        map.insert(EconomicRegime::Contraction, array![
            [ 1.00,  0.50,  0.15, -0.80, -0.70],  // GDP ↔ Unemployment stronger
            [ 0.50,  1.00,  0.40, -0.40, -0.50],  // Rate ↔ Spread stronger
            [ 0.15,  0.40,  1.00, -0.10,  0.20],  // Inflation decouples
            [-0.80, -0.40, -0.10,  1.00,  0.65],  // Unemployment ↔ Spread stronger
            [-0.70, -0.50,  0.20,  0.65,  1.00],  // Spread ↔ GDP much stronger
        ]);

        // Stagflation regime — unusual correlation structure
        map.insert(EconomicRegime::Stagflation, array![
            [ 1.00,  0.10, -0.40, -0.50, -0.45],  // GDP ↔ Inflation negative!
            [ 0.10,  1.00,  0.70, -0.10, -0.20],  // Rate follows inflation
            [-0.40,  0.70,  1.00,  0.30,  0.35],  // Inflation ↔ Unemployment positive!
            [-0.50, -0.10,  0.30,  1.00,  0.50],  // Unusual: both high
            [-0.45, -0.20,  0.35,  0.50,  1.00],
        ]);

        map
    }
}
```

### 3.5 Coherence Report

```rust
#[derive(Debug, Clone)]
pub struct CoherenceReport {
    /// Overall coherence score (0.0 = incoherent, 1.0 = perfectly coherent)
    pub overall_score: f64,
    /// Constraint violation count
    pub violations: Vec<CoherenceViolation>,
    /// Warnings (soft constraint deviations)
    pub warnings: Vec<String>,
    /// Correlation matrix Frobenius error
    pub correlation_frobenius_error: f64,
    /// Maximum element-wise correlation deviation
    pub max_correlation_deviation: f64,
    /// Per-period coherence scores
    pub period_scores: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct CoherenceViolation {
    pub period: usize,
    pub constraint: String,
    pub expected: String,
    pub actual: String,
    pub severity: ViolationSeverity,
    pub auto_corrected: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ViolationSeverity {
    /// Minor deviation, acceptable
    Info,
    /// Notable deviation, corrected
    Warning,
    /// Significant violation, may indicate model misconfiguration
    Error,
}
```

## 4. Configuration Schema

```yaml
external_realism:
  cross_factor_correlation:
    enabled: true

    # Correlation model
    model: var                             # var | dynamic_factor | direct_copula | independent

    # VAR-based correlation (recommended)
    var:
      lags: 2
      parameter_set: great_moderation     # Pre-calibrated parameters
      innovation_copula:
        type: student_t                    # gaussian | student_t | clayton
        degrees_of_freedom: 5             # For Student-t
      # Custom correlation matrix (overrides parameter set)
      custom_correlation_matrix: null

    # Dynamic factor model (alternative)
    dynamic_factor:
      n_factors: 2
      # Factor loadings (5 variables × 2 factors)
      loadings: null                       # Auto-estimated if null

    # Structural constraints
    constraints:
      phillips_curve:
        enabled: true
        slope: 0.06                        # κ: inflation sensitivity to unemployment
        nairu: 0.045                       # Natural rate of unemployment
        anchoring: 0.80                    # Expectation anchoring (high = stable)

      taylor_rule:
        enabled: true
        neutral_rate: 0.025                # r* = 2.5%
        inflation_target: 0.02             # π* = 2%
        inflation_response: 1.5            # Standard Taylor coefficient
        output_response: 0.5               # Standard Taylor coefficient
        smoothing: 0.75                    # High inertia (realistic)
        lower_bound: 0.0                   # Zero lower bound

      okuns_law:
        enabled: true
        coefficient: -0.5
        potential_growth: 0.02             # 2% potential growth

      credit_channel:
        enabled: true
        gdp_sensitivity: 80.0             # 80bps spread per 1% GDP decline
        unemployment_sensitivity: 30.0     # 30bps per 1pp unemployment rise
        base_spread_bps: 150.0

      sign_constraints:
        - trigger: gdp_growth
          constrained: unemployment
          relationship: negative            # GDP ↓ → Unemployment ↑
        - trigger: gdp_growth
          constrained: credit_spread
          relationship: negative            # GDP ↓ → Spreads ↑
        - trigger: gdp_growth
          constrained: consumer_confidence
          relationship: positive            # GDP ↓ → Confidence ↓

      bound_constraints:
        - variable: unemployment
          min: 0.02
          max: 0.25
        - variable: policy_rate
          min: 0.0
          max: 0.20
        - variable: cpi_inflation
          min: -0.05
          max: 0.30

    # Regime-dependent correlations
    regime_correlations:
      enabled: true
      profile: default                     # default | custom
      # Custom: override specific regime correlations
      custom: {}

    # Coherence validation
    validation:
      enabled: true
      max_correlation_deviation: 0.15     # Alert threshold
      fail_on_violation: false            # Fail generation on coherence violation
      auto_correct: true                  # Automatically fix violations
      export_report: true                 # Export coherence report

    # Output
    export_correlation_matrices: true
    export_factor_loadings: true
```

## 5. Default Constraint Set: "Standard Macro"

The default constraint set encodes well-established macroeconomic relationships:

| Constraint | Relationship | Parameters | Source |
|-----------|-------------|------------|--------|
| Phillips Curve | π ↔ u | κ=0.06, NAIRU=4.5% | Hazell et al. (2022) |
| Taylor Rule | i ↔ π, y | α_π=1.5, α_y=0.5, ρ=0.75 | Taylor (1993) |
| Okun's Law | Δu ↔ Δy | β=-0.5 | Ball et al. (2017) |
| Credit Channel | s ↔ y, u | 80bps/1%GDP, 30bps/1ppU | Gilchrist & Zakrajšek (2012) |
| GDP → Unemployment | Negative | With 1-quarter lag | NBER data |
| GDP → Credit Spread | Negative | With 1-2 quarter lag | Empirical |
| GDP → Confidence | Positive | Contemporaneous | Conference Board |
| Inflation → Rate | Positive | With 1-quarter lag | Taylor Rule |
| Rate → GDP | Negative | With 2-6 quarter lag | Romer & Romer (2004) |

## 6. Testing Strategy

### Unit Tests
- VAR coefficient matrices are stationary (eigenvalues within unit circle)
- Taylor Rule produces reasonable rates for given inflation/output gap
- Okun's Law correction stays within ±2pp deviation bounds
- Sign constraints correctly flip violated signs
- Bound constraints enforce min/max

### Integration Tests
- Full scenario coherence validation produces score >0.85
- Regime-switching changes active correlation matrix
- Student-t copula produces heavier tails than Gaussian
- Coherence report identifies intentionally injected violations

### Statistical Tests
- Realized correlation matrix within Frobenius norm <0.2 of target
- No sign violation in >95% of period transitions
- Taylor Rule deviation <100bps for 90% of periods
- Okun coefficient realized within [-0.3, -0.7] range

## References

1. Sims, C.A. (1980). "Macroeconomics and Reality." *Econometrica*, 48(1), 1-48.
2. Stock, J.H. & Watson, M.W. (2001). "Vector Autoregressions." *JEP*, 15(4), 101-115.
3. Stock, J.H. & Watson, M.W. (2016). "Dynamic Factor Models." *Handbook of Macroeconomics*, Vol. 2.
4. Taylor, J.B. (1993). "Discretion versus Policy Rules in Practice." *Carnegie-Rochester Conference Series*, 39, 195-214.
5. Hazell, J., Herreño, J., Nakamura, E., & Steinsson, J. (2022). "The Slope of the Phillips Curve." *QJE*, 137(3).
6. Gilchrist, S. & Zakrajšek, E. (2012). "Credit Spreads and Business Cycle Fluctuations." *AER*, 102(4), 1692-1720.
7. Embrechts, P., Lindskog, F., & McNeil, A. (2003). "Modelling Dependence with Copulas." ETHZ.
8. Ball, L., Leigh, D., & Loungani, P. (2017). "Okun's Law: Fit at 50?" *Journal of Money, Credit and Banking*, 49(7).
9. Romer, C.D. & Romer, D.H. (2004). "A New Measure of Monetary Shocks." *AER*, 94(4), 1055-1084.
