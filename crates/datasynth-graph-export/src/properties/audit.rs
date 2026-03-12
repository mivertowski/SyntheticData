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
        props.insert(
            "clientName".into(),
            Value::String(eng.client_name.clone()),
        );
        props.insert(
            "engagementType".into(),
            Value::String(format!("{:?}", eng.engagement_type)),
        );
        props.insert(
            "fiscalYear".into(),
            Value::Number(eng.fiscal_year.into()),
        );
        props.insert(
            "periodEndDate".into(),
            Value::String(eng.period_end_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "materiality".into(),
            serde_json::json!(eng.materiality),
        );
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
        props.insert("status".into(), Value::String("active".into()));
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
            Value::String(format!("{:?}", wp.section)),
        );
        props.insert(
            "procedureType".into(),
            Value::String(format!("{:?}", wp.procedure_type)),
        );
        props.insert(
            "populationSize".into(),
            Value::Number(wp.population_size.into()),
        );
        props.insert(
            "sampleSize".into(),
            Value::Number(wp.sample_size.into()),
        );
        props.insert(
            "exceptionsFound".into(),
            Value::Number(wp.exceptions_found.into()),
        );
        props.insert(
            "resultsSummary".into(),
            Value::String(wp.results_summary.clone()),
        );
        props.insert("status".into(), Value::String("completed".into()));
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
        props.insert(
            "evidenceRef".into(),
            Value::String(ev.evidence_ref.clone()),
        );
        props.insert("title".into(), Value::String(ev.title.clone()));
        props.insert(
            "evidenceType".into(),
            Value::String(format!("{:?}", ev.evidence_type)),
        );
        props.insert(
            "sourceType".into(),
            Value::String(format!("{:?}", ev.source_type)),
        );
        props.insert(
            "obtainedDate".into(),
            Value::String(ev.obtained_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "obtainedBy".into(),
            Value::String(ev.obtained_by.clone()),
        );
        props.insert(
            "reliability".into(),
            Value::String(format!("{:?}", ev.reliability_assessment)),
        );
        props.insert("status".into(), Value::String("obtained".into()));
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

        props.insert(
            "findingId".into(),
            Value::String(f.finding_id.to_string()),
        );
        props.insert(
            "findingRef".into(),
            Value::String(f.finding_ref.clone()),
        );
        props.insert("title".into(), Value::String(f.title.clone()));
        props.insert(
            "findingType".into(),
            Value::String(format!("{:?}", f.finding_type)),
        );
        props.insert(
            "severity".into(),
            Value::String(format!("{:?}", f.severity)),
        );
        props.insert(
            "condition".into(),
            Value::String(f.condition.clone()),
        );
        props.insert(
            "criteria".into(),
            Value::String(f.criteria.clone()),
        );
        props.insert("cause".into(), Value::String(f.cause.clone()));
        props.insert("effect".into(), Value::String(f.effect.clone()));
        props.insert(
            "isMisstatement".into(),
            Value::Bool(f.is_misstatement),
        );
        if let Some(impact) = f.monetary_impact {
            props.insert("monetaryImpact".into(), serde_json::json!(impact));
        }
        props.insert("status".into(), Value::String("open".into()));
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
        props.insert(
            "judgmentRef".into(),
            Value::String(j.judgment_ref.clone()),
        );
        props.insert(
            "judgmentType".into(),
            Value::String(format!("{:?}", j.judgment_type)),
        );
        props.insert("subject".into(), Value::String(j.subject.clone()));
        props.insert(
            "conclusion".into(),
            Value::String(j.conclusion.clone()),
        );
        props.insert(
            "rationale".into(),
            Value::String(j.rationale.clone()),
        );
        props.insert(
            "residualRisk".into(),
            Value::String(j.residual_risk.clone()),
        );
        props.insert(
            "consultationRequired".into(),
            Value::Bool(j.consultation_required),
        );
        props.insert("status".into(), Value::String("documented".into()));
        props.insert("type".into(), Value::String("judgment".into()));

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_types_are_correct() {
        assert_eq!(EngagementPropertySerializer.entity_type(), "audit_engagement");
        assert_eq!(WorkpaperPropertySerializer.entity_type(), "workpaper");
        assert_eq!(EvidencePropertySerializer.entity_type(), "audit_evidence");
        assert_eq!(FindingPropertySerializer.entity_type(), "audit_finding");
        assert_eq!(
            JudgmentPropertySerializer.entity_type(),
            "professional_judgment"
        );
    }
}
