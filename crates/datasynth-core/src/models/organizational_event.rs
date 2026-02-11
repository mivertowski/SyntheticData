//! Organizational event models for pattern drift simulation.
//!
//! Provides comprehensive organizational event modeling including:
//! - Acquisitions with integration phases
//! - Divestitures with entity removal
//! - Reorganizations with cost center remapping
//! - Leadership changes with policy shifts
//! - Workforce reductions with error rate impacts
//! - Mergers with goodwill and fair value adjustments

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Organizational event type with associated configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrganizationalEventType {
    /// Company acquisition event.
    Acquisition(AcquisitionConfig),
    /// Company or business unit divestiture.
    Divestiture(DivestitureConfig),
    /// Internal reorganization event.
    Reorganization(ReorganizationConfig),
    /// Leadership or management change.
    LeadershipChange(LeadershipChangeConfig),
    /// Workforce reduction event.
    WorkforceReduction(WorkforceReductionConfig),
    /// Company merger event.
    Merger(MergerConfig),
}

impl OrganizationalEventType {
    /// Get the event type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Acquisition(_) => "acquisition",
            Self::Divestiture(_) => "divestiture",
            Self::Reorganization(_) => "reorganization",
            Self::LeadershipChange(_) => "leadership_change",
            Self::WorkforceReduction(_) => "workforce_reduction",
            Self::Merger(_) => "merger",
        }
    }

    /// Get the default volume multiplier for this event type.
    pub fn default_volume_multiplier(&self) -> f64 {
        match self {
            Self::Acquisition(c) => c.volume_multiplier,
            Self::Divestiture(c) => c.volume_reduction,
            Self::Reorganization(_) => 1.0,
            Self::LeadershipChange(_) => 1.0,
            Self::WorkforceReduction(c) => 1.0 - (c.reduction_percent * 0.3), // 30% of reduction impacts volume
            Self::Merger(c) => c.volume_multiplier,
        }
    }

    /// Get the error rate impact for this event type.
    pub fn error_rate_impact(&self) -> f64 {
        match self {
            Self::Acquisition(c) => c.integration_error_rate,
            Self::Divestiture(_) => 0.02, // Some transition errors
            Self::Reorganization(c) => c.transition_error_rate,
            Self::LeadershipChange(c) => c.policy_change_error_rate,
            Self::WorkforceReduction(c) => c.error_rate_increase,
            Self::Merger(c) => c.integration_error_rate,
        }
    }

    /// Get the transition duration in months.
    pub fn transition_months(&self) -> u32 {
        match self {
            Self::Acquisition(c) => c.integration_phases.total_duration_months(),
            Self::Divestiture(c) => c.transition_months,
            Self::Reorganization(c) => c.transition_months,
            Self::LeadershipChange(c) => c.policy_transition_months,
            Self::WorkforceReduction(c) => c.transition_months,
            Self::Merger(c) => c.integration_phases.total_duration_months(),
        }
    }
}

/// Configuration for an acquisition event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionConfig {
    /// Code of the acquired entity.
    pub acquired_entity_code: String,
    /// Name of the acquired entity.
    #[serde(default)]
    pub acquired_entity_name: Option<String>,
    /// Date of acquisition closing.
    pub acquisition_date: NaiveDate,
    /// Volume multiplier after acquisition (e.g., 1.35 = 35% increase).
    #[serde(default = "default_acquisition_volume_mult")]
    pub volume_multiplier: f64,
    /// Error rate during integration period.
    #[serde(default = "default_integration_error_rate")]
    pub integration_error_rate: f64,
    /// Days of parallel posting (dual systems).
    #[serde(default = "default_parallel_posting_days")]
    pub parallel_posting_days: u32,
    /// Coding error rate during integration.
    #[serde(default = "default_coding_error_rate")]
    pub coding_error_rate: f64,
    /// Integration phase configuration.
    #[serde(default)]
    pub integration_phases: IntegrationPhaseConfig,
    /// Purchase price allocation details.
    #[serde(default)]
    pub purchase_price_allocation: Option<PurchasePriceAllocation>,
}

fn default_acquisition_volume_mult() -> f64 {
    1.35
}

fn default_integration_error_rate() -> f64 {
    0.05
}

fn default_parallel_posting_days() -> u32 {
    30
}

fn default_coding_error_rate() -> f64 {
    0.03
}

impl Default for AcquisitionConfig {
    fn default() -> Self {
        Self {
            acquired_entity_code: String::new(),
            acquired_entity_name: None,
            acquisition_date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"),
            volume_multiplier: 1.35,
            integration_error_rate: 0.05,
            parallel_posting_days: 30,
            coding_error_rate: 0.03,
            integration_phases: IntegrationPhaseConfig::default(),
            purchase_price_allocation: None,
        }
    }
}

/// Integration phase configuration for acquisitions and mergers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPhaseConfig {
    /// Parallel run period (both systems active).
    #[serde(default)]
    pub parallel_run: Option<DateRange>,
    /// Cutover date (switch to single system).
    pub cutover_date: NaiveDate,
    /// End of stabilization period.
    pub stabilization_end: NaiveDate,
    /// Error rate during parallel run.
    #[serde(default = "default_parallel_error_rate")]
    pub parallel_run_error_rate: f64,
    /// Error rate during stabilization.
    #[serde(default = "default_stabilization_error_rate")]
    pub stabilization_error_rate: f64,
}

fn default_parallel_error_rate() -> f64 {
    0.08
}

fn default_stabilization_error_rate() -> f64 {
    0.03
}

impl Default for IntegrationPhaseConfig {
    fn default() -> Self {
        let cutover = NaiveDate::from_ymd_opt(2024, 3, 1).expect("valid default date");
        Self {
            parallel_run: Some(DateRange {
                start: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"),
                end: NaiveDate::from_ymd_opt(2024, 2, 28).expect("valid default date"),
            }),
            cutover_date: cutover,
            stabilization_end: NaiveDate::from_ymd_opt(2024, 5, 31).expect("valid default date"),
            parallel_run_error_rate: 0.08,
            stabilization_error_rate: 0.03,
        }
    }
}

impl IntegrationPhaseConfig {
    /// Calculate total integration duration in months.
    pub fn total_duration_months(&self) -> u32 {
        let days = (self.stabilization_end - self.cutover_date).num_days();
        let parallel_days = self
            .parallel_run
            .as_ref()
            .map(|r| (r.end - r.start).num_days())
            .unwrap_or(0);
        ((days + parallel_days) / 30) as u32
    }

    /// Get the current integration phase for a given date.
    pub fn phase_at(&self, date: NaiveDate) -> IntegrationPhase {
        if let Some(ref parallel) = self.parallel_run {
            if date >= parallel.start && date <= parallel.end {
                return IntegrationPhase::ParallelRun;
            }
        }
        if date >= self.cutover_date && date <= self.stabilization_end {
            IntegrationPhase::Stabilization
        } else if date > self.stabilization_end {
            IntegrationPhase::Complete
        } else {
            IntegrationPhase::PreIntegration
        }
    }

    /// Get the error rate for a given date.
    pub fn error_rate_at(&self, date: NaiveDate) -> f64 {
        match self.phase_at(date) {
            IntegrationPhase::PreIntegration => 0.0,
            IntegrationPhase::ParallelRun => self.parallel_run_error_rate,
            IntegrationPhase::Stabilization => self.stabilization_error_rate,
            IntegrationPhase::Complete => 0.0,
        }
    }
}

/// Integration phase stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationPhase {
    /// Before integration starts.
    PreIntegration,
    /// Both systems running in parallel.
    ParallelRun,
    /// Post-cutover stabilization.
    Stabilization,
    /// Integration complete.
    Complete,
}

/// Date range helper struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date (inclusive).
    pub start: NaiveDate,
    /// End date (inclusive).
    pub end: NaiveDate,
}

impl DateRange {
    /// Check if a date is within this range.
    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }

    /// Get duration in days.
    pub fn duration_days(&self) -> i64 {
        (self.end - self.start).num_days()
    }
}

/// Purchase price allocation details for acquisition accounting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchasePriceAllocation {
    /// Total purchase price.
    #[serde(with = "rust_decimal::serde::str")]
    pub purchase_price: Decimal,
    /// Fair value of net identifiable assets.
    #[serde(with = "rust_decimal::serde::str")]
    pub net_identifiable_assets: Decimal,
    /// Goodwill recognized (purchase price - net identifiable assets).
    #[serde(with = "rust_decimal::serde::str")]
    pub goodwill: Decimal,
    /// Bargain purchase gain if applicable.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub bargain_purchase_gain: Option<Decimal>,
    /// Intangible assets acquired.
    #[serde(default)]
    pub intangible_assets: Vec<IntangibleAsset>,
}

/// Intangible asset acquired in an acquisition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntangibleAsset {
    /// Asset type (e.g., "customer_relationships", "brand", "technology").
    pub asset_type: String,
    /// Fair value.
    #[serde(with = "rust_decimal::serde::str")]
    pub fair_value: Decimal,
    /// Useful life in years (None = indefinite).
    pub useful_life_years: Option<u8>,
}

/// Configuration for a divestiture event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivestitureConfig {
    /// Code of the divested entity.
    pub divested_entity_code: String,
    /// Name of the divested entity.
    #[serde(default)]
    pub divested_entity_name: Option<String>,
    /// Date of divestiture closing.
    pub divestiture_date: NaiveDate,
    /// Revenue reduction factor (e.g., 0.70 = 30% reduction).
    #[serde(default = "default_volume_reduction")]
    pub volume_reduction: f64,
    /// Number of months for transition.
    #[serde(default = "default_transition_months")]
    pub transition_months: u32,
    /// Whether to remove entity from subsequent generation.
    #[serde(default = "default_true")]
    pub remove_entity: bool,
    /// Accounts to be remapped or closed.
    #[serde(default)]
    pub account_closures: Vec<String>,
    /// Gain/loss on disposal.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub disposal_gain_loss: Option<Decimal>,
}

fn default_volume_reduction() -> f64 {
    0.70
}

fn default_transition_months() -> u32 {
    3
}

fn default_true() -> bool {
    true
}

impl Default for DivestitureConfig {
    fn default() -> Self {
        Self {
            divested_entity_code: String::new(),
            divested_entity_name: None,
            divestiture_date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"),
            volume_reduction: 0.70,
            transition_months: 3,
            remove_entity: true,
            account_closures: Vec::new(),
            disposal_gain_loss: None,
        }
    }
}

/// Configuration for a reorganization event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorganizationConfig {
    /// Description of the reorganization.
    #[serde(default)]
    pub description: Option<String>,
    /// Effective date of reorganization.
    pub effective_date: NaiveDate,
    /// Cost center remapping (old -> new).
    #[serde(default)]
    pub cost_center_remapping: HashMap<String, String>,
    /// Department remapping (old -> new).
    #[serde(default)]
    pub department_remapping: HashMap<String, String>,
    /// Reporting line changes.
    #[serde(default)]
    pub reporting_changes: Vec<ReportingChange>,
    /// Number of months for transition.
    #[serde(default = "default_transition_months")]
    pub transition_months: u32,
    /// Error rate during transition.
    #[serde(default = "default_reorg_error_rate")]
    pub transition_error_rate: f64,
}

fn default_reorg_error_rate() -> f64 {
    0.04
}

impl Default for ReorganizationConfig {
    fn default() -> Self {
        Self {
            description: None,
            effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"),
            cost_center_remapping: HashMap::new(),
            department_remapping: HashMap::new(),
            reporting_changes: Vec::new(),
            transition_months: 3,
            transition_error_rate: 0.04,
        }
    }
}

/// A reporting line change in a reorganization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingChange {
    /// Entity or department being changed.
    pub entity: String,
    /// Previous reporting to.
    pub from_reports_to: String,
    /// New reporting to.
    pub to_reports_to: String,
}

/// Configuration for a leadership change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadershipChangeConfig {
    /// Role that changed (e.g., "CFO", "Controller").
    pub role: String,
    /// Date of change.
    pub change_date: NaiveDate,
    /// Policy changes associated with leadership change.
    #[serde(default)]
    pub policy_changes: Vec<PolicyChangeDetail>,
    /// Vendor review triggered.
    #[serde(default)]
    pub vendor_review_triggered: bool,
    /// Number of months for policy transition.
    #[serde(default = "default_policy_transition_months")]
    pub policy_transition_months: u32,
    /// Error rate during policy transition.
    #[serde(default = "default_policy_error_rate")]
    pub policy_change_error_rate: f64,
}

fn default_policy_transition_months() -> u32 {
    6
}

fn default_policy_error_rate() -> f64 {
    0.02
}

impl Default for LeadershipChangeConfig {
    fn default() -> Self {
        Self {
            role: "CFO".to_string(),
            change_date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"),
            policy_changes: Vec::new(),
            vendor_review_triggered: false,
            policy_transition_months: 6,
            policy_change_error_rate: 0.02,
        }
    }
}

/// A specific policy change detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChangeDetail {
    /// Policy area affected.
    pub policy_area: PolicyArea,
    /// Description of change.
    pub description: String,
    /// Old threshold or value (if applicable).
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub old_value: Option<Decimal>,
    /// New threshold or value (if applicable).
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub new_value: Option<Decimal>,
}

/// Policy area categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyArea {
    /// Approval thresholds.
    ApprovalThreshold,
    /// Vendor management.
    VendorManagement,
    /// Expense policies.
    ExpensePolicy,
    /// Revenue recognition.
    RevenueRecognition,
    /// Internal controls.
    InternalControls,
    /// Risk management.
    RiskManagement,
    /// Other policy area.
    Other,
}

/// Configuration for a workforce reduction event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkforceReductionConfig {
    /// Date of reduction.
    pub reduction_date: NaiveDate,
    /// Percentage of workforce reduced (0.0 to 1.0).
    #[serde(default = "default_reduction_percent")]
    pub reduction_percent: f64,
    /// Affected departments.
    #[serde(default)]
    pub affected_departments: Vec<String>,
    /// Error rate increase due to understaffing.
    #[serde(default = "default_error_increase")]
    pub error_rate_increase: f64,
    /// Processing time increase factor.
    #[serde(default = "default_processing_increase")]
    pub processing_time_increase: f64,
    /// Number of months for transition.
    #[serde(default = "default_workforce_transition")]
    pub transition_months: u32,
    /// Severance costs.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub severance_costs: Option<Decimal>,
}

fn default_reduction_percent() -> f64 {
    0.10
}

fn default_error_increase() -> f64 {
    0.05
}

fn default_processing_increase() -> f64 {
    1.3
}

fn default_workforce_transition() -> u32 {
    6
}

impl Default for WorkforceReductionConfig {
    fn default() -> Self {
        Self {
            reduction_date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"),
            reduction_percent: 0.10,
            affected_departments: Vec::new(),
            error_rate_increase: 0.05,
            processing_time_increase: 1.3,
            transition_months: 6,
            severance_costs: None,
        }
    }
}

/// Configuration for a merger event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergerConfig {
    /// Code of the merged entity.
    pub merged_entity_code: String,
    /// Name of the merged entity.
    #[serde(default)]
    pub merged_entity_name: Option<String>,
    /// Merger closing date.
    pub merger_date: NaiveDate,
    /// Volume multiplier after merger.
    #[serde(default = "default_merger_volume_mult")]
    pub volume_multiplier: f64,
    /// Integration error rate.
    #[serde(default = "default_integration_error_rate")]
    pub integration_error_rate: f64,
    /// Integration phase configuration.
    #[serde(default)]
    pub integration_phases: IntegrationPhaseConfig,
    /// Fair value adjustments required.
    #[serde(default)]
    pub fair_value_adjustments: Vec<FairValueAdjustment>,
    /// Goodwill recognized.
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub goodwill: Option<Decimal>,
}

fn default_merger_volume_mult() -> f64 {
    1.80
}

impl Default for MergerConfig {
    fn default() -> Self {
        Self {
            merged_entity_code: String::new(),
            merged_entity_name: None,
            merger_date: NaiveDate::from_ymd_opt(2024, 1, 1).expect("valid default date"),
            volume_multiplier: 1.80,
            integration_error_rate: 0.05,
            integration_phases: IntegrationPhaseConfig::default(),
            fair_value_adjustments: Vec::new(),
            goodwill: None,
        }
    }
}

/// Fair value adjustment for merger accounting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairValueAdjustment {
    /// Account affected.
    pub account: String,
    /// Adjustment amount.
    #[serde(with = "rust_decimal::serde::str")]
    pub adjustment_amount: Decimal,
    /// Reason for adjustment.
    pub reason: String,
}

/// A scheduled organizational event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalEvent {
    /// Unique event ID.
    pub event_id: String,
    /// Event type and configuration.
    pub event_type: OrganizationalEventType,
    /// Effective date of the event.
    pub effective_date: NaiveDate,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl OrganizationalEvent {
    /// Create a new organizational event.
    pub fn new(event_id: impl Into<String>, event_type: OrganizationalEventType) -> Self {
        let effective_date = match &event_type {
            OrganizationalEventType::Acquisition(c) => c.acquisition_date,
            OrganizationalEventType::Divestiture(c) => c.divestiture_date,
            OrganizationalEventType::Reorganization(c) => c.effective_date,
            OrganizationalEventType::LeadershipChange(c) => c.change_date,
            OrganizationalEventType::WorkforceReduction(c) => c.reduction_date,
            OrganizationalEventType::Merger(c) => c.merger_date,
        };

        Self {
            event_id: event_id.into(),
            event_type,
            effective_date,
            description: None,
            tags: Vec::new(),
        }
    }

    /// Check if the event is active at a given date.
    pub fn is_active_at(&self, date: NaiveDate) -> bool {
        if date < self.effective_date {
            return false;
        }
        let transition_months = self.event_type.transition_months();
        let end_date = self.effective_date + chrono::Duration::days(transition_months as i64 * 30);
        date <= end_date
    }

    /// Get the progress through the event (0.0 to 1.0).
    pub fn progress_at(&self, date: NaiveDate) -> f64 {
        if date < self.effective_date {
            return 0.0;
        }
        let transition_months = self.event_type.transition_months();
        if transition_months == 0 {
            return 1.0;
        }
        let days_elapsed = (date - self.effective_date).num_days() as f64;
        let total_days = transition_months as f64 * 30.0;
        (days_elapsed / total_days).min(1.0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_acquisition_config_defaults() {
        let config = AcquisitionConfig::default();
        assert!((config.volume_multiplier - 1.35).abs() < 0.001);
        assert!((config.integration_error_rate - 0.05).abs() < 0.001);
        assert_eq!(config.parallel_posting_days, 30);
    }

    #[test]
    fn test_integration_phase_detection() {
        let parallel_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let parallel_end = NaiveDate::from_ymd_opt(2024, 2, 28).unwrap();
        let cutover = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let stab_end = NaiveDate::from_ymd_opt(2024, 5, 31).unwrap();

        let config = IntegrationPhaseConfig {
            parallel_run: Some(DateRange {
                start: parallel_start,
                end: parallel_end,
            }),
            cutover_date: cutover,
            stabilization_end: stab_end,
            parallel_run_error_rate: 0.08,
            stabilization_error_rate: 0.03,
        };

        assert_eq!(
            config.phase_at(NaiveDate::from_ymd_opt(2023, 12, 1).unwrap()),
            IntegrationPhase::PreIntegration
        );
        assert_eq!(
            config.phase_at(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            IntegrationPhase::ParallelRun
        );
        assert_eq!(
            config.phase_at(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()),
            IntegrationPhase::Stabilization
        );
        assert_eq!(
            config.phase_at(NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()),
            IntegrationPhase::Complete
        );
    }

    #[test]
    fn test_organizational_event_progress() {
        let config = AcquisitionConfig {
            acquisition_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            integration_phases: IntegrationPhaseConfig {
                parallel_run: None,
                cutover_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                stabilization_end: NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(), // ~3 months
                parallel_run_error_rate: 0.0,
                stabilization_error_rate: 0.03,
            },
            ..Default::default()
        };

        let event =
            OrganizationalEvent::new("ACQ-001", OrganizationalEventType::Acquisition(config));

        // Before event
        let before = NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
        assert!(!event.is_active_at(before));
        assert!((event.progress_at(before) - 0.0).abs() < 0.001);

        // During event
        let during = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        assert!(event.is_active_at(during));
        assert!(event.progress_at(during) > 0.0 && event.progress_at(during) < 1.0);
    }

    #[test]
    fn test_date_range() {
        let range = DateRange {
            start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
        };

        assert!(range.contains(NaiveDate::from_ymd_opt(2024, 2, 15).unwrap()));
        assert!(!range.contains(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()));
        assert!(!range.contains(NaiveDate::from_ymd_opt(2024, 4, 1).unwrap()));
        assert_eq!(range.duration_days(), 90);
    }
}
