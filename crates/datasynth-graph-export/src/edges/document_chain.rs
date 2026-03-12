//! Document chain edge synthesizer.
//!
//! Produces edges that link P2P (Procure-to-Pay) and O2C (Order-to-Cash)
//! documents into chains based on their foreign-key references.
//!
//! ## Edge Types Produced
//!
//! | Code | Name                        | Direction                     |
//! |------|-----------------------------|-------------------------------|
//! |  60  | ReferencesOrder             | VendorInvoice -> PurchaseOrder|
//! |  62  | PaysInvoice                 | Payment -> VendorInvoice      |
//! |  64  | FulfillsOrder               | GoodsReceipt -> PurchaseOrder |
//! |  66  | BillsOrder                  | CustomerInvoice -> SalesOrder |
//! |  68  | DeliversOrder               | Delivery -> SalesOrder        |
//! |  69  | InvoiceReferencesDelivery   | CustomerInvoice -> Delivery   |
//!
//! **Note:** Codes 70 (PostsToAccount) and 99 (DocPostsJe) belong in the
//! AccountingEdgeSynthesizer (Task 11), not here.

use std::collections::HashMap;

use tracing::debug;

use crate::error::{ExportError, WarningSeverity};
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const REFERENCES_ORDER: u32 = 60;
const PAYS_INVOICE: u32 = 62;
const FULFILLS_ORDER: u32 = 64;
const BILLS_ORDER: u32 = 66;
const DELIVERS_ORDER: u32 = 68;
const INVOICE_REFERENCES_DELIVERY: u32 = 69;

/// Synthesizes P2P and O2C document chain edges.
///
/// For each document type, iterates the flattened document lists in
/// `ds_result.document_flows` and resolves foreign-key references
/// to numeric node IDs via `ctx.id_map`. Documents whose source or
/// target was budget-dropped (not in id_map) are silently skipped.
pub struct DocumentChainEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for DocumentChainEdgeSynthesizer {
    fn name(&self) -> &'static str {
        "document_chain"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let mut edges = Vec::new();

        // P2P chain edges
        edges.extend(self.synthesize_references_order(ctx));
        edges.extend(self.synthesize_pays_invoice(ctx));
        edges.extend(self.synthesize_fulfills_order(ctx));

        // O2C chain edges
        edges.extend(self.synthesize_bills_order(ctx));
        edges.extend(self.synthesize_delivers_order(ctx));
        edges.extend(self.synthesize_invoice_references_delivery(ctx));

        debug!(
            "DocumentChainEdgeSynthesizer produced {} total edges",
            edges.len()
        );
        Ok(edges)
    }
}

impl DocumentChainEdgeSynthesizer {
    /// ReferencesOrder (code 60): VendorInvoice -> PurchaseOrder.
    fn synthesize_references_order(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let invoices = &ctx.ds_result.document_flows.vendor_invoices;
        let mut edges = Vec::new();
        let mut skipped = 0usize;

        for invoice in invoices {
            let Some(ref po_ref) = invoice.purchase_order_id else {
                continue;
            };

            let Some(invoice_id) = ctx.id_map.get(&invoice.header.document_id) else {
                skipped += 1;
                continue;
            };
            let Some(po_id) = ctx.id_map.get(po_ref) else {
                skipped += 1;
                continue;
            };

            edges.push(ExportEdge {
                source: invoice_id,
                target: po_id,
                edge_type: REFERENCES_ORDER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        if skipped > 0 {
            ctx.warnings.add(
                "document_chain",
                WarningSeverity::Info,
                format!("ReferencesOrder: skipped {skipped} edges (budget-dropped nodes)"),
            );
        }
        debug!("ReferencesOrder: {} edges ({skipped} skipped)", edges.len());
        edges
    }

    /// PaysInvoice (code 62): Payment -> VendorInvoice (or CustomerInvoice).
    ///
    /// Iterates payment allocations to find which invoices each payment covers.
    fn synthesize_pays_invoice(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let payments = &ctx.ds_result.document_flows.payments;
        let mut edges = Vec::new();
        let mut skipped = 0usize;

        for payment in payments {
            let Some(payment_id) = ctx.id_map.get(&payment.header.document_id) else {
                skipped += 1;
                continue;
            };

            for allocation in &payment.allocations {
                let Some(invoice_id) = ctx.id_map.get(&allocation.invoice_id) else {
                    skipped += 1;
                    continue;
                };

                edges.push(ExportEdge {
                    source: payment_id,
                    target: invoice_id,
                    edge_type: PAYS_INVOICE,
                    weight: 1.0,
                    properties: HashMap::new(),
                });
            }
        }

        if skipped > 0 {
            ctx.warnings.add(
                "document_chain",
                WarningSeverity::Info,
                format!("PaysInvoice: skipped {skipped} edges (budget-dropped nodes)"),
            );
        }
        debug!("PaysInvoice: {} edges ({skipped} skipped)", edges.len());
        edges
    }

    /// FulfillsOrder (code 64): GoodsReceipt -> PurchaseOrder.
    fn synthesize_fulfills_order(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let goods_receipts = &ctx.ds_result.document_flows.goods_receipts;
        let mut edges = Vec::new();
        let mut skipped = 0usize;

        for gr in goods_receipts {
            let Some(ref po_ref) = gr.purchase_order_id else {
                continue;
            };

            let Some(gr_id) = ctx.id_map.get(&gr.header.document_id) else {
                skipped += 1;
                continue;
            };
            let Some(po_id) = ctx.id_map.get(po_ref) else {
                skipped += 1;
                continue;
            };

            edges.push(ExportEdge {
                source: gr_id,
                target: po_id,
                edge_type: FULFILLS_ORDER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        if skipped > 0 {
            ctx.warnings.add(
                "document_chain",
                WarningSeverity::Info,
                format!("FulfillsOrder: skipped {skipped} edges (budget-dropped nodes)"),
            );
        }
        debug!("FulfillsOrder: {} edges ({skipped} skipped)", edges.len());
        edges
    }

    /// BillsOrder (code 66): CustomerInvoice -> SalesOrder.
    fn synthesize_bills_order(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let invoices = &ctx.ds_result.document_flows.customer_invoices;
        let mut edges = Vec::new();
        let mut skipped = 0usize;

        for invoice in invoices {
            let Some(ref so_ref) = invoice.sales_order_id else {
                continue;
            };

            let Some(invoice_id) = ctx.id_map.get(&invoice.header.document_id) else {
                skipped += 1;
                continue;
            };
            let Some(so_id) = ctx.id_map.get(so_ref) else {
                skipped += 1;
                continue;
            };

            edges.push(ExportEdge {
                source: invoice_id,
                target: so_id,
                edge_type: BILLS_ORDER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        if skipped > 0 {
            ctx.warnings.add(
                "document_chain",
                WarningSeverity::Info,
                format!("BillsOrder: skipped {skipped} edges (budget-dropped nodes)"),
            );
        }
        debug!("BillsOrder: {} edges ({skipped} skipped)", edges.len());
        edges
    }

    /// DeliversOrder (code 68): Delivery -> SalesOrder.
    fn synthesize_delivers_order(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let deliveries = &ctx.ds_result.document_flows.deliveries;
        let mut edges = Vec::new();
        let mut skipped = 0usize;

        for delivery in deliveries {
            let Some(ref so_ref) = delivery.sales_order_id else {
                continue;
            };

            let Some(delivery_id) = ctx.id_map.get(&delivery.header.document_id) else {
                skipped += 1;
                continue;
            };
            let Some(so_id) = ctx.id_map.get(so_ref) else {
                skipped += 1;
                continue;
            };

            edges.push(ExportEdge {
                source: delivery_id,
                target: so_id,
                edge_type: DELIVERS_ORDER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        if skipped > 0 {
            ctx.warnings.add(
                "document_chain",
                WarningSeverity::Info,
                format!("DeliversOrder: skipped {skipped} edges (budget-dropped nodes)"),
            );
        }
        debug!("DeliversOrder: {} edges ({skipped} skipped)", edges.len());
        edges
    }

    /// InvoiceReferencesDelivery (code 69): CustomerInvoice -> Delivery.
    fn synthesize_invoice_references_delivery(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let invoices = &ctx.ds_result.document_flows.customer_invoices;
        let mut edges = Vec::new();
        let mut skipped = 0usize;

        for invoice in invoices {
            let Some(ref del_ref) = invoice.delivery_id else {
                continue;
            };

            let Some(invoice_id) = ctx.id_map.get(&invoice.header.document_id) else {
                skipped += 1;
                continue;
            };
            let Some(delivery_id) = ctx.id_map.get(del_ref) else {
                skipped += 1;
                continue;
            };

            edges.push(ExportEdge {
                source: invoice_id,
                target: delivery_id,
                edge_type: INVOICE_REFERENCES_DELIVERY,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        if skipped > 0 {
            ctx.warnings.add(
                "document_chain",
                WarningSeverity::Info,
                format!(
                    "InvoiceReferencesDelivery: skipped {skipped} edges (budget-dropped nodes)"
                ),
            );
        }
        debug!(
            "InvoiceReferencesDelivery: {} edges ({skipped} skipped)",
            edges.len()
        );
        edges
    }
}
