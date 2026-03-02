//! P2P (Procure-to-Pay) process event generator.
//!
//! Generates OCPM events for the complete P2P flow:
//! Create PO -> Approve PO -> Release PO -> Create GR -> Post GR ->
//! Receive Invoice -> Verify Invoice -> Post Invoice -> Execute Payment

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::{CaseGenerationResult, OcpmEventGenerator, OcpmUuidFactory, VariantType};
use crate::models::{
    ActivityType, CorrelationEvent, EventObjectRef, ObjectAttributeValue, ObjectType,
};
use datasynth_core::models::BusinessProcess;

/// P2P document references for event generation.
#[derive(Debug, Clone)]
pub struct P2pDocuments {
    /// Purchase order number
    pub po_number: String,
    /// Purchase order UUID
    pub po_id: Uuid,
    /// Goods receipt number
    pub gr_number: Option<String>,
    /// Goods receipt UUID
    pub gr_id: Option<Uuid>,
    /// Vendor invoice number
    pub invoice_number: Option<String>,
    /// Invoice UUID
    pub invoice_id: Option<Uuid>,
    /// Payment number
    pub payment_number: Option<String>,
    /// Payment UUID
    pub payment_id: Option<Uuid>,
    /// Vendor ID
    pub vendor_id: String,
    /// Company code
    pub company_code: String,
    /// Total amount
    pub amount: Decimal,
    /// Currency
    pub currency: String,
    /// Country code (ISO 3166-1 alpha-2) of the company.
    pub country_code: Option<String>,
}

impl P2pDocuments {
    /// Create new P2P documents with deterministic UUIDs from the given factory.
    pub fn new(
        po_number: &str,
        vendor_id: &str,
        company_code: &str,
        amount: Decimal,
        currency: &str,
        factory: &OcpmUuidFactory,
    ) -> Self {
        Self {
            po_number: po_number.into(),
            po_id: factory.next_document_id(),
            gr_number: None,
            gr_id: None,
            invoice_number: None,
            invoice_id: None,
            payment_number: None,
            payment_id: None,
            vendor_id: vendor_id.into(),
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

    /// Set goods receipt info with deterministic UUID from the given factory.
    pub fn with_goods_receipt(mut self, gr_number: &str, factory: &OcpmUuidFactory) -> Self {
        self.gr_number = Some(gr_number.into());
        self.gr_id = Some(factory.next_document_id());
        self
    }

    /// Set invoice info with deterministic UUID from the given factory.
    pub fn with_invoice(mut self, invoice_number: &str, factory: &OcpmUuidFactory) -> Self {
        self.invoice_number = Some(invoice_number.into());
        self.invoice_id = Some(factory.next_document_id());
        self
    }

    /// Set payment info with deterministic UUID from the given factory.
    pub fn with_payment(mut self, payment_number: &str, factory: &OcpmUuidFactory) -> Self {
        self.payment_number = Some(payment_number.into());
        self.payment_id = Some(factory.next_document_id());
        self
    }
}

impl OcpmEventGenerator {
    /// Generate complete P2P process events with OCEL 2.0 enrichment.
    ///
    /// Events are enriched with state transitions (from `ActivityType` transitions),
    /// resource workload (from resource pools), and correlation events (three-way
    /// match at verify, payment allocation at execute_payment).
    pub fn generate_p2p_case(
        &mut self,
        documents: &P2pDocuments,
        start_time: DateTime<Utc>,
        available_users: &[String],
    ) -> CaseGenerationResult {
        let case_id = self.new_case_id();
        let variant_type = self.select_variant_type();

        let mut events = Vec::new();
        let mut objects = Vec::new();
        let mut relationships = Vec::new();
        let mut correlation_events = Vec::new();
        let mut current_time = start_time;

        // Create object types
        let po_type = ObjectType::purchase_order();
        let gr_type = ObjectType::goods_receipt();
        let invoice_type = ObjectType::vendor_invoice();

        // Create PO object
        let po_object = self.create_object(
            &po_type,
            &documents.po_number,
            &documents.company_code,
            current_time,
        );
        objects.push(po_object.clone());

        // Activity: Create PO
        let create_po = ActivityType::create_po();
        let resource = self.select_resource(&create_po, available_users);
        let mut event = self.create_event(
            &create_po,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::created(po_object.object_id, &po_type.type_id)
                    .with_external_id(&documents.po_number),
            )
            .with_document_ref(&documents.po_number);
        Self::add_event_attribute(
            &mut event,
            "amount",
            ObjectAttributeValue::Decimal(documents.amount),
        );
        Self::add_event_attribute(
            &mut event,
            "vendor_id",
            ObjectAttributeValue::String(documents.vendor_id.clone()),
        );
        if let Some(ref cc) = documents.country_code {
            Self::add_event_attribute(
                &mut event,
                "country_code",
                ObjectAttributeValue::String(cc.clone()),
            );
        }
        event = self.enrich_event(event, &create_po, "pool-approver");
        events.push(event);

        // Activity: Approve PO
        current_time = self.calculate_event_time(current_time, &create_po);
        current_time += self.generate_inter_activity_delay(30, 480); // 30 min to 8 hours

        let approve_po = ActivityType::approve_po();
        let resource = self.select_resource(&approve_po, available_users);
        let mut event = self.create_event(
            &approve_po,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::updated(po_object.object_id, &po_type.type_id)
                    .with_external_id(&documents.po_number),
            )
            .with_document_ref(&documents.po_number);
        event = self.enrich_event(event, &approve_po, "pool-approver");
        events.push(event);

        // Activity: Release PO
        current_time = self.calculate_event_time(current_time, &approve_po);

        let release_po = ActivityType::release_po();
        let resource = self.select_resource(&release_po, available_users);
        let mut event = self.create_event(
            &release_po,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::updated(po_object.object_id, &po_type.type_id)
                    .with_external_id(&documents.po_number),
            )
            .with_document_ref(&documents.po_number);
        event = self.enrich_event(event, &release_po, "pool-approver");
        events.push(event);

        // Skip remaining steps for error paths
        if matches!(variant_type, VariantType::ErrorPath) {
            let case_trace = self.create_case_trace(
                case_id,
                &events,
                BusinessProcess::P2P,
                po_object.object_id,
                &po_type.type_id,
                &documents.company_code,
            );
            return CaseGenerationResult {
                events,
                objects,
                relationships,
                case_trace,
                variant_type,
                correlation_events,
            };
        }

        // Activity: Create GR
        current_time = self.calculate_event_time(current_time, &release_po);
        current_time += self.generate_inter_activity_delay(1440, 10080); // 1-7 days

        if let Some(gr_number) = &documents.gr_number {
            let gr_object =
                self.create_object(&gr_type, gr_number, &documents.company_code, current_time);
            objects.push(gr_object.clone());

            // Add relationship: GR references PO
            relationships.push(self.create_relationship(
                "references",
                gr_object.object_id,
                &gr_type.type_id,
                po_object.object_id,
                &po_type.type_id,
            ));

            let create_gr = ActivityType::create_gr();
            let resource = self.select_resource(&create_gr, available_users);
            let mut event = self.create_event(
                &create_gr,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(gr_object.object_id, &gr_type.type_id)
                        .with_external_id(gr_number),
                )
                .with_object(
                    EventObjectRef::updated(po_object.object_id, &po_type.type_id)
                        .with_external_id(&documents.po_number),
                )
                .with_document_ref(gr_number);
            event = self.enrich_event(event, &create_gr, "pool-warehouse");
            events.push(event);

            // Activity: Post GR
            current_time = self.calculate_event_time(current_time, &create_gr);

            let post_gr = ActivityType::post_gr();
            let resource = self.select_resource(&post_gr, available_users);
            let mut event = self.create_event(
                &post_gr,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::updated(gr_object.object_id, &gr_type.type_id)
                        .with_external_id(gr_number),
                )
                .with_document_ref(gr_number);
            event = self.enrich_event(event, &post_gr, "pool-warehouse");
            events.push(event);
        }

        // Activity: Receive Invoice
        current_time = self.calculate_event_time(current_time, &ActivityType::post_gr());
        current_time += self.generate_inter_activity_delay(1440, 20160); // 1-14 days

        if let Some(invoice_number) = &documents.invoice_number {
            let invoice_object = self.create_object(
                &invoice_type,
                invoice_number,
                &documents.company_code,
                current_time,
            );
            objects.push(invoice_object.clone());

            // Add relationship: Invoice references PO
            relationships.push(self.create_relationship(
                "references",
                invoice_object.object_id,
                &invoice_type.type_id,
                po_object.object_id,
                &po_type.type_id,
            ));

            let receive_invoice = ActivityType::receive_invoice();
            let resource = self.select_resource(&receive_invoice, available_users);
            let mut event = self.create_event(
                &receive_invoice,
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
                .with_document_ref(invoice_number);
            Self::add_event_attribute(
                &mut event,
                "invoice_amount",
                ObjectAttributeValue::Decimal(documents.amount),
            );
            event = self.enrich_event(event, &receive_invoice, "pool-ap");
            events.push(event);

            // Skip verify for exception paths sometimes
            if !matches!(variant_type, VariantType::ExceptionPath)
                || !self.should_skip_activity(0.3)
            {
                // Activity: Verify Invoice (3-way match)
                current_time = self.calculate_event_time(current_time, &receive_invoice);
                current_time += self.generate_inter_activity_delay(60, 480);

                let verify_invoice = ActivityType::verify_invoice();
                let resource = self.select_resource(&verify_invoice, available_users);
                let mut event = self.create_event(
                    &verify_invoice,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event
                    .with_object(
                        EventObjectRef::updated(invoice_object.object_id, &invoice_type.type_id)
                            .with_external_id(invoice_number),
                    )
                    .with_object(
                        EventObjectRef::read(po_object.object_id, &po_type.type_id)
                            .with_external_id(&documents.po_number),
                    );

                if let Some(gr_id) = documents.gr_id {
                    event = event.with_object(EventObjectRef::read(gr_id, &gr_type.type_id));

                    // Create three-way match correlation event
                    let corr = CorrelationEvent::three_way_match(
                        documents.po_id,
                        gr_id,
                        invoice_object.object_id,
                        current_time,
                        &resource,
                        &documents.company_code,
                    );
                    event = event.with_correlation_id(&corr.correlation_id);
                    correlation_events.push(corr);
                }

                Self::add_event_attribute(
                    &mut event,
                    "match_result",
                    ObjectAttributeValue::String("matched".into()),
                );
                event = self.enrich_event(event, &verify_invoice, "pool-ap");
                events.push(event);
            }

            // Activity: Post Invoice
            current_time = self.calculate_event_time(current_time, &ActivityType::verify_invoice());

            let post_invoice = ActivityType::post_invoice();
            let resource = self.select_resource(&post_invoice, available_users);
            let mut event = self.create_event(
                &post_invoice,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::updated(invoice_object.object_id, &invoice_type.type_id)
                        .with_external_id(invoice_number),
                )
                .with_object(
                    EventObjectRef::updated(po_object.object_id, &po_type.type_id)
                        .with_external_id(&documents.po_number),
                )
                .with_document_ref(invoice_number);
            event = self.enrich_event(event, &post_invoice, "pool-ap");
            events.push(event);

            // Activity: Execute Payment
            if documents.payment_number.is_some() {
                current_time = self.calculate_event_time(current_time, &post_invoice);
                current_time += self.generate_inter_activity_delay(1440, 43200); // 1-30 days (payment terms)

                let execute_payment = ActivityType::execute_payment();
                let resource = self.select_resource(&execute_payment, available_users);
                let mut event = self.create_event(
                    &execute_payment,
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
                        EventObjectRef::consumed(po_object.object_id, &po_type.type_id)
                            .with_external_id(&documents.po_number),
                    )
                    .with_document_ref(documents.payment_number.as_deref().unwrap_or(""));
                Self::add_event_attribute(
                    &mut event,
                    "payment_amount",
                    ObjectAttributeValue::Decimal(documents.amount),
                );

                // Create payment allocation correlation event
                if let Some(payment_id) = documents.payment_id {
                    let corr = CorrelationEvent::payment_allocation(
                        payment_id,
                        &[invoice_object.object_id],
                        current_time,
                        &resource,
                        &documents.company_code,
                    );
                    event = event.with_correlation_id(&corr.correlation_id);
                    correlation_events.push(corr);
                }

                event = self.enrich_event(event, &execute_payment, "pool-ap");
                events.push(event);
            }
        }

        let case_trace = self.create_case_trace(
            case_id,
            &events,
            BusinessProcess::P2P,
            po_object.object_id,
            &po_type.type_id,
            &documents.company_code,
        );

        CaseGenerationResult {
            events,
            objects,
            relationships,
            case_trace,
            variant_type,
            correlation_events,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_case_generation() {
        let mut generator = OcpmEventGenerator::new(42);
        let factory = OcpmUuidFactory::new(42);
        let documents = P2pDocuments::new(
            "PO-000001",
            "V000001",
            "1000",
            Decimal::new(10000, 0),
            "USD",
            &factory,
        )
        .with_goods_receipt("GR-000001", &factory)
        .with_invoice("INV-000001", &factory)
        .with_payment("PAY-000001", &factory);

        let result = generator.generate_p2p_case(
            &documents,
            Utc::now(),
            &["user001".into(), "user002".into()],
        );

        // Should have at least 3 events (create, approve, release)
        assert!(result.events.len() >= 3);
        // Should have objects
        assert!(!result.objects.is_empty());
        // Should have case trace
        assert!(!result.case_trace.activity_sequence.is_empty());
    }

    #[test]
    fn test_p2p_error_path() {
        // Use a seed that produces error paths more often
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
        let documents = P2pDocuments::new(
            "PO-000002",
            "V000001",
            "1000",
            Decimal::new(5000, 0),
            "USD",
            &factory,
        )
        .with_goods_receipt("GR-000002", &factory)
        .with_invoice("INV-000002", &factory);

        let result = generator.generate_p2p_case(&documents, Utc::now(), &[]);

        // Error path should stop early (only create, approve, release)
        assert_eq!(result.variant_type, VariantType::ErrorPath);
        assert_eq!(result.events.len(), 3);
    }

    #[test]
    fn test_p2p_events_have_state_transitions() {
        let mut generator = OcpmEventGenerator::with_config(
            42,
            super::super::OcpmGeneratorConfig {
                happy_path_rate: 1.0,
                exception_path_rate: 0.0,
                error_path_rate: 0.0,
                ..Default::default()
            },
        );

        let factory = OcpmUuidFactory::new(42);
        let documents = P2pDocuments::new(
            "PO-000010",
            "V000001",
            "1000",
            Decimal::new(10000, 0),
            "USD",
            &factory,
        )
        .with_goods_receipt("GR-000010", &factory)
        .with_invoice("INV-000010", &factory)
        .with_payment("PAY-000010", &factory);

        let result = generator.generate_p2p_case(
            &documents,
            Utc::now(),
            &["user001".into(), "user002".into()],
        );

        // All events should have state transitions (from_state populated)
        let events_with_state: Vec<_> = result
            .events
            .iter()
            .filter(|e| e.from_state.is_some())
            .collect();
        assert!(
            !events_with_state.is_empty(),
            "At least some events should have state transitions"
        );

        // First event (create PO) should transition to "created"
        let first = &result.events[0];
        assert_eq!(first.to_state.as_deref(), Some("created"));
    }

    #[test]
    fn test_p2p_events_have_resource_workload() {
        let mut generator = OcpmEventGenerator::with_config(
            42,
            super::super::OcpmGeneratorConfig {
                happy_path_rate: 1.0,
                exception_path_rate: 0.0,
                error_path_rate: 0.0,
                ..Default::default()
            },
        );

        let factory = OcpmUuidFactory::new(42);
        let documents = P2pDocuments::new(
            "PO-000020",
            "V000001",
            "1000",
            Decimal::new(10000, 0),
            "USD",
            &factory,
        )
        .with_goods_receipt("GR-000020", &factory)
        .with_invoice("INV-000020", &factory)
        .with_payment("PAY-000020", &factory);

        let result = generator.generate_p2p_case(
            &documents,
            Utc::now(),
            &["user001".into(), "user002".into()],
        );

        // All events should have resource workload populated
        let events_with_workload: Vec<_> = result
            .events
            .iter()
            .filter(|e| e.resource_workload.is_some())
            .collect();
        assert!(
            !events_with_workload.is_empty(),
            "At least some events should have resource workload"
        );

        // Resource workload should be positive
        for event in &events_with_workload {
            assert!(event.resource_workload.unwrap() > 0.0);
        }
    }

    #[test]
    fn test_p2p_three_way_match_correlation() {
        let mut generator = OcpmEventGenerator::with_config(
            42,
            super::super::OcpmGeneratorConfig {
                happy_path_rate: 1.0,
                exception_path_rate: 0.0,
                error_path_rate: 0.0,
                ..Default::default()
            },
        );

        let factory = OcpmUuidFactory::new(42);
        let documents = P2pDocuments::new(
            "PO-000030",
            "V000001",
            "1000",
            Decimal::new(10000, 0),
            "USD",
            &factory,
        )
        .with_goods_receipt("GR-000030", &factory)
        .with_invoice("INV-000030", &factory)
        .with_payment("PAY-000030", &factory);

        let result = generator.generate_p2p_case(
            &documents,
            Utc::now(),
            &["user001".into(), "user002".into()],
        );

        // Should have correlation events
        assert!(
            !result.correlation_events.is_empty(),
            "Happy path P2P should produce correlation events"
        );

        // Should have a three-way match correlation
        let three_way = result
            .correlation_events
            .iter()
            .find(|c| {
                c.correlation_type == crate::models::CorrelationEventType::ThreeWayMatch
            });
        assert!(
            three_way.is_some(),
            "Should have a three-way match correlation"
        );

        // The verify_invoice event should have a correlation_id
        let verify_events: Vec<_> = result
            .events
            .iter()
            .filter(|e| e.activity_id == "verify_invoice")
            .collect();
        assert!(!verify_events.is_empty());
        assert!(verify_events[0].correlation_id.is_some());
        assert!(verify_events[0]
            .correlation_id
            .as_deref()
            .unwrap()
            .starts_with("3WAY-"));

        // Should also have a payment allocation
        let payment_alloc = result
            .correlation_events
            .iter()
            .find(|c| {
                c.correlation_type == crate::models::CorrelationEventType::PaymentAllocation
            });
        assert!(
            payment_alloc.is_some(),
            "Should have a payment allocation correlation"
        );
    }
}
