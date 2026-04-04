//! Workpaper generator for audit engagements.
//!
//! Generates audit workpapers with appropriate procedures, sampling,
//! results, and review sign-offs per ISA 230.

use chrono::{Duration, NaiveDate};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use rust_decimal::Decimal;

use datasynth_core::models::audit::{
    Assertion, AuditEngagement, EngagementPhase, ProcedureType, RiskLevel, SamplingMethod,
    Workpaper, WorkpaperConclusion, WorkpaperScope, WorkpaperSection, WorkpaperStatus,
};

/// Configuration for workpaper generation.
#[derive(Debug, Clone)]
pub struct WorkpaperGeneratorConfig {
    /// Number of workpapers per section (min, max)
    pub workpapers_per_section: (u32, u32),
    /// Population size range for testing (min, max)
    pub population_size_range: (u64, u64),
    /// Sample size as percentage of population (min, max)
    pub sample_percentage_range: (f64, f64),
    /// Exception rate range (min, max)
    pub exception_rate_range: (f64, f64),
    /// Probability of unsatisfactory conclusion
    pub unsatisfactory_probability: f64,
    /// Days between preparation and first review (min, max)
    pub first_review_delay_range: (u32, u32),
    /// Days between first and second review (min, max)
    pub second_review_delay_range: (u32, u32),
}

impl Default for WorkpaperGeneratorConfig {
    fn default() -> Self {
        Self {
            workpapers_per_section: (3, 10),
            population_size_range: (100, 10000),
            sample_percentage_range: (0.01, 0.10),
            exception_rate_range: (0.0, 0.08),
            unsatisfactory_probability: 0.05,
            first_review_delay_range: (1, 5),
            second_review_delay_range: (1, 3),
        }
    }
}

/// Context for generating coherent workpapers with real financial data.
///
/// When provided, the workpaper title, objective, procedure, and scope are
/// enriched with concrete financial figures and risk context from the
/// engagement's artifact bag.
#[derive(Debug, Clone, Default)]
pub struct WorkpaperEnrichment {
    /// Account area name (e.g., "Revenue", "Trade Receivables").
    pub account_area: Option<String>,
    /// Real GL balance for the area.
    pub account_balance: Option<Decimal>,
    /// CRA risk level.
    pub risk_level: Option<String>, // "High", "Moderate", "Low", "Minimal"
    /// Performance materiality.
    pub materiality: Option<Decimal>,
    /// Sampling plan details (pre-formatted string for simplicity).
    pub sampling_info: Option<String>,
}

/// Generator for audit workpapers.
pub struct WorkpaperGenerator {
    /// Random number generator
    rng: ChaCha8Rng,
    /// Configuration
    config: WorkpaperGeneratorConfig,
    /// Counter per section for references
    section_counters: std::collections::HashMap<WorkpaperSection, u32>,
}

impl WorkpaperGenerator {
    /// Create a new generator with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: WorkpaperGeneratorConfig::default(),
            section_counters: std::collections::HashMap::new(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: WorkpaperGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
            section_counters: std::collections::HashMap::new(),
        }
    }

    /// Generate workpapers for an engagement phase.
    pub fn generate_workpapers_for_phase(
        &mut self,
        engagement: &AuditEngagement,
        phase: EngagementPhase,
        phase_date: NaiveDate,
        team_members: &[String],
    ) -> Vec<Workpaper> {
        let section = match phase {
            EngagementPhase::Planning => WorkpaperSection::Planning,
            EngagementPhase::RiskAssessment => WorkpaperSection::RiskAssessment,
            EngagementPhase::ControlTesting => WorkpaperSection::ControlTesting,
            EngagementPhase::SubstantiveTesting => WorkpaperSection::SubstantiveTesting,
            EngagementPhase::Completion => WorkpaperSection::Completion,
            EngagementPhase::Reporting => WorkpaperSection::Reporting,
        };

        let count = self.rng.random_range(
            self.config.workpapers_per_section.0..=self.config.workpapers_per_section.1,
        );

        (0..count)
            .map(|_| self.generate_workpaper(engagement, section, phase_date, team_members))
            .collect()
    }

    /// Generate a single workpaper.
    pub fn generate_workpaper(
        &mut self,
        engagement: &AuditEngagement,
        section: WorkpaperSection,
        base_date: NaiveDate,
        team_members: &[String],
    ) -> Workpaper {
        let counter = self.section_counters.entry(section).or_insert(0);
        *counter += 1;

        let workpaper_ref = format!("{}-{:03}", section.reference_prefix(), counter);
        let title = self.generate_workpaper_title(section);

        let mut wp = Workpaper::new(engagement.engagement_id, &workpaper_ref, &title, section);

        // Set objective and assertions
        let (objective, assertions) = self.generate_objective_and_assertions(section);
        wp = wp.with_objective(&objective, assertions);

        // Set procedure
        let (procedure, procedure_type) = self.generate_procedure(section);
        wp = wp.with_procedure(&procedure, procedure_type);

        // Set scope and sampling
        let (scope, population, sample, method) = self.generate_scope_and_sampling(section);
        wp = wp.with_scope(scope, population, sample, method);

        // Set results
        let (summary, exceptions, conclusion) =
            self.generate_results(sample, &engagement.overall_audit_risk);
        wp = wp.with_results(&summary, exceptions, conclusion);

        wp.risk_level_addressed = engagement.overall_audit_risk;

        // Set preparer
        let preparer = self.select_team_member(team_members, "staff");
        let preparer_name = self.generate_auditor_name();
        wp = wp.with_preparer(&preparer, &preparer_name, base_date);

        // Add first review
        let first_review_delay = self.rng.random_range(
            self.config.first_review_delay_range.0..=self.config.first_review_delay_range.1,
        );
        let first_review_date = base_date + Duration::days(first_review_delay as i64);
        let reviewer = self.select_team_member(team_members, "senior");
        let reviewer_name = self.generate_auditor_name();
        wp.add_first_review(&reviewer, &reviewer_name, first_review_date);

        // Maybe add second review
        if self.rng.random::<f64>() < 0.7 {
            let second_review_delay = self.rng.random_range(
                self.config.second_review_delay_range.0..=self.config.second_review_delay_range.1,
            );
            let second_review_date = first_review_date + Duration::days(second_review_delay as i64);
            let second_reviewer = self.select_team_member(team_members, "manager");
            let second_reviewer_name = self.generate_auditor_name();
            wp.add_second_review(&second_reviewer, &second_reviewer_name, second_review_date);
        } else {
            wp.status = WorkpaperStatus::FirstReviewComplete;
        }

        // Maybe add review notes
        if self.rng.random::<f64>() < 0.30 {
            let note = self.generate_review_note();
            wp.add_review_note(&reviewer, &note);
        }

        wp
    }

    /// Generate a workpaper enriched with real financial data and risk context.
    ///
    /// Produces the base workpaper via [`generate_workpaper`] and then enriches
    /// the title, objective, procedure, and scope fields using the provided
    /// [`WorkpaperEnrichment`] context.
    pub fn generate_workpaper_with_context(
        &mut self,
        engagement: &AuditEngagement,
        section: WorkpaperSection,
        base_date: NaiveDate,
        team_members: &[String],
        enrichment: &WorkpaperEnrichment,
    ) -> Workpaper {
        let mut wp = self.generate_workpaper(engagement, section, base_date, team_members);

        // --- Title enrichment ---
        if let (Some(area), Some(risk)) = (&enrichment.account_area, &enrichment.risk_level) {
            wp.title = format!("{} \u{2014} {} Risk", area, risk);
        } else if let Some(area) = &enrichment.account_area {
            wp.title = format!("{} {}", area, wp.title);
        }

        // --- Objective enrichment ---
        let mut addenda = Vec::new();
        if let Some(balance) = enrichment.account_balance {
            addenda.push(format!("GL Balance: ${}", balance));
        }
        if let Some(mat) = enrichment.materiality {
            addenda.push(format!("Performance materiality: ${}", mat));
        }
        if !addenda.is_empty() {
            wp.objective = format!("{} | {}", wp.objective, addenda.join(". "));
        }

        // --- Procedure enrichment ---
        if let Some(sampling) = &enrichment.sampling_info {
            wp.procedure_performed = format!("{} | Sample: {}", wp.procedure_performed, sampling);
        }

        // --- Scope adjustment by risk level ---
        if let Some(risk) = &enrichment.risk_level {
            let (lo, hi) = match risk.as_str() {
                "High" => (95.0, 100.0),
                "Moderate" => (75.0, 90.0),
                "Low" | "Minimal" => (50.0, 70.0),
                _ => (70.0, 100.0),
            };
            wp.scope.coverage_percentage = self.rng.random_range(lo..hi);
        }

        wp
    }

    /// Generate workpaper title based on section.
    fn generate_workpaper_title(&mut self, section: WorkpaperSection) -> String {
        let titles = match section {
            WorkpaperSection::Planning => vec![
                "Engagement Planning Memo",
                "Understanding the Entity and Environment",
                "Materiality Assessment",
                "Preliminary Analytical Procedures",
                "Risk Assessment Summary",
                "Audit Strategy and Approach",
                "Staffing and Resource Plan",
                "Client Acceptance Procedures",
            ],
            WorkpaperSection::RiskAssessment => vec![
                "Business Risk Assessment",
                "Fraud Risk Evaluation",
                "IT General Controls Assessment",
                "Internal Control Evaluation",
                "Significant Account Identification",
                "Risk of Material Misstatement Assessment",
                "Related Party Risk Assessment",
                "Going Concern Assessment",
            ],
            WorkpaperSection::ControlTesting => vec![
                "Revenue Recognition Controls Testing",
                "Purchase and Payables Controls Testing",
                "Treasury and Cash Controls Testing",
                "Payroll Controls Testing",
                "Fixed Asset Controls Testing",
                "Inventory Controls Testing",
                "IT Application Controls Testing",
                "Entity Level Controls Testing",
            ],
            WorkpaperSection::SubstantiveTesting => vec![
                "Revenue Cutoff Testing",
                "Accounts Receivable Confirmation",
                "Inventory Observation and Testing",
                "Fixed Asset Verification",
                "Accounts Payable Completeness",
                "Expense Testing",
                "Debt and Interest Testing",
                "Bank Reconciliation Review",
                "Journal Entry Testing",
                "Analytical Procedures - Revenue",
                "Analytical Procedures - Expenses",
            ],
            WorkpaperSection::Completion => vec![
                "Subsequent Events Review",
                "Management Representation Letter",
                "Attorney Letter Summary",
                "Going Concern Evaluation",
                "Summary of Uncorrected Misstatements",
                "Summary of Audit Differences",
                "Completion Checklist",
            ],
            WorkpaperSection::Reporting => vec![
                "Draft Financial Statements Review",
                "Disclosure Checklist",
                "Communication with Those Charged with Governance",
                "Report Issuance Checklist",
            ],
            WorkpaperSection::PermanentFile => vec![
                "Chart of Accounts",
                "Organization Structure",
                "Key Contracts Summary",
                "Related Party Identification",
            ],
        };

        let idx = self.rng.random_range(0..titles.len());
        titles[idx].to_string()
    }

    /// Generate objective and assertions for a section.
    fn generate_objective_and_assertions(
        &mut self,
        section: WorkpaperSection,
    ) -> (String, Vec<Assertion>) {
        match section {
            WorkpaperSection::Planning | WorkpaperSection::RiskAssessment => (
                "Understand the entity and assess risks of material misstatement".into(),
                vec![],
            ),
            WorkpaperSection::ControlTesting => (
                "Test the operating effectiveness of key controls".into(),
                Assertion::transaction_assertions(),
            ),
            WorkpaperSection::SubstantiveTesting => {
                let assertions = if self.rng.random::<f64>() < 0.5 {
                    Assertion::transaction_assertions()
                } else {
                    Assertion::balance_assertions()
                };
                (
                    "Obtain sufficient appropriate audit evidence regarding account balances"
                        .into(),
                    assertions,
                )
            }
            WorkpaperSection::Completion => (
                "Complete all required completion procedures".into(),
                vec![
                    Assertion::Completeness,
                    Assertion::PresentationAndDisclosure,
                ],
            ),
            WorkpaperSection::Reporting => (
                "Ensure compliance with reporting requirements".into(),
                vec![Assertion::PresentationAndDisclosure],
            ),
            WorkpaperSection::PermanentFile => {
                ("Maintain permanent file documentation".into(), vec![])
            }
        }
    }

    /// Generate procedure description and type.
    fn generate_procedure(&mut self, section: WorkpaperSection) -> (String, ProcedureType) {
        match section {
            WorkpaperSection::Planning | WorkpaperSection::RiskAssessment => (
                "Performed inquiries and reviewed documentation".into(),
                ProcedureType::InquiryObservation,
            ),
            WorkpaperSection::ControlTesting => {
                let procedures = [
                    (
                        "Selected a sample of transactions and tested the control operation",
                        ProcedureType::TestOfControls,
                    ),
                    (
                        "Observed the control being performed by personnel",
                        ProcedureType::InquiryObservation,
                    ),
                    (
                        "Inspected documentation of control performance",
                        ProcedureType::Inspection,
                    ),
                    (
                        "Reperformed the control procedure",
                        ProcedureType::Reperformance,
                    ),
                ];
                let idx = self.rng.random_range(0..procedures.len());
                (procedures[idx].0.into(), procedures[idx].1)
            }
            WorkpaperSection::SubstantiveTesting => {
                let procedures = [
                    (
                        "Selected a sample and agreed details to supporting documentation",
                        ProcedureType::SubstantiveTest,
                    ),
                    (
                        "Sent confirmations and agreed responses to records",
                        ProcedureType::Confirmation,
                    ),
                    (
                        "Recalculated amounts and agreed to supporting schedules",
                        ProcedureType::Recalculation,
                    ),
                    (
                        "Performed analytical procedures and investigated variances",
                        ProcedureType::AnalyticalProcedures,
                    ),
                    (
                        "Inspected physical assets and documentation",
                        ProcedureType::Inspection,
                    ),
                ];
                let idx = self.rng.random_range(0..procedures.len());
                (procedures[idx].0.into(), procedures[idx].1)
            }
            WorkpaperSection::Completion | WorkpaperSection::Reporting => (
                "Reviewed documentation and performed inquiries".into(),
                ProcedureType::InquiryObservation,
            ),
            WorkpaperSection::PermanentFile => (
                "Compiled and organized permanent file documentation".into(),
                ProcedureType::Inspection,
            ),
        }
    }

    /// Generate scope and sampling details.
    fn generate_scope_and_sampling(
        &mut self,
        section: WorkpaperSection,
    ) -> (WorkpaperScope, u64, u32, SamplingMethod) {
        let scope = WorkpaperScope {
            coverage_percentage: self.rng.random_range(70.0..100.0),
            period_start: None,
            period_end: None,
            limitations: Vec::new(),
        };

        match section {
            WorkpaperSection::ControlTesting | WorkpaperSection::SubstantiveTesting => {
                let population = self.rng.random_range(
                    self.config.population_size_range.0..=self.config.population_size_range.1,
                );
                let sample_pct = self.rng.random_range(
                    self.config.sample_percentage_range.0..=self.config.sample_percentage_range.1,
                );
                let sample = ((population as f64 * sample_pct).max(25.0) as u32).min(200);

                let method = if self.rng.random::<f64>() < 0.4 {
                    SamplingMethod::StatisticalRandom
                } else if self.rng.random::<f64>() < 0.3 {
                    SamplingMethod::MonetaryUnit
                } else {
                    SamplingMethod::Judgmental
                };

                (scope, population, sample, method)
            }
            _ => (scope, 0, 0, SamplingMethod::Judgmental),
        }
    }

    /// Generate test results.
    fn generate_results(
        &mut self,
        sample_size: u32,
        risk_level: &RiskLevel,
    ) -> (String, u32, WorkpaperConclusion) {
        if sample_size == 0 {
            return (
                "Procedures completed without exception".into(),
                0,
                WorkpaperConclusion::Satisfactory,
            );
        }

        // Higher risk = higher chance of exceptions
        let exception_probability = match risk_level {
            RiskLevel::Low => 0.10,
            RiskLevel::Medium => 0.25,
            RiskLevel::High | RiskLevel::Significant => 0.40,
        };

        let has_exceptions = self.rng.random::<f64>() < exception_probability;

        let (exceptions, conclusion) = if has_exceptions {
            let exception_rate = self.rng.random_range(
                self.config.exception_rate_range.0..=self.config.exception_rate_range.1,
            );
            let exceptions =
                ((sample_size as f64 * exception_rate).max(1.0) as u32).min(sample_size);

            let conclusion = if self.rng.random::<f64>() < self.config.unsatisfactory_probability {
                WorkpaperConclusion::Unsatisfactory
            } else {
                WorkpaperConclusion::SatisfactoryWithExceptions
            };

            (exceptions, conclusion)
        } else {
            (0, WorkpaperConclusion::Satisfactory)
        };

        let summary = match conclusion {
            WorkpaperConclusion::Satisfactory => {
                format!("Tested {sample_size} items with no exceptions noted")
            }
            WorkpaperConclusion::SatisfactoryWithExceptions => {
                format!(
                    "Tested {sample_size} items with {exceptions} exceptions noted. Exceptions were immaterial and have been evaluated"
                )
            }
            WorkpaperConclusion::Unsatisfactory => {
                format!(
                    "Tested {sample_size} items with {exceptions} exceptions noted. Exceptions represent material misstatement requiring adjustment"
                )
            }
            _ => format!("Tested {sample_size} items"),
        };

        (summary, exceptions, conclusion)
    }

    /// Select a team member based on role prefix.
    fn select_team_member(&mut self, team_members: &[String], role_hint: &str) -> String {
        let matching: Vec<&String> = team_members
            .iter()
            .filter(|m| m.to_lowercase().contains(role_hint))
            .collect();

        if let Some(&member) = matching.first() {
            member.clone()
        } else if !team_members.is_empty() {
            let idx = self.rng.random_range(0..team_members.len());
            team_members[idx].clone()
        } else {
            format!("{}001", role_hint.to_uppercase())
        }
    }

    /// Generate a plausible auditor name.
    fn generate_auditor_name(&mut self) -> String {
        let first_names = [
            "Michael", "Sarah", "David", "Jennifer", "Robert", "Emily", "James", "Amanda",
        ];
        let last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Davis"];

        let first_idx = self.rng.random_range(0..first_names.len());
        let last_idx = self.rng.random_range(0..last_names.len());

        format!("{} {}", first_names[first_idx], last_names[last_idx])
    }

    /// Generate a review note comment.
    fn generate_review_note(&mut self) -> String {
        let notes = [
            "Please expand on the rationale for sample selection",
            "Cross-reference needed to risk assessment workpaper",
            "Please document conclusion more clearly",
            "Need to include population definition",
            "Please add reference to prior year workpaper",
            "Document discussion with management regarding exceptions",
            "Clarify testing approach for this control",
            "Add evidence reference for supporting documentation",
        ];

        let idx = self.rng.random_range(0..notes.len());
        notes[idx].to_string()
    }

    /// Generate all workpapers for a complete engagement.
    pub fn generate_complete_workpaper_set(
        &mut self,
        engagement: &AuditEngagement,
        team_members: &[String],
    ) -> Vec<Workpaper> {
        let mut all_workpapers = Vec::new();

        // Planning workpapers
        all_workpapers.extend(self.generate_workpapers_for_phase(
            engagement,
            EngagementPhase::Planning,
            engagement.planning_start,
            team_members,
        ));

        // Risk assessment workpapers
        all_workpapers.extend(self.generate_workpapers_for_phase(
            engagement,
            EngagementPhase::RiskAssessment,
            engagement.planning_end,
            team_members,
        ));

        // Control testing workpapers
        all_workpapers.extend(self.generate_workpapers_for_phase(
            engagement,
            EngagementPhase::ControlTesting,
            engagement.fieldwork_start,
            team_members,
        ));

        // Substantive testing workpapers
        all_workpapers.extend(self.generate_workpapers_for_phase(
            engagement,
            EngagementPhase::SubstantiveTesting,
            engagement.fieldwork_start + Duration::days(14),
            team_members,
        ));

        // Completion workpapers
        all_workpapers.extend(self.generate_workpapers_for_phase(
            engagement,
            EngagementPhase::Completion,
            engagement.completion_start,
            team_members,
        ));

        // Reporting workpapers
        all_workpapers.extend(self.generate_workpapers_for_phase(
            engagement,
            EngagementPhase::Reporting,
            engagement.report_date - Duration::days(7),
            team_members,
        ));

        all_workpapers
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::audit::test_helpers::create_test_engagement;

    #[test]
    fn test_workpaper_generation() {
        let mut generator = WorkpaperGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into(), "SENIOR001".into(), "MANAGER001".into()];

        let wp = generator.generate_workpaper(
            &engagement,
            WorkpaperSection::SubstantiveTesting,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            &team,
        );

        assert!(!wp.workpaper_ref.is_empty());
        assert!(!wp.title.is_empty());
        assert!(!wp.preparer_id.is_empty());
    }

    #[test]
    fn test_phase_workpapers() {
        let mut generator = WorkpaperGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into(), "SENIOR001".into()];

        let workpapers = generator.generate_workpapers_for_phase(
            &engagement,
            EngagementPhase::ControlTesting,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            &team,
        );

        assert!(!workpapers.is_empty());
        for wp in &workpapers {
            assert_eq!(wp.section, WorkpaperSection::ControlTesting);
        }
    }

    #[test]
    fn test_complete_workpaper_set() {
        let mut generator = WorkpaperGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec![
            "STAFF001".into(),
            "STAFF002".into(),
            "SENIOR001".into(),
            "MANAGER001".into(),
        ];

        let workpapers = generator.generate_complete_workpaper_set(&engagement, &team);

        // Should have workpapers from all phases
        assert!(workpapers.len() >= 18); // At least 3 per 6 phases

        // Check we have workpapers from different sections
        let sections: std::collections::HashSet<_> = workpapers.iter().map(|w| w.section).collect();
        assert!(sections.len() >= 5);
    }

    #[test]
    fn test_workpaper_with_context_enriches_title_and_objective() {
        let mut generator = WorkpaperGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into(), "SENIOR001".into(), "MANAGER001".into()];

        let enrichment = WorkpaperEnrichment {
            account_area: Some("Revenue".into()),
            account_balance: Some(Decimal::new(5_000_000, 0)),
            risk_level: Some("High".into()),
            materiality: Some(Decimal::new(325_000, 0)),
            sampling_info: Some("MUS \u{2014} Population: 1200, Sample: 45".into()),
        };

        let wp = generator.generate_workpaper_with_context(
            &engagement,
            WorkpaperSection::SubstantiveTesting,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            &team,
            &enrichment,
        );

        assert!(
            wp.title.contains("Revenue") && wp.title.contains("High"),
            "Title should contain account area and risk: {}",
            wp.title
        );
        assert!(
            wp.objective.contains("GL Balance"),
            "Objective should contain GL balance: {}",
            wp.objective
        );
        assert!(
            wp.objective.contains("materiality"),
            "Objective should contain materiality: {}",
            wp.objective
        );
        assert!(
            wp.procedure_performed.contains("Sample: MUS"),
            "Procedure should contain sampling info: {}",
            wp.procedure_performed
        );
        // High risk should give 95-100% coverage.
        assert!(
            wp.scope.coverage_percentage >= 95.0,
            "High risk scope should be >=95%: {}",
            wp.scope.coverage_percentage
        );
    }

    #[test]
    fn test_workpaper_with_empty_enrichment_same_as_base() {
        let mut gen1 = WorkpaperGenerator::new(42);
        let mut gen2 = WorkpaperGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into(), "SENIOR001".into()];

        let base = gen1.generate_workpaper(
            &engagement,
            WorkpaperSection::Planning,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            &team,
        );

        let enriched = gen2.generate_workpaper_with_context(
            &engagement,
            WorkpaperSection::Planning,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            &team,
            &WorkpaperEnrichment::default(),
        );

        // With empty enrichment, objective should be unchanged (no addenda).
        assert_eq!(base.objective, enriched.objective);
    }

    #[test]
    fn test_workpaper_scope_adjustment_by_risk() {
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into()];

        for (risk, min_cov) in [("High", 95.0), ("Moderate", 75.0), ("Low", 50.0)] {
            let mut generator = WorkpaperGenerator::new(99);
            let enrichment = WorkpaperEnrichment {
                risk_level: Some(risk.into()),
                ..Default::default()
            };
            let wp = generator.generate_workpaper_with_context(
                &engagement,
                WorkpaperSection::SubstantiveTesting,
                NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                &team,
                &enrichment,
            );
            assert!(
                wp.scope.coverage_percentage >= min_cov,
                "Risk={risk}: expected coverage >= {min_cov}, got {}",
                wp.scope.coverage_percentage
            );
        }
    }

    #[test]
    fn test_workpaper_review_chain() {
        let mut generator = WorkpaperGenerator::new(42);
        let engagement = create_test_engagement();
        let team = vec!["STAFF001".into(), "SENIOR001".into(), "MANAGER001".into()];

        let wp = generator.generate_workpaper(
            &engagement,
            WorkpaperSection::SubstantiveTesting,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            &team,
        );

        // Should have preparer
        assert!(!wp.preparer_id.is_empty());

        // Should have first reviewer
        assert!(wp.reviewer_id.is_some());

        // First review date should be after preparer date
        assert!(wp.reviewer_date.unwrap() >= wp.preparer_date);
    }
}
