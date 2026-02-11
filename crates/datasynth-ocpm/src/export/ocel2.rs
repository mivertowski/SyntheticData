//! OCEL 2.0 JSON export functionality.
//!
//! This module exports OCPM event logs in the OCEL 2.0 JSON format,
//! which is the standard format for object-centric event logs.
//!
//! OCEL 2.0 Specification: https://www.ocel-standard.org/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::models::{
    EventLifecycle, ObjectAttributeValue, ObjectGraph, ObjectQualifier, OcpmEvent, OcpmEventLog,
    RelationshipIndex,
};

/// OCEL 2.0 complete log structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ocel2Log {
    /// Object types (metamodel)
    #[serde(rename = "objectTypes")]
    pub object_types: Vec<Ocel2ObjectType>,
    /// Event types (metamodel)
    #[serde(rename = "eventTypes")]
    pub event_types: Vec<Ocel2EventType>,
    /// Object instances
    pub objects: Vec<Ocel2Object>,
    /// Event instances
    pub events: Vec<Ocel2Event>,
    /// Global log attributes
    #[serde(rename = "ocel:global-log", skip_serializing_if = "Option::is_none")]
    pub global_log: Option<Ocel2GlobalLog>,
}

/// OCEL 2.0 global log metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ocel2GlobalLog {
    /// Log name
    #[serde(
        rename = "ocel:attribute-names",
        skip_serializing_if = "Option::is_none"
    )]
    pub attribute_names: Option<Vec<String>>,
    /// Ordering timestamp attribute
    #[serde(rename = "ocel:ordering", skip_serializing_if = "Option::is_none")]
    pub ordering: Option<String>,
    /// Version
    #[serde(rename = "ocel:version", skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// OCEL 2.0 object type definition.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ocel2ObjectType {
    /// Object type name
    pub name: String,
    /// Attributes for this object type
    pub attributes: Vec<Ocel2Attribute>,
}

/// OCEL 2.0 event type definition.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ocel2EventType {
    /// Event type name (activity)
    pub name: String,
    /// Attributes for this event type
    pub attributes: Vec<Ocel2Attribute>,
}

/// OCEL 2.0 attribute definition.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ocel2Attribute {
    /// Attribute name
    pub name: String,
    /// Attribute type (string, integer, float, boolean, time, etc.)
    #[serde(rename = "type")]
    pub attr_type: String,
}

/// OCEL 2.0 object instance.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ocel2Object {
    /// Object identifier (UUID string)
    pub id: String,
    /// Object type name
    #[serde(rename = "type")]
    pub object_type: String,
    /// Object attribute values
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, Ocel2Value>,
    /// Relationships to other objects
    #[serde(rename = "relationships", skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<Ocel2ObjectRelationship>,
}

/// OCEL 2.0 object relationship.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ocel2ObjectRelationship {
    /// Target object ID
    #[serde(rename = "objectId")]
    pub object_id: String,
    /// Relationship qualifier
    pub qualifier: String,
}

/// OCEL 2.0 event instance.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ocel2Event {
    /// Event identifier
    pub id: String,
    /// Event type (activity)
    #[serde(rename = "type")]
    pub event_type: String,
    /// Event timestamp
    pub time: String,
    /// Event attribute values
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, Ocel2Value>,
    /// Related objects with qualifiers
    pub relationships: Vec<Ocel2EventObjectRelationship>,
}

/// OCEL 2.0 event-to-object relationship.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ocel2EventObjectRelationship {
    /// Object ID
    #[serde(rename = "objectId")]
    pub object_id: String,
    /// Qualifier (created, updated, read, etc.)
    pub qualifier: String,
}

/// OCEL 2.0 attribute value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ocel2Value {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Null value
    Null,
}

impl From<&ObjectAttributeValue> for Ocel2Value {
    fn from(value: &ObjectAttributeValue) -> Self {
        match value {
            ObjectAttributeValue::String(s) => Ocel2Value::String(s.clone()),
            ObjectAttributeValue::Integer(i) => Ocel2Value::Integer(*i),
            ObjectAttributeValue::Decimal(d) => {
                // Convert Decimal to f64 for JSON compatibility
                Ocel2Value::Float(d.to_string().parse().unwrap_or(0.0))
            }
            ObjectAttributeValue::Date(d) => Ocel2Value::String(d.to_string()),
            ObjectAttributeValue::DateTime(dt) => Ocel2Value::String(dt.to_rfc3339()),
            ObjectAttributeValue::Boolean(b) => Ocel2Value::Boolean(*b),
            ObjectAttributeValue::Reference(id) => Ocel2Value::String(id.to_string()),
            ObjectAttributeValue::Null => Ocel2Value::Null,
        }
    }
}

/// OCEL 2.0 exporter.
pub struct Ocel2Exporter {
    /// Include metadata attributes
    pub include_metadata: bool,
    /// Include anomaly markers
    pub include_anomalies: bool,
    /// Pretty print JSON
    pub pretty_print: bool,
}

impl Default for Ocel2Exporter {
    fn default() -> Self {
        Self {
            include_metadata: true,
            include_anomalies: true,
            pretty_print: true,
        }
    }
}

impl Ocel2Exporter {
    /// Create a new exporter with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to include metadata attributes.
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set whether to include anomaly markers.
    pub fn with_anomalies(mut self, include: bool) -> Self {
        self.include_anomalies = include;
        self
    }

    /// Set whether to pretty print the output.
    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }

    /// Convert an OCPM event log to OCEL 2.0 format.
    pub fn convert(&self, log: &OcpmEventLog) -> Ocel2Log {
        let object_types = self.convert_object_types(log);
        let event_types = self.convert_event_types(log);
        let objects = self.convert_objects(&log.objects, &log.object_relationships);
        let events = self.convert_events(&log.events);

        let global_log = if self.include_metadata {
            Some(Ocel2GlobalLog {
                attribute_names: Some(vec![
                    "company_code".into(),
                    "resource_id".into(),
                    "document_ref".into(),
                ]),
                ordering: Some("time".into()),
                version: Some("2.0".into()),
            })
        } else {
            None
        };

        Ocel2Log {
            object_types,
            event_types,
            objects,
            events,
            global_log,
        }
    }

    /// Convert object type definitions.
    fn convert_object_types(&self, log: &OcpmEventLog) -> Vec<Ocel2ObjectType> {
        log.object_types
            .values()
            .map(|ot| {
                let mut attributes = vec![
                    Ocel2Attribute {
                        name: "external_id".into(),
                        attr_type: "string".into(),
                    },
                    Ocel2Attribute {
                        name: "company_code".into(),
                        attr_type: "string".into(),
                    },
                    Ocel2Attribute {
                        name: "current_state".into(),
                        attr_type: "string".into(),
                    },
                    Ocel2Attribute {
                        name: "created_at".into(),
                        attr_type: "time".into(),
                    },
                ];

                if self.include_anomalies {
                    attributes.push(Ocel2Attribute {
                        name: "is_anomaly".into(),
                        attr_type: "boolean".into(),
                    });
                }

                Ocel2ObjectType {
                    name: ot.type_id.clone(),
                    attributes,
                }
            })
            .collect()
    }

    /// Convert event type definitions.
    fn convert_event_types(&self, log: &OcpmEventLog) -> Vec<Ocel2EventType> {
        log.activity_types
            .values()
            .map(|at| {
                let mut attributes = vec![
                    Ocel2Attribute {
                        name: "resource_id".into(),
                        attr_type: "string".into(),
                    },
                    Ocel2Attribute {
                        name: "company_code".into(),
                        attr_type: "string".into(),
                    },
                    Ocel2Attribute {
                        name: "lifecycle".into(),
                        attr_type: "string".into(),
                    },
                ];

                if self.include_anomalies {
                    attributes.push(Ocel2Attribute {
                        name: "is_anomaly".into(),
                        attr_type: "boolean".into(),
                    });
                }

                Ocel2EventType {
                    name: at.activity_id.clone(),
                    attributes,
                }
            })
            .collect()
    }

    /// Convert object instances.
    fn convert_objects(
        &self,
        graph: &ObjectGraph,
        relationships: &RelationshipIndex,
    ) -> Vec<Ocel2Object> {
        graph
            .iter()
            .map(|obj| {
                let mut attributes: HashMap<String, Ocel2Value> = obj
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.clone(), Ocel2Value::from(v)))
                    .collect();

                // Add standard attributes
                attributes.insert(
                    "external_id".into(),
                    Ocel2Value::String(obj.external_id.clone()),
                );
                attributes.insert(
                    "company_code".into(),
                    Ocel2Value::String(obj.company_code.clone()),
                );
                attributes.insert(
                    "current_state".into(),
                    Ocel2Value::String(obj.current_state.clone()),
                );
                attributes.insert(
                    "created_at".into(),
                    Ocel2Value::String(obj.created_at.to_rfc3339()),
                );

                if self.include_anomalies {
                    attributes.insert("is_anomaly".into(), Ocel2Value::Boolean(obj.is_anomaly));
                }

                // Get relationships for this object
                let rels: Vec<Ocel2ObjectRelationship> = relationships
                    .get_outgoing(obj.object_id)
                    .into_iter()
                    .map(|rel| Ocel2ObjectRelationship {
                        object_id: rel.target_object_id.to_string(),
                        qualifier: rel.relationship_type.clone(),
                    })
                    .collect();

                Ocel2Object {
                    id: obj.object_id.to_string(),
                    object_type: obj.object_type_id.clone(),
                    attributes,
                    relationships: rels,
                }
            })
            .collect()
    }

    /// Convert events.
    fn convert_events(&self, events: &[OcpmEvent]) -> Vec<Ocel2Event> {
        events
            .iter()
            .map(|event| {
                let mut attributes: HashMap<String, Ocel2Value> = event
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.clone(), Ocel2Value::from(v)))
                    .collect();

                // Add standard attributes
                attributes.insert(
                    "resource_id".into(),
                    Ocel2Value::String(event.resource_id.clone()),
                );
                attributes.insert(
                    "company_code".into(),
                    Ocel2Value::String(event.company_code.clone()),
                );
                attributes.insert(
                    "lifecycle".into(),
                    Ocel2Value::String(lifecycle_to_string(&event.lifecycle)),
                );

                if let Some(ref doc_ref) = event.document_ref {
                    attributes.insert("document_ref".into(), Ocel2Value::String(doc_ref.clone()));
                }

                if let Some(case_id) = event.case_id {
                    attributes.insert("case_id".into(), Ocel2Value::String(case_id.to_string()));
                }

                if self.include_anomalies {
                    attributes.insert("is_anomaly".into(), Ocel2Value::Boolean(event.is_anomaly));
                }

                let relationships: Vec<Ocel2EventObjectRelationship> = event
                    .object_refs
                    .iter()
                    .map(|obj_ref| Ocel2EventObjectRelationship {
                        object_id: obj_ref.object_id.to_string(),
                        qualifier: qualifier_to_string(&obj_ref.qualifier),
                    })
                    .collect();

                Ocel2Event {
                    id: event.event_id.to_string(),
                    event_type: event.activity_id.clone(),
                    time: event.timestamp.to_rfc3339(),
                    attributes,
                    relationships,
                }
            })
            .collect()
    }

    /// Export an OCPM event log to a JSON file.
    pub fn export_to_file<P: AsRef<Path>>(
        &self,
        log: &OcpmEventLog,
        path: P,
    ) -> std::io::Result<()> {
        let ocel2_log = self.convert(log);
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        if self.pretty_print {
            serde_json::to_writer_pretty(writer, &ocel2_log)?;
        } else {
            serde_json::to_writer(writer, &ocel2_log)?;
        }

        Ok(())
    }

    /// Export an OCPM event log to a JSON string.
    pub fn export_to_string(&self, log: &OcpmEventLog) -> serde_json::Result<String> {
        let ocel2_log = self.convert(log);

        if self.pretty_print {
            serde_json::to_string_pretty(&ocel2_log)
        } else {
            serde_json::to_string(&ocel2_log)
        }
    }

    /// Export an OCPM event log to a writer.
    pub fn export_to_writer<W: Write>(
        &self,
        log: &OcpmEventLog,
        writer: W,
    ) -> serde_json::Result<()> {
        let ocel2_log = self.convert(log);

        if self.pretty_print {
            serde_json::to_writer_pretty(writer, &ocel2_log)
        } else {
            serde_json::to_writer(writer, &ocel2_log)
        }
    }
}

/// Convert EventLifecycle to OCEL 2.0 string.
fn lifecycle_to_string(lifecycle: &EventLifecycle) -> String {
    match lifecycle {
        EventLifecycle::Start => "start".into(),
        EventLifecycle::Complete => "complete".into(),
        EventLifecycle::Abort => "abort".into(),
        EventLifecycle::Suspend => "suspend".into(),
        EventLifecycle::Resume => "resume".into(),
        EventLifecycle::Atomic => "atomic".into(),
    }
}

/// Convert ObjectQualifier to OCEL 2.0 string.
fn qualifier_to_string(qualifier: &ObjectQualifier) -> String {
    match qualifier {
        ObjectQualifier::Created => "created".into(),
        ObjectQualifier::Updated => "updated".into(),
        ObjectQualifier::Read => "read".into(),
        ObjectQualifier::Consumed => "consumed".into(),
        ObjectQualifier::Context => "context".into(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_ocel2_exporter_creation() {
        let exporter = Ocel2Exporter::new()
            .with_metadata(true)
            .with_anomalies(true)
            .with_pretty_print(false);

        assert!(exporter.include_metadata);
        assert!(exporter.include_anomalies);
        assert!(!exporter.pretty_print);
    }

    #[test]
    fn test_ocel2_export_empty_log() {
        let log = OcpmEventLog::new().with_standard_types();
        let exporter = Ocel2Exporter::new();

        let ocel2 = exporter.convert(&log);

        assert!(!ocel2.object_types.is_empty());
        assert!(!ocel2.event_types.is_empty());
        assert!(ocel2.objects.is_empty());
        assert!(ocel2.events.is_empty());
    }

    #[test]
    fn test_ocel2_export_to_string() {
        let log = OcpmEventLog::new().with_standard_types();
        let exporter = Ocel2Exporter::new().with_pretty_print(false);

        let json = exporter.export_to_string(&log);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("objectTypes"));
        assert!(json_str.contains("eventTypes"));
    }

    #[test]
    fn test_attribute_value_conversion() {
        let str_val = ObjectAttributeValue::String("test".into());
        let int_val = ObjectAttributeValue::Integer(42);
        let bool_val = ObjectAttributeValue::Boolean(true);
        let null_val = ObjectAttributeValue::Null;

        assert!(matches!(Ocel2Value::from(&str_val), Ocel2Value::String(_)));
        assert!(matches!(
            Ocel2Value::from(&int_val),
            Ocel2Value::Integer(42)
        ));
        assert!(matches!(
            Ocel2Value::from(&bool_val),
            Ocel2Value::Boolean(true)
        ));
        assert!(matches!(Ocel2Value::from(&null_val), Ocel2Value::Null));
    }

    #[test]
    fn test_lifecycle_conversion() {
        assert_eq!(lifecycle_to_string(&EventLifecycle::Start), "start");
        assert_eq!(lifecycle_to_string(&EventLifecycle::Complete), "complete");
        assert_eq!(lifecycle_to_string(&EventLifecycle::Abort), "abort");
        assert_eq!(lifecycle_to_string(&EventLifecycle::Atomic), "atomic");
    }

    #[test]
    fn test_qualifier_conversion() {
        assert_eq!(qualifier_to_string(&ObjectQualifier::Created), "created");
        assert_eq!(qualifier_to_string(&ObjectQualifier::Updated), "updated");
        assert_eq!(qualifier_to_string(&ObjectQualifier::Read), "read");
        assert_eq!(qualifier_to_string(&ObjectQualifier::Consumed), "consumed");
    }
}
