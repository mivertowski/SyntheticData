//! Internal audit generator per ISA 610.
//!
//! Generates an internal audit function record and associated reports for an
//! audit engagement.  The external auditor's reliance extent is determined
//! probabilistically from the configured ratios.

use chrono::{Duration, NaiveDate};
use datasynth_core::utils::seeded_rng;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use uuid::Uuid;

/// Generate a UUID from the seeded RNG so output is fully deterministic.
fn rng_uuid(rng: &mut ChaCha8Rng) -> Uuid {
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    // Stamp as UUID v4 (version + variant bits).
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

use datasynth_core::models::audit::{
    ActionPlan, ActionPlanStatus, AuditEngagement, CompetenceRating, IaAssessment,
    IaRecommendation, IaReportRating, IaReportStatus, IaWorkAssessment, InternalAuditFunction,
    InternalAuditReport, ObjectivityRating, RecommendationPriority, RelianceExtent, ReportingLine,
};

/// Configuration for internal audit function and report generation (ISA 610).
#[derive(Debug, Clone)]
pub struct InternalAuditGeneratorConfig {
    /// Number of IA reports to generate per engagement (min, max).
    pub reports_per_function: (u32, u32),
    /// Fraction of engagements where no internal audit function exists.
    pub no_reliance_ratio: f64,
    /// Fraction of engagements where limited reliance is placed.
    pub limited_reliance_ratio: f64,
    /// Fraction of engagements where significant reliance is placed.
    pub significant_reliance_ratio: f64,
    /// Fraction of engagements where full reliance is placed.
    pub full_reliance_ratio: f64,
    /// Number of recommendations per IA report (min, max).
    pub recommendations_per_report: (u32, u32),
}

impl Default for InternalAuditGeneratorConfig {
    fn default() -> Self {
        Self {
            reports_per_function: (2, 5),
            no_reliance_ratio: 0.20,
            limited_reliance_ratio: 0.50,
            significant_reliance_ratio: 0.25,
            full_reliance_ratio: 0.05,
            recommendations_per_report: (1, 4),
        }
    }
}

/// Generator for internal audit function records and IA reports per ISA 610.
pub struct InternalAuditGenerator {
    /// Seeded random number generator.
    rng: ChaCha8Rng,
    /// Configuration.
    config: InternalAuditGeneratorConfig,
}

impl InternalAuditGenerator {
    /// Create a new generator with the given seed and default configuration.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config: InternalAuditGeneratorConfig::default(),
        }
    }

    /// Create a new generator with custom configuration.
    pub fn with_config(seed: u64, config: InternalAuditGeneratorConfig) -> Self {
        Self {
            rng: seeded_rng(seed, 0),
            config,
        }
    }

    /// Generate an internal audit function and its associated reports.
    ///
    /// Returns `(function, reports)`.  When the reliance extent is
    /// `NoReliance`, the reports vec is empty — the entity either has no
    /// internal audit function or the external auditor has decided not to use
    /// its work at all.
    pub fn generate(
        &mut self,
        engagement: &AuditEngagement,
    ) -> (InternalAuditFunction, Vec<InternalAuditReport>) {
        // Determine reliance extent from configured ratios.
        let roll: f64 = self.rng.random();
        let no_cutoff = self.config.no_reliance_ratio;
        let limited_cutoff = no_cutoff + self.config.limited_reliance_ratio;
        let significant_cutoff = limited_cutoff + self.config.significant_reliance_ratio;
        // Anything above significant_cutoff → FullReliance.

        let reliance = if roll < no_cutoff {
            RelianceExtent::NoReliance
        } else if roll < limited_cutoff {
            RelianceExtent::LimitedReliance
        } else if roll < significant_cutoff {
            RelianceExtent::SignificantReliance
        } else {
            RelianceExtent::FullReliance
        };

        // Derive objectivity / competence ratings consistent with reliance level.
        let (objectivity, competence, assessment) = match reliance {
            RelianceExtent::NoReliance => (
                ObjectivityRating::Low,
                CompetenceRating::Low,
                IaAssessment::Ineffective,
            ),
            RelianceExtent::LimitedReliance => (
                ObjectivityRating::Moderate,
                CompetenceRating::Moderate,
                IaAssessment::PartiallyEffective,
            ),
            RelianceExtent::SignificantReliance => (
                ObjectivityRating::High,
                CompetenceRating::Moderate,
                IaAssessment::LargelyEffective,
            ),
            RelianceExtent::FullReliance => (
                ObjectivityRating::High,
                CompetenceRating::High,
                IaAssessment::FullyEffective,
            ),
        };

        // Reporting line — AuditCommittee is most common.
        let reporting_line = self.pick_reporting_line();

        // Staff count: 3–15 proportional to reliance.
        let staff_count: u32 = match reliance {
            RelianceExtent::NoReliance => self.rng.random_range(1_u32..=4_u32),
            RelianceExtent::LimitedReliance => self.rng.random_range(3_u32..=8_u32),
            RelianceExtent::SignificantReliance => self.rng.random_range(6_u32..=12_u32),
            RelianceExtent::FullReliance => self.rng.random_range(10_u32..=15_u32),
        };

        // Annual plan coverage: 40–95%.
        let coverage: f64 = self.rng.random_range(0.40_f64..0.95_f64);

        // Quality assurance programme present for significant/full reliance.
        let quality_assurance = matches!(
            reliance,
            RelianceExtent::SignificantReliance | RelianceExtent::FullReliance
        );

        let head_name = self.head_of_ia_name();
        let qualifications = self.ia_qualifications(&reliance);

        let mut function =
            InternalAuditFunction::new(engagement.engagement_id, "Internal Audit", &head_name);
        // Override the UUID so output is fully deterministic.
        let func_id = rng_uuid(&mut self.rng);
        function.function_id = func_id;
        function.function_ref = format!("IAF-{}", &func_id.simple().to_string()[..8]);
        function.reporting_line = reporting_line;
        function.staff_count = staff_count;
        function.annual_plan_coverage = coverage;
        function.quality_assurance = quality_assurance;
        function.isa_610_assessment = assessment;
        function.objectivity_rating = objectivity;
        function.competence_rating = competence;
        function.systematic_discipline = !matches!(reliance, RelianceExtent::NoReliance);
        function.reliance_extent = reliance;
        function.head_of_ia_qualifications = qualifications;
        function.direct_assistance = matches!(
            reliance,
            RelianceExtent::SignificantReliance | RelianceExtent::FullReliance
        ) && self.rng.random::<f64>() < 0.40;

        if reliance == RelianceExtent::NoReliance {
            return (function, Vec::new());
        }

        // Generate IA reports.
        let report_count = self
            .rng
            .random_range(self.config.reports_per_function.0..=self.config.reports_per_function.1)
            as usize;

        let mut reports = Vec::with_capacity(report_count);
        let audit_areas = self.audit_areas();
        // Shuffle audit areas to pick distinct ones per report.
        let area_count = audit_areas.len();

        for i in 0..report_count {
            let area = audit_areas[i % area_count];
            let report_title = format!("{} — Internal Audit Review", area);

            // Dates within the engagement period.
            let fieldwork_days = (engagement.fieldwork_end - engagement.fieldwork_start)
                .num_days()
                .max(1);
            let offset = self.rng.random_range(0_i64..fieldwork_days);
            let report_date = engagement.fieldwork_start + Duration::days(offset);

            // Period covered by the IA review: the engagement fiscal year.
            let period_start =
                NaiveDate::from_ymd_opt(engagement.fiscal_year as i32, 1, 1).unwrap_or(report_date);
            let period_end = engagement.period_end_date;

            let mut report = InternalAuditReport::new(
                engagement.engagement_id,
                function.function_id,
                &report_title,
                area,
                report_date,
                period_start,
                period_end,
            );
            // Override UUID for determinism.
            let report_id = rng_uuid(&mut self.rng);
            report.report_id = report_id;
            report.report_ref = format!("IAR-{}", &report_id.simple().to_string()[..8]);

            // Set scope / methodology text.
            report.scope_description =
                format!("Review of {} processes and controls for the period.", area);
            report.methodology =
                "Risk-based audit approach with control testing and data analytics.".to_string();

            // Findings and ratings.
            let findings: u32 = self.rng.random_range(1_u32..=8_u32);
            let high_risk: u32 = self.rng.random_range(0_u32..=(findings.min(2)));
            report.findings_count = findings;
            report.high_risk_findings = high_risk;
            report.overall_rating = self.pick_report_rating(high_risk, findings);
            report.status = self.pick_report_status();

            // Generate recommendations.
            let rec_count = self.rng.random_range(
                self.config.recommendations_per_report.0..=self.config.recommendations_per_report.1,
            ) as usize;
            let mut recommendations = Vec::with_capacity(rec_count);
            let mut action_plans = Vec::with_capacity(rec_count);

            for _ in 0..rec_count {
                let priority = self.pick_priority(high_risk);
                let description = self.recommendation_description(area, priority);
                let management_response = Some(
                    "Management accepts recommendation and will implement by target date."
                        .to_string(),
                );

                let rec = IaRecommendation {
                    recommendation_id: rng_uuid(&mut self.rng),
                    description,
                    priority,
                    management_response,
                };

                // Action plan for each recommendation.
                let days_to_implement: i64 = match priority {
                    RecommendationPriority::Critical => self.rng.random_range(30_i64..=60_i64),
                    RecommendationPriority::High => self.rng.random_range(60_i64..=90_i64),
                    RecommendationPriority::Medium => self.rng.random_range(90_i64..=180_i64),
                    RecommendationPriority::Low => self.rng.random_range(180_i64..=365_i64),
                };
                let target_date = report_date + Duration::days(days_to_implement);
                let plan_status = self.pick_action_plan_status();

                let plan = ActionPlan {
                    plan_id: rng_uuid(&mut self.rng),
                    recommendation_id: rec.recommendation_id,
                    description: format!(
                        "Implement corrective action for: {}",
                        &rec.description[..rec.description.len().min(60)]
                    ),
                    responsible_party: self.responsible_party(),
                    target_date,
                    status: plan_status,
                };

                action_plans.push(plan);
                recommendations.push(rec);
            }

            report.recommendations = recommendations;
            report.management_action_plans = action_plans;

            // External auditor's assessment.
            report.external_auditor_assessment = Some(match reliance {
                RelianceExtent::LimitedReliance => IaWorkAssessment::PartiallyReliable,
                RelianceExtent::SignificantReliance => IaWorkAssessment::Reliable,
                RelianceExtent::FullReliance => IaWorkAssessment::Reliable,
                RelianceExtent::NoReliance => IaWorkAssessment::Unreliable,
            });

            // Populate reliance areas on the function from report audit areas.
            if !function.reliance_areas.contains(&area.to_string()) {
                function.reliance_areas.push(area.to_string());
            }

            reports.push(report);
        }

        (function, reports)
    }

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    fn pick_reporting_line(&mut self) -> ReportingLine {
        // AuditCommittee 60%, Board 15%, CFO 15%, CEO 10%.
        let roll: f64 = self.rng.random();
        if roll < 0.60 {
            ReportingLine::AuditCommittee
        } else if roll < 0.75 {
            ReportingLine::Board
        } else if roll < 0.90 {
            ReportingLine::CFO
        } else {
            ReportingLine::CEO
        }
    }

    fn head_of_ia_name(&mut self) -> String {
        let names = [
            "Sarah Mitchell",
            "David Chen",
            "Emma Thompson",
            "James Rodriguez",
            "Olivia Patel",
            "Michael Clarke",
            "Amira Hassan",
            "Robert Nielsen",
            "Priya Sharma",
            "Thomas Becker",
        ];
        let idx = self.rng.random_range(0..names.len());
        names[idx].to_string()
    }

    fn ia_qualifications(&mut self, reliance: &RelianceExtent) -> Vec<String> {
        let all_quals = ["CIA", "CISA", "CPA", "CA", "ACCA", "CRISC"];
        let count: usize = match reliance {
            RelianceExtent::NoReliance | RelianceExtent::LimitedReliance => {
                self.rng.random_range(0_usize..=1_usize)
            }
            RelianceExtent::SignificantReliance => self.rng.random_range(1_usize..=2_usize),
            RelianceExtent::FullReliance => self.rng.random_range(2_usize..=3_usize),
        };
        let mut quals = Vec::new();
        let mut remaining: Vec<&str> = all_quals.to_vec();
        for _ in 0..count {
            if remaining.is_empty() {
                break;
            }
            let idx = self.rng.random_range(0..remaining.len());
            quals.push(remaining.remove(idx).to_string());
        }
        quals
    }

    fn audit_areas(&self) -> Vec<&'static str> {
        vec![
            "Revenue Cycle",
            "IT General Controls",
            "Procurement & Payables",
            "Payroll & Human Resources",
            "Treasury & Cash Management",
            "Financial Reporting",
            "Compliance & Regulatory",
            "Inventory & Supply Chain",
            "Fixed Assets",
            "Tax Compliance",
            "Information Security",
            "Governance & Risk Management",
        ]
    }

    fn pick_report_rating(&mut self, high_risk: u32, findings: u32) -> IaReportRating {
        if high_risk >= 2 || findings >= 6 {
            IaReportRating::Unsatisfactory
        } else if high_risk >= 1 || findings >= 3 {
            // 70% NeedsImprovement, 30% Unsatisfactory.
            if self.rng.random::<f64>() < 0.70 {
                IaReportRating::NeedsImprovement
            } else {
                IaReportRating::Unsatisfactory
            }
        } else {
            // Mostly satisfactory.
            if self.rng.random::<f64>() < 0.80 {
                IaReportRating::Satisfactory
            } else {
                IaReportRating::NeedsImprovement
            }
        }
    }

    fn pick_report_status(&mut self) -> IaReportStatus {
        let roll: f64 = self.rng.random();
        if roll < 0.65 {
            IaReportStatus::Final
        } else {
            IaReportStatus::Draft
        }
    }

    fn pick_priority(&mut self, high_risk: u32) -> RecommendationPriority {
        let roll: f64 = self.rng.random();
        if high_risk >= 1 && roll < 0.15 {
            RecommendationPriority::Critical
        } else if roll < 0.30 {
            RecommendationPriority::High
        } else if roll < 0.70 {
            RecommendationPriority::Medium
        } else {
            RecommendationPriority::Low
        }
    }

    fn pick_action_plan_status(&mut self) -> ActionPlanStatus {
        let roll: f64 = self.rng.random();
        if roll < 0.40 {
            ActionPlanStatus::Open
        } else if roll < 0.65 {
            ActionPlanStatus::InProgress
        } else if roll < 0.85 {
            ActionPlanStatus::Implemented
        } else {
            ActionPlanStatus::Overdue
        }
    }

    fn recommendation_description(&self, area: &str, priority: RecommendationPriority) -> String {
        let suffix = match priority {
            RecommendationPriority::Critical => {
                "Immediate remediation required to address critical control failure."
            }
            RecommendationPriority::High => {
                "Strengthen controls to reduce risk exposure within the next quarter."
            }
            RecommendationPriority::Medium => {
                "Enhance monitoring procedures and update process documentation."
            }
            RecommendationPriority::Low => {
                "Implement process improvement to increase efficiency and control effectiveness."
            }
        };
        format!("{}: {}", area, suffix)
    }

    fn responsible_party(&mut self) -> String {
        let parties = [
            "Finance Director",
            "Head of Compliance",
            "IT Manager",
            "Operations Manager",
            "Controller",
            "CFO",
            "Risk Manager",
            "HR Director",
        ];
        let idx = self.rng.random_range(0..parties.len());
        parties[idx].to_string()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::audit::test_helpers::create_test_engagement;

    fn make_gen(seed: u64) -> InternalAuditGenerator {
        InternalAuditGenerator::new(seed)
    }

    // -------------------------------------------------------------------------

    /// Generator produces an IA function for any engagement.
    #[test]
    fn test_generates_ia_function() {
        let engagement = create_test_engagement();
        let mut gen = make_gen(42);
        let (function, _) = gen.generate(&engagement);

        assert_eq!(function.engagement_id, engagement.engagement_id);
        assert!(!function.head_of_ia.is_empty());
        assert!(!function.department_name.is_empty());
        assert!(function.function_ref.starts_with("IAF-"));
    }

    /// With NoReliance, the reports vec must be empty.
    #[test]
    fn test_no_reliance_empty_reports() {
        let engagement = create_test_engagement();
        // Force NoReliance by setting the ratio to 1.0.
        let config = InternalAuditGeneratorConfig {
            no_reliance_ratio: 1.0,
            limited_reliance_ratio: 0.0,
            significant_reliance_ratio: 0.0,
            full_reliance_ratio: 0.0,
            ..Default::default()
        };
        let mut gen = InternalAuditGenerator::with_config(10, config);
        let (function, reports) = gen.generate(&engagement);

        assert_eq!(function.reliance_extent, RelianceExtent::NoReliance);
        assert!(reports.is_empty(), "NoReliance should produce zero reports");
    }

    /// Report count is within the configured range when reliance > NoReliance.
    #[test]
    fn test_reports_within_range() {
        let engagement = create_test_engagement();
        // Force full reliance so we always get reports.
        let config = InternalAuditGeneratorConfig {
            no_reliance_ratio: 0.0,
            limited_reliance_ratio: 0.0,
            significant_reliance_ratio: 0.0,
            full_reliance_ratio: 1.0,
            reports_per_function: (2, 5),
            ..Default::default()
        };
        let mut gen = InternalAuditGenerator::with_config(7, config);
        let (_, reports) = gen.generate(&engagement);

        assert!(
            reports.len() >= 2 && reports.len() <= 5,
            "expected 2..=5 reports, got {}",
            reports.len()
        );
    }

    /// Every report has at least one recommendation.
    #[test]
    fn test_recommendations_generated() {
        let engagement = create_test_engagement();
        let config = InternalAuditGeneratorConfig {
            no_reliance_ratio: 0.0,
            limited_reliance_ratio: 1.0,
            ..Default::default()
        };
        let mut gen = InternalAuditGenerator::with_config(55, config);
        let (_, reports) = gen.generate(&engagement);

        for report in &reports {
            assert!(
                !report.recommendations.is_empty(),
                "report '{}' should have at least one recommendation",
                report.report_ref
            );
            // Each recommendation must have a matching action plan.
            assert_eq!(
                report.recommendations.len(),
                report.management_action_plans.len(),
                "recommendation/action-plan count mismatch in report '{}'",
                report.report_ref
            );
        }
    }

    /// Same seed must produce identical output.
    #[test]
    fn test_deterministic() {
        let engagement = create_test_engagement();

        let (func_a, reports_a) = {
            let mut gen = make_gen(999);
            gen.generate(&engagement)
        };
        let (func_b, reports_b) = {
            let mut gen = make_gen(999);
            gen.generate(&engagement)
        };

        assert_eq!(func_a.reliance_extent, func_b.reliance_extent);
        assert_eq!(func_a.head_of_ia, func_b.head_of_ia);
        assert_eq!(func_a.staff_count, func_b.staff_count);
        assert_eq!(reports_a.len(), reports_b.len());
        for (a, b) in reports_a.iter().zip(reports_b.iter()) {
            assert_eq!(a.report_ref, b.report_ref);
            assert_eq!(a.audit_area, b.audit_area);
            assert_eq!(a.overall_rating, b.overall_rating);
            assert_eq!(a.findings_count, b.findings_count);
        }
    }
}
