//! v1.3.0 entity node synthesizer.
//!
//! Creates graph nodes for the 28 entity types added in the v1.3.0 release
//! across three layers:
//!
//! ## Layer 1 — Governance / Audit
//!
//! | Code | Entity                     | External-ID pattern        |
//! |------|----------------------------|----------------------------|
//! | 380  | CombinedRiskAssessment     | `CRA-{id}`                 |
//! | 381  | MaterialityCalculation     | `MAT-{entity_code}-{period}` |
//! | 382  | AuditOpinion               | `OPINION-{opinion_id}`     |
//! | 383  | KeyAuditMatter             | `KAM-{idx}-{engagement_id}` |
//! | 384  | Sox302Certification        | `SOX302-{certification_id}` |
//! | 385  | Sox404Assessment           | `SOX404-{assessment_id}`   |
//! | 386  | GoingConcernAssessment     | `GC-{entity_code}-{period}` |
//! | 387  | ComponentAuditor           | `CAUD-{id}`                |
//! | 388  | GroupAuditPlan             | `GAP-{engagement_id}`      |
//! | 389  | ComponentInstruction       | `CINST-{id}`               |
//! | 390  | ComponentAuditorReport     | `CRPT-{id}`                |
//! | 391  | EngagementLetter           | `ELET-{id}`                |
//! | 392  | GroupStructure             | `GSTRUCT-{parent_entity}`  |
//!
//! ## Layer 2 — Process / Audit Procedure
//!
//! | Code | Entity                            | External-ID pattern          |
//! |------|-----------------------------------|------------------------------|
//! | 393  | SamplingPlan                      | `SPLAN-{id}`                 |
//! | 394  | SampledItem                       | `SITM-{item_id}`             |
//! | 395  | SignificantClassOfTransactions    | `SCOT-{id}`                  |
//! | 396  | UnusualItemFlag                   | `UFLAG-{id}`                 |
//! | 397  | AnalyticalRelationship            | `AREL-{id}`                  |
//! | 398  | AccountingEstimate                | `EST-{id}`                   |
//! | 399  | SubsequentEvent                   | `SEVT-{id}`                  |
//! | 401  | ServiceOrganization               | `SORG-{id}`                  |
//! | 402  | SocReport                         | `SOC-{id}`                   |
//!
//! ## Layer 3 — Financial
//!
//! | Code | Entity                    | External-ID pattern             |
//! |------|---------------------------|---------------------------------|
//! | 485  | ConsolidationSchedule     | `CONSOL-{period}`               |
//! | 486  | OperatingSegment          | `SEG-{segment_id}`              |
//! | 487  | EclModel                  | `ECL-{id}`                      |
//! | 488  | Provision                 | `PROV-{id}`                     |
//! | 489  | DefinedBenefitPlan        | `DBP-{id}`                      |
//! | 490  | StockGrant                | `SGRANT-{id}`                   |
//! | 491  | TemporaryDifference       | `TDIFF-{id}`                    |
//! | 492  | BusinessCombination       | `BC-{id}`                       |
//! | 493  | NciMeasurement            | `NCI-{entity_code}-{period}`    |
//! | 494  | FinancialStatementNote    | `FSNOTE-{note_number}`          |
//! | 495  | CurrencyTranslationResult | `CTR-{entity_code}-{period}`    |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes graph nodes for all 28 v1.3.0 entity types.
pub struct V130NodeSynthesizer;

impl NodeSynthesizer for V130NodeSynthesizer {
    fn name(&self) -> &'static str {
        "v130_entities"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();

        nodes.extend(synthesize_l1_governance(ctx));
        nodes.extend(synthesize_l2_process(ctx));
        nodes.extend(synthesize_l3_financial(ctx));

        debug!(
            "V130NodeSynthesizer: produced {} nodes total",
            nodes.len()
        );

        Ok(nodes)
    }
}

// ============================================================================
// Layer 1 — Governance (codes 380–392)
// ============================================================================

fn synthesize_l1_governance(ctx: &mut NodeSynthesisContext<'_>) -> Vec<ExportNode> {
    let mut nodes = Vec::new();
    let audit = &ctx.ds_result.audit;

    // CombinedRiskAssessment (380)
    for cra in &audit.combined_risk_assessments {
        let ext_id = format!("CRA-{}", cra.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(cra.entity_code));
        props.insert("accountArea".into(), serde_json::json!(cra.account_area));
        props.insert(
            "assertion".into(),
            serde_json::to_value(&cra.assertion).unwrap_or_default(),
        );
        props.insert(
            "inherentRisk".into(),
            serde_json::to_value(&cra.inherent_risk).unwrap_or_default(),
        );
        props.insert(
            "controlRisk".into(),
            serde_json::to_value(&cra.control_risk).unwrap_or_default(),
        );
        props.insert(
            "combinedRisk".into(),
            serde_json::to_value(&cra.combined_risk).unwrap_or_default(),
        );
        props.insert(
            "significantRisk".into(),
            serde_json::Value::Bool(cra.significant_risk),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 380,
            node_type_name: "combined_risk_assessment".into(),
            label: format!("CRA: {} - {}", cra.account_area, cra.assertion),
            layer: 1,
            properties: props,
        });
    }

    // MaterialityCalculation (381)
    for mat in &audit.materiality_calculations {
        let ext_id = format!("MAT-{}-{}", mat.entity_code, mat.period);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(mat.entity_code));
        props.insert("period".into(), serde_json::json!(mat.period));
        props.insert(
            "benchmark".into(),
            serde_json::to_value(&mat.benchmark).unwrap_or_default(),
        );
        props.insert(
            "overallMateriality".into(),
            serde_json::Value::String(mat.overall_materiality.to_string()),
        );
        props.insert(
            "performanceMateriality".into(),
            serde_json::Value::String(mat.performance_materiality.to_string()),
        );
        props.insert(
            "clearlyTrivial".into(),
            serde_json::Value::String(mat.clearly_trivial.to_string()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 381,
            node_type_name: "materiality_calculation".into(),
            label: format!("Materiality: {} {}", mat.entity_code, mat.period),
            layer: 1,
            properties: props,
        });
    }

    // AuditOpinion (382)
    for opinion in &audit.audit_opinions {
        let ext_id = format!("OPINION-{}", opinion.opinion_id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityName".into(), serde_json::json!(opinion.entity_name));
        props.insert(
            "opinionType".into(),
            serde_json::to_value(&opinion.opinion_type).unwrap_or_default(),
        );
        props.insert(
            "opinionDate".into(),
            serde_json::json!(opinion.opinion_date.to_string()),
        );
        props.insert(
            "goingConcernConclusion".into(),
            serde_json::to_value(&opinion.going_concern_conclusion).unwrap_or_default(),
        );
        props.insert(
            "eqcrPerformed".into(),
            serde_json::Value::Bool(opinion.eqcr_performed),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 382,
            node_type_name: "audit_opinion".into(),
            label: format!("Opinion: {} {:?}", opinion.entity_name, opinion.opinion_type),
            layer: 1,
            properties: props,
        });
    }

    // KeyAuditMatter (383)
    for kam in &audit.key_audit_matters {
        let ext_id = format!("KAM-{}", kam.kam_id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("title".into(), serde_json::json!(kam.title));
        props.insert(
            "financialStatementArea".into(),
            serde_json::json!(kam.financial_statement_area),
        );
        props.insert(
            "rommLevel".into(),
            serde_json::to_value(&kam.romm_level).unwrap_or_default(),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 383,
            node_type_name: "key_audit_matter".into(),
            label: format!("KAM: {}", kam.title),
            layer: 1,
            properties: props,
        });
    }

    // Sox302Certification (384)
    for cert in &audit.sox_302_certifications {
        let ext_id = format!("SOX302-{}", cert.certification_id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("companyCode".into(), serde_json::json!(cert.company_code));
        props.insert(
            "fiscalYear".into(),
            serde_json::json!(cert.fiscal_year),
        );
        props.insert(
            "certifierRole".into(),
            serde_json::to_value(&cert.certifier_role).unwrap_or_default(),
        );
        props.insert(
            "certifierName".into(),
            serde_json::json!(cert.certifier_name),
        );
        props.insert(
            "disclosureControlsEffective".into(),
            serde_json::Value::Bool(cert.disclosure_controls_effective),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 384,
            node_type_name: "sox_302_certification".into(),
            label: format!(
                "SOX 302: {} {} {:?}",
                cert.company_code, cert.fiscal_year, cert.certifier_role
            ),
            layer: 1,
            properties: props,
        });
    }

    // Sox404Assessment (385)
    for assessment in &audit.sox_404_assessments {
        let ext_id = format!("SOX404-{}", assessment.assessment_id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert(
            "companyCode".into(),
            serde_json::json!(assessment.company_code),
        );
        props.insert(
            "fiscalYear".into(),
            serde_json::json!(assessment.fiscal_year),
        );
        props.insert(
            "icfrEffective".into(),
            serde_json::Value::Bool(assessment.icfr_effective),
        );
        props.insert(
            "materialWeaknessCount".into(),
            serde_json::json!(assessment.material_weaknesses.len()),
        );
        props.insert(
            "keyControlsTested".into(),
            serde_json::json!(assessment.key_controls_tested),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 385,
            node_type_name: "sox_404_assessment".into(),
            label: format!(
                "SOX 404: {} {}",
                assessment.company_code, assessment.fiscal_year
            ),
            layer: 1,
            properties: props,
        });
    }

    // GoingConcernAssessment (386)
    for gc in &audit.going_concern_assessments {
        let ext_id = format!("GC-{}-{}", gc.entity_code, gc.assessment_period);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(gc.entity_code));
        props.insert("period".into(), serde_json::json!(gc.assessment_period));
        props.insert(
            "conclusion".into(),
            serde_json::to_value(&gc.auditor_conclusion).unwrap_or_default(),
        );
        props.insert(
            "materialUncertainty".into(),
            serde_json::Value::Bool(gc.material_uncertainty_exists),
        );
        props.insert(
            "indicatorCount".into(),
            serde_json::json!(gc.indicators.len()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 386,
            node_type_name: "going_concern_assessment".into(),
            label: format!("GC: {} {}", gc.entity_code, gc.assessment_period),
            layer: 1,
            properties: props,
        });
    }

    // ComponentAuditor (387)
    for caud in &audit.component_auditors {
        let ext_id = format!("CAUD-{}", caud.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("firmName".into(), serde_json::json!(caud.firm_name));
        props.insert(
            "jurisdiction".into(),
            serde_json::json!(caud.jurisdiction),
        );
        props.insert(
            "independenceConfirmed".into(),
            serde_json::Value::Bool(caud.independence_confirmed),
        );
        props.insert(
            "competenceAssessment".into(),
            serde_json::to_value(&caud.competence_assessment).unwrap_or_default(),
        );
        props.insert(
            "assignedEntityCount".into(),
            serde_json::json!(caud.assigned_entities.len()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 387,
            node_type_name: "component_auditor".into(),
            label: format!("Comp. Auditor: {} ({})", caud.firm_name, caud.jurisdiction),
            layer: 1,
            properties: props,
        });
    }

    // GroupAuditPlan (388) — at most one
    if let Some(gap) = &audit.group_audit_plan {
        let ext_id = format!("GAP-{}", gap.engagement_id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert(
            "engagementId".into(),
            serde_json::json!(gap.engagement_id),
        );
        props.insert(
            "groupMateriality".into(),
            serde_json::Value::String(gap.group_materiality.to_string()),
        );
        props.insert(
            "aggregationRisk".into(),
            serde_json::to_value(&gap.aggregation_risk).unwrap_or_default(),
        );
        props.insert(
            "significantComponentCount".into(),
            serde_json::json!(gap.significant_components.len()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 388,
            node_type_name: "group_audit_plan".into(),
            label: format!("Group Audit Plan: {}", gap.engagement_id),
            layer: 1,
            properties: props,
        });
    }

    // ComponentInstruction (389)
    for cinst in &audit.component_instructions {
        let ext_id = format!("CINST-{}", cinst.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(cinst.entity_code));
        props.insert(
            "componentAuditorId".into(),
            serde_json::json!(cinst.component_auditor_id),
        );
        props.insert(
            "materialityAllocated".into(),
            serde_json::Value::String(cinst.materiality_allocated.to_string()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 389,
            node_type_name: "component_instruction".into(),
            label: format!("Comp. Instruction: {} -> {}", cinst.component_auditor_id, cinst.entity_code),
            layer: 1,
            properties: props,
        });
    }

    // ComponentAuditorReport (390)
    for crpt in &audit.component_reports {
        let ext_id = format!("CRPT-{}", crpt.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(crpt.entity_code));
        props.insert(
            "componentAuditorId".into(),
            serde_json::json!(crpt.component_auditor_id),
        );
        props.insert(
            "misstatementsFound".into(),
            serde_json::json!(crpt.misstatements_identified.len()),
        );
        props.insert(
            "scopeLimitations".into(),
            serde_json::json!(crpt.scope_limitations.len()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 390,
            node_type_name: "component_auditor_report".into(),
            label: format!("Comp. Report: {}", crpt.entity_code),
            layer: 1,
            properties: props,
        });
    }

    // EngagementLetter (391)
    for elet in &audit.engagement_letters {
        let ext_id = format!("ELET-{}", elet.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert(
            "engagementId".into(),
            serde_json::json!(elet.engagement_id),
        );
        props.insert("addressee".into(), serde_json::json!(elet.addressee));
        props.insert("date".into(), serde_json::json!(elet.date.to_string()));
        props.insert(
            "scope".into(),
            serde_json::to_value(&elet.scope).unwrap_or_default(),
        );
        props.insert(
            "applicableFramework".into(),
            serde_json::json!(elet.applicable_framework),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 391,
            node_type_name: "engagement_letter".into(),
            label: format!("Engagement Letter: {}", elet.engagement_id),
            layer: 1,
            properties: props,
        });
    }

    // GroupStructure (392) — at most one
    if let Some(gs) = &ctx.ds_result.intercompany.group_structure {
        let ext_id = format!("GSTRUCT-{}", gs.parent_entity);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert(
            "parentEntity".into(),
            serde_json::json!(gs.parent_entity),
        );
        props.insert(
            "subsidiaryCount".into(),
            serde_json::json!(gs.subsidiaries.len()),
        );
        props.insert(
            "associateCount".into(),
            serde_json::json!(gs.associates.len()),
        );
        props.insert(
            "totalEntityCount".into(),
            serde_json::json!(gs.entity_count()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 392,
            node_type_name: "group_structure".into(),
            label: format!("Group Structure: {}", gs.parent_entity),
            layer: 1,
            properties: props,
        });
    }

    nodes
}

// ============================================================================
// Layer 2 — Process / Audit Procedure (codes 393–402)
// ============================================================================

fn synthesize_l2_process(ctx: &mut NodeSynthesisContext<'_>) -> Vec<ExportNode> {
    let mut nodes = Vec::new();
    let audit = &ctx.ds_result.audit;

    // SamplingPlan (393)
    for plan in &audit.sampling_plans {
        let ext_id = format!("SPLAN-{}", plan.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(plan.entity_code));
        props.insert("accountArea".into(), serde_json::json!(plan.account_area));
        props.insert("assertion".into(), serde_json::json!(plan.assertion));
        props.insert(
            "methodology".into(),
            serde_json::to_value(&plan.methodology).unwrap_or_default(),
        );
        props.insert(
            "populationSize".into(),
            serde_json::json!(plan.population_size),
        );
        props.insert(
            "sampleSize".into(),
            serde_json::json!(plan.sample_size),
        );
        props.insert(
            "craLevel".into(),
            serde_json::json!(plan.cra_level),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 393,
            node_type_name: "sampling_plan".into(),
            label: format!("Sampling Plan: {} {}", plan.account_area, plan.assertion),
            layer: 2,
            properties: props,
        });
    }

    // SampledItem (394)
    for item in &audit.sampled_items {
        let ext_id = format!("SITM-{}", item.item_id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("itemId".into(), serde_json::json!(item.item_id));
        props.insert(
            "amount".into(),
            serde_json::Value::String(item.amount.to_string()),
        );
        props.insert(
            "selectionType".into(),
            serde_json::to_value(&item.selection_type).unwrap_or_default(),
        );
        props.insert("tested".into(), serde_json::Value::Bool(item.tested));
        props.insert(
            "misstatementFound".into(),
            serde_json::Value::Bool(item.misstatement_found),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 394,
            node_type_name: "sampled_item".into(),
            label: format!("Sampled Item: {}", item.item_id),
            layer: 2,
            properties: props,
        });
    }

    // SignificantClassOfTransactions (395)
    for scot in &audit.significant_transaction_classes {
        let ext_id = format!("SCOT-{}", scot.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(scot.entity_code));
        props.insert("scotName".into(), serde_json::json!(scot.scot_name));
        props.insert(
            "businessProcess".into(),
            serde_json::json!(scot.business_process),
        );
        props.insert(
            "significanceLevel".into(),
            serde_json::to_value(&scot.significance_level).unwrap_or_default(),
        );
        props.insert(
            "transactionType".into(),
            serde_json::to_value(&scot.transaction_type).unwrap_or_default(),
        );
        props.insert(
            "monetaryValue".into(),
            serde_json::Value::String(scot.monetary_value.to_string()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 395,
            node_type_name: "significant_class_of_transactions".into(),
            label: format!("SCOT: {}", scot.scot_name),
            layer: 2,
            properties: props,
        });
    }

    // UnusualItemFlag (396)
    for uflag in &audit.unusual_items {
        let ext_id = format!("UFLAG-{}", uflag.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(uflag.entity_code));
        props.insert(
            "journalEntryId".into(),
            serde_json::json!(uflag.journal_entry_id),
        );
        props.insert(
            "severity".into(),
            serde_json::to_value(&uflag.severity).unwrap_or_default(),
        );
        props.insert(
            "investigationRequired".into(),
            serde_json::Value::Bool(uflag.investigation_required),
        );
        props.insert(
            "isLabeledAnomaly".into(),
            serde_json::Value::Bool(uflag.is_labeled_anomaly),
        );
        props.insert("actualValue".into(), serde_json::json!(uflag.actual_value));
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 396,
            node_type_name: "unusual_item_flag".into(),
            label: format!("Unusual Flag: {} {:?}", uflag.entity_code, uflag.severity),
            layer: 2,
            properties: props,
        });
    }

    // AnalyticalRelationship (397)
    for arel in &audit.analytical_relationships {
        let ext_id = format!("AREL-{}", arel.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(arel.entity_code));
        props.insert(
            "relationshipName".into(),
            serde_json::json!(arel.relationship_name),
        );
        props.insert("accountArea".into(), serde_json::json!(arel.account_area));
        props.insert(
            "relationshipType".into(),
            serde_json::to_value(&arel.relationship_type).unwrap_or_default(),
        );
        props.insert(
            "withinExpectedRange".into(),
            serde_json::Value::Bool(arel.within_expected_range),
        );
        props.insert("formula".into(), serde_json::json!(arel.formula));
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 397,
            node_type_name: "analytical_relationship".into(),
            label: format!("Analytical: {} — {}", arel.relationship_name, arel.entity_code),
            layer: 2,
            properties: props,
        });
    }

    // AccountingEstimate (398)
    for est in &audit.accounting_estimates {
        let ext_id = format!("EST-{}", est.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(est.entity_code));
        props.insert(
            "estimateType".into(),
            serde_json::to_value(&est.estimate_type).unwrap_or_default(),
        );
        props.insert("description".into(), serde_json::json!(est.description));
        props.insert(
            "managementPointEstimate".into(),
            serde_json::Value::String(est.management_point_estimate.to_string()),
        );
        props.insert(
            "estimationUncertainty".into(),
            serde_json::to_value(&est.estimation_uncertainty).unwrap_or_default(),
        );
        props.insert(
            "complexity".into(),
            serde_json::to_value(&est.complexity).unwrap_or_default(),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 398,
            node_type_name: "accounting_estimate".into(),
            label: format!("Estimate: {:?} — {}", est.estimate_type, est.entity_code),
            layer: 2,
            properties: props,
        });
    }

    // SubsequentEvent (399)
    for sevt in &audit.subsequent_events {
        let ext_id = format!("SEVT-{}", sevt.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(sevt.entity_code));
        props.insert(
            "eventDate".into(),
            serde_json::json!(sevt.event_date.to_string()),
        );
        props.insert(
            "eventType".into(),
            serde_json::to_value(&sevt.event_type).unwrap_or_default(),
        );
        props.insert(
            "classification".into(),
            serde_json::to_value(&sevt.classification).unwrap_or_default(),
        );
        props.insert(
            "disclosureRequired".into(),
            serde_json::Value::Bool(sevt.disclosure_required),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 399,
            node_type_name: "subsequent_event".into(),
            label: format!("Subsequent Event: {:?} — {}", sevt.event_type, sevt.entity_code),
            layer: 2,
            properties: props,
        });
    }

    // ServiceOrganization (401)
    for sorg in &audit.service_organizations {
        let ext_id = format!("SORG-{}", sorg.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("name".into(), serde_json::json!(sorg.name));
        props.insert(
            "serviceType".into(),
            serde_json::to_value(&sorg.service_type).unwrap_or_default(),
        );
        props.insert(
            "entitiesServed".into(),
            serde_json::json!(sorg.entities_served.len()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 401,
            node_type_name: "service_organization".into(),
            label: format!("Service Org: {}", sorg.name),
            layer: 2,
            properties: props,
        });
    }

    // SocReport (402)
    for soc in &audit.soc_reports {
        let ext_id = format!("SOC-{}", soc.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert(
            "serviceOrgId".into(),
            serde_json::json!(soc.service_org_id),
        );
        props.insert(
            "reportType".into(),
            serde_json::to_value(&soc.report_type).unwrap_or_default(),
        );
        props.insert(
            "opinionType".into(),
            serde_json::to_value(&soc.opinion_type).unwrap_or_default(),
        );
        props.insert(
            "periodStart".into(),
            serde_json::json!(soc.report_period_start.to_string()),
        );
        props.insert(
            "periodEnd".into(),
            serde_json::json!(soc.report_period_end.to_string()),
        );
        props.insert(
            "exceptionCount".into(),
            serde_json::json!(soc.exceptions_noted.len()),
        );
        props.insert("processFamily".into(), serde_json::json!("AUDIT"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 402,
            node_type_name: "soc_report".into(),
            label: format!("SOC Report: {} {:?}", soc.service_org_id, soc.report_type),
            layer: 2,
            properties: props,
        });
    }

    nodes
}

// ============================================================================
// Layer 3 — Financial (codes 485–495)
// ============================================================================

fn synthesize_l3_financial(ctx: &mut NodeSynthesisContext<'_>) -> Vec<ExportNode> {
    let mut nodes = Vec::new();
    let fr = &ctx.ds_result.financial_reporting;
    let acct = &ctx.ds_result.accounting_standards;
    let hr = &ctx.ds_result.hr;
    let tax = &ctx.ds_result.tax;
    let ic = &ctx.ds_result.intercompany;

    // ConsolidationSchedule (485)
    for sched in &fr.consolidation_schedules {
        let ext_id = format!("CONSOL-{}", sched.period);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("period".into(), serde_json::json!(sched.period));
        props.insert(
            "lineItemCount".into(),
            serde_json::json!(sched.line_items.len()),
        );
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 485,
            node_type_name: "consolidation_schedule".into(),
            label: format!("Consolidation: {}", sched.period),
            layer: 3,
            properties: props,
        });
    }

    // OperatingSegment (486)
    for seg in &fr.segment_reports {
        let ext_id = format!("SEG-{}", seg.segment_id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("name".into(), serde_json::json!(seg.name));
        props.insert("period".into(), serde_json::json!(seg.period));
        props.insert(
            "companyCode".into(),
            serde_json::json!(seg.company_code),
        );
        props.insert(
            "segmentType".into(),
            serde_json::to_value(&seg.segment_type).unwrap_or_default(),
        );
        props.insert(
            "revenueExternal".into(),
            serde_json::Value::String(seg.revenue_external.to_string()),
        );
        props.insert(
            "operatingProfit".into(),
            serde_json::Value::String(seg.operating_profit.to_string()),
        );
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 486,
            node_type_name: "operating_segment".into(),
            label: format!("Segment: {} ({})", seg.name, seg.period),
            layer: 3,
            properties: props,
        });
    }

    // EclModel (487)
    for ecl in &acct.ecl_models {
        let ext_id = format!("ECL-{}", ecl.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(ecl.entity_code));
        props.insert("framework".into(), serde_json::json!(ecl.framework));
        props.insert(
            "approach".into(),
            serde_json::to_value(&ecl.approach).unwrap_or_default(),
        );
        props.insert(
            "totalEcl".into(),
            serde_json::Value::String(ecl.total_ecl.to_string()),
        );
        props.insert(
            "totalExposure".into(),
            serde_json::Value::String(ecl.total_exposure.to_string()),
        );
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 487,
            node_type_name: "ecl_model".into(),
            label: format!("ECL: {} ({})", ecl.entity_code, ecl.framework),
            layer: 3,
            properties: props,
        });
    }

    // Provision (488)
    for prov in &acct.provisions {
        let ext_id = format!("PROV-{}", prov.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(prov.entity_code));
        props.insert(
            "provisionType".into(),
            serde_json::to_value(&prov.provision_type).unwrap_or_default(),
        );
        props.insert("description".into(), serde_json::json!(prov.description));
        props.insert(
            "bestEstimate".into(),
            serde_json::Value::String(prov.best_estimate.to_string()),
        );
        props.insert("framework".into(), serde_json::json!(prov.framework));
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 488,
            node_type_name: "provision".into(),
            label: format!("Provision: {:?} — {}", prov.provision_type, prov.entity_code),
            layer: 3,
            properties: props,
        });
    }

    // DefinedBenefitPlan (489)
    for plan in &hr.pension_plans {
        let ext_id = format!("DBP-{}", plan.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(plan.entity_code));
        props.insert("planName".into(), serde_json::json!(plan.plan_name));
        props.insert(
            "planType".into(),
            serde_json::to_value(&plan.plan_type).unwrap_or_default(),
        );
        props.insert(
            "participantCount".into(),
            serde_json::json!(plan.participant_count),
        );
        props.insert("currency".into(), serde_json::json!(plan.currency));
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 489,
            node_type_name: "defined_benefit_plan".into(),
            label: format!("Pension Plan: {}", plan.plan_name),
            layer: 3,
            properties: props,
        });
    }

    // StockGrant (490)
    for grant in &hr.stock_grants {
        let ext_id = format!("SGRANT-{}", grant.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(grant.entity_code));
        props.insert("employeeId".into(), serde_json::json!(grant.employee_id));
        props.insert(
            "instrumentType".into(),
            serde_json::to_value(&grant.instrument_type).unwrap_or_default(),
        );
        props.insert("quantity".into(), serde_json::json!(grant.quantity));
        props.insert(
            "totalGrantValue".into(),
            serde_json::Value::String(grant.total_grant_value.to_string()),
        );
        props.insert(
            "grantDate".into(),
            serde_json::json!(grant.grant_date.to_string()),
        );
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 490,
            node_type_name: "stock_grant".into(),
            label: format!("Stock Grant: {} {:?}", grant.entity_code, grant.instrument_type),
            layer: 3,
            properties: props,
        });
    }

    // TemporaryDifference (491)
    for tdiff in &tax.deferred_tax.temporary_differences {
        let ext_id = format!("TDIFF-{}", tdiff.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(tdiff.entity_code));
        props.insert("account".into(), serde_json::json!(tdiff.account));
        props.insert("description".into(), serde_json::json!(tdiff.description));
        props.insert(
            "deferredType".into(),
            serde_json::to_value(&tdiff.deferred_type).unwrap_or_default(),
        );
        props.insert(
            "difference".into(),
            serde_json::Value::String(tdiff.difference.to_string()),
        );
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 491,
            node_type_name: "temporary_difference".into(),
            label: format!("Temp. Diff: {} — {}", tdiff.description, tdiff.entity_code),
            layer: 3,
            properties: props,
        });
    }

    // BusinessCombination (492)
    for bc in &acct.business_combinations {
        let ext_id = format!("BC-{}", bc.id);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert(
            "acquirerEntity".into(),
            serde_json::json!(bc.acquirer_entity),
        );
        props.insert("acquireeName".into(), serde_json::json!(bc.acquiree_name));
        props.insert(
            "acquisitionDate".into(),
            serde_json::json!(bc.acquisition_date.to_string()),
        );
        props.insert(
            "goodwill".into(),
            serde_json::Value::String(bc.goodwill.to_string()),
        );
        props.insert("framework".into(), serde_json::json!(bc.framework));
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 492,
            node_type_name: "business_combination".into(),
            label: format!(
                "Business Combination: {} acquires {}",
                bc.acquirer_entity, bc.acquiree_name
            ),
            layer: 3,
            properties: props,
        });
    }

    // NciMeasurement (493)
    for nci in &ic.nci_measurements {
        let ext_id = format!("NCI-{}", nci.entity_code);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(nci.entity_code));
        props.insert(
            "nciPercentage".into(),
            serde_json::Value::String(nci.nci_percentage.to_string()),
        );
        props.insert(
            "nciShareNetAssets".into(),
            serde_json::Value::String(nci.nci_share_net_assets.to_string()),
        );
        props.insert(
            "nciShareProfit".into(),
            serde_json::Value::String(nci.nci_share_profit.to_string()),
        );
        props.insert(
            "totalNci".into(),
            serde_json::Value::String(nci.total_nci.to_string()),
        );
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 493,
            node_type_name: "nci_measurement".into(),
            label: format!("NCI: {}", nci.entity_code),
            layer: 3,
            properties: props,
        });
    }

    // FinancialStatementNote (494)
    for (idx, note) in fr.notes_to_financial_statements.iter().enumerate() {
        let ext_id = format!("FSNOTE-{idx}-{}", note.note_number);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert(
            "noteNumber".into(),
            serde_json::json!(note.note_number),
        );
        props.insert("title".into(), serde_json::json!(note.title));
        props.insert(
            "category".into(),
            serde_json::to_value(&note.category).unwrap_or_default(),
        );
        props.insert(
            "sectionCount".into(),
            serde_json::json!(note.content_sections.len()),
        );
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 494,
            node_type_name: "financial_statement_note".into(),
            label: format!("Note {}: {}", note.note_number, note.title),
            layer: 3,
            properties: props,
        });
    }

    // CurrencyTranslationResult (495)
    for ctr in &acct.currency_translation_results {
        let ext_id = format!("CTR-{}-{}", ctr.entity_code, ctr.period);
        let numeric_id = ctx.id_map.get_or_insert(&ext_id);

        let mut props = HashMap::new();
        props.insert("entityCode".into(), serde_json::json!(ctr.entity_code));
        props.insert("period".into(), serde_json::json!(ctr.period));
        props.insert(
            "functionalCurrency".into(),
            serde_json::json!(ctr.functional_currency),
        );
        props.insert(
            "presentationCurrency".into(),
            serde_json::json!(ctr.presentation_currency),
        );
        props.insert(
            "translationMethod".into(),
            serde_json::to_value(&ctr.translation_method).unwrap_or_default(),
        );
        props.insert(
            "ctaAmount".into(),
            serde_json::Value::String(ctr.cta_amount.to_string()),
        );
        props.insert("processFamily".into(), serde_json::json!("FINANCIAL"));

        nodes.push(ExportNode {
            id: Some(numeric_id),
            node_type: 495,
            node_type_name: "currency_translation_result".into(),
            label: format!(
                "CTA: {} {} -> {}",
                ctr.entity_code, ctr.functional_currency, ctr.presentation_currency
            ),
            layer: 3,
            properties: props,
        });
    }

    nodes
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_name() {
        let s = V130NodeSynthesizer;
        assert_eq!(s.name(), "v130_entities");
    }
}
