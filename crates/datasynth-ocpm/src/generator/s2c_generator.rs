//! S2C (Source-to-Contract) process event generator.
//!
//! Generates OCPM events for the complete S2C flow:
//! Create Project → Qualify Suppliers → Publish RFx → Submit Bids →
//! Evaluate Bids → Award Contract → Activate Contract → Complete Sourcing

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::{CaseGenerationResult, OcpmEventGenerator, VariantType};
use crate::models::{
    ActivityType, EventObjectRef, ObjectAttributeValue, ObjectRelationship, ObjectType,
};
use datasynth_core::models::BusinessProcess;

/// S2C document references for event generation.
#[derive(Debug, Clone)]
pub struct S2cDocuments {
    /// Sourcing project ID
    pub project_id: String,
    /// Sourcing project UUID
    pub project_uuid: Uuid,
    /// RFx event ID
    pub rfx_id: Option<String>,
    /// RFx UUID
    pub rfx_uuid: Option<Uuid>,
    /// Winning bid ID
    pub winning_bid_id: Option<String>,
    /// Winning bid UUID
    pub winning_bid_uuid: Option<Uuid>,
    /// Contract ID
    pub contract_id: Option<String>,
    /// Contract UUID
    pub contract_uuid: Option<Uuid>,
    /// Vendor ID
    pub vendor_id: String,
    /// Company code
    pub company_code: String,
    /// Estimated amount
    pub amount: Decimal,
}

impl S2cDocuments {
    /// Create new S2C documents.
    pub fn new(
        project_id: &str,
        vendor_id: &str,
        company_code: &str,
        amount: Decimal,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            project_uuid: Uuid::new_v4(),
            rfx_id: None,
            rfx_uuid: None,
            winning_bid_id: None,
            winning_bid_uuid: None,
            contract_id: None,
            contract_uuid: None,
            vendor_id: vendor_id.into(),
            company_code: company_code.into(),
            amount,
        }
    }

    /// Set RFx info.
    pub fn with_rfx(mut self, rfx_id: &str) -> Self {
        self.rfx_id = Some(rfx_id.into());
        self.rfx_uuid = Some(Uuid::new_v4());
        self
    }

    /// Set winning bid info.
    pub fn with_winning_bid(mut self, bid_id: &str) -> Self {
        self.winning_bid_id = Some(bid_id.into());
        self.winning_bid_uuid = Some(Uuid::new_v4());
        self
    }

    /// Set contract info.
    pub fn with_contract(mut self, contract_id: &str) -> Self {
        self.contract_id = Some(contract_id.into());
        self.contract_uuid = Some(Uuid::new_v4());
        self
    }
}

impl OcpmEventGenerator {
    /// Generate complete S2C process events.
    pub fn generate_s2c_case(
        &mut self,
        documents: &S2cDocuments,
        start_time: DateTime<Utc>,
        available_users: &[String],
    ) -> CaseGenerationResult {
        let case_id = self.new_case_id();
        let variant_type = self.select_variant_type();

        let mut events = Vec::new();
        let mut objects = Vec::new();
        let mut relationships = Vec::new();
        let mut current_time = start_time;

        let project_type = ObjectType::sourcing_project();
        let rfx_type = ObjectType::rfx_event();
        let bid_type = ObjectType::supplier_bid();
        let contract_type = ObjectType::procurement_contract();

        // Create project object
        let project_object = self.create_object(
            &project_type,
            &documents.project_id,
            &documents.company_code,
            current_time,
        );
        objects.push(project_object.clone());

        // Activity: Create Sourcing Project
        let create_project = ActivityType::create_sourcing_project();
        let resource = self.select_resource(&create_project, available_users);
        let mut event = self.create_event(
            &create_project,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::created(project_object.object_id, &project_type.type_id)
                    .with_external_id(&documents.project_id),
            )
            .with_document_ref(&documents.project_id);
        Self::add_event_attribute(
            &mut event,
            "estimated_value",
            ObjectAttributeValue::Decimal(documents.amount),
        );
        events.push(event);

        // Activity: Qualify Supplier
        current_time = self.calculate_event_time(current_time, &create_project);
        current_time += self.generate_inter_activity_delay(1440, 10080); // 1-7 days

        let qualify = ActivityType::qualify_supplier();
        let resource = self.select_resource(&qualify, available_users);
        let mut event = self.create_event(
            &qualify,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_document_ref(&documents.vendor_id);
        Self::add_event_attribute(
            &mut event,
            "vendor_id",
            ObjectAttributeValue::String(documents.vendor_id.clone()),
        );
        events.push(event);

        // Skip remaining steps for error paths (cancelled sourcing)
        if matches!(variant_type, VariantType::ErrorPath) {
            let case_trace = self.create_case_trace(
                case_id,
                &events,
                BusinessProcess::S2C,
                project_object.object_id,
                &project_type.type_id,
                &documents.company_code,
            );
            return CaseGenerationResult {
                events,
                objects,
                relationships,
                case_trace,
                variant_type,
            };
        }

        // Activity: Publish RFx
        if let Some(rfx_id) = &documents.rfx_id {
            current_time = self.calculate_event_time(current_time, &qualify);
            current_time += self.generate_inter_activity_delay(1440, 7200); // 1-5 days

            let rfx_object = self.create_object(
                &rfx_type,
                rfx_id,
                &documents.company_code,
                current_time,
            );
            objects.push(rfx_object.clone());

            relationships.push(ObjectRelationship::new(
                "belongs_to",
                rfx_object.object_id,
                &rfx_type.type_id,
                project_object.object_id,
                &project_type.type_id,
            ));

            let publish = ActivityType::publish_rfx();
            let resource = self.select_resource(&publish, available_users);
            let mut event = self.create_event(
                &publish,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(rfx_object.object_id, &rfx_type.type_id)
                        .with_external_id(rfx_id),
                )
                .with_object(
                    EventObjectRef::updated(project_object.object_id, &project_type.type_id)
                        .with_external_id(&documents.project_id),
                )
                .with_document_ref(rfx_id);
            events.push(event);

            // Activity: Submit Bid
            if let Some(bid_id) = &documents.winning_bid_id {
                current_time = self.calculate_event_time(current_time, &publish);
                current_time += self.generate_inter_activity_delay(7200, 20160); // 5-14 days

                let bid_object = self.create_object(
                    &bid_type,
                    bid_id,
                    &documents.company_code,
                    current_time,
                );
                objects.push(bid_object.clone());

                relationships.push(ObjectRelationship::new(
                    "responds_to",
                    bid_object.object_id,
                    &bid_type.type_id,
                    rfx_object.object_id,
                    &rfx_type.type_id,
                ));

                let submit = ActivityType::submit_bid();
                let resource = self.select_resource(&submit, available_users);
                let mut event = self.create_event(
                    &submit,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event
                    .with_object(
                        EventObjectRef::created(bid_object.object_id, &bid_type.type_id)
                            .with_external_id(bid_id),
                    )
                    .with_document_ref(bid_id);
                Self::add_event_attribute(
                    &mut event,
                    "bid_amount",
                    ObjectAttributeValue::Decimal(documents.amount),
                );
                events.push(event);

                // Activity: Evaluate Bids
                current_time = self.calculate_event_time(current_time, &submit);
                current_time += self.generate_inter_activity_delay(1440, 7200);

                let evaluate = ActivityType::evaluate_bids();
                let resource = self.select_resource(&evaluate, available_users);
                let mut event = self.create_event(
                    &evaluate,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event
                    .with_object(
                        EventObjectRef::read(rfx_object.object_id, &rfx_type.type_id)
                            .with_external_id(rfx_id),
                    )
                    .with_object(
                        EventObjectRef::read(bid_object.object_id, &bid_type.type_id)
                            .with_external_id(bid_id),
                    );
                events.push(event);
            }

            // Activity: Award Contract
            if let Some(contract_id) = &documents.contract_id {
                current_time = self.calculate_event_time(current_time, &ActivityType::evaluate_bids());
                current_time += self.generate_inter_activity_delay(1440, 4320);

                let contract_object = self.create_object(
                    &contract_type,
                    contract_id,
                    &documents.company_code,
                    current_time,
                );
                objects.push(contract_object.clone());

                relationships.push(ObjectRelationship::new(
                    "awarded_from",
                    contract_object.object_id,
                    &contract_type.type_id,
                    project_object.object_id,
                    &project_type.type_id,
                ));

                let award = ActivityType::award_contract();
                let resource = self.select_resource(&award, available_users);
                let mut event = self.create_event(
                    &award,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event
                    .with_object(
                        EventObjectRef::created(contract_object.object_id, &contract_type.type_id)
                            .with_external_id(contract_id),
                    )
                    .with_object(
                        EventObjectRef::updated(project_object.object_id, &project_type.type_id)
                            .with_external_id(&documents.project_id),
                    )
                    .with_document_ref(contract_id);
                Self::add_event_attribute(
                    &mut event,
                    "contract_value",
                    ObjectAttributeValue::Decimal(documents.amount),
                );
                events.push(event);

                // Activity: Activate Contract
                current_time = self.calculate_event_time(current_time, &award);
                current_time += self.generate_inter_activity_delay(60, 1440);

                let activate = ActivityType::activate_contract();
                let resource = self.select_resource(&activate, available_users);
                let mut event = self.create_event(
                    &activate,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event.with_object(
                    EventObjectRef::updated(contract_object.object_id, &contract_type.type_id)
                        .with_external_id(contract_id),
                );
                events.push(event);

                // Activity: Complete Sourcing
                current_time = self.calculate_event_time(current_time, &activate);

                let complete = ActivityType::complete_sourcing();
                let resource = self.select_resource(&complete, available_users);
                let mut event = self.create_event(
                    &complete,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event.with_object(
                    EventObjectRef::consumed(project_object.object_id, &project_type.type_id)
                        .with_external_id(&documents.project_id),
                );
                events.push(event);
            }
        }

        let case_trace = self.create_case_trace(
            case_id,
            &events,
            BusinessProcess::S2C,
            project_object.object_id,
            &project_type.type_id,
            &documents.company_code,
        );

        CaseGenerationResult {
            events,
            objects,
            relationships,
            case_trace,
            variant_type,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_s2c_case_generation() {
        let mut generator = OcpmEventGenerator::new(42);
        let documents = S2cDocuments::new("SP-000001", "V000001", "1000", Decimal::new(50000, 0))
            .with_rfx("RFX-000001")
            .with_winning_bid("BID-000001")
            .with_contract("CTR-000001");

        let result = generator.generate_s2c_case(
            &documents,
            Utc::now(),
            &["user001".into(), "user002".into()],
        );

        assert!(result.events.len() >= 2);
        assert!(!result.objects.is_empty());
        assert!(!result.case_trace.activity_sequence.is_empty());
    }

    #[test]
    fn test_s2c_error_path() {
        let mut generator = OcpmEventGenerator::with_config(
            123,
            super::super::OcpmGeneratorConfig {
                error_path_rate: 1.0,
                happy_path_rate: 0.0,
                exception_path_rate: 0.0,
                ..Default::default()
            },
        );

        let documents = S2cDocuments::new("SP-000002", "V000001", "1000", Decimal::new(25000, 0))
            .with_rfx("RFX-000002")
            .with_contract("CTR-000002");

        let result = generator.generate_s2c_case(&documents, Utc::now(), &[]);

        assert_eq!(result.variant_type, VariantType::ErrorPath);
        assert_eq!(result.events.len(), 2); // create project + qualify only
    }
}
