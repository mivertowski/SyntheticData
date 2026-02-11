//! Operational disruption modeling.
//!
//! Models realistic operational disruptions that can be injected into generated data:
//! - System outages (missing data windows)
//! - Migration artifacts (format changes, dual-running periods)
//! - Process changes (workflow shifts, policy changes)
//! - Data recovery patterns (backfill, catch-up processing)

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of operational disruptions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisruptionType {
    /// System outage causing missing data
    SystemOutage(OutageConfig),
    /// System migration with format changes
    SystemMigration(MigrationConfig),
    /// Process or policy change
    ProcessChange(ProcessChangeConfig),
    /// Data recovery or backfill
    DataRecovery(RecoveryConfig),
    /// Regulatory compliance change
    RegulatoryChange(RegulatoryConfig),
}

/// Configuration for a system outage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutageConfig {
    /// Start of the outage
    pub start_date: NaiveDate,
    /// End of the outage
    pub end_date: NaiveDate,
    /// Affected systems/modules
    pub affected_systems: Vec<String>,
    /// Whether data was completely lost vs just delayed
    pub data_loss: bool,
    /// Recovery mode (if not complete loss)
    pub recovery_mode: Option<RecoveryMode>,
    /// Outage cause for labeling
    pub cause: OutageCause,
}

/// Cause of an outage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutageCause {
    /// Planned maintenance
    PlannedMaintenance,
    /// Unplanned system failure
    SystemFailure,
    /// Network connectivity issues
    NetworkOutage,
    /// Database issues
    DatabaseFailure,
    /// Third-party service unavailable
    VendorOutage,
    /// Security incident
    SecurityIncident,
    /// Natural disaster
    Disaster,
}

/// How data was recovered after an outage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveryMode {
    /// Transactions processed after recovery with original dates
    BackdatedRecovery,
    /// Transactions processed with recovery date
    CurrentDateRecovery,
    /// Mix of both approaches
    MixedRecovery,
    /// Manual journal entries to reconcile
    ManualReconciliation,
}

/// Configuration for a system migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationConfig {
    /// Migration go-live date
    pub go_live_date: NaiveDate,
    /// Dual-running period start (before go-live)
    pub dual_run_start: Option<NaiveDate>,
    /// Dual-running period end (after go-live)
    pub dual_run_end: Option<NaiveDate>,
    /// Source system name
    pub source_system: String,
    /// Target system name
    pub target_system: String,
    /// Format changes applied
    pub format_changes: Vec<FormatChange>,
    /// Account mapping changes
    pub account_remapping: HashMap<String, String>,
    /// Data quality issues during migration
    pub migration_issues: Vec<MigrationIssue>,
}

/// Types of format changes during migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FormatChange {
    /// Date format change (e.g., MM/DD/YYYY to YYYY-MM-DD)
    DateFormat {
        old_format: String,
        new_format: String,
    },
    /// Amount precision change
    AmountPrecision { old_decimals: u8, new_decimals: u8 },
    /// Currency code format
    CurrencyCode {
        old_format: String,
        new_format: String,
    },
    /// Account number format
    AccountFormat {
        old_pattern: String,
        new_pattern: String,
    },
    /// Reference number format
    ReferenceFormat {
        old_pattern: String,
        new_pattern: String,
    },
    /// Text encoding change
    TextEncoding {
        old_encoding: String,
        new_encoding: String,
    },
    /// Field length change
    FieldLength {
        field: String,
        old_length: usize,
        new_length: usize,
    },
}

/// Issues that can occur during migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MigrationIssue {
    /// Duplicate records created
    DuplicateRecords { affected_count: usize },
    /// Missing records not migrated
    MissingRecords { affected_count: usize },
    /// Truncated data
    TruncatedData {
        field: String,
        affected_count: usize,
    },
    /// Encoding corruption
    EncodingCorruption { affected_count: usize },
    /// Mismatched balances
    BalanceMismatch { variance: f64 },
    /// Orphaned references
    OrphanedReferences { affected_count: usize },
}

/// Configuration for process changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessChangeConfig {
    /// Effective date of the change
    pub effective_date: NaiveDate,
    /// Type of process change
    pub change_type: ProcessChangeType,
    /// Transition period length in days
    pub transition_days: u32,
    /// Whether retroactive changes were applied
    pub retroactive: bool,
}

/// Types of process changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessChangeType {
    /// Approval threshold change
    ApprovalThreshold {
        old_threshold: f64,
        new_threshold: f64,
    },
    /// New approval level added
    NewApprovalLevel { level_name: String, threshold: f64 },
    /// Approval level removed
    RemovedApprovalLevel { level_name: String },
    /// Segregation of duties change
    SodPolicyChange {
        new_conflicts: Vec<(String, String)>,
    },
    /// Account posting rules change
    PostingRuleChange { affected_accounts: Vec<String> },
    /// Vendor management change
    VendorPolicyChange { policy_name: String },
    /// Period close procedure change
    CloseProcessChange {
        old_close_day: u8,
        new_close_day: u8,
    },
    /// Document retention change
    RetentionPolicyChange { old_years: u8, new_years: u8 },
}

/// Configuration for data recovery scenarios.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryConfig {
    /// When recovery started
    pub recovery_start: NaiveDate,
    /// When recovery completed
    pub recovery_end: NaiveDate,
    /// Period being recovered
    pub affected_period_start: NaiveDate,
    /// Period being recovered end
    pub affected_period_end: NaiveDate,
    /// Recovery approach
    pub recovery_type: RecoveryType,
    /// Quality of recovered data
    pub data_quality: RecoveredDataQuality,
}

/// Types of data recovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveryType {
    /// Full backup restoration
    BackupRestore,
    /// Reconstruction from source documents
    SourceReconstruction,
    /// Interface file reprocessing
    InterfaceReplay,
    /// Manual entry from paper records
    ManualReentry,
    /// Partial recovery with estimates
    PartialWithEstimates,
}

/// Quality level of recovered data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveredDataQuality {
    /// Complete and accurate
    Complete,
    /// Minor discrepancies
    MinorDiscrepancies,
    /// Estimated values used
    EstimatedValues,
    /// Significant gaps remain
    PartialRecovery,
}

/// Configuration for regulatory changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegulatoryConfig {
    /// Effective date
    pub effective_date: NaiveDate,
    /// Regulation name
    pub regulation_name: String,
    /// Type of regulatory change
    pub change_type: RegulatoryChangeType,
    /// Grace period in days
    pub grace_period_days: u32,
}

/// Types of regulatory changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegulatoryChangeType {
    /// New reporting requirement
    NewReporting { report_name: String },
    /// Changed chart of accounts structure
    CoaRestructure,
    /// New tax rules
    TaxChange { jurisdiction: String },
    /// Revenue recognition change
    RevenueRecognition,
    /// Lease accounting change
    LeaseAccounting,
    /// Data privacy requirement
    DataPrivacy { regulation: String },
}

/// A disruption event with timing and effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionEvent {
    /// Unique identifier
    pub event_id: String,
    /// Type of disruption
    pub disruption_type: DisruptionType,
    /// Detailed description
    pub description: String,
    /// Impact severity (1-5)
    pub severity: u8,
    /// Affected company codes
    pub affected_companies: Vec<String>,
    /// Labels for ML training
    pub labels: HashMap<String, String>,
}

/// Manages disruption scenarios for data generation.
pub struct DisruptionManager {
    /// Active disruption events
    events: Vec<DisruptionEvent>,
    /// Event counter for ID generation
    event_counter: u64,
}

impl DisruptionManager {
    /// Create a new disruption manager.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            event_counter: 0,
        }
    }

    /// Add a disruption event.
    pub fn add_event(
        &mut self,
        disruption_type: DisruptionType,
        description: &str,
        severity: u8,
        affected_companies: Vec<String>,
    ) -> String {
        self.event_counter += 1;
        let event_id = format!("DISRUPT-{:06}", self.event_counter);

        let labels = self.generate_labels(&disruption_type);

        let event = DisruptionEvent {
            event_id: event_id.clone(),
            disruption_type,
            description: description.to_string(),
            severity,
            affected_companies,
            labels,
        };

        self.events.push(event);
        event_id
    }

    /// Generate ML labels for a disruption type.
    fn generate_labels(&self, disruption_type: &DisruptionType) -> HashMap<String, String> {
        let mut labels = HashMap::new();

        match disruption_type {
            DisruptionType::SystemOutage(config) => {
                labels.insert("disruption_category".to_string(), "outage".to_string());
                labels.insert("cause".to_string(), format!("{:?}", config.cause));
                labels.insert("data_loss".to_string(), config.data_loss.to_string());
            }
            DisruptionType::SystemMigration(config) => {
                labels.insert("disruption_category".to_string(), "migration".to_string());
                labels.insert("source_system".to_string(), config.source_system.clone());
                labels.insert("target_system".to_string(), config.target_system.clone());
            }
            DisruptionType::ProcessChange(config) => {
                labels.insert(
                    "disruption_category".to_string(),
                    "process_change".to_string(),
                );
                labels.insert(
                    "change_type".to_string(),
                    format!("{:?}", config.change_type),
                );
                labels.insert("retroactive".to_string(), config.retroactive.to_string());
            }
            DisruptionType::DataRecovery(config) => {
                labels.insert("disruption_category".to_string(), "recovery".to_string());
                labels.insert(
                    "recovery_type".to_string(),
                    format!("{:?}", config.recovery_type),
                );
                labels.insert(
                    "data_quality".to_string(),
                    format!("{:?}", config.data_quality),
                );
            }
            DisruptionType::RegulatoryChange(config) => {
                labels.insert("disruption_category".to_string(), "regulatory".to_string());
                labels.insert("regulation".to_string(), config.regulation_name.clone());
                labels.insert(
                    "change_type".to_string(),
                    format!("{:?}", config.change_type),
                );
            }
        }

        labels
    }

    /// Check if a date falls within any outage period.
    pub fn is_in_outage(&self, date: NaiveDate, company_code: &str) -> Option<&DisruptionEvent> {
        self.events.iter().find(|event| {
            if !event.affected_companies.contains(&company_code.to_string())
                && !event.affected_companies.is_empty()
            {
                return false;
            }

            match &event.disruption_type {
                DisruptionType::SystemOutage(config) => {
                    date >= config.start_date && date <= config.end_date
                }
                _ => false,
            }
        })
    }

    /// Check if a date is in a migration dual-run period.
    pub fn is_in_dual_run(&self, date: NaiveDate, company_code: &str) -> Option<&DisruptionEvent> {
        self.events.iter().find(|event| {
            if !event.affected_companies.contains(&company_code.to_string())
                && !event.affected_companies.is_empty()
            {
                return false;
            }

            match &event.disruption_type {
                DisruptionType::SystemMigration(config) => {
                    let start = config.dual_run_start.unwrap_or(config.go_live_date);
                    let end = config.dual_run_end.unwrap_or(config.go_live_date);
                    date >= start && date <= end
                }
                _ => false,
            }
        })
    }

    /// Get format changes applicable to a date.
    pub fn get_format_changes(&self, date: NaiveDate, company_code: &str) -> Vec<&FormatChange> {
        let mut changes = Vec::new();

        for event in &self.events {
            if !event.affected_companies.contains(&company_code.to_string())
                && !event.affected_companies.is_empty()
            {
                continue;
            }

            if let DisruptionType::SystemMigration(config) = &event.disruption_type {
                if date >= config.go_live_date {
                    changes.extend(config.format_changes.iter());
                }
            }
        }

        changes
    }

    /// Get active process changes for a date.
    pub fn get_active_process_changes(
        &self,
        date: NaiveDate,
        company_code: &str,
    ) -> Vec<&ProcessChangeConfig> {
        self.events
            .iter()
            .filter(|event| {
                event.affected_companies.contains(&company_code.to_string())
                    || event.affected_companies.is_empty()
            })
            .filter_map(|event| match &event.disruption_type {
                DisruptionType::ProcessChange(config) if date >= config.effective_date => {
                    Some(config)
                }
                _ => None,
            })
            .collect()
    }

    /// Check if a date is in a recovery period.
    pub fn is_in_recovery(&self, date: NaiveDate, company_code: &str) -> Option<&DisruptionEvent> {
        self.events.iter().find(|event| {
            if !event.affected_companies.contains(&company_code.to_string())
                && !event.affected_companies.is_empty()
            {
                return false;
            }

            match &event.disruption_type {
                DisruptionType::DataRecovery(config) => {
                    date >= config.recovery_start && date <= config.recovery_end
                }
                _ => false,
            }
        })
    }

    /// Get all events.
    pub fn events(&self) -> &[DisruptionEvent] {
        &self.events
    }

    /// Get events affecting a specific company.
    pub fn events_for_company(&self, company_code: &str) -> Vec<&DisruptionEvent> {
        self.events
            .iter()
            .filter(|e| {
                e.affected_companies.contains(&company_code.to_string())
                    || e.affected_companies.is_empty()
            })
            .collect()
    }
}

impl Default for DisruptionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Effects that a disruption can have on generated data.
#[derive(Debug, Clone, Default)]
pub struct DisruptionEffect {
    /// Skip generating data for this date
    pub skip_generation: bool,
    /// Apply format transformation
    pub format_transform: Option<FormatChange>,
    /// Add recovery/backfill markers
    pub add_recovery_markers: bool,
    /// Duplicate to secondary system
    pub duplicate_to_system: Option<String>,
    /// Apply process rule changes
    pub process_changes: Vec<ProcessChangeType>,
    /// Labels to add to generated records
    pub labels: HashMap<String, String>,
}

/// Apply disruption effects to determine how data should be generated.
pub fn compute_disruption_effect(
    manager: &DisruptionManager,
    date: NaiveDate,
    company_code: &str,
) -> DisruptionEffect {
    let mut effect = DisruptionEffect::default();

    // Check for outage
    if let Some(outage_event) = manager.is_in_outage(date, company_code) {
        if let DisruptionType::SystemOutage(config) = &outage_event.disruption_type {
            if config.data_loss {
                effect.skip_generation = true;
            } else {
                effect.add_recovery_markers = true;
            }
            effect
                .labels
                .insert("outage_event".to_string(), outage_event.event_id.clone());
        }
    }

    // Check for dual-run
    if let Some(migration_event) = manager.is_in_dual_run(date, company_code) {
        if let DisruptionType::SystemMigration(config) = &migration_event.disruption_type {
            effect.duplicate_to_system = Some(config.target_system.clone());
            effect.labels.insert(
                "migration_event".to_string(),
                migration_event.event_id.clone(),
            );
        }
    }

    // Check for format changes
    let format_changes = manager.get_format_changes(date, company_code);
    if let Some(first_change) = format_changes.first() {
        effect.format_transform = Some((*first_change).clone());
    }

    // Check for process changes
    for process_change in manager.get_active_process_changes(date, company_code) {
        effect
            .process_changes
            .push(process_change.change_type.clone());
    }

    // Check for recovery period
    if let Some(recovery_event) = manager.is_in_recovery(date, company_code) {
        effect.add_recovery_markers = true;
        effect.labels.insert(
            "recovery_event".to_string(),
            recovery_event.event_id.clone(),
        );
    }

    effect
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_outage_detection() {
        let mut manager = DisruptionManager::new();

        let outage = OutageConfig {
            start_date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 3, 17).unwrap(),
            affected_systems: vec!["GL".to_string()],
            data_loss: false,
            recovery_mode: Some(RecoveryMode::BackdatedRecovery),
            cause: OutageCause::SystemFailure,
        };

        manager.add_event(
            DisruptionType::SystemOutage(outage),
            "GL system outage",
            3,
            vec!["1000".to_string()],
        );

        // During outage
        assert!(manager
            .is_in_outage(NaiveDate::from_ymd_opt(2024, 3, 16).unwrap(), "1000")
            .is_some());

        // Before outage
        assert!(manager
            .is_in_outage(NaiveDate::from_ymd_opt(2024, 3, 14).unwrap(), "1000")
            .is_none());

        // Different company
        assert!(manager
            .is_in_outage(NaiveDate::from_ymd_opt(2024, 3, 16).unwrap(), "2000")
            .is_none());
    }

    #[test]
    fn test_migration_dual_run() {
        let mut manager = DisruptionManager::new();

        let migration = MigrationConfig {
            go_live_date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            dual_run_start: Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            dual_run_end: Some(NaiveDate::from_ymd_opt(2024, 7, 15).unwrap()),
            source_system: "Legacy".to_string(),
            target_system: "S4HANA".to_string(),
            format_changes: vec![FormatChange::DateFormat {
                old_format: "MM/DD/YYYY".to_string(),
                new_format: "YYYY-MM-DD".to_string(),
            }],
            account_remapping: HashMap::new(),
            migration_issues: Vec::new(),
        };

        manager.add_event(
            DisruptionType::SystemMigration(migration),
            "S/4HANA migration",
            4,
            vec![], // All companies
        );

        // During dual-run
        assert!(manager
            .is_in_dual_run(NaiveDate::from_ymd_opt(2024, 6, 20).unwrap(), "1000")
            .is_some());

        // After dual-run
        assert!(manager
            .is_in_dual_run(NaiveDate::from_ymd_opt(2024, 7, 20).unwrap(), "1000")
            .is_none());
    }

    #[test]
    fn test_process_change() {
        let mut manager = DisruptionManager::new();

        let process_change = ProcessChangeConfig {
            effective_date: NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            change_type: ProcessChangeType::ApprovalThreshold {
                old_threshold: 10000.0,
                new_threshold: 5000.0,
            },
            transition_days: 30,
            retroactive: false,
        };

        manager.add_event(
            DisruptionType::ProcessChange(process_change),
            "Lower approval threshold",
            2,
            vec!["1000".to_string()],
        );

        // After change
        let changes = manager
            .get_active_process_changes(NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(), "1000");
        assert_eq!(changes.len(), 1);

        // Before change
        let changes = manager
            .get_active_process_changes(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), "1000");
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_compute_disruption_effect() {
        let mut manager = DisruptionManager::new();

        let outage = OutageConfig {
            start_date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 3, 17).unwrap(),
            affected_systems: vec!["GL".to_string()],
            data_loss: true,
            recovery_mode: None,
            cause: OutageCause::SystemFailure,
        };

        manager.add_event(
            DisruptionType::SystemOutage(outage),
            "GL system outage with data loss",
            5,
            vec!["1000".to_string()],
        );

        let effect = compute_disruption_effect(
            &manager,
            NaiveDate::from_ymd_opt(2024, 3, 16).unwrap(),
            "1000",
        );

        assert!(effect.skip_generation);
        assert!(effect.labels.contains_key("outage_event"));
    }
}
