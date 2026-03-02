//! MFG (Manufacturing) process event generator.
//!
//! Generates OCPM events for the complete manufacturing flow:
//! Create ProdOrder → Release → Start/Complete Operations → Quality Inspect →
//! Confirm → Close; Cycle Count → Reconcile

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::{CaseGenerationResult, OcpmEventGenerator, OcpmUuidFactory, VariantType};
use crate::models::{ActivityType, EventObjectRef, ObjectAttributeValue, ObjectType};
use datasynth_core::models::BusinessProcess;

/// MFG document references for event generation.
#[derive(Debug, Clone)]
pub struct MfgDocuments {
    /// Production order ID
    pub order_id: String,
    /// Production order UUID
    pub order_uuid: Uuid,
    /// Material ID
    pub material_id: String,
    /// Company code
    pub company_code: String,
    /// Quantity
    pub quantity: Decimal,
    /// Operation IDs
    pub operation_ids: Vec<String>,
    /// Quality inspection ID
    pub inspection_id: Option<String>,
    /// Inspection UUID
    pub inspection_uuid: Option<Uuid>,
    /// Cycle count ID
    pub cycle_count_id: Option<String>,
    /// Cycle count UUID
    pub cycle_count_uuid: Option<Uuid>,
}

impl MfgDocuments {
    /// Create new MFG documents.
    pub fn new(
        order_id: &str,
        material_id: &str,
        company_code: &str,
        quantity: Decimal,
        factory: &OcpmUuidFactory,
    ) -> Self {
        Self {
            order_id: order_id.into(),
            order_uuid: factory.next_document_id(),
            material_id: material_id.into(),
            company_code: company_code.into(),
            quantity,
            operation_ids: Vec::new(),
            inspection_id: None,
            inspection_uuid: None,
            cycle_count_id: None,
            cycle_count_uuid: None,
        }
    }

    /// Add operation IDs.
    pub fn with_operations(mut self, ids: Vec<&str>) -> Self {
        self.operation_ids = ids.into_iter().map(String::from).collect();
        self
    }

    /// Set inspection info.
    pub fn with_inspection(mut self, inspection_id: &str, factory: &OcpmUuidFactory) -> Self {
        self.inspection_id = Some(inspection_id.into());
        self.inspection_uuid = Some(factory.next_document_id());
        self
    }

    /// Set cycle count info with deterministic UUID from the given factory.
    pub fn with_cycle_count(mut self, count_id: &str, factory: &OcpmUuidFactory) -> Self {
        self.cycle_count_id = Some(count_id.into());
        self.cycle_count_uuid = Some(factory.next_document_id());
        self
    }
}

impl OcpmEventGenerator {
    /// Generate complete MFG process events.
    pub fn generate_mfg_case(
        &mut self,
        documents: &MfgDocuments,
        start_time: DateTime<Utc>,
        available_users: &[String],
    ) -> CaseGenerationResult {
        let case_id = self.new_case_id();
        let variant_type = self.select_variant_type();

        let mut events = Vec::new();
        let mut objects = Vec::new();
        let mut relationships = Vec::new();
        let mut current_time = start_time;

        let order_type = ObjectType::production_order();
        let op_type = ObjectType::routing_operation();
        let inspection_type = ObjectType::quality_inspection();
        let count_type = ObjectType::cycle_count();

        // Create Production Order
        let order_object = self.create_object(
            &order_type,
            &documents.order_id,
            &documents.company_code,
            current_time,
        );
        objects.push(order_object.clone());

        let create_order = ActivityType::create_production_order();
        let resource = self.select_resource(&create_order, available_users);
        let mut event = self.create_event(
            &create_order,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::created(order_object.object_id, &order_type.type_id)
                    .with_external_id(&documents.order_id),
            )
            .with_document_ref(&documents.order_id);
        Self::add_event_attribute(
            &mut event,
            "quantity",
            ObjectAttributeValue::Decimal(documents.quantity),
        );
        Self::add_event_attribute(
            &mut event,
            "material_id",
            ObjectAttributeValue::String(documents.material_id.clone()),
        );
        events.push(event);

        // Release Production Order
        current_time = self.calculate_event_time(current_time, &create_order);
        current_time += self.generate_inter_activity_delay(60, 1440);

        let release = ActivityType::release_production_order();
        let resource = self.select_resource(&release, available_users);
        let mut event = self.create_event(
            &release,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(order_object.object_id, &order_type.type_id)
                .with_external_id(&documents.order_id),
        );
        events.push(event);

        // Skip remaining for error paths (cancelled order)
        if matches!(variant_type, VariantType::ErrorPath) {
            let case_trace = self.create_case_trace(
                case_id,
                &events,
                BusinessProcess::Mfg,
                order_object.object_id,
                &order_type.type_id,
                &documents.company_code,
            );
            return CaseGenerationResult {
                events,
                objects,
                relationships,
                case_trace,
                variant_type,
                correlation_events: Vec::new(),
            };
        }

        // Start/Complete Operations
        for op_id in &documents.operation_ids {
            current_time = self.calculate_event_time(current_time, &release);
            current_time += self.generate_inter_activity_delay(30, 480);

            let op_object =
                self.create_object(&op_type, op_id, &documents.company_code, current_time);
            objects.push(op_object.clone());

            relationships.push(self.create_relationship(
                "belongs_to",
                op_object.object_id,
                &op_type.type_id,
                order_object.object_id,
                &order_type.type_id,
            ));

            // Start Operation
            let start_op = ActivityType::start_operation();
            let resource = self.select_resource(&start_op, available_users);
            let mut event = self.create_event(
                &start_op,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(op_object.object_id, &op_type.type_id)
                        .with_external_id(op_id),
                )
                .with_object(
                    EventObjectRef::updated(order_object.object_id, &order_type.type_id)
                        .with_external_id(&documents.order_id),
                );
            events.push(event);

            // Complete Operation
            current_time = self.calculate_event_time(current_time, &start_op);
            current_time += self.generate_inter_activity_delay(60, 2880); // 1h-2d

            let complete_op = ActivityType::complete_operation();
            let resource = self.select_resource(&complete_op, available_users);
            let mut event = self.create_event(
                &complete_op,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event.with_object(
                EventObjectRef::updated(op_object.object_id, &op_type.type_id)
                    .with_external_id(op_id),
            );
            events.push(event);
        }

        // Quality Inspection
        if let Some(inspection_id) = &documents.inspection_id {
            current_time += self.generate_inter_activity_delay(30, 240);

            let insp_object = self.create_object(
                &inspection_type,
                inspection_id,
                &documents.company_code,
                current_time,
            );
            objects.push(insp_object.clone());

            relationships.push(self.create_relationship(
                "inspects",
                insp_object.object_id,
                &inspection_type.type_id,
                order_object.object_id,
                &order_type.type_id,
            ));

            let create_insp = ActivityType::create_quality_inspection();
            let resource = self.select_resource(&create_insp, available_users);
            let mut event = self.create_event(
                &create_insp,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(insp_object.object_id, &inspection_type.type_id)
                        .with_external_id(inspection_id),
                )
                .with_document_ref(inspection_id);
            events.push(event);

            // Record result
            current_time = self.calculate_event_time(current_time, &create_insp);
            current_time += self.generate_inter_activity_delay(60, 480);

            let record = ActivityType::record_inspection_result();
            let resource = self.select_resource(&record, available_users);
            let mut event = self.create_event(
                &record,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event.with_object(
                EventObjectRef::updated(insp_object.object_id, &inspection_type.type_id)
                    .with_external_id(inspection_id),
            );
            let result_str = if matches!(variant_type, VariantType::ExceptionPath) {
                "conditional"
            } else {
                "accepted"
            };
            Self::add_event_attribute(
                &mut event,
                "inspection_result",
                ObjectAttributeValue::String(result_str.into()),
            );
            events.push(event);
        }

        // Confirm Production
        current_time += self.generate_inter_activity_delay(30, 120);

        let confirm = ActivityType::confirm_production();
        let resource = self.select_resource(&confirm, available_users);
        let mut event = self.create_event(
            &confirm,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(order_object.object_id, &order_type.type_id)
                .with_external_id(&documents.order_id),
        );
        events.push(event);

        // Close Production Order
        current_time = self.calculate_event_time(current_time, &confirm);
        current_time += self.generate_inter_activity_delay(60, 1440);

        let close = ActivityType::close_production_order();
        let resource = self.select_resource(&close, available_users);
        let mut event = self.create_event(
            &close,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::consumed(order_object.object_id, &order_type.type_id)
                .with_external_id(&documents.order_id),
        );
        events.push(event);

        // Cycle Count (independent sub-flow)
        if let Some(count_id) = &documents.cycle_count_id {
            current_time += self.generate_inter_activity_delay(1440, 7200);

            let count_object =
                self.create_object(&count_type, count_id, &documents.company_code, current_time);
            objects.push(count_object.clone());

            let start_count = ActivityType::start_cycle_count();
            let resource = self.select_resource(&start_count, available_users);
            let mut event = self.create_event(
                &start_count,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(count_object.object_id, &count_type.type_id)
                        .with_external_id(count_id),
                )
                .with_document_ref(count_id);
            events.push(event);

            current_time = self.calculate_event_time(current_time, &start_count);
            current_time += self.generate_inter_activity_delay(120, 1440);

            let reconcile = ActivityType::reconcile_cycle_count();
            let resource = self.select_resource(&reconcile, available_users);
            let mut event = self.create_event(
                &reconcile,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event.with_object(
                EventObjectRef::updated(count_object.object_id, &count_type.type_id)
                    .with_external_id(count_id),
            );
            events.push(event);
        }

        let case_trace = self.create_case_trace(
            case_id,
            &events,
            BusinessProcess::Mfg,
            order_object.object_id,
            &order_type.type_id,
            &documents.company_code,
        );

        CaseGenerationResult {
            events,
            objects,
            relationships,
            case_trace,
            variant_type,
            correlation_events: Vec::new(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_mfg_case_generation() {
        let mut generator = OcpmEventGenerator::new(42);
        let factory = OcpmUuidFactory::new(42);
        let documents = MfgDocuments::new(
            "PO-MFG-001",
            "MAT-001",
            "1000",
            Decimal::new(100, 0),
            &factory,
        )
        .with_operations(vec!["OP-010", "OP-020"])
        .with_inspection("QI-001", &factory)
        .with_cycle_count("CC-001", &factory);

        let result = generator.generate_mfg_case(&documents, Utc::now(), &["user001".into()]);

        assert!(result.events.len() >= 4);
        assert!(!result.objects.is_empty());
        assert!(!result.case_trace.activity_sequence.is_empty());
    }

    #[test]
    fn test_mfg_error_path() {
        let mut generator = OcpmEventGenerator::with_config(
            123,
            super::super::OcpmGeneratorConfig {
                error_path_rate: 1.0,
                happy_path_rate: 0.0,
                exception_path_rate: 0.0,
                ..Default::default()
            },
        );

        let factory = OcpmUuidFactory::new(123);
        let documents = MfgDocuments::new(
            "PO-MFG-002",
            "MAT-001",
            "1000",
            Decimal::new(50, 0),
            &factory,
        )
        .with_operations(vec!["OP-010"]);

        let result = generator.generate_mfg_case(&documents, Utc::now(), &[]);

        assert_eq!(result.variant_type, VariantType::ErrorPath);
        assert_eq!(result.events.len(), 2); // create + release only
    }
}
