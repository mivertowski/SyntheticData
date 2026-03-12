//! Source-to-Contract (S2C) edge synthesizer.
//!
//! Produces edges linking sourcing projects, RFx events, bids, vendors,
//! and procurement contracts.
//!
//! ## Edge Types Produced
//!
//! | Code | Name             | Direction                         |
//! |------|------------------|-----------------------------------|
//! |  83  | PROJECT_RFX      | sourcing_project -> rfx            |
//! |  84  | RFX_BID          | rfx -> bid                         |
//! |  85  | BID_SUPPLIER     | bid -> vendor                      |
//! |  86  | CONTRACT_PROJECT | contract -> sourcing_project       |
//! |  87  | CONTRACT_VENDOR  | contract -> vendor                 |
//! |  88  | RFX_WINNER       | rfx -> winning bid                 |
//! |  89  | CONTRACT_PO      | contract -> purchase_order         |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const PROJECT_RFX: u32 = 83;
const RFX_BID: u32 = 84;
const BID_SUPPLIER: u32 = 85;
const CONTRACT_PROJECT: u32 = 86;
const CONTRACT_VENDOR: u32 = 87;
const RFX_WINNER: u32 = 88;
// Reserved for contract -> PO link; stub until PO FK available on contract model.
#[allow(dead_code)]
const CONTRACT_PO: u32 = 89;

/// Synthesizes Source-to-Contract procurement edges.
pub struct S2CEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for S2CEdgeSynthesizer {
    fn name(&self) -> &'static str {
        "s2c"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let mut edges = Vec::new();

        edges.extend(self.synthesize_project_rfx(ctx));
        edges.extend(self.synthesize_rfx_bid(ctx));
        edges.extend(self.synthesize_bid_supplier(ctx));
        edges.extend(self.synthesize_contract_project(ctx));
        edges.extend(self.synthesize_contract_vendor(ctx));
        edges.extend(self.synthesize_rfx_winner(ctx));
        edges.extend(self.synthesize_contract_po(ctx));

        debug!("S2CEdgeSynthesizer produced {} total edges", edges.len());
        Ok(edges)
    }
}

impl S2CEdgeSynthesizer {
    /// PROJECT_RFX (code 83): sourcing_project -> rfx.
    fn synthesize_project_rfx(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let rfx_events = &ctx.ds_result.sourcing.rfx_events;
        let mut edges = Vec::new();

        for rfx in rfx_events {
            let Some(proj_id) = ctx.id_map.get(&rfx.sourcing_project_id) else {
                continue;
            };
            let Some(rfx_id) = ctx.id_map.get(&rfx.rfx_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: proj_id,
                target: rfx_id,
                edge_type: PROJECT_RFX,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("PROJECT_RFX: {} edges", edges.len());
        edges
    }

    /// RFX_BID (code 84): rfx -> bid.
    fn synthesize_rfx_bid(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let bids = &ctx.ds_result.sourcing.bids;
        let mut edges = Vec::new();

        for bid in bids {
            let Some(rfx_id) = ctx.id_map.get(&bid.rfx_id) else {
                continue;
            };
            let Some(bid_id) = ctx.id_map.get(&bid.bid_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: rfx_id,
                target: bid_id,
                edge_type: RFX_BID,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("RFX_BID: {} edges", edges.len());
        edges
    }

    /// BID_SUPPLIER (code 85): bid -> vendor.
    fn synthesize_bid_supplier(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let bids = &ctx.ds_result.sourcing.bids;
        let mut edges = Vec::new();

        for bid in bids {
            let Some(bid_id) = ctx.id_map.get(&bid.bid_id) else {
                continue;
            };
            let Some(vendor_id) = ctx.id_map.get(&bid.vendor_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: bid_id,
                target: vendor_id,
                edge_type: BID_SUPPLIER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("BID_SUPPLIER: {} edges", edges.len());
        edges
    }

    /// CONTRACT_PROJECT (code 86): contract -> sourcing_project.
    fn synthesize_contract_project(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let contracts = &ctx.ds_result.sourcing.contracts;
        let mut edges = Vec::new();

        for contract in contracts {
            let Some(ref proj_ref) = contract.sourcing_project_id else {
                continue;
            };
            let Some(contract_id) = ctx.id_map.get(&contract.contract_id) else {
                continue;
            };
            let Some(proj_id) = ctx.id_map.get(proj_ref) else {
                continue;
            };

            edges.push(ExportEdge {
                source: contract_id,
                target: proj_id,
                edge_type: CONTRACT_PROJECT,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("CONTRACT_PROJECT: {} edges", edges.len());
        edges
    }

    /// CONTRACT_VENDOR (code 87): contract -> vendor.
    fn synthesize_contract_vendor(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let contracts = &ctx.ds_result.sourcing.contracts;
        let mut edges = Vec::new();

        for contract in contracts {
            let Some(contract_id) = ctx.id_map.get(&contract.contract_id) else {
                continue;
            };
            let Some(vendor_id) = ctx.id_map.get(&contract.vendor_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: contract_id,
                target: vendor_id,
                edge_type: CONTRACT_VENDOR,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("CONTRACT_VENDOR: {} edges", edges.len());
        edges
    }

    /// RFX_WINNER (code 88): rfx -> winning bid.
    fn synthesize_rfx_winner(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let rfx_events = &ctx.ds_result.sourcing.rfx_events;
        let mut edges = Vec::new();

        for rfx in rfx_events {
            let Some(ref winner_ref) = rfx.awarded_bid_id else {
                continue;
            };
            let Some(rfx_id) = ctx.id_map.get(&rfx.rfx_id) else {
                continue;
            };
            let Some(winner_id) = ctx.id_map.get(winner_ref) else {
                continue;
            };

            edges.push(ExportEdge {
                source: rfx_id,
                target: winner_id,
                edge_type: RFX_WINNER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("RFX_WINNER: {} edges", edges.len());
        edges
    }

    /// CONTRACT_PO (code 89): contract -> purchase_order.
    ///
    /// The ProcurementContract model doesn't have a direct PO reference.
    /// In practice the PO links back to the contract via its own FK.
    /// This edge type will produce edges once linked PO IDs are available.
    fn synthesize_contract_po(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        // No direct PO FK on ProcurementContract; placeholder for future.
        let _ = ctx;
        debug!("CONTRACT_PO: 0 edges (no PO FK on contract model)");
        Vec::new()
    }
}
