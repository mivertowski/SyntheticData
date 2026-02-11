//! Bi-temporal data model support for audit trail requirements.
//!
//! This module provides temporal wrappers and types for tracking both:
//! - Business validity (when the fact is true in the real world)
//! - System recording (when we recorded this in the system)
//!
//! This is critical for audit trails and point-in-time queries.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Bi-temporal wrapper for any auditable entity.
///
/// Provides two temporal dimensions:
/// - **Business time**: When the fact was/is true in the real world
/// - **System time**: When the fact was recorded in the system
///
/// # Example
///
/// ```ignore
/// use datasynth_core::models::BiTemporal;
///
/// // A journal entry that was valid from Jan 1 to Jan 15
/// // but was recorded on Jan 5 and corrected on Jan 16
/// let entry = BiTemporal::new(journal_entry)
///     .with_valid_time(jan_1, Some(jan_15))
///     .with_recorded_by("user001")
///     .with_change_reason("Initial posting");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiTemporal<T> {
    /// The wrapped data
    pub data: T,

    /// Unique version ID for this temporal record
    pub version_id: Uuid,

    // === Business Time (Valid Time) ===
    /// When this fact became true in the business world
    pub valid_from: NaiveDateTime,
    /// When this fact stopped being true (None = still current)
    pub valid_to: Option<NaiveDateTime>,

    // === System Time (Transaction Time) ===
    /// When this record was created in the system
    pub recorded_at: DateTime<Utc>,
    /// When this record was superseded by a newer version (None = current version)
    pub superseded_at: Option<DateTime<Utc>>,

    // === Audit Metadata ===
    /// User/system that recorded this version
    pub recorded_by: String,
    /// Reason for change (for corrections/adjustments)
    pub change_reason: Option<String>,
    /// Previous version ID (for version chain)
    pub previous_version_id: Option<Uuid>,
    /// Change type classification
    pub change_type: TemporalChangeType,
}

impl<T> BiTemporal<T> {
    /// Create a new bi-temporal record with current timestamps.
    pub fn new(data: T) -> Self {
        let now = Utc::now();
        Self {
            data,
            version_id: Uuid::new_v4(),
            valid_from: now.naive_utc(),
            valid_to: None,
            recorded_at: now,
            superseded_at: None,
            recorded_by: String::new(),
            change_reason: None,
            previous_version_id: None,
            change_type: TemporalChangeType::Original,
        }
    }

    /// Set the valid time range.
    pub fn with_valid_time(mut self, from: NaiveDateTime, to: Option<NaiveDateTime>) -> Self {
        self.valid_from = from;
        self.valid_to = to;
        self
    }

    /// Set valid_from only.
    pub fn valid_from(mut self, from: NaiveDateTime) -> Self {
        self.valid_from = from;
        self
    }

    /// Set valid_to only.
    pub fn valid_to(mut self, to: NaiveDateTime) -> Self {
        self.valid_to = Some(to);
        self
    }

    /// Set the recorded_at timestamp.
    pub fn with_recorded_at(mut self, recorded_at: DateTime<Utc>) -> Self {
        self.recorded_at = recorded_at;
        self
    }

    /// Set who recorded this version.
    pub fn with_recorded_by(mut self, recorded_by: &str) -> Self {
        self.recorded_by = recorded_by.into();
        self
    }

    /// Set the change reason.
    pub fn with_change_reason(mut self, reason: &str) -> Self {
        self.change_reason = Some(reason.into());
        self
    }

    /// Set the change type.
    pub fn with_change_type(mut self, change_type: TemporalChangeType) -> Self {
        self.change_type = change_type;
        self
    }

    /// Link to a previous version.
    pub fn with_previous_version(mut self, previous_id: Uuid) -> Self {
        self.previous_version_id = Some(previous_id);
        self
    }

    /// Check if this record is currently valid (business time).
    pub fn is_currently_valid(&self) -> bool {
        let now = Utc::now().naive_utc();
        self.valid_from <= now && self.valid_to.is_none_or(|to| to > now)
    }

    /// Check if this is the current version (system time).
    pub fn is_current_version(&self) -> bool {
        self.superseded_at.is_none()
    }

    /// Check if this record was valid at a specific business time.
    pub fn was_valid_at(&self, at: NaiveDateTime) -> bool {
        self.valid_from <= at && self.valid_to.is_none_or(|to| to > at)
    }

    /// Check if this version was the current version at a specific system time.
    pub fn was_current_at(&self, at: DateTime<Utc>) -> bool {
        self.recorded_at <= at && self.superseded_at.is_none_or(|sup| sup > at)
    }

    /// Supersede this record with a new version.
    pub fn supersede(&mut self, superseded_at: DateTime<Utc>) {
        self.superseded_at = Some(superseded_at);
    }

    /// Create a correction of this record.
    pub fn correct(&self, new_data: T, corrected_by: &str, reason: &str) -> Self
    where
        T: Clone,
    {
        let now = Utc::now();
        Self {
            data: new_data,
            version_id: Uuid::new_v4(),
            valid_from: self.valid_from,
            valid_to: self.valid_to,
            recorded_at: now,
            superseded_at: None,
            recorded_by: corrected_by.into(),
            change_reason: Some(reason.into()),
            previous_version_id: Some(self.version_id),
            change_type: TemporalChangeType::Correction,
        }
    }

    /// Create a reversal of this record.
    pub fn reverse(&self, reversed_by: &str, reason: &str) -> Self
    where
        T: Clone,
    {
        let now = Utc::now();
        Self {
            data: self.data.clone(),
            version_id: Uuid::new_v4(),
            valid_from: now.naive_utc(),
            valid_to: None,
            recorded_at: now,
            superseded_at: None,
            recorded_by: reversed_by.into(),
            change_reason: Some(reason.into()),
            previous_version_id: Some(self.version_id),
            change_type: TemporalChangeType::Reversal,
        }
    }

    /// Get a reference to the underlying data.
    pub fn inner(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the underlying data.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Consume and return the underlying data.
    pub fn into_inner(self) -> T {
        self.data
    }
}

/// Type of temporal change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TemporalChangeType {
    /// Original record
    #[default]
    Original,
    /// Correction of a previous record
    Correction,
    /// Reversal of a previous record
    Reversal,
    /// Adjustment (e.g., period-end adjustments)
    Adjustment,
    /// Reclassification
    Reclassification,
    /// Late posting (posted in subsequent period)
    LatePosting,
}

impl TemporalChangeType {
    /// Check if this is an error correction type.
    pub fn is_correction(&self) -> bool {
        matches!(self, Self::Correction | Self::Reversal)
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Original => "Original posting",
            Self::Correction => "Error correction",
            Self::Reversal => "Reversal entry",
            Self::Adjustment => "Period adjustment",
            Self::Reclassification => "Account reclassification",
            Self::LatePosting => "Late posting",
        }
    }
}

/// Temporal query parameters for point-in-time queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemporalQuery {
    /// Query as of this business time (None = current)
    pub as_of_valid_time: Option<NaiveDateTime>,
    /// Query as of this system time (None = current)
    pub as_of_system_time: Option<DateTime<Utc>>,
    /// Include superseded versions
    pub include_history: bool,
}

impl TemporalQuery {
    /// Query for current data.
    pub fn current() -> Self {
        Self::default()
    }

    /// Query as of a specific business time.
    pub fn as_of_valid(time: NaiveDateTime) -> Self {
        Self {
            as_of_valid_time: Some(time),
            ..Default::default()
        }
    }

    /// Query as of a specific system time.
    pub fn as_of_system(time: DateTime<Utc>) -> Self {
        Self {
            as_of_system_time: Some(time),
            ..Default::default()
        }
    }

    /// Query with full history.
    pub fn with_history() -> Self {
        Self {
            include_history: true,
            ..Default::default()
        }
    }
}

/// Temporal version chain for tracking all versions of an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalVersionChain<T> {
    /// Entity ID (stable across versions)
    pub entity_id: Uuid,
    /// All versions ordered by recorded_at
    pub versions: Vec<BiTemporal<T>>,
}

impl<T> TemporalVersionChain<T> {
    /// Create a new version chain with an initial version.
    pub fn new(entity_id: Uuid, initial: BiTemporal<T>) -> Self {
        Self {
            entity_id,
            versions: vec![initial],
        }
    }

    /// Get the current version.
    pub fn current(&self) -> Option<&BiTemporal<T>> {
        self.versions.iter().find(|v| v.is_current_version())
    }

    /// Get the version that was current at a specific system time.
    pub fn version_at(&self, at: DateTime<Utc>) -> Option<&BiTemporal<T>> {
        self.versions.iter().find(|v| v.was_current_at(at))
    }

    /// Add a new version.
    pub fn add_version(&mut self, version: BiTemporal<T>) {
        // Supersede the current version
        if let Some(current) = self.versions.iter_mut().find(|v| v.is_current_version()) {
            current.supersede(version.recorded_at);
        }
        self.versions.push(version);
    }

    /// Get all versions.
    pub fn all_versions(&self) -> &[BiTemporal<T>] {
        &self.versions
    }

    /// Get the number of versions.
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

/// Temporal audit trail entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAuditEntry {
    /// Entry ID
    pub entry_id: Uuid,
    /// Entity ID being tracked
    pub entity_id: Uuid,
    /// Entity type
    pub entity_type: String,
    /// Version ID
    pub version_id: Uuid,
    /// Action performed
    pub action: TemporalAction,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// User who performed the action
    pub user_id: String,
    /// Reason for the action
    pub reason: Option<String>,
    /// Previous value (serialized)
    pub previous_value: Option<String>,
    /// New value (serialized)
    pub new_value: Option<String>,
}

/// Actions tracked in temporal audit trail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalAction {
    Create,
    Update,
    Correct,
    Reverse,
    Delete,
    Restore,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEntity {
        name: String,
        value: i32,
    }

    #[test]
    fn test_bitemporal_creation() {
        let entity = TestEntity {
            name: "Test".into(),
            value: 100,
        };
        let temporal = BiTemporal::new(entity).with_recorded_by("user001");

        assert!(temporal.is_current_version());
        assert!(temporal.is_currently_valid());
        assert_eq!(temporal.inner().value, 100);
    }

    #[test]
    fn test_bitemporal_correction() {
        let original = TestEntity {
            name: "Test".into(),
            value: 100,
        };
        let temporal = BiTemporal::new(original).with_recorded_by("user001");

        let corrected = TestEntity {
            name: "Test".into(),
            value: 150,
        };
        let correction = temporal.correct(corrected, "user002", "Amount was wrong");

        assert_eq!(correction.change_type, TemporalChangeType::Correction);
        assert_eq!(correction.previous_version_id, Some(temporal.version_id));
        assert_eq!(correction.inner().value, 150);
    }

    #[test]
    fn test_version_chain() {
        let entity = TestEntity {
            name: "Test".into(),
            value: 100,
        };
        let v1 = BiTemporal::new(entity.clone()).with_recorded_by("user001");
        let entity_id = Uuid::new_v4();

        let mut chain = TemporalVersionChain::new(entity_id, v1);

        let v2 = BiTemporal::new(TestEntity {
            name: "Test".into(),
            value: 200,
        })
        .with_recorded_by("user002")
        .with_change_type(TemporalChangeType::Correction);

        chain.add_version(v2);

        assert_eq!(chain.version_count(), 2);
        assert_eq!(chain.current().unwrap().inner().value, 200);
    }

    #[test]
    fn test_temporal_change_type() {
        assert!(TemporalChangeType::Correction.is_correction());
        assert!(TemporalChangeType::Reversal.is_correction());
        assert!(!TemporalChangeType::Original.is_correction());
    }
}
