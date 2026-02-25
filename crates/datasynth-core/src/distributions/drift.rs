//! Temporal drift simulation for realistic data distribution evolution.
//!
//! Implements gradual, sudden, and seasonal drift patterns commonly observed
//! in real-world enterprise data, useful for training drift detection models.
//!
//! Enhanced with:
//! - Regime changes (structural breaks like acquisitions, policy changes)
//! - Economic cycles (sinusoidal patterns with recessions)
//! - Parameter drifts (gradual changes in distribution parameters)

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Types of temporal drift patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DriftType {
    /// Gradual, continuous drift over time (like inflation).
    #[default]
    Gradual,
    /// Sudden, point-in-time shifts (like policy changes).
    Sudden,
    /// Recurring patterns that cycle (like seasonal variations).
    Recurring,
    /// Combination of gradual background drift with occasional sudden shifts.
    Mixed,
    /// Regime-based changes with structural breaks.
    Regime,
    /// Economic cycle patterns.
    EconomicCycle,
}

/// Type of regime change event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum RegimeChangeType {
    /// Acquisition - sudden volume and amount increase
    Acquisition,
    /// Divestiture - sudden volume and amount decrease
    Divestiture,
    /// Price increase - amounts increase, volume may decrease
    PriceIncrease,
    /// Price decrease - amounts decrease, volume may increase
    PriceDecrease,
    /// New product launch - volume ramp-up
    ProductLaunch,
    /// Product discontinuation - volume ramp-down
    ProductDiscontinuation,
    /// Policy change - affects patterns without changing volumes
    #[default]
    PolicyChange,
    /// Competitor entry - market disruption
    CompetitorEntry,
    /// Custom effect with specified multipliers
    Custom,
}

/// Effect of a regime change on specific fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeEffect {
    /// Field being affected (e.g., "transaction_volume", "amount_mean")
    pub field: String,
    /// Multiplier to apply (1.0 = no change, 1.5 = 50% increase)
    pub multiplier: f64,
}

/// A single regime change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeChange {
    /// Period when the change occurs (0-indexed)
    pub period: u32,
    /// Type of regime change
    pub change_type: RegimeChangeType,
    /// Description of the change
    #[serde(default)]
    pub description: Option<String>,
    /// Custom effects (only used if change_type is Custom)
    #[serde(default)]
    pub effects: Vec<RegimeEffect>,
    /// Transition duration in periods (0 = immediate, >0 = gradual)
    #[serde(default)]
    pub transition_periods: u32,
}

impl RegimeChange {
    /// Create a new regime change.
    pub fn new(period: u32, change_type: RegimeChangeType) -> Self {
        Self {
            period,
            change_type,
            description: None,
            effects: Vec::new(),
            transition_periods: 0,
        }
    }

    /// Get the volume multiplier for this regime change.
    pub fn volume_multiplier(&self) -> f64 {
        match self.change_type {
            RegimeChangeType::Acquisition => 1.35,
            RegimeChangeType::Divestiture => 0.70,
            RegimeChangeType::PriceIncrease => 0.95,
            RegimeChangeType::PriceDecrease => 1.10,
            RegimeChangeType::ProductLaunch => 1.20,
            RegimeChangeType::ProductDiscontinuation => 0.85,
            RegimeChangeType::PolicyChange => 1.0,
            RegimeChangeType::CompetitorEntry => 0.90,
            RegimeChangeType::Custom => self
                .effects
                .iter()
                .find(|e| e.field == "transaction_volume")
                .map(|e| e.multiplier)
                .unwrap_or(1.0),
        }
    }

    /// Get the amount mean multiplier for this regime change.
    pub fn amount_mean_multiplier(&self) -> f64 {
        match self.change_type {
            RegimeChangeType::Acquisition => 1.15,
            RegimeChangeType::Divestiture => 0.90,
            RegimeChangeType::PriceIncrease => 1.25,
            RegimeChangeType::PriceDecrease => 0.80,
            RegimeChangeType::ProductLaunch => 0.90, // New products often cheaper
            RegimeChangeType::ProductDiscontinuation => 1.10, // Remaining products higher-value
            RegimeChangeType::PolicyChange => 1.0,
            RegimeChangeType::CompetitorEntry => 0.95,
            RegimeChangeType::Custom => self
                .effects
                .iter()
                .find(|e| e.field == "amount_mean")
                .map(|e| e.multiplier)
                .unwrap_or(1.0),
        }
    }
}

/// Configuration for economic cycle patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicCycleConfig {
    /// Enable economic cycle patterns
    pub enabled: bool,
    /// Cycle length in periods (e.g., 48 months for a 4-year cycle)
    #[serde(default = "default_cycle_length")]
    pub cycle_length: u32,
    /// Amplitude of the cycle (0.0-1.0, affects the peak-to-trough ratio)
    #[serde(default = "default_amplitude")]
    pub amplitude: f64,
    /// Phase offset in periods (shifts the cycle start)
    #[serde(default)]
    pub phase_offset: u32,
    /// Recession periods (list of periods that are in recession)
    #[serde(default)]
    pub recession_periods: Vec<u32>,
    /// Recession severity multiplier (0.0-1.0, lower = more severe)
    #[serde(default = "default_recession_severity")]
    pub recession_severity: f64,
}

fn default_cycle_length() -> u32 {
    48 // 4-year business cycle
}

fn default_amplitude() -> f64 {
    0.15 // 15% peak-to-trough variation
}

fn default_recession_severity() -> f64 {
    0.75 // 25% reduction during recession
}

impl Default for EconomicCycleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cycle_length: 48,
            amplitude: 0.15,
            phase_offset: 0,
            recession_periods: Vec::new(),
            recession_severity: 0.75,
        }
    }
}

/// Types of parameter drift patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ParameterDriftType {
    /// Linear drift: parameter = initial + rate * period
    #[default]
    Linear,
    /// Exponential drift: parameter = initial * (1 + rate)^period
    Exponential,
    /// Logistic drift: S-curve transition between start and end values
    Logistic,
    /// Step drift: sudden change at specified periods
    Step,
}

/// Configuration for a parameter drift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDrift {
    /// Name of the parameter being drifted
    pub parameter: String,
    /// Type of drift pattern
    pub drift_type: ParameterDriftType,
    /// Initial value
    pub initial_value: f64,
    /// Final value (for logistic) or rate (for linear/exponential)
    pub target_or_rate: f64,
    /// Start period for the drift
    #[serde(default)]
    pub start_period: u32,
    /// End period for the drift (for logistic), or ignored for linear/exponential
    #[serde(default)]
    pub end_period: Option<u32>,
    /// Steepness for logistic curve (higher = sharper transition)
    #[serde(default = "default_steepness")]
    pub steepness: f64,
}

fn default_steepness() -> f64 {
    0.1
}

impl Default for ParameterDrift {
    fn default() -> Self {
        Self {
            parameter: String::new(),
            drift_type: ParameterDriftType::Linear,
            initial_value: 1.0,
            target_or_rate: 0.01,
            start_period: 0,
            end_period: None,
            steepness: 0.1,
        }
    }
}

impl ParameterDrift {
    /// Calculate the parameter value at a given period.
    pub fn value_at(&self, period: u32) -> f64 {
        if period < self.start_period {
            return self.initial_value;
        }

        let effective_period = period - self.start_period;

        match self.drift_type {
            ParameterDriftType::Linear => {
                self.initial_value + self.target_or_rate * (effective_period as f64)
            }
            ParameterDriftType::Exponential => {
                self.initial_value * (1.0 + self.target_or_rate).powi(effective_period as i32)
            }
            ParameterDriftType::Logistic => {
                let end_period = self.end_period.unwrap_or(self.start_period + 24);
                let midpoint = (self.start_period + end_period) as f64 / 2.0;
                let t = period as f64;
                let range = self.target_or_rate - self.initial_value;
                self.initial_value + range / (1.0 + (-self.steepness * (t - midpoint)).exp())
            }
            ParameterDriftType::Step => {
                if let Some(end) = self.end_period {
                    if period >= end {
                        return self.target_or_rate;
                    }
                }
                self.initial_value
            }
        }
    }
}

/// Configuration for temporal drift simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftConfig {
    /// Enable temporal drift simulation.
    pub enabled: bool,
    /// Amount mean drift per period (e.g., 0.02 = 2% shift per month).
    pub amount_mean_drift: f64,
    /// Amount variance drift per period.
    pub amount_variance_drift: f64,
    /// Anomaly rate drift per period.
    pub anomaly_rate_drift: f64,
    /// Concept drift rate (0.0-1.0).
    pub concept_drift_rate: f64,
    /// Probability of sudden drift in any period.
    pub sudden_drift_probability: f64,
    /// Magnitude of sudden drift events.
    pub sudden_drift_magnitude: f64,
    /// Enable seasonal drift patterns.
    pub seasonal_drift: bool,
    /// Period to start drift (0 = from beginning).
    pub drift_start_period: u32,
    /// Type of drift pattern.
    pub drift_type: DriftType,
    /// Regime changes (structural breaks)
    #[serde(default)]
    pub regime_changes: Vec<RegimeChange>,
    /// Economic cycle configuration
    #[serde(default)]
    pub economic_cycle: EconomicCycleConfig,
    /// Parameter drifts
    #[serde(default)]
    pub parameter_drifts: Vec<ParameterDrift>,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            amount_mean_drift: 0.02,
            amount_variance_drift: 0.0,
            anomaly_rate_drift: 0.0,
            concept_drift_rate: 0.01,
            sudden_drift_probability: 0.0,
            sudden_drift_magnitude: 2.0,
            seasonal_drift: false,
            drift_start_period: 0,
            drift_type: DriftType::Gradual,
            regime_changes: Vec::new(),
            economic_cycle: EconomicCycleConfig::default(),
            parameter_drifts: Vec::new(),
        }
    }
}

impl DriftConfig {
    /// Create a configuration with regime changes.
    pub fn with_regime_changes(regime_changes: Vec<RegimeChange>) -> Self {
        Self {
            enabled: true,
            drift_type: DriftType::Regime,
            regime_changes,
            ..Default::default()
        }
    }

    /// Create a configuration with economic cycle.
    pub fn with_economic_cycle(cycle_config: EconomicCycleConfig) -> Self {
        Self {
            enabled: true,
            drift_type: DriftType::EconomicCycle,
            economic_cycle: cycle_config,
            ..Default::default()
        }
    }
}

/// Drift adjustments computed for a specific period.
#[derive(Debug, Clone, Default)]
pub struct DriftAdjustments {
    /// Multiplier for amount mean (1.0 = no change).
    pub amount_mean_multiplier: f64,
    /// Multiplier for amount variance (1.0 = no change).
    pub amount_variance_multiplier: f64,
    /// Additive adjustment to anomaly rate.
    pub anomaly_rate_adjustment: f64,
    /// Overall concept drift factor (0.0-1.0).
    pub concept_drift_factor: f64,
    /// Whether a sudden drift event occurred.
    pub sudden_drift_occurred: bool,
    /// Seasonal factor (1.0 = baseline, varies by month).
    pub seasonal_factor: f64,
    /// Volume multiplier from regime changes (1.0 = no change).
    pub volume_multiplier: f64,
    /// Economic cycle factor (1.0 = neutral, varies with cycle).
    pub economic_cycle_factor: f64,
    /// Whether currently in a recession period.
    pub in_recession: bool,
    /// Active regime changes in this period.
    pub active_regime_changes: Vec<RegimeChangeType>,
    /// Parameter drift values for this period.
    pub parameter_values: std::collections::HashMap<String, f64>,
}

impl DriftAdjustments {
    /// No drift (identity adjustments).
    pub fn none() -> Self {
        Self {
            amount_mean_multiplier: 1.0,
            amount_variance_multiplier: 1.0,
            anomaly_rate_adjustment: 0.0,
            concept_drift_factor: 0.0,
            sudden_drift_occurred: false,
            seasonal_factor: 1.0,
            volume_multiplier: 1.0,
            economic_cycle_factor: 1.0,
            in_recession: false,
            active_regime_changes: Vec::new(),
            parameter_values: std::collections::HashMap::new(),
        }
    }

    /// Get the combined multiplier for transaction amounts.
    pub fn combined_amount_multiplier(&self) -> f64 {
        self.amount_mean_multiplier * self.seasonal_factor * self.economic_cycle_factor
    }

    /// Get the combined multiplier for transaction volume.
    pub fn combined_volume_multiplier(&self) -> f64 {
        self.volume_multiplier * self.seasonal_factor * self.economic_cycle_factor
    }
}

/// Controller for computing and applying temporal drift.
pub struct DriftController {
    config: DriftConfig,
    rng: ChaCha8Rng,
    /// Track which periods had sudden drift events for reproducibility.
    sudden_drift_periods: Vec<u32>,
    /// Total periods in the simulation.
    total_periods: u32,
}

impl DriftController {
    /// Create a new drift controller with the given configuration.
    pub fn new(config: DriftConfig, seed: u64, total_periods: u32) -> Self {
        let mut controller = Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
            sudden_drift_periods: Vec::new(),
            total_periods,
        };

        // Pre-compute sudden drift events for reproducibility
        if controller.config.enabled
            && (controller.config.drift_type == DriftType::Sudden
                || controller.config.drift_type == DriftType::Mixed)
        {
            controller.precompute_sudden_drifts();
        }

        controller
    }

    /// Pre-compute which periods will have sudden drift events.
    fn precompute_sudden_drifts(&mut self) {
        for period in 0..self.total_periods {
            if period >= self.config.drift_start_period
                && self.rng.random::<f64>() < self.config.sudden_drift_probability
            {
                self.sudden_drift_periods.push(period);
            }
        }
    }

    /// Check if drift is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Compute drift adjustments for a specific period (0-indexed).
    pub fn compute_adjustments(&self, period: u32) -> DriftAdjustments {
        if !self.config.enabled {
            return DriftAdjustments::none();
        }

        // No drift before start period
        if period < self.config.drift_start_period {
            return DriftAdjustments::none();
        }

        let effective_period = period - self.config.drift_start_period;
        let mut adjustments = DriftAdjustments::none();

        match self.config.drift_type {
            DriftType::Gradual => {
                self.apply_gradual_drift(&mut adjustments, effective_period);
            }
            DriftType::Sudden => {
                self.apply_sudden_drift(&mut adjustments, period);
            }
            DriftType::Recurring => {
                self.apply_recurring_drift(&mut adjustments, effective_period);
            }
            DriftType::Mixed => {
                // Combine gradual background drift with sudden events
                self.apply_gradual_drift(&mut adjustments, effective_period);
                self.apply_sudden_drift(&mut adjustments, period);
            }
            DriftType::Regime => {
                self.apply_regime_drift(&mut adjustments, period);
            }
            DriftType::EconomicCycle => {
                self.apply_economic_cycle(&mut adjustments, period);
            }
        }

        // Apply seasonal drift if enabled (additive to other drift)
        if self.config.seasonal_drift {
            adjustments.seasonal_factor = self.compute_seasonal_factor(period);
        }

        // Apply parameter drifts
        self.apply_parameter_drifts(&mut adjustments, period);

        adjustments
    }

    /// Apply gradual drift (compound growth model).
    fn apply_gradual_drift(&self, adjustments: &mut DriftAdjustments, effective_period: u32) {
        let p = effective_period as f64;

        // Compound growth: (1 + rate)^period
        adjustments.amount_mean_multiplier = (1.0 + self.config.amount_mean_drift).powf(p);

        adjustments.amount_variance_multiplier = (1.0 + self.config.amount_variance_drift).powf(p);

        // Linear accumulation for anomaly rate
        adjustments.anomaly_rate_adjustment = self.config.anomaly_rate_drift * p;

        // Concept drift accumulates but is bounded 0-1
        adjustments.concept_drift_factor = (self.config.concept_drift_rate * p).min(1.0);
    }

    /// Apply sudden drift based on pre-computed events.
    fn apply_sudden_drift(&self, adjustments: &mut DriftAdjustments, period: u32) {
        // Count how many sudden events have occurred up to this period
        let events_occurred: usize = self
            .sudden_drift_periods
            .iter()
            .filter(|&&p| p <= period)
            .count();

        if events_occurred > 0 {
            adjustments.sudden_drift_occurred = self.sudden_drift_periods.contains(&period);

            // Each sudden event multiplies by the magnitude
            let cumulative_magnitude = self
                .config
                .sudden_drift_magnitude
                .powi(events_occurred as i32);

            adjustments.amount_mean_multiplier *= cumulative_magnitude;
            adjustments.amount_variance_multiplier *= cumulative_magnitude.sqrt();
            // Variance grows slower
        }
    }

    /// Apply recurring (seasonal) drift patterns.
    fn apply_recurring_drift(&self, adjustments: &mut DriftAdjustments, effective_period: u32) {
        // 12-month cycle for seasonality
        let cycle_position = (effective_period % 12) as f64;
        let cycle_radians = (cycle_position / 12.0) * 2.0 * std::f64::consts::PI;

        // Sinusoidal pattern with configurable amplitude
        let seasonal_amplitude = self.config.concept_drift_rate;
        adjustments.amount_mean_multiplier = 1.0 + seasonal_amplitude * cycle_radians.sin();

        // Phase-shifted variance pattern
        adjustments.amount_variance_multiplier =
            1.0 + (seasonal_amplitude * 0.5) * (cycle_radians + std::f64::consts::FRAC_PI_2).sin();
    }

    /// Compute seasonal factor based on period (month).
    fn compute_seasonal_factor(&self, period: u32) -> f64 {
        // Map period to month (0-11)
        let month = period % 12;

        // Q4 spike (Oct-Dec), Q1 dip (Jan-Feb)
        match month {
            0 | 1 => 0.85, // Jan-Feb: post-holiday slowdown
            2 => 0.90,     // Mar: recovering
            3 | 4 => 0.95, // Apr-May: Q2 start
            5 => 1.0,      // Jun: mid-year
            6 | 7 => 0.95, // Jul-Aug: summer slowdown
            8 => 1.0,      // Sep: back to business
            9 => 1.10,     // Oct: Q4 ramp-up
            10 => 1.20,    // Nov: pre-holiday surge
            11 => 1.30,    // Dec: year-end close
            _ => 1.0,
        }
    }

    /// Get the list of periods with sudden drift events.
    pub fn sudden_drift_periods(&self) -> &[u32] {
        &self.sudden_drift_periods
    }

    /// Get the configuration.
    pub fn config(&self) -> &DriftConfig {
        &self.config
    }

    /// Apply regime change drift.
    fn apply_regime_drift(&self, adjustments: &mut DriftAdjustments, period: u32) {
        let mut volume_mult = 1.0;
        let mut amount_mult = 1.0;

        for regime_change in &self.config.regime_changes {
            if period >= regime_change.period {
                // Calculate transition factor
                let periods_since = period - regime_change.period;
                let transition_factor = if regime_change.transition_periods == 0 {
                    1.0
                } else {
                    (periods_since as f64 / regime_change.transition_periods as f64).min(1.0)
                };

                // Apply multipliers with transition
                let vol_change = regime_change.volume_multiplier() - 1.0;
                let amt_change = regime_change.amount_mean_multiplier() - 1.0;

                volume_mult *= 1.0 + vol_change * transition_factor;
                amount_mult *= 1.0 + amt_change * transition_factor;

                adjustments
                    .active_regime_changes
                    .push(regime_change.change_type);
            }
        }

        adjustments.volume_multiplier = volume_mult;
        adjustments.amount_mean_multiplier *= amount_mult;
    }

    /// Apply economic cycle pattern.
    fn apply_economic_cycle(&self, adjustments: &mut DriftAdjustments, period: u32) {
        let cycle = &self.config.economic_cycle;
        if !cycle.enabled {
            return;
        }

        // Calculate position in cycle (0.0 to 1.0)
        let adjusted_period = period + cycle.phase_offset;
        let cycle_position =
            (adjusted_period % cycle.cycle_length) as f64 / cycle.cycle_length as f64;

        // Sinusoidal cycle: 1.0 + amplitude * sin(2*pi*position)
        let cycle_radians = cycle_position * 2.0 * std::f64::consts::PI;
        let cycle_factor = 1.0 + cycle.amplitude * cycle_radians.sin();

        // Check for recession
        let in_recession = cycle.recession_periods.contains(&period);
        adjustments.in_recession = in_recession;

        // Apply recession severity if in recession
        let final_factor = if in_recession {
            cycle_factor * cycle.recession_severity
        } else {
            cycle_factor
        };

        adjustments.economic_cycle_factor = final_factor;
        adjustments.amount_mean_multiplier *= final_factor;
        adjustments.volume_multiplier = final_factor;
    }

    /// Apply parameter drifts.
    fn apply_parameter_drifts(&self, adjustments: &mut DriftAdjustments, period: u32) {
        for param_drift in &self.config.parameter_drifts {
            let value = param_drift.value_at(period);
            adjustments
                .parameter_values
                .insert(param_drift.parameter.clone(), value);
        }
    }

    /// Get regime changes that occurred up to a given period.
    pub fn regime_changes_until(&self, period: u32) -> Vec<&RegimeChange> {
        self.config
            .regime_changes
            .iter()
            .filter(|rc| rc.period <= period)
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_no_drift_when_disabled() {
        let config = DriftConfig::default();
        let controller = DriftController::new(config, 42, 12);

        let adjustments = controller.compute_adjustments(6);
        assert!(!controller.is_enabled());
        assert!((adjustments.amount_mean_multiplier - 1.0).abs() < 0.001);
        assert!((adjustments.anomaly_rate_adjustment).abs() < 0.001);
    }

    #[test]
    fn test_gradual_drift() {
        let config = DriftConfig {
            enabled: true,
            amount_mean_drift: 0.02,
            anomaly_rate_drift: 0.001,
            drift_type: DriftType::Gradual,
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        // Period 0: no drift yet
        let adj0 = controller.compute_adjustments(0);
        assert!((adj0.amount_mean_multiplier - 1.0).abs() < 0.001);

        // Period 6: ~12.6% drift (1.02^6 ≈ 1.126)
        let adj6 = controller.compute_adjustments(6);
        assert!(adj6.amount_mean_multiplier > 1.10);
        assert!(adj6.amount_mean_multiplier < 1.15);

        // Period 12: ~26.8% drift (1.02^12 ≈ 1.268)
        let adj12 = controller.compute_adjustments(12);
        assert!(adj12.amount_mean_multiplier > 1.20);
        assert!(adj12.amount_mean_multiplier < 1.30);
    }

    #[test]
    fn test_drift_start_period() {
        let config = DriftConfig {
            enabled: true,
            amount_mean_drift: 0.02,
            drift_start_period: 3,
            drift_type: DriftType::Gradual,
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        // Before drift start: no drift
        let adj2 = controller.compute_adjustments(2);
        assert!((adj2.amount_mean_multiplier - 1.0).abs() < 0.001);

        // At drift start: no drift yet (effective_period = 0)
        let adj3 = controller.compute_adjustments(3);
        assert!((adj3.amount_mean_multiplier - 1.0).abs() < 0.001);

        // After drift start: drift begins
        let adj6 = controller.compute_adjustments(6);
        assert!(adj6.amount_mean_multiplier > 1.0);
    }

    #[test]
    fn test_seasonal_factor() {
        let config = DriftConfig {
            enabled: true,
            seasonal_drift: true,
            drift_type: DriftType::Gradual,
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        // December (month 11) should have highest seasonal factor
        let adj_dec = controller.compute_adjustments(11);
        assert!(adj_dec.seasonal_factor > 1.2);

        // January (month 0) should have lower seasonal factor
        let adj_jan = controller.compute_adjustments(0);
        assert!(adj_jan.seasonal_factor < 0.9);
    }

    #[test]
    fn test_sudden_drift_reproducibility() {
        let config = DriftConfig {
            enabled: true,
            sudden_drift_probability: 0.5,
            sudden_drift_magnitude: 1.5,
            drift_type: DriftType::Sudden,
            ..Default::default()
        };

        // Same seed should produce same sudden drift periods
        let controller1 = DriftController::new(config.clone(), 42, 12);
        let controller2 = DriftController::new(config, 42, 12);

        assert_eq!(
            controller1.sudden_drift_periods(),
            controller2.sudden_drift_periods()
        );
    }

    #[test]
    fn test_regime_change() {
        let config = DriftConfig {
            enabled: true,
            drift_type: DriftType::Regime,
            regime_changes: vec![RegimeChange::new(6, RegimeChangeType::Acquisition)],
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        // Before regime change
        let adj_before = controller.compute_adjustments(5);
        assert!((adj_before.volume_multiplier - 1.0).abs() < 0.001);

        // After regime change
        let adj_after = controller.compute_adjustments(6);
        assert!(adj_after.volume_multiplier > 1.3); // Acquisition increases volume
        assert!(adj_after.amount_mean_multiplier > 1.1); // And amounts
        assert!(adj_after
            .active_regime_changes
            .contains(&RegimeChangeType::Acquisition));
    }

    #[test]
    fn test_regime_change_gradual_transition() {
        let config = DriftConfig {
            enabled: true,
            drift_type: DriftType::Regime,
            regime_changes: vec![RegimeChange {
                period: 6,
                change_type: RegimeChangeType::PriceIncrease,
                description: None,
                effects: vec![],
                transition_periods: 4, // 4 period transition
            }],
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        // At regime change start
        let adj_start = controller.compute_adjustments(6);
        // Midway through transition
        let adj_mid = controller.compute_adjustments(8);
        // After transition complete
        let adj_end = controller.compute_adjustments(10);

        // Should gradually increase
        assert!(adj_start.amount_mean_multiplier < adj_mid.amount_mean_multiplier);
        assert!(adj_mid.amount_mean_multiplier < adj_end.amount_mean_multiplier);
    }

    #[test]
    fn test_economic_cycle() {
        let config = DriftConfig {
            enabled: true,
            drift_type: DriftType::EconomicCycle,
            economic_cycle: EconomicCycleConfig {
                enabled: true,
                cycle_length: 12, // 1-year cycle for testing
                amplitude: 0.20,
                phase_offset: 0,
                recession_periods: vec![],
                recession_severity: 0.75,
            },
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 24);

        // At cycle start (period 0)
        let adj_0 = controller.compute_adjustments(0);
        // At cycle peak (period 3, which is 25% through cycle = 90 degrees)
        let adj_3 = controller.compute_adjustments(3);
        // At cycle trough (period 9, which is 75% through cycle = 270 degrees)
        let adj_9 = controller.compute_adjustments(9);

        // Peak should be higher than start
        assert!(adj_3.economic_cycle_factor > adj_0.economic_cycle_factor);
        // Trough should be lower than start
        assert!(adj_9.economic_cycle_factor < adj_0.economic_cycle_factor);
    }

    #[test]
    fn test_economic_cycle_recession() {
        let config = DriftConfig {
            enabled: true,
            drift_type: DriftType::EconomicCycle,
            economic_cycle: EconomicCycleConfig {
                enabled: true,
                cycle_length: 12,
                amplitude: 0.10,
                phase_offset: 0,
                recession_periods: vec![6, 7, 8],
                recession_severity: 0.70,
            },
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        // Not in recession
        let adj_5 = controller.compute_adjustments(5);
        assert!(!adj_5.in_recession);

        // In recession
        let adj_7 = controller.compute_adjustments(7);
        assert!(adj_7.in_recession);
        assert!(adj_7.economic_cycle_factor < adj_5.economic_cycle_factor);
    }

    #[test]
    fn test_parameter_drift_linear() {
        let config = DriftConfig {
            enabled: true,
            drift_type: DriftType::Gradual,
            parameter_drifts: vec![ParameterDrift {
                parameter: "discount_rate".to_string(),
                drift_type: ParameterDriftType::Linear,
                initial_value: 0.02,
                target_or_rate: 0.001, // Increases by 0.1% per period
                start_period: 0,
                end_period: None,
                steepness: 0.1,
            }],
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        let adj_0 = controller.compute_adjustments(0);
        let adj_6 = controller.compute_adjustments(6);

        let rate_0 = adj_0.parameter_values.get("discount_rate").unwrap();
        let rate_6 = adj_6.parameter_values.get("discount_rate").unwrap();

        // Should increase linearly
        assert!((rate_0 - 0.02).abs() < 0.0001);
        assert!((rate_6 - 0.026).abs() < 0.0001);
    }

    #[test]
    fn test_parameter_drift_logistic() {
        let config = DriftConfig {
            enabled: true,
            drift_type: DriftType::Gradual,
            parameter_drifts: vec![ParameterDrift {
                parameter: "market_share".to_string(),
                drift_type: ParameterDriftType::Logistic,
                initial_value: 0.10,  // 10% starting market share
                target_or_rate: 0.40, // 40% target market share
                start_period: 0,
                end_period: Some(24), // 24 period transition
                steepness: 0.3,
            }],
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 36);

        let adj_0 = controller.compute_adjustments(0);
        let adj_12 = controller.compute_adjustments(12);
        let adj_24 = controller.compute_adjustments(24);

        let share_0 = *adj_0.parameter_values.get("market_share").unwrap();
        let share_12 = *adj_12.parameter_values.get("market_share").unwrap();
        let share_24 = *adj_24.parameter_values.get("market_share").unwrap();

        // S-curve: starts slow, accelerates in middle, slows at end
        assert!(share_0 < 0.15); // Near initial
        assert!(share_12 > 0.20 && share_12 < 0.30); // Around midpoint
        assert!(share_24 > 0.35); // Near target
    }

    #[test]
    fn test_combined_drift_adjustments() {
        let adj = DriftAdjustments {
            amount_mean_multiplier: 1.2,
            seasonal_factor: 1.1,
            economic_cycle_factor: 0.9,
            volume_multiplier: 1.3,
            ..DriftAdjustments::none()
        };

        // Combined amount = 1.2 * 1.1 * 0.9 = 1.188
        assert!((adj.combined_amount_multiplier() - 1.188).abs() < 0.001);

        // Combined volume = 1.3 * 1.1 * 0.9 = 1.287
        assert!((adj.combined_volume_multiplier() - 1.287).abs() < 0.001);
    }

    #[test]
    fn test_regime_change_volume_multipliers() {
        assert!(RegimeChange::new(0, RegimeChangeType::Acquisition).volume_multiplier() > 1.0);
        assert!(RegimeChange::new(0, RegimeChangeType::Divestiture).volume_multiplier() < 1.0);
        assert!(
            (RegimeChange::new(0, RegimeChangeType::PolicyChange).volume_multiplier() - 1.0).abs()
                < 0.001
        );
    }
}
