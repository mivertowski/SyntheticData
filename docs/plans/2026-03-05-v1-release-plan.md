# DataSynth v1.0.0 Release Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Ship a feature-complete, production-hardened v1.0.0 reference release by wiring existing but unconnected generators, filling remaining gaps, and polishing packaging.

**Architecture:** The orchestrator (enhanced_orchestrator.rs) drives generation through numbered phases. Each phase creates a snapshot (e.g., HrSnapshot, ManufacturingSnapshot) that flows into EnhancedGenerationResult, which the CLI output_writer serializes to JSON. Generators already exist for most missing features — they just need to be called in the right phase and their output added to the snapshot.

**Tech Stack:** Rust workspace (15 crates), ChaCha8 RNG, rust_decimal, chrono, serde, tokio

---

## Task 1: Wire BenefitEnrollmentGenerator into HR Phase

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:520-537` (HrSnapshot struct)
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:3589-3738` (Phase 16 HR generation)
- Modify: `crates/datasynth-cli/src/output_writer.rs:546-569` (HR JSON output)

**Context:** BenefitEnrollmentGenerator exists at `crates/datasynth-generators/src/hr/benefit_enrollment_generator.rs` and is fully tested (4 tests). It's just never called in the orchestrator.

**Step 1: Add benefit fields to HrSnapshot**

In `enhanced_orchestrator.rs`, find the `HrSnapshot` struct (~line 520) and add:

```rust
pub struct HrSnapshot {
    pub payroll_runs: Vec<PayrollRun>,
    pub payroll_line_items: Vec<PayrollLineItem>,
    pub time_entries: Vec<TimeEntry>,
    pub expense_reports: Vec<ExpenseReport>,
    pub benefit_enrollments: Vec<BenefitEnrollment>,  // ADD
    pub payroll_run_count: usize,
    pub payroll_line_item_count: usize,
    pub time_entry_count: usize,
    pub expense_report_count: usize,
    pub benefit_enrollment_count: usize,  // ADD
}
```

Update the `Default` impl and all construction sites to include the new fields.

**Step 2: Call BenefitEnrollmentGenerator in Phase 16**

After the expense report generation block (~line 3720), add:

```rust
// Generate benefit enrollments
let mut benefit_gen = datasynth_generators::BenefitEnrollmentGenerator::new(seed + 33);
let employee_pairs: Vec<(String, String)> = employee_pool
    .iter()
    .map(|e| (e.employee_id.clone(), e.name.clone()))
    .collect();
let enrollments = benefit_gen.generate(
    company_code,
    &employee_pairs,
    start_date,
    currency,
);
let enrollment_count = enrollments.len();
hr_snapshot.benefit_enrollments.extend(enrollments);
hr_snapshot.benefit_enrollment_count += enrollment_count;
info!("  Generated {} benefit enrollments", enrollment_count);
```

**Step 3: Add benefit enrollments to output_writer**

In `output_writer.rs` after expense_reports export (~line 569), add:

```rust
write_json(
    &result.hr.benefit_enrollments,
    &hr_dir.join("benefit_enrollments.json"),
)?;
```

**Step 4: Run tests**

```bash
cargo test -p datasynth-runtime -- hr
cargo test -p datasynth-generators -- benefit
cargo build --release
```

**Step 5: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs crates/datasynth-cli/src/output_writer.rs
git commit -m "feat(runtime): wire BenefitEnrollmentGenerator into HR phase"
```

---

## Task 2: Wire BomGenerator + InventoryMovementGenerator into Manufacturing Phase

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:554-567` (ManufacturingSnapshot struct)
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:3880-3990` (Phase 18 MFG generation)
- Modify: `crates/datasynth-cli/src/output_writer.rs:579-597` (MFG JSON output)

**Context:** BomGenerator (`crates/datasynth-generators/src/manufacturing/bom_generator.rs`, 4 tests) and InventoryMovementGenerator (`crates/datasynth-generators/src/manufacturing/inventory_movement_generator.rs`, 3 tests) exist but are not called.

**Step 1: Add fields to ManufacturingSnapshot**

```rust
pub struct ManufacturingSnapshot {
    pub production_orders: Vec<ProductionOrder>,
    pub quality_inspections: Vec<QualityInspection>,
    pub cycle_counts: Vec<CycleCount>,
    pub bom_components: Vec<BomComponent>,           // ADD
    pub inventory_movements: Vec<InventoryMovement>,  // ADD
    pub production_order_count: usize,
    pub quality_inspection_count: usize,
    pub cycle_count_count: usize,
    pub bom_component_count: usize,                   // ADD
    pub inventory_movement_count: usize,              // ADD
}
```

**Step 2: Call BomGenerator after cycle counts (~line 3980)**

```rust
// Generate BOM components
let mut bom_gen = datasynth_generators::BomGenerator::new(seed + 53);
let material_pairs: Vec<(String, String)> = material_data
    .iter()
    .map(|m| (m.material_id.clone(), m.description.clone()))
    .collect();
let bom_components = bom_gen.generate(company_code, &material_pairs);
let bom_count = bom_components.len();
mfg_snapshot.bom_components = bom_components;
mfg_snapshot.bom_component_count = bom_count;
info!("  Generated {} BOM components", bom_count);
```

**Step 3: Call InventoryMovementGenerator**

```rust
// Generate inventory movements
let mut inv_mov_gen = datasynth_generators::InventoryMovementGenerator::new(seed + 54);
let movements = inv_mov_gen.generate(
    company_code,
    &material_pairs,
    start_date,
    end_date,
    2, // movements per material per period
    currency,
);
let mov_count = movements.len();
mfg_snapshot.inventory_movements = movements;
mfg_snapshot.inventory_movement_count = mov_count;
info!("  Generated {} inventory movements", mov_count);
```

**Step 4: Add to output_writer**

After cycle_counts export (~line 597):

```rust
write_json(
    &result.manufacturing.bom_components,
    &mfg_dir.join("bom_components.json"),
)?;
write_json(
    &result.manufacturing.inventory_movements,
    &mfg_dir.join("inventory_movements.json"),
)?;
```

**Step 5: Run tests and commit**

```bash
cargo test -p datasynth-runtime -- manufacturing
cargo test -p datasynth-generators -- bom
cargo test -p datasynth-generators -- inventory_movement
cargo build --release
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs crates/datasynth-cli/src/output_writer.rs
git commit -m "feat(runtime): wire BomGenerator and InventoryMovementGenerator into MFG phase"
```

---

## Task 3: Wire CashForecastGenerator + CashPoolGenerator into Treasury Phase

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:4549-4718` (Phase 22 Treasury)

**Context:** CashForecastGenerator needs ArAgingItem/ApAgingItem/ScheduledDisbursement inputs. CashPoolGenerator needs AccountBalance inputs. Both generators exist and are tested. TreasurySnapshot already has fields for forecasts, pools, and sweeps. The output_writer already exports them.

**Step 1: Prepare AR/AP aging input from existing subledger data**

After cash positions are generated (~line 4700), build input structs from the subledger:

```rust
// Build AR aging items from subledger AR open items
let ar_items: Vec<datasynth_generators::treasury::ArAgingItem> = subledger
    .ar_open_items
    .iter()
    .map(|item| datasynth_generators::treasury::ArAgingItem {
        amount: item.amount,
        due_date: item.due_date,
        customer_id: item.customer_id.clone(),
    })
    .collect();

// Build AP aging items from subledger AP open items
let ap_items: Vec<datasynth_generators::treasury::ApAgingItem> = subledger
    .ap_open_items
    .iter()
    .map(|item| datasynth_generators::treasury::ApAgingItem {
        amount: item.amount,
        due_date: item.due_date,
        vendor_id: item.vendor_id.clone(),
    })
    .collect();

// Generate cash forecast
let mut forecast_gen = datasynth_generators::treasury::CashForecastGenerator::new(
    self.config.treasury.cash_forecasting.clone(),
    seed + 93,
);
let forecast = forecast_gen.generate(
    entity_id,
    currency,
    end_date, // forecast from end of period
    &ar_items,
    &ap_items,
    &[], // scheduled disbursements (payroll, etc.)
);
snapshot.cash_forecasts.push(forecast);
```

**Step 2: Wire CashPoolGenerator**

```rust
// Generate cash pools if enabled
if self.config.treasury.cash_pooling.enabled {
    let mut pool_gen = datasynth_generators::treasury::CashPoolGenerator::new(
        self.config.treasury.cash_pooling.clone(),
        seed + 94,
    );
    // Create pool from company accounts
    let account_ids: Vec<String> = snapshot
        .cash_positions
        .iter()
        .map(|cp| cp.account_id.clone())
        .collect();
    if let Some(pool) = pool_gen.create_pool(
        &format!("{}_POOL", entity_id),
        currency,
        &account_ids,
    ) {
        // Generate daily sweeps for the period
        // Build participant balances from cash positions
        let participant_balances: Vec<_> = snapshot
            .cash_positions
            .iter()
            .map(|cp| datasynth_core::models::AccountBalance {
                // map fields from CashPosition
                ..Default::default()
            })
            .collect();
        // NOTE: Adapt based on actual CashPoolGenerator::generate_sweeps API
        snapshot.cash_pools.push(pool);
    }
}
```

**Step 3: Verify existing output_writer handles the data**

The output_writer at lines 974-984 already exports cash_forecasts and cash_pools — no changes needed there.

**Step 4: Run tests and commit**

```bash
cargo test -p datasynth-runtime -- treasury
cargo test -p datasynth-generators -- cash_forecast
cargo test -p datasynth-generators -- cash_pool
cargo build --release
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(runtime): wire CashForecastGenerator and CashPoolGenerator into treasury phase"
```

---

## Task 4: Wire CorrelationPreservation into Eval Gates

**Files:**
- Modify: `crates/datasynth-eval/src/gates/engine.rs:372-384` (CorrelationPreservation arm)
- Modify: `crates/datasynth-eval/src/lib.rs:194-226` (ComprehensiveEvaluation struct)
- Reference: `crates/datasynth-eval/src/statistical/correlation.rs` (existing module)

**Step 1: Add correlation field to ComprehensiveEvaluation**

In `lib.rs` (~line 202), add:

```rust
pub struct ComprehensiveEvaluation {
    pub statistical: StatisticalEvaluation,
    pub coherence: CoherenceEvaluation,
    pub quality: QualityEvaluation,
    pub ml_readiness: MLReadinessEvaluation,
    pub correlation: Option<CorrelationAnalysis>,  // ADD
    // ... rest
}
```

**Step 2: Populate correlation during evaluation**

Find where ComprehensiveEvaluation is constructed (likely in an `evaluate()` function). Add correlation analysis computation using the existing `statistical/correlation.rs` module.

**Step 3: Update gate engine to read the metric**

In `engine.rs` at ~line 372, replace the error arm:

```rust
QualityMetric::CorrelationPreservation => {
    if let Some(ref corr) = evaluation.correlation {
        let score = corr.overall_score;
        (Some(score), format!("correlation preservation: {:.4}", score))
    } else {
        (None, "correlation analysis not computed for this run".to_string())
    }
}
```

**Step 4: Run tests and commit**

```bash
cargo test -p datasynth-eval -- correlation
cargo test -p datasynth-eval -- gate
cargo build --release
git add crates/datasynth-eval/
git commit -m "feat(eval): wire CorrelationPreservation metric into quality gates"
```

---

## Task 5: Wire EntityGraphBuilder with Company Mapping

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:7022-7027` (EntityGraphBuilder skip)
- Reference: `crates/datasynth-graph/src/builders/entity_graph.rs`
- Reference: `crates/datasynth-core/src/models/` (Company model if exists, else CompanyConfig)

**Step 1: Create Company objects from CompanyConfig**

Replace the skip block at ~line 7022 with:

```rust
// Build Company objects from config for EntityGraphBuilder
let companies: Vec<Company> = self.config.companies.iter().map(|cc| {
    Company {
        company_code: cc.code.clone(),
        name: cc.name.clone(),
        country: cc.country.clone(),
        currency: cc.currency.clone(),
        // ... map remaining fields from CompanyConfig
    }
}).collect();
```

**Step 2: Build entity graph**

```rust
let entity_graph = EntityGraphBuilder::new()
    .with_companies(&companies)
    .with_intercompany_relationships(&intercompany_snapshot.relationships)
    .build();
```

**Step 3: Add to graph export snapshot**

Ensure the entity graph data is included in `GraphExportSnapshot` and exported.

**Step 4: Run tests and commit**

```bash
cargo test -p datasynth-graph -- entity
cargo test -p datasynth-runtime -- graph
cargo build --release
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat(runtime): wire EntityGraphBuilder with Company→CompanyConfig mapping"
```

---

## Task 6: Country Pack Integration for Remaining Generators

**Files:**
- Modify: `crates/datasynth-generators/src/master_data/employee_generator.rs`
- Modify: `crates/datasynth-generators/src/hr/expense_report_generator.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (pass packs to generators)

**Context:** 8 generators already have `set_country_pack()`. The pattern is: add `country_pack: Option<CountryPack>` field, add `set_country_pack()` method, use `self.country_pack.as_ref()` inside `generate()` for names/formats/rates.

**Step 1: Add country pack to EmployeeGenerator**

Follow the pattern from `vendor_generator.rs:327-330`:

```rust
pub fn set_country_pack(&mut self, pack: datasynth_core::CountryPack) {
    self.country_pack = Some(pack);
}
```

Use pack's `names.cultures` for employee names instead of hardcoded values.

**Step 2: Add country pack to ExpenseReportGenerator**

Use pack's `locale.currency` for expense amounts and `business_rules` for policy thresholds.

**Step 3: Wire in orchestrator**

In the HR phase, after creating each generator, call:
```rust
if let Some(pack) = self.country_pack_for(company.country.as_str()) {
    generator.set_country_pack(pack.clone());
}
```

**Step 4: Run tests and commit**

```bash
cargo test -p datasynth-generators -- employee
cargo test -p datasynth-generators -- expense
cargo build --release
git add crates/datasynth-generators/ crates/datasynth-runtime/
git commit -m "feat(generators): integrate country packs into Employee and ExpenseReport generators"
```

---

## Task 7: Bank Reconciliation Generator

**Files:**
- Create: `crates/datasynth-generators/src/banking/bank_reconciliation_generator.rs`
- Modify: `crates/datasynth-generators/src/banking/mod.rs` (add module)
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (call generator)
- Modify: `crates/datasynth-cli/src/output_writer.rs` (add output)

**Context:** Models exist in `datasynth-core/src/models/banking.rs` (BankReconciliation, BankStatementLine, ReconcilingItem). There's already a `bank_reconciliation_generator.rs` in `datasynth-generators/src/` (check if it exists at root level). If it does, wire it. If not, create one following the pattern of existing generators.

**Step 1: Check if generator already exists**

```bash
find crates/datasynth-generators -name "*reconcil*"
```

**Step 2: Create or wire the generator**

If creating new, follow the pattern from `cycle_count_generator.rs`:
- Constructor with seed
- `generate()` taking bank transactions, expected amounts, dates
- Match transactions to expected amounts with configurable tolerance
- Flag unmatched items as reconciling items

**Step 3: Wire into orchestrator**

Call after banking phase or in financial reporting phase where bank reconciliations are already generated.

**Step 4: Add tests**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_reconciliation() { /* ... */ }
    #[test]
    fn test_unmatched_items() { /* ... */ }
    #[test]
    fn test_deterministic() { /* ... */ }
}
```

**Step 5: Run tests and commit**

```bash
cargo test -p datasynth-generators -- reconcil
cargo build --release
git add crates/datasynth-generators/ crates/datasynth-runtime/ crates/datasynth-cli/
git commit -m "feat(generators): add bank reconciliation generator with statement matching"
```

---

## Task 8: Treasury Bank Guarantees and Netting Runs

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:4549-4718` (Phase 22)
- Reference: `crates/datasynth-core/src/models/treasury.rs` (BankGuarantee, NettingRun models)

**Context:** Models exist. Need generators (check if they exist in `crates/datasynth-generators/src/treasury/`). TreasurySnapshot already has fields. Output_writer already handles them.

**Step 1: Check existing treasury generators**

```bash
ls crates/datasynth-generators/src/treasury/
```

**Step 2: Create BankGuaranteeGenerator if missing**

Follow DebtGenerator pattern:
- Config-driven: guarantee types, amounts, expiry dates
- Link to vendors/customers as beneficiaries
- Status lifecycle (Active, Expired, Called, Released)

**Step 3: Create NettingRunGenerator if missing**

- Take intercompany relationships as input
- Compute bilateral/multilateral netting positions
- Generate settlement entries

**Step 4: Wire into Phase 22 and test**

```bash
cargo test -p datasynth-generators -- treasury
cargo build --release
git add crates/datasynth-generators/src/treasury/ crates/datasynth-runtime/
git commit -m "feat(treasury): add bank guarantee and netting run generators"
```

---

## Task 9: Streaming Anomaly Injection Support

**Files:**
- Modify: `crates/datasynth-runtime/src/streaming_orchestrator.rs:283-299`

**Context:** Currently anomaly injection and data quality phases are skipped in streaming mode because they require post-processing. A pragmatic v1.0 approach: inject anomalies inline during generation rather than as a post-processing step.

**Step 1: Add inline anomaly injection to streaming**

Instead of a separate phase, inject anomalies as items are generated in streaming mode:

```rust
GenerationPhase::AnomalyInjection => {
    if self.config.anomaly_injection.enabled {
        // In streaming mode, anomalies were already injected inline
        // during generation phases. Log summary.
        info!("Streaming mode: anomaly injection applied inline during generation");
    }
}
```

**Step 2: Add anomaly injection hooks to streaming generation phases**

In each streaming generation phase (JE, document flows, etc.), add a probabilistic inline injection check using the anomaly config's injection rate.

**Step 3: Test streaming with anomalies enabled**

```bash
cargo test -p datasynth-runtime -- streaming
cargo build --release
git add crates/datasynth-runtime/
git commit -m "feat(runtime): support inline anomaly injection in streaming mode"
```

---

## Task 10: Production Hardening - Crates.io Metadata

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Modify: All 14 `crates/*/Cargo.toml` files

**Step 1: Add workspace-level keywords and categories**

In root `Cargo.toml` [workspace.package]:

```toml
keywords = ["synthetic-data", "accounting", "financial", "test-data", "data-generation"]
categories = ["simulation", "command-line-utilities"]
documentation = "https://docs.rs/datasynth-core"
```

**Step 2: Add crate-specific metadata to each Cargo.toml**

Each crate should have:
```toml
keywords.workspace = true
categories.workspace = true
documentation.workspace = true
```

For crates with specific focus, override:
- datasynth-banking: `keywords = ["banking", "kyc", "aml", "fraud-detection", "synthetic-data"]`
- datasynth-graph: `keywords = ["graph", "pytorch-geometric", "neo4j", "dgl", "synthetic-data"]`
- datasynth-eval: `keywords = ["evaluation", "data-quality", "benford", "statistical-testing", "synthetic-data"]`

**Step 3: Fix non-workspace reqwest in datasynth-test-utils**

In `crates/datasynth-test-utils/Cargo.toml`, change:
```toml
# FROM:
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
# TO:
reqwest = { workspace = true }
```

**Step 4: Verify with dry-run**

```bash
cargo publish --dry-run -p datasynth-core
cargo publish --dry-run -p datasynth-config
```

**Step 5: Commit**

```bash
git add Cargo.toml crates/*/Cargo.toml crates/datasynth-ui/src-tauri/Cargo.toml
git commit -m "chore(packaging): complete crates.io metadata for all crates"
```

---

## Task 11: Release Process Documentation

**Files:**
- Create: `docs/RELEASING.md`
- Modify: `CHANGELOG.md`

**Step 1: Write release process doc**

Document:
- Versioning policy (SemVer)
- Publishing order (core -> config -> output -> standards -> banking -> generators -> server -> cli -> graph -> eval -> ocpm -> runtime -> fingerprint -> test-utils)
- Pre-release checklist (tests, clippy, fmt, dry-run)
- CHANGELOG format (Keep a Changelog)
- Migration notes template

**Step 2: Add v1.0.0 CHANGELOG section**

```markdown
## [1.0.0] - 2026-03-XX

### Added
- Benefit enrollment generation in HR pipeline
- BOM and inventory movement generation in manufacturing pipeline
- Cash forecasting and cash pooling in treasury pipeline
- Bank reconciliation generator
- CorrelationPreservation quality gate
- EntityGraphBuilder with Company mapping
- Country pack integration for Employee and ExpenseReport generators
- Inline anomaly injection in streaming mode
- Bank guarantee and netting run generators
- Complete crates.io metadata for all crates

### Changed
- Banking generation enabled by default
- Expanded SmallVec usage for performance

### Fixed
- Non-workspace reqwest dependency in datasynth-test-utils
```

**Step 3: Commit**

```bash
git add docs/RELEASING.md CHANGELOG.md
git commit -m "docs: add release process and v1.0.0 CHANGELOG draft"
```

---

## Task 12: Integration Tests for H2R, MFG, Banking

**Files:**
- Create: `crates/datasynth-generators/tests/hr_integration.rs`
- Create: `crates/datasynth-generators/tests/manufacturing_integration.rs`
- Create: `crates/datasynth-banking/tests/banking_integration.rs`

**Step 1: HR integration test**

Test that payroll -> time entries -> expenses -> benefits form a coherent HR dataset:
- Employee IDs are consistent across all generators
- Payroll amounts are within configured salary ranges
- Time entries only on business days
- Benefit enrollments reference valid employees

**Step 2: Manufacturing integration test**

Test that production orders -> quality inspections -> BOM -> inventory movements are coherent:
- Quality inspections reference valid production orders
- BOM components reference valid materials
- Inventory movements reference valid materials and storage locations

**Step 3: Banking integration test**

Expand existing banking tests to cover:
- AML typology injection rates match configuration
- Transaction labels reference valid transactions
- Customer/account relationships are consistent

**Step 4: Run all tests**

```bash
cargo test --workspace
```

**Step 5: Commit**

```bash
git add crates/datasynth-generators/tests/ crates/datasynth-banking/tests/
git commit -m "test: add integration tests for HR, manufacturing, and banking pipelines"
```

---

## Task 13: Performance - Clone Reduction and SmallVec Expansion

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (clone reduction)
- Modify: `crates/datasynth-core/src/models/expense_report.rs` (SmallVec for line items)
- Modify: `crates/datasynth-core/src/models/quality_inspection.rs` (SmallVec for characteristics)

**Step 1: Profile clone hotspots**

Run:
```bash
cargo build --release
cargo bench -- orchestrator
```

Identify the top 10 clone sites by examining `enhanced_orchestrator.rs` (165 clones reported).

**Step 2: Replace clones with references or Arc where possible**

Focus on:
- Config clones: Use `&self.config.X` references instead of `self.config.X.clone()`
- Master data clones: Use `Arc<MasterDataSnapshot>` if passed to multiple phases

**Step 3: Expand SmallVec**

JournalEntry already uses `SmallVec<[JournalEntryLine; 4]>`. Apply same pattern to:
- `ExpenseReport.line_items` -> `SmallVec<[ExpenseLineItem; 4]>`
- `QualityInspection.characteristics` -> `SmallVec<[InspectionCharacteristic; 4]>`
- `BomComponent` collections (typically 2-8 items)

**Step 4: Benchmark to verify improvement**

```bash
cargo bench -- orchestrator
```

Compare before/after.

**Step 5: Commit**

```bash
git add crates/datasynth-runtime/ crates/datasynth-core/
git commit -m "perf: reduce clones in orchestrator and expand SmallVec usage"
```

---

## Task 14: Version Bump to 1.0.0

**Files:**
- Modify: `Cargo.toml` (workspace version)
- Modify: `python/pyproject.toml` (if version tracked)
- Modify: `crates/datasynth-ui/src-tauri/tauri.conf.json` (if version tracked)

**Step 1: Bump workspace version**

In root `Cargo.toml`:
```toml
[workspace.package]
version = "1.0.0"
```

And the workspace root package:
```toml
[package]
version = "1.0.0"
```

**Step 2: Update Cargo.lock**

```bash
cargo check --workspace
```

**Step 3: Final verification**

```bash
cargo test --workspace
cargo clippy --workspace
cargo fmt --check
cargo doc --workspace --no-deps
```

**Step 4: Commit and tag**

```bash
git add Cargo.toml Cargo.lock crates/*/Cargo.toml
git commit -m "chore: bump version to 1.0.0"
git tag -a v1.0.0 -m "DataSynth v1.0.0 - Feature-complete reference release"
```

---

## Execution Order & Dependencies

```
Task 1  (HR benefits)          ─┐
Task 2  (MFG bom+inv)          ─┤── Independent, can run in parallel
Task 3  (Treasury forecast)    ─┤
Task 4  (Eval correlation)     ─┤
Task 6  (Country packs)        ─┘
                                 │
Task 5  (EntityGraph)          ──── Depends on Task 2 (needs MFG data for graph)
Task 7  (Bank reconciliation)  ──── Independent
Task 8  (Treasury guarantees)  ──── Depends on Task 3
Task 9  (Streaming anomalies)  ──── Independent
                                 │
Task 10 (Metadata)             ─┐
Task 11 (Release docs)         ─┤── Independent, can run in parallel
Task 12 (Integration tests)    ─┘── Depends on Tasks 1-3 (tests the new wiring)
                                 │
Task 13 (Performance)          ──── Run after all functional changes
Task 14 (Version bump)         ──── LAST - after all tasks complete and tests pass
```

## Parallel Execution Groups

**Group A** (Tasks 1, 2, 3, 4, 6): Wire existing generators - all independent
**Group B** (Tasks 5, 7, 8, 9): Fill gaps - mostly independent
**Group C** (Tasks 10, 11): Packaging - independent
**Group D** (Task 12): Integration tests - after Groups A+B
**Group E** (Task 13): Performance - after Group D
**Group F** (Task 14): Version bump - after everything
