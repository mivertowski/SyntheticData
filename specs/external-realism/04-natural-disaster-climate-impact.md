# Spec 04: Natural Disaster & Climate Impact Module

**Status**: Draft
**Priority**: High
**Depends on**: Spec 01 (Macro Factor Engine), Spec 02 (External Shock System)
**Extends**: `datasynth-core/src/models/esg.rs`, `market_drift.rs`

---

## 1. Problem Statement

The framework's ESG module (`datasynth-core/src/models/esg.rs`) generates emission records,
energy consumption, and climate disclosures but lacks the modeling of **how climate and
natural disaster events impact enterprise financial data**. Real enterprises face both
acute physical risks (hurricanes, floods, wildfires) and chronic transition risks (carbon
pricing, stranded assets, regulatory phase-outs). Without these, generated data misses
the financial fingerprints of business interruption, insurance claims, impairment charges,
and the cost trajectory of the net-zero transition.

## 2. Scientific Foundation

### 2.1 Catastrophe Modeling (Cat Models)

Commercial catastrophe models (AIR Worldwide, Moody's RMS) use a four-component pipeline
(Grossi & Kunreuther, 2005):

1. **Hazard module**: Simulates event frequency and intensity using historical catalogs
   and physical models (wind fields, flood maps, seismic shake maps)
2. **Vulnerability module**: Maps hazard intensity to damage ratios via fragility curves
3. **Exposure module**: Inventories assets at risk with location and construction data
4. **Financial module**: Calculates losses including business interruption (BI), demand
   surge, and loss amplification

For synthetic data, we abstract this into a **simplified stochastic event set** with
configurable frequency-severity distributions.

**Key parameters**:
- Poisson arrival rate for events by type and region
- Severity follows a Pareto or lognormal distribution (heavy-tailed)
- Business interruption duration: lognormal(μ, σ) days
- Recovery follows empirical curve shapes (see Spec 02)

**Reference**: Grossi, P. & Kunreuther, H. (2005). *Catastrophe Modeling: A New
Approach to Managing Risk*. Springer.

### 2.2 NGFS Climate Scenarios (Phase V, 2024)

The Network for Greening the Financial System defines four scenario categories across
a physical-risk vs. transition-risk matrix:

| Category | Scenarios | Temp. Outcome | Physical Risk | Transition Risk |
|----------|-----------|--------------|---------------|-----------------|
| **Orderly** | Net Zero 2050, Below 2°C | 1.5-1.8°C | Low | Low-Moderate |
| **Disorderly** | Delayed Transition, Divergent Net Zero | <2°C | Low | High |
| **Hot House World** | NDCs, Current Policies | >2.5°C | Severe | Low |
| **Too Little, Too Late** | Fragmented World | >2°C | High | High |

**Carbon price trajectories** (NGFS Phase V):
- Net Zero 2050 (Orderly): ~$300/tCO₂ by 2035
- Low-Demand scenario: ~$200/tCO₂ by 2035
- Delayed Transition: Steep post-2030 ramp
- Current Policies: Near-zero carbon pricing

**Damage functions** (Kotz, Levermann & Wenz, 2024): Climate shocks have **persistent**
effects on GDP growth (not just temporary level effects), incorporating precipitation
patterns, temperature variability, and lagged GDP effects up to 10 years. Under Current
Policies, cumulative GDP losses reach 10-30% by 2100 (tail risks up to 50%).

**Reference**: NGFS (2024). "NGFS Climate Scenarios Phase V: Technical Documentation."
Network for Greening the Financial System.

### 2.3 TCFD Framework

The Task Force on Climate-related Financial Disclosures (now absorbed into IFRS S2/ISSB)
recommends scenario analysis with ≥3 contrasting scenarios, converting climate risks to
financial metrics (EBITDA, CAPEX, OPEX, asset valuation).

Key distinction:
- **Acute physical risks**: Extreme weather events (increasing frequency/severity)
- **Chronic physical risks**: Long-term shifts (temperature, sea level, water stress)
- **Transition risks**: Policy, technology, market, and reputational risks from
  decarbonization

**Reference**: TCFD (2017). "Recommendations of the Task Force on Climate-related
Financial Disclosures." Financial Stability Board.

### 2.4 Empirical Recovery Data by Disaster Type

From NIST business survey data and academic literature:

| Disaster Type | Typical BI Duration | Revenue Impact | Permanent Closure Rate |
|--------------|--------------------|--------------------|----------------------|
| Tornado | Days to weeks | 5-15% (local) | Low (<5%) |
| Hurricane (Cat 3+) | Weeks to months | 15-40% (regional) | 10-20% for SMEs |
| Earthquake (>6.0) | Months to years | 20-50% | High for SMEs (25%+) |
| Flood (major) | Weeks to months | 10-30% | 15% for SMEs |
| Wildfire | Weeks to months | 10-25% | Moderate |
| Pandemic | Months to years | Sector-dependent | High for contact-intensive |
| Drought (prolonged) | Months | 5-20% (agriculture) | Low |

85% of baseline production capacity is the empirically validated "recovered" threshold
(NIST, 2024).

**Reference**: FEMA/NIST (2024). "Using Disaster Surveys to Model Business
Interruption." *Natural Hazards Review*.

## 3. Proposed Design

### 3.1 Natural Disaster Event Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NaturalDisasterType {
    /// Tropical cyclone / hurricane / typhoon
    Hurricane(HurricaneParams),
    /// Tectonic earthquake
    Earthquake(EarthquakeParams),
    /// River, coastal, or flash flooding
    Flood(FloodParams),
    /// Wildfire / bushfire
    Wildfire(WildfireParams),
    /// Tornado / severe storm
    Tornado(TornadoParams),
    /// Extended drought
    Drought(DroughtParams),
    /// Volcanic eruption
    VolcanicEruption(VolcanicParams),
    /// Tsunami
    Tsunami(TsunamiParams),
    /// Extreme heat / cold wave
    ExtremeTemperature(TempExtremeParams),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HurricaneParams {
    /// Saffir-Simpson category (1-5)
    pub category: u8,
    /// Affected geographic region
    pub region: String,
    /// Wind speed (mph) — determines damage ratio
    pub max_wind_speed_mph: f64,
    /// Storm surge height (feet) — drives flood damage
    pub storm_surge_ft: f64,
    /// Radius of damaging winds (miles)
    pub damage_radius_miles: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarthquakeParams {
    /// Richter magnitude
    pub magnitude: f64,
    /// Depth (km) — shallower = more damaging
    pub depth_km: f64,
    /// Affected region
    pub epicenter_region: String,
    /// Modified Mercalli Intensity at key locations
    pub peak_intensity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloodParams {
    /// Flood type
    pub flood_type: FloodType,
    /// Return period (years) — indicates severity
    pub return_period_years: u32,
    /// Inundation depth at key locations (meters)
    pub max_depth_m: f64,
    /// Duration of flooding (days)
    pub duration_days: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FloodType {
    River,
    Coastal,
    Flash,
    Pluvial,
}
```

### 3.2 Natural Disaster Financial Impact Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterFinancialImpact {
    /// Direct property/asset damage (as fraction of exposed asset value)
    pub damage_ratio: f64,
    /// Business interruption parameters
    pub business_interruption: BusinessInterruption,
    /// Extra expense (cleanup, temporary facilities, overtime)
    pub extra_expense_multiplier: f64,
    /// Contingent business interruption (supplier/customer impact)
    pub contingent_bi: ContingentBI,
    /// Demand surge (post-disaster cost inflation for repairs)
    pub demand_surge_pct: f64,
    /// Insurance recovery (0.0 = uninsured, 1.0 = fully insured)
    pub insurance_coverage_ratio: f64,
    /// Impairment charges required
    pub impairment_charges: ImpairmentEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessInterruption {
    /// Duration of full interruption (periods)
    pub full_stop_periods: usize,
    /// Duration of partial operations (periods)
    pub partial_operation_periods: usize,
    /// Capacity during partial operations (0.0-1.0)
    pub partial_capacity: f64,
    /// Recovery curve shape
    pub recovery_shape: RecoveryShape,
    /// Revenue loss during full stop (fraction)
    pub revenue_loss_full: f64,
    /// Revenue loss during partial (fraction)
    pub revenue_loss_partial: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContingentBI {
    /// Fraction of suppliers in affected region
    pub supplier_exposure: f64,
    /// Fraction of customers in affected region
    pub customer_exposure: f64,
    /// Supply chain recovery lag (additional periods)
    pub supply_chain_lag_periods: usize,
}
```

### 3.3 Climate Transition Risk Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateTransitionModel {
    /// NGFS scenario selection
    pub scenario: NgfsScenario,
    /// Carbon price trajectory
    pub carbon_price: CarbonPriceTrajectory,
    /// Stranded asset risk
    pub stranded_assets: StrandedAssetConfig,
    /// Technology transition effects
    pub technology_transition: TechTransitionConfig,
    /// Regulatory phase-outs
    pub phase_outs: Vec<RegulatoryPhaseOut>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NgfsScenario {
    /// Immediate, smooth policy action → 1.5°C
    NetZero2050,
    /// Gradual strengthening → <2°C
    Below2C,
    /// Action starts late (post-2030), abrupt → <2°C
    DelayedTransition,
    /// Regional divergence, some sectors stranded
    DivergentNetZero,
    /// Only current NDC commitments → ~2.5°C
    NationallyDetermined,
    /// No new policy → >3°C
    CurrentPolicies,
    /// Fragmented response → >2°C with both risk types high
    FragmentedWorld,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonPriceTrajectory {
    /// Starting carbon price ($/tCO₂)
    pub initial_price: f64,
    /// Target carbon price by end of generation period
    pub target_price: f64,
    /// Growth model
    pub growth_model: CarbonPriceGrowth,
    /// Sector-specific carbon intensity (tCO₂ per $M revenue)
    pub sector_intensities: HashMap<IndustrySector, f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CarbonPriceGrowth {
    /// Linear increase
    Linear,
    /// Exponential growth (typical of orderly scenarios)
    Exponential { annual_growth_rate: f64 },
    /// Step function (sudden policy change)
    Step { step_period: usize, step_size: f64 },
    /// S-curve (gradual then rapid then plateau)
    Logistic { midpoint_period: usize, steepness: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrandedAssetConfig {
    /// Fraction of fixed assets at stranding risk
    pub at_risk_fraction: f64,
    /// Stranding trigger (carbon price threshold)
    pub trigger_carbon_price: f64,
    /// Impairment schedule once triggered
    pub impairment_schedule: Vec<(usize, f64)>, // (period_offset, impairment_fraction)
    /// Affected asset categories
    pub affected_categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryPhaseOut {
    /// What is being phased out
    pub target: String,
    /// Announcement period
    pub announced_period: usize,
    /// Effective ban/phase-out period
    pub effective_period: usize,
    /// Revenue impact (fraction of total revenue affected)
    pub revenue_at_risk: f64,
    /// Capex required for transition
    pub transition_capex_pct: f64,
}
```

### 3.4 Chronic Physical Risk Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicPhysicalRisk {
    /// Temperature trend (°C per decade)
    pub warming_rate: f64,
    /// Sea level rise (mm per year)
    pub sea_level_rise_mm_per_year: f64,
    /// Water stress trend
    pub water_stress: WaterStressModel,
    /// Agricultural yield impact
    pub crop_yield_impact: f64,
    /// Energy demand shift (heating ↓, cooling ↑)
    pub energy_demand_shift: EnergyDemandShift,
    /// Gradual increase in insurance premiums
    pub insurance_premium_escalation: f64,
    /// Health/productivity impact from heat stress
    pub labor_productivity_loss: LaborProductivityLoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaborProductivityLoss {
    /// Outdoor worker productivity loss per °C above threshold
    pub outdoor_loss_per_degree: f64,
    /// Temperature threshold for productivity impact (°C)
    pub threshold_temp_c: f64,
    /// Fraction of workforce affected (outdoor/manual)
    pub affected_workforce_fraction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyDemandShift {
    /// Heating degree day reduction rate (per decade)
    pub hdd_reduction_pct: f64,
    /// Cooling degree day increase rate (per decade)
    pub cdd_increase_pct: f64,
    /// Net energy cost impact per degree of warming
    pub net_cost_impact_per_degree: f64,
}
```

### 3.5 Carbon Cost Pass-Through to Generators

```rust
impl ClimateTransitionModel {
    /// Compute the carbon cost impact for a given period and sector
    pub fn carbon_cost_impact(
        &self,
        period: usize,
        sector: &IndustrySector,
    ) -> CarbonCostEffect {
        let carbon_price = self.carbon_price.price_at(period);
        let intensity = self.carbon_price.sector_intensities
            .get(sector)
            .copied()
            .unwrap_or(0.0);

        // Carbon cost as fraction of revenue
        let carbon_cost_pct = carbon_price * intensity / 1_000_000.0;

        // Check for stranded asset trigger
        let stranding_active = carbon_price >= self.stranded_assets.trigger_carbon_price;

        CarbonCostEffect {
            cogs_increase_pct: carbon_cost_pct * 0.70, // 70% hits COGS
            opex_increase_pct: carbon_cost_pct * 0.30, // 30% hits OpEx
            capex_multiplier: if stranding_active { 1.15 } else { 1.0 },
            impairment_trigger: stranding_active,
            margin_compression_pct: carbon_cost_pct * 0.50, // 50% absorbed
        }
    }
}
```

### 3.6 Sector-Specific Carbon Intensities

| Sector | tCO₂ per $M Revenue | Carbon Price Sensitivity |
|--------|---------------------|------------------------|
| Energy (O&G) | 850 | Very High |
| Utilities | 720 | Very High |
| Materials (steel, cement) | 500 | High |
| Transportation | 380 | High |
| Manufacturing | 180 | Moderate |
| Agriculture | 250 | Moderate |
| Real Estate | 120 | Moderate |
| Financial Services | 15 | Low (but financed emissions) |
| Technology | 25 | Low |
| Healthcare | 45 | Low |

## 4. Configuration Schema

```yaml
external_realism:
  natural_disasters:
    enabled: true

    # Stochastic disaster generation
    stochastic:
      enabled: true
      # Regional hazard profiles
      regions:
        - name: "US_Southeast"
          hazards:
            - type: hurricane
              annual_frequency: 0.8          # Average 0.8 events/year
              category_distribution: [0.35, 0.30, 0.20, 0.10, 0.05]  # Cat 1-5
            - type: flood
              annual_frequency: 1.2
              severity_distribution: lognormal
              severity_params: { mu: 2.0, sigma: 1.5 }
        - name: "US_West"
          hazards:
            - type: earthquake
              annual_frequency: 0.3
              magnitude_distribution: { min: 4.0, max: 8.0, gutenberg_richter_b: 1.0 }
            - type: wildfire
              annual_frequency: 1.5
              severity_distribution: pareto
              severity_params: { alpha: 2.0, min: 0.01 }

    # Explicit disaster events
    events:
      - type: hurricane
        params:
          category: 4
          region: "US_Southeast"
          max_wind_speed_mph: 145
          storm_surge_ft: 12
        onset_period: 30
        financial_impact:
          damage_ratio: 0.15
          business_interruption:
            full_stop_periods: 1
            partial_operation_periods: 3
            partial_capacity: 0.60
            recovery_shape: V
          insurance_coverage_ratio: 0.80
          demand_surge_pct: 0.20

    # Exposure configuration
    entity_exposure:
      # Map company regions to hazard regions
      region_mapping:
        "US_*": "US_Southeast"
        "EU_*": "EU_Western"
      # Fraction of assets exposed per region
      asset_exposure_by_region:
        "US_Southeast": 0.30
        "US_West": 0.25

  # Climate transition risk
  climate_transition:
    enabled: true
    scenario: net_zero_2050               # NGFS scenario

    carbon_price:
      initial_price: 30.0                 # Current $/tCO₂
      target_price: 250.0                 # Target by end of period
      growth_model:
        exponential:
          annual_growth_rate: 0.12        # 12% annual price increase

    stranded_assets:
      at_risk_fraction: 0.08             # 8% of fixed assets at risk
      trigger_carbon_price: 150.0         # Stranding begins at $150/tCO₂
      impairment_schedule:
        - [0, 0.20]                       # 20% impaired at trigger
        - [12, 0.40]                      # 40% after 1 year
        - [24, 0.70]                      # 70% after 2 years

    phase_outs:
      - target: "coal_operations"
        announced_period: 6
        effective_period: 60
        revenue_at_risk: 0.05
        transition_capex_pct: 0.03

    sector_overrides:
      energy:
        carbon_intensity: 850
        transition_capex_multiplier: 2.0
      manufacturing:
        carbon_intensity: 180
        transition_capex_multiplier: 1.3

  # Chronic physical risk (long-term trends)
  chronic_physical:
    enabled: true
    warming_rate_per_decade: 0.2          # °C per decade
    sea_level_rise_mm_per_year: 3.6
    insurance_premium_escalation: 0.05    # 5% annual increase
    labor_productivity:
      threshold_temp_c: 35.0
      outdoor_loss_per_degree: 0.02       # 2% per °C above threshold
      affected_workforce_fraction: 0.15

  # Output
  export_climate_metrics: true
  export_disaster_timeline: true
```

## 5. Pre-Built Climate Scenarios

### 5.1 Net Zero Pathway

```yaml
scenario: net_zero_pathway
climate_transition:
  scenario: net_zero_2050
  carbon_price: { initial: 50, target: 300, growth: exponential_0.12 }
  stranded_assets: { at_risk: 0.10, trigger: 120 }
natural_disasters:
  frequency_multiplier: 1.0              # No additional physical risk
chronic_physical:
  warming_rate: 0.15                     # Limited warming
```

### 5.2 Delayed Action (Disorderly)

```yaml
scenario: delayed_transition
climate_transition:
  scenario: delayed_transition
  carbon_price:
    initial: 20
    target: 400                          # Higher terminal price needed
    growth:
      step:
        step_period: 60                  # Sudden carbon price jump in 2030
        step_size: 200
  stranded_assets: { at_risk: 0.25, trigger: 100 }  # More stranding risk
```

### 5.3 Hot House World

```yaml
scenario: hot_house_world
climate_transition:
  scenario: current_policies
  carbon_price: { initial: 10, target: 30, growth: linear }  # Minimal policy
natural_disasters:
  frequency_multiplier: 1.5             # 50% more frequent extreme events
  severity_multiplier: 1.3              # 30% more severe
chronic_physical:
  warming_rate: 0.35                    # Higher warming trajectory
  insurance_premium_escalation: 0.10    # 10% annual insurance cost increase
  labor_productivity:
    affected_workforce_fraction: 0.25   # More workers affected
```

## 6. Downstream Effects

| Climate Factor | Affected Generator | Financial Effect |
|---------------|-------------------|-----------------|
| Carbon price | je_generator | COGS increase, carbon tax line items |
| Carbon price | manufacturing | Production cost increase |
| Stranded assets | fa_generator | Impairment charges, accelerated depreciation |
| Physical damage | fa_generator | Asset write-downs |
| BI duration | o2c_generator, p2p_generator | Revenue/procurement halt |
| Insurance premium | je_generator | Operating expense increase |
| Demand surge | p2p_generator | Post-disaster procurement cost spike |
| Warming trend | hr/payroll | Overtime, productivity adjustments |
| Phase-outs | revenue_recognition | Revenue stream wind-down |
| ESG disclosure | audit/ | Additional audit procedures |

## 7. Testing Strategy

- **Disaster frequency**: Poisson-generated events match configured annual rates (±20%)
- **Severity distribution**: Cat model outputs follow Pareto/lognormal tail behavior
- **Recovery curves**: BI recovery follows configured shape (V/U/L)
- **Carbon cost linearity**: Carbon cost scales linearly with price × intensity
- **Stranded asset trigger**: Impairment activates exactly at configured carbon price
- **NGFS coherence**: Orderly scenario has lower physical risk than Hot House
- **Insurance coverage**: Net loss = gross loss × (1 - coverage ratio)
- **Chronic trend**: Warming effects accumulate monotonically over time

## References

1. Grossi, P. & Kunreuther, H. (2005). *Catastrophe Modeling: A New Approach to Managing Risk*. Springer.
2. NGFS (2024). "NGFS Climate Scenarios Phase V: Technical Documentation." Network for Greening the Financial System.
3. TCFD (2017). "Recommendations of the Task Force on Climate-related Financial Disclosures." FSB.
4. Kotz, M., Levermann, A., & Wenz, L. (2024). "The Economic Commitment of Climate Change." *Nature*, 628, 551-557.
5. Battiston, S., Mandel, A., Monasterolo, I., et al. (2017). "A Climate Stress-Test of the Financial System." *Nature Climate Change*, 7, 283-288.
6. Dietz, S., Bowen, A., Dixon, C., & Gradwell, P. (2016). "'Climate Value at Risk' of Global Financial Assets." *Nature Climate Change*, 6, 676-679.
7. NIST (2024). "Using Disaster Surveys to Model Business Interruption." *Natural Hazards Review*.
