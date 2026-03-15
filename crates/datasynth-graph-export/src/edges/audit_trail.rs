//! Audit trail edge synthesizer.
//!
//! Produces edges linking audit artifacts (workpapers, findings, evidence,
//! judgments, engagements, scopes) and their relationships to people.
//!
//! ## Edge Types Produced
//!
//! | Code | Name                    | Direction                        |
//! |------|-------------------------|----------------------------------|
//! |  72  | BELONGS_TO_ENGAGEMENT   | workpaper -> engagement           |
//! |  73  | FINDING_IN_WORKPAPER    | finding -> workpaper              |
//! |  74  | EVIDENCE_IN_WORKPAPER   | evidence -> workpaper             |
//! |  77  | JUDGMENT_ON_ASSESSMENT  | judgment -> risk_assessment       |
//! | 100  | WP_PREPARED_BY          | workpaper -> employee             |
//! | 101  | WP_REVIEWED_BY          | workpaper -> employee             |
//! | 103  | ENGAGEMENT_PARTNER      | engagement -> employee            |
//! | 104  | FINDING_RESPONSIBLE     | finding -> employee               |
//! | 132  | ASSESSMENT_ON_SCOPE     | risk_assessment -> scope          |
//! | 134  | ENGAGEMENT_HAS_SCOPE    | engagement -> scope               |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const BELONGS_TO_ENGAGEMENT: u32 = 72;
const FINDING_IN_WORKPAPER: u32 = 73;
const EVIDENCE_IN_WORKPAPER: u32 = 74;
const JUDGMENT_ON_ASSESSMENT: u32 = 77;
const WP_PREPARED_BY: u32 = 100;
const WP_REVIEWED_BY: u32 = 101;
const ENGAGEMENT_PARTNER: u32 = 103;
const FINDING_RESPONSIBLE: u32 = 104;
// These codes are reserved for scope-linked edges; synthesizers are stubs until
// scope nodes are available (Task 12).
#[allow(dead_code)]
const ASSESSMENT_ON_SCOPE: u32 = 132;
#[allow(dead_code)]
const ENGAGEMENT_HAS_SCOPE: u32 = 134;

/// Synthesizes audit trail edges linking audit artifacts, people, and scopes.
pub struct AuditTrailEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for AuditTrailEdgeSynthesizer {
    fn name(&self) -> &'static str {
        "audit_trail"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let mut edges = Vec::new();

        edges.extend(self.synthesize_belongs_to_engagement(ctx));
        edges.extend(self.synthesize_finding_in_workpaper(ctx));
        edges.extend(self.synthesize_evidence_in_workpaper(ctx));
        edges.extend(self.synthesize_judgment_on_assessment(ctx));
        edges.extend(self.synthesize_wp_prepared_by(ctx));
        edges.extend(self.synthesize_wp_reviewed_by(ctx));
        edges.extend(self.synthesize_engagement_partner(ctx));
        edges.extend(self.synthesize_finding_responsible(ctx));
        // 132 and 134 require scope nodes which may not be present
        edges.extend(self.synthesize_assessment_on_scope(ctx));
        edges.extend(self.synthesize_engagement_has_scope(ctx));

        debug!(
            "AuditTrailEdgeSynthesizer produced {} total edges",
            edges.len()
        );
        Ok(edges)
    }
}

impl AuditTrailEdgeSynthesizer {
    /// BELONGS_TO_ENGAGEMENT (code 72): workpaper -> engagement.
    fn synthesize_belongs_to_engagement(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let workpapers = &ctx.ds_result.audit.workpapers;
        let mut edges = Vec::new();

        for wp in workpapers {
            let wp_ext_id = wp.workpaper_ref.as_str();
            let eng_ext_id = wp.engagement_id.to_string();

            let Some(wp_id) = ctx.id_map.get(wp_ext_id) else {
                continue;
            };
            let Some(eng_id) = ctx.id_map.get(&eng_ext_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: wp_id,
                target: eng_id,
                edge_type: BELONGS_TO_ENGAGEMENT,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("BELONGS_TO_ENGAGEMENT: {} edges", edges.len());
        edges
    }

    /// FINDING_IN_WORKPAPER (code 73): finding -> workpaper.
    fn synthesize_finding_in_workpaper(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let findings = &ctx.ds_result.audit.findings;
        let mut edges = Vec::new();

        for finding in findings {
            let Some(ref wp_id_str) = finding.workpaper_id else {
                continue;
            };

            let Some(finding_id) = ctx.id_map.get(&finding.finding_ref) else {
                continue;
            };
            let Some(wp_id) = ctx.id_map.get(wp_id_str) else {
                continue;
            };

            edges.push(ExportEdge {
                source: finding_id,
                target: wp_id,
                edge_type: FINDING_IN_WORKPAPER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("FINDING_IN_WORKPAPER: {} edges", edges.len());
        edges
    }

    /// EVIDENCE_IN_WORKPAPER (code 74): evidence -> workpaper.
    fn synthesize_evidence_in_workpaper(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let evidence_list = &ctx.ds_result.audit.evidence;
        let mut edges = Vec::new();

        for evidence in evidence_list {
            let ev_ext_id = evidence.evidence_ref.as_str();
            let Some(ev_id) = ctx.id_map.get(ev_ext_id) else {
                continue;
            };

            for wp_uuid in &evidence.linked_workpapers {
                let wp_ext_id = wp_uuid.to_string();
                if let Some(wp_id) = ctx.id_map.get(&wp_ext_id) {
                    edges.push(ExportEdge {
                        source: ev_id,
                        target: wp_id,
                        edge_type: EVIDENCE_IN_WORKPAPER,
                        weight: 1.0,
                        properties: HashMap::new(),
                    });
                }
            }
        }

        debug!("EVIDENCE_IN_WORKPAPER: {} edges", edges.len());
        edges
    }

    /// JUDGMENT_ON_ASSESSMENT (code 77): judgment -> risk_assessment.
    ///
    /// ProfessionalJudgments of type RiskAssessment are linked to the
    /// corresponding RiskAssessment nodes. Falls back to matching by
    /// engagement_id if no direct FK exists.
    fn synthesize_judgment_on_assessment(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let judgments = &ctx.ds_result.audit.judgments;
        let risk_assessments = &ctx.ds_result.audit.risk_assessments;
        let mut edges = Vec::new();

        if judgments.is_empty() || risk_assessments.is_empty() {
            return edges;
        }

        // Build engagement_id -> list of risk_assessment external IDs
        let mut risks_by_engagement: HashMap<String, Vec<&str>> = HashMap::new();
        for ra in risk_assessments {
            risks_by_engagement
                .entry(ra.engagement_id.to_string())
                .or_default()
                .push(&ra.risk_ref);
        }

        for judgment in judgments {
            let jdg_ext_id = judgment.judgment_ref.as_str();
            let Some(jdg_id) = ctx.id_map.get(jdg_ext_id) else {
                continue;
            };

            // Link to risk assessments in the same engagement
            let eng_key = judgment.engagement_id.to_string();
            if let Some(risk_refs) = risks_by_engagement.get(&eng_key) {
                for risk_ref in risk_refs {
                    if let Some(ra_id) = ctx.id_map.get(risk_ref) {
                        edges.push(ExportEdge {
                            source: jdg_id,
                            target: ra_id,
                            edge_type: JUDGMENT_ON_ASSESSMENT,
                            weight: 1.0,
                            properties: HashMap::new(),
                        });
                    }
                }
            }
        }

        debug!("JUDGMENT_ON_ASSESSMENT: {} edges", edges.len());
        edges
    }

    /// WP_PREPARED_BY (code 100): workpaper -> employee.
    fn synthesize_wp_prepared_by(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        if !ctx.config.edge_synthesis.people_edges {
            return Vec::new();
        }

        let workpapers = &ctx.ds_result.audit.workpapers;
        let mut edges = Vec::new();

        for wp in workpapers {
            if wp.preparer_id.is_empty() {
                continue;
            }
            let Some(wp_id) = ctx.id_map.get(&wp.workpaper_ref) else {
                continue;
            };
            let Some(emp_id) = ctx.id_map.get(&wp.preparer_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: wp_id,
                target: emp_id,
                edge_type: WP_PREPARED_BY,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("WP_PREPARED_BY: {} edges", edges.len());
        edges
    }

    /// WP_REVIEWED_BY (code 101): workpaper -> employee.
    fn synthesize_wp_reviewed_by(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        if !ctx.config.edge_synthesis.people_edges {
            return Vec::new();
        }

        let workpapers = &ctx.ds_result.audit.workpapers;
        let mut edges = Vec::new();

        for wp in workpapers {
            if let Some(ref reviewer) = wp.reviewer_id {
                let Some(wp_id) = ctx.id_map.get(&wp.workpaper_ref) else {
                    continue;
                };
                if let Some(reviewer_id) = ctx.id_map.get(reviewer) {
                    edges.push(ExportEdge {
                        source: wp_id,
                        target: reviewer_id,
                        edge_type: WP_REVIEWED_BY,
                        weight: 1.0,
                        properties: HashMap::new(),
                    });
                }
            }
        }

        debug!("WP_REVIEWED_BY: {} edges", edges.len());
        edges
    }

    /// ENGAGEMENT_PARTNER (code 103): engagement -> employee.
    fn synthesize_engagement_partner(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        if !ctx.config.edge_synthesis.people_edges {
            return Vec::new();
        }

        let engagements = &ctx.ds_result.audit.engagements;
        let mut edges = Vec::new();

        for eng in engagements {
            if eng.engagement_partner_id.is_empty() {
                continue;
            }
            let eng_ext_id = eng.engagement_ref.as_str();
            let Some(eng_id) = ctx.id_map.get(eng_ext_id) else {
                continue;
            };
            let Some(partner_id) = ctx.id_map.get(&eng.engagement_partner_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: eng_id,
                target: partner_id,
                edge_type: ENGAGEMENT_PARTNER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("ENGAGEMENT_PARTNER: {} edges", edges.len());
        edges
    }

    /// FINDING_RESPONSIBLE (code 104): finding -> employee.
    ///
    /// Uses `remediation_plan.responsible_party` to link findings to the
    /// employee responsible for remediation.
    fn synthesize_finding_responsible(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        if !ctx.config.edge_synthesis.people_edges {
            return Vec::new();
        }

        let findings = &ctx.ds_result.audit.findings;
        let mut edges = Vec::new();

        for finding in findings {
            let Some(ref plan) = finding.remediation_plan else {
                continue;
            };
            if plan.responsible_party.is_empty() {
                continue;
            }
            let Some(finding_id) = ctx.id_map.get(&finding.finding_ref) else {
                continue;
            };
            let Some(emp_id) = ctx.id_map.get(&plan.responsible_party) else {
                continue;
            };

            edges.push(ExportEdge {
                source: finding_id,
                target: emp_id,
                edge_type: FINDING_RESPONSIBLE,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("FINDING_RESPONSIBLE: {} edges", edges.len());
        edges
    }

    /// ASSESSMENT_ON_SCOPE (code 132): risk_assessment -> scope.
    ///
    /// Scope nodes may not exist in the id_map if audit scope generation
    /// is not enabled. In that case, this produces zero edges.
    fn synthesize_assessment_on_scope(
        &self,
        _ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        // Risk assessments don't have a direct scope_id FK in the current model.
        // This edge type requires scope node synthesis (Task 12) to populate.
        debug!("ASSESSMENT_ON_SCOPE: 0 edges (scope nodes not yet synthesized)");
        Vec::new()
    }

    /// ENGAGEMENT_HAS_SCOPE (code 134): engagement -> scope.
    ///
    /// Similar to above, depends on scope nodes being present.
    fn synthesize_engagement_has_scope(
        &self,
        _ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        // Scope nodes are synthesized in Task 12; this is a placeholder.
        debug!("ENGAGEMENT_HAS_SCOPE: 0 edges (scope nodes not yet synthesized)");
        Vec::new()
    }
}
