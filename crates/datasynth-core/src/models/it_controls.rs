//! IT control models for ITGC (IT General Controls) testing.
//!
//! These models support audit procedures related to IT access management
//! and change management, key areas assessed under ISA 315 and SOX 404.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// IT access log entry for ITGC testing.
///
/// Captures user authentication and authorization events across IT systems.
/// Auditors review access logs to assess logical access controls (ISA 315,
/// SOX 404 ITGC) including segregation of duties and privileged access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLog {
    /// Unique identifier for this log entry
    pub log_id: Uuid,
    /// Timestamp of the access event
    pub timestamp: NaiveDateTime,
    /// Employee identifier (references master data)
    pub user_id: String,
    /// Display name of the user
    pub user_name: String,
    /// IT system accessed (e.g. "SAP-FI", "Active Directory", "Oracle-HR")
    pub system: String,
    /// Action performed: "login", "logout", "failed_login", "privilege_change", "data_export"
    pub action: String,
    /// Whether the action succeeded
    pub success: bool,
    /// Source IP address (internal network 10.x.x.x)
    pub ip_address: String,
    /// Session duration in minutes (populated for logout events)
    pub session_duration_minutes: Option<u32>,
}

/// Change management record for ITGC testing.
///
/// Documents changes to IT systems including configuration changes, code
/// deployments, patches, and emergency fixes. Auditors assess change
/// management controls for proper authorization, testing, and rollback
/// planning (ISA 315, SOX 404 ITGC).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeManagementRecord {
    /// Unique identifier for this change record
    pub change_id: Uuid,
    /// IT system affected
    pub system: String,
    /// Type: "config_change", "code_deployment", "access_change", "patch", "emergency_fix"
    pub change_type: String,
    /// Description of the change
    pub description: String,
    /// Employee who requested the change
    pub requested_by: String,
    /// Employee who approved (None = unapproved, an ITGC finding)
    pub approved_by: Option<String>,
    /// Employee who implemented the change
    pub implemented_by: String,
    /// Date the change was requested
    pub request_date: NaiveDateTime,
    /// Date the change was implemented
    pub implementation_date: NaiveDateTime,
    /// Whether the change was tested before deployment
    pub tested: bool,
    /// Reference to test evidence documentation
    pub test_evidence: Option<String>,
    /// Whether a rollback plan was documented
    pub rollback_plan: bool,
}
