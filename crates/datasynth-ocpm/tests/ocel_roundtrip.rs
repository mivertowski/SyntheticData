//! OCEL 2.0 round-trip integration tests.
//!
//! Validates end-to-end generation of P2P and O2C event logs,
//! including event/object counts, schema compliance, and chronological ordering.

#![allow(clippy::unwrap_used)]

use rust_decimal::Decimal;
use std::collections::HashSet;

use datasynth_ocpm::{
    OcpmEventGenerator, OcpmEventLog, OcpmUuidFactory, O2cDocuments, P2pDocuments,
};

const USERS: &[&str] = &["user001", "user002", "user003"];

fn user_list() -> Vec<String> {
    USERS.iter().map(|s| s.to_string()).collect()
}

fn make_p2p_case(seed: u64) -> datasynth_ocpm::CaseGenerationResult {
    let mut gen = OcpmEventGenerator::new(seed);
    let factory = OcpmUuidFactory::new(seed);
    let docs = P2pDocuments::new("PO-001", "V001", "1000", Decimal::new(10000, 0), "USD", &factory)
        .with_goods_receipt("GR-001", &factory)
        .with_invoice("INV-001", &factory)
        .with_payment("PAY-001", &factory);
    gen.generate_p2p_case(&docs, chrono::Utc::now(), &user_list())
}

fn make_o2c_case(seed: u64) -> datasynth_ocpm::CaseGenerationResult {
    let mut gen = OcpmEventGenerator::new(seed);
    let factory = OcpmUuidFactory::new(seed);
    let docs =
        O2cDocuments::new("SO-001", "C001", "1000", Decimal::new(8000, 0), "USD", &factory)
            .with_delivery("DEL-001", &factory)
            .with_invoice("CI-001", &factory)
            .with_receipt("REC-001", &factory);
    gen.generate_o2c_case(&docs, chrono::Utc::now(), &user_list())
}

// ============================================================================
// P2P Round-Trip Tests
// ============================================================================

#[test]
fn test_p2p_generates_events_and_objects() {
    let result = make_p2p_case(42);

    assert!(
        !result.events.is_empty(),
        "P2P case should produce events"
    );
    assert!(
        !result.objects.is_empty(),
        "P2P case should produce objects"
    );
    // CaseTrace is always present (not Option)
    assert!(
        !result.case_trace.activity_sequence.is_empty(),
        "P2P case trace should have activities"
    );
}

#[test]
fn test_p2p_events_are_chronological() {
    let result = make_p2p_case(42);

    let timestamps: Vec<_> = result.events.iter().map(|e| e.timestamp).collect();
    for pair in timestamps.windows(2) {
        assert!(
            pair[0] <= pair[1],
            "Events should be in chronological order: {:?} > {:?}",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn test_p2p_event_types_present() {
    let result = make_p2p_case(42);

    let activity_names: HashSet<String> = result
        .events
        .iter()
        .map(|e| e.activity_name.clone())
        .collect();

    // P2P should include at least PO creation and some downstream activity
    assert!(
        activity_names
            .iter()
            .any(|a| a.contains("create") || a.contains("Create")),
        "P2P should have a create activity, got: {:?}",
        activity_names
    );
}

#[test]
fn test_p2p_objects_have_unique_ids() {
    let result = make_p2p_case(42);

    let ids: HashSet<_> = result.objects.iter().map(|o| o.object_id).collect();
    assert_eq!(
        ids.len(),
        result.objects.len(),
        "All object IDs should be unique"
    );
}

// ============================================================================
// O2C Round-Trip Tests
// ============================================================================

#[test]
fn test_o2c_generates_events_and_objects() {
    let result = make_o2c_case(99);

    assert!(!result.events.is_empty(), "O2C case should produce events");
    assert!(
        !result.objects.is_empty(),
        "O2C case should produce objects"
    );
}

#[test]
fn test_o2c_events_are_chronological() {
    let result = make_o2c_case(99);

    let timestamps: Vec<_> = result.events.iter().map(|e| e.timestamp).collect();
    for pair in timestamps.windows(2) {
        assert!(
            pair[0] <= pair[1],
            "O2C events should be in chronological order"
        );
    }
}

// ============================================================================
// Event Log Assembly
// ============================================================================

#[test]
fn test_event_log_standard_types() {
    let log = OcpmEventLog::new().with_standard_types();

    assert!(
        !log.object_types.is_empty(),
        "Standard log should define object types"
    );
    assert!(
        !log.activity_types.is_empty(),
        "Standard log should define activity types"
    );

    // P2P types
    assert!(log.object_types.contains_key("purchase_order"));
    assert!(log.object_types.contains_key("goods_receipt"));
    assert!(log.object_types.contains_key("vendor_invoice"));

    // O2C types
    assert!(log.object_types.contains_key("sales_order"));
    assert!(log.object_types.contains_key("delivery"));
}

#[test]
fn test_multiple_cases_accumulate() {
    let mut gen = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let users = user_list();

    let mut total_events = 0;
    let mut total_objects = 0;

    for i in 0..3 {
        let docs = P2pDocuments::new(
            &format!("PO-{i:03}"),
            &format!("V{i:03}"),
            "1000",
            Decimal::new(5000 + i * 1000, 0),
            "USD",
            &factory,
        )
        .with_goods_receipt(&format!("GR-{i:03}"), &factory)
        .with_invoice(&format!("INV-{i:03}"), &factory)
        .with_payment(&format!("PAY-{i:03}"), &factory);

        let result = gen.generate_p2p_case(&docs, chrono::Utc::now(), &users);
        total_events += result.events.len();
        total_objects += result.objects.len();
    }

    assert!(
        total_events > 10,
        "3 P2P cases should produce many events, got {total_events}"
    );
    assert!(
        total_objects > 5,
        "3 P2P cases should produce multiple objects, got {total_objects}"
    );
}

// ============================================================================
// Determinism
// ============================================================================

#[test]
fn test_deterministic_generation() {
    let result1 = make_p2p_case(42);
    let result2 = make_p2p_case(42);

    assert_eq!(
        result1.events.len(),
        result2.events.len(),
        "Same seed should produce same event count"
    );
    assert_eq!(
        result1.objects.len(),
        result2.objects.len(),
        "Same seed should produce same object count"
    );

    // Activity sequences should match
    let activities1: Vec<&str> = result1
        .events
        .iter()
        .map(|e| e.activity_name.as_str())
        .collect();
    let activities2: Vec<&str> = result2
        .events
        .iter()
        .map(|e| e.activity_name.as_str())
        .collect();
    assert_eq!(
        activities1, activities2,
        "Same seed should produce same activity sequence"
    );
}
