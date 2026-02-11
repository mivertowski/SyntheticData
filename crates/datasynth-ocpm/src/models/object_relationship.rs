//! Object relationship model for OCPM.
//!
//! Object relationships capture the many-to-many connections between
//! business objects, such as "Order contains OrderLines" or
//! "Invoice references PurchaseOrder".

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::ObjectAttributeValue;

/// Many-to-many relationship between object instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRelationship {
    /// Unique relationship ID
    pub relationship_id: Uuid,
    /// Relationship type (from ObjectRelationshipType)
    pub relationship_type: String,
    /// Source object ID
    pub source_object_id: Uuid,
    /// Source object type
    pub source_type_id: String,
    /// Target object ID
    pub target_object_id: Uuid,
    /// Target object type
    pub target_type_id: String,
    /// When the relationship was established
    pub established_at: DateTime<Utc>,
    /// Optional quantity for the relationship (e.g., items ordered)
    pub quantity: Option<Decimal>,
    /// Additional attributes
    pub attributes: HashMap<String, ObjectAttributeValue>,
}

impl ObjectRelationship {
    /// Create a new object relationship.
    pub fn new(
        relationship_type: &str,
        source_object_id: Uuid,
        source_type_id: &str,
        target_object_id: Uuid,
        target_type_id: &str,
    ) -> Self {
        Self {
            relationship_id: Uuid::new_v4(),
            relationship_type: relationship_type.into(),
            source_object_id,
            source_type_id: source_type_id.into(),
            target_object_id,
            target_type_id: target_type_id.into(),
            established_at: Utc::now(),
            quantity: None,
            attributes: HashMap::new(),
        }
    }

    /// Set the established timestamp.
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.established_at = timestamp;
        self
    }

    /// Set the quantity.
    pub fn with_quantity(mut self, quantity: Decimal) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Add an attribute.
    pub fn with_attribute(mut self, key: &str, value: ObjectAttributeValue) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }
}

/// Index for fast relationship lookups.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipIndex {
    /// All relationships
    relationships: Vec<ObjectRelationship>,
    /// Index: source_object_id -> relationship indices
    by_source: HashMap<Uuid, Vec<usize>>,
    /// Index: target_object_id -> relationship indices
    by_target: HashMap<Uuid, Vec<usize>>,
    /// Index: relationship_type -> relationship indices
    by_type: HashMap<String, Vec<usize>>,
}

impl RelationshipIndex {
    /// Create a new relationship index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a relationship to the index.
    pub fn add(&mut self, relationship: ObjectRelationship) {
        let idx = self.relationships.len();

        self.by_source
            .entry(relationship.source_object_id)
            .or_default()
            .push(idx);

        self.by_target
            .entry(relationship.target_object_id)
            .or_default()
            .push(idx);

        self.by_type
            .entry(relationship.relationship_type.clone())
            .or_default()
            .push(idx);

        self.relationships.push(relationship);
    }

    /// Get all relationships from a source object.
    pub fn get_outgoing(&self, source_id: Uuid) -> Vec<&ObjectRelationship> {
        self.by_source
            .get(&source_id)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.relationships.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all relationships to a target object.
    pub fn get_incoming(&self, target_id: Uuid) -> Vec<&ObjectRelationship> {
        self.by_target
            .get(&target_id)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.relationships.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all relationships of a specific type.
    pub fn get_by_type(&self, relationship_type: &str) -> Vec<&ObjectRelationship> {
        self.by_type
            .get(relationship_type)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.relationships.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all relationships.
    pub fn all(&self) -> &[ObjectRelationship] {
        &self.relationships
    }

    /// Get the total number of relationships.
    pub fn len(&self) -> usize {
        self.relationships.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.relationships.is_empty()
    }

    /// Iterate over all relationships.
    pub fn iter(&self) -> impl Iterator<Item = &ObjectRelationship> {
        self.relationships.iter()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_creation() {
        let source_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();

        let rel = ObjectRelationship::new(
            "contains",
            source_id,
            "purchase_order",
            target_id,
            "order_line",
        )
        .with_quantity(Decimal::from(10));

        assert_eq!(rel.relationship_type, "contains");
        assert_eq!(rel.source_object_id, source_id);
        assert_eq!(rel.target_object_id, target_id);
        assert_eq!(rel.quantity, Some(Decimal::from(10)));
    }

    #[test]
    fn test_relationship_index() {
        let mut index = RelationshipIndex::new();

        let po_id = Uuid::new_v4();
        let line1_id = Uuid::new_v4();
        let line2_id = Uuid::new_v4();

        index.add(ObjectRelationship::new(
            "contains",
            po_id,
            "purchase_order",
            line1_id,
            "order_line",
        ));
        index.add(ObjectRelationship::new(
            "contains",
            po_id,
            "purchase_order",
            line2_id,
            "order_line",
        ));

        assert_eq!(index.len(), 2);
        assert_eq!(index.get_outgoing(po_id).len(), 2);
        assert_eq!(index.get_incoming(line1_id).len(), 1);
        assert_eq!(index.get_by_type("contains").len(), 2);
    }
}
