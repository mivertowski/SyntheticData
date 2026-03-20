//! v1.3.0 edge synthesizer.
//!
//! Produces edges linking the 28 v1.3.0 entity types to each other and to
//! existing nodes (engagements, journal entries, audit workpapers, GL accounts,
//! employees, and entities).
//!
//! ## Edge Types Produced
//!
//! | Code | Name                        | Direction                                          |
//! |------|-----------------------------|----------------------------------------------------|
//! | 160  | CRA_FOR_ENTITY              | combined_risk_assessment → (entity engagement)     |
//! | 161  | MATERIALITY_FOR_ENTITY      | materiality_calculation → (engagement by entity)   |
//! | 162  | OPINION_FOR_ENGAGEMENT      | audit_opinion → engagement                         |
//! | 163  | KAM_IN_OPINION              | key_audit_matter → audit_opinion                   |
//! | 164  | SOX302_FOR_ENTITY           | sox_302_certification → (company entity)           |
//! | 165  | SOX404_FOR_ENTITY           | sox_404_assessment → (company entity)              |
//! | 166  | GC_FOR_ENTITY               | going_concern_assessment → (entity engagement)     |
//! | 167  | COMPONENT_AUDITOR_ASSIGNED  | component_auditor → group_audit_plan               |
//! | 168  | INSTRUCTION_TO_AUDITOR      | component_instruction → component_auditor          |
//! | 169  | REPORT_FROM_AUDITOR         | component_auditor_report → component_instruction   |
//! | 170  | ELET_FOR_ENGAGEMENT         | engagement_letter → engagement                     |
//! | 171  | SPLAN_FOR_CRA               | sampling_plan → combined_risk_assessment           |
//! | 172  | SAMPLED_ITEM_IN_PLAN        | sampled_item → sampling_plan (via item_id)         |
//! | 173  | SCOT_FOR_ENTITY             | significant_class_of_transactions → entity engs   |
//! | 174  | UFLAG_ON_JE                 | unusual_item_flag → journal_entry                  |
//! | 175  | AREL_FOR_ENTITY             | analytical_relationship → entity engagement        |
//! | 176  | ESTIMATE_FOR_ENTITY         | accounting_estimate → entity engagement            |
//! | 177  | SEVT_FOR_ENTITY             | subsequent_event → entity engagement               |
//! | 178  | SOC_FOR_SERVICE_ORG         | soc_report → service_organization                  |
//! | 179  | SEGMENT_PARENT              | operating_segment → group_structure                |
//! | 180  | ECL_FOR_ENTITY              | ecl_model → entity engagement                      |
//! | 181  | PROVISION_FOR_ENTITY        | provision → entity engagement                      |
//! | 182  | PENSION_FOR_ENTITY          | defined_benefit_plan → entity engagement           |
//! | 183  | STOCK_GRANT_FOR_EMPLOYEE    | stock_grant → employee                             |
//! | 184  | TDIFF_FOR_ACCOUNT           | temporary_difference → gl_account                  |
//! | 185  | BC_ACQUIRER                 | business_combination → acquirer entity engagement  |
//! | 186  | NCI_FOR_GROUP               | nci_measurement → group_structure                  |
//! | 187  | GSTRUCT_PARENT_ENG          | group_structure → engagement (of parent entity)    |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

// ============================================================================
// Edge type constants
// ============================================================================

const CRA_FOR_ENTITY: u32 = 160;
const MATERIALITY_FOR_ENTITY: u32 = 161;
const OPINION_FOR_ENGAGEMENT: u32 = 162;
const KAM_IN_OPINION: u32 = 163;
const SOX302_FOR_ENTITY: u32 = 164;
const SOX404_FOR_ENTITY: u32 = 165;
const GC_FOR_ENTITY: u32 = 166;
const COMPONENT_AUDITOR_ASSIGNED: u32 = 167;
const INSTRUCTION_TO_AUDITOR: u32 = 168;
const REPORT_FROM_AUDITOR: u32 = 169;
const ELET_FOR_ENGAGEMENT: u32 = 170;
const SPLAN_FOR_CRA: u32 = 171;
#[allow(dead_code)] // sampled items currently lack a back-link to their plan
const SAMPLED_ITEM_IN_PLAN: u32 = 172;
const SCOT_FOR_ENTITY: u32 = 173;
const UFLAG_ON_JE: u32 = 174;
const AREL_FOR_ENTITY: u32 = 175;
const ESTIMATE_FOR_ENTITY: u32 = 176;
const SEVT_FOR_ENTITY: u32 = 177;
const SOC_FOR_SERVICE_ORG: u32 = 178;
const SEGMENT_PARENT: u32 = 179;
const ECL_FOR_ENTITY: u32 = 180;
const PROVISION_FOR_ENTITY: u32 = 181;
const PENSION_FOR_ENTITY: u32 = 182;
const STOCK_GRANT_FOR_EMPLOYEE: u32 = 183;
const TDIFF_FOR_ACCOUNT: u32 = 184;
const BC_ACQUIRER: u32 = 185;
const NCI_FOR_GROUP: u32 = 186;
const GSTRUCT_PARENT_ENG: u32 = 187;

/// Edge synthesizer for all v1.3.0 cross-entity relationships.
pub struct V130EdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for V130EdgeSynthesizer {
    fn name(&self) -> &'static str {
        "v130_edges"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let mut edges = Vec::new();

        edges.extend(synthesize_cra_for_entity(ctx));
        edges.extend(synthesize_materiality_for_entity(ctx));
        edges.extend(synthesize_opinion_for_engagement(ctx));
        edges.extend(synthesize_kam_in_opinion(ctx));
        edges.extend(synthesize_sox302_for_entity(ctx));
        edges.extend(synthesize_sox404_for_entity(ctx));
        edges.extend(synthesize_gc_for_entity(ctx));
        edges.extend(synthesize_component_auditor_assigned(ctx));
        edges.extend(synthesize_instruction_to_auditor(ctx));
        edges.extend(synthesize_report_from_auditor(ctx));
        edges.extend(synthesize_elet_for_engagement(ctx));
        edges.extend(synthesize_splan_for_cra(ctx));
        // SAMPLED_ITEM_IN_PLAN: SampledItem has no back-link to SamplingPlan id → 0 edges
        edges.extend(synthesize_scot_for_entity(ctx));
        edges.extend(synthesize_uflag_on_je(ctx));
        edges.extend(synthesize_arel_for_entity(ctx));
        edges.extend(synthesize_estimate_for_entity(ctx));
        edges.extend(synthesize_sevt_for_entity(ctx));
        edges.extend(synthesize_soc_for_service_org(ctx));
        edges.extend(synthesize_segment_parent(ctx));
        edges.extend(synthesize_ecl_for_entity(ctx));
        edges.extend(synthesize_provision_for_entity(ctx));
        edges.extend(synthesize_pension_for_entity(ctx));
        edges.extend(synthesize_stock_grant_for_employee(ctx));
        edges.extend(synthesize_tdiff_for_account(ctx));
        edges.extend(synthesize_bc_acquirer(ctx));
        edges.extend(synthesize_nci_for_group(ctx));
        edges.extend(synthesize_gstruct_parent_eng(ctx));

        debug!("V130EdgeSynthesizer produced {} total edges", edges.len());
        Ok(edges)
    }
}

// ============================================================================
// Helper — look up a first-matched engagement for an entity code.
// ============================================================================

/// Build a map from entity_code → engagement_ref for the audit engagements.
fn engagement_ref_by_entity(ctx: &EdgeSynthesisContext<'_>) -> HashMap<String, String> {
    ctx.ds_result
        .audit
        .engagements
        .iter()
        .map(|eng| (eng.client_entity_id.clone(), eng.engagement_ref.clone()))
        .collect()
}

// ============================================================================
// Edge synthesis helpers (one per edge type)
// ============================================================================

/// CRA_FOR_ENTITY (160): combined_risk_assessment → engagement (of same entity).
fn synthesize_cra_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for cra in &ctx.ds_result.audit.combined_risk_assessments {
        let cra_ext_id = format!("CRA-{}", cra.id);
        let Some(cra_node) = ctx.id_map.get(&cra_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&cra.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: cra_node,
            target: eng_node,
            edge_type: CRA_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("CRA_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// MATERIALITY_FOR_ENTITY (161): materiality_calculation → engagement (of same entity).
fn synthesize_materiality_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for mat in &ctx.ds_result.audit.materiality_calculations {
        let mat_ext_id = format!("MAT-{}-{}", mat.entity_code, mat.period);
        let Some(mat_node) = ctx.id_map.get(&mat_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&mat.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: mat_node,
            target: eng_node,
            edge_type: MATERIALITY_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("MATERIALITY_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// OPINION_FOR_ENGAGEMENT (162): audit_opinion → engagement.
///
/// `AuditOpinion.engagement_id` is a UUID; engagements are keyed by `engagement_ref`.
/// We resolve via an `engagement_id → engagement_ref` map.
fn synthesize_opinion_for_engagement(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let audit = &ctx.ds_result.audit;
    // Build engagement_id (UUID string) → engagement_ref map
    let eng_id_to_ref: HashMap<String, &str> = audit
        .engagements
        .iter()
        .map(|eng| (eng.engagement_id.to_string(), eng.engagement_ref.as_str()))
        .collect();

    let mut edges = Vec::new();

    for opinion in &audit.audit_opinions {
        let opinion_ext_id = format!("OPINION-{}", opinion.opinion_id);
        let Some(opinion_node) = ctx.id_map.get(&opinion_ext_id) else {
            continue;
        };
        let eng_uuid_str = opinion.engagement_id.to_string();
        let Some(&eng_ref) = eng_id_to_ref.get(&eng_uuid_str) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref) else {
            continue;
        };

        edges.push(ExportEdge {
            source: opinion_node,
            target: eng_node,
            edge_type: OPINION_FOR_ENGAGEMENT,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("OPINION_FOR_ENGAGEMENT: {} edges", edges.len());
    edges
}

/// KAM_IN_OPINION (163): key_audit_matter → audit_opinion.
///
/// `AuditOpinion.key_audit_matters` is embedded; we iterate opinions and
/// link each KAM to its parent opinion.
fn synthesize_kam_in_opinion(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let audit = &ctx.ds_result.audit;
    let mut edges = Vec::new();

    for opinion in &audit.audit_opinions {
        let opinion_ext_id = format!("OPINION-{}", opinion.opinion_id);
        let Some(opinion_node) = ctx.id_map.get(&opinion_ext_id) else {
            continue;
        };

        for kam in &opinion.key_audit_matters {
            let kam_ext_id = format!("KAM-{}", kam.kam_id);
            let Some(kam_node) = ctx.id_map.get(&kam_ext_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: kam_node,
                target: opinion_node,
                edge_type: KAM_IN_OPINION,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }
    }

    debug!("KAM_IN_OPINION: {} edges", edges.len());
    edges
}

/// SOX302_FOR_ENTITY (164): sox_302_certification → engagement (of same company).
fn synthesize_sox302_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for cert in &ctx.ds_result.audit.sox_302_certifications {
        let cert_ext_id = format!("SOX302-{}", cert.certification_id);
        let Some(cert_node) = ctx.id_map.get(&cert_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&cert.company_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: cert_node,
            target: eng_node,
            edge_type: SOX302_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("SOX302_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// SOX404_FOR_ENTITY (165): sox_404_assessment → engagement (of same company).
fn synthesize_sox404_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for assessment in &ctx.ds_result.audit.sox_404_assessments {
        let assess_ext_id = format!("SOX404-{}", assessment.assessment_id);
        let Some(assess_node) = ctx.id_map.get(&assess_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&assessment.company_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: assess_node,
            target: eng_node,
            edge_type: SOX404_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("SOX404_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// GC_FOR_ENTITY (166): going_concern_assessment → engagement (of same entity).
fn synthesize_gc_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for gc in &ctx.ds_result.audit.going_concern_assessments {
        let gc_ext_id = format!("GC-{}-{}", gc.entity_code, gc.assessment_period);
        let Some(gc_node) = ctx.id_map.get(&gc_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&gc.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: gc_node,
            target: eng_node,
            edge_type: GC_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("GC_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// COMPONENT_AUDITOR_ASSIGNED (167): component_auditor → group_audit_plan.
///
/// The group audit plan is keyed as `"GAP-{engagement_id}"`.  We look it up
/// by iterating over the single group plan (if any).
fn synthesize_component_auditor_assigned(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let audit = &ctx.ds_result.audit;
    let mut edges = Vec::new();

    let Some(gap) = &audit.group_audit_plan else {
        debug!("COMPONENT_AUDITOR_ASSIGNED: no group audit plan, skipping");
        return edges;
    };

    let gap_ext_id = format!("GAP-{}", gap.engagement_id);
    let Some(gap_node) = ctx.id_map.get(&gap_ext_id) else {
        return edges;
    };

    for caud in &audit.component_auditors {
        let caud_ext_id = format!("CAUD-{}", caud.id);
        let Some(caud_node) = ctx.id_map.get(&caud_ext_id) else {
            continue;
        };

        edges.push(ExportEdge {
            source: caud_node,
            target: gap_node,
            edge_type: COMPONENT_AUDITOR_ASSIGNED,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("COMPONENT_AUDITOR_ASSIGNED: {} edges", edges.len());
    edges
}

/// INSTRUCTION_TO_AUDITOR (168): component_instruction → component_auditor.
fn synthesize_instruction_to_auditor(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let audit = &ctx.ds_result.audit;
    let mut edges = Vec::new();

    for cinst in &audit.component_instructions {
        let cinst_ext_id = format!("CINST-{}", cinst.id);
        let Some(cinst_node) = ctx.id_map.get(&cinst_ext_id) else {
            continue;
        };
        let caud_ext_id = format!("CAUD-{}", cinst.component_auditor_id);
        let Some(caud_node) = ctx.id_map.get(&caud_ext_id) else {
            continue;
        };

        edges.push(ExportEdge {
            source: cinst_node,
            target: caud_node,
            edge_type: INSTRUCTION_TO_AUDITOR,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("INSTRUCTION_TO_AUDITOR: {} edges", edges.len());
    edges
}

/// REPORT_FROM_AUDITOR (169): component_auditor_report → component_instruction.
fn synthesize_report_from_auditor(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let audit = &ctx.ds_result.audit;
    let mut edges = Vec::new();

    for crpt in &audit.component_reports {
        let crpt_ext_id = format!("CRPT-{}", crpt.id);
        let Some(crpt_node) = ctx.id_map.get(&crpt_ext_id) else {
            continue;
        };
        let cinst_ext_id = format!("CINST-{}", crpt.instruction_id);
        let Some(cinst_node) = ctx.id_map.get(&cinst_ext_id) else {
            continue;
        };

        edges.push(ExportEdge {
            source: crpt_node,
            target: cinst_node,
            edge_type: REPORT_FROM_AUDITOR,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("REPORT_FROM_AUDITOR: {} edges", edges.len());
    edges
}

/// ELET_FOR_ENGAGEMENT (170): engagement_letter → engagement.
///
/// `EngagementLetter.engagement_id` is a String (matches `AuditEngagement.engagement_id`
/// which is a Uuid serialised as string).  Engagements are keyed by `engagement_ref`.
fn synthesize_elet_for_engagement(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let audit = &ctx.ds_result.audit;
    // Build engagement_id (UUID string) → engagement_ref map
    let eng_id_to_ref: HashMap<String, &str> = audit
        .engagements
        .iter()
        .map(|eng| (eng.engagement_id.to_string(), eng.engagement_ref.as_str()))
        .collect();

    let mut edges = Vec::new();

    for elet in &audit.engagement_letters {
        let elet_ext_id = format!("ELET-{}", elet.id);
        let Some(elet_node) = ctx.id_map.get(&elet_ext_id) else {
            continue;
        };
        // elet.engagement_id is a String
        let Some(&eng_ref) = eng_id_to_ref.get(&elet.engagement_id) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref) else {
            continue;
        };

        edges.push(ExportEdge {
            source: elet_node,
            target: eng_node,
            edge_type: ELET_FOR_ENGAGEMENT,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("ELET_FOR_ENGAGEMENT: {} edges", edges.len());
    edges
}

/// SPLAN_FOR_CRA (171): sampling_plan → combined_risk_assessment.
///
/// `SamplingPlan` has `entity_code`, `account_area`, `assertion` which together
/// match the CRA.  The CRA external ID is `"CRA-{cra.id}"` and the CRA slug is
/// computed as `"CRA-{entity_code}-{account_area.upper()}-{assertion.upper()}"`.
/// We look up by reconstructing the CRA ext_id from plan fields.
fn synthesize_splan_for_cra(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let audit = &ctx.ds_result.audit;
    // Build (entity_code, account_area, assertion) → cra.id map
    let cra_by_key: HashMap<(&str, &str, &str), &str> = audit
        .combined_risk_assessments
        .iter()
        .map(|cra| {
            (
                (
                    cra.entity_code.as_str(),
                    cra.account_area.as_str(),
                    cra.id.as_str(),
                ),
                cra.id.as_str(),
            )
        })
        .collect();

    // Simpler: build a set of all CRA ext_ids and match by plan fields.
    // SamplingPlan.cra_level is just the level string ("High", etc.), not the CRA id.
    // Match CRA → sampling plan via entity_code + account_area + assertion (as string).
    let cra_lookup: HashMap<(&str, &str, String), &str> = audit
        .combined_risk_assessments
        .iter()
        .map(|cra| {
            let key = (
                cra.entity_code.as_str(),
                cra.account_area.as_str(),
                format!("{:?}", cra.assertion).to_lowercase(),
            );
            (key, cra.id.as_str())
        })
        .collect();

    let mut edges = Vec::new();

    for plan in &audit.sampling_plans {
        let plan_ext_id = format!("SPLAN-{}", plan.id);
        let Some(plan_node) = ctx.id_map.get(&plan_ext_id) else {
            continue;
        };

        // assertion stored as string in SamplingPlan
        let key = (
            plan.entity_code.as_str(),
            plan.account_area.as_str(),
            plan.assertion.to_lowercase(),
        );
        let Some(&cra_id) = cra_lookup.get(&key) else {
            continue;
        };
        let cra_ext_id = format!("CRA-{cra_id}");
        let Some(cra_node) = ctx.id_map.get(&cra_ext_id) else {
            continue;
        };

        edges.push(ExportEdge {
            source: plan_node,
            target: cra_node,
            edge_type: SPLAN_FOR_CRA,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    // suppress unused variable warning
    let _ = cra_by_key;

    debug!("SPLAN_FOR_CRA: {} edges", edges.len());
    edges
}

/// SCOT_FOR_ENTITY (173): significant_class_of_transactions → engagement (same entity).
fn synthesize_scot_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for scot in &ctx.ds_result.audit.significant_transaction_classes {
        let scot_ext_id = format!("SCOT-{}", scot.id);
        let Some(scot_node) = ctx.id_map.get(&scot_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&scot.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: scot_node,
            target: eng_node,
            edge_type: SCOT_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("SCOT_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// UFLAG_ON_JE (174): unusual_item_flag → journal_entry.
///
/// `UnusualItemFlag.journal_entry_id` holds the `document_id` (UUID string) of
/// the flagged journal entry.  JEs are keyed by their `document_id` directly.
fn synthesize_uflag_on_je(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let mut edges = Vec::new();

    for uflag in &ctx.ds_result.audit.unusual_items {
        let uflag_ext_id = format!("UFLAG-{}", uflag.id);
        let Some(uflag_node) = ctx.id_map.get(&uflag_ext_id) else {
            continue;
        };
        // JEs are registered in the id_map by their document_id string
        let Some(je_node) = ctx.id_map.get(uflag.journal_entry_id.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: uflag_node,
            target: je_node,
            edge_type: UFLAG_ON_JE,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("UFLAG_ON_JE: {} edges", edges.len());
    edges
}

/// AREL_FOR_ENTITY (175): analytical_relationship → engagement (same entity).
fn synthesize_arel_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for arel in &ctx.ds_result.audit.analytical_relationships {
        let arel_ext_id = format!("AREL-{}", arel.id);
        let Some(arel_node) = ctx.id_map.get(&arel_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&arel.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: arel_node,
            target: eng_node,
            edge_type: AREL_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("AREL_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// ESTIMATE_FOR_ENTITY (176): accounting_estimate → engagement (same entity).
fn synthesize_estimate_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for est in &ctx.ds_result.audit.accounting_estimates {
        let est_ext_id = format!("EST-{}", est.id);
        let Some(est_node) = ctx.id_map.get(&est_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&est.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: est_node,
            target: eng_node,
            edge_type: ESTIMATE_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("ESTIMATE_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// SEVT_FOR_ENTITY (177): subsequent_event → engagement (same entity).
fn synthesize_sevt_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for sevt in &ctx.ds_result.audit.subsequent_events {
        let sevt_ext_id = format!("SEVT-{}", sevt.id);
        let Some(sevt_node) = ctx.id_map.get(&sevt_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&sevt.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: sevt_node,
            target: eng_node,
            edge_type: SEVT_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("SEVT_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// SOC_FOR_SERVICE_ORG (178): soc_report → service_organization.
fn synthesize_soc_for_service_org(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let audit = &ctx.ds_result.audit;
    let mut edges = Vec::new();

    for soc in &audit.soc_reports {
        let soc_ext_id = format!("SOC-{}", soc.id);
        let Some(soc_node) = ctx.id_map.get(&soc_ext_id) else {
            continue;
        };
        let sorg_ext_id = format!("SORG-{}", soc.service_org_id);
        let Some(sorg_node) = ctx.id_map.get(&sorg_ext_id) else {
            continue;
        };

        edges.push(ExportEdge {
            source: soc_node,
            target: sorg_node,
            edge_type: SOC_FOR_SERVICE_ORG,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("SOC_FOR_SERVICE_ORG: {} edges", edges.len());
    edges
}

/// SEGMENT_PARENT (179): operating_segment → group_structure.
///
/// Segments belong to a company code.  We look up the group_structure by
/// `company_code` matching the `parent_entity`.
fn synthesize_segment_parent(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let mut edges = Vec::new();

    let Some(ref gs) = ctx.ds_result.intercompany.group_structure else {
        debug!("SEGMENT_PARENT: no group structure, skipping");
        return edges;
    };

    let gstruct_ext_id = format!("GSTRUCT-{}", gs.parent_entity);
    let Some(gstruct_node) = ctx.id_map.get(&gstruct_ext_id) else {
        return edges;
    };

    for seg in &ctx.ds_result.financial_reporting.segment_reports {
        if seg.company_code != gs.parent_entity {
            continue;
        }
        let seg_ext_id = format!("SEG-{}", seg.segment_id);
        let Some(seg_node) = ctx.id_map.get(&seg_ext_id) else {
            continue;
        };

        edges.push(ExportEdge {
            source: seg_node,
            target: gstruct_node,
            edge_type: SEGMENT_PARENT,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("SEGMENT_PARENT: {} edges", edges.len());
    edges
}

/// ECL_FOR_ENTITY (180): ecl_model → engagement (same entity).
fn synthesize_ecl_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for ecl in &ctx.ds_result.accounting_standards.ecl_models {
        let ecl_ext_id = format!("ECL-{}", ecl.id);
        let Some(ecl_node) = ctx.id_map.get(&ecl_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&ecl.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: ecl_node,
            target: eng_node,
            edge_type: ECL_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("ECL_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// PROVISION_FOR_ENTITY (181): provision → engagement (same entity).
fn synthesize_provision_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for prov in &ctx.ds_result.accounting_standards.provisions {
        let prov_ext_id = format!("PROV-{}", prov.id);
        let Some(prov_node) = ctx.id_map.get(&prov_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&prov.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: prov_node,
            target: eng_node,
            edge_type: PROVISION_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("PROVISION_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// PENSION_FOR_ENTITY (182): defined_benefit_plan → engagement (same entity).
fn synthesize_pension_for_entity(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for plan in &ctx.ds_result.hr.pension_plans {
        let plan_ext_id = format!("DBP-{}", plan.id);
        let Some(plan_node) = ctx.id_map.get(&plan_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&plan.entity_code) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: plan_node,
            target: eng_node,
            edge_type: PENSION_FOR_ENTITY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("PENSION_FOR_ENTITY: {} edges", edges.len());
    edges
}

/// STOCK_GRANT_FOR_EMPLOYEE (183): stock_grant → employee.
///
/// Employees are registered in the id_map by their `employee_id` field.
fn synthesize_stock_grant_for_employee(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let mut edges = Vec::new();

    for grant in &ctx.ds_result.hr.stock_grants {
        let grant_ext_id = format!("SGRANT-{}", grant.id);
        let Some(grant_node) = ctx.id_map.get(&grant_ext_id) else {
            continue;
        };
        let Some(emp_node) = ctx.id_map.get(grant.employee_id.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: grant_node,
            target: emp_node,
            edge_type: STOCK_GRANT_FOR_EMPLOYEE,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("STOCK_GRANT_FOR_EMPLOYEE: {} edges", edges.len());
    edges
}

/// TDIFF_FOR_ACCOUNT (184): temporary_difference → gl_account.
///
/// `TemporaryDifference.account` holds the GL account number.
/// GL accounts are keyed by their account number in the id_map.
fn synthesize_tdiff_for_account(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let mut edges = Vec::new();

    for tdiff in &ctx.ds_result.tax.deferred_tax.temporary_differences {
        let tdiff_ext_id = format!("TDIFF-{}", tdiff.id);
        let Some(tdiff_node) = ctx.id_map.get(&tdiff_ext_id) else {
            continue;
        };
        let Some(acct_node) = ctx.id_map.get(tdiff.account.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: tdiff_node,
            target: acct_node,
            edge_type: TDIFF_FOR_ACCOUNT,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("TDIFF_FOR_ACCOUNT: {} edges", edges.len());
    edges
}

/// BC_ACQUIRER (185): business_combination → engagement (of acquirer entity).
fn synthesize_bc_acquirer(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let mut edges = Vec::new();

    for bc in &ctx.ds_result.accounting_standards.business_combinations {
        let bc_ext_id = format!("BC-{}", bc.id);
        let Some(bc_node) = ctx.id_map.get(&bc_ext_id) else {
            continue;
        };
        let Some(eng_ref) = eng_by_entity.get(&bc.acquirer_entity) else {
            continue;
        };
        let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
            continue;
        };

        edges.push(ExportEdge {
            source: bc_node,
            target: eng_node,
            edge_type: BC_ACQUIRER,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("BC_ACQUIRER: {} edges", edges.len());
    edges
}

/// NCI_FOR_GROUP (186): nci_measurement → group_structure.
fn synthesize_nci_for_group(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let mut edges = Vec::new();

    let Some(ref gs) = ctx.ds_result.intercompany.group_structure else {
        debug!("NCI_FOR_GROUP: no group structure, skipping");
        return edges;
    };

    let gstruct_ext_id = format!("GSTRUCT-{}", gs.parent_entity);
    let Some(gstruct_node) = ctx.id_map.get(&gstruct_ext_id) else {
        return edges;
    };

    for nci in &ctx.ds_result.intercompany.nci_measurements {
        let nci_ext_id = format!("NCI-{}", nci.entity_code);
        let Some(nci_node) = ctx.id_map.get(&nci_ext_id) else {
            continue;
        };

        edges.push(ExportEdge {
            source: nci_node,
            target: gstruct_node,
            edge_type: NCI_FOR_GROUP,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("NCI_FOR_GROUP: {} edges", edges.len());
    edges
}

/// GSTRUCT_PARENT_ENG (187): group_structure → engagement (of parent entity).
fn synthesize_gstruct_parent_eng(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let mut edges = Vec::new();

    let Some(ref gs) = ctx.ds_result.intercompany.group_structure else {
        debug!("GSTRUCT_PARENT_ENG: no group structure, skipping");
        return edges;
    };

    let gstruct_ext_id = format!("GSTRUCT-{}", gs.parent_entity);
    let Some(gstruct_node) = ctx.id_map.get(&gstruct_ext_id) else {
        return edges;
    };

    // Look for an engagement whose entity_code matches the parent
    let eng_by_entity = engagement_ref_by_entity(ctx);
    let Some(eng_ref) = eng_by_entity.get(&gs.parent_entity) else {
        debug!(
            "GSTRUCT_PARENT_ENG: no engagement for parent entity {}",
            gs.parent_entity
        );
        return edges;
    };
    let Some(eng_node) = ctx.id_map.get(eng_ref.as_str()) else {
        return edges;
    };

    edges.push(ExportEdge {
        source: gstruct_node,
        target: eng_node,
        edge_type: GSTRUCT_PARENT_ENG,
        weight: 1.0,
        properties: HashMap::new(),
    });

    debug!("GSTRUCT_PARENT_ENG: {} edges", edges.len());
    edges
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::traits::EdgeSynthesizer;

    #[test]
    fn synthesizer_name() {
        let s = V130EdgeSynthesizer;
        assert_eq!(s.name(), "v130_edges");
    }

    #[test]
    fn edge_type_codes_distinct() {
        let codes = [
            CRA_FOR_ENTITY,
            MATERIALITY_FOR_ENTITY,
            OPINION_FOR_ENGAGEMENT,
            KAM_IN_OPINION,
            SOX302_FOR_ENTITY,
            SOX404_FOR_ENTITY,
            GC_FOR_ENTITY,
            COMPONENT_AUDITOR_ASSIGNED,
            INSTRUCTION_TO_AUDITOR,
            REPORT_FROM_AUDITOR,
            ELET_FOR_ENGAGEMENT,
            SPLAN_FOR_CRA,
            SAMPLED_ITEM_IN_PLAN,
            SCOT_FOR_ENTITY,
            UFLAG_ON_JE,
            AREL_FOR_ENTITY,
            ESTIMATE_FOR_ENTITY,
            SEVT_FOR_ENTITY,
            SOC_FOR_SERVICE_ORG,
            SEGMENT_PARENT,
            ECL_FOR_ENTITY,
            PROVISION_FOR_ENTITY,
            PENSION_FOR_ENTITY,
            STOCK_GRANT_FOR_EMPLOYEE,
            TDIFF_FOR_ACCOUNT,
            BC_ACQUIRER,
            NCI_FOR_GROUP,
            GSTRUCT_PARENT_ENG,
        ];
        let mut seen = std::collections::HashSet::new();
        for &code in &codes {
            assert!(seen.insert(code), "Duplicate edge type code: {code}");
        }
        assert_eq!(seen.len(), 28);
    }

    #[test]
    fn edge_type_codes_in_range() {
        let codes = [
            CRA_FOR_ENTITY,
            MATERIALITY_FOR_ENTITY,
            OPINION_FOR_ENGAGEMENT,
            KAM_IN_OPINION,
            SOX302_FOR_ENTITY,
            SOX404_FOR_ENTITY,
            GC_FOR_ENTITY,
            COMPONENT_AUDITOR_ASSIGNED,
            INSTRUCTION_TO_AUDITOR,
            REPORT_FROM_AUDITOR,
            ELET_FOR_ENGAGEMENT,
            SPLAN_FOR_CRA,
            SAMPLED_ITEM_IN_PLAN,
            SCOT_FOR_ENTITY,
            UFLAG_ON_JE,
            AREL_FOR_ENTITY,
            ESTIMATE_FOR_ENTITY,
            SEVT_FOR_ENTITY,
            SOC_FOR_SERVICE_ORG,
            SEGMENT_PARENT,
            ECL_FOR_ENTITY,
            PROVISION_FOR_ENTITY,
            PENSION_FOR_ENTITY,
            STOCK_GRANT_FOR_EMPLOYEE,
            TDIFF_FOR_ACCOUNT,
            BC_ACQUIRER,
            NCI_FOR_GROUP,
            GSTRUCT_PARENT_ENG,
        ];
        for &code in &codes {
            assert!(
                (160..=187).contains(&code),
                "Edge type code {code} outside expected range 160-187"
            );
        }
    }
}
