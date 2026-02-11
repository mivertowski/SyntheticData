//! Object type definitions for OCPM.
//!
//! Object types define the schema for business objects that participate
//! in processes, including their lifecycle states and allowed relationships.

use datasynth_core::models::BusinessProcess;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Definition of a business object type in OCPM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectType {
    /// Unique identifier for the object type (e.g., "purchase_order")
    pub type_id: String,
    /// Human-readable name (e.g., "Purchase Order")
    pub name: String,
    /// Business process this type belongs to
    pub business_process: BusinessProcess,
    /// Lifecycle states for this object type
    pub lifecycle_states: Vec<ObjectLifecycleState>,
    /// Allowed relationships to other object types
    pub relationships: Vec<ObjectRelationshipType>,
    /// Activities that can occur on this object type
    pub allowed_activities: Vec<String>,
    /// Attributes schema (key -> type)
    pub attributes: HashMap<String, AttributeType>,
}

impl ObjectType {
    /// Create a Purchase Order object type for P2P.
    pub fn purchase_order() -> Self {
        Self {
            type_id: "purchase_order".into(),
            name: "Purchase Order".into(),
            business_process: BusinessProcess::P2P,
            lifecycle_states: vec![
                ObjectLifecycleState::new("created", "Created", true, false),
                ObjectLifecycleState::new("approved", "Approved", false, false),
                ObjectLifecycleState::new("released", "Released", false, false),
                ObjectLifecycleState::new("received", "Goods Received", false, false),
                ObjectLifecycleState::new("invoiced", "Invoiced", false, false),
                ObjectLifecycleState::new("paid", "Paid", false, true),
                ObjectLifecycleState::new("cancelled", "Cancelled", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "contains",
                    "Contains",
                    "order_line",
                    Cardinality::OneToMany,
                    false,
                ),
                ObjectRelationshipType::new(
                    "fulfilled_by",
                    "Fulfilled By",
                    "goods_receipt",
                    Cardinality::OneToMany,
                    false,
                ),
                ObjectRelationshipType::new(
                    "invoiced_by",
                    "Invoiced By",
                    "vendor_invoice",
                    Cardinality::OneToMany,
                    false,
                ),
            ],
            allowed_activities: vec![
                "create_po".into(),
                "approve_po".into(),
                "release_po".into(),
                "change_po".into(),
                "cancel_po".into(),
            ],
            attributes: HashMap::from([
                ("po_number".into(), AttributeType::String),
                ("vendor_id".into(), AttributeType::String),
                ("total_amount".into(), AttributeType::Decimal),
                ("currency".into(), AttributeType::String),
                ("created_date".into(), AttributeType::Date),
            ]),
        }
    }

    /// Create a Goods Receipt object type for P2P.
    pub fn goods_receipt() -> Self {
        Self {
            type_id: "goods_receipt".into(),
            name: "Goods Receipt".into(),
            business_process: BusinessProcess::P2P,
            lifecycle_states: vec![
                ObjectLifecycleState::new("created", "Created", true, false),
                ObjectLifecycleState::new("posted", "Posted", false, true),
                ObjectLifecycleState::new("reversed", "Reversed", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "references",
                    "References",
                    "purchase_order",
                    Cardinality::ManyToOne,
                    true,
                ),
                ObjectRelationshipType::new(
                    "contains",
                    "Contains",
                    "material",
                    Cardinality::OneToMany,
                    true,
                ),
            ],
            allowed_activities: vec!["create_gr".into(), "post_gr".into(), "reverse_gr".into()],
            attributes: HashMap::from([
                ("gr_number".into(), AttributeType::String),
                ("po_number".into(), AttributeType::String),
                ("receipt_date".into(), AttributeType::Date),
                ("quantity".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Vendor Invoice object type for P2P.
    pub fn vendor_invoice() -> Self {
        Self {
            type_id: "vendor_invoice".into(),
            name: "Vendor Invoice".into(),
            business_process: BusinessProcess::P2P,
            lifecycle_states: vec![
                ObjectLifecycleState::new("received", "Received", true, false),
                ObjectLifecycleState::new("verified", "Verified", false, false),
                ObjectLifecycleState::new("posted", "Posted", false, false),
                ObjectLifecycleState::new("paid", "Paid", false, true),
                ObjectLifecycleState::new("rejected", "Rejected", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "references_po",
                    "References PO",
                    "purchase_order",
                    Cardinality::ManyToOne,
                    false,
                ),
                ObjectRelationshipType::new(
                    "references_gr",
                    "References GR",
                    "goods_receipt",
                    Cardinality::ManyToMany,
                    false,
                ),
            ],
            allowed_activities: vec![
                "receive_invoice".into(),
                "verify_invoice".into(),
                "post_invoice".into(),
                "reject_invoice".into(),
            ],
            attributes: HashMap::from([
                ("invoice_number".into(), AttributeType::String),
                ("vendor_id".into(), AttributeType::String),
                ("invoice_amount".into(), AttributeType::Decimal),
                ("invoice_date".into(), AttributeType::Date),
            ]),
        }
    }

    /// Create a Sales Order object type for O2C.
    pub fn sales_order() -> Self {
        Self {
            type_id: "sales_order".into(),
            name: "Sales Order".into(),
            business_process: BusinessProcess::O2C,
            lifecycle_states: vec![
                ObjectLifecycleState::new("created", "Created", true, false),
                ObjectLifecycleState::new("credit_checked", "Credit Checked", false, false),
                ObjectLifecycleState::new("released", "Released", false, false),
                ObjectLifecycleState::new("delivered", "Delivered", false, false),
                ObjectLifecycleState::new("invoiced", "Invoiced", false, false),
                ObjectLifecycleState::new("paid", "Paid", false, true),
                ObjectLifecycleState::new("cancelled", "Cancelled", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "contains",
                    "Contains",
                    "order_line",
                    Cardinality::OneToMany,
                    false,
                ),
                ObjectRelationshipType::new(
                    "fulfilled_by",
                    "Fulfilled By",
                    "delivery",
                    Cardinality::OneToMany,
                    false,
                ),
            ],
            allowed_activities: vec![
                "create_so".into(),
                "check_credit".into(),
                "release_so".into(),
                "change_so".into(),
                "cancel_so".into(),
            ],
            attributes: HashMap::from([
                ("so_number".into(), AttributeType::String),
                ("customer_id".into(), AttributeType::String),
                ("total_amount".into(), AttributeType::Decimal),
                ("currency".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Delivery object type for O2C.
    pub fn delivery() -> Self {
        Self {
            type_id: "delivery".into(),
            name: "Delivery".into(),
            business_process: BusinessProcess::O2C,
            lifecycle_states: vec![
                ObjectLifecycleState::new("created", "Created", true, false),
                ObjectLifecycleState::new("picked", "Picked", false, false),
                ObjectLifecycleState::new("packed", "Packed", false, false),
                ObjectLifecycleState::new("shipped", "Shipped", false, true),
                ObjectLifecycleState::new("cancelled", "Cancelled", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "fulfills",
                "Fulfills",
                "sales_order",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec![
                "create_delivery".into(),
                "pick".into(),
                "pack".into(),
                "ship".into(),
            ],
            attributes: HashMap::from([
                ("delivery_number".into(), AttributeType::String),
                ("so_number".into(), AttributeType::String),
                ("ship_date".into(), AttributeType::Date),
            ]),
        }
    }

    /// Create a Customer Invoice object type for O2C.
    pub fn customer_invoice() -> Self {
        Self {
            type_id: "customer_invoice".into(),
            name: "Customer Invoice".into(),
            business_process: BusinessProcess::O2C,
            lifecycle_states: vec![
                ObjectLifecycleState::new("created", "Created", true, false),
                ObjectLifecycleState::new("posted", "Posted", false, false),
                ObjectLifecycleState::new("sent", "Sent", false, false),
                ObjectLifecycleState::new("paid", "Paid", false, true),
                ObjectLifecycleState::new("written_off", "Written Off", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "references_so",
                    "References SO",
                    "sales_order",
                    Cardinality::ManyToOne,
                    false,
                ),
                ObjectRelationshipType::new(
                    "references_delivery",
                    "References Delivery",
                    "delivery",
                    Cardinality::ManyToMany,
                    false,
                ),
            ],
            allowed_activities: vec![
                "create_invoice".into(),
                "post_invoice".into(),
                "send_invoice".into(),
            ],
            attributes: HashMap::from([
                ("invoice_number".into(), AttributeType::String),
                ("customer_id".into(), AttributeType::String),
                ("invoice_amount".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Get all standard P2P object types.
    pub fn p2p_types() -> Vec<Self> {
        vec![
            Self::purchase_order(),
            Self::goods_receipt(),
            Self::vendor_invoice(),
        ]
    }

    /// Get all standard O2C object types.
    pub fn o2c_types() -> Vec<Self> {
        vec![
            Self::sales_order(),
            Self::delivery(),
            Self::customer_invoice(),
        ]
    }
}

/// State in an object's lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLifecycleState {
    /// State identifier
    pub state_id: String,
    /// Human-readable name
    pub name: String,
    /// Is this an initial state (object starts here)
    pub is_initial: bool,
    /// Is this a terminal state (object ends here)
    pub is_terminal: bool,
    /// Valid transitions from this state
    pub valid_transitions: Vec<String>,
}

impl ObjectLifecycleState {
    /// Create a new lifecycle state.
    pub fn new(state_id: &str, name: &str, is_initial: bool, is_terminal: bool) -> Self {
        Self {
            state_id: state_id.into(),
            name: name.into(),
            is_initial,
            is_terminal,
            valid_transitions: Vec::new(),
        }
    }

    /// Add valid transitions from this state.
    pub fn with_transitions(mut self, transitions: Vec<&str>) -> Self {
        self.valid_transitions = transitions.into_iter().map(String::from).collect();
        self
    }
}

/// Type of relationship between object types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRelationshipType {
    /// Relationship type identifier
    pub relationship_id: String,
    /// Human-readable name
    pub name: String,
    /// Target object type ID
    pub target_type_id: String,
    /// Cardinality of the relationship
    pub cardinality: Cardinality,
    /// Is this relationship mandatory
    pub is_mandatory: bool,
}

impl ObjectRelationshipType {
    /// Create a new relationship type.
    pub fn new(
        relationship_id: &str,
        name: &str,
        target_type_id: &str,
        cardinality: Cardinality,
        is_mandatory: bool,
    ) -> Self {
        Self {
            relationship_id: relationship_id.into(),
            name: name.into(),
            target_type_id: target_type_id.into(),
            cardinality,
            is_mandatory,
        }
    }
}

/// Cardinality of object relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    /// One source to one target
    OneToOne,
    /// One source to many targets
    OneToMany,
    /// Many sources to one target
    ManyToOne,
    /// Many sources to many targets
    ManyToMany,
}

/// Attribute types for object attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeType {
    /// String value
    String,
    /// Integer value
    Integer,
    /// Decimal value (for monetary amounts)
    Decimal,
    /// Date value
    Date,
    /// DateTime value
    DateTime,
    /// Boolean value
    Boolean,
    /// Reference to another object type
    Reference(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_order_type() {
        let po_type = ObjectType::purchase_order();
        assert_eq!(po_type.type_id, "purchase_order");
        assert_eq!(po_type.business_process, BusinessProcess::P2P);
        assert!(!po_type.lifecycle_states.is_empty());
        assert!(!po_type.relationships.is_empty());
    }

    #[test]
    fn test_p2p_types() {
        let types = ObjectType::p2p_types();
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn test_o2c_types() {
        let types = ObjectType::o2c_types();
        assert_eq!(types.len(), 3);
    }
}
