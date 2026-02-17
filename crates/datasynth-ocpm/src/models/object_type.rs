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

    // ========== S2C Object Types ==========

    /// Create a Sourcing Project object type.
    pub fn sourcing_project() -> Self {
        Self {
            type_id: "sourcing_project".into(),
            name: "Sourcing Project".into(),
            business_process: BusinessProcess::S2C,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("rfx_active", "RFx Active", false, false),
                ObjectLifecycleState::new("awarded", "Awarded", false, false),
                ObjectLifecycleState::new("completed", "Completed", false, true),
                ObjectLifecycleState::new("cancelled", "Cancelled", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "has_rfx",
                    "Has RFx",
                    "rfx_event",
                    Cardinality::OneToMany,
                    false,
                ),
                ObjectRelationshipType::new(
                    "awarded_contract",
                    "Awarded Contract",
                    "procurement_contract",
                    Cardinality::OneToOne,
                    false,
                ),
            ],
            allowed_activities: vec![
                "create_sourcing_project".into(),
                "publish_rfx".into(),
                "award_contract".into(),
                "complete_sourcing".into(),
            ],
            attributes: HashMap::from([
                ("project_id".into(), AttributeType::String),
                ("category".into(), AttributeType::String),
                ("estimated_value".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Supplier Qualification object type.
    pub fn supplier_qualification() -> Self {
        Self {
            type_id: "supplier_qualification".into(),
            name: "Supplier Qualification".into(),
            business_process: BusinessProcess::S2C,
            lifecycle_states: vec![
                ObjectLifecycleState::new("pending", "Pending", true, false),
                ObjectLifecycleState::new("in_progress", "In Progress", false, false),
                ObjectLifecycleState::new("qualified", "Qualified", false, true),
                ObjectLifecycleState::new("disqualified", "Disqualified", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "qualifies",
                "Qualifies",
                "vendor",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["qualify_supplier".into()],
            attributes: HashMap::from([
                ("qualification_id".into(), AttributeType::String),
                ("vendor_id".into(), AttributeType::String),
                ("score".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create an RFx Event object type.
    pub fn rfx_event() -> Self {
        Self {
            type_id: "rfx_event".into(),
            name: "RFx Event".into(),
            business_process: BusinessProcess::S2C,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("published", "Published", false, false),
                ObjectLifecycleState::new("closed", "Closed", false, false),
                ObjectLifecycleState::new("awarded", "Awarded", false, true),
                ObjectLifecycleState::new("cancelled", "Cancelled", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "belongs_to",
                    "Belongs To",
                    "sourcing_project",
                    Cardinality::ManyToOne,
                    true,
                ),
                ObjectRelationshipType::new(
                    "has_bids",
                    "Has Bids",
                    "supplier_bid",
                    Cardinality::OneToMany,
                    false,
                ),
            ],
            allowed_activities: vec!["publish_rfx".into(), "evaluate_bids".into()],
            attributes: HashMap::from([
                ("rfx_id".into(), AttributeType::String),
                ("rfx_type".into(), AttributeType::String),
                ("deadline".into(), AttributeType::DateTime),
            ]),
        }
    }

    /// Create a Supplier Bid object type.
    pub fn supplier_bid() -> Self {
        Self {
            type_id: "supplier_bid".into(),
            name: "Supplier Bid".into(),
            business_process: BusinessProcess::S2C,
            lifecycle_states: vec![
                ObjectLifecycleState::new("submitted", "Submitted", true, false),
                ObjectLifecycleState::new("under_evaluation", "Under Evaluation", false, false),
                ObjectLifecycleState::new("accepted", "Accepted", false, true),
                ObjectLifecycleState::new("rejected", "Rejected", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "responds_to",
                "Responds To",
                "rfx_event",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["submit_bid".into()],
            attributes: HashMap::from([
                ("bid_id".into(), AttributeType::String),
                ("vendor_id".into(), AttributeType::String),
                ("bid_amount".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Bid Evaluation object type.
    pub fn bid_evaluation() -> Self {
        Self {
            type_id: "bid_evaluation".into(),
            name: "Bid Evaluation".into(),
            business_process: BusinessProcess::S2C,
            lifecycle_states: vec![
                ObjectLifecycleState::new("created", "Created", true, false),
                ObjectLifecycleState::new("finalized", "Finalized", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "evaluates",
                "Evaluates",
                "rfx_event",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["evaluate_bids".into()],
            attributes: HashMap::from([
                ("evaluation_id".into(), AttributeType::String),
                ("winning_bid_id".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Procurement Contract object type.
    pub fn procurement_contract() -> Self {
        Self {
            type_id: "procurement_contract".into(),
            name: "Procurement Contract".into(),
            business_process: BusinessProcess::S2C,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("active", "Active", false, false),
                ObjectLifecycleState::new("expired", "Expired", false, true),
                ObjectLifecycleState::new("terminated", "Terminated", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "awarded_from",
                "Awarded From",
                "sourcing_project",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["award_contract".into(), "activate_contract".into()],
            attributes: HashMap::from([
                ("contract_id".into(), AttributeType::String),
                ("vendor_id".into(), AttributeType::String),
                ("contract_value".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Get all S2C object types.
    pub fn s2c_types() -> Vec<Self> {
        vec![
            Self::sourcing_project(),
            Self::supplier_qualification(),
            Self::rfx_event(),
            Self::supplier_bid(),
            Self::bid_evaluation(),
            Self::procurement_contract(),
        ]
    }

    // ========== H2R Object Types ==========

    /// Create a Payroll Run object type.
    pub fn payroll_run() -> Self {
        Self {
            type_id: "payroll_run".into(),
            name: "Payroll Run".into(),
            business_process: BusinessProcess::H2R,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("calculated", "Calculated", false, false),
                ObjectLifecycleState::new("approved", "Approved", false, false),
                ObjectLifecycleState::new("posted", "Posted", false, true),
                ObjectLifecycleState::new("reversed", "Reversed", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "contains",
                "Contains",
                "payroll_line_item",
                Cardinality::OneToMany,
                false,
            )],
            allowed_activities: vec![
                "create_payroll_run".into(),
                "calculate_payroll".into(),
                "approve_payroll".into(),
                "post_payroll".into(),
            ],
            attributes: HashMap::from([
                ("payroll_id".into(), AttributeType::String),
                ("period".into(), AttributeType::String),
                ("total_gross".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Payroll Line Item object type.
    pub fn payroll_line_item() -> Self {
        Self {
            type_id: "payroll_line_item".into(),
            name: "Payroll Line Item".into(),
            business_process: BusinessProcess::H2R,
            lifecycle_states: vec![
                ObjectLifecycleState::new("created", "Created", true, false),
                ObjectLifecycleState::new("processed", "Processed", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "belongs_to",
                "Belongs To",
                "payroll_run",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["calculate_payroll".into()],
            attributes: HashMap::from([
                ("employee_id".into(), AttributeType::String),
                ("gross_amount".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Time Entry object type.
    pub fn time_entry() -> Self {
        Self {
            type_id: "time_entry".into(),
            name: "Time Entry".into(),
            business_process: BusinessProcess::H2R,
            lifecycle_states: vec![
                ObjectLifecycleState::new("pending", "Pending", true, false),
                ObjectLifecycleState::new("approved", "Approved", false, true),
                ObjectLifecycleState::new("rejected", "Rejected", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["submit_time_entry".into(), "approve_time_entry".into()],
            attributes: HashMap::from([
                ("entry_id".into(), AttributeType::String),
                ("employee_id".into(), AttributeType::String),
                ("hours".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create an Expense Report object type.
    pub fn expense_report() -> Self {
        Self {
            type_id: "expense_report".into(),
            name: "Expense Report".into(),
            business_process: BusinessProcess::H2R,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("submitted", "Submitted", false, false),
                ObjectLifecycleState::new("approved", "Approved", false, true),
                ObjectLifecycleState::new("rejected", "Rejected", false, true),
                ObjectLifecycleState::new("paid", "Paid", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["submit_expense".into(), "approve_expense".into()],
            attributes: HashMap::from([
                ("report_id".into(), AttributeType::String),
                ("employee_id".into(), AttributeType::String),
                ("total_amount".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Get all H2R object types.
    pub fn h2r_types() -> Vec<Self> {
        vec![
            Self::payroll_run(),
            Self::payroll_line_item(),
            Self::time_entry(),
            Self::expense_report(),
        ]
    }

    // ========== MFG Object Types ==========

    /// Create a Production Order object type.
    pub fn production_order() -> Self {
        Self {
            type_id: "production_order".into(),
            name: "Production Order".into(),
            business_process: BusinessProcess::Mfg,
            lifecycle_states: vec![
                ObjectLifecycleState::new("planned", "Planned", true, false),
                ObjectLifecycleState::new("released", "Released", false, false),
                ObjectLifecycleState::new("in_process", "In Process", false, false),
                ObjectLifecycleState::new("confirmed", "Confirmed", false, false),
                ObjectLifecycleState::new("closed", "Closed", false, true),
                ObjectLifecycleState::new("cancelled", "Cancelled", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "has_operations",
                "Has Operations",
                "routing_operation",
                Cardinality::OneToMany,
                false,
            )],
            allowed_activities: vec![
                "create_production_order".into(),
                "release_production_order".into(),
                "start_operation".into(),
                "confirm_production".into(),
                "close_production_order".into(),
            ],
            attributes: HashMap::from([
                ("order_id".into(), AttributeType::String),
                ("material_id".into(), AttributeType::String),
                ("quantity".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Routing Operation object type.
    pub fn routing_operation() -> Self {
        Self {
            type_id: "routing_operation".into(),
            name: "Routing Operation".into(),
            business_process: BusinessProcess::Mfg,
            lifecycle_states: vec![
                ObjectLifecycleState::new("pending", "Pending", true, false),
                ObjectLifecycleState::new("in_process", "In Process", false, false),
                ObjectLifecycleState::new("completed", "Completed", false, true),
                ObjectLifecycleState::new("cancelled", "Cancelled", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "belongs_to",
                "Belongs To",
                "production_order",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["start_operation".into(), "complete_operation".into()],
            attributes: HashMap::from([
                ("operation_id".into(), AttributeType::String),
                ("operation_number".into(), AttributeType::Integer),
                ("work_center".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Quality Inspection object type.
    pub fn quality_inspection() -> Self {
        Self {
            type_id: "quality_inspection".into(),
            name: "Quality Inspection".into(),
            business_process: BusinessProcess::Mfg,
            lifecycle_states: vec![
                ObjectLifecycleState::new("pending", "Pending", true, false),
                ObjectLifecycleState::new("completed", "Completed", false, false),
                ObjectLifecycleState::new("accepted", "Accepted", false, true),
                ObjectLifecycleState::new("rejected", "Rejected", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "inspects",
                "Inspects",
                "production_order",
                Cardinality::ManyToOne,
                false,
            )],
            allowed_activities: vec![
                "create_quality_inspection".into(),
                "record_inspection_result".into(),
            ],
            attributes: HashMap::from([
                ("inspection_id".into(), AttributeType::String),
                ("lot_id".into(), AttributeType::String),
                ("result".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Cycle Count object type.
    pub fn cycle_count() -> Self {
        Self {
            type_id: "cycle_count".into(),
            name: "Cycle Count".into(),
            business_process: BusinessProcess::Mfg,
            lifecycle_states: vec![
                ObjectLifecycleState::new("planned", "Planned", true, false),
                ObjectLifecycleState::new("in_progress", "In Progress", false, false),
                ObjectLifecycleState::new("counted", "Counted", false, false),
                ObjectLifecycleState::new("reconciled", "Reconciled", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["start_cycle_count".into(), "reconcile_cycle_count".into()],
            attributes: HashMap::from([
                ("count_id".into(), AttributeType::String),
                ("warehouse".into(), AttributeType::String),
                ("variance".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Get all MFG object types.
    pub fn mfg_types() -> Vec<Self> {
        vec![
            Self::production_order(),
            Self::routing_operation(),
            Self::quality_inspection(),
            Self::cycle_count(),
        ]
    }

    // ========== BANK Object Types ==========

    /// Create a Banking Customer object type.
    pub fn banking_customer() -> Self {
        Self {
            type_id: "banking_customer".into(),
            name: "Banking Customer".into(),
            business_process: BusinessProcess::Bank,
            lifecycle_states: vec![
                ObjectLifecycleState::new("onboarding", "Onboarding", true, false),
                ObjectLifecycleState::new("active", "Active", false, false),
                ObjectLifecycleState::new("frozen", "Frozen", false, false),
                ObjectLifecycleState::new("closed", "Closed", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "owns",
                "Owns",
                "bank_account",
                Cardinality::OneToMany,
                false,
            )],
            allowed_activities: vec!["onboard_customer".into(), "perform_kyc_review".into()],
            attributes: HashMap::from([
                ("customer_id".into(), AttributeType::String),
                ("kyc_status".into(), AttributeType::String),
                ("risk_rating".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Bank Account object type.
    pub fn bank_account() -> Self {
        Self {
            type_id: "bank_account".into(),
            name: "Bank Account".into(),
            business_process: BusinessProcess::Bank,
            lifecycle_states: vec![
                ObjectLifecycleState::new("active", "Active", true, false),
                ObjectLifecycleState::new("frozen", "Frozen", false, false),
                ObjectLifecycleState::new("closed", "Closed", false, true),
                ObjectLifecycleState::new("dormant", "Dormant", false, false),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "owned_by",
                    "Owned By",
                    "banking_customer",
                    Cardinality::ManyToOne,
                    true,
                ),
                ObjectRelationshipType::new(
                    "has_transactions",
                    "Has Transactions",
                    "bank_transaction",
                    Cardinality::OneToMany,
                    false,
                ),
            ],
            allowed_activities: vec!["open_account".into(), "freeze_account".into()],
            attributes: HashMap::from([
                ("account_id".into(), AttributeType::String),
                ("account_type".into(), AttributeType::String),
                ("balance".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Bank Transaction object type.
    pub fn bank_transaction() -> Self {
        Self {
            type_id: "bank_transaction".into(),
            name: "Bank Transaction".into(),
            business_process: BusinessProcess::Bank,
            lifecycle_states: vec![
                ObjectLifecycleState::new("pending", "Pending", true, false),
                ObjectLifecycleState::new("authorized", "Authorized", false, false),
                ObjectLifecycleState::new("completed", "Completed", false, true),
                ObjectLifecycleState::new("failed", "Failed", false, true),
                ObjectLifecycleState::new("reversed", "Reversed", false, true),
                ObjectLifecycleState::new("flagged", "Flagged", false, false),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "on_account",
                "On Account",
                "bank_account",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec![
                "execute_transaction".into(),
                "authorize_transaction".into(),
                "complete_transaction".into(),
                "flag_suspicious".into(),
            ],
            attributes: HashMap::from([
                ("transaction_id".into(), AttributeType::String),
                ("amount".into(), AttributeType::Decimal),
                ("transaction_type".into(), AttributeType::String),
            ]),
        }
    }

    /// Get all BANK object types.
    pub fn bank_types() -> Vec<Self> {
        vec![
            Self::banking_customer(),
            Self::bank_account(),
            Self::bank_transaction(),
        ]
    }

    // ========== AUDIT Object Types ==========

    /// Create an Audit Engagement object type.
    pub fn audit_engagement() -> Self {
        Self {
            type_id: "audit_engagement".into(),
            name: "Audit Engagement".into(),
            business_process: BusinessProcess::Audit,
            lifecycle_states: vec![
                ObjectLifecycleState::new("planning", "Planning", true, false),
                ObjectLifecycleState::new("in_progress", "In Progress", false, false),
                ObjectLifecycleState::new("under_review", "Under Review", false, false),
                ObjectLifecycleState::new("complete", "Complete", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "has_workpapers",
                    "Has Workpapers",
                    "workpaper",
                    Cardinality::OneToMany,
                    false,
                ),
                ObjectRelationshipType::new(
                    "has_findings",
                    "Has Findings",
                    "audit_finding",
                    Cardinality::OneToMany,
                    false,
                ),
            ],
            allowed_activities: vec![
                "create_engagement".into(),
                "plan_engagement".into(),
                "complete_engagement".into(),
            ],
            attributes: HashMap::from([
                ("engagement_id".into(), AttributeType::String),
                ("engagement_type".into(), AttributeType::String),
                ("fiscal_year".into(), AttributeType::Integer),
            ]),
        }
    }

    /// Create a Workpaper object type.
    pub fn workpaper() -> Self {
        Self {
            type_id: "workpaper".into(),
            name: "Workpaper".into(),
            business_process: BusinessProcess::Audit,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("reviewed", "Reviewed", false, false),
                ObjectLifecycleState::new("complete", "Complete", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "belongs_to",
                    "Belongs To",
                    "audit_engagement",
                    Cardinality::ManyToOne,
                    true,
                ),
                ObjectRelationshipType::new(
                    "has_evidence",
                    "Has Evidence",
                    "audit_evidence",
                    Cardinality::OneToMany,
                    false,
                ),
            ],
            allowed_activities: vec!["create_workpaper".into(), "review_workpaper".into()],
            attributes: HashMap::from([
                ("workpaper_id".into(), AttributeType::String),
                ("workpaper_type".into(), AttributeType::String),
                ("preparer".into(), AttributeType::String),
            ]),
        }
    }

    /// Create an Audit Finding object type.
    pub fn audit_finding() -> Self {
        Self {
            type_id: "audit_finding".into(),
            name: "Audit Finding".into(),
            business_process: BusinessProcess::Audit,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("pending_review", "Pending Review", false, false),
                ObjectLifecycleState::new("closed", "Closed", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "belongs_to",
                "Belongs To",
                "audit_engagement",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["raise_finding".into(), "remediate_finding".into()],
            attributes: HashMap::from([
                ("finding_id".into(), AttributeType::String),
                ("severity".into(), AttributeType::String),
                ("description".into(), AttributeType::String),
            ]),
        }
    }

    /// Create an Audit Evidence object type.
    pub fn audit_evidence() -> Self {
        Self {
            type_id: "audit_evidence".into(),
            name: "Audit Evidence".into(),
            business_process: BusinessProcess::Audit,
            lifecycle_states: vec![
                ObjectLifecycleState::new("collected", "Collected", true, false),
                ObjectLifecycleState::new("assessed", "Assessed", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "supports",
                "Supports",
                "workpaper",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["collect_evidence".into()],
            attributes: HashMap::from([
                ("evidence_id".into(), AttributeType::String),
                ("evidence_type".into(), AttributeType::String),
                ("reliability".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Risk Assessment object type.
    pub fn risk_assessment() -> Self {
        Self {
            type_id: "risk_assessment".into(),
            name: "Risk Assessment".into(),
            business_process: BusinessProcess::Audit,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("approved", "Approved", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "belongs_to",
                "Belongs To",
                "audit_engagement",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["assess_risk".into()],
            attributes: HashMap::from([
                ("risk_id".into(), AttributeType::String),
                ("risk_level".into(), AttributeType::String),
                ("inherent_risk".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Professional Judgment object type.
    pub fn professional_judgment() -> Self {
        Self {
            type_id: "professional_judgment".into(),
            name: "Professional Judgment".into(),
            business_process: BusinessProcess::Audit,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("approved", "Approved", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["record_judgment".into()],
            attributes: HashMap::from([
                ("judgment_id".into(), AttributeType::String),
                ("topic".into(), AttributeType::String),
                ("conclusion".into(), AttributeType::String),
            ]),
        }
    }

    /// Get all AUDIT object types.
    pub fn audit_types() -> Vec<Self> {
        vec![
            Self::audit_engagement(),
            Self::workpaper(),
            Self::audit_finding(),
            Self::audit_evidence(),
            Self::risk_assessment(),
            Self::professional_judgment(),
        ]
    }

    // ========== Bank Reconciliation Object Types (R2R subfamily) ==========

    /// Create a Bank Reconciliation object type.
    pub fn bank_reconciliation() -> Self {
        Self {
            type_id: "bank_reconciliation".into(),
            name: "Bank Reconciliation".into(),
            business_process: BusinessProcess::R2R,
            lifecycle_states: vec![
                ObjectLifecycleState::new("in_progress", "In Progress", true, false),
                ObjectLifecycleState::new("approved", "Approved", false, false),
                ObjectLifecycleState::new("posted", "Posted", false, false),
                ObjectLifecycleState::new("completed", "Completed", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "has_lines",
                "Has Lines",
                "bank_statement_line",
                Cardinality::OneToMany,
                false,
            )],
            allowed_activities: vec![
                "import_bank_statement".into(),
                "approve_reconciliation".into(),
                "post_recon_entries".into(),
                "complete_reconciliation".into(),
            ],
            attributes: HashMap::from([
                ("reconciliation_id".into(), AttributeType::String),
                ("bank_account_id".into(), AttributeType::String),
                ("period".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Bank Statement Line object type.
    pub fn bank_statement_line() -> Self {
        Self {
            type_id: "bank_statement_line".into(),
            name: "Bank Statement Line".into(),
            business_process: BusinessProcess::R2R,
            lifecycle_states: vec![
                ObjectLifecycleState::new("unmatched", "Unmatched", true, false),
                ObjectLifecycleState::new("auto_matched", "Auto Matched", false, true),
                ObjectLifecycleState::new("manually_matched", "Manually Matched", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "belongs_to",
                "Belongs To",
                "bank_reconciliation",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["auto_match_items".into(), "manual_match_item".into()],
            attributes: HashMap::from([
                ("line_id".into(), AttributeType::String),
                ("amount".into(), AttributeType::Decimal),
                ("description".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Reconciling Item object type.
    pub fn reconciling_item() -> Self {
        Self {
            type_id: "reconciling_item".into(),
            name: "Reconciling Item".into(),
            business_process: BusinessProcess::R2R,
            lifecycle_states: vec![
                ObjectLifecycleState::new("outstanding", "Outstanding", true, false),
                ObjectLifecycleState::new("resolved", "Resolved", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "belongs_to",
                "Belongs To",
                "bank_reconciliation",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["create_reconciling_item".into(), "resolve_exception".into()],
            attributes: HashMap::from([
                ("item_id".into(), AttributeType::String),
                ("amount".into(), AttributeType::Decimal),
                ("item_type".into(), AttributeType::String),
            ]),
        }
    }

    /// Get all Bank Reconciliation object types.
    pub fn bank_recon_types() -> Vec<Self> {
        vec![
            Self::bank_reconciliation(),
            Self::bank_statement_line(),
            Self::reconciling_item(),
        ]
    }

    // ========== TAX Object Types ==========

    /// Create a Tax Line object type.
    pub fn tax_line() -> Self {
        Self {
            type_id: "tax_line".into(),
            name: "Tax Line".into(),
            business_process: BusinessProcess::Tax,
            lifecycle_states: vec![
                ObjectLifecycleState::new("created", "Created", true, false),
                ObjectLifecycleState::new("calculated", "Calculated", false, false),
                ObjectLifecycleState::new("posted", "Posted", false, true),
                ObjectLifecycleState::new("reversed", "Reversed", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "belongs_to",
                "Belongs To",
                "tax_return",
                Cardinality::ManyToOne,
                false,
            )],
            allowed_activities: vec![
                "tax_determination".into(),
                "tax_line_created".into(),
            ],
            attributes: HashMap::from([
                ("tax_line_id".into(), AttributeType::String),
                ("tax_code".into(), AttributeType::String),
                ("tax_amount".into(), AttributeType::Decimal),
                ("jurisdiction".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Tax Return object type.
    pub fn tax_return() -> Self {
        Self {
            type_id: "tax_return".into(),
            name: "Tax Return".into(),
            business_process: BusinessProcess::Tax,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("prepared", "Prepared", false, false),
                ObjectLifecycleState::new("reviewed", "Reviewed", false, false),
                ObjectLifecycleState::new("filed", "Filed", false, false),
                ObjectLifecycleState::new("assessed", "Assessed", false, false),
                ObjectLifecycleState::new("paid", "Paid", false, true),
                ObjectLifecycleState::new("amended", "Amended", false, false),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "has_lines",
                "Has Lines",
                "tax_line",
                Cardinality::OneToMany,
                false,
            )],
            allowed_activities: vec![
                "tax_return_filed".into(),
                "tax_return_assessed".into(),
                "tax_paid".into(),
            ],
            attributes: HashMap::from([
                ("return_id".into(), AttributeType::String),
                ("return_type".into(), AttributeType::String),
                ("period".into(), AttributeType::String),
                ("total_tax".into(), AttributeType::Decimal),
                ("jurisdiction".into(), AttributeType::String),
            ]),
        }
    }

    /// Get all Tax object types.
    pub fn tax_types() -> Vec<Self> {
        vec![Self::tax_line(), Self::tax_return()]
    }

    // ========== TREASURY Object Types ==========

    /// Create a Cash Position object type.
    pub fn cash_position() -> Self {
        Self {
            type_id: "cash_position".into(),
            name: "Cash Position".into(),
            business_process: BusinessProcess::Treasury,
            lifecycle_states: vec![
                ObjectLifecycleState::new("snapshot", "Snapshot", true, false),
                ObjectLifecycleState::new("confirmed", "Confirmed", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["cash_position_calculated".into()],
            attributes: HashMap::from([
                ("position_id".into(), AttributeType::String),
                ("entity_id".into(), AttributeType::String),
                ("currency".into(), AttributeType::String),
                ("balance".into(), AttributeType::Decimal),
                ("as_of_date".into(), AttributeType::Date),
            ]),
        }
    }

    /// Create a Cash Forecast object type.
    pub fn cash_forecast() -> Self {
        Self {
            type_id: "cash_forecast".into(),
            name: "Cash Forecast".into(),
            business_process: BusinessProcess::Treasury,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("approved", "Approved", false, false),
                ObjectLifecycleState::new("actual", "Actual", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["forecast_generated".into()],
            attributes: HashMap::from([
                ("forecast_id".into(), AttributeType::String),
                ("entity_id".into(), AttributeType::String),
                ("horizon_days".into(), AttributeType::Integer),
                ("net_flow".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Hedge Instrument object type.
    pub fn hedge_instrument() -> Self {
        Self {
            type_id: "hedge_instrument".into(),
            name: "Hedge Instrument".into(),
            business_process: BusinessProcess::Treasury,
            lifecycle_states: vec![
                ObjectLifecycleState::new("designated", "Designated", true, false),
                ObjectLifecycleState::new("effective", "Effective", false, false),
                ObjectLifecycleState::new("matured", "Matured", false, true),
                ObjectLifecycleState::new("dedesignated", "De-designated", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["hedge_designated".into()],
            attributes: HashMap::from([
                ("hedge_id".into(), AttributeType::String),
                ("hedge_type".into(), AttributeType::String),
                ("notional_amount".into(), AttributeType::Decimal),
                ("currency_pair".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Debt Instrument object type.
    pub fn debt_instrument() -> Self {
        Self {
            type_id: "debt_instrument".into(),
            name: "Debt Instrument".into(),
            business_process: BusinessProcess::Treasury,
            lifecycle_states: vec![
                ObjectLifecycleState::new("active", "Active", true, false),
                ObjectLifecycleState::new("drawn", "Drawn", false, false),
                ObjectLifecycleState::new("repaid", "Repaid", false, true),
                ObjectLifecycleState::new("matured", "Matured", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["covenant_measured".into()],
            attributes: HashMap::from([
                ("instrument_id".into(), AttributeType::String),
                ("instrument_type".into(), AttributeType::String),
                ("principal".into(), AttributeType::Decimal),
                ("maturity_date".into(), AttributeType::Date),
            ]),
        }
    }

    /// Get all Treasury object types.
    pub fn treasury_types() -> Vec<Self> {
        vec![
            Self::cash_position(),
            Self::cash_forecast(),
            Self::hedge_instrument(),
            Self::debt_instrument(),
        ]
    }

    // ========== PROJECT ACCOUNTING Object Types ==========

    /// Create a Project object type.
    pub fn project() -> Self {
        Self {
            type_id: "project".into(),
            name: "Project".into(),
            business_process: BusinessProcess::ProjectAccounting,
            lifecycle_states: vec![
                ObjectLifecycleState::new("created", "Created", true, false),
                ObjectLifecycleState::new("active", "Active", false, false),
                ObjectLifecycleState::new("on_hold", "On Hold", false, false),
                ObjectLifecycleState::new("completed", "Completed", false, true),
                ObjectLifecycleState::new("closed", "Closed", false, true),
            ],
            relationships: vec![
                ObjectRelationshipType::new(
                    "has_costs",
                    "Has Costs",
                    "project_cost_line",
                    Cardinality::OneToMany,
                    false,
                ),
                ObjectRelationshipType::new(
                    "has_milestones",
                    "Has Milestones",
                    "project_milestone",
                    Cardinality::OneToMany,
                    false,
                ),
                ObjectRelationshipType::new(
                    "has_change_orders",
                    "Has Change Orders",
                    "change_order",
                    Cardinality::OneToMany,
                    false,
                ),
            ],
            allowed_activities: vec![
                "project_created".into(),
                "revenue_recognized".into(),
            ],
            attributes: HashMap::from([
                ("project_id".into(), AttributeType::String),
                ("project_name".into(), AttributeType::String),
                ("budget".into(), AttributeType::Decimal),
                ("method".into(), AttributeType::String),
            ]),
        }
    }

    /// Create a Project Cost Line object type.
    pub fn project_cost_line() -> Self {
        Self {
            type_id: "project_cost_line".into(),
            name: "Project Cost Line".into(),
            business_process: BusinessProcess::ProjectAccounting,
            lifecycle_states: vec![
                ObjectLifecycleState::new("posted", "Posted", true, false),
                ObjectLifecycleState::new("allocated", "Allocated", false, true),
                ObjectLifecycleState::new("reversed", "Reversed", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "belongs_to",
                "Belongs To",
                "project",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["cost_posted".into()],
            attributes: HashMap::from([
                ("cost_line_id".into(), AttributeType::String),
                ("cost_category".into(), AttributeType::String),
                ("amount".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Project Milestone object type.
    pub fn project_milestone() -> Self {
        Self {
            type_id: "project_milestone".into(),
            name: "Project Milestone".into(),
            business_process: BusinessProcess::ProjectAccounting,
            lifecycle_states: vec![
                ObjectLifecycleState::new("pending", "Pending", true, false),
                ObjectLifecycleState::new("achieved", "Achieved", false, true),
                ObjectLifecycleState::new("cancelled", "Cancelled", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "belongs_to",
                "Belongs To",
                "project",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["milestone_achieved".into()],
            attributes: HashMap::from([
                ("milestone_id".into(), AttributeType::String),
                ("milestone_name".into(), AttributeType::String),
                ("completion_pct".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Create a Change Order object type.
    pub fn change_order() -> Self {
        Self {
            type_id: "change_order".into(),
            name: "Change Order".into(),
            business_process: BusinessProcess::ProjectAccounting,
            lifecycle_states: vec![
                ObjectLifecycleState::new("requested", "Requested", true, false),
                ObjectLifecycleState::new("approved", "Approved", false, false),
                ObjectLifecycleState::new("applied", "Applied", false, true),
                ObjectLifecycleState::new("rejected", "Rejected", false, true),
            ],
            relationships: vec![ObjectRelationshipType::new(
                "modifies",
                "Modifies",
                "project",
                Cardinality::ManyToOne,
                true,
            )],
            allowed_activities: vec!["change_order_processed".into()],
            attributes: HashMap::from([
                ("change_order_id".into(), AttributeType::String),
                ("scope_change".into(), AttributeType::String),
                ("cost_impact".into(), AttributeType::Decimal),
            ]),
        }
    }

    /// Get all Project Accounting object types.
    pub fn project_accounting_types() -> Vec<Self> {
        vec![
            Self::project(),
            Self::project_cost_line(),
            Self::project_milestone(),
            Self::change_order(),
        ]
    }

    // ========== ESG Object Types ==========

    /// Create an ESG Data Point object type.
    pub fn esg_data_point() -> Self {
        Self {
            type_id: "esg_data_point".into(),
            name: "ESG Data Point".into(),
            business_process: BusinessProcess::Esg,
            lifecycle_states: vec![
                ObjectLifecycleState::new("collected", "Collected", true, false),
                ObjectLifecycleState::new("validated", "Validated", false, false),
                ObjectLifecycleState::new("reported", "Reported", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["esg_data_collected".into()],
            attributes: HashMap::from([
                ("data_point_id".into(), AttributeType::String),
                ("metric_name".into(), AttributeType::String),
                ("value".into(), AttributeType::Decimal),
                ("unit".into(), AttributeType::String),
                ("period".into(), AttributeType::Date),
            ]),
        }
    }

    /// Create an Emission Record object type.
    pub fn emission_record() -> Self {
        Self {
            type_id: "emission_record".into(),
            name: "Emission Record".into(),
            business_process: BusinessProcess::Esg,
            lifecycle_states: vec![
                ObjectLifecycleState::new("calculated", "Calculated", true, false),
                ObjectLifecycleState::new("verified", "Verified", false, false),
                ObjectLifecycleState::new("assured", "Assured", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec![
                "emission_calculated".into(),
                "assurance_completed".into(),
            ],
            attributes: HashMap::from([
                ("record_id".into(), AttributeType::String),
                ("scope".into(), AttributeType::String),
                ("co2e_tonnes".into(), AttributeType::Decimal),
                ("source".into(), AttributeType::String),
            ]),
        }
    }

    /// Create an ESG Disclosure object type.
    pub fn esg_disclosure() -> Self {
        Self {
            type_id: "esg_disclosure".into(),
            name: "ESG Disclosure".into(),
            business_process: BusinessProcess::Esg,
            lifecycle_states: vec![
                ObjectLifecycleState::new("draft", "Draft", true, false),
                ObjectLifecycleState::new("reviewed", "Reviewed", false, false),
                ObjectLifecycleState::new("published", "Published", false, true),
            ],
            relationships: vec![],
            allowed_activities: vec!["disclosure_prepared".into()],
            attributes: HashMap::from([
                ("disclosure_id".into(), AttributeType::String),
                ("framework".into(), AttributeType::String),
                ("topic".into(), AttributeType::String),
                ("standard_id".into(), AttributeType::String),
            ]),
        }
    }

    /// Get all ESG object types.
    pub fn esg_types() -> Vec<Self> {
        vec![
            Self::esg_data_point(),
            Self::emission_record(),
            Self::esg_disclosure(),
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

    #[test]
    fn test_s2c_types() {
        let types = ObjectType::s2c_types();
        assert_eq!(types.len(), 6);
        assert_eq!(types[0].type_id, "sourcing_project");
        assert_eq!(types[0].business_process, BusinessProcess::S2C);
    }

    #[test]
    fn test_h2r_types() {
        let types = ObjectType::h2r_types();
        assert_eq!(types.len(), 4);
        assert_eq!(types[0].type_id, "payroll_run");
    }

    #[test]
    fn test_mfg_types() {
        let types = ObjectType::mfg_types();
        assert_eq!(types.len(), 4);
        assert_eq!(types[0].type_id, "production_order");
    }

    #[test]
    fn test_bank_types() {
        let types = ObjectType::bank_types();
        assert_eq!(types.len(), 3);
        assert_eq!(types[0].type_id, "banking_customer");
    }

    #[test]
    fn test_audit_types() {
        let types = ObjectType::audit_types();
        assert_eq!(types.len(), 6);
        assert_eq!(types[0].type_id, "audit_engagement");
    }

    #[test]
    fn test_bank_recon_types() {
        let types = ObjectType::bank_recon_types();
        assert_eq!(types.len(), 3);
        assert_eq!(types[0].type_id, "bank_reconciliation");
    }

    #[test]
    fn test_tax_types() {
        let types = ObjectType::tax_types();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].type_id, "tax_line");
        assert_eq!(types[0].business_process, BusinessProcess::Tax);
    }

    #[test]
    fn test_treasury_types() {
        let types = ObjectType::treasury_types();
        assert_eq!(types.len(), 4);
        assert_eq!(types[0].type_id, "cash_position");
        assert_eq!(types[0].business_process, BusinessProcess::Treasury);
    }

    #[test]
    fn test_project_accounting_types() {
        let types = ObjectType::project_accounting_types();
        assert_eq!(types.len(), 4);
        assert_eq!(types[0].type_id, "project");
        assert_eq!(types[0].business_process, BusinessProcess::ProjectAccounting);
    }

    #[test]
    fn test_esg_types() {
        let types = ObjectType::esg_types();
        assert_eq!(types.len(), 3);
        assert_eq!(types[0].type_id, "esg_data_point");
        assert_eq!(types[0].business_process, BusinessProcess::Esg);
    }
}
