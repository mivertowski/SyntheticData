//! KYC Profile node synthesizer.
//!
//! Creates `kyc_profile` nodes (entity code 504) from banking customers.
//! Each customer generates one KYC profile node with risk rating and PEP status.

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes KYC profile nodes from banking customers.
pub struct KycProfileNodeSynthesizer;

impl NodeSynthesizer for KycProfileNodeSynthesizer {
    fn name(&self) -> &'static str {
        "kyc_profile"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let customers = &ctx.ds_result.banking.customers;

        if customers.is_empty() {
            debug!("KycProfileNodeSynthesizer: no banking customers, skipping");
            return Ok(nodes);
        }

        debug!(
            "KycProfileNodeSynthesizer: creating {} KYC profile nodes",
            customers.len()
        );

        for cust in customers {
            let external_id = format!("KYC-{}", cust.customer_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "customerId".into(),
                serde_json::json!(cust.customer_id.to_string()),
            );
            props.insert(
                "riskRating".into(),
                serde_json::json!(format!("{:?}", cust.risk_tier)),
            );
            props.insert("isPep".into(), serde_json::json!(cust.is_pep));
            props.insert(
                "onboardingDate".into(),
                serde_json::json!(format!("{}T00:00:00Z", cust.onboarding_date)),
            );
            if let Some(ref review_date) = cust.last_kyc_review {
                props.insert(
                    "lastReviewDate".into(),
                    serde_json::json!(format!("{review_date}T00:00:00Z")),
                );
            }
            props.insert(
                "status".into(),
                serde_json::json!(if cust.is_active {
                    "active"
                } else {
                    "inactive"
                }),
            );
            props.insert(
                "customerType".into(),
                serde_json::json!(format!("{:?}", cust.customer_type)),
            );
            props.insert("isMule".into(), serde_json::json!(cust.is_mule));
            props.insert("kycTruthful".into(), serde_json::json!(cust.kyc_truthful));
            props.insert("nodeTypeName".into(), serde_json::json!("kyc_profile"));
            props.insert("processFamily".into(), serde_json::json!("BANK"));

            let label = format!("KYC: {}", cust.name.display_name());

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 504,
                node_type_name: "kyc_profile".into(),
                label,
                layer: 1, // Governance
                properties: props,
            });
        }

        Ok(nodes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn synthesizer_metadata() {
        let s = KycProfileNodeSynthesizer;
        assert_eq!(s.name(), "kyc_profile");
    }
}
