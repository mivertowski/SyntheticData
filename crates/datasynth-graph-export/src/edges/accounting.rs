//! Accounting network edge synthesizer.
//!
//! Produces edges linking journal entries, GL accounts, account hierarchy,
//! and cost centers.
//!
//! ## Edge Types Produced
//!
//! | Code | Name              | Direction                     |
//! |------|-------------------|-------------------------------|
//! |  70  | POSTS_TO_ACCOUNT  | JE -> account                 |
//! |  99  | DOC_POSTS_JE      | document -> JE                |
//! | 112  | ACCOUNT_POSTS_TO  | account -> parent account     |
//! | 124  | ACCOUNT_HAS_TYPE  | account -> account_type       |
//! | 133  | JE_COST_CENTER    | JE -> cost_center             |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const POSTS_TO_ACCOUNT: u32 = 70;
const DOC_POSTS_JE: u32 = 99;
const ACCOUNT_POSTS_TO: u32 = 112;
const ACCOUNT_HAS_TYPE: u32 = 124;
const JE_COST_CENTER: u32 = 133;

/// Synthesizes accounting network edges.
pub struct AccountingEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for AccountingEdgeSynthesizer {
    fn name(&self) -> &'static str {
        "accounting"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        if !ctx.config.edge_synthesis.accounting_network_edges {
            debug!("AccountingEdgeSynthesizer skipped (accounting_network_edges=false)");
            return Ok(Vec::new());
        }

        let mut edges = Vec::new();

        edges.extend(self.synthesize_posts_to_account(ctx));
        edges.extend(self.synthesize_doc_posts_je(ctx));
        edges.extend(self.synthesize_account_posts_to(ctx));
        edges.extend(self.synthesize_account_has_type(ctx));
        edges.extend(self.synthesize_je_cost_center(ctx));

        debug!(
            "AccountingEdgeSynthesizer produced {} total edges",
            edges.len()
        );
        Ok(edges)
    }
}

impl AccountingEdgeSynthesizer {
    /// POSTS_TO_ACCOUNT (code 70): JE -> account.
    ///
    /// For each journal entry line, creates an edge from the JE to the
    /// GL account referenced by the line.
    fn synthesize_posts_to_account(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let journal_entries = &ctx.ds_result.journal_entries;
        let mut edges = Vec::new();
        // Deduplicate: (je_id, account_number) pairs to avoid one edge per line
        let mut seen: std::collections::HashSet<(u64, u64)> = std::collections::HashSet::new();

        for je in journal_entries {
            let je_ext_id = je.header.document_id.to_string();
            let Some(je_id) = ctx.id_map.get(&je_ext_id) else {
                continue;
            };

            for line in &je.lines {
                let Some(acct_id) = ctx.id_map.get(&line.gl_account) else {
                    continue;
                };
                let pair = (je_id, acct_id);
                if !seen.insert(pair) {
                    continue;
                }

                edges.push(ExportEdge {
                    source: je_id,
                    target: acct_id,
                    edge_type: POSTS_TO_ACCOUNT,
                    weight: 1.0,
                    properties: HashMap::new(),
                });
            }
        }

        debug!("POSTS_TO_ACCOUNT: {} edges", edges.len());
        edges
    }

    /// DOC_POSTS_JE (code 99): document -> JE.
    ///
    /// Links P2P/O2C documents to their corresponding journal entries
    /// via the `journal_entry_id` FK on DocumentHeader.
    fn synthesize_doc_posts_je(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let flows = &ctx.ds_result.document_flows;
        let mut edges = Vec::new();

        // Helper: extract (document_id, journal_entry_id) pairs from any document with a header
        macro_rules! link_docs {
            ($docs:expr) => {
                for doc in $docs {
                    let Some(ref je_ref) = doc.header.journal_entry_id else {
                        continue;
                    };
                    let Some(doc_id) = ctx.id_map.get(&doc.header.document_id) else {
                        continue;
                    };
                    let Some(je_id) = ctx.id_map.get(je_ref) else {
                        continue;
                    };
                    edges.push(ExportEdge {
                        source: doc_id,
                        target: je_id,
                        edge_type: DOC_POSTS_JE,
                        weight: 1.0,
                        properties: HashMap::new(),
                    });
                }
            };
        }

        link_docs!(&flows.vendor_invoices);
        link_docs!(&flows.customer_invoices);
        link_docs!(&flows.payments);
        link_docs!(&flows.goods_receipts);

        debug!("DOC_POSTS_JE: {} edges", edges.len());
        edges
    }

    /// ACCOUNT_POSTS_TO (code 112): account -> parent account.
    ///
    /// Builds the GL account hierarchy using `parent_account` FK on GLAccount.
    fn synthesize_account_posts_to(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let accounts = &ctx.ds_result.chart_of_accounts.accounts;
        let mut edges = Vec::new();

        for account in accounts {
            let Some(ref parent_num) = account.parent_account else {
                continue;
            };
            let Some(acct_id) = ctx.id_map.get(&account.account_number) else {
                continue;
            };
            let Some(parent_id) = ctx.id_map.get(parent_num) else {
                continue;
            };

            edges.push(ExportEdge {
                source: acct_id,
                target: parent_id,
                edge_type: ACCOUNT_POSTS_TO,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("ACCOUNT_POSTS_TO: {} edges", edges.len());
        edges
    }

    /// ACCOUNT_HAS_TYPE (code 124): account -> account_type node.
    ///
    /// Links accounts to their AccountType classification nodes.
    /// Account type nodes must be registered in the id_map by a NodeSynthesizer
    /// using a convention like "ACCT_TYPE_Asset", "ACCT_TYPE_Revenue", etc.
    fn synthesize_account_has_type(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let accounts = &ctx.ds_result.chart_of_accounts.accounts;
        let mut edges = Vec::new();

        for account in accounts {
            let type_ext_id = format!("ACCT_TYPE_{:?}", account.account_type);
            let Some(acct_id) = ctx.id_map.get(&account.account_number) else {
                continue;
            };
            let Some(type_id) = ctx.id_map.get(&type_ext_id) else {
                // Account type nodes may not be synthesized yet
                continue;
            };

            edges.push(ExportEdge {
                source: acct_id,
                target: type_id,
                edge_type: ACCOUNT_HAS_TYPE,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("ACCOUNT_HAS_TYPE: {} edges", edges.len());
        edges
    }

    /// JE_COST_CENTER (code 133): JE -> cost_center.
    ///
    /// Links journal entry lines to their cost center assignment.
    /// Deduplicates to one edge per (JE, cost_center) pair.
    fn synthesize_je_cost_center(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let journal_entries = &ctx.ds_result.journal_entries;
        let mut edges = Vec::new();
        let mut seen: std::collections::HashSet<(u64, u64)> = std::collections::HashSet::new();

        for je in journal_entries {
            let je_ext_id = je.header.document_id.to_string();
            let Some(je_id) = ctx.id_map.get(&je_ext_id) else {
                continue;
            };

            for line in &je.lines {
                let Some(ref cc) = line.cost_center else {
                    continue;
                };
                let Some(cc_id) = ctx.id_map.get(cc) else {
                    continue;
                };
                let pair = (je_id, cc_id);
                if !seen.insert(pair) {
                    continue;
                }

                edges.push(ExportEdge {
                    source: je_id,
                    target: cc_id,
                    edge_type: JE_COST_CENTER,
                    weight: 1.0,
                    properties: HashMap::new(),
                });
            }
        }

        debug!("JE_COST_CENTER: {} edges", edges.len());
        edges
    }
}
