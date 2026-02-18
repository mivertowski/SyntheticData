//! H2R (Hire-to-Retire) process event generator.
//!
//! Generates OCPM events for the H2R flow:
//! Submit Time → Approve Time → Create Payroll → Calculate → Approve → Post
//! Submit Expense → Approve Expense

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::{CaseGenerationResult, OcpmEventGenerator, OcpmUuidFactory, VariantType};
use crate::models::{ActivityType, EventObjectRef, ObjectAttributeValue, ObjectType};
use datasynth_core::models::BusinessProcess;

/// H2R document references for event generation.
#[derive(Debug, Clone)]
pub struct H2rDocuments {
    /// Payroll run ID
    pub payroll_id: String,
    /// Payroll run UUID
    pub payroll_uuid: Uuid,
    /// Employee ID
    pub employee_id: String,
    /// Company code
    pub company_code: String,
    /// Gross amount
    pub gross_amount: Decimal,
    /// Time entry IDs
    pub time_entry_ids: Vec<String>,
    /// Expense report ID
    pub expense_report_id: Option<String>,
    /// Expense report UUID
    pub expense_report_uuid: Option<Uuid>,
}

impl H2rDocuments {
    /// Create new H2R documents.
    pub fn new(
        payroll_id: &str,
        employee_id: &str,
        company_code: &str,
        gross_amount: Decimal,
        factory: &OcpmUuidFactory,
    ) -> Self {
        Self {
            payroll_id: payroll_id.into(),
            payroll_uuid: factory.next_document_id(),
            employee_id: employee_id.into(),
            company_code: company_code.into(),
            gross_amount,
            time_entry_ids: Vec::new(),
            expense_report_id: None,
            expense_report_uuid: None,
        }
    }

    /// Add time entry IDs.
    pub fn with_time_entries(mut self, ids: Vec<&str>) -> Self {
        self.time_entry_ids = ids.into_iter().map(String::from).collect();
        self
    }

    /// Set expense report info.
    pub fn with_expense_report(mut self, report_id: &str, factory: &OcpmUuidFactory) -> Self {
        self.expense_report_id = Some(report_id.into());
        self.expense_report_uuid = Some(factory.next_document_id());
        self
    }
}

impl OcpmEventGenerator {
    /// Generate complete H2R process events.
    pub fn generate_h2r_case(
        &mut self,
        documents: &H2rDocuments,
        start_time: DateTime<Utc>,
        available_users: &[String],
    ) -> CaseGenerationResult {
        let case_id = self.new_case_id();
        let variant_type = self.select_variant_type();

        let mut events = Vec::new();
        let mut objects = Vec::new();
        let mut relationships = Vec::new();
        let mut current_time = start_time;

        let payroll_type = ObjectType::payroll_run();
        let time_type = ObjectType::time_entry();
        let expense_type = ObjectType::expense_report();

        // Submit time entries first
        for time_id in &documents.time_entry_ids {
            let time_object =
                self.create_object(&time_type, time_id, &documents.company_code, current_time);
            objects.push(time_object.clone());

            let submit_time = ActivityType::submit_time_entry();
            let resource = self.select_resource(&submit_time, available_users);
            let mut event = self.create_event(
                &submit_time,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(time_object.object_id, &time_type.type_id)
                        .with_external_id(time_id),
                )
                .with_document_ref(time_id);
            Self::add_event_attribute(
                &mut event,
                "employee_id",
                ObjectAttributeValue::String(documents.employee_id.clone()),
            );
            events.push(event);

            // Approve time entry (skip for exception paths sometimes)
            if !matches!(variant_type, VariantType::ExceptionPath)
                || !self.should_skip_activity(0.3)
            {
                current_time = self.calculate_event_time(current_time, &submit_time);
                current_time += self.generate_inter_activity_delay(60, 1440);

                let approve_time = ActivityType::approve_time_entry();
                let resource = self.select_resource(&approve_time, available_users);
                let mut event = self.create_event(
                    &approve_time,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event.with_object(
                    EventObjectRef::updated(time_object.object_id, &time_type.type_id)
                        .with_external_id(time_id),
                );
                events.push(event);
            }

            current_time += self.generate_inter_activity_delay(30, 120);
        }

        // Submit expense report if present
        if let Some(expense_id) = &documents.expense_report_id {
            let expense_object = self.create_object(
                &expense_type,
                expense_id,
                &documents.company_code,
                current_time,
            );
            objects.push(expense_object.clone());

            let submit_expense = ActivityType::submit_expense();
            let resource = self.select_resource(&submit_expense, available_users);
            let mut event = self.create_event(
                &submit_expense,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(expense_object.object_id, &expense_type.type_id)
                        .with_external_id(expense_id),
                )
                .with_document_ref(expense_id);
            events.push(event);

            if !matches!(variant_type, VariantType::ErrorPath) {
                current_time = self.calculate_event_time(current_time, &submit_expense);
                current_time += self.generate_inter_activity_delay(60, 2880);

                let approve_expense = ActivityType::approve_expense();
                let resource = self.select_resource(&approve_expense, available_users);
                let mut event = self.create_event(
                    &approve_expense,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event.with_object(
                    EventObjectRef::updated(expense_object.object_id, &expense_type.type_id)
                        .with_external_id(expense_id),
                );
                events.push(event);
            }
        }

        // Skip payroll for error paths
        if matches!(variant_type, VariantType::ErrorPath) {
            let case_trace = self.create_case_trace(
                case_id,
                &events,
                BusinessProcess::H2R,
                documents.payroll_uuid,
                &payroll_type.type_id,
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

        // Create Payroll Run
        current_time += self.generate_inter_activity_delay(1440, 7200);

        let payroll_object = self.create_object(
            &payroll_type,
            &documents.payroll_id,
            &documents.company_code,
            current_time,
        );
        objects.push(payroll_object.clone());

        let create_payroll = ActivityType::create_payroll_run();
        let resource = self.select_resource(&create_payroll, available_users);
        let mut event = self.create_event(
            &create_payroll,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::created(payroll_object.object_id, &payroll_type.type_id)
                    .with_external_id(&documents.payroll_id),
            )
            .with_document_ref(&documents.payroll_id);
        Self::add_event_attribute(
            &mut event,
            "gross_amount",
            ObjectAttributeValue::Decimal(documents.gross_amount),
        );
        events.push(event);

        // Calculate Payroll
        current_time = self.calculate_event_time(current_time, &create_payroll);

        let calculate = ActivityType::calculate_payroll();
        let resource = self.select_resource(&calculate, available_users);
        let mut event = self.create_event(
            &calculate,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(payroll_object.object_id, &payroll_type.type_id)
                .with_external_id(&documents.payroll_id),
        );
        events.push(event);

        // Approve Payroll
        current_time = self.calculate_event_time(current_time, &calculate);
        current_time += self.generate_inter_activity_delay(60, 480);

        let approve = ActivityType::approve_payroll();
        let resource = self.select_resource(&approve, available_users);
        let mut event = self.create_event(
            &approve,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(payroll_object.object_id, &payroll_type.type_id)
                .with_external_id(&documents.payroll_id),
        );
        events.push(event);

        // Post Payroll
        current_time = self.calculate_event_time(current_time, &approve);

        let post = ActivityType::post_payroll();
        let resource = self.select_resource(&post, available_users);
        let mut event = self.create_event(
            &post,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::consumed(payroll_object.object_id, &payroll_type.type_id)
                .with_external_id(&documents.payroll_id),
        );
        // Link time entries to payroll
        for time_obj in objects.iter().filter(|o| o.object_type_id == "time_entry") {
            relationships.push(self.create_relationship(
                "feeds_into",
                time_obj.object_id,
                &time_type.type_id,
                payroll_object.object_id,
                &payroll_type.type_id,
            ));
        }
        events.push(event);

        let case_trace = self.create_case_trace(
            case_id,
            &events,
            BusinessProcess::H2R,
            payroll_object.object_id,
            &payroll_type.type_id,
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
    fn test_h2r_case_generation() {
        let mut generator = OcpmEventGenerator::new(42);
        let factory = OcpmUuidFactory::new(42);
        let documents = H2rDocuments::new(
            "PR-000001",
            "EMP001",
            "1000",
            Decimal::new(5000, 0),
            &factory,
        )
        .with_time_entries(vec!["TE-001", "TE-002"])
        .with_expense_report("ER-001", &factory);

        let result = generator.generate_h2r_case(
            &documents,
            Utc::now(),
            &["user001".into(), "user002".into()],
        );

        assert!(result.events.len() >= 4); // at least time entries + payroll create
        assert!(!result.objects.is_empty());
        assert!(!result.case_trace.activity_sequence.is_empty());
    }
}
