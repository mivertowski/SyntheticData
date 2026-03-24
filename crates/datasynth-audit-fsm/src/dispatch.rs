//! Step dispatcher mapping FSM step commands to audit generators.
//!
//! [`StepDispatcher`] is the bridge between the FSM engine's abstract step
//! commands (e.g. `"determine_overall_materiality"`) and the concrete
//! generators in `datasynth-generators`.  Each dispatch method:
//!
//! 1. Reads prerequisite artifacts from the [`ArtifactBag`].
//! 2. Builds the generator-specific input struct from [`EngagementContext`].
//! 3. Calls the generator.
//! 4. Pushes the resulting artifact(s) into the bag.
//!
//! Commands that do not match a known mapping fall through to a generic
//! workpaper generator, ensuring every step produces at least one artifact.

use tracing::warn;

use datasynth_core::models::audit::WorkpaperSection;
use datasynth_generators::audit::{
    analytical_procedure_generator::AnalyticalProcedureGenerator,
    audit_opinion_generator::{AuditOpinionGenerator, AuditOpinionInput},
    confirmation_generator::ConfirmationGenerator,
    cra_generator::CraGenerator,
    engagement_letter_generator::EngagementLetterGenerator,
    going_concern_generator::GoingConcernGenerator,
    materiality_generator::{MaterialityGenerator, MaterialityInput},
    sampling_plan_generator::SamplingPlanGenerator,
    subsequent_event_generator::SubsequentEventGenerator,
    AuditEngagementGenerator, EvidenceGenerator, FindingGenerator, RiskAssessmentGenerator,
    WorkpaperGenerator,
};

use crate::artifact::ArtifactBag;
use crate::context::EngagementContext;
use crate::schema::BlueprintStep;

// ---------------------------------------------------------------------------
// StepDispatcher
// ---------------------------------------------------------------------------

/// Maps FSM step commands to concrete audit generators and accumulates the
/// resulting artifacts in an [`ArtifactBag`].
///
/// Each generator is pre-initialised with a deterministic seed derived from
/// a base seed plus a 7000-series discriminator offset.
pub struct StepDispatcher {
    engagement_gen: AuditEngagementGenerator,
    letter_gen: EngagementLetterGenerator,
    materiality_gen: MaterialityGenerator,
    risk_gen: RiskAssessmentGenerator,
    cra_gen: CraGenerator,
    workpaper_gen: WorkpaperGenerator,
    evidence_gen: EvidenceGenerator,
    finding_gen: FindingGenerator,
    sampling_gen: SamplingPlanGenerator,
    analytical_gen: AnalyticalProcedureGenerator,
    gc_gen: GoingConcernGenerator,
    se_gen: SubsequentEventGenerator,
    opinion_gen: AuditOpinionGenerator,
    confirmation_gen: ConfirmationGenerator,
}

impl StepDispatcher {
    /// Create a new dispatcher, initialising each generator with a
    /// discriminated seed derived from `base_seed`.
    pub fn new(base_seed: u64) -> Self {
        Self {
            engagement_gen: AuditEngagementGenerator::new(base_seed + 7000),
            letter_gen: EngagementLetterGenerator::new(base_seed + 7100),
            materiality_gen: MaterialityGenerator::new(base_seed + 7200),
            risk_gen: RiskAssessmentGenerator::new(base_seed + 7300),
            cra_gen: CraGenerator::new(base_seed + 7400),
            workpaper_gen: WorkpaperGenerator::new(base_seed + 7500),
            evidence_gen: EvidenceGenerator::new(base_seed + 7600),
            finding_gen: FindingGenerator::new(base_seed + 7700),
            sampling_gen: SamplingPlanGenerator::new(base_seed + 7800),
            analytical_gen: AnalyticalProcedureGenerator::new(base_seed + 7900),
            gc_gen: GoingConcernGenerator::new(base_seed + 8000),
            se_gen: SubsequentEventGenerator::new(base_seed + 8100),
            opinion_gen: AuditOpinionGenerator::new(base_seed + 8200),
            confirmation_gen: ConfirmationGenerator::new(base_seed + 8300),
        }
    }

    /// Dispatch a step command to the appropriate generator.
    ///
    /// The `procedure_id` is used for context in generic workpaper titles.
    /// Unknown commands are routed to a generic workpaper generator so that
    /// every step produces at least one artifact.
    pub fn dispatch(
        &mut self,
        step: &BlueprintStep,
        procedure_id: &str,
        context: &EngagementContext,
        bag: &mut ArtifactBag,
    ) {
        let cmd = step.command.as_deref().unwrap_or("");
        match cmd {
            "evaluate_client_acceptance" => self.dispatch_engagement(context, bag),
            "agree_engagement_terms" => self.dispatch_engagement_letter(context, bag),
            "determine_overall_materiality" => self.dispatch_materiality(context, bag),
            "identify_risks" => self.dispatch_risk_assessment(context, bag),
            "assess_risks" => self.dispatch_cra(context, bag),
            "design_controls_tests" | "design_substantive_procedures"
            | "design_analytical_procedures" => {
                self.dispatch_workpaper(step, procedure_id, context, bag);
            }
            "perform_controls_tests" | "perform_tests_of_details" => {
                self.dispatch_sampling(context, bag);
            }
            "perform_analytical_procedures" => {
                self.dispatch_analytical_procedures(context, bag);
            }
            "send_confirmations" => {
                self.dispatch_confirmations(context, bag);
            }
            "evaluate_management_assessment" | "determine_going_concern_doubt" => {
                self.dispatch_going_concern(context, bag);
            }
            "perform_subsequent_events_review" => {
                self.dispatch_subsequent_events(context, bag);
            }
            "evaluate_findings" | "evaluate_misstatements" => {
                self.dispatch_findings(context, bag);
            }
            "form_audit_opinion" => {
                self.dispatch_opinion(context, bag);
            }
            _ => {
                self.dispatch_generic_workpaper(step, procedure_id, context, bag);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Individual dispatch methods
    // -----------------------------------------------------------------------

    /// Generate an `AuditEngagement` from context and push it into the bag.
    fn dispatch_engagement(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let engagement = self.engagement_gen.generate_engagement(
            &ctx.company_code,
            &ctx.company_name,
            ctx.fiscal_year as u16,
            ctx.report_date,
            ctx.total_revenue,
            None,
        );
        bag.engagements.push(engagement);
    }

    /// Generate an `EngagementLetter` (ISA 210). Requires an engagement in
    /// the bag; skips with a warning if none is available.
    fn dispatch_engagement_letter(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => {
                warn!("dispatch_engagement_letter: no engagement in bag — skipping");
                return;
            }
        };
        let entity_count = ctx.entity_codes.len().max(1);
        let letter = self.letter_gen.generate(
            &engagement.engagement_id.to_string(),
            &ctx.company_name,
            entity_count,
            ctx.report_date,
            &ctx.currency,
            "IFRS", // default framework; overlay could refine this
            ctx.engagement_start,
        );
        bag.engagement_letters.push(letter);
    }

    /// Generate a `MaterialityCalculation` (ISA 320) from context financials.
    fn dispatch_materiality(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let input = MaterialityInput {
            entity_code: ctx.company_code.clone(),
            period: format!("FY{}", ctx.fiscal_year),
            revenue: ctx.total_revenue,
            pretax_income: ctx.pretax_income,
            total_assets: ctx.total_assets,
            equity: ctx.equity,
            gross_profit: ctx.gross_profit,
        };
        let calc = self.materiality_gen.generate(&input);
        bag.materiality_calculations.push(calc);
    }

    /// Generate `RiskAssessment` records (ISA 315/330). Requires an
    /// engagement in the bag.
    fn dispatch_risk_assessment(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => {
                warn!("dispatch_risk_assessment: no engagement in bag — skipping");
                return;
            }
        };
        let risks = self.risk_gen.generate_risks_for_engagement(
            engagement,
            &ctx.team_member_ids,
            &ctx.accounts,
        );
        bag.risk_assessments.extend(risks);
    }

    /// Generate `CombinedRiskAssessment` records (ISA 315) for the entity.
    fn dispatch_cra(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let cras = self.cra_gen.generate_for_entity(&ctx.company_code, None);
        bag.combined_risk_assessments.extend(cras);
    }

    /// Generate a workpaper (ISA 230) for a design/planning step.
    fn dispatch_workpaper(
        &mut self,
        step: &BlueprintStep,
        _procedure_id: &str,
        ctx: &EngagementContext,
        bag: &mut ArtifactBag,
    ) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => {
                warn!("dispatch_workpaper: no engagement in bag — skipping");
                return;
            }
        };

        let section = section_for_command(step.command.as_deref().unwrap_or(""));
        let wp = self.workpaper_gen.generate_workpaper(
            engagement,
            section,
            ctx.engagement_start,
            &ctx.team_member_ids,
        );

        // Also generate evidence for this workpaper.
        let evidence = self.evidence_gen.generate_evidence_for_workpaper(
            &wp,
            &ctx.team_member_ids,
            ctx.engagement_start,
        );

        bag.evidence.extend(evidence);
        bag.workpapers.push(wp);
    }

    /// Generate `SamplingPlan` + `SampledItem` records (ISA 530) from CRAs
    /// already in the bag.
    fn dispatch_sampling(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        if bag.combined_risk_assessments.is_empty() {
            warn!("dispatch_sampling: no CRAs in bag — generating CRAs first");
            self.dispatch_cra(ctx, bag);
        }

        // Use performance materiality from the first materiality calc, if available.
        let tolerable_error = bag
            .materiality_calculations
            .first()
            .map(|m| m.performance_materiality);

        let (plans, items) = self
            .sampling_gen
            .generate_for_cras(&bag.combined_risk_assessments, tolerable_error);
        bag.sampling_plans.extend(plans);
        bag.sampled_items.extend(items);
    }

    /// Generate `AnalyticalProcedureResult` records (ISA 520).
    fn dispatch_analytical_procedures(
        &mut self,
        ctx: &EngagementContext,
        bag: &mut ArtifactBag,
    ) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => {
                warn!("dispatch_analytical_procedures: no engagement in bag — skipping");
                return;
            }
        };
        let results = self
            .analytical_gen
            .generate_procedures(engagement, &ctx.accounts);
        bag.analytical_results.extend(results);
    }

    /// Generate `ExternalConfirmation` + `ConfirmationResponse` records
    /// (ISA 505).
    fn dispatch_confirmations(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => {
                warn!("dispatch_confirmations: no engagement in bag — skipping");
                return;
            }
        };
        let (confirmations, responses) = self.confirmation_gen.generate_confirmations(
            engagement,
            &bag.workpapers,
            &ctx.accounts,
        );
        bag.confirmations.extend(confirmations);
        bag.confirmation_responses.extend(responses);
    }

    /// Generate a `GoingConcernAssessment` (ISA 570).
    fn dispatch_going_concern(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let period = format!("FY{}", ctx.fiscal_year);
        let assessment =
            self.gc_gen
                .generate_for_entity(&ctx.company_code, ctx.report_date, &period);
        bag.going_concern_assessments.push(assessment);
    }

    /// Generate `SubsequentEvent` records (ISA 560).
    fn dispatch_subsequent_events(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let events = self
            .se_gen
            .generate_for_entity(&ctx.company_code, ctx.report_date);
        bag.subsequent_events.extend(events);
    }

    /// Generate `AuditFinding` records (ISA 265). Requires an engagement
    /// and workpapers in the bag.
    fn dispatch_findings(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => {
                warn!("dispatch_findings: no engagement in bag — skipping");
                return;
            }
        };
        let findings = self.finding_gen.generate_findings_for_engagement(
            engagement,
            &bag.workpapers,
            &ctx.team_member_ids,
        );
        bag.findings.extend(findings);
    }

    /// Generate an `AuditOpinion` + `KeyAuditMatter` records
    /// (ISA 700/701/705/706). Reads findings, going concern, etc. from the bag.
    fn dispatch_opinion(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => {
                warn!("dispatch_opinion: no engagement in bag — skipping");
                return;
            }
        };

        let input = AuditOpinionInput {
            entity_code: ctx.company_code.clone(),
            entity_name: ctx.company_name.clone(),
            engagement_id: engagement.engagement_id,
            period_end: ctx.report_date,
            findings: bag.findings.clone(),
            going_concern: bag.going_concern_assessments.last().cloned(),
            component_reports: Vec::new(),
            is_us_listed: ctx.is_us_listed,
            auditor_name: "DataSynth Audit LLP".to_string(),
            engagement_partner: engagement.engagement_partner_name.clone(),
        };

        let generated = self.opinion_gen.generate(&input);
        bag.audit_opinions.push(generated.opinion);
        bag.key_audit_matters.extend(generated.key_audit_matters);
    }

    /// Fallback: generate a generic workpaper for any unrecognised command.
    fn dispatch_generic_workpaper(
        &mut self,
        step: &BlueprintStep,
        _procedure_id: &str,
        ctx: &EngagementContext,
        bag: &mut ArtifactBag,
    ) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => {
                // Without an engagement we cannot generate a workpaper.
                // This is normal early in the FSM before evaluate_client_acceptance.
                return;
            }
        };

        let section = section_for_command(step.command.as_deref().unwrap_or(""));
        let wp = self.workpaper_gen.generate_workpaper(
            engagement,
            section,
            ctx.engagement_start,
            &ctx.team_member_ids,
        );
        bag.workpapers.push(wp);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Map a step command string to the most appropriate `WorkpaperSection`.
fn section_for_command(cmd: &str) -> WorkpaperSection {
    if cmd.contains("planning") || cmd.contains("materiality") || cmd.contains("acceptance") {
        WorkpaperSection::Planning
    } else if cmd.contains("risk") || cmd.contains("assess") {
        WorkpaperSection::RiskAssessment
    } else if cmd.contains("control") {
        WorkpaperSection::ControlTesting
    } else if cmd.contains("substantive")
        || cmd.contains("detail")
        || cmd.contains("analytical")
        || cmd.contains("confirm")
        || cmd.contains("sampling")
    {
        WorkpaperSection::SubstantiveTesting
    } else if cmd.contains("opinion")
        || cmd.contains("going_concern")
        || cmd.contains("subsequent")
        || cmd.contains("conclusion")
        || cmd.contains("finding")
    {
        WorkpaperSection::Completion
    } else if cmd.contains("report") || cmd.contains("issue") {
        WorkpaperSection::Reporting
    } else {
        WorkpaperSection::Planning
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Build a minimal `BlueprintStep` with the given command.
    fn step_with_command(id: &str, cmd: &str) -> BlueprintStep {
        BlueprintStep {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            actor: None,
            command: Some(cmd.to_string()),
            emits: None,
            binding: crate::schema::BindingLevel::default(),
            guards: Vec::new(),
            evidence: Vec::new(),
            standards: Vec::new(),
            decision: None,
        }
    }

    #[test]
    fn test_dispatch_engagement() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        let step = step_with_command("s1", "evaluate_client_acceptance");
        dispatcher.dispatch(&step, "client_acceptance", &ctx, &mut bag);

        assert_eq!(bag.engagements.len(), 1);
        assert_eq!(bag.engagements[0].client_entity_id, "TEST01");
    }

    #[test]
    fn test_dispatch_materiality() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        let step = step_with_command("s1", "determine_overall_materiality");
        dispatcher.dispatch(&step, "planning_materiality", &ctx, &mut bag);

        assert_eq!(bag.materiality_calculations.len(), 1);
        assert_eq!(bag.materiality_calculations[0].entity_code, "TEST01");
    }

    #[test]
    fn test_dispatch_engagement_letter_requires_engagement() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Without an engagement, dispatch should skip.
        let step = step_with_command("s1", "agree_engagement_terms");
        dispatcher.dispatch(&step, "engagement_terms", &ctx, &mut bag);
        assert!(bag.engagement_letters.is_empty());

        // Generate an engagement first, then retry.
        let eng_step = step_with_command("s0", "evaluate_client_acceptance");
        dispatcher.dispatch(&eng_step, "client_acceptance", &ctx, &mut bag);
        dispatcher.dispatch(&step, "engagement_terms", &ctx, &mut bag);
        assert_eq!(bag.engagement_letters.len(), 1);
    }

    #[test]
    fn test_dispatch_risk_assessment() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Create engagement first.
        let eng_step = step_with_command("s0", "evaluate_client_acceptance");
        dispatcher.dispatch(&eng_step, "client_acceptance", &ctx, &mut bag);

        let step = step_with_command("s1", "identify_risks");
        dispatcher.dispatch(&step, "risk_assessment", &ctx, &mut bag);

        assert!(!bag.risk_assessments.is_empty());
        // At least the two ISA 240 presumed risks.
        assert!(bag.risk_assessments.len() >= 2);
    }

    #[test]
    fn test_dispatch_cra() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        let step = step_with_command("s1", "assess_risks");
        dispatcher.dispatch(&step, "risk_assessment_cra", &ctx, &mut bag);

        assert!(!bag.combined_risk_assessments.is_empty());
    }

    #[test]
    fn test_dispatch_going_concern() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        let step = step_with_command("s1", "determine_going_concern_doubt");
        dispatcher.dispatch(&step, "going_concern", &ctx, &mut bag);

        assert_eq!(bag.going_concern_assessments.len(), 1);
    }

    #[test]
    fn test_dispatch_subsequent_events() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        let step = step_with_command("s1", "perform_subsequent_events_review");
        dispatcher.dispatch(&step, "subsequent_events", &ctx, &mut bag);

        // May be 0 events (generator randomly decides count 0..=5), but
        // the dispatch itself should not panic.
        assert!(bag.subsequent_events.len() <= 5);
    }

    #[test]
    fn test_dispatch_unknown_command_creates_workpaper() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // First create an engagement so the generic workpaper can use it.
        let eng_step = step_with_command("s0", "evaluate_client_acceptance");
        dispatcher.dispatch(&eng_step, "client_acceptance", &ctx, &mut bag);

        let step = step_with_command("s1", "some_unknown_command");
        dispatcher.dispatch(&step, "misc_procedure", &ctx, &mut bag);

        assert!(!bag.workpapers.is_empty());
    }

    #[test]
    fn test_dispatch_opinion() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Build up prerequisite artifacts.
        let steps = [
            ("s0", "evaluate_client_acceptance"),
            ("s1", "determine_overall_materiality"),
            ("s2", "identify_risks"),
            ("s3", "assess_risks"),
            ("s4", "determine_going_concern_doubt"),
            ("s5", "evaluate_findings"),
            ("s6", "form_audit_opinion"),
        ];

        for (id, cmd) in &steps {
            let step = step_with_command(id, cmd);
            dispatcher.dispatch(&step, "test_proc", &ctx, &mut bag);
        }

        assert_eq!(bag.audit_opinions.len(), 1);
        // KAMs may or may not be present depending on the random draw.
    }

    #[test]
    fn test_dispatch_sampling() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // dispatch_sampling will auto-generate CRAs if none present.
        let step = step_with_command("s1", "perform_tests_of_details");
        dispatcher.dispatch(&step, "substantive", &ctx, &mut bag);

        // CRAs should have been created as a prerequisite.
        assert!(!bag.combined_risk_assessments.is_empty());
        // Sampling plans may be empty if all CRAs are below Moderate,
        // but the dispatch should not panic.
    }

    #[test]
    fn test_dispatch_analytical_procedures() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Need an engagement for the analytical procedure generator.
        let eng_step = step_with_command("s0", "evaluate_client_acceptance");
        dispatcher.dispatch(&eng_step, "client_acceptance", &ctx, &mut bag);

        let step = step_with_command("s1", "perform_analytical_procedures");
        dispatcher.dispatch(&step, "analytical", &ctx, &mut bag);

        assert!(!bag.analytical_results.is_empty());
    }

    #[test]
    fn test_dispatch_confirmations() {
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Need an engagement.
        let eng_step = step_with_command("s0", "evaluate_client_acceptance");
        dispatcher.dispatch(&eng_step, "client_acceptance", &ctx, &mut bag);

        let step = step_with_command("s1", "send_confirmations");
        dispatcher.dispatch(&step, "confirmations", &ctx, &mut bag);

        assert!(!bag.confirmations.is_empty());
    }
}
