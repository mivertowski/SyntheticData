//! Property serializers for Audit entities.
//!
//! Covers: AuditEngagement, Workpaper, AuditEvidence, AuditFinding, ProfessionalJudgment.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

// ──────────────────────────── Engagement ────────────────────────────

/// Property serializer for audit engagements (entity type code 350).
pub struct EngagementPropertySerializer;

impl PropertySerializer for EngagementPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "audit_engagement"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let eng = ctx
            .ds_result
            .audit
            .engagements
            .iter()
            .find(|e| e.engagement_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(14);

        props.insert(
            "engagementId".into(),
            Value::String(eng.engagement_id.to_string()),
        );
        props.insert(
            "engagementRef".into(),
            Value::String(eng.engagement_ref.clone()),
        );
        props.insert("clientName".into(), Value::String(eng.client_name.clone()));
        props.insert(
            "engagementType".into(),
            serde_json::to_value(&eng.engagement_type).unwrap_or(Value::Null),
        );
        props.insert("fiscalYear".into(), Value::Number(eng.fiscal_year.into()));
        props.insert(
            "periodEndDate".into(),
            Value::String(eng.period_end_date.format("%Y-%m-%d").to_string()),
        );
        props.insert("materiality".into(), serde_json::json!(eng.materiality));
        props.insert(
            "performanceMateriality".into(),
            serde_json::json!(eng.performance_materiality),
        );
        props.insert(
            "materialityBasis".into(),
            Value::String(eng.materiality_basis.clone()),
        );
        props.insert(
            "planningStart".into(),
            Value::String(eng.planning_start.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "fieldworkStart".into(),
            Value::String(eng.fieldwork_start.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "status".into(),
            serde_json::to_value(&eng.status).unwrap_or(Value::Null),
        );
        props.insert("type".into(), Value::String("engagement".into()));

        Some(props)
    }
}

// ──────────────────────────── Workpaper ─────────────────────────────

/// Property serializer for workpapers (entity type code 351).
pub struct WorkpaperPropertySerializer;

impl PropertySerializer for WorkpaperPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "workpaper"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let wp = ctx
            .ds_result
            .audit
            .workpapers
            .iter()
            .find(|w| w.workpaper_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(12);

        props.insert(
            "workpaperId".into(),
            Value::String(wp.workpaper_id.to_string()),
        );
        props.insert(
            "workpaperRef".into(),
            Value::String(wp.workpaper_ref.clone()),
        );
        props.insert("title".into(), Value::String(wp.title.clone()));
        props.insert(
            "section".into(),
            serde_json::to_value(&wp.section).unwrap_or(Value::Null),
        );
        props.insert(
            "procedureType".into(),
            serde_json::to_value(&wp.procedure_type).unwrap_or(Value::Null),
        );
        props.insert(
            "populationSize".into(),
            Value::Number(wp.population_size.into()),
        );
        props.insert("sampleSize".into(), Value::Number(wp.sample_size.into()));
        props.insert(
            "exceptionsFound".into(),
            Value::Number(wp.exceptions_found.into()),
        );
        props.insert(
            "resultsSummary".into(),
            Value::String(wp.results_summary.clone()),
        );
        props.insert(
            "status".into(),
            serde_json::to_value(&wp.status).unwrap_or(Value::Null),
        );
        props.insert("type".into(), Value::String("workpaper".into()));

        Some(props)
    }
}

// ──────────────────────────── Evidence ──────────────────────────────

/// Property serializer for audit evidence (entity type code 352).
pub struct EvidencePropertySerializer;

impl PropertySerializer for EvidencePropertySerializer {
    fn entity_type(&self) -> &'static str {
        "audit_evidence"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let ev = ctx
            .ds_result
            .audit
            .evidence
            .iter()
            .find(|e| e.evidence_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert(
            "evidenceId".into(),
            Value::String(ev.evidence_id.to_string()),
        );
        props.insert("evidenceRef".into(), Value::String(ev.evidence_ref.clone()));
        props.insert("title".into(), Value::String(ev.title.clone()));
        props.insert(
            "evidenceType".into(),
            serde_json::to_value(&ev.evidence_type).unwrap_or(Value::Null),
        );
        props.insert(
            "sourceType".into(),
            serde_json::to_value(&ev.source_type).unwrap_or(Value::Null),
        );
        props.insert(
            "obtainedDate".into(),
            Value::String(ev.obtained_date.format("%Y-%m-%d").to_string()),
        );
        props.insert("obtainedBy".into(), Value::String(ev.obtained_by.clone()));
        props.insert(
            "reliability".into(),
            serde_json::to_value(&ev.reliability_assessment).unwrap_or(Value::Null),
        );
        props.insert(
            "status".into(),
            serde_json::to_value(&ev.status).unwrap_or(Value::Null),
        );
        props.insert("type".into(), Value::String("evidence".into()));

        Some(props)
    }
}

// ──────────────────────────── Finding ────────────────────────────────

/// Property serializer for audit findings (entity type code 354).
pub struct FindingPropertySerializer;

impl PropertySerializer for FindingPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "audit_finding"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let f = ctx
            .ds_result
            .audit
            .findings
            .iter()
            .find(|f| f.finding_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(14);

        props.insert("findingId".into(), Value::String(f.finding_id.to_string()));
        props.insert("findingRef".into(), Value::String(f.finding_ref.clone()));
        props.insert("title".into(), Value::String(f.title.clone()));
        props.insert(
            "findingType".into(),
            serde_json::to_value(&f.finding_type).unwrap_or(Value::Null),
        );
        props.insert(
            "severity".into(),
            serde_json::to_value(&f.severity).unwrap_or(Value::Null),
        );
        props.insert("condition".into(), Value::String(f.condition.clone()));
        props.insert("criteria".into(), Value::String(f.criteria.clone()));
        props.insert("cause".into(), Value::String(f.cause.clone()));
        props.insert("effect".into(), Value::String(f.effect.clone()));
        props.insert("isMisstatement".into(), Value::Bool(f.is_misstatement));
        if let Some(impact) = f.monetary_impact {
            props.insert("monetaryImpact".into(), serde_json::json!(impact));
        }
        props.insert(
            "status".into(),
            serde_json::to_value(&f.status).unwrap_or(Value::Null),
        );
        props.insert("type".into(), Value::String("finding".into()));

        Some(props)
    }
}

// ──────────────────────────── Judgment ───────────────────────────────

/// Property serializer for professional judgments (entity type code 355).
pub struct JudgmentPropertySerializer;

impl PropertySerializer for JudgmentPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "professional_judgment"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let j = ctx
            .ds_result
            .audit
            .judgments
            .iter()
            .find(|j| j.judgment_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert(
            "judgmentId".into(),
            Value::String(j.judgment_id.to_string()),
        );
        props.insert("judgmentRef".into(), Value::String(j.judgment_ref.clone()));
        props.insert(
            "judgmentType".into(),
            serde_json::to_value(&j.judgment_type).unwrap_or(Value::Null),
        );
        props.insert("subject".into(), Value::String(j.subject.clone()));
        props.insert("conclusion".into(), Value::String(j.conclusion.clone()));
        props.insert("rationale".into(), Value::String(j.rationale.clone()));
        props.insert(
            "residualRisk".into(),
            Value::String(j.residual_risk.clone()),
        );
        props.insert(
            "consultationRequired".into(),
            Value::Bool(j.consultation_required),
        );
        props.insert(
            "status".into(),
            serde_json::to_value(&j.status).unwrap_or(Value::Null),
        );
        props.insert("type".into(), Value::String("judgment".into()));

        Some(props)
    }
}

// ──────────────────────────── External Confirmation (ISA 505) ───────

/// Property serializer for external confirmations (entity type code 366).
pub struct ConfirmationPropertySerializer;

impl PropertySerializer for ConfirmationPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "external_confirmation"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let conf = ctx
            .ds_result
            .audit
            .confirmations
            .iter()
            .find(|c| c.confirmation_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(16);

        props.insert(
            "confirmationId".into(),
            Value::String(conf.confirmation_id.to_string()),
        );
        props.insert(
            "confirmationRef".into(),
            Value::String(conf.confirmation_ref.clone()),
        );
        props.insert(
            "engagementId".into(),
            Value::String(conf.engagement_id.to_string()),
        );
        if let Some(wp_id) = &conf.workpaper_id {
            props.insert("workpaperId".into(), Value::String(wp_id.to_string()));
        }
        props.insert(
            "confirmationType".into(),
            serde_json::to_value(&conf.confirmation_type).unwrap_or(Value::Null),
        );
        props.insert(
            "recipientName".into(),
            Value::String(conf.recipient_name.clone()),
        );
        props.insert(
            "recipientType".into(),
            serde_json::to_value(&conf.recipient_type).unwrap_or(Value::Null),
        );
        if let Some(acct) = &conf.account_id {
            props.insert("accountId".into(), Value::String(acct.clone()));
        }
        props.insert(
            "bookBalance".into(),
            Value::String(conf.book_balance.to_string()),
        );
        props.insert(
            "confirmationDate".into(),
            Value::String(conf.confirmation_date.format("%Y-%m-%d").to_string()),
        );
        if let Some(sent) = &conf.sent_date {
            props.insert(
                "sentDate".into(),
                Value::String(sent.format("%Y-%m-%d").to_string()),
            );
        }
        if let Some(deadline) = &conf.response_deadline {
            props.insert(
                "responseDeadline".into(),
                Value::String(deadline.format("%Y-%m-%d").to_string()),
            );
        }
        props.insert(
            "status".into(),
            serde_json::to_value(&conf.status).unwrap_or(Value::Null),
        );
        props.insert(
            "positiveNegative".into(),
            serde_json::to_value(&conf.positive_negative).unwrap_or(Value::Null),
        );
        props.insert("type".into(), Value::String("external_confirmation".into()));

        Some(props)
    }
}

// ──────────────────────────── Confirmation Response (ISA 505) ──────

/// Property serializer for confirmation responses (entity type code 367).
pub struct ConfirmationResponsePropertySerializer;

impl PropertySerializer for ConfirmationResponsePropertySerializer {
    fn entity_type(&self) -> &'static str {
        "confirmation_response"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let resp = ctx
            .ds_result
            .audit
            .confirmation_responses
            .iter()
            .find(|r| r.response_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(14);

        props.insert(
            "responseId".into(),
            Value::String(resp.response_id.to_string()),
        );
        props.insert(
            "responseRef".into(),
            Value::String(resp.response_ref.clone()),
        );
        props.insert(
            "confirmationId".into(),
            Value::String(resp.confirmation_id.to_string()),
        );
        props.insert(
            "engagementId".into(),
            Value::String(resp.engagement_id.to_string()),
        );
        props.insert(
            "responseDate".into(),
            Value::String(resp.response_date.format("%Y-%m-%d").to_string()),
        );
        if let Some(bal) = &resp.confirmed_balance {
            props.insert("confirmedBalance".into(), Value::String(bal.to_string()));
        }
        props.insert(
            "responseType".into(),
            serde_json::to_value(&resp.response_type).unwrap_or(Value::Null),
        );
        props.insert("hasException".into(), Value::Bool(resp.has_exception));
        if let Some(amt) = &resp.exception_amount {
            props.insert("exceptionAmount".into(), Value::String(amt.to_string()));
        }
        if let Some(desc) = &resp.exception_description {
            props.insert("exceptionDescription".into(), Value::String(desc.clone()));
        }
        props.insert("reconciled".into(), Value::Bool(resp.reconciled));
        if let Some(expl) = &resp.reconciliation_explanation {
            props.insert(
                "reconciliationExplanation".into(),
                Value::String(expl.clone()),
            );
        }
        props.insert("type".into(), Value::String("confirmation_response".into()));

        Some(props)
    }
}

// ──────────────────────────── Audit Procedure Step (ISA 330) ───────

/// Property serializer for audit procedure steps (entity type code 368).
pub struct ProcedureStepPropertySerializer;

impl PropertySerializer for ProcedureStepPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "audit_procedure_step"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let step = ctx
            .ds_result
            .audit
            .procedure_steps
            .iter()
            .find(|s| s.step_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(18);

        props.insert("stepId".into(), Value::String(step.step_id.to_string()));
        props.insert("stepRef".into(), Value::String(step.step_ref.clone()));
        props.insert(
            "workpaperId".into(),
            Value::String(step.workpaper_id.to_string()),
        );
        props.insert(
            "engagementId".into(),
            Value::String(step.engagement_id.to_string()),
        );
        props.insert("stepNumber".into(), Value::Number(step.step_number.into()));
        props.insert(
            "description".into(),
            Value::String(step.description.clone()),
        );
        props.insert(
            "procedureType".into(),
            serde_json::to_value(&step.procedure_type).unwrap_or(Value::Null),
        );
        props.insert(
            "assertion".into(),
            serde_json::to_value(&step.assertion).unwrap_or(Value::Null),
        );
        if let Some(d) = &step.planned_date {
            props.insert(
                "plannedDate".into(),
                Value::String(d.format("%Y-%m-%d").to_string()),
            );
        }
        if let Some(d) = &step.performed_date {
            props.insert(
                "performedDate".into(),
                Value::String(d.format("%Y-%m-%d").to_string()),
            );
        }
        if let Some(by) = &step.performed_by_name {
            props.insert("performedByName".into(), Value::String(by.clone()));
        }
        props.insert(
            "status".into(),
            serde_json::to_value(&step.status).unwrap_or(Value::Null),
        );
        if let Some(r) = &step.result {
            props.insert(
                "result".into(),
                serde_json::to_value(r).unwrap_or(Value::Null),
            );
        }
        props.insert("exceptionNoted".into(), Value::Bool(step.exception_noted));
        if let Some(desc) = &step.exception_description {
            props.insert("exceptionDescription".into(), Value::String(desc.clone()));
        }
        if let Some(sid) = &step.sample_id {
            props.insert("sampleId".into(), Value::String(sid.to_string()));
        }
        props.insert("type".into(), Value::String("audit_procedure_step".into()));

        Some(props)
    }
}

// ──────────────────────────── Audit Sample (ISA 530) ───────────────

/// Property serializer for audit samples (entity type code 369).
pub struct SamplePropertySerializer;

impl PropertySerializer for SamplePropertySerializer {
    fn entity_type(&self) -> &'static str {
        "audit_sample"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let sample = ctx
            .ds_result
            .audit
            .samples
            .iter()
            .find(|s| s.sample_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(18);

        props.insert(
            "sampleId".into(),
            Value::String(sample.sample_id.to_string()),
        );
        props.insert("sampleRef".into(), Value::String(sample.sample_ref.clone()));
        props.insert(
            "workpaperId".into(),
            Value::String(sample.workpaper_id.to_string()),
        );
        props.insert(
            "engagementId".into(),
            Value::String(sample.engagement_id.to_string()),
        );
        props.insert(
            "populationDescription".into(),
            Value::String(sample.population_description.clone()),
        );
        props.insert(
            "populationSize".into(),
            serde_json::json!(sample.population_size),
        );
        if let Some(pv) = &sample.population_value {
            props.insert("populationValue".into(), Value::String(pv.to_string()));
        }
        props.insert(
            "samplingMethod".into(),
            serde_json::to_value(&sample.sampling_method).unwrap_or(Value::Null),
        );
        props.insert(
            "sampleSize".into(),
            Value::Number(sample.sample_size.into()),
        );
        if let Some(si) = &sample.sampling_interval {
            props.insert("samplingInterval".into(), Value::String(si.to_string()));
        }
        props.insert(
            "confidenceLevel".into(),
            serde_json::json!(sample.confidence_level),
        );
        if let Some(tm) = &sample.tolerable_misstatement {
            props.insert(
                "tolerableMisstatement".into(),
                Value::String(tm.to_string()),
            );
        }
        if let Some(em) = &sample.expected_misstatement {
            props.insert("expectedMisstatement".into(), Value::String(em.to_string()));
        }
        props.insert(
            "totalMisstatementFound".into(),
            Value::String(sample.total_misstatement_found.to_string()),
        );
        if let Some(pm) = &sample.projected_misstatement {
            props.insert(
                "projectedMisstatement".into(),
                Value::String(pm.to_string()),
            );
        }
        if let Some(c) = &sample.conclusion {
            props.insert(
                "conclusion".into(),
                serde_json::to_value(c).unwrap_or(Value::Null),
            );
        }
        props.insert("itemCount".into(), Value::Number(sample.items.len().into()));
        props.insert("type".into(), Value::String("audit_sample".into()));

        Some(props)
    }
}

// ──────────────────────────── Analytical Procedure Result (ISA 520) ─

/// Property serializer for analytical procedure results (entity type code 375).
pub struct AnalyticalProcedurePropertySerializer;

impl PropertySerializer for AnalyticalProcedurePropertySerializer {
    fn entity_type(&self) -> &'static str {
        "analytical_procedure_result"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let ap = ctx
            .ds_result
            .audit
            .analytical_results
            .iter()
            .find(|a| a.result_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(22);

        props.insert("resultId".into(), Value::String(ap.result_id.to_string()));
        props.insert("resultRef".into(), Value::String(ap.result_ref.clone()));
        props.insert(
            "engagementId".into(),
            Value::String(ap.engagement_id.to_string()),
        );
        if let Some(wp_id) = &ap.workpaper_id {
            props.insert("workpaperId".into(), Value::String(wp_id.to_string()));
        }
        props.insert(
            "procedurePhase".into(),
            serde_json::to_value(&ap.procedure_phase).unwrap_or(Value::Null),
        );
        props.insert(
            "accountOrArea".into(),
            Value::String(ap.account_or_area.clone()),
        );
        if let Some(acct) = &ap.account_id {
            props.insert("accountId".into(), Value::String(acct.clone()));
        }
        props.insert(
            "analyticalMethod".into(),
            serde_json::to_value(&ap.analytical_method).unwrap_or(Value::Null),
        );
        props.insert(
            "expectation".into(),
            Value::String(ap.expectation.to_string()),
        );
        props.insert(
            "expectationBasis".into(),
            Value::String(ap.expectation_basis.clone()),
        );
        props.insert("threshold".into(), Value::String(ap.threshold.to_string()));
        props.insert(
            "thresholdBasis".into(),
            Value::String(ap.threshold_basis.clone()),
        );
        props.insert(
            "actualValue".into(),
            Value::String(ap.actual_value.to_string()),
        );
        props.insert("variance".into(), Value::String(ap.variance.to_string()));
        props.insert(
            "variancePercentage".into(),
            serde_json::json!(ap.variance_percentage),
        );
        props.insert(
            "requiresInvestigation".into(),
            Value::Bool(ap.requires_investigation),
        );
        if let Some(expl) = &ap.explanation {
            props.insert("explanation".into(), Value::String(expl.clone()));
        }
        if let Some(corr) = &ap.explanation_corroborated {
            props.insert("explanationCorroborated".into(), Value::Bool(*corr));
        }
        if let Some(c) = &ap.conclusion {
            props.insert(
                "conclusion".into(),
                serde_json::to_value(c).unwrap_or(Value::Null),
            );
        }
        props.insert(
            "status".into(),
            serde_json::to_value(&ap.status).unwrap_or(Value::Null),
        );
        props.insert(
            "type".into(),
            Value::String("analytical_procedure_result".into()),
        );

        Some(props)
    }
}

// ──────────────────────────── Internal Audit Function (ISA 610) ────

/// Property serializer for internal audit functions (entity type code 376).
pub struct IaFunctionPropertySerializer;

impl PropertySerializer for IaFunctionPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "internal_audit_function"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let iaf = ctx
            .ds_result
            .audit
            .ia_functions
            .iter()
            .find(|f| f.function_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(20);

        props.insert(
            "functionId".into(),
            Value::String(iaf.function_id.to_string()),
        );
        props.insert(
            "functionRef".into(),
            Value::String(iaf.function_ref.clone()),
        );
        props.insert(
            "engagementId".into(),
            Value::String(iaf.engagement_id.to_string()),
        );
        props.insert(
            "departmentName".into(),
            Value::String(iaf.department_name.clone()),
        );
        props.insert(
            "reportingLine".into(),
            serde_json::to_value(&iaf.reporting_line).unwrap_or(Value::Null),
        );
        props.insert("headOfIa".into(), Value::String(iaf.head_of_ia.clone()));
        props.insert(
            "headOfIaQualifications".into(),
            serde_json::json!(iaf.head_of_ia_qualifications),
        );
        props.insert("staffCount".into(), Value::Number(iaf.staff_count.into()));
        props.insert(
            "annualPlanCoverage".into(),
            serde_json::json!(iaf.annual_plan_coverage),
        );
        props.insert(
            "qualityAssurance".into(),
            Value::Bool(iaf.quality_assurance),
        );
        props.insert(
            "isa610Assessment".into(),
            serde_json::to_value(&iaf.isa_610_assessment).unwrap_or(Value::Null),
        );
        props.insert(
            "objectivityRating".into(),
            serde_json::to_value(&iaf.objectivity_rating).unwrap_or(Value::Null),
        );
        props.insert(
            "competenceRating".into(),
            serde_json::to_value(&iaf.competence_rating).unwrap_or(Value::Null),
        );
        props.insert(
            "systematicDiscipline".into(),
            Value::Bool(iaf.systematic_discipline),
        );
        props.insert(
            "relianceExtent".into(),
            serde_json::to_value(&iaf.reliance_extent).unwrap_or(Value::Null),
        );
        props.insert(
            "relianceAreas".into(),
            serde_json::json!(iaf.reliance_areas),
        );
        props.insert(
            "directAssistance".into(),
            Value::Bool(iaf.direct_assistance),
        );
        props.insert(
            "type".into(),
            Value::String("internal_audit_function".into()),
        );

        Some(props)
    }
}

// ──────────────────────────── Internal Audit Report (ISA 610) ──────

/// Property serializer for internal audit reports (entity type code 377).
pub struct IaReportPropertySerializer;

impl PropertySerializer for IaReportPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "internal_audit_report"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let iar = ctx
            .ds_result
            .audit
            .ia_reports
            .iter()
            .find(|r| r.report_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(18);

        props.insert("reportId".into(), Value::String(iar.report_id.to_string()));
        props.insert("reportRef".into(), Value::String(iar.report_ref.clone()));
        props.insert(
            "engagementId".into(),
            Value::String(iar.engagement_id.to_string()),
        );
        props.insert(
            "iaFunctionId".into(),
            Value::String(iar.ia_function_id.to_string()),
        );
        props.insert(
            "reportTitle".into(),
            Value::String(iar.report_title.clone()),
        );
        props.insert("auditArea".into(), Value::String(iar.audit_area.clone()));
        props.insert(
            "reportDate".into(),
            Value::String(iar.report_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "periodStart".into(),
            Value::String(iar.period_start.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "periodEnd".into(),
            Value::String(iar.period_end.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "scopeDescription".into(),
            Value::String(iar.scope_description.clone()),
        );
        props.insert("methodology".into(), Value::String(iar.methodology.clone()));
        props.insert(
            "overallRating".into(),
            serde_json::to_value(&iar.overall_rating).unwrap_or(Value::Null),
        );
        props.insert(
            "findingsCount".into(),
            Value::Number(iar.findings_count.into()),
        );
        props.insert(
            "highRiskFindings".into(),
            Value::Number(iar.high_risk_findings.into()),
        );
        props.insert(
            "recommendationCount".into(),
            Value::Number(iar.recommendations.len().into()),
        );
        props.insert(
            "status".into(),
            serde_json::to_value(&iar.status).unwrap_or(Value::Null),
        );
        if let Some(assessment) = &iar.external_auditor_assessment {
            props.insert(
                "externalAuditorAssessment".into(),
                serde_json::to_value(assessment).unwrap_or(Value::Null),
            );
        }
        props.insert("type".into(), Value::String("internal_audit_report".into()));

        Some(props)
    }
}

// ──────────────────────────── Related Party (ISA 550) ──────────────

/// Property serializer for related parties (entity type code 378).
pub struct RelatedPartyPropertySerializer;

impl PropertySerializer for RelatedPartyPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "related_party"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let rp = ctx
            .ds_result
            .audit
            .related_parties
            .iter()
            .find(|p| p.party_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(14);

        props.insert("partyId".into(), Value::String(rp.party_id.to_string()));
        props.insert("partyRef".into(), Value::String(rp.party_ref.clone()));
        props.insert(
            "engagementId".into(),
            Value::String(rp.engagement_id.to_string()),
        );
        props.insert("partyName".into(), Value::String(rp.party_name.clone()));
        props.insert(
            "partyType".into(),
            serde_json::to_value(&rp.party_type).unwrap_or(Value::Null),
        );
        props.insert(
            "relationshipBasis".into(),
            serde_json::to_value(&rp.relationship_basis).unwrap_or(Value::Null),
        );
        if let Some(pct) = rp.ownership_percentage {
            props.insert("ownershipPercentage".into(), serde_json::json!(pct));
        }
        props.insert(
            "boardRepresentation".into(),
            Value::Bool(rp.board_representation),
        );
        props.insert("keyManagement".into(), Value::Bool(rp.key_management));
        props.insert(
            "disclosedInFinancials".into(),
            Value::Bool(rp.disclosed_in_financials),
        );
        if let Some(adequate) = rp.disclosure_adequate {
            props.insert("disclosureAdequate".into(), Value::Bool(adequate));
        }
        props.insert(
            "identifiedBy".into(),
            serde_json::to_value(&rp.identified_by).unwrap_or(Value::Null),
        );
        props.insert("type".into(), Value::String("related_party".into()));

        Some(props)
    }
}

// ──────────────────────────── Related Party Transaction (ISA 550) ──

/// Property serializer for related party transactions (entity type code 379).
pub struct RelatedPartyTransactionPropertySerializer;

impl PropertySerializer for RelatedPartyTransactionPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "related_party_transaction"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let rpt = ctx
            .ds_result
            .audit
            .related_party_transactions
            .iter()
            .find(|t| t.transaction_ref == node_external_id)?;

        let mut props = HashMap::with_capacity(18);

        props.insert(
            "transactionId".into(),
            Value::String(rpt.transaction_id.to_string()),
        );
        props.insert(
            "transactionRef".into(),
            Value::String(rpt.transaction_ref.clone()),
        );
        props.insert(
            "engagementId".into(),
            Value::String(rpt.engagement_id.to_string()),
        );
        props.insert(
            "relatedPartyId".into(),
            Value::String(rpt.related_party_id.to_string()),
        );
        props.insert(
            "transactionType".into(),
            serde_json::to_value(&rpt.transaction_type).unwrap_or(Value::Null),
        );
        props.insert("description".into(), Value::String(rpt.description.clone()));
        props.insert("amount".into(), Value::String(rpt.amount.to_string()));
        props.insert("currency".into(), Value::String(rpt.currency.clone()));
        props.insert(
            "transactionDate".into(),
            Value::String(rpt.transaction_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "termsDescription".into(),
            Value::String(rpt.terms_description.clone()),
        );
        if let Some(al) = rpt.arms_length {
            props.insert("armsLength".into(), Value::Bool(al));
        }
        if let Some(evidence) = &rpt.arms_length_evidence {
            props.insert("armsLengthEvidence".into(), Value::String(evidence.clone()));
        }
        if let Some(rationale) = &rpt.business_rationale {
            props.insert("businessRationale".into(), Value::String(rationale.clone()));
        }
        if let Some(by) = &rpt.approved_by {
            props.insert("approvedBy".into(), Value::String(by.clone()));
        }
        props.insert(
            "disclosedInFinancials".into(),
            Value::Bool(rpt.disclosed_in_financials),
        );
        if let Some(adequate) = rpt.disclosure_adequate {
            props.insert("disclosureAdequate".into(), Value::Bool(adequate));
        }
        props.insert(
            "managementOverrideRisk".into(),
            Value::Bool(rpt.management_override_risk),
        );
        props.insert(
            "type".into(),
            Value::String("related_party_transaction".into()),
        );

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_types_are_correct() {
        assert_eq!(
            EngagementPropertySerializer.entity_type(),
            "audit_engagement"
        );
        assert_eq!(WorkpaperPropertySerializer.entity_type(), "workpaper");
        assert_eq!(EvidencePropertySerializer.entity_type(), "audit_evidence");
        assert_eq!(FindingPropertySerializer.entity_type(), "audit_finding");
        assert_eq!(
            JudgmentPropertySerializer.entity_type(),
            "professional_judgment"
        );
    }

    #[test]
    fn new_entity_types_are_correct() {
        assert_eq!(
            ConfirmationPropertySerializer.entity_type(),
            "external_confirmation"
        );
        assert_eq!(
            ConfirmationResponsePropertySerializer.entity_type(),
            "confirmation_response"
        );
        assert_eq!(
            ProcedureStepPropertySerializer.entity_type(),
            "audit_procedure_step"
        );
        assert_eq!(SamplePropertySerializer.entity_type(), "audit_sample");
        assert_eq!(
            AnalyticalProcedurePropertySerializer.entity_type(),
            "analytical_procedure_result"
        );
        assert_eq!(
            IaFunctionPropertySerializer.entity_type(),
            "internal_audit_function"
        );
        assert_eq!(
            IaReportPropertySerializer.entity_type(),
            "internal_audit_report"
        );
        assert_eq!(
            RelatedPartyPropertySerializer.entity_type(),
            "related_party"
        );
        assert_eq!(
            RelatedPartyTransactionPropertySerializer.entity_type(),
            "related_party_transaction"
        );
    }
}
