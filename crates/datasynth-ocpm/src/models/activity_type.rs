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

    /// Get all activities across all processes.
    pub fn all_activities() -> Vec<Self> {
        let mut all = Vec::new();
        all.extend(Self::p2p_activities());
        all.extend(Self::o2c_activities());
        all.extend(Self::r2r_activities());
        all.extend(Self::a2r_activities());
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
    fn test_all_activities() {
        let all = ActivityType::all_activities();
        // 9 P2P + 10 O2C + 15 R2R + 2 A2R = 36
        assert_eq!(all.len(), 36);
    }
}
