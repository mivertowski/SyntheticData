# Historical Data Plugin — Specification & Design

> **Status:** Draft
> **Date:** 2026-02-17
> **Scope:** Pluggable architecture for incorporating real-world historical data and economic events into synthetic data generation

---

## 1. Executive Summary

### 1.1 Problem Statement

DataSynth currently generates realistic temporal patterns using mathematical models — Ornstein-Uhlenbeck processes for FX rates, sinusoidal economic cycles via `DriftConfig`, configurable `RegimeChange` breakpoints, and log-normal amount distributions with Benford compliance. These produce statistically plausible data, but when generating data for **historical periods** (e.g., 2019-2022), the output does not reflect real-world events:

- FX rates follow random walks instead of actual EUR/USD movements
- No reflection of COVID-19 demand shocks, supply chain disruptions, or stimulus effects
- Interest rate environments don't match central bank policy changes
- Commodity price spikes (e.g., 2022 energy crisis) are absent
- Industry-specific events (e.g., semiconductor shortage, banking crises) have no effect

### 1.2 Proposed Solution

Introduce a **Historical Data Provider** plugin system that:

1. **Supplies real-world time series** (FX rates, interest rates, commodity prices, market indices) to existing generators
2. **Injects economic event overlays** that modulate generation parameters (demand multipliers, anomaly rates, payment delays) during historically significant periods
3. **Supports sector-specific event packs** with industry-tailored impacts
4. **Is fully pluggable** — providers can be bundled data files, commercial data feeds, or customer-supplied datasets
5. **Degrades gracefully** — generators fall back to synthetic models when historical data is unavailable

### 1.3 Design Principles

| Principle | Rationale |
|-----------|-----------|
| **Pluggable providers** | Different customers need different data sources; commercial offering requires tiered access |
| **Overlay, don't replace** | Historical data modulates existing generators rather than replacing them; preserves all current functionality |
| **Deterministic replay** | Same seed + same historical data = identical output; historical data is versioned |
| **Graceful fallback** | Missing data for a date range falls back to synthetic models transparently |
| **Commercial tiering** | Free tier with bundled basics; premium packs sold separately; BYOD always supported |
| **Privacy-safe** | No proprietary customer data embedded in packs; only public/licensed market data |

---

## 2. Current Architecture — Integration Points

The historical data plugin must integrate with existing subsystems without disrupting them. Below are the specific touch-points.

### 2.1 FX Rate Generation

**Current:** `FxRateService` in `datasynth-generators/src/fx/` generates synthetic FX rates using configurable base rates, volatility, and random drift.

**Integration:** Replace or blend synthetic rates with actual historical rates (e.g., ECB daily reference rates, Federal Reserve H.10 data). When historical data is available for the requested date, use it directly; otherwise interpolate or fall back to synthetic generation.

### 2.2 Economic Cycles & Drift

**Current:** `DriftConfig` in `datasynth-core/src/distributions/drift.rs` models economic cycles as sinusoidal waves with configurable `cycle_period_months`, `amplitude`, `recession_probability`, and `recession_depth`. `RegimeChange` breakpoints allow manual parameter shifts at specific dates.

**Integration:** Auto-populate `RegimeChange` entries from historical event databases. For example, a "COVID-19 Recession" event (March 2020) would inject a regime change with appropriate parameter shifts rather than relying on random recession probability.

### 2.3 Amount Distributions

**Current:** `AmountSampler` and mixture models in `datasynth-core/src/distributions/` generate transaction amounts using log-normal components with Benford compliance.

**Integration:** Historical events modulate the mixture weights and parameters. During a recession, the "major transaction" component weight decreases; during a boom, it increases. Industry-specific events (e.g., commodity price spike) shift the `mu` parameter of relevant amount components.

### 2.4 Temporal Patterns

**Current:** `TemporalSampler` and `PeriodEndDynamics` in `datasynth-core/src/distributions/temporal.rs` model seasonality and period-end spikes. `HolidayCalendar` covers 11 regions.

**Integration:** Historical events can suppress or amplify temporal patterns. For example, during lockdowns, period-end processing spikes may shift earlier (remote work patterns) or become more extreme (delayed processing).

### 2.5 Anomaly Injection

**Current:** `AnomalyInjector` in `datasynth-generators/src/anomaly/` injects anomalies at configurable rates by type.

**Integration:** Economic stress periods historically correlate with increased fraud and error rates. The plugin can supply event-driven anomaly rate multipliers (e.g., 1.5x during recessions, 2x during banking crises).

### 2.6 Plugin System

**Current:** `PluginRegistry` in `datasynth-core/src/traits/registry.rs` supports `GeneratorPlugin`, `SinkPlugin`, and `TransformPlugin` with thread-safe registration via `Arc<RwLock<...>>`.

**Integration:** Add a new `HistoricalDataProvider` trait to the plugin system, registered and discovered via the same `PluginRegistry` pattern.

---

## 3. Core Trait Design

### 3.1 HistoricalDataProvider Trait

```rust
/// Primary trait for supplying historical data to generators.
/// Providers can be bundled files, remote APIs, or customer-supplied datasets.
pub trait HistoricalDataProvider: Send + Sync {
    /// Unique identifier for this provider (e.g., "ecb-fx-rates", "fred-macro")
    fn provider_id(&self) -> &str;

    /// Human-readable name and version
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    /// Data categories this provider supports
    fn capabilities(&self) -> Vec<HistoricalDataCapability>;

    /// Date range for which this provider has data
    fn available_range(&self) -> Option<DateRange>;

    /// Retrieve an FX rate for a currency pair on a given date.
    /// Returns None if data is unavailable (triggers fallback).
    fn fx_rate(
        &self,
        base: &str,
        quote: &str,
        date: NaiveDate,
    ) -> Result<Option<Decimal>, SynthError>;

    /// Retrieve a batch of FX rates for a date range (efficient bulk access).
    fn fx_rate_series(
        &self,
        base: &str,
        quote: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<DatedValue>, SynthError>;

    /// Retrieve an interest/benchmark rate on a given date.
    fn interest_rate(
        &self,
        rate_type: InterestRateType,
        date: NaiveDate,
    ) -> Result<Option<Decimal>, SynthError>;

    /// Retrieve a commodity price on a given date.
    fn commodity_price(
        &self,
        commodity: CommodityType,
        date: NaiveDate,
    ) -> Result<Option<Decimal>, SynthError>;

    /// Retrieve a market/equity index value on a given date.
    fn market_index(
        &self,
        index: MarketIndexType,
        date: NaiveDate,
    ) -> Result<Option<Decimal>, SynthError>;

    /// Retrieve active economic events for a given date.
    /// Multiple events can be active simultaneously.
    fn active_events(
        &self,
        date: NaiveDate,
        sector: Option<&str>,
    ) -> Result<Vec<EconomicEvent>, SynthError>;

    /// Retrieve the economic regime classification for a date.
    fn economic_regime(
        &self,
        date: NaiveDate,
        region: Option<&str>,
    ) -> Result<Option<EconomicRegime>, SynthError>;

    /// Initialize the provider with configuration (API keys, file paths, etc.)
    fn initialize(&mut self, config: &serde_json::Value) -> Result<(), SynthError>;

    /// Optional: prefetch data for a date range to optimize batch generation.
    fn prefetch(&self, _start: NaiveDate, _end: NaiveDate) -> Result<(), SynthError> {
        Ok(()) // default no-op
    }
}
```

### 3.2 Supporting Types

```rust
#[derive(Debug, Clone)]
pub struct DatedValue {
    pub date: NaiveDate,
    pub value: Decimal,
}

#[derive(Debug, Clone)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HistoricalDataCapability {
    FxRates,
    InterestRates,
    CommodityPrices,
    MarketIndices,
    EconomicEvents,
    EconomicRegimes,
    SectorEvents { sector: String },
}

#[derive(Debug, Clone)]
pub enum InterestRateType {
    FedFundsRate,
    Euribor { tenor_months: u32 },
    Libor { currency: String, tenor_months: u32 },
    Sofr,
    Sonia,
    Estr,
    PrimeRate,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum CommodityType {
    CrudeOilBrent,
    CrudeOilWti,
    NaturalGas,
    Gold,
    Silver,
    Copper,
    Wheat,
    Corn,
    Steel,
    Lumber,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum MarketIndexType {
    SP500,
    DowJones,
    Nasdaq,
    EuroStoxx50,
    Ftse100,
    Nikkei225,
    ShanghaiComposite,
    Dax,
    Custom(String),
}
```

### 3.3 Economic Event Model

Events represent discrete historical occurrences that modulate generation parameters over a defined period.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicEvent {
    /// Unique event identifier (e.g., "covid-19-pandemic")
    pub event_id: String,
    /// Human-readable name
    pub name: String,
    /// Event classification
    pub event_type: EconomicEventType,
    /// Date the event's effects begin
    pub start_date: NaiveDate,
    /// Date the event's effects end (None = ongoing)
    pub end_date: Option<NaiveDate>,
    /// Affected geographic regions (empty = global)
    pub regions: Vec<String>,
    /// Affected industry sectors (empty = all sectors)
    pub sectors: Vec<String>,
    /// Severity from 0.0 (negligible) to 1.0 (extreme)
    pub severity: f64,
    /// Parameter impacts this event produces
    pub impacts: Vec<EventImpact>,
    /// Optional narrative description for audit trail / reporting
    pub description: Option<String>,
    /// Source attribution for the event data
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomicEventType {
    Recession,
    FinancialCrisis,
    Pandemic,
    SupplyChainDisruption,
    RegulatoryChange,
    GeopoliticalConflict,
    NaturalDisaster,
    CommodityShock,
    CurrencyCrisis,
    TechDisruption,
    TradeWar,
    MonetaryPolicyShift,
    SectorBoom,
    SectorBust,
    Custom(String),
}

/// Describes how an event impacts a specific generation parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventImpact {
    /// Which parameter is affected
    pub target: ImpactTarget,
    /// How the parameter changes
    pub effect: ImpactEffect,
    /// Ramp-up curve: how quickly the impact reaches full strength
    pub onset: TransitionCurve,
    /// Recovery curve: how the impact fades after the event ends
    pub recovery: TransitionCurve,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactTarget {
    /// Multiplier on transaction volume
    TransactionVolume,
    /// Shift in transaction amount distribution (mu parameter)
    TransactionAmountMean,
    /// Change in amount variance
    TransactionAmountVariance,
    /// Multiplier on anomaly injection rate
    AnomalyRate,
    /// Multiplier on payment delay (days)
    PaymentDelay,
    /// Shift in demand patterns
    MarketDemand,
    /// Supplier lead time multiplier
    SupplierLeadTime,
    /// Employee headcount growth rate
    HeadcountGrowth,
    /// Revenue growth rate modifier
    RevenueGrowth,
    /// Credit risk multiplier (affects AR aging)
    CreditRisk,
    /// Inventory turnover modifier
    InventoryTurnover,
    /// Custom target with string key
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactEffect {
    /// Multiply the current value by this factor (1.0 = no change)
    Multiplier(f64),
    /// Add an absolute offset to the current value
    AdditiveOffset(f64),
    /// Override the value entirely
    Override(f64),
    /// Shift the value by a percentage (-0.5 = decrease 50%)
    PercentageShift(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCurve {
    /// Immediate step change
    Immediate,
    /// Linear ramp over N days
    Linear { days: u32 },
    /// Exponential approach with half-life in days
    Exponential { half_life_days: u32 },
    /// S-curve (logistic) centered at midpoint_days
    Sigmoid { midpoint_days: u32, steepness: f64 },
}
```

### 3.4 Economic Regime Model

Regimes classify the macroeconomic environment to automatically adjust generation profiles.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicRegime {
    pub regime_type: RegimeType,
    pub confidence: f64,          // 0.0-1.0
    pub gdp_growth_annual: Option<f64>,
    pub inflation_rate: Option<f64>,
    pub unemployment_rate: Option<f64>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegimeType {
    Expansion,
    Peak,
    Contraction,
    Trough,
    Recovery,
    Stagflation,
    Deflation,
    Overheating,
}
```

Each `RegimeType` maps to a default set of parameter adjustments (documented in Section 7) that modulate generation behavior when no specific events override them.

---

## 4. Provider Registry & Resolution

### 4.1 Extended Plugin Registry

The existing `PluginRegistry` is extended with a new provider category:

```rust
// Addition to PluginRegistry
pub struct PluginRegistry {
    generators: Arc<RwLock<HashMap<String, Arc<dyn GeneratorPlugin>>>>,
    sinks: Arc<RwLock<HashMap<String, Arc<RwLock<Box<dyn SinkPlugin>>>>>>,
    transforms: Arc<RwLock<HashMap<String, Arc<dyn TransformPlugin>>>>,
    // NEW
    historical_providers: Arc<RwLock<Vec<Arc<dyn HistoricalDataProvider>>>>,
}

impl PluginRegistry {
    pub fn register_historical_provider(
        &self,
        provider: Box<dyn HistoricalDataProvider>,
    ) -> Result<(), SynthError>;

    pub fn get_historical_providers(&self) -> Vec<Arc<dyn HistoricalDataProvider>>;
}
```

### 4.2 HistoricalDataResolver

When multiple providers are registered, a resolver determines which provider to query for each data point. This enables layering (e.g., a free FX provider + a premium sector events provider).

```rust
pub struct HistoricalDataResolver {
    providers: Vec<Arc<dyn HistoricalDataProvider>>,
    /// Priority order: later entries override earlier ones for overlapping capabilities
    priority: Vec<String>,
    /// Cache for resolved values (date → capability → value)
    cache: Arc<RwLock<LruCache<CacheKey, CachedValue>>>,
    /// Fallback behavior when no provider has data
    fallback: FallbackStrategy,
}

#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    /// Use existing synthetic models (default — preserves backward compatibility)
    SyntheticFallback,
    /// Interpolate from nearest available data points
    Interpolate { max_gap_days: u32 },
    /// Fail with an error
    Strict,
}
```

The resolver follows this resolution order for each query:

1. Check providers in priority order (highest priority first)
2. First provider that returns `Some(value)` wins
3. If all providers return `None`, apply `FallbackStrategy`
4. Cache the result for subsequent queries on the same date

### 4.3 Prefetch Optimization

During orchestrator initialization, the resolver prefetches data for the configured generation period:

```rust
impl HistoricalDataResolver {
    /// Called by the orchestrator before generation begins.
    /// Allows providers to bulk-load data for the date range.
    pub fn prefetch_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<PrefetchSummary, SynthError>;
}

pub struct PrefetchSummary {
    pub provider_id: String,
    pub capabilities_loaded: Vec<HistoricalDataCapability>,
    pub date_range_covered: DateRange,
    pub data_points_loaded: usize,
    pub gaps: Vec<DateRange>,  // periods where no data is available
}
```

---

## 5. Generator Integration Pattern

### 5.1 HistoricalContext

Generators receive historical context through a shared, read-only context object injected by the orchestrator:

```rust
/// Injected into generators to provide historical data access.
/// Immutable after construction — safe for concurrent generator access.
pub struct HistoricalContext {
    resolver: Arc<HistoricalDataResolver>,
    generation_start: NaiveDate,
    generation_end: NaiveDate,
    /// Pre-computed event timeline for the generation period
    event_timeline: Arc<EventTimeline>,
    /// Pre-computed regime timeline for the generation period
    regime_timeline: Arc<RegimeTimeline>,
}

impl HistoricalContext {
    /// Get the combined impact multiplier for a target on a given date.
    /// Aggregates all active events' impacts, applying transition curves.
    pub fn impact_multiplier(
        &self,
        target: &ImpactTarget,
        date: NaiveDate,
    ) -> f64;

    /// Get the FX rate, falling back to synthetic if unavailable.
    pub fn fx_rate_or_synthetic(
        &self,
        base: &str,
        quote: &str,
        date: NaiveDate,
        synthetic_fallback: impl FnOnce() -> Decimal,
    ) -> Decimal;

    /// Get the current economic regime for a date.
    pub fn regime(&self, date: NaiveDate) -> Option<&EconomicRegime>;

    /// Get all active events for a date, optionally filtered by sector.
    pub fn active_events(
        &self,
        date: NaiveDate,
        sector: Option<&str>,
    ) -> &[EconomicEvent];

    /// Check if historical data is available for this date range.
    pub fn has_data(&self, capability: &HistoricalDataCapability) -> bool;
}
```

### 5.2 EventTimeline (Pre-computed)

To avoid per-record provider lookups, the orchestrator pre-computes an event timeline during initialization:

```rust
pub struct EventTimeline {
    /// Events sorted by start_date
    events: Vec<EconomicEvent>,
    /// Pre-computed daily impact multipliers for each target.
    /// Key: (ImpactTarget, NaiveDate) → aggregated multiplier.
    daily_impacts: HashMap<ImpactTarget, BTreeMap<NaiveDate, f64>>,
}

impl EventTimeline {
    /// Build from a list of events, pre-computing daily impacts
    /// across the generation date range.
    pub fn build(
        events: Vec<EconomicEvent>,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Self;
}
```

### 5.3 Integration Examples

**FX Rate Service integration:**

```rust
// In fx_rate_service.rs — modified generate method
impl FxRateService {
    pub fn rate_for_date(
        &self,
        base: &str,
        quote: &str,
        date: NaiveDate,
    ) -> Decimal {
        if let Some(ctx) = &self.historical_context {
            ctx.fx_rate_or_synthetic(base, quote, date, || {
                self.synthetic_rate(base, quote, date)
            })
        } else {
            self.synthetic_rate(base, quote, date)
        }
    }
}
```

**Amount distribution modulation:**

```rust
// In je_generator.rs — when sampling amounts
let base_amount = self.amount_sampler.sample(&mut self.rng);
let amount = if let Some(ctx) = &self.historical_context {
    let multiplier = ctx.impact_multiplier(
        &ImpactTarget::TransactionAmountMean,
        posting_date,
    );
    base_amount * Decimal::from_f64(multiplier).unwrap_or(Decimal::ONE)
} else {
    base_amount
};
```

**Anomaly rate modulation:**

```rust
// In anomaly/injector.rs — when deciding whether to inject
let base_rate = self.config.injection_rate;
let effective_rate = if let Some(ctx) = &self.historical_context {
    let multiplier = ctx.impact_multiplier(
        &ImpactTarget::AnomalyRate,
        record_date,
    );
    (base_rate * multiplier).min(1.0)
} else {
    base_rate
};
```

---

## 6. Orchestrator Integration

### 6.1 Enhanced Orchestrator Changes

The `EnhancedOrchestrator` gains a new early phase for historical data initialization:

```rust
// In enhanced_orchestrator.rs
impl EnhancedOrchestrator {
    pub fn generate(&mut self) -> SynthResult<GenerationResult> {
        // NEW: Phase 0 — Historical Data Initialization
        let historical_context = self.phase_historical_data()?;

        // Existing phases receive historical_context
        self.phase_chart_of_accounts()?;
        self.phase_master_data(&historical_context)?;
        self.phase_document_flows(&historical_context)?;
        self.phase_journal_entries(&historical_context)?;
        self.phase_anomaly_injection(&historical_context)?;
        // ... remaining phases
    }

    fn phase_historical_data(&self) -> SynthResult<Option<Arc<HistoricalContext>>> {
        let providers = self.plugin_registry.get_historical_providers();
        if providers.is_empty() {
            return Ok(None); // No historical data — pure synthetic mode
        }

        let resolver = HistoricalDataResolver::new(providers, &self.config);
        let summary = resolver.prefetch_range(
            self.config.start_date(),
            self.config.end_date(),
        )?;

        self.report_prefetch_summary(&summary);

        let events = resolver.all_events(
            self.config.start_date(),
            self.config.end_date(),
            self.config.industry(),
        )?;

        let timeline = EventTimeline::build(
            events,
            self.config.start_date(),
            self.config.end_date(),
        );

        Ok(Some(Arc::new(HistoricalContext {
            resolver: Arc::new(resolver),
            generation_start: self.config.start_date(),
            generation_end: self.config.end_date(),
            event_timeline: Arc::new(timeline),
            regime_timeline: Arc::new(self.build_regime_timeline(&resolver)?),
        })))
    }
}
```

### 6.2 PhaseConfig Extension

```rust
pub struct PhaseConfig {
    // ... existing fields ...

    // NEW
    pub use_historical_data: bool,            // default: true when providers registered
    pub historical_fallback: FallbackStrategy, // default: SyntheticFallback
    pub historical_cache_size: usize,          // default: 10_000 entries
}
```

---

## 7. Regime-to-Parameter Default Mappings

When an `EconomicRegime` is active but no specific `EconomicEvent` overrides a target, the following default multipliers apply. These are configurable per-customer.

| Regime | TransactionVolume | AmountMean | AnomalyRate | PaymentDelay | MarketDemand | CreditRisk |
|--------|:-:|:-:|:-:|:-:|:-:|:-:|
| **Expansion** | 1.15 | 1.10 | 0.85 | 0.90 | 1.20 | 0.80 |
| **Peak** | 1.05 | 1.15 | 0.90 | 0.95 | 1.10 | 0.85 |
| **Contraction** | 0.80 | 0.85 | 1.30 | 1.40 | 0.70 | 1.50 |
| **Trough** | 0.70 | 0.75 | 1.50 | 1.60 | 0.60 | 1.80 |
| **Recovery** | 0.95 | 0.95 | 1.10 | 1.10 | 0.90 | 1.20 |
| **Stagflation** | 0.75 | 1.20 | 1.40 | 1.50 | 0.65 | 1.60 |
| **Deflation** | 0.85 | 0.70 | 1.20 | 1.30 | 0.75 | 1.30 |
| **Overheating** | 1.20 | 1.30 | 1.10 | 0.85 | 1.30 | 1.10 |

These defaults live in a `RegimeParameterMap` that customers can override:

```rust
pub struct RegimeParameterMap {
    defaults: HashMap<RegimeType, HashMap<ImpactTarget, f64>>,
}

impl RegimeParameterMap {
    pub fn standard() -> Self; // table above
    pub fn with_override(
        mut self,
        regime: RegimeType,
        target: ImpactTarget,
        multiplier: f64,
    ) -> Self;
}
```

---

## 8. Bundled & Commercial Providers

### 8.1 Provider Tiers

| Tier | Provider | Data | Licensing |
|------|----------|------|-----------|
| **Free (bundled)** | `BundledEconomicEventsProvider` | Major global recessions, crises, pandemics (50+ events, 2000-present). NBER recession dates. | Public domain / CC0 |
| **Free (bundled)** | `BundledFxProvider` | Monthly average FX rates for 20 major currency pairs (ECB reference rates). | ECB open data license |
| **Standard** | `DetailedFxProvider` | Daily FX rates for 150+ pairs from central banks. | Commercial license |
| **Standard** | `InterestRateProvider` | Daily central bank rates, SOFR, EURIBOR, yield curves. | Commercial license |
| **Standard** | `CommodityPriceProvider` | Daily commodity prices (energy, metals, agriculture). | Commercial license |
| **Premium** | `SectorEventProvider` | Sector-specific events with calibrated impacts (see 8.3). | Commercial license |
| **Premium** | `MacroRegimeProvider` | GDP, CPI, unemployment, PMI → regime classification. | Commercial license |
| **Enterprise** | `RealTimeFeedProvider` | Live API integration (Bloomberg, Refinitiv, FRED). | Customer's data license |
| **BYOD** | `CustomFileProvider` | Customer-supplied CSV/JSON files. | N/A |

### 8.2 BundledEconomicEventsProvider (Free Tier)

Ships with the binary. Contains curated events from publicly available data:

```
events/
├── global/
│   ├── recessions.yaml        # NBER + CEPR recession dates
│   ├── financial_crises.yaml  # 2008 GFC, 2010 Euro debt, etc.
│   ├── pandemics.yaml         # COVID-19 with phased impacts
│   └── geopolitical.yaml      # Major conflicts, trade wars
├── regional/
│   ├── us.yaml
│   ├── eu.yaml
│   └── asia_pacific.yaml
└── schema.yaml                # Event file format specification
```

Example bundled event:

```yaml
- event_id: covid-19-pandemic
  name: "COVID-19 Global Pandemic"
  event_type: Pandemic
  start_date: "2020-03-01"
  end_date: "2021-06-30"
  regions: []  # global
  sectors: []  # all sectors
  severity: 0.9
  description: "Global pandemic causing widespread economic disruption"
  source: "NBER, WHO"
  impacts:
    - target: TransactionVolume
      effect: { Multiplier: 0.60 }
      onset: { Exponential: { half_life_days: 14 } }
      recovery: { Sigmoid: { midpoint_days: 180, steepness: 0.03 } }
    - target: PaymentDelay
      effect: { Multiplier: 1.80 }
      onset: { Linear: { days: 21 } }
      recovery: { Linear: { days: 120 } }
    - target: AnomalyRate
      effect: { Multiplier: 1.40 }
      onset: { Linear: { days: 30 } }
      recovery: { Exponential: { half_life_days: 90 } }
    - target: MarketDemand
      effect: { Multiplier: 0.55 }
      onset: { Exponential: { half_life_days: 7 } }
      recovery: { Sigmoid: { midpoint_days: 270, steepness: 0.02 } }
```

### 8.3 Sector Event Packs (Premium)

Sold as add-on packs per industry vertical. Each pack contains sector-specific events with calibrated impacts:

| Pack | Example Events |
|------|---------------|
| **Manufacturing** | Semiconductor shortage (2020-2023), steel tariffs (2018), Suez Canal blockage (2021), automotive chip crisis, rare earth supply disruptions |
| **Financial Services** | SVB collapse (2023), LIBOR transition, Basel III implementation, crypto winter (2022), meme stock volatility (2021) |
| **Retail / E-commerce** | Holiday demand spikes (calibrated by year), Amazon Prime Day effects, supply chain bottlenecks, consumer confidence shifts |
| **Energy** | Oil price wars (2020), EU energy crisis (2022), renewable transition milestones, OPEC decisions |
| **Healthcare / Pharma** | Drug patent cliffs, FDA approvals, pandemic-driven demand surges, regulatory changes |
| **Technology** | Cloud migration waves, AI investment cycles, data privacy regulations (GDPR, CCPA) |
| **Real Estate** | Interest rate cycle impacts, commercial vacancy trends, housing market corrections |

### 8.4 CustomFileProvider (BYOD)

Customers can supply their own historical data in standardized formats:

```rust
pub struct CustomFileProvider {
    /// Directory containing customer data files
    data_dir: PathBuf,
    /// Loaded time series indexed by type
    fx_rates: HashMap<CurrencyPair, BTreeMap<NaiveDate, Decimal>>,
    events: Vec<EconomicEvent>,
    interest_rates: HashMap<String, BTreeMap<NaiveDate, Decimal>>,
}

impl CustomFileProvider {
    /// Load from a directory of CSV/YAML files following the schema
    pub fn from_directory(path: &Path) -> Result<Self, SynthError>;
}
```

Supported file formats:

**FX rates (CSV):**
```csv
date,base,quote,rate
2020-01-02,EUR,USD,1.1213
2020-01-03,EUR,USD,1.1163
```

**Events (YAML):** Same schema as bundled events (see 8.2).

**Interest rates (CSV):**
```csv
date,rate_type,rate
2020-01-02,fed_funds,1.75
2020-01-03,fed_funds,1.75
```

---

## 9. Configuration

### 9.1 YAML Configuration Schema

New top-level section in the DataSynth configuration:

```yaml
historical_data:
  enabled: true

  # Provider configuration (multiple providers can be active simultaneously)
  providers:
    # Bundled free providers (always available)
    - type: bundled_events
      enabled: true

    - type: bundled_fx
      enabled: true

    # Premium provider (requires license key)
    - type: sector_events
      enabled: true
      license_key: "${DATASYNTH_SECTOR_LICENSE}"
      packs:
        - manufacturing
        - financial_services

    # Standard provider (requires license key)
    - type: detailed_fx
      enabled: true
      license_key: "${DATASYNTH_FX_LICENSE}"
      pairs: [EUR/USD, GBP/USD, USD/JPY, USD/CHF]  # empty = all available

    # Live API feed (enterprise)
    - type: api_feed
      enabled: false
      endpoint: "https://api.example.com/v1/historical"
      api_key: "${DATASYNTH_FEED_API_KEY}"
      cache_dir: "./cache/historical"
      cache_ttl_hours: 24

    # Customer-supplied data (BYOD)
    - type: custom_file
      enabled: true
      data_dir: "./historical_data"
      # Files expected: fx_rates.csv, events.yaml, interest_rates.csv

  # Resolution behavior
  resolution:
    priority: [custom_file, api_feed, sector_events, detailed_fx, bundled_fx, bundled_events]
    fallback: synthetic    # synthetic | interpolate | strict
    interpolation_max_gap_days: 5
    cache_size: 10000

  # Impact scaling — globally scale all historical impacts
  impact_scaling:
    global_multiplier: 1.0        # 1.0 = use impacts as-is; 0.5 = half strength
    per_target_overrides:
      TransactionVolume: 1.0
      AnomalyRate: 0.8            # slightly dampen anomaly rate effects
      PaymentDelay: 1.2           # amplify payment delay effects

  # Regime parameter overrides
  regime_overrides:
    Contraction:
      TransactionVolume: 0.85     # less severe than default 0.80
    Trough:
      CreditRisk: 2.00            # more severe than default 1.80

  # Selective feature enablement
  features:
    fx_rates: true                # use historical FX rates
    interest_rates: true          # use historical interest rates
    commodity_prices: false       # keep synthetic commodity prices
    economic_events: true         # apply economic event impacts
    economic_regimes: true        # apply regime-based parameter shifts
    sector_events: true           # apply sector-specific events
```

### 9.2 Config Schema Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub resolution: ResolutionConfig,
    #[serde(default)]
    pub impact_scaling: ImpactScalingConfig,
    #[serde(default)]
    pub regime_overrides: HashMap<String, HashMap<String, f64>>,
    #[serde(default)]
    pub features: HistoricalFeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    #[serde(rename = "bundled_events")]
    BundledEvents { enabled: bool },
    #[serde(rename = "bundled_fx")]
    BundledFx { enabled: bool },
    #[serde(rename = "sector_events")]
    SectorEvents {
        enabled: bool,
        license_key: Option<String>,
        packs: Vec<String>,
    },
    #[serde(rename = "detailed_fx")]
    DetailedFx {
        enabled: bool,
        license_key: Option<String>,
        pairs: Vec<String>,
    },
    #[serde(rename = "api_feed")]
    ApiFeed {
        enabled: bool,
        endpoint: String,
        api_key: Option<String>,
        cache_dir: Option<PathBuf>,
        cache_ttl_hours: Option<u64>,
    },
    #[serde(rename = "custom_file")]
    CustomFile {
        enabled: bool,
        data_dir: PathBuf,
    },
}
```

### 9.3 Validation Rules

Added to `datasynth-config/src/validation.rs`:

- `impact_scaling.global_multiplier`: must be ≥ 0.0 and ≤ 10.0
- `per_target_overrides`: values must be ≥ 0.0 and ≤ 10.0
- `resolution.interpolation_max_gap_days`: must be 1-365
- `resolution.cache_size`: must be 100-1,000,000
- `providers`: no duplicate types (except `custom_file` which allows multiple)
- `custom_file.data_dir`: must exist and contain at least one recognized file
- `license_key`: validated against license server on provider initialization
- `api_feed.endpoint`: must be a valid HTTPS URL
- Generation date range must overlap with at least one provider's `available_range()` (warning, not error)

---

## 10. Crate Structure

### 10.1 New Crate: `datasynth-historical`

```
crates/datasynth-historical/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── traits.rs              # HistoricalDataProvider trait, supporting types
│   ├── resolver.rs            # HistoricalDataResolver, caching, fallback
│   ├── context.rs             # HistoricalContext, EventTimeline, RegimeTimeline
│   ├── regime.rs              # RegimeParameterMap, regime-to-impact defaults
│   ├── transition.rs          # TransitionCurve evaluation (onset/recovery math)
│   ├── providers/
│   │   ├── mod.rs
│   │   ├── bundled_events.rs  # BundledEconomicEventsProvider (free tier)
│   │   ├── bundled_fx.rs      # BundledFxProvider (free tier)
│   │   ├── custom_file.rs     # CustomFileProvider (BYOD)
│   │   └── api_feed.rs        # ApiFeedProvider (enterprise, feature-gated)
│   ├── data/
│   │   ├── events/            # Bundled YAML event files (compiled in)
│   │   │   ├── global/
│   │   │   └── regional/
│   │   └── fx/                # Bundled monthly FX CSV (compiled in)
│   └── validation.rs          # Config validation for historical_data section
├── tests/
│   ├── provider_tests.rs
│   ├── resolver_tests.rs
│   ├── timeline_tests.rs
│   └── integration_tests.rs
└── benches/
    └── timeline_bench.rs      # Benchmark impact_multiplier lookups
```

### 10.2 Feature Flags

```toml
[package]
name = "datasynth-historical"

[features]
default = ["bundled"]
bundled = []                    # Include bundled event/FX data (free tier)
api-feed = ["dep:reqwest"]      # Enable API feed provider
premium = []                    # Enable premium provider stubs (license-checked)

[dependencies]
datasynth-core = { path = "../datasynth-core" }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1", features = ["serde-str"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
lru = "0.12"
reqwest = { version = "0.12", features = ["json"], optional = true }
```

### 10.3 Dependency Graph

```
datasynth-config ──depends on──▶ datasynth-historical (for HistoricalDataConfig)
datasynth-historical ──depends on──▶ datasynth-core (for SynthError, traits, Decimal)
datasynth-generators ──depends on──▶ datasynth-historical (for HistoricalContext)
datasynth-runtime ──depends on──▶ datasynth-historical (for resolver, provider init)
```

---

## 11. Commercial Licensing Model

### 11.1 License Verification

Premium and standard providers verify licenses at initialization time:

```rust
pub struct LicenseInfo {
    pub tier: LicenseTier,
    pub customer_id: String,
    pub valid_until: NaiveDate,
    pub entitled_packs: Vec<String>,
    pub max_generation_years: Option<u32>,  // None = unlimited
}

pub enum LicenseTier {
    Free,       // bundled providers only
    Standard,   // + detailed FX, interest rates, commodities
    Premium,    // + sector event packs
    Enterprise, // + API feeds, unlimited history, priority support
}
```

License keys are verified via:
1. **Offline:** Signed JWT tokens with embedded entitlements (no network required)
2. **Online (optional):** License server check at `https://license.datasynth.dev/v1/verify`

### 11.2 Data Pack Distribution

Data packs are distributed as versioned, signed archives:

```
datasynth-historical-manufacturing-v2.1.0.dshp  (DataSynth Historical Pack)
├── manifest.yaml      # pack metadata, version, checksum
├── events.yaml        # sector-specific events
├── impacts.yaml       # calibrated impact parameters
├── signature.sig      # Ed25519 signature for integrity verification
└── LICENSE             # data attribution and usage terms
```

Packs can be:
- **Bundled at build time** (compiled into the binary)
- **Loaded at runtime** from a configurable directory (`DATASYNTH_PACKS_DIR`)
- **Downloaded on demand** via CLI: `datasynth-data packs install manufacturing`

### 11.3 Pricing Structure (Suggested)

| Tier | Includes | Model |
|------|----------|-------|
| **Free** | Bundled events (50+ major global events), monthly FX for 20 pairs | Included with DataSynth |
| **Standard** | Daily FX (150+ pairs), interest rates, commodity prices | Per-seat annual subscription |
| **Premium** | All Standard + 1-3 sector packs | Per-seat annual subscription |
| **Enterprise** | All Premium + unlimited packs + API feeds + custom calibration | Annual contract |

---

## 12. CLI & Server API Extensions

### 12.1 CLI Commands

```bash
# List installed providers and their status
datasynth-data historical list

# Show available date range and capabilities for all providers
datasynth-data historical info

# Validate that providers have data covering the config's date range
datasynth-data historical validate --config config.yaml

# Install a data pack (premium/standard — requires license key)
datasynth-data historical install manufacturing --license-key $KEY

# Update bundled data to latest version
datasynth-data historical update

# Preview event impacts for a date range (diagnostic)
datasynth-data historical preview --start 2020-01-01 --end 2021-12-31 --sector manufacturing

# Import custom data
datasynth-data historical import --type fx --file ./my_fx_rates.csv
datasynth-data historical import --type events --file ./my_events.yaml
```

### 12.2 Server REST API

```
GET  /api/historical/providers         → list registered providers
GET  /api/historical/capabilities      → combined capability matrix
GET  /api/historical/events            → query events by date range / sector
GET  /api/historical/fx/{base}/{quote} → FX rate time series
GET  /api/historical/regime            → regime classification time series
POST /api/historical/preview           → preview impacts for a config
POST /api/historical/providers         → register a custom provider at runtime
```

### 12.3 WebSocket Events

New event types on the `/ws/events` channel:

```json
{ "type": "historical_data_loaded", "providers": 3, "events": 47, "date_range": "2019-01-01/2022-12-31" }
{ "type": "historical_event_active", "event_id": "covid-19-pandemic", "date": "2020-03-15", "severity": 0.9 }
{ "type": "historical_regime_change", "from": "Expansion", "to": "Contraction", "date": "2020-03-01" }
{ "type": "historical_fallback", "capability": "CommodityPrices", "date": "2020-06-15", "reason": "no_provider" }
```

---

## 13. Evaluation Integration

The existing `datasynth-eval` module gains new validation tests for historically-grounded data:

| Test | Description |
|------|-------------|
| **FX rate accuracy** | Compare generated FX rates against provider data; report MAE, max deviation |
| **Event impact coherence** | Verify that transaction volumes, anomaly counts, and payment delays shift in the expected direction during known events |
| **Regime consistency** | Check that regime transitions produce the expected parameter shifts (e.g., contraction → lower volume) |
| **Temporal alignment** | Validate that regime changes and event boundaries align with configured dates (no off-by-one errors) |
| **Fallback coverage** | Report percentage of generation dates served by historical data vs. synthetic fallback |
| **Cross-provider consistency** | When multiple providers supply overlapping data, verify agreement within tolerance |

These feed into the existing `AutoTuner` which can generate config patches to adjust `impact_scaling` based on evaluation gaps.

---

## 14. Performance Considerations

| Concern | Mitigation |
|---------|------------|
| **Startup latency** | Prefetch runs once during orchestrator init; amortized across millions of generated records |
| **Per-record lookup cost** | `EventTimeline` pre-computes daily impacts into `BTreeMap`; O(log n) lookup per date |
| **Memory for time series** | 10 years of daily FX rates for 20 pairs ≈ 3 MB; well within budget |
| **API feed latency** | Local cache with configurable TTL; generation never blocks on network after prefetch |
| **Bundled data binary size** | Compressed YAML events ~50 KB; monthly FX CSV ~200 KB; negligible impact |
| **Premium pack loading** | Lazy-loaded from disk on first access; not compiled into binary |

### 14.1 Benchmarks (Target)

- `impact_multiplier()` lookup: < 100 ns per call (pre-computed BTreeMap)
- Prefetch for 10-year range with bundled providers: < 10 ms
- Full generation throughput with historical data: < 5% regression vs. pure synthetic

---

## 15. Implementation Plan

### Phase 1: Core Infrastructure (Foundation)

1. Create `datasynth-historical` crate with trait definitions
2. Implement `HistoricalDataResolver` with caching and fallback
3. Implement `HistoricalContext` and `EventTimeline` pre-computation
4. Implement `TransitionCurve` evaluation (onset/recovery math)
5. Implement `RegimeParameterMap` with default mappings
6. Add `HistoricalDataConfig` to `datasynth-config` schema and validation
7. Add unit tests for all core types

### Phase 2: Bundled Providers (Free Tier)

1. Curate 50+ global economic events (2000-present) into YAML
2. Compile ECB monthly reference rates for 20 currency pairs
3. Implement `BundledEconomicEventsProvider`
4. Implement `BundledFxProvider`
5. Implement `CustomFileProvider` (BYOD)
6. Integration tests with bundled data

### Phase 3: Generator Integration

1. Inject `HistoricalContext` into `EnhancedOrchestrator` phase pipeline
2. Integrate with `FxRateService` (historical rate lookup with synthetic fallback)
3. Integrate with `JeGenerator` (amount distribution modulation)
4. Integrate with `AnomalyInjector` (event-driven anomaly rate scaling)
5. Integrate with `DriftConfig` (auto-populate regime changes from events)
6. Integrate with P2P/O2C generators (payment delay, lead time modulation)
7. End-to-end integration tests

### Phase 4: CLI, Server & Evaluation

1. Add `datasynth-data historical` CLI subcommands
2. Add server REST API endpoints for historical data queries
3. Add WebSocket event types for historical data progress
4. Add evaluation tests for FX accuracy and event impact coherence
5. Performance benchmarks

### Phase 5: Commercial Providers (Standard/Premium)

1. Implement license verification (offline JWT + optional online check)
2. Implement data pack format (.dshp) with signature verification
3. Build `DetailedFxProvider`, `InterestRateProvider`, `CommodityPriceProvider`
4. Build sector event packs (manufacturing, financial services, retail)
5. Implement `ApiFeedProvider` for enterprise live data feeds
6. Pack distribution infrastructure (CLI install, update)

### Phase 6: Advanced Features

1. Impact calibration tooling (tune event impacts from real datasets)
2. AutoTuner integration (evaluate and auto-adjust impact scaling)
3. Python wrapper support for historical data configuration
4. Desktop UI integration (event timeline visualization, provider management)

---

## 16. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Data licensing constraints | Some historical data sources have restrictive licenses | Bundled tier uses only public-domain/open-data sources; commercial tiers negotiate redistribution rights |
| Event impact calibration accuracy | Poorly calibrated impacts produce unrealistic data | Provide conservative defaults; AutoTuner can refine; customers can override any parameter |
| Provider API reliability | Enterprise feeds may have outages | Mandatory local cache; prefetch at init time; synthetic fallback always available |
| Binary size growth | Bundled data adds to binary | Events YAML is small (~50 KB compressed); FX data is modest (~200 KB); large datasets stay external |
| Backward compatibility | Existing configs must work unchanged | `historical_data` section is entirely optional; all generators check `if let Some(ctx)` before using |
| Determinism with live feeds | API data may change between runs | Cache with content-addressed keys; document that determinism requires pinned data versions |

---

## 17. Open Questions

1. **Granularity of bundled FX data:** Monthly averages (smaller, simpler) vs. daily close prices (more accurate but ~12x larger)?
2. **Event overlap resolution:** When multiple events affect the same target on the same date, multiply their effects or take the maximum?
3. **Regional scoping:** Should events automatically filter by company country code, or require explicit sector/region matching in config?
4. **Python wrapper ergonomics:** Should BYOD data be passable as pandas DataFrames directly, or only via file paths?
5. **Pack versioning:** Semantic versioning for packs, or date-based versioning tied to the data coverage period?
