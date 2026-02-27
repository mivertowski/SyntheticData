# Codebase Quality, Coherence & Consolidation Fixes

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix all 30 issues identified in the comprehensive audit — account numbering, CLI output, intercompany wiring, data coherence, constructor consistency, country pack APIs, code quality, and consolidation.

**Architecture:** Bottom-up approach — fix foundational account numbering first (everything depends on it), then wire disconnected modules into the orchestrator, then standardize patterns, then consolidate duplicated code.

**Tech Stack:** Rust, `datasynth-core` accounts.rs, `datasynth-generators`, `datasynth-runtime` enhanced_orchestrator.rs, `datasynth-cli`, `datasynth-output`

---

## Phase 1: Account Numbering Unification (CRITICAL)

The codebase has three incompatible account numbering schemes:
- `accounts.rs` uses 4-digit codes: `"1100"`, `"2000"`, `"4000"`
- `DocumentFlowJeGenerator` uses 6-digit: `"120000"`, `"210000"`, `"400000"`
- `CoaGenerator` auto-generates starting at `100000` incrementing by 10

This means JEs from document flows reference accounts that don't exist in the CoA, and subledger generators (using 4-digit from accounts.rs) don't match document-flow JEs.

### Task 1: Migrate DocumentFlowJeConfig to use accounts.rs constants

**Files:**
- Modify: `crates/datasynth-generators/src/document_flow/document_flow_je_generator.rs:28-57`
- Test: `crates/datasynth-generators/tests/document_flow_je_accounts.rs` (create)

**Step 1: Write the failing test**

Create `crates/datasynth-generators/tests/document_flow_je_accounts.rs`:

```rust
use datasynth_core::accounts::{
    cash_accounts, control_accounts, expense_accounts, revenue_accounts,
};

/// Verify that DocumentFlowJeConfig defaults use centralized account constants.
#[test]
fn test_document_flow_je_config_uses_central_accounts() {
    let config = datasynth_generators::document_flow::DocumentFlowJeConfig::default();

    assert_eq!(config.ar_account, control_accounts::AR_CONTROL);
    assert_eq!(config.ap_account, control_accounts::AP_CONTROL);
    assert_eq!(config.inventory_account, control_accounts::INVENTORY);
    assert_eq!(config.gr_ir_clearing_account, control_accounts::GR_IR_CLEARING);
    assert_eq!(config.cash_account, cash_accounts::OPERATING_CASH);
    assert_eq!(config.revenue_account, revenue_accounts::PRODUCT_REVENUE);
    assert_eq!(config.cogs_account, expense_accounts::COGS);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p datasynth-generators test_document_flow_je_config_uses_central_accounts`
Expected: FAIL (currently defaults are 6-digit strings like "120000")

**Step 3: Update DocumentFlowJeConfig::default() to use accounts.rs constants**

In `crates/datasynth-generators/src/document_flow/document_flow_je_generator.rs`, change lines 45-57:

```rust
use datasynth_core::accounts::{
    cash_accounts, control_accounts, expense_accounts, revenue_accounts,
};

impl Default for DocumentFlowJeConfig {
    fn default() -> Self {
        Self {
            inventory_account: control_accounts::INVENTORY.to_string(),
            gr_ir_clearing_account: control_accounts::GR_IR_CLEARING.to_string(),
            ap_account: control_accounts::AP_CONTROL.to_string(),
            cash_account: cash_accounts::OPERATING_CASH.to_string(),
            ar_account: control_accounts::AR_CONTROL.to_string(),
            revenue_account: revenue_accounts::PRODUCT_REVENUE.to_string(),
            cogs_account: expense_accounts::COGS.to_string(),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p datasynth-generators test_document_flow_je_config_uses_central_accounts`
Expected: PASS

**Step 5: Run full test suite to check for regressions**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: All existing tests pass (some JE balance tests may need adjustment if they relied on 6-digit codes)

**Step 6: Commit**

```bash
git add crates/datasynth-generators/src/document_flow/document_flow_je_generator.rs crates/datasynth-generators/tests/document_flow_je_accounts.rs
git commit -m "fix: unify document flow JE accounts with centralized accounts.rs constants"
```

### Task 2: Align CoaGenerator to emit well-known account numbers

The CoA generator starts at 100000 and increments by 10, producing arbitrary account numbers that never match the canonical constants in accounts.rs. We need the CoA to include the canonical accounts alongside its generated ones.

**Files:**
- Modify: `crates/datasynth-generators/src/coa_generator.rs:53-86` (generate_asset_accounts and similar)
- Test: `crates/datasynth-generators/tests/coa_canonical_accounts.rs` (create)

**Step 1: Write the failing test**

Create `crates/datasynth-generators/tests/coa_canonical_accounts.rs`:

```rust
use datasynth_core::accounts::{
    cash_accounts, control_accounts, equity_accounts, expense_accounts,
    revenue_accounts, suspense_accounts,
};
use datasynth_core::models::{ChartOfAccounts, Complexity};

/// Verify the CoA contains all canonical accounts from accounts.rs.
#[test]
fn test_coa_contains_canonical_accounts() {
    let mut gen = datasynth_generators::CoaGenerator::new(42, Complexity::Small);
    let coa = gen.generate();

    let canonical = vec![
        control_accounts::AR_CONTROL,
        control_accounts::AP_CONTROL,
        control_accounts::INVENTORY,
        control_accounts::FIXED_ASSETS,
        control_accounts::GR_IR_CLEARING,
        cash_accounts::OPERATING_CASH,
        cash_accounts::BANK_ACCOUNT,
        revenue_accounts::PRODUCT_REVENUE,
        revenue_accounts::SERVICE_REVENUE,
        expense_accounts::COGS,
        expense_accounts::DEPRECIATION,
        expense_accounts::SALARIES_WAGES,
        equity_accounts::RETAINED_EARNINGS,
        equity_accounts::COMMON_STOCK,
        suspense_accounts::GENERAL_SUSPENSE,
    ];

    for account_num in &canonical {
        assert!(
            coa.accounts().iter().any(|a| a.account_number == *account_num),
            "CoA missing canonical account: {}",
            account_num
        );
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p datasynth-generators test_coa_contains_canonical_accounts`
Expected: FAIL — CoA has auto-generated 6-digit numbers, not the 4-digit canonical ones

**Step 3: Add canonical account seeding to CoaGenerator**

In `crates/datasynth-generators/src/coa_generator.rs`, add a `seed_canonical_accounts` method that inserts all well-known accounts from `accounts.rs` before generating additional accounts. Call it at the start of `generate()`:

```rust
use datasynth_core::accounts::{
    cash_accounts, control_accounts, equity_accounts, expense_accounts,
    liability_accounts, revenue_accounts, suspense_accounts, tax_accounts,
};

impl CoaGenerator {
    /// Seed the CoA with all canonical accounts from accounts.rs.
    /// These are the well-known accounts that all generators reference.
    fn seed_canonical_accounts(&self, coa: &mut ChartOfAccounts) {
        use datasynth_core::models::{AccountType, AccountSubType, GLAccount};

        let canonical = vec![
            // Assets
            (cash_accounts::OPERATING_CASH, "Operating Cash", AccountType::Asset, AccountSubType::Cash),
            (cash_accounts::BANK_ACCOUNT, "Bank Account", AccountType::Asset, AccountSubType::Cash),
            (cash_accounts::PETTY_CASH, "Petty Cash", AccountType::Asset, AccountSubType::Cash),
            (cash_accounts::WIRE_CLEARING, "Wire Transfer Clearing", AccountType::Asset, AccountSubType::Cash),
            (control_accounts::AR_CONTROL, "Accounts Receivable", AccountType::Asset, AccountSubType::AccountsReceivable),
            (control_accounts::INVENTORY, "Inventory", AccountType::Asset, AccountSubType::Inventory),
            (control_accounts::FIXED_ASSETS, "Fixed Assets", AccountType::Asset, AccountSubType::FixedAssets),
            (control_accounts::ACCUMULATED_DEPRECIATION, "Accumulated Depreciation", AccountType::Asset, AccountSubType::AccumulatedDepreciation),
            (control_accounts::IC_AR_CLEARING, "IC AR Clearing", AccountType::Asset, AccountSubType::AccountsReceivable),
            (tax_accounts::INPUT_VAT, "Input VAT", AccountType::Asset, AccountSubType::OtherAssets),
            (tax_accounts::DEFERRED_TAX_ASSET, "Deferred Tax Asset", AccountType::Asset, AccountSubType::OtherAssets),
            // Liabilities
            (control_accounts::AP_CONTROL, "Accounts Payable", AccountType::Liability, AccountSubType::AccountsPayable),
            (control_accounts::IC_AP_CLEARING, "IC AP Clearing", AccountType::Liability, AccountSubType::AccountsPayable),
            (tax_accounts::SALES_TAX_PAYABLE, "Sales Tax Payable", AccountType::Liability, AccountSubType::TaxPayable),
            (tax_accounts::VAT_PAYABLE, "VAT Payable", AccountType::Liability, AccountSubType::TaxPayable),
            (liability_accounts::ACCRUED_EXPENSES, "Accrued Expenses", AccountType::Liability, AccountSubType::AccruedLiabilities),
            (liability_accounts::ACCRUED_SALARIES, "Accrued Salaries", AccountType::Liability, AccountSubType::AccruedLiabilities),
            (liability_accounts::UNEARNED_REVENUE, "Unearned Revenue", AccountType::Liability, AccountSubType::UnearnedRevenue),
            (control_accounts::GR_IR_CLEARING, "GR/IR Clearing", AccountType::Liability, AccountSubType::AccountsPayable),
            (liability_accounts::IC_PAYABLE, "IC Payable", AccountType::Liability, AccountSubType::AccountsPayable),
            // Equity
            (equity_accounts::COMMON_STOCK, "Common Stock", AccountType::Equity, AccountSubType::CommonStock),
            (equity_accounts::RETAINED_EARNINGS, "Retained Earnings", AccountType::Equity, AccountSubType::RetainedEarnings),
            (equity_accounts::CURRENT_YEAR_EARNINGS, "Current Year Earnings", AccountType::Equity, AccountSubType::RetainedEarnings),
            (equity_accounts::CTA, "Currency Translation Adjustment", AccountType::Equity, AccountSubType::RetainedEarnings),
            // Revenue
            (revenue_accounts::PRODUCT_REVENUE, "Product Revenue", AccountType::Revenue, AccountSubType::ProductRevenue),
            (revenue_accounts::SERVICE_REVENUE, "Service Revenue", AccountType::Revenue, AccountSubType::ServiceRevenue),
            (revenue_accounts::IC_REVENUE, "IC Revenue", AccountType::Revenue, AccountSubType::ProductRevenue),
            // Expenses
            (expense_accounts::COGS, "Cost of Goods Sold", AccountType::Expense, AccountSubType::CostOfGoodsSold),
            (expense_accounts::DEPRECIATION, "Depreciation Expense", AccountType::Expense, AccountSubType::Depreciation),
            (expense_accounts::SALARIES_WAGES, "Salaries & Wages", AccountType::Expense, AccountSubType::SalariesWages),
            (expense_accounts::BENEFITS, "Benefits", AccountType::Expense, AccountSubType::Benefits),
            (expense_accounts::INTEREST_EXPENSE, "Interest Expense", AccountType::Expense, AccountSubType::InterestExpense),
            (expense_accounts::FX_GAIN_LOSS, "FX Gain/Loss", AccountType::Expense, AccountSubType::OtherExpense),
            // Suspense
            (suspense_accounts::GENERAL_SUSPENSE, "General Suspense", AccountType::Expense, AccountSubType::SuspenseClearing),
            (suspense_accounts::PAYROLL_CLEARING, "Payroll Clearing", AccountType::Expense, AccountSubType::SuspenseClearing),
        ];

        for (num, name, acct_type, sub_type) in canonical {
            let account = GLAccount::new(
                num.to_string(),
                name.to_string(),
                acct_type,
                sub_type,
            );
            coa.add_account(account);
        }
    }
}
```

Then in the `generate()` method, call `self.seed_canonical_accounts(&mut coa)` before generating additional accounts. Adjust `generate_asset_accounts` etc. to start their numbering *after* the canonical range (e.g., start at 100000 as before — the canonical accounts use 4-digit codes in a different range, so no collision).

**Step 4: Run test to verify it passes**

Run: `cargo test -p datasynth-generators test_coa_contains_canonical_accounts`
Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/datasynth-generators/src/coa_generator.rs crates/datasynth-generators/tests/coa_canonical_accounts.rs
git commit -m "fix: seed CoA with canonical accounts from accounts.rs for cross-generator consistency"
```

### Task 3: Align trial balance builder to use JE account numbers

The `build_trial_balance_from_flows()` method at `enhanced_orchestrator.rs:2236-2322` computes trial balance from document-flow aggregates with hardcoded 4-digit codes, but doesn't actually aggregate from journal entries. It should derive from actual JE data.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (build_trial_balance_from_flows method)
- Test: existing orchestrator tests should still pass

**Step 1: Read the current implementation**

Read `crates/datasynth-runtime/src/enhanced_orchestrator.rs` lines 2236-2322 to understand the current approach.

**Step 2: Replace with JE-derived trial balance**

Change `build_trial_balance_from_flows` to aggregate balances from actual `JournalEntry` data by summing debit/credit amounts per account number. This ensures the trial balance matches the JEs exactly.

```rust
fn build_trial_balance_from_entries(
    entries: &[JournalEntry],
    coa: &ChartOfAccounts,
    company_code: &str,
    fiscal_year: u16,
    fiscal_period: u8,
) -> Vec<TrialBalanceLine> {
    use std::collections::HashMap;
    use datasynth_core::accounts::AccountCategory;

    let mut account_balances: HashMap<String, (Decimal, Decimal)> = HashMap::new();

    for entry in entries {
        if entry.header.company_code == company_code {
            for line in &entry.lines {
                let (debit, credit) = account_balances
                    .entry(line.account_number.clone())
                    .or_insert((Decimal::ZERO, Decimal::ZERO));
                *debit += line.debit_amount;
                *credit += line.credit_amount;
            }
        }
    }

    account_balances
        .into_iter()
        .map(|(account_number, (debit, credit))| {
            let account_name = coa
                .accounts()
                .iter()
                .find(|a| a.account_number == account_number)
                .map(|a| a.name.clone())
                .unwrap_or_else(|| format!("Account {}", account_number));

            TrialBalanceLine {
                account_number,
                account_name,
                debit_balance: debit,
                credit_balance: credit,
                net_balance: debit - credit,
                company_code: company_code.to_string(),
                fiscal_year,
                fiscal_period,
            }
        })
        .collect()
}
```

Update the call sites to pass `&entries` alongside document flows.

**Step 3: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "fix: derive trial balance from actual JE data instead of hardcoded aggregates"
```

---

## Phase 2: CLI Output Pipeline (CRITICAL)

The CLI writes only `sample_entries.json` (first 1000 JEs), banking JSON, anomaly labels, and lineage. All other generated data (master data, document flows, subledger, standards, HR, manufacturing, etc.) is discarded.

### Task 4: Wire datasynth-output exporters into CLI

**Files:**
- Modify: `crates/datasynth-cli/src/main.rs:556-684`
- Modify: `crates/datasynth-cli/Cargo.toml` (add datasynth-output dependency if missing)

**Step 1: Add datasynth-output dependency to CLI**

Check `crates/datasynth-cli/Cargo.toml` for `datasynth-output` dependency. Add if missing:
```toml
datasynth-output = { path = "../datasynth-output" }
```

**Step 2: Create output_writer module in CLI**

Create `crates/datasynth-cli/src/output_writer.rs` that takes `EnhancedGenerationResult` and output dir, and calls the appropriate `datasynth-output` sinks (CsvSink, JsonLinesSink) for each data category:

```rust
use std::path::Path;
use datasynth_output::csv_sink::CsvSink;
use datasynth_runtime::enhanced_orchestrator::EnhancedGenerationResult;

pub fn write_all_output(result: &EnhancedGenerationResult, output_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(output_dir)?;
    let sink = CsvSink::new(output_dir);

    // Journal Entries
    sink.write_journal_entries(&result.journal_entries)?;

    // Master Data
    sink.write_vendors(&result.master_data.vendors)?;
    sink.write_customers(&result.master_data.customers)?;
    sink.write_materials(&result.master_data.materials)?;
    sink.write_assets(&result.master_data.assets)?;
    sink.write_employees(&result.master_data.employees)?;

    // Document Flows
    sink.write_purchase_orders(&result.document_flows.purchase_orders)?;
    sink.write_goods_receipts(&result.document_flows.goods_receipts)?;
    sink.write_vendor_invoices(&result.document_flows.vendor_invoices)?;
    sink.write_payments(&result.document_flows.payments)?;
    sink.write_sales_orders(&result.document_flows.sales_orders)?;
    sink.write_deliveries(&result.document_flows.deliveries)?;
    sink.write_customer_invoices(&result.document_flows.customer_invoices)?;
    sink.write_customer_receipts(&result.document_flows.customer_receipts)?;

    // ... continue for all snapshot categories

    Ok(())
}
```

The exact method names depend on what CsvSink exposes. If CsvSink doesn't have per-table methods, use `csv::Writer::from_path` directly for each `Vec<T: Serialize>`.

**Step 3: Replace the truncated output in main.rs**

Replace the sample_entries.json + banking JSON block (lines 556-605) with a call to the new `output_writer::write_all_output()`, keeping the anomaly labels and lineage writers as-is.

**Step 4: Test end-to-end**

Run: `cargo run -p datasynth-cli -- generate --demo --output /tmp/datasynth-test-output`
Verify: Check that `/tmp/datasynth-test-output/` contains CSV files for vendors, customers, purchase_orders, etc.

**Step 5: Commit**

```bash
git add crates/datasynth-cli/
git commit -m "feat: wire datasynth-output exporters into CLI to write all generated data"
```

---

## Phase 3: Intercompany Module Wiring (CRITICAL)

The intercompany module (`crates/datasynth-generators/src/intercompany/`) is fully implemented but never called from the orchestrator.

### Task 5: Wire intercompany generation into orchestrator

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (add Phase 3b: Intercompany)
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (EnhancedGenerationResult, add ic fields)

**Step 1: Add IntercompanySnapshot to result**

In `enhanced_orchestrator.rs`, add after the existing snapshot structs:

```rust
/// Intercompany data snapshot.
#[derive(Debug, Clone, Default)]
pub struct IntercompanySnapshot {
    pub ic_transactions: Vec<datasynth_core::models::IntercompanyTransaction>,
    pub matched_pairs: Vec<datasynth_core::models::ICMatchedPair>,
    pub eliminations: Vec<datasynth_core::models::EliminationEntry>,
}
```

Add `pub intercompany: IntercompanySnapshot` to `EnhancedGenerationResult`.

**Step 2: Add phase_intercompany method**

Create a new phase method that:
1. Builds `OwnershipStructure` from company configs
2. Creates `ICGenerator` with config and ownership
3. Generates IC transactions
4. Runs `ICMatchingEngine` to match pairs
5. Runs `EliminationGenerator` for consolidation entries

**Step 3: Call it in generate() after Phase 3 (document flows)**

Insert between Phase 3 and Phase 4 in the generate() method.

**Step 4: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat: wire intercompany module into orchestrator with IC transactions, matching, eliminations"
```

---

## Phase 4: Financial Statement Coherence (CRITICAL)

### Task 6: Derive financial statements from actual JE trial balances

Currently `phase_financial_reporting` builds statements from document-flow aggregates. It should use the JE-derived trial balance from Task 3.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (phase_financial_reporting)

**Step 1: Read current implementation**

Read the `phase_financial_reporting` method to understand how it currently aggregates.

**Step 2: Refactor to accept journal entries**

Change the method signature to also accept `&[JournalEntry]` and `&ChartOfAccounts`. Use the JE-derived trial balance to build the income statement, balance sheet, and cash flow statement.

**Step 3: Update generate() call site**

Pass `&entries` and `&coa` to `phase_financial_reporting`.

**Step 4: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "fix: derive financial statements from actual JE trial balances for coherence"
```

---

## Phase 5: Accounting Standards Snapshot Fix (CRITICAL)

### Task 7: Persist AccountingStandards generated data

Phases 12/13 (diffusion/causal) and Phase 17 (accounting standards) generate data but the `AccountingStandardsSnapshot` discards it. Verify the snapshot struct stores the generated data and is included in the output.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (AccountingStandardsSnapshot, phase_accounting_standards)

**Step 1: Read AccountingStandardsSnapshot and phase_accounting_standards**

Identify what data is generated and what is stored/discarded.

**Step 2: Ensure all generated data is stored in the snapshot**

If contracts, leases, fair-value measurements, or impairment tests are generated but not stored, add them to the snapshot struct.

**Step 3: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "fix: persist all accounting standards generated data in snapshot"
```

---

## Phase 6: Constructor & API Pattern Standardization (HIGH)

### Task 8: Standardize constructor ordering to (config, seed)

Three patterns exist: `(seed, config)`, `(config, seed)`, `(config, rng)`. Standardize to `(config, seed)` which is the most common.

**Files:**
- Grep for all `::new(seed` patterns to find generators using (seed, config) ordering
- Fix each to use (config, seed)

**Step 1: Find all instances**

Run grep to find generators with `::new(seed` or `::new(self.seed` patterns.

**Step 2: Fix each generator**

For each generator with `(seed, config)` ordering, swap to `(config, seed)`. Update all call sites in the orchestrator.

**Step 3: Run full test suite**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: All tests pass

**Step 4: Commit**

```bash
git add -A
git commit -m "refactor: standardize all generator constructors to (config, seed) ordering"
```

### Task 9: Unify country pack API to builder pattern

Three different patterns exist for passing country packs:
1. Builder: `builder.country_pack(pack).build()` (BankingOrchestrator)
2. Setter: `gen.set_country_pack(pack)` (PayrollGenerator)
3. Per-call: passed as argument to each method

Standardize to the setter pattern since most generators already use it and it's the simplest to add.

**Files:**
- Find all generators that accept country packs
- Ensure they all use `set_country_pack(&mut self, pack: CountryPack)`
- Update orchestrator call sites

**Step 1: Find all country pack patterns**

Grep for `country_pack` across the workspace.

**Step 2: Standardize to setter pattern**

For any generators using other patterns, add `set_country_pack` method. Keep builder pattern in BankingOrchestrator (it delegates to the setter internally).

**Step 3: Run tests**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: PASS

**Step 4: Commit**

```bash
git add -A
git commit -m "refactor: standardize country pack API to setter pattern across all generators"
```

---

## Phase 7: Generator Wiring Gaps (HIGH)

### Task 10: Rotate created_by across employees

Currently `enhanced_orchestrator.rs:3173-3178` always uses `employees.first()` for `created_by`. Fix to rotate through available employees.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs`

**Step 1: Find all occurrences**

Grep for `.first()` and `created_by` patterns in the orchestrator.

**Step 2: Replace with round-robin selection**

Replace each `employees.first()` with an index that increments: `employees[i % employees.len()]`.

**Step 3: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "fix: rotate created_by across employees instead of always using first"
```

### Task 11: Wire FA/Inventory subledger generators

The FA and Inventory subledger generators exist but are never called from the orchestrator. Wire them in during the subledger phase.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (phase that generates subledger data)

**Step 1: Read the FA/Inventory generator APIs**

Read `crates/datasynth-generators/src/subledger/fa_generator.rs` and `inventory_generator.rs` to understand their constructor and generate signatures.

**Step 2: Wire into orchestrator**

Add calls to FA and Inventory subledger generators in the appropriate phase, using fixed assets and materials from master data.

**Step 3: Add generated data to SubledgerSnapshot**

Ensure `SubledgerSnapshot` has fields for FA and inventory subledger records.

**Step 4: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat: wire FA and inventory subledger generators into orchestrator"
```

### Task 12: Generate JEs from payroll and manufacturing

Payroll runs and manufacturing production orders should produce journal entries. Currently they produce domain data but no JEs.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (phase_hr_data, phase_manufacturing)

**Step 1: Add payroll JE generation**

After generating payroll runs in `phase_hr_data`, create JEs:
- DR Salaries & Wages (6100), CR Payroll Clearing (9100) for gross pay
- DR Payroll Clearing (9100), CR Cash (1000) for net pay

**Step 2: Add manufacturing JE generation**

After generating production orders in `phase_manufacturing`, create JEs:
- DR WIP/Manufacturing Overhead (5300), CR Raw Materials (5100)
- DR Finished Goods/Inventory (1200), CR WIP (5300) on completion

**Step 3: Append these JEs to the entries vector**

In `generate()`, collect the JEs from HR and manufacturing phases and append to `entries`.

**Step 4: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat: generate journal entries from payroll and manufacturing phases"
```

### Task 13: Add master-data generators country pack support

Vendor, Customer, and Material generators don't accept country packs. Add `set_country_pack` to each.

**Files:**
- Modify: `crates/datasynth-generators/src/master_data/vendor_generator.rs`
- Modify: `crates/datasynth-generators/src/master_data/customer_generator.rs`
- Modify: `crates/datasynth-generators/src/master_data/material_generator.rs`

**Step 1: Add country_pack field and setter to each generator**

For each generator, add:
```rust
country_pack: Option<datasynth_core::CountryPack>,

pub fn set_country_pack(&mut self, pack: datasynth_core::CountryPack) {
    self.country_pack = Some(pack);
}
```

**Step 2: Use country pack data in generation**

Where these generators produce names, addresses, or locale-specific data, check if `self.country_pack` is set and use its data.

**Step 3: Wire in orchestrator**

In the master data phase, call `gen.set_country_pack(pack.clone())` if a country pack is available.

**Step 4: Run tests**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/datasynth-generators/src/master_data/ crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat: add country pack support to vendor, customer, and material generators"
```

---

## Phase 8: OCPM & Banking Integration (HIGH)

### Task 14: Persist OCPM event log to output

OCPM events are generated but never written to output files.

**Files:**
- Modify: `crates/datasynth-cli/src/output_writer.rs` (or main.rs)
- The OCPM data is in `result.ocpm`

**Step 1: Add OCPM output to the CLI writer**

Write `result.ocpm.events` as `event_log.json` (OCEL 2.0 format) and `result.ocpm.objects` as `objects.json`.

**Step 2: Test**

Run: `cargo run -p datasynth-cli -- generate --demo --output /tmp/test-output`
Verify: `/tmp/test-output/event_log.json` exists and contains events.

**Step 3: Commit**

```bash
git add crates/datasynth-cli/
git commit -m "feat: persist OCPM event log to output directory"
```

### Task 15: Share master data with banking module

Banking generates its own customers independently. It should use core master data as a base.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (phase_banking_data)

**Step 1: Read phase_banking_data**

Understand how BankingOrchestratorBuilder is currently configured.

**Step 2: Pass core customer data to banking**

If `BankingOrchestratorBuilder` supports setting external customers, use it. Otherwise, after banking generation, cross-reference banking customers with core customers by matching names/IDs.

**Step 3: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat: share core master data with banking module for coherent customer base"
```

---

## Phase 9: Code Quality Fixes (MEDIUM)

### Task 16: Fix InjectorStats pub with all-private fields

**Files:**
- Modify: `crates/datasynth-generators/src/anomaly/injector.rs:178-190`

**Step 1: Make InjectorStats fields pub or add accessor methods**

Either make the fields `pub` (if they're useful for external consumers) or add getter methods.

**Step 2: Run tests**

Run: `cargo test -p datasynth-generators`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/datasynth-generators/src/anomaly/injector.rs
git commit -m "fix: make InjectorStats fields accessible"
```

### Task 17: Remove CountryPack clone in payroll hot path

**Files:**
- Modify: `crates/datasynth-generators/src/hr/payroll_generator.rs:114`

**Step 1: Change clone to reference**

Change the country pack from being cloned to being borrowed where possible. If the generator stores `Option<CountryPack>`, use `as_ref()` instead of `clone()`.

**Step 2: Run tests**

Run: `cargo test -p datasynth-generators`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/datasynth-generators/src/hr/payroll_generator.rs
git commit -m "perf: avoid unnecessary CountryPack clone in payroll generator"
```

### Task 18: Remove dead code

**Files:**
- Modify: `crates/datasynth-generators/src/subledger/inventory_generator.rs:47-48` (dead `position_counter`)
- Any other dead code identified by `cargo clippy`

**Step 1: Run clippy**

Run: `cargo clippy --workspace 2>&1 | grep "warning\|dead_code\|unused"`

**Step 2: Fix each instance**

Remove unused fields, imports, and dead code.

**Step 3: Run tests**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: PASS

**Step 4: Commit**

```bash
git add -A
git commit -m "chore: remove dead code and unused fields"
```

### Task 19: Fix gate_result always None

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs:1333`

**Step 1: Read the eval gate code**

Understand `datasynth_eval::gates::GateResult` and when it should be computed.

**Step 2: Wire quality gate evaluation**

After generation completes but before returning the result, run the quality gate evaluation if configured:

```rust
let gate_result = if self.config.global.quality_gate_enabled.unwrap_or(false) {
    // Run quality gate evaluation
    Some(datasynth_eval::gates::evaluate(&entries, &coa))
} else {
    None
};
```

**Step 3: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat: wire quality gate evaluation into orchestrator result"
```

### Task 20: Replace magic seed offsets in banking

**Files:**
- Modify: `crates/datasynth-banking/src/` (wherever undocumented seed arithmetic exists)

**Step 1: Find magic seed offsets**

Grep for patterns like `seed + ` or `seed.wrapping_add` in banking crate.

**Step 2: Replace with named constants**

Define constants like `const CUSTOMER_SEED_OFFSET: u64 = 1000;` etc. and document the purpose.

**Step 3: Run tests**

Run: `cargo test -p datasynth-banking`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/datasynth-banking/
git commit -m "refactor: replace magic seed offsets with named constants in banking module"
```

---

## Phase 10: Code Consolidation (MEDIUM)

### Task 21: Extract shared weighted_select utility

Weighted distribution selection is duplicated 17 times across generators. Extract into a shared utility.

**Files:**
- Create: `crates/datasynth-core/src/utils.rs`
- Modify: `crates/datasynth-core/src/lib.rs` (add `pub mod utils;`)

**Step 1: Create shared utility**

```rust
//! Shared generator utilities.

use rand::Rng;

/// Select from weighted options. Weights don't need to sum to 1.0.
pub fn weighted_select<'a, T, R: Rng>(rng: &mut R, options: &'a [(T, f64)]) -> &'a T {
    let total: f64 = options.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen::<f64>() * total;
    for (item, weight) in options {
        roll -= weight;
        if roll <= 0.0 {
            return item;
        }
    }
    &options.last().unwrap().0
}

/// Sample a Decimal in a range using the RNG.
pub fn sample_decimal_range<R: Rng>(
    rng: &mut R,
    min: rust_decimal::Decimal,
    max: rust_decimal::Decimal,
) -> rust_decimal::Decimal {
    use rust_decimal::prelude::ToPrimitive;
    let min_f = min.to_f64().unwrap_or(0.0);
    let max_f = max.to_f64().unwrap_or(min_f + 1.0);
    let val = rng.gen_range(min_f..=max_f);
    rust_decimal::Decimal::from_f64_retain(val).unwrap_or(min)
}
```

**Step 2: Write test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_weighted_select() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let options = vec![("a", 0.9), ("b", 0.1)];
        let mut a_count = 0;
        for _ in 0..100 {
            if *weighted_select(&mut rng, &options) == "a" {
                a_count += 1;
            }
        }
        assert!(a_count > 70, "Expected ~90% 'a', got {}", a_count);
    }
}
```

**Step 3: Run tests**

Run: `cargo test -p datasynth-core test_weighted_select`
Expected: PASS

**Step 4: Commit (utility only, migration in separate commits)**

```bash
git add crates/datasynth-core/src/utils.rs crates/datasynth-core/src/lib.rs
git commit -m "feat: add shared weighted_select and sample_decimal_range utilities"
```

### Task 22: Migrate generators to use shared weighted_select

**Files:**
- Modify: Various generators that have inline weighted_select implementations

**Step 1: Find all duplicated implementations**

Grep for common patterns like `gen_range.*total` or `roll -= weight` or similar weighted selection code.

**Step 2: Replace each with `datasynth_core::utils::weighted_select`**

For each duplicate, replace the inline implementation with a call to the shared utility.

**Step 3: Run full test suite**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: All tests pass

**Step 4: Commit**

```bash
git add -A
git commit -m "refactor: migrate generators to shared weighted_select utility (DRY)"
```

### Task 23: Extract shared RNG initialization pattern

14+ generators duplicate the `ChaCha8Rng::seed_from_u64(seed)` pattern with slight variations. While we can't eliminate all of it, we can standardize.

**Files:**
- Modify: `crates/datasynth-core/src/utils.rs` (add rng helper)

**Step 1: Add helper function**

```rust
/// Create a seeded RNG for a generator, with an optional discriminator for sub-generators.
pub fn seeded_rng(seed: u64, discriminator: u64) -> rand_chacha::ChaCha8Rng {
    use rand::SeedableRng;
    rand_chacha::ChaCha8Rng::seed_from_u64(seed.wrapping_add(discriminator))
}
```

**Step 2: Migrate generators incrementally**

Replace `ChaCha8Rng::seed_from_u64(seed + magic_number)` with `utils::seeded_rng(seed, NAMED_CONSTANT)`.

**Step 3: Run tests**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: PASS

**Step 4: Commit**

```bash
git add -A
git commit -m "refactor: extract seeded_rng utility for consistent RNG initialization"
```

### Task 24: Consolidate duplicated Decimal range sampling

24 occurrences of manual Decimal range sampling. Migrate to `utils::sample_decimal_range`.

**Files:**
- Modify: Various generators

**Step 1: Grep for Decimal range patterns**

Find `gen_range.*to_f64` or `Decimal::from_f64_retain` patterns.

**Step 2: Replace with shared utility**

Replace each instance with `datasynth_core::utils::sample_decimal_range(&mut self.rng, min, max)`.

**Step 3: Run tests**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: PASS

**Step 4: Commit**

```bash
git add -A
git commit -m "refactor: consolidate Decimal range sampling to shared utility"
```

---

## Phase 11: Remaining Wiring & Quality (MEDIUM-LOW)

### Task 25: Link sales quotes to O2C flow

Sales quotes should reference the same customers that appear in O2C chains.

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (phase_sales_kpi_budgets)

**Step 1: Pass customer data to sales quote generator**

In `phase_sales_kpi_budgets`, pass `self.master_data.customers` so generated quotes reference real customer IDs.

**Step 2: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat: link sales quotes to core customer master data"
```

### Task 26: Link sourcing projects to P2P vendors

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (phase_sourcing_data)

**Step 1: Pass vendor data to sourcing generator**

In `phase_sourcing_data`, pass `self.master_data.vendors` so generated sourcing projects reference real vendor IDs.

**Step 2: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat: link sourcing projects to core vendor master data"
```

### Task 27: Fix OCPM UUID factory reimplementation

**Files:**
- Modify: `crates/datasynth-ocpm/src/generator/mod.rs` or wherever OCPM generates UUIDs

**Step 1: Find OCPM UUID generation**

Grep for `Uuid::new_v4()` or custom UUID generation in `datasynth-ocpm`.

**Step 2: Replace with `DeterministicUuidFactory`**

Use `datasynth_core::uuid_factory::DeterministicUuidFactory` with `GeneratorType::Ocpm` discriminator for deterministic, collision-free UUIDs.

**Step 3: Run tests**

Run: `cargo test -p datasynth-ocpm`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/datasynth-ocpm/
git commit -m "refactor: use DeterministicUuidFactory in OCPM instead of Uuid::new_v4()"
```

### Task 28: Add missing statistics tracking

Several phases don't update `EnhancedGenerationStatistics`. Add tracking for:
- Intercompany transaction/elimination counts
- FA subledger record count
- Inventory subledger record count

**Files:**
- Modify: `crates/datasynth-runtime/src/enhanced_orchestrator.rs` (EnhancedGenerationStatistics, various phase methods)

**Step 1: Add fields to statistics struct**

```rust
#[serde(default)]
pub ic_transaction_count: usize,
#[serde(default)]
pub ic_elimination_count: usize,
#[serde(default)]
pub fa_subledger_count: usize,
#[serde(default)]
pub inventory_subledger_count: usize,
```

**Step 2: Update phase methods to populate stats**

In each relevant phase method, set the corresponding stat field.

**Step 3: Run tests**

Run: `cargo test -p datasynth-runtime`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/datasynth-runtime/src/enhanced_orchestrator.rs
git commit -m "feat: add missing statistics tracking for intercompany, FA, inventory"
```

### Task 29: Clean up re-export bloat in lib.rs files

**Files:**
- Modify: `crates/datasynth-generators/src/lib.rs`
- Modify: `crates/datasynth-core/src/lib.rs`

**Step 1: Audit glob re-exports**

Check for `pub use module::*` patterns that re-export too broadly.

**Step 2: Replace with explicit re-exports where practical**

For modules with name collisions or excessive re-exports, switch to explicit `pub use module::{Type1, Type2}`.

**Step 3: Run tests**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: PASS

**Step 4: Commit**

```bash
git add -A
git commit -m "refactor: clean up glob re-exports in lib.rs files"
```

### Task 30: Final cargo fmt + clippy + full test suite

**Files:** Entire workspace

**Step 1: Format**

Run: `cargo fmt --all`

**Step 2: Clippy**

Run: `cargo clippy --workspace 2>&1 | grep -v "protoc"`
Fix any remaining warnings.

**Step 3: Full test suite**

Run: `cargo test --workspace`
Expected: All tests pass.

**Step 4: Commit**

```bash
git add -A
git commit -m "chore: cargo fmt + clippy fixes for clean workspace"
```

---

## Summary

| Phase | Tasks | Severity | Description |
|-------|-------|----------|-------------|
| 1 | 1-3 | CRITICAL | Account numbering unification |
| 2 | 4 | CRITICAL | CLI output pipeline |
| 3 | 5 | CRITICAL | Intercompany module wiring |
| 4 | 6 | CRITICAL | Financial statement coherence |
| 5 | 7 | CRITICAL | Accounting standards snapshot |
| 6 | 8-9 | HIGH | Constructor & API standardization |
| 7 | 10-13 | HIGH | Generator wiring gaps |
| 8 | 14-15 | HIGH | OCPM & banking integration |
| 9 | 16-20 | MEDIUM | Code quality fixes |
| 10 | 21-24 | MEDIUM | Code consolidation (DRY) |
| 11 | 25-30 | MEDIUM-LOW | Remaining wiring & quality |

**Total: 30 tasks across 11 phases**

**Estimated commits: ~30 (one per task)**

**Test strategy: TDD where feasible (Tasks 1-2), regression testing for all others**
