# Spec 02: External Shock & Event System

**Status**: Draft
**Priority**: High
**Depends on**: Spec 01 (Macroeconomic Factor Engine)
**Extends**: `datasynth-core/src/distributions/event_timeline.rs`, `market_drift.rs`

---

## 1. Problem Statement

The current `event_timeline.rs` handles organizational events (M&A, restructuring) and
`market_drift.rs` has basic `PriceShockEvent` types, but there is no unified framework
for modeling **external shocks** — sudden, discrete events originating outside the
enterprise that propagate through multiple financial dimensions simultaneously. Real
enterprises experience supply disruptions, demand shocks, regulatory interventions,
pandemics, and commodity price spikes that create correlated, time-varying effects
across all generators.

## 2. Scientific Foundation

### 2.1 Shock Propagation Theory

External shocks propagate through economic networks via multiple channels (Acemoglu
et al., 2012). The **cascade model** distinguishes:

- **First-order effects**: Direct impact on exposed entities (e.g., supplier goes bankrupt)
- **Second-order effects**: Indirect impacts through network connections (e.g., supply shortage)
- **Systemic effects**: Economy-wide responses (e.g., central bank intervention)

The propagation follows a **damped wave** pattern:

```
Impact(t, d) = I₀ · e^{-αt} · e^{-βd} · (1 + γ·sin(ωt))
```

Where:
- `I₀` = initial shock magnitude
- `d` = network distance from shock epicenter
- `α` = temporal decay rate
- `β` = spatial/network decay rate
- `γ` = oscillation amplitude (reflecting adjustment cycles)
- `ω` = oscillation frequency

**Reference**: Acemoglu, D., Carvalho, V.M., Ozdaglar, A., & Tahbaz-Salehi, A. (2012).
"The Network Origins of Aggregate Fluctuations." *Econometrica*, 80(5), 1977-2016.

### 2.2 Recovery Curve Typology

Post-shock recovery follows characteristic shapes (Reinhart & Rogoff, 2009):

| Shape | Formula | Typical Scenario |
|-------|---------|-----------------|
| **V-shaped** | `r(t) = 1 - I₀·e^{-λt}` | Demand shock, natural disaster |
| **U-shaped** | `r(t) = 1 - I₀·(1 - tanh(λ(t-d)))` | Financial crisis |
| **L-shaped** | `r(t) = 1 - I₀·(1 - t/T)^{1/n}` | Structural change, permanent loss |
| **W-shaped** | `r(t) = V(t) - δ·e^{-μ(t-t₂)}` | Double-dip recession |
| **K-shaped** | `r_A(t) ≠ r_B(t)` | Divergent sector recovery (COVID) |
| **Swoosh** | `r(t) = 1 - I₀·e^{-λ√t}` | Gradual, asymptotic recovery |

**Reference**: Reinhart, C.M. & Rogoff, K.S. (2009). *This Time Is Different: Eight
Centuries of Financial Folly*. Princeton University Press.

### 2.3 Event Study Methodology

Financial event studies (MacKinlay, 1997) provide the framework for measuring
abnormal returns around events. We adapt this to measure **abnormal transaction
patterns** — deviations from expected volumes, amounts, and timing.

```
Abnormal_Activity(t) = Actual(t) - Expected(t | no shock)
CAR(t₁, t₂) = Σ Abnormal_Activity(t) for t in [t₁, t₂]
```

**Reference**: MacKinlay, A.C. (1997). "Event Studies in Economics and Finance."
*Journal of Economic Literature*, 35(1), 13-39.

## 3. Proposed Design

### 3.1 Shock Taxonomy

```rust
/// Top-level classification of external shocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalShockType {
    /// Macroeconomic shocks (recession, inflation spike, rate shock)
    Macroeconomic(MacroShockVariant),
    /// Supply-side disruptions
    SupplyChain(SupplyShockVariant),
    /// Demand-side changes
    Demand(DemandShockVariant),
    /// Geopolitical events (see Spec 03 for details)
    Geopolitical(GeopoliticalShockVariant),
    /// Natural disasters and climate events (see Spec 04)
    NaturalDisaster(NaturalDisasterVariant),
    /// Regulatory and policy changes
    Regulatory(RegulatoryShockVariant),
    /// Technology disruption
    Technology(TechShockVariant),
    /// Public health crises
    Pandemic(PandemicShockVariant),
    /// Market-specific events
    Market(MarketShockVariant),
    /// Custom user-defined shock
    Custom(CustomShockDefinition),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroShockVariant {
    InterestRateHike { magnitude_bps: u32 },
    InterestRateCut { magnitude_bps: u32 },
    InflationSpike { target_rate: f64 },
    CreditCrunch { spread_widening_bps: u32 },
    CurrencyCrisis { depreciation_pct: f64, currency: String },
    SovereignDebtCrisis { country: String },
    StockMarketCrash { drawdown_pct: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupplyShockVariant {
    SingleSupplierFailure { supplier_id: Option<String> },
    RegionalDisruption { region: String, severity: f64 },
    CommodityPriceSpike { commodity: String, increase_pct: f64 },
    LogisticsBottleneck { lanes: Vec<String>, delay_days: u32 },
    EnergyPriceShock { energy_type: String, increase_pct: f64 },
    RawMaterialShortage { material: String, availability_pct: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DemandShockVariant {
    ConsumerConfidenceCollapse { decline_pct: f64 },
    SectoralDemandSurge { sector: String, increase_pct: f64 },
    SubstitutionShift { from_product: String, to_product: String },
    SeasonalAnomaly { deviation_sigma: f64 },
    DigitalAcceleration { online_shift_pct: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PandemicShockVariant {
    Lockdown { severity: LockdownSeverity, duration_months: u32 },
    SupplyChainFreeze { affected_regions: Vec<String> },
    DemandRecomposition { essential_surge: f64, discretionary_decline: f64 },
    WorkforceImpact { absenteeism_rate: f64, remote_work_pct: f64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LockdownSeverity {
    /// Light restrictions — some closures, capacity limits
    Mild,
    /// Broad closures, essential services only
    Moderate,
    /// Full lockdown, stay-at-home orders
    Severe,
    /// Total economic standstill
    Total,
}
```

### 3.2 Shock Lifecycle Model

Every external shock follows a structured lifecycle:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalShock {
    /// Unique identifier
    pub id: String,
    /// Type and parameters of the shock
    pub shock_type: ExternalShockType,
    /// When the shock occurs (period index)
    pub onset_period: usize,
    /// How the shock unfolds over time
    pub lifecycle: ShockLifecycle,
    /// Which dimensions are affected and how
    pub impact_channels: Vec<ImpactChannel>,
    /// Recovery shape after peak impact
    pub recovery: RecoveryCurve,
    /// Probability of triggering secondary shocks
    pub contagion: Option<ContagionConfig>,
    /// Human-readable description
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShockLifecycle {
    /// Pre-shock warning period (gradual buildup)
    pub warning_periods: usize,
    /// Ramp-up from onset to peak (can be 0 for sudden shocks)
    pub ramp_up_periods: usize,
    /// Duration at peak impact
    pub peak_duration_periods: usize,
    /// Recovery duration (to reach recovery_target)
    pub recovery_periods: usize,
    /// Target recovery level (1.0 = full, 0.9 = permanent 10% loss)
    pub recovery_target: f64,
    /// Overall lifecycle phase at any point
    pub current_phase: ShockPhase,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShockPhase {
    Warning,
    RampUp,
    Peak,
    Recovery,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCurve {
    pub shape: RecoveryShape,
    /// For K-shaped: which sectors recover faster
    pub divergent_sectors: Option<HashMap<IndustrySector, f64>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RecoveryShape {
    V,          // Quick, symmetric bounce-back
    U,          // Extended bottom, full recovery
    L,          // Permanent structural loss
    W,          // Double-dip with secondary decline
    K,          // Divergent recovery by sector
    Swoosh,     // Gradual, asymptotic recovery
    Custom,     // User-defined recovery function
}
```

### 3.3 Impact Channels

Each shock affects multiple financial dimensions through defined channels:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactChannel {
    /// Which dimension is affected
    pub channel: ChannelType,
    /// Peak impact magnitude (multiplicative: 0.5 = 50% decline, 1.5 = 50% increase)
    pub peak_magnitude: f64,
    /// Lag from shock onset to channel activation (periods)
    pub lag_periods: usize,
    /// Which entities/sectors are affected
    pub scope: ImpactScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    // Volume channels
    TransactionVolume,
    SalesOrderVolume,
    PurchaseOrderVolume,
    ProductionVolume,
    HiringRate,

    // Amount channels
    TransactionAmount,
    RevenuePerUnit,
    CostOfGoodsSold,
    OperatingExpenses,
    WageGrowth,

    // Timing channels
    PaymentDelay,
    OrderLeadTime,
    ProductionCycleTime,
    CollectionPeriod,

    // Risk channels
    DefaultProbability,
    FraudRate,
    QualityDefectRate,
    InventoryShrinkage,
    AuditFindingRate,

    // Financial channels
    InterestExpense,
    FxVolatility,
    AssetImpairment,
    GoodwillImpairment,

    // Behavioral channels
    ApprovalThresholdStrictness,
    ControlEffectiveness,
    EmployeeErrorRate,
    VendorReliability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactScope {
    /// Affects all entities equally
    Universal,
    /// Affects specific sectors with given sensitivity
    SectorSpecific(HashMap<IndustrySector, f64>),
    /// Affects specific geographic regions
    RegionSpecific(Vec<String>),
    /// Affects entities matching criteria
    Conditional(ImpactCondition),
}
```

### 3.4 Shock Effect Computation

```rust
impl ExternalShock {
    /// Compute the effect multiplier for a given channel at a given period
    pub fn effect_at(&self, period: usize, channel: &ChannelType) -> f64 {
        if period < self.onset_period {
            // Pre-shock: check for warning signals
            return self.warning_effect(period, channel);
        }

        let elapsed = period - self.onset_period;
        let lifecycle = &self.lifecycle;

        // Find the relevant impact channel
        let impact = match self.impact_channels.iter().find(|c| &c.channel == channel) {
            Some(imp) => imp,
            None => return 1.0, // No effect on this channel
        };

        // Apply lag
        if elapsed < impact.lag_periods {
            return 1.0;
        }
        let effective_elapsed = elapsed - impact.lag_periods;

        // Compute lifecycle-phase-adjusted magnitude
        let phase_factor = match self.phase_at(effective_elapsed) {
            ShockPhase::Warning => 0.0,
            ShockPhase::RampUp => {
                let progress = effective_elapsed as f64 / lifecycle.ramp_up_periods as f64;
                progress.powf(1.5) // Accelerating ramp
            }
            ShockPhase::Peak => 1.0,
            ShockPhase::Recovery => {
                let recovery_elapsed = effective_elapsed
                    - lifecycle.ramp_up_periods
                    - lifecycle.peak_duration_periods;
                self.recovery.compute(recovery_elapsed, lifecycle)
            }
            ShockPhase::Resolved => 1.0 - lifecycle.recovery_target,
        };

        // Convert to multiplier
        // peak_magnitude < 1.0 → decline, > 1.0 → increase
        let deviation = (impact.peak_magnitude - 1.0) * phase_factor;
        1.0 + deviation
    }
}
```

### 3.5 Shock Sequencer

Manages multiple concurrent shocks with interaction effects:

```rust
pub struct ShockSequencer {
    /// All configured shocks, sorted by onset
    shocks: Vec<ExternalShock>,
    /// Interaction model for overlapping shocks
    interaction: ShockInteraction,
    /// Secondary shocks generated by contagion
    triggered_shocks: Vec<ExternalShock>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShockInteraction {
    /// Multiply effects (compounding)
    Multiplicative,
    /// Add effects (linear superposition)
    Additive,
    /// Take the maximum absolute effect
    DominantShock,
    /// Saturating model: combined = 1 - Π(1 - effect_i)
    Saturating,
}

impl ShockSequencer {
    /// Compute combined effect across all active shocks for a given period/channel
    pub fn combined_effect(&self, period: usize, channel: &ChannelType) -> f64 {
        let effects: Vec<f64> = self.shocks.iter()
            .chain(self.triggered_shocks.iter())
            .map(|s| s.effect_at(period, channel))
            .filter(|&e| (e - 1.0).abs() > f64::EPSILON)
            .collect();

        if effects.is_empty() {
            return 1.0;
        }

        match self.interaction {
            ShockInteraction::Multiplicative => effects.iter().product(),
            ShockInteraction::Additive => {
                1.0 + effects.iter().map(|e| e - 1.0).sum::<f64>()
            }
            ShockInteraction::DominantShock => {
                effects.iter()
                    .max_by(|a, b| (a - 1.0).abs().partial_cmp(&(b - 1.0).abs()).unwrap())
                    .copied()
                    .unwrap_or(1.0)
            }
            ShockInteraction::Saturating => {
                let declines: f64 = effects.iter()
                    .filter(|&&e| e < 1.0)
                    .map(|e| 1.0 - e)
                    .fold(1.0, |acc, d| acc * (1.0 - d));
                let increases: f64 = effects.iter()
                    .filter(|&&e| e > 1.0)
                    .map(|e| e - 1.0)
                    .fold(1.0, |acc, i| acc * (1.0 + i));
                declines * increases
            }
        }
    }
}
```

## 4. Pre-Built Shock Templates

### 4.1 Economic Crisis Template

```yaml
external_shocks:
  shocks:
    - id: "credit_crunch_2008"
      type:
        macroeconomic:
          credit_crunch:
            spread_widening_bps: 400
      onset_period: 18
      lifecycle:
        warning_periods: 3
        ramp_up_periods: 4
        peak_duration_periods: 6
        recovery_periods: 24
        recovery_target: 0.95
      recovery:
        shape: U
      impact_channels:
        - channel: transaction_volume
          peak_magnitude: 0.65         # 35% decline in volume
          lag_periods: 1
          scope: universal
        - channel: default_probability
          peak_magnitude: 3.5          # 3.5x increase in defaults
          lag_periods: 3
          scope:
            sector_specific:
              financial_services: 1.5
              real_estate: 1.8
              retail: 1.2
              healthcare: 0.3
        - channel: payment_delay
          peak_magnitude: 1.8          # 80% longer payment times
          lag_periods: 2
          scope: universal
        - channel: asset_impairment
          peak_magnitude: 2.5          # 2.5x impairment charges
          lag_periods: 4
          scope:
            sector_specific:
              real_estate: 2.0
              financial_services: 1.5
```

### 4.2 Pandemic Template

```yaml
    - id: "pandemic_lockdown"
      type:
        pandemic:
          lockdown:
            severity: severe
            duration_months: 3
      onset_period: 24
      lifecycle:
        warning_periods: 1
        ramp_up_periods: 1            # Sudden onset
        peak_duration_periods: 3
        recovery_periods: 18
        recovery_target: 0.92         # Some permanent structural change
      recovery:
        shape: K
        divergent_sectors:
          technology: 1.15            # Tech benefits (above pre-crisis)
          healthcare: 1.05
          retail: 0.85                # Permanent online shift
          hospitality: 0.70           # Slow recovery
          manufacturing: 0.95
      impact_channels:
        - channel: transaction_volume
          peak_magnitude: 0.40        # 60% decline at peak lockdown
          lag_periods: 0
          scope:
            sector_specific:
              hospitality: 2.0
              retail: 1.5
              technology: 0.3
              healthcare: 0.5
        - channel: production_volume
          peak_magnitude: 0.50
          lag_periods: 0
          scope: universal
        - channel: employee_error_rate
          peak_magnitude: 1.6         # Remote work transition errors
          lag_periods: 1
          scope: universal
        - channel: sales_order_volume
          peak_magnitude: 0.45
          lag_periods: 0
          scope:
            sector_specific:
              technology: 0.5         # Less decline for tech
              hospitality: 2.5        # More decline
```

### 4.3 Supply Chain Disruption Template

```yaml
    - id: "chip_shortage"
      type:
        supply_chain:
          raw_material_shortage:
            material: "semiconductors"
            availability_pct: 0.40
      onset_period: 12
      lifecycle:
        warning_periods: 2
        ramp_up_periods: 6
        peak_duration_periods: 12
        recovery_periods: 18
        recovery_target: 0.98
      recovery:
        shape: swoosh
      impact_channels:
        - channel: production_volume
          peak_magnitude: 0.55
          lag_periods: 2
          scope:
            sector_specific:
              technology: 1.8
              manufacturing: 1.5
              retail: 0.5
        - channel: cost_of_goods_sold
          peak_magnitude: 1.35        # 35% COGS increase
          lag_periods: 3
          scope:
            sector_specific:
              technology: 1.5
              manufacturing: 1.2
        - channel: order_lead_time
          peak_magnitude: 3.0         # 3x lead times
          lag_periods: 1
          scope:
            sector_specific:
              technology: 1.5
              manufacturing: 1.3
```

## 5. Configuration Schema

```yaml
external_realism:
  external_shocks:
    enabled: true

    # How to combine overlapping shock effects
    interaction_model: saturating     # multiplicative | additive | dominant_shock | saturating

    # Probability-based automatic shock generation
    auto_generate:
      enabled: false
      annual_shock_probability: 0.15  # P(≥1 major shock per year)
      severity_distribution: lognormal
      type_weights:
        macroeconomic: 0.30
        supply_chain: 0.25
        regulatory: 0.20
        demand: 0.15
        natural_disaster: 0.05
        pandemic: 0.05

    # Explicitly defined shocks
    shocks: []                        # List of ExternalShock definitions

    # Contagion settings
    contagion:
      enabled: true
      max_cascade_depth: 3            # Max secondary shock generations
      min_trigger_magnitude: 0.7      # Min shock magnitude to trigger cascades

    # Which channels are active
    active_channels:
      volume: true
      amount: true
      timing: true
      risk: true
      financial: true
      behavioral: true

    # Output
    export_shock_timeline: true       # Export shock events & effects as CSV
```

## 6. Integration Points

### 6.1 With Macroeconomic Engine (Spec 01)

Macro shocks modify the `MacroFactors` trajectory:
```rust
// A credit crunch shock feeds back into the macro engine
macro_engine.apply_shock_overlay(period, |factors| {
    factors.credit_spread_bps += shock.spread_widening;
    factors.gdp_growth *= shock.gdp_impact;
    factors.regime = EconomicRegime::Contraction;
});
```

### 6.2 With Existing Generators

The `ShockSequencer` provides a unified query interface that generators call:
```rust
let volume_factor = shock_sequencer.combined_effect(period, &ChannelType::TransactionVolume);
let amount_factor = shock_sequencer.combined_effect(period, &ChannelType::TransactionAmount);
// Apply to existing generation logic
let adjusted_volume = base_volume * volume_factor;
```

### 6.3 With Anomaly Injection

Shocks can alter anomaly rates contextually:
```rust
let fraud_factor = shock_sequencer.combined_effect(period, &ChannelType::FraudRate);
// During crises, fraud detection may decrease while fraud attempts increase
anomaly_config.fraud_rate *= fraud_factor;
```

## 7. Testing Strategy

- **Lifecycle correctness**: Verify each phase (warning → ramp → peak → recovery → resolved)
- **Recovery shape validation**: V-shape recovers faster than U-shape at same parameters
- **Multi-shock interaction**: Verify saturating model doesn't exceed physical bounds
- **Contagion depth**: Secondary shocks respect max_cascade_depth
- **Channel independence**: Shock to volume doesn't leak into amount without explicit channel
- **Template validation**: All pre-built templates produce reasonable output ranges

## 8. Performance

- Shock evaluation per period per channel: O(S) where S = active shocks (typically < 10)
- Memory: ~1KB per shock definition
- No per-record overhead beyond the period-level lookup

## References

1. Acemoglu, D., Carvalho, V.M., Ozdaglar, A., & Tahbaz-Salehi, A. (2012). "The Network Origins of Aggregate Fluctuations." *Econometrica*, 80(5), 1977-2016.
2. Reinhart, C.M. & Rogoff, K.S. (2009). *This Time Is Different*. Princeton University Press.
3. MacKinlay, A.C. (1997). "Event Studies in Economics and Finance." *Journal of Economic Literature*, 35(1), 13-39.
4. Barro, R.J. (2006). "Rare Disasters and Asset Markets in the Twentieth Century." *Quarterly Journal of Economics*, 121(3), 823-866.
5. Baker, S.R., Bloom, N., & Davis, S.J. (2016). "Measuring Economic Policy Uncertainty." *Quarterly Journal of Economics*, 131(4), 1593-1636.
