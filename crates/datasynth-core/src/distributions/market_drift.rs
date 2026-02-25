//! Market drift models for economic and industry cycle simulation.
//!
//! Provides comprehensive market drift modeling including:
//! - Economic cycles (sinusoidal, asymmetric, mean-reverting)
//! - Industry-specific cycles
//! - Commodity price drift
//! - Price shock events

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main market drift model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketDriftModel {
    /// Economic cycle model.
    #[serde(default)]
    pub economic_cycle: EconomicCycleModel,
    /// Industry-specific cycles.
    #[serde(default)]
    pub industry_cycles: HashMap<MarketIndustryType, IndustryCycleConfig>,
    /// Commodity drift configuration.
    #[serde(default)]
    pub commodity_drift: CommodityDriftConfig,
    /// Price shock events.
    #[serde(default)]
    pub price_shocks: Vec<PriceShockEvent>,
}

impl MarketDriftModel {
    /// Compute market effects for a given period.
    pub fn compute_effects(&self, period: u32, rng: &mut ChaCha8Rng) -> MarketEffects {
        let mut effects = MarketEffects::neutral();

        // Economic cycle
        if self.economic_cycle.enabled {
            let cycle_effect = self.economic_cycle.effect_at_period(period);
            effects.economic_cycle_factor = cycle_effect.cycle_factor;
            effects.is_recession = cycle_effect.is_recession;
        }

        // Commodity effects
        if self.commodity_drift.enabled {
            effects.commodity_effects = self.commodity_drift.effects_at_period(period, rng);
        }

        // Price shock effects
        for shock in &self.price_shocks {
            if shock.is_active_at_period(period) {
                effects.apply_shock(shock, period);
            }
        }

        effects
    }
}

/// Industry type for industry-specific market cycles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketIndustryType {
    /// Technology sector.
    Technology,
    /// Retail sector.
    Retail,
    /// Manufacturing sector.
    Manufacturing,
    /// Financial services sector.
    FinancialServices,
    /// Healthcare sector.
    Healthcare,
    /// Energy sector.
    Energy,
    /// Real estate sector.
    RealEstate,
}

impl MarketIndustryType {
    /// Get the typical cycle period for this industry.
    pub fn typical_cycle_months(&self) -> u32 {
        match self {
            Self::Technology => 36,
            Self::Retail => 12,
            Self::Manufacturing => 48,
            Self::FinancialServices => 60,
            Self::Healthcare => 36,
            Self::Energy => 48,
            Self::RealEstate => 84,
        }
    }

    /// Get the typical cycle amplitude for this industry.
    pub fn typical_amplitude(&self) -> f64 {
        match self {
            Self::Technology => 0.25,
            Self::Retail => 0.35,
            Self::Manufacturing => 0.20,
            Self::FinancialServices => 0.15,
            Self::Healthcare => 0.10,
            Self::Energy => 0.30,
            Self::RealEstate => 0.20,
        }
    }
}

/// Market effects computed for a period.
#[derive(Debug, Clone, Default)]
pub struct MarketEffects {
    /// Economic cycle factor (1.0 = neutral).
    pub economic_cycle_factor: f64,
    /// Whether in recession.
    pub is_recession: bool,
    /// Commodity price effects.
    pub commodity_effects: CommodityEffects,
    /// Active price shocks.
    pub active_shocks: Vec<String>,
    /// Price shock multiplier.
    pub shock_multiplier: f64,
}

impl MarketEffects {
    /// Create neutral effects.
    pub fn neutral() -> Self {
        Self {
            economic_cycle_factor: 1.0,
            is_recession: false,
            commodity_effects: CommodityEffects::default(),
            active_shocks: Vec::new(),
            shock_multiplier: 1.0,
        }
    }

    /// Apply a price shock.
    fn apply_shock(&mut self, shock: &PriceShockEvent, period: u32) {
        self.active_shocks.push(shock.shock_id.clone());
        let progress = shock.progress_at_period(period);
        let shock_factor = 1.0
            + shock.price_increase_range.0
            + (shock.price_increase_range.1 - shock.price_increase_range.0) * progress;
        self.shock_multiplier *= shock_factor;
    }
}

// =============================================================================
// Economic Cycle Model
// =============================================================================

/// Economic cycle model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicCycleModel {
    /// Enable economic cycle.
    #[serde(default)]
    pub enabled: bool,
    /// Cycle type.
    #[serde(default)]
    pub cycle_type: CycleType,
    /// Cycle period in months.
    #[serde(default = "default_cycle_period")]
    pub period_months: u32,
    /// Amplitude of cycle effect.
    #[serde(default = "default_amplitude")]
    pub amplitude: f64,
    /// Phase offset in months.
    #[serde(default)]
    pub phase_offset: u32,
    /// Recession configuration.
    #[serde(default)]
    pub recession: RecessionConfig,
}

fn default_cycle_period() -> u32 {
    48
}

fn default_amplitude() -> f64 {
    0.15
}

impl Default for EconomicCycleModel {
    fn default() -> Self {
        Self {
            enabled: false,
            cycle_type: CycleType::Sinusoidal,
            period_months: 48,
            amplitude: 0.15,
            phase_offset: 0,
            recession: RecessionConfig::default(),
        }
    }
}

impl EconomicCycleModel {
    /// Calculate the cycle effect at a given period.
    pub fn effect_at_period(&self, period: u32) -> CycleEffect {
        if !self.enabled {
            return CycleEffect {
                cycle_factor: 1.0,
                is_recession: false,
                cycle_position: 0.0,
            };
        }

        let adjusted_period = period + self.phase_offset;
        let cycle_position =
            (adjusted_period % self.period_months) as f64 / self.period_months as f64;

        let base_factor = match self.cycle_type {
            CycleType::Sinusoidal => {
                let radians = cycle_position * 2.0 * std::f64::consts::PI;
                1.0 + self.amplitude * radians.sin()
            }
            CycleType::Asymmetric => {
                // Faster decline, slower recovery
                let radians = cycle_position * 2.0 * std::f64::consts::PI;
                let sine_value = radians.sin();
                if sine_value < 0.0 {
                    1.0 + self.amplitude * sine_value * 1.3 // Deeper troughs
                } else {
                    1.0 + self.amplitude * sine_value * 0.7 // Shallower peaks
                }
            }
            CycleType::MeanReverting => {
                // Oscillates with dampening
                let radians = cycle_position * 2.0 * std::f64::consts::PI;
                let dampening = (-cycle_position * 0.5).exp();
                1.0 + self.amplitude * radians.sin() * dampening
            }
        };

        // Check for recession
        let is_recession = self.recession.enabled && self.recession.is_recession_at(period);
        let recession_factor = if is_recession {
            match self.recession.severity {
                RecessionSeverity::Mild => 0.90,
                RecessionSeverity::Moderate => 0.80,
                RecessionSeverity::Severe => 0.65,
            }
        } else {
            1.0
        };

        CycleEffect {
            cycle_factor: base_factor * recession_factor,
            is_recession,
            cycle_position,
        }
    }
}

/// Cycle type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CycleType {
    /// Simple sinusoidal cycle.
    #[default]
    Sinusoidal,
    /// Asymmetric cycle (faster decline, slower recovery).
    Asymmetric,
    /// Mean-reverting with dampening.
    MeanReverting,
}

/// Cycle effect at a point in time.
#[derive(Debug, Clone)]
pub struct CycleEffect {
    /// Cycle factor (multiplier).
    pub cycle_factor: f64,
    /// Whether in recession.
    pub is_recession: bool,
    /// Position in cycle (0.0 to 1.0).
    pub cycle_position: f64,
}

/// Recession configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecessionConfig {
    /// Enable recession simulation.
    #[serde(default)]
    pub enabled: bool,
    /// Probability of recession per year.
    #[serde(default = "default_recession_prob")]
    pub probability_per_year: f64,
    /// Recession onset type.
    #[serde(default)]
    pub onset: RecessionOnset,
    /// Duration range in months.
    #[serde(default = "default_recession_duration")]
    pub duration_months: (u32, u32),
    /// Recession severity.
    #[serde(default)]
    pub severity: RecessionSeverity,
    /// Specific recession periods (optional, for deterministic simulation).
    #[serde(default)]
    pub recession_periods: Vec<(u32, u32)>, // (start_month, duration)
}

fn default_recession_prob() -> f64 {
    0.10
}

fn default_recession_duration() -> (u32, u32) {
    (12, 24)
}

impl Default for RecessionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            probability_per_year: 0.10,
            onset: RecessionOnset::Gradual,
            duration_months: (12, 24),
            severity: RecessionSeverity::Moderate,
            recession_periods: Vec::new(),
        }
    }
}

impl RecessionConfig {
    /// Check if a given period is in recession.
    pub fn is_recession_at(&self, period: u32) -> bool {
        for (start, duration) in &self.recession_periods {
            if period >= *start && period < start + duration {
                return true;
            }
        }
        false
    }
}

/// Recession onset type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecessionOnset {
    /// Gradual onset over several months.
    #[default]
    Gradual,
    /// Sudden onset (e.g., crisis).
    Sudden,
}

/// Recession severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecessionSeverity {
    /// Mild recession (10% reduction).
    Mild,
    /// Moderate recession (20% reduction).
    #[default]
    Moderate,
    /// Severe recession (35% reduction).
    Severe,
}

// =============================================================================
// Industry Cycles
// =============================================================================

/// Industry-specific cycle configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryCycleConfig {
    /// Cycle period in months.
    #[serde(default = "default_industry_period")]
    pub period_months: u32,
    /// Cycle amplitude.
    #[serde(default = "default_industry_amplitude")]
    pub amplitude: f64,
    /// Phase offset relative to economic cycle.
    #[serde(default)]
    pub phase_offset: u32,
    /// Correlation with general economy.
    #[serde(default = "default_correlation")]
    pub economic_correlation: f64,
}

fn default_industry_period() -> u32 {
    36
}

fn default_industry_amplitude() -> f64 {
    0.20
}

fn default_correlation() -> f64 {
    0.7
}

impl Default for IndustryCycleConfig {
    fn default() -> Self {
        Self {
            period_months: 36,
            amplitude: 0.20,
            phase_offset: 0,
            economic_correlation: 0.7,
        }
    }
}

// =============================================================================
// Commodity Drift
// =============================================================================

/// Commodity drift configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommodityDriftConfig {
    /// Enable commodity drift.
    #[serde(default)]
    pub enabled: bool,
    /// Commodity configurations.
    #[serde(default)]
    pub commodities: Vec<CommodityConfig>,
}

impl CommodityDriftConfig {
    /// Calculate commodity effects at a period.
    pub fn effects_at_period(&self, period: u32, rng: &mut ChaCha8Rng) -> CommodityEffects {
        let mut effects = CommodityEffects::default();

        for commodity in &self.commodities {
            let price_factor = commodity.price_factor_at(period, rng);
            effects
                .price_factors
                .insert(commodity.name.clone(), price_factor);

            // Calculate pass-through effect on costs
            effects.cogs_impact += (price_factor - 1.0) * commodity.cogs_pass_through;
            effects.overhead_impact += (price_factor - 1.0) * commodity.overhead_pass_through;
        }

        effects
    }
}

/// Commodity configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommodityConfig {
    /// Commodity name.
    pub name: String,
    /// Base price.
    #[serde(default = "default_base_price")]
    pub base_price: f64,
    /// Price volatility (standard deviation as fraction of price).
    #[serde(default = "default_volatility")]
    pub volatility: f64,
    /// Correlation with economic cycle.
    #[serde(default = "default_econ_correlation")]
    pub economic_correlation: f64,
    /// Pass-through to COGS (fraction).
    #[serde(default)]
    pub cogs_pass_through: f64,
    /// Pass-through to overhead (fraction).
    #[serde(default)]
    pub overhead_pass_through: f64,
}

fn default_base_price() -> f64 {
    100.0
}

fn default_volatility() -> f64 {
    0.20
}

fn default_econ_correlation() -> f64 {
    0.5
}

impl CommodityConfig {
    /// Calculate price factor at a period.
    pub fn price_factor_at(&self, period: u32, rng: &mut ChaCha8Rng) -> f64 {
        // Mean-reverting random walk
        let random: f64 = rng.random();
        let z_score = (random - 0.5) * 2.0; // Approximate normal
        let price_change = z_score * self.volatility;

        // Trend component (slight mean reversion)
        let trend = -0.01 * period as f64 / 12.0;

        1.0 + price_change + trend
    }
}

/// Commodity effects.
#[derive(Debug, Clone, Default)]
pub struct CommodityEffects {
    /// Price factors by commodity name.
    pub price_factors: HashMap<String, f64>,
    /// Impact on COGS.
    pub cogs_impact: f64,
    /// Impact on overhead.
    pub overhead_impact: f64,
}

// =============================================================================
// Price Shocks
// =============================================================================

/// Price shock event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceShockEvent {
    /// Shock identifier.
    pub shock_id: String,
    /// Shock type.
    pub shock_type: PriceShockType,
    /// Start period.
    pub start_period: u32,
    /// Duration in months.
    pub duration_months: u32,
    /// Price increase range (min, max) as fraction.
    #[serde(default = "default_price_increase")]
    pub price_increase_range: (f64, f64),
    /// Affected categories.
    #[serde(default)]
    pub affected_categories: Vec<String>,
}

fn default_price_increase() -> (f64, f64) {
    (0.10, 0.30)
}

impl PriceShockEvent {
    /// Check if shock is active at a period.
    pub fn is_active_at_period(&self, period: u32) -> bool {
        period >= self.start_period && period < self.start_period + self.duration_months
    }

    /// Get progress through the shock (0.0 to 1.0).
    pub fn progress_at_period(&self, period: u32) -> f64 {
        if !self.is_active_at_period(period) {
            return 0.0;
        }
        let elapsed = period - self.start_period;
        elapsed as f64 / self.duration_months as f64
    }
}

/// Price shock type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PriceShockType {
    /// Supply chain disruption.
    #[default]
    SupplyDisruption,
    /// Demand surge.
    DemandSurge,
    /// Regulatory change.
    RegulatoryChange,
    /// Geopolitical event.
    GeopoliticalEvent,
    /// Natural disaster.
    NaturalDisaster,
}

impl PriceShockType {
    /// Get typical duration range for this shock type.
    pub fn typical_duration_months(&self) -> (u32, u32) {
        match self {
            Self::SupplyDisruption => (3, 12),
            Self::DemandSurge => (2, 6),
            Self::RegulatoryChange => (6, 24),
            Self::GeopoliticalEvent => (6, 18),
            Self::NaturalDisaster => (1, 6),
        }
    }
}

// =============================================================================
// Market Drift Controller
// =============================================================================

/// Market drift controller.
pub struct MarketDriftController {
    model: MarketDriftModel,
    rng: ChaCha8Rng,
}

impl MarketDriftController {
    /// Create a new market drift controller.
    pub fn new(model: MarketDriftModel, seed: u64) -> Self {
        Self {
            model,
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Compute market effects for a period.
    pub fn compute_effects(&mut self, period: u32) -> MarketEffects {
        self.model.compute_effects(period, &mut self.rng)
    }

    /// Get the underlying model.
    pub fn model(&self) -> &MarketDriftModel {
        &self.model
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sinusoidal_cycle() {
        let model = EconomicCycleModel {
            enabled: true,
            cycle_type: CycleType::Sinusoidal,
            period_months: 12,
            amplitude: 0.20,
            phase_offset: 0,
            recession: RecessionConfig::default(),
        };

        let effect_0 = model.effect_at_period(0);
        let effect_3 = model.effect_at_period(3); // 25% through cycle (peak)
        let effect_9 = model.effect_at_period(9); // 75% through cycle (trough)

        // At start, factor should be near 1.0
        assert!((effect_0.cycle_factor - 1.0).abs() < 0.1);
        // At peak, factor should be above 1.0
        assert!(effect_3.cycle_factor > 1.0);
        // At trough, factor should be below 1.0
        assert!(effect_9.cycle_factor < 1.0);
    }

    #[test]
    fn test_recession() {
        let model = EconomicCycleModel {
            enabled: true,
            cycle_type: CycleType::Sinusoidal,
            period_months: 48,
            amplitude: 0.15,
            phase_offset: 0,
            recession: RecessionConfig {
                enabled: true,
                severity: RecessionSeverity::Moderate,
                recession_periods: vec![(12, 6)], // Recession from month 12-17
                ..Default::default()
            },
        };

        let effect_10 = model.effect_at_period(10);
        let effect_14 = model.effect_at_period(14);

        assert!(!effect_10.is_recession);
        assert!(effect_14.is_recession);
        assert!(effect_14.cycle_factor < effect_10.cycle_factor);
    }

    #[test]
    fn test_price_shock() {
        let shock = PriceShockEvent {
            shock_id: "SHOCK-001".to_string(),
            shock_type: PriceShockType::SupplyDisruption,
            start_period: 6,
            duration_months: 3,
            price_increase_range: (0.10, 0.30),
            affected_categories: vec!["raw_materials".to_string()],
        };

        assert!(!shock.is_active_at_period(5));
        assert!(shock.is_active_at_period(6));
        assert!(shock.is_active_at_period(8));
        assert!(!shock.is_active_at_period(9));

        let progress = shock.progress_at_period(7);
        assert!(progress > 0.3 && progress < 0.5);
    }

    #[test]
    fn test_market_drift_model() {
        let model = MarketDriftModel {
            economic_cycle: EconomicCycleModel {
                enabled: true,
                period_months: 12,
                amplitude: 0.15,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let effects = model.compute_effects(6, &mut rng);

        assert!((effects.economic_cycle_factor - 1.0).abs() < 0.5);
    }
}
