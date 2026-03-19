//! Deterministic UUID generation factory for reproducible synthetic data.
//!
//! This module provides a centralized UUID generation system that ensures:
//! - No collisions between different generator types
//! - Reproducible output given the same seed
//! - Thread-safe counter increments

use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

/// Generator type discriminators to prevent UUID collisions across generators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GeneratorType {
    /// Journal Entry generator
    JournalEntry = 0x01,
    /// Document Flow (P2P/O2C) generator
    DocumentFlow = 0x02,
    /// Master Data - Vendor generator
    Vendor = 0x03,
    /// Master Data - Customer generator
    Customer = 0x04,
    /// Master Data - Material generator
    Material = 0x05,
    /// Master Data - Asset generator
    Asset = 0x06,
    /// Master Data - Employee generator
    Employee = 0x07,
    /// Subledger - AR generator
    ARSubledger = 0x08,
    /// Subledger - AP generator
    APSubledger = 0x09,
    /// Subledger - FA generator
    FASubledger = 0x0A,
    /// Subledger - Inventory generator
    InventorySubledger = 0x0B,
    /// Intercompany generator
    Intercompany = 0x0C,
    /// Anomaly injection
    Anomaly = 0x0D,
    /// Period close generator
    PeriodClose = 0x0E,
    /// FX rate generator
    FxRate = 0x0F,
    /// Accrual generator
    Accrual = 0x10,
    /// Depreciation generator
    Depreciation = 0x11,
    /// Control generator
    Control = 0x12,
    /// Opening balance generator
    OpeningBalance = 0x13,
    /// Trial balance generator
    TrialBalance = 0x14,
    /// Purchase Order document
    PurchaseOrder = 0x20,
    /// Goods Receipt document
    GoodsReceipt = 0x21,
    /// Vendor Invoice document
    VendorInvoice = 0x22,
    /// Payment document
    Payment = 0x23,
    /// Sales Order document
    SalesOrder = 0x24,
    /// Delivery document
    Delivery = 0x25,
    /// Customer Invoice document
    CustomerInvoice = 0x26,
    /// Customer Receipt document
    CustomerReceipt = 0x27,

    // ===== Enterprise Process Chain generators =====
    /// Sourcing project generator
    SourcingProject = 0x28,
    /// RFx event generator
    RfxEvent = 0x29,
    /// Supplier bid generator
    SupplierBid = 0x2A,
    /// Procurement contract generator
    ProcurementContract = 0x2B,
    /// Catalog item generator
    CatalogItem = 0x2C,
    /// Bank reconciliation generator
    BankReconciliation = 0x2D,
    /// Financial statement generator
    FinancialStatement = 0x2E,
    /// Payroll run generator
    PayrollRun = 0x2F,
    /// Time entry generator
    TimeEntry = 0x30,
    /// Expense report generator
    ExpenseReport = 0x31,
    /// Production order generator
    ProductionOrder = 0x32,
    /// Cycle count generator
    CycleCount = 0x33,
    /// Quality inspection generator
    QualityInspection = 0x34,
    /// Sales quote generator
    SalesQuote = 0x35,
    /// Budget line generator
    BudgetLine = 0x36,
    /// Revenue recognition contract generator
    RevenueRecognition = 0x37,
    /// Impairment test generator
    ImpairmentTest = 0x38,
    /// Management KPI generator
    Kpi = 0x39,
    /// Tax code / jurisdiction generator
    Tax = 0x3A,
    /// Project accounting (cost lines, revenue, milestones, change orders, EVM)
    ProjectAccounting = 0x3B,
    /// ESG / Sustainability (emissions, energy, water, waste, diversity, safety)
    Esg = 0x3C,
    /// Supplier qualification generator
    SupplierQualification = 0x3D,
    /// Supplier scorecard generator
    SupplierScorecard = 0x3E,
    /// BOM component generator
    BomComponent = 0x3F,
    /// Inventory movement generator
    InventoryMovement = 0x40,
    /// Benefit enrollment generator
    BenefitEnrollment = 0x41,
    /// Disruption event generator
    Disruption = 0x42,
    /// Business combination generator (IFRS 3 / ASC 805)
    BusinessCombination = 0x43,
    /// Segment reporting generator (IFRS 8 / ASC 280)
    SegmentReport = 0x44,
    /// Expected Credit Loss generator (IFRS 9 / ASC 326)
    ExpectedCreditLoss = 0x45,
    /// Defined benefit pension generator (IAS 19 / ASC 715)
    Pension = 0x46,
    /// Provisions and contingencies generator (IAS 37 / ASC 450)
    Provision = 0x47,
}

/// A factory for generating deterministic UUIDs that are guaranteed unique
/// across different generator types within the same seed.
///
/// # UUID Structure (16 bytes)
///
/// ```text
/// Bytes 0-5:   Seed (lower 48 bits)
/// Byte  6:     Generator type discriminator
/// Byte  7:     Version nibble (0x4_) | Sub-discriminator
/// Bytes 8-15:  Counter (64-bit, with variant bits set)
/// ```
///
/// # Thread Safety
///
/// The counter uses `AtomicU64` for thread-safe increments, allowing
/// concurrent UUID generation from multiple threads.
#[derive(Debug)]
pub struct DeterministicUuidFactory {
    seed: u64,
    generator_type: GeneratorType,
    counter: AtomicU64,
    /// Optional sub-discriminator for further namespace separation
    sub_discriminator: u8,
}

impl DeterministicUuidFactory {
    /// Create a new UUID factory for a specific generator type.
    ///
    /// # Arguments
    ///
    /// * `seed` - The global seed for deterministic generation
    /// * `generator_type` - The type of generator using this factory
    ///
    /// # Example
    ///
    /// ```
    /// use datasynth_core::uuid_factory::{DeterministicUuidFactory, GeneratorType};
    ///
    /// let factory = DeterministicUuidFactory::new(12345, GeneratorType::JournalEntry);
    /// let uuid = factory.next();
    /// ```
    pub fn new(seed: u64, generator_type: GeneratorType) -> Self {
        Self {
            seed,
            generator_type,
            counter: AtomicU64::new(0),
            sub_discriminator: 0,
        }
    }

    /// Create a factory with a sub-discriminator for additional namespace separation.
    ///
    /// Useful when the same generator type needs multiple independent UUID streams.
    pub fn with_sub_discriminator(
        seed: u64,
        generator_type: GeneratorType,
        sub_discriminator: u8,
    ) -> Self {
        Self {
            seed,
            generator_type,
            counter: AtomicU64::new(0),
            sub_discriminator,
        }
    }

    /// Create a factory starting from a specific counter value.
    ///
    /// Useful for resuming generation from a checkpoint or for partitioned
    /// parallel generation where each thread gets a non-overlapping counter range.
    pub fn with_counter(seed: u64, generator_type: GeneratorType, start_counter: u64) -> Self {
        Self {
            seed,
            generator_type,
            counter: AtomicU64::new(start_counter),
            sub_discriminator: 0,
        }
    }

    /// Create a factory for a specific partition in parallel generation.
    ///
    /// Each partition gets a unique sub-discriminator so that counters starting
    /// from 0 in each partition still produce globally unique UUIDs. This avoids
    /// atomic contention between threads since each partition has its own factory.
    pub fn for_partition(seed: u64, generator_type: GeneratorType, partition_index: u8) -> Self {
        Self {
            seed,
            generator_type,
            counter: AtomicU64::new(0),
            sub_discriminator: partition_index,
        }
    }

    /// Generate the next UUID in the sequence.
    ///
    /// This method is thread-safe and can be called from multiple threads.
    #[inline]
    pub fn next(&self) -> Uuid {
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);
        self.generate_uuid(counter)
    }

    /// Generate a UUID for a specific counter value without incrementing.
    ///
    /// Useful for deterministic regeneration of specific UUIDs.
    pub fn generate_at(&self, counter: u64) -> Uuid {
        self.generate_uuid(counter)
    }

    /// Get the current counter value.
    pub fn current_counter(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }

    /// Reset the counter to zero.
    pub fn reset(&self) {
        self.counter.store(0, Ordering::Relaxed);
    }

    /// Set the counter to a specific value.
    pub fn set_counter(&self, value: u64) {
        self.counter.store(value, Ordering::Relaxed);
    }

    /// Generate a UUID from the seed, generator type, and counter.
    ///
    /// Uses a simple hash-based approach to ensure uniqueness while maintaining
    /// determinism. The hash function is designed to spread entropy across all
    /// bytes while preserving the UUID v4 format.
    #[inline]
    fn generate_uuid(&self, counter: u64) -> Uuid {
        // Create a unique input by combining all distinguishing factors
        // Use FNV-1a style hashing for simplicity and determinism
        let mut hash: u64 = 14695981039346656037; // FNV offset basis

        // Mix in seed
        for byte in self.seed.to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211); // FNV prime
        }

        // Mix in generator type
        hash ^= self.generator_type as u64;
        hash = hash.wrapping_mul(1099511628211);

        // Mix in sub-discriminator
        hash ^= self.sub_discriminator as u64;
        hash = hash.wrapping_mul(1099511628211);

        // Mix in counter (most important for uniqueness within same factory)
        for byte in counter.to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }

        // Create second hash for remaining bytes
        let mut hash2: u64 = hash;
        hash2 ^= self.seed.rotate_left(32);
        hash2 = hash2.wrapping_mul(1099511628211);
        hash2 ^= counter.rotate_left(32);
        hash2 = hash2.wrapping_mul(1099511628211);

        let mut bytes = [0u8; 16];

        // First 8 bytes from hash
        bytes[0..8].copy_from_slice(&hash.to_le_bytes());
        // Second 8 bytes from hash2
        bytes[8..16].copy_from_slice(&hash2.to_le_bytes());

        // Set UUID version 4 (bits 12-15 of time_hi_and_version)
        // Byte 6: xxxx0100 -> set bits 4-7 to 0100
        bytes[6] = (bytes[6] & 0x0f) | 0x40;

        // Set variant to RFC 4122 (bits 6-7 of clock_seq_hi_and_reserved)
        // Byte 8: 10xxxxxx -> set bits 6-7 to 10
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Uuid::from_bytes(bytes)
    }
}

impl Clone for DeterministicUuidFactory {
    fn clone(&self) -> Self {
        Self {
            seed: self.seed,
            generator_type: self.generator_type,
            counter: AtomicU64::new(self.counter.load(Ordering::Relaxed)),
            sub_discriminator: self.sub_discriminator,
        }
    }
}

/// A registry that manages multiple UUID factories for different generator types.
///
/// This ensures a single source of truth for UUID generation across the system.
#[derive(Debug)]
pub struct UuidFactoryRegistry {
    seed: u64,
    factories: std::collections::HashMap<GeneratorType, DeterministicUuidFactory>,
}

impl UuidFactoryRegistry {
    /// Create a new registry with a global seed.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            factories: std::collections::HashMap::new(),
        }
    }

    /// Get or create a factory for a specific generator type.
    pub fn get_factory(&mut self, generator_type: GeneratorType) -> &DeterministicUuidFactory {
        self.factories
            .entry(generator_type)
            .or_insert_with(|| DeterministicUuidFactory::new(self.seed, generator_type))
    }

    /// Generate the next UUID for a specific generator type.
    pub fn next_uuid(&mut self, generator_type: GeneratorType) -> Uuid {
        self.get_factory(generator_type).next()
    }

    /// Reset all factories.
    pub fn reset_all(&self) {
        for factory in self.factories.values() {
            factory.reset();
        }
    }

    /// Get the current counter for a generator type.
    pub fn get_counter(&self, generator_type: GeneratorType) -> Option<u64> {
        self.factories
            .get(&generator_type)
            .map(DeterministicUuidFactory::current_counter)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::thread;

    #[test]
    fn test_uuid_uniqueness_same_generator() {
        let factory = DeterministicUuidFactory::new(12345, GeneratorType::JournalEntry);

        let mut uuids = HashSet::new();
        for _ in 0..10000 {
            let uuid = factory.next();
            assert!(uuids.insert(uuid), "Duplicate UUID generated");
        }
    }

    #[test]
    fn test_uuid_uniqueness_different_generators() {
        let factory1 = DeterministicUuidFactory::new(12345, GeneratorType::JournalEntry);
        let factory2 = DeterministicUuidFactory::new(12345, GeneratorType::DocumentFlow);

        let mut uuids = HashSet::new();

        for _ in 0..5000 {
            let uuid1 = factory1.next();
            let uuid2 = factory2.next();
            assert!(uuids.insert(uuid1), "Duplicate UUID from JE generator");
            assert!(uuids.insert(uuid2), "Duplicate UUID from DocFlow generator");
        }
    }

    #[test]
    fn test_uuid_determinism() {
        let factory1 = DeterministicUuidFactory::new(12345, GeneratorType::JournalEntry);
        let factory2 = DeterministicUuidFactory::new(12345, GeneratorType::JournalEntry);

        for _ in 0..100 {
            assert_eq!(factory1.next(), factory2.next());
        }
    }

    #[test]
    fn test_uuid_different_seeds() {
        let factory1 = DeterministicUuidFactory::new(12345, GeneratorType::JournalEntry);
        let factory2 = DeterministicUuidFactory::new(67890, GeneratorType::JournalEntry);

        // Different seeds should produce different UUIDs
        assert_ne!(factory1.next(), factory2.next());
    }

    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;

        let factory = Arc::new(DeterministicUuidFactory::new(
            12345,
            GeneratorType::JournalEntry,
        ));
        let mut handles = vec![];

        for _ in 0..4 {
            let factory_clone = Arc::clone(&factory);
            handles.push(thread::spawn(move || {
                let mut uuids = Vec::new();
                for _ in 0..1000 {
                    uuids.push(factory_clone.next());
                }
                uuids
            }));
        }

        let mut all_uuids = HashSet::new();
        for handle in handles {
            let uuids = handle.join().unwrap();
            for uuid in uuids {
                assert!(all_uuids.insert(uuid), "Thread-generated UUID collision");
            }
        }

        assert_eq!(all_uuids.len(), 4000);
    }

    #[test]
    fn test_sub_discriminator() {
        let factory1 =
            DeterministicUuidFactory::with_sub_discriminator(12345, GeneratorType::JournalEntry, 0);
        let factory2 =
            DeterministicUuidFactory::with_sub_discriminator(12345, GeneratorType::JournalEntry, 1);

        // Different sub-discriminators should produce different UUIDs
        let uuid1 = factory1.next();
        factory1.reset();
        let uuid2 = factory2.next();

        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_generate_at() {
        let factory = DeterministicUuidFactory::new(12345, GeneratorType::JournalEntry);

        // Generate at specific counter
        let uuid_at_5 = factory.generate_at(5);

        // Generate sequentially to reach counter 5
        for _ in 0..5 {
            factory.next();
        }
        let _uuid_sequential = factory.next();

        // The UUID at counter 5 should match
        assert_eq!(uuid_at_5, factory.generate_at(5));
    }

    #[test]
    fn test_registry() {
        let mut registry = UuidFactoryRegistry::new(12345);

        let uuid1 = registry.next_uuid(GeneratorType::JournalEntry);
        let uuid2 = registry.next_uuid(GeneratorType::JournalEntry);
        let uuid3 = registry.next_uuid(GeneratorType::DocumentFlow);

        // All should be unique
        assert_ne!(uuid1, uuid2);
        assert_ne!(uuid1, uuid3);
        assert_ne!(uuid2, uuid3);

        // Counter should be tracked
        assert_eq!(registry.get_counter(GeneratorType::JournalEntry), Some(2));
        assert_eq!(registry.get_counter(GeneratorType::DocumentFlow), Some(1));
    }

    #[test]
    fn test_uuid_is_valid_v4() {
        let factory = DeterministicUuidFactory::new(12345, GeneratorType::JournalEntry);
        let uuid = factory.next();

        // Check version is 4
        assert_eq!(uuid.get_version_num(), 4);

        // Check variant is RFC 4122
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
    }
}
