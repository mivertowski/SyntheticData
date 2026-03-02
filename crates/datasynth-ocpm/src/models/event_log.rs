//! Event log model for OCPM.
//!
//! The event log is the main container for all OCPM data, including
//! events, objects, relationships, resources, and variants.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use super::{
    ActivityType, CaseTrace, CorrelationEvent, ObjectGraph, ObjectInstance, ObjectRelationship,
    ObjectType, OcpmEvent, ProcessVariant, RelationshipIndex, Resource,
};

/// Complete OCPM event log with all events, objects, and relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcpmEventLog {
    /// Log metadata
    pub metadata: EventLogMetadata,
    /// Object type definitions
    pub object_types: HashMap<String, ObjectType>,
    /// Activity type definitions
    pub activity_types: HashMap<String, ActivityType>,
    /// All object instances
    pub objects: ObjectGraph,
    /// All object relationships
    pub object_relationships: RelationshipIndex,
    /// All events
    pub events: Vec<OcpmEvent>,
    /// Resources
    pub resources: HashMap<String, Resource>,
    /// Process variants (computed)
    pub variants: HashMap<String, ProcessVariant>,
    /// Case traces
    pub cases: HashMap<Uuid, CaseTrace>,
    /// Correlation events (three-way match, payment allocation, etc.)
    pub correlation_events: Vec<CorrelationEvent>,
    /// Event index by object ID
    #[serde(skip)]
    events_by_object: HashMap<Uuid, Vec<usize>>,
    /// Event index by activity
    #[serde(skip)]
    events_by_activity: HashMap<String, Vec<usize>>,
    /// Event index by date
    #[serde(skip)]
    events_by_date: BTreeMap<NaiveDate, Vec<usize>>,
}

impl Default for OcpmEventLog {
    fn default() -> Self {
        Self::new()
    }
}

impl OcpmEventLog {
    /// Create a new empty event log.
    pub fn new() -> Self {
        Self {
            metadata: EventLogMetadata::default(),
            object_types: HashMap::new(),
            activity_types: HashMap::new(),
            objects: ObjectGraph::new(),
            object_relationships: RelationshipIndex::new(),
            events: Vec::new(),
            resources: HashMap::new(),
            variants: HashMap::new(),
            cases: HashMap::new(),
            correlation_events: Vec::new(),
            events_by_object: HashMap::new(),
            events_by_activity: HashMap::new(),
            events_by_date: BTreeMap::new(),
        }
    }

    /// Create with metadata.
    pub fn with_metadata(metadata: EventLogMetadata) -> Self {
        Self {
            metadata,
            ..Self::new()
        }
    }

    /// Register an object type.
    pub fn register_object_type(&mut self, object_type: ObjectType) {
        self.object_types
            .insert(object_type.type_id.clone(), object_type);
    }

    /// Register an activity type.
    pub fn register_activity_type(&mut self, activity_type: ActivityType) {
        self.activity_types
            .insert(activity_type.activity_id.clone(), activity_type);
    }

    /// Register a resource.
    pub fn register_resource(&mut self, resource: Resource) {
        self.resources
            .insert(resource.resource_id.clone(), resource);
    }

    /// Add an object.
    pub fn add_object(&mut self, object: ObjectInstance) {
        self.objects.add_object(object);
        self.metadata.object_count = self.objects.len();
    }

    /// Add an object relationship.
    pub fn add_relationship(&mut self, relationship: ObjectRelationship) {
        self.object_relationships.add(relationship);
    }

    /// Add an event and update indices.
    pub fn add_event(&mut self, event: OcpmEvent) {
        let idx = self.events.len();
        let date = event.timestamp.date_naive();

        // Index by object
        for obj_ref in &event.object_refs {
            self.events_by_object
                .entry(obj_ref.object_id)
                .or_default()
                .push(idx);
        }

        // Index by activity
        self.events_by_activity
            .entry(event.activity_id.clone())
            .or_default()
            .push(idx);

        // Index by date
        self.events_by_date.entry(date).or_default().push(idx);

        self.events.push(event);
        self.metadata.event_count = self.events.len();
    }

    /// Add a case trace.
    pub fn add_case(&mut self, case: CaseTrace) {
        self.cases.insert(case.case_id, case);
        self.metadata.case_count = self.cases.len();
    }

    /// Add a correlation event.
    pub fn add_correlation_event(&mut self, correlation: CorrelationEvent) {
        self.correlation_events.push(correlation);
    }

    /// Get events for an object.
    pub fn events_for_object(&self, object_id: Uuid) -> Vec<&OcpmEvent> {
        self.events_by_object
            .get(&object_id)
            .map(|indices| indices.iter().filter_map(|&i| self.events.get(i)).collect())
            .unwrap_or_default()
    }

    /// Get events for an activity.
    pub fn events_for_activity(&self, activity_id: &str) -> Vec<&OcpmEvent> {
        self.events_by_activity
            .get(activity_id)
            .map(|indices| indices.iter().filter_map(|&i| self.events.get(i)).collect())
            .unwrap_or_default()
    }

    /// Get events for a date range.
    pub fn events_in_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<&OcpmEvent> {
        self.events_by_date
            .range(start..=end)
            .flat_map(|(_, indices)| indices.iter().filter_map(|&i| self.events.get(i)))
            .collect()
    }

    /// Compute process variants from completed cases.
    pub fn compute_variants(&mut self) {
        let mut variant_map: HashMap<Vec<String>, ProcessVariant> = HashMap::new();
        let mut variant_counter = 0usize;

        for case in self.cases.values() {
            if case.is_completed() {
                let key = case.activity_sequence.clone();

                // Check if variant exists, if not create it
                if !variant_map.contains_key(&key) {
                    variant_counter += 1;
                    let mut v = ProcessVariant::new(
                        &format!("V{}", variant_counter),
                        case.business_process,
                    );
                    v.activity_sequence = key.clone();
                    variant_map.insert(key.clone(), v);
                }

                let variant = variant_map.get_mut(&key).expect("variant just inserted");

                if let Some(duration) = case.duration_hours() {
                    variant.add_case(case.case_id, duration);
                }
            }
        }

        // Calculate frequency percentages
        let total_cases: u64 = variant_map.values().map(|v| v.frequency).sum();
        for variant in variant_map.values_mut() {
            variant.frequency_percent = if total_cases > 0 {
                variant.frequency as f64 / total_cases as f64 * 100.0
            } else {
                0.0
            };
        }

        self.variants = variant_map
            .into_values()
            .map(|v| (v.variant_id.clone(), v))
            .collect();

        self.metadata.variant_count = self.variants.len();
    }

    /// Get summary statistics.
    pub fn summary(&self) -> EventLogSummary {
        EventLogSummary {
            event_count: self.events.len(),
            object_count: self.objects.len(),
            relationship_count: self.object_relationships.len(),
            case_count: self.cases.len(),
            variant_count: self.variants.len(),
            resource_count: self.resources.len(),
            object_type_count: self.object_types.len(),
            activity_type_count: self.activity_types.len(),
        }
    }

    /// Initialize with standard types for all process families.
    pub fn with_standard_types(mut self) -> Self {
        // Register P2P types
        for obj_type in ObjectType::p2p_types() {
            self.register_object_type(obj_type);
        }
        for activity in ActivityType::p2p_activities() {
            self.register_activity_type(activity);
        }

        // Register O2C types
        for obj_type in ObjectType::o2c_types() {
            self.register_object_type(obj_type);
        }
        for activity in ActivityType::o2c_activities() {
            self.register_activity_type(activity);
        }

        // Register R2R + A2R types
        for activity in ActivityType::r2r_activities() {
            self.register_activity_type(activity);
        }
        for activity in ActivityType::a2r_activities() {
            self.register_activity_type(activity);
        }

        // Register S2C types
        for obj_type in ObjectType::s2c_types() {
            self.register_object_type(obj_type);
        }
        for activity in ActivityType::s2c_activities() {
            self.register_activity_type(activity);
        }

        // Register H2R types
        for obj_type in ObjectType::h2r_types() {
            self.register_object_type(obj_type);
        }
        for activity in ActivityType::h2r_activities() {
            self.register_activity_type(activity);
        }

        // Register MFG types
        for obj_type in ObjectType::mfg_types() {
            self.register_object_type(obj_type);
        }
        for activity in ActivityType::mfg_activities() {
            self.register_activity_type(activity);
        }

        // Register BANK types
        for obj_type in ObjectType::bank_types() {
            self.register_object_type(obj_type);
        }
        for activity in ActivityType::bank_activities() {
            self.register_activity_type(activity);
        }

        // Register AUDIT types
        for obj_type in ObjectType::audit_types() {
            self.register_object_type(obj_type);
        }
        for activity in ActivityType::audit_activities() {
            self.register_activity_type(activity);
        }

        // Register Bank Reconciliation types (R2R subfamily)
        for obj_type in ObjectType::bank_recon_types() {
            self.register_object_type(obj_type);
        }
        for activity in ActivityType::bank_recon_activities() {
            self.register_activity_type(activity);
        }

        // Register standard resources
        self.register_resource(Resource::erp_system());
        self.register_resource(Resource::workflow_system());

        self
    }

    /// Rebuild indices (call after deserialization).
    pub fn rebuild_indices(&mut self) {
        self.events_by_object.clear();
        self.events_by_activity.clear();
        self.events_by_date.clear();

        for (idx, event) in self.events.iter().enumerate() {
            let date = event.timestamp.date_naive();

            for obj_ref in &event.object_refs {
                self.events_by_object
                    .entry(obj_ref.object_id)
                    .or_default()
                    .push(idx);
            }

            self.events_by_activity
                .entry(event.activity_id.clone())
                .or_default()
                .push(idx);

            self.events_by_date.entry(date).or_default().push(idx);
        }
    }
}

/// Event log metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventLogMetadata {
    /// Log name
    pub log_name: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Company codes included
    pub company_codes: Vec<String>,
    /// Start date of data
    pub start_date: Option<NaiveDate>,
    /// End date of data
    pub end_date: Option<NaiveDate>,
    /// Total event count
    pub event_count: usize,
    /// Total object count
    pub object_count: usize,
    /// Total case count
    pub case_count: usize,
    /// Total variant count
    pub variant_count: usize,
    /// Generator version
    pub generator_version: String,
}

impl EventLogMetadata {
    /// Create new metadata.
    pub fn new(log_name: &str) -> Self {
        Self {
            log_name: log_name.into(),
            created_at: Utc::now(),
            generator_version: env!("CARGO_PKG_VERSION").into(),
            ..Default::default()
        }
    }
}

/// Summary statistics for an event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogSummary {
    pub event_count: usize,
    pub object_count: usize,
    pub relationship_count: usize,
    pub case_count: usize,
    pub variant_count: usize,
    pub resource_count: usize,
    pub object_type_count: usize,
    pub activity_type_count: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_event_log_creation() {
        let log = OcpmEventLog::new().with_standard_types();

        assert!(!log.object_types.is_empty());
        assert!(!log.activity_types.is_empty());
        assert!(!log.resources.is_empty());
    }

    #[test]
    fn test_event_log_summary() {
        let log = OcpmEventLog::new();
        let summary = log.summary();

        assert_eq!(summary.event_count, 0);
        assert_eq!(summary.object_count, 0);
    }
}
