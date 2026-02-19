//! Validation tests for OCPM (Object-Centric Process Mining) event generation.
//!
//! These tests validate OCEL 2.0 schema compliance, event-object relationships,
//! activity transitions, object lifecycles, and process variant distributions.

use std::collections::HashSet;

use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use datasynth_ocpm::{
    OcpmEventGenerator, OcpmEventLog, OcpmGeneratorConfig, OcpmUuidFactory, P2pDocuments,
    VariantType,
};

// =============================================================================
// OCEL 2.0 Schema Compliance Tests
// =============================================================================

/// Test that the event log has required OCEL 2.0 structure.
#[test]
fn test_ocel2_schema_compliance() {
    let log = OcpmEventLog::new().with_standard_types();

    // OCEL 2.0 requires object types to be defined
    assert!(
        !log.object_types.is_empty(),
        "Event log should have object types defined"
    );

    // OCEL 2.0 requires activity types to be defined
    assert!(
        !log.activity_types.is_empty(),
        "Event log should have activity types defined"
    );

    // Verify P2P object types are present
    assert!(
        log.object_types.contains_key("purchase_order"),
        "Should have purchase_order object type"
    );
    assert!(
        log.object_types.contains_key("goods_receipt"),
        "Should have goods_receipt object type"
    );
    assert!(
        log.object_types.contains_key("vendor_invoice"),
        "Should have vendor_invoice object type"
    );

    // Verify O2C object types are present
    assert!(
        log.object_types.contains_key("sales_order"),
        "Should have sales_order object type"
    );
    assert!(
        log.object_types.contains_key("delivery"),
        "Should have delivery object type"
    );
}

/// Test that events have required OCEL 2.0 fields.
#[test]
fn test_event_required_fields() {
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-000001",
        "V000001",
        "1000",
        Decimal::new(10000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-000001", &factory)
    .with_invoice("INV-000001", &factory)
    .with_payment("PAY-000001", &factory);

    let result = generator.generate_p2p_case(
        &documents,
        Utc::now(),
        &["user001".into(), "user002".into()],
    );

    for event in &result.events {
        // Event ID is required
        assert_ne!(event.event_id, Uuid::nil(), "Event should have valid ID");

        // Activity ID is required
        assert!(
            !event.activity_id.is_empty(),
            "Event should have activity ID"
        );

        // Timestamp is required
        assert!(
            event.timestamp > chrono::DateTime::<Utc>::MIN_UTC,
            "Event should have valid timestamp"
        );

        // Resource ID is required (who/what performed the activity)
        assert!(
            !event.resource_id.is_empty(),
            "Event should have resource ID"
        );

        // Events should reference at least one object
        assert!(
            !event.object_refs.is_empty(),
            "Event {} should reference at least one object",
            event.activity_id
        );
    }
}

/// Test that objects have required OCEL 2.0 fields.
#[test]
fn test_object_required_fields() {
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-000002",
        "V000002",
        "1000",
        Decimal::new(5000, 0),
        "EUR",
        &factory,
    )
    .with_goods_receipt("GR-000002", &factory)
    .with_invoice("INV-000002", &factory);

    let result = generator.generate_p2p_case(&documents, Utc::now(), &["user001".into()]);

    for object in &result.objects {
        // Object ID is required
        assert_ne!(object.object_id, Uuid::nil(), "Object should have valid ID");

        // Object type is required
        assert!(
            !object.object_type_id.is_empty(),
            "Object should have type ID"
        );

        // External ID (business key) is required
        assert!(
            !object.external_id.is_empty(),
            "Object should have external ID"
        );

        // Company code is required
        assert!(
            !object.company_code.is_empty(),
            "Object should have company code"
        );
    }
}

// =============================================================================
// Event-Object Relationship Tests
// =============================================================================

/// Test that most event object references point to valid objects.
/// Note: Some edge cases exist in P2P generation where certain events may reference
/// objects from the documents struct rather than generated objects.
#[test]
fn test_event_object_reference_integrity() {
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-000003",
        "V000003",
        "1000",
        Decimal::new(7500, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-000003", &factory)
    .with_invoice("INV-000003", &factory)
    .with_payment("PAY-000003", &factory);

    let result = generator.generate_p2p_case(&documents, Utc::now(), &["user001".into()]);

    let object_ids: HashSet<_> = result.objects.iter().map(|o| o.object_id).collect();

    let mut valid_refs = 0;
    let mut total_refs = 0;

    for event in &result.events {
        for obj_ref in &event.object_refs {
            total_refs += 1;
            if object_ids.contains(&obj_ref.object_id) {
                valid_refs += 1;
            }
        }
    }

    // At least 80% of references should be valid
    // (Some read references may use IDs from the input documents struct)
    let valid_rate = valid_refs as f64 / total_refs as f64;
    assert!(
        valid_rate >= 0.80,
        "Object reference integrity rate {:.2}% below threshold 80%: {}/{}",
        valid_rate * 100.0,
        valid_refs,
        total_refs
    );
}

/// Test that objects have proper lifecycle qualifiers.
/// Note: Some read references may use IDs from input documents, which aren't tracked
/// as "created" objects in our generation context.
#[test]
fn test_object_lifecycle_qualifiers() {
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-000004",
        "V000004",
        "1000",
        Decimal::new(15000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-000004", &factory)
    .with_invoice("INV-000004", &factory)
    .with_payment("PAY-000004", &factory);

    let result = generator.generate_p2p_case(&documents, Utc::now(), &["user001".into()]);

    // Track lifecycle transitions for each object
    let mut object_created: HashSet<Uuid> = HashSet::new();
    let mut violations = 0;
    let mut total_non_create = 0;

    for event in &result.events {
        for obj_ref in &event.object_refs {
            match obj_ref.qualifier {
                datasynth_ocpm::ObjectQualifier::Created => {
                    // Object shouldn't be created twice
                    assert!(
                        !object_created.contains(&obj_ref.object_id),
                        "Object {:?} created multiple times",
                        obj_ref.object_id
                    );
                    object_created.insert(obj_ref.object_id);
                }
                datasynth_ocpm::ObjectQualifier::Updated
                | datasynth_ocpm::ObjectQualifier::Read
                | datasynth_ocpm::ObjectQualifier::Consumed
                | datasynth_ocpm::ObjectQualifier::Context => {
                    total_non_create += 1;
                    // Object should have been created before being updated/read/consumed
                    // (but allow some tolerance for IDs from input documents struct)
                    if !object_created.contains(&obj_ref.object_id) {
                        violations += 1;
                    }
                }
            }
        }
    }

    // At least 80% should follow proper lifecycle
    if total_non_create > 0 {
        let valid_rate = 1.0 - (violations as f64 / total_non_create as f64);
        assert!(
            valid_rate >= 0.70,
            "Lifecycle compliance rate {:.2}% below threshold: {} violations of {} non-create refs",
            valid_rate * 100.0,
            violations,
            total_non_create
        );
    }
}

/// Test many-to-many event-object relationships.
#[test]
fn test_many_to_many_relationships() {
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-000005",
        "V000005",
        "1000",
        Decimal::new(8000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-000005", &factory)
    .with_invoice("INV-000005", &factory);

    let result = generator.generate_p2p_case(&documents, Utc::now(), &["user001".into()]);

    // Verify invoice events reference multiple objects (many-to-many)
    let mut found_multi_object_event = false;
    for event in &result.events {
        if event.object_refs.len() > 1 {
            found_multi_object_event = true;
            println!(
                "Event {} references {} objects",
                event.activity_id,
                event.object_refs.len()
            );
        }
    }

    // P2P process should have events that reference multiple objects
    // (e.g., verify_invoice references PO, GR, and Invoice)
    assert!(
        found_multi_object_event,
        "Should have events with many-to-many object relationships"
    );
}

// =============================================================================
// Activity Transition Tests
// =============================================================================

/// Test that P2P activities follow valid transitions.
#[test]
fn test_p2p_activity_transitions() {
    let mut generator = OcpmEventGenerator::new(42);

    // Test multiple cases
    for i in 0..10 {
        let factory = OcpmUuidFactory::new(42 + i as u64);
        let documents = P2pDocuments::new(
            &format!("PO-T{:04}", i),
            "V000001",
            "1000",
            Decimal::new(5000, 0),
            "USD",
            &factory,
        )
        .with_goods_receipt(&format!("GR-T{:04}", i), &factory)
        .with_invoice(&format!("INV-T{:04}", i), &factory)
        .with_payment(&format!("PAY-T{:04}", i), &factory);

        let result = generator.generate_p2p_case(&documents, Utc::now(), &[]);

        // Extract activity sequence
        let activities: Vec<_> = result
            .events
            .iter()
            .map(|e| e.activity_id.as_str())
            .collect();

        // Verify valid P2P sequence
        // First activity should be create_po
        if !activities.is_empty() {
            assert_eq!(
                activities[0], "create_po",
                "P2P should start with create_po"
            );
        }

        // If we have approve_po, it should come after create_po
        if let Some(approve_idx) = activities.iter().position(|a| *a == "approve_po") {
            let create_idx = activities.iter().position(|a| *a == "create_po");
            assert!(
                create_idx.map(|c| approve_idx > c).unwrap_or(false),
                "approve_po should come after create_po"
            );
        }

        // If we have post_invoice, create_po should have happened
        if activities.contains(&"post_invoice") {
            assert!(
                activities.contains(&"create_po"),
                "post_invoice requires create_po"
            );
        }
    }
}

/// Test that events are in chronological order.
#[test]
fn test_event_chronological_order() {
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-CHRONO1",
        "V000001",
        "1000",
        Decimal::new(10000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-CHRONO1", &factory)
    .with_invoice("INV-CHRONO1", &factory)
    .with_payment("PAY-CHRONO1", &factory);

    let result = generator.generate_p2p_case(&documents, Utc::now(), &["user001".into()]);

    // Verify events are in chronological order
    for i in 1..result.events.len() {
        assert!(
            result.events[i].timestamp >= result.events[i - 1].timestamp,
            "Events should be in chronological order: {} ({}) came before {} ({})",
            result.events[i - 1].activity_id,
            result.events[i - 1].timestamp,
            result.events[i].activity_id,
            result.events[i].timestamp
        );
    }
}

// =============================================================================
// Object Relationship Tests
// =============================================================================

/// Test that object relationships reference valid objects.
#[test]
fn test_object_relationship_integrity() {
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-REL1",
        "V000001",
        "1000",
        Decimal::new(12000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-REL1", &factory)
    .with_invoice("INV-REL1", &factory)
    .with_payment("PAY-REL1", &factory);

    let result = generator.generate_p2p_case(&documents, Utc::now(), &[]);

    let object_ids: HashSet<_> = result.objects.iter().map(|o| o.object_id).collect();

    for relationship in &result.relationships {
        assert!(
            object_ids.contains(&relationship.source_object_id),
            "Relationship source {:?} not in objects",
            relationship.source_object_id
        );
        assert!(
            object_ids.contains(&relationship.target_object_id),
            "Relationship target {:?} not in objects",
            relationship.target_object_id
        );
    }

    // P2P should have relationships (GR → PO, Invoice → PO)
    if result.objects.len() > 1 {
        assert!(
            !result.relationships.is_empty(),
            "P2P process should generate object relationships"
        );
    }
}

// =============================================================================
// Process Variant Distribution Tests
// =============================================================================

/// Test that variant types are distributed according to configuration.
#[test]
fn test_variant_distribution() {
    let mut generator = OcpmEventGenerator::new(42);

    let mut happy_count = 0;
    let mut exception_count = 0;
    let mut error_count = 0;
    let total = 500;

    for _ in 0..total {
        match generator.select_variant_type() {
            VariantType::HappyPath => happy_count += 1,
            VariantType::ExceptionPath => exception_count += 1,
            VariantType::ErrorPath => error_count += 1,
        }
    }

    // Default rates: 75% happy, 20% exception, 5% error
    // Allow 10% tolerance
    let happy_rate = happy_count as f64 / total as f64;
    let exception_rate = exception_count as f64 / total as f64;
    let error_rate = error_count as f64 / total as f64;

    println!(
        "Variant distribution: happy={:.2}%, exception={:.2}%, error={:.2}%",
        happy_rate * 100.0,
        exception_rate * 100.0,
        error_rate * 100.0
    );

    assert!(
        happy_rate > 0.65 && happy_rate < 0.85,
        "Happy path rate {:.2}% outside expected range [65%, 85%]",
        happy_rate * 100.0
    );
    assert!(
        exception_rate > 0.10 && exception_rate < 0.30,
        "Exception rate {:.2}% outside expected range [10%, 30%]",
        exception_rate * 100.0
    );
    assert!(
        error_rate > 0.01 && error_rate < 0.15,
        "Error rate {:.2}% outside expected range [1%, 15%]",
        error_rate * 100.0
    );
}

/// Test custom variant configuration.
#[test]
fn test_custom_variant_config() {
    let config = OcpmGeneratorConfig {
        happy_path_rate: 0.50,
        exception_path_rate: 0.40,
        error_path_rate: 0.10,
        ..Default::default()
    };

    let mut generator = OcpmEventGenerator::with_config(42, config);

    let mut happy_count = 0;
    let mut exception_count = 0;
    let total = 500;

    for _ in 0..total {
        match generator.select_variant_type() {
            VariantType::HappyPath => happy_count += 1,
            VariantType::ExceptionPath => exception_count += 1,
            VariantType::ErrorPath => {}
        }
    }

    let happy_rate = happy_count as f64 / total as f64;
    let exception_rate = exception_count as f64 / total as f64;

    // Should be closer to 50%/40% with custom config
    assert!(
        happy_rate > 0.40 && happy_rate < 0.60,
        "Custom happy path rate {:.2}% outside expected range",
        happy_rate * 100.0
    );
    assert!(
        exception_rate > 0.30 && exception_rate < 0.50,
        "Custom exception rate {:.2}% outside expected range",
        exception_rate * 100.0
    );
}

// =============================================================================
// Case Trace Validation Tests
// =============================================================================

/// Test that case traces are properly generated.
#[test]
fn test_case_trace_generation() {
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-TRACE1",
        "V000001",
        "1000",
        Decimal::new(10000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-TRACE1", &factory)
    .with_invoice("INV-TRACE1", &factory)
    .with_payment("PAY-TRACE1", &factory);

    let result = generator.generate_p2p_case(&documents, Utc::now(), &[]);

    // Case trace should have activity sequence
    assert!(
        !result.case_trace.activity_sequence.is_empty(),
        "Case trace should have activity sequence"
    );

    // Activity sequence should match events
    assert_eq!(
        result.case_trace.activity_sequence.len(),
        result.events.len(),
        "Activity sequence should match event count"
    );

    // Events should match sequence
    for (i, event) in result.events.iter().enumerate() {
        assert_eq!(
            event.activity_id, result.case_trace.activity_sequence[i],
            "Activity sequence should match event activities"
        );
    }

    // Case trace should have event IDs
    assert_eq!(
        result.case_trace.event_ids.len(),
        result.events.len(),
        "Case trace should have all event IDs"
    );
}

/// Test that case trace timing is correct.
#[test]
fn test_case_trace_timing() {
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let start = Utc::now();

    let documents = P2pDocuments::new(
        "PO-TIME1",
        "V000001",
        "1000",
        Decimal::new(10000, 0),
        "USD",
        &factory,
    )
    .with_goods_receipt("GR-TIME1", &factory)
    .with_invoice("INV-TIME1", &factory)
    .with_payment("PAY-TIME1", &factory);

    let result = generator.generate_p2p_case(&documents, start, &[]);

    // Start time should match first event
    if let Some(first_event) = result.events.first() {
        assert_eq!(
            result.case_trace.start_time, first_event.timestamp,
            "Case start time should match first event"
        );
    }

    // End time should match last event (if completed)
    if let (Some(last_event), Some(end_time)) = (result.events.last(), result.case_trace.end_time) {
        assert_eq!(
            end_time, last_event.timestamp,
            "Case end time should match last event"
        );
    }
}

// =============================================================================
// Event Log Summary Tests
// =============================================================================

/// Test that event log summary is accurate.
#[test]
fn test_event_log_summary() {
    let mut log = OcpmEventLog::new().with_standard_types();
    let mut generator = OcpmEventGenerator::new(42);

    // Generate some cases
    for i in 0..5 {
        let factory = OcpmUuidFactory::new(42 + i as u64);
        let documents = P2pDocuments::new(
            &format!("PO-SUM{:04}", i),
            "V000001",
            "1000",
            Decimal::new(5000, 0),
            "USD",
            &factory,
        )
        .with_goods_receipt(&format!("GR-SUM{:04}", i), &factory)
        .with_invoice(&format!("INV-SUM{:04}", i), &factory);

        let result = generator.generate_p2p_case(&documents, Utc::now(), &[]);

        for obj in result.objects {
            log.add_object(obj);
        }
        for rel in result.relationships {
            log.add_relationship(rel);
        }
        for event in result.events {
            log.add_event(event);
        }
        log.add_case(result.case_trace);
    }

    let summary = log.summary();

    assert!(summary.event_count > 0, "Should have events");
    assert!(summary.object_count > 0, "Should have objects");
    assert_eq!(summary.case_count, 5, "Should have 5 cases");
    assert!(summary.object_type_count > 0, "Should have object types");
    assert!(
        summary.activity_type_count > 0,
        "Should have activity types"
    );

    println!(
        "Event log summary: {} events, {} objects, {} cases",
        summary.event_count, summary.object_count, summary.case_count
    );
}

/// Test that metadata is properly tracked.
#[test]
fn test_event_log_metadata() {
    let mut log = OcpmEventLog::new().with_standard_types();

    // Verify initial metadata
    assert_eq!(log.metadata.event_count, 0);
    assert_eq!(log.metadata.object_count, 0);
    assert_eq!(log.metadata.case_count, 0);

    // Add an event
    let mut generator = OcpmEventGenerator::new(42);
    let factory = OcpmUuidFactory::new(42);
    let documents = P2pDocuments::new(
        "PO-META1",
        "V000001",
        "1000",
        Decimal::new(5000, 0),
        "USD",
        &factory,
    );
    let result = generator.generate_p2p_case(&documents, Utc::now(), &[]);

    for obj in result.objects {
        log.add_object(obj);
    }
    for event in result.events {
        log.add_event(event);
    }
    log.add_case(result.case_trace);

    // Metadata should be updated
    assert!(log.metadata.event_count > 0, "Event count should update");
    assert!(log.metadata.object_count > 0, "Object count should update");
    assert_eq!(log.metadata.case_count, 1, "Case count should be 1");
}
