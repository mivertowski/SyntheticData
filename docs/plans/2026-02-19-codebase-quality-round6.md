# Codebase Quality Round 6 — GL Account Correctness, Cross-Reference Coherence, Code Consistency

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix GL account collisions/misuses, wire remaining fabricated IDs to real master data pools, standardize RNG construction and tracing across all generators, and clean up dead code.

**Architecture:** Three-phase approach — fix data correctness bugs first (GL accounts), then cross-reference coherence (fabricated IDs), then mechanical consistency (RNG, tracing, dead code). Each phase is mostly independent; tasks within a phase are parallelizable.

**Tech Stack:** Rust, datasynth-core accounts.rs constants, seeded_rng() utility, tracing::debug!

---

## Phase 1: GL Account Correctness (4 tasks)

### Task 1: Fix year_end.rs GL account collisions

**Complexity: S** | **Risk: HIGH — semantic bugs in closing entries**

**Files:**
- Modify: `crates/datasynth-generators/src/period_close/year_end.rs`
- Reference: `crates/datasynth-core/src/accounts.rs`

**Context:** `YearEndCloseConfig::default()` uses hardcoded GL strings that collide with `accounts.rs` constants:
- `"3500"` for income_summary but `equity_accounts::CTA = "3500"` in accounts.rs
- `"2300"` for current_tax_payable but `liability_accounts::UNEARNED_REVENUE = "2300"`; should be `tax_accounts::SALES_TAX_PAYABLE = "2100"`
- `"2350"` for deferred_tax_liability but `accounts.rs` has `DEFERRED_TAX_LIABILITY = "2500"`
- `"7100"` for tax_expense but `expense_accounts::INTEREST_EXPENSE = "7100"`; should be `tax_accounts::TAX_EXPENSE = "8000"`
- `"1600"` for deferred_tax_asset — consistent with `tax_accounts::DEFERRED_TAX_ASSET = "1600"` (OK)
- `"3300"` for retained_earnings — consistent with `equity_accounts::RETAINED_EARNINGS = "3300"` (OK)
- `"3400"` for dividend_account — consistent with `equity_accounts::DIVIDENDS_PAID = "3400"` (OK)

**Step 1: Add `INCOME_SUMMARY` constant to accounts.rs**

Add to `equity_accounts` module: `pub const INCOME_SUMMARY: &str = "3600";` (new account, avoids CTA collision with "3500").

**Step 2: Fix year_end.rs defaults**

Replace the hardcoded defaults in `YearEndCloseConfig::default()`:
```rust
income_summary_account: equity_accounts::INCOME_SUMMARY.to_string(),      // "3600" (was "3500" = CTA)
retained_earnings_account: equity_accounts::RETAINED_EARNINGS.to_string(), // "3300" (unchanged)
dividend_account: equity_accounts::DIVIDENDS_PAID.to_string(),             // "3400" (unchanged)
current_tax_payable_account: tax_accounts::SALES_TAX_PAYABLE.to_string(),  // "2100" (was "2300" = UNEARNED_REVENUE)
deferred_tax_liability_account: liability_accounts::DEFERRED_TAX_LIABILITY.to_string(), // "2500" (was "2350")
deferred_tax_asset_account: tax_accounts::DEFERRED_TAX_ASSET.to_string(),  // "1600" (unchanged)
tax_expense_account: tax_accounts::TAX_EXPENSE.to_string(),                // "8000" (was "7100" = INTEREST_EXPENSE)
```

Add the necessary imports: `use datasynth_core::accounts::{equity_accounts, tax_accounts, liability_accounts};`

**Step 3: Run tests**

Run: `cargo test -p datasynth-generators -- year_end`
Expected: All pass (defaults changed, tests should still work since they exercise the config, not hardcoded values)

**Step 4: Commit**

```bash
git add crates/datasynth-core/src/accounts.rs crates/datasynth-generators/src/period_close/year_end.rs
git commit -m "fix: year_end.rs GL account collisions — use accounts.rs constants"
```

---

### Task 2: Fix depreciation.rs expense account

**Complexity: S** | **Risk: HIGH — depreciation posted to wrong GL**

**Files:**
- Modify: `crates/datasynth-generators/src/period_close/depreciation.rs`
- Reference: `crates/datasynth-core/src/accounts.rs`

**Context:** Line 29 uses `"6100"` for `default_expense_account` but `expense_accounts::SALARIES_WAGES = "6100"`. Should be `expense_accounts::DEPRECIATION = "6000"`.

**Step 1: Fix the default**

Replace:
```rust
default_expense_account: "6100".to_string(),
```
With:
```rust
default_expense_account: expense_accounts::DEPRECIATION.to_string(),
```

Also replace `"1510"` with the constant:
```rust
default_accum_depr_account: control_accounts::ACCUMULATED_DEPRECIATION.to_string(),
```

Add import: `use datasynth_core::accounts::{expense_accounts, control_accounts};`

**Step 2: Run tests**

Run: `cargo test -p datasynth-generators -- depreciation`
Expected: All pass

**Step 3: Commit**

```bash
git add crates/datasynth-generators/src/period_close/depreciation.rs
git commit -m "fix: depreciation expense posted to SALARIES_WAGES — use DEPRECIATION constant"
```

---

### Task 3: Fix AR generator credit memo tax account

**Complexity: S** | **Risk: MEDIUM — wrong tax account in credit memo JE**

**Files:**
- Modify: `crates/datasynth-generators/src/subledger/ar_generator.rs`
- Reference: `crates/datasynth-core/src/accounts.rs`

**Context:** Line ~431 uses `"2300"` (UNEARNED_REVENUE) for a tax debit in credit memo JE. Should be `tax_accounts::SALES_TAX_PAYABLE` ("2100"). Also check other hardcoded strings in this file and replace with constants where they exist.

**Step 1: Fix credit memo JE GL accounts**

In `generate_credit_memo_je`, replace hardcoded `"2300"` with `tax_accounts::SALES_TAX_PAYABLE.to_string()`. Check that the AR control account and revenue account references also use constants.

Add import if not present: `use datasynth_core::accounts::tax_accounts;`

**Step 2: Run tests**

Run: `cargo test -p datasynth-generators -- ar_generator`
Expected: All pass

**Step 3: Commit**

```bash
git add crates/datasynth-generators/src/subledger/ar_generator.rs
git commit -m "fix: AR credit memo uses UNEARNED_REVENUE for tax — use SALES_TAX_PAYABLE"
```

---

### Task 4: Fix AP generator hardcoded GL strings

**Complexity: S** | **Risk: MEDIUM — inconsistent GL accounts**

**Files:**
- Modify: `crates/datasynth-generators/src/subledger/ap_generator.rs`
- Modify: `crates/datasynth-core/src/accounts.rs` (add missing constants)
- Reference: `crates/datasynth-core/src/accounts.rs`

**Context:** AP generator uses bare string literals at lines 129, 262, 272, 283, 305, 346. Key issues:
- `"1400"` for VAT/tax receivable but `accounts.rs` has `INPUT_VAT = "1160"` — these may be intentionally different accounts; investigate
- `"4800"` for discount income has no constant in `accounts.rs`

**Step 1: Add missing constants to accounts.rs**

Add to `income_accounts` or appropriate module:
- `pub const PURCHASE_DISCOUNTS: &str = "4800";`

Check if `"1400"` should be `INPUT_VAT` ("1160") or if it's a separate account (tax receivable vs input VAT). If intentionally different, add `pub const TAX_RECEIVABLE: &str = "1400";` to `asset_accounts`.

**Step 2: Replace hardcoded strings in ap_generator.rs**

Replace all bare GL string literals with the appropriate constants from `accounts.rs`. Import the relevant modules.

**Step 3: Run tests**

Run: `cargo test -p datasynth-generators -- ap_generator`
Expected: All pass

**Step 4: Commit**

```bash
git add crates/datasynth-core/src/accounts.rs crates/datasynth-generators/src/subledger/ap_generator.rs
git commit -m "fix: AP generator GL strings — use accounts.rs constants, add missing PURCHASE_DISCOUNTS"
```

---

## Phase 2: Cross-Reference Coherence (3 tasks)

### Task 5: Add currency + employee pool to SalesQuoteGenerator

**Complexity: M** | **Risk: MEDIUM — fabricated IDs + wrong currency**

**Files:**
- Modify: `crates/datasynth-generators/src/sales_quote_generator.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (caller)

**Context:** SalesQuoteGenerator has three issues:
1. Hardcoded `"USD"` currency (line 220)
2. Fabricated `SR-{:02}` sales rep IDs (line 208) — should come from employee pool
3. Fabricated `SO-{:06}` sales order IDs (line 186) — should come from real O2C sales orders

**Step 1: Add `with_pools()` builder to SalesQuoteGenerator**

Add fields and builder method:
```rust
employee_ids_pool: Vec<String>,
customer_ids_pool: Vec<String>,

pub fn with_pools(mut self, employee_ids: Vec<String>, customer_ids: Vec<String>) -> Self {
    self.employee_ids_pool = employee_ids;
    self.customer_ids_pool = customer_ids;
    self
}
```

**Step 2: Add currency parameter to generate()**

Add `currency: &str` parameter. Replace `"USD".to_string()` with `currency.to_string()`.

**Step 3: Use employee pool for sales_rep_id**

When pool is non-empty, draw sales_rep_id from it. Keep `SR-{:02}` fallback when pool is empty.

**Step 4: Use customer pool for customer_id**

When pool is non-empty, draw customer_id from it instead of fabricating.

**Step 5: Accept optional sales_order_ids for won quotes**

Add `sales_order_ids: Option<&[String]>` parameter. When `Some`, pick a real SO ID for won quotes. When `None`, keep `SO-{:06}` fallback.

**Step 6: Update orchestrator caller**

At the orchestrator call site (~line 3887), pass:
- Company currency
- Employee IDs from `self.master_data.employees`
- Customer IDs from `self.master_data.customers`
- Sales order IDs from `document_flows.sales_orders` (if available)

**Step 7: Run tests**

Run: `cargo test -p datasynth-generators -- sales_quote && cargo test -p datasynth-runtime`
Expected: All pass

**Step 8: Commit**

```bash
git add crates/datasynth-generators/src/sales_quote_generator.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "fix: SalesQuoteGenerator — wire currency, employee pool, customer pool, SO IDs"
```

---

### Task 6: Add employee pool to BankReconciliationGenerator

**Complexity: S** | **Risk: MEDIUM — fabricated preparer/reviewer IDs**

**Files:**
- Modify: `crates/datasynth-generators/src/bank_reconciliation_generator.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (caller)

**Context:** Lines 278-283 fabricate `USR-{:04}` IDs for preparer and reviewer. Same pattern fixed in HR generators in round 5.

**Step 1: Add `with_employee_pool()` builder**

```rust
employee_ids_pool: Vec<String>,

pub fn with_employee_pool(mut self, employee_ids: Vec<String>) -> Self {
    self.employee_ids_pool = employee_ids;
    self
}
```

**Step 2: Use pool for preparer/reviewer**

When pool is non-empty, draw preparer_id and reviewer_id from it. Keep `USR-{:04}` fallback.

**Step 3: Update orchestrator caller**

At the orchestrator call site (~line 2927), pass employee IDs from `self.master_data.employees`.

**Step 4: Run tests**

Run: `cargo test -p datasynth-generators -- bank_reconciliation && cargo test -p datasynth-runtime`
Expected: All pass

**Step 5: Commit**

```bash
git add crates/datasynth-generators/src/bank_reconciliation_generator.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "fix: BankReconciliationGenerator — wire employee pool for preparer/reviewer"
```

---

### Task 7: Add employee pool to CycleCountGenerator

**Complexity: S** | **Risk: MEDIUM — fabricated counter/supervisor IDs**

**Files:**
- Modify: `crates/datasynth-generators/src/manufacturing/cycle_count_generator.rs`
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (caller)

**Context:** Lines 90-94 fabricate `WH-{:02}` (counter), `SUP-{:02}` (supervisor), and `WH-{:03}` (warehouse) IDs.

**Step 1: Add `with_employee_pool()` builder**

```rust
employee_ids_pool: Vec<String>,

pub fn with_employee_pool(mut self, employee_ids: Vec<String>) -> Self {
    self.employee_ids_pool = employee_ids;
    self
}
```

**Step 2: Use pool for counter_id and supervisor_id**

When pool is non-empty, draw from it. Keep `WH-{:02}`/`SUP-{:02}` fallback. Leave `warehouse_id` as `WH-{:03}` for now (no warehouse master data exists).

**Step 3: Update orchestrator caller**

At the orchestrator call site (~line 3816), pass employee IDs from `self.master_data.employees`.

**Step 4: Run tests**

Run: `cargo test -p datasynth-generators -- cycle_count && cargo test -p datasynth-runtime`
Expected: All pass

**Step 5: Commit**

```bash
git add crates/datasynth-generators/src/manufacturing/cycle_count_generator.rs crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "fix: CycleCountGenerator — wire employee pool for counter/supervisor"
```

---

## Phase 3: Code Consistency (5 tasks)

### Task 8: Replace raw ChaCha8Rng::seed_from_u64 with seeded_rng()

**Complexity: M (mechanical but many files)** | **Risk: LOW**

**Files:** ~35 generator files across sourcing/, manufacturing/, standards/, intercompany/, treasury/, master_data/, audit/, esg/, period_close/, relationships/

**Context:** `seeded_rng(seed, discriminator)` from `datasynth_core::utils` is the canonical constructor. ~35 generators bypass it by calling `ChaCha8Rng::seed_from_u64(seed)` directly. Both are functionally equivalent for discriminator=0, but inconsistent.

**Step 1: In each affected file**

Replace:
```rust
use rand_chacha::ChaCha8Rng;
// ...
rng: ChaCha8Rng::seed_from_u64(seed),
```

With:
```rust
use datasynth_core::utils::seeded_rng;
// ...
rng: seeded_rng(seed, 0),
```

Remove unused `use rand::SeedableRng;` imports where present (only needed for `seed_from_u64`).

**Affected files (non-exhaustive — grep for `ChaCha8Rng::seed_from_u64` in generators/):**
- `control_generator.rs`
- `manufacturing/production_order_generator.rs`, `quality_inspection_generator.rs`, `cycle_count_generator.rs`
- `sourcing/rfx_generator.rs`, `contract_generator.rs`, `bid_generator.rs`, `bid_evaluation_generator.rs`, `scorecard_generator.rs`, `qualification_generator.rs`, `sourcing_project_generator.rs`, `spend_analysis_generator.rs`
- `standards/revenue_recognition_generator.rs`, `impairment_generator.rs`
- `intercompany/ic_generator.rs`
- `treasury/cash_position_generator.rs`, `debt_generator.rs`, `hedging_generator.rs`, `cash_forecast_generator.rs`, `cash_pool_generator.rs`
- `relationships/entity_graph_generator.rs`
- `master_data/material_generator.rs`, `vendor_generator.rs`, `customer_generator.rs`, `asset_generator.rs`, `employee_generator.rs`
- `audit/engagement_generator.rs`, `evidence_generator.rs`, `judgment_generator.rs`, `finding_generator.rs`, `workpaper_generator.rs`, `risk_generator.rs`
- `esg/emission_generator.rs`, `supplier_esg_generator.rs`
- `period_close/financial_statement_generator.rs`

**Step 2: Run tests**

Run: `cargo test --workspace --exclude datasynth-ui`
Expected: All pass (functional behavior identical)

**Step 3: Commit**

```bash
git add -A
git commit -m "refactor: replace ChaCha8Rng::seed_from_u64 with seeded_rng() across all generators"
```

---

### Task 9: Add tracing::debug! to generators missing it

**Complexity: M (mechanical)** | **Risk: LOW**

**Files:** ~20 generator files across sourcing/, treasury/, standards/, manufacturing/, sales_quote, ar

**Context:** The established convention is `tracing::debug!` at the start of each public `generate()` method. ~20 generators are missing this.

**Step 1: Add tracing to each affected generator**

For each generator, add at the top of the public `generate()` method:
```rust
debug!(param_count = inputs.len(), "Generating [type]");
```

Include relevant context fields (counts, date ranges, etc.) in the debug log.

Add `use tracing::debug;` import to each file.

**Affected files:**
- All 8 sourcing generators
- All 5 treasury generators
- `standards/revenue_recognition_generator.rs`, `impairment_generator.rs`
- `manufacturing/cycle_count_generator.rs`
- `sales_quote_generator.rs`
- `subledger/ar_generator.rs` (generate_invoice)

**Step 2: Run tests**

Run: `cargo test --workspace --exclude datasynth-ui`
Expected: All pass

**Step 3: Commit**

```bash
git add -A
git commit -m "feat: add tracing::debug! to all generate() entry points"
```

---

### Task 10: Remove dead code

**Complexity: M** | **Risk: LOW**

**Files:**
- `crates/datasynth-eval/src/report/comparison.rs` — remove `BaselineComparer` (~180 lines)
- 9 ESG + project accounting generators — remove dead `uuid_factory` fields
- `crates/datasynth-generators/src/balance/trial_balance_generator.rs` — remove `calculate_period_variances`
- `crates/datasynth-generators/src/balance/balance_tracker.rs` — remove `apply_line`
- `crates/datasynth-eval/src/enhancement/auto_tuner.rs` — remove `MultiplyByGapFactor` variant
- `crates/datasynth-generators/src/master_data/vendor_generator.rs` — remove `_spend_category` binding

**Step 1: Remove BaselineComparer**

Delete lines ~244-445 in `comparison.rs` (the struct, MetricDefinition, and impl block). The pub re-exports in `report/mod.rs` don't include it, so nothing breaks.

**Step 2: Remove dead uuid_factory from 9 generators**

In each of these files, remove the `uuid_factory` field, its `#[allow(dead_code)]` annotation, the comment, and the `DeterministicUuidFactory::new(...)` call in `new()`:
- `esg/emission_generator.rs`
- `esg/supplier_esg_generator.rs`
- `esg/disclosure_generator.rs`
- `esg/workforce_generator.rs`
- `esg/energy_generator.rs`
- `project_accounting/project_generator.rs`
- `project_accounting/change_order_generator.rs` (×2)
- `project_accounting/earned_value_generator.rs`
- `project_accounting/project_cost_generator.rs`

Remove now-unused `use datasynth_core::uuid_factory::...` imports.

**Step 3: Remove dead functions and variants**

- `trial_balance_generator.rs`: Remove `calculate_period_variances` function and its `#[allow(dead_code)]`
- `balance_tracker.rs`: Remove `apply_line` method and its `#[allow(dead_code)]`
- `auto_tuner.rs`: Remove `MultiplyByGapFactor` variant and its `#[allow(dead_code)]`
- `vendor_generator.rs`: Remove `let _spend_category = category;` binding

**Step 4: Run tests**

Run: `cargo test --workspace --exclude datasynth-ui`
Expected: All pass

**Step 5: Commit**

```bash
git add -A
git commit -m "refactor: remove dead code — BaselineComparer, uuid_factory fields, dead functions"
```

---

### Task 11: Fix clippy items and minor code quality

**Complexity: S** | **Risk: LOW**

**Files:**
- `crates/datasynth-generators/src/subledger/document_flow_linker.rs` — derive Default
- `crates/datasynth-generators/src/industry/healthcare/settings.rs` — field_reassign_with_default
- `crates/datasynth-generators/src/fraud/red_flags.rs` — field_reassign_with_default
- `crates/datasynth-test-utils/src/server.rs` — field_reassign_with_default
- `crates/datasynth-eval/Cargo.toml` — remove duplicate `rust_decimal_macros` from `[dependencies]`

**Step 1: Fix derivable Default**

In `document_flow_linker.rs`, replace:
```rust
#[allow(clippy::derivable_impls)]
impl Default for DocumentFlowLinker {
    fn default() -> Self { Self::new() }
}
```
With `#[derive(Default)]` on the struct. If `new()` has non-trivial init, keep it but call `Self::default()` internally.

**Step 2: Fix field_reassign_with_default (3 files)**

Replace `let mut x = Self::default(); x.field = val;` pattern with `Self { field, ..Self::default() }`.

**Step 3: Remove duplicate dependency**

In `crates/datasynth-eval/Cargo.toml`, remove `rust_decimal_macros` from `[dependencies]` (keep in `[dev-dependencies]`).

**Step 4: Run fmt + clippy + tests**

```bash
cargo fmt --all
cargo clippy --workspace --exclude datasynth-ui
cargo test --workspace --exclude datasynth-ui
```
Expected: 0 clippy warnings (except protoc), all tests pass

**Step 5: Commit**

```bash
git add -A
git commit -m "refactor: fix clippy items — derivable Default, field_reassign_with_default, duplicate dep"
```

---

## Phase 4: Verification (1 task)

### Task 12: Final verification pass

**Complexity: S**

**Step 1: Run fmt + clippy**

```bash
cargo fmt --all
cargo clippy --workspace --exclude datasynth-ui
```
Expected: 0 warnings (except protoc)

**Step 2: Run full test suite**

```bash
cargo test --workspace --exclude datasynth-ui
```
Expected: All tests pass

**Step 3: Run demo generation**

```bash
cargo run -p datasynth-cli -- generate --demo --output /tmp/r6-test
```
Expected: No panics, all files generated correctly

---

## Critical Files

| File | Role |
|------|------|
| `crates/datasynth-core/src/accounts.rs` | GL account constants — source of truth |
| `crates/datasynth-generators/src/period_close/year_end.rs` | Year-end closing entries — GL collision fix |
| `crates/datasynth-generators/src/period_close/depreciation.rs` | Depreciation expense — GL fix |
| `crates/datasynth-generators/src/subledger/ar_generator.rs` | AR credit memo — tax account fix |
| `crates/datasynth-generators/src/subledger/ap_generator.rs` | AP GL strings — constants fix |
| `crates/datasynth-generators/src/sales_quote_generator.rs` | Currency + pool wiring |
| `crates/datasynth-generators/src/bank_reconciliation_generator.rs` | Employee pool wiring |
| `crates/datasynth-generators/src/manufacturing/cycle_count_generator.rs` | Employee pool wiring |
| `crates/datasynth-runtime/src/enhanced_orchestrator.rs` | Caller updates for pools |
| ~35 generator files | seeded_rng() migration |
| ~20 generator files | tracing addition |

## Deferred Findings (for future rounds)

- **ACDOCA export wiring** — complete implementation exists in SapExporter but never called from pipeline
- **ControlExporter CSV wiring** — 8 CSV files exist in ControlExporter but never called
- **ACDOCA/BsegEntry serde str annotations** — Decimal fields missing str serialization
- **Config validation gaps** — rate fields in tax/treasury/HR/manufacturing lack bounds checking
- **Dead config sections** — 7 config sections defined but never consumed at runtime
- **Industry preset enrichment** — 8+ sections use Default for all 5 industries
- **String enum config validation** — pay_frequency, pool_type, effectiveness_method etc.
- **co_occurrence/temporal_cluster anomaly integration** — built but not wired into injection loop
- **audit/test_helpers.rs compiled into production** — should be cfg(test) gated
- **SalesQuote SO-{:06} linking** — needs post-generation linking to real O2C sales orders

## Execution Order

```
Phase 1: Tasks 1-4 (GL account fixes) — parallelizable, no interdependencies
Phase 2: Tasks 5-7 (cross-reference pools) — parallelizable, but each touches orchestrator
Phase 3: Tasks 8-11 (consistency) — 8 and 9 parallelizable; 10 and 11 parallelizable
Phase 4: Task 12 (verification) — final
```
