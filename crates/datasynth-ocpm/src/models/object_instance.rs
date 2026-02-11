//! Object instance model for OCPM.
//!
//! Object instances are specific occurrences of object types that
//! participate in process executions.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A business object instance in OCPM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInstance {
    /// Unique object identifier
    pub object_id: Uuid,
    /// Object type reference
    pub object_type_id: String,
    /// External ID (e.g., "PO-1000-0000000001")
    pub external_id: String,
    /// Current lifecycle state
    pub current_state: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp (if terminal state reached)
    pub completed_at: Option<DateTime<Utc>>,
    /// Company code
    pub company_code: String,
    /// Object attributes
    pub attributes: HashMap<String, ObjectAttributeValue>,
    /// Is this object marked as anomalous
    pub is_anomaly: bool,
    /// Anomaly type if applicable
    pub anomaly_type: Option<String>,
}

impl ObjectInstance {
    /// Create a new object instance.
    pub fn new(object_type_id: &str, external_id: &str, company_code: &str) -> Self {
        Self {
            object_id: Uuid::new_v4(),
            object_type_id: object_type_id.into(),
            external_id: external_id.into(),
            current_state: "created".into(),
            created_at: Utc::now(),
            completed_at: None,
            company_code: company_code.into(),
            attributes: HashMap::new(),
            is_anomaly: false,
            anomaly_type: None,
        }
    }

    /// Create with a specific UUID (for deterministic generation).
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.object_id = id;
        self
    }

    /// Set the initial state.
    pub fn with_state(mut self, state: &str) -> Self {
        self.current_state = state.into();
        self
    }

    /// Set creation timestamp.
    pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }

    /// Add an attribute.
    pub fn with_attribute(mut self, key: &str, value: ObjectAttributeValue) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    /// Mark as completed.
    pub fn complete(&mut self, terminal_state: &str) {
        self.current_state = terminal_state.into();
        self.completed_at = Some(Utc::now());
    }

    /// Transition to a new state.
    pub fn transition(&mut self, new_state: &str) {
        self.current_state = new_state.into();
    }

    /// Mark as anomalous.
    pub fn mark_anomaly(&mut self, anomaly_type: &str) {
        self.is_anomaly = true;
        self.anomaly_type = Some(anomaly_type.into());
    }

    /// Check if the object is in a terminal state.
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Get a string attribute value.
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.attributes.get(key) {
            Some(ObjectAttributeValue::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get a decimal attribute value.
    pub fn get_decimal(&self, key: &str) -> Option<Decimal> {
        match self.attributes.get(key) {
            Some(ObjectAttributeValue::Decimal(d)) => Some(*d),
            _ => None,
        }
    }
}

/// Attribute values for object instances.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ObjectAttributeValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Decimal value (for monetary amounts)
    Decimal(Decimal),
    /// Date value
    Date(NaiveDate),
    /// DateTime value
    DateTime(DateTime<Utc>),
    /// Boolean value
    Boolean(bool),
    /// Reference to another object
    Reference(Uuid),
    /// Null/missing value
    Null,
}

impl From<String> for ObjectAttributeValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for ObjectAttributeValue {
    fn from(s: &str) -> Self {
        Self::String(s.into())
    }
}

impl From<i64> for ObjectAttributeValue {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<Decimal> for ObjectAttributeValue {
    fn from(d: Decimal) -> Self {
        Self::Decimal(d)
    }
}

impl From<NaiveDate> for ObjectAttributeValue {
    fn from(d: NaiveDate) -> Self {
        Self::Date(d)
    }
}

impl From<DateTime<Utc>> for ObjectAttributeValue {
    fn from(dt: DateTime<Utc>) -> Self {
        Self::DateTime(dt)
    }
}

impl From<bool> for ObjectAttributeValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<Uuid> for ObjectAttributeValue {
    fn from(id: Uuid) -> Self {
        Self::Reference(id)
    }
}

/// Graph of objects with relationships indexed for fast lookup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectGraph {
    /// All objects by ID
    pub objects: HashMap<Uuid, ObjectInstance>,
    /// Objects indexed by type
    objects_by_type: HashMap<String, Vec<Uuid>>,
    /// Objects indexed by external ID
    objects_by_external_id: HashMap<String, Uuid>,
}

impl ObjectGraph {
    /// Create a new empty object graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an object to the graph.
    pub fn add_object(&mut self, object: ObjectInstance) {
        let object_id = object.object_id;
        let type_id = object.object_type_id.clone();
        let external_id = object.external_id.clone();

        self.objects.insert(object_id, object);

        self.objects_by_type
            .entry(type_id)
            .or_default()
            .push(object_id);

        self.objects_by_external_id.insert(external_id, object_id);
    }

    /// Get an object by ID.
    pub fn get(&self, object_id: Uuid) -> Option<&ObjectInstance> {
        self.objects.get(&object_id)
    }

    /// Get a mutable reference to an object.
    pub fn get_mut(&mut self, object_id: Uuid) -> Option<&mut ObjectInstance> {
        self.objects.get_mut(&object_id)
    }

    /// Get an object by external ID.
    pub fn get_by_external_id(&self, external_id: &str) -> Option<&ObjectInstance> {
        self.objects_by_external_id
            .get(external_id)
            .and_then(|id| self.objects.get(id))
    }

    /// Get all objects of a specific type.
    pub fn get_by_type(&self, type_id: &str) -> Vec<&ObjectInstance> {
        self.objects_by_type
            .get(type_id)
            .map(|ids| ids.iter().filter_map(|id| self.objects.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get the total number of objects.
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Iterate over all objects.
    pub fn iter(&self) -> impl Iterator<Item = &ObjectInstance> {
        self.objects.values()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_object_instance_creation() {
        let obj = ObjectInstance::new("purchase_order", "PO-001", "1000");
        assert_eq!(obj.object_type_id, "purchase_order");
        assert_eq!(obj.external_id, "PO-001");
        assert_eq!(obj.company_code, "1000");
        assert!(!obj.is_anomaly);
    }

    #[test]
    fn test_object_graph() {
        let mut graph = ObjectGraph::new();

        let po1 = ObjectInstance::new("purchase_order", "PO-001", "1000");
        let po2 = ObjectInstance::new("purchase_order", "PO-002", "1000");
        let gr1 = ObjectInstance::new("goods_receipt", "GR-001", "1000");

        graph.add_object(po1);
        graph.add_object(po2);
        graph.add_object(gr1);

        assert_eq!(graph.len(), 3);
        assert_eq!(graph.get_by_type("purchase_order").len(), 2);
        assert_eq!(graph.get_by_type("goods_receipt").len(), 1);
        assert!(graph.get_by_external_id("PO-001").is_some());
    }
}
