//! Core OCPM event generator.
//!
//! Generates OCPM events from document flows and business processes.
//!
//! Uses the centralized `DeterministicUuidFactory` from `datasynth-core`
//! for reproducible, collision-free UUID generation.

use chrono::{DateTime, Duration, Utc};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use uuid::Uuid;

use crate::models::{
    ActivityType, CaseTrace, CorrelationEvent, EventLifecycle, EventObjectRef,
    LifecycleStateMachine, ObjectAttributeValue, ObjectInstance, ObjectQualifier,
    ObjectRelationship, ObjectType, OcpmEvent, ResourcePool,
};
use datasynth_core::models::BusinessProcess;
use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
use std::collections::HashMap;

/// Sub-discriminator values for OCPM UUID streams.
///
/// These separate the OCPM UUID namespaces (Case, Event, Object, Document, Relationship)
/// while reusing the `DocumentFlow` generator type from the centralized factory.
const OCPM_SUB_DISC_CASE: u8 = 0xC0;
const OCPM_SUB_DISC_EVENT: u8 = 0xE0;
const OCPM_SUB_DISC_OBJECT: u8 = 0xB0;
/// Sub-discriminator for document-level UUIDs (used in document struct builders).
const OCPM_SUB_DISC_DOCUMENT: u8 = 0xD0;
/// Sub-discriminator for relationship UUIDs.
const OCPM_SUB_DISC_RELATIONSHIP: u8 = 0xA0;

/// Deterministic UUID factory for OCPM, wrapping the centralized
/// `DeterministicUuidFactory` from `datasynth-core`.
///
/// Maintains separate UUID streams (Case, Event, Object, Document, Relationship)
/// via sub-discriminators to prevent collisions between different entity types.
#[derive(Debug, Clone)]
pub struct OcpmUuidFactory {
    case_factory: DeterministicUuidFactory,
    event_factory: DeterministicUuidFactory,
    object_factory: DeterministicUuidFactory,
    document_factory: DeterministicUuidFactory,
    relationship_factory: DeterministicUuidFactory,
}

impl OcpmUuidFactory {
    /// Create a new OCPM UUID factory with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            case_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::DocumentFlow,
                OCPM_SUB_DISC_CASE,
            ),
            event_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::DocumentFlow,
                OCPM_SUB_DISC_EVENT,
            ),
            object_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::DocumentFlow,
                OCPM_SUB_DISC_OBJECT,
            ),
            document_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::DocumentFlow,
                OCPM_SUB_DISC_DOCUMENT,
            ),
            relationship_factory: DeterministicUuidFactory::with_sub_discriminator(
                seed,
                GeneratorType::DocumentFlow,
                OCPM_SUB_DISC_RELATIONSHIP,
            ),
        }
    }

    /// Generate the next case UUID.
    pub fn next_case_id(&self) -> Uuid {
        self.case_factory.next()
    }

    /// Generate the next event UUID.
    pub fn next_event_id(&self) -> Uuid {
        self.event_factory.next()
    }

    /// Generate the next object UUID.
    pub fn next_object_id(&self) -> Uuid {
        self.object_factory.next()
    }

    /// Generate the next document UUID (for document struct builders).
    pub fn next_document_id(&self) -> Uuid {
        self.document_factory.next()
    }

    /// Generate the next relationship UUID.
    pub fn next_relationship_id(&self) -> Uuid {
        self.relationship_factory.next()
    }

    /// Get the current case counter.
    pub fn case_count(&self) -> u64 {
        self.case_factory.current_counter()
    }

    /// Get the current event counter.
    pub fn event_count(&self) -> u64 {
        self.event_factory.current_counter()
    }

    /// Get the current object counter.
    pub fn object_count(&self) -> u64 {
        self.object_factory.current_counter()
    }
}

/// Configuration for OCPM event generation.
#[derive(Debug, Clone)]
pub struct OcpmGeneratorConfig {
    /// Enable P2P process events
    pub generate_p2p: bool,
    /// Enable O2C process events
    pub generate_o2c: bool,
    /// Enable S2C process events
    pub generate_s2c: bool,
    /// Enable H2R process events
    pub generate_h2r: bool,
    /// Enable MFG process events
    pub generate_mfg: bool,
    /// Enable Bank Reconciliation process events
    pub generate_bank_recon: bool,
    /// Enable Banking process events
    pub generate_bank: bool,
    /// Enable Audit process events
    pub generate_audit: bool,
    /// Rate of happy path (normal) variants
    pub happy_path_rate: f64,
    /// Rate of exception path variants
    pub exception_path_rate: f64,
    /// Rate of error path variants
    pub error_path_rate: f64,
    /// Add duration variability to events
    pub add_duration_variability: bool,
    /// Standard deviation factor for duration
    pub duration_std_dev_factor: f64,
}

impl Default for OcpmGeneratorConfig {
    fn default() -> Self {
        Self {
            generate_p2p: true,
            generate_o2c: true,
            generate_s2c: true,
            generate_h2r: true,
            generate_mfg: true,
            generate_bank_recon: true,
            generate_bank: true,
            generate_audit: true,
            happy_path_rate: 0.75,
            exception_path_rate: 0.20,
            error_path_rate: 0.05,
            add_duration_variability: true,
            duration_std_dev_factor: 0.3,
        }
    }
}

/// Main OCPM event generator.
pub struct OcpmEventGenerator {
    /// Random number generator
    rng: ChaCha8Rng,
    /// Configuration
    config: OcpmGeneratorConfig,
    /// P2P activity types
    p2p_activities: Vec<ActivityType>,
    /// O2C activity types
    o2c_activities: Vec<ActivityType>,
    /// S2C activity types
    s2c_activities: Vec<ActivityType>,
    /// H2R activity types
    h2r_activities: Vec<ActivityType>,
    /// MFG activity types
    mfg_activities: Vec<ActivityType>,
    /// Bank Recon activity types
    bank_recon_activities: Vec<ActivityType>,
    /// Banking activity types
    bank_activities: Vec<ActivityType>,
    /// Audit activity types
    audit_activities: Vec<ActivityType>,
    /// Deterministic UUID factory for reproducible generation
    uuid_factory: OcpmUuidFactory,
    /// Lifecycle state machines keyed by object type
    state_machines: HashMap<String, LifecycleStateMachine>,
    /// Resource pools for assignment
    resource_pools: Vec<ResourcePool>,
}

impl OcpmEventGenerator {
    /// Create a new OCPM event generator with a seed.
    pub fn new(seed: u64) -> Self {
        use crate::models::{all_state_machines, default_resource_pools};

        let sm_map = all_state_machines()
            .into_iter()
            .map(|sm| (sm.object_type.clone(), sm))
            .collect();

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config: OcpmGeneratorConfig::default(),
            p2p_activities: ActivityType::p2p_activities(),
            o2c_activities: ActivityType::o2c_activities(),
            s2c_activities: ActivityType::s2c_activities(),
            h2r_activities: ActivityType::h2r_activities(),
            mfg_activities: ActivityType::mfg_activities(),
            bank_recon_activities: ActivityType::bank_recon_activities(),
            bank_activities: ActivityType::bank_activities(),
            audit_activities: ActivityType::audit_activities(),
            uuid_factory: OcpmUuidFactory::new(seed),
            state_machines: sm_map,
            resource_pools: default_resource_pools(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: OcpmGeneratorConfig) -> Self {
        use crate::models::{all_state_machines, default_resource_pools};

        let sm_map = all_state_machines()
            .into_iter()
            .map(|sm| (sm.object_type.clone(), sm))
            .collect();

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            p2p_activities: ActivityType::p2p_activities(),
            o2c_activities: ActivityType::o2c_activities(),
            s2c_activities: ActivityType::s2c_activities(),
            h2r_activities: ActivityType::h2r_activities(),
            mfg_activities: ActivityType::mfg_activities(),
            bank_recon_activities: ActivityType::bank_recon_activities(),
            bank_activities: ActivityType::bank_activities(),
            audit_activities: ActivityType::audit_activities(),
            uuid_factory: OcpmUuidFactory::new(seed),
            state_machines: sm_map,
            resource_pools: default_resource_pools(),
        }
    }

    /// Generate a new deterministic case ID.
    pub fn new_case_id(&self) -> Uuid {
        self.uuid_factory.next_case_id()
    }

    /// Generate a new deterministic event ID.
    pub fn new_event_id(&self) -> Uuid {
        self.uuid_factory.next_event_id()
    }

    /// Generate a new deterministic object ID.
    pub fn new_object_id(&self) -> Uuid {
        self.uuid_factory.next_object_id()
    }

    /// Generate a new deterministic document UUID (for document struct builders).
    pub fn next_document_id(&self) -> Uuid {
        self.uuid_factory.next_document_id()
    }

    /// Generate a new deterministic relationship UUID.
    pub fn next_relationship_id(&self) -> Uuid {
        self.uuid_factory.next_relationship_id()
    }

    /// Get access to the UUID factory for advanced use cases.
    pub fn uuid_factory(&self) -> &OcpmUuidFactory {
        &self.uuid_factory
    }

    /// Select a process variant type based on configuration.
    pub fn select_variant_type(&mut self) -> VariantType {
        let r: f64 = self.rng.random();
        if r < self.config.happy_path_rate {
            VariantType::HappyPath
        } else if r < self.config.happy_path_rate + self.config.exception_path_rate {
            VariantType::ExceptionPath
        } else {
            VariantType::ErrorPath
        }
    }

    /// Calculate event timestamp with variability.
    pub fn calculate_event_time(
        &mut self,
        base_time: DateTime<Utc>,
        activity: &ActivityType,
    ) -> DateTime<Utc> {
        if let Some(typical_minutes) = activity.typical_duration_minutes {
            let std_dev = activity.duration_std_dev.unwrap_or(typical_minutes * 0.3);

            if self.config.add_duration_variability {
                // Add some variability using normal-like distribution
                let variability: f64 = self.rng.random_range(-2.0..2.0) * std_dev;
                let actual_minutes = (typical_minutes + variability).max(1.0);
                base_time + Duration::minutes(actual_minutes as i64)
            } else {
                base_time + Duration::minutes(typical_minutes as i64)
            }
        } else {
            base_time + Duration::minutes(5) // Default 5 minutes
        }
    }

    /// Create an event from an activity type, using the deterministic UUID factory.
    pub fn create_event(
        &mut self,
        activity: &ActivityType,
        timestamp: DateTime<Utc>,
        resource_id: &str,
        company_code: &str,
        case_id: Uuid,
    ) -> OcpmEvent {
        let event_id = self.uuid_factory.next_event_id();
        OcpmEvent::new(
            &activity.activity_id,
            &activity.name,
            timestamp,
            resource_id,
            company_code,
        )
        .with_id(event_id)
        .with_case(case_id)
        .with_lifecycle(if activity.is_automated {
            EventLifecycle::Atomic
        } else {
            EventLifecycle::Complete
        })
    }

    /// Create an object instance for a document, using the deterministic UUID factory.
    pub fn create_object(
        &self,
        object_type: &ObjectType,
        external_id: &str,
        company_code: &str,
        created_at: DateTime<Utc>,
    ) -> ObjectInstance {
        let object_id = self.uuid_factory.next_object_id();
        ObjectInstance::new(&object_type.type_id, external_id, company_code)
            .with_id(object_id)
            .with_state("active")
            .with_created_at(created_at)
    }

    /// Create an object relationship with a deterministic UUID.
    pub fn create_relationship(
        &self,
        relationship_type: &str,
        source_object_id: Uuid,
        source_type_id: &str,
        target_object_id: Uuid,
        target_type_id: &str,
    ) -> ObjectRelationship {
        let rel_id = self.uuid_factory.next_relationship_id();
        ObjectRelationship::new(
            relationship_type,
            source_object_id,
            source_type_id,
            target_object_id,
            target_type_id,
        )
        .with_id(rel_id)
    }

    /// Create object reference for an event.
    pub fn create_object_ref(
        &self,
        object: &ObjectInstance,
        qualifier: ObjectQualifier,
    ) -> EventObjectRef {
        EventObjectRef::new(object.object_id, &object.object_type_id, qualifier)
            .with_external_id(&object.external_id)
    }

    /// Add an attribute to an event.
    pub fn add_event_attribute(event: &mut OcpmEvent, key: &str, value: ObjectAttributeValue) {
        event.attributes.insert(key.into(), value);
    }

    /// Generate a complete case trace from events, using the deterministic UUID factory.
    pub fn create_case_trace(
        &self,
        _case_id: Uuid,
        events: &[OcpmEvent],
        business_process: BusinessProcess,
        primary_object_id: Uuid,
        primary_object_type: &str,
        company_code: &str,
    ) -> CaseTrace {
        let activity_sequence: Vec<String> = events.iter().map(|e| e.activity_id.clone()).collect();

        let start_time = events.first().map(|e| e.timestamp).unwrap_or_else(Utc::now);
        let end_time = events.last().map(|e| e.timestamp);

        let case_trace_id = self.uuid_factory.next_case_id();
        let mut trace = CaseTrace::new(
            business_process,
            primary_object_id,
            primary_object_type,
            company_code,
        )
        .with_id(case_trace_id);
        trace.activity_sequence = activity_sequence;
        trace.event_ids = events.iter().map(|e| e.event_id).collect();
        trace.start_time = start_time;
        trace.end_time = end_time;
        trace
    }

    /// Select a resource for an activity.
    pub fn select_resource(
        &mut self,
        activity: &ActivityType,
        available_users: &[String],
    ) -> String {
        if activity.is_automated {
            "SYSTEM".into()
        } else if available_users.is_empty() {
            format!("USER{:04}", self.rng.random_range(1..100))
        } else {
            let idx = self.rng.random_range(0..available_users.len());
            available_users[idx].clone()
        }
    }

    /// Get P2P activities.
    pub fn p2p_activities(&self) -> &[ActivityType] {
        &self.p2p_activities
    }

    /// Get O2C activities.
    pub fn o2c_activities(&self) -> &[ActivityType] {
        &self.o2c_activities
    }

    /// Get S2C activities.
    pub fn s2c_activities(&self) -> &[ActivityType] {
        &self.s2c_activities
    }

    /// Get H2R activities.
    pub fn h2r_activities(&self) -> &[ActivityType] {
        &self.h2r_activities
    }

    /// Get MFG activities.
    pub fn mfg_activities(&self) -> &[ActivityType] {
        &self.mfg_activities
    }

    /// Get Bank Reconciliation activities.
    pub fn bank_recon_activities(&self) -> &[ActivityType] {
        &self.bank_recon_activities
    }

    /// Get Banking activities.
    pub fn bank_activities(&self) -> &[ActivityType] {
        &self.bank_activities
    }

    /// Get Audit activities.
    pub fn audit_activities(&self) -> &[ActivityType] {
        &self.audit_activities
    }

    /// Get generator config.
    pub fn config(&self) -> &OcpmGeneratorConfig {
        &self.config
    }

    /// Generate random delay between activities (in minutes).
    pub fn generate_inter_activity_delay(
        &mut self,
        min_minutes: i64,
        max_minutes: i64,
    ) -> Duration {
        let minutes = self.rng.random_range(min_minutes..=max_minutes);
        Duration::minutes(minutes)
    }

    /// Check if an activity should be skipped (for exception paths).
    pub fn should_skip_activity(&mut self, skip_probability: f64) -> bool {
        self.rng.random::<f64>() < skip_probability
    }

    /// Generate a random boolean with given probability.
    pub fn random_bool(&mut self, probability: f64) -> bool {
        self.rng.random::<f64>() < probability
    }

    /// Look up state transition for an activity on an object type using the
    /// lifecycle state machine.
    ///
    /// Returns `(from_state, to_state)` if a matching transition is found.
    pub fn find_state_transition(
        &self,
        object_type: &str,
        activity_name: &str,
    ) -> Option<(String, String)> {
        if let Some(sm) = self.state_machines.get(object_type) {
            for t in &sm.transitions {
                if t.activity_name == activity_name {
                    return Some((t.from_state.clone(), t.to_state.clone()));
                }
            }
        }
        None
    }

    /// Assign a resource from the named pool, returning the resource ID and
    /// the resource's current workload after assignment.
    pub fn assign_resource_from_pool(&mut self, pool_id: &str) -> Option<(String, f64)> {
        for pool in &mut self.resource_pools {
            if pool.pool_id == pool_id {
                if let Some(resource_id) = pool.assign() {
                    let resource_id = resource_id.to_string();
                    let workload = pool
                        .resources
                        .iter()
                        .find(|r| r.resource_id == resource_id)
                        .map(|r| r.current_workload)
                        .unwrap_or(0.0);
                    return Some((resource_id, workload));
                }
            }
        }
        None
    }

    /// Enrich an event with state transition and resource workload.
    ///
    /// Uses the `ActivityType`'s own state transitions (which define the primary
    /// object type's from_state → to_state for this activity) and assigns a
    /// resource from the specified pool for workload tracking.
    pub fn enrich_event(
        &mut self,
        event: OcpmEvent,
        activity: &ActivityType,
        pool_id: &str,
    ) -> OcpmEvent {
        let mut event = event;

        // Use the activity's first state transition (primary object type)
        if let Some(transition) = activity.state_transitions.first() {
            let from = transition.from_state.as_deref().unwrap_or("initial");
            event = event.with_state_transition(from, &transition.to_state);
        }

        // Add resource workload from pool
        if let Some((_resource_id, workload)) = self.assign_resource_from_pool(pool_id) {
            event = event.with_resource_workload(workload);
        }

        event
    }

    /// Get a reference to the lifecycle state machines.
    pub fn state_machines(&self) -> &HashMap<String, LifecycleStateMachine> {
        &self.state_machines
    }
}

/// Type of process variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantType {
    /// Normal/happy path - all activities completed successfully
    HappyPath,
    /// Exception path - some activities skipped or modified
    ExceptionPath,
    /// Error path - process aborted or failed
    ErrorPath,
}

impl VariantType {
    /// Get a description of this variant type.
    pub fn description(&self) -> &'static str {
        match self {
            Self::HappyPath => "Standard process execution",
            Self::ExceptionPath => "Process with exceptions or variations",
            Self::ErrorPath => "Process failed or aborted",
        }
    }
}

/// Result of generating events for a case.
#[derive(Debug)]
pub struct CaseGenerationResult {
    /// Generated events
    pub events: Vec<OcpmEvent>,
    /// Generated objects
    pub objects: Vec<ObjectInstance>,
    /// Generated relationships
    pub relationships: Vec<ObjectRelationship>,
    /// Case trace
    pub case_trace: CaseTrace,
    /// Variant type used
    pub variant_type: VariantType,
    /// Correlation events (three-way match, payment allocation, etc.)
    pub correlation_events: Vec<CorrelationEvent>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = OcpmEventGenerator::new(42);
        assert!(!generator.p2p_activities.is_empty());
        assert!(!generator.o2c_activities.is_empty());
    }

    #[test]
    fn test_variant_selection() {
        let mut generator = OcpmEventGenerator::new(42);

        // Generate many variants and check distribution
        let mut happy = 0;
        let mut exception = 0;
        let mut error = 0;

        for _ in 0..1000 {
            match generator.select_variant_type() {
                VariantType::HappyPath => happy += 1,
                VariantType::ExceptionPath => exception += 1,
                VariantType::ErrorPath => error += 1,
            }
        }

        // Should be roughly 75%/20%/5%
        assert!(happy > 600 && happy < 850);
        assert!(exception > 100 && exception < 300);
        assert!(error > 10 && error < 100);
    }

    #[test]
    fn test_case_id_generation() {
        let generator = OcpmEventGenerator::new(42);
        let id1 = generator.new_case_id();
        let id2 = generator.new_case_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_deterministic_uuid_generation() {
        // Two generators with the same seed should produce identical UUIDs
        let gen1 = OcpmEventGenerator::new(12345);
        let gen2 = OcpmEventGenerator::new(12345);

        // Case IDs should be identical
        assert_eq!(gen1.new_case_id(), gen2.new_case_id());
        assert_eq!(gen1.new_case_id(), gen2.new_case_id());

        // Event IDs should be identical
        assert_eq!(gen1.new_event_id(), gen2.new_event_id());

        // Object IDs should be identical
        assert_eq!(gen1.new_object_id(), gen2.new_object_id());
    }

    #[test]
    fn test_different_seeds_produce_different_uuids() {
        let gen1 = OcpmEventGenerator::new(12345);
        let gen2 = OcpmEventGenerator::new(67890);

        assert_ne!(gen1.new_case_id(), gen2.new_case_id());
        assert_ne!(gen1.new_event_id(), gen2.new_event_id());
        assert_ne!(gen1.new_object_id(), gen2.new_object_id());
    }

    #[test]
    fn test_uuid_factory_counters() {
        let generator = OcpmEventGenerator::new(42);

        assert_eq!(generator.uuid_factory().case_count(), 0);
        generator.new_case_id();
        generator.new_case_id();
        assert_eq!(generator.uuid_factory().case_count(), 2);

        assert_eq!(generator.uuid_factory().event_count(), 0);
        generator.new_event_id();
        assert_eq!(generator.uuid_factory().event_count(), 1);
    }

    #[test]
    fn test_event_creation() {
        let mut generator = OcpmEventGenerator::new(42);
        let activity = ActivityType::create_po();
        let case_id = generator.new_case_id();

        let event = generator.create_event(&activity, Utc::now(), "user001", "1000", case_id);

        assert_eq!(event.activity_id, "create_po");
        assert_eq!(event.case_id, Some(case_id));
    }
}
