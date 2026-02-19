//! O2C (Order-to-Cash) process event generator.
//!
//! Generates OCPM events for the complete O2C flow:
//! Create SO -> Check Credit -> Release SO -> Create Delivery -> Pick -> Pack ->
//! Ship -> Create Invoice -> Post Invoice -> Receive Payment

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::{CaseGenerationResult, OcpmEventGenerator, OcpmUuidFactory, VariantType};
use crate::models::{ActivityType, EventObjectRef, ObjectAttributeValue, ObjectType};
use datasynth_core::models::BusinessProcess;

/// O2C document references for event generation.
#[derive(Debug, Clone)]
pub struct O2cDocuments {
    /// Sales order number
    pub so_number: String,
    /// Sales order UUID
    pub so_id: Uuid,
    /// Delivery number
    pub delivery_number: Option<String>,
    /// Delivery UUID
    pub delivery_id: Option<Uuid>,
    /// Customer invoice number
    pub invoice_number: Option<String>,
    /// Invoice UUID
    pub invoice_id: Option<Uuid>,
    /// Receipt number
    pub receipt_number: Option<String>,
    /// Receipt UUID
    pub receipt_id: Option<Uuid>,
    /// Customer ID
    pub customer_id: String,
    /// Company code
    pub company_code: String,
    /// Total amount
    pub amount: Decimal,
    /// Currency
    pub currency: String,
    /// Country code (ISO 3166-1 alpha-2) of the company.
    pub country_code: Option<String>,
}

impl O2cDocuments {
    /// Create new O2C documents with deterministic UUIDs from the given factory.
    pub fn new(
        so_number: &str,
        customer_id: &str,
        company_code: &str,
        amount: Decimal,
        currency: &str,
        factory: &OcpmUuidFactory,
    ) -> Self {
        Self {
            so_number: so_number.into(),
            so_id: factory.next_document_id(),
            delivery_number: None,
            delivery_id: None,
            invoice_number: None,
            invoice_id: None,
            receipt_number: None,
            receipt_id: None,
            customer_id: customer_id.into(),
            company_code: company_code.into(),
            amount,
            currency: currency.into(),
            country_code: None,
        }
    }

    /// Set country code for the company.
    pub fn with_country_code(mut self, country_code: &str) -> Self {
        self.country_code = Some(country_code.into());
        self
    }

    /// Set delivery info with deterministic UUID from the given factory.
    pub fn with_delivery(mut self, delivery_number: &str, factory: &OcpmUuidFactory) -> Self {
        self.delivery_number = Some(delivery_number.into());
        self.delivery_id = Some(factory.next_document_id());
        self
    }

    /// Set invoice info with deterministic UUID from the given factory.
    pub fn with_invoice(mut self, invoice_number: &str, factory: &OcpmUuidFactory) -> Self {
        self.invoice_number = Some(invoice_number.into());
        self.invoice_id = Some(factory.next_document_id());
        self
    }

    /// Set receipt info with deterministic UUID from the given factory.
    pub fn with_receipt(mut self, receipt_number: &str, factory: &OcpmUuidFactory) -> Self {
        self.receipt_number = Some(receipt_number.into());
        self.receipt_id = Some(factory.next_document_id());
        self
    }
}

impl OcpmEventGenerator {
    /// Generate complete O2C process events.
    pub fn generate_o2c_case(
        &mut self,
        documents: &O2cDocuments,
        start_time: DateTime<Utc>,
        available_users: &[String],
    ) -> CaseGenerationResult {
        let case_id = self.new_case_id();
        let variant_type = self.select_variant_type();

        let mut events = Vec::new();
        let mut objects = Vec::new();
        let mut relationships = Vec::new();
        let mut current_time = start_time;

        // Create object types
        let so_type = ObjectType::sales_order();
        let delivery_type = ObjectType::delivery();
        let invoice_type = ObjectType::customer_invoice();

        // Create SO object
        let so_object = self.create_object(
            &so_type,
            &documents.so_number,
            &documents.company_code,
            current_time,
        );
        objects.push(so_object.clone());

        // Activity: Create SO
        let create_so = ActivityType::create_so();
        let resource = self.select_resource(&create_so, available_users);
        let mut event = self.create_event(
            &create_so,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::created(so_object.object_id, &so_type.type_id)
                    .with_external_id(&documents.so_number),
            )
            .with_document_ref(&documents.so_number);
        Self::add_event_attribute(
            &mut event,
            "amount",
            ObjectAttributeValue::Decimal(documents.amount),
        );
        Self::add_event_attribute(
            &mut event,
            "customer_id",
            ObjectAttributeValue::String(documents.customer_id.clone()),
        );
        if let Some(ref cc) = documents.country_code {
            Self::add_event_attribute(
                &mut event,
                "country_code",
                ObjectAttributeValue::String(cc.clone()),
            );
        }
        events.push(event);

        // Activity: Check Credit
        current_time = self.calculate_event_time(current_time, &create_so);

        let check_credit = ActivityType::check_credit();
        let resource = self.select_resource(&check_credit, available_users);
        let mut event = self.create_event(
            &check_credit,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(so_object.object_id, &so_type.type_id)
                .with_external_id(&documents.so_number),
        );
        Self::add_event_attribute(
            &mut event,
            "credit_result",
            ObjectAttributeValue::String("approved".into()),
        );
        events.push(event);

        // Activity: Release SO
        current_time = self.calculate_event_time(current_time, &check_credit);

        let release_so = ActivityType::release_so();
        let resource = self.select_resource(&release_so, available_users);
        let mut event = self.create_event(
            &release_so,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(so_object.object_id, &so_type.type_id)
                .with_external_id(&documents.so_number),
        );
        events.push(event);

        // Skip remaining steps for error paths
        if matches!(variant_type, VariantType::ErrorPath) {
            let case_trace = self.create_case_trace(
                case_id,
                &events,
                BusinessProcess::O2C,
                so_object.object_id,
                &so_type.type_id,
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

        // Activity: Create Delivery
        if let Some(delivery_number) = &documents.delivery_number {
            current_time = self.calculate_event_time(current_time, &release_so);
            current_time += self.generate_inter_activity_delay(60, 480);

            let delivery_object = self.create_object(
                &delivery_type,
                delivery_number,
                &documents.company_code,
                current_time,
            );
            objects.push(delivery_object.clone());

            // Add relationship: Delivery references SO
            relationships.push(self.create_relationship(
                "references",
                delivery_object.object_id,
                &delivery_type.type_id,
                so_object.object_id,
                &so_type.type_id,
            ));

            let create_delivery = ActivityType::create_delivery();
            let resource = self.select_resource(&create_delivery, available_users);
            let mut event = self.create_event(
                &create_delivery,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(delivery_object.object_id, &delivery_type.type_id)
                        .with_external_id(delivery_number),
                )
                .with_document_ref(delivery_number);
            events.push(event);

            // Activity: Pick
            current_time = self.calculate_event_time(current_time, &create_delivery);

            let pick = ActivityType::pick();
            let resource = self.select_resource(&pick, available_users);
            let mut event = self.create_event(
                &pick,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event.with_object(
                EventObjectRef::updated(delivery_object.object_id, &delivery_type.type_id)
                    .with_external_id(delivery_number),
            );
            events.push(event);

            // Activity: Pack
            current_time = self.calculate_event_time(current_time, &pick);

            let pack = ActivityType::pack();
            let resource = self.select_resource(&pack, available_users);
            let mut event = self.create_event(
                &pack,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event.with_object(
                EventObjectRef::updated(delivery_object.object_id, &delivery_type.type_id)
                    .with_external_id(delivery_number),
            );
            events.push(event);

            // Activity: Ship
            current_time = self.calculate_event_time(current_time, &pack);

            let ship = ActivityType::ship();
            let resource = self.select_resource(&ship, available_users);
            let mut event = self.create_event(
                &ship,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::updated(delivery_object.object_id, &delivery_type.type_id)
                        .with_external_id(delivery_number),
                )
                .with_object(
                    EventObjectRef::updated(so_object.object_id, &so_type.type_id)
                        .with_external_id(&documents.so_number),
                );
            events.push(event);
        }

        // Activity: Create Customer Invoice
        if let Some(invoice_number) = &documents.invoice_number {
            current_time = self.calculate_event_time(current_time, &ActivityType::ship());
            current_time += self.generate_inter_activity_delay(60, 1440);

            let invoice_object = self.create_object(
                &invoice_type,
                invoice_number,
                &documents.company_code,
                current_time,
            );
            objects.push(invoice_object.clone());

            // Add relationship: Invoice references SO
            relationships.push(self.create_relationship(
                "references",
                invoice_object.object_id,
                &invoice_type.type_id,
                so_object.object_id,
                &so_type.type_id,
            ));

            let create_invoice = ActivityType::create_customer_invoice();
            let resource = self.select_resource(&create_invoice, available_users);
            let mut event = self.create_event(
                &create_invoice,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(invoice_object.object_id, &invoice_type.type_id)
                        .with_external_id(invoice_number),
                )
                .with_object(
                    EventObjectRef::updated(so_object.object_id, &so_type.type_id)
                        .with_external_id(&documents.so_number),
                )
                .with_document_ref(invoice_number);
            Self::add_event_attribute(
                &mut event,
                "invoice_amount",
                ObjectAttributeValue::Decimal(documents.amount),
            );
            events.push(event);

            // Activity: Post Customer Invoice
            current_time = self.calculate_event_time(current_time, &create_invoice);

            let post_invoice = ActivityType::post_customer_invoice();
            let resource = self.select_resource(&post_invoice, available_users);
            let mut event = self.create_event(
                &post_invoice,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event.with_object(
                EventObjectRef::updated(invoice_object.object_id, &invoice_type.type_id)
                    .with_external_id(invoice_number),
            );
            events.push(event);

            // Activity: Receive Payment
            if documents.receipt_number.is_some() {
                current_time = self.calculate_event_time(current_time, &post_invoice);
                current_time += self.generate_inter_activity_delay(1440, 43200); // 1-30 days

                let receive_payment = ActivityType::receive_payment();
                let resource = self.select_resource(&receive_payment, available_users);
                let mut event = self.create_event(
                    &receive_payment,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event
                    .with_object(
                        EventObjectRef::consumed(invoice_object.object_id, &invoice_type.type_id)
                            .with_external_id(invoice_number),
                    )
                    .with_object(
                        EventObjectRef::consumed(so_object.object_id, &so_type.type_id)
                            .with_external_id(&documents.so_number),
                    )
                    .with_document_ref(documents.receipt_number.as_deref().unwrap_or(""));
                Self::add_event_attribute(
                    &mut event,
                    "payment_amount",
                    ObjectAttributeValue::Decimal(documents.amount),
                );
                events.push(event);
            }
        }

        let case_trace = self.create_case_trace(
            case_id,
            &events,
            BusinessProcess::O2C,
            so_object.object_id,
            &so_type.type_id,
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
    fn test_o2c_case_generation() {
        let mut generator = OcpmEventGenerator::new(42);
        let factory = OcpmUuidFactory::new(42);
        let documents = O2cDocuments::new(
            "SO-000001",
            "C000001",
            "1000",
            Decimal::new(15000, 0),
            "USD",
            &factory,
        )
        .with_delivery("DEL-000001", &factory)
        .with_invoice("INV-000001", &factory)
        .with_receipt("REC-000001", &factory);

        let result = generator.generate_o2c_case(
            &documents,
            Utc::now(),
            &["user001".into(), "user002".into()],
        );

        // Should have at least 3 events (create, check_credit, release)
        assert!(result.events.len() >= 3);
        // Should have objects
        assert!(!result.objects.is_empty());
        // Should have case trace
        assert!(!result.case_trace.activity_sequence.is_empty());
    }
}
