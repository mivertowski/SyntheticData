//! Technology transition models for pattern drift simulation.
//!
//! Provides comprehensive technology change modeling including:
//! - ERP migrations with cutover phases
//! - Module implementations
//! - Integration upgrades

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Technology transition event type with associated configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TechnologyTransitionType {
    /// Full ERP system migration.
    ErpMigration(ErpMigrationConfig),
    /// New module implementation.
    ModuleImplementation(ModuleImplementationConfig),
    /// Integration or interface upgrade.
    IntegrationUpgrade(IntegrationUpgradeConfig),
}

impl TechnologyTransitionType {
    /// Get the event type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::ErpMigration(_) => "erp_migration",
            Self::ModuleImplementation(_) => "module_implementation",
            Self::IntegrationUpgrade(_) => "integration_upgrade",
        }
    }

    /// Get the error rate impact for this transition.
    pub fn error_rate_impact(&self) -> f64 {
        match self {
            Self::ErpMigration(c) => c.migration_issues.combined_error_rate(),
            Self::ModuleImplementation(c) => c.implementation_error_rate,
            Self::IntegrationUpgrade(c) => c.transition_error_rate,
        }
    }

    /// Get the transition duration in months.
    pub fn transition_months(&self) -> u32 {
        match self {
            Self::ErpMigration(c) => c.phases.total_duration_months(),
            Self::ModuleImplementation(c) => c.rollout_months,
            Self::IntegrationUpgrade(c) => c.transition_months,
        }
    }
}

/// Configuration for ERP migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErpMigrationConfig {
    /// Source system identifier.
    pub source_system: String,
    /// Target system identifier.
    pub target_system: String,
    /// Migration phases configuration.
    #[serde(default)]
    pub phases: MigrationPhases,
    /// Migration issue configuration.
    #[serde(default)]
    pub migration_issues: MigrationIssueConfig,
    /// Data migration strategy.
    #[serde(default)]
    pub data_migration_strategy: DataMigrationStrategy,
    /// Entities being migrated.
    #[serde(default)]
    pub migrated_entities: Vec<String>,
    /// Legacy system decommission date.
    #[serde(default)]
    pub decommission_date: Option<NaiveDate>,
}

impl Default for ErpMigrationConfig {
    fn default() -> Self {
        Self {
            source_system: "SAP_R3".to_string(),
            target_system: "SAP_S4HANA".to_string(),
            phases: MigrationPhases::default(),
            migration_issues: MigrationIssueConfig::default(),
            data_migration_strategy: DataMigrationStrategy::BigBang,
            migrated_entities: Vec::new(),
            decommission_date: None,
        }
    }
}

/// Migration phases configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPhases {
    /// Preparation phase start.
    #[serde(default)]
    pub preparation_start: Option<NaiveDate>,
    /// Data migration start.
    #[serde(default)]
    pub data_migration_start: Option<NaiveDate>,
    /// Parallel run period start.
    #[serde(default)]
    pub parallel_run_start: Option<NaiveDate>,
    /// Cutover date (go-live).
    pub cutover_date: NaiveDate,
    /// Stabilization end date.
    pub stabilization_end: NaiveDate,
    /// Hypercare end date.
    #[serde(default)]
    pub hypercare_end: Option<NaiveDate>,
}

impl Default for MigrationPhases {
    fn default() -> Self {
        Self {
            preparation_start: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            data_migration_start: Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
            parallel_run_start: Some(NaiveDate::from_ymd_opt(2024, 8, 1).unwrap()),
            cutover_date: NaiveDate::from_ymd_opt(2024, 9, 1).unwrap(),
            stabilization_end: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
            hypercare_end: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        }
    }
}

impl MigrationPhases {
    /// Calculate total migration duration in months.
    pub fn total_duration_months(&self) -> u32 {
        let start = self.preparation_start.unwrap_or(self.cutover_date);
        let end = self.hypercare_end.unwrap_or(self.stabilization_end);
        let days = (end - start).num_days();
        (days / 30) as u32
    }

    /// Get the current migration phase for a given date.
    pub fn phase_at(&self, date: NaiveDate) -> MigrationPhase {
        if let Some(prep) = self.preparation_start {
            if date < prep {
                return MigrationPhase::PreMigration;
            }
        }

        if let Some(data_mig) = self.data_migration_start {
            if date < data_mig {
                return MigrationPhase::Preparation;
            }
        }

        if let Some(parallel) = self.parallel_run_start {
            if date < parallel {
                return MigrationPhase::DataMigration;
            }
            if date < self.cutover_date {
                return MigrationPhase::ParallelRun;
            }
        }

        if date < self.cutover_date {
            return MigrationPhase::DataMigration;
        }

        if date < self.stabilization_end {
            return MigrationPhase::Stabilization;
        }

        if let Some(hypercare) = self.hypercare_end {
            if date < hypercare {
                return MigrationPhase::Hypercare;
            }
        }

        MigrationPhase::Complete
    }
}

/// Migration phase stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationPhase {
    /// Before migration starts.
    PreMigration,
    /// Preparation and planning.
    Preparation,
    /// Data migration in progress.
    DataMigration,
    /// Both systems running in parallel.
    ParallelRun,
    /// Post-cutover stabilization.
    Stabilization,
    /// Hypercare period.
    Hypercare,
    /// Migration complete.
    Complete,
}

impl MigrationPhase {
    /// Get the typical error rate multiplier for this phase.
    pub fn error_rate_multiplier(&self) -> f64 {
        match self {
            Self::PreMigration => 1.0,
            Self::Preparation => 1.0,
            Self::DataMigration => 1.5,
            Self::ParallelRun => 2.0,
            Self::Stabilization => 1.8,
            Self::Hypercare => 1.3,
            Self::Complete => 1.0,
        }
    }

    /// Get the typical processing time multiplier.
    pub fn processing_time_multiplier(&self) -> f64 {
        match self {
            Self::PreMigration => 1.0,
            Self::Preparation => 1.0,
            Self::DataMigration => 1.2,
            Self::ParallelRun => 1.5, // Dual entry
            Self::Stabilization => 1.3,
            Self::Hypercare => 1.1,
            Self::Complete => 1.0,
        }
    }
}

/// Data migration strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataMigrationStrategy {
    /// All at once migration.
    #[default]
    BigBang,
    /// Phased migration by module or entity.
    Phased,
    /// Parallel run with gradual cutover.
    Parallel,
    /// Hybrid approach.
    Hybrid,
}

impl DataMigrationStrategy {
    /// Get the risk level associated with this strategy.
    pub fn risk_level(&self) -> &'static str {
        match self {
            Self::BigBang => "high",
            Self::Phased => "medium",
            Self::Parallel => "low",
            Self::Hybrid => "medium",
        }
    }
}

/// Migration issue configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationIssueConfig {
    /// Rate of duplicate records during migration.
    #[serde(default = "default_duplicate_rate")]
    pub duplicate_rate: f64,
    /// Rate of missing records.
    #[serde(default = "default_missing_rate")]
    pub missing_rate: f64,
    /// Rate of format mismatch issues.
    #[serde(default = "default_format_mismatch_rate")]
    pub format_mismatch_rate: f64,
    /// Rate of mapping errors.
    #[serde(default = "default_mapping_error_rate")]
    pub mapping_error_rate: f64,
    /// Rate of timing/cutoff issues.
    #[serde(default = "default_cutoff_issue_rate")]
    pub cutoff_issue_rate: f64,
}

fn default_duplicate_rate() -> f64 {
    0.02
}

fn default_missing_rate() -> f64 {
    0.01
}

fn default_format_mismatch_rate() -> f64 {
    0.03
}

fn default_mapping_error_rate() -> f64 {
    0.02
}

fn default_cutoff_issue_rate() -> f64 {
    0.01
}

impl Default for MigrationIssueConfig {
    fn default() -> Self {
        Self {
            duplicate_rate: 0.02,
            missing_rate: 0.01,
            format_mismatch_rate: 0.03,
            mapping_error_rate: 0.02,
            cutoff_issue_rate: 0.01,
        }
    }
}

impl MigrationIssueConfig {
    /// Calculate combined error rate.
    pub fn combined_error_rate(&self) -> f64 {
        // These aren't strictly additive, but this gives a reasonable approximation
        (self.duplicate_rate
            + self.missing_rate
            + self.format_mismatch_rate
            + self.mapping_error_rate
            + self.cutoff_issue_rate)
            .min(0.20) // Cap at 20%
    }
}

/// Configuration for module implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleImplementationConfig {
    /// Module being implemented.
    pub module_name: String,
    /// System the module is being added to.
    #[serde(default)]
    pub target_system: Option<String>,
    /// Go-live date.
    pub go_live_date: NaiveDate,
    /// Number of months for rollout.
    #[serde(default = "default_module_rollout")]
    pub rollout_months: u32,
    /// Implementation error rate.
    #[serde(default = "default_implementation_error_rate")]
    pub implementation_error_rate: f64,
    /// Training completion rate (0.0 to 1.0).
    #[serde(default = "default_training_rate")]
    pub training_completion_rate: f64,
    /// Affected business processes.
    #[serde(default)]
    pub affected_processes: Vec<String>,
    /// Configuration changes.
    #[serde(default)]
    pub configuration_changes: Vec<ConfigurationChange>,
}

fn default_module_rollout() -> u32 {
    4
}

fn default_implementation_error_rate() -> f64 {
    0.04
}

fn default_training_rate() -> f64 {
    0.85
}

impl Default for ModuleImplementationConfig {
    fn default() -> Self {
        Self {
            module_name: String::new(),
            target_system: None,
            go_live_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            rollout_months: 4,
            implementation_error_rate: 0.04,
            training_completion_rate: 0.85,
            affected_processes: Vec::new(),
            configuration_changes: Vec::new(),
        }
    }
}

/// Configuration change for module implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChange {
    /// Configuration item.
    pub item: String,
    /// Old value.
    #[serde(default)]
    pub old_value: Option<String>,
    /// New value.
    pub new_value: String,
}

/// Configuration for integration upgrade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationUpgradeConfig {
    /// Integration being upgraded.
    pub integration_name: String,
    /// Source system.
    #[serde(default)]
    pub source_system: Option<String>,
    /// Target system.
    #[serde(default)]
    pub target_system: Option<String>,
    /// Upgrade date.
    pub upgrade_date: NaiveDate,
    /// Number of months for transition.
    #[serde(default = "default_integration_transition")]
    pub transition_months: u32,
    /// Error rate during transition.
    #[serde(default = "default_integration_error_rate")]
    pub transition_error_rate: f64,
    /// Message format changes.
    #[serde(default)]
    pub format_changes: Vec<FormatChange>,
    /// New fields added.
    #[serde(default)]
    pub new_fields: Vec<String>,
    /// Fields deprecated.
    #[serde(default)]
    pub deprecated_fields: Vec<String>,
}

fn default_integration_transition() -> u32 {
    2
}

fn default_integration_error_rate() -> f64 {
    0.03
}

impl Default for IntegrationUpgradeConfig {
    fn default() -> Self {
        Self {
            integration_name: String::new(),
            source_system: None,
            target_system: None,
            upgrade_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            transition_months: 2,
            transition_error_rate: 0.03,
            format_changes: Vec::new(),
            new_fields: Vec::new(),
            deprecated_fields: Vec::new(),
        }
    }
}

/// Format change in integration upgrade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatChange {
    /// Field affected.
    pub field: String,
    /// Old format.
    pub old_format: String,
    /// New format.
    pub new_format: String,
}

/// A scheduled technology transition event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyTransitionEvent {
    /// Unique event ID.
    pub event_id: String,
    /// Event type and configuration.
    pub event_type: TechnologyTransitionType,
    /// Effective date of the event.
    pub effective_date: NaiveDate,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl TechnologyTransitionEvent {
    /// Create a new technology transition event.
    pub fn new(event_id: impl Into<String>, event_type: TechnologyTransitionType) -> Self {
        let effective_date = match &event_type {
            TechnologyTransitionType::ErpMigration(c) => c.phases.cutover_date,
            TechnologyTransitionType::ModuleImplementation(c) => c.go_live_date,
            TechnologyTransitionType::IntegrationUpgrade(c) => c.upgrade_date,
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
        match &self.event_type {
            TechnologyTransitionType::ErpMigration(c) => {
                let start = c.phases.preparation_start.unwrap_or(c.phases.cutover_date);
                let end = c.phases.hypercare_end.unwrap_or(c.phases.stabilization_end);
                date >= start && date <= end
            }
            TechnologyTransitionType::ModuleImplementation(c) => {
                let end = c.go_live_date + chrono::Duration::days(c.rollout_months as i64 * 30);
                date >= c.go_live_date && date <= end
            }
            TechnologyTransitionType::IntegrationUpgrade(c) => {
                let end = c.upgrade_date + chrono::Duration::days(c.transition_months as i64 * 30);
                date >= c.upgrade_date && date <= end
            }
        }
    }

    /// Get the progress through the event (0.0 to 1.0).
    pub fn progress_at(&self, date: NaiveDate) -> f64 {
        let (start, total_days) = match &self.event_type {
            TechnologyTransitionType::ErpMigration(c) => {
                let start = c.phases.preparation_start.unwrap_or(c.phases.cutover_date);
                let end = c.phases.hypercare_end.unwrap_or(c.phases.stabilization_end);
                (start, (end - start).num_days() as f64)
            }
            TechnologyTransitionType::ModuleImplementation(c) => {
                (c.go_live_date, c.rollout_months as f64 * 30.0)
            }
            TechnologyTransitionType::IntegrationUpgrade(c) => {
                (c.upgrade_date, c.transition_months as f64 * 30.0)
            }
        };

        if date < start {
            return 0.0;
        }
        if total_days <= 0.0 {
            return 1.0;
        }

        let days_elapsed = (date - start).num_days() as f64;
        (days_elapsed / total_days).min(1.0)
    }

    /// Get the current migration phase (for ERP migrations).
    pub fn migration_phase_at(&self, date: NaiveDate) -> Option<MigrationPhase> {
        match &self.event_type {
            TechnologyTransitionType::ErpMigration(c) => Some(c.phases.phase_at(date)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_phases() {
        let phases = MigrationPhases {
            preparation_start: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            data_migration_start: Some(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
            parallel_run_start: Some(NaiveDate::from_ymd_opt(2024, 8, 1).unwrap()),
            cutover_date: NaiveDate::from_ymd_opt(2024, 9, 1).unwrap(),
            stabilization_end: NaiveDate::from_ymd_opt(2024, 11, 30).unwrap(),
            hypercare_end: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        };

        assert_eq!(
            phases.phase_at(NaiveDate::from_ymd_opt(2023, 12, 1).unwrap()),
            MigrationPhase::PreMigration
        );
        assert_eq!(
            phases.phase_at(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
            MigrationPhase::Preparation
        );
        assert_eq!(
            phases.phase_at(NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()),
            MigrationPhase::DataMigration
        );
        assert_eq!(
            phases.phase_at(NaiveDate::from_ymd_opt(2024, 8, 15).unwrap()),
            MigrationPhase::ParallelRun
        );
        assert_eq!(
            phases.phase_at(NaiveDate::from_ymd_opt(2024, 10, 1).unwrap()),
            MigrationPhase::Stabilization
        );
        assert_eq!(
            phases.phase_at(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap()),
            MigrationPhase::Hypercare
        );
        assert_eq!(
            phases.phase_at(NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()),
            MigrationPhase::Complete
        );
    }

    #[test]
    fn test_migration_issue_combined_rate() {
        let issues = MigrationIssueConfig::default();
        let combined = issues.combined_error_rate();

        // Should be reasonable combined rate
        assert!(combined > 0.0);
        assert!(combined <= 0.20);
    }

    #[test]
    fn test_technology_transition_event() {
        let config = ErpMigrationConfig::default();
        let event = TechnologyTransitionEvent::new(
            "ERP-001",
            TechnologyTransitionType::ErpMigration(config.clone()),
        );

        // Before start
        assert!(!event.is_active_at(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()));

        // During migration
        assert!(event.is_active_at(NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()));

        // After completion
        assert!(!event.is_active_at(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()));
    }

    #[test]
    fn test_data_migration_strategy_risk() {
        assert_eq!(DataMigrationStrategy::BigBang.risk_level(), "high");
        assert_eq!(DataMigrationStrategy::Parallel.risk_level(), "low");
    }
}
