//! Property serializers for Order-to-Cash (O2C) document entities.
//!
//! Covers: SalesOrder, Delivery, CustomerInvoice.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

// ──────────────────────────── Sales Order ────────────────────────

/// Property serializer for sales orders (entity type code 210).
pub struct SalesOrderPropertySerializer;

impl PropertySerializer for SalesOrderPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "sales_order"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let so = ctx
            .ds_result
            .document_flows
            .sales_orders
            .iter()
            .find(|s| s.header.document_id == node_external_id)?;

        let mut props = HashMap::with_capacity(12);

        props.insert(
            "documentId".into(),
            Value::String(so.header.document_id.clone()),
        );
        props.insert("documentType".into(), Value::String("SalesOrder".into()));
        props.insert(
            "companyCode".into(),
            Value::String(so.header.company_code.clone()),
        );
        props.insert(
            "documentDate".into(),
            Value::String(so.header.document_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", so.header.status)),
        );
        props.insert("customerId".into(), Value::String(so.customer_id.clone()));
        props.insert(
            "soType".into(),
            Value::String(format!("{:?}", so.so_type)),
        );
        props.insert(
            "amount".into(),
            serde_json::json!(so.total_gross_amount),
        );
        props.insert(
            "netAmount".into(),
            serde_json::json!(so.total_net_amount),
        );
        props.insert(
            "salesOrg".into(),
            Value::String(so.sales_org.clone()),
        );
        props.insert(
            "lineCount".into(),
            Value::Number(so.items.len().into()),
        );

        Some(props)
    }
}

// ──────────────────────────── Delivery ──────────────────────────────

/// Property serializer for deliveries (entity type code 211).
pub struct DeliveryPropertySerializer;

impl PropertySerializer for DeliveryPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "delivery"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let del = ctx
            .ds_result
            .document_flows
            .deliveries
            .iter()
            .find(|d| d.header.document_id == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert(
            "documentId".into(),
            Value::String(del.header.document_id.clone()),
        );
        props.insert("documentType".into(), Value::String("Delivery".into()));
        props.insert(
            "companyCode".into(),
            Value::String(del.header.company_code.clone()),
        );
        props.insert(
            "documentDate".into(),
            Value::String(del.header.document_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", del.delivery_status)),
        );
        props.insert("customerId".into(), Value::String(del.customer_id.clone()));
        props.insert(
            "deliveryType".into(),
            Value::String(format!("{:?}", del.delivery_type)),
        );
        props.insert(
            "totalQuantity".into(),
            serde_json::json!(del.total_quantity),
        );
        if let Some(ref so_id) = del.sales_order_id {
            props.insert("salesOrderId".into(), Value::String(so_id.clone()));
        }
        props.insert(
            "shippingPoint".into(),
            Value::String(del.shipping_point.clone()),
        );

        Some(props)
    }
}

// ──────────────────────────── Customer Invoice ──────────────────────

/// Property serializer for customer invoices (entity type code 212).
pub struct CustomerInvoicePropertySerializer;

impl PropertySerializer for CustomerInvoicePropertySerializer {
    fn entity_type(&self) -> &'static str {
        "customer_invoice"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let inv = ctx
            .ds_result
            .document_flows
            .customer_invoices
            .iter()
            .find(|i| i.header.document_id == node_external_id)?;

        let mut props = HashMap::with_capacity(12);

        props.insert(
            "documentId".into(),
            Value::String(inv.header.document_id.clone()),
        );
        props.insert(
            "documentType".into(),
            Value::String("CustomerInvoice".into()),
        );
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
        props.insert("customerId".into(), Value::String(inv.customer_id.clone()));
        props.insert(
            "invoiceType".into(),
            Value::String(format!("{:?}", inv.invoice_type)),
        );
        props.insert(
            "amount".into(),
            serde_json::json!(inv.total_gross_amount),
        );
        props.insert(
            "netAmount".into(),
            serde_json::json!(inv.total_net_amount),
        );
        props.insert(
            "taxAmount".into(),
            serde_json::json!(inv.total_tax_amount),
        );
        props.insert(
            "salesOrg".into(),
            Value::String(inv.sales_org.clone()),
        );
        props.insert(
            "lineCount".into(),
            Value::Number(inv.items.len().into()),
        );

        Some(props)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn entity_types_are_correct() {
        assert_eq!(SalesOrderPropertySerializer.entity_type(), "sales_order");
        assert_eq!(DeliveryPropertySerializer.entity_type(), "delivery");
        assert_eq!(
            CustomerInvoicePropertySerializer.entity_type(),
            "customer_invoice"
        );
    }
}
