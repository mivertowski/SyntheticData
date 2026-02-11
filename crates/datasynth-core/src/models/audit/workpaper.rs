//! Workpaper models per ISA 230.
//!
//! Workpapers document audit procedures performed, evidence obtained,
//! and conclusions reached.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::engagement::RiskLevel;

/// Audit workpaper representing documented audit work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workpaper {
    /// Unique workpaper ID
    pub workpaper_id: Uuid,
    /// Workpaper reference (e.g., "A-100", "B-200")
    pub workpaper_ref: String,
    /// Engagement ID this workpaper belongs to
    pub engagement_id: Uuid,
    /// Workpaper title
    pub title: String,
    /// Section/area of the audit
    pub section: WorkpaperSection,
    /// Audit objective addressed
    pub objective: String,
    /// Financial statement assertions tested
    pub assertions_tested: Vec<Assertion>,
    /// Procedure performed
    pub procedure_performed: String,
    /// Procedure type
    pub procedure_type: ProcedureType,

    // === Scope ===
    /// Testing scope
    pub scope: WorkpaperScope,
    /// Population size (total items)
    pub population_size: u64,
    /// Sample size (items tested)
    pub sample_size: u32,
    /// Sampling method used
    pub sampling_method: SamplingMethod,

    // === Results ===
    /// Summary of results
    pub results_summary: String,
    /// Number of exceptions found
    pub exceptions_found: u32,
    /// Exception rate
    pub exception_rate: f64,
    /// Conclusion reached
    pub conclusion: WorkpaperConclusion,
    /// Risk level addressed
    pub risk_level_addressed: RiskLevel,

    // === References ===
    /// Evidence reference IDs
    pub evidence_refs: Vec<Uuid>,
    /// Cross-references to other workpapers
    pub cross_references: Vec<String>,
    /// Related account IDs
    pub account_ids: Vec<String>,

    // === Sign-offs ===
    /// Preparer user ID
    pub preparer_id: String,
    /// Preparer name
    pub preparer_name: String,
    /// Date prepared
    pub preparer_date: NaiveDate,
    /// First reviewer ID
    pub reviewer_id: Option<String>,
    /// First reviewer name
    pub reviewer_name: Option<String>,
    /// First review date
    pub reviewer_date: Option<NaiveDate>,
    /// Second reviewer (manager) ID
    pub second_reviewer_id: Option<String>,
    /// Second reviewer name
    pub second_reviewer_name: Option<String>,
    /// Second review date
    pub second_reviewer_date: Option<NaiveDate>,

    // === Status ===
    /// Workpaper status
    pub status: WorkpaperStatus,
    /// Version number
    pub version: u32,
    /// Review notes
    pub review_notes: Vec<ReviewNote>,

    // === Timestamps ===
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workpaper {
    /// Create a new workpaper.
    pub fn new(
        engagement_id: Uuid,
        workpaper_ref: &str,
        title: &str,
        section: WorkpaperSection,
    ) -> Self {
        let now = Utc::now();
        Self {
            workpaper_id: Uuid::new_v4(),
            workpaper_ref: workpaper_ref.into(),
            engagement_id,
            title: title.into(),
            section,
            objective: String::new(),
            assertions_tested: Vec::new(),
            procedure_performed: String::new(),
            procedure_type: ProcedureType::InquiryObservation,
            scope: WorkpaperScope::default(),
            population_size: 0,
            sample_size: 0,
            sampling_method: SamplingMethod::Judgmental,
            results_summary: String::new(),
            exceptions_found: 0,
            exception_rate: 0.0,
            conclusion: WorkpaperConclusion::Satisfactory,
            risk_level_addressed: RiskLevel::Medium,
            evidence_refs: Vec::new(),
            cross_references: Vec::new(),
            account_ids: Vec::new(),
            preparer_id: String::new(),
            preparer_name: String::new(),
            preparer_date: now.date_naive(),
            reviewer_id: None,
            reviewer_name: None,
            reviewer_date: None,
            second_reviewer_id: None,
            second_reviewer_name: None,
            second_reviewer_date: None,
            status: WorkpaperStatus::Draft,
            version: 1,
            review_notes: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the objective and assertions.
    pub fn with_objective(mut self, objective: &str, assertions: Vec<Assertion>) -> Self {
        self.objective = objective.into();
        self.assertions_tested = assertions;
        self
    }

    /// Set the procedure.
    pub fn with_procedure(mut self, procedure: &str, procedure_type: ProcedureType) -> Self {
        self.procedure_performed = procedure.into();
        self.procedure_type = procedure_type;
        self
    }

    /// Set the scope and sampling.
    pub fn with_scope(
        mut self,
        scope: WorkpaperScope,
        population: u64,
        sample: u32,
        method: SamplingMethod,
    ) -> Self {
        self.scope = scope;
        self.population_size = population;
        self.sample_size = sample;
        self.sampling_method = method;
        self
    }

    /// Set the results.
    pub fn with_results(
        mut self,
        summary: &str,
        exceptions: u32,
        conclusion: WorkpaperConclusion,
    ) -> Self {
        self.results_summary = summary.into();
        self.exceptions_found = exceptions;
        self.exception_rate = if self.sample_size > 0 {
            exceptions as f64 / self.sample_size as f64
        } else {
            0.0
        };
        self.conclusion = conclusion;
        self
    }

    /// Set the preparer.
    pub fn with_preparer(mut self, id: &str, name: &str, date: NaiveDate) -> Self {
        self.preparer_id = id.into();
        self.preparer_name = name.into();
        self.preparer_date = date;
        self
    }

    /// Add first reviewer sign-off.
    pub fn add_first_review(&mut self, id: &str, name: &str, date: NaiveDate) {
        self.reviewer_id = Some(id.into());
        self.reviewer_name = Some(name.into());
        self.reviewer_date = Some(date);
        self.status = WorkpaperStatus::FirstReviewComplete;
        self.updated_at = Utc::now();
    }

    /// Add second reviewer sign-off.
    pub fn add_second_review(&mut self, id: &str, name: &str, date: NaiveDate) {
        self.second_reviewer_id = Some(id.into());
        self.second_reviewer_name = Some(name.into());
        self.second_reviewer_date = Some(date);
        self.status = WorkpaperStatus::Complete;
        self.updated_at = Utc::now();
    }

    /// Add a review note.
    pub fn add_review_note(&mut self, reviewer: &str, note: &str) {
        self.review_notes.push(ReviewNote {
            note_id: Uuid::new_v4(),
            reviewer_id: reviewer.into(),
            note: note.into(),
            status: ReviewNoteStatus::Open,
            created_at: Utc::now(),
            resolved_at: None,
        });
        self.updated_at = Utc::now();
    }

    /// Check if the workpaper is complete.
    pub fn is_complete(&self) -> bool {
        matches!(self.status, WorkpaperStatus::Complete)
    }

    /// Check if all review notes are resolved.
    pub fn all_notes_resolved(&self) -> bool {
        self.review_notes.iter().all(|n| {
            matches!(
                n.status,
                ReviewNoteStatus::Resolved | ReviewNoteStatus::NotApplicable
            )
        })
    }
}

/// Workpaper section/area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkpaperSection {
    /// Planning documentation
    #[default]
    Planning,
    /// Risk assessment procedures
    RiskAssessment,
    /// Internal control testing
    ControlTesting,
    /// Substantive testing
    SubstantiveTesting,
    /// Completion procedures
    Completion,
    /// Reporting
    Reporting,
    /// Permanent file
    PermanentFile,
}

impl WorkpaperSection {
    /// Get the typical workpaper reference prefix for this section.
    pub fn reference_prefix(&self) -> &'static str {
        match self {
            Self::Planning => "A",
            Self::RiskAssessment => "B",
            Self::ControlTesting => "C",
            Self::SubstantiveTesting => "D",
            Self::Completion => "E",
            Self::Reporting => "F",
            Self::PermanentFile => "P",
        }
    }
}

/// Financial statement assertions per ISA 315.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assertion {
    // Transaction assertions
    /// Transactions occurred and relate to the entity
    Occurrence,
    /// All transactions that should have been recorded have been recorded
    Completeness,
    /// Amounts and data relating to transactions have been recorded appropriately
    Accuracy,
    /// Transactions have been recorded in the correct accounting period
    Cutoff,
    /// Transactions have been recorded in the proper accounts
    Classification,

    // Balance assertions
    /// Assets, liabilities, and equity interests exist
    Existence,
    /// The entity holds rights to assets and liabilities are obligations
    RightsAndObligations,
    /// Assets, liabilities, and equity interests are included at appropriate amounts
    ValuationAndAllocation,

    // Presentation assertions
    /// Financial information is appropriately presented and described
    PresentationAndDisclosure,
}

impl Assertion {
    /// Get all transaction-level assertions.
    pub fn transaction_assertions() -> Vec<Self> {
        vec![
            Self::Occurrence,
            Self::Completeness,
            Self::Accuracy,
            Self::Cutoff,
            Self::Classification,
        ]
    }

    /// Get all balance-level assertions.
    pub fn balance_assertions() -> Vec<Self> {
        vec![
            Self::Existence,
            Self::Completeness,
            Self::RightsAndObligations,
            Self::ValuationAndAllocation,
        ]
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Occurrence => "Transactions and events have occurred and pertain to the entity",
            Self::Completeness => {
                "All transactions and events that should have been recorded have been recorded"
            }
            Self::Accuracy => "Amounts and other data have been recorded appropriately",
            Self::Cutoff => "Transactions and events have been recorded in the correct period",
            Self::Classification => {
                "Transactions and events have been recorded in the proper accounts"
            }
            Self::Existence => "Assets, liabilities, and equity interests exist",
            Self::RightsAndObligations => {
                "The entity holds rights to assets and liabilities are obligations of the entity"
            }
            Self::ValuationAndAllocation => {
                "Assets, liabilities, and equity interests are included at appropriate amounts"
            }
            Self::PresentationAndDisclosure => {
                "Financial information is appropriately presented and described"
            }
        }
    }
}

/// Type of audit procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProcedureType {
    /// Inquiry and observation
    #[default]
    InquiryObservation,
    /// Inspection of records
    Inspection,
    /// External confirmation
    Confirmation,
    /// Recalculation
    Recalculation,
    /// Reperformance
    Reperformance,
    /// Analytical procedures
    AnalyticalProcedures,
    /// Test of controls
    TestOfControls,
    /// Substantive test of details
    SubstantiveTest,
    /// Combined approach
    Combined,
}

/// Testing scope.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkpaperScope {
    /// Coverage percentage
    pub coverage_percentage: f64,
    /// Period covered start
    pub period_start: Option<NaiveDate>,
    /// Period covered end
    pub period_end: Option<NaiveDate>,
    /// Scope limitations
    pub limitations: Vec<String>,
}

/// Sampling method used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SamplingMethod {
    /// Statistical random sampling
    StatisticalRandom,
    /// Monetary unit sampling
    MonetaryUnit,
    /// Judgmental selection
    #[default]
    Judgmental,
    /// Haphazard selection
    Haphazard,
    /// Block selection
    Block,
    /// All items (100% testing)
    AllItems,
}

/// Workpaper conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkpaperConclusion {
    /// Satisfactory - no exceptions or immaterial exceptions
    #[default]
    Satisfactory,
    /// Satisfactory with exceptions noted
    SatisfactoryWithExceptions,
    /// Unsatisfactory - material exceptions
    Unsatisfactory,
    /// Unable to conclude - scope limitation
    UnableToConclude,
    /// Additional procedures required
    AdditionalProceduresRequired,
}

/// Workpaper status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkpaperStatus {
    /// Draft
    #[default]
    Draft,
    /// Pending review
    PendingReview,
    /// First review complete
    FirstReviewComplete,
    /// Pending second review
    PendingSecondReview,
    /// Complete
    Complete,
    /// Superseded
    Superseded,
}

/// Review note on a workpaper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewNote {
    /// Note ID
    pub note_id: Uuid,
    /// Reviewer who added the note
    pub reviewer_id: String,
    /// Note content
    pub note: String,
    /// Note status
    pub status: ReviewNoteStatus,
    /// When the note was created
    pub created_at: DateTime<Utc>,
    /// When the note was resolved
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Review note status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReviewNoteStatus {
    /// Open - needs action
    #[default]
    Open,
    /// In progress
    InProgress,
    /// Resolved
    Resolved,
    /// Not applicable
    NotApplicable,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_workpaper_creation() {
        let wp = Workpaper::new(
            Uuid::new_v4(),
            "C-100",
            "Revenue Recognition Testing",
            WorkpaperSection::SubstantiveTesting,
        );

        assert_eq!(wp.workpaper_ref, "C-100");
        assert_eq!(wp.section, WorkpaperSection::SubstantiveTesting);
        assert_eq!(wp.status, WorkpaperStatus::Draft);
    }

    #[test]
    fn test_workpaper_with_results() {
        let wp = Workpaper::new(
            Uuid::new_v4(),
            "D-100",
            "Accounts Receivable Confirmation",
            WorkpaperSection::SubstantiveTesting,
        )
        .with_scope(
            WorkpaperScope::default(),
            1000,
            50,
            SamplingMethod::StatisticalRandom,
        )
        .with_results(
            "Confirmed 50 balances with 2 exceptions",
            2,
            WorkpaperConclusion::SatisfactoryWithExceptions,
        );

        assert_eq!(wp.exception_rate, 0.04);
        assert_eq!(
            wp.conclusion,
            WorkpaperConclusion::SatisfactoryWithExceptions
        );
    }

    #[test]
    fn test_review_signoff() {
        let mut wp = Workpaper::new(
            Uuid::new_v4(),
            "A-100",
            "Planning Memo",
            WorkpaperSection::Planning,
        );

        wp.add_first_review(
            "reviewer1",
            "John Smith",
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        );
        assert_eq!(wp.status, WorkpaperStatus::FirstReviewComplete);

        wp.add_second_review(
            "manager1",
            "Jane Doe",
            NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(),
        );
        assert_eq!(wp.status, WorkpaperStatus::Complete);
        assert!(wp.is_complete());
    }

    #[test]
    fn test_assertions() {
        let txn_assertions = Assertion::transaction_assertions();
        assert_eq!(txn_assertions.len(), 5);
        assert!(txn_assertions.contains(&Assertion::Occurrence));

        let bal_assertions = Assertion::balance_assertions();
        assert_eq!(bal_assertions.len(), 4);
        assert!(bal_assertions.contains(&Assertion::Existence));
    }
}
