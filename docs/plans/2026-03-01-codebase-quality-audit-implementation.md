# Codebase Quality Audit — Full Implementation Design

**Date**: 2026-03-01
**Version**: v0.9.5 target
**Scope**: Critical bug fixes, country pack expansion, generator completeness, test coverage

## Context

A comprehensive codebase audit post-v0.9.4 identified issues across 4 priority tiers. This document captures all findings and their implementation design.

---

## Phase 1: Critical Fixes & Code Quality

### 1.1 Mutex Poisoning in Streaming Channel

**File**: `crates/datasynth-core/src/streaming/channel.rs`
**Problem**: 11 `.expect()` calls on mutex locks. If any thread panics while holding the lock, all subsequent operations cascade-panic.
**Fix**: Replace `.expect("...")` with `.unwrap_or_else(|poisoned| poisoned.into_inner())` on all 11 sites (6 `Mutex::lock()` + 5 `Condvar::wait_while()`/`wait_timeout()`).
**Test**: Add `test_channel_recovers_from_poisoned_mutex` — spawn thread that panics while holding lock, verify channel still works.

### 1.2 Credit Memo Generation (O2C Dead Code)

**File**: `crates/datasynth-generators/src/document_flow/o2c_generator.rs`
**Problem**: `returns_rate: f64 = 0.03` exists but is never used. `is_return` always `false`. `ARCreditMemo` model already exists with full approval workflow at `crates/datasynth-core/src/models/subledger/ar/credit_memo.rs`.
**Fix**:
- Add `credit_memo: Option<ARCreditMemo>` to `O2CDocumentChain`
- Add `generate_return_credit_memo()` method to `O2CGenerator`
- In `generate_chain()`, after invoice generation, check `rng.gen_bool(returns_rate)` and generate credit memo if true
- Credit memo references original invoice, credits 10-100% of amount, uses random `CreditMemoReason`

### 1.3 Progressive Tax Bracket Computation

**File**: `crates/datasynth-generators/src/hr/payroll_generator.rs`
**Problem**: Lines 251-258 skip progressive tax types (rate 0.0 placeholder), falling back to config defaults. Country packs have proper `income_tax_brackets` with bracket data.
**Fix**:
- Add `compute_progressive_tax(annual_income, brackets) -> Decimal` helper
- Add `income_tax_brackets: Vec<TaxBracket>` field to `PayrollRates`
- In per-employee loop, when brackets present, compute per-employee tax from annualized salary
- Fallback to flat rate when no brackets available

### 1.4 NetSuite String Allocation Anti-pattern

**File**: `crates/datasynth-output/src/formats/netsuite.rs`
**Problem**: `.unwrap_or_default()` on `Option<String>` and `.unwrap_or(&String::new())` on `HashMap::get()` allocate heap strings for every missing value.
**Fix**: Replace with `.as_deref().unwrap_or("")` and `.map(|s| s.as_str()).unwrap_or("")`.

### 1.5 Hot Path Clones in Orchestrator

**File**: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`
**Problem**: 5 expensive `.clone()` calls: COA, master_data, graph, journal entries, ownership structure.
**Fix**:
- COA: result already uses `Arc<ChartOfAccounts>` internally; propagate `Arc::clone` instead of inner clone
- Journal entries (line 2752): use ownership transfer or `mem::take` since `journal` is local
- Master data: wrap in `Arc` on orchestrator + result struct
- Graph/ownership: transfer ownership where possible, `Arc` where shared

---

## Phase 2: Country Pack Expansion (7 New Countries)

### Architecture

Country packs are JSON files at `crates/datasynth-core/country-packs/`, embedded via `include_str!`, deep-merged onto `_default.json`. Schema in `crates/datasynth-core/src/country/schema.rs` has 16 sections.

### New Countries

| Country | Code | Region | Currency | Framework | Notes |
|---------|------|--------|----------|-----------|-------|
| France | FR | EMEA | EUR | French GAAP (PCG) | Already has accounting framework |
| Japan | JP | APAC | JPY | JGAAP/IFRS | 0-decimal currency, fiscal Apr-Mar |
| China | CN | APAC | CNY | CAS/IFRS | Lunar holidays (existing algorithm) |
| India | IN | APAC | INR | Ind AS/IFRS | Indian number grouping [3,2] |
| Italy | IT | EMEA | EUR | OIC/IFRS | Mandatory e-invoicing (FatturaPA) |
| Spain | ES | EMEA | EUR | PGC/IFRS | SII real-time reporting |
| Canada | CA | AMERICAS | CAD | ASPE/IFRS | Bilingual (EN/FR), close to US |

### Holiday Calendars

Add `Region::FR`, `Region::IT`, `Region::ES`, `Region::CA` to `distributions/holidays.rs` with corresponding holiday functions. JP, CN, IN already have holiday calendars.

### Registration

Add 7 `include_str!` constants + update `builtin_only()` in `crates/datasynth-core/src/country/mod.rs`.

---

## Phase 3: Generator Completeness

### 3.1 OrganizationalEvent Generator

**Model**: `crates/datasynth-core/src/models/organizational_event.rs` (654 lines, 6 event types)
**Config**: `OrganizationalEventsSchemaConfig` in schema.rs
**New file**: `crates/datasynth-generators/src/organizational_event_generator.rs`
**Output**: `Vec<OrganizationalEvent>` with configurable frequency and type distribution

### 3.2 ProcessEvolution Generator

**Model**: `crates/datasynth-core/src/models/process_evolution.rs` (605 lines, S-curve adoption)
**Config**: `ProcessEvolutionSchemaConfig` in schema.rs
**New file**: `crates/datasynth-generators/src/process_evolution_generator.rs`
**Output**: `Vec<ProcessEvolutionEvent>` with workflow transitions and automation rollout

### 3.3 DriftEvent Generator (Meta-Generator)

**Model**: `crates/datasynth-core/src/models/drift_events.rs` (559 lines, 10 drift categories)
**New file**: `crates/datasynth-generators/src/drift_event_generator.rs`
**Dependencies**: Observes OrganizationalEvents and ProcessEvolutionEvents to generate ground-truth drift labels
**Output**: `Vec<LabeledDriftEvent>` for ML training

### 3.4 ISA 505 Confirmation Generator

**Model**: `crates/datasynth-standards/src/audit/confirmation.rs` (634 lines, full ISA 505)
**Config**: `ConfirmationsConfig` in schema.rs (confirmation_count, positive_response_rate)
**New file**: `crates/datasynth-generators/src/standards/confirmation_generator.rs`
**Output**: `Vec<ExternalConfirmation>` — AR/AP/Bank/Legal confirmations with response simulation

---

## Phase 4: Test Coverage Expansion

### 4.1 datasynth-eval Integration Tests

**Current**: 321 inline tests, 0 integration tests
**Add**: Benford analysis, balance sheet evaluation, document chain evaluation, auto-tuner, comprehensive evaluation

### 4.2 datasynth-output Integration Tests

**Current**: 0 tests in csv_sink.rs and json_sink.rs
**Add**: Write/read-back, Unicode handling, special characters, decimal precision, format compliance (FEC, GoBD, NetSuite)

### 4.3 datasynth-config Integration Tests

**Current**: 138 inline tests, limited integration coverage
**Add**: All presets validation, invalid config rejection, config inheritance, preset override behavior

### 4.4 datasynth-banking Integration Tests

**Current**: 64 inline + 1 integration test file
**Add**: Per-typology tests (structuring, layering, funnel, round-tripping), label accuracy verification

---

## Verification

```bash
cargo fmt --all && cargo clippy --workspace
cargo test --workspace
cargo build --release
./target/release/datasynth-data generate --demo --output ./test_output
```

Expected: All tests pass, no clippy warnings (except `protoc not found`).
