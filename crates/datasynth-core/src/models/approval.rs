//! Approval workflow models for journal entries.
//!
//! Provides multi-level approval chain logic based on amount thresholds,
//! with realistic timestamps and action history.

use crate::models::UserPersona;
use chrono::{DateTime, Datelike, Duration, NaiveTime, Timelike, Utc, Weekday};
use rand::Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of an approval workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    /// Entry is in draft state, not yet submitted
    #[default]
    Draft,
    /// Entry is pending approval
    Pending,
    /// Entry has been fully approved
    Approved,
    /// Entry was rejected
    Rejected,
    /// Entry was auto-approved (below threshold)
    AutoApproved,
    /// Entry requires revision before resubmission
    RequiresRevision,
}

impl ApprovalStatus {
    /// Check if this status represents a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Approved | Self::Rejected | Self::AutoApproved)
    }

    /// Check if approval is complete (approved or auto-approved).
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved | Self::AutoApproved)
    }
}

/// Type of approval action taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalActionType {
    /// Entry was submitted for approval
    Submit,
    /// Approver approved the entry
    Approve,
    /// Approver rejected the entry
    Reject,
    /// Approver requested revision
    RequestRevision,
    /// Preparer revised and resubmitted
    Resubmit,
    /// Entry was auto-approved by system
    AutoApprove,
    /// Entry was escalated to higher level
    Escalate,
}

/// Individual action in the approval workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalAction {
    /// User ID of the person taking action
    pub actor_id: String,

    /// Display name of the actor
    pub actor_name: String,

    /// Role/persona of the actor
    pub actor_role: UserPersona,

    /// Type of action taken
    pub action: ApprovalActionType,

    /// Timestamp of the action
    pub action_timestamp: DateTime<Utc>,

    /// Comments/notes from the actor
    pub comments: Option<String>,

    /// Approval level this action applies to
    pub approval_level: u8,
}

impl ApprovalAction {
    /// Create a new approval action.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        actor_id: String,
        actor_name: String,
        actor_role: UserPersona,
        action: ApprovalActionType,
        level: u8,
    ) -> Self {
        Self {
            actor_id,
            actor_name,
            actor_role,
            action,
            action_timestamp: Utc::now(),
            comments: None,
            approval_level: level,
        }
    }

    /// Add a comment to the action.
    pub fn with_comment(mut self, comment: &str) -> Self {
        self.comments = Some(comment.to_string());
        self
    }

    /// Set the timestamp.
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.action_timestamp = timestamp;
        self
    }
}

/// Complete approval workflow for a journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalWorkflow {
    /// Current status of the workflow
    pub status: ApprovalStatus,

    /// All actions taken in this workflow
    pub actions: Vec<ApprovalAction>,

    /// Number of approval levels required
    pub required_levels: u8,

    /// Current approval level achieved
    pub current_level: u8,

    /// User ID of the preparer
    pub preparer_id: String,

    /// Display name of the preparer
    pub preparer_name: String,

    /// When the entry was submitted for approval
    pub submitted_at: Option<DateTime<Utc>>,

    /// When the entry was finally approved
    pub approved_at: Option<DateTime<Utc>>,

    /// Transaction amount (for threshold calculation)
    pub amount: Decimal,
}

impl ApprovalWorkflow {
    /// Create a new draft workflow.
    pub fn new(preparer_id: String, preparer_name: String, amount: Decimal) -> Self {
        Self {
            status: ApprovalStatus::Draft,
            actions: Vec::new(),
            required_levels: 0,
            current_level: 0,
            preparer_id,
            preparer_name,
            submitted_at: None,
            approved_at: None,
            amount,
        }
    }

    /// Create an auto-approved workflow.
    pub fn auto_approved(
        preparer_id: String,
        preparer_name: String,
        amount: Decimal,
        timestamp: DateTime<Utc>,
    ) -> Self {
        let action = ApprovalAction {
            actor_id: "SYSTEM".to_string(),
            actor_name: "Automated System".to_string(),
            actor_role: UserPersona::AutomatedSystem,
            action: ApprovalActionType::AutoApprove,
            action_timestamp: timestamp,
            comments: Some("Amount below auto-approval threshold".to_string()),
            approval_level: 0,
        };

        Self {
            status: ApprovalStatus::AutoApproved,
            actions: vec![action],
            required_levels: 0,
            current_level: 0,
            preparer_id,
            preparer_name,
            submitted_at: Some(timestamp),
            approved_at: Some(timestamp),
            amount,
        }
    }

    /// Submit the workflow for approval.
    pub fn submit(&mut self, timestamp: DateTime<Utc>) {
        self.status = ApprovalStatus::Pending;
        self.submitted_at = Some(timestamp);

        let action = ApprovalAction {
            actor_id: self.preparer_id.clone(),
            actor_name: self.preparer_name.clone(),
            actor_role: UserPersona::JuniorAccountant, // Assumed
            action: ApprovalActionType::Submit,
            action_timestamp: timestamp,
            comments: None,
            approval_level: 0,
        };
        self.actions.push(action);
    }

    /// Add an approval action.
    pub fn approve(
        &mut self,
        approver_id: String,
        approver_name: String,
        approver_role: UserPersona,
        timestamp: DateTime<Utc>,
        comment: Option<String>,
    ) {
        self.current_level += 1;

        let mut action = ApprovalAction::new(
            approver_id,
            approver_name,
            approver_role,
            ApprovalActionType::Approve,
            self.current_level,
        )
        .with_timestamp(timestamp);

        if let Some(c) = comment {
            action = action.with_comment(&c);
        }

        self.actions.push(action);

        // Check if fully approved
        if self.current_level >= self.required_levels {
            self.status = ApprovalStatus::Approved;
            self.approved_at = Some(timestamp);
        }
    }

    /// Reject the workflow.
    pub fn reject(
        &mut self,
        rejector_id: String,
        rejector_name: String,
        rejector_role: UserPersona,
        timestamp: DateTime<Utc>,
        reason: &str,
    ) {
        self.status = ApprovalStatus::Rejected;

        let action = ApprovalAction::new(
            rejector_id,
            rejector_name,
            rejector_role,
            ApprovalActionType::Reject,
            self.current_level + 1,
        )
        .with_timestamp(timestamp)
        .with_comment(reason);

        self.actions.push(action);
    }

    /// Request revision.
    pub fn request_revision(
        &mut self,
        reviewer_id: String,
        reviewer_name: String,
        reviewer_role: UserPersona,
        timestamp: DateTime<Utc>,
        reason: &str,
    ) {
        self.status = ApprovalStatus::RequiresRevision;

        let action = ApprovalAction::new(
            reviewer_id,
            reviewer_name,
            reviewer_role,
            ApprovalActionType::RequestRevision,
            self.current_level + 1,
        )
        .with_timestamp(timestamp)
        .with_comment(reason);

        self.actions.push(action);
    }

    /// Check if workflow is complete.
    pub fn is_complete(&self) -> bool {
        self.status.is_terminal()
    }

    /// Get the final approver (if approved).
    pub fn final_approver(&self) -> Option<&ApprovalAction> {
        self.actions
            .iter()
            .rev()
            .find(|a| a.action == ApprovalActionType::Approve)
    }
}

/// Approval chain configuration with amount thresholds.
#[derive(Debug, Clone)]
pub struct ApprovalChain {
    /// Thresholds in ascending order
    pub thresholds: Vec<ApprovalThreshold>,
    /// Auto-approve threshold (below this amount, no approval needed)
    pub auto_approve_threshold: Decimal,
}

impl Default for ApprovalChain {
    fn default() -> Self {
        Self::standard()
    }
}

impl ApprovalChain {
    /// Create a standard approval chain.
    pub fn standard() -> Self {
        Self {
            auto_approve_threshold: Decimal::from(1000),
            thresholds: vec![
                ApprovalThreshold {
                    amount: Decimal::from(1000),
                    level: 1,
                    required_personas: vec![UserPersona::SeniorAccountant],
                },
                ApprovalThreshold {
                    amount: Decimal::from(10000),
                    level: 2,
                    required_personas: vec![UserPersona::SeniorAccountant, UserPersona::Controller],
                },
                ApprovalThreshold {
                    amount: Decimal::from(100000),
                    level: 3,
                    required_personas: vec![
                        UserPersona::SeniorAccountant,
                        UserPersona::Controller,
                        UserPersona::Manager,
                    ],
                },
                ApprovalThreshold {
                    amount: Decimal::from(500000),
                    level: 4,
                    required_personas: vec![
                        UserPersona::SeniorAccountant,
                        UserPersona::Controller,
                        UserPersona::Manager,
                        UserPersona::Executive,
                    ],
                },
            ],
        }
    }

    /// Determine the required approval level for an amount.
    pub fn required_level(&self, amount: Decimal) -> u8 {
        let abs_amount = amount.abs();

        if abs_amount < self.auto_approve_threshold {
            return 0;
        }

        for threshold in self.thresholds.iter().rev() {
            if abs_amount >= threshold.amount {
                return threshold.level;
            }
        }

        1 // Default to level 1 if above auto-approve but no threshold matched
    }

    /// Get the required personas for a given amount.
    pub fn required_personas(&self, amount: Decimal) -> Vec<UserPersona> {
        let level = self.required_level(amount);

        if level == 0 {
            return Vec::new();
        }

        self.thresholds
            .iter()
            .find(|t| t.level == level)
            .map(|t| t.required_personas.clone())
            .unwrap_or_default()
    }

    /// Check if an amount qualifies for auto-approval.
    pub fn is_auto_approve(&self, amount: Decimal) -> bool {
        amount.abs() < self.auto_approve_threshold
    }
}

/// Single threshold in the approval chain.
#[derive(Debug, Clone)]
pub struct ApprovalThreshold {
    /// Amount threshold
    pub amount: Decimal,
    /// Approval level required
    pub level: u8,
    /// Personas required to approve at this level
    pub required_personas: Vec<UserPersona>,
}

/// Generator for realistic approval workflows.
#[derive(Debug, Clone)]
pub struct ApprovalWorkflowGenerator {
    /// Approval chain configuration
    pub chain: ApprovalChain,
    /// Rejection rate (0.0 to 1.0)
    pub rejection_rate: f64,
    /// Revision request rate (0.0 to 1.0)
    pub revision_rate: f64,
    /// Average approval delay in hours
    pub average_delay_hours: f64,
}

impl Default for ApprovalWorkflowGenerator {
    fn default() -> Self {
        Self {
            chain: ApprovalChain::standard(),
            rejection_rate: 0.02,
            revision_rate: 0.05,
            average_delay_hours: 4.0,
        }
    }
}

impl ApprovalWorkflowGenerator {
    /// Generate a realistic approval timestamp during working hours.
    pub fn generate_approval_timestamp(
        &self,
        base_timestamp: DateTime<Utc>,
        rng: &mut impl Rng,
    ) -> DateTime<Utc> {
        // Add delay (exponential distribution around average)
        let delay_hours = self.average_delay_hours * (-rng.gen::<f64>().ln());
        let delay_hours = delay_hours.min(48.0); // Cap at 48 hours

        let mut result = base_timestamp + Duration::hours(delay_hours as i64);

        // Adjust to working hours (9 AM - 6 PM)
        let time = result.time();
        let hour = time.hour();

        if hour < 9 {
            // Before 9 AM, move to 9 AM
            result = result
                .date_naive()
                .and_time(NaiveTime::from_hms_opt(9, 0, 0).expect("valid time components"))
                .and_utc();
        } else if hour >= 18 {
            // After 6 PM, move to next day 9 AM
            result = (result.date_naive() + Duration::days(1))
                .and_time(
                    NaiveTime::from_hms_opt(9, rng.gen_range(0..59), 0)
                        .expect("valid time components"),
                )
                .and_utc();
        }

        // Skip weekends
        let weekday = result.weekday();
        if weekday == Weekday::Sat {
            result += Duration::days(2);
        } else if weekday == Weekday::Sun {
            result += Duration::days(1);
        }

        result
    }

    /// Determine the outcome of an approval action.
    pub fn determine_outcome(&self, rng: &mut impl Rng) -> ApprovalActionType {
        let roll: f64 = rng.gen();

        if roll < self.rejection_rate {
            ApprovalActionType::Reject
        } else if roll < self.rejection_rate + self.revision_rate {
            ApprovalActionType::RequestRevision
        } else {
            ApprovalActionType::Approve
        }
    }
}

/// Common rejection/revision reasons.
pub mod rejection_reasons {
    /// Reasons for rejection.
    pub const REJECTION_REASONS: &[&str] = &[
        "Missing supporting documentation",
        "Amount exceeds budget allocation",
        "Incorrect account coding",
        "Duplicate entry detected",
        "Policy violation",
        "Vendor not approved",
        "Missing purchase order reference",
        "Expense not business-related",
        "Incorrect cost center",
        "Authorization not obtained",
    ];

    /// Reasons for revision request.
    pub const REVISION_REASONS: &[&str] = &[
        "Please provide additional documentation",
        "Clarify business purpose",
        "Split between multiple cost centers",
        "Update account coding",
        "Add reference number",
        "Correct posting date",
        "Update description",
        "Verify amount",
        "Add tax information",
        "Update vendor information",
    ];
}

/// Individual approval record for tracking approval relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    /// Unique approval record ID
    pub approval_id: String,
    /// Document being approved
    pub document_number: String,
    /// Document type
    pub document_type: String,
    /// Company code
    pub company_code: String,
    /// Requester's user ID
    pub requester_id: String,
    /// Requester's name (optional)
    pub requester_name: Option<String>,
    /// Approver's user ID
    pub approver_id: String,
    /// Approver's name
    pub approver_name: String,
    /// Approval date
    pub approval_date: chrono::NaiveDate,
    /// Approval action taken
    pub action: String,
    /// Amount being approved
    pub amount: Decimal,
    /// Approver's approval limit (if any)
    pub approval_limit: Option<Decimal>,
    /// Comments/notes
    pub comments: Option<String>,
    /// If delegated, from whom
    pub delegation_from: Option<String>,
    /// Whether this was auto-approved
    pub is_auto_approved: bool,
}

impl ApprovalRecord {
    /// Creates a new approval record.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        document_number: String,
        approver_id: String,
        approver_name: String,
        requester_id: String,
        approval_date: chrono::NaiveDate,
        amount: Decimal,
        action: String,
        company_code: String,
    ) -> Self {
        Self {
            approval_id: uuid::Uuid::new_v4().to_string(),
            document_number,
            document_type: "JE".to_string(),
            company_code,
            requester_id,
            requester_name: None,
            approver_id,
            approver_name,
            approval_date,
            action,
            amount,
            approval_limit: None,
            comments: None,
            delegation_from: None,
            is_auto_approved: false,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_status() {
        assert!(ApprovalStatus::Approved.is_terminal());
        assert!(ApprovalStatus::Rejected.is_terminal());
        assert!(!ApprovalStatus::Pending.is_terminal());
        assert!(ApprovalStatus::Approved.is_approved());
        assert!(ApprovalStatus::AutoApproved.is_approved());
    }

    #[test]
    fn test_approval_chain_levels() {
        let chain = ApprovalChain::standard();

        // Below auto-approve threshold
        assert_eq!(chain.required_level(Decimal::from(500)), 0);

        // Level 1
        assert_eq!(chain.required_level(Decimal::from(5000)), 1);

        // Level 2
        assert_eq!(chain.required_level(Decimal::from(50000)), 2);

        // Level 3
        assert_eq!(chain.required_level(Decimal::from(200000)), 3);

        // Level 4
        assert_eq!(chain.required_level(Decimal::from(1000000)), 4);
    }

    #[test]
    fn test_workflow_lifecycle() {
        let mut workflow = ApprovalWorkflow::new(
            "JSMITH001".to_string(),
            "John Smith".to_string(),
            Decimal::from(5000),
        );

        workflow.required_levels = 1;
        workflow.submit(Utc::now());

        assert_eq!(workflow.status, ApprovalStatus::Pending);

        workflow.approve(
            "MBROWN001".to_string(),
            "Mary Brown".to_string(),
            UserPersona::SeniorAccountant,
            Utc::now(),
            None,
        );

        assert_eq!(workflow.status, ApprovalStatus::Approved);
        assert!(workflow.is_complete());
    }

    #[test]
    fn test_auto_approval() {
        let workflow = ApprovalWorkflow::auto_approved(
            "JSMITH001".to_string(),
            "John Smith".to_string(),
            Decimal::from(500),
            Utc::now(),
        );

        assert_eq!(workflow.status, ApprovalStatus::AutoApproved);
        assert!(workflow.is_complete());
        assert!(workflow.approved_at.is_some());
    }

    #[test]
    fn test_rejection() {
        let mut workflow = ApprovalWorkflow::new(
            "JSMITH001".to_string(),
            "John Smith".to_string(),
            Decimal::from(5000),
        );

        workflow.required_levels = 1;
        workflow.submit(Utc::now());

        workflow.reject(
            "MBROWN001".to_string(),
            "Mary Brown".to_string(),
            UserPersona::SeniorAccountant,
            Utc::now(),
            "Missing documentation",
        );

        assert_eq!(workflow.status, ApprovalStatus::Rejected);
        assert!(workflow.is_complete());
    }

    #[test]
    fn test_required_personas() {
        let chain = ApprovalChain::standard();

        let personas = chain.required_personas(Decimal::from(50000));
        assert!(personas.contains(&UserPersona::SeniorAccountant));
        assert!(personas.contains(&UserPersona::Controller));
    }
}
