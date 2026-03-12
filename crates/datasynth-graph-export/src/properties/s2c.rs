//! Property serializers for Source-to-Contract (S2C) entities.
//!
//! Covers: SourcingProject, RfxEvent, SupplierBid, ProcurementContract.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

// ──────────────────────────── Sourcing Project ──────────────────────

/// Property serializer for sourcing projects (entity type code 500).
pub struct SourcingProjectPropertySerializer;

impl PropertySerializer for SourcingProjectPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "sourcing_project"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let proj = ctx
            .ds_result
            .sourcing
            .sourcing_projects
            .iter()
            .find(|p| p.project_id == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert(
            "projectId".into(),
            Value::String(proj.project_id.clone()),
        );
        props.insert(
            "projectName".into(),
            Value::String(proj.project_name.clone()),
        );
        props.insert(
            "projectType".into(),
            Value::String(format!("{:?}", proj.project_type)),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", proj.status)),
        );
        props.insert(
            "estimatedAnnualSpend".into(),
            serde_json::json!(proj.estimated_annual_spend),
        );
        props.insert(
            "targetSavingsPct".into(),
            serde_json::json!(proj.target_savings_pct),
        );
        props.insert("ownerId".into(), Value::String(proj.owner_id.clone()));
        props.insert(
            "companyCode".into(),
            Value::String(proj.company_code.clone()),
        );
        props.insert("type".into(), Value::String("sourcing_project".into()));

        Some(props)
    }
}

// ──────────────────────────── RFx Event ─────────────────────────────

/// Property serializer for RFx events (entity type code 501).
pub struct RfxEventPropertySerializer;

impl PropertySerializer for RfxEventPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "rfx_event"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let rfx = ctx
            .ds_result
            .sourcing
            .rfx_events
            .iter()
            .find(|r| r.rfx_id == node_external_id)?;

        let mut props = HashMap::with_capacity(8);

        props.insert("rfxId".into(), Value::String(rfx.rfx_id.clone()));
        props.insert(
            "rfxType".into(),
            Value::String(format!("{:?}", rfx.rfx_type)),
        );
        props.insert("title".into(), Value::String(rfx.title.clone()));
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", rfx.status)),
        );
        props.insert(
            "sourcingProjectId".into(),
            Value::String(rfx.sourcing_project_id.clone()),
        );
        props.insert(
            "companyCode".into(),
            Value::String(rfx.company_code.clone()),
        );
        props.insert("type".into(), Value::String("rfx_event".into()));

        Some(props)
    }
}

// ──────────────────────────── Supplier Bid ──────────────────────────

/// Property serializer for supplier bids (entity type code 502).
pub struct SupplierBidPropertySerializer;

impl PropertySerializer for SupplierBidPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "supplier_bid"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let bid = ctx
            .ds_result
            .sourcing
            .bids
            .iter()
            .find(|b| b.bid_id == node_external_id)?;

        let mut props = HashMap::with_capacity(8);

        props.insert("bidId".into(), Value::String(bid.bid_id.clone()));
        props.insert("rfxId".into(), Value::String(bid.rfx_id.clone()));
        props.insert("vendorId".into(), Value::String(bid.vendor_id.clone()));
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", bid.status)),
        );
        props.insert("amount".into(), serde_json::json!(bid.total_amount));
        props.insert(
            "submissionDate".into(),
            Value::String(bid.submission_date.format("%Y-%m-%d").to_string()),
        );
        props.insert("type".into(), Value::String("supplier_bid".into()));

        Some(props)
    }
}

// ──────────────────────────── Procurement Contract ──────────────────

/// Property serializer for procurement contracts (entity type code 504).
pub struct ProcurementContractPropertySerializer;

impl PropertySerializer for ProcurementContractPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "procurement_contract"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let contract = ctx
            .ds_result
            .sourcing
            .contracts
            .iter()
            .find(|c| c.contract_id == node_external_id)?;

        let mut props = HashMap::with_capacity(12);

        props.insert(
            "contractId".into(),
            Value::String(contract.contract_id.clone()),
        );
        props.insert("title".into(), Value::String(contract.title.clone()));
        props.insert(
            "contractType".into(),
            Value::String(format!("{:?}", contract.contract_type)),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", contract.status)),
        );
        props.insert(
            "vendorId".into(),
            Value::String(contract.vendor_id.clone()),
        );
        props.insert(
            "totalValue".into(),
            serde_json::json!(contract.total_value),
        );
        props.insert(
            "consumedValue".into(),
            serde_json::json!(contract.consumed_value),
        );
        props.insert(
            "startDate".into(),
            Value::String(contract.start_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "endDate".into(),
            Value::String(contract.end_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "companyCode".into(),
            Value::String(contract.company_code.clone()),
        );
        props.insert("type".into(), Value::String("procurement_contract".into()));

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
            SourcingProjectPropertySerializer.entity_type(),
            "sourcing_project"
        );
        assert_eq!(RfxEventPropertySerializer.entity_type(), "rfx_event");
        assert_eq!(SupplierBidPropertySerializer.entity_type(), "supplier_bid");
        assert_eq!(
            ProcurementContractPropertySerializer.entity_type(),
            "procurement_contract"
        );
    }
}
