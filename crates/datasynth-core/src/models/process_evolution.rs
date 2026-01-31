//! Process evolution models for pattern drift simulation.
//!
//! Provides comprehensive process change modeling including:
//! - Approval workflow changes
//! - Process automation transitions
//! - Policy changes
//! - Control enhancements

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Process evolution event type with associated configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessEvolutionType {
    /// Change in approval workflow.
    ApprovalWorkflowChange(ApprovalWorkflowChangeConfig),
    /// Automation of a previously manual process.
    ProcessAutomation(ProcessAutomationConfig),
    /// Policy change affecting processes.
    PolicyChange(PolicyChangeConfig),
    /// Enhancement to existing controls.
    ControlEnhancement(ControlEnhancementConfig),
}

impl ProcessEvolutionType {
    /// Get the event type name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::ApprovalWorkflowChange(_) => "approval_workflow_change",
            Self::ProcessAutomation(_) => "process_automation",
            Self::PolicyChange(_) => "policy_change",
            Self::ControlEnhancement(_) => "control_enhancement",
        }
    }

    /// Get the processing time impact factor.
    pub fn processing_time_factor(&self) -> f64 {
        match self {
            Self::ApprovalWorkflowChange(c) => c.time_delta,
            Self::ProcessAutomation(c) => c.processing_time_reduction,
            Self::PolicyChange(_) => 1.0, // No direct impact
            Self::ControlEnhancement(c) => c.processing_time_impact,
        }
    }

    /// Get the error rate impact.
    pub fn error_rate_impact(&self) -> f64 {
        match self {
            Self::ApprovalWorkflowChange(c) => c.error_rate_impact,
            Self::ProcessAutomation(c) => c.error_rate_after - c.error_rate_before,
            Self::PolicyChange(c) => c.transition_error_rate,
            Self::ControlEnhancement(c) => -c.error_reduction, // Negative because it reduces errors
        }
    }

    /// Get the transition duration in months.
    pub fn transition_months(&self) -> u32 {
        match self {
            Self::ApprovalWorkflowChange(c) => c.transition_months,
            Self::ProcessAutomation(c) => c.rollout_months,
            Self::PolicyChange(c) => c.transition_months,
            Self::ControlEnhancement(c) => c.implementation_months,
        }
    }
}

/// Workflow type for approval processes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowType {
    /// Single approver workflow.
    #[default]
    SingleApprover,
    /// Dual approval (maker-checker).
    DualApproval,
    /// Multi-level approval chain.
    MultiLevel,
    /// Automated approval with rules.
    Automated,
    /// Matrix approval (multiple dimensions).
    Matrix,
    /// Parallel approval (multiple concurrent).
    Parallel,
}

impl WorkflowType {
    /// Get typical processing time multiplier relative to single approver.
    pub fn processing_time_multiplier(&self) -> f64 {
        match self {
            Self::SingleApprover => 1.0,
            Self::DualApproval => 1.5,
            Self::MultiLevel => 2.5,
            Self::Automated => 0.2,
            Self::Matrix => 2.0,
            Self::Parallel => 1.2,
        }
    }

    /// Get typical error detection rate improvement.
    pub fn error_detection_rate(&self) -> f64 {
        match self {
            Self::SingleApprover => 0.70,
            Self::DualApproval => 0.85,
            Self::MultiLevel => 0.90,
            Self::Automated => 0.95,
            Self::Matrix => 0.88,
            Self::Parallel => 0.82,
        }
    }
}

/// Configuration for approval workflow change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalWorkflowChangeConfig {
    /// Previous workflow type.
    pub from: WorkflowType,
    /// New workflow type.
    pub to: WorkflowType,
    /// Processing time change factor (e.g., 0.8 = 20% faster).
    #[serde(default = "default_time_delta")]
    pub time_delta: f64,
    /// Impact on error rate during transition.
    #[serde(default = "default_workflow_error_impact")]
    pub error_rate_impact: f64,
    /// Number of months for transition.
    #[serde(default = "default_workflow_transition")]
    pub transition_months: u32,
    /// Threshold changes associated with the workflow change.
    #[serde(default)]
    pub threshold_changes: Vec<ThresholdChange>,
}

fn default_time_delta() -> f64 {
    1.0
}

fn default_workflow_error_impact() -> f64 {
    0.02
}

fn default_workflow_transition() -> u32 {
    3
}

impl Default for ApprovalWorkflowChangeConfig {
    fn default() -> Self {
        Self {
            from: WorkflowType::SingleApprover,
            to: WorkflowType::DualApproval,
            time_delta: 1.5,
            error_rate_impact: 0.02,
            transition_months: 3,
            threshold_changes: Vec::new(),
        }
    }
}

impl ApprovalWorkflowChangeConfig {
    /// Calculate the time delta from workflow types if not explicitly set.
    pub fn calculated_time_delta(&self) -> f64 {
        self.to.processing_time_multiplier() / self.from.processing_time_multiplier()
    }
}

/// Threshold change details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdChange {
    /// Threshold category (e.g., "amount", "risk_level").
    pub category: String,
    /// Old threshold value.
    #[serde(with = "rust_decimal::serde::str")]
    pub old_threshold: Decimal,
    /// New threshold value.
    #[serde(with = "rust_decimal::serde::str")]
    pub new_threshold: Decimal,
}

/// Configuration for process automation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessAutomationConfig {
    /// Name of the process being automated.
    pub process_name: String,
    /// Manual processing rate before automation (0.0 to 1.0).
    #[serde(default = "default_manual_rate_before")]
    pub manual_rate_before: f64,
    /// Manual processing rate after automation (0.0 to 1.0).
    #[serde(default = "default_manual_rate_after")]
    pub manual_rate_after: f64,
    /// Error rate before automation.
    #[serde(default = "default_error_rate_before")]
    pub error_rate_before: f64,
    /// Error rate after automation.
    #[serde(default = "default_error_rate_after")]
    pub error_rate_after: f64,
    /// Processing time reduction factor (e.g., 0.3 = 70% faster).
    #[serde(default = "default_processing_reduction")]
    pub processing_time_reduction: f64,
    /// Number of months for rollout.
    #[serde(default = "default_rollout_months")]
    pub rollout_months: u32,
    /// Automation rollout curve type.
    #[serde(default)]
    pub rollout_curve: RolloutCurve,
    /// Affected transaction types.
    #[serde(default)]
    pub affected_transaction_types: Vec<String>,
}

fn default_manual_rate_before() -> f64 {
    0.80
}

fn default_manual_rate_after() -> f64 {
    0.15
}

fn default_error_rate_before() -> f64 {
    0.05
}

fn default_error_rate_after() -> f64 {
    0.01
}

fn default_processing_reduction() -> f64 {
    0.30
}

fn default_rollout_months() -> u32 {
    6
}

impl Default for ProcessAutomationConfig {
    fn default() -> Self {
        Self {
            process_name: "three_way_match".to_string(),
            manual_rate_before: 0.80,
            manual_rate_after: 0.15,
            error_rate_before: 0.05,
            error_rate_after: 0.01,
            processing_time_reduction: 0.30,
            rollout_months: 6,
            rollout_curve: RolloutCurve::SCurve,
            affected_transaction_types: Vec::new(),
        }
    }
}

impl ProcessAutomationConfig {
    /// Calculate automation rate at a given point in rollout (0.0 to 1.0 progress).
    pub fn automation_rate_at_progress(&self, progress: f64) -> f64 {
        let target_automation = 1.0 - self.manual_rate_after;
        let starting_automation = 1.0 - self.manual_rate_before;
        let range = target_automation - starting_automation;

        match self.rollout_curve {
            RolloutCurve::Linear => starting_automation + range * progress,
            RolloutCurve::SCurve => {
                // Logistic S-curve
                let steepness = 8.0;
                let midpoint = 0.5;
                let s_value = 1.0 / (1.0 + (-steepness * (progress - midpoint)).exp());
                starting_automation + range * s_value
            }
            RolloutCurve::Exponential => {
                // Exponential approach to target
                starting_automation + range * (1.0 - (-3.0 * progress).exp())
            }
            RolloutCurve::Step => {
                if progress >= 1.0 {
                    target_automation
                } else {
                    starting_automation
                }
            }
        }
    }

    /// Calculate error rate at a given point in rollout.
    pub fn error_rate_at_progress(&self, progress: f64) -> f64 {
        // Error rate typically follows the automation rate
        let automation_progress = self.automation_rate_at_progress(progress);
        let target_automation = 1.0 - self.manual_rate_after;
        let starting_automation = 1.0 - self.manual_rate_before;

        if (target_automation - starting_automation).abs() < 0.001 {
            return self.error_rate_before;
        }

        let automation_fraction =
            (automation_progress - starting_automation) / (target_automation - starting_automation);
        self.error_rate_before
            + (self.error_rate_after - self.error_rate_before) * automation_fraction
    }
}

/// Rollout curve type for automation adoption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RolloutCurve {
    /// Linear adoption.
    Linear,
    /// S-curve adoption (slow start, fast middle, slow end).
    #[default]
    SCurve,
    /// Exponential adoption (fast start, slowing).
    Exponential,
    /// Step function (immediate switch).
    Step,
}

/// Configuration for policy change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChangeConfig {
    /// Policy category affected.
    pub category: PolicyCategory,
    /// Description of the change.
    #[serde(default)]
    pub description: Option<String>,
    /// Old policy value (for threshold-based policies).
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub old_value: Option<Decimal>,
    /// New policy value (for threshold-based policies).
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub new_value: Option<Decimal>,
    /// Error rate during transition period.
    #[serde(default = "default_policy_transition_error")]
    pub transition_error_rate: f64,
    /// Number of months for transition.
    #[serde(default = "default_policy_transition")]
    pub transition_months: u32,
    /// Controls affected by this policy change.
    #[serde(default)]
    pub affected_controls: Vec<String>,
}

fn default_policy_transition_error() -> f64 {
    0.03
}

fn default_policy_transition() -> u32 {
    3
}

impl Default for PolicyChangeConfig {
    fn default() -> Self {
        Self {
            category: PolicyCategory::ApprovalThreshold,
            description: None,
            old_value: None,
            new_value: None,
            transition_error_rate: 0.03,
            transition_months: 3,
            affected_controls: Vec::new(),
        }
    }
}

/// Policy category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCategory {
    /// Approval thresholds.
    #[default]
    ApprovalThreshold,
    /// Expense policies.
    ExpensePolicy,
    /// Travel policies.
    TravelPolicy,
    /// Procurement policies.
    ProcurementPolicy,
    /// Credit policies.
    CreditPolicy,
    /// Inventory policies.
    InventoryPolicy,
    /// Documentation requirements.
    DocumentationRequirement,
    /// Other policy.
    Other,
}

impl PolicyCategory {
    /// Get the category code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ApprovalThreshold => "APPR",
            Self::ExpensePolicy => "EXPS",
            Self::TravelPolicy => "TRVL",
            Self::ProcurementPolicy => "PROC",
            Self::CreditPolicy => "CRED",
            Self::InventoryPolicy => "INVT",
            Self::DocumentationRequirement => "DOCS",
            Self::Other => "OTHR",
        }
    }
}

/// Configuration for control enhancement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlEnhancementConfig {
    /// Control ID being enhanced.
    pub control_id: String,
    /// Description of enhancement.
    #[serde(default)]
    pub description: Option<String>,
    /// Tolerance change.
    #[serde(default)]
    pub tolerance_change: Option<ToleranceChange>,
    /// Error reduction achieved (e.g., 0.02 = 2% fewer errors).
    #[serde(default = "default_error_reduction")]
    pub error_reduction: f64,
    /// Processing time impact (e.g., 1.1 = 10% slower due to more checks).
    #[serde(default = "default_processing_impact")]
    pub processing_time_impact: f64,
    /// Number of months for implementation.
    #[serde(default = "default_implementation_months")]
    pub implementation_months: u32,
    /// Additional evidence requirements.
    #[serde(default)]
    pub additional_evidence: Vec<String>,
}

fn default_error_reduction() -> f64 {
    0.02
}

fn default_processing_impact() -> f64 {
    1.05
}

fn default_implementation_months() -> u32 {
    2
}

impl Default for ControlEnhancementConfig {
    fn default() -> Self {
        Self {
            control_id: String::new(),
            description: None,
            tolerance_change: None,
            error_reduction: 0.02,
            processing_time_impact: 1.05,
            implementation_months: 2,
            additional_evidence: Vec::new(),
        }
    }
}

/// Tolerance change for controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToleranceChange {
    /// Old tolerance value.
    #[serde(with = "rust_decimal::serde::str")]
    pub old_tolerance: Decimal,
    /// New tolerance value.
    #[serde(with = "rust_decimal::serde::str")]
    pub new_tolerance: Decimal,
    /// Tolerance type.
    #[serde(default)]
    pub tolerance_type: ToleranceType,
}

/// Type of tolerance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToleranceType {
    /// Absolute amount tolerance.
    #[default]
    Absolute,
    /// Percentage tolerance.
    Percentage,
    /// Count tolerance.
    Count,
}

/// A scheduled process evolution event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvolutionEvent {
    /// Unique event ID.
    pub event_id: String,
    /// Event type and configuration.
    pub event_type: ProcessEvolutionType,
    /// Effective date of the event.
    pub effective_date: NaiveDate,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl ProcessEvolutionEvent {
    /// Create a new process evolution event.
    pub fn new(
        event_id: impl Into<String>,
        event_type: ProcessEvolutionType,
        effective_date: NaiveDate,
    ) -> Self {
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
mod tests {
    use super::*;

    #[test]
    fn test_workflow_type_multipliers() {
        assert!((WorkflowType::SingleApprover.processing_time_multiplier() - 1.0).abs() < 0.001);
        assert!(WorkflowType::DualApproval.processing_time_multiplier() > 1.0);
        assert!(WorkflowType::Automated.processing_time_multiplier() < 1.0);
    }

    #[test]
    fn test_process_automation_s_curve() {
        let config = ProcessAutomationConfig {
            manual_rate_before: 0.80,
            manual_rate_after: 0.20,
            ..Default::default()
        };

        let start = config.automation_rate_at_progress(0.0);
        let mid = config.automation_rate_at_progress(0.5);
        let end = config.automation_rate_at_progress(1.0);

        // S-curve should show slow start, fast middle, slow end
        assert!(start < mid);
        assert!(mid < end);
        assert!((end - 0.80).abs() < 0.02); // Should be near target automation (with small tolerance for S-curve)
    }

    #[test]
    fn test_process_evolution_event_progress() {
        let config = ProcessAutomationConfig {
            rollout_months: 6,
            ..Default::default()
        };

        let event = ProcessEvolutionEvent::new(
            "AUTO-001",
            ProcessEvolutionType::ProcessAutomation(config),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );

        // Before event
        assert!(!event.is_active_at(NaiveDate::from_ymd_opt(2023, 12, 1).unwrap()));

        // During event (3 months in = 50% progress)
        let during = NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        assert!(event.is_active_at(during));
        let progress = event.progress_at(during);
        assert!(progress > 0.4 && progress < 0.6);

        // After event
        let after = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
        assert!(!event.is_active_at(after));
        assert!((event.progress_at(after) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_approval_workflow_time_delta() {
        let config = ApprovalWorkflowChangeConfig {
            from: WorkflowType::SingleApprover,
            to: WorkflowType::Automated,
            ..Default::default()
        };

        let calculated = config.calculated_time_delta();
        assert!(calculated < 1.0); // Automated should be faster
    }
}
