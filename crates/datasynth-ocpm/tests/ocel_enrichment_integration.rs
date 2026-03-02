//! Integration tests for OCEL 2.0 enrichment features.

#![allow(clippy::unwrap_used)]

use chrono::Utc;
use uuid::Uuid;

use datasynth_ocpm::{
    purchase_order_state_machine, CorrelationEvent, EventLifecycle, OcpmEvent, ResourcePool,
};

#[test]
fn test_po_lifecycle_full_happy_path() {
    let sm = purchase_order_state_machine();
    sm.validate().unwrap();
    let mut current = "Draft";
    let path = [
        "Submitted",
        "Approved",
        "Released",
        "FullyReceived",
        "Closed",
    ];
    for expected_next in &path {
        let transitions = sm.transitions_from(current);
        assert!(!transitions.is_empty(), "No transitions from {}", current);
        let matching = transitions.iter().find(|t| t.to_state == *expected_next);
        assert!(
            matching.is_some(),
            "No transition from {} to {}",
            current,
            expected_next
        );
        current = expected_next;
    }
    assert!(sm.is_terminal(current));
}

#[test]
fn test_correlation_event_three_way_match() {
    let event = CorrelationEvent::three_way_match(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Utc::now(),
        "AP-CLERK-001",
        "C001",
    )
    .with_attribute("match_tolerance", serde_json::json!(0.01));
    assert_eq!(event.object_refs.len(), 3);
    assert!(event.attributes.contains_key("match_tolerance"));
    let json = serde_json::to_string_pretty(&event).unwrap();
    let restored: CorrelationEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.object_refs.len(), 3);
}

#[test]
fn test_resource_pool_workload_balancing() {
    let mut pool = ResourcePool::new("ap", "AP Clerk", 3, "AP-CLERK");
    let mut assignments: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for _ in 0..9 {
        let r = pool.assign().unwrap().to_string();
        *assignments.entry(r).or_default() += 1;
    }
    for (_resource, count) in &assignments {
        assert_eq!(*count, 3);
    }
}

#[test]
fn test_enriched_event_full_attributes() {
    let event = OcpmEvent::new("ACT-APPROVE", "Approve PO", Utc::now(), "MGR-001", "C001")
        .with_state_transition("Submitted", "Approved")
        .with_resource_workload(0.65)
        .with_correlation_id("BATCH-2024-001")
        .with_lifecycle(EventLifecycle::Complete);
    assert_eq!(event.from_state.as_deref(), Some("Submitted"));
    assert_eq!(event.to_state.as_deref(), Some("Approved"));
    assert_eq!(event.resource_workload, Some(0.65));
    assert!(event.lifecycle.is_completion());
}
