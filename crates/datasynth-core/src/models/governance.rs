//! Governance models for board minutes and corporate oversight documentation.
//!
//! These models capture the output of board and audit committee meetings,
//! supporting ISA 260 communication with those charged with governance.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Minutes of a board or audit committee meeting.
///
/// Auditors review board minutes (ISA 300, ISA 315) to understand the
/// governance environment, key decisions, and risk discussions that may
/// affect the audit approach.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardMinutes {
    /// Unique identifier for this meeting
    pub meeting_id: Uuid,
    /// Date of the meeting
    pub meeting_date: NaiveDate,
    /// Type of meeting: "regular", "special", or "audit_committee"
    pub meeting_type: String,
    /// Names or IDs of attendees
    #[serde(default)]
    pub attendees: Vec<String>,
    /// Key decisions made during the meeting
    #[serde(default)]
    pub key_decisions: Vec<String>,
    /// Risk topics discussed
    #[serde(default)]
    pub risk_discussions: Vec<String>,
    /// Matters specifically discussed by or relevant to the audit committee
    #[serde(default)]
    pub audit_committee_matters: Vec<String>,
}
