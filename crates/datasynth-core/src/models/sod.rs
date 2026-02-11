//! Segregation of Duties (SoD) definitions and conflict detection.
//!
//! Implements SoD conflict types and rules commonly used in
//! SOX compliance and internal audit frameworks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::internal_control::RiskLevel;
use super::user::UserPersona;

/// Types of Segregation of Duties conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SodConflictType {
    /// Same person prepared and approved a transaction
    PreparerApprover,
    /// Same person requested and approved their own request
    RequesterApprover,
    /// Same person performed reconciliation and posted entries
    ReconcilerPoster,
    /// Same person maintains vendor master data and processes payments
    MasterDataMaintainer,
    /// Same person created and released a payment
    PaymentReleaser,
    /// Same person posted to sensitive accounts without independent review
    JournalEntryPoster,
    /// Same person has access to multiple conflicting functions
    SystemAccessConflict,
}

impl std::fmt::Display for SodConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreparerApprover => write!(f, "Preparer/Approver"),
            Self::RequesterApprover => write!(f, "Requester/Approver"),
            Self::ReconcilerPoster => write!(f, "Reconciler/Poster"),
            Self::MasterDataMaintainer => write!(f, "Master Data Maintainer"),
            Self::PaymentReleaser => write!(f, "Payment Releaser"),
            Self::JournalEntryPoster => write!(f, "Journal Entry Poster"),
            Self::SystemAccessConflict => write!(f, "System Access Conflict"),
        }
    }
}

/// Definition of a SoD conflict pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SodConflictPair {
    /// Type of conflict
    pub conflict_type: SodConflictType,
    /// First role in the conflict
    pub role_a: UserPersona,
    /// Second role in the conflict (can be same as role_a)
    pub role_b: UserPersona,
    /// Description of the conflict
    pub description: String,
    /// Severity of this conflict type
    pub severity: RiskLevel,
}

impl SodConflictPair {
    /// Create a new SoD conflict pair.
    pub fn new(
        conflict_type: SodConflictType,
        role_a: UserPersona,
        role_b: UserPersona,
        description: impl Into<String>,
        severity: RiskLevel,
    ) -> Self {
        Self {
            conflict_type,
            role_a,
            role_b,
            description: description.into(),
            severity,
        }
    }

    /// Get standard SoD conflict pairs.
    pub fn standard_conflicts() -> Vec<Self> {
        vec![
            Self::new(
                SodConflictType::PreparerApprover,
                UserPersona::JuniorAccountant,
                UserPersona::SeniorAccountant,
                "Same person prepared and approved journal entry",
                RiskLevel::High,
            ),
            Self::new(
                SodConflictType::PreparerApprover,
                UserPersona::SeniorAccountant,
                UserPersona::Controller,
                "Same person prepared and approved high-value transaction",
                RiskLevel::High,
            ),
            Self::new(
                SodConflictType::RequesterApprover,
                UserPersona::JuniorAccountant,
                UserPersona::Manager,
                "Same person requested and approved their own expense/requisition",
                RiskLevel::Critical,
            ),
            Self::new(
                SodConflictType::PaymentReleaser,
                UserPersona::SeniorAccountant,
                UserPersona::SeniorAccountant,
                "Same person created and released payment",
                RiskLevel::Critical,
            ),
            Self::new(
                SodConflictType::MasterDataMaintainer,
                UserPersona::SeniorAccountant,
                UserPersona::JuniorAccountant,
                "Same person maintains vendor master and processes payments",
                RiskLevel::High,
            ),
            Self::new(
                SodConflictType::ReconcilerPoster,
                UserPersona::JuniorAccountant,
                UserPersona::JuniorAccountant,
                "Same person performed account reconciliation and posted adjustments",
                RiskLevel::Medium,
            ),
            Self::new(
                SodConflictType::JournalEntryPoster,
                UserPersona::JuniorAccountant,
                UserPersona::JuniorAccountant,
                "Posted to sensitive GL accounts without independent review",
                RiskLevel::High,
            ),
        ]
    }
}

/// Record of a specific SoD violation on a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SodViolation {
    /// Type of conflict that occurred
    pub conflict_type: SodConflictType,
    /// User ID who caused the violation
    pub actor_id: String,
    /// Description of the conflicting action
    pub conflicting_action: String,
    /// When the violation occurred
    pub timestamp: DateTime<Utc>,
    /// Severity of this specific violation
    pub severity: RiskLevel,
}

impl SodViolation {
    /// Create a new SoD violation record.
    pub fn new(
        conflict_type: SodConflictType,
        actor_id: impl Into<String>,
        conflicting_action: impl Into<String>,
        severity: RiskLevel,
    ) -> Self {
        Self {
            conflict_type,
            actor_id: actor_id.into(),
            conflicting_action: conflicting_action.into(),
            timestamp: Utc::now(),
            severity,
        }
    }

    /// Create a violation with a specific timestamp.
    pub fn with_timestamp(
        conflict_type: SodConflictType,
        actor_id: impl Into<String>,
        conflicting_action: impl Into<String>,
        severity: RiskLevel,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            conflict_type,
            actor_id: actor_id.into(),
            conflicting_action: conflicting_action.into(),
            timestamp,
            severity,
        }
    }
}

/// SoD rule that defines what constitutes a conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SodRule {
    /// Rule identifier
    pub rule_id: String,
    /// Rule name
    pub name: String,
    /// Conflict type this rule detects
    pub conflict_type: SodConflictType,
    /// Description of the rule
    pub description: String,
    /// Whether this rule is active
    pub is_active: bool,
    /// Risk level if this rule is violated
    pub risk_level: RiskLevel,
}

impl SodRule {
    /// Create a new SoD rule.
    pub fn new(
        rule_id: impl Into<String>,
        name: impl Into<String>,
        conflict_type: SodConflictType,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            name: name.into(),
            conflict_type,
            description: String::new(),
            is_active: true,
            risk_level: RiskLevel::High,
        }
    }

    /// Builder method to set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder method to set risk level.
    pub fn with_risk_level(mut self, level: RiskLevel) -> Self {
        self.risk_level = level;
        self
    }

    /// Get standard SoD rules.
    pub fn standard_rules() -> Vec<Self> {
        vec![
            Self::new(
                "SOD001",
                "Preparer-Approver Conflict",
                SodConflictType::PreparerApprover,
            )
            .with_description("User cannot approve their own journal entries")
            .with_risk_level(RiskLevel::High),
            Self::new(
                "SOD002",
                "Payment Dual Control",
                SodConflictType::PaymentReleaser,
            )
            .with_description("User cannot both create and release the same payment")
            .with_risk_level(RiskLevel::Critical),
            Self::new(
                "SOD003",
                "Vendor Master-Payment Conflict",
                SodConflictType::MasterDataMaintainer,
            )
            .with_description("User cannot maintain vendor master data and process payments")
            .with_risk_level(RiskLevel::High),
            Self::new(
                "SOD004",
                "Requester-Approver Conflict",
                SodConflictType::RequesterApprover,
            )
            .with_description("User cannot approve their own requisitions or expenses")
            .with_risk_level(RiskLevel::Critical),
            Self::new(
                "SOD005",
                "Reconciler-Poster Conflict",
                SodConflictType::ReconcilerPoster,
            )
            .with_description("User cannot both reconcile accounts and post adjusting entries")
            .with_risk_level(RiskLevel::Medium),
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sod_conflict_display() {
        assert_eq!(
            SodConflictType::PreparerApprover.to_string(),
            "Preparer/Approver"
        );
        assert_eq!(
            SodConflictType::PaymentReleaser.to_string(),
            "Payment Releaser"
        );
    }

    #[test]
    fn test_standard_conflicts() {
        let conflicts = SodConflictPair::standard_conflicts();
        assert!(!conflicts.is_empty());

        // Should have critical severity conflicts
        let critical: Vec<_> = conflicts
            .iter()
            .filter(|c| c.severity == RiskLevel::Critical)
            .collect();
        assert!(!critical.is_empty());
    }

    #[test]
    fn test_sod_violation_creation() {
        let violation = SodViolation::new(
            SodConflictType::PreparerApprover,
            "USER001",
            "Approved own journal entry",
            RiskLevel::High,
        );

        assert_eq!(violation.actor_id, "USER001");
        assert_eq!(violation.conflict_type, SodConflictType::PreparerApprover);
    }

    #[test]
    fn test_standard_rules() {
        let rules = SodRule::standard_rules();
        assert!(!rules.is_empty());

        // All standard rules should be active
        assert!(rules.iter().all(|r| r.is_active));
    }
}
