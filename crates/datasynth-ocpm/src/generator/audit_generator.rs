//! AUDIT process event generator.
//!
//! Generates OCPM events for the audit engagement lifecycle:
//! Create Engagement → Plan → Assess Risk → Create Workpapers → Collect Evidence →
//! Review → Raise Findings → Remediate → Record Judgments → Complete

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{CaseGenerationResult, OcpmEventGenerator, VariantType};
use crate::models::{
    ActivityType, EventObjectRef, ObjectAttributeValue, ObjectRelationship, ObjectType,
};
use datasynth_core::models::BusinessProcess;

/// Audit document references for event generation.
#[derive(Debug, Clone)]
pub struct AuditDocuments {
    /// Engagement ID
    pub engagement_id: String,
    /// Engagement UUID
    pub engagement_uuid: Uuid,
    /// Workpaper IDs
    pub workpaper_ids: Vec<String>,
    /// Finding IDs
    pub finding_ids: Vec<String>,
    /// Evidence IDs
    pub evidence_ids: Vec<String>,
    /// Risk assessment IDs
    pub risk_ids: Vec<String>,
    /// Judgment IDs
    pub judgment_ids: Vec<String>,
    /// Company code
    pub company_code: String,
}

impl AuditDocuments {
    /// Create new Audit documents.
    pub fn new(engagement_id: &str, company_code: &str) -> Self {
        Self {
            engagement_id: engagement_id.into(),
            engagement_uuid: Uuid::new_v4(),
            workpaper_ids: Vec::new(),
            finding_ids: Vec::new(),
            evidence_ids: Vec::new(),
            risk_ids: Vec::new(),
            judgment_ids: Vec::new(),
            company_code: company_code.into(),
        }
    }

    /// Add workpaper IDs.
    pub fn with_workpapers(mut self, ids: Vec<&str>) -> Self {
        self.workpaper_ids = ids.into_iter().map(String::from).collect();
        self
    }

    /// Add finding IDs.
    pub fn with_findings(mut self, ids: Vec<&str>) -> Self {
        self.finding_ids = ids.into_iter().map(String::from).collect();
        self
    }

    /// Add evidence IDs.
    pub fn with_evidence(mut self, ids: Vec<&str>) -> Self {
        self.evidence_ids = ids.into_iter().map(String::from).collect();
        self
    }

    /// Add risk assessment IDs.
    pub fn with_risks(mut self, ids: Vec<&str>) -> Self {
        self.risk_ids = ids.into_iter().map(String::from).collect();
        self
    }

    /// Add judgment IDs.
    pub fn with_judgments(mut self, ids: Vec<&str>) -> Self {
        self.judgment_ids = ids.into_iter().map(String::from).collect();
        self
    }
}

impl OcpmEventGenerator {
    /// Generate complete audit engagement process events.
    pub fn generate_audit_case(
        &mut self,
        documents: &AuditDocuments,
        start_time: DateTime<Utc>,
        available_users: &[String],
    ) -> CaseGenerationResult {
        let case_id = self.new_case_id();
        let variant_type = self.select_variant_type();

        let mut events = Vec::new();
        let mut objects = Vec::new();
        let mut relationships = Vec::new();
        let mut current_time = start_time;

        let engagement_type = ObjectType::audit_engagement();
        let workpaper_type = ObjectType::workpaper();
        let finding_type = ObjectType::audit_finding();
        let evidence_type = ObjectType::audit_evidence();
        let risk_type = ObjectType::risk_assessment();
        let judgment_type = ObjectType::professional_judgment();

        // Create Engagement
        let engagement_object = self.create_object(
            &engagement_type,
            &documents.engagement_id,
            &documents.company_code,
            current_time,
        );
        objects.push(engagement_object.clone());

        let create_eng = ActivityType::create_engagement();
        let resource = self.select_resource(&create_eng, available_users);
        let mut event = self.create_event(
            &create_eng,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event
            .with_object(
                EventObjectRef::created(engagement_object.object_id, &engagement_type.type_id)
                    .with_external_id(&documents.engagement_id),
            )
            .with_document_ref(&documents.engagement_id);
        events.push(event);

        // Plan Engagement
        current_time = self.calculate_event_time(current_time, &create_eng);
        current_time += self.generate_inter_activity_delay(1440, 7200); // 1-5 days

        let plan = ActivityType::plan_engagement();
        let resource = self.select_resource(&plan, available_users);
        let mut event = self.create_event(
            &plan,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::updated(engagement_object.object_id, &engagement_type.type_id)
                .with_external_id(&documents.engagement_id),
        );
        events.push(event);

        // Skip remaining for error paths (engagement on hold)
        if matches!(variant_type, VariantType::ErrorPath) {
            let case_trace = self.create_case_trace(
                case_id,
                &events,
                BusinessProcess::Audit,
                engagement_object.object_id,
                &engagement_type.type_id,
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

        // Assess Risks
        for risk_id in &documents.risk_ids {
            current_time += self.generate_inter_activity_delay(480, 2880);

            let risk_object =
                self.create_object(&risk_type, risk_id, &documents.company_code, current_time);
            objects.push(risk_object.clone());

            relationships.push(ObjectRelationship::new(
                "belongs_to",
                risk_object.object_id,
                &risk_type.type_id,
                engagement_object.object_id,
                &engagement_type.type_id,
            ));

            let assess = ActivityType::assess_risk();
            let resource = self.select_resource(&assess, available_users);
            let mut event = self.create_event(
                &assess,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(risk_object.object_id, &risk_type.type_id)
                        .with_external_id(risk_id),
                )
                .with_object(
                    EventObjectRef::read(engagement_object.object_id, &engagement_type.type_id)
                        .with_external_id(&documents.engagement_id),
                )
                .with_document_ref(risk_id);
            events.push(event);
        }

        // Create Workpapers + Collect Evidence
        let mut workpaper_objects = Vec::new();
        for (wp_idx, wp_id) in documents.workpaper_ids.iter().enumerate() {
            current_time += self.generate_inter_activity_delay(480, 2880);

            let wp_object = self.create_object(
                &workpaper_type,
                wp_id,
                &documents.company_code,
                current_time,
            );
            objects.push(wp_object.clone());
            workpaper_objects.push(wp_object.clone());

            relationships.push(ObjectRelationship::new(
                "belongs_to",
                wp_object.object_id,
                &workpaper_type.type_id,
                engagement_object.object_id,
                &engagement_type.type_id,
            ));

            let create_wp = ActivityType::create_workpaper();
            let resource = self.select_resource(&create_wp, available_users);
            let mut event = self.create_event(
                &create_wp,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(wp_object.object_id, &workpaper_type.type_id)
                        .with_external_id(wp_id),
                )
                .with_object(
                    EventObjectRef::read(engagement_object.object_id, &engagement_type.type_id)
                        .with_external_id(&documents.engagement_id),
                )
                .with_document_ref(wp_id);
            events.push(event);

            // Collect evidence for this workpaper
            if let Some(ev_id) = documents.evidence_ids.get(wp_idx) {
                current_time += self.generate_inter_activity_delay(120, 1440);

                let ev_object = self.create_object(
                    &evidence_type,
                    ev_id,
                    &documents.company_code,
                    current_time,
                );
                objects.push(ev_object.clone());

                relationships.push(ObjectRelationship::new(
                    "supports",
                    ev_object.object_id,
                    &evidence_type.type_id,
                    wp_object.object_id,
                    &workpaper_type.type_id,
                ));

                let collect = ActivityType::collect_evidence();
                let resource = self.select_resource(&collect, available_users);
                let mut event = self.create_event(
                    &collect,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event
                    .with_object(
                        EventObjectRef::created(ev_object.object_id, &evidence_type.type_id)
                            .with_external_id(ev_id),
                    )
                    .with_object(
                        EventObjectRef::read(wp_object.object_id, &workpaper_type.type_id)
                            .with_external_id(wp_id),
                    )
                    .with_document_ref(ev_id);
                events.push(event);
            }

            // Review workpaper
            current_time += self.generate_inter_activity_delay(480, 2880);

            let review = ActivityType::review_workpaper();
            let resource = self.select_resource(&review, available_users);
            let mut event = self.create_event(
                &review,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event.with_object(
                EventObjectRef::updated(wp_object.object_id, &workpaper_type.type_id)
                    .with_external_id(wp_id),
            );
            events.push(event);
        }

        // Raise Findings (exception path: findings requiring remediation)
        for finding_id in &documents.finding_ids {
            current_time += self.generate_inter_activity_delay(240, 1440);

            let finding_object = self.create_object(
                &finding_type,
                finding_id,
                &documents.company_code,
                current_time,
            );
            objects.push(finding_object.clone());

            relationships.push(ObjectRelationship::new(
                "belongs_to",
                finding_object.object_id,
                &finding_type.type_id,
                engagement_object.object_id,
                &engagement_type.type_id,
            ));

            let raise = ActivityType::raise_finding();
            let resource = self.select_resource(&raise, available_users);
            let mut event = self.create_event(
                &raise,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(finding_object.object_id, &finding_type.type_id)
                        .with_external_id(finding_id),
                )
                .with_object(
                    EventObjectRef::read(engagement_object.object_id, &engagement_type.type_id)
                        .with_external_id(&documents.engagement_id),
                )
                .with_document_ref(finding_id);
            events.push(event);

            // Remediate finding
            if !matches!(variant_type, VariantType::ExceptionPath)
                || !self.should_skip_activity(0.3)
            {
                current_time += self.generate_inter_activity_delay(1440, 14400); // 1-10 days

                let remediate = ActivityType::remediate_finding();
                let resource = self.select_resource(&remediate, available_users);
                let mut event = self.create_event(
                    &remediate,
                    current_time,
                    &resource,
                    &documents.company_code,
                    case_id,
                );
                event = event.with_object(
                    EventObjectRef::updated(finding_object.object_id, &finding_type.type_id)
                        .with_external_id(finding_id),
                );
                Self::add_event_attribute(
                    &mut event,
                    "remediation_status",
                    ObjectAttributeValue::String("resolved".into()),
                );
                events.push(event);
            }
        }

        // Record Judgments
        for judgment_id in &documents.judgment_ids {
            current_time += self.generate_inter_activity_delay(120, 480);

            let judgment_object = self.create_object(
                &judgment_type,
                judgment_id,
                &documents.company_code,
                current_time,
            );
            objects.push(judgment_object.clone());

            let record = ActivityType::record_judgment();
            let resource = self.select_resource(&record, available_users);
            let mut event = self.create_event(
                &record,
                current_time,
                &resource,
                &documents.company_code,
                case_id,
            );
            event = event
                .with_object(
                    EventObjectRef::created(judgment_object.object_id, &judgment_type.type_id)
                        .with_external_id(judgment_id),
                )
                .with_document_ref(judgment_id);
            events.push(event);
        }

        // Complete Engagement
        current_time += self.generate_inter_activity_delay(1440, 7200);

        let complete = ActivityType::complete_engagement();
        let resource = self.select_resource(&complete, available_users);
        let mut event = self.create_event(
            &complete,
            current_time,
            &resource,
            &documents.company_code,
            case_id,
        );
        event = event.with_object(
            EventObjectRef::consumed(engagement_object.object_id, &engagement_type.type_id)
                .with_external_id(&documents.engagement_id),
        );
        events.push(event);

        let case_trace = self.create_case_trace(
            case_id,
            &events,
            BusinessProcess::Audit,
            engagement_object.object_id,
            &engagement_type.type_id,
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
    fn test_audit_case_generation() {
        let mut generator = OcpmEventGenerator::new(42);
        let documents = AuditDocuments::new("AE-001", "1000")
            .with_workpapers(vec!["WP-001", "WP-002"])
            .with_evidence(vec!["EV-001", "EV-002"])
            .with_risks(vec!["RA-001"])
            .with_findings(vec!["AF-001"])
            .with_judgments(vec!["PJ-001"]);

        let result = generator.generate_audit_case(
            &documents,
            Utc::now(),
            &["auditor001".into(), "auditor002".into()],
        );

        assert!(result.events.len() >= 4); // create + plan + at least some fieldwork
        assert!(!result.objects.is_empty());
        assert!(!result.case_trace.activity_sequence.is_empty());
    }

    #[test]
    fn test_audit_error_path() {
        let mut generator = OcpmEventGenerator::with_config(
            123,
            super::super::OcpmGeneratorConfig {
                error_path_rate: 1.0,
                happy_path_rate: 0.0,
                exception_path_rate: 0.0,
                ..Default::default()
            },
        );

        let documents = AuditDocuments::new("AE-002", "1000")
            .with_workpapers(vec!["WP-003"])
            .with_findings(vec!["AF-002"]);

        let result = generator.generate_audit_case(&documents, Utc::now(), &[]);

        assert_eq!(result.variant_type, VariantType::ErrorPath);
        assert_eq!(result.events.len(), 2); // create + plan only
    }
}
