//! Behavioral drift models for realistic entity behavior evolution.
//!
//! Provides comprehensive behavioral drift modeling including:
//! - Vendor behavioral drift (payment terms, quality, pricing)
//! - Customer behavioral drift (payment patterns, order patterns)
//! - Employee behavioral drift (approval patterns, error patterns)
//! - Collective behavioral drift (year-end intensity, automation adoption)

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Context for behavioral drift calculations.
#[derive(Debug, Clone, Default)]
pub struct DriftContext {
    /// Economic cycle factor (1.0 = neutral, <1.0 = downturn, >1.0 = growth).
    pub economic_cycle_factor: f64,
    /// Whether currently in a recession.
    pub is_recession: bool,
    /// Current inflation rate.
    pub inflation_rate: f64,
    /// Market sentiment.
    pub market_sentiment: MarketSentiment,
    /// Period number (0-indexed).
    pub period: u32,
    /// Total periods in simulation.
    pub total_periods: u32,
}

impl DriftContext {
    /// Create a neutral context.
    pub fn neutral() -> Self {
        Self {
            economic_cycle_factor: 1.0,
            is_recession: false,
            inflation_rate: 0.02,
            market_sentiment: MarketSentiment::Neutral,
            period: 0,
            total_periods: 12,
        }
    }

    /// Get years elapsed (fractional).
    pub fn years_elapsed(&self) -> f64 {
        self.period as f64 / 12.0
    }
}

/// Market sentiment indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MarketSentiment {
    /// Very pessimistic market.
    VeryPessimistic,
    /// Pessimistic market.
    Pessimistic,
    /// Neutral market.
    #[default]
    Neutral,
    /// Optimistic market.
    Optimistic,
    /// Very optimistic market.
    VeryOptimistic,
}

impl MarketSentiment {
    /// Get the sentiment factor (0.6 to 1.4).
    pub fn factor(&self) -> f64 {
        match self {
            Self::VeryPessimistic => 0.6,
            Self::Pessimistic => 0.8,
            Self::Neutral => 1.0,
            Self::Optimistic => 1.2,
            Self::VeryOptimistic => 1.4,
        }
    }
}

/// Trait for behavioral drift modeling.
pub trait BehavioralDrift {
    /// Get the behavioral state at a given date.
    fn behavioral_state_at(&self, date: NaiveDate, context: &DriftContext) -> BehavioralState;

    /// Evolve the behavior to the current date.
    fn evolve(&mut self, current_date: NaiveDate, context: &DriftContext);
}

/// Behavioral state snapshot.
#[derive(Debug, Clone, Default)]
pub struct BehavioralState {
    /// Payment behavior adjustment (days delta).
    pub payment_days_delta: f64,
    /// Order pattern adjustment factor.
    pub order_factor: f64,
    /// Error rate adjustment factor.
    pub error_factor: f64,
    /// Processing time adjustment factor.
    pub processing_time_factor: f64,
    /// Quality adjustment factor.
    pub quality_factor: f64,
    /// Price sensitivity factor.
    pub price_sensitivity: f64,
}

impl BehavioralState {
    /// Create a neutral state.
    pub fn neutral() -> Self {
        Self {
            payment_days_delta: 0.0,
            order_factor: 1.0,
            error_factor: 1.0,
            processing_time_factor: 1.0,
            quality_factor: 1.0,
            price_sensitivity: 1.0,
        }
    }
}

// =============================================================================
// Vendor Behavioral Drift
// =============================================================================

/// Vendor behavioral drift configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VendorBehavioralDrift {
    /// Payment terms drift configuration.
    #[serde(default)]
    pub payment_terms_drift: PaymentTermsDrift,
    /// Vendor quality drift configuration.
    #[serde(default)]
    pub quality_drift: VendorQualityDrift,
    /// Pricing behavior drift configuration.
    #[serde(default)]
    pub pricing_drift: PricingBehaviorDrift,
}

impl VendorBehavioralDrift {
    /// Calculate combined behavioral state.
    pub fn state_at(&self, context: &DriftContext) -> BehavioralState {
        let years = context.years_elapsed();

        // Payment terms extension
        let payment_days = self.payment_terms_drift.extension_rate_per_year
            * years
            * (1.0
                + self.payment_terms_drift.economic_sensitivity
                    * (context.economic_cycle_factor - 1.0));

        // Quality drift
        let quality_factor = if years < 1.0 {
            // New vendor improvement
            1.0 + self.quality_drift.new_vendor_improvement_rate * years
        } else {
            // Complacency after first year
            1.0 + self.quality_drift.new_vendor_improvement_rate
                - self.quality_drift.complacency_decline_rate * (years - 1.0)
        };

        // Price sensitivity to inflation
        let price_sensitivity =
            1.0 + self.pricing_drift.inflation_pass_through * context.inflation_rate * years;

        BehavioralState {
            payment_days_delta: payment_days,
            quality_factor: quality_factor.clamp(0.7, 1.3),
            price_sensitivity,
            ..BehavioralState::neutral()
        }
    }
}

/// Payment terms drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTermsDrift {
    /// Average days extension per year.
    #[serde(default = "default_extension_rate")]
    pub extension_rate_per_year: f64,
    /// Economic sensitivity (how much economic conditions affect terms).
    #[serde(default = "default_economic_sensitivity")]
    pub economic_sensitivity: f64,
}

fn default_extension_rate() -> f64 {
    2.5
}

fn default_economic_sensitivity() -> f64 {
    1.0
}

impl Default for PaymentTermsDrift {
    fn default() -> Self {
        Self {
            extension_rate_per_year: 2.5,
            economic_sensitivity: 1.0,
        }
    }
}

/// Vendor quality drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorQualityDrift {
    /// Quality improvement rate for new vendors (per year).
    #[serde(default = "default_improvement_rate")]
    pub new_vendor_improvement_rate: f64,
    /// Quality decline rate due to complacency (per year after first year).
    #[serde(default = "default_decline_rate")]
    pub complacency_decline_rate: f64,
}

fn default_improvement_rate() -> f64 {
    0.02
}

fn default_decline_rate() -> f64 {
    0.01
}

impl Default for VendorQualityDrift {
    fn default() -> Self {
        Self {
            new_vendor_improvement_rate: 0.02,
            complacency_decline_rate: 0.01,
        }
    }
}

/// Pricing behavior drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingBehaviorDrift {
    /// How much inflation is passed through (0.0 to 1.0).
    #[serde(default = "default_pass_through")]
    pub inflation_pass_through: f64,
    /// Price volatility factor.
    #[serde(default = "default_volatility")]
    pub price_volatility: f64,
}

fn default_pass_through() -> f64 {
    0.80
}

fn default_volatility() -> f64 {
    0.10
}

impl Default for PricingBehaviorDrift {
    fn default() -> Self {
        Self {
            inflation_pass_through: 0.80,
            price_volatility: 0.10,
        }
    }
}

// =============================================================================
// Customer Behavioral Drift
// =============================================================================

/// Customer behavioral drift configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomerBehavioralDrift {
    /// Customer payment drift configuration.
    #[serde(default)]
    pub payment_drift: CustomerPaymentDrift,
    /// Order pattern drift configuration.
    #[serde(default)]
    pub order_drift: OrderPatternDrift,
}

impl CustomerBehavioralDrift {
    /// Calculate combined behavioral state.
    pub fn state_at(&self, context: &DriftContext) -> BehavioralState {
        // Payment delays during downturns
        let payment_days = if context.is_recession || context.economic_cycle_factor < 0.9 {
            let severity = 1.0 - context.economic_cycle_factor;
            self.payment_drift.downturn_days_extension.0 as f64
                + (self.payment_drift.downturn_days_extension.1 as f64
                    - self.payment_drift.downturn_days_extension.0 as f64)
                    * severity
        } else {
            0.0
        };

        // Order pattern shift (digital adoption)
        let years = context.years_elapsed();
        let order_factor = 1.0 + self.order_drift.digital_shift_rate * years;

        BehavioralState {
            payment_days_delta: payment_days,
            order_factor,
            ..BehavioralState::neutral()
        }
    }
}

/// Customer payment drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerPaymentDrift {
    /// Days extension range during economic downturn (min, max).
    #[serde(default = "default_downturn_extension")]
    pub downturn_days_extension: (u32, u32),
    /// Bad debt rate increase during downturn.
    #[serde(default = "default_bad_debt_increase")]
    pub downturn_bad_debt_increase: f64,
}

fn default_downturn_extension() -> (u32, u32) {
    (5, 15)
}

fn default_bad_debt_increase() -> f64 {
    0.02
}

impl Default for CustomerPaymentDrift {
    fn default() -> Self {
        Self {
            downturn_days_extension: (5, 15),
            downturn_bad_debt_increase: 0.02,
        }
    }
}

/// Order pattern drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPatternDrift {
    /// Rate of shift to digital channels (per year).
    #[serde(default = "default_digital_shift")]
    pub digital_shift_rate: f64,
    /// Order consolidation rate (fewer, larger orders).
    #[serde(default = "default_consolidation")]
    pub order_consolidation_rate: f64,
}

fn default_digital_shift() -> f64 {
    0.05
}

fn default_consolidation() -> f64 {
    0.02
}

impl Default for OrderPatternDrift {
    fn default() -> Self {
        Self {
            digital_shift_rate: 0.05,
            order_consolidation_rate: 0.02,
        }
    }
}

// =============================================================================
// Employee Behavioral Drift
// =============================================================================

/// Employee behavioral drift configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmployeeBehavioralDrift {
    /// Approval pattern drift configuration.
    #[serde(default)]
    pub approval_drift: ApprovalPatternDrift,
    /// Error pattern drift configuration.
    #[serde(default)]
    pub error_drift: ErrorPatternDrift,
}

impl EmployeeBehavioralDrift {
    /// Calculate combined behavioral state.
    pub fn state_at(&self, context: &DriftContext, is_period_end: bool) -> BehavioralState {
        let years = context.years_elapsed();

        // EOM intensity increase
        let eom_factor = if is_period_end {
            1.0 + self.approval_drift.eom_intensity_increase_per_year * years
        } else {
            1.0
        };

        // Learning curve effect on errors
        // Error factor > 1.0 means more errors; 1.0 means baseline
        let months = context.period as f64;
        let error_factor = if months < self.error_drift.learning_curve_months as f64 {
            // New employee learning curve: starts at (1 + new_employee_error_rate) and decreases to 1.0
            let progress = months / self.error_drift.learning_curve_months as f64;
            1.0 + self.error_drift.new_employee_error_rate * (1.0 - progress)
        } else {
            // Fatigue factor after learning: gradually increases from 1.0
            let fatigue_years = (months - self.error_drift.learning_curve_months as f64) / 12.0;
            1.0 + self.error_drift.fatigue_error_increase * fatigue_years
        };

        BehavioralState {
            processing_time_factor: eom_factor,
            error_factor: error_factor.clamp(0.5, 2.0),
            ..BehavioralState::neutral()
        }
    }
}

/// Approval pattern drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPatternDrift {
    /// EOM intensity increase per year.
    #[serde(default = "default_eom_intensity")]
    pub eom_intensity_increase_per_year: f64,
    /// Volume threshold for rubber-stamp behavior.
    #[serde(default = "default_rubber_stamp")]
    pub rubber_stamp_volume_threshold: u32,
}

fn default_eom_intensity() -> f64 {
    0.05
}

fn default_rubber_stamp() -> u32 {
    50
}

impl Default for ApprovalPatternDrift {
    fn default() -> Self {
        Self {
            eom_intensity_increase_per_year: 0.05,
            rubber_stamp_volume_threshold: 50,
        }
    }
}

/// Error pattern drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPatternDrift {
    /// Initial error rate for new employees.
    #[serde(default = "default_new_error_rate")]
    pub new_employee_error_rate: f64,
    /// Learning curve duration in months.
    #[serde(default = "default_learning_months")]
    pub learning_curve_months: u32,
    /// Error rate increase due to fatigue (per year after learning).
    #[serde(default = "default_fatigue_increase")]
    pub fatigue_error_increase: f64,
}

fn default_new_error_rate() -> f64 {
    0.08
}

fn default_learning_months() -> u32 {
    6
}

fn default_fatigue_increase() -> f64 {
    0.01
}

impl Default for ErrorPatternDrift {
    fn default() -> Self {
        Self {
            new_employee_error_rate: 0.08,
            learning_curve_months: 6,
            fatigue_error_increase: 0.01,
        }
    }
}

// =============================================================================
// Collective Behavioral Drift
// =============================================================================

/// Collective behavioral drift across the organization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollectiveBehavioralDrift {
    /// Year-end intensity drift.
    #[serde(default)]
    pub year_end_intensity: YearEndIntensityDrift,
    /// Automation adoption drift.
    #[serde(default)]
    pub automation_adoption: AutomationAdoptionDrift,
    /// Remote work impact drift.
    #[serde(default)]
    pub remote_work_impact: RemoteWorkDrift,
}

impl CollectiveBehavioralDrift {
    /// Calculate collective state.
    pub fn state_at(&self, context: &DriftContext, month: u32) -> CollectiveState {
        let years = context.years_elapsed();

        // Year-end intensity
        let is_year_end = month == 11 || month == 0; // December or January
        let year_end_factor = if is_year_end {
            1.0 + self.year_end_intensity.intensity_increase_per_year * years
        } else {
            1.0
        };

        // Automation adoption (S-curve)
        let automation_rate = if self.automation_adoption.s_curve_enabled {
            let midpoint_years = self.automation_adoption.adoption_midpoint_months as f64 / 12.0;
            let steepness = self.automation_adoption.steepness;
            1.0 / (1.0 + (-steepness * (years - midpoint_years)).exp())
        } else {
            0.0
        };

        // Remote work posting time flattening
        let posting_time_variance = if self.remote_work_impact.enabled {
            1.0 - self.remote_work_impact.posting_time_flattening * years.min(2.0)
        } else {
            1.0
        };

        CollectiveState {
            year_end_intensity_factor: year_end_factor,
            automation_rate,
            posting_time_variance_factor: posting_time_variance.max(0.5),
        }
    }
}

/// Collective behavioral state.
#[derive(Debug, Clone, Default)]
pub struct CollectiveState {
    /// Year-end intensity factor.
    pub year_end_intensity_factor: f64,
    /// Automation adoption rate (0.0 to 1.0).
    pub automation_rate: f64,
    /// Posting time variance factor.
    pub posting_time_variance_factor: f64,
}

/// Year-end intensity drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearEndIntensityDrift {
    /// Intensity increase per year.
    #[serde(default = "default_intensity_increase")]
    pub intensity_increase_per_year: f64,
}

fn default_intensity_increase() -> f64 {
    0.05
}

impl Default for YearEndIntensityDrift {
    fn default() -> Self {
        Self {
            intensity_increase_per_year: 0.05,
        }
    }
}

/// Automation adoption drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationAdoptionDrift {
    /// Enable S-curve adoption model.
    #[serde(default)]
    pub s_curve_enabled: bool,
    /// Adoption midpoint in months.
    #[serde(default = "default_midpoint")]
    pub adoption_midpoint_months: u32,
    /// Steepness of adoption curve.
    #[serde(default = "default_steepness")]
    pub steepness: f64,
}

fn default_midpoint() -> u32 {
    24
}

fn default_steepness() -> f64 {
    0.15
}

impl Default for AutomationAdoptionDrift {
    fn default() -> Self {
        Self {
            s_curve_enabled: false,
            adoption_midpoint_months: 24,
            steepness: 0.15,
        }
    }
}

/// Remote work impact drift configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteWorkDrift {
    /// Enable remote work impact.
    #[serde(default)]
    pub enabled: bool,
    /// Posting time flattening factor (reduction in time-of-day variance).
    #[serde(default = "default_flattening")]
    pub posting_time_flattening: f64,
}

fn default_flattening() -> f64 {
    0.3
}

impl Default for RemoteWorkDrift {
    fn default() -> Self {
        Self {
            enabled: false,
            posting_time_flattening: 0.3,
        }
    }
}

// =============================================================================
// Behavioral Drift Controller
// =============================================================================

/// Main controller for all behavioral drift.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BehavioralDriftConfig {
    /// Enable behavioral drift.
    #[serde(default)]
    pub enabled: bool,
    /// Vendor behavioral drift.
    #[serde(default)]
    pub vendor_behavior: VendorBehavioralDrift,
    /// Customer behavioral drift.
    #[serde(default)]
    pub customer_behavior: CustomerBehavioralDrift,
    /// Employee behavioral drift.
    #[serde(default)]
    pub employee_behavior: EmployeeBehavioralDrift,
    /// Collective behavioral drift.
    #[serde(default)]
    pub collective: CollectiveBehavioralDrift,
}

impl BehavioralDriftConfig {
    /// Compute all behavioral effects for a given context.
    pub fn compute_effects(
        &self,
        context: &DriftContext,
        month: u32,
        is_period_end: bool,
    ) -> BehavioralEffects {
        if !self.enabled {
            return BehavioralEffects::neutral();
        }

        BehavioralEffects {
            vendor: self.vendor_behavior.state_at(context),
            customer: self.customer_behavior.state_at(context),
            employee: self.employee_behavior.state_at(context, is_period_end),
            collective: self.collective.state_at(context, month),
        }
    }
}

/// Combined behavioral effects.
#[derive(Debug, Clone, Default)]
pub struct BehavioralEffects {
    /// Vendor behavioral state.
    pub vendor: BehavioralState,
    /// Customer behavioral state.
    pub customer: BehavioralState,
    /// Employee behavioral state.
    pub employee: BehavioralState,
    /// Collective behavioral state.
    pub collective: CollectiveState,
}

impl BehavioralEffects {
    /// Create neutral effects.
    pub fn neutral() -> Self {
        Self {
            vendor: BehavioralState::neutral(),
            customer: BehavioralState::neutral(),
            employee: BehavioralState::neutral(),
            collective: CollectiveState::default(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_behavioral_drift() {
        let drift = VendorBehavioralDrift::default();
        let context = DriftContext {
            period: 24, // 2 years
            total_periods: 36,
            ..DriftContext::neutral()
        };

        let state = drift.state_at(&context);
        // Payment terms should have increased
        assert!(state.payment_days_delta > 0.0);
        // Quality factor should have decreased due to complacency
        assert!(state.quality_factor < 1.02);
    }

    #[test]
    fn test_customer_downturn_drift() {
        let drift = CustomerBehavioralDrift::default();
        let context = DriftContext {
            is_recession: true,
            economic_cycle_factor: 0.8,
            ..DriftContext::neutral()
        };

        let state = drift.state_at(&context);
        // Payment delays during downturn
        assert!(state.payment_days_delta > 0.0);
    }

    #[test]
    fn test_employee_learning_curve() {
        let drift = EmployeeBehavioralDrift::default();

        // New employee (month 1)
        let context_new = DriftContext {
            period: 1,
            ..DriftContext::neutral()
        };
        let state_new = drift.state_at(&context_new, false);
        assert!(state_new.error_factor > 1.0); // Higher errors initially

        // Experienced employee (month 12)
        let context_exp = DriftContext {
            period: 12,
            ..DriftContext::neutral()
        };
        let state_exp = drift.state_at(&context_exp, false);
        assert!(state_exp.error_factor < state_new.error_factor); // Lower errors
    }

    #[test]
    fn test_automation_s_curve() {
        let drift = CollectiveBehavioralDrift {
            automation_adoption: AutomationAdoptionDrift {
                s_curve_enabled: true,
                adoption_midpoint_months: 24,
                steepness: 0.15,
            },
            ..Default::default()
        };

        // Early (month 6)
        let context_early = DriftContext {
            period: 6,
            ..DriftContext::neutral()
        };
        let state_early = drift.state_at(&context_early, 6);

        // Midpoint (month 24)
        let context_mid = DriftContext {
            period: 24,
            ..DriftContext::neutral()
        };
        let state_mid = drift.state_at(&context_mid, 0);

        // Late (month 48)
        let context_late = DriftContext {
            period: 48,
            ..DriftContext::neutral()
        };
        let state_late = drift.state_at(&context_late, 0);

        // S-curve: slow start, fast middle, slow end
        assert!(state_early.automation_rate < 0.5);
        assert!((state_mid.automation_rate - 0.5).abs() < 0.2);
        assert!(state_late.automation_rate > 0.5);
    }
}
