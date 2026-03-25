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
    AuditEngagementGenerator, AvailableControl, AvailableRisk, EvidenceGenerator, FindingGenerator,
    JudgmentGenerator, RiskAssessmentGenerator, WorkpaperGenerator,
};

use crate::artifact::ArtifactBag;
use crate::content::{
    ContentGenerator, FindingContext, TemplateContentGenerator, WorkpaperContext,
};
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
    judgment_gen: JudgmentGenerator,
    sampling_gen: SamplingPlanGenerator,
    analytical_gen: AnalyticalProcedureGenerator,
    gc_gen: GoingConcernGenerator,
    se_gen: SubsequentEventGenerator,
    opinion_gen: AuditOpinionGenerator,
    confirmation_gen: ConfirmationGenerator,
    content_gen: Box<dyn ContentGenerator>,
}

impl StepDispatcher {
    /// Create a new dispatcher, initialising each generator with a
    /// discriminated seed derived from `base_seed`.
    ///
    /// Uses the default [`TemplateContentGenerator`] for narrative generation.
    pub fn new(base_seed: u64) -> Self {
        Self::new_with_content(base_seed, Box::new(TemplateContentGenerator))
    }

    /// Create a dispatcher with a custom [`ContentGenerator`] for narrative
    /// enrichment of findings and workpapers.
    pub fn new_with_content(base_seed: u64, content_gen: Box<dyn ContentGenerator>) -> Self {
        Self {
            engagement_gen: AuditEngagementGenerator::new(base_seed + 7000),
            letter_gen: EngagementLetterGenerator::new(base_seed + 7100),
            materiality_gen: MaterialityGenerator::new(base_seed + 7200),
            risk_gen: RiskAssessmentGenerator::new(base_seed + 7300),
            cra_gen: CraGenerator::new(base_seed + 7400),
            workpaper_gen: WorkpaperGenerator::new(base_seed + 7500),
            evidence_gen: EvidenceGenerator::new(base_seed + 7600),
            finding_gen: FindingGenerator::new(base_seed + 7700),
            judgment_gen: JudgmentGenerator::new(base_seed + 8400),
            sampling_gen: SamplingPlanGenerator::new(base_seed + 7800),
            analytical_gen: AnalyticalProcedureGenerator::new(base_seed + 7900),
            gc_gen: GoingConcernGenerator::new(base_seed + 8000),
            se_gen: SubsequentEventGenerator::new(base_seed + 8100),
            opinion_gen: AuditOpinionGenerator::new(base_seed + 8200),
            confirmation_gen: ConfirmationGenerator::new(base_seed + 8300),
            content_gen,
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

        // Auto-bootstrap engagement for IA blueprints (which lack
        // `evaluate_client_acceptance`) — create the engagement record
        // the first time a substantive command runs.  Commands that
        // already dispatch to `dispatch_engagement` are excluded so we
        // don't double-create.
        if bag.engagements.is_empty()
            && !matches!(
                cmd,
                "" | "start_universe_update"
                    | "submit_universe"
                    | "approve_universe"
                    | "activate"
                    | "submit_for_review"
                    | "cycle_back"
                    | "complete_cycle"
                    | "evaluate_client_acceptance"
                    | "conduct_opening_meeting"
                    | "conduct_meeting"
                    | "define_engagement_scope"
            )
        {
            self.dispatch_engagement(context, bag);
        }

        match cmd {
            // ----- Engagement generation (FSA + IA) -----
            "evaluate_client_acceptance"
            | "conduct_opening_meeting"
            | "conduct_meeting"
            | "define_engagement_scope" => self.dispatch_engagement(context, bag),

            // ----- Engagement letter / charter (IA uses charter) -----
            "agree_engagement_terms" | "draft_ia_charter" | "finalize_mandate" => {
                self.dispatch_engagement_letter(context, bag);
            }

            // ----- Materiality -----
            "determine_overall_materiality" => self.dispatch_materiality(context, bag),

            // ----- Risk assessment (ISA 315/330) -----
            "identify_engagement_risks"
            | "assess_engagement_risks"
            | "assess_universe_risks"
            | "identify_risks" => {
                self.dispatch_risk_assessment(context, bag);
            }

            // ----- Combined risk assessment -----
            "assess_risks" | "assess_control_design" | "evaluate_control_effectiveness" => {
                self.dispatch_cra(context, bag);
            }

            // ----- Workpaper generation (design steps) -----
            "design_test_procedures"
            | "design_work_program"
            | "design_controls_tests"
            | "design_substantive_procedures"
            | "design_analytical_procedures" => {
                self.dispatch_workpaper(step, procedure_id, context, bag);
            }

            // ----- Sampling / test execution -----
            "perform_test_procedures"
            | "perform_controls_tests"
            | "perform_tests_of_details"
            | "review_test_results"
            | "request_additional_testing" => {
                self.dispatch_sampling(context, bag);
            }

            // ----- Analytical procedures -----
            "perform_analytical_procedures" => {
                self.dispatch_analytical_procedures(context, bag);
            }

            // ----- Confirmations -----
            "send_confirmations" => {
                self.dispatch_confirmations(context, bag);
            }

            // ----- Going concern -----
            "evaluate_management_assessment" | "determine_going_concern_doubt" => {
                self.dispatch_going_concern(context, bag);
            }

            // ----- Subsequent events -----
            "perform_subsequent_events_review" => {
                self.dispatch_subsequent_events(context, bag);
            }

            // ----- Finding development (C2CE model) -----
            "identify_condition"
            | "map_criteria"
            | "analyze_cause"
            | "assess_effect"
            | "draft_finding"
            | "close_finding"
            | "obtain_response"
            | "identify_finding_condition"
            | "map_finding_criteria"
            | "analyze_cause_and_effect"
            | "evaluate_finding_significance"
            | "evaluate_findings"
            | "evaluate_misstatements"
            | "discuss_findings"
            | "discuss_preliminary_findings"
            | "agree_action_plans"
            | "develop_recommendations" => {
                self.dispatch_findings(step, procedure_id, context, bag);
            }

            // ----- Opinion / reporting -----
            "form_audit_opinion"
            | "develop_engagement_conclusions"
            | "finalize_audit_report"
            | "issue_report" => {
                self.dispatch_opinion(context, bag);
            }

            // ----- Judgment / engagement quality / approvals (ISA 220) -----
            "review_engagement_quality"
            | "exercise_skepticism"
            | "apply_due_care"
            | "supervise_engagement_quality"
            | "conduct_periodic_assessment"
            | "oversee_qaip"
            | "evaluate_staff_performance"
            | "commission_external_qa"
            | "confirm_cae_qualifications"
            | "document_risk_acceptance"
            | "disclose_errors_omissions"
            | "approve_charter"
            | "approve_control_evaluation"
            | "approve_draft_report"
            | "approve_plan"
            | "approve_procedure"
            | "approve_risk_assessment"
            | "approve_scope"
            | "approve_testing"
            | "approve_work_program" => {
                self.dispatch_judgment(context, bag);
            }

            // ----- Documentation / evidence / submissions -----
            "document_engagement_work"
            | "archive_engagement_documentation"
            | "protect_information"
            | "establish_confidentiality_policies"
            | "submit_charter"
            | "submit_control_evaluation"
            | "submit_draft_report"
            | "submit_final_report"
            | "submit_plan"
            | "submit_risk_assessment"
            | "submit_scope"
            | "submit_testing" => {
                self.dispatch_evidence(context, bag);
            }

            // ----- Planning / scoping / initialization (produces Planning workpapers) -----
            "define_engagement_scope_detail"
            | "determine_engagement_timeline"
            | "draft_annual_plan"
            | "develop_work_program"
            | "scope_engagement"
            | "develop_staffing_plan"
            | "develop_ia_budget"
            | "assign_team_members"
            | "determine_sme_needs"
            | "confirm_resource_competencies"
            | "identify_auditable_entities"
            | "prioritize_audit_entities"
            | "identify_stakeholders"
            | "recruit_and_develop_staff"
            | "start_allocation"
            | "start_control_evaluation"
            | "start_draft_report"
            | "start_final_report"
            | "start_mandate_development"
            | "start_monitoring"
            | "start_plan_development"
            | "start_procedure"
            | "start_risk_assessment"
            | "start_scoping"
            | "start_testing"
            | "start_verification"
            | "finalize_allocation"
            | "finalize_plan"
            | "revise_charter"
            | "revise_draft_report"
            | "revise_plan" => {
                self.dispatch_workpaper_section(
                    step,
                    procedure_id,
                    context,
                    bag,
                    WorkpaperSection::Planning,
                );
            }

            // ----- Reporting -----
            "draft_audit_report"
            | "review_draft_report"
            | "prepare_draft_report"
            | "evaluate_management_responses"
            | "send_report_for_response"
            | "receive_response"
            | "request_response"
            | "distribute_final_report"
            | "communicate_approved_plan"
            | "present_plan_to_board"
            | "present_resource_requirements"
            | "communicate_plan_and_results" => {
                self.dispatch_workpaper_section(
                    step,
                    procedure_id,
                    context,
                    bag,
                    WorkpaperSection::Reporting,
                );
            }

            // ----- Follow-up / monitoring (produces findings) -----
            "track_action_plan_status"
            | "escalate_overdue_actions"
            | "report_follow_up_status"
            | "verify_remediation_implementation"
            | "conclude_on_remediation"
            | "close_monitoring"
            | "initiate_follow_up"
            | "re_verify_action"
            | "complete_verification" => {
                self.dispatch_findings(step, procedure_id, context, bag);
            }

            // ----- Ethics / governance -----
            "establish_ethics_code"
            | "conduct_ethics_training"
            | "monitor_ethics_compliance"
            | "assess_objectivity_threats"
            | "implement_objectivity_safeguards"
            | "disclose_impairments"
            | "assess_competencies"
            | "develop_cpd_plans"
            | "verify_standards_conformance"
            | "maintain_ethical_standards"
            | "safeguard_objectivity"
            | "establish_independence"
            | "define_ia_mandate"
            | "establish_board_interaction"
            | "obtain_board_support"
            | "assess_technology_capabilities"
            | "implement_technology"
            | "manage_technological_resources" => {
                self.dispatch_workpaper_section(
                    step,
                    procedure_id,
                    context,
                    bag,
                    WorkpaperSection::Planning,
                );
            }

            // ----- Performance / monitoring -----
            "define_performance_metrics"
            | "track_performance"
            | "report_performance_to_board"
            | "measure_ia_performance"
            | "perform_ongoing_monitoring"
            | "monitor_budget_utilization" => {
                self.dispatch_workpaper_section(
                    step,
                    procedure_id,
                    context,
                    bag,
                    WorkpaperSection::Completion,
                );
            }

            // ----- Everything else: generic workpaper fallback -----
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
        procedure_id: &str,
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
        let mut wp = self.workpaper_gen.generate_workpaper(
            engagement,
            section,
            ctx.engagement_start,
            &ctx.team_member_ids,
        );

        // Enrich workpaper objective with content generator narrative.
        let standards_refs: Vec<String> = step.standards.iter().map(|s| s.ref_id.clone()).collect();
        let actor = step.actor.as_deref().unwrap_or("audit_team");
        wp.objective = self
            .content_gen
            .generate_workpaper_narrative(&WorkpaperContext {
                procedure_id: procedure_id.to_string(),
                section: format!("{:?}", section),
                actor: actor.to_string(),
                standards_refs: if standards_refs.is_empty() {
                    vec!["ISA 230".to_string()]
                } else {
                    standards_refs
                },
            });

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
    fn dispatch_analytical_procedures(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
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

    /// Generate a `ProfessionalJudgment` record (ISA 200). Requires an
    /// engagement in the bag.
    fn dispatch_judgment(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => return,
        };
        let judgment = self
            .judgment_gen
            .generate_judgment(engagement, &ctx.team_member_ids);
        bag.judgments.push(judgment);
    }

    /// Generate `AuditEvidence` records for the most recent workpaper in the
    /// bag (ISA 500).
    fn dispatch_evidence(&mut self, ctx: &EngagementContext, bag: &mut ArtifactBag) {
        if let Some(wp) = bag.workpapers.last() {
            let evidence = self.evidence_gen.generate_evidence_for_workpaper(
                wp,
                &ctx.team_member_ids,
                ctx.engagement_start,
            );
            bag.evidence.extend(evidence);
        }
    }

    /// Generate a workpaper in a specific section (ISA 230). Requires an
    /// engagement in the bag.
    fn dispatch_workpaper_section(
        &mut self,
        step: &BlueprintStep,
        procedure_id: &str,
        ctx: &EngagementContext,
        bag: &mut ArtifactBag,
        section: WorkpaperSection,
    ) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => return,
        };
        let mut wp = self.workpaper_gen.generate_workpaper(
            engagement,
            section,
            ctx.engagement_start,
            &ctx.team_member_ids,
        );

        // Enrich workpaper objective with content generator narrative.
        let standards_refs: Vec<String> = step.standards.iter().map(|s| s.ref_id.clone()).collect();
        let actor = step.actor.as_deref().unwrap_or("audit_team");
        wp.objective = self
            .content_gen
            .generate_workpaper_narrative(&WorkpaperContext {
                procedure_id: procedure_id.to_string(),
                section: format!("{:?}", section),
                actor: actor.to_string(),
                standards_refs: if standards_refs.is_empty() {
                    vec!["ISA 230".to_string()]
                } else {
                    standards_refs
                },
            });

        bag.workpapers.push(wp);
    }

    /// Generate `AuditFinding` records (ISA 265). Requires an engagement
    /// and workpapers in the bag.
    ///
    /// When the engagement context provides control IDs or anomaly
    /// references, findings are linked to controls and enriched with
    /// anomaly cross-references.  The active [`ContentGenerator`] is used
    /// to produce finding narratives.
    fn dispatch_findings(
        &mut self,
        step: &BlueprintStep,
        procedure_id: &str,
        ctx: &EngagementContext,
        bag: &mut ArtifactBag,
    ) {
        let engagement = match bag.engagements.last() {
            Some(e) => e,
            None => {
                warn!("dispatch_findings: no engagement in bag — skipping");
                return;
            }
        };

        // Build AvailableControl list from context control_ids.
        let controls: Vec<AvailableControl> = ctx
            .control_ids
            .iter()
            .map(|id| AvailableControl {
                control_id: id.clone(),
                assertions: Vec::new(),
                process_areas: Vec::new(),
            })
            .collect();

        // Build AvailableRisk list from risk assessments already in the bag.
        let risks: Vec<AvailableRisk> = bag
            .risk_assessments
            .iter()
            .map(|r| AvailableRisk {
                risk_id: r.risk_id.to_string(),
                engagement_id: r.engagement_id,
                account_or_process: r.account_or_process.clone(),
            })
            .collect();

        // Use context-aware generation when controls or risks are available.
        let mut findings = if controls.is_empty() && risks.is_empty() {
            self.finding_gen.generate_findings_for_engagement(
                engagement,
                &bag.workpapers,
                &ctx.team_member_ids,
            )
        } else {
            self.finding_gen.generate_findings_with_context(
                engagement,
                &bag.workpapers,
                &ctx.team_member_ids,
                &controls,
                &risks,
            )
        };

        // Enrich findings with anomaly cross-references.
        if !ctx.anomaly_refs.is_empty() {
            for (i, finding) in findings.iter_mut().enumerate() {
                // Round-robin assign anomaly refs so each finding links to at least one.
                let anomaly_ref = &ctx.anomaly_refs[i % ctx.anomaly_refs.len()];
                if !finding.condition.is_empty() {
                    finding.condition =
                        format!("{} [Linked anomaly: {}]", finding.condition, anomaly_ref);
                }
            }
        }

        // Enrich findings with journal entry evidence references.
        if !ctx.journal_entry_ids.is_empty() {
            for (i, finding) in findings.iter_mut().enumerate() {
                let je_ref = &ctx.journal_entry_ids[i % ctx.journal_entry_ids.len()];
                if !finding.effect.is_empty() {
                    finding.effect = format!("{} [Supporting JE: {}]", finding.effect, je_ref);
                }
            }
        }

        // Standards refs from the step (for narrative context).
        let standards_refs: Vec<String> = step.standards.iter().map(|s| s.ref_id.clone()).collect();

        // Use content generator to enrich finding narratives.
        for finding in &mut findings {
            let narrative = self
                .content_gen
                .generate_finding_narrative(&FindingContext {
                    procedure_id: procedure_id.to_string(),
                    step_id: step.id.clone(),
                    standards_refs: if standards_refs.is_empty() {
                        vec![finding.finding_type.isa_reference().to_string()]
                    } else {
                        standards_refs.clone()
                    },
                    finding_type: format!("{:?}", finding.finding_type),
                    condition: finding.condition.clone(),
                    criteria: finding.criteria.clone(),
                });
            finding.condition = narrative;
        }

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
#[allow(clippy::if_same_then_else)]
fn section_for_command(cmd: &str) -> WorkpaperSection {
    // --- Completion: findings / C2CE / monitoring / follow-up ---
    if cmd.contains("finding")
        || cmd.contains("condition")
        || cmd.contains("criteria")
        || cmd.contains("cause")
        || cmd.contains("effect")
        || cmd.contains("recommendation")
        || cmd.contains("monitoring")
        || cmd.contains("follow_up")
        || cmd.contains("remediation")
        || cmd.contains("verification")
        || cmd.contains("opinion")
        || cmd.contains("going_concern")
        || cmd.contains("subsequent")
        || cmd.contains("conclusion")
    {
        WorkpaperSection::Completion
    // --- Reporting ---
    } else if cmd.contains("report")
        || cmd.contains("draft")
        || cmd.contains("issue")
        || cmd.contains("distribute")
        || cmd.contains("communicate")
    {
        WorkpaperSection::Reporting
    // --- Substantive testing ---
    } else if cmd.contains("test")
        || cmd.contains("execute")
        || cmd.contains("perform")
        || cmd.contains("substantive")
        || cmd.contains("detail")
        || cmd.contains("analytical")
        || cmd.contains("confirm")
        || cmd.contains("sampling")
    {
        WorkpaperSection::SubstantiveTesting
    // --- Control testing ---
    } else if cmd.contains("control") {
        WorkpaperSection::ControlTesting
    // --- Risk assessment ---
    } else if cmd.contains("risk") || cmd.contains("assess") {
        WorkpaperSection::RiskAssessment
    // --- Planning (IA governance, admin, scope, etc.) ---
    } else if cmd.contains("planning")
        || cmd.contains("materiality")
        || cmd.contains("acceptance")
        || cmd.contains("universe")
        || cmd.contains("plan")
        || cmd.contains("scope")
        || cmd.contains("timeline")
        || cmd.contains("allocation")
        || cmd.contains("ethics")
        || cmd.contains("independence")
        || cmd.contains("objectivity")
        || cmd.contains("competency")
        || cmd.contains("confidentiality")
        || cmd.contains("budget")
        || cmd.contains("staffing")
        || cmd.contains("technology")
        || cmd.contains("quality")
    {
        WorkpaperSection::Planning
    } else {
        // Unrecognised command — default to Planning.
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

        // Generate an engagement first, then dispatch the letter.
        let eng_step = step_with_command("s0", "evaluate_client_acceptance");
        dispatcher.dispatch(&eng_step, "client_acceptance", &ctx, &mut bag);

        let step = step_with_command("s1", "agree_engagement_terms");
        dispatcher.dispatch(&step, "engagement_terms", &ctx, &mut bag);
        assert_eq!(bag.engagement_letters.len(), 1);
    }

    #[test]
    fn test_dispatch_engagement_letter_auto_bootstrap() {
        // The auto-bootstrap creates an engagement when the bag is empty
        // and a substantive command runs. `agree_engagement_terms` is
        // substantive, so it should auto-create an engagement and then
        // successfully produce a letter.
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        let step = step_with_command("s1", "agree_engagement_terms");
        dispatcher.dispatch(&step, "engagement_terms", &ctx, &mut bag);

        assert_eq!(
            bag.engagements.len(),
            1,
            "auto-bootstrap should create an engagement"
        );
        assert_eq!(
            bag.engagement_letters.len(),
            1,
            "letter should be generated after bootstrap"
        );
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

    // -------------------------------------------------------------------
    // IA-specific dispatch tests
    // -------------------------------------------------------------------

    #[test]
    fn test_ia_dispatch_produces_artifacts() {
        // Simulate an IA engagement by running commands from the IA
        // blueprint in a representative order. The auto-bootstrap should
        // create the engagement, and subsequent commands should produce
        // typed artifacts (not just generic workpapers).
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // IA engagement setup (no evaluate_client_acceptance).
        let cmds = [
            ("s01", "conduct_opening_meeting"),
            ("s02", "define_engagement_scope"),
            ("s03", "identify_engagement_risks"),
            ("s04", "assess_engagement_risks"),
            ("s05", "assess_control_design"),
            ("s06", "evaluate_control_effectiveness"),
            ("s07", "design_test_procedures"),
            ("s08", "design_work_program"),
            ("s09", "perform_test_procedures"),
            ("s10", "draft_ia_charter"),
            ("s11", "determine_overall_materiality"),
            ("s12", "develop_engagement_conclusions"),
            ("s13", "finalize_audit_report"),
        ];

        for (id, cmd) in &cmds {
            let step = step_with_command(id, cmd);
            dispatcher.dispatch(&step, "ia_proc", &ctx, &mut bag);
        }

        assert!(
            bag.total_artifacts() > 0,
            "IA dispatch must produce at least one artifact, got 0"
        );

        // Auto-bootstrap should have created an engagement.
        assert!(
            !bag.engagements.is_empty(),
            "auto-bootstrap should have created an engagement"
        );

        // conduct_opening_meeting dispatches to engagement gen — may
        // produce additional engagement records.
        assert!(
            !bag.engagements.is_empty(),
            "expected at least 1 engagement, got 0"
        );

        // Risk assessments from identify/assess engagement risks.
        assert!(
            !bag.risk_assessments.is_empty(),
            "expected risk assessments from IA risk commands"
        );

        // CRAs from assess_control_design / evaluate_control_effectiveness.
        assert!(
            !bag.combined_risk_assessments.is_empty(),
            "expected CRAs from IA control evaluation commands"
        );

        // Workpapers from design_test_procedures / design_work_program.
        assert!(
            !bag.workpapers.is_empty(),
            "expected workpapers from IA design commands"
        );

        // Materiality from determine_overall_materiality.
        assert!(
            !bag.materiality_calculations.is_empty(),
            "expected materiality calculations"
        );

        // Opinion from develop_engagement_conclusions / finalize_audit_report.
        assert!(
            !bag.audit_opinions.is_empty(),
            "expected audit opinions from IA reporting commands"
        );
    }

    #[test]
    fn test_ia_finding_commands_dispatch() {
        // Test C2CE commands produce findings.
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Bootstrap engagement + workpapers so findings have context.
        let setup_cmds = [
            ("s0", "conduct_opening_meeting"),
            ("s1", "design_test_procedures"),
        ];
        for (id, cmd) in &setup_cmds {
            let step = step_with_command(id, cmd);
            dispatcher.dispatch(&step, "setup", &ctx, &mut bag);
        }

        let finding_cmds = [
            "identify_condition",
            "map_criteria",
            "analyze_cause",
            "assess_effect",
            "draft_finding",
            "identify_finding_condition",
            "map_finding_criteria",
            "analyze_cause_and_effect",
            "evaluate_finding_significance",
            "evaluate_findings",
            "evaluate_misstatements",
        ];

        let findings_before = bag.findings.len();
        for (i, cmd) in finding_cmds.iter().enumerate() {
            let step = step_with_command(&format!("f{}", i), cmd);
            dispatcher.dispatch(&step, "c2ce", &ctx, &mut bag);
        }

        assert!(
            bag.findings.len() > findings_before,
            "C2CE commands should produce findings; before={}, after={}",
            findings_before,
            bag.findings.len()
        );
    }

    #[test]
    fn test_ia_auto_bootstrap_skips_fsm_commands() {
        // FSM lifecycle commands (activate, submit_for_review, etc.)
        // should NOT trigger auto-bootstrap of the engagement.
        let mut dispatcher = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        let skip_cmds = [
            "activate",
            "submit_for_review",
            "cycle_back",
            "complete_cycle",
            "start_universe_update",
            "submit_universe",
            "approve_universe",
        ];

        for (i, cmd) in skip_cmds.iter().enumerate() {
            let step = step_with_command(&format!("x{}", i), cmd);
            dispatcher.dispatch(&step, "fsm_ctrl", &ctx, &mut bag);
        }

        assert!(
            bag.engagements.is_empty(),
            "FSM lifecycle commands should not trigger auto-bootstrap; got {} engagements",
            bag.engagements.len()
        );
    }

    #[test]
    fn test_section_for_command_ia_keywords() {
        // IA-specific keyword mapping checks.
        assert_eq!(
            section_for_command("identify_finding_condition"),
            WorkpaperSection::Completion
        );
        assert_eq!(
            section_for_command("track_action_plan_status"),
            WorkpaperSection::Planning
        );
        assert_eq!(
            section_for_command("perform_test_procedures"),
            WorkpaperSection::SubstantiveTesting
        );
        assert_eq!(
            section_for_command("draft_audit_report"),
            WorkpaperSection::Reporting
        );
        assert_eq!(
            section_for_command("assess_universe_risks"),
            WorkpaperSection::RiskAssessment
        );
        assert_eq!(
            section_for_command("establish_ethics_code"),
            WorkpaperSection::Planning
        );
        assert_eq!(
            section_for_command("develop_engagement_conclusions"),
            WorkpaperSection::Completion
        );
    }

    #[test]
    fn test_ia_judgment_dispatch() {
        let mut d = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Bootstrap engagement.
        d.dispatch(
            &step_with_command("e1", "evaluate_client_acceptance"),
            "p",
            &ctx,
            &mut bag,
        );
        // Dispatch a judgment command.
        d.dispatch(
            &step_with_command("j1", "review_engagement_quality"),
            "p",
            &ctx,
            &mut bag,
        );
        assert!(
            !bag.judgments.is_empty(),
            "judgment dispatch should produce at least one judgment"
        );
    }

    #[test]
    fn test_ia_evidence_dispatch() {
        let mut d = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Bootstrap engagement.
        d.dispatch(
            &step_with_command("e1", "evaluate_client_acceptance"),
            "p",
            &ctx,
            &mut bag,
        );
        // Create a workpaper for evidence to attach to.
        d.dispatch(
            &step_with_command("w1", "design_work_program"),
            "p",
            &ctx,
            &mut bag,
        );
        // Dispatch a documentation/evidence command.
        d.dispatch(
            &step_with_command("d1", "document_engagement_work"),
            "p",
            &ctx,
            &mut bag,
        );
        assert!(
            !bag.evidence.is_empty(),
            "evidence dispatch should produce evidence records"
        );
    }

    #[test]
    fn test_ia_planning_workpaper_dispatch() {
        let mut d = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Bootstrap engagement.
        d.dispatch(
            &step_with_command("e1", "evaluate_client_acceptance"),
            "p",
            &ctx,
            &mut bag,
        );
        let before = bag.workpapers.len();
        // Dispatch a planning command.
        d.dispatch(
            &step_with_command("p1", "draft_annual_plan"),
            "p",
            &ctx,
            &mut bag,
        );
        assert!(
            bag.workpapers.len() > before,
            "planning dispatch should produce a workpaper; before={}, after={}",
            before,
            bag.workpapers.len()
        );
    }

    // -------------------------------------------------------------------
    // ContentGenerator wiring tests
    // -------------------------------------------------------------------

    #[test]
    fn test_findings_have_content_generator_narrative() {
        let mut d = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Bootstrap engagement + workpapers.
        d.dispatch(
            &step_with_command("e1", "evaluate_client_acceptance"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("w1", "design_test_procedures"),
            "p",
            &ctx,
            &mut bag,
        );

        // Generate findings via dispatch.
        d.dispatch(
            &step_with_command("f1", "evaluate_findings"),
            "test_proc",
            &ctx,
            &mut bag,
        );

        assert!(
            !bag.findings.is_empty(),
            "should produce at least one finding"
        );

        // The TemplateContentGenerator produces narratives containing "During the"
        // which is its distinctive format.
        let first_finding = &bag.findings[0];
        assert!(
            first_finding.condition.contains("During the"),
            "finding condition should contain content-generator narrative; got: {}",
            first_finding.condition,
        );
    }

    #[test]
    fn test_findings_with_anomaly_refs() {
        let mut d = StepDispatcher::new(42);
        let ctx = EngagementContext::test_with_anomalies();
        let mut bag = ArtifactBag::default();

        // Bootstrap.
        d.dispatch(
            &step_with_command("e1", "evaluate_client_acceptance"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("w1", "design_test_procedures"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("f1", "evaluate_findings"),
            "test_proc",
            &ctx,
            &mut bag,
        );

        assert!(!bag.findings.is_empty(), "should produce findings");

        // Anomaly refs should be threaded into finding conditions.
        let has_anomaly_ref = bag.findings.iter().any(|f| {
            f.condition.contains("Linked anomaly: ANOM-001")
                || f.condition.contains("Linked anomaly: ANOM-002")
        });
        assert!(
            has_anomaly_ref,
            "at least one finding should reference an anomaly"
        );
    }

    #[test]
    fn test_findings_with_je_evidence_refs() {
        let mut d = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Bootstrap.
        d.dispatch(
            &step_with_command("e1", "evaluate_client_acceptance"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("w1", "design_test_procedures"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("f1", "evaluate_findings"),
            "test_proc",
            &ctx,
            &mut bag,
        );

        assert!(!bag.findings.is_empty(), "should produce findings");

        // JE refs should be threaded into finding effects.
        let has_je_ref = bag
            .findings
            .iter()
            .any(|f| f.effect.contains("Supporting JE: JE-2025-"));
        assert!(has_je_ref, "at least one finding should reference a JE");
    }

    #[test]
    fn test_findings_linked_to_controls() {
        let mut d = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        // Bootstrap with engagement + risk assessments.
        d.dispatch(
            &step_with_command("e1", "evaluate_client_acceptance"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("r1", "identify_risks"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("w1", "design_test_procedures"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("f1", "evaluate_findings"),
            "test_proc",
            &ctx,
            &mut bag,
        );

        assert!(!bag.findings.is_empty(), "should produce findings");

        // At least some findings should have control linkage (since context has control_ids).
        let has_control_link = bag
            .findings
            .iter()
            .any(|f| !f.related_control_ids.is_empty());
        assert!(
            has_control_link,
            "at least one finding should be linked to controls"
        );
    }

    #[test]
    fn test_workpaper_objectives_enriched() {
        let mut d = StepDispatcher::new(42);
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        d.dispatch(
            &step_with_command("e1", "evaluate_client_acceptance"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("w1", "design_test_procedures"),
            "test_proc",
            &ctx,
            &mut bag,
        );

        assert!(
            !bag.workpapers.is_empty(),
            "should produce at least one workpaper"
        );

        // The TemplateContentGenerator produces objectives containing "Workpaper for".
        let first_wp = &bag.workpapers[0];
        assert!(
            first_wp.objective.contains("Workpaper for"),
            "workpaper objective should contain content-generator narrative; got: {}",
            first_wp.objective,
        );
    }

    #[test]
    fn test_new_with_content_uses_custom_generator() {
        use crate::content::{ContentGenerator, FindingContext, ResponseContext, WorkpaperContext};

        /// A test content generator that prefixes all output with "CUSTOM:".
        struct CustomGen;
        impl ContentGenerator for CustomGen {
            fn generate_finding_narrative(&self, _ctx: &FindingContext) -> String {
                "CUSTOM: finding narrative".to_string()
            }
            fn generate_workpaper_narrative(&self, _ctx: &WorkpaperContext) -> String {
                "CUSTOM: workpaper objective".to_string()
            }
            fn generate_management_response(&self, _ctx: &ResponseContext) -> String {
                "CUSTOM: management response".to_string()
            }
        }

        let mut d = StepDispatcher::new_with_content(42, Box::new(CustomGen));
        let ctx = EngagementContext::test_default();
        let mut bag = ArtifactBag::default();

        d.dispatch(
            &step_with_command("e1", "evaluate_client_acceptance"),
            "p",
            &ctx,
            &mut bag,
        );
        d.dispatch(
            &step_with_command("w1", "design_test_procedures"),
            "test_proc",
            &ctx,
            &mut bag,
        );

        assert!(!bag.workpapers.is_empty());
        assert!(
            bag.workpapers[0].objective.contains("CUSTOM:"),
            "custom content generator should be used; got: {}",
            bag.workpapers[0].objective,
        );

        // Now test findings.
        d.dispatch(
            &step_with_command("f1", "evaluate_findings"),
            "test_proc",
            &ctx,
            &mut bag,
        );

        assert!(!bag.findings.is_empty());
        assert!(
            bag.findings[0].condition.contains("CUSTOM:"),
            "custom content generator should be used for findings; got: {}",
            bag.findings[0].condition,
        );
    }
}
