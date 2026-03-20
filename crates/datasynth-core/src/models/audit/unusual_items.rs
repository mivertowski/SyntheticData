//! Unusual item markers for audit analytical review — ISA 520.
//!
//! Auditors performing analytical procedures (ISA 520) are required to identify
//! items that are unusual in nature, size, timing, or relationship.  This module
//! provides the data model for recording those flags so that they can be
//! investigated and documented in the audit file.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The dimension along which a journal entry was identified as unusual.
///
/// An entry may be flagged on multiple dimensions simultaneously.  The number
/// of dimensions determines the [`UnusualSeverity`] of the flag:
/// - 1 dimension  → [`UnusualSeverity::Minor`]
/// - 2 dimensions → [`UnusualSeverity::Moderate`]
/// - 3+ dimensions → [`UnusualSeverity::Significant`]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnusualDimension {
    /// Amount significantly exceeds the normal range for the account
    /// (typically > 3 standard deviations from the account mean).
    Size,
    /// Period-end clustering (posted in the last 3 days of the period),
    /// weekend posting, or other timing anomaly.
    Timing,
    /// Unexpected counterparty or GL account combination that occurs
    /// in fewer than 1% of all entries in the period.
    Relationship,
    /// First occurrence of this account being used by a particular poster,
    /// or an unusual repetition of the same amount.
    Frequency,
    /// Manual source entry to an account that is almost always posted
    /// automatically (system interface, batch job).
    Nature,
}

impl std::fmt::Display for UnusualDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Size => "Size",
            Self::Timing => "Timing",
            Self::Relationship => "Relationship",
            Self::Frequency => "Frequency",
            Self::Nature => "Nature",
        };
        write!(f, "{s}")
    }
}

/// Overall severity of an unusual item flag, determined by the number of
/// dimensions on which the entry was flagged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnusualSeverity {
    /// One unusual dimension — noteworthy but unlikely to require formal
    /// investigation on its own.
    Minor,
    /// Two unusual dimensions — warrants review and documentation.
    Moderate,
    /// Three or more unusual dimensions — requires formal investigation
    /// and audit-quality evidence.
    Significant,
}

impl UnusualSeverity {
    /// Derive severity from the number of triggered dimensions.
    pub fn from_dimension_count(n: usize) -> Self {
        match n {
            0 | 1 => Self::Minor,
            2 => Self::Moderate,
            _ => Self::Significant,
        }
    }
}

impl std::fmt::Display for UnusualSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Minor => "Minor",
            Self::Moderate => "Moderate",
            Self::Significant => "Significant",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Flag struct
// ---------------------------------------------------------------------------

/// An unusual item flag raised for a specific journal entry.
///
/// One flag is raised per journal entry that triggers at least one unusual
/// dimension.  Multiple dimensions can be present in a single flag.
///
/// The flag links back to the source journal entry via [`journal_entry_id`]
/// (which corresponds to `JournalEntryHeader::document_id` as a string).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusualItemFlag {
    /// Unique identifier for this flag.
    pub id: String,
    /// Entity / company code the journal entry belongs to.
    pub entity_code: String,
    /// The `document_id` (UUID string) of the flagged journal entry.
    pub journal_entry_id: String,
    /// The GL account(s) involved in the flagged condition.
    pub gl_accounts: Vec<String>,
    /// Dimensions on which the entry was flagged as unusual.
    pub dimensions: Vec<UnusualDimension>,
    /// Aggregate severity derived from the number of dimensions.
    pub severity: UnusualSeverity,
    /// Human-readable explanation of why the entry was flagged.
    pub description: String,
    /// What the model expected to observe (e.g. "amount within 3σ of mean").
    pub expected_value: Option<String>,
    /// What was actually observed (e.g. "amount = 1,250,000 vs mean 42,000").
    pub actual_value: String,
    /// Whether formal investigation steps are required.
    pub investigation_required: bool,
    /// Whether the underlying journal entry has an anomaly label
    /// (`JournalEntryHeader::is_anomaly`).
    pub is_labeled_anomaly: bool,
}
