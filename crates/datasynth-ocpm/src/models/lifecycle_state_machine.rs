//! Lifecycle state machines for OCEL 2.0 object types.
//!
//! Each object type (purchase order, sales order, vendor invoice, etc.) has a
//! lifecycle modeled as a finite state machine. Transitions carry probabilities
//! and timing constraints used by the generator to produce realistic event logs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A finite state machine describing an object type's lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleStateMachine {
    /// Object type this state machine governs (e.g. "purchase_order").
    pub object_type: String,
    /// The state every new object starts in.
    pub initial_state: String,
    /// States from which no further transitions are possible.
    pub terminal_states: Vec<String>,
    /// All valid state transitions with probabilities and timing.
    pub transitions: Vec<StateTransition>,
}

/// A single transition between two states in a lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Source state.
    pub from_state: String,
    /// Target state.
    pub to_state: String,
    /// Probability of taking this transition (outgoing probabilities from a
    /// state should sum to ~1.0).
    pub probability: f64,
    /// Minimum lag in hours before this transition fires.
    pub min_lag_hours: f64,
    /// Maximum lag in hours before this transition fires.
    pub max_lag_hours: f64,
    /// Activity name emitted when the transition fires.
    pub activity_name: String,
}

impl LifecycleStateMachine {
    /// Return all transitions leaving the given state.
    pub fn transitions_from(&self, state: &str) -> Vec<&StateTransition> {
        self.transitions
            .iter()
            .filter(|t| t.from_state == state)
            .collect()
    }

    /// Check whether `state` is a terminal (absorbing) state.
    pub fn is_terminal(&self, state: &str) -> bool {
        self.terminal_states.contains(&state.to_string())
    }

    /// Validate that outgoing transition probabilities from each state sum to
    /// approximately 1.0 (within 0.05 tolerance).
    pub fn validate(&self) -> Result<(), String> {
        let mut state_probs: HashMap<String, f64> = HashMap::new();
        for t in &self.transitions {
            *state_probs.entry(t.from_state.clone()).or_default() += t.probability;
        }
        for (state, prob) in &state_probs {
            if (*prob - 1.0).abs() > 0.05 {
                return Err(format!(
                    "State '{}' transitions sum to {:.2}, expected ~1.0",
                    state, prob
                ));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Factory helpers
// ---------------------------------------------------------------------------

fn transition(
    from: &str,
    to: &str,
    probability: f64,
    min_lag_hours: f64,
    max_lag_hours: f64,
    activity: &str,
) -> StateTransition {
    StateTransition {
        from_state: from.into(),
        to_state: to.into(),
        probability,
        min_lag_hours,
        max_lag_hours,
        activity_name: activity.into(),
    }
}

/// Purchase-order lifecycle state machine.
pub fn purchase_order_state_machine() -> LifecycleStateMachine {
    LifecycleStateMachine {
        object_type: "purchase_order".into(),
        initial_state: "Draft".into(),
        terminal_states: vec!["Closed".into(), "Cancelled".into(), "Rejected".into()],
        transitions: vec![
            transition("Draft", "Submitted", 0.95, 1.0, 24.0, "submit_po"),
            transition("Draft", "Cancelled", 0.05, 0.5, 4.0, "cancel_po"),
            transition("Submitted", "Approved", 0.90, 2.0, 48.0, "approve_po"),
            transition("Submitted", "Rejected", 0.10, 2.0, 48.0, "reject_po"),
            transition("Approved", "Released", 1.0, 0.5, 4.0, "release_po"),
            transition(
                "Released",
                "PartiallyReceived",
                0.30,
                24.0,
                720.0,
                "receive_partial",
            ),
            transition(
                "Released",
                "FullyReceived",
                0.70,
                24.0,
                720.0,
                "receive_full",
            ),
            transition(
                "PartiallyReceived",
                "FullyReceived",
                1.0,
                24.0,
                360.0,
                "receive_remaining",
            ),
            transition("FullyReceived", "Closed", 1.0, 1.0, 48.0, "close_po"),
        ],
    }
}

/// Sales-order lifecycle state machine.
pub fn sales_order_state_machine() -> LifecycleStateMachine {
    LifecycleStateMachine {
        object_type: "sales_order".into(),
        initial_state: "Created".into(),
        terminal_states: vec!["Closed".into(), "Cancelled".into(), "Returned".into()],
        transitions: vec![
            transition("Created", "Confirmed", 0.92, 0.5, 8.0, "confirm_so"),
            transition("Created", "Cancelled", 0.08, 0.5, 4.0, "cancel_so"),
            transition("Confirmed", "Shipped", 0.95, 4.0, 120.0, "ship_so"),
            transition(
                "Confirmed",
                "Cancelled",
                0.05,
                1.0,
                24.0,
                "cancel_confirmed_so",
            ),
            transition("Shipped", "Delivered", 0.98, 24.0, 240.0, "deliver_so"),
            transition("Shipped", "Returned", 0.02, 48.0, 480.0, "return_so"),
            transition("Delivered", "Invoiced", 1.0, 1.0, 48.0, "invoice_so"),
            transition("Invoiced", "Closed", 1.0, 1.0, 72.0, "close_so"),
        ],
    }
}

/// Vendor-invoice lifecycle state machine.
pub fn vendor_invoice_state_machine() -> LifecycleStateMachine {
    LifecycleStateMachine {
        object_type: "vendor_invoice".into(),
        initial_state: "Received".into(),
        terminal_states: vec!["Paid".into(), "Cancelled".into()],
        transitions: vec![
            transition("Received", "Registered", 0.95, 0.5, 8.0, "register_invoice"),
            transition(
                "Received",
                "Cancelled",
                0.05,
                0.5,
                4.0,
                "cancel_invoice_receipt",
            ),
            transition("Registered", "Matched", 0.85, 1.0, 48.0, "match_invoice"),
            transition("Registered", "Blocked", 0.15, 1.0, 24.0, "block_invoice"),
            transition("Blocked", "Matched", 0.80, 4.0, 120.0, "unblock_invoice"),
            transition(
                "Blocked",
                "Cancelled",
                0.20,
                4.0,
                72.0,
                "cancel_blocked_invoice",
            ),
            transition("Matched", "Approved", 1.0, 0.5, 24.0, "approve_invoice"),
            transition("Approved", "Paid", 1.0, 24.0, 720.0, "pay_invoice"),
        ],
    }
}

/// Return all predefined lifecycle state machines.
pub fn all_state_machines() -> Vec<LifecycleStateMachine> {
    vec![
        purchase_order_state_machine(),
        sales_order_state_machine(),
        vendor_invoice_state_machine(),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_order_sm_validates() {
        let sm = purchase_order_state_machine();
        sm.validate().unwrap();
        assert_eq!(sm.object_type, "purchase_order");
        assert_eq!(sm.initial_state, "Draft");
    }

    #[test]
    fn test_sales_order_sm_validates() {
        let sm = sales_order_state_machine();
        sm.validate().unwrap();
        assert_eq!(sm.object_type, "sales_order");
        assert_eq!(sm.initial_state, "Created");
    }

    #[test]
    fn test_vendor_invoice_sm_validates() {
        let sm = vendor_invoice_state_machine();
        sm.validate().unwrap();
        assert_eq!(sm.object_type, "vendor_invoice");
        assert_eq!(sm.initial_state, "Received");
    }

    #[test]
    fn test_transitions_from() {
        let sm = purchase_order_state_machine();
        let from_draft = sm.transitions_from("Draft");
        assert_eq!(from_draft.len(), 2);
        let prob_sum: f64 = from_draft.iter().map(|t| t.probability).sum();
        assert!((prob_sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_is_terminal() {
        let sm = sales_order_state_machine();
        assert!(sm.is_terminal("Closed"));
        assert!(sm.is_terminal("Cancelled"));
        assert!(sm.is_terminal("Returned"));
        assert!(!sm.is_terminal("Confirmed"));
        assert!(!sm.is_terminal("Shipped"));
    }

    #[test]
    fn test_all_state_machines_count() {
        let machines = all_state_machines();
        assert_eq!(machines.len(), 3);
        let types: Vec<&str> = machines.iter().map(|m| m.object_type.as_str()).collect();
        assert!(types.contains(&"purchase_order"));
        assert!(types.contains(&"sales_order"));
        assert!(types.contains(&"vendor_invoice"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let sm = purchase_order_state_machine();
        let json = serde_json::to_string(&sm).unwrap();
        let deserialized: LifecycleStateMachine = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.object_type, sm.object_type);
        assert_eq!(deserialized.transitions.len(), sm.transitions.len());
        assert_eq!(deserialized.terminal_states, sm.terminal_states);
    }
}
