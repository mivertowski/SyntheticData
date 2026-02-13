//! BANK (Banking) process event generator.
//!
//! Generates OCPM events for the banking operations flow:
//! Onboard Customer → KYC Review → Open Account → Execute/Authorize/Complete Transactions

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::{CaseGenerationResult, OcpmEventGenerator, VariantType};
use crate::models::{
    ActivityType, EventObjectRef, ObjectAttributeValue, ObjectRelationship, ObjectType,
};
use datasynth_core::models::BusinessProcess;

/// Banking document references for event generation.
#[derive(Debug, Clone)]
pub struct BankDocuments {
    /// Banking customer ID
    pub customer_id: String,
    /// Customer UUID
    pub customer_uuid: Uuid,
    /// Bank account ID
    pub account_id: Option<String>,
    /// Account UUID
    pub account_uuid: Option<Uuid>,
    /// Transaction IDs
    pub transaction_ids: Vec<String>,
    /// Company code
    pub company_code: String,
    /// Transaction amounts
    pub transaction_amounts: Vec<Decimal>,
}

impl BankDocuments {
    /// Create new Banking documents.
    pub fn new(customer_id: &str, company_code: &str) -> Self {
        Self {
            customer_id: customer_id.into(),
            customer_uuid: Uuid::new_v4(),
            account_id: None,
            account_uuid: None,
            transaction_ids: Vec::new(),
            company_code: company_code.into(),
            transaction_amounts: Vec::new(),
        }
    }

    /// Set account info.
    pub fn with_account(mut self, account_id: &str) -> Self {
        self.account_id = Some(account_id.into());
        self.account_uuid = Some(Uuid::new_v4());
        self
    }

    /// Add transaction IDs with amounts.
    pub fn with_transactions(mut self, ids: Vec<&str>, amounts: Vec<Decimal>) -> Self {
        self.transaction_ids = ids.into_iter().map(String::from).collect();
        self.transaction_amounts = amounts;
        self
    }
}

impl OcpmEventGenerator {
    /// Generate complete banking process events.
    pub fn generate_bank_case(
        &mut self,
        documents: &BankDocuments,
        start_time: DateTime<Utc>,
        available_users: &[String],
    ) -> CaseGenerationResult {
        let case_id = self.new_case_id();
        let variant_type = self.select_variant_type();

        let mut events = Vec::new();
        let mut objects = Vec::new();
        let mut relationships = Vec::new();
        let mut current_time = start_time;

        let customer_type = ObjectType::banking_customer();
        let account_type = ObjectType::bank_account();
        let txn_type = ObjectType::bank_transaction();

        // Onboard Customer
        let customer_object = self.create_object(
            &customer_type,
            &documents.customer_id,
            &documents.company_code,
            current_time,
        );
        objects.push(customer_object.clone());

        let onboard = ActivityType::onboard_customer();
        let resource = self.select_resource(&onboard, available_users);
        let mut event = self.create_event(
            &onboard,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::created(customer_object.object_id, &customer_type.type_id)
                    .with_external_id(&documents.customer_id),
            )
            .with_document_ref(&documents.customer_id);
        events.push(event);

        // KYC Review
        current_time = self.calculate_event_time(current_time, &onboard);
        current_time += self.generate_inter_activity_delay(60, 2880); // 1h-2d

        let kyc = ActivityType::perform_kyc_review();
        let resource = self.select_resource(&kyc, available_users);
        let mut event = self.create_event(
            &kyc,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(customer_object.object_id, &customer_type.type_id)
                .with_external_id(&documents.customer_id),
        );
        Self::add_event_attribute(
            &mut event,
            "kyc_result",
            ObjectAttributeValue::String("approved".into()),
        );
        events.push(event);

        // Skip remaining for error paths (KYC failed)
        if matches!(variant_type, VariantType::ErrorPath) {
            let case_trace = self.create_case_trace(
                case_id,
                &events,
                BusinessProcess::Bank,
                customer_object.object_id,
                &customer_type.type_id,
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

        // Open Account
        if let Some(account_id) = &documents.account_id {
            current_time = self.calculate_event_time(current_time, &kyc);
            current_time += self.generate_inter_activity_delay(30, 480);

            let account_object = self.create_object(
                &account_type,
                account_id,
                &documents.company_code,
                current_time,
            );
            objects.push(account_object.clone());

            relationships.push(ObjectRelationship::new(
                "owned_by",
                account_object.object_id,
                &account_type.type_id,
                customer_object.object_id,
                &customer_type.type_id,
            ));

            let open = ActivityType::open_account();
            let resource = self.select_resource(&open, available_users);
            let mut event = self.create_event(
                &open,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(account_object.object_id, &account_type.type_id)
                        .with_external_id(account_id),
                )
                .with_object(
                    EventObjectRef::read(customer_object.object_id, &customer_type.type_id)
                        .with_external_id(&documents.customer_id),
                )
                .with_document_ref(account_id);
            events.push(event);

            // Process transactions
            for (i, txn_id) in documents.transaction_ids.iter().enumerate() {
                current_time += self.generate_inter_activity_delay(1, 1440);

                let txn_object = self.create_object(
                    &txn_type,
                    txn_id,
                    &documents.company_code,
                    current_time,
                );
                objects.push(txn_object.clone());

                relationships.push(ObjectRelationship::new(
                    "on_account",
                    txn_object.object_id,
                    &txn_type.type_id,
                    account_object.object_id,
                    &account_type.type_id,
                ));

                let amount = documents
                    .transaction_amounts
                    .get(i)
                    .copied()
                    .unwrap_or_else(|| Decimal::new(1000, 0));

                // Execute Transaction
                let execute = ActivityType::execute_bank_transaction();
                let resource = self.select_resource(&execute, available_users);
                let mut event = self.create_event(
                    &execute,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event
                    .with_object(
                        EventObjectRef::created(txn_object.object_id, &txn_type.type_id)
                            .with_external_id(txn_id),
                    )
                    .with_object(
                        EventObjectRef::read(account_object.object_id, &account_type.type_id)
                            .with_external_id(account_id),
                    )
                    .with_document_ref(txn_id);
                Self::add_event_attribute(
                    &mut event,
                    "amount",
                    ObjectAttributeValue::Decimal(amount),
                );
                events.push(event);

                // Flag suspicious (exception path)
                if matches!(variant_type, VariantType::ExceptionPath)
                    && self.should_skip_activity(0.7)
                {
                    current_time = self.calculate_event_time(current_time, &execute);

                    let flag = ActivityType::flag_suspicious();
                    let resource = self.select_resource(&flag, available_users);
                    let mut event = self.create_event(
                        &flag,
                        current_time,
                        &resource,
                        &documents.company_code,
                        case_id,
                    );
                    event = event.with_object(
                        EventObjectRef::updated(txn_object.object_id, &txn_type.type_id)
                            .with_external_id(txn_id),
                    );
                    events.push(event);

                    // Freeze account on exception
                    if self.should_skip_activity(0.5) {
                        current_time += self.generate_inter_activity_delay(5, 60);

                        let freeze = ActivityType::freeze_account();
                        let resource = self.select_resource(&freeze, available_users);
                        let mut event = self.create_event(
                            &freeze,
                            current_time,
                            &resource,
                            &documents.company_code,
                            case_id,
                        );
                        event = event.with_object(
                            EventObjectRef::updated(
                                account_object.object_id,
                                &account_type.type_id,
                            )
                            .with_external_id(account_id),
                        );
                        events.push(event);
                    }
                    continue;
                }

                // Authorize Transaction
                current_time = self.calculate_event_time(current_time, &execute);

                let authorize = ActivityType::authorize_transaction();
                let resource = self.select_resource(&authorize, available_users);
                let mut event = self.create_event(
                    &authorize,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event.with_object(
                    EventObjectRef::updated(txn_object.object_id, &txn_type.type_id)
                        .with_external_id(txn_id),
                );
                events.push(event);

                // Complete Transaction
                current_time = self.calculate_event_time(current_time, &authorize);

                let complete = ActivityType::complete_bank_transaction();
                let resource = self.select_resource(&complete, available_users);
                let mut event = self.create_event(
                    &complete,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event.with_object(
                    EventObjectRef::consumed(txn_object.object_id, &txn_type.type_id)
                        .with_external_id(txn_id),
                );
                events.push(event);
            }
        }

        let case_trace = self.create_case_trace(
            case_id,
            &events,
            BusinessProcess::Bank,
            customer_object.object_id,
            &customer_type.type_id,
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
    fn test_bank_case_generation() {
        let mut generator = OcpmEventGenerator::new(42);
        let documents = BankDocuments::new("BC-001", "1000")
            .with_account("BA-001")
            .with_transactions(
                vec!["TXN-001", "TXN-002"],
                vec![Decimal::new(5000, 0), Decimal::new(3000, 0)],
            );

        let result = generator.generate_bank_case(
            &documents,
            Utc::now(),
            &["user001".into()],
        );

        assert!(result.events.len() >= 3); // onboard + kyc + open account
        assert!(!result.objects.is_empty());
        assert!(!result.case_trace.activity_sequence.is_empty());
    }

    #[test]
    fn test_bank_error_path() {
        let mut generator = OcpmEventGenerator::with_config(
            123,
            super::super::OcpmGeneratorConfig {
                error_path_rate: 1.0,
                happy_path_rate: 0.0,
                exception_path_rate: 0.0,
                ..Default::default()
            },
        );

        let documents = BankDocuments::new("BC-002", "1000")
            .with_account("BA-002")
            .with_transactions(vec!["TXN-003"], vec![Decimal::new(1000, 0)]);

        let result = generator.generate_bank_case(&documents, Utc::now(), &[]);

        assert_eq!(result.variant_type, VariantType::ErrorPath);
        assert_eq!(result.events.len(), 2); // onboard + kyc only
    }
}
