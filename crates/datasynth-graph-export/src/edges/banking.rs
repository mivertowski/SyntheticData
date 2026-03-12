//! Banking edge synthesizer.
//!
//! Produces edges linking bank accounts, transactions, and customers.
//!
//! ## Edge Types Produced
//!
//! | Code | Name              | Direction                       |
//! |------|-------------------|---------------------------------|
//! |  80  | BANK_ACCOUNT_OWNER| bank_account -> customer        |
//! |  81  | BANK_TXN_ACCOUNT  | transaction -> bank_account     |
//! |  82  | RECON_ACCOUNT     | bank_account -> GL account      |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const BANK_ACCOUNT_OWNER: u32 = 80;
const BANK_TXN_ACCOUNT: u32 = 81;
// Reserved for bank account -> GL reconciliation account; stub until FK available.
#[allow(dead_code)]
const RECON_ACCOUNT: u32 = 82;

/// Synthesizes banking relationship edges.
pub struct BankingEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for BankingEdgeSynthesizer {
    fn name(&self) -> &'static str {
        "banking"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        if ctx.config.skip_banking {
            debug!("BankingEdgeSynthesizer skipped (skip_banking=true)");
            return Ok(Vec::new());
        }

        let mut edges = Vec::new();

        edges.extend(self.synthesize_bank_account_owner(ctx));
        edges.extend(self.synthesize_bank_txn_account(ctx));
        edges.extend(self.synthesize_recon_account(ctx));

        debug!(
            "BankingEdgeSynthesizer produced {} total edges",
            edges.len()
        );
        Ok(edges)
    }
}

impl BankingEdgeSynthesizer {
    /// BANK_ACCOUNT_OWNER (code 80): bank_account -> customer.
    fn synthesize_bank_account_owner(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let accounts = &ctx.ds_result.banking.accounts;
        let mut edges = Vec::new();

        for account in accounts {
            let acct_ext_id = account.account_id.to_string();
            let owner_ext_id = account.primary_owner_id.to_string();

            let Some(acct_id) = ctx.id_map.get(&acct_ext_id) else {
                continue;
            };
            let Some(owner_id) = ctx.id_map.get(&owner_ext_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: acct_id,
                target: owner_id,
                edge_type: BANK_ACCOUNT_OWNER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("BANK_ACCOUNT_OWNER: {} edges", edges.len());
        edges
    }

    /// BANK_TXN_ACCOUNT (code 81): transaction -> bank_account.
    fn synthesize_bank_txn_account(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let transactions = &ctx.ds_result.banking.transactions;
        let mut edges = Vec::new();

        for txn in transactions {
            let txn_ext_id = txn.transaction_id.to_string();
            let acct_ext_id = txn.account_id.to_string();

            let Some(txn_id) = ctx.id_map.get(&txn_ext_id) else {
                continue;
            };
            let Some(acct_id) = ctx.id_map.get(&acct_ext_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: txn_id,
                target: acct_id,
                edge_type: BANK_TXN_ACCOUNT,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("BANK_TXN_ACCOUNT: {} edges", edges.len());
        edges
    }

    /// RECON_ACCOUNT (code 82): bank_account -> GL account.
    ///
    /// Bank accounts don't have a direct `recon_gl_account` FK in the current
    /// model. This edge type will be populated once the banking model adds
    /// reconciliation account references. Currently returns empty.
    fn synthesize_recon_account(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        // No recon_gl_account FK on BankAccount yet; placeholder for future.
        let _ = ctx;
        debug!("RECON_ACCOUNT: 0 edges (no FK available in banking model)");
        Vec::new()
    }
}
