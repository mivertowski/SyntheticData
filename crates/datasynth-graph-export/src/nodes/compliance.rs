//! Compliance node synthesizer.
//!
//! Creates compliance and regulatory nodes:
//! - `compliance_standard` (480) — from ComplianceStandardRecord
//! - `compliance_finding` (481) — from ComplianceFinding
//! - `regulatory_filing` (482) — from RegulatoryFiling
//! - `audit_procedure` (483) — from AuditProcedureRecord

use std::collections::HashMap;

use rust_decimal::prelude::ToPrimitive;
use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes compliance and regulatory framework nodes.
pub struct ComplianceNodeSynthesizer;

impl NodeSynthesizer for ComplianceNodeSynthesizer {
    fn name(&self) -> &'static str {
        "compliance"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let cr = &ctx.ds_result.compliance_regulations;

        let total = cr.standard_records.len()
            + cr.findings.len()
            + cr.filings.len()
            + cr.audit_procedures.len();

        if total == 0 {
            debug!("ComplianceNodeSynthesizer: compliance snapshot is empty, skipping");
            return Ok(nodes);
        }

        debug!("ComplianceNodeSynthesizer: synthesizing ~{total} compliance nodes");

        // Compliance Standards (480)
        for std_rec in &cr.standard_records {
            let external_id = format!("COMP-STD-{}", std_rec.standard_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "standardId".into(),
                serde_json::json!(std_rec.standard_id),
            );
            props.insert("body".into(), serde_json::json!(std_rec.body));
            props.insert("number".into(), serde_json::json!(std_rec.number));
            props.insert("title".into(), serde_json::json!(std_rec.title));
            props.insert("category".into(), serde_json::json!(std_rec.category));
            props.insert("domain".into(), serde_json::json!(std_rec.domain));
            props.insert(
                "jurisdiction".into(),
                serde_json::json!(std_rec.jurisdiction),
            );
            props.insert("version".into(), serde_json::json!(std_rec.version));
            props.insert("isActive".into(), serde_json::json!(std_rec.is_active));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("compliance_standard"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 480,
                node_type_name: "compliance_standard".into(),
                label: format!("{} {} - {}", std_rec.body, std_rec.number, std_rec.title),
                layer: 1, // Governance
                properties: props,
            });
        }

        // Compliance Findings (481)
        for finding in &cr.findings {
            let external_id = format!("COMP-FINDING-{}", finding.finding_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "findingId".into(),
                serde_json::json!(finding.finding_id.to_string()),
            );
            props.insert("title".into(), serde_json::json!(finding.title));
            props.insert(
                "description".into(),
                serde_json::json!(finding.description),
            );
            props.insert(
                "severity".into(),
                serde_json::json!(format!("{:?}", finding.severity)),
            );
            props.insert(
                "deficiencyLevel".into(),
                serde_json::json!(format!("{:?}", finding.deficiency_level)),
            );
            props.insert(
                "remediationStatus".into(),
                serde_json::json!(format!("{:?}", finding.remediation_status)),
            );
            props.insert("isRepeat".into(), serde_json::json!(finding.is_repeat));
            if let Some(ref impact) = finding.financial_impact {
                props.insert(
                    "financialImpact".into(),
                    serde_json::json!(impact.to_f64().unwrap_or(0.0)),
                );
            }
            props.insert(
                "identifiedDate".into(),
                serde_json::json!(format!("{}T00:00:00Z", finding.identified_date)),
            );
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("compliance_finding"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 481,
                node_type_name: "compliance_finding".into(),
                label: format!("Compliance Finding: {}", finding.title),
                layer: 1, // Governance
                properties: props,
            });
        }

        // Regulatory Filings (482)
        for filing in &cr.filings {
            let external_id = format!(
                "REG-FILING-{:?}-{}-{}",
                filing.filing_type, filing.company_code, filing.period_end
            );
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "filingType".into(),
                serde_json::json!(format!("{:?}", filing.filing_type)),
            );
            props.insert(
                "companyCode".into(),
                serde_json::json!(filing.company_code),
            );
            props.insert(
                "jurisdiction".into(),
                serde_json::json!(filing.jurisdiction),
            );
            props.insert(
                "periodEnd".into(),
                serde_json::json!(format!("{}T00:00:00Z", filing.period_end)),
            );
            props.insert(
                "deadline".into(),
                serde_json::json!(format!("{}T00:00:00Z", filing.deadline)),
            );
            props.insert(
                "status".into(),
                serde_json::json!(format!("{:?}", filing.status)),
            );
            props.insert("regulator".into(), serde_json::json!(filing.regulator));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("regulatory_filing"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 482,
                node_type_name: "regulatory_filing".into(),
                label: format!(
                    "Filing: {:?} ({} {})",
                    filing.filing_type, filing.company_code, filing.period_end
                ),
                layer: 1, // Governance
                properties: props,
            });
        }

        // Audit Procedures (483)
        for proc_rec in &cr.audit_procedures {
            let external_id = format!("COMP-PROC-{}", proc_rec.procedure_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "procedureId".into(),
                serde_json::json!(proc_rec.procedure_id),
            );
            props.insert("title".into(), serde_json::json!(proc_rec.title));
            props.insert(
                "description".into(),
                serde_json::json!(proc_rec.description),
            );
            props.insert(
                "procedureType".into(),
                serde_json::json!(proc_rec.procedure_type),
            );
            props.insert(
                "standardId".into(),
                serde_json::json!(proc_rec.standard_id),
            );
            props.insert(
                "samplingMethod".into(),
                serde_json::json!(proc_rec.sampling_method),
            );
            props.insert(
                "sampleSize".into(),
                serde_json::json!(proc_rec.sample_size),
            );
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("audit_procedure"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 483,
                node_type_name: "audit_procedure".into(),
                label: format!("Procedure: {}", proc_rec.title),
                layer: 1, // Governance
                properties: props,
            });
        }

        debug!(
            "ComplianceNodeSynthesizer: produced {} nodes",
            nodes.len()
        );
        Ok(nodes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_metadata() {
        let s = ComplianceNodeSynthesizer;
        assert_eq!(s.name(), "compliance");
    }
}
