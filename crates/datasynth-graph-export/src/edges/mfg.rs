//! Manufacturing edge synthesizer.
//!
//! Produces edges linking quality inspections to production orders
//! and materials.
//!
//! ## Edge Types Produced
//!
//! | Code | Name                 | Direction                          |
//! |------|----------------------|------------------------------------|
//! |  48  | QI_PRODUCTION_ORDER  | quality_inspection -> prod_order   |
//! |  95  | QI_REFERENCE         | quality_inspection -> material     |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const QI_PRODUCTION_ORDER: u32 = 48;
const QI_REFERENCE: u32 = 95;

/// Synthesizes manufacturing edges.
pub struct MFGEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for MFGEdgeSynthesizer {
    fn name(&self) -> &'static str {
        "mfg"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let mut edges = Vec::new();

        edges.extend(self.synthesize_qi_production_order(ctx));
        edges.extend(self.synthesize_qi_reference(ctx));

        debug!("MFGEdgeSynthesizer produced {} total edges", edges.len());
        Ok(edges)
    }
}

impl MFGEdgeSynthesizer {
    /// QI_PRODUCTION_ORDER (code 48): quality_inspection -> production_order.
    ///
    /// Uses the `reference_type` / `reference_id` fields on QualityInspection
    /// to link inspections to their source production orders.
    fn synthesize_qi_production_order(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let inspections = &ctx.ds_result.manufacturing.quality_inspections;
        let mut edges = Vec::new();

        for qi in inspections {
            // Only link inspections that reference a production order
            if qi.reference_type != "production_order" {
                continue;
            }

            let Some(qi_id) = ctx.id_map.get(&qi.inspection_id) else {
                continue;
            };
            let Some(po_id) = ctx.id_map.get(&qi.reference_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: qi_id,
                target: po_id,
                edge_type: QI_PRODUCTION_ORDER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("QI_PRODUCTION_ORDER: {} edges", edges.len());
        edges
    }

    /// QI_REFERENCE (code 95): quality_inspection -> material.
    ///
    /// Links inspections to the material/product being inspected via `material_id`.
    fn synthesize_qi_reference(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let inspections = &ctx.ds_result.manufacturing.quality_inspections;
        let mut edges = Vec::new();

        for qi in inspections {
            if qi.material_id.is_empty() {
                continue;
            }

            let Some(qi_id) = ctx.id_map.get(&qi.inspection_id) else {
                continue;
            };
            let Some(mat_id) = ctx.id_map.get(&qi.material_id) else {
                continue;
            };

            edges.push(ExportEdge {
                source: qi_id,
                target: mat_id,
                edge_type: QI_REFERENCE,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("QI_REFERENCE: {} edges", edges.len());
        edges
    }
}
