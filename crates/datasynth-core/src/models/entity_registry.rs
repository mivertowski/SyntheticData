//! Entity registry for centralized master data management.
//!
//! Provides a central registry tracking all master data entities with
//! temporal validity, ensuring referential integrity across transactions.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Unique identifier for any entity in the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    /// Type of the entity
    pub entity_type: EntityType,
    /// Unique identifier within the type
    pub id: String,
}

impl EntityId {
    /// Create a new entity ID.
    pub fn new(entity_type: EntityType, id: impl Into<String>) -> Self {
        Self {
            entity_type,
            id: id.into(),
        }
    }

    /// Create a vendor entity ID.
    pub fn vendor(id: impl Into<String>) -> Self {
        Self::new(EntityType::Vendor, id)
    }

    /// Create a customer entity ID.
    pub fn customer(id: impl Into<String>) -> Self {
        Self::new(EntityType::Customer, id)
    }

    /// Create a material entity ID.
    pub fn material(id: impl Into<String>) -> Self {
        Self::new(EntityType::Material, id)
    }

    /// Create a fixed asset entity ID.
    pub fn fixed_asset(id: impl Into<String>) -> Self {
        Self::new(EntityType::FixedAsset, id)
    }

    /// Create an employee entity ID.
    pub fn employee(id: impl Into<String>) -> Self {
        Self::new(EntityType::Employee, id)
    }

    /// Create a cost center entity ID.
    pub fn cost_center(id: impl Into<String>) -> Self {
        Self::new(EntityType::CostCenter, id)
    }

    /// Create a profit center entity ID.
    pub fn profit_center(id: impl Into<String>) -> Self {
        Self::new(EntityType::ProfitCenter, id)
    }

    /// Create a GL account entity ID.
    pub fn gl_account(id: impl Into<String>) -> Self {
        Self::new(EntityType::GlAccount, id)
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.entity_type, self.id)
    }
}

/// Types of entities that can be registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Vendor/Supplier
    Vendor,
    /// Customer
    Customer,
    /// Material/Product
    Material,
    /// Fixed Asset
    FixedAsset,
    /// Employee
    Employee,
    /// Cost Center
    CostCenter,
    /// Profit Center
    ProfitCenter,
    /// GL Account
    GlAccount,
    /// Company Code
    CompanyCode,
    /// Business Partner (generic)
    BusinessPartner,
    /// Project/WBS Element
    Project,
    /// Internal Order
    InternalOrder,
    /// Company/legal entity
    Company,
    /// Department
    Department,
    /// Contract
    Contract,
    /// Asset (general)
    Asset,
    /// Bank account
    BankAccount,
    /// Purchase order
    PurchaseOrder,
    /// Sales order
    SalesOrder,
    /// Invoice
    Invoice,
    /// Payment
    Payment,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Vendor => "VENDOR",
            Self::Customer => "CUSTOMER",
            Self::Material => "MATERIAL",
            Self::FixedAsset => "FIXED_ASSET",
            Self::Employee => "EMPLOYEE",
            Self::CostCenter => "COST_CENTER",
            Self::ProfitCenter => "PROFIT_CENTER",
            Self::GlAccount => "GL_ACCOUNT",
            Self::CompanyCode => "COMPANY_CODE",
            Self::BusinessPartner => "BUSINESS_PARTNER",
            Self::Project => "PROJECT",
            Self::InternalOrder => "INTERNAL_ORDER",
            Self::Company => "COMPANY",
            Self::Department => "DEPARTMENT",
            Self::Contract => "CONTRACT",
            Self::Asset => "ASSET",
            Self::BankAccount => "BANK_ACCOUNT",
            Self::PurchaseOrder => "PURCHASE_ORDER",
            Self::SalesOrder => "SALES_ORDER",
            Self::Invoice => "INVOICE",
            Self::Payment => "PAYMENT",
        };
        write!(f, "{}", name)
    }
}

/// Status of an entity at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityStatus {
    /// Entity is active and can be used in transactions
    #[default]
    Active,
    /// Entity is blocked for new transactions
    Blocked,
    /// Entity is marked for deletion
    MarkedForDeletion,
    /// Entity has been archived
    Archived,
}

/// Record of an entity in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRecord {
    /// Entity identifier
    pub entity_id: EntityId,
    /// Human-readable name/description
    pub name: String,
    /// Company code this entity belongs to (if applicable)
    pub company_code: Option<String>,
    /// Date the entity was created
    pub created_date: NaiveDate,
    /// Date the entity becomes valid (may differ from created)
    pub valid_from: NaiveDate,
    /// Date the entity is valid until (None = indefinite)
    pub valid_to: Option<NaiveDate>,
    /// Current status
    pub status: EntityStatus,
    /// Date status last changed
    pub status_changed_date: Option<NaiveDate>,
    /// Additional attributes as key-value pairs
    pub attributes: HashMap<String, String>,
}

impl EntityRecord {
    /// Create a new entity record.
    pub fn new(entity_id: EntityId, name: impl Into<String>, created_date: NaiveDate) -> Self {
        Self {
            entity_id,
            name: name.into(),
            company_code: None,
            created_date,
            valid_from: created_date,
            valid_to: None,
            status: EntityStatus::Active,
            status_changed_date: None,
            attributes: HashMap::new(),
        }
    }

    /// Set company code.
    pub fn with_company_code(mut self, company_code: impl Into<String>) -> Self {
        self.company_code = Some(company_code.into());
        self
    }

    /// Set validity period.
    pub fn with_validity(mut self, from: NaiveDate, to: Option<NaiveDate>) -> Self {
        self.valid_from = from;
        self.valid_to = to;
        self
    }

    /// Add an attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Check if the entity is valid on a given date.
    pub fn is_valid_on(&self, date: NaiveDate) -> bool {
        date >= self.valid_from
            && self.valid_to.map_or(true, |to| date <= to)
            && self.status == EntityStatus::Active
    }

    /// Check if the entity can be used in transactions on a given date.
    pub fn can_transact_on(&self, date: NaiveDate) -> bool {
        self.is_valid_on(date) && self.status == EntityStatus::Active
    }
}

/// Event in an entity's lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityEvent {
    /// Entity this event relates to
    pub entity_id: EntityId,
    /// Type of event
    pub event_type: EntityEventType,
    /// Date the event occurred
    pub event_date: NaiveDate,
    /// Description of the event
    pub description: Option<String>,
}

/// Types of entity lifecycle events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityEventType {
    /// Entity was created
    Created,
    /// Entity was activated
    Activated,
    /// Entity was blocked
    Blocked,
    /// Entity was unblocked
    Unblocked,
    /// Entity was marked for deletion
    MarkedForDeletion,
    /// Entity was archived
    Archived,
    /// Entity validity period changed
    ValidityChanged,
    /// Entity was transferred to another company
    Transferred,
    /// Entity attributes were modified
    Modified,
}

/// Central registry for all master data entities.
///
/// Ensures referential integrity by tracking entity existence and validity
/// over time. All transaction generators should check this registry before
/// using any master data reference.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityRegistry {
    /// All registered entities
    entities: HashMap<EntityId, EntityRecord>,
    /// Index by entity type
    by_type: HashMap<EntityType, Vec<EntityId>>,
    /// Index by company code
    by_company: HashMap<String, Vec<EntityId>>,
    /// Timeline of entity events
    entity_timeline: BTreeMap<NaiveDate, Vec<EntityEvent>>,
}

impl EntityRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new entity.
    pub fn register(&mut self, record: EntityRecord) {
        let entity_id = record.entity_id.clone();
        let entity_type = entity_id.entity_type;
        let company_code = record.company_code.clone();
        let created_date = record.created_date;

        // Add to main storage
        self.entities.insert(entity_id.clone(), record);

        // Update type index
        self.by_type
            .entry(entity_type)
            .or_default()
            .push(entity_id.clone());

        // Update company index
        if let Some(cc) = company_code {
            self.by_company
                .entry(cc)
                .or_default()
                .push(entity_id.clone());
        }

        // Record creation event
        let event = EntityEvent {
            entity_id,
            event_type: EntityEventType::Created,
            event_date: created_date,
            description: Some("Entity created".to_string()),
        };
        self.entity_timeline
            .entry(created_date)
            .or_default()
            .push(event);
    }

    /// Get an entity by ID.
    pub fn get(&self, entity_id: &EntityId) -> Option<&EntityRecord> {
        self.entities.get(entity_id)
    }

    /// Get a mutable reference to an entity.
    pub fn get_mut(&mut self, entity_id: &EntityId) -> Option<&mut EntityRecord> {
        self.entities.get_mut(entity_id)
    }

    /// Check if an entity exists.
    pub fn exists(&self, entity_id: &EntityId) -> bool {
        self.entities.contains_key(entity_id)
    }

    /// Check if an entity exists and is valid on a given date.
    pub fn is_valid(&self, entity_id: &EntityId, date: NaiveDate) -> bool {
        self.entities
            .get(entity_id)
            .is_some_and(|r| r.is_valid_on(date))
    }

    /// Check if an entity can be used in transactions on a given date.
    pub fn can_transact(&self, entity_id: &EntityId, date: NaiveDate) -> bool {
        self.entities
            .get(entity_id)
            .is_some_and(|r| r.can_transact_on(date))
    }

    /// Get all entities of a given type.
    pub fn get_by_type(&self, entity_type: EntityType) -> Vec<&EntityRecord> {
        self.by_type
            .get(&entity_type)
            .map(|ids| ids.iter().filter_map(|id| self.entities.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all entities of a given type that are valid on a date.
    pub fn get_valid_by_type(
        &self,
        entity_type: EntityType,
        date: NaiveDate,
    ) -> Vec<&EntityRecord> {
        self.get_by_type(entity_type)
            .into_iter()
            .filter(|r| r.is_valid_on(date))
            .collect()
    }

    /// Get all entities for a company code.
    pub fn get_by_company(&self, company_code: &str) -> Vec<&EntityRecord> {
        self.by_company
            .get(company_code)
            .map(|ids| ids.iter().filter_map(|id| self.entities.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all entity IDs of a given type.
    pub fn get_ids_by_type(&self, entity_type: EntityType) -> Vec<&EntityId> {
        self.by_type
            .get(&entity_type)
            .map(|ids| ids.iter().collect())
            .unwrap_or_default()
    }

    /// Get count of entities by type.
    pub fn count_by_type(&self, entity_type: EntityType) -> usize {
        self.by_type.get(&entity_type).map_or(0, |ids| ids.len())
    }

    /// Get total count of all entities.
    pub fn total_count(&self) -> usize {
        self.entities.len()
    }

    /// Update entity status.
    pub fn update_status(
        &mut self,
        entity_id: &EntityId,
        new_status: EntityStatus,
        date: NaiveDate,
    ) -> bool {
        if let Some(record) = self.entities.get_mut(entity_id) {
            let old_status = record.status;
            record.status = new_status;
            record.status_changed_date = Some(date);

            // Record status change event
            let event_type = match new_status {
                EntityStatus::Active if old_status == EntityStatus::Blocked => {
                    EntityEventType::Unblocked
                }
                EntityStatus::Active => EntityEventType::Activated,
                EntityStatus::Blocked => EntityEventType::Blocked,
                EntityStatus::MarkedForDeletion => EntityEventType::MarkedForDeletion,
                EntityStatus::Archived => EntityEventType::Archived,
            };

            let event = EntityEvent {
                entity_id: entity_id.clone(),
                event_type,
                event_date: date,
                description: Some(format!(
                    "Status changed from {:?} to {:?}",
                    old_status, new_status
                )),
            };
            self.entity_timeline.entry(date).or_default().push(event);

            true
        } else {
            false
        }
    }

    /// Block an entity for new transactions.
    pub fn block(&mut self, entity_id: &EntityId, date: NaiveDate) -> bool {
        self.update_status(entity_id, EntityStatus::Blocked, date)
    }

    /// Unblock an entity.
    pub fn unblock(&mut self, entity_id: &EntityId, date: NaiveDate) -> bool {
        self.update_status(entity_id, EntityStatus::Active, date)
    }

    /// Get events that occurred on a specific date.
    pub fn get_events_on(&self, date: NaiveDate) -> &[EntityEvent] {
        self.entity_timeline
            .get(&date)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get events in a date range.
    pub fn get_events_in_range(&self, from: NaiveDate, to: NaiveDate) -> Vec<&EntityEvent> {
        self.entity_timeline
            .range(from..=to)
            .flat_map(|(_, events)| events.iter())
            .collect()
    }

    /// Get the timeline entry dates.
    pub fn timeline_dates(&self) -> impl Iterator<Item = &NaiveDate> {
        self.entity_timeline.keys()
    }

    /// Validate that an entity reference can be used on a transaction date.
    /// Returns an error message if invalid.
    pub fn validate_reference(
        &self,
        entity_id: &EntityId,
        transaction_date: NaiveDate,
    ) -> Result<(), String> {
        match self.entities.get(entity_id) {
            None => Err(format!("Entity {} does not exist", entity_id)),
            Some(record) => {
                if transaction_date < record.valid_from {
                    Err(format!(
                        "Entity {} is not valid until {} (transaction date: {})",
                        entity_id, record.valid_from, transaction_date
                    ))
                } else if let Some(valid_to) = record.valid_to {
                    if transaction_date > valid_to {
                        Err(format!(
                            "Entity {} validity expired on {} (transaction date: {})",
                            entity_id, valid_to, transaction_date
                        ))
                    } else if record.status != EntityStatus::Active {
                        Err(format!(
                            "Entity {} has status {:?} (not active)",
                            entity_id, record.status
                        ))
                    } else {
                        Ok(())
                    }
                } else if record.status != EntityStatus::Active {
                    Err(format!(
                        "Entity {} has status {:?} (not active)",
                        entity_id, record.status
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Rebuild all indices (call after deserialization).
    pub fn rebuild_indices(&mut self) {
        self.by_type.clear();
        self.by_company.clear();

        for (entity_id, record) in &self.entities {
            self.by_type
                .entry(entity_id.entity_type)
                .or_default()
                .push(entity_id.clone());

            if let Some(cc) = &record.company_code {
                self.by_company
                    .entry(cc.clone())
                    .or_default()
                    .push(entity_id.clone());
            }
        }
    }

    // === Backward compatibility aliases ===

    /// Alias for `register` - registers a new entity.
    pub fn register_entity(&mut self, record: EntityRecord) {
        self.register(record);
    }

    /// Record an event for an entity.
    pub fn record_event(&mut self, event: EntityEvent) {
        self.entity_timeline
            .entry(event.event_date)
            .or_default()
            .push(event);
    }

    /// Check if an entity is valid on a given date.
    /// Alias for `is_valid`.
    pub fn is_valid_on(&self, entity_id: &EntityId, date: NaiveDate) -> bool {
        self.is_valid(entity_id, date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_date(days: i64) -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() + chrono::Duration::days(days)
    }

    #[test]
    fn test_entity_registration() {
        let mut registry = EntityRegistry::new();

        let entity_id = EntityId::vendor("V-001");
        let record = EntityRecord::new(entity_id.clone(), "Test Vendor", test_date(0));

        registry.register(record);

        assert!(registry.exists(&entity_id));
        assert_eq!(registry.count_by_type(EntityType::Vendor), 1);
    }

    #[test]
    fn test_entity_validity() {
        let mut registry = EntityRegistry::new();

        let entity_id = EntityId::vendor("V-001");
        let record = EntityRecord::new(entity_id.clone(), "Test Vendor", test_date(10))
            .with_validity(test_date(10), Some(test_date(100)));

        registry.register(record);

        // Before validity period
        assert!(!registry.is_valid(&entity_id, test_date(5)));

        // During validity period
        assert!(registry.is_valid(&entity_id, test_date(50)));

        // After validity period
        assert!(!registry.is_valid(&entity_id, test_date(150)));
    }

    #[test]
    fn test_entity_blocking() {
        let mut registry = EntityRegistry::new();

        let entity_id = EntityId::vendor("V-001");
        let record = EntityRecord::new(entity_id.clone(), "Test Vendor", test_date(0));

        registry.register(record);

        // Initially can transact
        assert!(registry.can_transact(&entity_id, test_date(5)));

        // Block the entity
        registry.block(&entity_id, test_date(10));

        // Cannot transact after blocking
        assert!(!registry.can_transact(&entity_id, test_date(15)));

        // Unblock
        registry.unblock(&entity_id, test_date(20));

        // Can transact again
        assert!(registry.can_transact(&entity_id, test_date(25)));
    }

    #[test]
    fn test_entity_timeline() {
        let mut registry = EntityRegistry::new();

        let entity1 = EntityId::vendor("V-001");
        let entity2 = EntityId::vendor("V-002");

        registry.register(EntityRecord::new(entity1.clone(), "Vendor 1", test_date(0)));
        registry.register(EntityRecord::new(entity2.clone(), "Vendor 2", test_date(5)));

        let events_day0 = registry.get_events_on(test_date(0));
        assert_eq!(events_day0.len(), 1);

        let events_range = registry.get_events_in_range(test_date(0), test_date(10));
        assert_eq!(events_range.len(), 2);
    }

    #[test]
    fn test_company_index() {
        let mut registry = EntityRegistry::new();

        let entity1 = EntityId::vendor("V-001");
        let entity2 = EntityId::vendor("V-002");
        let entity3 = EntityId::customer("C-001");

        registry.register(
            EntityRecord::new(entity1.clone(), "Vendor 1", test_date(0)).with_company_code("1000"),
        );
        registry.register(
            EntityRecord::new(entity2.clone(), "Vendor 2", test_date(0)).with_company_code("2000"),
        );
        registry.register(
            EntityRecord::new(entity3.clone(), "Customer 1", test_date(0))
                .with_company_code("1000"),
        );

        let company_1000_entities = registry.get_by_company("1000");
        assert_eq!(company_1000_entities.len(), 2);
    }

    #[test]
    fn test_validate_reference() {
        let mut registry = EntityRegistry::new();

        let entity_id = EntityId::vendor("V-001");
        let record = EntityRecord::new(entity_id.clone(), "Test Vendor", test_date(10))
            .with_validity(test_date(10), Some(test_date(100)));

        registry.register(record);

        // Before validity
        assert!(registry
            .validate_reference(&entity_id, test_date(5))
            .is_err());

        // During validity
        assert!(registry
            .validate_reference(&entity_id, test_date(50))
            .is_ok());

        // After validity
        assert!(registry
            .validate_reference(&entity_id, test_date(150))
            .is_err());

        // Non-existent entity
        let fake_id = EntityId::vendor("V-999");
        assert!(registry
            .validate_reference(&fake_id, test_date(50))
            .is_err());
    }
}
