//! v1.4.0 edge synthesizer.
//!
//! Produces edges linking journal entries to the employees who posted them,
//! and internal controls to the journal entries they cover.
//!
//! ## Edge Types Produced
//!
//! | Code | Name            | Direction                              |
//! |------|-----------------|----------------------------------------|
//! | 188  | JE_POSTED_BY    | journal_entry → employee (posted_by)   |
//! | 189  | CONTROL_APPLIED | internal_control → journal_entry       |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type constants for v1.4.0 edges.
const JE_POSTED_BY: u32 = 188;
const CONTROL_APPLIED: u32 = 189;

/// Synthesizer for v1.4.0 people-and-controls edges.
pub struct V140EdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for V140EdgeSynthesizer {
    fn name(&self) -> &'static str {
        "v140_edges"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let mut edges = Vec::new();

        if ctx.config.edge_synthesis.people_edges {
            edges.extend(synthesize_je_posted_by(ctx));
        }

        if ctx.config.edge_synthesis.accounting_network_edges {
            edges.extend(synthesize_control_applied(ctx));
        }

        debug!("V140EdgeSynthesizer produced {} total edges", edges.len());
        Ok(edges)
    }
}

/// JE_POSTED_BY (code 188): journal_entry → employee.
///
/// Links each journal entry to the employee identified by `header.created_by`.
/// This is distinct from `JE_CREATED_BY` (code 98) which was the original
/// CREATED_BY edge; `JE_POSTED_BY` is the authoritative "posted by" label
/// useful for segregation-of-duties analysis.
fn synthesize_je_posted_by(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let journal_entries = &ctx.ds_result.journal_entries;
    let mut edges = Vec::new();
    let mut seen: std::collections::HashSet<(u64, u64)> = std::collections::HashSet::new();

    for je in journal_entries {
        if je.header.created_by.is_empty() {
            continue;
        }
        let je_ext_id = je.header.document_id.to_string();
        let Some(je_id) = ctx.id_map.get(&je_ext_id) else {
            continue;
        };
        let Some(emp_id) = ctx.id_map.get(&je.header.created_by) else {
            continue;
        };
        let pair = (je_id, emp_id);
        if !seen.insert(pair) {
            continue;
        }

        edges.push(ExportEdge {
            source: je_id,
            target: emp_id,
            edge_type: JE_POSTED_BY,
            weight: 1.0,
            properties: HashMap::new(),
        });
    }

    debug!("JE_POSTED_BY: {} edges", edges.len());
    edges
}

/// CONTROL_APPLIED (code 189): internal_control → journal_entry.
///
/// Links each internal control to the journal entries it covers via
/// `je.header.control_ids`. Enables graph queries such as
/// "which JEs are not covered by any control?" (fraud risk indicator).
fn synthesize_control_applied(ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
    let journal_entries = &ctx.ds_result.journal_entries;
    let mut edges = Vec::new();
    let mut seen: std::collections::HashSet<(u64, u64)> = std::collections::HashSet::new();

    for je in journal_entries {
        if je.header.control_ids.is_empty() {
            continue;
        }
        let je_ext_id = je.header.document_id.to_string();
        let Some(je_id) = ctx.id_map.get(&je_ext_id) else {
            continue;
        };

        for ctrl_id in &je.header.control_ids {
            let Some(ctrl_node_id) = ctx.id_map.get(ctrl_id) else {
                continue;
            };
            let pair = (ctrl_node_id, je_id);
            if !seen.insert(pair) {
                continue;
            }

            edges.push(ExportEdge {
                source: ctrl_node_id,
                target: je_id,
                edge_type: CONTROL_APPLIED,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }
    }

    debug!("CONTROL_APPLIED: {} edges", edges.len());
    edges
}

#[cfg(test)]
mod tests {
    #[test]
    fn edge_type_codes_are_unique() {
        // 188 and 189 are not in any other synthesizer's range.
        assert_ne!(super::JE_POSTED_BY, super::CONTROL_APPLIED);
        assert_eq!(super::JE_POSTED_BY, 188);
        assert_eq!(super::CONTROL_APPLIED, 189);
    }
}
