//! Reference Process Model export for process mining validation.
//!
//! Reference models define the canonical/expected process flows for P2P, O2C,
//! and R2R business processes. These models can be used to:
//!
//! - Validate generated event logs against expected patterns
//! - Train conformance checking algorithms
//! - Compare actual vs. expected process behavior
//! - Identify process deviations and anomalies

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use datasynth_core::models::BusinessProcess;
use serde::{Deserialize, Serialize};

/// A reference process model defining the canonical flow for a business process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceProcessModel {
    /// Unique process identifier
    pub process_id: String,
    /// Human-readable process name
    pub process_name: String,
    /// Business process type
    pub business_process: BusinessProcess,
    /// Process description
    pub description: String,
    /// Version of this reference model
    pub version: String,
    /// Activities in this process
    pub activities: Vec<ReferenceActivity>,
    /// Valid transitions between activities
    pub transitions: Vec<ReferenceTransition>,
    /// Known process variants
    pub variants: Vec<ReferenceVariant>,
}

/// An activity in the reference process model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceActivity {
    /// Activity identifier
    pub activity_id: String,
    /// Human-readable activity name
    pub name: String,
    /// Whether this activity is required in the standard flow
    pub is_required: bool,
    /// Whether this is a start activity
    pub is_start: bool,
    /// Whether this is an end activity
    pub is_end: bool,
    /// Typical duration in minutes
    pub typical_duration_minutes: Option<f64>,
    /// Standard deviation of duration
    pub duration_std_dev: Option<f64>,
    /// Whether this activity is automated
    pub is_automated: bool,
    /// Object types involved in this activity
    pub involved_object_types: Vec<String>,
}

/// A valid transition between activities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceTransition {
    /// Source activity ID
    pub from_activity: String,
    /// Target activity ID
    pub to_activity: String,
    /// Whether this transition is part of the standard/happy path
    pub is_standard_path: bool,
    /// Probability of this transition (0.0-1.0)
    pub probability: Option<f64>,
    /// Condition for this transition (if any)
    pub condition: Option<String>,
    /// Description of when this transition occurs
    pub description: Option<String>,
}

/// A known process variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceVariant {
    /// Variant identifier
    pub variant_id: String,
    /// Variant name
    pub name: String,
    /// Sequence of activity IDs
    pub activity_sequence: Vec<String>,
    /// Whether this is the standard/happy path variant
    pub is_standard: bool,
    /// Expected frequency of this variant (0.0-1.0)
    pub expected_frequency: Option<f64>,
    /// Description of this variant
    pub description: Option<String>,
}

impl ReferenceProcessModel {
    /// Create a new reference process model.
    pub fn new(
        process_id: &str,
        process_name: &str,
        business_process: BusinessProcess,
        description: &str,
    ) -> Self {
        Self {
            process_id: process_id.into(),
            process_name: process_name.into(),
            business_process,
            description: description.into(),
            version: "1.0".into(),
            activities: Vec::new(),
            transitions: Vec::new(),
            variants: Vec::new(),
        }
    }

    /// Add an activity to the model.
    pub fn add_activity(&mut self, activity: ReferenceActivity) {
        self.activities.push(activity);
    }

    /// Add a transition to the model.
    pub fn add_transition(&mut self, transition: ReferenceTransition) {
        self.transitions.push(transition);
    }

    /// Add a variant to the model.
    pub fn add_variant(&mut self, variant: ReferenceVariant) {
        self.variants.push(variant);
    }

    /// Export the model to a JSON file.
    pub fn export_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Export the model to a JSON string.
    pub fn export_to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    // ========== Standard Reference Models ==========

    /// Create the standard P2P (Procure-to-Pay) reference model.
    pub fn p2p_standard() -> Self {
        let mut model = Self::new(
            "P2P_STANDARD",
            "Procure-to-Pay Standard Process",
            BusinessProcess::P2P,
            "Standard procure-to-pay process from purchase requisition through payment.",
        );

        // Activities
        model.add_activity(ReferenceActivity {
            activity_id: "create_po".into(),
            name: "Create Purchase Order".into(),
            is_required: true,
            is_start: true,
            is_end: false,
            typical_duration_minutes: Some(15.0),
            duration_std_dev: Some(5.0),
            is_automated: false,
            involved_object_types: vec!["purchase_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "approve_po".into(),
            name: "Approve Purchase Order".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(30.0),
            duration_std_dev: Some(15.0),
            is_automated: false,
            involved_object_types: vec!["purchase_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "release_po".into(),
            name: "Release Purchase Order".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(5.0),
            duration_std_dev: Some(2.0),
            is_automated: true,
            involved_object_types: vec!["purchase_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "create_gr".into(),
            name: "Create Goods Receipt".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(10.0),
            duration_std_dev: Some(5.0),
            is_automated: false,
            involved_object_types: vec!["goods_receipt".into(), "purchase_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "post_gr".into(),
            name: "Post Goods Receipt".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(2.0),
            duration_std_dev: Some(1.0),
            is_automated: true,
            involved_object_types: vec!["goods_receipt".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "receive_invoice".into(),
            name: "Receive Invoice".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(5.0),
            duration_std_dev: Some(2.0),
            is_automated: false,
            involved_object_types: vec!["vendor_invoice".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "verify_invoice".into(),
            name: "Verify Invoice (3-Way Match)".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(20.0),
            duration_std_dev: Some(10.0),
            is_automated: false,
            involved_object_types: vec![
                "vendor_invoice".into(),
                "purchase_order".into(),
                "goods_receipt".into(),
            ],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "post_invoice".into(),
            name: "Post Invoice".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(3.0),
            duration_std_dev: Some(1.0),
            is_automated: true,
            involved_object_types: vec!["vendor_invoice".into(), "purchase_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "execute_payment".into(),
            name: "Execute Payment".into(),
            is_required: true,
            is_start: false,
            is_end: true,
            typical_duration_minutes: Some(1.0),
            duration_std_dev: Some(0.5),
            is_automated: true,
            involved_object_types: vec!["vendor_invoice".into(), "purchase_order".into()],
        });

        // Standard path transitions
        model.add_transition(ReferenceTransition {
            from_activity: "create_po".into(),
            to_activity: "approve_po".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("All POs require approval".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "approve_po".into(),
            to_activity: "release_po".into(),
            is_standard_path: true,
            probability: Some(0.95),
            condition: Some("PO approved".into()),
            description: Some("Approved POs are released to vendor".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "approve_po".into(),
            to_activity: "create_po".into(),
            is_standard_path: false,
            probability: Some(0.05),
            condition: Some("PO rejected".into()),
            description: Some("Rejected POs may be revised and resubmitted".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "release_po".into(),
            to_activity: "create_gr".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: Some("Goods received".into()),
            description: Some("Goods receipt created when goods arrive".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "create_gr".into(),
            to_activity: "post_gr".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("GR posted to inventory".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "post_gr".into(),
            to_activity: "receive_invoice".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Invoice received from vendor".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "receive_invoice".into(),
            to_activity: "verify_invoice".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Invoice verified against PO and GR".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "verify_invoice".into(),
            to_activity: "post_invoice".into(),
            is_standard_path: true,
            probability: Some(0.90),
            condition: Some("3-way match successful".into()),
            description: Some("Matched invoices are posted".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "verify_invoice".into(),
            to_activity: "receive_invoice".into(),
            is_standard_path: false,
            probability: Some(0.10),
            condition: Some("Match exception".into()),
            description: Some("Invoices with discrepancies may be corrected".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "post_invoice".into(),
            to_activity: "execute_payment".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: Some("Payment due date reached".into()),
            description: Some("Payment executed on due date".into()),
        });

        // Standard variant
        model.add_variant(ReferenceVariant {
            variant_id: "P2P_HAPPY".into(),
            name: "Standard P2P Flow".into(),
            activity_sequence: vec![
                "create_po".into(),
                "approve_po".into(),
                "release_po".into(),
                "create_gr".into(),
                "post_gr".into(),
                "receive_invoice".into(),
                "verify_invoice".into(),
                "post_invoice".into(),
                "execute_payment".into(),
            ],
            is_standard: true,
            expected_frequency: Some(0.75),
            description: Some("Standard procure-to-pay without exceptions".into()),
        });

        model.add_variant(ReferenceVariant {
            variant_id: "P2P_REWORK".into(),
            name: "P2P with Approval Rework".into(),
            activity_sequence: vec![
                "create_po".into(),
                "approve_po".into(),
                "create_po".into(), // Rework
                "approve_po".into(),
                "release_po".into(),
                "create_gr".into(),
                "post_gr".into(),
                "receive_invoice".into(),
                "verify_invoice".into(),
                "post_invoice".into(),
                "execute_payment".into(),
            ],
            is_standard: false,
            expected_frequency: Some(0.05),
            description: Some("P2P with PO rejection and revision".into()),
        });

        model.add_variant(ReferenceVariant {
            variant_id: "P2P_MATCH_EXCEPTION".into(),
            name: "P2P with Match Exception".into(),
            activity_sequence: vec![
                "create_po".into(),
                "approve_po".into(),
                "release_po".into(),
                "create_gr".into(),
                "post_gr".into(),
                "receive_invoice".into(),
                "verify_invoice".into(),
                "receive_invoice".into(), // Corrected invoice
                "verify_invoice".into(),
                "post_invoice".into(),
                "execute_payment".into(),
            ],
            is_standard: false,
            expected_frequency: Some(0.10),
            description: Some("P2P with invoice matching exception".into()),
        });

        model
    }

    /// Create the standard O2C (Order-to-Cash) reference model.
    pub fn o2c_standard() -> Self {
        let mut model = Self::new(
            "O2C_STANDARD",
            "Order-to-Cash Standard Process",
            BusinessProcess::O2C,
            "Standard order-to-cash process from sales order through customer payment.",
        );

        // Activities
        model.add_activity(ReferenceActivity {
            activity_id: "create_so".into(),
            name: "Create Sales Order".into(),
            is_required: true,
            is_start: true,
            is_end: false,
            typical_duration_minutes: Some(10.0),
            duration_std_dev: Some(5.0),
            is_automated: false,
            involved_object_types: vec!["sales_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "check_credit".into(),
            name: "Check Credit".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(2.0),
            duration_std_dev: Some(1.0),
            is_automated: true,
            involved_object_types: vec!["sales_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "release_so".into(),
            name: "Release Sales Order".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(5.0),
            duration_std_dev: Some(2.0),
            is_automated: false,
            involved_object_types: vec!["sales_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "create_delivery".into(),
            name: "Create Delivery".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(5.0),
            duration_std_dev: Some(2.0),
            is_automated: false,
            involved_object_types: vec!["delivery".into(), "sales_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "pick".into(),
            name: "Pick".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(30.0),
            duration_std_dev: Some(15.0),
            is_automated: false,
            involved_object_types: vec!["delivery".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "pack".into(),
            name: "Pack".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(20.0),
            duration_std_dev: Some(10.0),
            is_automated: false,
            involved_object_types: vec!["delivery".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "ship".into(),
            name: "Ship".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(10.0),
            duration_std_dev: Some(5.0),
            is_automated: false,
            involved_object_types: vec!["delivery".into(), "sales_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "create_customer_invoice".into(),
            name: "Create Customer Invoice".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(5.0),
            duration_std_dev: Some(2.0),
            is_automated: false,
            involved_object_types: vec!["customer_invoice".into(), "sales_order".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "post_customer_invoice".into(),
            name: "Post Customer Invoice".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(2.0),
            duration_std_dev: Some(1.0),
            is_automated: true,
            involved_object_types: vec!["customer_invoice".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "receive_payment".into(),
            name: "Receive Payment".into(),
            is_required: true,
            is_start: false,
            is_end: true,
            typical_duration_minutes: Some(1.0),
            duration_std_dev: Some(0.5),
            is_automated: true,
            involved_object_types: vec!["customer_invoice".into(), "sales_order".into()],
        });

        // Transitions
        model.add_transition(ReferenceTransition {
            from_activity: "create_so".into(),
            to_activity: "check_credit".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("All orders go through credit check".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "check_credit".into(),
            to_activity: "release_so".into(),
            is_standard_path: true,
            probability: Some(0.95),
            condition: Some("Credit approved".into()),
            description: Some("Orders with approved credit are released".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "check_credit".into(),
            to_activity: "create_so".into(),
            is_standard_path: false,
            probability: Some(0.05),
            condition: Some("Credit blocked".into()),
            description: Some("Blocked orders may be revised".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "release_so".into(),
            to_activity: "create_delivery".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Delivery created for released orders".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "create_delivery".into(),
            to_activity: "pick".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Items picked from warehouse".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "pick".into(),
            to_activity: "pack".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Items packed for shipping".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "pack".into(),
            to_activity: "ship".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Goods shipped to customer".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "ship".into(),
            to_activity: "create_customer_invoice".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Invoice created after shipment".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "create_customer_invoice".into(),
            to_activity: "post_customer_invoice".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Invoice posted to AR".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "post_customer_invoice".into(),
            to_activity: "receive_payment".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: Some("Payment received".into()),
            description: Some("Customer payment received".into()),
        });

        // Variants
        model.add_variant(ReferenceVariant {
            variant_id: "O2C_HAPPY".into(),
            name: "Standard O2C Flow".into(),
            activity_sequence: vec![
                "create_so".into(),
                "check_credit".into(),
                "release_so".into(),
                "create_delivery".into(),
                "pick".into(),
                "pack".into(),
                "ship".into(),
                "create_customer_invoice".into(),
                "post_customer_invoice".into(),
                "receive_payment".into(),
            ],
            is_standard: true,
            expected_frequency: Some(0.75),
            description: Some("Standard order-to-cash without exceptions".into()),
        });

        model.add_variant(ReferenceVariant {
            variant_id: "O2C_CREDIT_BLOCK".into(),
            name: "O2C with Credit Block".into(),
            activity_sequence: vec![
                "create_so".into(),
                "check_credit".into(),
                "create_so".into(), // Revision
                "check_credit".into(),
                "release_so".into(),
                "create_delivery".into(),
                "pick".into(),
                "pack".into(),
                "ship".into(),
                "create_customer_invoice".into(),
                "post_customer_invoice".into(),
                "receive_payment".into(),
            ],
            is_standard: false,
            expected_frequency: Some(0.05),
            description: Some("O2C with initial credit block".into()),
        });

        model
    }

    /// Create the standard R2R (Record-to-Report) reference model.
    pub fn r2r_standard() -> Self {
        let mut model = Self::new(
            "R2R_STANDARD",
            "Record-to-Report Standard Process",
            BusinessProcess::R2R,
            "Standard record-to-report process for period close and financial reporting.",
        );

        // Activities
        model.add_activity(ReferenceActivity {
            activity_id: "post_je".into(),
            name: "Post Journal Entry".into(),
            is_required: true,
            is_start: true,
            is_end: false,
            typical_duration_minutes: Some(5.0),
            duration_std_dev: Some(2.0),
            is_automated: false,
            involved_object_types: vec!["journal_entry".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "review_je".into(),
            name: "Review Journal Entry".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(15.0),
            duration_std_dev: Some(8.0),
            is_automated: false,
            involved_object_types: vec!["journal_entry".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "approve_je".into(),
            name: "Approve Journal Entry".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(10.0),
            duration_std_dev: Some(5.0),
            is_automated: false,
            involved_object_types: vec!["journal_entry".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "run_depr".into(),
            name: "Run Depreciation".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(60.0),
            duration_std_dev: Some(20.0),
            is_automated: true,
            involved_object_types: vec!["fixed_asset".into(), "journal_entry".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "post_accruals".into(),
            name: "Post Accruals".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(20.0),
            duration_std_dev: Some(10.0),
            is_automated: false,
            involved_object_types: vec!["accrual_entry".into(), "journal_entry".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "fx_reval".into(),
            name: "FX Revaluation".into(),
            is_required: false,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(30.0),
            duration_std_dev: Some(10.0),
            is_automated: true,
            involved_object_types: vec!["journal_entry".into(), "fx_adjustment".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "ic_elim".into(),
            name: "Run IC Elimination".into(),
            is_required: false,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(45.0),
            duration_std_dev: Some(15.0),
            is_automated: true,
            involved_object_types: vec!["ic_transaction".into(), "elimination_entry".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "gen_tb".into(),
            name: "Generate Trial Balance".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(10.0),
            duration_std_dev: Some(5.0),
            is_automated: true,
            involved_object_types: vec!["trial_balance".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "review_tb".into(),
            name: "Review Trial Balance".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(60.0),
            duration_std_dev: Some(30.0),
            is_automated: false,
            involved_object_types: vec!["trial_balance".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "approve_tb".into(),
            name: "Approve Trial Balance".into(),
            is_required: true,
            is_start: false,
            is_end: false,
            typical_duration_minutes: Some(30.0),
            duration_std_dev: Some(15.0),
            is_automated: false,
            involved_object_types: vec!["trial_balance".into()],
        });

        model.add_activity(ReferenceActivity {
            activity_id: "close_period".into(),
            name: "Close Period".into(),
            is_required: true,
            is_start: false,
            is_end: true,
            typical_duration_minutes: Some(30.0),
            duration_std_dev: Some(15.0),
            is_automated: false,
            involved_object_types: vec!["fiscal_period".into()],
        });

        // Transitions - simplified flow
        model.add_transition(ReferenceTransition {
            from_activity: "post_je".into(),
            to_activity: "review_je".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("All JEs require review".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "review_je".into(),
            to_activity: "approve_je".into(),
            is_standard_path: true,
            probability: Some(0.95),
            condition: Some("JE approved".into()),
            description: Some("Reviewed JEs go to approval".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "approve_je".into(),
            to_activity: "run_depr".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: Some("Period end".into()),
            description: Some("Depreciation run at period end".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "run_depr".into(),
            to_activity: "post_accruals".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Accruals posted after depreciation".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "post_accruals".into(),
            to_activity: "fx_reval".into(),
            is_standard_path: true,
            probability: Some(0.6),
            condition: Some("Multi-currency".into()),
            description: Some("FX revaluation if multi-currency".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "post_accruals".into(),
            to_activity: "gen_tb".into(),
            is_standard_path: true,
            probability: Some(0.4),
            condition: Some("Single currency".into()),
            description: Some("Skip FX for single currency".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "fx_reval".into(),
            to_activity: "ic_elim".into(),
            is_standard_path: true,
            probability: Some(0.5),
            condition: Some("Multi-entity".into()),
            description: Some("IC elimination for multi-entity".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "fx_reval".into(),
            to_activity: "gen_tb".into(),
            is_standard_path: true,
            probability: Some(0.5),
            condition: Some("Single entity".into()),
            description: Some("Skip IC for single entity".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "ic_elim".into(),
            to_activity: "gen_tb".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Generate TB after eliminations".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "gen_tb".into(),
            to_activity: "review_tb".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("All TBs require review".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "review_tb".into(),
            to_activity: "approve_tb".into(),
            is_standard_path: true,
            probability: Some(0.90),
            condition: Some("TB balanced".into()),
            description: Some("Balanced TBs go to approval".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "review_tb".into(),
            to_activity: "post_je".into(),
            is_standard_path: false,
            probability: Some(0.10),
            condition: Some("Adjustments needed".into()),
            description: Some("Post adjusting entries".into()),
        });

        model.add_transition(ReferenceTransition {
            from_activity: "approve_tb".into(),
            to_activity: "close_period".into(),
            is_standard_path: true,
            probability: Some(1.0),
            condition: None,
            description: Some("Period closed after TB approval".into()),
        });

        // Variants
        model.add_variant(ReferenceVariant {
            variant_id: "R2R_SIMPLE".into(),
            name: "Simple Period Close".into(),
            activity_sequence: vec![
                "post_je".into(),
                "review_je".into(),
                "approve_je".into(),
                "run_depr".into(),
                "post_accruals".into(),
                "gen_tb".into(),
                "review_tb".into(),
                "approve_tb".into(),
                "close_period".into(),
            ],
            is_standard: true,
            expected_frequency: Some(0.40),
            description: Some("Single entity, single currency close".into()),
        });

        model.add_variant(ReferenceVariant {
            variant_id: "R2R_MULTI_CURRENCY".into(),
            name: "Multi-Currency Period Close".into(),
            activity_sequence: vec![
                "post_je".into(),
                "review_je".into(),
                "approve_je".into(),
                "run_depr".into(),
                "post_accruals".into(),
                "fx_reval".into(),
                "gen_tb".into(),
                "review_tb".into(),
                "approve_tb".into(),
                "close_period".into(),
            ],
            is_standard: false,
            expected_frequency: Some(0.35),
            description: Some("Single entity with FX revaluation".into()),
        });

        model.add_variant(ReferenceVariant {
            variant_id: "R2R_CONSOLIDATION".into(),
            name: "Full Consolidation Close".into(),
            activity_sequence: vec![
                "post_je".into(),
                "review_je".into(),
                "approve_je".into(),
                "run_depr".into(),
                "post_accruals".into(),
                "fx_reval".into(),
                "ic_elim".into(),
                "gen_tb".into(),
                "review_tb".into(),
                "approve_tb".into(),
                "close_period".into(),
            ],
            is_standard: false,
            expected_frequency: Some(0.20),
            description: Some("Multi-entity consolidation with eliminations".into()),
        });

        model.add_variant(ReferenceVariant {
            variant_id: "R2R_ADJUSTMENTS".into(),
            name: "Close with Adjustments".into(),
            activity_sequence: vec![
                "post_je".into(),
                "review_je".into(),
                "approve_je".into(),
                "run_depr".into(),
                "post_accruals".into(),
                "gen_tb".into(),
                "review_tb".into(),
                "post_je".into(), // Adjusting entry
                "review_je".into(),
                "approve_je".into(),
                "gen_tb".into(),
                "review_tb".into(),
                "approve_tb".into(),
                "close_period".into(),
            ],
            is_standard: false,
            expected_frequency: Some(0.05),
            description: Some("Close requiring adjusting entries".into()),
        });

        model
    }

    /// Get all standard reference models.
    pub fn all_standard_models() -> Vec<Self> {
        vec![
            Self::p2p_standard(),
            Self::o2c_standard(),
            Self::r2r_standard(),
        ]
    }
}

/// Exporter for reference process models.
#[derive(Debug, Clone, Default)]
pub struct ReferenceModelExporter {
    /// Include variants in export
    pub include_variants: bool,
    /// Include transition probabilities
    pub include_probabilities: bool,
}

impl ReferenceModelExporter {
    /// Create a new reference model exporter.
    pub fn new() -> Self {
        Self {
            include_variants: true,
            include_probabilities: true,
        }
    }

    /// Set whether to include variants.
    pub fn with_variants(mut self, include: bool) -> Self {
        self.include_variants = include;
        self
    }

    /// Set whether to include probabilities.
    pub fn with_probabilities(mut self, include: bool) -> Self {
        self.include_probabilities = include;
        self
    }

    /// Export all standard models to a directory.
    pub fn export_all_to_directory<P: AsRef<Path>>(&self, dir: P) -> std::io::Result<()> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        for model in ReferenceProcessModel::all_standard_models() {
            let filename = format!("{}.json", model.process_id.to_lowercase());
            let path = dir.join(filename);
            model.export_to_file(&path)?;
        }

        Ok(())
    }

    /// Export a single model to a file.
    pub fn export_to_file<P: AsRef<Path>>(
        &self,
        model: &ReferenceProcessModel,
        path: P,
    ) -> std::io::Result<()> {
        model.export_to_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_reference_model() {
        let model = ReferenceProcessModel::p2p_standard();

        assert_eq!(model.process_id, "P2P_STANDARD");
        assert_eq!(model.business_process, BusinessProcess::P2P);
        assert_eq!(model.activities.len(), 9);
        assert!(!model.transitions.is_empty());
        assert!(!model.variants.is_empty());

        // Check start and end activities
        let start_activity = model.activities.iter().find(|a| a.is_start).unwrap();
        assert_eq!(start_activity.activity_id, "create_po");

        let end_activity = model.activities.iter().find(|a| a.is_end).unwrap();
        assert_eq!(end_activity.activity_id, "execute_payment");

        // Check standard variant
        let standard_variant = model.variants.iter().find(|v| v.is_standard).unwrap();
        assert_eq!(standard_variant.variant_id, "P2P_HAPPY");
        assert_eq!(standard_variant.activity_sequence.len(), 9);
    }

    #[test]
    fn test_o2c_reference_model() {
        let model = ReferenceProcessModel::o2c_standard();

        assert_eq!(model.process_id, "O2C_STANDARD");
        assert_eq!(model.business_process, BusinessProcess::O2C);
        assert_eq!(model.activities.len(), 10);

        // Check standard variant
        let standard_variant = model.variants.iter().find(|v| v.is_standard).unwrap();
        assert_eq!(standard_variant.activity_sequence.len(), 10);
    }

    #[test]
    fn test_r2r_reference_model() {
        let model = ReferenceProcessModel::r2r_standard();

        assert_eq!(model.process_id, "R2R_STANDARD");
        assert_eq!(model.business_process, BusinessProcess::R2R);
        assert_eq!(model.activities.len(), 11);
        assert_eq!(model.variants.len(), 4);

        // Check that some activities are optional (FX, IC)
        let fx_activity = model
            .activities
            .iter()
            .find(|a| a.activity_id == "fx_reval")
            .unwrap();
        assert!(!fx_activity.is_required);
    }

    #[test]
    fn test_all_standard_models() {
        let models = ReferenceProcessModel::all_standard_models();
        assert_eq!(models.len(), 3);

        let process_ids: Vec<_> = models.iter().map(|m| m.process_id.as_str()).collect();
        assert!(process_ids.contains(&"P2P_STANDARD"));
        assert!(process_ids.contains(&"O2C_STANDARD"));
        assert!(process_ids.contains(&"R2R_STANDARD"));
    }

    #[test]
    fn test_export_to_string() {
        let model = ReferenceProcessModel::p2p_standard();
        let json = model.export_to_string().unwrap();

        assert!(json.contains("P2P_STANDARD"));
        assert!(json.contains("create_po"));
        assert!(json.contains("execute_payment"));
    }

    #[test]
    fn test_reference_model_exporter() {
        let exporter = ReferenceModelExporter::new()
            .with_variants(true)
            .with_probabilities(true);

        assert!(exporter.include_variants);
        assert!(exporter.include_probabilities);
    }

    #[test]
    fn test_transition_coverage() {
        let model = ReferenceProcessModel::p2p_standard();

        // Every activity except start should have an incoming transition
        for activity in &model.activities {
            if !activity.is_start {
                let has_incoming = model
                    .transitions
                    .iter()
                    .any(|t| t.to_activity == activity.activity_id);
                assert!(
                    has_incoming,
                    "Activity {} has no incoming transition",
                    activity.activity_id
                );
            }
        }

        // Every activity except end should have an outgoing transition
        for activity in &model.activities {
            if !activity.is_end {
                let has_outgoing = model
                    .transitions
                    .iter()
                    .any(|t| t.from_activity == activity.activity_id);
                assert!(
                    has_outgoing,
                    "Activity {} has no outgoing transition",
                    activity.activity_id
                );
            }
        }
    }
}
