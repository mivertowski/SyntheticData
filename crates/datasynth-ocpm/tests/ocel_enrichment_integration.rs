//! Integration tests for OCEL 2.0 enrichment features.
//!
//! Tests verify that generated P2P and O2C events contain enrichment fields
//! (state transitions, resource workload, correlation IDs).

#![allow(clippy::unwrap_used)]

use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use datasynth_ocpm::{
    purchase_order_state_machine, CorrelationEvent, EventLifecycle, O2cDocuments, OcpmEvent,
    OcpmEventGenerator, OcpmGeneratorConfig, OcpmUuidFactory, P2pDocuments, ResourcePool,
};

// =============================================================================
// Low-level enrichment primitive tests
// =============================================================================

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

// =============================================================================
// P2P enrichment integration tests
// =============================================================================

/// Generate a full P2P document set (PO, GR, Invoice, Payment) via
/// OcpmEventGenerator and verify events have from_state/to_state populated
/// and at least one event has resource_workload.
#[test]
fn test_p2p_generation_produces_enriched_events() {
    let mut generator = OcpmEventGenerator::with_config(
        42,
        OcpmGeneratorConfig {
            happy_path_rate: 1.0,
            exception_path_rate: 0.0,
            error_path_rate: 0.0,
            ..Default::default()
        },
    );

    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-ENRICH-001",
        "V000001",
        "1000",
        Decimal::new(25000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-ENRICH-001", &factory)
    .with_invoice("INV-ENRICH-001", &factory)
    .with_payment("PAY-ENRICH-001", &factory);

    let result = generator.generate_p2p_case(
        &documents,
        Utc::now(),
        &["user001".into(), "user002".into(), "user003".into()],
    );

    // Happy path with all documents should produce a full set of events
    assert!(
        result.events.len() >= 7,
        "Full P2P happy path should produce at least 7 events (create, approve, release, \
         create_gr, post_gr, receive_invoice, verify_invoice, post_invoice, execute_payment), \
         got {}",
        result.events.len()
    );

    // Verify state transitions: at least some events should have from_state/to_state
    let events_with_state: Vec<&OcpmEvent> = result
        .events
        .iter()
        .filter(|e| e.from_state.is_some() && e.to_state.is_some())
        .collect();
    assert!(
        !events_with_state.is_empty(),
        "At least some P2P events should have state transitions (from_state + to_state)"
    );

    // Verify resource_workload: at least one event should have it
    let events_with_workload: Vec<&OcpmEvent> = result
        .events
        .iter()
        .filter(|e| e.resource_workload.is_some())
        .collect();
    assert!(
        !events_with_workload.is_empty(),
        "At least some P2P events should have resource_workload populated"
    );

    // Verify workload values are in valid range
    for event in &events_with_workload {
        let w = event.resource_workload.unwrap();
        assert!(
            w >= 0.0 && w <= 1.0,
            "Resource workload should be in [0.0, 1.0], got {}",
            w
        );
    }
}

// =============================================================================
// O2C enrichment integration tests
// =============================================================================

/// Generate a full O2C document set (SO, Delivery, Invoice, Receipt) via
/// OcpmEventGenerator and verify state transitions and workloads are populated.
#[test]
fn test_o2c_generation_produces_enriched_events() {
    let mut generator = OcpmEventGenerator::with_config(
        99,
        OcpmGeneratorConfig {
            happy_path_rate: 1.0,
            exception_path_rate: 0.0,
            error_path_rate: 0.0,
            ..Default::default()
        },
    );

    let factory = OcpmUuidFactory::new(99);
    let documents = O2cDocuments::new(
        "SO-ENRICH-001",
        "C000001",
        "2000",
        Decimal::new(35000, 0),
        "EUR",
        &factory,
    )
    .with_delivery("DEL-ENRICH-001", &factory)
    .with_invoice("CINV-ENRICH-001", &factory)
    .with_receipt("REC-ENRICH-001", &factory);

    let result = generator.generate_o2c_case(
        &documents,
        Utc::now(),
        &["sales001".into(), "whse001".into(), "ar001".into()],
    );

    // Happy path with all documents should produce a full set of events
    // O2C: create_so, check_credit, release_so, create_delivery, pick, pack, ship,
    //       create_customer_invoice, post_customer_invoice, receive_payment = 10
    assert!(
        result.events.len() >= 8,
        "Full O2C happy path should produce at least 8 events, got {}",
        result.events.len()
    );

    // Verify state transitions are present
    let events_with_state: Vec<&OcpmEvent> = result
        .events
        .iter()
        .filter(|e| e.from_state.is_some() && e.to_state.is_some())
        .collect();
    assert!(
        !events_with_state.is_empty(),
        "At least some O2C events should have state transitions"
    );

    // Verify resource workloads are present
    let events_with_workload: Vec<&OcpmEvent> = result
        .events
        .iter()
        .filter(|e| e.resource_workload.is_some())
        .collect();
    assert!(
        !events_with_workload.is_empty(),
        "At least some O2C events should have resource_workload"
    );

    // All workloads should be in valid range
    for event in &events_with_workload {
        let w = event.resource_workload.unwrap();
        assert!(
            w >= 0.0 && w <= 1.0,
            "O2C resource workload should be in [0.0, 1.0], got {}",
            w
        );
    }
}

// =============================================================================
// State transition validity tests
// =============================================================================

/// Collect all state transitions from generated P2P and O2C events and verify:
/// - States are non-empty strings when present
/// - from_state and to_state are both set or both unset (no partial transitions)
#[test]
fn test_enrichment_state_transitions_valid() {
    let mut generator = OcpmEventGenerator::with_config(
        77,
        OcpmGeneratorConfig {
            happy_path_rate: 1.0,
            exception_path_rate: 0.0,
            error_path_rate: 0.0,
            ..Default::default()
        },
    );

    // Generate P2P events
    let factory = OcpmUuidFactory::new(77);
    let p2p_docs = P2pDocuments::new(
        "PO-TRANS-001",
        "V000010",
        "1000",
        Decimal::new(8000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-TRANS-001", &factory)
    .with_invoice("INV-TRANS-001", &factory)
    .with_payment("PAY-TRANS-001", &factory);

    let p2p_result = generator.generate_p2p_case(&p2p_docs, Utc::now(), &["user001".into()]);

    // Generate O2C events (using a fresh generator to reset resource pools)
    let mut generator2 = OcpmEventGenerator::with_config(
        78,
        OcpmGeneratorConfig {
            happy_path_rate: 1.0,
            exception_path_rate: 0.0,
            error_path_rate: 0.0,
            ..Default::default()
        },
    );

    let factory2 = OcpmUuidFactory::new(78);
    let o2c_docs = O2cDocuments::new(
        "SO-TRANS-001",
        "C000010",
        "2000",
        Decimal::new(12000, 0),
        "EUR",
        &factory2,
    )
    .with_delivery("DEL-TRANS-001", &factory2)
    .with_invoice("CINV-TRANS-001", &factory2)
    .with_receipt("REC-TRANS-001", &factory2);

    let o2c_result = generator2.generate_o2c_case(&o2c_docs, Utc::now(), &["sales001".into()]);

    // Combine all events from both processes
    let all_events: Vec<&OcpmEvent> = p2p_result
        .events
        .iter()
        .chain(o2c_result.events.iter())
        .collect();

    assert!(
        !all_events.is_empty(),
        "Should have generated events from both P2P and O2C"
    );

    let mut transitions_found = 0;

    for event in &all_events {
        // Verify from_state and to_state are both set or both unset
        match (&event.from_state, &event.to_state) {
            (Some(from), Some(to)) => {
                // Both present: verify non-empty
                assert!(
                    !from.is_empty(),
                    "from_state should be non-empty for event {}",
                    event.activity_id
                );
                assert!(
                    !to.is_empty(),
                    "to_state should be non-empty for event {}",
                    event.activity_id
                );
                transitions_found += 1;
            }
            (None, None) => {
                // Both absent: acceptable (not all activities have transitions)
            }
            (Some(_), None) | (None, Some(_)) => {
                panic!(
                    "Partial state transition for event {}: from_state={:?}, to_state={:?}. \
                     Both should be set or both unset.",
                    event.activity_id, event.from_state, event.to_state
                );
            }
        }
    }

    // We should have found at least some transitions across both processes
    assert!(
        transitions_found > 0,
        "Should have found at least one state transition across P2P and O2C events"
    );
}

// =============================================================================
// P2P three-way match correlation tests
// =============================================================================

/// Generate P2P events including a three-way match and verify at least one
/// event has a correlation_id starting with "3WAY-".
#[test]
fn test_p2p_three_way_match_correlation() {
    let mut generator = OcpmEventGenerator::with_config(
        55,
        OcpmGeneratorConfig {
            happy_path_rate: 1.0,
            exception_path_rate: 0.0,
            error_path_rate: 0.0,
            ..Default::default()
        },
    );

    let factory = OcpmUuidFactory::new(55);
    let documents = P2pDocuments::new(
        "PO-3WAY-001",
        "V000020",
        "1000",
        Decimal::new(50000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-3WAY-001", &factory)
    .with_invoice("INV-3WAY-001", &factory)
    .with_payment("PAY-3WAY-001", &factory);

    let result = generator.generate_p2p_case(
        &documents,
        Utc::now(),
        &["buyer001".into(), "approver001".into(), "ap001".into()],
    );

    // Should have correlation events including a three-way match
    assert!(
        !result.correlation_events.is_empty(),
        "Happy path P2P with GR + Invoice should produce correlation events"
    );

    let three_way_correlations: Vec<_> = result
        .correlation_events
        .iter()
        .filter(|c| c.correlation_type == datasynth_ocpm::CorrelationEventType::ThreeWayMatch)
        .collect();
    assert!(
        !three_way_correlations.is_empty(),
        "Should have at least one ThreeWayMatch correlation event"
    );

    // Verify the three-way match correlation has 3 object refs (PO, GR, Invoice)
    for corr in &three_way_correlations {
        assert_eq!(
            corr.object_refs.len(),
            3,
            "ThreeWayMatch correlation should reference exactly 3 objects (PO, GR, Invoice)"
        );
        assert!(
            corr.correlation_id.starts_with("3WAY-"),
            "ThreeWayMatch correlation_id should start with '3WAY-', got '{}'",
            corr.correlation_id
        );
    }

    // At least one event should carry the 3WAY correlation_id
    let events_with_3way: Vec<&OcpmEvent> = result
        .events
        .iter()
        .filter(|e| {
            e.correlation_id
                .as_deref()
                .map(|id| id.starts_with("3WAY-"))
                .unwrap_or(false)
        })
        .collect();
    assert!(
        !events_with_3way.is_empty(),
        "At least one P2P event should have a correlation_id starting with '3WAY-'"
    );

    // The verify_invoice activity should be the one with the 3WAY correlation
    let verify_with_3way: Vec<_> = events_with_3way
        .iter()
        .filter(|e| e.activity_id == "verify_invoice")
        .collect();
    assert!(
        !verify_with_3way.is_empty(),
        "The verify_invoice event should carry the 3WAY correlation_id"
    );
}
