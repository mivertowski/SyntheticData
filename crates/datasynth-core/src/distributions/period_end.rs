//! Period-end dynamics and decay curves for realistic volume modeling.
//!
//! Implements various models for period-end volume spikes including:
//! - Flat multipliers (legacy behavior)
//! - Exponential acceleration curves
//! - Custom daily profiles
//! - Extended crunch periods

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model for period-end volume patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PeriodEndModel {
    /// Simple flat multiplier (legacy behavior).
    FlatMultiplier {
        /// Volume multiplier during period-end
        multiplier: f64,
    },

    /// Exponential acceleration curve approaching period end.
    ExponentialAcceleration {
        /// Days before period end to start acceleration (negative, e.g., -10)
        start_day: i32,
        /// Multiplier at the start of acceleration
        base_multiplier: f64,
        /// Peak multiplier on the last day
        peak_multiplier: f64,
        /// Decay rate (higher = steeper curve, typically 0.1-0.5)
        decay_rate: f64,
    },

    /// Custom daily profile with explicit multipliers.
    DailyProfile {
        /// Map of days-to-close -> multiplier (e.g., -5 -> 1.5, -1 -> 3.0)
        profile: HashMap<i32, f64>,
        /// Interpolation method for days not in profile
        #[serde(default)]
        interpolation: InterpolationMethod,
    },

    /// Extended crunch period with sustained high volume.
    ExtendedCrunch {
        /// Days before period end to start (negative, e.g., -10)
        start_day: i32,
        /// Number of days at sustained high volume
        sustained_high_days: i32,
        /// Peak multiplier during crunch
        peak_multiplier: f64,
        /// Ramp-up rate (days from start to peak)
        #[serde(default = "default_ramp_days")]
        ramp_up_days: i32,
    },
}

fn default_ramp_days() -> i32 {
    3
}

/// Interpolation method for daily profiles.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpolationMethod {
    /// Use the nearest defined value
    #[default]
    Nearest,
    /// Linear interpolation between defined values
    Linear,
    /// Step function (use previous defined value)
    Step,
}

impl Default for PeriodEndModel {
    fn default() -> Self {
        Self::ExponentialAcceleration {
            start_day: -10,
            base_multiplier: 1.0,
            peak_multiplier: 3.5,
            decay_rate: 0.3,
        }
    }
}

impl PeriodEndModel {
    /// Calculate the multiplier for a given number of days to period end.
    ///
    /// `days_to_end` is negative or zero (0 = last day, -1 = day before, etc.)
    pub fn get_multiplier(&self, days_to_end: i32) -> f64 {
        match self {
            PeriodEndModel::FlatMultiplier { multiplier } => {
                if (-5..=0).contains(&days_to_end) {
                    *multiplier
                } else {
                    1.0
                }
            }

            PeriodEndModel::ExponentialAcceleration {
                start_day,
                base_multiplier,
                peak_multiplier,
                decay_rate,
            } => {
                if days_to_end < *start_day || days_to_end > 0 {
                    return 1.0;
                }

                // Normalize position: 0.0 at start_day, 1.0 at day 0
                let total_days = (-start_day) as f64;
                let position = (days_to_end - start_day) as f64 / total_days;

                // Exponential growth: base + (peak - base) * (e^(rate*pos) - 1) / (e^rate - 1)
                let exp_factor = (decay_rate * position).exp();
                let exp_max = decay_rate.exp();
                let normalized = (exp_factor - 1.0) / (exp_max - 1.0);

                base_multiplier + (peak_multiplier - base_multiplier) * normalized
            }

            PeriodEndModel::DailyProfile {
                profile,
                interpolation,
            } => {
                if let Some(&mult) = profile.get(&days_to_end) {
                    return mult;
                }

                // Handle days not in profile
                let keys: Vec<i32> = profile.keys().copied().collect();
                if keys.is_empty() {
                    return 1.0;
                }

                match interpolation {
                    InterpolationMethod::Nearest => {
                        let nearest = keys
                            .iter()
                            .min_by_key(|&&k| (k - days_to_end).abs())
                            .expect("valid date components");
                        *profile.get(nearest).unwrap_or(&1.0)
                    }
                    InterpolationMethod::Linear => {
                        let mut below = None;
                        let mut above = None;
                        for &k in &keys {
                            if k <= days_to_end
                                && (below.is_none() || k > below.expect("valid date components"))
                            {
                                below = Some(k);
                            }
                            if k >= days_to_end
                                && (above.is_none() || k < above.expect("valid date components"))
                            {
                                above = Some(k);
                            }
                        }

                        match (below, above) {
                            (Some(b), Some(a)) if b != a => {
                                let b_val = profile.get(&b).unwrap_or(&1.0);
                                let a_val = profile.get(&a).unwrap_or(&1.0);
                                let t = (days_to_end - b) as f64 / (a - b) as f64;
                                b_val + (a_val - b_val) * t
                            }
                            (Some(b), _) => *profile.get(&b).unwrap_or(&1.0),
                            (_, Some(a)) => *profile.get(&a).unwrap_or(&1.0),
                            _ => 1.0,
                        }
                    }
                    InterpolationMethod::Step => {
                        let prev = keys.iter().filter(|&&k| k <= days_to_end).max();
                        prev.and_then(|k| profile.get(k).copied()).unwrap_or(1.0)
                    }
                }
            }

            PeriodEndModel::ExtendedCrunch {
                start_day,
                sustained_high_days,
                peak_multiplier,
                ramp_up_days,
            } => {
                if days_to_end < *start_day || days_to_end > 0 {
                    return 1.0;
                }

                let ramp_end = start_day + ramp_up_days;
                let sustain_end = ramp_end + sustained_high_days;

                if days_to_end < ramp_end {
                    // Ramp-up phase
                    let ramp_position = (days_to_end - start_day) as f64 / *ramp_up_days as f64;
                    1.0 + (peak_multiplier - 1.0) * ramp_position
                } else if days_to_end < sustain_end {
                    // Sustained high phase
                    *peak_multiplier
                } else {
                    // Gradual decrease (optional wind-down)
                    let wind_down_days = (-sustain_end) as f64;
                    let position = (days_to_end - sustain_end) as f64 / wind_down_days;
                    1.0 + (peak_multiplier - 1.0) * (1.0 - position * 0.3)
                }
            }
        }
    }

    /// Create a flat multiplier model.
    pub fn flat(multiplier: f64) -> Self {
        Self::FlatMultiplier { multiplier }
    }

    /// Create an exponential acceleration model with typical accounting close parameters.
    pub fn exponential_accounting() -> Self {
        Self::ExponentialAcceleration {
            start_day: -10,
            base_multiplier: 1.0,
            peak_multiplier: 3.5,
            decay_rate: 0.3,
        }
    }

    /// Create a custom daily profile.
    pub fn custom_profile(profile: HashMap<i32, f64>) -> Self {
        Self::DailyProfile {
            profile,
            interpolation: InterpolationMethod::Linear,
        }
    }
}

/// Configuration for a specific period end type (month, quarter, year).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodEndConfig {
    /// Whether this period-end effect is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// The model to use for this period end
    #[serde(default)]
    pub model: PeriodEndModel,
    /// Additional multiplier applied on top of the model
    #[serde(default = "default_one")]
    pub additional_multiplier: f64,
}

fn default_true() -> bool {
    true
}

fn default_one() -> f64 {
    1.0
}

impl Default for PeriodEndConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            model: PeriodEndModel::default(),
            additional_multiplier: 1.0,
        }
    }
}

impl PeriodEndConfig {
    /// Create a disabled config.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            model: PeriodEndModel::default(),
            additional_multiplier: 1.0,
        }
    }

    /// Get the multiplier for days to end.
    pub fn get_multiplier(&self, days_to_end: i32) -> f64 {
        if !self.enabled {
            return 1.0;
        }
        self.model.get_multiplier(days_to_end) * self.additional_multiplier
    }
}

/// Dynamics for all period-end types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodEndDynamics {
    /// Month-end configuration
    #[serde(default)]
    pub month_end: PeriodEndConfig,
    /// Quarter-end configuration
    #[serde(default)]
    pub quarter_end: PeriodEndConfig,
    /// Year-end configuration
    #[serde(default)]
    pub year_end: PeriodEndConfig,
}

impl Default for PeriodEndDynamics {
    fn default() -> Self {
        Self {
            month_end: PeriodEndConfig {
                enabled: true,
                model: PeriodEndModel::ExponentialAcceleration {
                    start_day: -10,
                    base_multiplier: 1.0,
                    peak_multiplier: 2.5,
                    decay_rate: 0.25,
                },
                additional_multiplier: 1.0,
            },
            quarter_end: PeriodEndConfig {
                enabled: true,
                model: PeriodEndModel::ExponentialAcceleration {
                    start_day: -12,
                    base_multiplier: 1.0,
                    peak_multiplier: 4.0,
                    decay_rate: 0.3,
                },
                additional_multiplier: 1.0,
            },
            year_end: PeriodEndConfig {
                enabled: true,
                model: PeriodEndModel::ExtendedCrunch {
                    start_day: -15,
                    sustained_high_days: 7,
                    peak_multiplier: 6.0,
                    ramp_up_days: 3,
                },
                additional_multiplier: 1.0,
            },
        }
    }
}

impl PeriodEndDynamics {
    /// Create with specific configurations for each period type.
    pub fn new(
        month_end: PeriodEndConfig,
        quarter_end: PeriodEndConfig,
        year_end: PeriodEndConfig,
    ) -> Self {
        Self {
            month_end,
            quarter_end,
            year_end,
        }
    }

    /// Create dynamics with all period ends disabled.
    pub fn disabled() -> Self {
        Self {
            month_end: PeriodEndConfig::disabled(),
            quarter_end: PeriodEndConfig::disabled(),
            year_end: PeriodEndConfig::disabled(),
        }
    }

    /// Create with flat multipliers (legacy behavior).
    pub fn flat(month: f64, quarter: f64, year: f64) -> Self {
        Self {
            month_end: PeriodEndConfig {
                enabled: true,
                model: PeriodEndModel::flat(month),
                additional_multiplier: 1.0,
            },
            quarter_end: PeriodEndConfig {
                enabled: true,
                model: PeriodEndModel::flat(quarter),
                additional_multiplier: 1.0,
            },
            year_end: PeriodEndConfig {
                enabled: true,
                model: PeriodEndModel::flat(year),
                additional_multiplier: 1.0,
            },
        }
    }

    /// Get the multiplier for a specific date.
    ///
    /// Determines which period-end type applies and calculates the appropriate multiplier.
    /// Priority: year-end > quarter-end > month-end
    pub fn get_multiplier(&self, date: NaiveDate, period_end: NaiveDate) -> f64 {
        let days_to_end = (date - period_end).num_days() as i32;

        // Determine which period-end type this is
        let is_year_end = period_end.month() == 12;
        let is_quarter_end = matches!(period_end.month(), 3 | 6 | 9 | 12);

        // Use the most specific applicable config
        if is_year_end && self.year_end.enabled {
            self.year_end.get_multiplier(days_to_end)
        } else if is_quarter_end && self.quarter_end.enabled {
            self.quarter_end.get_multiplier(days_to_end)
        } else if self.month_end.enabled {
            self.month_end.get_multiplier(days_to_end)
        } else {
            1.0
        }
    }

    /// Get the multiplier for a date, automatically determining the period end.
    pub fn get_multiplier_for_date(&self, date: NaiveDate) -> f64 {
        let period_end = Self::last_day_of_month(date);
        self.get_multiplier(date, period_end)
    }

    /// Get the last day of the month for a date.
    fn last_day_of_month(date: NaiveDate) -> NaiveDate {
        let year = date.year();
        let month = date.month();

        if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("valid date components")
                - Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).expect("valid date components")
                - Duration::days(1)
        }
    }

    /// Check if a date is within a period-end window.
    pub fn is_in_period_end(&self, date: NaiveDate) -> bool {
        let period_end = Self::last_day_of_month(date);
        let days_to_end = (date - period_end).num_days() as i32;

        // Check if within any active window
        let in_month_end = self.month_end.enabled && (-10..=0).contains(&days_to_end);
        let in_quarter_end = self.quarter_end.enabled
            && matches!(period_end.month(), 3 | 6 | 9 | 12)
            && (-12..=0).contains(&days_to_end);
        let in_year_end =
            self.year_end.enabled && period_end.month() == 12 && (-15..=0).contains(&days_to_end);

        in_month_end || in_quarter_end || in_year_end
    }
}

/// Configuration struct for YAML/JSON deserialization.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeriodEndSchemaConfig {
    /// Overall model type
    #[serde(default)]
    pub model: Option<String>,

    /// Month-end configuration
    #[serde(default)]
    pub month_end: Option<PeriodEndModelConfig>,

    /// Quarter-end configuration
    #[serde(default)]
    pub quarter_end: Option<PeriodEndModelConfig>,

    /// Year-end configuration
    #[serde(default)]
    pub year_end: Option<PeriodEndModelConfig>,
}

/// Schema config for a period-end model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodEndModelConfig {
    /// Inherit from another config (e.g., "month_end")
    #[serde(default)]
    pub inherit_from: Option<String>,

    /// Additional multiplier on top of inherited or base model
    #[serde(default)]
    pub additional_multiplier: Option<f64>,

    /// Days before period end to start acceleration
    #[serde(default)]
    pub start_day: Option<i32>,

    /// Base multiplier
    #[serde(default)]
    pub base_multiplier: Option<f64>,

    /// Peak multiplier
    #[serde(default)]
    pub peak_multiplier: Option<f64>,

    /// Decay rate for exponential model
    #[serde(default)]
    pub decay_rate: Option<f64>,

    /// Sustained high days for crunch model
    #[serde(default)]
    pub sustained_high_days: Option<i32>,
}

impl PeriodEndSchemaConfig {
    /// Convert to PeriodEndDynamics.
    pub fn to_dynamics(&self) -> PeriodEndDynamics {
        let mut dynamics = PeriodEndDynamics::default();

        // Apply model type if specified
        if let Some(model_type) = &self.model {
            match model_type.as_str() {
                "flat" => {
                    dynamics = PeriodEndDynamics::flat(2.5, 4.0, 6.0);
                }
                "exponential" | "exponential_acceleration" => {
                    // Use defaults (already exponential)
                }
                "extended_crunch" => {
                    dynamics.month_end.model = PeriodEndModel::ExtendedCrunch {
                        start_day: -10,
                        sustained_high_days: 3,
                        peak_multiplier: 2.5,
                        ramp_up_days: 2,
                    };
                    dynamics.quarter_end.model = PeriodEndModel::ExtendedCrunch {
                        start_day: -12,
                        sustained_high_days: 4,
                        peak_multiplier: 4.0,
                        ramp_up_days: 3,
                    };
                }
                _ => {}
            }
        }

        // Apply specific overrides
        if let Some(config) = &self.month_end {
            Self::apply_config(&mut dynamics.month_end, config, None);
        }

        if let Some(config) = &self.quarter_end {
            let inherit = config.inherit_from.as_ref().map(|_| &dynamics.month_end);
            Self::apply_config(&mut dynamics.quarter_end, config, inherit);
        }

        if let Some(config) = &self.year_end {
            let inherit = config.inherit_from.as_ref().map(|from| {
                if from == "quarter_end" {
                    &dynamics.quarter_end
                } else {
                    &dynamics.month_end
                }
            });
            Self::apply_config(&mut dynamics.year_end, config, inherit);
        }

        dynamics
    }

    fn apply_config(
        target: &mut PeriodEndConfig,
        config: &PeriodEndModelConfig,
        inherit: Option<&PeriodEndConfig>,
    ) {
        // Start from inherited config if available
        if let Some(inherited) = inherit {
            target.model = inherited.model.clone();
            target.additional_multiplier = inherited.additional_multiplier;
        }

        // Apply additional multiplier
        if let Some(mult) = config.additional_multiplier {
            target.additional_multiplier = mult;
        }

        // Update model parameters based on what's provided
        match &mut target.model {
            PeriodEndModel::ExponentialAcceleration {
                start_day,
                base_multiplier,
                peak_multiplier,
                decay_rate,
            } => {
                if let Some(sd) = config.start_day {
                    *start_day = sd;
                }
                if let Some(bm) = config.base_multiplier {
                    *base_multiplier = bm;
                }
                if let Some(pm) = config.peak_multiplier {
                    *peak_multiplier = pm;
                }
                if let Some(dr) = config.decay_rate {
                    *decay_rate = dr;
                }
            }
            PeriodEndModel::ExtendedCrunch {
                start_day,
                sustained_high_days,
                peak_multiplier,
                ..
            } => {
                if let Some(sd) = config.start_day {
                    *start_day = sd;
                }
                if let Some(shd) = config.sustained_high_days {
                    *sustained_high_days = shd;
                }
                if let Some(pm) = config.peak_multiplier {
                    *peak_multiplier = pm;
                }
            }
            PeriodEndModel::FlatMultiplier { multiplier } => {
                if let Some(pm) = config.peak_multiplier {
                    *multiplier = pm;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_multiplier() {
        let model = PeriodEndModel::flat(3.0);

        // Within period-end window
        assert!((model.get_multiplier(0) - 3.0).abs() < 0.01);
        assert!((model.get_multiplier(-3) - 3.0).abs() < 0.01);
        assert!((model.get_multiplier(-5) - 3.0).abs() < 0.01);

        // Outside window
        assert!((model.get_multiplier(-6) - 1.0).abs() < 0.01);
        assert!((model.get_multiplier(-10) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_exponential_acceleration() {
        let model = PeriodEndModel::ExponentialAcceleration {
            start_day: -10,
            base_multiplier: 1.0,
            peak_multiplier: 3.5,
            decay_rate: 0.3,
        };

        // At start (day -10), should be near base
        let at_start = model.get_multiplier(-10);
        assert!((1.0..1.3).contains(&at_start));

        // At peak (day 0), should be at peak
        let at_peak = model.get_multiplier(0);
        assert!((at_peak - 3.5).abs() < 0.01);

        // Midway should be between
        let mid = model.get_multiplier(-5);
        assert!(mid > 1.5 && mid < 3.0);

        // Outside window should be 1.0
        assert!((model.get_multiplier(-15) - 1.0).abs() < 0.01);
        assert!((model.get_multiplier(1) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_daily_profile_linear() {
        let mut profile = HashMap::new();
        profile.insert(-10, 1.0);
        profile.insert(-5, 2.0);
        profile.insert(0, 4.0);

        let model = PeriodEndModel::DailyProfile {
            profile,
            interpolation: InterpolationMethod::Linear,
        };

        // Exact values
        assert!((model.get_multiplier(-10) - 1.0).abs() < 0.01);
        assert!((model.get_multiplier(-5) - 2.0).abs() < 0.01);
        assert!((model.get_multiplier(0) - 4.0).abs() < 0.01);

        // Interpolated: midpoint between -10 (1.0) and -5 (2.0) should be ~1.5
        let interp = model.get_multiplier(-7);
        assert!(interp > 1.3 && interp < 1.7);
    }

    #[test]
    fn test_extended_crunch() {
        let model = PeriodEndModel::ExtendedCrunch {
            start_day: -10,
            sustained_high_days: 5,
            peak_multiplier: 4.0,
            ramp_up_days: 3,
        };

        // Before window
        assert!((model.get_multiplier(-15) - 1.0).abs() < 0.01);

        // At start, beginning ramp-up
        let at_start = model.get_multiplier(-10);
        assert!((1.0..2.0).contains(&at_start));

        // After ramp-up, in sustained phase
        let in_sustained = model.get_multiplier(-5);
        assert!((in_sustained - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_period_end_dynamics() {
        let dynamics = PeriodEndDynamics::default();

        // Regular month-end (June 30, 2024)
        let june_25 = NaiveDate::from_ymd_opt(2024, 6, 25).unwrap();
        let mult = dynamics.get_multiplier_for_date(june_25);
        assert!(mult > 1.0); // Should have some elevation

        // Quarter-end (March 31, 2024)
        let march_28 = NaiveDate::from_ymd_opt(2024, 3, 28).unwrap();
        let q_mult = dynamics.get_multiplier_for_date(march_28);
        assert!(q_mult > mult); // Quarter-end should be higher

        // Year-end (December 31, 2024)
        let dec_20 = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let y_mult = dynamics.get_multiplier_for_date(dec_20);
        assert!(y_mult > q_mult); // Year-end should be highest
    }

    #[test]
    fn test_period_end_config_disabled() {
        let config = PeriodEndConfig::disabled();
        assert!((config.get_multiplier(0) - 1.0).abs() < 0.01);
        assert!((config.get_multiplier(-5) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_is_in_period_end() {
        let dynamics = PeriodEndDynamics::default();

        // Mid-month should not be in period-end
        let mid_month = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert!(!dynamics.is_in_period_end(mid_month));

        // Last day should be in period-end
        let last_day = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        assert!(dynamics.is_in_period_end(last_day));

        // 5 days before should be in period-end
        let five_before = NaiveDate::from_ymd_opt(2024, 6, 25).unwrap();
        assert!(dynamics.is_in_period_end(five_before));
    }

    #[test]
    fn test_schema_config_conversion() {
        let schema = PeriodEndSchemaConfig {
            model: Some("exponential".to_string()),
            month_end: Some(PeriodEndModelConfig {
                inherit_from: None,
                additional_multiplier: None,
                start_day: Some(-8),
                base_multiplier: None,
                peak_multiplier: Some(3.0),
                decay_rate: None,
                sustained_high_days: None,
            }),
            quarter_end: Some(PeriodEndModelConfig {
                inherit_from: Some("month_end".to_string()),
                additional_multiplier: Some(1.5),
                start_day: None,
                base_multiplier: None,
                peak_multiplier: None,
                decay_rate: None,
                sustained_high_days: None,
            }),
            year_end: None,
        };

        let dynamics = schema.to_dynamics();

        // Check month-end was customized
        if let PeriodEndModel::ExponentialAcceleration {
            peak_multiplier, ..
        } = &dynamics.month_end.model
        {
            assert!((*peak_multiplier - 3.0).abs() < 0.01);
        }

        // Check quarter-end inherited and has additional multiplier
        assert!((dynamics.quarter_end.additional_multiplier - 1.5).abs() < 0.01);
    }
}
