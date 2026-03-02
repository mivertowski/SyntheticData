//! Bank Reconciliation process event generator.
//!
//! Generates OCPM events for the bank reconciliation flow:
//! Import Statement → Auto Match → Manual Match → Create Reconciling Items →
//! Resolve Exceptions → Approve → Post → Complete

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::{CaseGenerationResult, OcpmEventGenerator, OcpmUuidFactory, VariantType};
use crate::models::{ActivityType, EventObjectRef, ObjectAttributeValue, ObjectType};
use datasynth_core::models::BusinessProcess;

/// Bank Reconciliation document references for event generation.
#[derive(Debug, Clone)]
pub struct BankReconDocuments {
    /// Reconciliation ID
    pub reconciliation_id: String,
    /// Reconciliation UUID
    pub reconciliation_uuid: Uuid,
    /// Bank account ID
    pub bank_account_id: String,
    /// Company code
    pub company_code: String,
    /// Statement line IDs
    pub statement_line_ids: Vec<String>,
    /// Reconciling item IDs
    pub reconciling_item_ids: Vec<String>,
    /// Total statement amount
    pub total_amount: Decimal,
}

impl BankReconDocuments {
    /// Create new Bank Recon documents.
    pub fn new(
        reconciliation_id: &str,
        bank_account_id: &str,
        company_code: &str,
        total_amount: Decimal,
        factory: &OcpmUuidFactory,
    ) -> Self {
        Self {
            reconciliation_id: reconciliation_id.into(),
            reconciliation_uuid: factory.next_document_id(),
            bank_account_id: bank_account_id.into(),
            company_code: company_code.into(),
            statement_line_ids: Vec::new(),
            reconciling_item_ids: Vec::new(),
            total_amount,
        }
    }

    /// Add statement line IDs.
    pub fn with_statement_lines(mut self, ids: Vec<&str>) -> Self {
        self.statement_line_ids = ids.into_iter().map(String::from).collect();
        self
    }

    /// Add reconciling item IDs.
    pub fn with_reconciling_items(mut self, ids: Vec<&str>) -> Self {
        self.reconciling_item_ids = ids.into_iter().map(String::from).collect();
        self
    }
}

impl OcpmEventGenerator {
    /// Generate complete bank reconciliation process events.
    pub fn generate_bank_recon_case(
        &mut self,
        documents: &BankReconDocuments,
        start_time: DateTime<Utc>,
        available_users: &[String],
    ) -> CaseGenerationResult {
        let case_id = self.new_case_id();
        let variant_type = self.select_variant_type();

        let mut events = Vec::new();
        let mut objects = Vec::new();
        let mut relationships = Vec::new();
        let mut current_time = start_time;

        let recon_type = ObjectType::bank_reconciliation();
        let line_type = ObjectType::bank_statement_line();
        let item_type = ObjectType::reconciling_item();

        // Create Reconciliation object
        let recon_object = self.create_object(
            &recon_type,
            &documents.reconciliation_id,
            &documents.company_code,
            current_time,
        );
        objects.push(recon_object.clone());

        // Import Bank Statement
        let import = ActivityType::import_bank_statement();
        let resource = self.select_resource(&import, available_users);
        let mut event = self.create_event(
            &import,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::created(recon_object.object_id, &recon_type.type_id)
                    .with_external_id(&documents.reconciliation_id),
            )
            .with_document_ref(&documents.reconciliation_id);
        Self::add_event_attribute(
            &mut event,
            "bank_account_id",
            ObjectAttributeValue::String(documents.bank_account_id.clone()),
        );
        Self::add_event_attribute(
            &mut event,
            "total_amount",
            ObjectAttributeValue::Decimal(documents.total_amount),
        );
        events.push(event);

        // Create statement line objects
        let mut line_objects = Vec::new();
        for line_id in &documents.statement_line_ids {
            let line_object =
                self.create_object(&line_type, line_id, &documents.company_code, current_time);
            relationships.push(self.create_relationship(
                "belongs_to",
                line_object.object_id,
                &line_type.type_id,
                recon_object.object_id,
                &recon_type.type_id,
            ));
            line_objects.push(line_object.clone());
            objects.push(line_object);
        }

        // Auto Match Items
        current_time = self.calculate_event_time(current_time, &import);

        let auto_match = ActivityType::auto_match_items();
        let resource = self.select_resource(&auto_match, available_users);
        let mut event = self.create_event(
            &auto_match,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        // Reference lines being auto-matched
        for line_obj in &line_objects {
            event = event.with_object(
                EventObjectRef::updated(line_obj.object_id, &line_type.type_id)
                    .with_external_id(&line_obj.external_id),
            );
        }
        let auto_match_count = line_objects.len();
        Self::add_event_attribute(
            &mut event,
            "matched_count",
            ObjectAttributeValue::Integer(auto_match_count as i64),
        );
        events.push(event);

        // Manual Match (exception and happy paths)
        if !documents.reconciling_item_ids.is_empty() {
            current_time = self.calculate_event_time(current_time, &auto_match);
            current_time += self.generate_inter_activity_delay(30, 240);

            for item_id in &documents.reconciling_item_ids {
                let item_object =
                    self.create_object(&item_type, item_id, &documents.company_code, current_time);
                relationships.push(self.create_relationship(
                    "belongs_to",
                    item_object.object_id,
                    &item_type.type_id,
                    recon_object.object_id,
                    &recon_type.type_id,
                ));
                objects.push(item_object.clone());

                // Create reconciling item
                let create_item = ActivityType::create_reconciling_item();
                let resource = self.select_resource(&create_item, available_users);
                let mut event = self.create_event(
                    &create_item,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event
                    .with_object(
                        EventObjectRef::created(item_object.object_id, &item_type.type_id)
                            .with_external_id(item_id),
                    )
                    .with_document_ref(item_id);
                events.push(event);

                // Resolve exception (skip for error paths)
                if !matches!(variant_type, VariantType::ErrorPath) {
                    current_time += self.generate_inter_activity_delay(30, 480);

                    let resolve = ActivityType::resolve_exception();
                    let resource = self.select_resource(&resolve, available_users);
                    let mut event = self.create_event(
                        &resolve,
                        current_time,
                        &resource,
                        &documents.company_code,
                        case_id,
                    );
                    event = event.with_object(
                        EventObjectRef::updated(item_object.object_id, &item_type.type_id)
                            .with_external_id(item_id),
                    );
                    events.push(event);
                }

                current_time += self.generate_inter_activity_delay(15, 60);
            }
        }

        // Skip approval/posting for error paths with unresolved items
        if matches!(variant_type, VariantType::ErrorPath) {
            let case_trace = self.create_case_trace(
                case_id,
                &events,
                BusinessProcess::R2R,
                recon_object.object_id,
                &recon_type.type_id,
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

        // Approve Reconciliation
        current_time += self.generate_inter_activity_delay(30, 240);

        let approve = ActivityType::approve_reconciliation();
        let resource = self.select_resource(&approve, available_users);
        let mut event = self.create_event(
            &approve,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(recon_object.object_id, &recon_type.type_id)
                .with_external_id(&documents.reconciliation_id),
        );
        events.push(event);

        // Post Reconciliation Entries
        current_time = self.calculate_event_time(current_time, &approve);

        let post = ActivityType::post_recon_entries();
        let resource = self.select_resource(&post, available_users);
        let mut event = self.create_event(
            &post,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(recon_object.object_id, &recon_type.type_id)
                .with_external_id(&documents.reconciliation_id),
        );
        events.push(event);

        // Complete Reconciliation
        current_time = self.calculate_event_time(current_time, &post);

        let complete = ActivityType::complete_reconciliation();
        let resource = self.select_resource(&complete, available_users);
        let mut event = self.create_event(
            &complete,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::consumed(recon_object.object_id, &recon_type.type_id)
                .with_external_id(&documents.reconciliation_id),
        );
        events.push(event);

        let case_trace = self.create_case_trace(
            case_id,
            &events,
            BusinessProcess::R2R,
            recon_object.object_id,
            &recon_type.type_id,
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
    fn test_bank_recon_case_generation() {
        let mut generator = OcpmEventGenerator::new(42);
        let factory = OcpmUuidFactory::new(42);
        let documents = BankReconDocuments::new(
            "BR-000001",
            "BA-001",
            "1000",
            Decimal::new(100000, 0),
            &factory,
        )
        .with_statement_lines(vec!["BSL-001", "BSL-002", "BSL-003"])
        .with_reconciling_items(vec!["RI-001"]);

        let result =
            generator.generate_bank_recon_case(&documents, Utc::now(), &["user001".into()]);

        assert!(result.events.len() >= 3);
        assert!(!result.objects.is_empty());
        assert!(!result.case_trace.activity_sequence.is_empty());
    }
}
