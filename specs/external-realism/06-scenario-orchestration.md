# Spec 06: Scenario Orchestration & Stress Testing Framework

**Status**: Draft
**Priority**: High
**Depends on**: Specs 01-05 (all factor/shock modules)
**Extends**: New top-level orchestration layer

---

## 1. Problem Statement

Individual factor models (macro, shocks, geopolitical, climate, supply chain) are
powerful in isolation, but their real value emerges when **orchestrated together** into
coherent scenarios. Users need the ability to say "generate data that looks like the
2008 financial crisis" or "stress test my ML model against a CCAR severely adverse
scenario" without manually configuring dozens of parameters. Currently, there is no
scenario abstraction layer that bundles multiple external factors into reproducible,
named, internally consistent configurations.

## 2. Scientific Foundation

### 2.1 Regulatory Stress Test Frameworks

#### CCAR/DFAST (U.S. Federal Reserve)

The Fed publishes annual stress test scenarios with specific macroeconomic paths:

**2025 Severely Adverse Scenario Parameters**:
- Unemployment: +5.9pp (peak ≥10%)
- Commercial real estate price decline: -30%
- House prices: -33%
- Equity markets: -50%
- Real GDP: -7.8%
- VIX peak: 65
- 10-year Treasury: decline to ~1.5%

The scenarios cover a 3-year projection horizon with quarterly granularity. A
**Global Market Shock (GMS)** component applies instantaneous risk factor shocks
to trading book positions.

The Fed ensures internal consistency via a small-scale macroeconomic model incorporating
a **Phillips Curve** (inflation given unemployment) and a **Taylor Rule** (short-term
rates given inflation and output gap).

**Reference**: Federal Reserve Board (2025). "2025 Supervisory Stress Test Scenarios."

#### EBA EU-Wide Stress Test

The EBA uses a **constrained bottom-up** approach:
- GDP decline: -6.3% cumulative over 3 years (2025 adverse)
- 64 banks covering ~75% of EU banking assets
- Variables: GDP, inflation, unemployment, real estate, stocks, FX, rates, and GVA
  across 16 economic sectors
- "No policy change" convention (no reactive monetary/fiscal policy)

**Reference**: EBA (2025). "EU-Wide Stress Test Results."

#### Reverse Stress Testing

Starts from a predetermined failure point (e.g., capital ratio breach) and works
backward to identify triggering scenarios:
- **Consolidated Distance to Breakpoint (CDBP)**: NPL increase needed to deplete
  capital buffers (Feyen & Mare, World Bank, 2021)
- **Multi-scenario stochastic**: Distribution of outcomes across many scenarios
  identifies worst-case tail events (Budnik et al., ECB, 2024)

**Reference**: Feyen, E. & Mare, D. (2021). "Measuring Systemic Banking Resilience."
World Bank WP 9864.

### 2.2 Historical Crisis Fingerprints

Each historical crisis has a distinct "financial fingerprint" — a characteristic
pattern of correlated changes across financial dimensions:

| Crisis | GDP Impact | Duration | Key Feature | Recovery Shape |
|--------|-----------|----------|-------------|---------------|
| 2008 GFC | -4.3% (US) | 18 months | Credit freeze, housing collapse | U-shaped |
| 2020 COVID | -9% Q2 (US) | 2 months | Sudden stop, K-shaped divergence | K-shaped |
| 2010 Euro Debt | -4.5% (periphery) | 3+ years | Sovereign spreads, bank contagion | L-shaped |
| 1997 Asian Crisis | -13% (Indonesia) | 18 months | Currency collapse, capital flight | V-shaped (most) |
| 2001 Dot-com | -0.3% (US) | 8 months | Tech valuation collapse | Swoosh |
| 1970s Stagflation | Stagnant | ~8 years | Oil shock, wage-price spiral | L-shaped |
| 2022 Rate Shock | Resilient | Ongoing | Fastest rate hikes in 40 years | V-shaped |

### 2.3 Monte Carlo Scenario Generation

For probabilistic stress testing, large ensembles of scenarios are generated:

1. **Classical Monte Carlo**: Sample from calibrated distributions, compute metrics
   per scenario (VaR, Expected Shortfall)
2. **Filtered Historical Simulation**: Resample historical innovations with current
   volatility scaling
3. **Factor-based**: Sample common factors, derive scenario paths via loadings

Typical ensemble sizes: 1,000-10,000 scenarios for distributional coverage.

**Reference**: Glasserman, P. (2004). *Monte Carlo Methods in Financial Engineering*.
Springer.

## 3. Proposed Design

### 3.1 Scenario Definition

```rust
/// A complete scenario definition — bundles all external factor configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Unique scenario identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description / narrative
    pub description: String,
    /// Category for organization
    pub category: ScenarioCategory,
    /// Severity level
    pub severity: ScenarioSeverity,
    /// Duration of the scenario (periods)
    pub duration_periods: usize,
    /// Macroeconomic factor path (Spec 01)
    pub macro_path: MacroScenarioPath,
    /// External shocks to inject (Spec 02)
    pub shocks: Vec<ExternalShock>,
    /// Geopolitical events (Spec 03)
    pub geopolitical_events: Vec<GeopoliticalEvent>,
    /// Regulatory changes (Spec 03)
    pub regulatory_changes: Vec<RegulatoryChange>,
    /// Natural disasters (Spec 04)
    pub natural_disasters: Vec<NaturalDisasterEvent>,
    /// Climate scenario (Spec 04)
    pub climate: Option<ClimateTransitionModel>,
    /// Supply chain disruptions (Spec 05)
    pub supply_chain_shocks: Vec<SupplyChainShockDef>,
    /// Behavioral overrides
    pub behavioral_overrides: Option<BehavioralOverrides>,
    /// Metadata
    pub metadata: ScenarioMetadata,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScenarioCategory {
    /// Based on a historical crisis
    HistoricalReplay,
    /// Regulatory stress test (CCAR, EBA, etc.)
    RegulatoryStressTest,
    /// Climate/ESG scenario (NGFS, TCFD)
    ClimateScenario,
    /// Geopolitical scenario
    GeopoliticalScenario,
    /// Pandemic / public health
    PandemicScenario,
    /// Custom user-defined
    Custom,
    /// Baseline / business-as-usual
    Baseline,
    /// Monte Carlo ensemble member
    MonteCarloSample,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScenarioSeverity {
    /// Normal operating conditions
    Baseline,
    /// Mild stress (1-in-10 year event)
    Mild,
    /// Moderate stress (1-in-25 year event)
    Moderate,
    /// Severe stress (1-in-100 year event, typical of CCAR adverse)
    Severe,
    /// Extreme / tail event (1-in-200+ year)
    Extreme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroScenarioPath {
    /// Either use the macro engine (Spec 01) or provide explicit paths
    pub source: MacroPathSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroPathSource {
    /// Generate using the macro engine with given initial conditions
    Engine {
        model: MacroModel,
        initial_regime: EconomicRegime,
    },
    /// Use pre-defined quarterly paths for key variables
    ExplicitPath {
        gdp_growth: Vec<f64>,
        policy_rate: Vec<f64>,
        inflation: Vec<f64>,
        unemployment: Vec<f64>,
        credit_spread_bps: Vec<f64>,
    },
    /// Scale from a named parameter set
    ParameterSet {
        base: String, // "great_moderation", "gfc_crisis", etc.
        severity_scale: f64,
    },
}
```

### 3.2 Scenario Library

```rust
pub struct ScenarioLibrary {
    /// Built-in scenarios
    builtin: HashMap<String, Scenario>,
    /// User-defined scenarios
    custom: HashMap<String, Scenario>,
}

impl ScenarioLibrary {
    pub fn new() -> Self {
        let mut lib = Self::default();
        lib.register_builtin("baseline", Self::baseline());
        lib.register_builtin("2008_financial_crisis", Self::gfc_2008());
        lib.register_builtin("2020_covid_pandemic", Self::covid_2020());
        lib.register_builtin("2010_eurozone_crisis", Self::eurozone_2010());
        lib.register_builtin("1970s_stagflation", Self::stagflation_1970s());
        lib.register_builtin("2022_rate_shock", Self::rate_shock_2022());
        lib.register_builtin("ccar_severely_adverse", Self::ccar_severely_adverse());
        lib.register_builtin("eba_adverse_2025", Self::eba_adverse_2025());
        lib.register_builtin("ngfs_net_zero_2050", Self::ngfs_net_zero());
        lib.register_builtin("ngfs_delayed_transition", Self::ngfs_delayed());
        lib.register_builtin("ngfs_hot_house", Self::ngfs_hot_house());
        lib.register_builtin("trade_war_escalation", Self::trade_war());
        lib.register_builtin("sanctions_regime", Self::sanctions_regime());
        lib.register_builtin("pandemic_severe", Self::pandemic_severe());
        lib.register_builtin("supply_chain_crisis", Self::supply_chain_crisis());
        lib
    }
}
```

### 3.3 Pre-Built Scenario: 2008 Global Financial Crisis

```rust
impl ScenarioLibrary {
    fn gfc_2008() -> Scenario {
        Scenario {
            id: "2008_financial_crisis".into(),
            name: "2008 Global Financial Crisis".into(),
            description: "Subprime mortgage crisis → credit freeze → global recession. \
                          Calibrated to replicate the financial fingerprint of 2007-2009.".into(),
            category: ScenarioCategory::HistoricalReplay,
            severity: ScenarioSeverity::Severe,
            duration_periods: 36, // 3-year scenario
            macro_path: MacroScenarioPath {
                source: MacroPathSource::ExplicitPath {
                    // Quarterly GDP growth (annualized)
                    gdp_growth: vec![
                        0.01, -0.01, -0.02, -0.04,   // 2007: gradual slowdown
                        -0.06, -0.08, -0.04, -0.02,   // 2008: deep contraction
                        0.00, 0.01, 0.02, 0.025,      // 2009: slow recovery
                    ],
                    policy_rate: vec![
                        0.0525, 0.0450, 0.0375, 0.0200,
                        0.0100, 0.0025, 0.0025, 0.0025,
                        0.0025, 0.0025, 0.0025, 0.0025,
                    ],
                    inflation: vec![
                        0.028, 0.025, 0.020, 0.010,
                        0.005, -0.005, 0.000, 0.010,
                        0.015, 0.018, 0.020, 0.020,
                    ],
                    unemployment: vec![
                        0.046, 0.048, 0.052, 0.060,
                        0.070, 0.085, 0.095, 0.100,
                        0.098, 0.095, 0.092, 0.088,
                    ],
                    credit_spread_bps: vec![
                        80.0, 120.0, 200.0, 400.0,
                        600.0, 800.0, 500.0, 350.0,
                        250.0, 200.0, 180.0, 150.0,
                    ],
                },
            },
            shocks: vec![
                // Credit crunch shock
                ExternalShock {
                    id: "gfc_credit_crunch".into(),
                    shock_type: ExternalShockType::Macroeconomic(
                        MacroShockVariant::CreditCrunch {
                            spread_widening_bps: 600,
                        }
                    ),
                    onset_period: 4,
                    lifecycle: ShockLifecycle {
                        warning_periods: 2,
                        ramp_up_periods: 4,
                        peak_duration_periods: 6,
                        recovery_periods: 18,
                        recovery_target: 0.90,
                        current_phase: ShockPhase::Warning,
                    },
                    impact_channels: vec![
                        ImpactChannel {
                            channel: ChannelType::DefaultProbability,
                            peak_magnitude: 3.5,
                            lag_periods: 2,
                            scope: ImpactScope::Universal,
                        },
                        ImpactChannel {
                            channel: ChannelType::TransactionVolume,
                            peak_magnitude: 0.65,
                            lag_periods: 1,
                            scope: ImpactScope::Universal,
                        },
                        ImpactChannel {
                            channel: ChannelType::PaymentDelay,
                            peak_magnitude: 1.8,
                            lag_periods: 2,
                            scope: ImpactScope::Universal,
                        },
                    ],
                    recovery: RecoveryCurve {
                        shape: RecoveryShape::U,
                        divergent_sectors: None,
                    },
                    contagion: Some(ContagionConfig { /* ... */ }),
                    description: "Credit market freeze".into(),
                },
                // Stock market crash
                ExternalShock {
                    id: "gfc_equity_crash".into(),
                    shock_type: ExternalShockType::Macroeconomic(
                        MacroShockVariant::StockMarketCrash {
                            drawdown_pct: 0.50,
                        }
                    ),
                    onset_period: 6,
                    /* ... */
                },
            ],
            /* ... remaining fields */
        }
    }
}
```

### 3.4 Scenario Orchestrator

```rust
/// Top-level orchestrator that coordinates all external factor modules
pub struct ScenarioOrchestrator {
    /// Active scenario
    scenario: Scenario,
    /// Macroeconomic engine (Spec 01)
    macro_engine: MacroeconomicEngine,
    /// Shock sequencer (Spec 02)
    shock_sequencer: ShockSequencer,
    /// Geopolitical module (Spec 03)
    geopolitical: GeopoliticalModule,
    /// Climate/disaster module (Spec 04)
    climate: ClimateModule,
    /// Supply chain network (Spec 05)
    supply_chain: Option<SupplyChainNetwork>,
    /// Cross-factor correlation engine (Spec 07)
    correlation_engine: CrossFactorEngine,
}

impl ScenarioOrchestrator {
    /// Initialize from a scenario definition
    pub fn from_scenario(
        scenario: Scenario,
        rng_seed: u64,
    ) -> Result<Self, ScenarioError> {
        // Validate scenario coherence
        Self::validate_coherence(&scenario)?;

        // Initialize each sub-module
        let macro_engine = MacroeconomicEngine::from_path(
            &scenario.macro_path,
            scenario.duration_periods,
            rng_seed,
        )?;

        let shock_sequencer = ShockSequencer::new(
            scenario.shocks.clone(),
            ShockInteraction::Saturating,
        );

        // ... initialize other modules

        Ok(Self { scenario, macro_engine, shock_sequencer, /* ... */ })
    }

    /// Compute all external adjustments for a given period
    pub fn adjustments_at(
        &mut self,
        period: usize,
        sector: &IndustrySector,
        entity_id: &str,
    ) -> ExternalAdjustments {
        // 1. Macro factors
        let macro_adj = self.macro_engine.get_adjustments(period, *sector);

        // 2. Shock effects
        let shock_adj = self.shock_sequencer.combined_adjustments(period);

        // 3. Geopolitical uncertainty
        let geo_adj = self.geopolitical.adjustments_at(period, sector);

        // 4. Climate/disaster effects
        let climate_adj = self.climate.adjustments_at(period, sector);

        // 5. Supply chain effects
        let sc_adj = self.supply_chain.as_ref()
            .map(|sc| sc.adjustments_for(entity_id, period))
            .unwrap_or_default();

        // 6. Apply cross-factor correlation adjustments
        let raw = ExternalAdjustments::combine(
            macro_adj, shock_adj, geo_adj, climate_adj, sc_adj,
        );

        self.correlation_engine.ensure_coherence(raw, period)
    }
}

/// Combined adjustments from all external factor modules
#[derive(Debug, Clone, Default)]
pub struct ExternalAdjustments {
    /// Volume multipliers
    pub transaction_volume: f64,
    pub sales_volume: f64,
    pub procurement_volume: f64,
    pub production_volume: f64,
    pub hiring_rate: f64,

    /// Amount multipliers
    pub revenue_per_unit: f64,
    pub cogs: f64,
    pub opex: f64,
    pub wage_growth: f64,

    /// Timing multipliers
    pub payment_delay: f64,
    pub lead_time: f64,
    pub collection_period: f64,

    /// Risk multipliers
    pub default_probability: f64,
    pub fraud_rate: f64,
    pub audit_finding_rate: f64,
    pub control_effectiveness: f64,

    /// Financial multipliers
    pub interest_expense: f64,
    pub fx_volatility: f64,
    pub impairment_charges: f64,

    /// Economic context
    pub economic_regime: EconomicRegime,
    pub yield_curve: Option<YieldCurveParams>,
    pub carbon_price: Option<f64>,
    pub uncertainty_level: f64,
}
```

### 3.5 Monte Carlo Ensemble Generation

```rust
pub struct MonteCarloGenerator {
    /// Base scenario to perturb
    base_scenario: Scenario,
    /// Number of scenarios to generate
    n_scenarios: usize,
    /// Perturbation parameters
    perturbation: PerturbationConfig,
    /// RNG
    rng: ChaCha8Rng,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerturbationConfig {
    /// Scale of macro factor perturbations (fraction of historical volatility)
    pub macro_perturbation_scale: f64,
    /// Whether to randomize shock timing
    pub randomize_shock_timing: bool,
    /// Timing jitter (periods)
    pub timing_jitter: usize,
    /// Whether to randomize shock severity
    pub randomize_shock_severity: bool,
    /// Severity jitter (fraction of configured severity)
    pub severity_jitter: f64,
}

impl MonteCarloGenerator {
    /// Generate an ensemble of scenario variants
    pub fn generate_ensemble(&mut self) -> Vec<Scenario> {
        (0..self.n_scenarios)
            .map(|i| {
                let mut scenario = self.base_scenario.clone();
                scenario.id = format!("{}__mc_{}", self.base_scenario.id, i);
                scenario.category = ScenarioCategory::MonteCarloSample;

                // Perturb macro path
                if let MacroPathSource::ExplicitPath { ref mut gdp_growth, .. } =
                    scenario.macro_path.source
                {
                    for v in gdp_growth.iter_mut() {
                        *v += self.rng.gen::<f64>() * self.perturbation.macro_perturbation_scale * 0.01;
                    }
                }

                // Perturb shock timing
                if self.perturbation.randomize_shock_timing {
                    for shock in &mut scenario.shocks {
                        let jitter = self.rng.gen_range(
                            -(self.perturbation.timing_jitter as i32)
                            ..=(self.perturbation.timing_jitter as i32)
                        );
                        shock.onset_period = (shock.onset_period as i32 + jitter).max(0) as usize;
                    }
                }

                scenario
            })
            .collect()
    }
}
```

## 4. Configuration Schema

```yaml
external_realism:
  scenarios:
    enabled: true

    # Use a pre-built scenario
    preset: "2008_financial_crisis"       # Name from scenario library

    # OR define a custom scenario
    custom:
      id: "my_scenario"
      name: "Custom Downturn with Trade War"
      category: custom
      severity: moderate
      duration_periods: 24
      # Reference other spec configs inline
      macro_path:
        source: parameter_set
        base: "great_moderation"
        severity_scale: 1.5
      shocks:
        - $ref: "#/external_shocks/shocks/0"   # Reference shock definitions
      geopolitical:
        - $ref: "#/geopolitical/events/0"

    # Monte Carlo ensemble
    monte_carlo:
      enabled: false
      base_scenario: "ccar_severely_adverse"
      n_scenarios: 100
      perturbation:
        macro_perturbation_scale: 0.5
        randomize_shock_timing: true
        timing_jitter: 2
        randomize_shock_severity: true
        severity_jitter: 0.2

    # Scenario comparison mode
    comparison:
      enabled: false
      scenarios:
        - "baseline"
        - "ccar_severely_adverse"
        - "ngfs_net_zero_2050"
      # Generate data for each scenario with same seed for comparison
      shared_seed: true

    # Output
    export_scenario_metadata: true
    export_adjustments_timeline: true      # Period-by-period adjustment factors
```

## 5. Scenario Library Catalog

### 5.1 Historical Replays

| Scenario ID | Period | Severity | Key Factors |
|------------|--------|----------|-------------|
| `2008_financial_crisis` | 36mo | Severe | Credit crunch, housing crash, equity -50% |
| `2020_covid_pandemic` | 24mo | Severe | Lockdown, K-recovery, digital acceleration |
| `2010_eurozone_crisis` | 48mo | Severe | Sovereign spreads, bank contagion, austerity |
| `1997_asian_crisis` | 18mo | Severe | Currency collapse, capital flight |
| `2001_dotcom_bust` | 18mo | Moderate | Tech valuation crash, VC freeze |
| `1970s_stagflation` | 96mo | Severe | Oil shock, wage-price spiral, high inflation |
| `2022_rate_shock` | 24mo | Moderate | Rapid rate hikes, bond losses, bank stress |
| `2021_supply_chain_crisis` | 30mo | Moderate | Chip shortage, logistics bottleneck |

### 5.2 Regulatory Stress Tests

| Scenario ID | Framework | Severity | Horizon |
|------------|-----------|----------|---------|
| `ccar_severely_adverse` | CCAR/DFAST | Severe | 9 quarters |
| `ccar_adverse` | CCAR/DFAST | Moderate | 9 quarters |
| `eba_adverse_2025` | EBA | Severe | 3 years |
| `boe_exploratory_2024` | BoE | Severe | 5 years |

### 5.3 Climate / ESG Scenarios

| Scenario ID | Framework | Temp. Target | Horizon |
|------------|-----------|-------------|---------|
| `ngfs_net_zero_2050` | NGFS Orderly | 1.5°C | 120mo |
| `ngfs_below_2c` | NGFS Orderly | <2°C | 120mo |
| `ngfs_delayed_transition` | NGFS Disorderly | <2°C | 120mo |
| `ngfs_hot_house` | NGFS Hot House | >3°C | 120mo |

### 5.4 Geopolitical

| Scenario ID | Type | Severity | Duration |
|------------|------|----------|----------|
| `trade_war_escalation` | Trade policy | Moderate | 36mo |
| `sanctions_regime` | Sanctions | Severe | 48mo |
| `regional_conflict` | Armed conflict | Severe | 24mo |

## 6. Integration Architecture

```
                    ┌─────────────────────┐
                    │   User Config YAML  │
                    │   scenarios:        │
                    │     preset: "..."   │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │  Scenario Library    │
                    │  (resolve preset    │
                    │   or custom def)    │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │ Scenario Orchestrator│
                    │                     │
                    │  Pre-compute all    │
                    │  factor time series │
                    │  for full duration  │
                    └─────────┬───────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
    ┌─────────▼──┐  ┌────────▼───┐  ┌────────▼────────┐
    │ Macro      │  │ Shock      │  │ Supply Chain    │
    │ Engine     │  │ Sequencer  │  │ Network         │
    └─────────┬──┘  └────────┬───┘  └────────┬────────┘
              └───────────────┼───────────────┘
                              │
                    ┌─────────▼───────────┐
                    │ ExternalAdjustments  │
                    │ (per period)         │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │ GenerationOrchest.   │
                    │ (existing pipeline)  │
                    │                     │
                    │ Generators use      │
                    │ adjustments to      │
                    │ modify output       │
                    └─────────────────────┘
```

## 7. Scenario Comparison Output

When `comparison.enabled = true`, the orchestrator generates separate output
directories for each scenario, enabling direct A/B comparison:

```
output/
  baseline/
    journal_entries.csv
    vendors.csv
    ...
  ccar_severely_adverse/
    journal_entries.csv
    vendors.csv
    ...
  ngfs_net_zero_2050/
    journal_entries.csv
    vendors.csv
    ...
  _comparison/
    scenario_metadata.json
    adjustment_timelines.csv       # Side-by-side factor values
    divergence_metrics.json        # Statistical divergence between scenarios
```

## 8. Testing Strategy

- **Scenario loading**: All built-in scenarios parse without error
- **Coherence validation**: Scenario internal consistency checks pass (see Spec 07)
- **Reproducibility**: Same scenario + seed produces identical output
- **Monte Carlo coverage**: Ensemble statistics bracket the base scenario
- **Comparison mode**: Multiple scenarios generate with shared entity IDs
- **Library extensibility**: Custom scenarios register and resolve correctly
- **Performance**: Scenario initialization < 100ms for all built-in scenarios

## References

1. Federal Reserve Board (2025). "2025 Supervisory Stress Test Scenarios."
2. EBA (2025). "EU-Wide Stress Test Results."
3. Feyen, E. & Mare, D. (2021). "Measuring Systemic Banking Resilience." World Bank WP 9864.
4. Budnik, K., et al. (2024). "Multiple-Scenario Stochastic Interpretation." ECB WP 2941.
5. Glasserman, P. (2004). *Monte Carlo Methods in Financial Engineering*. Springer.
6. Reinhart, C.M. & Rogoff, K.S. (2009). *This Time Is Different*. Princeton University Press.
7. NGFS (2024). "NGFS Climate Scenarios Phase V." Network for Greening the Financial System.
