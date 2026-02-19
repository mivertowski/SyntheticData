# Codebase Quality Round 5 — Data Correctness, Coherence & Output Integrity

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix critical data flow bugs (empty collections, wrong stats, non-determinism), improve currency coherence, fix serialization inconsistencies, and complete manifest registration.

**Architecture:** Most changes target `enhanced_orchestrator.rs` (data flow bugs, currency wiring) and `output_writer.rs` / `main.rs` (serialization + manifest). Generator changes add currency parameters following the pattern established in round 4 for IC and expense generators.

**Tech Stack:** Rust, serde, rust_decimal, chrono

**Branch:** `feature/codebase-quality-round5` in worktree `.worktrees/codebase-quality-r5`

---

## Phase 1: Critical Data Correctness (6 tasks)

### Task 1: Wire treasury hedge_relationships via generate() API

The orchestrator only calls `generate_ir_swap()` per debt instrument, which returns a single `HedgingInstrument`. The `HedgingGenerator::generate(trade_date, &exposures)` method returns `(Vec<HedgingInstrument>, Vec<HedgeRelationship>)` but is never called, so `snapshot.hedge_relationships` is always empty. The anomaly injector then fires on an empty slice producing zero labels.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:4400-4415`

**Step 1: Build FX exposures from IC transactions and pass to generate()**

After the IR swap loop (line 4415), add a call to `hedge_gen.generate()` using IC transactions or a synthetic set of FX exposures. The orchestrator already has access to `document_flows` which may contain foreign currency payments:

```rust
// After the IR swap loop (~line 4415), add FX hedge generation:
// Build FX exposures from any foreign currency payments in document flows
let fx_exposures: Vec<datasynth_generators::treasury::FxExposure> = document_flows
    .payments
    .iter()
    .filter(|p| p.header.currency != currency)
    .map(|p| datasynth_generators::treasury::FxExposure {
        currency_pair: format!("{}/{}", p.header.currency, currency),
        foreign_currency: p.header.currency.clone(),
        net_amount: p.amount,
        settlement_date: p.header.document_date
            + chrono::Duration::days(30),
        description: format!("AP payment {}", p.header.document_id),
    })
    .collect();

if !fx_exposures.is_empty() {
    let (fx_instruments, fx_relationships) =
        hedge_gen.generate(start_date, &fx_exposures);
    snapshot.hedging_instruments.extend(fx_instruments);
    snapshot.hedge_relationships.extend(fx_relationships);
}
```

**Step 2: Run tests**

```bash
cargo test -p datasynth-runtime -- treasury
cargo clippy -p datasynth-runtime
```

**Step 3: Commit**
```bash
git commit -m "fix: wire treasury hedge_relationships via generate() API"
```

---

### Task 2: Remove premature stats.total_entries update

`stats.total_entries` and `stats.total_line_items` are set inside `phase_journal_entries()` at line 1926, counting only doc-flow + standalone JEs. Then FA, IC, payroll, and manufacturing JEs are appended to `entries` (lines 1394-1466), and the count is overwritten at line 1471. The premature update at 1926 produces an incorrect log message.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:1920-1932`

**Step 1: Remove the premature stats update inside phase_journal_entries**

At ~line 1920-1932, remove or comment out:
```rust
// REMOVE these lines inside phase_journal_entries():
stats.total_entries = entries.len() as u64;
stats.total_line_items = entries.iter().map(|e| e.lines.len() as u64).sum();
```

The authoritative count at line 1471 (after all JE-generating phases complete) handles this correctly.

**Step 2: Run tests**
```bash
cargo test -p datasynth-runtime
```

**Step 3: Commit**
```bash
git commit -m "fix: remove premature stats.total_entries update in phase_journal_entries"
```

---

### Task 3: Fix non-deterministic Local::now() date fallbacks

Two locations use `chrono::Local::now()` as a fallback when parsing `start_date` fails. Since `start_date` is validated earlier, these are unreachable in practice, but they break determinism guarantees if ever triggered.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:4703-4705` and `5951-5953`

**Step 1: Replace both Local::now() fallbacks with SynthError**

At line ~4705 (subledger reconciliation):
```rust
// BEFORE:
.unwrap_or_else(|_| chrono::Local::now().date_naive());

// AFTER:
.map_err(|e| SynthError::config(format!("Invalid start_date: {}", e)))?;
```

Same pattern at line ~5953 (balance validation).

Note: These methods already return `SynthResult<...>`, so `?` propagation works directly.

**Step 2: Run tests**
```bash
cargo test -p datasynth-runtime
```

**Step 3: Commit**
```bash
git commit -m "fix: replace non-deterministic Local::now() fallbacks with error propagation"
```

---

### Task 4: Fix exchange_rate missing serde string annotation

`JournalEntryHeader.exchange_rate` at line 204 is `Decimal` without `#[serde(with = "rust_decimal::serde::str")]`, so it serializes as a JSON number (float). All other monetary `Decimal` fields on `JournalEntryLine` serialize as strings. This causes inconsistency for consumers.

**Files:**
- Modify: `crates/datasynth-core/src/models/journal_entry.rs:203-204`

**Step 1: Add serde str annotation**

```rust
// BEFORE:
/// Exchange rate to local currency (1.0 if same currency)
pub exchange_rate: Decimal,

// AFTER:
/// Exchange rate to local currency (1.0 if same currency)
#[serde(with = "rust_decimal::serde::str")]
pub exchange_rate: Decimal,
```

**Step 2: Run tests** (this will change existing test output expectations)
```bash
cargo test -p datasynth-core -- journal_entry
cargo test --workspace
```

**Step 3: Commit**
```bash
git commit -m "fix: serialize exchange_rate as string for consistency with other Decimal fields"
```

---

### Task 5: Fix CSV source field using Debug format instead of snake_case

The CSV writer at `output_writer.rs:64` uses `{:?}` for `h.source`, producing `Manual` / `Automated` (Debug format). JSON serialization via serde produces `manual` / `automated` (snake_case). This breaks consumers joining CSV and JSON on this field.

**Files:**
- Modify: `crates/datasynth-cli/src/output_writer.rs:64-77`
- Modify: `crates/datasynth-core/src/models/journal_entry.rs` — add `Display` impl for `TransactionSource`

**Step 1: Add Display impl for TransactionSource**

In `journal_entry.rs` after the `TransactionSource` enum definition (~line 35):
```rust
impl std::fmt::Display for TransactionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "manual"),
            Self::Automated => write!(f, "automated"),
            Self::Recurring => write!(f, "recurring"),
            Self::Reversal => write!(f, "reversal"),
            Self::Adjustment => write!(f, "adjustment"),
            Self::Statistical => write!(f, "statistical"),
        }
    }
}
```

**Step 2: Update CSV format string**

In `output_writer.rs:64`, change `{:?}` at position 13 to `{}`:
```
"{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}"
```
(All 25 positions use `{}` now, no `{:?}`)

**Step 3: Run tests**
```bash
cargo test -p datasynth-cli
cargo test -p datasynth-core -- transaction_source
```

**Step 4: Commit**
```bash
git commit -m "fix: CSV source field uses snake_case Display instead of Debug format"
```

---

### Task 6: Wire DataQualityConfig from schema instead of hardcoded minimal()

The orchestrator's `inject_data_quality()` at line 6000 ignores `self.config.data_quality` entirely and hardcodes `DataQualityConfig::minimal()`. All user-configured data quality rates are silently ignored.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:5998-6001`

**Step 1: Check if a from_schema method exists**

```bash
grep -n "fn from_schema\|fn from_config\|DataQualityConfig::new\|impl.*From.*DataQuality" crates/datasynth-generators/src/data_quality/*.rs
```

If no conversion exists, create a simple one that maps schema rates to the generator config, or at minimum pass through the `enabled` flag and rates.

**Step 2: Replace hardcoded config**

If a constructor from schema exists, use it. Otherwise, the minimal fix is:
```rust
// BEFORE:
let config = DataQualityConfig::minimal();

// AFTER (minimal viable):
let config = if self.config.data_quality.enabled {
    DataQualityConfig::from_rates(
        self.config.data_quality.missing_values.mcar_rate,
        self.config.data_quality.typos.keyboard_rate,
        self.config.data_quality.format_variations.date_format_mix_rate,
        self.config.data_quality.duplicates.exact_duplicate_rate,
    )
} else {
    DataQualityConfig::minimal()
};
```

Exact implementation depends on what `DataQualityConfig` accepts. If no `from_rates` constructor exists, create one in `datasynth-generators/src/data_quality/`.

**Step 3: Run tests**
```bash
cargo test -p datasynth-generators -- data_quality
cargo test -p datasynth-runtime
```

**Step 4: Commit**
```bash
git commit -m "fix: wire DataQualityConfig from schema config instead of hardcoded minimal()"
```

---

## Phase 2: Currency Coherence (3 tasks)

### Task 7: Fix hardcoded "USD" in inventory_generator

The inventory generator has 8 instances of hardcoded `"USD"` strings. It should accept a `currency` parameter.

**Files:**
- Modify: `crates/datasynth-generators/src/subledger/inventory_generator.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (caller)

**Step 1: Add currency field to InventoryGenerator or its config**

Follow the pattern from round 4's IC generator fix:
- Add `currency: String` to the generator struct or its config
- Replace all 8 `"USD".to_string()` with `self.currency.clone()` (or `currency.to_string()`)

**Step 2: Update orchestrator caller to pass company currency**

In `enhanced_orchestrator.rs`, where `InventoryGenerator` is constructed/called, pass the company currency (same pattern as IC and expense report generators from round 4).

**Step 3: Run tests**
```bash
cargo test -p datasynth-generators -- inventory
cargo test -p datasynth-runtime
```

**Step 4: Commit**
```bash
git commit -m "fix: inventory_generator accepts currency parameter instead of hardcoded USD"
```

---

### Task 8: Fix hardcoded "USD" in balance_tracker and opening_balance_generator

`balance_tracker.rs` has 4 instances and `opening_balance_generator.rs` has 2 instances of hardcoded `"USD"`.

**Files:**
- Modify: `crates/datasynth-generators/src/balance/balance_tracker.rs`
- Modify: `crates/datasynth-generators/src/balance/opening_balance_generator.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (callers)

**Step 1: Add currency parameter to both generators**

For `balance_tracker.rs`:
- Add `currency: String` field to `BalanceTracker` struct
- Replace 4 `"USD".to_string()` with `self.currency.clone()`

For `opening_balance_generator.rs`:
- Add `currency: String` to `OpeningBalanceGenerator` or its config
- Replace 2 `"USD".to_string()` with `self.currency.clone()`

**Step 2: Update orchestrator callers to pass company currency**

**Step 3: Run tests**
```bash
cargo test -p datasynth-generators -- balance
cargo test -p datasynth-runtime
```

**Step 4: Commit**
```bash
git commit -m "fix: balance generators accept currency parameter instead of hardcoded USD"
```

---

### Task 9: Fix hardcoded "USD" in IC matching_engine

`matching_engine.rs` line 60 hardcodes `base_currency: "USD".to_string()`. It should use the IC config's `default_currency` (added in round 4).

**Files:**
- Modify: `crates/datasynth-generators/src/intercompany/matching_engine.rs:60`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (caller)

**Step 1: Add currency parameter to MatchingEngine::new()**

**Step 2: Pass IC default_currency from orchestrator**

**Step 3: Run tests**
```bash
cargo test -p datasynth-generators -- matching
cargo test -p datasynth-runtime -- intercompany
```

**Step 4: Commit**
```bash
git commit -m "fix: IC matching_engine uses config currency instead of hardcoded USD"
```

---

## Phase 3: Fabricated ID Cleanup (1 task)

### Task 10: Replace fabricated IDs in HR generators with pool-based selection

Three HR generators fabricate cross-reference IDs that don't match any generated entity:
- `time_entry_generator.rs`: `PROJ-{:04}`, `CC-{:03}`, `MGR-{:04}`
- `expense_report_generator.rs`: `MGR-{:04}`, `CC-{:03}`
- `payroll_generator.rs`: `USR-{:04}`, `CC-100`/`CC-200`/`CC-300`

**Files:**
- Modify: `crates/datasynth-generators/src/hr/time_entry_generator.rs`
- Modify: `crates/datasynth-generators/src/hr/expense_report_generator.rs`
- Modify: `crates/datasynth-generators/src/hr/payroll_generator.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (callers)

**Step 1: Add employee_ids and cost_center_ids parameters to generators**

For each generator, add optional pools:
```rust
pub fn generate_with_pools(
    &mut self,
    // ... existing params ...
    employee_ids: &[String],
    cost_center_ids: &[String],
) -> ...
```

When `employee_ids` is non-empty, select approved_by / posted_by from the pool using `self.rng.gen_range()`. When `cost_center_ids` is non-empty, select cost_centers from it. Fall back to current fabricated values if pools are empty (backward compatibility).

**Step 2: Update orchestrator to pass employee IDs and cost centers**

The orchestrator has `self.master_data.employees` available. Extract IDs:
```rust
let employee_ids: Vec<String> = self.master_data.employees.iter().map(|e| e.employee_id.clone()).collect();
let cost_center_ids: Vec<String> = self.master_data.employees.iter()
    .filter_map(|e| e.cost_center.clone())
    .collect::<std::collections::HashSet<_>>()
    .into_iter().collect();
```

**Step 3: Run tests**
```bash
cargo test -p datasynth-generators -- hr
cargo test -p datasynth-runtime
```

**Step 4: Commit**
```bash
git commit -m "fix: HR generators use actual employee/cost-center IDs instead of fabricated ones"
```

---

## Phase 4: Manifest Registration (1 task)

### Task 11: Register all missing manifest entries in main.rs

The manifest at `main.rs:751-911` only registers ~30 of ~90 written files. Add entries for all files written by `output_writer.rs`.

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs:751-911`

**Step 1: Audit output_writer.rs for all written paths**

Cross-reference every `write_json` / `write_json_safe` call's output path with `register()` calls.

**Step 2: Add missing register() calls**

Add entries for (grouped by section):

**Treasury** (missing 6):
- `treasury/hedge_relationships.json` → `result.treasury.hedge_relationships.len()`
- `treasury/cash_positions.json` → `result.treasury.cash_positions.len()`
- `treasury/cash_forecasts.json` → `result.treasury.cash_forecasts.len()`
- `treasury/cash_pools.json` → `result.treasury.cash_pools.len()`
- `treasury/cash_pool_sweeps.json` → `result.treasury.cash_pool_sweeps.len()`
- `treasury/treasury_anomaly_labels.json` → `result.treasury.treasury_anomaly_labels.len()`

**Project Accounting** (missing 3):
- `project_accounting/cost_lines.json` → `result.project_accounting.cost_lines.len()`
- `project_accounting/revenue_records.json` → `result.project_accounting.revenue_records.len()`
- `project_accounting/earned_value_metrics.json` → `result.project_accounting.earned_value_metrics.len()`

**Tax** (missing 6):
- `tax/tax_jurisdictions.json` → `result.tax.tax_jurisdictions.len()`
- `tax/tax_codes.json` → `result.tax.tax_codes.len()`
- `tax/tax_lines.json` → `result.tax.tax_lines.len()`
- `tax/tax_returns.json` → `result.tax.tax_returns.len()`
- `tax/withholding_records.json` → `result.tax.withholding_records.len()`
- `tax/tax_anomaly_labels.json` → `result.tax.tax_anomaly_labels.len()`

**ESG** (missing ~13):
- All files under `esg/` written by output_writer

**Banking** (missing 5):
- `banking/banking_accounts.json`, `banking/aml_transaction_labels.json`, etc.

**Sourcing** (missing 8):
- `sourcing/spend_analyses.json`, `sourcing/supplier_qualifications.json`, etc.

**Audit** (missing 5):
- `audit/audit_workpapers.json`, `audit/audit_evidence.json`, etc.

**Intercompany** (missing 2):
- `intercompany/ic_seller_journal_entries.json`, `intercompany/ic_buyer_journal_entries.json`

**HR** (missing 1):
- `hr/payroll_line_items.json`

**Step 3: Run tests**
```bash
cargo test -p datasynth-cli
```

**Step 4: Commit**
```bash
git commit -m "fix: register all ~50 missing output files in run manifest"
```

---

## Phase 5: Verification (2 tasks)

### Task 12: cargo fmt + clippy + full test suite

```bash
cargo fmt --all
cargo clippy --workspace --exclude datasynth-ui  # 0 warnings expected (only protoc not found)
cargo test --workspace --exclude datasynth-ui
```

Fix any issues introduced.

### Task 13: Demo generation end-to-end verification

```bash
cargo run -p datasynth-cli -- generate --demo --output /tmp/r5-test
```

Verify:
- `generation_statistics.json` total_entries matches actual JE count in `journal_entries.csv`
- `run_manifest.json` lists all written files
- No panics or empty outputs for enabled features
- Treasury `hedge_relationships.json` is non-empty (if treasury enabled)
- `journal_entries.json` has exchange_rate as string, not float

---

## Critical Files

| File | Role |
|------|------|
| `crates/datasynth-runtime/src/enhanced_orchestrator.rs` | Central orchestrator — most changes |
| `crates/datasynth-core/src/models/journal_entry.rs` | exchange_rate annotation + Display impl |
| `crates/datasynth-cli/src/output_writer.rs` | CSV format fix |
| `crates/datasynth-cli/src/main.rs` | Manifest registration |
| `crates/datasynth-generators/src/subledger/inventory_generator.rs` | Currency param |
| `crates/datasynth-generators/src/balance/balance_tracker.rs` | Currency param |
| `crates/datasynth-generators/src/balance/opening_balance_generator.rs` | Currency param |
| `crates/datasynth-generators/src/intercompany/matching_engine.rs` | Currency param |
| `crates/datasynth-generators/src/hr/time_entry_generator.rs` | Pool-based IDs |
| `crates/datasynth-generators/src/hr/expense_report_generator.rs` | Pool-based IDs |
| `crates/datasynth-generators/src/hr/payroll_generator.rs` | Pool-based IDs |

## Execution Order

```
Phase 1: Tasks 1-6 (data correctness) — sequential, mixed files
Phase 2: Tasks 7-9 (currency coherence) — parallelizable
Phase 3: Task 10 (fabricated IDs) — independent
Phase 4: Task 11 (manifest) — independent
Phase 5: Tasks 12-13 (verification) — final
```

## Deferred Findings (for future rounds)

These are real issues found by the five-agent audit but deferred as out-of-scope:

**Config Cleanup (large scope):**
- 17+ dead config sections never read at runtime (streaming, rate_limit, temporal_attributes, relationships, scenario behavioral fields, drift sections, industry_specific, fingerprint_privacy, webhooks, compliance, user_personas, departments)
- 11+ sections with no validation (data_quality, scenario, temporal drift, audit, source_to_pay, financial_reporting, hr, manufacturing, sales_quotes, tax, treasury, project_accounting, esg)
- Preset sections identical across industries (master_data counts, fraud types, distributions)
- `output.formats`, `output.partition_by_*`, `output.include_bseg`, `output.batch_size`, `output.compression` — all parsed but not implemented

**Output Pipeline (medium scope):**
- 5 CSV exporters in `datasynth-output` never called from CLI
- CLAUDE.md lists ~15 data types that are never written (acdoca, payslips, leases, fair_value_measurements, etc.)
- Parquet sink never called from standard generation path

**Empty Snapshot Fields:**
- `TreasurySnapshot.cash_forecasts`, `.cash_pools`, `.cash_pool_sweeps` — always empty, no generator wired
- `ProjectAccountingSnapshot.revenue_records` — no generator called
- `SubledgerSnapshot.inventory_movements` — no generator called

**Config Gating Inconsistencies:**
- OCPM ignores `config.ocpm.enabled`
- Audit ignores `config.audit.enabled`
- Treasury, HR, Project Accounting have no PhaseConfig flags

## Verification

```bash
cargo fmt --all
cargo clippy --workspace --exclude datasynth-ui  # 0 warnings
cargo test --workspace --exclude datasynth-ui     # all pass
cargo run -p datasynth-cli -- generate --demo --output /tmp/r5-test
```
