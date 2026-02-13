//! Activity type definitions for OCPM.
//!
//! Activity types define the schema for business activities that can occur
//! on objects, including which object types they affect and what state
//! transitions they trigger.

use datasynth_core::models::BusinessProcess;
use serde::{Deserialize, Serialize};

/// Definition of a business activity in OCPM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityType {
    /// Unique activity type ID
    pub activity_id: String,
    /// Human-readable name (e.g., "Create Order", "Post Invoice")
    pub name: String,
    /// Business process this activity belongs to
    pub business_process: BusinessProcess,
    /// Object types this activity operates on
    pub involved_object_types: Vec<String>,
    /// Lifecycle transitions this activity triggers
    pub state_transitions: Vec<ActivityStateTransition>,
    /// Resource types that can perform this activity
    pub allowed_resource_types: Vec<String>,
    /// Typical duration in minutes (for simulation)
    pub typical_duration_minutes: Option<f64>,
    /// Standard deviation of duration
    pub duration_std_dev: Option<f64>,
    /// Is this a manual or automated activity
    pub is_automated: bool,
    /// Does this activity create a new object
    pub creates_object: bool,
    /// Does this activity complete/terminate an object
    pub completes_object: bool,
}

impl ActivityType {
    /// Create a new activity type.
    pub fn new(activity_id: &str, name: &str, business_process: BusinessProcess) -> Self {
        Self {
            activity_id: activity_id.into(),
            name: name.into(),
            business_process,
            involved_object_types: Vec::new(),
            state_transitions: Vec::new(),
            allowed_resource_types: vec!["user".into()],
            typical_duration_minutes: None,
            duration_std_dev: None,
            is_automated: false,
            creates_object: false,
            completes_object: false,
        }
    }

    /// Add involved object types.
    pub fn with_object_types(mut self, types: Vec<&str>) -> Self {
        self.involved_object_types = types.into_iter().map(String::from).collect();
        self
    }

    /// Add state transitions.
    pub fn with_transitions(mut self, transitions: Vec<ActivityStateTransition>) -> Self {
        self.state_transitions = transitions;
        self
    }

    /// Set typical duration.
    pub fn with_duration(mut self, minutes: f64, std_dev: f64) -> Self {
        self.typical_duration_minutes = Some(minutes);
        self.duration_std_dev = Some(std_dev);
        self
    }

    /// Mark as automated.
    pub fn automated(mut self) -> Self {
        self.is_automated = true;
        self
    }

    /// Mark as creating an object.
    pub fn creates(mut self) -> Self {
        self.creates_object = true;
        self
    }

    /// Mark as completing an object.
    pub fn completes(mut self) -> Self {
        self.completes_object = true;
        self
    }

    // ========== P2P Activities ==========

    /// Create Purchase Order activity.
    pub fn create_po() -> Self {
        Self::new("create_po", "Create Purchase Order", BusinessProcess::P2P)
            .with_object_types(vec!["purchase_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "purchase_order",
                None,
                "created",
            )])
            .with_duration(15.0, 5.0)
            .creates()
    }

    /// Approve Purchase Order activity.
    pub fn approve_po() -> Self {
        Self::new("approve_po", "Approve Purchase Order", BusinessProcess::P2P)
            .with_object_types(vec!["purchase_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "purchase_order",
                Some("created"),
                "approved",
            )])
            .with_duration(30.0, 15.0)
    }

    /// Release Purchase Order activity.
    pub fn release_po() -> Self {
        Self::new("release_po", "Release Purchase Order", BusinessProcess::P2P)
            .with_object_types(vec!["purchase_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "purchase_order",
                Some("approved"),
                "released",
            )])
            .with_duration(5.0, 2.0)
            .automated()
    }

    /// Create Goods Receipt activity.
    pub fn create_gr() -> Self {
        Self::new("create_gr", "Create Goods Receipt", BusinessProcess::P2P)
            .with_object_types(vec!["goods_receipt", "purchase_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("goods_receipt", None, "created"),
                ActivityStateTransition::new("purchase_order", Some("released"), "received"),
            ])
            .with_duration(10.0, 5.0)
            .creates()
    }

    /// Post Goods Receipt activity.
    pub fn post_gr() -> Self {
        Self::new("post_gr", "Post Goods Receipt", BusinessProcess::P2P)
            .with_object_types(vec!["goods_receipt"])
            .with_transitions(vec![ActivityStateTransition::new(
                "goods_receipt",
                Some("created"),
                "posted",
            )])
            .with_duration(2.0, 1.0)
            .automated()
    }

    /// Receive Invoice activity.
    pub fn receive_invoice() -> Self {
        Self::new("receive_invoice", "Receive Invoice", BusinessProcess::P2P)
            .with_object_types(vec!["vendor_invoice"])
            .with_transitions(vec![ActivityStateTransition::new(
                "vendor_invoice",
                None,
                "received",
            )])
            .with_duration(5.0, 2.0)
            .creates()
    }

    /// Verify Invoice (three-way match) activity.
    pub fn verify_invoice() -> Self {
        Self::new("verify_invoice", "Verify Invoice", BusinessProcess::P2P)
            .with_object_types(vec!["vendor_invoice", "purchase_order", "goods_receipt"])
            .with_transitions(vec![ActivityStateTransition::new(
                "vendor_invoice",
                Some("received"),
                "verified",
            )])
            .with_duration(20.0, 10.0)
    }

    /// Post Invoice activity.
    pub fn post_invoice() -> Self {
        Self::new("post_invoice", "Post Invoice", BusinessProcess::P2P)
            .with_object_types(vec!["vendor_invoice", "purchase_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("vendor_invoice", Some("verified"), "posted"),
                ActivityStateTransition::new("purchase_order", Some("received"), "invoiced"),
            ])
            .with_duration(3.0, 1.0)
            .automated()
    }

    /// Execute Payment activity.
    pub fn execute_payment() -> Self {
        Self::new("execute_payment", "Execute Payment", BusinessProcess::P2P)
            .with_object_types(vec!["vendor_invoice", "purchase_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("vendor_invoice", Some("posted"), "paid"),
                ActivityStateTransition::new("purchase_order", Some("invoiced"), "paid"),
            ])
            .with_duration(1.0, 0.5)
            .automated()
            .completes()
    }

    // ========== O2C Activities ==========

    /// Create Sales Order activity.
    pub fn create_so() -> Self {
        Self::new("create_so", "Create Sales Order", BusinessProcess::O2C)
            .with_object_types(vec!["sales_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "sales_order",
                None,
                "created",
            )])
            .with_duration(10.0, 5.0)
            .creates()
    }

    /// Check Credit activity.
    pub fn check_credit() -> Self {
        Self::new("check_credit", "Check Credit", BusinessProcess::O2C)
            .with_object_types(vec!["sales_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "sales_order",
                Some("created"),
                "credit_checked",
            )])
            .with_duration(2.0, 1.0)
            .automated()
    }

    /// Release Sales Order activity.
    pub fn release_so() -> Self {
        Self::new("release_so", "Release Sales Order", BusinessProcess::O2C)
            .with_object_types(vec!["sales_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "sales_order",
                Some("credit_checked"),
                "released",
            )])
            .with_duration(5.0, 2.0)
    }

    /// Create Delivery activity.
    pub fn create_delivery() -> Self {
        Self::new("create_delivery", "Create Delivery", BusinessProcess::O2C)
            .with_object_types(vec!["delivery", "sales_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "delivery", None, "created",
            )])
            .with_duration(5.0, 2.0)
            .creates()
    }

    /// Pick activity.
    pub fn pick() -> Self {
        Self::new("pick", "Pick", BusinessProcess::O2C)
            .with_object_types(vec!["delivery"])
            .with_transitions(vec![ActivityStateTransition::new(
                "delivery",
                Some("created"),
                "picked",
            )])
            .with_duration(30.0, 15.0)
    }

    /// Pack activity.
    pub fn pack() -> Self {
        Self::new("pack", "Pack", BusinessProcess::O2C)
            .with_object_types(vec!["delivery"])
            .with_transitions(vec![ActivityStateTransition::new(
                "delivery",
                Some("picked"),
                "packed",
            )])
            .with_duration(20.0, 10.0)
    }

    /// Ship activity.
    pub fn ship() -> Self {
        Self::new("ship", "Ship", BusinessProcess::O2C)
            .with_object_types(vec!["delivery", "sales_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("delivery", Some("packed"), "shipped"),
                ActivityStateTransition::new("sales_order", Some("released"), "delivered"),
            ])
            .with_duration(10.0, 5.0)
    }

    /// Create Customer Invoice activity.
    pub fn create_customer_invoice() -> Self {
        Self::new(
            "create_customer_invoice",
            "Create Customer Invoice",
            BusinessProcess::O2C,
        )
        .with_object_types(vec!["customer_invoice", "sales_order"])
        .with_transitions(vec![
            ActivityStateTransition::new("customer_invoice", None, "created"),
            ActivityStateTransition::new("sales_order", Some("delivered"), "invoiced"),
        ])
        .with_duration(5.0, 2.0)
        .creates()
    }

    /// Post Customer Invoice activity.
    pub fn post_customer_invoice() -> Self {
        Self::new(
            "post_customer_invoice",
            "Post Customer Invoice",
            BusinessProcess::O2C,
        )
        .with_object_types(vec!["customer_invoice"])
        .with_transitions(vec![ActivityStateTransition::new(
            "customer_invoice",
            Some("created"),
            "posted",
        )])
        .with_duration(2.0, 1.0)
        .automated()
    }

    /// Receive Payment activity.
    pub fn receive_payment() -> Self {
        Self::new("receive_payment", "Receive Payment", BusinessProcess::O2C)
            .with_object_types(vec!["customer_invoice", "sales_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("customer_invoice", Some("posted"), "paid"),
                ActivityStateTransition::new("sales_order", Some("invoiced"), "paid"),
            ])
            .with_duration(1.0, 0.5)
            .automated()
            .completes()
    }

    /// Get all standard P2P activities.
    pub fn p2p_activities() -> Vec<Self> {
        vec![
            Self::create_po(),
            Self::approve_po(),
            Self::release_po(),
            Self::create_gr(),
            Self::post_gr(),
            Self::receive_invoice(),
            Self::verify_invoice(),
            Self::post_invoice(),
            Self::execute_payment(),
        ]
    }

    /// Get all standard O2C activities.
    pub fn o2c_activities() -> Vec<Self> {
        vec![
            Self::create_so(),
            Self::check_credit(),
            Self::release_so(),
            Self::create_delivery(),
            Self::pick(),
            Self::pack(),
            Self::ship(),
            Self::create_customer_invoice(),
            Self::post_customer_invoice(),
            Self::receive_payment(),
        ]
    }

    // ========== R2R (Record-to-Report) Activities ==========

    /// Post Journal Entry activity.
    pub fn post_journal_entry() -> Self {
        Self::new("post_je", "Post Journal Entry", BusinessProcess::R2R)
            .with_object_types(vec!["journal_entry"])
            .with_transitions(vec![ActivityStateTransition::new(
                "journal_entry",
                None,
                "posted",
            )])
            .with_duration(5.0, 2.0)
            .creates()
    }

    /// Review Journal Entry activity.
    pub fn review_journal_entry() -> Self {
        Self::new("review_je", "Review Journal Entry", BusinessProcess::R2R)
            .with_object_types(vec!["journal_entry"])
            .with_transitions(vec![ActivityStateTransition::new(
                "journal_entry",
                Some("posted"),
                "reviewed",
            )])
            .with_duration(15.0, 8.0)
    }

    /// Approve Journal Entry activity.
    pub fn approve_journal_entry() -> Self {
        Self::new("approve_je", "Approve Journal Entry", BusinessProcess::R2R)
            .with_object_types(vec!["journal_entry"])
            .with_transitions(vec![ActivityStateTransition::new(
                "journal_entry",
                Some("reviewed"),
                "approved",
            )])
            .with_duration(10.0, 5.0)
    }

    /// Reverse Journal Entry activity.
    pub fn reverse_journal_entry() -> Self {
        Self::new("reverse_je", "Reverse Journal Entry", BusinessProcess::R2R)
            .with_object_types(vec!["journal_entry"])
            .with_transitions(vec![ActivityStateTransition::new(
                "journal_entry",
                Some("approved"),
                "reversed",
            )])
            .with_duration(3.0, 1.0)
    }

    /// FX Revaluation activity.
    pub fn fx_revaluation() -> Self {
        Self::new("fx_reval", "FX Revaluation", BusinessProcess::R2R)
            .with_object_types(vec!["journal_entry", "fx_adjustment"])
            .with_transitions(vec![ActivityStateTransition::new(
                "fx_adjustment",
                None,
                "calculated",
            )])
            .with_duration(30.0, 10.0)
            .automated()
            .creates()
    }

    /// Currency Translation activity (for consolidation).
    pub fn currency_translation() -> Self {
        Self::new("curr_trans", "Currency Translation", BusinessProcess::R2R)
            .with_object_types(vec!["translation_adjustment"])
            .with_transitions(vec![ActivityStateTransition::new(
                "translation_adjustment",
                None,
                "translated",
            )])
            .with_duration(15.0, 5.0)
            .automated()
            .creates()
    }

    /// Run Depreciation activity.
    pub fn run_depreciation() -> Self {
        Self::new("run_depr", "Run Depreciation", BusinessProcess::A2R)
            .with_object_types(vec!["fixed_asset", "journal_entry"])
            .with_transitions(vec![ActivityStateTransition::new(
                "fixed_asset",
                Some("active"),
                "depreciated",
            )])
            .with_duration(60.0, 20.0)
            .automated()
    }

    /// Asset Impairment Test activity.
    pub fn asset_impairment_test() -> Self {
        Self::new("impair_test", "Asset Impairment Test", BusinessProcess::A2R)
            .with_object_types(vec!["fixed_asset", "impairment_assessment"])
            .with_transitions(vec![ActivityStateTransition::new(
                "impairment_assessment",
                None,
                "assessed",
            )])
            .with_duration(120.0, 60.0)
            .creates()
    }

    /// Post Accruals activity.
    pub fn post_accruals() -> Self {
        Self::new("post_accruals", "Post Accruals", BusinessProcess::R2R)
            .with_object_types(vec!["accrual_entry", "journal_entry"])
            .with_transitions(vec![ActivityStateTransition::new(
                "accrual_entry",
                None,
                "posted",
            )])
            .with_duration(20.0, 10.0)
            .creates()
    }

    /// Reverse Accruals activity.
    pub fn reverse_accruals() -> Self {
        Self::new("reverse_accruals", "Reverse Accruals", BusinessProcess::R2R)
            .with_object_types(vec!["accrual_entry", "journal_entry"])
            .with_transitions(vec![ActivityStateTransition::new(
                "accrual_entry",
                Some("posted"),
                "reversed",
            )])
            .with_duration(10.0, 5.0)
            .automated()
    }

    /// Run Intercompany Elimination activity.
    pub fn run_ic_elimination() -> Self {
        Self::new(
            "ic_elim",
            "Run IC Elimination",
            BusinessProcess::Intercompany,
        )
        .with_object_types(vec!["ic_transaction", "elimination_entry"])
        .with_transitions(vec![ActivityStateTransition::new(
            "elimination_entry",
            None,
            "eliminated",
        )])
        .with_duration(45.0, 15.0)
        .automated()
        .creates()
    }

    /// Close Period activity.
    pub fn close_period() -> Self {
        Self::new("close_period", "Close Period", BusinessProcess::R2R)
            .with_object_types(vec!["fiscal_period"])
            .with_transitions(vec![ActivityStateTransition::new(
                "fiscal_period",
                Some("open"),
                "closed",
            )])
            .with_duration(30.0, 15.0)
            .completes()
    }

    /// Reopen Period activity (for adjustments).
    pub fn reopen_period() -> Self {
        Self::new("reopen_period", "Reopen Period", BusinessProcess::R2R)
            .with_object_types(vec!["fiscal_period"])
            .with_transitions(vec![ActivityStateTransition::new(
                "fiscal_period",
                Some("closed"),
                "reopened",
            )])
            .with_duration(5.0, 2.0)
    }

    /// Generate Trial Balance activity.
    pub fn generate_trial_balance() -> Self {
        Self::new("gen_tb", "Generate Trial Balance", BusinessProcess::R2R)
            .with_object_types(vec!["trial_balance"])
            .with_transitions(vec![ActivityStateTransition::new(
                "trial_balance",
                None,
                "generated",
            )])
            .with_duration(10.0, 5.0)
            .automated()
            .creates()
    }

    /// Review Trial Balance activity.
    pub fn review_trial_balance() -> Self {
        Self::new("review_tb", "Review Trial Balance", BusinessProcess::R2R)
            .with_object_types(vec!["trial_balance"])
            .with_transitions(vec![ActivityStateTransition::new(
                "trial_balance",
                Some("generated"),
                "reviewed",
            )])
            .with_duration(60.0, 30.0)
    }

    /// Approve Trial Balance activity.
    pub fn approve_trial_balance() -> Self {
        Self::new("approve_tb", "Approve Trial Balance", BusinessProcess::R2R)
            .with_object_types(vec!["trial_balance"])
            .with_transitions(vec![ActivityStateTransition::new(
                "trial_balance",
                Some("reviewed"),
                "approved",
            )])
            .with_duration(30.0, 15.0)
            .completes()
    }

    /// Run Consolidation activity.
    pub fn run_consolidation() -> Self {
        Self::new("run_consol", "Run Consolidation", BusinessProcess::R2R)
            .with_object_types(vec!["consolidated_balance"])
            .with_transitions(vec![ActivityStateTransition::new(
                "consolidated_balance",
                None,
                "consolidated",
            )])
            .with_duration(120.0, 45.0)
            .automated()
            .creates()
    }

    /// Get all standard R2R (Record-to-Report) activities.
    pub fn r2r_activities() -> Vec<Self> {
        vec![
            Self::post_journal_entry(),
            Self::review_journal_entry(),
            Self::approve_journal_entry(),
            Self::reverse_journal_entry(),
            Self::fx_revaluation(),
            Self::currency_translation(),
            Self::post_accruals(),
            Self::reverse_accruals(),
            Self::run_ic_elimination(),
            Self::close_period(),
            Self::reopen_period(),
            Self::generate_trial_balance(),
            Self::review_trial_balance(),
            Self::approve_trial_balance(),
            Self::run_consolidation(),
        ]
    }

    /// Get all A2R (Acquire-to-Retire) activities for fixed assets.
    pub fn a2r_activities() -> Vec<Self> {
        vec![Self::run_depreciation(), Self::asset_impairment_test()]
    }

    // ========== S2C (Source-to-Contract) Activities ==========

    /// Create Sourcing Project activity.
    pub fn create_sourcing_project() -> Self {
        Self::new("create_sourcing_project", "Create Sourcing Project", BusinessProcess::S2C)
            .with_object_types(vec!["sourcing_project"])
            .with_transitions(vec![ActivityStateTransition::new(
                "sourcing_project", None, "draft",
            )])
            .with_duration(30.0, 10.0)
            .creates()
    }

    /// Qualify Supplier activity.
    pub fn qualify_supplier() -> Self {
        Self::new("qualify_supplier", "Qualify Supplier", BusinessProcess::S2C)
            .with_object_types(vec!["supplier_qualification", "vendor"])
            .with_transitions(vec![ActivityStateTransition::new(
                "supplier_qualification", None, "in_progress",
            )])
            .with_duration(480.0, 120.0)
            .creates()
    }

    /// Publish RFx activity.
    pub fn publish_rfx() -> Self {
        Self::new("publish_rfx", "Publish RFx", BusinessProcess::S2C)
            .with_object_types(vec!["rfx_event", "sourcing_project"])
            .with_transitions(vec![
                ActivityStateTransition::new("rfx_event", None, "published"),
                ActivityStateTransition::new("sourcing_project", Some("draft"), "rfx_active"),
            ])
            .with_duration(60.0, 20.0)
            .creates()
    }

    /// Submit Bid activity.
    pub fn submit_bid() -> Self {
        Self::new("submit_bid", "Submit Bid", BusinessProcess::S2C)
            .with_object_types(vec!["supplier_bid", "rfx_event"])
            .with_transitions(vec![ActivityStateTransition::new(
                "supplier_bid", None, "submitted",
            )])
            .with_duration(20.0, 10.0)
            .creates()
    }

    /// Evaluate Bids activity.
    pub fn evaluate_bids() -> Self {
        Self::new("evaluate_bids", "Evaluate Bids", BusinessProcess::S2C)
            .with_object_types(vec!["bid_evaluation", "rfx_event"])
            .with_transitions(vec![
                ActivityStateTransition::new("bid_evaluation", None, "finalized"),
                ActivityStateTransition::new("rfx_event", Some("published"), "closed"),
            ])
            .with_duration(240.0, 60.0)
            .creates()
    }

    /// Award Contract activity.
    pub fn award_contract() -> Self {
        Self::new("award_contract", "Award Contract", BusinessProcess::S2C)
            .with_object_types(vec!["procurement_contract", "sourcing_project"])
            .with_transitions(vec![
                ActivityStateTransition::new("procurement_contract", None, "draft"),
                ActivityStateTransition::new("sourcing_project", Some("rfx_active"), "awarded"),
            ])
            .with_duration(60.0, 30.0)
            .creates()
    }

    /// Activate Contract activity.
    pub fn activate_contract() -> Self {
        Self::new("activate_contract", "Activate Contract", BusinessProcess::S2C)
            .with_object_types(vec!["procurement_contract"])
            .with_transitions(vec![ActivityStateTransition::new(
                "procurement_contract", Some("draft"), "active",
            )])
            .with_duration(15.0, 5.0)
    }

    /// Complete Sourcing activity.
    pub fn complete_sourcing() -> Self {
        Self::new("complete_sourcing", "Complete Sourcing", BusinessProcess::S2C)
            .with_object_types(vec!["sourcing_project"])
            .with_transitions(vec![ActivityStateTransition::new(
                "sourcing_project", Some("awarded"), "completed",
            )])
            .with_duration(10.0, 5.0)
            .completes()
    }

    /// Get all standard S2C activities.
    pub fn s2c_activities() -> Vec<Self> {
        vec![
            Self::create_sourcing_project(),
            Self::qualify_supplier(),
            Self::publish_rfx(),
            Self::submit_bid(),
            Self::evaluate_bids(),
            Self::award_contract(),
            Self::activate_contract(),
            Self::complete_sourcing(),
        ]
    }

    // ========== H2R (Hire-to-Retire) Activities ==========

    /// Create Payroll Run activity.
    pub fn create_payroll_run() -> Self {
        Self::new("create_payroll_run", "Create Payroll Run", BusinessProcess::H2R)
            .with_object_types(vec!["payroll_run"])
            .with_transitions(vec![ActivityStateTransition::new(
                "payroll_run", None, "draft",
            )])
            .with_duration(30.0, 10.0)
            .creates()
    }

    /// Calculate Payroll activity.
    pub fn calculate_payroll() -> Self {
        Self::new("calculate_payroll", "Calculate Payroll", BusinessProcess::H2R)
            .with_object_types(vec!["payroll_run", "payroll_line_item"])
            .with_transitions(vec![ActivityStateTransition::new(
                "payroll_run", Some("draft"), "calculated",
            )])
            .with_duration(120.0, 30.0)
            .automated()
    }

    /// Approve Payroll activity.
    pub fn approve_payroll() -> Self {
        Self::new("approve_payroll", "Approve Payroll", BusinessProcess::H2R)
            .with_object_types(vec!["payroll_run"])
            .with_transitions(vec![ActivityStateTransition::new(
                "payroll_run", Some("calculated"), "approved",
            )])
            .with_duration(60.0, 30.0)
    }

    /// Post Payroll activity.
    pub fn post_payroll() -> Self {
        Self::new("post_payroll", "Post Payroll", BusinessProcess::H2R)
            .with_object_types(vec!["payroll_run"])
            .with_transitions(vec![ActivityStateTransition::new(
                "payroll_run", Some("approved"), "posted",
            )])
            .with_duration(5.0, 2.0)
            .automated()
            .completes()
    }

    /// Submit Time Entry activity.
    pub fn submit_time_entry() -> Self {
        Self::new("submit_time_entry", "Submit Time Entry", BusinessProcess::H2R)
            .with_object_types(vec!["time_entry"])
            .with_transitions(vec![ActivityStateTransition::new(
                "time_entry", None, "pending",
            )])
            .with_duration(5.0, 2.0)
            .creates()
    }

    /// Approve Time Entry activity.
    pub fn approve_time_entry() -> Self {
        Self::new("approve_time_entry", "Approve Time Entry", BusinessProcess::H2R)
            .with_object_types(vec!["time_entry"])
            .with_transitions(vec![ActivityStateTransition::new(
                "time_entry", Some("pending"), "approved",
            )])
            .with_duration(10.0, 5.0)
            .completes()
    }

    /// Submit Expense Report activity.
    pub fn submit_expense() -> Self {
        Self::new("submit_expense", "Submit Expense Report", BusinessProcess::H2R)
            .with_object_types(vec!["expense_report"])
            .with_transitions(vec![ActivityStateTransition::new(
                "expense_report", None, "submitted",
            )])
            .with_duration(15.0, 5.0)
            .creates()
    }

    /// Approve Expense Report activity.
    pub fn approve_expense() -> Self {
        Self::new("approve_expense", "Approve Expense Report", BusinessProcess::H2R)
            .with_object_types(vec!["expense_report"])
            .with_transitions(vec![ActivityStateTransition::new(
                "expense_report", Some("submitted"), "approved",
            )])
            .with_duration(30.0, 15.0)
            .completes()
    }

    /// Get all standard H2R activities.
    pub fn h2r_activities() -> Vec<Self> {
        vec![
            Self::create_payroll_run(),
            Self::calculate_payroll(),
            Self::approve_payroll(),
            Self::post_payroll(),
            Self::submit_time_entry(),
            Self::approve_time_entry(),
            Self::submit_expense(),
            Self::approve_expense(),
        ]
    }

    // ========== MFG (Manufacturing) Activities ==========

    /// Create Production Order activity.
    pub fn create_production_order() -> Self {
        Self::new("create_production_order", "Create Production Order", BusinessProcess::Mfg)
            .with_object_types(vec!["production_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "production_order", None, "planned",
            )])
            .with_duration(20.0, 10.0)
            .creates()
    }

    /// Release Production Order activity.
    pub fn release_production_order() -> Self {
        Self::new("release_production_order", "Release Production Order", BusinessProcess::Mfg)
            .with_object_types(vec!["production_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "production_order", Some("planned"), "released",
            )])
            .with_duration(10.0, 5.0)
    }

    /// Start Operation activity.
    pub fn start_operation() -> Self {
        Self::new("start_operation", "Start Operation", BusinessProcess::Mfg)
            .with_object_types(vec!["routing_operation", "production_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("routing_operation", None, "in_process"),
                ActivityStateTransition::new("production_order", Some("released"), "in_process"),
            ])
            .with_duration(5.0, 2.0)
    }

    /// Complete Operation activity.
    pub fn complete_operation() -> Self {
        Self::new("complete_operation", "Complete Operation", BusinessProcess::Mfg)
            .with_object_types(vec!["routing_operation"])
            .with_transitions(vec![ActivityStateTransition::new(
                "routing_operation", Some("in_process"), "completed",
            )])
            .with_duration(60.0, 30.0)
    }

    /// Confirm Production activity.
    pub fn confirm_production() -> Self {
        Self::new("confirm_production", "Confirm Production", BusinessProcess::Mfg)
            .with_object_types(vec!["production_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "production_order", Some("in_process"), "confirmed",
            )])
            .with_duration(15.0, 5.0)
    }

    /// Close Production Order activity.
    pub fn close_production_order() -> Self {
        Self::new("close_production_order", "Close Production Order", BusinessProcess::Mfg)
            .with_object_types(vec!["production_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "production_order", Some("confirmed"), "closed",
            )])
            .with_duration(10.0, 5.0)
            .completes()
    }

    /// Create Quality Inspection activity.
    pub fn create_quality_inspection() -> Self {
        Self::new("create_quality_inspection", "Create Quality Inspection", BusinessProcess::Mfg)
            .with_object_types(vec!["quality_inspection"])
            .with_transitions(vec![ActivityStateTransition::new(
                "quality_inspection", None, "pending",
            )])
            .with_duration(10.0, 5.0)
            .creates()
    }

    /// Record Inspection Result activity.
    pub fn record_inspection_result() -> Self {
        Self::new("record_inspection_result", "Record Inspection Result", BusinessProcess::Mfg)
            .with_object_types(vec!["quality_inspection"])
            .with_transitions(vec![ActivityStateTransition::new(
                "quality_inspection", Some("pending"), "completed",
            )])
            .with_duration(30.0, 15.0)
            .completes()
    }

    /// Start Cycle Count activity.
    pub fn start_cycle_count() -> Self {
        Self::new("start_cycle_count", "Start Cycle Count", BusinessProcess::Mfg)
            .with_object_types(vec!["cycle_count"])
            .with_transitions(vec![ActivityStateTransition::new(
                "cycle_count", None, "in_progress",
            )])
            .with_duration(15.0, 5.0)
            .creates()
    }

    /// Reconcile Cycle Count activity.
    pub fn reconcile_cycle_count() -> Self {
        Self::new("reconcile_cycle_count", "Reconcile Cycle Count", BusinessProcess::Mfg)
            .with_object_types(vec!["cycle_count"])
            .with_transitions(vec![ActivityStateTransition::new(
                "cycle_count", Some("in_progress"), "reconciled",
            )])
            .with_duration(45.0, 20.0)
            .completes()
    }

    /// Get all standard MFG activities.
    pub fn mfg_activities() -> Vec<Self> {
        vec![
            Self::create_production_order(),
            Self::release_production_order(),
            Self::start_operation(),
            Self::complete_operation(),
            Self::confirm_production(),
            Self::close_production_order(),
            Self::create_quality_inspection(),
            Self::record_inspection_result(),
            Self::start_cycle_count(),
            Self::reconcile_cycle_count(),
        ]
    }

    // ========== BANK (Banking) Activities ==========

    /// Onboard Customer activity.
    pub fn onboard_customer() -> Self {
        Self::new("onboard_customer", "Onboard Customer", BusinessProcess::Bank)
            .with_object_types(vec!["banking_customer"])
            .with_transitions(vec![ActivityStateTransition::new(
                "banking_customer", None, "onboarding",
            )])
            .with_duration(60.0, 20.0)
            .creates()
    }

    /// Perform KYC Review activity.
    pub fn perform_kyc_review() -> Self {
        Self::new("perform_kyc_review", "Perform KYC Review", BusinessProcess::Bank)
            .with_object_types(vec!["banking_customer"])
            .with_transitions(vec![ActivityStateTransition::new(
                "banking_customer", Some("onboarding"), "active",
            )])
            .with_duration(120.0, 60.0)
    }

    /// Open Account activity.
    pub fn open_account() -> Self {
        Self::new("open_account", "Open Account", BusinessProcess::Bank)
            .with_object_types(vec!["bank_account", "banking_customer"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_account", None, "active",
            )])
            .with_duration(30.0, 10.0)
            .creates()
    }

    /// Execute Transaction activity.
    pub fn execute_bank_transaction() -> Self {
        Self::new("execute_transaction", "Execute Transaction", BusinessProcess::Bank)
            .with_object_types(vec!["bank_transaction", "bank_account"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_transaction", None, "pending",
            )])
            .with_duration(1.0, 0.5)
            .creates()
    }

    /// Authorize Transaction activity.
    pub fn authorize_transaction() -> Self {
        Self::new("authorize_transaction", "Authorize Transaction", BusinessProcess::Bank)
            .with_object_types(vec!["bank_transaction"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_transaction", Some("pending"), "authorized",
            )])
            .with_duration(2.0, 1.0)
            .automated()
    }

    /// Complete Transaction activity.
    pub fn complete_bank_transaction() -> Self {
        Self::new("complete_transaction", "Complete Transaction", BusinessProcess::Bank)
            .with_object_types(vec!["bank_transaction"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_transaction", Some("authorized"), "completed",
            )])
            .with_duration(1.0, 0.5)
            .automated()
            .completes()
    }

    /// Flag Suspicious Activity activity.
    pub fn flag_suspicious() -> Self {
        Self::new("flag_suspicious", "Flag Suspicious Activity", BusinessProcess::Bank)
            .with_object_types(vec!["bank_transaction"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_transaction", Some("pending"), "flagged",
            )])
            .with_duration(5.0, 2.0)
            .automated()
    }

    /// Freeze Account activity.
    pub fn freeze_account() -> Self {
        Self::new("freeze_account", "Freeze Account", BusinessProcess::Bank)
            .with_object_types(vec!["bank_account"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_account", Some("active"), "frozen",
            )])
            .with_duration(5.0, 2.0)
    }

    /// Get all standard BANK activities.
    pub fn bank_activities() -> Vec<Self> {
        vec![
            Self::onboard_customer(),
            Self::perform_kyc_review(),
            Self::open_account(),
            Self::execute_bank_transaction(),
            Self::authorize_transaction(),
            Self::complete_bank_transaction(),
            Self::flag_suspicious(),
            Self::freeze_account(),
        ]
    }

    // ========== AUDIT Activities ==========

    /// Create Engagement activity.
    pub fn create_engagement() -> Self {
        Self::new("create_engagement", "Create Engagement", BusinessProcess::Audit)
            .with_object_types(vec!["audit_engagement"])
            .with_transitions(vec![ActivityStateTransition::new(
                "audit_engagement", None, "planning",
            )])
            .with_duration(60.0, 20.0)
            .creates()
    }

    /// Plan Engagement activity.
    pub fn plan_engagement() -> Self {
        Self::new("plan_engagement", "Plan Engagement", BusinessProcess::Audit)
            .with_object_types(vec!["audit_engagement"])
            .with_transitions(vec![ActivityStateTransition::new(
                "audit_engagement", Some("planning"), "in_progress",
            )])
            .with_duration(480.0, 120.0)
    }

    /// Assess Risk activity.
    pub fn assess_risk() -> Self {
        Self::new("assess_risk", "Assess Risk", BusinessProcess::Audit)
            .with_object_types(vec!["risk_assessment", "audit_engagement"])
            .with_transitions(vec![ActivityStateTransition::new(
                "risk_assessment", None, "draft",
            )])
            .with_duration(120.0, 60.0)
            .creates()
    }

    /// Create Workpaper activity.
    pub fn create_workpaper() -> Self {
        Self::new("create_workpaper", "Create Workpaper", BusinessProcess::Audit)
            .with_object_types(vec!["workpaper", "audit_engagement"])
            .with_transitions(vec![ActivityStateTransition::new(
                "workpaper", None, "draft",
            )])
            .with_duration(60.0, 30.0)
            .creates()
    }

    /// Review Workpaper activity.
    pub fn review_workpaper() -> Self {
        Self::new("review_workpaper", "Review Workpaper", BusinessProcess::Audit)
            .with_object_types(vec!["workpaper"])
            .with_transitions(vec![ActivityStateTransition::new(
                "workpaper", Some("draft"), "reviewed",
            )])
            .with_duration(90.0, 45.0)
    }

    /// Collect Evidence activity.
    pub fn collect_evidence() -> Self {
        Self::new("collect_evidence", "Collect Evidence", BusinessProcess::Audit)
            .with_object_types(vec!["audit_evidence", "workpaper"])
            .with_transitions(vec![ActivityStateTransition::new(
                "audit_evidence", None, "collected",
            )])
            .with_duration(30.0, 15.0)
            .creates()
    }

    /// Raise Finding activity.
    pub fn raise_finding() -> Self {
        Self::new("raise_finding", "Raise Finding", BusinessProcess::Audit)
            .with_object_types(vec!["audit_finding", "audit_engagement"])
            .with_transitions(vec![ActivityStateTransition::new(
                "audit_finding", None, "draft",
            )])
            .with_duration(45.0, 20.0)
            .creates()
    }

    /// Remediate Finding activity.
    pub fn remediate_finding() -> Self {
        Self::new("remediate_finding", "Remediate Finding", BusinessProcess::Audit)
            .with_object_types(vec!["audit_finding"])
            .with_transitions(vec![ActivityStateTransition::new(
                "audit_finding", Some("draft"), "closed",
            )])
            .with_duration(480.0, 240.0)
    }

    /// Record Judgment activity.
    pub fn record_judgment() -> Self {
        Self::new("record_judgment", "Record Judgment", BusinessProcess::Audit)
            .with_object_types(vec!["professional_judgment"])
            .with_transitions(vec![ActivityStateTransition::new(
                "professional_judgment", None, "approved",
            )])
            .with_duration(60.0, 30.0)
            .creates()
    }

    /// Complete Engagement activity.
    pub fn complete_engagement() -> Self {
        Self::new("complete_engagement", "Complete Engagement", BusinessProcess::Audit)
            .with_object_types(vec!["audit_engagement"])
            .with_transitions(vec![ActivityStateTransition::new(
                "audit_engagement", Some("in_progress"), "complete",
            )])
            .with_duration(30.0, 15.0)
            .completes()
    }

    /// Get all standard AUDIT activities.
    pub fn audit_activities() -> Vec<Self> {
        vec![
            Self::create_engagement(),
            Self::plan_engagement(),
            Self::assess_risk(),
            Self::create_workpaper(),
            Self::review_workpaper(),
            Self::collect_evidence(),
            Self::raise_finding(),
            Self::remediate_finding(),
            Self::record_judgment(),
            Self::complete_engagement(),
        ]
    }

    // ========== Bank Reconciliation Activities (R2R subfamily) ==========

    /// Import Bank Statement activity.
    pub fn import_bank_statement() -> Self {
        Self::new("import_bank_statement", "Import Bank Statement", BusinessProcess::R2R)
            .with_object_types(vec!["bank_reconciliation", "bank_statement_line"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_reconciliation", None, "in_progress",
            )])
            .with_duration(10.0, 5.0)
            .automated()
            .creates()
    }

    /// Auto Match Items activity.
    pub fn auto_match_items() -> Self {
        Self::new("auto_match_items", "Auto Match Items", BusinessProcess::R2R)
            .with_object_types(vec!["bank_statement_line"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_statement_line", Some("unmatched"), "auto_matched",
            )])
            .with_duration(5.0, 2.0)
            .automated()
    }

    /// Manual Match Item activity.
    pub fn manual_match_item() -> Self {
        Self::new("manual_match_item", "Manual Match Item", BusinessProcess::R2R)
            .with_object_types(vec!["bank_statement_line", "reconciling_item"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_statement_line", Some("unmatched"), "manually_matched",
            )])
            .with_duration(15.0, 10.0)
    }

    /// Create Reconciling Item activity.
    pub fn create_reconciling_item() -> Self {
        Self::new("create_reconciling_item", "Create Reconciling Item", BusinessProcess::R2R)
            .with_object_types(vec!["reconciling_item"])
            .with_transitions(vec![ActivityStateTransition::new(
                "reconciling_item", None, "outstanding",
            )])
            .with_duration(10.0, 5.0)
            .creates()
    }

    /// Resolve Exception activity.
    pub fn resolve_exception() -> Self {
        Self::new("resolve_exception", "Resolve Exception", BusinessProcess::R2R)
            .with_object_types(vec!["reconciling_item"])
            .with_transitions(vec![ActivityStateTransition::new(
                "reconciling_item", Some("outstanding"), "resolved",
            )])
            .with_duration(30.0, 15.0)
            .completes()
    }

    /// Approve Reconciliation activity.
    pub fn approve_reconciliation() -> Self {
        Self::new("approve_reconciliation", "Approve Reconciliation", BusinessProcess::R2R)
            .with_object_types(vec!["bank_reconciliation"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_reconciliation", Some("in_progress"), "approved",
            )])
            .with_duration(20.0, 10.0)
    }

    /// Post Reconciliation Entries activity.
    pub fn post_recon_entries() -> Self {
        Self::new("post_recon_entries", "Post Reconciliation Entries", BusinessProcess::R2R)
            .with_object_types(vec!["bank_reconciliation"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_reconciliation", Some("approved"), "posted",
            )])
            .with_duration(5.0, 2.0)
            .automated()
    }

    /// Complete Reconciliation activity.
    pub fn complete_reconciliation() -> Self {
        Self::new("complete_reconciliation", "Complete Reconciliation", BusinessProcess::R2R)
            .with_object_types(vec!["bank_reconciliation"])
            .with_transitions(vec![ActivityStateTransition::new(
                "bank_reconciliation", Some("posted"), "completed",
            )])
            .with_duration(5.0, 2.0)
            .completes()
    }

    /// Get all Bank Reconciliation activities (R2R subfamily).
    pub fn bank_recon_activities() -> Vec<Self> {
        vec![
            Self::import_bank_statement(),
            Self::auto_match_items(),
            Self::manual_match_item(),
            Self::create_reconciling_item(),
            Self::resolve_exception(),
            Self::approve_reconciliation(),
            Self::post_recon_entries(),
            Self::complete_reconciliation(),
        ]
    }

    /// Get all activities across all processes.
    pub fn all_activities() -> Vec<Self> {
        let mut all = Vec::new();
        all.extend(Self::p2p_activities());
        all.extend(Self::o2c_activities());
        all.extend(Self::r2r_activities());
        all.extend(Self::a2r_activities());
        all.extend(Self::s2c_activities());
        all.extend(Self::h2r_activities());
        all.extend(Self::mfg_activities());
        all.extend(Self::bank_activities());
        all.extend(Self::audit_activities());
        all.extend(Self::bank_recon_activities());
        all
    }
}

/// State transition triggered by an activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStateTransition {
    /// Object type ID affected
    pub object_type_id: String,
    /// From state (None = any state, including initial)
    pub from_state: Option<String>,
    /// To state
    pub to_state: String,
}

impl ActivityStateTransition {
    /// Create a new state transition.
    pub fn new(object_type_id: &str, from_state: Option<&str>, to_state: &str) -> Self {
        Self {
            object_type_id: object_type_id.into(),
            from_state: from_state.map(String::from),
            to_state: to_state.into(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_type_creation() {
        let activity = ActivityType::create_po();
        assert_eq!(activity.activity_id, "create_po");
        assert!(activity.creates_object);
        assert!(!activity.is_automated);
    }

    #[test]
    fn test_p2p_activities() {
        let activities = ActivityType::p2p_activities();
        assert_eq!(activities.len(), 9);
    }

    #[test]
    fn test_o2c_activities() {
        let activities = ActivityType::o2c_activities();
        assert_eq!(activities.len(), 10);
    }

    #[test]
    fn test_r2r_activities() {
        let activities = ActivityType::r2r_activities();
        assert_eq!(activities.len(), 15);

        // Check some key activities
        let je_activity = ActivityType::post_journal_entry();
        assert_eq!(je_activity.activity_id, "post_je");
        assert_eq!(je_activity.business_process, BusinessProcess::R2R);
        assert!(je_activity.creates_object);

        let fx_activity = ActivityType::fx_revaluation();
        assert!(fx_activity.is_automated);

        let close_activity = ActivityType::close_period();
        assert!(close_activity.completes_object);
    }

    #[test]
    fn test_a2r_activities() {
        let activities = ActivityType::a2r_activities();
        assert_eq!(activities.len(), 2);

        let depr_activity = ActivityType::run_depreciation();
        assert_eq!(depr_activity.activity_id, "run_depr");
        assert_eq!(depr_activity.business_process, BusinessProcess::A2R);
        assert!(depr_activity.is_automated);
    }

    #[test]
    fn test_s2c_activities() {
        let activities = ActivityType::s2c_activities();
        assert_eq!(activities.len(), 8);

        let create = ActivityType::create_sourcing_project();
        assert_eq!(create.activity_id, "create_sourcing_project");
        assert_eq!(create.business_process, BusinessProcess::S2C);
        assert!(create.creates_object);

        let complete = ActivityType::complete_sourcing();
        assert!(complete.completes_object);
    }

    #[test]
    fn test_h2r_ocpm_activities() {
        let activities = ActivityType::h2r_activities();
        assert_eq!(activities.len(), 8);

        let payroll = ActivityType::create_payroll_run();
        assert_eq!(payroll.business_process, BusinessProcess::H2R);
        assert!(payroll.creates_object);
    }

    #[test]
    fn test_mfg_activities() {
        let activities = ActivityType::mfg_activities();
        assert_eq!(activities.len(), 10);

        let create = ActivityType::create_production_order();
        assert_eq!(create.business_process, BusinessProcess::Mfg);
        assert!(create.creates_object);

        let close = ActivityType::close_production_order();
        assert!(close.completes_object);
    }

    #[test]
    fn test_bank_activities() {
        let activities = ActivityType::bank_activities();
        assert_eq!(activities.len(), 8);

        let onboard = ActivityType::onboard_customer();
        assert_eq!(onboard.business_process, BusinessProcess::Bank);
        assert!(onboard.creates_object);
    }

    #[test]
    fn test_audit_activities() {
        let activities = ActivityType::audit_activities();
        assert_eq!(activities.len(), 10);

        let create = ActivityType::create_engagement();
        assert_eq!(create.business_process, BusinessProcess::Audit);
        assert!(create.creates_object);

        let complete = ActivityType::complete_engagement();
        assert!(complete.completes_object);
    }

    #[test]
    fn test_bank_recon_activities() {
        let activities = ActivityType::bank_recon_activities();
        assert_eq!(activities.len(), 8);

        let import = ActivityType::import_bank_statement();
        assert_eq!(import.business_process, BusinessProcess::R2R);
        assert!(import.creates_object);
        assert!(import.is_automated);
    }

    #[test]
    fn test_all_activities() {
        let all = ActivityType::all_activities();
        // 9 P2P + 10 O2C + 15 R2R + 2 A2R + 8 S2C + 8 H2R + 10 MFG + 8 BANK + 10 AUDIT + 8 BankRecon = 88
        assert_eq!(all.len(), 88);
    }
}
