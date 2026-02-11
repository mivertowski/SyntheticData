//! Evidence models per ISA 500.
//!
//! Audit evidence is all information used by the auditor to arrive at
//! conclusions on which the auditor's opinion is based.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::workpaper::Assertion;

/// Audit evidence representing supporting documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvidence {
    /// Unique evidence ID
    pub evidence_id: Uuid,
    /// External reference
    pub evidence_ref: String,
    /// Engagement ID
    pub engagement_id: Uuid,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Source of evidence
    pub source_type: EvidenceSource,
    /// Evidence title
    pub title: String,
    /// Description
    pub description: String,

    // === Obtaining Information ===
    /// Date evidence was obtained
    pub obtained_date: NaiveDate,
    /// Who obtained the evidence
    pub obtained_by: String,
    /// File hash for integrity verification
    pub file_hash: Option<String>,
    /// File path or storage location
    pub file_path: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,

    // === Reliability Assessment per ISA 500.A31 ===
    /// Reliability assessment
    pub reliability_assessment: ReliabilityAssessment,

    // === Relevance ===
    /// Assertions addressed by this evidence
    pub assertions_addressed: Vec<Assertion>,
    /// Account IDs impacted
    pub accounts_impacted: Vec<String>,
    /// Process areas covered
    pub process_areas: Vec<String>,

    // === Cross-References ===
    /// Linked workpaper IDs
    pub linked_workpapers: Vec<Uuid>,
    /// Related evidence IDs
    pub related_evidence: Vec<Uuid>,

    // === AI Extraction (optional) ===
    /// AI-extracted key terms
    pub ai_extracted_terms: Option<HashMap<String, String>>,
    /// AI extraction confidence
    pub ai_confidence: Option<f64>,
    /// AI summary
    pub ai_summary: Option<String>,

    // === Metadata ===
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AuditEvidence {
    /// Create new audit evidence.
    pub fn new(
        engagement_id: Uuid,
        evidence_type: EvidenceType,
        source_type: EvidenceSource,
        title: &str,
    ) -> Self {
        let now = Utc::now();
        Self {
            evidence_id: Uuid::new_v4(),
            evidence_ref: format!("EV-{}", Uuid::new_v4().simple()),
            engagement_id,
            evidence_type,
            source_type,
            title: title.into(),
            description: String::new(),
            obtained_date: now.date_naive(),
            obtained_by: String::new(),
            file_hash: None,
            file_path: None,
            file_size: None,
            reliability_assessment: ReliabilityAssessment::default(),
            assertions_addressed: Vec::new(),
            accounts_impacted: Vec::new(),
            process_areas: Vec::new(),
            linked_workpapers: Vec::new(),
            related_evidence: Vec::new(),
            ai_extracted_terms: None,
            ai_confidence: None,
            ai_summary: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.into();
        self
    }

    /// Set who obtained the evidence.
    pub fn with_obtained_by(mut self, obtained_by: &str, date: NaiveDate) -> Self {
        self.obtained_by = obtained_by.into();
        self.obtained_date = date;
        self
    }

    /// Set file information.
    pub fn with_file_info(mut self, path: &str, hash: &str, size: u64) -> Self {
        self.file_path = Some(path.into());
        self.file_hash = Some(hash.into());
        self.file_size = Some(size);
        self
    }

    /// Set reliability assessment.
    pub fn with_reliability(mut self, assessment: ReliabilityAssessment) -> Self {
        self.reliability_assessment = assessment;
        self
    }

    /// Add assertions addressed.
    pub fn with_assertions(mut self, assertions: Vec<Assertion>) -> Self {
        self.assertions_addressed = assertions;
        self
    }

    /// Add AI extraction results.
    pub fn with_ai_extraction(
        mut self,
        terms: HashMap<String, String>,
        confidence: f64,
        summary: &str,
    ) -> Self {
        self.ai_extracted_terms = Some(terms);
        self.ai_confidence = Some(confidence);
        self.ai_summary = Some(summary.into());
        self
    }

    /// Link to a workpaper.
    pub fn link_workpaper(&mut self, workpaper_id: Uuid) {
        if !self.linked_workpapers.contains(&workpaper_id) {
            self.linked_workpapers.push(workpaper_id);
            self.updated_at = Utc::now();
        }
    }

    /// Get the overall reliability level.
    pub fn overall_reliability(&self) -> ReliabilityLevel {
        self.reliability_assessment.overall_reliability
    }

    /// Check if this is high-quality evidence.
    pub fn is_high_quality(&self) -> bool {
        matches!(
            self.reliability_assessment.overall_reliability,
            ReliabilityLevel::High
        ) && matches!(
            self.source_type,
            EvidenceSource::ExternalThirdParty | EvidenceSource::AuditorPrepared
        )
    }
}

/// Type of audit evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Confirmation from third party
    Confirmation,
    /// Client-prepared document
    #[default]
    Document,
    /// Auditor-prepared analysis
    Analysis,
    /// Screenshot or system extract
    SystemExtract,
    /// Contract or agreement
    Contract,
    /// Bank statement
    BankStatement,
    /// Invoice
    Invoice,
    /// Email correspondence
    Email,
    /// Meeting minutes
    MeetingMinutes,
    /// Management representation
    ManagementRepresentation,
    /// Legal letter
    LegalLetter,
    /// Specialist report
    SpecialistReport,
    /// Physical inventory observation
    PhysicalObservation,
    /// Recalculation spreadsheet
    Recalculation,
}

/// Source of audit evidence per ISA 500.A31.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    /// External source, directly from third party
    ExternalThirdParty,
    /// External source, provided by client
    #[default]
    ExternalClientProvided,
    /// Internal source, client prepared
    InternalClientPrepared,
    /// Auditor prepared
    AuditorPrepared,
}

impl EvidenceSource {
    /// Get the inherent reliability of this source type.
    pub fn inherent_reliability(&self) -> ReliabilityLevel {
        match self {
            Self::ExternalThirdParty => ReliabilityLevel::High,
            Self::AuditorPrepared => ReliabilityLevel::High,
            Self::ExternalClientProvided => ReliabilityLevel::Medium,
            Self::InternalClientPrepared => ReliabilityLevel::Low,
        }
    }

    /// Get a description for ISA documentation.
    pub fn description(&self) -> &'static str {
        match self {
            Self::ExternalThirdParty => "Obtained directly from independent external source",
            Self::AuditorPrepared => "Prepared by the auditor",
            Self::ExternalClientProvided => "External evidence provided by client",
            Self::InternalClientPrepared => "Prepared internally by client personnel",
        }
    }
}

/// Reliability assessment per ISA 500.A31.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReliabilityAssessment {
    /// Independence of source
    pub independence_of_source: ReliabilityLevel,
    /// Effectiveness of related controls
    pub effectiveness_of_controls: ReliabilityLevel,
    /// Qualifications of information provider
    pub qualifications_of_provider: ReliabilityLevel,
    /// Objectivity of information provider
    pub objectivity_of_provider: ReliabilityLevel,
    /// Overall reliability conclusion
    pub overall_reliability: ReliabilityLevel,
    /// Assessment notes
    pub notes: String,
}

impl ReliabilityAssessment {
    /// Create a new reliability assessment.
    pub fn new(
        independence: ReliabilityLevel,
        controls: ReliabilityLevel,
        qualifications: ReliabilityLevel,
        objectivity: ReliabilityLevel,
        notes: &str,
    ) -> Self {
        let overall = Self::calculate_overall(independence, controls, qualifications, objectivity);
        Self {
            independence_of_source: independence,
            effectiveness_of_controls: controls,
            qualifications_of_provider: qualifications,
            objectivity_of_provider: objectivity,
            overall_reliability: overall,
            notes: notes.into(),
        }
    }

    /// Calculate overall reliability from components.
    fn calculate_overall(
        independence: ReliabilityLevel,
        controls: ReliabilityLevel,
        qualifications: ReliabilityLevel,
        objectivity: ReliabilityLevel,
    ) -> ReliabilityLevel {
        let scores = [
            independence.score(),
            controls.score(),
            qualifications.score(),
            objectivity.score(),
        ];
        let avg = scores.iter().sum::<u8>() / 4;
        ReliabilityLevel::from_score(avg)
    }

    /// Create a high reliability assessment.
    pub fn high(notes: &str) -> Self {
        Self::new(
            ReliabilityLevel::High,
            ReliabilityLevel::High,
            ReliabilityLevel::High,
            ReliabilityLevel::High,
            notes,
        )
    }

    /// Create a medium reliability assessment.
    pub fn medium(notes: &str) -> Self {
        Self::new(
            ReliabilityLevel::Medium,
            ReliabilityLevel::Medium,
            ReliabilityLevel::Medium,
            ReliabilityLevel::Medium,
            notes,
        )
    }

    /// Create a low reliability assessment.
    pub fn low(notes: &str) -> Self {
        Self::new(
            ReliabilityLevel::Low,
            ReliabilityLevel::Low,
            ReliabilityLevel::Low,
            ReliabilityLevel::Low,
            notes,
        )
    }
}

/// Reliability level for evidence assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReliabilityLevel {
    /// High reliability
    High,
    /// Medium reliability
    #[default]
    Medium,
    /// Low reliability
    Low,
}

impl ReliabilityLevel {
    /// Get numeric score.
    pub fn score(&self) -> u8 {
        match self {
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        }
    }

    /// Create from score.
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=1 => Self::Low,
            2 => Self::Medium,
            _ => Self::High,
        }
    }
}

/// Evidence sufficiency evaluation per ISA 500.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSufficiency {
    /// Assertion being evaluated
    pub assertion: Assertion,
    /// Account or area
    pub account_or_area: String,
    /// Evidence pieces collected
    pub evidence_count: u32,
    /// Total reliability score
    pub total_reliability_score: f64,
    /// Risk level being addressed
    pub risk_level: super::engagement::RiskLevel,
    /// Is evidence sufficient?
    pub is_sufficient: bool,
    /// Sufficiency conclusion notes
    pub conclusion_notes: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_creation() {
        let evidence = AuditEvidence::new(
            Uuid::new_v4(),
            EvidenceType::Confirmation,
            EvidenceSource::ExternalThirdParty,
            "Bank Confirmation",
        );

        assert_eq!(evidence.evidence_type, EvidenceType::Confirmation);
        assert_eq!(evidence.source_type, EvidenceSource::ExternalThirdParty);
    }

    #[test]
    fn test_reliability_assessment() {
        let assessment = ReliabilityAssessment::new(
            ReliabilityLevel::High,
            ReliabilityLevel::Medium,
            ReliabilityLevel::High,
            ReliabilityLevel::Medium,
            "External confirmation with good controls",
        );

        assert_eq!(assessment.overall_reliability, ReliabilityLevel::Medium);
    }

    #[test]
    fn test_source_reliability() {
        assert_eq!(
            EvidenceSource::ExternalThirdParty.inherent_reliability(),
            ReliabilityLevel::High
        );
        assert_eq!(
            EvidenceSource::InternalClientPrepared.inherent_reliability(),
            ReliabilityLevel::Low
        );
    }

    #[test]
    fn test_evidence_quality() {
        let evidence = AuditEvidence::new(
            Uuid::new_v4(),
            EvidenceType::Confirmation,
            EvidenceSource::ExternalThirdParty,
            "Bank Confirmation",
        )
        .with_reliability(ReliabilityAssessment::high("Direct confirmation"));

        assert!(evidence.is_high_quality());
    }
}
