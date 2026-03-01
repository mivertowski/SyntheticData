# Spec 01: Macroeconomic Factor Engine

**Status**: Draft
**Priority**: High
**Depends on**: None (foundational)
**Extends**: `datasynth-core/src/distributions/drift.rs`, `market_drift.rs`

---

## 1. Problem Statement

The current economic cycle model uses a simple sinusoidal function with fixed amplitude
and period. Real macroeconomic dynamics involve multiple correlated factors (GDP growth,
interest rates, inflation, unemployment, credit conditions) that co-evolve through
complex nonlinear relationships. The absence of these dynamics means generated data
lacks realistic cross-variable coherence — e.g., a recession scenario should
simultaneously show declining revenues, rising credit losses, tightening payment terms,
and falling investment, but currently each is modeled independently.

## 2. Scientific Foundation

### 2.1 Vector Autoregression (VAR) Models

The gold standard for modeling interdependent macroeconomic time series (Sims, 1980).
A VAR(p) model captures how each variable depends on its own lags and the lags of all
other variables:

```
Y_t = c + A_1·Y_{t-1} + A_2·Y_{t-2} + ... + A_p·Y_{t-p} + ε_t
```

Where:
- `Y_t` = vector of macro factors at time t (GDP, rates, inflation, unemployment, credit spread)
- `A_i` = coefficient matrices capturing cross-variable dynamics
- `ε_t` ~ N(0, Σ) = correlated innovation vector

**Why VAR for synthetic data**: Pre-calibrated coefficient matrices from historical data
produce realistic co-movement without requiring the full complexity of DSGE models.

**Reference**: Sims, C.A. (1980). "Macroeconomics and Reality." *Econometrica*, 48(1), 1-48.

### 2.2 Regime-Switching Models

Hamilton's (1989) Markov-switching model allows the economy to transition between
discrete states (expansion, contraction) with state-dependent dynamics:

```
Y_t = μ_{S_t} + Φ_{S_t}·Y_{t-1} + σ_{S_t}·ε_t
P(S_t = j | S_{t-1} = i) = p_{ij}
```

The transition matrix governs the average duration and frequency of recessions.

**Calibration from NBER data** (1945-2024):
- Average expansion: 64.2 months
- Average contraction: 11.1 months
- P(expansion→recession) ≈ 0.016/month
- P(recession→expansion) ≈ 0.090/month

**Reference**: Hamilton, J.D. (1989). "A New Approach to the Economic Analysis of
Nonstationary Time Series." *Econometrica*, 57(2), 357-384.

### 2.3 Interest Rate Term Structure

The **Nelson-Siegel (1987)** model parameterizes the yield curve with three factors:

```
y(τ) = β_0 + β_1·[(1 - e^{-τ/λ}) / (τ/λ)]
          + β_2·[(1 - e^{-τ/λ}) / (τ/λ) - e^{-τ/λ}]
```

Where:
- `β_0` = long-term level (influenced by inflation expectations)
- `β_1` = slope (short vs. long rates, linked to monetary policy)
- `β_2` = curvature (hump shape)
- `λ` = decay parameter (controls where curvature peaks)
- `τ` = maturity

For dynamic simulation, use the **Diebold-Li (2006)** extension where β parameters
follow AR(1) processes linked to the macro factor engine.

**Reference**: Nelson, C.R. & Siegel, A.F. (1987). "Parsimonious Modeling of Yield
Curves." *Journal of Business*, 60(4), 473-489.

### 2.4 Credit Cycle Dynamics

Credit conditions follow the economic cycle with a lag. The **Merton (1974)** structural
model links default probability to asset values:

```
PD = Φ(-DD)  where  DD = (ln(V/D) + (μ - σ²/2)·T) / (σ·√T)
```

For synthetic data, we use **credit rating transition matrices** (Moody's methodology)
that vary by economic regime:

- Expansion: Upgrade probability ~5.8%, downgrade ~4.2% annually
- Contraction: Upgrade probability ~2.1%, downgrade ~9.7% annually

**Reference**: Merton, R.C. (1974). "On the Pricing of Corporate Debt." *Journal of
Finance*, 29(2), 449-470.

### 2.5 Phillips Curve — Inflation-Unemployment Tradeoff

The New Keynesian Phillips Curve relates inflation to economic slack:

```
π_t = β·E[π_{t+1}] + κ·(y_t - y*_t) + ε_t
```

Where `κ` (slope) varies by era — flatter since the 1990s (Hazell et al., 2022).
For synthetic data, this ensures inflation and unemployment move coherently.

**Reference**: Hazell, J., Herreño, J., Nakamura, E., & Steinsson, J. (2022). "The
Slope of the Phillips Curve." *Quarterly Journal of Economics*, 137(3), 1299-1344.

## 3. Proposed Design

### 3.1 Core Data Structures

```rust
/// Macroeconomic factor time series for a single period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroFactors {
    /// Real GDP growth rate (annualized, e.g., 0.025 = 2.5%)
    pub gdp_growth: f64,
    /// Policy/short-term interest rate (e.g., 0.05 = 5.0%)
    pub policy_rate: f64,
    /// Consumer price inflation (annualized, e.g., 0.03 = 3.0%)
    pub cpi_inflation: f64,
    /// Producer price inflation (annualized)
    pub ppi_inflation: f64,
    /// Unemployment rate (e.g., 0.045 = 4.5%)
    pub unemployment_rate: f64,
    /// Credit spread over risk-free rate (bps, e.g., 150 = 1.5%)
    pub credit_spread_bps: f64,
    /// Consumer confidence index (normalized 0-100)
    pub consumer_confidence: f64,
    /// Economic regime
    pub regime: EconomicRegime,
    /// Yield curve parameters (Nelson-Siegel)
    pub yield_curve: YieldCurveParams,
    /// Sector-specific multipliers
    pub sector_factors: HashMap<IndustrySector, SectorMacroState>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EconomicRegime {
    /// Steady growth, low volatility
    Expansion,
    /// Slowing growth, rising uncertainty
    LateExpansion,
    /// Negative growth, elevated defaults
    Contraction,
    /// Bottoming out, early recovery signals
    Trough,
    /// Rapid growth, possible overheating
    EarlyRecovery,
    /// High inflation with stagnant growth
    Stagflation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldCurveParams {
    /// Long-term level (β₀)
    pub level: f64,
    /// Slope factor (β₁) — negative means normal curve
    pub slope: f64,
    /// Curvature factor (β₂)
    pub curvature: f64,
    /// Decay parameter (λ)
    pub decay: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorMacroState {
    /// Sector-specific GDP sensitivity (β coefficient)
    pub gdp_beta: f64,
    /// Sector-specific inflation pass-through rate
    pub inflation_passthrough: f64,
    /// Sector credit spread addon (bps)
    pub credit_spread_addon_bps: f64,
    /// Sector employment sensitivity
    pub employment_beta: f64,
    /// Current sector growth differential vs. economy
    pub growth_differential: f64,
}
```

### 3.2 Macroeconomic Factor Engine

```rust
pub struct MacroeconomicEngine {
    /// Pre-computed time series for the full generation period
    factors: Vec<MacroFactors>,
    /// Model type used for generation
    model: MacroModel,
    /// RNG for stochastic components
    rng: ChaCha8Rng,
}

pub enum MacroModel {
    /// Independent factor paths (simple, existing behavior)
    Independent(IndependentConfig),
    /// VAR-based correlated factors
    Var(VarConfig),
    /// Regime-switching with state-dependent dynamics
    MarkovSwitching(MarkovSwitchingConfig),
    /// Driven by a scenario definition
    ScenarioDriven(ScenarioPath),
}
```

### 3.3 VAR Model Configuration

```rust
pub struct VarConfig {
    /// Number of lags (typically 1-4 for monthly data)
    pub lags: usize,
    /// Coefficient matrices A_1 ... A_p (5×5 for 5 factors)
    /// Pre-calibrated from historical data
    pub coefficients: Vec<Array2<f64>>,
    /// Innovation covariance matrix Σ
    pub innovation_covariance: Array2<f64>,
    /// Initial conditions (starting values)
    pub initial_values: Vec<MacroFactors>,
    /// Optional regime-switching overlay
    pub regime_switching: Option<RegimeSwitchingOverlay>,
}
```

### 3.4 Pre-Calibrated Parameter Sets

The engine ships with pre-calibrated VAR parameters for different economic eras:

| Parameter Set | Period | Characteristics |
|--------------|--------|-----------------|
| `post_war_stable` | 1950-1970 | Low rates, stable growth, low inflation |
| `great_inflation` | 1970-1982 | High inflation, volatile rates, stagflation |
| `great_moderation` | 1983-2007 | Declining rates, stable growth, low vol |
| `gfc_crisis` | 2007-2009 | Credit crisis, near-zero rates, deflation risk |
| `zirp_era` | 2010-2021 | Ultra-low rates, QE, asset inflation |
| `post_covid_inflation` | 2021-2024 | Supply shocks, rate hikes, labor shortage |
| `neutral_baseline` | N/A | Stylized steady-state parameters |

### 3.5 Sector Sensitivity Profiles

Empirical GDP betas by industry (sourced from economic literature):

| Sector | GDP β | Inflation Pass-through | Interest Rate Sensitivity | Employment β |
|--------|-------|----------------------|--------------------------|-------------|
| Technology | 1.4 | 0.30 | 0.8 | 0.9 |
| Retail | 1.2 | 0.75 | 0.6 | 1.1 |
| Manufacturing | 1.5 | 0.85 | 1.0 | 1.3 |
| Financial Services | 1.8 | 0.20 | 2.0 | 0.7 |
| Healthcare | 0.6 | 0.50 | 0.4 | 0.3 |
| Energy | 1.3 | 0.90 | 0.7 | 0.8 |
| Real Estate | 1.6 | 0.60 | 2.5 | 1.0 |
| Utilities | 0.4 | 0.70 | 1.5 | 0.2 |

### 3.6 Integration with Existing Pipeline

The `MacroFactors` for each period feed into the existing `DriftContext`:

```rust
impl MacroeconomicEngine {
    /// Get factor-adjusted drift multipliers for a given period
    pub fn get_adjustments(&self, period: usize, sector: IndustrySector) -> DriftAdjustments {
        let factors = &self.factors[period];
        let sector_state = factors.sector_factors.get(&sector);

        DriftAdjustments {
            amount_multiplier: self.compute_amount_multiplier(factors, sector_state),
            volume_multiplier: self.compute_volume_multiplier(factors, sector_state),
            anomaly_rate_multiplier: self.compute_anomaly_multiplier(factors),
            concept_drift_factor: self.compute_concept_drift(factors),
            // New fields
            credit_risk_multiplier: self.compute_credit_risk(factors, sector_state),
            payment_delay_multiplier: self.compute_payment_delay(factors),
            pricing_multiplier: self.compute_pricing(factors, sector_state),
        }
    }

    /// Get the yield for a given maturity at a given period
    pub fn yield_at(&self, period: usize, maturity_years: f64) -> f64 {
        let yc = &self.factors[period].yield_curve;
        let tau = maturity_years;
        let decay_factor = (1.0 - (-tau / yc.decay).exp()) / (tau / yc.decay);
        yc.level + yc.slope * decay_factor
            + yc.curvature * (decay_factor - (-tau / yc.decay).exp())
    }
}
```

## 4. Configuration Schema

```yaml
external_realism:
  macroeconomic:
    enabled: true
    model: var                        # independent | var | markov_switching | scenario_driven

    # Initial conditions
    initial_state:
      gdp_growth: 0.025
      policy_rate: 0.05
      cpi_inflation: 0.03
      ppi_inflation: 0.035
      unemployment_rate: 0.04
      credit_spread_bps: 150
      consumer_confidence: 65.0

    # VAR model settings
    var:
      lags: 2
      parameter_set: great_moderation   # Use pre-calibrated parameters
      custom_coefficients: null          # Or provide custom A matrices
      innovation_scale: 1.0             # Scale shock volatility (1.0 = historical)

    # Regime switching overlay
    regime_switching:
      enabled: true
      transition_matrix:                # Monthly transition probabilities
        expansion_to_contraction: 0.016
        contraction_to_expansion: 0.090
        expansion_to_stagflation: 0.004
        stagflation_to_contraction: 0.050
      initial_regime: expansion

    # Yield curve dynamics
    yield_curve:
      enabled: true
      model: nelson_siegel              # nelson_siegel | flat | custom
      initial_params:
        level: 0.04                     # β₀ — long-term rate
        slope: -0.02                    # β₁ — term premium
        curvature: 0.01                 # β₂ — hump
        decay: 1.5                      # λ

    # Credit cycle
    credit_cycle:
      enabled: true
      base_default_rate: 0.02           # Annual default rate in expansion
      recession_multiplier: 3.5         # Default rate multiplier in contraction
      credit_lag_months: 6              # Credit deterioration lags GDP
      recovery_rate_expansion: 0.45     # LGD = 1 - recovery
      recovery_rate_contraction: 0.25

    # Sector sensitivities
    sector_sensitivities:
      profile: default                  # default | custom
      custom: {}                        # Override specific sectors

    # Output options
    export_factor_series: true          # Export macro factor time series as CSV
    export_yield_curves: true           # Export yield curve snapshots
```

## 5. Downstream Effects Mapping

Each macro factor drives specific generator behaviors:

| Macro Factor | Affected Generator | Effect |
|-------------|-------------------|--------|
| GDP growth | je_generator | Transaction volume scaling |
| GDP growth | o2c_generator | Sales order frequency & size |
| GDP growth | p2p_generator | Purchase order volumes |
| Policy rate | treasury | Borrowing costs, deposit income |
| Policy rate | banking | Loan pricing, NIM |
| CPI inflation | vendor pricing | Invoice amounts, COGS |
| PPI inflation | manufacturing | Raw material costs |
| Unemployment | hr/payroll | Hiring rate, turnover, wage growth |
| Credit spread | banking | Loan loss provisions |
| Credit spread | customer behavior | Payment delays, defaults |
| Consumer confidence | o2c_generator | Demand elasticity |
| Yield curve | treasury | Bond portfolio valuation |
| Regime | anomaly/injector | Anomaly rate, fraud patterns |

## 6. Testing Strategy

### Unit Tests
- VAR coefficient matrix stability (eigenvalues within unit circle)
- Nelson-Siegel yield curve produces reasonable shapes (normal, inverted, flat)
- Regime transitions respect NBER-calibrated durations
- Sector beta multipliers produce bounded adjustments

### Integration Tests
- Full-period macro factor generation with VAR model
- Factor series coherence (inflation up → rates up with lag)
- Sector divergence (healthcare GDP β < manufacturing GDP β)
- Yield curve dynamics respond to policy rate changes

### Statistical Validation
- Generated GDP growth distribution matches empirical moments
- Autocorrelation structure of factors matches historical patterns
- Recession frequency within expected NBER bounds
- Cross-factor correlations within ±0.1 of target correlation matrix

## 7. Performance Considerations

- **Initialization**: VAR simulation for N periods is O(N·p·k²) where p = lags, k = factors
  - For 120 months, 2 lags, 5 factors: ~2,400 operations (negligible)
- **Per-record lookup**: O(1) — factors are pre-computed per period
- **Memory**: ~200 bytes per period × 120 periods = ~24KB (negligible)
- Yield curve evaluation: 3 exp() calls per maturity query

## 8. Migration Path

1. **Phase 1**: Add `MacroeconomicEngine` as optional component in `GenerationOrchestrator`
2. **Phase 2**: Wire `MacroFactors` into `DriftContext` alongside existing drift models
3. **Phase 3**: Provide pre-calibrated VAR parameter sets
4. **Phase 4**: Add yield curve to treasury and banking generators

No breaking changes — existing configs without `macroeconomic` section use current behavior.

## References

1. Sims, C.A. (1980). "Macroeconomics and Reality." *Econometrica*, 48(1), 1-48.
2. Hamilton, J.D. (1989). "A New Approach to the Economic Analysis of Nonstationary Time Series." *Econometrica*, 57(2), 357-384.
3. Nelson, C.R. & Siegel, A.F. (1987). "Parsimonious Modeling of Yield Curves." *Journal of Business*, 60(4), 473-489.
4. Diebold, F.X. & Li, C. (2006). "Forecasting the Term Structure of Government Bond Yields." *Journal of Econometrics*, 130(2), 337-364.
5. Merton, R.C. (1974). "On the Pricing of Corporate Debt." *Journal of Finance*, 29(2), 449-470.
6. Hazell, J., Herreño, J., Nakamura, E., & Steinsson, J. (2022). "The Slope of the Phillips Curve." *Quarterly Journal of Economics*, 137(3), 1299-1344.
7. Lütkepohl, H. (2005). *New Introduction to Multiple Time Series Analysis*. Springer.
8. Stock, J.H. & Watson, M.W. (2001). "Vector Autoregressions." *Journal of Economic Perspectives*, 15(4), 101-115.
