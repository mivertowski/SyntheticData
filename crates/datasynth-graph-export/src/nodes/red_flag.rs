//! Red Flag node synthesizer.
//!
//! Creates `red_flag` nodes (entity code 510) from fraud red flag indicators
//! generated during the fraud injection pass.

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes red flag nodes from fraud indicators.
pub struct RedFlagNodeSynthesizer;

impl NodeSynthesizer for RedFlagNodeSynthesizer {
    fn name(&self) -> &'static str {
        "red_flag"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let red_flags = &ctx.ds_result.red_flags;

        if red_flags.is_empty() {
            debug!("RedFlagNodeSynthesizer: no red flags, skipping");
            return Ok(nodes);
        }

        debug!(
            "RedFlagNodeSynthesizer: creating {} red flag nodes",
            red_flags.len()
        );

        for (i, flag) in red_flags.iter().enumerate() {
            let external_id = format!("RED-FLAG-{:06}", i + 1);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "documentId".into(),
                serde_json::json!(flag.document_id),
            );
            props.insert(
                "category".into(),
                serde_json::json!(format!("{:?}", flag.category)),
            );
            props.insert(
                "severity".into(),
                serde_json::json!(format!("{:?}", flag.strength)),
            );
            props.insert(
                "description".into(),
                serde_json::json!(flag.pattern_name),
            );
            props.insert("confidence".into(), serde_json::json!(flag.confidence));
            props.insert("isFraudulent".into(), serde_json::json!(flag.is_fraudulent));
            // Serialize details as a JSON object
            let details: serde_json::Value = flag
                .details
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                .collect::<serde_json::Map<String, serde_json::Value>>()
                .into();
            props.insert("details".into(), details);
            props.insert("nodeTypeName".into(), serde_json::json!("red_flag"));

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 510,
                node_type_name: "red_flag".into(),
                label: format!("Red Flag: {} on {}", flag.pattern_name, flag.document_id),
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
        let s = RedFlagNodeSynthesizer;
        assert_eq!(s.name(), "red_flag");
    }
}
