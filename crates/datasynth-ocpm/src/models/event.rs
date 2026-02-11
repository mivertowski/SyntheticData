//! Event model for OCPM.
//!
//! Events represent occurrences of activities on objects. A key feature
//! of OCPM is that events can involve multiple objects (many-to-many).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::ObjectAttributeValue;

/// An event instance in OCPM event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcpmEvent {
    /// Unique event ID
    pub event_id: Uuid,
    /// Activity type that occurred
    pub activity_id: String,
    /// Activity name (for convenience)
    pub activity_name: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Lifecycle transition (Start, Complete, Abort, etc.)
    pub lifecycle: EventLifecycle,
    /// Resource (user/system) that performed the event
    pub resource_id: String,
    /// Resource name (for convenience)
    pub resource_name: Option<String>,
    /// Company code
    pub company_code: String,
    /// Objects involved in this event (many-to-many)
    pub object_refs: Vec<EventObjectRef>,
    /// Event attributes
    pub attributes: HashMap<String, ObjectAttributeValue>,
    /// Related document reference (JE, PO number, etc.)
    pub document_ref: Option<String>,
    /// Related journal entry ID
    pub journal_entry_id: Option<Uuid>,
    /// Anomaly flag
    pub is_anomaly: bool,
    /// Anomaly type if applicable
    pub anomaly_type: Option<String>,
    /// Case ID for process instance tracking
    pub case_id: Option<Uuid>,
}

impl OcpmEvent {
    /// Create a new event.
    pub fn new(
        activity_id: &str,
        activity_name: &str,
        timestamp: DateTime<Utc>,
        resource_id: &str,
        company_code: &str,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            activity_id: activity_id.into(),
            activity_name: activity_name.into(),
            timestamp,
            lifecycle: EventLifecycle::Complete,
            resource_id: resource_id.into(),
            resource_name: None,
            company_code: company_code.into(),
            object_refs: Vec::new(),
            attributes: HashMap::new(),
            document_ref: None,
            journal_entry_id: None,
            is_anomaly: false,
            anomaly_type: None,
            case_id: None,
        }
    }

    /// Set a specific event ID.
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.event_id = id;
        self
    }

    /// Set the lifecycle phase.
    pub fn with_lifecycle(mut self, lifecycle: EventLifecycle) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    /// Set the resource name.
    pub fn with_resource_name(mut self, name: &str) -> Self {
        self.resource_name = Some(name.into());
        self
    }

    /// Add an object reference.
    pub fn with_object(mut self, object_ref: EventObjectRef) -> Self {
        self.object_refs.push(object_ref);
        self
    }

    /// Add multiple object references.
    pub fn with_objects(mut self, refs: Vec<EventObjectRef>) -> Self {
        self.object_refs.extend(refs);
        self
    }

    /// Add an attribute.
    pub fn with_attribute(mut self, key: &str, value: ObjectAttributeValue) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Set document reference.
    pub fn with_document_ref(mut self, doc_ref: &str) -> Self {
        self.document_ref = Some(doc_ref.into());
        self
    }

    /// Set journal entry ID.
    pub fn with_journal_entry(mut self, je_id: Uuid) -> Self {
        self.journal_entry_id = Some(je_id);
        self
    }

    /// Set case ID.
    pub fn with_case(mut self, case_id: Uuid) -> Self {
        self.case_id = Some(case_id);
        self
    }

    /// Mark as anomalous.
    pub fn mark_anomaly(&mut self, anomaly_type: &str) {
        self.is_anomaly = true;
        self.anomaly_type = Some(anomaly_type.into());
    }

    /// Get all object IDs involved in this event.
    pub fn object_ids(&self) -> Vec<Uuid> {
        self.object_refs.iter().map(|r| r.object_id).collect()
    }

    /// Get object refs of a specific type.
    pub fn objects_of_type(&self, type_id: &str) -> Vec<&EventObjectRef> {
        self.object_refs
            .iter()
            .filter(|r| r.object_type_id == type_id)
            .collect()
    }

    /// Check if this event creates any object.
    pub fn creates_objects(&self) -> bool {
        self.object_refs
            .iter()
            .any(|r| r.qualifier == ObjectQualifier::Created)
    }

    /// Check if this event completes any object.
    pub fn completes_objects(&self) -> bool {
        self.object_refs
            .iter()
            .any(|r| r.qualifier == ObjectQualifier::Consumed)
    }
}

/// Event lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventLifecycle {
    /// Activity started
    Start,
    /// Activity completed
    #[default]
    Complete,
    /// Activity aborted
    Abort,
    /// Activity suspended
    Suspend,
    /// Activity resumed
    Resume,
    /// Atomic event (no duration, single timestamp)
    Atomic,
}

impl EventLifecycle {
    /// Check if this is a completion event.
    pub fn is_completion(&self) -> bool {
        matches!(self, Self::Complete | Self::Abort)
    }

    /// Check if this is a start event.
    pub fn is_start(&self) -> bool {
        matches!(self, Self::Start)
    }
}

/// Reference from event to object with qualifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventObjectRef {
    /// Object ID
    pub object_id: Uuid,
    /// Object type ID
    pub object_type_id: String,
    /// Object external ID (for convenience)
    pub external_id: Option<String>,
    /// Qualifier describing the relationship
    pub qualifier: ObjectQualifier,
}

impl EventObjectRef {
    /// Create a new object reference.
    pub fn new(object_id: Uuid, object_type_id: &str, qualifier: ObjectQualifier) -> Self {
        Self {
            object_id,
            object_type_id: object_type_id.into(),
            external_id: None,
            qualifier,
        }
    }

    /// Set the external ID.
    pub fn with_external_id(mut self, external_id: &str) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    /// Create a reference for a created object.
    pub fn created(object_id: Uuid, object_type_id: &str) -> Self {
        Self::new(object_id, object_type_id, ObjectQualifier::Created)
    }

    /// Create a reference for an updated object.
    pub fn updated(object_id: Uuid, object_type_id: &str) -> Self {
        Self::new(object_id, object_type_id, ObjectQualifier::Updated)
    }

    /// Create a reference for a read/referenced object.
    pub fn read(object_id: Uuid, object_type_id: &str) -> Self {
        Self::new(object_id, object_type_id, ObjectQualifier::Read)
    }

    /// Create a reference for a consumed/completed object.
    pub fn consumed(object_id: Uuid, object_type_id: &str) -> Self {
        Self::new(object_id, object_type_id, ObjectQualifier::Consumed)
    }

    /// Create a reference for a context object.
    pub fn context(object_id: Uuid, object_type_id: &str) -> Self {
        Self::new(object_id, object_type_id, ObjectQualifier::Context)
    }
}

/// Qualifier for event-object relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObjectQualifier {
    /// Object is created by this event
    Created,
    /// Object is updated by this event
    #[default]
    Updated,
    /// Object is read/referenced by this event (no change)
    Read,
    /// Object is consumed/completed by this event
    Consumed,
    /// Object is a context object (indirect involvement)
    Context,
}

impl ObjectQualifier {
    /// Check if this qualifier indicates an object change.
    pub fn changes_object(&self) -> bool {
        matches!(self, Self::Created | Self::Updated | Self::Consumed)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = OcpmEvent::new(
            "create_po",
            "Create Purchase Order",
            Utc::now(),
            "user001",
            "1000",
        );

        assert_eq!(event.activity_id, "create_po");
        assert_eq!(event.lifecycle, EventLifecycle::Complete);
        assert!(!event.is_anomaly);
    }

    #[test]
    fn test_event_with_objects() {
        let po_id = Uuid::new_v4();
        let vendor_id = Uuid::new_v4();

        let event = OcpmEvent::new("create_po", "Create PO", Utc::now(), "user001", "1000")
            .with_object(EventObjectRef::created(po_id, "purchase_order"))
            .with_object(EventObjectRef::read(vendor_id, "vendor"));

        assert_eq!(event.object_refs.len(), 2);
        assert!(event.creates_objects());
    }

    #[test]
    fn test_object_qualifier() {
        assert!(ObjectQualifier::Created.changes_object());
        assert!(ObjectQualifier::Updated.changes_object());
        assert!(!ObjectQualifier::Read.changes_object());
        assert!(!ObjectQualifier::Context.changes_object());
    }
}
