# Design Decisions

Key architectural choices and their rationale.

## 1. Deterministic RNG

**Decision:** Use seeded ChaCha8 RNG for all randomness.

**Rationale:**
- Reproducible output for testing and debugging
- Consistent results across runs
- Parallel generation with per-thread seeds

**Implementation:**
```rust
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

let mut rng = ChaCha8Rng::seed_from_u64(config.global.seed);
```

**Trade-off:** Slightly slower than system RNG, but reproducibility is essential for financial data testing.

---

## 2. Precise Decimal Arithmetic

**Decision:** Use `rust_decimal::Decimal` for all monetary values.

**Rationale:**
- IEEE 754 floating-point causes rounding errors
- Financial systems require exact decimal representation
- Debits must exactly equal credits

**Implementation:**
```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

let amount = dec!(1234.56);
let tax = amount * dec!(0.077);  // Exact
```

**Serialization:** Decimals serialized as strings to preserve precision:
```json
{"amount": "1234.56"}
```

---

## 3. Balanced Entry Enforcement

**Decision:** JournalEntry enforces debits = credits at construction.

**Rationale:**
- Invalid accounting entries should be impossible
- Catches bugs early in generation
- Guarantees trial balance coherence

**Implementation:**
```rust
impl JournalEntry {
    pub fn new(header: JournalEntryHeader, lines: Vec<JournalEntryLine>) -> Result<Self> {
        let entry = Self { header, lines };
        if !entry.is_balanced() {
            return Err(Error::UnbalancedEntry);
        }
        Ok(entry)
    }
}
```

---

## 4. Collision-Free UUIDs

**Decision:** Use FNV-1a hash-based UUID generation with generator-type discriminators.

**Rationale:**
- Document IDs must be unique across all generators
- Deterministic generation requires deterministic IDs
- Different generator types might generate same sequence

**Implementation:**
```rust
pub struct DeterministicUuidFactory {
    counter: AtomicU64,
    seed: u64,
}

pub enum GeneratorType {
    JournalEntry = 0x01,
    DocumentFlow = 0x02,
    Vendor = 0x03,
    // ...
}

impl DeterministicUuidFactory {
    pub fn generate(&self, gen_type: GeneratorType) -> Uuid {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let hash_input = (self.seed, gen_type as u8, counter);
        Uuid::from_bytes(fnv1a_hash(&hash_input))
    }
}
```

---

## 5. Empirical Distributions

**Decision:** Base statistical distributions on academic research.

**Rationale:**
- Synthetic data should match real-world patterns
- Benford's Law is expected in authentic financial data
- Line item distributions affect detection algorithms

**Sources:**
- Line item counts: GL research showing 60.68% two-line, 88% even counts
- Amounts: Log-normal with round-number bias
- Temporal: Month/quarter/year-end spikes

**Implementation:**
```rust
pub struct LineItemSampler {
    distribution: EmpiricalDistribution,
}

impl LineItemSampler {
    pub fn new() -> Self {
        Self {
            distribution: EmpiricalDistribution::from_data(&[
                (2, 0.6068),
                (3, 0.0524),
                (4, 0.1732),
                // ...
            ]),
        }
    }
}
```

---

## 6. Document Chain Integrity

**Decision:** Maintain proper reference chains with explicit links.

**Rationale:**
- Real document flows have traceable references
- Process mining requires complete chains
- Audit trails need document relationships

**Implementation:**
```rust
pub struct DocumentReference {
    pub from_type: DocumentType,
    pub from_id: String,
    pub to_type: DocumentType,
    pub to_id: String,
    pub reference_type: ReferenceType,
}

// Payment explicitly references invoices
let payment_ref = DocumentReference {
    from_type: DocumentType::Payment,
    from_id: payment.id.clone(),
    to_type: DocumentType::Invoice,
    to_id: invoice.id.clone(),
    reference_type: ReferenceType::PaymentFor,
};
```

---

## 7. Three-Way Match Validation

**Decision:** Implement actual PO/GR/Invoice matching with tolerances.

**Rationale:**
- Real P2P processes include match validation
- Variances are common and should be generated
- Match status affects downstream processing

**Implementation:**
```rust
pub fn validate_match(po: &PurchaseOrder, gr: &GoodsReceipt, inv: &Invoice,
                      config: &MatchConfig) -> MatchResult {
    let qty_variance = (gr.quantity - po.quantity).abs() / po.quantity;
    let price_variance = (inv.unit_price - po.unit_price).abs() / po.unit_price;

    if qty_variance > config.quantity_tolerance {
        return MatchResult::QuantityVariance(qty_variance);
    }
    if price_variance > config.price_tolerance {
        return MatchResult::PriceVariance(price_variance);
    }
    MatchResult::Matched
}
```

---

## 8. Memory Guard Architecture

**Decision:** Cross-platform memory tracking with soft/hard limits.

**Rationale:**
- Large generations can exhaust memory
- OOM kills are unrecoverable
- Graceful degradation preferred

**Implementation:**
```rust
pub fn check(&self) -> MemoryStatus {
    let current = self.get_memory_usage();
    let growth_rate = (current - self.last_usage) as f64 / elapsed_ms;

    MemoryStatus {
        current_usage: current,
        exceeds_soft_limit: current > self.config.soft_limit,
        exceeds_hard_limit: current > self.config.hard_limit,
        growth_rate,
    }
}
```

---

## 9. Layered Crate Architecture

**Decision:** Strict layering with no circular dependencies.

**Rationale:**
- Clear separation of concerns
- Independent crate compilation
- Easier testing and maintenance

**Layers:**
1. Foundation: `datasynth-core` (no internal dependencies)
2. Services: `datasynth-config`, `datasynth-output`
3. Processing: `datasynth-generators`, `datasynth-graph`
4. Orchestration: `datasynth-runtime`
5. Application: `datasynth-cli`, `datasynth-server`, `datasynth-ui`

---

## 10. Configuration-Driven Behavior

**Decision:** All behavior controlled by external configuration.

**Rationale:**
- Flexibility without code changes
- Reproducible scenarios
- User-customizable presets

**Scope:** Configuration controls:
- Industry and complexity
- Transaction volumes and patterns
- Anomaly types and rates
- Output formats
- All feature toggles

---

## 11. Trait-Based Extensibility

**Decision:** Define traits in core, implement in higher layers.

**Rationale:**
- Dependency inversion
- Pluggable implementations
- Easy testing with mocks

**Example:**
```rust
// Defined in datasynth-core
pub trait Generator<T> {
    fn generate_batch(&mut self, count: usize) -> Result<Vec<T>>;
}

// Implemented in datasynth-generators
impl Generator<JournalEntry> for JournalEntryGenerator {
    fn generate_batch(&mut self, count: usize) -> Result<Vec<JournalEntry>> {
        // Implementation
    }
}
```

---

## 12. Parallel-Safe Design

**Decision:** Design all generators to be thread-safe.

**Rationale:**
- Generation can be parallelized
- Modern systems have many cores
- Linear scaling improves throughput

**Implementation:**
- Per-thread RNG seeds: `seed + thread_id`
- Atomic counters for UUID factory
- No shared mutable state during generation
- Rayon for parallel iteration

## 13. Country Pack Architecture

**Decision:** Replace hardcoded country-specific data with pluggable JSON country packs loaded at runtime.

**Rationale:**
- ~7,500 lines of hardcoded country data (holidays, names, tax rates, addresses) were scattered across generators
- Adding a new country required code changes in multiple crates
- JSON format enables non-developers to contribute country data
- Layered merge (`_default.json` → country → user overrides) provides flexibility without duplication

**Implementation:**
- 16-section JSON schema covering locale, names, holidays, tax, address, phone, banking, payroll, and more
- `include_str!` embeds built-in packs (US, DE, GB) for zero-config usage
- `CountryPackRegistry` resolves packs by country code with fallback to defaults
- `deep_merge()` recursively merges objects while replacing arrays/scalars
- `NamePool` fields changed from `Vec<&'static str>` to `Vec<String>` to support deserialized data

**Trade-off:** JSON parsing adds a one-time startup cost (~2ms per pack), but eliminates the maintenance burden of hardcoded data and enables external/commercial country packs via `country_packs.external_dir`.

---

## See Also

- [Architecture Overview](README.md)
- [Domain Models](domain-models.md)
- [datasynth-core](../crates/datasynth-core.md)
