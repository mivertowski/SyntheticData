//! Property serializers for Manufacturing entities.
//!
//! Covers: ProductionOrder, QualityInspection, CycleCount.

use std::collections::HashMap;

use serde_json::Value;

use crate::traits::{PropertySerializer, SerializationContext};

// ──────────────────────────── Production Order ──────────────────────

/// Property serializer for production orders (entity type code 700).
pub struct ProductionOrderPropertySerializer;

impl PropertySerializer for ProductionOrderPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "production_order"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let po = ctx
            .ds_result
            .manufacturing
            .production_orders
            .iter()
            .find(|p| p.order_id == node_external_id)?;

        let mut props = HashMap::with_capacity(12);

        props.insert("orderId".into(), Value::String(po.order_id.clone()));
        props.insert(
            "companyCode".into(),
            Value::String(po.company_code.clone()),
        );
        props.insert(
            "materialId".into(),
            Value::String(po.material_id.clone()),
        );
        props.insert(
            "materialDescription".into(),
            Value::String(po.material_description.clone()),
        );
        props.insert(
            "orderType".into(),
            Value::String(format!("{:?}", po.order_type)),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", po.status)),
        );
        props.insert(
            "plannedQuantity".into(),
            serde_json::json!(po.planned_quantity),
        );
        props.insert(
            "actualQuantity".into(),
            serde_json::json!(po.actual_quantity),
        );
        props.insert(
            "scrapQuantity".into(),
            serde_json::json!(po.scrap_quantity),
        );
        props.insert(
            "plannedStart".into(),
            Value::String(po.planned_start.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "plannedEnd".into(),
            Value::String(po.planned_end.format("%Y-%m-%d").to_string()),
        );
        props.insert("type".into(), Value::String("production_order".into()));

        Some(props)
    }
}

// ──────────────────────────── Quality Inspection ────────────────────

/// Property serializer for quality inspections (entity type code 701).
pub struct QualityInspectionPropertySerializer;

impl PropertySerializer for QualityInspectionPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "quality_inspection"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let qi = ctx
            .ds_result
            .manufacturing
            .quality_inspections
            .iter()
            .find(|q| q.inspection_id == node_external_id)?;

        let mut props = HashMap::with_capacity(12);

        props.insert(
            "inspectionId".into(),
            Value::String(qi.inspection_id.clone()),
        );
        props.insert(
            "companyCode".into(),
            Value::String(qi.company_code.clone()),
        );
        props.insert(
            "inspectionType".into(),
            Value::String(format!("{:?}", qi.inspection_type)),
        );
        props.insert(
            "materialId".into(),
            Value::String(qi.material_id.clone()),
        );
        props.insert(
            "materialDescription".into(),
            Value::String(qi.material_description.clone()),
        );
        props.insert(
            "inspectionDate".into(),
            Value::String(qi.inspection_date.format("%Y-%m-%d").to_string()),
        );
        props.insert("lotSize".into(), serde_json::json!(qi.lot_size));
        props.insert("sampleSize".into(), serde_json::json!(qi.sample_size));
        props.insert(
            "defectsFound".into(),
            Value::Number(qi.defect_count.into()),
        );
        props.insert(
            "referenceType".into(),
            Value::String(qi.reference_type.clone()),
        );
        props.insert(
            "referenceId".into(),
            Value::String(qi.reference_id.clone()),
        );
        props.insert("type".into(), Value::String("quality_inspection".into()));

        Some(props)
    }
}

// ──────────────────────────── Cycle Count ────────────────────────────

/// Property serializer for cycle counts (entity type code 702).
pub struct CycleCountPropertySerializer;

impl PropertySerializer for CycleCountPropertySerializer {
    fn entity_type(&self) -> &'static str {
        "cycle_count"
    }

    fn serialize(
        &self,
        node_external_id: &str,
        ctx: &SerializationContext<'_>,
    ) -> Option<HashMap<String, Value>> {
        let cc = ctx
            .ds_result
            .manufacturing
            .cycle_counts
            .iter()
            .find(|c| c.count_id == node_external_id)?;

        let mut props = HashMap::with_capacity(10);

        props.insert("countId".into(), Value::String(cc.count_id.clone()));
        props.insert(
            "companyCode".into(),
            Value::String(cc.company_code.clone()),
        );
        props.insert(
            "warehouseId".into(),
            Value::String(cc.warehouse_id.clone()),
        );
        props.insert(
            "countDate".into(),
            Value::String(cc.count_date.format("%Y-%m-%d").to_string()),
        );
        props.insert(
            "status".into(),
            Value::String(format!("{:?}", cc.status)),
        );
        props.insert(
            "totalItemsCounted".into(),
            Value::Number(cc.total_items_counted.into()),
        );
        props.insert(
            "totalVariances".into(),
            Value::Number(cc.total_variances.into()),
        );
        props.insert(
            "varianceRate".into(),
            serde_json::json!(cc.variance_rate),
        );
        props.insert("type".into(), Value::String("cycle_count".into()));

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
            ProductionOrderPropertySerializer.entity_type(),
            "production_order"
        );
        assert_eq!(
            QualityInspectionPropertySerializer.entity_type(),
            "quality_inspection"
        );
        assert_eq!(CycleCountPropertySerializer.entity_type(), "cycle_count");
    }
}
