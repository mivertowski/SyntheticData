//! Property serializers for Procure-to-Pay (P2P) document entities.
//!
//! Covers: PurchaseOrder, GoodsReceipt, VendorInvoice, Payment.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

// ──────────────────────────── Purchase Order ────────────────────────

/// Property serializer for purchase orders (entity type code 200).
pub struct PurchaseOrderPropertySerializer;

impl PropertySerializer for PurchaseOrderPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "purchase_order"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let po = ctx
            .ds_result
            .document_flows
            .purchase_orders
            .iter()
            .find(|p| p.header.document_id == node_external_id)?;

        let mut props = HashMap::with_capacity(12);

        props.insert(
            "documentId".into(),
            Value::String(po.header.document_id.clone()),
        );
        props.insert("documentType".into(), Value::String("PurchaseOrder".into()));
        props.insert(
            "companyCode".into(),
            Value::String(po.header.company_code.clone()),
        );
        props.insert(
            "documentDate".into(),
            Value::String(po.header.document_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", po.header.status)),
        );
        props.insert("vendorId".into(), Value::String(po.vendor_id.clone()));
        props.insert("poType".into(), Value::String(format!("{:?}", po.po_type)));
        props.insert("amount".into(), serde_json::json!(po.total_gross_amount));
        props.insert("netAmount".into(), serde_json::json!(po.total_net_amount));
        props.insert("currency".into(), Value::String(po.header.currency.clone()));
        props.insert("lineCount".into(), Value::Number(po.items.len().into()));
        props.insert(
            "purchasingOrg".into(),
            Value::String(po.purchasing_org.clone()),
        );

        Some(props)
    }
}

// ──────────────────────────── Goods Receipt ─────────────────────────

/// Property serializer for goods receipts (entity type code 201).
pub struct GoodsReceiptPropertySerializer;

impl PropertySerializer for GoodsReceiptPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "goods_receipt"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let gr = ctx
            .ds_result
            .document_flows
            .goods_receipts
            .iter()
            .find(|g| g.header.document_id == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert(
            "documentId".into(),
            Value::String(gr.header.document_id.clone()),
        );
        props.insert("documentType".into(), Value::String("GoodsReceipt".into()));
        props.insert(
            "companyCode".into(),
            Value::String(gr.header.company_code.clone()),
        );
        props.insert(
            "documentDate".into(),
            Value::String(gr.header.document_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", gr.header.status)),
        );
        props.insert("amount".into(), serde_json::json!(gr.total_value));
        props.insert("totalQuantity".into(), serde_json::json!(gr.total_quantity));
        if let Some(ref po_id) = gr.purchase_order_id {
            props.insert("purchaseOrderId".into(), Value::String(po_id.clone()));
        }
        if let Some(ref vendor) = gr.vendor_id {
            props.insert("vendorId".into(), Value::String(vendor.clone()));
        }
        props.insert("plant".into(), Value::String(gr.plant.clone()));

        Some(props)
    }
}

// ──────────────────────────── Vendor Invoice ────────────────────────

/// Property serializer for vendor invoices (entity type code 202).
pub struct VendorInvoicePropertySerializer;

impl PropertySerializer for VendorInvoicePropertySerializer {
    fn entity_type(&self) -> &'static str {
        "vendor_invoice"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let inv = ctx
            .ds_result
            .document_flows
            .vendor_invoices
            .iter()
            .find(|i| i.header.document_id == node_external_id)?;

        let mut props = HashMap::with_capacity(12);

        props.insert(
            "documentId".into(),
            Value::String(inv.header.document_id.clone()),
        );
        props.insert("documentType".into(), Value::String("VendorInvoice".into()));
        props.insert(
            "companyCode".into(),
            Value::String(inv.header.company_code.clone()),
        );
        props.insert(
            "documentDate".into(),
            Value::String(inv.header.document_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", inv.header.status)),
        );
        props.insert("vendorId".into(), Value::String(inv.vendor_id.clone()));
        props.insert(
            "invoiceType".into(),
            Value::String(format!("{:?}", inv.invoice_type)),
        );
        props.insert("amount".into(), serde_json::json!(inv.gross_amount));
        props.insert("netAmount".into(), serde_json::json!(inv.net_amount));
        props.insert("taxAmount".into(), serde_json::json!(inv.tax_amount));
        props.insert(
            "payableAmount".into(),
            serde_json::json!(inv.payable_amount),
        );
        props.insert("lineCount".into(), Value::Number(inv.items.len().into()));

        Some(props)
    }
}

// ──────────────────────────── Payment ────────────────────────────────

/// Property serializer for payments (entity type code 203).
pub struct PaymentPropertySerializer;

impl PropertySerializer for PaymentPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "payment"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let pmt = ctx
            .ds_result
            .document_flows
            .payments
            .iter()
            .find(|p| p.header.document_id == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert(
            "documentId".into(),
            Value::String(pmt.header.document_id.clone()),
        );
        props.insert("documentType".into(), Value::String("Payment".into()));
        props.insert(
            "companyCode".into(),
            Value::String(pmt.header.company_code.clone()),
        );
        props.insert(
            "documentDate".into(),
            Value::String(pmt.header.document_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", pmt.payment_status)),
        );
        props.insert("amount".into(), serde_json::json!(pmt.amount));
        props.insert("currency".into(), Value::String(pmt.currency.clone()));
        props.insert(
            "paymentType".into(),
            Value::String(format!("{:?}", pmt.payment_type)),
        );
        props.insert(
            "paymentMethod".into(),
            Value::String(format!("{:?}", pmt.payment_method)),
        );
        props.insert(
            "businessPartnerId".into(),
            Value::String(pmt.business_partner_id.clone()),
        );
        props.insert("isVendor".into(), Value::Bool(pmt.is_vendor));

        Some(props)
    }
}

// ──────────────────────────── Vendor ─────────────────────────────────

/// Property serializer for vendors (entity type code 350).
///
/// Serializes risk-relevant fields from the `Vendor` master data model.
pub struct VendorPropertySerializer;

impl PropertySerializer for VendorPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "vendor"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let vendor = ctx
            .ds_result
            .master_data
            .vendors
            .iter()
            .find(|v| v.vendor_id == node_external_id)?;

        let mut props = HashMap::with_capacity(14);

        props.insert("vendorId".into(), Value::String(vendor.vendor_id.clone()));
        props.insert("name".into(), Value::String(vendor.name.clone()));
        props.insert(
            "vendorCategory".into(),
            Value::String(format!("{:?}", vendor.vendor_type)),
        );
        props.insert("country".into(), Value::String(vendor.country.clone()));
        props.insert(
            "paymentTerms".into(),
            Value::String(format!("{:?}", vendor.payment_terms)),
        );
        props.insert(
            "paymentTermsDays".into(),
            Value::Number(vendor.payment_terms_days.into()),
        );
        props.insert("isOneTime".into(), Value::Bool(vendor.is_one_time));
        props.insert("isActive".into(), Value::Bool(vendor.is_active));
        props.insert(
            "isIntercompany".into(),
            Value::Bool(vendor.is_intercompany),
        );
        props.insert("currency".into(), Value::String(vendor.currency.clone()));
        props.insert(
            "withholdingTaxApplicable".into(),
            Value::Bool(vendor.withholding_tax_applicable),
        );
        if let Some(ref tax_id) = vendor.tax_id {
            props.insert("taxId".into(), Value::String(tax_id.clone()));
        }
        if let Some(ref purchasing_org) = vendor.purchasing_org {
            props.insert(
                "purchasingOrg".into(),
                Value::String(purchasing_org.clone()),
            );
        }
        if let Some(ref ic_code) = vendor.intercompany_code {
            props.insert("intercompanyCode".into(), Value::String(ic_code.clone()));
        }

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_types_are_correct() {
        assert_eq!(
            PurchaseOrderPropertySerializer.entity_type(),
            "purchase_order"
        );
        assert_eq!(
            GoodsReceiptPropertySerializer.entity_type(),
            "goods_receipt"
        );
        assert_eq!(
            VendorInvoicePropertySerializer.entity_type(),
            "vendor_invoice"
        );
        assert_eq!(PaymentPropertySerializer.entity_type(), "payment");
        assert_eq!(VendorPropertySerializer.entity_type(), "vendor");
    }
}
