//! Entity relationship edge synthesizer.
//!
//! Produces cross-layer people edges linking documents and journal entries
//! to their creators/approvers, and vendor/customer ownership edges.
//!
//! ## Edge Types Produced
//!
//! | Code | Name                | Direction                          |
//! |------|---------------------|------------------------------------|
//! |  96  | DOC_CREATED_BY      | document -> employee               |
//! |  98  | JE_CREATED_BY       | JE -> employee                     |
//! | 135  | VENDOR_SUPPLIES     | vendor -> document (via vendor_id)  |
//! | 136  | CUSTOMER_HAS_ORDER  | customer -> document (via cust_id)  |
//! | 137  | JE_APPROVED_BY      | JE -> employee (via approved_by)    |

use std::collections::HashMap;

use tracing::debug;

use crate::error::ExportError;
use crate::traits::EdgeSynthesisContext;
use crate::types::ExportEdge;

/// Edge type codes produced by this synthesizer.
const DOC_CREATED_BY: u32 = 96;
const JE_CREATED_BY: u32 = 98;
const VENDOR_SUPPLIES: u32 = 135;
const CUSTOMER_HAS_ORDER: u32 = 136;
const JE_APPROVED_BY: u32 = 137;

/// Synthesizes entity relationship edges (people, vendor, customer cross-links).
pub struct EntityRelationshipEdgeSynthesizer;

impl crate::traits::EdgeSynthesizer for EntityRelationshipEdgeSynthesizer {
    fn name(&self) -> &'static str {
        "entity_relationships"
    }

    fn synthesize(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Result<Vec<ExportEdge>, ExportError> {
        let mut edges = Vec::new();

        if ctx.config.edge_synthesis.people_edges {
            edges.extend(self.synthesize_doc_created_by(ctx));
            edges.extend(self.synthesize_je_created_by(ctx));
            edges.extend(self.synthesize_je_approved_by(ctx));
        }

        if ctx.config.edge_synthesis.cross_layer_edges {
            edges.extend(self.synthesize_vendor_supplies(ctx));
            edges.extend(self.synthesize_customer_has_order(ctx));
        }

        debug!(
            "EntityRelationshipEdgeSynthesizer produced {} total edges",
            edges.len()
        );
        Ok(edges)
    }
}

impl EntityRelationshipEdgeSynthesizer {
    /// DOC_CREATED_BY (code 96): document -> employee.
    ///
    /// Uses `header.created_by_employee_id` on P2P/O2C documents.
    fn synthesize_doc_created_by(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let flows = &ctx.ds_result.document_flows;
        let mut edges = Vec::new();

        // Helper macro for documents with header.created_by_employee_id
        macro_rules! link_creator {
            ($docs:expr) => {
                for doc in $docs {
                    let Some(ref emp_ref) = doc.header.created_by_employee_id else {
                        continue;
                    };
                    let Some(doc_id) = ctx.id_map.get(&doc.header.document_id) else {
                        continue;
                    };
                    let Some(emp_id) = ctx.id_map.get(emp_ref) else {
                        continue;
                    };
                    edges.push(ExportEdge {
                        source: doc_id,
                        target: emp_id,
                        edge_type: DOC_CREATED_BY,
                        weight: 1.0,
                        properties: HashMap::new(),
                    });
                }
            };
        }

        link_creator!(&flows.purchase_orders);
        link_creator!(&flows.vendor_invoices);
        link_creator!(&flows.goods_receipts);
        link_creator!(&flows.sales_orders);
        link_creator!(&flows.customer_invoices);
        link_creator!(&flows.deliveries);
        link_creator!(&flows.payments);

        debug!("DOC_CREATED_BY: {} edges", edges.len());
        edges
    }

    /// JE_CREATED_BY (code 98): JE -> employee.
    ///
    /// Uses `header.created_by` on JournalEntryHeader.
    fn synthesize_je_created_by(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let journal_entries = &ctx.ds_result.journal_entries;
        let mut edges = Vec::new();

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

            edges.push(ExportEdge {
                source: je_id,
                target: emp_id,
                edge_type: JE_CREATED_BY,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("JE_CREATED_BY: {} edges", edges.len());
        edges
    }

    /// JE_APPROVED_BY (code 137): JE -> employee.
    ///
    /// Uses `header.approved_by` on JournalEntryHeader.
    fn synthesize_je_approved_by(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let journal_entries = &ctx.ds_result.journal_entries;
        let mut edges = Vec::new();

        for je in journal_entries {
            let Some(ref approver) = je.header.approved_by else {
                continue;
            };
            if approver.is_empty() {
                continue;
            }
            let je_ext_id = je.header.document_id.to_string();
            let Some(je_id) = ctx.id_map.get(&je_ext_id) else {
                continue;
            };
            let Some(approver_id) = ctx.id_map.get(approver) else {
                continue;
            };

            edges.push(ExportEdge {
                source: je_id,
                target: approver_id,
                edge_type: JE_APPROVED_BY,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("JE_APPROVED_BY: {} edges", edges.len());
        edges
    }

    /// VENDOR_SUPPLIES (code 135): vendor -> document (PO / vendor invoice).
    ///
    /// Links vendors to the P2P documents they are associated with via vendor_id.
    fn synthesize_vendor_supplies(&self, ctx: &mut EdgeSynthesisContext<'_>) -> Vec<ExportEdge> {
        let flows = &ctx.ds_result.document_flows;
        let mut edges = Vec::new();

        // PurchaseOrder.vendor_id -> PO
        for po in &flows.purchase_orders {
            let Some(vendor_id) = ctx.id_map.get(&po.vendor_id) else {
                continue;
            };
            let Some(po_id) = ctx.id_map.get(&po.header.document_id) else {
                continue;
            };
            edges.push(ExportEdge {
                source: vendor_id,
                target: po_id,
                edge_type: VENDOR_SUPPLIES,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        // VendorInvoice.vendor_id -> invoice
        for vi in &flows.vendor_invoices {
            let Some(vendor_id) = ctx.id_map.get(&vi.vendor_id) else {
                continue;
            };
            let Some(vi_id) = ctx.id_map.get(&vi.header.document_id) else {
                continue;
            };
            edges.push(ExportEdge {
                source: vendor_id,
                target: vi_id,
                edge_type: VENDOR_SUPPLIES,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("VENDOR_SUPPLIES: {} edges", edges.len());
        edges
    }

    /// CUSTOMER_HAS_ORDER (code 136): customer -> document (SO / customer invoice).
    ///
    /// Links customers to the O2C documents they are associated with via customer_id.
    fn synthesize_customer_has_order(
        &self,
        ctx: &mut EdgeSynthesisContext<'_>,
    ) -> Vec<ExportEdge> {
        let flows = &ctx.ds_result.document_flows;
        let mut edges = Vec::new();

        // SalesOrder.customer_id -> SO
        for so in &flows.sales_orders {
            let Some(cust_id) = ctx.id_map.get(&so.customer_id) else {
                continue;
            };
            let Some(so_id) = ctx.id_map.get(&so.header.document_id) else {
                continue;
            };
            edges.push(ExportEdge {
                source: cust_id,
                target: so_id,
                edge_type: CUSTOMER_HAS_ORDER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        // CustomerInvoice.customer_id -> invoice
        for ci in &flows.customer_invoices {
            let Some(cust_id) = ctx.id_map.get(&ci.customer_id) else {
                continue;
            };
            let Some(ci_id) = ctx.id_map.get(&ci.header.document_id) else {
                continue;
            };
            edges.push(ExportEdge {
                source: cust_id,
                target: ci_id,
                edge_type: CUSTOMER_HAS_ORDER,
                weight: 1.0,
                properties: HashMap::new(),
            });
        }

        debug!("CUSTOMER_HAS_ORDER: {} edges", edges.len());
        edges
    }
}
