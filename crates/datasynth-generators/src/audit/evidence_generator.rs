//! Evidence generator for audit engagements.
//!
//! Generates audit evidence with appropriate reliability assessments,
//! source classifications, and cross-references per ISA 500.

use chrono::{Duration, NaiveDate};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use uuid::Uuid;

use datasynth_core::models::audit::{
    Assertion, AuditEngagement, AuditEvidence, EvidenceSource, EvidenceType, ReliabilityAssessment,
    ReliabilityLevel, Workpaper,
};

/// Configuration for evidence generation.
#[derive(Debug, Clone)]
pub struct EvidenceGeneratorConfig {
    /// Evidence pieces per workpaper (min, max)
    pub evidence_per_workpaper: (u32, u32),
    /// Probability of external third-party evidence
    pub external_third_party_probability: f64,
    /// Probability of high reliability evidence
    pub high_reliability_probability: f64,
    /// Probability of AI extraction
    pub ai_extraction_probability: f64,
    /// File size range in bytes (min, max)
    pub file_size_range: (u64, u64),
}

impl Default for EvidenceGeneratorConfig {
    fn default() -> Self {
        Self {
            evidence_per_workpaper: (1, 5),
            external_third_party_probability: 0.20,
            high_reliability_probability: 0.40,
            ai_extraction_probability: 0.15,
            file_size_range: (10_000, 5_000_000),
        }
    }
}

/// Generator for audit evidence.
pub struct EvidenceGenerator {
    rng: ChaCha8Rng,
    config: EvidenceGeneratorConfig,
    evidence_counter: u32,
}

impl EvidenceGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: EvidenceGeneratorConfig::default(),
            evidence_counter: 0,
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: EvidenceGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            evidence_counter: 0,
        }
    }

    /// Generate evidence for a workpaper.
    pub fn generate_evidence_for_workpaper(
        &mut self,
        workpaper: &Workpaper,
        team_members: &[String],
        base_date: NaiveDate,
    ) -> Vec<AuditEvidence> {
        let count = self
            .rng
            .gen_range(self.config.evidence_per_workpaper.0..=self.config.evidence_per_workpaper.1);

        (0..count)
            .map(|i| {
                self.generate_evidence(
                    workpaper.engagement_id,
                    Some(workpaper.workpaper_id),
                    &workpaper.assertions_tested,
                    team_members,
                    base_date + Duration::days(i as i64),
                )
            })
            .collect()
    }

    /// Generate a single piece of evidence.
    pub fn generate_evidence(
        &mut self,
        engagement_id: Uuid,
        workpaper_id: Option<Uuid>,
        assertions: &[Assertion],
        team_members: &[String],
        obtained_date: NaiveDate,
    ) -> AuditEvidence {
        self.evidence_counter += 1;

        // Determine evidence type and source
        let (evidence_type, source_type) = self.select_evidence_type_and_source();
        let title = self.generate_evidence_title(evidence_type);

        let mut evidence = AuditEvidence::new(engagement_id, evidence_type, source_type, &title);

        evidence.evidence_ref = format!("EV-{:06}", self.evidence_counter);

        // Set description
        let description = self.generate_evidence_description(evidence_type, source_type);
        evidence = evidence.with_description(&description);

        // Set obtained by
        let obtainer = self.select_team_member(team_members);
        evidence = evidence.with_obtained_by(&obtainer, obtained_date);

        // Set file info
        let file_size = self
            .rng
            .gen_range(self.config.file_size_range.0..=self.config.file_size_range.1);
        let file_path = self.generate_file_path(evidence_type, self.evidence_counter);
        let file_hash = format!("sha256:{:064x}", self.rng.gen::<u128>());
        evidence = evidence.with_file_info(&file_path, &file_hash, file_size);

        // Set reliability assessment
        let reliability = self.generate_reliability_assessment(source_type);
        evidence = evidence.with_reliability(reliability);

        // Set assertions
        if assertions.is_empty() {
            evidence = evidence.with_assertions(vec![self.random_assertion()]);
        } else {
            evidence = evidence.with_assertions(assertions.to_vec());
        }

        // Link to workpaper if provided
        if let Some(wp_id) = workpaper_id {
            evidence.link_workpaper(wp_id);
        }

        // Maybe add AI extraction
        if self.rng.gen::<f64>() < self.config.ai_extraction_probability {
            let terms = self.generate_ai_terms(evidence_type);
            let confidence = self.rng.gen_range(0.75..0.98);
            let summary = self.generate_ai_summary(evidence_type);
            evidence = evidence.with_ai_extraction(terms, confidence, &summary);
        }

        evidence
    }

    /// Generate evidence for an entire engagement.
    pub fn generate_evidence_for_engagement(
        &mut self,
        engagement: &AuditEngagement,
        workpapers: &[Workpaper],
        team_members: &[String],
    ) -> Vec<AuditEvidence> {
        let mut all_evidence = Vec::new();

        for workpaper in workpapers {
            let evidence = self.generate_evidence_for_workpaper(
                workpaper,
                team_members,
                workpaper.preparer_date,
            );
            all_evidence.extend(evidence);
        }

        // Add some standalone evidence not linked to specific workpapers
        let standalone_count = self.rng.gen_range(5..15);
        for i in 0..standalone_count {
            let date = engagement.fieldwork_start + Duration::days(i as i64 * 3);
            let evidence =
                self.generate_evidence(engagement.engagement_id, None, &[], team_members, date);
            all_evidence.push(evidence);
        }

        all_evidence
    }

    /// Select evidence type and source.
    fn select_evidence_type_and_source(&mut self) -> (EvidenceType, EvidenceSource) {
        let is_external = self.rng.gen::<f64>() < self.config.external_third_party_probability;

        if is_external {
            let external_types = [
                (
                    EvidenceType::Confirmation,
                    EvidenceSource::ExternalThirdParty,
                ),
                (
                    EvidenceType::BankStatement,
                    EvidenceSource::ExternalThirdParty,
                ),
                (
                    EvidenceType::LegalLetter,
                    EvidenceSource::ExternalThirdParty,
                ),
                (
                    EvidenceType::Contract,
                    EvidenceSource::ExternalClientProvided,
                ),
            ];
            let idx = self.rng.gen_range(0..external_types.len());
            external_types[idx]
        } else {
            let internal_types = [
                (
                    EvidenceType::Document,
                    EvidenceSource::InternalClientPrepared,
                ),
                (
                    EvidenceType::Invoice,
                    EvidenceSource::InternalClientPrepared,
                ),
                (
                    EvidenceType::SystemExtract,
                    EvidenceSource::InternalClientPrepared,
                ),
                (EvidenceType::Analysis, EvidenceSource::AuditorPrepared),
                (EvidenceType::Recalculation, EvidenceSource::AuditorPrepared),
                (
                    EvidenceType::MeetingMinutes,
                    EvidenceSource::InternalClientPrepared,
                ),
                (EvidenceType::Email, EvidenceSource::InternalClientPrepared),
            ];
            let idx = self.rng.gen_range(0..internal_types.len());
            internal_types[idx]
        }
    }

    /// Generate evidence title.
    fn generate_evidence_title(&mut self, evidence_type: EvidenceType) -> String {
        let titles = match evidence_type {
            EvidenceType::Confirmation => vec![
                "Bank Confirmation - Primary Account",
                "AR Confirmation - Major Customer",
                "AP Confirmation - Key Vendor",
                "Legal Confirmation",
                "Investment Confirmation",
            ],
            EvidenceType::BankStatement => vec![
                "Bank Statement - Operating Account",
                "Bank Statement - Payroll Account",
                "Bank Statement - Investment Account",
                "Bank Statement - Foreign Currency",
            ],
            EvidenceType::Invoice => vec![
                "Vendor Invoice Sample",
                "Customer Invoice Sample",
                "Intercompany Invoice",
                "Service Invoice",
            ],
            EvidenceType::Contract => vec![
                "Customer Contract",
                "Vendor Agreement",
                "Lease Agreement",
                "Employment Contract Sample",
                "Loan Agreement",
            ],
            EvidenceType::Document => vec![
                "Supporting Documentation",
                "Source Document",
                "Transaction Support",
                "Authorization Document",
            ],
            EvidenceType::Analysis => vec![
                "Analytical Review",
                "Variance Analysis",
                "Trend Analysis",
                "Ratio Analysis",
                "Account Reconciliation Review",
            ],
            EvidenceType::SystemExtract => vec![
                "ERP System Extract",
                "GL Detail Extract",
                "Transaction Log Extract",
                "User Access Report",
            ],
            EvidenceType::MeetingMinutes => vec![
                "Board Meeting Minutes",
                "Audit Committee Minutes",
                "Management Meeting Notes",
            ],
            EvidenceType::Email => vec![
                "Management Inquiry Response",
                "Confirmation Follow-up",
                "Exception Explanation",
            ],
            EvidenceType::Recalculation => vec![
                "Depreciation Recalculation",
                "Interest Recalculation",
                "Tax Provision Recalculation",
                "Allowance Recalculation",
            ],
            EvidenceType::LegalLetter => vec!["Attorney Response Letter", "Litigation Summary"],
            EvidenceType::ManagementRepresentation => vec![
                "Management Representation Letter",
                "Specific Representation",
            ],
            EvidenceType::SpecialistReport => vec![
                "Valuation Specialist Report",
                "Actuary Report",
                "IT Specialist Assessment",
            ],
            EvidenceType::PhysicalObservation => vec![
                "Inventory Count Observation",
                "Fixed Asset Inspection",
                "Physical Verification",
            ],
        };

        let idx = self.rng.gen_range(0..titles.len());
        titles[idx].to_string()
    }

    /// Generate evidence description.
    fn generate_evidence_description(
        &mut self,
        evidence_type: EvidenceType,
        source: EvidenceSource,
    ) -> String {
        let source_desc = source.description();
        match evidence_type {
            EvidenceType::Confirmation => {
                format!("External confirmation {}. Response received and agreed to client records.", source_desc)
            }
            EvidenceType::BankStatement => {
                format!("Bank statement {}. Statement obtained for period-end reconciliation.", source_desc)
            }
            EvidenceType::Invoice => {
                "Invoice selected as part of sample testing. Examined for appropriate approval, accuracy, and proper period recording.".into()
            }
            EvidenceType::Analysis => {
                "Auditor-prepared analytical procedure. Expectations developed based on prior year, industry data, and management budgets.".into()
            }
            EvidenceType::SystemExtract => {
                format!("System report {}. Extract validated for completeness and accuracy.", source_desc)
            }
            _ => format!("Supporting documentation {}.", source_desc),
        }
    }

    /// Generate reliability assessment.
    fn generate_reliability_assessment(&mut self, source: EvidenceSource) -> ReliabilityAssessment {
        let base_reliability = source.inherent_reliability();

        let independence = base_reliability;
        let controls = if self.rng.gen::<f64>() < self.config.high_reliability_probability {
            ReliabilityLevel::High
        } else {
            ReliabilityLevel::Medium
        };
        let qualifications = if self.rng.gen::<f64>() < 0.7 {
            ReliabilityLevel::High
        } else {
            ReliabilityLevel::Medium
        };
        let objectivity = match source {
            EvidenceSource::ExternalThirdParty | EvidenceSource::AuditorPrepared => {
                ReliabilityLevel::High
            }
            _ => {
                if self.rng.gen::<f64>() < 0.5 {
                    ReliabilityLevel::Medium
                } else {
                    ReliabilityLevel::Low
                }
            }
        };

        let notes = match base_reliability {
            ReliabilityLevel::High => {
                "Evidence obtained from independent source with high reliability"
            }
            ReliabilityLevel::Medium => "Evidence obtained from client with adequate controls",
            ReliabilityLevel::Low => "Internal evidence requires corroboration",
        };

        ReliabilityAssessment::new(independence, controls, qualifications, objectivity, notes)
    }

    /// Generate file path for evidence.
    fn generate_file_path(&mut self, evidence_type: EvidenceType, counter: u32) -> String {
        let extension = match evidence_type {
            EvidenceType::SystemExtract => "xlsx",
            EvidenceType::Analysis | EvidenceType::Recalculation => "xlsx",
            EvidenceType::MeetingMinutes | EvidenceType::ManagementRepresentation => "pdf",
            EvidenceType::Email => "msg",
            _ => {
                if self.rng.gen::<f64>() < 0.6 {
                    "pdf"
                } else {
                    "xlsx"
                }
            }
        };

        format!("/evidence/EV-{:06}.{}", counter, extension)
    }

    /// Select a random team member.
    fn select_team_member(&mut self, team_members: &[String]) -> String {
        if team_members.is_empty() {
            format!("STAFF{:03}", self.rng.gen_range(1..100))
        } else {
            let idx = self.rng.gen_range(0..team_members.len());
            team_members[idx].clone()
        }
    }

    /// Generate a random assertion.
    fn random_assertion(&mut self) -> Assertion {
        let assertions = [
            Assertion::Occurrence,
            Assertion::Completeness,
            Assertion::Accuracy,
            Assertion::Cutoff,
            Assertion::Classification,
            Assertion::Existence,
            Assertion::RightsAndObligations,
            Assertion::ValuationAndAllocation,
            Assertion::PresentationAndDisclosure,
        ];
        let idx = self.rng.gen_range(0..assertions.len());
        assertions[idx]
    }

    /// Generate AI-extracted terms.
    fn generate_ai_terms(
        &mut self,
        evidence_type: EvidenceType,
    ) -> std::collections::HashMap<String, String> {
        let mut terms = std::collections::HashMap::new();

        match evidence_type {
            EvidenceType::Invoice => {
                terms.insert(
                    "invoice_number".into(),
                    format!("INV-{:06}", self.rng.gen_range(100000..999999)),
                );
                terms.insert(
                    "amount".into(),
                    format!("{:.2}", self.rng.gen_range(1000.0..100000.0)),
                );
                terms.insert("vendor".into(), "Extracted Vendor Name".into());
            }
            EvidenceType::Contract => {
                terms.insert("effective_date".into(), "2025-01-01".into());
                terms.insert("term_years".into(), format!("{}", self.rng.gen_range(1..5)));
                terms.insert(
                    "total_value".into(),
                    format!("{:.2}", self.rng.gen_range(50000.0..500000.0)),
                );
            }
            EvidenceType::BankStatement => {
                terms.insert(
                    "ending_balance".into(),
                    format!("{:.2}", self.rng.gen_range(100000.0..10000000.0)),
                );
                terms.insert("statement_date".into(), "2025-12-31".into());
            }
            _ => {
                terms.insert("document_date".into(), "2025-12-31".into());
                terms.insert(
                    "reference".into(),
                    format!("REF-{:06}", self.rng.gen_range(100000..999999)),
                );
            }
        }

        terms
    }

    /// Generate AI summary.
    fn generate_ai_summary(&mut self, evidence_type: EvidenceType) -> String {
        match evidence_type {
            EvidenceType::Invoice => {
                "Invoice for goods/services with standard payment terms. Amount within expected range.".into()
            }
            EvidenceType::Contract => {
                "Multi-year agreement with standard commercial terms. Key provisions identified.".into()
            }
            EvidenceType::BankStatement => {
                "Month-end bank statement showing reconciled balance. No unusual items noted.".into()
            }
            _ => "Document reviewed and key data points extracted.".into(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_generation() {
        let mut generator = EvidenceGenerator::new(42);
        let evidence = generator.generate_evidence(
            Uuid::new_v4(),
            None,
            &[Assertion::Occurrence],
            &["STAFF001".into()],
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        );

        assert!(!evidence.evidence_ref.is_empty());
        assert!(!evidence.title.is_empty());
        assert!(evidence.file_size.is_some());
    }

    #[test]
    fn test_evidence_reliability() {
        let mut generator = EvidenceGenerator::new(42);

        // Generate multiple evidence pieces and check reliability
        for _ in 0..10 {
            let evidence = generator.generate_evidence(
                Uuid::new_v4(),
                None,
                &[],
                &["STAFF001".into()],
                NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            );

            // Verify reliability assessment is set
            assert!(!evidence.reliability_assessment.notes.is_empty());
        }
    }

    #[test]
    fn test_evidence_with_ai_extraction() {
        let config = EvidenceGeneratorConfig {
            ai_extraction_probability: 1.0, // Always extract
            ..Default::default()
        };
        let mut generator = EvidenceGenerator::with_config(42, config);

        let evidence = generator.generate_evidence(
            Uuid::new_v4(),
            None,
            &[],
            &["STAFF001".into()],
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        );

        assert!(evidence.ai_extracted_terms.is_some());
        assert!(evidence.ai_confidence.is_some());
        assert!(evidence.ai_summary.is_some());
    }

    #[test]
    fn test_evidence_workpaper_link() {
        let mut generator = EvidenceGenerator::new(42);
        let workpaper_id = Uuid::new_v4();

        let evidence = generator.generate_evidence(
            Uuid::new_v4(),
            Some(workpaper_id),
            &[Assertion::Completeness],
            &["STAFF001".into()],
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        );

        assert!(evidence.linked_workpapers.contains(&workpaper_id));
    }
}
