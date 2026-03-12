//! Intercompany node synthesizer.
//!
//! Creates intercompany nodes from the intercompany snapshot:
//! - `ic_matched_pair` (460) — matched IC transaction pairs
//! - `ic_elimination` (461) — elimination entries for consolidation
//! - `ic_netting` (462) — netting runs (from treasury, if IC-related)
//!
//! Note: IC journal entries are regular JournalEntry nodes (code 300) and are
//! handled by HypergraphBuilder, not this synthesizer.

use std::collections::HashMap;

use rust_decimal::prelude::ToPrimitive;
use tracing::debug;

use crate::error::ExportError;
use crate::traits::{NodeSynthesisContext, NodeSynthesizer};
use crate::types::ExportNode;

/// Synthesizes intercompany nodes from IC matched pairs and elimination entries.
pub struct IntercompanyNodeSynthesizer;

impl NodeSynthesizer for IntercompanyNodeSynthesizer {
    fn name(&self) -> &'static str {
        "intercompany"
    }

    fn synthesize(
        &self,
        ctx: &mut NodeSynthesisContext<'_>,
    ) -> Result<Vec<ExportNode>, ExportError> {
        let mut nodes = Vec::new();
        let ic = &ctx.ds_result.intercompany;

        let total = ic.matched_pairs.len() + ic.elimination_entries.len();

        if total == 0 {
            debug!("IntercompanyNodeSynthesizer: intercompany snapshot is empty, skipping");
            return Ok(nodes);
        }

        debug!("IntercompanyNodeSynthesizer: synthesizing ~{total} intercompany nodes");

        // IC Matched Pairs (460)
        for pair in &ic.matched_pairs {
            let external_id = format!("IC-PAIR-{}", pair.ic_reference);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert(
                "icTransactionId".into(),
                serde_json::json!(pair.ic_reference),
            );
            props.insert(
                "senderCompany".into(),
                serde_json::json!(pair.seller_company),
            );
            props.insert(
                "receiverCompany".into(),
                serde_json::json!(pair.buyer_company),
            );
            props.insert(
                "amount".into(),
                serde_json::json!(pair.amount.to_f64().unwrap_or(0.0)),
            );
            props.insert("currency".into(), serde_json::json!(pair.currency));
            props.insert(
                "transactionType".into(),
                serde_json::json!(format!("{:?}", pair.transaction_type)),
            );
            props.insert(
                "status".into(),
                serde_json::json!(format!("{:?}", pair.settlement_status)),
            );
            props.insert(
                "transactionDate".into(),
                serde_json::json!(format!("{}T00:00:00Z", pair.transaction_date)),
            );
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("ic_matched_pair"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 460,
                node_type_name: "ic_matched_pair".into(),
                label: format!(
                    "IC: {} → {} ({})",
                    pair.seller_company, pair.buyer_company, pair.ic_reference
                ),
                layer: 3, // Accounting
                properties: props,
            });
        }

        // IC Elimination Entries (461)
        for elim in &ic.elimination_entries {
            let external_id = format!("IC-ELIM-{}", elim.entry_id);
            let numeric_id = ctx.id_map.get_or_insert(&external_id);

            let mut props = HashMap::new();
            props.insert("entryId".into(), serde_json::json!(elim.entry_id));
            props.insert(
                "eliminationType".into(),
                serde_json::json!(format!("{:?}", elim.elimination_type)),
            );
            props.insert(
                "consolidationEntity".into(),
                serde_json::json!(elim.consolidation_entity),
            );
            props.insert(
                "fiscalPeriod".into(),
                serde_json::json!(elim.fiscal_period),
            );
            props.insert(
                "totalDebit".into(),
                serde_json::json!(elim.total_debit.to_f64().unwrap_or(0.0)),
            );
            props.insert(
                "totalCredit".into(),
                serde_json::json!(elim.total_credit.to_f64().unwrap_or(0.0)),
            );
            props.insert("currency".into(), serde_json::json!(elim.currency));
            props.insert(
                "nodeTypeName".into(),
                serde_json::json!("ic_elimination"),
            );

            nodes.push(ExportNode {
                id: Some(numeric_id),
                node_type: 461,
                node_type_name: "ic_elimination".into(),
                label: format!(
                    "IC Elim: {} ({})",
                    elim.entry_id, elim.consolidation_entity
                ),
                layer: 3, // Accounting
                properties: props,
            });
        }

        debug!(
            "IntercompanyNodeSynthesizer: produced {} nodes",
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
        let s = IntercompanyNodeSynthesizer;
        assert_eq!(s.name(), "intercompany");
    }
}
