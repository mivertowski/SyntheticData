//! Drift event types for ground truth labeling.
//!
//! Provides comprehensive drift event typing for ML model training:
//! - Statistical shifts (mean, variance, distribution)
//! - Categorical shifts (proportions, new categories)
//! - Temporal shifts (seasonality, trends)
//! - Regulatory and audit focus changes

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Drift event type with associated metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum DriftEventType {
    /// Statistical distribution shift.
    Statistical(StatisticalDriftEvent),
    /// Categorical distribution shift.
    Categorical(CategoricalDriftEvent),
    /// Temporal pattern shift.
    Temporal(TemporalDriftEvent),
    /// Organizational event drift.
    Organizational(OrganizationalDriftEvent),
    /// Process evolution drift.
    Process(ProcessDriftEvent),
    /// Technology transition drift.
    Technology(TechnologyDriftEvent),
    /// Regulatory change drift.
    Regulatory(RegulatoryDriftLabel),
    /// Audit focus shift.
    AuditFocus(AuditFocusDriftEvent),
    /// Market/economic drift.
    Market(MarketDriftEvent),
    /// Behavioral drift.
    Behavioral(BehavioralDriftEvent),
}

impl DriftEventType {
    /// Get the category name.
    pub fn category_name(&self) -> &'static str {
        match self {
            Self::Statistical(_) => "statistical",
            Self::Categorical(_) => "categorical",
            Self::Temporal(_) => "temporal",
            Self::Organizational(_) => "organizational",
            Self::Process(_) => "process",
            Self::Technology(_) => "technology",
            Self::Regulatory(_) => "regulatory",
            Self::AuditFocus(_) => "audit_focus",
            Self::Market(_) => "market",
            Self::Behavioral(_) => "behavioral",
        }
    }

    /// Get the specific type name.
    pub fn type_name(&self) -> &str {
        match self {
            Self::Statistical(e) => e.shift_type.as_str(),
            Self::Categorical(e) => e.shift_type.as_str(),
            Self::Temporal(e) => e.shift_type.as_str(),
            Self::Organizational(e) => &e.event_type,
            Self::Process(e) => &e.process_type,
            Self::Technology(e) => &e.transition_type,
            Self::Regulatory(e) => &e.regulation_type,
            Self::AuditFocus(e) => &e.focus_type,
            Self::Market(e) => e.market_type.as_str(),
            Self::Behavioral(e) => &e.behavior_type,
        }
    }

    /// Get the detection difficulty.
    pub fn detection_difficulty(&self) -> DetectionDifficulty {
        match self {
            Self::Statistical(e) => e.detection_difficulty,
            Self::Categorical(e) => e.detection_difficulty,
            Self::Temporal(e) => e.detection_difficulty,
            Self::Organizational(e) => e.detection_difficulty,
            Self::Process(e) => e.detection_difficulty,
            Self::Technology(e) => e.detection_difficulty,
            Self::Regulatory(e) => e.detection_difficulty,
            Self::AuditFocus(e) => e.detection_difficulty,
            Self::Market(e) => e.detection_difficulty,
            Self::Behavioral(e) => e.detection_difficulty,
        }
    }
}

/// Detection difficulty level for drift events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DetectionDifficulty {
    /// Easy to detect (large magnitude, clear signal).
    Easy,
    /// Medium difficulty (moderate signal-to-noise).
    #[default]
    Medium,
    /// Hard to detect (subtle, gradual, or noisy).
    Hard,
}

impl DetectionDifficulty {
    /// Get a numeric score (0.0 = easy, 1.0 = hard).
    pub fn score(&self) -> f64 {
        match self {
            Self::Easy => 0.0,
            Self::Medium => 0.5,
            Self::Hard => 1.0,
        }
    }
}

// =============================================================================
// Statistical Drift Events
// =============================================================================

/// Statistical drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalDriftEvent {
    /// Type of statistical shift.
    pub shift_type: StatisticalShiftType,
    /// Affected field/feature.
    pub affected_field: String,
    /// Magnitude of the shift.
    pub magnitude: f64,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Additional metrics.
    #[serde(default)]
    pub metrics: HashMap<String, f64>,
}

/// Type of statistical shift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatisticalShiftType {
    /// Mean shift.
    MeanShift,
    /// Variance change.
    VarianceChange,
    /// Distribution shape change.
    DistributionChange,
    /// Correlation change.
    CorrelationChange,
    /// Tail behavior change.
    TailChange,
    /// Benford distribution deviation.
    BenfordDeviation,
}

impl StatisticalShiftType {
    /// Get the type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MeanShift => "mean_shift",
            Self::VarianceChange => "variance_change",
            Self::DistributionChange => "distribution_change",
            Self::CorrelationChange => "correlation_change",
            Self::TailChange => "tail_change",
            Self::BenfordDeviation => "benford_deviation",
        }
    }
}

// =============================================================================
// Categorical Drift Events
// =============================================================================

/// Categorical drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoricalDriftEvent {
    /// Type of categorical shift.
    pub shift_type: CategoricalShiftType,
    /// Affected field/feature.
    pub affected_field: String,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Category proportions before (if applicable).
    #[serde(default)]
    pub proportions_before: HashMap<String, f64>,
    /// Category proportions after (if applicable).
    #[serde(default)]
    pub proportions_after: HashMap<String, f64>,
    /// New categories introduced.
    #[serde(default)]
    pub new_categories: Vec<String>,
    /// Categories removed.
    #[serde(default)]
    pub removed_categories: Vec<String>,
}

/// Type of categorical shift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CategoricalShiftType {
    /// Proportion shift between existing categories.
    ProportionShift,
    /// New category introduced.
    NewCategory,
    /// Category removed/deprecated.
    CategoryRemoval,
    /// Category consolidation.
    Consolidation,
}

impl CategoricalShiftType {
    /// Get the type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProportionShift => "proportion_shift",
            Self::NewCategory => "new_category",
            Self::CategoryRemoval => "category_removal",
            Self::Consolidation => "consolidation",
        }
    }
}

// =============================================================================
// Temporal Drift Events
// =============================================================================

/// Temporal drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDriftEvent {
    /// Type of temporal shift.
    pub shift_type: TemporalShiftType,
    /// Affected field/feature.
    #[serde(default)]
    pub affected_field: Option<String>,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Magnitude of change.
    #[serde(default)]
    pub magnitude: f64,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Type of temporal shift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalShiftType {
    /// Seasonality pattern change.
    SeasonalityChange,
    /// Trend change.
    TrendChange,
    /// Periodicity change.
    PeriodicityChange,
    /// Intraday pattern change.
    IntradayChange,
    /// Processing lag change.
    LagChange,
}

impl TemporalShiftType {
    /// Get the type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SeasonalityChange => "seasonality_change",
            Self::TrendChange => "trend_change",
            Self::PeriodicityChange => "periodicity_change",
            Self::IntradayChange => "intraday_change",
            Self::LagChange => "lag_change",
        }
    }
}

// =============================================================================
// Organizational, Process, and Technology Drift Events
// =============================================================================

/// Organizational drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalDriftEvent {
    /// Event type (e.g., "acquisition", "divestiture").
    pub event_type: String,
    /// Related event ID.
    pub related_event_id: String,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Affected entities.
    #[serde(default)]
    pub affected_entities: Vec<String>,
    /// Impact metrics.
    #[serde(default)]
    pub impact_metrics: HashMap<String, f64>,
}

/// Process drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDriftEvent {
    /// Process type (e.g., "automation", "workflow_change").
    pub process_type: String,
    /// Related event ID.
    pub related_event_id: String,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Affected processes.
    #[serde(default)]
    pub affected_processes: Vec<String>,
}

/// Technology drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyDriftEvent {
    /// Transition type (e.g., "erp_migration", "module_implementation").
    pub transition_type: String,
    /// Related event ID.
    pub related_event_id: String,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Systems involved.
    #[serde(default)]
    pub systems: Vec<String>,
    /// Current phase.
    #[serde(default)]
    pub current_phase: Option<String>,
}

// =============================================================================
// Regulatory and Audit Drift Events
// =============================================================================

/// Regulatory drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryDriftLabel {
    /// Regulation type.
    pub regulation_type: String,
    /// Standard or regulation name.
    pub regulation_name: String,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Affected accounts.
    #[serde(default)]
    pub affected_accounts: Vec<String>,
    /// Compliance framework.
    #[serde(default)]
    pub framework: Option<String>,
}

/// Audit focus drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFocusDriftEvent {
    /// Focus type.
    pub focus_type: String,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Risk areas.
    #[serde(default)]
    pub risk_areas: Vec<String>,
    /// Priority level.
    #[serde(default)]
    pub priority_level: u8,
}

// =============================================================================
// Market and Behavioral Drift Events
// =============================================================================

/// Market drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDriftEvent {
    /// Market type.
    pub market_type: MarketEventType,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Magnitude.
    #[serde(default)]
    pub magnitude: f64,
    /// Is recession.
    #[serde(default)]
    pub is_recession: bool,
    /// Affected sectors.
    #[serde(default)]
    pub affected_sectors: Vec<String>,
}

/// Market event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketEventType {
    /// Economic cycle change.
    EconomicCycle,
    /// Recession start.
    RecessionStart,
    /// Recession end.
    RecessionEnd,
    /// Price shock.
    PriceShock,
    /// Commodity price change.
    CommodityChange,
}

impl MarketEventType {
    /// Get the type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EconomicCycle => "economic_cycle",
            Self::RecessionStart => "recession_start",
            Self::RecessionEnd => "recession_end",
            Self::PriceShock => "price_shock",
            Self::CommodityChange => "commodity_change",
        }
    }
}

/// Behavioral drift event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralDriftEvent {
    /// Behavior type (e.g., "vendor_quality", "customer_payment").
    pub behavior_type: String,
    /// Entity type affected.
    pub entity_type: String,
    /// Detection difficulty.
    #[serde(default)]
    pub detection_difficulty: DetectionDifficulty,
    /// Behavior metrics.
    #[serde(default)]
    pub metrics: HashMap<String, f64>,
}

// =============================================================================
// Labeled Drift Event
// =============================================================================

/// A labeled drift event with full metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledDriftEvent {
    /// Unique event ID.
    pub event_id: String,
    /// Event type.
    pub event_type: DriftEventType,
    /// Start date.
    pub start_date: NaiveDate,
    /// End date (None for ongoing).
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    /// Start period (0-indexed).
    pub start_period: u32,
    /// End period (None for ongoing).
    #[serde(default)]
    pub end_period: Option<u32>,
    /// Affected fields/features.
    #[serde(default)]
    pub affected_fields: Vec<String>,
    /// Magnitude of the drift.
    pub magnitude: f64,
    /// Detection difficulty.
    pub detection_difficulty: DetectionDifficulty,
    /// Related organizational event ID if applicable.
    #[serde(default)]
    pub related_org_event: Option<String>,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl LabeledDriftEvent {
    /// Create a new labeled drift event.
    pub fn new(
        event_id: impl Into<String>,
        event_type: DriftEventType,
        start_date: NaiveDate,
        start_period: u32,
        magnitude: f64,
    ) -> Self {
        let detection_difficulty = event_type.detection_difficulty();
        Self {
            event_id: event_id.into(),
            event_type,
            start_date,
            end_date: None,
            start_period,
            end_period: None,
            affected_fields: Vec::new(),
            magnitude,
            detection_difficulty,
            related_org_event: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Check if the event is active at a given period.
    pub fn is_active_at(&self, period: u32) -> bool {
        if period < self.start_period {
            return false;
        }
        match self.end_period {
            Some(end) => period <= end,
            None => true,
        }
    }

    /// Get the duration in periods (None if ongoing).
    pub fn duration_periods(&self) -> Option<u32> {
        self.end_period.map(|end| end - self.start_period + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_event_type_names() {
        let stat_event = DriftEventType::Statistical(StatisticalDriftEvent {
            shift_type: StatisticalShiftType::MeanShift,
            affected_field: "amount".to_string(),
            magnitude: 0.15,
            detection_difficulty: DetectionDifficulty::Easy,
            metrics: HashMap::new(),
        });

        assert_eq!(stat_event.category_name(), "statistical");
        assert_eq!(stat_event.type_name(), "mean_shift");
    }

    #[test]
    fn test_labeled_drift_event() {
        let event = LabeledDriftEvent::new(
            "DRIFT-001",
            DriftEventType::Statistical(StatisticalDriftEvent {
                shift_type: StatisticalShiftType::MeanShift,
                affected_field: "amount".to_string(),
                magnitude: 0.20,
                detection_difficulty: DetectionDifficulty::Medium,
                metrics: HashMap::new(),
            }),
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            6,
            0.20,
        );

        assert!(event.is_active_at(6));
        assert!(event.is_active_at(12)); // Ongoing
        assert!(!event.is_active_at(5));
    }

    #[test]
    fn test_detection_difficulty_score() {
        assert!(DetectionDifficulty::Easy.score() < DetectionDifficulty::Medium.score());
        assert!(DetectionDifficulty::Medium.score() < DetectionDifficulty::Hard.score());
    }
}
